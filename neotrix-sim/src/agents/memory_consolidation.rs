use crate::agents::memory_stream::{MemoryNode, MemoryStream};

/// Memory consolidation engine: transfers working memories to semantic storage,
/// forgets low-value memories, and strengthens frequently accessed ones.
pub struct MemoryConsolidator {
    /// Minimum importance threshold for consolidation
    pub consolidation_threshold: f32,
    /// Rate at which unaccessed memories lose strength (per tick)
    pub forgetting_rate: f32,
    /// Maximum number of consolidated memories to retain
    pub max_consolidated: usize,
}

impl Default for MemoryConsolidator {
    fn default() -> Self {
        Self {
            consolidation_threshold: 0.6,
            forgetting_rate: 0.001,
            max_consolidated: 500,
        }
    }
}

impl MemoryConsolidator {
    pub fn new(consolidation_threshold: f32, forgetting_rate: f32, max_consolidated: usize) -> Self {
        Self {
            consolidation_threshold,
            forgetting_rate,
            max_consolidated,
        }
    }

    /// Transfer high-importance working memories to semantic storage.
    /// Returns the indices of memories that were consolidated.
    pub fn consolidate(
        &self,
        working: &mut MemoryStream,
        semantic: &mut Vec<MemoryNode>,
        current_tick: u64,
    ) -> Vec<u64> {
        let mut consolidated_ids = Vec::new();
        let candidates: Vec<MemoryNode> = working
            .recent(working.len())
            .iter()
            .filter(|m| m.importance >= self.consolidation_threshold)
            .cloned()
            .collect();

        for mut mem in candidates {
            mem.last_accessed_tick = current_tick;
            let id = mem.id;
            let already_semantic = semantic.iter().any(|s| {
                s.agent_id == mem.agent_id
                    && s.description == mem.description
                    && s.kind == mem.kind
            });
            if !already_semantic {
                semantic.push(mem);
                consolidated_ids.push(id);
            }
        }

        if semantic.len() > self.max_consolidated {
            semantic.sort_by(|a, b| {
                b.importance
                    .partial_cmp(&a.importance)
                    .unwrap()
                    .then(b.last_accessed_tick.cmp(&a.last_accessed_tick))
            });
            semantic.truncate(self.max_consolidated);
        }

        consolidated_ids
    }

    /// Decide whether a memory should be forgotten.
    /// Returns true if the memory's effective strength has decayed below threshold.
    pub fn should_forget(&self, memory: &MemoryNode, current_tick: u64) -> bool {
        let age = current_tick.saturating_sub(memory.last_accessed_tick) as f32;
        let decay = (-self.forgetting_rate * age).exp();
        let effective_strength = memory.importance * decay;
        (effective_strength) < 0.1
    }

    /// Strengthen a memory based on recall count.
    /// More recalls = stronger memory trace (spaced repetition effect).
    pub fn strengthen(&self, memory: &mut MemoryNode, recall_count: u32) {
        let boost = (recall_count as f32 * 0.02).min(0.4);
        memory.importance = (memory.importance + boost).min(1.0);
    }

    /// Run a full consolidation + forgetting cycle.
    /// Returns the number of memories forgotten.
    pub fn run_cycle(
        &self,
        working: &mut MemoryStream,
        semantic: &mut Vec<MemoryNode>,
        current_tick: u64,
    ) -> usize {
        // 1. Consolidate important working memories to semantic
        let _consolidated = self.consolidate(working, semantic, current_tick);

        // 2. Strengthen semantic memories that were recently accessed
        for mem in semantic.iter_mut() {
            if current_tick.saturating_sub(mem.last_accessed_tick) < 10 {
                self.strengthen(mem, 1);
            }
        }

        // 3. Forget weak semantic memories
        let before = semantic.len();
        semantic.retain(|m| !self.should_forget(m, current_tick));
        before - semantic.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::memory_stream::MemoryKind;

    fn make_mem(importance: f32, tick: u64, _access_count: u32) -> MemoryNode {
        MemoryNode {
            id: 0,
            kind: MemoryKind::Observation,
            agent_id: "agent_0".into(),
            created_tick: tick,
            last_accessed_tick: tick,
            description: "test".into(),
            importance,
            keywords: vec![],
            citations: vec![],
            embedding: None,
        }
    }

    #[test]
    fn consolidate_transfers_high_importance() {
        let mut working = MemoryStream::new(100);
        working.add(make_mem(0.8, 0, 0));
        working.add(make_mem(0.3, 0, 0));
        let mut semantic = Vec::new();
        let cons = MemoryConsolidator::new(0.6, 0.001, 500);
        let ids = cons.consolidate(&mut working, &mut semantic, 10);
        assert_eq!(ids.len(), 1);
        assert_eq!(semantic.len(), 1);
    }

    #[test]
    fn should_forget_old_low_importance() {
        let cons = MemoryConsolidator::new(0.6, 0.01, 500);
        let old_mem = make_mem(0.1, 0, 0);
        assert!(cons.should_forget(&old_mem, 500));

        let important = make_mem(0.9, 0, 0);
        assert!(!cons.should_forget(&important, 500));
    }

    #[test]
    fn strengthen_increases_importance() {
        let cons = MemoryConsolidator::default();
        let mut mem = make_mem(0.5, 0, 0);
        cons.strengthen(&mut mem, 5);
        assert!(mem.importance > 0.5);
    }

    #[test]
    fn run_cycle_forgets_weak_memories() {
        let mut working = MemoryStream::new(100);
        working.add(make_mem(0.9, 0, 0));
        working.add(make_mem(0.05, 0, 0));
        let mut semantic = vec![make_mem(0.02, 0, 0)];
        let cons = MemoryConsolidator::new(0.6, 0.01, 500);
        let forgotten = cons.run_cycle(&mut working, &mut semantic, 500);
        assert!(forgotten >= 1);
    }

    #[test]
    fn consolidation_respects_max() {
        let mut working = MemoryStream::new(100);
        for i in 0..20 {
            working.add(make_mem(0.9, i, 0));
        }
        let mut semantic = Vec::new();
        let cons = MemoryConsolidator::new(0.5, 0.001, 5);
        cons.consolidate(&mut working, &mut semantic, 100);
        assert!(semantic.len() <= 5);
    }
}
