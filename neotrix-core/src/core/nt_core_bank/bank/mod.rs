use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;
use chrono::Utc;

use crate::core::knowledge::TaskType;
use crate::core::nt_core_bank::{MemoryTier, ReasoningMemory, ReasoningBankStats, MemoryDetailedStats};
use crate::core::nt_core_walsh::WalshMemoryIndex;
use crate::core::nt_core_kron::KroneckerCleanup;
use super::iteration::Bm25Index;

pub mod store;
pub mod search;
pub mod maintenance;
pub mod ext;
pub mod seeds;

pub struct ReasoningBank {
    memories: VecDeque<ReasoningMemory>,
    max_memories: usize,
    task_type_index: HashMap<TaskType, Vec<usize>>,
    bm25: RwLock<Bm25Index>,
    bm25_dirty: AtomicBool,
    hypergraph: Option<crate::core::nt_core_graph::HyperGraph>,
    wh_index: Option<WalshMemoryIndex>,
    pub(crate) kronecker: Option<KroneckerCleanup>,
}

impl ReasoningBank {
    pub fn new(max_memories: usize) -> Self {
        Self {
            memories: VecDeque::with_capacity(max_memories),
            max_memories,
            task_type_index: HashMap::new(),
            bm25: RwLock::new(Bm25Index::empty()),
            bm25_dirty: AtomicBool::new(false),
            hypergraph: None,
            wh_index: Some(WalshMemoryIndex::new()),
            kronecker: None,
        }
    }

    pub fn new_without_wh(max_memories: usize) -> Self {
        let mut bank = Self::new(max_memories);
        bank.wh_index = None;
        bank
    }

    pub(crate) fn set_kronecker(&mut self, k: KroneckerCleanup) {
        self.kronecker = Some(k);
    }

    pub fn store(&mut self, memory: ReasoningMemory) {
        if self.memories.len() >= self.max_memories {
            if let Some(oldest) = self.memories.pop_front() {
                if let Some(indices) = self.task_type_index.get_mut(&oldest.task_type) {
                    indices.retain(|&i| i != 0);
                    for indices in self.task_type_index.values_mut() {
                        for idx in indices.iter_mut() {
                            if *idx > 0 { *idx -= 1; }
                        }
                    }
                }
                if let Some(ref mut wh) = self.wh_index {
                    wh.remove(&oldest.id);
                }
            }
        }
        let new_idx = self.memories.len();
        self.task_type_index.entry(memory.task_type).or_default().push(new_idx);
        if let Some(ref mut wh) = self.wh_index {
            let mut text = format!("{} {:?}", memory.task_description, memory.task_type);
            if let Some(ref v) = memory.t3_views.struct_view { text.push_str(&format!(" struct:{}", v)); }
            if let Some(ref v) = memory.t3_views.semantic_view { text.push_str(&format!(" semantic:{}", v)); }
            if let Some(ref v) = memory.t3_views.reflect_view { text.push_str(&format!(" reflect:{}", v)); }
            wh.store(&memory.id, &text);
        }
        self.memories.push_back(memory);
        self.bm25_dirty.store(true, Ordering::SeqCst);
    }

    pub fn store_with_embedding(&mut self, mut memory: ReasoningMemory, embedding: Vec<f64>) {
        memory.embedding = Some(embedding);
        self.store(memory);
    }

    pub fn memories(&self) -> &VecDeque<ReasoningMemory> {
        &self.memories
    }

    pub fn stats(&self) -> ReasoningBankStats {
        let total = self.memories.len();
        let successes = self.memories.iter().filter(|m| m.success).count();
        ReasoningBankStats {
            total_memories: total,
            success_count: successes,
            success_rate: if total > 0 { successes as f64 / total as f64 } else { 0.0 },
        }
    }

    pub fn stats_detailed(&self) -> MemoryDetailedStats {
        let total = self.memories.len();
        let mut tier_working = 0;
        let mut tier_episodic = 0;
        let mut tier_semantic = 0;
        let mut tier_procedural = 0;
        let mut total_confidence = 0.0;
        let mut total_importance = 0.0;
        for m in &self.memories {
            match m.tier {
                MemoryTier::Working => tier_working += 1,
                MemoryTier::Episodic => tier_episodic += 1,
                MemoryTier::Semantic => tier_semantic += 1,
                MemoryTier::Procedural => tier_procedural += 1,
            }
            total_confidence += m.lifecycle.confidence;
            total_importance += m.lifecycle.importance;
        }
        MemoryDetailedStats {
            total,
            tier_working,
            tier_episodic,
            tier_semantic,
            tier_procedural,
            avg_confidence: if total > 0 { total_confidence / total as f64 } else { 0.0 },
            avg_importance: if total > 0 { total_importance / total as f64 } else { 0.0 },
        }
    }

    pub fn quality_score(&self) -> f64 {
        if self.memories.is_empty() {
            return 0.0;
        }
        let now = Utc::now().timestamp();
        let total = self.memories.len() as f64;

        let avg_reward: f64 = self.memories.iter().map(|m| m.reward).sum::<f64>() / total;

        let unique_types = self.task_type_index.len() as f64;
        let diversity = (unique_types / 8.0).min(1.0);

        let max_access = self.memories.iter().map(|m| m.lifecycle.access_count).max().unwrap_or(1).max(1) as f64;
        let access_health: f64 = self.memories.iter().map(|m| {
            let age = (now - m.timestamp).max(0) as f64;
            let recency = (-age / 604800.0).exp();
            let access_ratio = (m.lifecycle.access_count as f64) / max_access;
            recency * 0.6 + access_ratio * 0.4
        }).sum::<f64>() / total;

        let score = avg_reward * 0.4 + diversity * 0.3 + access_health * 0.3;
        score.clamp(0.0, 1.0)
    }
}
