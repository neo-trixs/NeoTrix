use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types;

// ── Local type definitions (subset of L3 nt_memory_types) ──

#[derive(Debug, Clone, PartialEq)]
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
}

#[derive(Debug, Clone, PartialEq)]
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
}

#[derive(Debug, Clone)]
pub struct KnowledgeNode {
    pub id: String,
    pub title: String,
    pub node_type: NodeType,
    pub content: Option<String>,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Default)]
pub struct CrawlCycleReport {
    pub attempted: usize,
    pub completed: usize,
    pub failed: usize,
    pub nodes_created: usize,
    pub edges_created: usize,
    pub urls_processed: Vec<String>,
    pub errors: Vec<(String, String)>,
    pub by_domain: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct DiscoveryPipelineConfig {
    pub min_stars_for_topic_discovery: i64,
    pub repos_per_topic: usize,
    pub scan_only_new_topics: bool,
    pub max_popular_repo_pages: usize,
}

impl Default for DiscoveryPipelineConfig {
    fn default() -> Self {
        Self {
            min_stars_for_topic_discovery: 1000,
            repos_per_topic: 10,
            scan_only_new_topics: false,
            max_popular_repo_pages: 5,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GithubDiscoveryStats {
    pub topics_found: usize,
    pub repos_found: usize,
    pub repos_ingested: usize,
    pub repos_skipped_existing: usize,
    pub api_calls: usize,
    pub errors: Vec<(String, String)>,
}

// ── KnowledgeBase — L2 wrapper that delegates to L3 ──

pub struct KnowledgeBase {
    pub conn: Mutex<Connection>,
    pub db_path: PathBuf,
}

impl From<crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase> for KnowledgeBase {
    fn from(real: crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase) -> Self {
        let db_path = real.db_path.clone();
        let conn = real.conn.into_inner().unwrap_or_else(|e| e.into_inner());
        Self {
            conn: Mutex::new(conn),
            db_path,
        }
    }
}

impl KnowledgeBase {
    pub fn open(path: Option<PathBuf>) -> Result<Self, String> {
        let real = crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase::open(path)?;
        let db_path = real.db_path.clone();
        let conn = real
            .conn
            .into_inner()
            .unwrap_or_else(|e| e.into_inner());
        Ok(Self {
            conn: Mutex::new(conn),
            db_path,
        })
    }

    pub fn clone_connection(&self) -> Result<Self, String> {
        let conn = Connection::open(&self.db_path).map_err(|e| {
            format!("cannot open {}: {}", self.db_path.display(), e)
        })?;
        let _ = crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_schema::initialize(&conn);
        Ok(Self {
            conn: Mutex::new(conn),
            db_path: self.db_path.clone(),
        })
    }

    pub fn find_node_by_url(&self, url: &str) -> Result<Option<KnowledgeNode>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let result =
            crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_store::find_node_by_url(
                &conn, url,
            )
            .map_err(|e| format!("find_node_by_url: {}", e))?;
        result.map(|n| Ok(conv::from_real_node(n))).transpose()
    }

    pub fn insert_or_get_node(
        &self,
        title: &str,
        node_type: NodeType,
        summary: Option<&str>,
        url: Option<&str>,
        domain: Option<&str>,
    ) -> Result<String, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_store::insert_or_get_node(
            &conn,
            title,
            conv::to_real_nt(&node_type),
            summary,
            url,
            domain,
        )
        .map_err(|e| format!("insert_or_get_node: {}", e))
    }

    pub fn update_node_content(&self, id: &str, content: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let mut node =
            crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_store::get_node(&conn, id)
                .map_err(|e| format!("get_node: {}", e))?
                .ok_or_else(|| format!("Node not found: {}", id))?;
        node.content = Some(content.to_string());
        node.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_store::update_node(&conn, &node)
            .map_err(|e| format!("update_node_content: {}", e))
    }

    pub fn update_node_metadata(
        &self,
        id: &str,
        metadata: &serde_json::Value,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_store::update_node_metadata(
            &conn, id, metadata,
        )
        .map_err(|e| format!("update_node_metadata: {}", e))
    }

    pub fn upsert_edge(
        &self,
        source_id: &str,
        target_id: &str,
        relation_type: RelationType,
        weight: f64,
        description: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_store::upsert_edge(
            &conn,
            source_id,
            target_id,
            conv::to_real_rt(&relation_type),
            weight,
            description,
        )
        .map_err(|e| format!("upsert_edge: {}", e))
    }

    pub fn kv_get(
        &self,
        namespace: &str,
        key: &str,
    ) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::kv_get(
            &conn, namespace, key,
        )
    }

    pub fn kv_set(
        &self,
        namespace: &str,
        key: &str,
        value: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::kv_set(
            &conn, namespace, key, value,
        )
    }

