use crate::core::nt_core_consciousness::global_workspace::GlobalLatentWorkspace;
use crate::core::nt_core_gwt::monitor::EntropyMonitor;
use crate::core::nt_core_reasoning::dead_end_detector::{DeadEndReport, DeadEndType};
use crate::core::nt_core_reasoning::strategy_selector::SelfHealingSelector;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptLevel {
    None,
    Hint,
    Nudge,
    Interrupt,
    HardAbort,
}

#[derive(Debug, Clone)]
pub struct InterruptSignal {
    pub level: InterruptLevel,
    pub source: String,
    pub reason: String,
    pub suggested_recovery: String,
    pub timestamp: u64,
    pub target_step: String,
}

#[derive(Debug)]
pub struct SelfInterruptConfig {
    pub max_consecutive_loops: usize,
    pub max_cycle_duration_us: u64,
    pub entropy_deadlock_threshold: f64,
    pub interrupt_cooldown_cycles: usize,
    pub enable_auto_recovery: bool,
}

impl Default for SelfInterruptConfig {
    fn default() -> Self {
        Self {
            max_consecutive_loops: 5,
            max_cycle_duration_us: 500000,
            entropy_deadlock_threshold: 0.3,
            interrupt_cooldown_cycles: 10,
            enable_auto_recovery: true,
        }
    }
}

#[derive(Debug)]
pub struct SelfInterruptSystem {
    pub config: SelfInterruptConfig,
    pub last_interrupt: Option<InterruptSignal>,
    pub interrupt_history: Vec<InterruptSignal>,
    pub interrupt_count: u64,
    pub cycles_since_last_interrupt: usize,
    pub recovery_success_rate: f64,
    pub entropy_monitor: Option<EntropyMonitor>,
    pub active_interrupt: Option<InterruptSignal>,
    total_recovery_attempts: u64,
    total_recovery_successes: u64,
    consecutive_low_entropy_cycles: usize,
}

#[derive(Debug)]
pub struct InterruptStats {
    pub total_interrupts: u64,
    pub hint_count: u64,
    pub nudge_count: u64,
    pub interrupt_count: u64,
    pub hard_abort_count: u64,
    pub recovery_success_rate: f64,
    pub avg_cycles_between_interrupts: f64,
}

impl SelfInterruptSystem {
    pub fn new(config: SelfInterruptConfig) -> Self {
        Self {
            config,
            last_interrupt: None,
            interrupt_history: Vec::with_capacity(64),
            interrupt_count: 0,
            cycles_since_last_interrupt: 0,
            recovery_success_rate: 1.0,
            entropy_monitor: None,
            active_interrupt: None,
            total_recovery_attempts: 0,
            total_recovery_successes: 0,
            consecutive_low_entropy_cycles: 0,
        }
    }

    pub fn with_entropy_monitor(mut self, monitor: EntropyMonitor) -> Self {
        self.entropy_monitor = Some(monitor);
        self
    }

    pub fn monitor_cycle(
        &mut self,
        cycle_duration_us: u64,
        entropy: f64,
        dead_end_reports: &[DeadEndReport],
    ) -> Option<InterruptSignal> {
        self.cycles_since_last_interrupt += 1;

        if self.cycles_since_last_interrupt < self.config.interrupt_cooldown_cycles {
            return None;
        }

        if let Some(signal) = self.check_timeout(cycle_duration_us) {
            return Some(signal);
        }

        if let Some(signal) = self.check_entropy_deadlock(entropy) {
            return Some(signal);
        }

        if let Some(signal) = self.check_dead_end_reports(dead_end_reports) {
            return Some(signal);
        }

        None
    }

