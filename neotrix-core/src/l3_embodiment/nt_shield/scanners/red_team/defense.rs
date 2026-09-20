use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::attack::AttackStrategy;
use super::result::VulnerabilityCategory;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DefenseMechanism {
    InputFilter,
    OutputFilter,
    SystemPromptGuard,
    ContentModeration,
    JailbreakDetection,
    TokenLimit,
    InstructionHierarchy,
    ContextIsolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenseProfile {
    pub target: String,
    pub blocked: HashMap<DefenseMechanism, Vec<VulnerabilityCategory>>,
    pub bypassed: HashMap<DefenseMechanism, Vec<VulnerabilityCategory>>,
    pub effectiveness: f64,
}

impl DefenseProfile {
    pub fn new(target: &str) -> Self {
        Self {
            target: target.to_string(),
            blocked: HashMap::new(),
            bypassed: HashMap::new(),
            effectiveness: 0.0,
        }
    }

    pub fn record_blocked(&mut self, mechanism: DefenseMechanism, category: VulnerabilityCategory) {
        self.blocked
            .entry(mechanism)
            .or_default()
            .push(category);
        self.recalculate_effectiveness();
    }

    pub fn record_bypassed(
        &mut self,
        mechanism: DefenseMechanism,
        category: VulnerabilityCategory,
    ) {
        self.bypassed
            .entry(mechanism)
            .or_default()
            .push(category);
        self.recalculate_effectiveness();
    }

    fn recalculate_effectiveness(&mut self) {
        let total_blocked: usize = self.blocked.values().map(|v| v.len()).sum();
        let total_bypassed: usize = self.bypassed.values().map(|v| v.len()).sum();
        let total = total_blocked + total_bypassed;
        self.effectiveness = if total == 0 {
            0.0
        } else {
            total_blocked as f64 / total as f64
        };
    }

    pub fn is_vulnerable_to(&self, category: &VulnerabilityCategory) -> bool {
        self.bypassed
            .values()
            .any(|cats| cats.contains(category))
    }
}

pub struct DefenseProfiler;

impl DefenseProfiler {
    pub fn profile(target: &str) -> DefenseProfile {
        let mut profile = DefenseProfile::new(target);
        let mechanisms = [
            DefenseMechanism::InputFilter,
            DefenseMechanism::OutputFilter,
            DefenseMechanism::SystemPromptGuard,
            DefenseMechanism::ContentModeration,
            DefenseMechanism::JailbreakDetection,
            DefenseMechanism::TokenLimit,
            DefenseMechanism::InstructionHierarchy,
            DefenseMechanism::ContextIsolation,
        ];
        for m in mechanisms {
            profile
                .blocked
                .entry(m)
                .or_default();
        }
        profile
    }
}

pub struct StrategyRotator;

impl StrategyRotator {
    pub fn rotate(blocked_strategies: &[AttackStrategy]) -> AttackStrategy {
        let blocked_turn_counts: Vec<usize> =
            blocked_strategies.iter().map(|s| s.turn_count()).collect();

        if blocked_turn_counts.iter().all(|&c| c <= 1) {
            AttackStrategy::MultiTurn {
                turns: vec![
                    "establish rapport".into(),
                    "build context".into(),
                    "test boundary".into(),
                    "escalate carefully".into(),
                ],
                escalation: true,
            }
        } else {
            let max_blocked = blocked_turn_counts.iter().copied().max().unwrap_or(1);
            let stages_count = max_blocked + 2;
            let intensity_curve: Vec<f64> = (0..stages_count)
                .map(|i| (i as f64 + 1.0) / stages_count as f64)
                .collect();
            let stages = (0..stages_count)
                .map(|i| format!("stage_{}", i))
                .collect();
            AttackStrategy::Crescendo {
                stages,
                intensity_curve,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defense_profile_new() {
        let p = DefenseProfile::new("test-model");
        assert_eq!(p.target, "test-model");
        assert!(p.blocked.is_empty());
        assert!(p.bypassed.is_empty());
    }

    #[test]
    fn test_record_blocked_increases_effectiveness() {
        let mut p = DefenseProfile::new("m");
        p.record_blocked(
            DefenseMechanism::InputFilter,
            VulnerabilityCategory::PromptInjection,
        );
        p.record_blocked(
            DefenseMechanism::OutputFilter,
            VulnerabilityCategory::DataExfiltration,
        );
        assert_eq!(p.effectiveness, 1.0);
    }

    #[test]
    fn test_record_bypassed_decreases_effectiveness() {
        let mut p = DefenseProfile::new("m");
        p.record_blocked(
            DefenseMechanism::InputFilter,
            VulnerabilityCategory::PromptInjection,
        );
        p.record_bypassed(
            DefenseMechanism::OutputFilter,
            VulnerabilityCategory::Jailbreak,
        );
        assert!((p.effectiveness - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_is_vulnerable_to() {
        let mut p = DefenseProfile::new("m");
        assert!(!p.is_vulnerable_to(&VulnerabilityCategory::Jailbreak));
        p.record_bypassed(
            DefenseMechanism::ContentModeration,
            VulnerabilityCategory::Jailbreak,
        );
        assert!(p.is_vulnerable_to(&VulnerabilityCategory::Jailbreak));
    }

    #[test]
    fn test_profiler_populates_all_mechanisms() {
        let p = DefenseProfiler::profile("target");
        assert_eq!(p.blocked.len(), 8);
    }

    #[test]
    fn test_rotate_after_single_turn_blocks() {
        let blocked = vec![AttackStrategy::SingleTurn {
            payload: "test".into(),
        }];
        let rotated = StrategyRotator::rotate(&blocked);
        assert!(matches!(rotated, AttackStrategy::MultiTurn { .. }));
    }

    #[test]
    fn test_rotate_after_multi_turn_blocks() {
        let blocked = vec![AttackStrategy::MultiTurn {
            turns: vec!["a".into(), "b".into(), "c".into()],
            escalation: false,
        }];
        let rotated = StrategyRotator::rotate(&blocked);
        assert!(matches!(rotated, AttackStrategy::Crescendo { .. }));
        if let AttackStrategy::Crescendo {
            stages,
            intensity_curve,
        } = rotated
        {
            assert_eq!(stages.len(), 5);
            assert_eq!(intensity_curve.len(), 5);
            assert!((intensity_curve.last().unwrap() - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_rotate_empty_blocked() {
        let rotated = StrategyRotator::rotate(&[]);
        assert!(matches!(rotated, AttackStrategy::MultiTurn { .. }));
    }
}
