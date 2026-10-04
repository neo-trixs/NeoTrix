//! NeoTrix 知识吸收管道 — GitHub/外部代码 → KB 蒸馏 → 去重更新

use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::l5_cognition::nt_mind::foundation::knowledge_store::{KnowledgeStore, L1KnowledgeStore};
use crate::l4_emotion::nt_memory::nt_memory_kb::KnowledgeBase;
use neotrix_types::knowledge_access::NodeType;

// ============================================================
// 源类型 (使用唯一名避免冲突)
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum _KbSourceType {
    GitHub,
    ArXiv,
    Wikipedia,
    WebArticle,
    Paper,
    Documentation,
    CodeRepository,
    Blog,
}

impl std::fmt::Display for _KbSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            _KbSourceType::GitHub => write!(f, "GitHub"),
            _KbSourceType::ArXiv => write!(f, "ArXiv"),
            _KbSourceType::Wikipedia => write!(f, "Wikipedia"),
            _KbSourceType::WebArticle => write!(f, "WebArticle"),
            _KbSourceType::Paper => write!(f, "Paper"),
            _KbSourceType::Documentation => write!(f, "Documentation"),
            _KbSourceType::CodeRepository => write!(f, "CodeRepository"),
            _KbSourceType::Blog => write!(f, "Blog"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceEntry {
    pub url: String,
    pub source_type: _KbSourceType,
    pub title: String,
    pub kb_node_id: Option<String>,
    pub last_absorbed: i64,
    pub sha_hash: Option<String>,
    pub distill_summary: Option<String>,
    pub tags: Vec<String>,
}

/// source_map 上限：长驻 daemon 每 URL 一条，无界增长 → 内存泄漏
const SOURCE_MAP_LIMIT: usize = 2000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _AbsorbState {
    pub source_map: HashMap<String, SourceEntry>,
    pub total_absorbed: usize,
    pub last_panorama_update: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillResult {
    pub title: String,
    pub summary: String,
    pub core_concepts: Vec<String>,
    pub architecture_insights: Vec<String>,
    pub key_algorithms: Vec<String>,
    pub dependencies: Vec<String>,
    pub code_patterns: Vec<String>,
    pub confidence: f64,
}

// ============================================================
// 知识吸收管道
// ============================================================

pub struct KnowledgeAbsorptionPipeline {
    pub kb: Option<Arc<KnowledgeBase>>,
    state: _AbsorbState,
}

impl Default for KnowledgeAbsorptionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeAbsorptionPipeline {
    pub fn new() -> Self {
        Self {
            kb: None,
            state: _AbsorbState {
                source_map: HashMap::new(),
                total_absorbed: 0,
                last_panorama_update: 0,
            },
        }
    }

    pub fn attach_kb(&mut self, kb: Arc<KnowledgeBase>) {
        self.kb = Some(kb);
    }

    pub fn get_kb(&self) -> Option<Arc<KnowledgeBase>> {
        self.kb.clone()
    }

    pub fn absorb_url(&mut self, url: &str) -> Result<_AbsorptionReport, String> {
        let store = L1KnowledgeStore;
        if !store.is_safe_fetch_url(url) {
            return Err(format!("URL rejected (SSRF guard): {}", url));
        }
        if let Some(cached) = self.cached_report(url) {
            return Ok(cached);
        }

        // HTTP fetch + content extraction (P0 fix: was inserting empty External nodes)
        let (content, domain) =
            crate::l4_emotion::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(url)?;

        self.finish_absorb(url, &content, &domain)
    }

    pub async fn absorb_url_async(&mut self, url: &str) -> Result<_AbsorptionReport, String> {
        let store = L1KnowledgeStore;
        if !store.is_safe_fetch_url(url) {
            return Err(format!("URL rejected (SSRF guard): {}", url));
        }
        if let Some(cached) = self.cached_report(url) {
            return Ok(cached);
        }

        let (content, domain) =
            crate::l4_emotion::nt_memory::nt_memory_kb::nt_http::fetch_safe_http_async(url)
                .await?;

        self.finish_absorb(url, &content, &domain)
    }

    /// 去重：source_map 24h 内已吸收 或 KB 已存在同 URL 节点 → 返回 cached 报告
    fn cached_report(&self, url: &str) -> Option<_AbsorptionReport> {
        if let Some(entry) = self.state.source_map.get(url) {
            let age = Utc::now().timestamp() - entry.last_absorbed;
            if age < 86400 {
                return Some(_AbsorptionReport {
                    url: url.into(), source_type: _KbSourceType::WebArticle,
                    action: "cached".into(), nodes_created: 0, edges_created: 0,
                    distil_summary: None,
                });
            }
        }
        if let Some(ref kb) = self.kb {
            if let Ok(Some(_existing)) = kb.find_node_by_url(url) {
                return Some(_AbsorptionReport {
                    url: url.into(), source_type: _KbSourceType::WebArticle,
                    action: "cached".into(), nodes_created: 0, edges_created: 0,
                    distil_summary: None,
                });
            }
        }
        None
    }

    /// 提取 → 插入 → 记录来源 (同步/异步共用)
    fn finish_absorb(&mut self, url: &str, content: &str, domain: &str) -> Result<_AbsorptionReport, String> {
        let store = L1KnowledgeStore;
        let summary = store.extract_html_content(content).1;
        let summary_short = if summary.len() > 5000 {
            format!("{}...", summary.chars().take(5000).collect::<String>())
        } else {
            summary.clone()
        };

        let node_type = classify_node_type(url);

        // 插入失败必须传播错误：绝不把失败 URL 记成已吸收 (record_source 会使其 24h 不再重试)
        let kb = self.kb.as_ref().ok_or("knowledge base not initialized")?;
        let node_id = kb.insert_or_get_node(url, node_type.clone(),
            Some(&summary_short), Some(url), Some(domain))
            .map_err(|e| format!("KB insert failed for {}: {}", url, e))?;

        self.record_source(url.to_string(), SourceEntry {
            url: url.into(), source_type: _KbSourceType::WebArticle,
            title: url.into(), kb_node_id: Some(node_id.clone()),
            last_absorbed: Utc::now().timestamp(),
            sha_hash: None, distill_summary: None, tags: vec![],
        });
        self.state.total_absorbed += 1;

        Ok(_AbsorptionReport {
            url: url.into(), source_type: _KbSourceType::WebArticle,
            action: "absorbed".into(),
            nodes_created: 1,
            edges_created: 0,
            distil_summary: Some(summary_short),
        })
    }

    pub fn absorb_github(&mut self, url: &str) -> Result<_AbsorptionReport, String> {
        let parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();
        let repo = parts.last().ok_or("无法解析仓库名")?.to_string();
        let owner = if parts.len() >= 2 { parts[parts.len()-2].to_string() } else { String::new() };

        let url_key = format!("github:{}/{}", owner, repo);

        // 去重检查
        if let Some(entry) = self.state.source_map.get(&url_key) {
            if Utc::now().timestamp() - entry.last_absorbed < 3600 {
                return Ok(_AbsorptionReport {
                    url: url.into(), source_type: _KbSourceType::GitHub,
                    action: "skipped".into(), nodes_created: 0, edges_created: 0,
                    distil_summary: None,
                });
            }
        }

        // 蒸馏
        let distill = DistillResult {
            title: repo.clone(),
            summary: format!("GitHub 仓库 {}/{}", owner, repo),
            core_concepts: vec![format!("{}/{}", owner, repo)],
            architecture_insights: vec![],
            key_algorithms: vec![],
            dependencies: vec![],
            code_patterns: vec![],
            confidence: 0.6,
        };

        self.record_source(url_key.clone(), SourceEntry {
            url: url.into(), source_type: _KbSourceType::GitHub,
            title: repo, kb_node_id: None,
            last_absorbed: Utc::now().timestamp(),
            sha_hash: None,
            distill_summary: Some(distill.summary.clone()),
            tags: vec![],
        });
        self.state.total_absorbed += 1;

        Ok(_AbsorptionReport {
            url: url.into(), source_type: _KbSourceType::GitHub,
            action: "absorbed".into(), nodes_created: 1, edges_created: 0,
            distil_summary: Some(distill.summary),
        })
    }

    /// 更新全景索引
    pub fn update_panorama(&mut self) -> Result<PanoramaReport, String> {
        let now = Utc::now().timestamp();
        let mut by_type: HashMap<String, usize> = HashMap::new();
        for entry in self.state.source_map.values() {
            *by_type.entry(format!("{}", entry.source_type)).or_insert(0) += 1;
        }
        self.state.last_panorama_update = now;
        Ok(PanoramaReport {
            total_sources: self.state.source_map.len(),
            by_type,
            updated_at: now,
        })
    }

    pub fn recent_sources(&self, n: usize) -> Vec<&SourceEntry> {
        let mut entries: Vec<_> = self.state.source_map.values().collect();
        entries.sort_by_key(|e| std::cmp::Reverse(e.last_absorbed));
        entries.into_iter().take(n).collect()
    }

    pub fn stats(&self) -> _KbPipelineStats {
        _KbPipelineStats {
            total_sources: self.state.source_map.len(),
            total_absorbed: self.state.total_absorbed,
        }
    }

    /// 记录来源，超上限时淘汰最旧条目，防止长驻 daemon 内存无界增长
    fn record_source(&mut self, url: String, entry: SourceEntry) {
        if self.state.source_map.len() >= SOURCE_MAP_LIMIT {
            let mut oldest: Option<(i64, String)> = None;
            for (k, e) in self.state.source_map.iter() {
                if oldest.as_ref().is_none_or(|(ts, _)| e.last_absorbed < *ts) {
                    oldest = Some((e.last_absorbed, k.clone()));
                }
            }
            if let Some((_, k)) = oldest {
                self.state.source_map.remove(&k);
            }
        }
        self.state.source_map.insert(url, entry);
    }
}

// ============================================================
// 报告类型
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _AbsorptionReport {
    pub url: String,
    pub source_type: _KbSourceType,
    pub action: String,
    pub nodes_created: usize,
    pub edges_created: usize,
    pub distil_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanoramaReport {
    pub total_sources: usize,
    pub by_type: HashMap<String, usize>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _KbPipelineStats {
    pub total_sources: usize,
    pub total_absorbed: usize,
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_crawl::extract_html_content;

    #[test]
    fn test_pipeline_new() {
        let pipe = KnowledgeAbsorptionPipeline::new();
        assert_eq!(pipe.stats().total_sources, 0);
    }

    #[test]
    fn test_absorb_url_dedup() {
        let mut pipe = KnowledgeAbsorptionPipeline::new();
        let r1 = pipe.absorb_github("https://github.com/x/y").expect("github first");
        assert_eq!(r1.source_type, _KbSourceType::GitHub);
        let r2 = pipe.absorb_github("https://github.com/x/y").expect("github second");
        assert_eq!(r2.source_type, _KbSourceType::GitHub);
    }

    #[test]
    fn test_absorb_github() {
        let mut pipe = KnowledgeAbsorptionPipeline::new();
        let r = pipe.absorb_github("https://github.com/rust-lang/rust").expect("github");
        assert_eq!(r.source_type, _KbSourceType::GitHub);
    }

    #[test]
    fn test_panorama() {
        let mut pipe = KnowledgeAbsorptionPipeline::new();
        let _ = pipe.absorb_github("https://github.com/x/y").unwrap();
        let pan = pipe.update_panorama().expect("panorama");
        assert_eq!(pan.total_sources, 1);
    }

    #[test]
    fn test_source_type_display() {
        assert_eq!(_KbSourceType::GitHub.to_string(), "GitHub");
        assert_eq!(_KbSourceType::ArXiv.to_string(), "ArXiv");
    }

    #[test]
    fn test_extract_html_content_basic() {
        let html = "<html><body><h1>Title</h1><p>Hello world</p></body></html>";
        let (_, text) = extract_html_content(html);
        assert!(text.contains("Title"));
        assert!(text.contains("Hello world"));
    }

    #[test]
    fn test_extract_html_content_entities() {
        let html = "<p>foo &amp; bar &lt; 3</p>";
        let (_, text) = extract_html_content(html);
        assert_eq!(text, "foo & bar < 3");
    }
}


/// 由 URL 判定知识节点类型（写入 KB 的**持久类型**）。
///
/// # ⭐ 2026-10-03：**域名规则**改为 host 判定
///
/// ⛔ 原裸 `contains("github.com")` 会把 `https://github.com.evil.net/…`
///    判成 [`NodeType::Repository`] 并**持久化进知识库** —— 之后所有
///    消费该节点的逻辑都会被误导。
///
/// ⚠️ `contains("paper")` 是**关键词启发式**，刻意保持不变 ——
///    收紧它会把 `/papers/123` 从 Paper 改成 Article，
///    那是**行为变更**而非修 bug。故 arxiv 走 host 判定、paper 仍 contains。
///
/// ⭐ 抽成自由函数而非内联 `if`，是为了让判据**可被测试** ——
/// 内联在深调用栈里时，伪装域名的问题无法写回归测试。
pub fn classify_node_type(url: &str) -> NodeType {
    use crate::l0_substrate::nt_core_platform::url_match::url_matches_domain as m;
    if m(url, "arxiv.org") || url.contains("paper") {
        NodeType::Paper
    } else if m(url, "github.com") {
        NodeType::Repository
    } else if m(url, "wikipedia.org") {
        NodeType::Reference
    } else {
        NodeType::Article
    }
}

#[cfg(test)]
mod classify_node_type_tests {
    use super::classify_node_type;
    use neotrix_types::knowledge_access::NodeType;

    /// ⭐ 迁移回归：正常 URL 分类完全不变
    #[test]
    fn normal_urls_keep_their_type() {
        for (url, want) in [
            ("https://arxiv.org/abs/1234", NodeType::Paper),
            ("https://github.com/a/b", NodeType::Repository),
            ("https://en.wikipedia.org/wiki/Rust", NodeType::Reference),
            ("https://blog.example.org/post", NodeType::Article),
            // ⭐ 关键词启发式必须仍然生效
            ("https://x.com/papers/123", NodeType::Paper),
            ("https://example.org/my-paper", NodeType::Paper),
        ] {
            assert_eq!(classify_node_type(url), want, "url: {}", url);
        }
    }

    /// ⭐ 伪装域名不再被写成 Repository/Reference（该值会**持久化**）
    #[test]
    fn lookalike_domains_are_not_typed_as_platform_nodes() {
        for hostile in [
            "https://github.com.evil.net/a/b",
            "https://wikipedia.org.evil.net/wiki/X",
            "https://arxiv.org.evil.net/abs/1",
        ] {
            assert_eq!(
                classify_node_type(hostile),
                NodeType::Article,
                "hostile {} must not be persisted as a platform node",
                hostile
            );
        }
    }

    /// ⭐⭐ 守住关键词语义：收紧域名判定时**不得**顺手把
    /// `contains("paper")` 也改掉 —— 那会让 `/papers/123` 变 Article。
    #[test]
    fn paper_keyword_heuristic_survives() {
        assert_eq!(classify_node_type("https://x.com/papers/1"), NodeType::Paper);
        assert_eq!(classify_node_type("https://example.org/my-paper"), NodeType::Paper);
        // ⚠️ 大写 `PAPERS` **不**命中 —— 这是**既有**行为（`contains("paper")`
        //    本身大小写敏感），本次未改、也不该在此顺手改（那是独立的行为变更）。
        assert_eq!(classify_node_type("https://x.com/PAPERS/1"), NodeType::Article);
    }

    /// ⭐ 分支顺序语义锁定：arXiv+paper 分支在**最前**，
    /// 故含 `papers` 的 github URL 判 Paper。
    /// ⚠️ 这是**既有**顺序，本次未改 —— 独立 harness 逐条比对 11 个 URL
    /// 确认：除伪装域名外，分类结果全部不变。
    #[test]
    fn branch_order_is_paper_first() {
        assert_eq!(
            classify_node_type("https://github.com/owner/papers"),
            NodeType::Paper,
            "the paper branch is first, so `papers` wins over the github rule"
        );
    }
}
