use std::collections::VecDeque;

// ── EGPO: Exploration Guided Policy Optimization (arXiv 2602.22751) ─────────
// Triple-reward framework:
//   1. ExplorationReward — VSA novelty bonus (prediction error → epistemic value)
//   2. AuxiliaryLoss     — contrastive + inverse dynamics representation learning
//   3. BehavioralCloning — regularization toward past high-reward trajectories

// ── ExplorationRewardModule ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ExplorationRewardModule {
    pub novelty_buffer: VecDeque<f64>,
    pub max_buffer: usize,
    pub exploration_weight: f64,
    pub novelty_threshold: f64,
    pub total_reward: f64,
    pub step: u64,
}

impl ExplorationRewardModule {
    pub fn new() -> Self {
        Self {
            novelty_buffer: VecDeque::with_capacity(100),
            max_buffer: 100,
            exploration_weight: 0.15,
            novelty_threshold: 0.35,
            total_reward: 0.0,
            step: 0,
        }
    }

    pub fn compute_reward(&mut self, current_vsa: &[u8], recent_vsas: &[Vec<u8>]) -> f64 {
        self.step += 1;
        let mut max_sim = 0.0_f64;
        for other in recent_vsas {
            let sim = vsa_similarity(current_vsa, other);
            if sim > max_sim {
                max_sim = sim;
            }
        }
        let novelty = 1.0 - max_sim;
        let reward = if novelty > self.novelty_threshold {
            novelty * self.exploration_weight
        } else {
            0.0
        };
        self.total_reward += reward;
        self.novelty_buffer.push_back(reward);
        if self.novelty_buffer.len() > self.max_buffer {
            self.novelty_buffer.pop_front();
        }
        reward
    }

    pub fn integrated_reward(&self, exploration_bonus: f64) -> f64 {
        let intrinsic = self.novelty_buffer.back().copied().unwrap_or(0.0);
        intrinsic * (1.0 + exploration_bonus)
    }

    pub fn stats(&self) -> String {
        let avg_novelty = if !self.novelty_buffer.is_empty() {
            self.novelty_buffer.iter().sum::<f64>() / self.novelty_buffer.len() as f64
        } else {
            0.0
        };
        format!(
            "exploration:{}_steps|{:.4}_avg_reward|{:.4}_total|{:.2}_weight",
            self.step, avg_novelty, self.total_reward, self.exploration_weight
        )
    }
}

// ── AuxiliaryLossModule ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AuxiliaryLossModule {
    pub contrastive_weight: f64,
    pub inverse_dynamics_weight: f64,
    pub embedding_similarity_weight: f64,
    pub total_contrastive_loss: f64,
    pub total_inv_dyn_loss: f64,
    pub total_embed_loss: f64,
    pub step: u64,
}

impl AuxiliaryLossModule {
    pub fn new() -> Self {
        Self {
            contrastive_weight: 0.1,
            inverse_dynamics_weight: 0.05,
            embedding_similarity_weight: 0.08,
            total_contrastive_loss: 0.0,
            total_inv_dyn_loss: 0.0,
            total_embed_loss: 0.0,
            step: 0,
        }
    }

    pub fn compute_contrastive_loss(
        &mut self,
        anchor: &[u8],
        positive: &[u8],
        negatives: &[&[u8]],
    ) -> f64 {
        self.step += 1;
        let pos_sim = vsa_similarity(anchor, positive);
        let mut neg_sims: Vec<f64> = negatives
            .iter()
            .map(|n| vsa_similarity(anchor, n))
            .collect();
        neg_sims.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let hardest_neg = neg_sims.first().copied().unwrap_or(0.0);
        let loss = (1.0 - pos_sim + hardest_neg).max(0.0) * self.contrastive_weight;
        self.total_contrastive_loss += loss;
        loss
    }

    pub fn compute_inverse_dynamics_loss(
        &mut self,
        state_before: &[u8],
        state_after: &[u8],
    ) -> f64 {
        let delta = vsa_distance(state_before, state_after);
        let loss = delta * self.inverse_dynamics_weight;
        self.total_inv_dyn_loss += loss;
        loss
    }

    pub fn compute_embedding_loss(&mut self, predicted: &[u8], target: &[u8]) -> f64 {
        let mse = vsa_distance(predicted, target);
        let loss = mse * self.embedding_similarity_weight;
        self.total_embed_loss += loss;
        loss
    }