    pub fn find_repositories(
        &self,
        domain: &str,
        min_stars: Option<i64>,
    ) -> Result<Vec<KnowledgeNode>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let all =
            crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_store::get_all_nodes(&conn)
                .map_err(|e| format!("get_all_nodes: {}", e))?;
        drop(conn);
        let local_all: Vec<KnowledgeNode> = all.into_iter().map(conv::from_real_node).collect();
        let repos: Vec<KnowledgeNode> = local_all
            .into_iter()
            .filter(|n| n.node_type == NodeType::Repository)
            .filter(|n| n.domain.as_deref() == Some(domain))
            .filter(|n| {
                if let Some(min) = min_stars {
                    n.metadata
                        .as_ref()
                        .and_then(|m| m.get("stars").and_then(|v| v.as_i64()))
                        .map(|s| s >= min)
                        .unwrap_or(false)
                } else {
                    true
                }
            })
            .collect();
        Ok(repos)
    }

    pub fn ingest_arxiv(&self, id: &str) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::ingest_from_arxiv(&conn, id)
    }

    pub fn ingest_wikipedia(&self, topic: &str) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::ingest_from_wikipedia(
            &conn, topic,
        )
    }

    pub fn run_github_topics_discovery(
        &self,
        config: &DiscoveryPipelineConfig,
    ) -> Result<GithubDiscoveryStats, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let real_config =
            crate::neotrix::l3_memory_impl::nt_memory_kb::nt_discovery_github_topics::DiscoveryPipelineConfig {
                min_stars_for_topic_discovery: config.min_stars_for_topic_discovery,
                repos_per_topic: config.repos_per_topic,
                scan_only_new_topics: config.scan_only_new_topics,
                max_popular_repo_pages: config.max_popular_repo_pages,
            };
        let stats =
            crate::neotrix::l3_memory_impl::nt_memory_kb::nt_discovery_github_topics::run_github_topics_discovery(
                &conn, &real_config,
            )?;
        Ok(GithubDiscoveryStats {
            repos_ingested: stats.repos_ingested,
            ..Default::default()
        })
    }
}

/// Convert a real L3 NodeType to the bridge's NodeType (all 31 variants).
pub fn from_real_node_type(nt: &nt_memory_types::NodeType) -> NodeType {
    conv::from_real_nt(nt)
}

// ── Conversion helpers (private, bridge only) ──

mod conv {
    use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types as real_types;

    pub fn to_real_nt(nt: &super::NodeType) -> real_types::NodeType {
        match nt {
            super::NodeType::Concept => real_types::NodeType::Concept,
            super::NodeType::Paper => real_types::NodeType::Paper,
            super::NodeType::Repository => real_types::NodeType::Repository,
            super::NodeType::Person => real_types::NodeType::Person,
            super::NodeType::Event => real_types::NodeType::Event,
            super::NodeType::Source => real_types::NodeType::Source,
            super::NodeType::Tool => real_types::NodeType::Tool,
            super::NodeType::Framework => real_types::NodeType::Framework,
            super::NodeType::Algorithm => real_types::NodeType::Algorithm,
            super::NodeType::Theory => real_types::NodeType::Theory,
            super::NodeType::Method => real_types::NodeType::Method,
            super::NodeType::Dataset => real_types::NodeType::Dataset,
            super::NodeType::Benchmark => real_types::NodeType::Benchmark,
            super::NodeType::Organization => real_types::NodeType::Organization,
            super::NodeType::Book => real_types::NodeType::Book,
            super::NodeType::Course => real_types::NodeType::Course,
            super::NodeType::Article => real_types::NodeType::Article,
            super::NodeType::CodeSnippet => real_types::NodeType::CodeSnippet,
            super::NodeType::Idea => real_types::NodeType::Idea,
            super::NodeType::Question => real_types::NodeType::Question,
            super::NodeType::Insight => real_types::NodeType::Insight,
            super::NodeType::HarnessProfile => real_types::NodeType::HarnessProfile,
            super::NodeType::Image => real_types::NodeType::Image,
            super::NodeType::EvolutionPattern => real_types::NodeType::EvolutionPattern,
            super::NodeType::ConversationEvolution => real_types::NodeType::ConversationEvolution,
            super::NodeType::Textbook => real_types::NodeType::Textbook,
            super::NodeType::Resource => real_types::NodeType::Resource,
            super::NodeType::External => real_types::NodeType::External,
            super::NodeType::Summary => real_types::NodeType::Summary,
            super::NodeType::Guide => real_types::NodeType::Guide,
            super::NodeType::Skill => real_types::NodeType::Skill,
            super::NodeType::Reference => real_types::NodeType::Reference,
        }
    }

