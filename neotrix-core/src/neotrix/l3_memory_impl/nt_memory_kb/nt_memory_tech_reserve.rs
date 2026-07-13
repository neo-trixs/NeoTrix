use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::nt_memory_types::*;

// ---------------------------------------------------------------------------
// 四维技术储备 (4D Technical Reserve)
// ---------------------------------------------------------------------------

/// 技术储备四维度分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TechReserveDimension {
    /// D1: 技术原理 — Algorithm, Theory, Method, Concept
    TechnicalPrinciples,
    /// D2: 产品生态 — Tool, Framework, Dataset, Repository (含版本追踪)
    ProductEcosystem,
    /// D3: 代码资产 — CodeSnippet, code patterns, API signatures
    CodeAssets,
    /// D4: 架构参考 — Guide, Textbook, design patterns, best practices
    ArchitectureReference,
}

impl TechReserveDimension {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TechnicalPrinciples => "technical_principles",
            Self::ProductEcosystem => "product_ecosystem",
            Self::CodeAssets => "code_assets",
            Self::ArchitectureReference => "architecture_reference",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "technical_principles" | "D1" | "d1" => Some(Self::TechnicalPrinciples),
            "product_ecosystem" | "D2" | "d2" => Some(Self::ProductEcosystem),
            "code_assets" | "D3" | "d3" => Some(Self::CodeAssets),
            "architecture_reference" | "D4" | "d4" => Some(Self::ArchitectureReference),
            _ => None,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::TechnicalPrinciples => "技术原理 — 算法、理论、方法、核心概念",
            Self::ProductEcosystem => "产品生态 — 成熟工具、框架、库、平台（含版本追踪）",
            Self::CodeAssets => "代码资产 — 代码片段、API 签名、SDK 用法模式",
            Self::ArchitectureReference => "架构参考 — 设计模式、集成方案、最佳实践、指南",
        }
    }

    /// 从 NodeType 自动分类到维度
    pub fn from_node_type(nt: &NodeType) -> Option<Self> {
        match nt {
            NodeType::Algorithm | NodeType::Theory | NodeType::Method | NodeType::Concept => {
                Some(Self::TechnicalPrinciples)
            }
            NodeType::Tool | NodeType::Framework | NodeType::Dataset | NodeType::Repository
            | NodeType::Benchmark | NodeType::Organization => {
                Some(Self::ProductEcosystem)
            }
            NodeType::CodeSnippet | NodeType::Reference | NodeType::Resource => {
                Some(Self::CodeAssets)
            }
            NodeType::Guide | NodeType::Textbook | NodeType::Insight | NodeType::Summary => {
                Some(Self::ArchitectureReference)
            }
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// 技术储备条目
// ---------------------------------------------------------------------------

/// 一条技术储备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechReserveEntry {
    /// 知识库节点 ID
    pub node_id: String,
    /// 标题
    pub title: String,
    /// 所属维度
    pub dimension: TechReserveDimension,
    /// 技术领域标签 (如 "llm", "database", "frontend", "crawler")
    pub domain_tags: Vec<String>,
    /// 成熟度评分 0.0-1.0 (基于 stars, 引用数等)
    pub maturity: f64,
    /// 最新版本号 (产品生态维度专用)
    pub latest_version: Option<String>,
    /// 更新时间戳
    pub updated_at: i64,
    /// 外部 URL
    pub url: Option<String>,
    /// 摘要
    pub summary: Option<String>,
}

/// 技术储备查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechReserveQuery {
    /// 搜索关键词
    pub query: String,
    /// 限定维度 (None = 全部)
    pub dimension: Option<TechReserveDimension>,
    /// 限定技术领域
    pub domain: Option<String>,
    /// 最低成熟度
    pub min_maturity: Option<f64>,
    /// 最大返回数
    pub top_k: usize,
}

/// 架构差距分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureGap {
    /// 缺失的技术领域
    pub domain: String,
    /// 缺失的维度
    pub dimension: TechReserveDimension,
    /// 差距严重程度 0.0-1.0
    pub severity: f64,
    /// 建议优先处理
    pub priority: u8,
    /// 描述
    pub description: String,
    /// 相关的已知成熟产品/方案
    pub known_solutions: Vec<String>,
    /// 建议的行动
    pub suggested_action: String,
}

