use serde::{Deserialize, Serialize};

// D3 架构倒置: NodeType / RelationType 下沉至 core (nt_core_kb_types), 此处
// re-export 保持 `nt_memory_types::NodeType/RelationType` 调用方路径不变。
// 单一事实源在 core; 其余 KB 领域类型 (KnowledgeNode/Edge/...) 保留本模块。
pub use crate::core::nt_core_kb_types::{NodeType, RelationType};

/// Permission-aware retrieval level (P0-2, Cycle 159). Maps to the caller's
/// clearance: Public < Internal < Confidential < Secret. Each level can access
/// all nodes at or below its sensitivity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum PermissionLevel {
    Public,
    Internal,
    #[default]
    Confidential,
    Secret,
}

impl PermissionLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Internal => "internal",
            Self::Confidential => "confidential",
            Self::Secret => "secret",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "public" => Self::Public,
            "internal" => Self::Internal,
            "secret" => Self::Secret,
            _ => Self::Confidential,
        }
    }
}

/// Sensitivity classification derived from NodeType — no DB schema change needed.
/// Public node types are safe to share; sensitive types carry provenance data
/// (thinking traces, self-test failures, detection findings, goal results).
pub fn node_sensitivity(node_type: &NodeType) -> PermissionLevel {
    match node_type {
        NodeType::ThinkingTrace
        | NodeType::SelfTestFailure
        | NodeType::DetectionFinding
        | NodeType::GoalResult
        | NodeType::ConversationEvolution
        | NodeType::HarnessProfile => PermissionLevel::Secret,
        NodeType::EventRecord | NodeType::Session | NodeType::EvolutionPattern => PermissionLevel::Internal,
        _ => PermissionLevel::Public,
    }
}