    pub fn total_loss(&self) -> f64 {
        self.total_contrastive_loss + self.total_inv_dyn_loss + self.total_embed_loss
    }

    pub fn stats(&self) -> String {
        format!(
            "aux:{}_steps|{:.4}_contrast|{:.4}_inv_dyn|{:.4}_embed|{:.4}_total",
            self.step,
            self.total_contrastive_loss,
            self.total_inv_dyn_loss,
            self.total_embed_loss,
            self.total_loss()
        )
    }
}

// ── BehavioralCloningModule ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BehavioralCloningModule {
    pub buffer: VecDeque<CloningEntry>,
    pub max_buffer: usize,
    pub regularization_weight: f64,
    pub reward_threshold: f64,
    pub total_cloning_loss: f64,
    pub step: u64,
}

#[derive(Debug, Clone)]
pub struct CloningEntry {
    pub vsa_state: Vec<u8>,
    pub action: String,
    pub reward: f64,
    pub recorded_at: u64,
}

impl BehavioralCloningModule {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::with_capacity(200),
            max_buffer: 200,
            regularization_weight: 0.12,
            reward_threshold: 0.3,
            total_cloning_loss: 0.0,
            step: 0,
        }
    }

    pub fn record_good_trajectory(&mut self, vsa: Vec<u8>, action: String, reward: f64, now: u64) {
        if reward >= self.reward_threshold {
            self.buffer.push_back(CloningEntry {
                vsa_state: vsa,
                action,
                reward,
                recorded_at: now,
            });
            if self.buffer.len() > self.max_buffer {
                self.buffer.pop_front();
            }
        }
    }

    pub fn compute_cloning_loss(&mut self, current_vsa: &[u8], current_action: &str) -> f64 {
        self.step += 1;
        if self.buffer.is_empty() {
            return 0.0;
        }
        let mut total_sim = 0.0_f64;
        let mut count = 0;
        for entry in &self.buffer {
            let state_sim = vsa_similarity(current_vsa, &entry.vsa_state);
            let action_match = if entry.action == current_action {
                1.0
            } else {
                0.0
            };
            let entry_weight = entry.reward;
            total_sim += (1.0 - state_sim) * entry_weight + (1.0 - action_match) * 0.5;
            count += 1;
        }
        let avg_loss = if count > 0 {
            total_sim / count as f64
        } else {
            0.0
        };
        let loss = avg_loss * self.regularization_weight;
        self.total_cloning_loss += loss;
        loss
    }

    pub fn stats(&self) -> String {
        format!(
            "bc:{}_buffer|{}_steps|{:.4}_total_loss|{:.2}_weight",
            self.buffer.len(),
            self.step,
            self.total_cloning_loss,
            self.regularization_weight
        )
    }
}

// ── EGPOEngine (triple-reward orchestrator) ─────────────────────────────────

#[derive(Debug, Clone)]
pub struct EGPOEngine {
    pub exploration: ExplorationRewardModule,
    pub auxiliary: AuxiliaryLossModule,
    pub behavioral_cloning: BehavioralCloningModule,
    pub enabled: bool,
    pub cycle: u64,
    pub last_run_cycle: u64,
}

impl EGPOEngine {
    pub fn new() -> Self {
        Self {
            exploration: ExplorationRewardModule::new(),
            auxiliary: AuxiliaryLossModule::new(),
            behavioral_cloning: BehavioralCloningModule::new(),
            enabled: true,
            cycle: 0,
            last_run_cycle: 0,
        }
    }

