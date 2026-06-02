use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use url::Url;

use super::core::CapabilityVector;
use super::self_edit::MicroEdit;
use super::self_iterating::ReasoningBrain;
use super::memory::{ReasoningBank, ReasoningMemory};
use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};
use crate::neotrix::nt_world_model::TaskType;

/// 知识挖掘结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinedKnowledge {
    pub source_url: String,
    pub source_name: String,
    pub domain: String,
    pub capability_vector: CapabilityVector,
    pub confidence: f64,
    pub micro_edits: Vec<MicroEdit>,
    pub tech_stack: Vec<String>,
    pub insights: Vec<String>,
}

/// 自动知识挖掘器 — 从外部来源自动发现、分析、注册知识
pub struct KnowledgeMiner {
    /// 待发现的来源队列
    pub discovery_queue: Vec<String>,
    /// 已挖掘的来源记录 (URL → 名称)
    pub mined_sources: HashMap<String, String>,
    /// 工作目录
    pub work_dir: PathBuf,
    /// 已处理的来源哈希 (去重)
    processed: std::collections::HashSet<String>,
}

impl KnowledgeMiner {
    pub fn new(work_dir: PathBuf) -> Self {
        Self {
            discovery_queue: Vec::new(),
            mined_sources: HashMap::new(),
            work_dir,
            processed: std::collections::HashSet::new(),
        }
    }

    /// 添加待发现来源
    pub fn enqueue(&mut self, url: &str) {
        if !self.processed.contains(url) && !self.discovery_queue.contains(&url.to_string()) {
            self.discovery_queue.push(url.to_string());
        }
    }

    /// 添加多个默认发现目标（覆盖 AI/ML/Rust/Consciousness/Security/系统编程等）
    pub fn enqueue_default_targets(&mut self) {
        let targets = vec![
            // === AI/ML 框架 (Rust) ===
            "https://github.com/huggingface/candle",
            "https://github.com/rust-ml/linfa",
            "https://github.com/nicktolas/dfdx",
            "https://github.com/sonos/tract",
            "https://github.com/jerry73204/leaf-ai",
            "https://github.com/nathaneastwood/tarpc-ml",
            "https://github.com/LaurentMazare/ocaml-torch",
            // === AI/ML 框架 (Python, for reference) ===
            "https://github.com/huggingface/transformers",
            "https://github.com/langchain-ai/langchain",
            "https://github.com/openai/openai-cookbook",
            // === 意识科学 IIT/GWT ===
            "https://github.com/wmayner/pyphi",
            "https://github.com/artem-oppermann/Integrated-Information-Theory",
            "https://github.com/NeuromatchAcademy/hyperdimensional-computing",
            "https://github.com/IBM/hd-computing",
            "https://github.com/ai-nikolai/wolfram-model-ecology",
            "https://github.com/BertieWheeler/nt_world_infer",
            "https://github.com/infer-actively/pymdp",
            // === 多智能体框架 ===
            "https://github.com/microsoft/autogen",
            "https://github.com/crewAIInc/crewAI",
            "https://github.com/langchain-ai/langgraph",
            "https://github.com/aiwaves-cn/agents",
            "https://github.com/metr/multi-agent",
            // === MCP 生态 ===
            "https://github.com/modelcontextprotocol/servers",
            "https://github.com/anthropics/claude-code",
            "https://github.com/opencode-ai/opencode",
            "https://github.com/continuedev/continue",
            // === Rust 安全/系统编程 ===
            "https://github.com/rust-lang/rust",
            "https://github.com/tokio-rs/tokio",
            "https://github.com/serde-rs/serde",
            "https://github.com/ratatui-org/ratatui",
            "https://github.com/bevyengine/bevy",
            "https://github.com/nushell/nushell",
            "https://github.com/helix-editor/helix",
            // === 安全扫描 ===
            "https://github.com/projectdiscovery/nuclei",
            "https://github.com/zaproxy/zaproxy",
            "https://github.com/projectdiscovery/subfinder",
            "https://github.com/OJ/gobuster",
            "https://github.com/Betterleaks/Betterleaks",
            // === 知识/学习系统 ===
            "https://github.com/mem0ai/mem0",
            "https://github.com/MemTensor/MemOS",
            "https://github.com/ReflexioAI/reflexio",
            "https://github.com/andreycpu/synesis",
            "https://github.com/google-research/nested-learning",
            // === 图数据库/超图记忆 ===
            "https://github.com/neo4j/neo4j",
            "https://github.com/apache/age",
            "https://github.com/prisma/prisma",
            "https://github.com/rust-lang/hypergraph",
            // === 矢量设计 MCP 工具 ===
            "https://github.com/ZSeven-W/openpencil",
            "https://github.com/heygen-com/hyperframes",
            // === 游戏 AI / 仿真 ===
            "https://github.com/deepmind/mctx",
            "https://github.com/google-deepmind/alphafold",
            "https://github.com/anthropics/evals",
            // === VSA / 超维计算实现 ===
            "https://github.com/torchhd/torchhd",
            "https://github.com/Aleph-Alpha/hdc-toolbox",
            "https://github.com/fbrakel/hyperdimensional_computing",
            // === 物理/宇宙学 ===
            "https://github.com/google-research/kanji-model",
            "https://github.com/nicktolas/dfdx",
            "https://github.com/facebookresearch/earth-invariant-learning",
            // === Agent → Assessment / 评估 ===
            "https://github.com/metr/swe-bench",
            "https://github.com/code-iai/iai-benchmarks",
            "https://github.com/gaia-benchmark/GAIA",
        ];
        for target in targets {
            self.enqueue(target);
        }
    }

