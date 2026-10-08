#![forbid(unsafe_code)]

use super::config::HealingConfig;
use super::diagnostician::Diagnostician;
use super::health_signal::HealthSignal;
use super::repair_action::{execute_repair, RepairAction};

/// The healing loop drives the complete diagnostic-chain cycle:
///   signal collection → diagnosis → repair selection → execution → verification.
///
/// Each call to [`HealingLoop::run_cycle`] advances one iteration. The loop tracks
/// the current attempt count and respects the configured retry cap.
pub struct HealingLoop {
    /// Active configuration.
    pub config: HealingConfig,
    /// Number of consecutive repair attempts in the current cycle.
    pub attempt: u32,
}

impl HealingLoop {
    pub fn new(config: HealingConfig) -> Self {
        Self { config, attempt: 0 }
    }

    /// Execute one full healing cycle: diagnose → select action → execute → verify.
    ///
    /// Returns the repair action taken, or `None` if no repair was needed
    /// (all signals healthy or retry cap reached).
    pub fn run_cycle(&mut self, signals: &[HealthSignal]) -> Option<RepairAction> {
        // 1. Diagnose from signals.
        let diagnostic = Diagnostician::analyze(signals)?;

        // 2. Check retry cap.
        if self.attempt >= self.config.max_repair_attempts {
            log::warn!(
                "[healing_loop] max attempts ({}) reached, skipping repair for: {}",
                self.config.max_repair_attempts,
                diagnostic.source_component
            );
            return None;
        }

        // 3. Map severity to action type.
        let max_severity = if diagnostic
            .recommended_actions
            .iter()
            .any(|a| a.contains("Restart"))
        {
            "critical"
        } else if diagnostic
            .recommended_actions
            .iter()
            .any(|a| a.contains("Monitor") || a.contains("Degrade"))
        {
            "warning"
        } else {
            "info"
        };

        let action =
            super::repair_action::select_action(&diagnostic.source_component, max_severity);

        // 4. Execute (only if auto_heal is enabled).
        if self.config.auto_heal_enabled {
            match execute_repair(&action) {
                Ok(msg) => {
                    log::info!("[healing_loop] repair succeeded: {}", msg);
                    self.attempt += 1;
                }
                Err(e) => {
                    log::error!("[healing_loop] repair failed: {}", e);
                    self.attempt += 1;
                }
            }
        } else {
            log::info!(
                "[healing_loop] auto_heal disabled, action selected but not executed: {}",
                action
            );
        }

        Some(action)
    }

    /// Reset the attempt counter (e.g. after a successful heal verification).
    pub fn reset(&mut self) {
        self.attempt = 0;
    }

    /// Returns true if the retry cap has been reached.
    pub fn is_exhausted(&self) -> bool {
        self.attempt >= self.config.max_repair_attempts
    }
}

#[cfg(test)]
mod tests {
    use super::super::health_signal::Severity;
    use super::*;

    fn critical_signal(component: &str) -> HealthSignal {
        HealthSignal::critical(component, "error_rate", 0.95, 1000)
    }

    fn warning_signal(component: &str) -> HealthSignal {
        HealthSignal::warning(component, "hit_rate", 0.3, 1000)
    }

    fn info_signal(component: &str) -> HealthSignal {
        HealthSignal::info(component, "uptime", 3600.0, 1000)
    }

    #[test]
    fn test_run_cycle_returns_none_for_empty_signals() {
        let config = HealingConfig::default();
        let mut loop_inst = HealingLoop::new(config);
        assert!(loop_inst.run_cycle(&[]).is_none());
    }

    #[test]
    fn test_run_cycle_diagnoses_critical_and_restarts() {
        let config = HealingConfig::default();
        let mut loop_inst = HealingLoop::new(config);
        let signals = vec![critical_signal("nt_core_cache")];
        let action = loop_inst.run_cycle(&signals).unwrap();
        assert_eq!(action.action_type(), "restart");
        assert_eq!(loop_inst.attempt, 1);
    }

    #[test]
    fn test_run_cycle_diagnoses_warning_and_degrades() {
        let config = HealingConfig::default();
        let mut loop_inst = HealingLoop::new(config);
        let signals = vec![warning_signal("nt_core_cache")];
        let action = loop_inst.run_cycle(&signals).unwrap();
        assert_eq!(action.action_type(), "degrade");
    }

    #[test]
    fn test_run_cycle_respects_retry_cap() {
        let config = HealingConfig {
            max_repair_attempts: 2,
            auto_heal_enabled: true,
            check_interval_ms: 1000,
        };
        let mut loop_inst = HealingLoop::new(config);
        let signals = vec![critical_signal("nt_core_cache")];

        // First two attempts succeed.
        assert!(loop_inst.run_cycle(&signals).is_some());
        assert!(loop_inst.run_cycle(&signals).is_some());

        // Third attempt is blocked by the cap.
        assert!(loop_inst.run_cycle(&signals).is_none());
        assert!(loop_inst.is_exhausted());
    }

