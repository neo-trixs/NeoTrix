use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use url::Url;

use super::core::CapabilityVector;
use super::self_edit::MicroEdit;
use super::self_iterating::ReasoningBrain;
use super::memory::{ReasoningBank, ReasoningMemory};
use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};
use crate::neotrix::nt_memory_kb::KnowledgeBase;
use crate::neotrix::nt_world_model::TaskType;

/// 网络来源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WebSourceType {
    Wikipedia,
    ArXiv,
    GitHub,
    GenericUrl,
    KnowledgeBase,
}

impl WebSourceType {
    pub fn name(&self) -> &'static str {
        match self {
            WebSourceType::Wikipedia => "Wikipedia",
            WebSourceType::ArXiv => "ArXiv",
            WebSourceType::GitHub => "GitHub",
            WebSourceType::GenericUrl => "GenericUrl",
            WebSourceType::KnowledgeBase => "KnowledgeBase",
        }
    }

    pub fn detect(url_str: &str) -> Self {
        let lower = url_str.to_lowercase();
        if lower.contains("wikipedia.org") || lower.contains("wikidata.org") || lower.contains("wikimedia.org") {
            WebSourceType::Wikipedia
        } else if lower.contains("arxiv.org") || lower.contains("aclweb.org") || lower.contains("semanticscholar.org") {
            WebSourceType::ArXiv
        } else if lower.contains("github.com") {
            WebSourceType::GitHub
        } else if lower.contains("wiki") || lower.contains("knowledge") || lower.contains("encyclopedia") {
            WebSourceType::KnowledgeBase
        } else {
            WebSourceType::GenericUrl
        }
    }

    pub fn to_task_type(&self) -> TaskType {
        match self {
            WebSourceType::Wikipedia | WebSourceType::KnowledgeBase => TaskType::CodeAnalysis,
            WebSourceType::ArXiv => TaskType::CodeAnalysis,
            WebSourceType::GitHub => TaskType::CodeGeneration,
            WebSourceType::GenericUrl => TaskType::General,
        }
    }
}

/// 网页挖掘结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebMinedKnowledge {
    pub source_url: String,
    pub source_name: String,
    pub source_type: WebSourceType,
    pub title: String,
    pub summary: String,
    pub content_length: usize,
    pub extracted_insights: Vec<String>,
    pub edits: Vec<(String, f64)>,
    pub confidence: f64,
}

/// 统一网页知识挖掘器 — Wikipedia / arXiv / GitHub / 公开网址
pub struct WebKnowledgeMiner {
    pub work_dir: PathBuf,
    pub http_client: reqwest::blocking::Client,
    pub mined_history: Vec<WebMinedKnowledge>,
    pub nt_memory_kb: Option<KnowledgeBase>,
    processed: HashMap<String, bool>,
}