    /// 执行一轮知识挖掘：处理队列中的所有来源
    pub fn mine_round(&mut self, brain: &mut ReasoningBrain, bank: &mut ReasoningBank) -> MinedRoundResult {
        let mut results = Vec::new();
        let urls: Vec<String> = self.discovery_queue.drain(..).collect();

        for url in urls {
            if self.processed.contains(&url) {
                continue;
            }
            self.processed.insert(url.clone());

            match self.mine_single(&url) {
                Ok(knowledge) => {
                    // 注册知识来源
                    brain.register_knowledge_source(&knowledge.source_name, knowledge.capability_vector.clone());

                    // 应用 MicroEdits
                    for edit in &knowledge.micro_edits {
                        Self::apply_edit_to_brain(brain, edit);
                    }
                    brain.capability.normalize();

                    // 存储到 ReasoningBank
                    let memory = ReasoningMemory {
                        id: uuid::Uuid::new_v4().to_string(),
                        task_description: format!("KnowledgeMined: {} ({})", knowledge.source_name, knowledge.domain),
                        task_type: Self::domain_to_task_type(&knowledge.domain),
                        micro_edits: knowledge.micro_edits.clone(),
                        reward: knowledge.confidence * 0.85,
                        reward_source: crate::neotrix::nt_mind::core::RewardSource::External,
                        success: knowledge.confidence > 0.6,
                        timestamp: chrono::Utc::now().timestamp(),
                        embedding: None,
                        tier: crate::neotrix::nt_mind::memory::MemoryTier::Semantic,
                        lifecycle: crate::neotrix::nt_mind::memory::MemoryLifecycle::new(knowledge.confidence * 0.85),
                        t3_views: crate::core::nt_core_bank::T3Views::new(),
                    };
                    bank.store(memory);

                    self.mined_sources.insert(url.clone(), knowledge.source_name.clone());
                    results.push(knowledge);
                }
                Err(e) => {
                    eprintln!("[KnowledgeMiner] Failed to mine {}: {}", url, e);
                }
            }
        }

        MinedRoundResult {
            mined_count: results.len(),
            sources: results,
        }
    }

