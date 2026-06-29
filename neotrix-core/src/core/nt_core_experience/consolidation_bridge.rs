use crate::core::nt_core_experience::gate::AttentionGate;
use crate::core::nt_core_hcube::ebbinghaus_decay::{DecayConfig, EbbinghausDecay, MemoryTrace};
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_REPLAY_INTERVAL_TURNS: usize = 50;
const DEFAULT_CONSOLIDATION_BATCH: usize = 10;
const DEFAULT_FREQ_ALPHA: f64 = 0.3;

#[derive(Debug, Clone)]
pub struct BridgeConfig {
    pub replay_interval_turns: usize,
    pub consolidation_batch: usize,
    pub freq_alpha: f64,
    pub ebbinghaus_base_decay: f64,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            replay_interval_turns: DEFAULT_REPLAY_INTERVAL_TURNS,
            consolidation_batch: DEFAULT_CONSOLIDATION_BATCH,
            freq_alpha: DEFAULT_FREQ_ALPHA,
            ebbinghaus_base_decay: 0.5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConsolidatedMemory {
    pub knowledge_id: u64,
    pub final_utility: f64,
    pub replay_count: u32,
    pub vsa_vector: Vec<u8>,
    pub timestamp_ns: u64,
    pub access_count: u32,
    pub trace_id: u64,
}

impl ConsolidatedMemory {
    pub fn new(knowledge_id: u64, utility: f64, vsa: Vec<u8>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self {
            knowledge_id,
            final_utility: utility,
            replay_count: 0,
            vsa_vector: vsa,
            timestamp_ns: now,
            access_count: 0,
            trace_id: 0,
        }
    }
}

pub struct ConsolidationBridgeV2 {
    config: BridgeConfig,
    turn_counter: usize,
    pub consolidated: Vec<ConsolidatedMemory>,
    pub total_replays: usize,
    pub total_pruned: usize,
    pub last_replay_ns: u64,
    ebbinghaus: EbbinghausDecay,
}

impl ConsolidationBridgeV2 {
    pub fn new(config: BridgeConfig) -> Self {
        Self {
            config,
            turn_counter: 0,
            consolidated: Vec::new(),
            total_replays: 0,
            total_pruned: 0,
            last_replay_ns: 0,
            ebbinghaus: EbbinghausDecay::new(DecayConfig::default()),
        }
    }

    pub fn tick(&mut self) -> bool {
        self.turn_counter += 1;
        if self.turn_counter >= self.config.replay_interval_turns {
            self.turn_counter = 0;
            true
        } else {
            false
        }
    }

    pub fn feed_gated(&mut self, gate: &mut AttentionGate) -> Vec<ConsolidatedMemory> {
        let now_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        self.last_replay_ns = now_ns;

        let top_data: Vec<(u64, f64, Vec<u8>)> = {
            let top = gate.top_replay(self.config.consolidation_batch);
            top.into_iter()
                .map(|item| {
                    let score = gate.replay_score(item);
                    (item.id, score, item.vsa_vector.clone())
                })
                .collect()
        };

        let mut new_memories = Vec::new();
        for (id, replay_score, vsa) in top_data {
            if gate.should_consolidate(replay_score) {
                new_memories.push(ConsolidatedMemory::new(id, replay_score, vsa));
                gate.record_access(id);
            }
        }

        let _drained = gate.drain_consolidated();

        for mem in &new_memories {
            self.ebbinghaus.add_memory(
                mem.vsa_vector.clone(),
                &format!("gate_consolidated_{}", mem.knowledge_id),
            );
            self.consolidated.push(mem.clone());
        }

        self.total_replays += new_memories.len();
        self.prune_low_utility(0.2);
        new_memories
    }

    pub fn prune_low_utility(&mut self, threshold: f64) {
        let before = self.consolidated.len();
        self.consolidated.retain(|m| m.final_utility >= threshold);
        self.total_pruned += before - self.consolidated.len();
    }

    pub fn memory_traces(&self) -> &[MemoryTrace] {
        &[]
    }

    pub fn stats(&self) -> BridgeV2Stats {
        BridgeV2Stats {
            total_consolidated: self.consolidated.len(),
            total_replays: self.total_replays,
            total_pruned: self.total_pruned,
            turn_counter: self.turn_counter,
            replay_interval: self.config.replay_interval_turns,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BridgeV2Stats {
    pub total_consolidated: usize,
    pub total_replays: usize,
    pub total_pruned: usize,
    pub turn_counter: usize,
    pub replay_interval: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_experience::UtilitySignal;
    use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

    fn make_gate() -> AttentionGate {
        AttentionGate::new(QuantizedVSA::random_vector())
            .with_noise_threshold(0.0)
            .with_consolidation_threshold(0.5)
    }

    #[test]
    fn test_tick_triggers_at_interval() {
        let mut bridge = ConsolidationBridgeV2::new(BridgeConfig::default());
        for _ in 0..49 {
            assert!(!bridge.tick());
        }
        assert!(bridge.tick());
    }

    #[test]
    fn test_feed_gated_returns_consolidated() {
        let mut gate = make_gate();
        let v = QuantizedVSA::random_vector();
        gate.gate(1, &v, vec![UtilitySignal::Importance(0.9)]);
        gate.gate(2, &v, vec![UtilitySignal::Importance(0.3)]);

        let mut bridge = ConsolidationBridgeV2::new(BridgeConfig::default());
        let result = bridge.feed_gated(&mut gate);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].knowledge_id, 1);
    }

    #[test]
    fn test_prune_low_utility() {
        let mut bridge = ConsolidationBridgeV2::new(BridgeConfig::default());
        bridge
            .consolidated
            .push(ConsolidatedMemory::new(1, 0.9, vec![0; 512]));
        bridge
            .consolidated
            .push(ConsolidatedMemory::new(2, 0.1, vec![0; 512]));
        bridge.prune_low_utility(0.5);
        assert_eq!(bridge.consolidated.len(), 1);
        assert_eq!(bridge.consolidated[0].knowledge_id, 1);
    }

    #[test]
    fn test_stats_tracking() {
        let bridge = ConsolidationBridgeV2::new(BridgeConfig::default());
        let s = bridge.stats();
        assert_eq!(s.total_consolidated, 0);
        assert_eq!(s.total_replays, 0);
    }

    #[test]
    fn test_replay_count_tracks() {
        let mut gate = make_gate();
        let v = QuantizedVSA::random_vector();
        gate.gate(1, &v, vec![UtilitySignal::Importance(0.9)]);
        gate.record_access(1);
        gate.record_access(1);

        let mut bridge = ConsolidationBridgeV2::new(BridgeConfig::default());
        let result = bridge.feed_gated(&mut gate);
        assert!(!result.is_empty());
        assert_eq!(result[0].replay_count, 0);
    }

    #[test]
    fn test_empty_gate_produces_empty_result() {
        let mut gate = make_gate();
        let mut bridge = ConsolidationBridgeV2::new(BridgeConfig::default());
        let result = bridge.feed_gated(&mut gate);
        assert!(result.is_empty());
    }
}
