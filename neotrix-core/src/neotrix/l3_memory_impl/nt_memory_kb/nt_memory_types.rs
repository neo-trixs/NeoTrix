use serde::{Deserialize, Serialize};

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
    /// Evolution pattern — learned behavioral pattern from SEAL pipeline
    EvolutionPattern,
    /// Conversation evolution record — distilled from user interactions
    ConversationEvolution,
    /// Textbook — long-form educational content
    Textbook,
    /// Resource — external resource reference
    Resource,
    /// External — external entity or system
    External,
    /// Summary — condensed summary of content
    Summary,
    /// Guide — how-to guide or tutorial
    Guide,
    /// Skill — learned/absorbed skill
    Skill,
    /// Reference — external reference link
    Reference,
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
            NodeType::EvolutionPattern => "evolution_pattern",
            NodeType::ConversationEvolution => "conversation_evolution",
            NodeType::Textbook => "textbook",
            NodeType::Resource => "resource",
            NodeType::External => "external",
            NodeType::Summary => "summary",
            NodeType::Guide => "guide",
            NodeType::Skill => "skill",
            NodeType::Reference => "reference",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match normalize_type(s).as_str() {
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
            "evolution_pattern" => NodeType::EvolutionPattern,
            "conversation_evolution" => NodeType::ConversationEvolution,
            "textbook" => NodeType::Textbook,
            "resource" => NodeType::Resource,
            "external" => NodeType::External,
            "summary" => NodeType::Summary,
            "guide" => NodeType::Guide,
            "skill" => NodeType::Skill,
            "reference" => NodeType::Reference,
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
    /// Generic "related to" — most common pipeline relation (174K edges)
    RelatedTo,
    /// Evolutionary descent — one node evolved from another
    EvolvedFrom,
    /// Resource provides capability for a task
    ResourceFor,
    /// Topic association
    AboutTopic,
    /// Ownership or membership
    BelongsTo,
    /// Sub-topic hierarchy
    SubTopicOf,
    /// Cross-domain bridge
    CrossDomain,
    /// Container relationship
    Contains,
    /// Influence relationship
    Influenced,
    /// Architectural component
    ArchPartOf,
    /// Categorization
    Categorized,
    /// Generic relates-to (lower-volume variant of RelatedTo)
    RelatesTo,
    /// Insight specifically about a topic
    InsightAbout,
    /// Implements a protocol/interface
    Implements,
    /// Uses a tool/library
    Uses,
    /// Visualizes or renders
    Visualizes,
    /// Brand/identity association
    BrandFor,
    /// Illustrates a concept
    Illustrates,
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
            RelationType::RelatedTo => "related_to",
            RelationType::EvolvedFrom => "evolved_from",
            RelationType::ResourceFor => "resource_for",
            RelationType::AboutTopic => "about_topic",
            RelationType::BelongsTo => "belongs_to",
            RelationType::SubTopicOf => "sub_topic_of",
            RelationType::CrossDomain => "cross_domain",
            RelationType::Contains => "contains",
            RelationType::Influenced => "influenced",
            RelationType::ArchPartOf => "arch_part_of",
            RelationType::Categorized => "categorized",
            RelationType::RelatesTo => "relates_to",
            RelationType::InsightAbout => "insight_about",
            RelationType::Implements => "implements",
            RelationType::Uses => "uses",
            RelationType::Visualizes => "visualizes",
            RelationType::BrandFor => "brand_for",
            RelationType::Illustrates => "illustrates",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match normalize_type(s).as_str() {
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
            "related_to" => RelationType::RelatedTo,
            "evolved_from" => RelationType::EvolvedFrom,
            "resource_for" => RelationType::ResourceFor,
            "about_topic" => RelationType::AboutTopic,
            "belongs_to" => RelationType::BelongsTo,
            "sub_topic_of" => RelationType::SubTopicOf,
            "cross_domain" => RelationType::CrossDomain,
            "contains" => RelationType::Contains,
            "influenced" => RelationType::Influenced,
            "arch_part_of" => RelationType::ArchPartOf,
            "categorized" => RelationType::Categorized,
            "relates_to" => RelationType::RelatesTo,
            "insight_about" => RelationType::InsightAbout,
            "implements" => RelationType::Implements,
            "uses" => RelationType::Uses,
            "visualizes" => RelationType::Visualizes,
            "brand_for" => RelationType::BrandFor,
            "illustrates" => RelationType::Illustrates,
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
    pub signals: Option<[f64; 4]>,
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
fn normalize_type(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for (i, ch) in s.char_indices() {
        if i > 0 && ch.is_uppercase() {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert!(true);
    }

    #[test]
    fn test_normalize_type() {
        assert_eq!(normalize_type("EvolutionPattern"), "evolution_pattern");
        assert_eq!(normalize_type("ConversationEvolution"), "conversation_evolution");
        assert_eq!(normalize_type("CodeSnippet"), "code_snippet");
        assert_eq!(normalize_type("HarnessProfile"), "harness_profile");
        assert_eq!(normalize_type("ResourceFor"), "resource_for");
        assert_eq!(normalize_type("SubTopicOf"), "sub_topic_of");
        assert_eq!(normalize_type("ArchPartOf"), "arch_part_of");
        assert_eq!(normalize_type("concept"), "concept");
        assert_eq!(normalize_type("repository"), "repository");
        assert_eq!(normalize_type("insight"), "insight");
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