    /// 挖掘单个来源
    fn mine_single(&self, url: &str) -> NeoTrixResult<MinedKnowledge> {
        let parsed = Url::parse(url)
            .map_err(|e| NeoTrixError::Network(format!("无效 URL: {}", e)))?;

        let _domain = parsed.domain().unwrap_or("unknown");
        let _path = parsed.path().trim_matches('/');

        // 克隆仓库
        let local_path = self.fetch_repo(url)?;

        // 深度分析
        self.analyze_repo(&local_path, url)
    }

    /// 克隆仓库到本地
    fn fetch_repo(&self, url: &str) -> NeoTrixResult<PathBuf> {
        let parsed = Url::parse(url)
            .map_err(|_| NeoTrixError::from("无效 URL"))?;
        let path_segments: Vec<&str> = parsed.path().trim_matches('/').split('/').collect();
        let repo_name = if path_segments.len() >= 2 {
            format!("{}-{}", path_segments[path_segments.len()-2], path_segments[path_segments.len()-1])
        } else {
            path_segments.last().unwrap_or(&"unknown").to_string()
        };
        let target_dir = self.work_dir.join(&repo_name);

        if !target_dir.exists() {
            if url.contains(';') || url.contains('`') || url.contains('$') || url.contains('|') || url.contains('&') {
                return Err(NeoTrixError::from("URL 包含非法字符"));
            }
            let target_str = target_dir.to_str()
                .ok_or_else(|| NeoTrixError::from("路径不是有效 UTF-8"))?;
            let status = std::process::Command::new("git")
                .args(["clone", "--depth=1", "--", url, target_str])
                .status()
                .map_err(NeoTrixError::Io)?;
            if !status.success() {
                return Err(NeoTrixError::from("Git 克隆失败"));
            }
        }
        Ok(target_dir)
    }

