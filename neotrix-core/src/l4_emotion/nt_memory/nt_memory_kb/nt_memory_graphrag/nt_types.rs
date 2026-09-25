//! nt_types — GraphRAG 共享基础: ID/时间 helpers + 实体/关系/配置/结果类型 + KB 类型桥.
//! 从 `nt_memory_graphrag/mod.rs` 纯搬移, 行为零变更.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use neotrix_types::knowledge_access::{KnowledgeEdge, KnowledgeNode, NodeType, RelationType};

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

pub(crate) fn generate_id() -> String {
    let count = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:016x}{:016x}", now, count)
}

pub(crate) fn now_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ─── Entity Node ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityNode {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub source_node_id: String,
    pub confidence: f64,
    pub properties: HashMap<String, String>,
    pub created_at: u64,
}

// ─── Relation Edge ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationEdge {
    pub id: String,
    pub source_entity: String,
    pub target_entity: String,
    pub relation_type: String,
    pub weight: f64,
    pub evidence: String,
    pub confidence: f64,
    pub created_at: u64,
}

// ─── Type Bridge: EntityNode ↔ KnowledgeNode ─────────────────────────

impl From<EntityNode> for KnowledgeNode {
    fn from(e: EntityNode) -> Self {
        KnowledgeNode {
            id: e.id,
            node_type: NodeType::from_str(&e.entity_type),
            title: e.name,
            summary: None,
            content: None,
            url: None,
            domain: None,
            language: "en".to_string(),
            confidence: e.confidence,
            importance: 0.5,
            recall_weight: 1.0,
            created_at: e.created_at as i64,
            updated_at: e.created_at as i64,
            access_count: 0,
            metadata: Some(serde_json::json!({
                "source_node_id": e.source_node_id,
                "properties": e.properties,
            })),
            temporal: None,
            supersedes: None,
            source_episode: Some(e.source_node_id),
            parent_id: None,
            depth: 0,
            cluster_id: None,
        }
    }
}

impl From<KnowledgeNode> for EntityNode {
    fn from(k: KnowledgeNode) -> Self {
        let source_node_id = k
            .source_episode
            .or_else(|| {
                k.metadata.as_ref().and_then(|m| {
                    m.get("source_node_id")
                        .and_then(|v| v.as_str().map(String::from))
                })
            })
            .unwrap_or_default();
        let properties = k
            .metadata
            .as_ref()
            .and_then(|m| m.get("properties"))
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        EntityNode {
            id: k.id,
            name: k.title,
            entity_type: k.node_type.as_str().to_string(),
            source_node_id,
            confidence: k.confidence,
            properties,
            created_at: k.created_at as u64,
        }
    }
}

// ─── Type Bridge: RelationEdge ↔ KnowledgeEdge ───────────────────────

impl From<RelationEdge> for KnowledgeEdge {
    fn from(r: RelationEdge) -> Self {
        KnowledgeEdge {
            id: r.id,
            source_id: r.source_entity,
            target_id: r.target_entity,
            relation_type: RelationType::from_str(&r.relation_type),
            weight: r.weight,
            description: Some(r.evidence),
            created_at: r.created_at as i64,
            metadata: Some(serde_json::json!({
                "confidence": r.confidence,
            })),
        }
    }
}

impl From<KnowledgeEdge> for RelationEdge {
    fn from(k: KnowledgeEdge) -> Self {
        let confidence = k
            .metadata
            .as_ref()
            .and_then(|m| m.get("confidence"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        RelationEdge {
            id: k.id,
            source_entity: k.source_id,
            target_entity: k.target_id,
            relation_type: k.relation_type.as_str().to_string(),
            weight: k.weight,
            evidence: k.description.unwrap_or_default(),
            confidence,
            created_at: k.created_at as u64,
        }
    }
}

// ─── Graph Query Mode ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphQueryMode {
    Local {
        max_depth: usize,
        max_neighbors: usize,
    },
    Global {
        community_level: usize,
    },
    Hybrid {
        local_depth: usize,
        global_level: usize,
    },
    Auto,
}

// ─── Config ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum ExtractionMode {
    #[default]
    Heuristic,
    Llm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRagConfig {
    pub max_entities_per_doc: usize,
    pub min_confidence: f64,
    pub enable_incremental_updates: bool,
    pub max_graph_size: usize,
    pub extraction_mode: ExtractionMode,
}

impl Default for GraphRagConfig {
    fn default() -> Self {
        GraphRagConfig {
            max_entities_per_doc: 50,
            min_confidence: 0.3,
            enable_incremental_updates: true,
            max_graph_size: 100000,
            extraction_mode: ExtractionMode::Heuristic,
        }
    }
}

// ─── Stats ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphRagStats {
    pub total_entities: usize,
    pub total_relations: usize,
    pub extraction_runs: u64,
    pub avg_extraction_time_ms: f64,
}

// ─── Subgraph Result ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubgraphResult {
    pub entities: Vec<EntityNode>,
    pub relations: Vec<RelationEdge>,
    pub traversal_depth: usize,
    pub query_mode: String,
}

// ─── Hybrid Result ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridResult {
    pub local_results: Vec<SubgraphResult>,
    pub global_results: Vec<GlobalSummary>,
    pub merged_entities: Vec<EntityNode>,
    pub merged_relations: Vec<RelationEdge>,
}

// ─── Global Summary ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSummary {
    pub community_id: String,
    pub topic_keywords: Vec<String>,
    pub summary_text: String,
    pub confidence: f64,
    pub last_updated: u64,
    pub entity_count: usize,
    pub relation_count: usize,
}

// ─── Incremental Change ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalChange {
    pub added_entities: Vec<EntityNode>,
    pub added_relations: Vec<RelationEdge>,
    pub timestamp: u64,
}

// ─── Community ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: String,
    pub entity_ids: Vec<String>,
    pub summary: String,
    pub size: usize,
    pub avg_confidence: f64,
}