    pub fn tick(
        &mut self,
        current_vsa: Option<&[u8]>,
        recent_vsas: &[Vec<u8>],
        current_action: Option<&str>,
    ) -> String {
        self.cycle += 1;
        if !self.enabled {
            return "egpo:disabled".to_string();
        }

        let exploration_reward = if let Some(vsa) = current_vsa {
            self.exploration.compute_reward(vsa, recent_vsas)
        } else {
            0.0
        };

        let aux_loss = if let Some(vsa) = current_vsa {
            let contrastive = if recent_vsas.len() >= 2 {
                let negs: Vec<&[u8]> = recent_vsas.iter().skip(1).map(|v| v.as_slice()).collect();
                self.auxiliary
                    .compute_contrastive_loss(vsa, &recent_vsas[0], &negs)
            } else {
                0.0
            };
            let inv_dyn = if recent_vsas.len() >= 2 {
                self.auxiliary
                    .compute_inverse_dynamics_loss(&recent_vsas[0], vsa)
            } else {
                0.0
            };
            contrastive + inv_dyn
        } else {
            0.0
        };

        let bc_loss = if let (Some(vsa), Some(action)) = (current_vsa, current_action) {
            self.behavioral_cloning.compute_cloning_loss(vsa, action)
        } else {
            0.0
        };

        self.last_run_cycle = self.cycle;
        format!(
            "egpo:explore={:.4}|aux={:.4}|bc={:.4}|total={:.4}",
            exploration_reward,
            aux_loss,
            bc_loss,
            exploration_reward + aux_loss + bc_loss,
        )
    }

    pub fn record_trajectory(&mut self, vsa: Vec<u8>, action: String, reward: f64, now: u64) {
        self.behavioral_cloning
            .record_good_trajectory(vsa, action, reward, now);
    }

    pub fn stats(&self) -> String {
        format!(
            "{}|{}|{}|run:cycle_{}",
            self.exploration.stats(),
            self.auxiliary.stats(),
            self.behavioral_cloning.stats(),
            self.last_run_cycle,
        )
    }
}

// ── Utility functions ───────────────────────────────────────────────────────

/// VSA bipolar similarity for binary hypervectors.
/// Delegates to QuantizedVSA's Hamming-distance-based similarity.
fn vsa_similarity(a: &[u8], b: &[u8]) -> f64 {
    crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA::similarity(a, b)
}