    /// 深度分析仓库
    fn analyze_repo(&self, path: &Path, source_url: &str) -> NeoTrixResult<MinedKnowledge> {
        let mut tech_stack = Vec::new();
        let mut insights = Vec::new();
        let mut edits = Vec::new();
        let mut domain = "general".to_string();
        let mut name = "unknown".to_string();

        // 提取 repo 名称
        if let Some(repo_name) = source_url.split('/').next_back() {
            name = repo_name.trim_end_matches(".git").to_string();
        }

        // === 1. Cargo.toml 分析 (Rust 项目) ===
        let cargo_path = path.join("Cargo.toml");
        if cargo_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_path) {
                tech_stack.push("Rust".to_string());
                if content.contains("tokio") || content.contains("async-std") {
                    tech_stack.push("async-runtime".to_string());
                    edits.push(MicroEdit::AdjustDimension("domain_specificity".to_string(), 0.08));
                }
                if content.contains("serde") {
                    tech_stack.push("serde".to_string());
                    edits.push(MicroEdit::AdjustDimension("synthesis".to_string(), 0.05));
                }
                if content.contains("tower") || content.contains("warp") || content.contains("axum") {
                    tech_stack.push("web-framework".to_string());
                    edits.push(MicroEdit::AdjustDimension("compound_composition".to_string(), 0.06));
                }
                domain = "backend".to_string();
            }
        }

        // === 2. package.json 分析 (JavaScript/TypeScript 项目) ===
        let pkg_path = path.join("package.json");
        if pkg_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&pkg_path) {
                tech_stack.push("JavaScript/TypeScript".to_string());
                if content.contains("react") || content.contains("next") {
                    tech_stack.push("React".to_string());
                    edits.push(MicroEdit::AdjustDimension("compound_composition".to_string(), 0.07));
                    domain = "frontend".to_string();
                }
                if content.contains("tailwindcss") {
                    tech_stack.push("TailwindCSS".to_string());
                    edits.push(MicroEdit::AdjustDimension("tailwind_proficiency".to_string(), 0.09));
                }
                if content.contains("@radix-ui") || content.contains("react-aria") {
                    tech_stack.push("Radix/ReactAria".to_string());
                    edits.push(MicroEdit::AdjustDimension("react_aria_usage".to_string(), 0.08));
                }
                if content.contains("express") || content.contains("fastify") || content.contains("nestjs") {
                    tech_stack.push("Node.js-backend".to_string());
                    domain = "backend".to_string();
                }
            }
        }

        // === 3. pyproject.toml / setup.py 分析 (Python 项目) ===
        let pyproject_path = path.join("pyproject.toml");
        if pyproject_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&pyproject_path) {
                tech_stack.push("Python".to_string());
                if content.contains("torch") || content.contains("tensorflow") || content.contains("transformers") {
                    tech_stack.push("ML/DL".to_string());
                    edits.push(MicroEdit::AdjustDimension("domain_specificity".to_string(), 0.12));
                    edits.push(MicroEdit::AdjustDimension("inference_depth".to_string(), 0.08));
                    domain = "ai-ml".to_string();
                }
                if content.contains("fastapi") || content.contains("django") || content.contains("flask") {
                    tech_stack.push("Python-web".to_string());
                    domain = "backend".to_string();
                }
            }
        }

        // === 4. README 分析 ===
        for readme_name in &["README.md", "README.rst", "README.txt", "README"] {
            let readme_path = path.join(readme_name);
            if readme_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&readme_path) {
                    let (readme_edits, readme_insights) = Self::analyze_readme(&content);
                    edits.extend(readme_edits);
                    insights.extend(readme_insights);
                }
                break;
            }
        }

        // === 5. Dockerfile 分析 ===
        let docker_path = path.join("Dockerfile");
        if docker_path.exists() {
            tech_stack.push("Docker".to_string());
            edits.push(MicroEdit::AdjustDimension("domain_specificity".to_string(), 0.06));
        }

        // === 6. .github/workflows 分析 ===
        let gh_actions = path.join(".github").join("workflows");
        if gh_actions.exists() {
            tech_stack.push("GitHub-Actions".to_string());
            edits.push(MicroEdit::AdjustDimension("verification".to_string(), 0.07));
        }

        edits.push(MicroEdit::NormalizeVector);

        // 生成能力向量
        let mut cv = CapabilityVector::default();
        for edit in &edits {
            if let MicroEdit::AdjustDimension(name, delta) = edit {
                if let Some(idx) = CapabilityVector::index_from_name(name) {
                    cv.arr_mut()[idx] = (*delta).min(1.0);
                }
            }
        }
        cv.normalize();
        cv.set_provenance(source_url.to_string());

        let confidence = 0.6 + (tech_stack.len() as f64) * 0.05;

        Ok(MinedKnowledge {
            source_url: source_url.to_string(),
            source_name: name,
            domain,
            capability_vector: cv,
            confidence: confidence.min(0.95),
            micro_edits: edits,
            tech_stack,
            insights,
        })
    }

    /// README 分析
    fn analyze_readme(content: &str) -> (Vec<MicroEdit>, Vec<String>) {
        let mut edits = Vec::new();
        let mut insights = Vec::new();
        let lower = content.to_lowercase();

        // 框架/库特征检测
        if lower.contains("machine learning") || lower.contains("deep learning") || lower.contains("neural") {
            edits.push(MicroEdit::AdjustDimension("domain_specificity".to_string(), 0.10));
            edits.push(MicroEdit::AdjustDimension("inference_depth".to_string(), 0.07));
            insights.push("ML/DL 框架: 增强领域专精度".to_string());
        }
        if lower.contains("api") || lower.contains("rest") || lower.contains("grpc") {
            edits.push(MicroEdit::AdjustDimension("compound_composition".to_string(), 0.06));
            insights.push("API 设计: 增强复合组合能力".to_string());
        }
        if lower.contains("database") || lower.contains("sql") || lower.contains("nosql") || lower.contains("storage") {
            edits.push(MicroEdit::AdjustDimension("synthesis".to_string(), 0.05));
            insights.push("数据存储: 增强综合分析能力".to_string());
        }
        if lower.contains("nt_shield") || lower.contains("auth") || lower.contains("encryption") || lower.contains("oauth") {
            edits.push(MicroEdit::AdjustDimension("quality_gates".to_string(), 0.08));
            edits.push(MicroEdit::AdjustDimension("verification".to_string(), 0.07));
            insights.push("安全认证: 增强质量门控和验证能力".to_string());
        }
        if lower.contains("mobile") || lower.contains("ios") || lower.contains("android") || lower.contains("swift") {
            edits.push(MicroEdit::AdjustDimension("domain_specificity".to_string(), 0.09));
            insights.push("移动开发: 增强领域专精度".to_string());
        }
        if lower.contains("testing") || lower.contains("test") || lower.contains("ci/cd") || lower.contains("continuous") {
            edits.push(MicroEdit::AdjustDimension("verification".to_string(), 0.06));
            insights.push("测试/CI: 增强验证能力".to_string());
        }
        if lower.contains("design system") || lower.contains("design tokens") || lower.contains("component library") {
            edits.push(MicroEdit::AdjustDimension("compound_composition".to_string(), 0.08));
            edits.push(MicroEdit::AdjustDimension("semantic_layer".to_string(), 0.06));
            insights.push("设计系统: 增强复合组件和语义层能力".to_string());
        }

        (edits, insights)
    }

    fn apply_edit_to_brain(brain: &mut ReasoningBrain, edit: &MicroEdit) {
        match edit {
            MicroEdit::AdjustDimension(dim, delta) => {
                if let Some(idx) = CapabilityVector::index_from_name(dim) {
                    let val = &mut brain.capability.arr_mut()[idx];
                    *val = (*val + delta).clamp(0.0, 1.0);
                }
            }
            MicroEdit::NormalizeVector => {
                brain.capability.normalize();
            }
            _ => {}
        }
    }

    pub fn domain_to_task_type(domain: &str) -> TaskType {
        match domain {
            "frontend" | "ui" => TaskType::UIDesign,
            "backend" | "api" => TaskType::CodeGeneration,
            "ai-ml" | "ml" => TaskType::CodeAnalysis,
            "mobile" => TaskType::UIDesign,
            "devops" | "infra" => TaskType::Planning,
            "nt_shield" => TaskType::Security,
            _ => TaskType::General,
        }
    }

    /// 获取已挖掘来源的统计信息
    pub fn stats(&self) -> KnowledgeMinerStats {
        KnowledgeMinerStats {
            total_enqueued: self.discovery_queue.len() + self.mined_sources.len(),
            mined_count: self.mined_sources.len(),
            pending_count: self.discovery_queue.len(),
            sources: self.mined_sources.iter()
                .map(|(url, name)| format!("{} ({})", name, url))
                .collect(),
        }
    }

    /// 将知识挖掘结果导出为可读报告
    pub fn generate_report(&self, results: &[MinedKnowledge]) -> String {
        if results.is_empty() {
            return "  没有新的知识挖掘结果".to_string();
        }
        let mut report = String::new();
        report.push_str(&format!("知识挖掘报告 ({} 个新来源):\n", results.len()));
        for (i, r) in results.iter().enumerate() {
            report.push_str(&format!("\n  [{}.] {} (领域: {}, 置信度: {:.2})\n", i+1, r.source_name, r.domain, r.confidence));
            report.push_str(&format!("      来源: {}\n", r.source_url));
            report.push_str(&format!("      技术栈: [{}]\n", r.tech_stack.join(", ")));
            report.push_str(&format!("      洞察: [{}]\n", r.insights.join("; ")));
            report.push_str(&format!("      MicroEdits: {:?}\n", r.micro_edits));
        }
        report
    }

    /// 清除待处理队列中已处理的项
    pub fn clean_processed(&mut self) {
        self.discovery_queue.retain(|url| !self.processed.contains(url));
    }
}