/// Temporal validity window for fact accuracy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalValidity {
    /// When this fact became valid (Unix timestamp)
    pub valid_from: i64,
    /// When this fact expires (None = no expiry)
    pub valid_until: Option<i64>,
    /// Confidence in temporal bounds (0.0-1.0)
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub id: String,
    pub node_type: NodeType,
    pub title: String,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub language: String,
    pub confidence: f64,
    pub importance: f64,
    pub created_at: i64,
    pub updated_at: i64,
    pub access_count: i64,
    pub metadata: Option<serde_json::Value>,
    /// Temporal validity window for fact accuracy
    pub temporal: Option<TemporalValidity>,
    /// UUID of the node this node supersedes (for fact versioning)
    pub supersedes: Option<String>,
    /// Episode provenance tracking
    pub source_episode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation_type: RelationType,
    pub weight: f64,
    pub description: Option<String>,
    pub created_at: i64,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub node_type: Option<NodeType>,
    pub domain: Option<String>,
    pub min_importance: Option<f64>,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub node: KnowledgeNode,
    pub score: f64,
    pub matched_on: Vec<SearchMatchType>,
    pub signals: Option<[f64; 4]>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SearchMatchType {
    FtsTitle,
    FtsSummary,
    FtsContent,
    GraphRelation,
    VectorSimilarity,
    TagExact,
    Bm25,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPath {
    pub nodes: Vec<KnowledgeNode>,
    pub edges: Vec<KnowledgeEdge>,
    pub total_distance: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeStats {
    pub total_nodes: i64,
    pub total_edges: i64,
    pub by_type: Vec<(String, i64)>,
    pub by_domain: Vec<(String, i64)>,
    pub crawl_pending: i64,
    pub crawl_completed: i64,
    pub db_size_bytes: i64,
}

/// ConversationRecord — 外部对话进化训练数据
/// 每次 user ↔ LLM 交互的完整记录，供意识核自我进化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationRecord {
    pub id: String,
    pub session_id: String,
    pub task_description: String,
    pub user_intent: String,
    pub strategy_used: String,
    pub e8_mode: String,
    pub specialist_winner: String,
    pub actions_taken: Vec<String>,
    pub obstacles_encountered: Vec<String>,
    pub fix_patterns: Vec<String>,
    pub outcome: String,
    pub effectiveness: f64,
    pub reasoning_iterations: u32,
    pub error_count: u32,
    pub timestamp: i64,
}

/// EvolutionRecord — 从对话提炼的进化知识
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionRecord {
    pub id: String,
    pub source_conversation_id: String,
    pub pattern_type: EvolutionPatternType,
    pub description: String,
    pub before_behavior: String,
    pub after_behavior: String,
    pub effectiveness_gain: f64,
    pub applied_to: Vec<String>,
    pub verified: bool,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EvolutionPatternType {
    /// 重复出现的错误模式
    RecurringError,
    /// 沟通方式优化
    CommunicationOptimization,
    /// 问题分解策略
    ProblemDecomposition,
    /// 验证流程改进
    VerificationImprovement,
    /// 工具使用模式
    ToolUsagePattern,
    /// 新策略发现
    StrategyDiscovery,
    /// 行为准则更新
    PrincipleUpdate,
}

/// ProceduralMemoryRecord — captured successful E8 pattern sequence as a reusable skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralMemoryRecord {
    pub id: String,
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub e8_sequence: Vec<u8>,
    pub trigger_pattern: Vec<u8>,
    pub success_rate: f64,
    pub execution_count: u64,
    pub avg_reward: f64,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveSearchResult {
    pub query: String,
    pub complexity: String,
    pub confidence: f64,
    pub action: String,
    pub results: Vec<SearchResult>,
    pub graded: Vec<GradedDocument>,
}

/// Serializable snapshot of a graded document for the AdaptiveSearchResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradedDocument {
    pub node_id: String,
    pub relevance: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlQueueItem {
    pub id: String,
    pub url: String,
    pub depth: i64,
    pub domain: Option<String>,
    pub priority: i64,
    pub status: String,
    pub discovered_at: i64,
    pub last_attempt: Option<i64>,
    pub retry_count: i64,
    pub error_message: Option<String>,
}

/// Normalize type strings: PascalCase → snake_case (also passes snake_case through).
/// Handles both `EvolutionPattern` → `evolution_pattern` and `resource_for` → `resource_for`.
/// (实现已随 NodeType/RelationType 下沉至 core `nt_core_kb_types`。)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert!(true);
    }

    #[test]
    fn test_node_type_pascal_cases() {
        assert_eq!(NodeType::from_str("EvolutionPattern"), NodeType::EvolutionPattern);
        assert_eq!(NodeType::from_str("ConversationEvolution"), NodeType::ConversationEvolution);
        assert_eq!(NodeType::from_str("Textbook"), NodeType::Textbook);
        assert_eq!(NodeType::from_str("Resource"), NodeType::Resource);
        assert_eq!(NodeType::from_str("External"), NodeType::External);
        assert_eq!(NodeType::from_str("Summary"), NodeType::Summary);
        assert_eq!(NodeType::from_str("Guide"), NodeType::Guide);
        assert_eq!(NodeType::from_str("Skill"), NodeType::Skill);
        assert_eq!(NodeType::from_str("Reference"), NodeType::Reference);
    }

    #[test]
    fn test_node_type_snake_cases() {
        assert_eq!(NodeType::from_str("evolution_pattern"), NodeType::EvolutionPattern);
        assert_eq!(NodeType::from_str("conversation_evolution"), NodeType::ConversationEvolution);
        assert_eq!(NodeType::from_str("textbook"), NodeType::Textbook);
        assert_eq!(NodeType::from_str("concept"), NodeType::Concept);
    }

    #[test]
    fn test_relation_type_pascal_cases() {
        assert_eq!(RelationType::from_str("RelatedTo"), RelationType::RelatedTo);
        assert_eq!(RelationType::from_str("EvolvedFrom"), RelationType::EvolvedFrom);
        assert_eq!(RelationType::from_str("ResourceFor"), RelationType::ResourceFor);
        assert_eq!(RelationType::from_str("AboutTopic"), RelationType::AboutTopic);
        assert_eq!(RelationType::from_str("BelongsTo"), RelationType::BelongsTo);
        assert_eq!(RelationType::from_str("SubTopicOf"), RelationType::SubTopicOf);
        assert_eq!(RelationType::from_str("CrossDomain"), RelationType::CrossDomain);
        assert_eq!(RelationType::from_str("ArchPartOf"), RelationType::ArchPartOf);
        assert_eq!(RelationType::from_str("InsightAbout"), RelationType::InsightAbout);
        assert_eq!(RelationType::from_str("BrandFor"), RelationType::BrandFor);
        assert_eq!(RelationType::from_str("Illustrates"), RelationType::Illustrates);
    }

    #[test]
    fn test_relation_type_snake_cases() {
        assert_eq!(RelationType::from_str("related_to"), RelationType::RelatedTo);
        assert_eq!(RelationType::from_str("evolved_from"), RelationType::EvolvedFrom);
        assert_eq!(RelationType::from_str("resource_for"), RelationType::ResourceFor);
        assert_eq!(RelationType::from_str("references"), RelationType::References);
        assert_eq!(RelationType::from_str("brand_for"), RelationType::BrandFor);
    }

    #[test]
    fn test_as_str_roundtrip() {
        for variant in [
            NodeType::Concept, NodeType::Paper, NodeType::Repository,
            NodeType::EvolutionPattern, NodeType::ConversationEvolution,
            NodeType::Textbook, NodeType::Resource, NodeType::External,
            NodeType::Summary, NodeType::Guide, NodeType::Skill, NodeType::Reference,
        ] {
            let s = variant.as_str();
            let back = NodeType::from_str(s);
            assert_eq!(variant, back, "roundtrip failed for {:?} -> {} -> {:?}", variant, s, back);
        }
    }

    #[test]
    fn test_relation_as_str_roundtrip() {
        for variant in [
            RelationType::References, RelationType::RelatedTo, RelationType::EvolvedFrom,
            RelationType::ResourceFor, RelationType::AboutTopic, RelationType::BelongsTo,
            RelationType::SubTopicOf, RelationType::CrossDomain, RelationType::Contains,
            RelationType::Influenced, RelationType::ArchPartOf, RelationType::Categorized,
            RelationType::RelatesTo, RelationType::InsightAbout, RelationType::Implements,
            RelationType::Uses, RelationType::Visualizes, RelationType::BrandFor,
            RelationType::Illustrates,
        ] {
            let s = variant.as_str();
            let back = RelationType::from_str(s);
            assert_eq!(variant, back, "roundtrip failed for {:?} -> {} -> {:?}", variant, s, back);
        }
    }

    #[test]
    fn test_fallback() {
        assert_eq!(NodeType::from_str("nonexistent_type_xyz"), NodeType::Concept);
        assert_eq!(RelationType::from_str("nonexistent_type_xyz"), RelationType::Related);
    }
}
