//! DPOStage — Direct Preference Optimization for SEAL pipeline
//!
//! Learns from preference pairs (chosen vs rejected responses):
//! 1. Collects preference pairs from KB (ConversationRecord outcomes)
//! 2. Computes DPO loss: -E[log σ(β * (r_chosen - r_rejected))]
//! 3. Updates capability vector based on pairwise preference signal
//!
//! Reference: Direct Preference Optimization (Rafailov et al., 2023)

use std::collections::VecDeque;
use serde::{Serialize, Deserialize};
use super::pipeline::StageResult;

/// Maximum number of preference pairs to store.
pub const DPO_BUFFER_SIZE: usize = 100;

/// Beta parameter for DPO (inverse temperature of softmax).
pub const DPO_BETA: f64 = 0.1;

/// A single preference pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferencePair {
    pub task: String,
    pub chosen_mode: u8,
    pub rejected_mode: u8,
    pub chosen_reward: f64,
    pub rejected_reward: f64,
    pub timestamp: u64,
}

/// DPO buffer storing preference pairs for training.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpoBuffer {
    pub pairs: VecDeque<PreferencePair>,
    pub max_size: usize,
}

impl Default for DpoBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl DpoBuffer {
    pub fn new() -> Self {
        Self {
            pairs: VecDeque::with_capacity(DPO_BUFFER_SIZE),
            max_size: DPO_BUFFER_SIZE,
        }
    }

    pub fn push(&mut self, pair: PreferencePair) {
        if self.pairs.len() >= self.max_size {
            self.pairs.pop_front();
        }
        self.pairs.push_back(pair);
    }

    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    pub fn clear(&mut self) {
        self.pairs.clear();
    }
}

/// DPO Stage for the SEAL pipeline.
///
/// Processes preference pairs to update capability vectors via
/// the DPO loss signal: -log σ(β * (score_chosen - score_rejected)).
#[derive(Debug, Clone)]
pub struct DpoStage {
    pub buffer: DpoBuffer,
    pub beta: f64,
    pub total_updates: u64,
}

impl Default for DpoStage {
    fn default() -> Self {
        Self::new()
    }
}

impl DpoStage {
    pub fn new() -> Self {
        Self {
            buffer: DpoBuffer::new(),
            beta: DPO_BETA,
            total_updates: 0,
        }
    }

    /// Process the DPO stage: compute preference loss and return stage result.
    pub fn process(
        &mut self,
        preference_pairs: Vec<PreferencePair>,
        current_reward: f64,
    ) -> (StageResult, f64) {
        let result = StageResult::new("dpo_stage");

        if preference_pairs.is_empty() {
            return (result, current_reward);
        }

        // Store incoming pairs
        for pair in &preference_pairs {
            self.buffer.push(pair.clone());
        }

        // Compute DPO loss
        let dpo_loss = self.compute_dpo_loss();
        self.total_updates += 1;

        // DPO reward modifier: penalize if preference signal is violated
        let penalty = (dpo_loss * self.beta).min(0.5);
        let adjusted_reward = current_reward * (1.0 - penalty);

        (result, adjusted_reward)
    }

    /// Compute DPO loss over the current buffer.
    fn compute_dpo_loss(&self) -> f64 {
        if self.buffer.is_empty() {
            return 0.0;
        }

        let n = self.buffer.len() as f64;
        let total_loss: f64 = self.buffer.pairs.iter().map(|pair| {
            let reward_margin = pair.chosen_reward - pair.rejected_reward;
            // DPO loss: -log σ(β * margin)
            let logit = self.beta * reward_margin;
            // Numerical stable -log(sigmoid(x))
            if logit > 0.0 {
                (-logit).exp().ln_1p()
            } else {
                logit.exp().ln_1p() - logit
            }
        }).sum();
        total_loss / n
    }

    /// Sample a batch of preference pairs for training.
    pub fn sample_batch(&self, batch_size: usize) -> Vec<&PreferencePair> {
        let size = batch_size.min(self.buffer.len());
        self.buffer.pairs.iter().rev().take(size).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dpo_buffer_push_and_len() {
        let mut buffer = DpoBuffer::new();
        assert!(buffer.is_empty());
        buffer.push(PreferencePair {
            task: "test".into(), chosen_mode: 0, rejected_mode: 1,
            chosen_reward: 1.0, rejected_reward: 0.0, timestamp: 1,
        });
        assert_eq!(buffer.len(), 1);
    }

    #[test]
    fn test_dpo_buffer_max_size() {
        let mut buffer = DpoBuffer::new();
        buffer.max_size = 3;
        for i in 0..5 {
            buffer.push(PreferencePair {
                task: "t".into(), chosen_mode: i, rejected_mode: i + 1,
                chosen_reward: 1.0, rejected_reward: 0.0, timestamp: i as u64,
            });
        }
        assert_eq!(buffer.len(), 3);
    }

    #[test]
    fn test_dpo_loss_positive_margin() {
        let mut stage = DpoStage::new();
        stage.buffer.push(PreferencePair {
            task: "code_review".into(), chosen_mode: 5, rejected_mode: 10,
            chosen_reward: 0.9, rejected_reward: 0.2, timestamp: 1,
        });
        let loss = stage.compute_dpo_loss();
        assert!(loss >= 0.0);
        assert!(loss < 1.0, "positive margin should give small loss");
    }

    #[test]
    fn test_dpo_loss_negative_margin() {
        let mut stage = DpoStage::new();
        stage.buffer.push(PreferencePair {
            task: "test".into(), chosen_mode: 0, rejected_mode: 1,
            chosen_reward: 0.2, rejected_reward: 0.9, timestamp: 1,
        });
        let loss = stage.compute_dpo_loss();
        assert!(loss >= 0.0);
        assert!(loss > 0.5, "negative margin should give larger loss");
    }

    #[test]
    fn test_process_with_empty_pairs() {
        let mut stage = DpoStage::new();
        let (result, reward) = stage.process(vec![], 1.0);
        assert!((reward - 1.0).abs() < 1e-9);
        assert!(!result.stage_name.is_empty());
    }

    #[test]
    fn test_process_adjusts_reward() {
        let mut stage = DpoStage::new();
        let pairs = vec![
            PreferencePair {
                task: "bug".into(), chosen_mode: 0, rejected_mode: 1,
                chosen_reward: 0.9, rejected_reward: 0.9, timestamp: 1,
            },
            PreferencePair {
                task: "bug".into(), chosen_mode: 2, rejected_mode: 3,
                chosen_reward: 0.1, rejected_reward: 0.8, timestamp: 2,
            },
        ];
        let (_, adjusted) = stage.process(pairs, 1.0);
        assert!(adjusted < 1.0, "negative preferences should reduce reward");
    }

    #[test]
    fn test_sample_batch_returns_recent() {
        let mut stage = DpoStage::new();
        for i in 0..10 {
            stage.buffer.push(PreferencePair {
                task: "t".into(), chosen_mode: i, rejected_mode: i + 10,
                chosen_reward: 1.0, rejected_reward: 0.0, timestamp: i as u64,
            });
        }
        let batch = stage.sample_batch(3);
        assert_eq!(batch.len(), 3);
    }

    #[test]
    fn test_total_updates_tracked() {
        let mut stage = DpoStage::new();
        assert_eq!(stage.total_updates, 0);
        let pair = PreferencePair {
            task: "t".into(), chosen_mode: 0, rejected_mode: 1,
            chosen_reward: 0.9, rejected_reward: 0.1, timestamp: 1,
        };
        stage.process(vec![pair], 1.0);
        assert_eq!(stage.total_updates, 1);
    }
}