    pub fn from_real_nt(nt: &real_types::NodeType) -> super::NodeType {
        match nt {
            real_types::NodeType::Concept => super::NodeType::Concept,
            real_types::NodeType::Paper => super::NodeType::Paper,
            real_types::NodeType::Repository => super::NodeType::Repository,
            real_types::NodeType::Person => super::NodeType::Person,
            real_types::NodeType::Event => super::NodeType::Event,
            real_types::NodeType::Source => super::NodeType::Source,
            real_types::NodeType::Tool => super::NodeType::Tool,
            real_types::NodeType::Framework => super::NodeType::Framework,
            real_types::NodeType::Algorithm => super::NodeType::Algorithm,
            real_types::NodeType::Theory => super::NodeType::Theory,
            real_types::NodeType::Method => super::NodeType::Method,
            real_types::NodeType::Dataset => super::NodeType::Dataset,
            real_types::NodeType::Benchmark => super::NodeType::Benchmark,
            real_types::NodeType::Organization => super::NodeType::Organization,
            real_types::NodeType::Book => super::NodeType::Book,
            real_types::NodeType::Course => super::NodeType::Course,
            real_types::NodeType::Article => super::NodeType::Article,
            real_types::NodeType::CodeSnippet => super::NodeType::CodeSnippet,
            real_types::NodeType::Idea => super::NodeType::Idea,
            real_types::NodeType::Question => super::NodeType::Question,
            real_types::NodeType::Insight => super::NodeType::Insight,
            real_types::NodeType::HarnessProfile => super::NodeType::HarnessProfile,
            real_types::NodeType::Image => super::NodeType::Image,
            real_types::NodeType::EvolutionPattern => super::NodeType::EvolutionPattern,
            real_types::NodeType::ConversationEvolution => super::NodeType::ConversationEvolution,
            real_types::NodeType::Textbook => super::NodeType::Textbook,
            real_types::NodeType::Resource => super::NodeType::Resource,
            real_types::NodeType::External => super::NodeType::External,
            real_types::NodeType::Summary => super::NodeType::Summary,
            real_types::NodeType::Guide => super::NodeType::Guide,
            real_types::NodeType::Skill => super::NodeType::Skill,
            real_types::NodeType::Reference => super::NodeType::Reference,
        }
    }

    pub fn to_real_rt(rt: &super::RelationType) -> real_types::RelationType {
        match rt {
            super::RelationType::References => real_types::RelationType::References,
            super::RelationType::SubclassOf => real_types::RelationType::SubclassOf,
            super::RelationType::InstanceOf => real_types::RelationType::InstanceOf,
            super::RelationType::Causes => real_types::RelationType::Causes,
            super::RelationType::PrerequisiteOf => real_types::RelationType::PrerequisiteOf,
            super::RelationType::Contradicts => real_types::RelationType::Contradicts,
            super::RelationType::Supports => real_types::RelationType::Supports,
            super::RelationType::BeforeInTime => real_types::RelationType::BeforeInTime,
            super::RelationType::AfterInTime => real_types::RelationType::AfterInTime,
            super::RelationType::Related => real_types::RelationType::Related,
            super::RelationType::PartOf => real_types::RelationType::PartOf,
            super::RelationType::DevelopedBy => real_types::RelationType::DevelopedBy,
            super::RelationType::ImplementedIn => real_types::RelationType::ImplementedIn,
            super::RelationType::InspiredBy => real_types::RelationType::InspiredBy,
            super::RelationType::Citation => real_types::RelationType::Citation,
            super::RelationType::ExtensionOf => real_types::RelationType::ExtensionOf,
            super::RelationType::DependsOn => real_types::RelationType::DependsOn,
            super::RelationType::Improves => real_types::RelationType::Improves,
            super::RelationType::Outperforms => real_types::RelationType::Outperforms,
            super::RelationType::RelatedTo => real_types::RelationType::RelatedTo,
            super::RelationType::EvolvedFrom => real_types::RelationType::EvolvedFrom,
            super::RelationType::ResourceFor => real_types::RelationType::ResourceFor,
            super::RelationType::AboutTopic => real_types::RelationType::AboutTopic,
            super::RelationType::BelongsTo => real_types::RelationType::BelongsTo,
            super::RelationType::SubTopicOf => real_types::RelationType::SubTopicOf,
            super::RelationType::CrossDomain => real_types::RelationType::CrossDomain,
            super::RelationType::Contains => real_types::RelationType::Contains,
            super::RelationType::Influenced => real_types::RelationType::Influenced,
            super::RelationType::ArchPartOf => real_types::RelationType::ArchPartOf,
            super::RelationType::Categorized => real_types::RelationType::Categorized,
            super::RelationType::RelatesTo => real_types::RelationType::RelatesTo,
            super::RelationType::InsightAbout => real_types::RelationType::InsightAbout,
            super::RelationType::Implements => real_types::RelationType::Implements,
            super::RelationType::Uses => real_types::RelationType::Uses,
            super::RelationType::Visualizes => real_types::RelationType::Visualizes,
            super::RelationType::BrandFor => real_types::RelationType::BrandFor,
            super::RelationType::Illustrates => real_types::RelationType::Illustrates,
        }
    }

    pub fn from_real_node(node: real_types::KnowledgeNode) -> super::KnowledgeNode {
        super::KnowledgeNode {
            id: node.id,
            title: node.title,
            node_type: from_real_nt(&node.node_type),
            content: node.content,
            url: node.url,
            domain: node.domain,
            metadata: node.metadata,
            created_at: node.created_at,
            updated_at: node.updated_at,
        }
    }
}