#[derive(Debug, Clone)]
pub struct MinedRoundResult {
    pub mined_count: usize,
    pub sources: Vec<MinedKnowledge>,
}

#[derive(Debug, Clone)]
pub struct KnowledgeMinerStats {
    pub total_enqueued: usize,
    pub mined_count: usize,
    pub pending_count: usize,
    pub sources: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_miner_new() {
        let miner = KnowledgeMiner::new(PathBuf::from("/tmp/test"));
        assert_eq!(miner.discovery_queue.len(), 0);
        assert_eq!(miner.mined_sources.len(), 0);
    }

    #[test]
    fn test_enqueue() {
        let mut miner = KnowledgeMiner::new(PathBuf::from("/tmp/test"));
        miner.enqueue("https://github.com/foo/bar");
        assert_eq!(miner.discovery_queue.len(), 1);
        miner.enqueue("https://github.com/foo/bar"); // 去重
        assert_eq!(miner.discovery_queue.len(), 1);
    }

    #[test]
    fn test_enqueue_default_targets() {
        let mut miner = KnowledgeMiner::new(PathBuf::from("/tmp/test"));
        miner.enqueue_default_targets();
        assert!(miner.discovery_queue.len() >= 10);
    }

    #[test]
    fn test_domain_to_task_type() {
        assert_eq!(KnowledgeMiner::domain_to_task_type("frontend"), TaskType::UIDesign);
        assert_eq!(KnowledgeMiner::domain_to_task_type("backend"), TaskType::CodeGeneration);
        assert_eq!(KnowledgeMiner::domain_to_task_type("ai-ml"), TaskType::CodeAnalysis);
        assert_eq!(KnowledgeMiner::domain_to_task_type("general"), TaskType::General);
    }

