use std::collections::{HashMap, VecDeque};
use std::sync::atomic::AtomicBool;
use std::sync::RwLock;

use chrono::Utc;

use super::ReasoningBank;
use crate::core::nt_core_bank::iteration::Bm25Index;
#[cfg(feature = "e8-theory")]
use crate::core::nt_core_walsh::WalshMemoryIndex;

impl ReasoningBank {
    pub fn new(max_memories: usize) -> Self {
        Self {
            memories: VecDeque::with_capacity(max_memories),
            max_memories,
            task_type_index: HashMap::new(),
            bm25: RwLock::new(Bm25Index::empty()),
            bm25_dirty: AtomicBool::new(false),
            hypergraph: None,
            #[cfg(feature = "e8-theory")]
            wh_index: Some(WalshMemoryIndex::new()),
            wraps_layered_memory: false,
            layered: None,
        }
    }

    #[cfg(feature = "e8-theory")]
    pub fn new_without_wh(max_memories: usize) -> Self {
        let mut bank = Self::new(max_memories);
        bank.wh_index = None;
        bank
    }

    pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm_a == 0.0 && norm_b == 0.0 { return 1.0; }
        if norm_a == 0.0 || norm_b == 0.0 { return 0.0; }
        dot / (norm_a * norm_b)
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

    pub fn apply_tier_decay(&mut self, config: &crate::core::nt_core_bank::tier::LifecycleConfig) -> usize {
        let mut count = 0;
        for mem in &mut self.memories {
            let old = mem.lifecycle.importance;
            mem.lifecycle.importance = config.apply_decay(mem.tier, mem.lifecycle.importance, 1);
            if (mem.lifecycle.importance - old).abs() > 1e-10 {
                count += 1;
            }
        }
        count
    }
}
