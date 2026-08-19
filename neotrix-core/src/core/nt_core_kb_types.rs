//! KB 共享领域类型 — NodeType / RelationType 下沉至 core (D3 架构倒置)。
//!
//! core 层 (second_brain 等) 需要知识图谱节点/边类型枚举, 原实现驻留
//! NT-MEMORY `nt_memory_types.rs`, 构成 core → l3 反向依赖。这些是**纯
//! serde 枚举** (无状态、无 l3 依赖), 下沉至 core 作为单一事实源; NT-MEMORY
//! `nt_memory_types.rs` re-export, 调用方路径全部保持不变。

use serde::{Deserialize, Serialize};

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
    WikiPage,
    /// Thinking trace — RL training record from SiliconSelf.add_thinking_trace()
    ThinkingTrace,
    /// Self-test failure — detection module's self_test() result
    SelfTestFailure,
    /// Event record — EventBus event persisted for traceability
    EventRecord,
    /// Detection finding — evaluate() output from a detection module
    DetectionFinding,
    /// Goal result — goal completion/failure record from goal_loop
    GoalResult,
    /// Session — session lifecycle event (start/stop/checkpoint)
    Session,
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
    WikiLink,
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

/// 知识图谱节点 — 纯数据 (D3 下沉, 单一事实源在 core)。
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

/// 知识图谱边 — 纯数据 (D3 下沉)。
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
}