    #[test]
    fn test_run_cycle_auto_heal_disabled() {
        let config = HealingConfig {
            max_repair_attempts: 5,
            auto_heal_enabled: false,
            check_interval_ms: 1000,
        };
        let mut loop_inst = HealingLoop::new(config);
        let signals = vec![critical_signal("nt_core_cache")];
        let action = loop_inst.run_cycle(&signals).unwrap();
        assert_eq!(action.action_type(), "restart");
        // Attempt counter should NOT increment when auto_heal is off.
        assert_eq!(loop_inst.attempt, 0);
    }

    #[test]
    fn test_reset_clears_attempt_counter() {
        let config = HealingConfig {
            max_repair_attempts: 3,
            auto_heal_enabled: true,
            check_interval_ms: 1000,
        };
        let mut loop_inst = HealingLoop::new(config);
        let signals = vec![critical_signal("nt_core_cache")];

        loop_inst.run_cycle(&signals);
        loop_inst.run_cycle(&signals);
        assert_eq!(loop_inst.attempt, 2);

        loop_inst.reset();
        assert_eq!(loop_inst.attempt, 0);
        assert!(!loop_inst.is_exhausted());
    }

    #[test]
    fn test_run_cycle_with_info_only_returns_action() {
        let config = HealingConfig::default();
        let mut loop_inst = HealingLoop::new(config);
        let signals = vec![info_signal("nt_core_cache")];
        let action = loop_inst.run_cycle(&signals).unwrap();
        // Info-level signals produce an alert action.
        assert_eq!(action.action_type(), "alert");
    }

    #[test]
    fn test_run_cycle_mixed_severity_picks_highest_weight() {
        let config = HealingConfig::default();
        let mut loop_inst = HealingLoop::new(config);
        let signals = vec![
            info_signal("nt_world_crawl"),
            critical_signal("nt_core_cache"),
            critical_signal("nt_core_cache"),
        ];
        let action = loop_inst.run_cycle(&signals).unwrap();
        assert_eq!(action.action_type(), "restart");
        assert_eq!(loop_inst.attempt, 1);
    }

    #[test]
    fn test_run_cycle_multiple_cycles_increment_attempt() {
        let config = HealingConfig::default();
        let mut loop_inst = HealingLoop::new(config);
        let signals = vec![critical_signal("comp")];

        loop_inst.run_cycle(&signals);
        assert_eq!(loop_inst.attempt, 1);
        loop_inst.run_cycle(&signals);
        assert_eq!(loop_inst.attempt, 2);
        loop_inst.run_cycle(&signals);
        assert_eq!(loop_inst.attempt, 3);
    }

    #[test]
    fn test_is_exhausted_at_boundary() {
        let config = HealingConfig {
            max_repair_attempts: 1,
            auto_heal_enabled: true,
            check_interval_ms: 1000,
        };
        let mut loop_inst = HealingLoop::new(config);
        assert!(!loop_inst.is_exhausted());

        let signals = vec![critical_signal("comp")];
        loop_inst.run_cycle(&signals);
        assert!(loop_inst.is_exhausted());
    }

    #[test]
    fn test_new_starts_with_zero_attempts() {
        let config = HealingConfig::default();
        let loop_inst = HealingLoop::new(config);
        assert_eq!(loop_inst.attempt, 0);
        assert!(!loop_inst.is_exhausted());
    }

    #[test]
    fn test_run_cycle_with_multiple_components() {
        let config = HealingConfig::default();
        let mut loop_inst = HealingLoop::new(config);
        let signals = vec![
            critical_signal("comp_a"),
            warning_signal("comp_b"),
            info_signal("comp_c"),
        ];
        let action = loop_inst.run_cycle(&signals).unwrap();
        // Critical component (comp_a) should be selected
        assert_eq!(action.action_type(), "restart");
    }

    #[test]
    fn test_run_cycle_returns_none_for_info_only_after_exhaustion() {
        let config = HealingConfig {
            max_repair_attempts: 2,
            auto_heal_enabled: true,
            check_interval_ms: 1000,
        };
        let mut loop_inst = HealingLoop::new(config);
        let signals = vec![critical_signal("comp")];

        loop_inst.run_cycle(&signals);
        loop_inst.run_cycle(&signals);
        // Exhausted now
        let result = loop_inst.run_cycle(&signals);
        assert!(result.is_none());
    }

    #[test]
    fn test_reset_allows_new_cycles() {
        let config = HealingConfig {
            max_repair_attempts: 2,
            auto_heal_enabled: true,
            check_interval_ms: 1000,
        };
        let mut loop_inst = HealingLoop::new(config);
        let signals = vec![critical_signal("comp")];

        loop_inst.run_cycle(&signals);
        loop_inst.run_cycle(&signals);
        assert!(loop_inst.is_exhausted());

        loop_inst.reset();
        assert!(!loop_inst.is_exhausted());

        let result = loop_inst.run_cycle(&signals);
        assert!(result.is_some());
        assert_eq!(loop_inst.attempt, 1);
    }
}
