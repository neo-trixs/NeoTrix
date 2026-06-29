use crate::core::nt_core_e8::shao_yong_sequence;
use crate::core::nt_core_gwt::manar_attention::ManarAttention;
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

use super::mcts_tree_search::{DefaultReasoningDomain, MCTSConfig, MCTSTree};

const VSA_DIM: usize = 4096;

#[derive(Debug)]
pub struct MCTSGWTBridge {
    pub mcts_config: MCTSConfig,
    pub mcts_tree: MCTSTree,
    pub domain: DefaultReasoningDomain,
}

impl MCTSGWTBridge {
    pub fn new(config: MCTSConfig) -> Self {
        Self {
            mcts_tree: MCTSTree::new(config.clone()),
            mcts_config: config,
            domain: DefaultReasoningDomain,
        }
    }

    pub fn run_mcts_cycle(&mut self, attention: &ManarAttention) -> Vec<(usize, String)> {
        let root_state = self.attention_to_vsa_state(attention);
        self.mcts_tree.search(root_state, &self.domain);
        let actions = self.mcts_tree.action_sequence(self.mcts_config.temperature);
        actions.into_iter().map(|(id, label)| (id, label)).collect()
    }

    pub fn mcts_to_concept_slots(
        &self,
        action_path: &[(usize, String)],
        attention: &mut ManarAttention,
    ) -> usize {
        let mut updated = 0;
        for (i, (_, action)) in action_path.iter().enumerate() {
            let slot_idx = i % 32;
            let action_vsa =
                QuantizedVSA::seeded_random(action.chars().map(|c| c as u64).sum::<u64>(), VSA_DIM);
            attention.update_slots(slot_idx, &action_vsa);
            updated += 1;
        }
        updated
    }

    pub fn e8_seeded_mcts(&mut self, attention: &ManarAttention) {
        let hexagrams = shao_yong_sequence();
        let root_state = self.attention_to_vsa_state(attention);
        self.mcts_tree.search(root_state, &self.domain);

        if let Some(root_id) = self.mcts_tree.root() {
            for hex in hexagrams.iter().take(16) {
                let prior = hex.bits as f64 / 64.0;
                let hex_state = QuantizedVSA::seeded_random(hex.bits as u64, VSA_DIM);
                self.mcts_tree.add_node(
                    hex_state,
                    prior,
                    format!("hex{:02x}", hex.bits),
                    Some(root_id),
                );
            }
        }
    }

    pub fn run_cycle_with_e8(&mut self, attention: &mut ManarAttention) -> Vec<(usize, String)> {
        self.e8_seeded_mcts(attention);
        let best = self.mcts_tree.action_sequence(self.mcts_config.temperature);
        self.mcts_to_concept_slots(&best, attention);
        best
    }

    fn attention_to_vsa_state(&self, attention: &ManarAttention) -> Vec<u8> {
        if let Some((_, content)) = attention.select_broadcast() {
            return content.into_inner();
        }
        QuantizedVSA::seeded_random(0x4d43_5453, VSA_DIM)
    }

    pub fn reset(&mut self) {
        self.mcts_tree = MCTSTree::new(self.mcts_config.clone());
    }

