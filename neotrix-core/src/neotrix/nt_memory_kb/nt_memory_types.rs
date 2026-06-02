use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    Concept,
    Paper,
    Repository,
    Person,
    Event,
    Source,
    Tool,
    Framework,
    Algorithm,
    Theory,
    Method,
    Dataset,
    Benchmark,
    Organization,
    Book,
    Course,
    Article,
    CodeSnippet,
    Idea,
    Question,
    Insight,
    HarnessProfile,
    Image,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Concept => "concept",
            NodeType::Paper => "paper",
            NodeType::Repository => "repository",
            NodeType::Person => "person",
            NodeType::Event => "event",
            NodeType::Source => "source",
            NodeType::Tool => "tool",
            NodeType::Framework => "framework",
            NodeType::Algorithm => "algorithm",
            NodeType::Theory => "theory",
            NodeType::Method => "method",
            NodeType::Dataset => "dataset",
            NodeType::Benchmark => "benchmark",
            NodeType::Organization => "organization",
            NodeType::Book => "book",
            NodeType::Course => "course",
            NodeType::Article => "article",
            NodeType::CodeSnippet => "code_snippet",
            NodeType::Idea => "idea",
            NodeType::Question => "question",
            NodeType::Insight => "insight",
            NodeType::HarnessProfile => "harness_profile",
            NodeType::Image => "image",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "concept" => NodeType::Concept,
            "paper" => NodeType::Paper,
            "repository" => NodeType::Repository,
            "person" => NodeType::Person,
            "event" => NodeType::Event,
            "source" => NodeType::Source,
            "tool" => NodeType::Tool,
            "framework" => NodeType::Framework,
            "algorithm" => NodeType::Algorithm,
            "theory" => NodeType::Theory,
            "method" => NodeType::Method,
            "dataset" => NodeType::Dataset,
            "benchmark" => NodeType::Benchmark,
            "organization" => NodeType::Organization,
            "book" => NodeType::Book,
            "course" => NodeType::Course,
            "article" => NodeType::Article,
            "code_snippet" => NodeType::CodeSnippet,
            "idea" => NodeType::Idea,
            "question" => NodeType::Question,
            "insight" => NodeType::Insight,
            "harness_profile" => NodeType::HarnessProfile,
            "image" => NodeType::Image,
            _ => NodeType::Concept,
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationType {
    References,
    SubclassOf,
    InstanceOf,
    Causes,
    PrerequisiteOf,
    Contradicts,
    Supports,
    BeforeInTime,
    AfterInTime,
    Related,
    PartOf,
    DevelopedBy,
    ImplementedIn,
    InspiredBy,
    Citation,
    ExtensionOf,
    DependsOn,
    Improves,
    Outperforms,
}

impl RelationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationType::References => "references",
            RelationType::SubclassOf => "subclass_of",
            RelationType::InstanceOf => "instance_of",
            RelationType::Causes => "causes",
            RelationType::PrerequisiteOf => "prerequisite_of",
            RelationType::Contradicts => "contradicts",
            RelationType::Supports => "supports",
            RelationType::BeforeInTime => "before_in_time",
            RelationType::AfterInTime => "after_in_time",
            RelationType::Related => "related",
            RelationType::PartOf => "part_of",
            RelationType::DevelopedBy => "developed_by",
            RelationType::ImplementedIn => "implemented_in",
            RelationType::InspiredBy => "inspired_by",
            RelationType::Citation => "citation",
            RelationType::ExtensionOf => "extension_of",
            RelationType::DependsOn => "depends_on",
            RelationType::Improves => "improves",
            RelationType::Outperforms => "outperforms",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "references" => RelationType::References,
            "subclass_of" => RelationType::SubclassOf,
            "instance_of" => RelationType::InstanceOf,
            "causes" => RelationType::Causes,
            "prerequisite_of" => RelationType::PrerequisiteOf,
            "contradicts" => RelationType::Contradicts,
            "supports" => RelationType::Supports,
            "before_in_time" => RelationType::BeforeInTime,
            "after_in_time" => RelationType::AfterInTime,
            "related" => RelationType::Related,
            "part_of" => RelationType::PartOf,
            "developed_by" => RelationType::DevelopedBy,
            "implemented_in" => RelationType::ImplementedIn,
            "inspired_by" => RelationType::InspiredBy,
            "citation" => RelationType::Citation,
            "extension_of" => RelationType::ExtensionOf,
            "depends_on" => RelationType::DependsOn,
            "improves" => RelationType::Improves,
            "outperforms" => RelationType::Outperforms,
            _ => RelationType::Related,
        }
    }
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