    pub fn check_timeout(&self, cycle_duration_us: u64) -> Option<InterruptSignal> {
        if cycle_duration_us > self.config.max_cycle_duration_us {
            let ratio = cycle_duration_us as f64 / self.config.max_cycle_duration_us as f64;
            let level = if ratio > 3.0 {
                InterruptLevel::Interrupt
            } else if ratio > 2.0 {
                InterruptLevel::Nudge
            } else {
                InterruptLevel::Hint
            };
            Some(InterruptSignal {
                level,
                source: "timeout".into(),
                reason: format!(
                    "Cycle duration {}us exceeds limit {}us (ratio {:.1}x)",
                    cycle_duration_us, self.config.max_cycle_duration_us, ratio
                ),
                suggested_recovery: "switch to faster reasoning strategy".into(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                target_step: "REASON".into(),
            })
        } else {
            None
        }
    }

    pub fn check_entropy_deadlock(&mut self, entropy: f64) -> Option<InterruptSignal> {
        if entropy < self.config.entropy_deadlock_threshold {
            self.consecutive_low_entropy_cycles += 1;
        } else {
            self.consecutive_low_entropy_cycles = 0;
            return None;
        }

        if self.consecutive_low_entropy_cycles >= 5 {
            self.consecutive_low_entropy_cycles = 0;
            Some(InterruptSignal {
                level: InterruptLevel::Interrupt,
                source: "entropy_monitor".into(),
                reason: format!(
                    "Entropy {:.3} below threshold {:.3} for {} consecutive cycles",
                    entropy,
                    self.config.entropy_deadlock_threshold,
                    self.consecutive_low_entropy_cycles
                ),
                suggested_recovery: "inject stochastic stimulus or switch strategy".into(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                target_step: "GATHER".into(),
            })
        } else if self.consecutive_low_entropy_cycles >= 3 {
            Some(InterruptSignal {
                level: InterruptLevel::Nudge,
                source: "entropy_monitor".into(),
                reason: format!(
                    "Entropy {:.3} trending low — possible deadlock forming",
                    entropy
                ),
                suggested_recovery: "consider alternative reasoning path".into(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                target_step: "GATHER".into(),
            })
        } else {
            None
        }
    }

    pub fn check_dead_end_reports(&self, reports: &[DeadEndReport]) -> Option<InterruptSignal> {
        for report in reports {
            match report.detected_type {
                DeadEndType::Loop => {
                    if let Some(len) = report.loop_length {
                        if len >= self.config.max_consecutive_loops {
                            return Some(InterruptSignal {
                                level: InterruptLevel::HardAbort,
                                source: "dead_end".into(),
                                reason: format!("Reasoning loop of length {} detected", len),
                                suggested_recovery: format!("{:?}", report.recovery),
                                timestamp: report.timestamp,
                                target_step: "REASON".into(),
                            });
                        }
                    }
                    return Some(InterruptSignal {
                        level: InterruptLevel::Interrupt,
                        source: "dead_end".into(),
                        reason: "Reasoning loop detected".into(),
                        suggested_recovery: format!("{:?}", report.recovery),
                        timestamp: report.timestamp,
                        target_step: "REASON".into(),
                    });
                }
                DeadEndType::DepthExceeded => {
                    return Some(InterruptSignal {
                        level: InterruptLevel::HardAbort,
                        source: "dead_end".into(),
                        reason: "Maximum reasoning depth exceeded".into(),
                        suggested_recovery: format!("{:?}", report.recovery),
                        timestamp: report.timestamp,
                        target_step: "REASON".into(),
                    });
                }
                DeadEndType::SemanticDeadlock => {
                    return Some(InterruptSignal {
                        level: InterruptLevel::Interrupt,
                        source: "dead_end".into(),
                        reason: "Semantic deadlock — reasoning cannot make progress".into(),
                        suggested_recovery: format!("{:?}", report.recovery),
                        timestamp: report.timestamp,
                        target_step: "REASON".into(),
                    });
                }
                DeadEndType::ContradictionFlood => {
                    return Some(InterruptSignal {
                        level: InterruptLevel::Nudge,
                        source: "dead_end".into(),
                        reason: "Contradiction flood — too many conflicting hypotheses".into(),
                        suggested_recovery: format!("{:?}", report.recovery),
                        timestamp: report.timestamp,
                        target_step: "REASON".into(),
                    });
                }
                DeadEndType::Divergence | DeadEndType::ConfidenceStagnation => {
                    return Some(InterruptSignal {
                        level: InterruptLevel::Hint,
                        source: "dead_end".into(),
                        reason: format!("{:?} detected", report.detected_type),
                        suggested_recovery: format!("{:?}", report.recovery),
                        timestamp: report.timestamp,
                        target_step: "REASON".into(),
                    });
                }
            }
        }
        None
    }

    pub fn issue_interrupt(&mut self, signal: InterruptSignal) {
        self.active_interrupt = Some(signal.clone());
        self.last_interrupt = Some(signal.clone());
        self.interrupt_history.push(signal.clone());
        if self.interrupt_history.len() > 64 {
            self.interrupt_history.remove(0);
        }
        self.interrupt_count += 1;
        self.cycles_since_last_interrupt = 0;
        self.consecutive_low_entropy_cycles = 0;
    }

    pub fn clear_interrupt(&mut self) {
        self.active_interrupt = None;
    }

    pub fn has_active_interrupt(&self) -> bool {
        self.active_interrupt.is_some()
    }

    pub fn recommend_recovery(&self, strategy_selector: &SelfHealingSelector) -> String {
        if let Some(ref signal) = self.active_interrupt {
            match signal.level {
                InterruptLevel::HardAbort => format!(
                    "HARD_ABORT: restart cycle with strategy {:?}",
                    strategy_selector.current_strategy
                ),
                InterruptLevel::Interrupt => format!(
                    "SWITCH_STRATEGY: from {:?} to {:?}",
                    strategy_selector.current_strategy, strategy_selector.config.recovery_strategy
                ),
                InterruptLevel::Nudge => format!(
                    "CONSIDER_ALTERNATIVE: current {:?} has {} switches in {} steps",
                    strategy_selector.current_strategy,
                    strategy_selector.switch_count,
                    strategy_selector.total_steps
                ),
                InterruptLevel::Hint => "CONTINUE with slight perturbation".into(),
                InterruptLevel::None => "NO_ACTION".into(),
            }
        } else {
            "NO_ACTION".into()
        }
    }

    pub fn record_recovery_outcome(&mut self, success: bool) {
        self.total_recovery_attempts += 1;
        if success {
            self.total_recovery_successes += 1;
        }
        self.recovery_success_rate = if self.total_recovery_attempts > 0 {
            self.total_recovery_successes as f64 / self.total_recovery_attempts as f64
        } else {
            1.0
        };
    }

    pub fn inject_to_workspace(
        &self,
        workspace: &mut GlobalLatentWorkspace,
    ) -> Option<InterruptSignal> {
        let signal = self
            .active_interrupt
            .as_ref()
            .or(self.last_interrupt.as_ref())?;

        let context = format!(
            "INTERRUPT|{}|{}|{}",
            signal.source, signal.reason, signal.suggested_recovery
        );
        let vector = match signal.level {
            InterruptLevel::HardAbort => vec![0xFF; 64],
            InterruptLevel::Interrupt => vec![0xAA; 64],
            InterruptLevel::Nudge => vec![0x55; 64],
            InterruptLevel::Hint => vec![0x33; 64],
            InterruptLevel::None => return None,
        };

        workspace.submit_proposal("self_interrupt", vector, &context, signal.timestamp);

        Some(signal.clone())
    }

    pub fn stats(&self) -> InterruptStats {
        let mut hint_count = 0;
        let mut nudge_count = 0;
        let mut interrupt_count = 0;
        let mut hard_abort_count = 0;

        for sig in &self.interrupt_history {
            match sig.level {
                InterruptLevel::Hint => hint_count += 1,
                InterruptLevel::Nudge => nudge_count += 1,
                InterruptLevel::Interrupt => interrupt_count += 1,
                InterruptLevel::HardAbort => hard_abort_count += 1,
                InterruptLevel::None => {}
            }
        }

        let avg_cycles = if self.interrupt_count > 0 {
            (self.interrupt_history.len() as f64) / (self.interrupt_count as f64).max(1.0)
        } else {
            0.0
        };

        InterruptStats {
            total_interrupts: self.interrupt_count,
            hint_count,
            nudge_count,
            interrupt_count,
            hard_abort_count,
            recovery_success_rate: self.recovery_success_rate,
            avg_cycles_between_interrupts: avg_cycles,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_reasoning::dead_end_detector::DeadEndType;
    use crate::core::nt_core_reasoning::strategy_selector::{ReasoningStrategy, StrategyConfig};

    #[test]
    fn test_default_config() {
        let config = SelfInterruptConfig::default();
        assert_eq!(config.max_consecutive_loops, 5);
        assert_eq!(config.max_cycle_duration_us, 500000);
        assert!((config.entropy_deadlock_threshold - 0.3).abs() < 1e-9);
        assert_eq!(config.interrupt_cooldown_cycles, 10);
        assert!(config.enable_auto_recovery);
    }

    #[test]
    fn test_new_system_starts_clean() {
        let sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        assert!(sys.last_interrupt.is_none());
        assert!(sys.active_interrupt.is_none());
        assert_eq!(sys.interrupt_count, 0);
        assert_eq!(sys.cycles_since_last_interrupt, 0);
    }

    #[test]
    fn test_timeout_hint() {
        let sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        let sig = sys.check_timeout(600000).unwrap();
        assert_eq!(sig.level, InterruptLevel::Hint);
        assert_eq!(sig.source, "timeout");
    }

    #[test]
    fn test_timeout_nudge() {
        let sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        let sig = sys.check_timeout(1_100_000).unwrap();
        assert_eq!(sig.level, InterruptLevel::Nudge);
    }

    #[test]
    fn test_timeout_interrupt() {
        let sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        let sig = sys.check_timeout(2_000_000).unwrap();
        assert_eq!(sig.level, InterruptLevel::Interrupt);
    }

    #[test]
    fn test_timeout_under_threshold() {
        let sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        assert!(sys.check_timeout(100000).is_none());
    }

    #[test]
    fn test_entropy_deadlock_nudge_at_3() {
        let mut sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        // First 3 low entropy readings
        assert!(sys.check_entropy_deadlock(0.2).is_none());
        assert!(sys.check_entropy_deadlock(0.2).is_none());
        let sig = sys.check_entropy_deadlock(0.2).unwrap();
        assert_eq!(sig.level, InterruptLevel::Nudge);
        assert_eq!(sig.source, "entropy_monitor");
    }

    #[test]
    fn test_entropy_deadlock_interrupt_at_5() {
        let mut sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        for _ in 0..3 {
            sys.check_entropy_deadlock(0.2);
        }
        // 4th — still nudge (but check_entropy_deadlock resets if > threshold, so keep feeding)
        sys.check_entropy_deadlock(0.2);
        let sig = sys.check_entropy_deadlock(0.2).unwrap();
        assert_eq!(sig.level, InterruptLevel::Interrupt);
    }

    #[test]
    fn test_entropy_resets_on_high_value() {
        let mut sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        sys.check_entropy_deadlock(0.2);
        sys.check_entropy_deadlock(0.2);
        // High entropy resets counter
        assert!(sys.check_entropy_deadlock(0.8).is_none());
        // Counter should be 0 again
        assert!(sys.check_entropy_deadlock(0.2).is_none());
    }

    #[test]
    fn test_dead_end_loop_triggers_interrupt() {
        let sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        let report = DeadEndReport {
            detected_type: DeadEndType::Loop,
            detection_step: 10,
            loop_length: Some(3),
            vsa_similarity: 0.98,
            confidence_delta: 0.01,
            recovery:
                crate::core::nt_core_reasoning::dead_end_detector::RecoveryStrategy::Backtrack(3),
            evidence: vec!["loop detected".into()],
            timestamp: 100,
        };
        let sig = sys.check_dead_end_reports(&[report]).unwrap();
        assert_eq!(sig.level, InterruptLevel::Interrupt);
        assert_eq!(sig.source, "dead_end");
    }

    #[test]
    fn test_dead_end_loop_hard_abort_when_long() {
        let sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        let report = DeadEndReport {
            detected_type: DeadEndType::Loop,
            detection_step: 10,
            loop_length: Some(6),
            vsa_similarity: 0.98,
            confidence_delta: 0.01,
            recovery:
                crate::core::nt_core_reasoning::dead_end_detector::RecoveryStrategy::Backtrack(5),
            evidence: vec!["long loop".into()],
            timestamp: 100,
        };
        let sig = sys.check_dead_end_reports(&[report]).unwrap();
        assert_eq!(sig.level, InterruptLevel::HardAbort);
    }

    #[test]
    fn test_dead_end_depth_exceeded_hard_abort() {
        let sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        let report = DeadEndReport {
            detected_type: DeadEndType::DepthExceeded,
            detection_step: 20,
            loop_length: None,
            vsa_similarity: 0.0,
            confidence_delta: 0.0,
            recovery: crate::core::nt_core_reasoning::dead_end_detector::RecoveryStrategy::DecomposeSubProblem,
            evidence: vec!["max depth".into()],
            timestamp: 200,
        };
        let sig = sys.check_dead_end_reports(&[report]).unwrap();
        assert_eq!(sig.level, InterruptLevel::HardAbort);
    }

    #[test]
    fn test_dead_end_contradiction_nudge() {
        let sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        let report = DeadEndReport {
            detected_type: DeadEndType::ContradictionFlood,
            detection_step: 5,
            loop_length: None,
            vsa_similarity: 0.0,
            confidence_delta: 0.0,
            recovery: crate::core::nt_core_reasoning::dead_end_detector::RecoveryStrategy::SwitchReasoningMode,
            evidence: vec!["contradictions".into()],
            timestamp: 50,
        };
        let sig = sys.check_dead_end_reports(&[report]).unwrap();
        assert_eq!(sig.level, InterruptLevel::Nudge);
    }

    #[test]
    fn test_issue_interrupt_sets_active() {
        let mut sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        let sig = InterruptSignal {
            level: InterruptLevel::Interrupt,
            source: "test".into(),
            reason: "test".into(),
            suggested_recovery: "test".into(),
            timestamp: 1,
            target_step: "REASON".into(),
        };
        sys.issue_interrupt(sig);
        assert!(sys.has_active_interrupt());
        assert_eq!(sys.interrupt_count, 1);
        assert_eq!(sys.cycles_since_last_interrupt, 0);
    }

    #[test]
    fn test_clear_interrupt() {
        let mut sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        let sig = InterruptSignal {
            level: InterruptLevel::HardAbort,
            source: "test".into(),
            reason: "test".into(),
            suggested_recovery: "test".into(),
            timestamp: 2,
            target_step: "REASON".into(),
        };
        sys.issue_interrupt(sig);
        assert!(sys.has_active_interrupt());
        sys.clear_interrupt();
        assert!(!sys.has_active_interrupt());
    }

    #[test]
    fn test_recovery_tracking() {
        let mut sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        assert!((sys.recovery_success_rate - 1.0).abs() < 1e-9);
        sys.record_recovery_outcome(true);
        sys.record_recovery_outcome(false);
        sys.record_recovery_outcome(true);
        assert!((sys.recovery_success_rate - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_inject_to_workspace_with_active_interrupt() {
        let mut workspace = GlobalLatentWorkspace::new();
        let mut sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        let sig = InterruptSignal {
            level: InterruptLevel::Interrupt,
            source: "test".into(),
            reason: "deadlock".into(),
            suggested_recovery: "switch strategy".into(),
            timestamp: 42,
            target_step: "REASON".into(),
        };
        sys.issue_interrupt(sig);
        let result = sys.inject_to_workspace(&mut workspace);
        assert!(result.is_some());
        assert_eq!(workspace.proposal_count(), 1);
        let proposal = &workspace.slots[0];
        assert_eq!(proposal.module_name, "self_interrupt");
        assert!(proposal.context.contains("INTERRUPT"));
    }

    #[test]
    fn test_inject_to_workspace_no_interrupt() {
        let mut workspace = GlobalLatentWorkspace::new();
        let sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        assert!(sys.inject_to_workspace(&mut workspace).is_none());
    }

    #[test]
    fn test_stats_accuracy() {
        let mut sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        let levels = [
            InterruptLevel::Hint,
            InterruptLevel::Nudge,
            InterruptLevel::Interrupt,
            InterruptLevel::HardAbort,
            InterruptLevel::Hint,
        ];
        for (i, &level) in levels.iter().enumerate() {
            sys.issue_interrupt(InterruptSignal {
                level,
                source: "stats_test".into(),
                reason: format!("reason {}", i),
                suggested_recovery: "recovery".into(),
                timestamp: i as u64,
                target_step: "REASON".into(),
            });
        }
        let stats = sys.stats();
        assert_eq!(stats.total_interrupts, 5);
        assert_eq!(stats.hint_count, 2);
        assert_eq!(stats.nudge_count, 1);
        assert_eq!(stats.interrupt_count, 1);
        assert_eq!(stats.hard_abort_count, 1);
    }

    #[test]
    fn test_cooldown_suppresses_interrupts() {
        let mut sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        sys.config.interrupt_cooldown_cycles = 5;

        // Issue first interrupt
        let sig1 = InterruptSignal {
            level: InterruptLevel::Interrupt,
            source: "test".into(),
            reason: "first".into(),
            suggested_recovery: "x".into(),
            timestamp: 1,
            target_step: "REASON".into(),
        };
        sys.issue_interrupt(sig1);
        assert_eq!(sys.cycles_since_last_interrupt, 0);

        // monitor_cycle should return None during cooldown
        let result = sys.monitor_cycle(1_000_000, 0.1, &[]);
        assert!(result.is_none());
        assert_eq!(sys.cycles_since_last_interrupt, 1);

        // After cooldown, should trigger
        sys.cycles_since_last_interrupt = 10;
        let result = sys.monitor_cycle(1_000_000, 0.1, &[]);
        assert!(result.is_some());
    }

    #[test]
    fn test_recommend_recovery_no_interrupt() {
        let sel = SelfHealingSelector::new(StrategyConfig::default());
        let sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        assert_eq!(sys.recommend_recovery(&sel), "NO_ACTION");
    }

    #[test]
    fn test_recommend_recovery_hard_abort() {
        let sel = SelfHealingSelector::new(StrategyConfig::default());
        let mut sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        sys.issue_interrupt(InterruptSignal {
            level: InterruptLevel::HardAbort,
            source: "test".into(),
            reason: "fatal".into(),
            suggested_recovery: "restart".into(),
            timestamp: 0,
            target_step: "REASON".into(),
        });
        let rec = sys.recommend_recovery(&sel);
        assert!(rec.starts_with("HARD_ABORT"));
    }

    #[test]
    fn test_recommend_recovery_interrupt() {
        let sel = SelfHealingSelector::new(StrategyConfig::default());
        let mut sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        sys.issue_interrupt(InterruptSignal {
            level: InterruptLevel::Interrupt,
            source: "test".into(),
            reason: "switch".into(),
            suggested_recovery: "change strategy".into(),
            timestamp: 0,
            target_step: "REASON".into(),
        });
        let rec = sys.recommend_recovery(&sel);
        assert!(rec.starts_with("SWITCH_STRATEGY"));
    }

    #[test]
    fn test_interrupt_history_bounded() {
        let mut sys = SelfInterruptSystem::new(SelfInterruptConfig::default());
        for i in 0..70 {
            sys.issue_interrupt(InterruptSignal {
                level: InterruptLevel::Hint,
                source: "test".into(),
                reason: format!("evict {}", i),
                suggested_recovery: "rec".into(),
                timestamp: i as u64,
                target_step: "REASON".into(),
            });
        }
        assert_eq!(sys.interrupt_history.len(), 64);
        // Oldest should be evicted
        assert!(sys.interrupt_history[0].reason != "evict 0");
    }
}
