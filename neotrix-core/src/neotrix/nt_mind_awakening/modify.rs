/// SelfModify — 干预执行器 + checkpoint/rollback (A-08)
///
/// 将 CausalPredictor 生成的 InterventionOutcome 转换为实际脑状态修改。
/// 每次修改前 checkpoint(), 失败后 rollback()。
use crate::core::nt_core_cap::CapabilityVector;
use crate::neotrix::nt_mind::self_iterating::ReasoningBrain;

/// 可执行的干预操作
#[derive(Debug, Clone)]
pub enum Intervention {
    /// 调整能力向量某维度的值
    CapabilityDelta { index: usize, delta: f64 },
    /// 修改学习率
    LearningRate(f64),
    /// 添加扩展维度
    ExtensionAdd { name: String, values: [f64; 23] },
}

/// 干预结果
#[derive(Debug, Clone)]
pub struct InterventionResult {
    pub applied: bool,
    pub delta_phi: f64,
    pub rolled_back: bool,
    pub message: String,
}

/// 干预执行器
#[derive(Debug, Clone)]
pub struct SelfModify {
    checkpoint_capability: Option<CapabilityVector>,
    checkpoint_learning_rate: Option<f64>,
    checkpoint_custom_sources: Option<std::collections::HashMap<String, CapabilityVector>>,
}

impl SelfModify {
    pub fn new() -> Self {
        Self {
            checkpoint_capability: None,
            checkpoint_learning_rate: None,
            checkpoint_custom_sources: None,
        }
    }

    /// 保存当前脑状态快照用于 rollback
    pub fn checkpoint(&mut self, brain: &ReasoningBrain) {
        self.checkpoint_capability = Some(brain.capability.clone());
        self.checkpoint_learning_rate = Some(brain.learning_rate);
        self.checkpoint_custom_sources = Some(brain.custom_sources.clone());
    }

    /// 应用单个干预到 ReasoningBrain
    pub fn apply(
        &mut self,
        intervention: &Intervention,
        brain: &mut ReasoningBrain,
    ) -> Result<(), String> {
        match intervention {
            Intervention::CapabilityDelta { index, delta } => {
                if *index >= brain.capability.arr.len() {
                    return Err(format!(
                        "capability index {} out of bounds ({})",
                        index,
                        brain.capability.arr.len()
                    ));
                }
                brain.capability.arr[*index] =
                    (brain.capability.arr[*index] + delta).clamp(0.0, 1.0);
                brain.capability.normalize();
            }
            Intervention::LearningRate(lr) => {
                if *lr < 0.001 || *lr > 1.0 {
                    return Err(format!("learning rate {:.4} out of range [0.001, 1.0]", lr));
                }
                brain.learning_rate = *lr;
            }
            Intervention::ExtensionAdd { name, values } => {
                for &v in values.iter() {
                    brain.capability.add_extension_dim(name, v);
                }
            }
        }
        Ok(())
    }

    /// 应用多个干预（全部成功或全部回滚）
    pub fn apply_batch(
        &mut self,
        interventions: &[Intervention],
        brain: &mut ReasoningBrain,
    ) -> InterventionResult {
        self.checkpoint(brain);
        let before = brain.capability.arr.iter().sum::<f64>();
        for intervention in interventions {
            if let Err(e) = self.apply(intervention, brain) {
                self.rollback(brain);
                return InterventionResult {
                    applied: false,
                    delta_phi: 0.0,
                    rolled_back: true,
                    message: format!("intervention failed: {}, rolled back", e),
                };
            }
        }
        let after = brain.capability.arr.iter().sum::<f64>();
        InterventionResult {
            applied: true,
            delta_phi: after - before,
            rolled_back: false,
            message: format!(
                "applied {} interventions, Δcapability={:.4}",
                interventions.len(),
                after - before
            ),
        }
    }

    /// 回滚到最后一次 checkpoint
    pub fn rollback(&mut self, brain: &mut ReasoningBrain) -> bool {
        let mut rolled_back = false;
        if let Some(ref cap) = self.checkpoint_capability {
            brain.capability = cap.clone();
            rolled_back = true;
        }
        if let Some(lr) = self.checkpoint_learning_rate {
            brain.learning_rate = lr;
        }
        if let Some(ref sources) = self.checkpoint_custom_sources {
            brain.custom_sources = sources.clone();
        }
        self.checkpoint_capability = None;
        self.checkpoint_learning_rate = None;
        self.checkpoint_custom_sources = None;
        rolled_back
    }

    /// 提交 checkpoint（丢弃快照）
    pub fn commit(&mut self) {
        self.checkpoint_capability = None;
        self.checkpoint_learning_rate = None;
        self.checkpoint_custom_sources = None;
    }
}

impl Default for SelfModify {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_brain() -> ReasoningBrain {
        let mut brain = ReasoningBrain::new();
        brain.capability.arr = vec![0.5; 23];
        brain.learning_rate = 0.05;
        brain
    }

    #[test]
    fn test_checkpoint_rollback() {
        let mut brain = test_brain();
        let mut modifier = SelfModify::new();
        modifier.checkpoint(&brain);
        brain.capability.arr[0] = 0.9;
        modifier.rollback(&mut brain);
        assert!((brain.capability.arr[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_apply_capability_delta() {
        let mut brain = test_brain();
        let mut modifier = SelfModify::new();
        let result = modifier.apply_batch(
            &[Intervention::CapabilityDelta {
                index: 0,
                delta: 0.3,
            }],
            &mut brain,
        );
        assert!(result.applied);
        assert!((brain.capability.arr[0] - 0.8).abs() < 1e-4);
    }

    #[test]
    fn test_rollback_on_failure() {
        let mut brain = test_brain();
        let mut modifier = SelfModify::new();
        let original = brain.capability.arr[0];
        let result = modifier.apply_batch(
            &[
                Intervention::CapabilityDelta {
                    index: 0,
                    delta: 0.3,
                },
                Intervention::LearningRate(2.0), // invalid — out of range
            ],
            &mut brain,
        );
        assert!(!result.applied);
        assert!(result.rolled_back);
        assert!((brain.capability.arr[0] - original).abs() < 1e-6);
    }

    #[test]
    fn test_bounds_checking() {
        let mut brain = test_brain();
        let mut modifier = SelfModify::new();
        let result = modifier.apply_batch(
            &[Intervention::CapabilityDelta {
                index: 99,
                delta: 0.1,
            }],
            &mut brain,
        );
        assert!(!result.applied);
        assert!(result.rolled_back);
    }

    #[test]
    fn test_commit() {
        let mut brain = test_brain();
        let mut modifier = SelfModify::new();
        modifier.checkpoint(&brain);
        brain.capability.arr[0] = 0.9;
        modifier.commit();
        modifier.rollback(&mut brain);
        assert!((brain.capability.arr[0] - 0.9).abs() < 1e-6); // no rollback since committed
    }
}