impl WebKnowledgeMiner {
    pub fn new(work_dir: PathBuf) -> Self {
        let client = reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(60))
            .user_agent("NeoTrix/1.0 (knowledge-miner; +https://github.com/neotrix)")
            .build()
            .unwrap_or_default();
        Self {
            work_dir,
            http_client: client,
            mined_history: Vec::new(),
            nt_memory_kb: None,
            processed: HashMap::new(),
        }
    }

    /// 挖掘单个 URL
    pub fn mine_url(&mut self, url_str: &str) -> NeoTrixResult<WebMinedKnowledge> {
        if self.processed.contains_key(url_str) {
            return Err(NeoTrixError::from("已处理过"));
        }
        self.processed.insert(url_str.to_string(), true);

        let source_type = WebSourceType::detect(url_str);
        match source_type {
            WebSourceType::Wikipedia => self.mine_wikipedia(url_str),
            WebSourceType::ArXiv => self.mine_arxiv(url_str),
            WebSourceType::GitHub => self.mine_github(url_str),
            WebSourceType::KnowledgeBase | WebSourceType::GenericUrl => self.mine_generic(url_str),
        }
    }

    pub fn attach_kb(&mut self, kb: KnowledgeBase) {
        self.nt_memory_kb = Some(kb);
    }

    /// 批量挖掘 + 自动持久化到 KnowledgeBase
    pub fn mine_all_persist(&mut self, urls: &[&str], brain: &mut ReasoningBrain, bank: &mut ReasoningBank) -> WebMineResult {
        let result = self.mine_all(urls, brain, bank);
        if let Some(ref kb) = self.nt_memory_kb {
            for mined in &self.mined_history {
                let source_name = mined.source_type.name();
                let _ = kb.persist_mined(
                    &mined.title, &mined.summary, &mined.source_url,
                    source_name, mined.confidence, &mined.edits, &mined.extracted_insights,
                );
            }
        }
        result
    }

    /// 批量挖掘 — 注入 brain + bank
    pub fn mine_all(&mut self, urls: &[&str], brain: &mut ReasoningBrain, bank: &mut ReasoningBank) -> WebMineResult {
        let mut total_edits = 0usize;
        let mut total_reward = 0.0f64;
        let mut success = 0usize;
        let details = Vec::new();

        for (_i, url) in urls.iter().enumerate() {
            match self.mine_url(url) {
                Ok(knowledge) => {
                    self.mined_history.push(knowledge.clone());
                    let edits_count = knowledge.edits.len();
                    total_edits += edits_count;

                    // 注册知识来源
                    brain.register_knowledge_source(&knowledge.source_name, Self::edits_to_vector(&knowledge.edits));

                    // 应用 MicroEdits
                    for (dim, delta) in &knowledge.edits {
                        if let Some(idx) = CapabilityVector::index_from_name(dim) {
                            let val = &mut brain.capability.arr_mut()[idx];
                            *val = (*val + delta).clamp(0.0, 1.0);
                        }
                    }

                    let reward = edits_count as f64 * 0.02;
                    total_reward += reward;
                    success += 1;

                    // 存储到 ReasoningBank
                    let task_type = knowledge.source_type.to_task_type();
                    let micro_edits: Vec<MicroEdit> = knowledge.edits.iter()
                        .map(|(d, v)| MicroEdit::AdjustDimension(d.clone(), *v))
                        .collect();
                    let mem = ReasoningMemory::new(
                        &format!("WebMiner:{} {}", knowledge.source_type.name(), knowledge.title),
                        task_type,
                        &micro_edits,
                        reward,
                    );
                    bank.store(mem);

//                    details.push(format!("[{}] ✅ {} — {} ({} edits, reward {:.2})",
//                        i+1, knowledge.source_type.name(), knowledge.title, edits_count, reward));
                }
                Err(_e) => {
//                    details.push(format!("[{}] ❌ {} — {}", i+1, url, e));
                }
            }
        }

        brain.capability.normalize();

        WebMineResult {
            total_urls: urls.len(),
            success_count: success,
            total_edits,
            total_reward,
            details,
        }
    }

    // ==================== 来源专用处理器 ====================

    /// Wikipedia API 挖掘
    fn mine_wikipedia(&self, url_str: &str) -> NeoTrixResult<WebMinedKnowledge> {
        let parsed = Url::parse(url_str)
            .map_err(|_| NeoTrixError::Network("无效 Wikipedia URL".to_string()))?;
        let path = parsed.path().trim_end_matches('/');
        let article_name = path.split('/').next_back().unwrap_or("Unknown");

        // 使用 Wikipedia REST API 获取摘要
        let api_url = format!(
            "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
            urlencoding(article_name)
        );
        let resp = self.http_client.get(&api_url)
            .send()
            .map_err(|e| NeoTrixError::Network(format!("Wikipedia API 请求失败: {}", e)))?;
        let text = resp.text()
            .map_err(|e| NeoTrixError::Network(format!("Wikipedia 响应读取失败: {}", e)))?;

        // 解析 JSON 响应
        let parsed_json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| NeoTrixError::Serde(format!("Wikipedia JSON 解析失败: {}", e)))?;

        let title = parsed_json["title"].as_str().unwrap_or(article_name).to_string();
        let extract = parsed_json["extract"].as_str().unwrap_or("");
        let content_length = extract.len();

        let (edits, insights) = Self::analyze_wikipedia_content(&title, extract);

        Ok(WebMinedKnowledge {
            source_url: url_str.to_string(),
            source_name: format!("wiki_{}", title.replace(' ', "_")),
            source_type: WebSourceType::Wikipedia,
            title,
            summary: extract.chars().take(200).collect(),
            content_length,
            extracted_insights: insights,
            edits,
            confidence: 0.85,
        })
    }

    /// arXiv API 挖掘
    fn mine_arxiv(&self, url_str: &str) -> NeoTrixResult<WebMinedKnowledge> {
        let parsed = Url::parse(url_str)
            .map_err(|_| NeoTrixError::Network("无效 arXiv URL".to_string()))?;
        let paper_id = parsed.path().trim_end_matches('/')
            .split('/').next_back().unwrap_or("unknown");

        // arXiv API
        let api_url = format!("http://export.arxiv.org/api/query?id_list={}", paper_id);
        let resp = self.http_client.get(&api_url)
            .send()
            .map_err(|e| NeoTrixError::Network(format!("arXiv API 请求失败: {}", e)))?;
        let xml_text = resp.text()
            .map_err(|e| NeoTrixError::Network(format!("arXiv 响应读取失败: {}", e)))?;

        // 简易 XML 解析
        let title = extract_xml_tag(&xml_text, "title").unwrap_or_else(|| format!("arXiv:{}", paper_id));
        let summary = extract_xml_tag(&xml_text, "summary").unwrap_or_default();
        let content_length = summary.len();

        let mut edits = vec![
            ("inference_depth".to_string(), 0.12),
            ("domain_specificity".to_string(), 0.10),
            ("analysis".to_string(), 0.08),
        ];
        let mut insights = vec!["学术论文: 增强推理深度与领域专精度".to_string()];

        let lower = summary.to_lowercase();
        if lower.contains("machine learning") || lower.contains("deep learning") || lower.contains("neural") {
            edits.push(("experimental".to_string(), 0.06));
            insights.push("ML/AI 论文: 增强实验性".to_string());
        }
        if lower.contains("time") || lower.contains("history") || lower.contains("evolution") {
            edits.push(("synthesis".to_string(), 0.06));
            insights.push("历史/进化相关: 增强综合能力".to_string());
        }

        Ok(WebMinedKnowledge {
            source_url: url_str.to_string(),
            source_name: format!("arxiv_{}", title.chars().take(30).collect::<String>().replace(' ', "_")),
            source_type: WebSourceType::ArXiv,
            title,
            summary: summary.chars().take(200).collect(),
            content_length,
            extracted_insights: insights,
            edits,
            confidence: 0.80,
        })
    }

    /// GitHub 挖掘（直接通过文件名分析）
    fn mine_github(&self, url_str: &str) -> NeoTrixResult<WebMinedKnowledge> {
        let parsed = Url::parse(url_str)
            .map_err(|_| NeoTrixError::Network("无效 GitHub URL".to_string()))?;
        let path_segments: Vec<&str> = parsed.path().trim_matches('/').split('/').collect();
        let repo_full = if path_segments.len() >= 2 {
            format!("{}/{}", path_segments[0], path_segments[1])
        } else {
            return Err(NeoTrixError::from("GitHub URL 格式: owner/repo"));
        };

        // 使用 GitHub API 获取仓库信息
        let api_url = format!("https://api.github.com/repos/{}", repo_full);
        let resp = self.http_client.get(&api_url)
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .map_err(|e| NeoTrixError::Network(format!("GitHub API 请求失败: {}", e)))?;
        let text = resp.text()
            .map_err(|e| NeoTrixError::Network(format!("GitHub 响应读取失败: {}", e)))?;

        let parsed_json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| NeoTrixError::Serde(format!("GitHub JSON 解析失败: {}", e)))?;

        let title = parsed_json["full_name"].as_str().unwrap_or(&repo_full).to_string();
        let desc = parsed_json["description"].as_str().unwrap_or("").to_string();
        let lang = parsed_json["language"].as_str().unwrap_or("unknown").to_string();
        let topics: Vec<String> = parsed_json["topics"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let content_length = desc.len();

        let mut edits = Vec::new();
        let mut insights = Vec::new();

        // 根据语言和话题注入知识
        match lang.as_str() {
            "Rust" => {
                edits.push(("domain_specificity".to_string(), 0.08));
                insights.push("Rust 项目: 增强领域专精度".to_string());
            }
            "Python" => {
                edits.push(("analysis".to_string(), 0.06));
                insights.push("Python 项目: 增强分析能力".to_string());
            }
            "TypeScript" | "JavaScript" => {
                edits.push(("compound_composition".to_string(), 0.06));
                insights.push("JS/TS 项目: 增强复合组合能力".to_string());
            }
            _ => {}
        }

        let desc_lower = desc.to_lowercase();
        if desc_lower.contains("ai") || desc_lower.contains("machine learning") || desc_lower.contains("deep learning") {
            edits.push(("inference_depth".to_string(), 0.07));
            insights.push("AI/ML 项目: 增强推理深度".to_string());
        }
        if desc_lower.contains("database") || desc_lower.contains("storage") || desc_lower.contains("sql") {
            edits.push(("synthesis".to_string(), 0.05));
            insights.push("数据项目: 增强综合能力".to_string());
        }
        if topics.contains(&"nt_shield".to_string()) || desc_lower.contains("nt_shield") {
            edits.push(("quality_gates".to_string(), 0.07));
            insights.push("安全项目: 增强质量门控".to_string());
        }
        if desc_lower.contains("design") || desc_lower.contains("ui") || desc_lower.contains("frontend") {
            edits.push(("accessibility".to_string(), 0.05));
            insights.push("前端/设计项目: 增强可访问性".to_string());
        }

        if edits.is_empty() {
            edits.push(("domain_specificity".to_string(), 0.04));
        }

        Ok(WebMinedKnowledge {
            source_url: url_str.to_string(),
            source_name: format!("gh_{}", title.replace('/', "_")),
            source_type: WebSourceType::GitHub,
            title,
            summary: desc,
            content_length,
            extracted_insights: insights,
            edits,
            confidence: 0.75,
        })
    }

    /// 通用 URL 挖掘
    fn mine_generic(&self, url_str: &str) -> NeoTrixResult<WebMinedKnowledge> {
        let resp = self.http_client.get(url_str)
            .send()
            .map_err(|e| NeoTrixError::Network(format!("请求失败: {}", e)))?;
        let text = resp.text()
            .map_err(|e| NeoTrixError::Network(format!("响应读取失败: {}", e)))?;
        let content_length = text.len();

        let parsed = Url::parse(url_str)
            .map_err(|e| NeoTrixError::Network(format!("URL 解析失败: {}", e)))?;
        let page_name = parsed.path().trim_end_matches('/')
            .split('/').next_back().filter(|s| !s.is_empty()).unwrap_or("page");

        // HTML解析：提取标题和正文
        let title = extract_html_title(&text).unwrap_or_else(|| page_name.to_string());
        let body_text = strip_html_tags(&text);
        let lower = body_text.to_lowercase();

        let mut edits = Vec::new();
        let mut insights = Vec::new();

        if lower.contains("history") || lower.contains("timeline") || lower.contains("century") || lower.contains("ancient") {
            edits.push(("synthesis".to_string(), 0.07));
            edits.push(("inference_depth".to_string(), 0.05));
            insights.push("历史内容: 增强综合与推理能力".to_string());
        }
        if lower.contains("science") || lower.contains("physics") || lower.contains("biology") || lower.contains("chemistry") {
            edits.push(("domain_specificity".to_string(), 0.08));
            insights.push("科学内容: 增强领域专精度".to_string());
        }
        if lower.contains("philosophy") || lower.contains("culture") || lower.contains("society") {
            edits.push(("creativity".to_string(), 0.06));
            insights.push("人文内容: 增强创造力".to_string());
        }
        if lower.contains("technology") || lower.contains("engineering") || lower.contains("computer") {
            edits.push(("analysis".to_string(), 0.07));
            insights.push("科技内容: 增强分析能力".to_string());
        }
        if lower.contains("dimension") || lower.contains("multiverse") || lower.contains("spacetime") || lower.contains("quantum") {
            edits.push(("experimental".to_string(), 0.08));
            edits.push(("inference_depth".to_string(), 0.06));
            insights.push("维度/量子内容: 增强实验性与推理深度".to_string());
        }
        if lower.contains("evolution") || lower.contains("civilization") || lower.contains("human") {
            edits.push(("synthesis".to_string(), 0.06));
            insights.push("人类/文明内容: 增强综合能力".to_string());
        }

        if edits.is_empty() {
            edits.push(("domain_specificity".to_string(), 0.03));
            insights.push("通用内容: 少量领域专精度提升".to_string());
        }

        Ok(WebMinedKnowledge {
            source_url: url_str.to_string(),
            source_name: format!("web_{}", title.chars().take(20).collect::<String>().replace(' ', "_")),
            source_type: WebSourceType::GenericUrl,
            title,
            summary: body_text.chars().take(200).collect(),
            content_length,
            extracted_insights: insights,
            edits,
            confidence: 0.70,
        })
    }

    // ==================== 辅助方法 ====================

    fn analyze_wikipedia_content(title: &str, extract: &str) -> (Vec<(String, f64)>, Vec<String>) {
        let mut edits = Vec::new();
        let mut insights = Vec::new();
        let lower = extract.to_lowercase();

        // 文明/历史
        if lower.contains("civilization") || lower.contains("empire") || lower.contains("dynasty")
            || lower.contains("history") || lower.contains("century") || lower.contains("ancient") {
            edits.push(("synthesis".to_string(), 0.08));
            insights.push("文明/历史: 增强综合分析能力".to_string());
        }
        // 时间线/进化
        if lower.contains("evolution") || lower.contains("timeline") || lower.contains("chronology")
            || lower.contains("million years") || lower.contains("billion years") {
            edits.push(("inference_depth".to_string(), 0.07));
            insights.push("时间线/进化: 增强推理深度".to_string());
        }
        // 物理/维度
        if lower.contains("dimension") || lower.contains("spacetime") || lower.contains("relativity")
            || lower.contains("quantum") || lower.contains("string theory") || lower.contains("multiverse") {
            edits.push(("experimental".to_string(), 0.08));
            edits.push(("domain_specificity".to_string(), 0.07));
            insights.push("物理/维度: 增强实验性和领域专精度".to_string());
        }
        // 文化/哲学
        if lower.contains("philosophy") || lower.contains("culture") || lower.contains("religion")
            || lower.contains("art") || lower.contains("literature") {
            edits.push(("creativity".to_string(), 0.06));
            insights.push("文化/哲学: 增强创造力".to_string());
        }
        // 科技
        if lower.contains("technology") || lower.contains("industrial") || lower.contains("digital")
            || lower.contains("invention") || lower.contains("discovery") {
            edits.push(("analysis".to_string(), 0.06));
            insights.push("科技/发明: 增强分析能力".to_string());
        }
        // 战争/冲突 — 文明演进的重要维度
        if lower.contains("war") || lower.contains("conflict") || lower.contains("revolution")
            || lower.contains("battle") || lower.contains("invasion") {
            edits.push(("analysis".to_string(), 0.04));
            insights.push("冲突/战争: 增强分析能力".to_string());
        }
        // 地理/生态
        if lower.contains("geography") || lower.contains("continent") || lower.contains("ocean")
            || lower.contains("climate") || lower.contains("species") {
            edits.push(("domain_specificity".to_string(), 0.05));
            insights.push("地理/生态: 增强领域专精度".to_string());
        }
        // 标题相关性加强
        let title_lower = title.to_lowercase();
        if title_lower.contains("history") || title_lower.contains("time") || title_lower.contains("evolution") {
            edits.push(("synthesis".to_string(), 0.05));
        }
        if title_lower.contains("dimension") || title_lower.contains("universe") || title_lower.contains("physics") {
            edits.push(("experimental".to_string(), 0.05));
        }

        (edits, insights)
    }

    fn edits_to_vector(edits: &[(String, f64)]) -> CapabilityVector {
        let mut cv = CapabilityVector::default();
        for (dim, delta) in edits {
            if let Some(idx) = CapabilityVector::index_from_name(dim) {
                cv.arr_mut()[idx] = *delta;
            }
        }
        cv.normalize();
        cv
    }

    /// 获取挖掘统计
    pub fn stats(&self) -> WebMinerStats {
        let by_type: HashMap<WebSourceType, usize> = self.mined_history.iter()
            .fold(HashMap::new(), |mut acc, k| {
                *acc.entry(k.source_type).or_insert(0) += 1;
                acc
            });
        let total_edits: usize = self.mined_history.iter().map(|k| k.edits.len()).sum();
        WebMinerStats {
            total_mined: self.mined_history.len(),
            total_edits,
            by_type,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WebMineResult {
    pub total_urls: usize,
    pub success_count: usize,
    pub total_edits: usize,
    pub total_reward: f64,
    pub details: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WebMinerStats {
    pub total_mined: usize,
    pub total_edits: usize,
    pub by_type: HashMap<WebSourceType, usize>,
}

// ==================== 工具函数 ====================

fn urlencoding(s: &str) -> String {
    s.replace(' ', "_")
}

fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    xml.find(&open).and_then(|start| {
        let content_start = start + open.len();
        xml[content_start..].find(&close).map(|end| {
            xml[content_start..content_start + end].trim().to_string()
        })
    })
}

fn extract_html_title(html: &str) -> Option<String> {
    html.find("<title>").and_then(|start| {
        let content_start = start + 7;
        html[content_start..].find("</title>").map(|end| {
            html[content_start..content_start + end].trim().to_string()
        })
    })
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    // 清理多余空白
    let cleaned: Vec<&str> = result.split_whitespace().collect();
    cleaned.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_source_type_detect() {
        assert_eq!(WebSourceType::detect("https://en.wikipedia.org/wiki/History"), WebSourceType::Wikipedia);
        assert_eq!(WebSourceType::detect("https://arxiv.org/abs/2506.10943"), WebSourceType::ArXiv);
        assert_eq!(WebSourceType::detect("https://github.com/rust-lang/rust"), WebSourceType::GitHub);
        assert_eq!(WebSourceType::detect("https://example.com/page"), WebSourceType::GenericUrl);
        assert_eq!(WebSourceType::detect("https://knowledge-base.com"), WebSourceType::KnowledgeBase);
    }

    #[test]
    fn test_source_type_name() {
        assert_eq!(WebSourceType::Wikipedia.name(), "Wikipedia");
        assert_eq!(WebSourceType::ArXiv.name(), "ArXiv");
        assert_eq!(WebSourceType::GitHub.name(), "GitHub");
    }

    #[test]
    fn test_strip_html_tags() {
        let html = "<html><body><p>Hello World</p></body></html>";
        assert_eq!(strip_html_tags(html), "Hello World");
    }

    #[test]
    fn test_extract_html_title() {
        let html = "<html><title>Test Title</title><body></body></html>";
        assert_eq!(extract_html_title(html), Some("Test Title".to_string()));
    }

    #[test]
    fn test_extract_xml_tag() {
        let xml = "<entry><title>Paper Title</title><summary>Abstract here</summary></entry>";
        assert_eq!(extract_xml_tag(xml, "title"), Some("Paper Title".to_string()));
        assert_eq!(extract_xml_tag(xml, "summary"), Some("Abstract here".to_string()));
    }

    #[test]
    fn test_urlencoding() {
        assert_eq!(urlencoding("History of Earth"), "History_of_Earth");
        assert_eq!(urlencoding("Simple"), "Simple");
    }

    #[test]
    fn test_analyze_wikipedia_civilization() {
        let (edits, _) = WebKnowledgeMiner::analyze_wikipedia_content("Ancient Rome", "The Roman Empire was a civilization that dominated the Mediterranean world for centuries.");
        assert!(!edits.is_empty());
        assert!(edits.iter().any(|(n, _)| n == "synthesis"));
    }

    #[test]
    fn test_analyze_wikipedia_dimension() {
        let (edits, _) = WebKnowledgeMiner::analyze_wikipedia_content("Spacetime", "In physics, spacetime is a mathematical model that combines space and time into a single continuum.");
        assert!(edits.iter().any(|(n, _)| n == "experimental"));
    }

    #[test]
    fn test_web_miner_new() {
        let miner = WebKnowledgeMiner::new(PathBuf::from("/tmp"));
        assert_eq!(miner.mined_history.len(), 0);
    }

    #[test]
    fn test_stats_empty() {
        let miner = WebKnowledgeMiner::new(PathBuf::from("/tmp"));
        let stats = miner.stats();
        assert_eq!(stats.total_mined, 0);
        assert_eq!(stats.total_edits, 0);
    }
}