// ---------------------------------------------------------------------------
// TechReserveStore — 四维技术储备存储与查询
// ---------------------------------------------------------------------------

/// 技术储备存储 — 四维分类 + 版本追踪 + 架构差距分析
pub struct TechReserveStore {
    entries: Vec<TechReserveEntry>,
    /// 二级索引: 领域标签 → 条目索引
    domain_index: HashMap<String, Vec<usize>>,
    /// 二级索引: 维度 → 条目索引
    dimension_index: HashMap<TechReserveDimension, Vec<usize>>,
}

impl Default for TechReserveStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TechReserveStore {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            domain_index: HashMap::new(),
            dimension_index: HashMap::new(),
        }
    }

    /// 从 KB 节点重建索引
    pub fn rebuild_from_nodes(&mut self, nodes: &[KnowledgeNode]) {
        self.entries.clear();
        self.domain_index.clear();
        self.dimension_index.clear();

        for node in nodes {
            let Some(dimension) = TechReserveDimension::from_node_type(&node.node_type) else {
                continue;
            };

            let metadata = node.metadata.as_ref();
            let domain_tags: Vec<String> = metadata
                .and_then(|m| m.get("tags"))
                .and_then(|t| t.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            let maturity = metadata
                .and_then(|m| m.get("maturity"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5);

            let latest_version = metadata
                .and_then(|m| m.get("latest_version"))
                .and_then(|v| v.as_str().map(String::from));

            let entry = TechReserveEntry {
                node_id: node.id.clone(),
                title: node.title.clone(),
                dimension,
                domain_tags: domain_tags.clone(),
                maturity,
                latest_version,
                updated_at: node.updated_at,
                url: node.url.clone(),
                summary: node.summary.clone(),
            };

            let idx = self.entries.len();
            self.entries.push(entry);

            for tag in &domain_tags {
                self.domain_index.entry(tag.clone()).or_default().push(idx);
            }
            self.dimension_index.entry(dimension).or_default().push(idx);
        }
    }

    /// 从 entries 重建索引 (内部变更后用)
    fn rebuild_index(&mut self) {
        self.domain_index.clear();
        self.dimension_index.clear();
        for (idx, entry) in self.entries.iter().enumerate() {
            for tag in &entry.domain_tags {
                self.domain_index.entry(tag.clone()).or_default().push(idx);
            }
            self.dimension_index.entry(entry.dimension).or_default().push(idx);
        }
    }

    /// 添加一条技术储备
    pub fn add_entry(&mut self, entry: TechReserveEntry) {
        let idx = self.entries.len();
        self.entries.push(entry);
        let entry = &self.entries[idx];
        for tag in &entry.domain_tags {
            self.domain_index.entry(tag.clone()).or_default().push(idx);
        }
        self.dimension_index.entry(entry.dimension).or_default().push(idx);
    }

    /// 查询技术储备
    pub fn query(&self, q: &TechReserveQuery) -> Vec<&TechReserveEntry> {
        let query_lower = q.query.to_lowercase();
        let mut results: Vec<&TechReserveEntry> = self
            .entries
            .iter()
            .filter(|e| {
                if let Some(dim) = &q.dimension {
                    if e.dimension != *dim {
                        return false;
                    }
                }
                if let Some(domain) = &q.domain {
                    if !e.domain_tags.iter().any(|t| t.eq_ignore_ascii_case(domain)) {
                        return false;
                    }
                }
                if let Some(min_m) = q.min_maturity {
                    if e.maturity < min_m {
                        return false;
                    }
                }
                if !q.query.is_empty() {
                    let title_match = e.title.to_lowercase().contains(&query_lower);
                    let tag_match = e.domain_tags.iter().any(|t| t.to_lowercase().contains(&query_lower));
                    let summary_match = e
                        .summary
                        .as_ref()
                        .map_or(false, |s| s.to_lowercase().contains(&query_lower));
                    title_match || tag_match || summary_match
                } else {
                    true
                }
            })
            .collect();

        // 按成熟度降序排列
        results.sort_by(|a, b| b.maturity.partial_cmp(&a.maturity).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(q.top_k.max(1));
        results
    }

    /// 获取指定技术领域的最新成熟产品信息
    /// 返回成熟度最高的 N 个产品/框架/工具
    pub fn latest_mature_products(&self, domain: &str, top_k: usize) -> Vec<&TechReserveEntry> {
        self.query(&TechReserveQuery {
            query: String::new(),
            dimension: Some(TechReserveDimension::ProductEcosystem),
            domain: Some(domain.to_string()),
            min_maturity: Some(0.5),
            top_k,
        })
    }

    /// 获取指定技术的完整四维信息
    pub fn full_tech_profile(&self, tech_name: &str) -> TechProfile {
        let q = &TechReserveQuery {
            query: tech_name.to_string(),
            dimension: None,
            domain: None,
            min_maturity: None,
            top_k: 20,
        };
        let results = self.query(q);

        TechProfile {
            tech_name: tech_name.to_string(),
            total_entries: results.len(),
            principles: results.iter().filter(|e| e.dimension == TechReserveDimension::TechnicalPrinciples).map(|e| (*e).clone()).collect(),
            products: results.iter().filter(|e| e.dimension == TechReserveDimension::ProductEcosystem).map(|e| (*e).clone()).collect(),
            code_assets: results.iter().filter(|e| e.dimension == TechReserveDimension::CodeAssets).map(|e| (*e).clone()).collect(),
            architecture_refs: results.iter().filter(|e| e.dimension == TechReserveDimension::ArchitectureReference).map(|e| (*e).clone()).collect(),
        }
    }

    /// 架构差距分析：检查缺失的技术维度
    pub fn analyze_gaps(&self, required_domains: &[&str]) -> Vec<ArchitectureGap> {
        let mut gaps = Vec::new();

        for domain in required_domains {
            let domain_lower = domain.to_lowercase();

            // 检查 D1: 技术原理
            let has_principles = self.entries.iter().any(|e| {
                e.dimension == TechReserveDimension::TechnicalPrinciples
                    && e.domain_tags.iter().any(|t| t.to_lowercase() == domain_lower)
            });
            if !has_principles {
                gaps.push(ArchitectureGap {
                    domain: domain.to_string(),
                    dimension: TechReserveDimension::TechnicalPrinciples,
                    severity: 0.8,
                    priority: 1,
                    description: format!("缺乏「{}」领域的技术原理知识", domain),
                    known_solutions: vec![],
                    suggested_action: format!("搜索 {} 的核心论文、算法文档和理论资料并入库", domain),
                });
            }

            // 检查 D2: 产品生态
            let has_products = self.entries.iter().any(|e| {
                e.dimension == TechReserveDimension::ProductEcosystem
                    && e.domain_tags.iter().any(|t| t.to_lowercase() == domain_lower)
            });
            if !has_products {
                gaps.push(ArchitectureGap {
                    domain: domain.to_string(),
                    dimension: TechReserveDimension::ProductEcosystem,
                    severity: 0.7,
                    priority: 2,
                    description: format!("缺乏「{}」领域的产品生态信息", domain),
                    known_solutions: vec![],
                    suggested_action: format!("扫描 GitHub topics 和产品目录，收集 {} 领域的工具/框架", domain),
                });
            }

            // 检查 D3: 代码资产
            let has_code = self.entries.iter().any(|e| {
                e.dimension == TechReserveDimension::CodeAssets
                    && e.domain_tags.iter().any(|t| t.to_lowercase() == domain_lower)
            });
            if !has_code {
                gaps.push(ArchitectureGap {
                    domain: domain.to_string(),
                    dimension: TechReserveDimension::CodeAssets,
                    severity: 0.6,
                    priority: 3,
                    description: format!("缺乏「{}」领域的代码资产", domain),
                    known_solutions: vec![],
                    suggested_action: format!("采集 {} 领域的 SDK 示例、API 用法和代码片段", domain),
                });
            }

            // 检查 D4: 架构参考
            let has_arch = self.entries.iter().any(|e| {
                e.dimension == TechReserveDimension::ArchitectureReference
                    && e.domain_tags.iter().any(|t| t.to_lowercase() == domain_lower)
            });
            if !has_arch {
                gaps.push(ArchitectureGap {
                    domain: domain.to_string(),
                    dimension: TechReserveDimension::ArchitectureReference,
                    severity: 0.5,
                    priority: 4,
                    description: format!("缺乏「{}」领域的架构参考", domain),
                    known_solutions: vec![],
                    suggested_action: format!("收集 {} 领域的设计文档、架构决策记录和最佳实践指南", domain),
                });
            }
        }

        gaps.sort_by(|a, b| a.priority.cmp(&b.priority));
        gaps
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn all_entries(&self) -> &[TechReserveEntry] {
        &self.entries
    }

    /// 按维度统计条目数
    pub fn stats_by_dimension(&self) -> HashMap<TechReserveDimension, usize> {
        let mut stats = HashMap::new();
        for entry in &self.entries {
            *stats.entry(entry.dimension).or_insert(0) += 1;
        }
        stats
    }
}

/// 完整技术档案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechProfile {
    pub tech_name: String,
    pub total_entries: usize,
    pub principles: Vec<TechReserveEntry>,
    pub products: Vec<TechReserveEntry>,
    pub code_assets: Vec<TechReserveEntry>,
    pub architecture_refs: Vec<TechReserveEntry>,
}

// ---------------------------------------------------------------------------
// 技术标签分类器 — 自动从文本提取技术领域标签
// ---------------------------------------------------------------------------

/// 常见技术领域的关键词模式
const TECH_DOMAIN_PATTERNS: &[(&str, &[&str])] = &[
    ("llm", &["large language model", "llm", "transformer", "gpt", "language model"]),
    ("rag", &["rag", "retrieval augmented", "vector search", "hybrid search"]),
    ("agent", &["agent", "autonomous", "tool use", "ai agent"]),
    ("crawler", &["crawler", "scraper", "web crawl", "html parse", "spider"]),
    ("database", &["database", "sqlite", "postgresql", "dbms", "nosql", "vector db"]),
    ("frontend", &["react", "vue", "web app", "ui", "frontend", "component"]),
    ("backend", &["api server", "backend", "rest api", "microservice"]),
    ("ml", &["machine learning", "deep learning", "neural network", "pytorch", "tensorflow"]),
    ("search", &["search", "information retrieval", "index", "bm25", "fts"]),
    ("security", &["security", "authentication", "oauth", "encryption", "vulnerability"]),
    ("devops", &["devops", "ci/cd", "deploy", "kubernetes", "docker"]),
    ("rust", &["rust", "cargo", "rustc", "unsafe rust"]),
    ("python", &["python", "pypi", "pip", "python3"]),
    ("javascript", &["javascript", "typescript", "node.js", "npm"]),
    ("data", &["data pipeline", "etl", "data processing", "stream", "batch"]),
    ("embedding", &["embedding", "vector", "semantic search", "embed", "bert"]),
    ("mcp", &["mcp", "model context protocol", "tool server", "json-rpc"]),
    ("knowledge_graph", &["knowledge graph", "graph rag", "neo4j", "graph db"]),
    ("multi_modal", &["multimodal", "vision", "speech", "audio", "image gen"]),
    ("fine_tuning", &["fine tune", "lora", "qlora", "sft", "rlhf"]),
];

/// 从文本中提取技术领域标签
pub fn extract_tech_domains(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    TECH_DOMAIN_PATTERNS
        .iter()
        .filter(|(_, keywords)| keywords.iter().any(|kw| lower.contains(kw)))
        .map(|(domain, _)| domain.to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(node_type: NodeType, title: &str, tags: &[&str], maturity: f64) -> KnowledgeNode {
        let metadata = serde_json::json!({
            "tags": tags,
            "maturity": maturity,
        });
        KnowledgeNode {
            id: format!("test-{}", title),
            node_type,
            title: title.to_string(),
            summary: Some(format!("Summary of {}", title)),
            content: Some(format!("Content of {}", title)),
            url: Some(format!("https://example.com/{}", title)),
            domain: Some("test".into()),
            language: "en".into(),
            confidence: 0.8,
            importance: 0.5,
            created_at: 1000000,
            updated_at: 1000000,
            access_count: 0,
            metadata: Some(metadata),
            temporal: None,
            supersedes: None,
            source_episode: None,
        }
    }

    #[test]
    fn test_dimension_from_node_type() {
        assert_eq!(
            TechReserveDimension::from_node_type(&NodeType::Algorithm),
            Some(TechReserveDimension::TechnicalPrinciples)
        );
        assert_eq!(
            TechReserveDimension::from_node_type(&NodeType::Tool),
            Some(TechReserveDimension::ProductEcosystem)
        );
        assert_eq!(
            TechReserveDimension::from_node_type(&NodeType::CodeSnippet),
            Some(TechReserveDimension::CodeAssets)
        );
        assert_eq!(
            TechReserveDimension::from_node_type(&NodeType::Guide),
            Some(TechReserveDimension::ArchitectureReference)
        );
    }

    #[test]
    fn test_rebuild_from_nodes() {
        let nodes = vec![
            make_node(NodeType::Algorithm, "Transformer", &["llm", "ml"], 0.9),
            make_node(NodeType::Tool, "LangChain", &["llm", "agent"], 0.8),
            make_node(NodeType::Guide, "RAG Best Practices", &["rag", "llm"], 0.7),
            make_node(NodeType::CodeSnippet, "Python Async", &["python"], 0.5),
            make_node(NodeType::Person, "Not a tech", &[], 0.3),
        ];

        let mut store = TechReserveStore::new();
        store.rebuild_from_nodes(&nodes);

        assert_eq!(store.entry_count(), 4); // Person 被过滤
        let stats = store.stats_by_dimension();
        assert_eq!(*stats.get(&TechReserveDimension::TechnicalPrinciples).unwrap(), 1);
        assert_eq!(*stats.get(&TechReserveDimension::ProductEcosystem).unwrap(), 1);
        assert_eq!(*stats.get(&TechReserveDimension::CodeAssets).unwrap(), 1);
        assert_eq!(*stats.get(&TechReserveDimension::ArchitectureReference).unwrap(), 1);
    }

    #[test]
    fn test_query_by_dimension() {
        let mut store = TechReserveStore::new();
        store.add_entry(TechReserveEntry {
            node_id: "1".into(), title: "GPT-4".into(),
            dimension: TechReserveDimension::ProductEcosystem,
            domain_tags: vec!["llm".into(), "openai".into()],
            maturity: 0.95, latest_version: Some("gpt-4-turbo".into()),
            updated_at: 1000000, url: None, summary: None,
        });
        store.add_entry(TechReserveEntry {
            node_id: "2".into(), title: "Transformer Paper".into(),
            dimension: TechReserveDimension::TechnicalPrinciples,
            domain_tags: vec!["llm".into()],
            maturity: 0.9, latest_version: None,
            updated_at: 1000000, url: None, summary: None,
        });

        let results = store.query(&TechReserveQuery {
            query: String::new(),
            dimension: Some(TechReserveDimension::ProductEcosystem),
            domain: None, min_maturity: None, top_k: 10,
        });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "GPT-4");
    }

    #[test]
    fn test_latest_mature_products() {
        let mut store = TechReserveStore::new();
        store.add_entry(TechReserveEntry {
            node_id: "a".into(), title: "LangChain".into(),
            dimension: TechReserveDimension::ProductEcosystem,
            domain_tags: vec!["llm".into(), "agent".into()],
            maturity: 0.85, latest_version: Some("0.3.0".into()),
            updated_at: 1000000, url: None, summary: None,
        });
        store.add_entry(TechReserveEntry {
            node_id: "b".into(), title: "AutoGPT".into(),
            dimension: TechReserveDimension::ProductEcosystem,
            domain_tags: vec!["agent".into()],
            maturity: 0.7, latest_version: None,
            updated_at: 1000000, url: None, summary: None,
        });

        let products = store.latest_mature_products("agent", 5);
        assert_eq!(products.len(), 2);
        assert_eq!(products[0].title, "LangChain"); // higher maturity first
    }

    #[test]
    fn test_full_tech_profile() {
        let mut store = TechReserveStore::new();
        store.add_entry(TechReserveEntry {
            node_id: "1".into(), title: "RAG Paper".into(),
            dimension: TechReserveDimension::TechnicalPrinciples,
            domain_tags: vec!["rag".into()],
            maturity: 0.9, latest_version: None,
            updated_at: 1000000, url: None, summary: None,
        });
        store.add_entry(TechReserveEntry {
            node_id: "2".into(), title: "ChromaDB".into(),
            dimension: TechReserveDimension::ProductEcosystem,
            domain_tags: vec!["rag".into(), "vector".into()],
            maturity: 0.8, latest_version: Some("0.5.0".into()),
            updated_at: 1000000, url: None, summary: None,
        });

        let profile = store.full_tech_profile("rag");
        assert_eq!(profile.total_entries, 2);
        assert_eq!(profile.principles.len(), 1);
        assert_eq!(profile.products.len(), 1);
    }

    #[test]
    fn test_architecture_gap_analysis() {
        let mut store = TechReserveStore::new();
        store.add_entry(TechReserveEntry {
            node_id: "1".into(), title: "Transformers".into(),
            dimension: TechReserveDimension::TechnicalPrinciples,
            domain_tags: vec!["llm".into()],
            maturity: 0.9, latest_version: None,
            updated_at: 1000000, url: None, summary: None,
        });

        let required = vec!["llm", "rag", "agent"];
        let gaps = store.analyze_gaps(&required);

        // llm 有 D1, 缺 D2/D3/D4 → 3 gaps
        // rag 全缺 → 4 gaps
        // agent 全缺 → 4 gaps
        // total: 3 + 4 + 4 = 11
        assert_eq!(gaps.len(), 11);

        // llm 的 D1 不应该在 gaps 里
        assert!(!gaps.iter().any(|g| g.domain == "llm" && g.dimension == TechReserveDimension::TechnicalPrinciples));
    }

    #[test]
    fn test_extract_tech_domains() {
        let text = "We use a large language model with RAG for retrieval augmented generation";
        let domains = extract_tech_domains(text);
        assert!(domains.contains(&"llm".to_string()));
        assert!(domains.contains(&"rag".to_string()));
    }

    #[test]
    fn test_empty_store() {
        let store = TechReserveStore::new();
        assert_eq!(store.entry_count(), 0);
        let stats = store.stats_by_dimension();
        assert!(stats.is_empty());
    }

    #[test]
    fn test_dimension_str_roundtrip() {
        for d in &[
            TechReserveDimension::TechnicalPrinciples,
            TechReserveDimension::ProductEcosystem,
            TechReserveDimension::CodeAssets,
            TechReserveDimension::ArchitectureReference,
        ] {
            assert_eq!(TechReserveDimension::from_str(d.as_str()), Some(*d));
        }
    }

    #[test]
    fn test_query_with_min_maturity() {
        let mut store = TechReserveStore::new();
        store.add_entry(TechReserveEntry {
            node_id: "1".into(), title: "Tool A".into(),
            dimension: TechReserveDimension::ProductEcosystem,
            domain_tags: vec!["test".into()],
            maturity: 0.3, latest_version: None,
            updated_at: 1000000, url: None, summary: None,
        });
        store.add_entry(TechReserveEntry {
            node_id: "2".into(), title: "Tool B".into(),
            dimension: TechReserveDimension::ProductEcosystem,
            domain_tags: vec!["test".into()],
            maturity: 0.8, latest_version: None,
            updated_at: 1000000, url: None, summary: None,
        });

        let results = store.query(&TechReserveQuery {
            query: String::new(),
            dimension: Some(TechReserveDimension::ProductEcosystem),
            domain: None, min_maturity: Some(0.6), top_k: 10,
        });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Tool B");
    }

    #[test]
    fn test_search_by_keyword() {
        let mut store = TechReserveStore::new();
        store.add_entry(TechReserveEntry {
            node_id: "1".into(), title: "Fast Search Engine".into(),
            dimension: TechReserveDimension::ProductEcosystem,
            domain_tags: vec!["search".into()],
            maturity: 0.8, latest_version: None,
            updated_at: 1000000, url: None, summary: None,
        });
        store.add_entry(TechReserveEntry {
            node_id: "2".into(), title: "Sorting Algorithm".into(),
            dimension: TechReserveDimension::TechnicalPrinciples,
            domain_tags: vec!["algorithm".into()],
            maturity: 0.9, latest_version: None,
            updated_at: 1000000, url: None, summary: None,
        });

        let results = store.query(&TechReserveQuery {
            query: "search".into(),
            dimension: None, domain: None, min_maturity: None, top_k: 10,
        });
        assert_eq!(results.len(), 1);
    }
}
