//! nt_entity_graph — 实体图容器 (EntityGraph).
//! 从 `nt_memory_graphrag/mod.rs` 纯搬移, 行为零变更.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::nt_types::{EntityNode, RelationEdge};

// ─── Entity Graph ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityGraph {
    pub entities: HashMap<String, EntityNode>,
    pub relations: HashMap<String, RelationEdge>,
    pub adjacency: HashMap<String, Vec<(String, String, String)>>,
}

impl Default for EntityGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityGraph {
    pub fn new() -> Self {
        EntityGraph {
            entities: HashMap::new(),
            relations: HashMap::new(),
            adjacency: HashMap::new(),
        }
    }
}