    #[test]
    fn test_analyze_readme_ml() {
        let content = "This is a machine learning framework for deep neural networks";
        let (edits, insights) = KnowledgeMiner::analyze_readme(content);
        assert!(!edits.is_empty(), "ML content should generate edits");
        assert!(insights.iter().any(|i| i.contains("ML/DL")));
    }

    #[test]
    fn test_analyze_readme_nt_shield() {
        let content = "Security authentication with OAuth2 and encryption";
        let (edits, insights) = KnowledgeMiner::analyze_readme(content);
        assert!(!edits.is_empty());
        assert!(insights.iter().any(|i| i.contains("安全")));
    }

    #[test]
    fn test_generate_report_empty() {
        let miner = KnowledgeMiner::new(PathBuf::from("/tmp/test"));
        let report = miner.generate_report(&[]);
        assert!(report.contains("没有新的知识挖掘结果"));
    }

    #[test]
    fn test_generate_report_nonempty() {
        let miner = KnowledgeMiner::new(PathBuf::from("/tmp/test"));
        let results = vec![
            MinedKnowledge {
                source_url: "https://github.com/test/repo".to_string(),
                source_name: "repo".to_string(),
                domain: "backend".to_string(),
                capability_vector: CapabilityVector::default(),
                confidence: 0.85,
                micro_edits: vec![],
                tech_stack: vec!["Rust".to_string(), "tokio".to_string()],
                insights: vec!["增强领域专精度".to_string()],
            }
        ];
        let report = miner.generate_report(&results);
        assert!(report.contains("backend"));
        assert!(report.contains("0.85"));
    }

    #[test]
    fn test_clean_processed() {
        let mut miner = KnowledgeMiner::new(PathBuf::from("/tmp/test"));
        let url = "https://github.com/foo/bar";
        miner.enqueue(url);
        miner.processed.insert(url.to_string());
        miner.clean_processed();
        assert_eq!(miner.discovery_queue.len(), 0);
    }

    #[test]
    fn test_stats() {
        let mut miner = KnowledgeMiner::new(PathBuf::from("/tmp/test"));
        miner.enqueue("https://github.com/a/b");
        miner.mined_sources.insert("u1".to_string(), "s1".to_string());
        let stats = miner.stats();
        assert_eq!(stats.total_enqueued, 2);
        assert_eq!(stats.mined_count, 1);
    }
}