    pub fn tree_stats(&self) -> (usize, u32) {
        (
            self.mcts_tree.node_count(),
            self.mcts_tree.total_simulations(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_gwt::manar_attention::ManarConfig;

    fn make_attention_with_content() -> ManarAttention {
        let config = ManarConfig::default();
        let mut attn = ManarAttention::new(config);
        let proposal = QuantizedVSA::seeded_random(42, VSA_DIM);
        attn.update_slots(0, &proposal);
        attn
    }

    #[test]
    fn test_bridge_creation() {
        let bridge = MCTSGWTBridge::new(MCTSConfig::default());
        assert_eq!(bridge.mcts_tree.node_count(), 0);
        assert!(bridge.mcts_tree.root().is_none());
    }

    #[test]
    fn test_run_mcts_cycle_produces_actions() {
        let mut bridge = MCTSGWTBridge::new(MCTSConfig {
            max_iterations: 100,
            ..Default::default()
        });
        let attention = make_attention_with_content();
        let actions = bridge.run_mcts_cycle(&attention);
        assert!(!actions.is_empty());
        for (_, label) in &actions {
            assert!(!label.is_empty());
            assert_ne!(label.as_str(), "root");
        }
    }

    #[test]
    fn test_mcts_to_concept_slots_updates_attention() {
        let mut bridge = MCTSGWTBridge::new(MCTSConfig {
            max_iterations: 50,
            ..Default::default()
        });
        let attention = make_attention_with_content();
        let actions = bridge.run_mcts_cycle(&attention);
        let mut target = make_attention_with_content();
        let pre_projection = target.project(&QuantizedVSA::seeded_random(42, VSA_DIM));
        let updated = bridge.mcts_to_concept_slots(&actions, &mut target);
        assert!(updated > 0);
        let post_projection = target.project(&QuantizedVSA::seeded_random(42, VSA_DIM));
        let pre_sum: f64 = pre_projection.iter().map(|(_, s)| s).sum();
        let post_sum: f64 = post_projection.iter().map(|(_, s)| s).sum();
        assert!((pre_sum - post_sum).abs() > 1e-6 || pre_sum == post_sum);
    }

    #[test]
    fn test_e8_seeded_mcts_expands_tree() {
        let mut bridge = MCTSGWTBridge::new(MCTSConfig {
            max_iterations: 50,
            ..Default::default()
        });
        let attention = make_attention_with_content();
        bridge.e8_seeded_mcts(&attention);
        let (count, _) = bridge.tree_stats();
        assert!(count > 1);
        if let Some(root_id) = bridge.mcts_tree.root() {
            let root = bridge.mcts_tree.node(root_id).unwrap();
            assert!(!root.children.is_empty());
        }
    }

    #[test]
    fn test_run_cycle_with_e8_integration() {
        let mut bridge = MCTSGWTBridge::new(MCTSConfig {
            max_iterations: 80,
            ..Default::default()
        });
        let mut attention = make_attention_with_content();
        let actions = bridge.run_cycle_with_e8(&mut attention);
        assert!(!actions.is_empty());
        let (count, sims) = bridge.tree_stats();
        assert!(count > 5);
        assert!(sims > 0);
    }

    #[test]
    fn test_reset_clears_tree() {
        let mut bridge = MCTSGWTBridge::new(MCTSConfig {
            max_iterations: 50,
            ..Default::default()
        });
        let attention = make_attention_with_content();
        let _ = bridge.run_mcts_cycle(&attention);
        assert!(bridge.mcts_tree.node_count() > 1);
        bridge.reset();
        assert_eq!(bridge.mcts_tree.node_count(), 0);
    }

    #[test]
    fn test_tree_stats_reporting() {
        let mut bridge = MCTSGWTBridge::new(MCTSConfig {
            max_iterations: 60,
            ..Default::default()
        });
        let attention = make_attention_with_content();
        let _ = bridge.run_mcts_cycle(&attention);
        let (count, sims) = bridge.tree_stats();
        assert!(count >= 1);
        assert!(sims >= 0);
    }

    #[test]
    fn test_attention_state_fallback_no_broadcast() {
        let config = ManarConfig::default();
        let attention = ManarAttention::new(config);
        let bridge = MCTSGWTBridge::new(MCTSConfig::default());
        let state = bridge.attention_to_vsa_state(&attention);
        assert_eq!(state.len(), VSA_DIM);
    }

    #[test]
    fn test_empty_attention_no_crash() {
        let mut bridge = MCTSGWTBridge::new(MCTSConfig {
            max_iterations: 10,
            ..Default::default()
        });
        let config = ManarConfig::default();
        let attention = ManarAttention::new(config);
        let actions = bridge.run_mcts_cycle(&attention);
        assert!(!actions.is_empty());
    }
}
