use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};

use super::ReasoningBank;
use crate::core::fs_util::{atomic_write_signed, verify_sig};
use crate::core::nt_core_knowledge::TaskType;
use crate::core::nt_core_bank::iteration::Bm25Index;
use crate::core::nt_core_bank::ReasoningMemory;

#[derive(Serialize, Deserialize)]
struct ReasoningBankSnapshot {
    memories: Vec<ReasoningMemory>,
    max_memories: usize,
    task_type_index: HashMap<TaskType, Vec<usize>>,
    hypergraph: Option<crate::core::nt_core_graph::HyperGraph>,
    wraps_layered_memory: bool,
}

impl ReasoningBank {
    pub fn save_to(&self, path: &std::path::Path) -> Result<(), String> {
        let snapshot = ReasoningBankSnapshot {
            memories: self.memories.iter().cloned().collect(),
            max_memories: self.max_memories,
            task_type_index: self.task_type_index.clone(),
            hypergraph: self.hypergraph.clone(),
            wraps_layered_memory: self.wraps_layered_memory,
        };
        // Use versioned envelope for future migration support
        let enveloped = crate::core::persist_envelope::write_envelope(&snapshot)
            .map_err(|e| format!("envelope bank: {}", e))?;
        atomic_write_signed(path, &enveloped)
            .map_err(|e| format!("atomic write bank: {}", e))?;
        Ok(())
    }

    pub fn load_from(path: &std::path::Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self {
                memories: std::collections::VecDeque::new(),
                max_memories: 100,
                task_type_index: HashMap::new(),
                bm25: RwLock::new(Bm25Index::empty()),
                bm25_dirty: AtomicBool::new(false),
                hypergraph: None,
                #[cfg(feature = "e8-theory")]
                wh_index: None,
                wraps_layered_memory: false,
                layered: None,
            });
        }
        let json_bytes = std::fs::read_to_string(path).map_err(|e| format!("read bank: {}", e))?;
        if let Err(e) = verify_sig(path, json_bytes.as_bytes()) {
            log::warn!("[bank] integrity check skipped (pre-existing file): {}", e);
        }
        // Read through versioned envelope (handles backward compat with plain JSON)
        let snapshot: ReasoningBankSnapshot = crate::core::persist_envelope::read_envelope(path, json_bytes.as_bytes())
            .map_err(|e| format!("deserialize bank: {}", e))?;

        let mut task_type_index: HashMap<TaskType, Vec<usize>> = HashMap::new();
        for (i, mem) in snapshot.memories.iter().enumerate() {
            task_type_index.entry(mem.task_type).or_default().push(i);
        }

        Ok(Self {
            memories: snapshot.memories.into(),
            max_memories: snapshot.max_memories,
            task_type_index,
            bm25: RwLock::new(Bm25Index::empty()),
            bm25_dirty: AtomicBool::new(true),
            hypergraph: snapshot.hypergraph,
            #[cfg(feature = "e8-theory")]
            wh_index: None,
            wraps_layered_memory: snapshot.wraps_layered_memory,
            layered: None,
        })
    }

    pub fn enable_hypergraph(&mut self, initial_capacity: usize) {
        let graph = crate::core::nt_core_graph::HyperGraph::with_capacity(initial_capacity);
        self.hypergraph = Some(graph);
    }

    pub fn index_memory(&mut self, memory_id: &str) -> Result<(), String> {
        let graph = self.hypergraph.as_mut().ok_or_else(|| "hypergraph not enabled".to_string())?;
        let mem = self.memories.iter().find(|m| m.id == memory_id).cloned()
            .ok_or_else(|| format!("memory not found: {}", memory_id))?;

        let node_type = match mem.task_type {
            TaskType::Learning | TaskType::Research | TaskType::Reflection | TaskType::MetaCognition => crate::core::nt_core_graph::HyperNodeType::Concept,
            TaskType::CodeAnalysis | TaskType::CodeReview | TaskType::CodeGeneration => crate::core::nt_core_graph::HyperNodeType::Pattern,
            TaskType::UIDesign | TaskType::Security => crate::core::nt_core_graph::HyperNodeType::Skill,
            TaskType::Planning => crate::core::nt_core_graph::HyperNodeType::Goal,
            _ => crate::core::nt_core_graph::HyperNodeType::Memory,
        };

        let mut node = crate::core::nt_core_graph::HyperNode::new(&mem.id, node_type, &mem.task_description, mem.reward);
        if let Some(ref emb) = mem.embedding { node.embedding = emb.clone(); }
        graph.add_node(node);

        let existing_ids: Vec<String> = graph.nodes.keys().cloned().collect();
        for other_id in &existing_ids {
            if other_id == memory_id { continue; }
            if let Some(other_node) = graph.nodes.get(other_id) {
                let mut strength = 0.0;
                if let Some(ref emb) = mem.embedding {
                    if !other_node.embedding.is_empty() {
                        strength = crate::core::nt_core_graph::HyperGraph::cosine_similarity(emb, &other_node.embedding);
                    }
                }
                if strength == 0.0 {
                    if let Some(other_mem) = self.memories.iter().find(|m| m.id == *other_id) {
                        if other_mem.task_type == mem.task_type { strength = 0.5; }
                    }
                }
                if strength > 0.3 {
                    graph.add_edge(memory_id, other_id, crate::core::nt_core_graph::EdgeRelation::SimilarTo, strength);
                }
            }
        }
        Ok(())
    }

    pub fn hypergraph_traverse(&self, start_memory_id: &str, depth: usize) -> Vec<String> {
        let graph = match self.hypergraph { Some(ref g) => g, None => return Vec::new() };
        let nodes = graph.traverse(start_memory_id, depth);
        nodes.iter().map(|n| n.id.clone()).collect()
    }

    #[cfg(feature = "rkyv-storage")]
    pub fn store_to_rkyv(&self, subdir: &str) -> Result<(), String> {
        use std::path::PathBuf;
        let dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."))
            .join("neotrix").join("rkyv").join(subdir);
        std::fs::create_dir_all(&dir).map_err(|e| format!("create rkyv dir: {}", e))?;
        let path = dir.join("bank.json");
        let memories: Vec<&ReasoningMemory> = self.memories.iter().collect();
        let json = serde_json::to_string_pretty(&memories)
            .map_err(|e| format!("serialize bank: {}", e))?;
        std::fs::write(&path, &json).map_err(|e| format!("write bank: {}", e))?;
        log::info!("[rkyv] stored {} memories to {:?}", memories.len(), path);
        Ok(())
    }

    #[cfg(feature = "rkyv-storage")]
    pub fn load_from_rkyv(&mut self, subdir: &str) -> Result<usize, String> {
        use std::path::PathBuf;
        let path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."))
            .join("neotrix").join("rkyv").join(subdir).join("bank.json");
        if !path.exists() {
            return Ok(0);
        }
        let json = std::fs::read_to_string(&path)
            .map_err(|e| format!("read bank: {}", e))?;
        let memories: Vec<ReasoningMemory> = serde_json::from_str(&json)
            .map_err(|e| format!("deserialize bank: {}", e))?;
        let count = memories.len();
        for mem in memories {
            self.store(mem);
        }
        log::info!("[rkyv] loaded {} memories from {:?}", count, path);
        Ok(count)
    }
}