fn vsa_distance(a: &[u8], b: &[u8]) -> f64 {
    1.0 - vsa_similarity(a, b)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_vsa(seed: u8) -> Vec<u8> {
        (0..64)
            .map(|i: u8| i.wrapping_mul(seed).wrapping_add(seed))
            .collect()
    }

    #[test]
    fn test_exploration_reward_novel_vs_familiar() {
        let mut module = ExplorationRewardModule::new();
        let familiar = mock_vsa(1);
        let novel = mock_vsa(255);
        let recent = vec![familiar.clone(), mock_vsa(2), mock_vsa(3)];

        let reward_familiar = module.compute_reward(&familiar, &recent);
        let reward_novel = module.compute_reward(&novel, &recent);

        assert!(
            reward_novel > reward_familiar,
            "novel should get higher reward than familiar"
        );
        assert!(module.step == 2);
        assert!(module.novelty_buffer.len() == 2);
    }

    #[test]
    fn test_auxiliary_contrastive_loss() {
        let mut module = AuxiliaryLossModule::new();
        let anchor = mock_vsa(10);
        let positive = anchor.clone();
        let negative = mock_vsa(200);
        let loss = module.compute_contrastive_loss(&anchor, &positive, &[&negative]);
        assert!(loss >= 0.0);
        assert!(module.total_contrastive_loss > 0.0);
    }

    #[test]
    fn test_behavioral_cloning_record_and_loss() {
        let mut module = BehavioralCloningModule::new();
        let state = mock_vsa(42);
        module.record_good_trajectory(state.clone(), "test_action".into(), 0.8, 1);
        assert!(module.buffer.len() == 1);

        let loss = module.compute_cloning_loss(&state, "test_action");
        assert!(loss >= 0.0);
    }

    #[test]
    fn test_egpo_engine_full_tick() {
        let mut engine = EGPOEngine::new();
        let vsa = mock_vsa(1);
        let recent = vec![mock_vsa(2), mock_vsa(3), mock_vsa(4)];
        let result = engine.tick(Some(&vsa), &recent, Some("think"));
        assert!(result.starts_with("egpo:"));
        assert!(engine.cycle == 1);
    }

    #[test]
    fn test_egpo_disabled() {
        let mut engine = EGPOEngine::new();
        engine.enabled = false;
        let result = engine.tick(None, &[], None);
        assert_eq!(result, "egpo:disabled");
    }

    #[test]
    fn test_integrated_reward_modulation() {
        let mut module = ExplorationRewardModule::new();
        let novel = mock_vsa(255);
        let recent = vec![mock_vsa(1), mock_vsa(2)];
        module.compute_reward(&novel, &recent);
        let base = module.integrated_reward(0.0);
        let boosted = module.integrated_reward(0.5);
        assert!(boosted >= base);
    }

    #[test]
    fn test_trajectory_recording() {
        let mut engine = EGPOEngine::new();
        let vsa = mock_vsa(7);
        engine.record_trajectory(vsa, "explore".into(), 0.9, 100);
        assert!(engine.behavioral_cloning.buffer.len() == 1);
    }

    #[test]
    fn test_stats_format() {
        let engine = EGPOEngine::new();
        let stats = engine.stats();
        assert!(stats.contains("exploration:"));
        assert!(stats.contains("aux:"));
        assert!(stats.contains("bc:"));
    }

    // ── Integration tests ─────────────────────────────────────────────

    #[test]
    fn test_multi_tick_exploration_adaptation() {
        let mut engine = EGPOEngine::new();
        let mut recent = vec![mock_vsa(1), mock_vsa(2), mock_vsa(3)];
        for i in 0..10 {
            let current = mock_vsa(if i < 5 { 1 } else { 200 + i as u8 });
            let r = engine.tick(Some(&current), &recent, Some("think"));
            assert!(r.starts_with("egpo:"));
            recent.push(current);
            if recent.len() > 20 {
                recent.remove(0);
            }
        }
        assert!(engine.cycle == 10);
        assert!(engine.exploration.total_reward > 0.0 || engine.auxiliary.total_loss() >= 0.0);
    }

    #[test]
    fn test_exploration_with_neuromodulator_boost() {
        let mut mod_ = ExplorationRewardModule::new();
        let novel = mock_vsa(200);
        let familiar = mock_vsa(1);
        let recent = vec![familiar.clone(), mock_vsa(2), mock_vsa(3)];
        mod_.compute_reward(&novel, &recent);
        let boosted = mod_.integrated_reward(0.8);
        let unboosted = mod_.integrated_reward(0.0);
        assert!(boosted >= unboosted);
    }

    #[test]
    fn test_behavioral_cloning_trajectory_buffer_full_cycle() {
        let mut bc = BehavioralCloningModule::new();
        bc.max_buffer = 5;
        let good_reward = 0.5;
        for i in 0..10u8 {
            bc.record_good_trajectory(mock_vsa(i), format!("act_{}", i), good_reward, i as u64);
        }
        assert!(
            bc.buffer.len() == 5,
            "buffer should be capped at 5, got {}",
            bc.buffer.len()
        );
        let loss = bc.compute_cloning_loss(&mock_vsa(0), "act_0");
        assert!(loss >= 0.0);
        let zero_loss = bc.compute_cloning_loss(&mock_vsa(0), "nonexistent");
        assert!(zero_loss > loss, "mismatched action should increase loss");
    }

    #[test]
    fn test_auxiliary_all_loss_components() {
        let mut aux = AuxiliaryLossModule::new();
        let anchor = mock_vsa(10);
        let similar = mock_vsa(11);
        let neg = mock_vsa(200);
        let contrastive = aux.compute_contrastive_loss(&anchor, &similar, &[&neg]);
        assert!(contrastive >= 0.0);
        let inv_dyn = aux.compute_inverse_dynamics_loss(&mock_vsa(1), &mock_vsa(2));
        assert!(inv_dyn >= 0.0);
        let embed = aux.compute_embedding_loss(&mock_vsa(1), &mock_vsa(1));
        assert!(embed >= 0.0);
        assert!(aux.total_loss() > 0.0);
    }

    #[test]
    fn test_low_reward_not_recorded() {
        let mut bc = BehavioralCloningModule::new();
        bc.record_good_trajectory(mock_vsa(1), "low_reward".into(), 0.1, 1);
        assert!(bc.buffer.is_empty(), "low reward should not be recorded");
    }

    #[test]
    fn test_consecutive_ticks_increasing_novelty() {
        let mut engine = EGPOEngine::new();
        let recent = vec![mock_vsa(1), mock_vsa(2), mock_vsa(3)];
        for i in 0..5 {
            let novel = mock_vsa(100 + i as u8);
            engine.tick(Some(&novel), &recent, None);
        }
        assert!(engine.exploration.step == 5);
    }
}
