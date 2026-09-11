use serde::{Deserialize, Serialize};

#[allow(async_fn_in_trait)]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, String>;
    fn dim(&self) -> usize;
    fn name(&self) -> &str;
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
    EvolutionPattern,
    ConversationEvolution,
    Textbook,
    Resource,
    External,
    Summary,
    Guide,
    Skill,
    Reference,
    WikiPage,
    ThinkingTrace,
    SelfTestFailure,
    EventRecord,
    DetectionFinding,
    GoalResult,
    Session,
    Filing,
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
            NodeType::WikiPage => "wiki_page",
            NodeType::ThinkingTrace => "thinking_trace",
            NodeType::SelfTestFailure => "self_test_failure",
            NodeType::EventRecord => "event_record",
            NodeType::DetectionFinding => "detection_finding",
            NodeType::GoalResult => "goal_result",
            NodeType::Session => "session",
            NodeType::Filing => "filing",
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
            "wiki_page" => NodeType::WikiPage,
            "thinking_trace" => NodeType::ThinkingTrace,
            "self_test_failure" => NodeType::SelfTestFailure,
            "event_record" => NodeType::EventRecord,
            "detection_finding" => NodeType::DetectionFinding,
            "goal_result" => NodeType::GoalResult,
            "session" => NodeType::Session,
            "filing" => NodeType::Filing,
            _ => NodeType::Concept,
        }
    }
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
    RelatedTo,
    EvolvedFrom,
    ResourceFor,
    AboutTopic,
    BelongsTo,
    SubTopicOf,
    CrossDomain,
    Contains,
    Influenced,
    ArchPartOf,
    Categorized,
    RelatesTo,
    InsightAbout,
    Implements,
    Uses,
    Visualizes,
    BrandFor,
    Illustrates,
    WikiLink,
    /// Explicit hierarchical parent-child relation (node A is parent of node B).
    HierarchicalParent,
    /// Bridge between two domain clusters (links knowledge across domains).
    CrossDomainBridge,
    /// Node belongs to a domain cluster.
    BelongsToCluster,
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
            RelationType::WikiLink => "wiki_link",
            RelationType::HierarchicalParent => "hierarchical_parent",
            RelationType::CrossDomainBridge => "cross_domain_bridge",
            RelationType::BelongsToCluster => "belongs_to_cluster",
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
            "hierarchical_parent" => RelationType::HierarchicalParent,
            "cross_domain_bridge" => RelationType::CrossDomainBridge,
            "belongs_to_cluster" => RelationType::BelongsToCluster,
            _ => RelationType::Related,
        }
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalValidity {
    pub valid_from: i64,
    pub valid_until: Option<i64>,
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
    pub recall_weight: f64,
    pub created_at: i64,
    pub updated_at: i64,
    pub access_count: i64,
    pub metadata: Option<serde_json::Value>,
    pub temporal: Option<TemporalValidity>,
    pub supersedes: Option<String>,
    pub source_episode: Option<String>,
    /// Hierarchical parent node ID (None = root-level node).
    pub parent_id: Option<String>,
    /// Depth in the knowledge hierarchy (0 = root, 1 = child of root, etc.).
    pub depth: i32,
    /// Domain cluster grouping — nodes with the same cluster_id belong to the
    /// same coherent knowledge domain (e.g. "rust_async", "ml_transformers").
    pub cluster_id: Option<String>,
}

/// Domain cluster — groups coherent knowledge nodes into a named domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeCluster {
    pub id: String,
    /// Human-readable domain name (e.g. "rust_async", "ml_transformers").
    pub name: String,
    /// Optional description of what this domain covers.
    pub description: Option<String>,
    /// Parent cluster ID for inter-domain hierarchy (None = top-level domain).
    pub parent_cluster_id: Option<String>,
    /// Number of nodes currently in this cluster.
    pub node_count: i64,
    /// Cluster-level confidence: average confidence of member nodes.
    pub avg_confidence: f64,
    /// Timestamp of last node addition or cluster metadata update.
    pub updated_at: i64,
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

#[allow(async_fn_in_trait)]
pub trait KnowledgeAccess: Send + Sync {
    fn get_node(&self, id: &str) -> Result<Option<KnowledgeNode>, String>;
    fn search(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeNode>, String>;
    fn nodes_by_type(&self, node_type: NodeType, limit: usize) -> Result<Vec<KnowledgeNode>, String>;
    fn edges_for_node(&self, node_id: &str) -> Result<Vec<KnowledgeEdge>, String>;
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>, String>;
    fn embedding_dim(&self) -> usize;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_roundtrip() {
        assert_eq!(NodeType::from_str("WikiPage"), NodeType::WikiPage);
        assert_eq!(NodeType::WikiPage.as_str(), "wiki_page");
        assert_eq!(NodeType::from_str("wiki_page"), NodeType::WikiPage);
        assert_eq!(NodeType::from_str("unknown_thing"), NodeType::Concept);
    }

    #[test]
    fn test_relation_type_roundtrip() {
        assert_eq!(RelationType::from_str("RelatedTo"), RelationType::RelatedTo);
        assert_eq!(RelationType::RelatedTo.as_str(), "related_to");
        assert_eq!(RelationType::from_str("unknown_rel"), RelationType::Related);
    }

    #[test]
    fn test_knowledge_node_serialization() {
        let node = KnowledgeNode {
            id: "n1".into(),
            node_type: NodeType::Paper,
            title: "Test".into(),
            summary: None,
            content: None,
            url: None,
            domain: None,
            language: "en".into(),
            confidence: 0.9,
            importance: 0.5,
            recall_weight: 1.0,
            created_at: 0,
            updated_at: 0,
            access_count: 0,
            metadata: None,
            temporal: None,
            supersedes: None,
            source_episode: None,
        };
        let json = serde_json::to_string(&node).unwrap();
        let back: KnowledgeNode = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "n1");
        assert_eq!(back.node_type, NodeType::Paper);
    }
}
