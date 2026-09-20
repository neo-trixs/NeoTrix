#![forbid(unsafe_code)]

/// Healing configuration — controls the diagnostic chain behavior.
///
/// Default values follow R-P37 (self-healing maturity) and R-P38 (retry cap):
/// - `max_repair_attempts` defaults to 3 (R-P38 hard cap)
/// - `auto_heal_enabled` defaults to true
/// - `check_interval_ms` defaults to 5000 (5s polling)
#[derive(Debug, Clone)]
pub struct HealingConfig {
    /// Interval between health signal polls in milliseconds.
    pub check_interval_ms: u64,
    /// Maximum number of repair attempts before giving up (R-P38 cap).
    pub max_repair_attempts: u32,
    /// Whether auto-healing is enabled. When false, only diagnosis runs.
    pub auto_heal_enabled: bool,
}

impl Default for HealingConfig {
    fn default() -> Self {
        Self {
            check_interval_ms: 5000,
            max_repair_attempts: 3,
            auto_heal_enabled: true,
        }
    }
}

impl HealingConfig {
    pub fn new(check_interval_ms: u64, max_repair_attempts: u32, auto_heal_enabled: bool) -> Self {
        Self {
            check_interval_ms,
            max_repair_attempts,
            auto_heal_enabled,
        }
    }

    /// Preset: disabled healing (diagnostic-only mode).
    pub fn diagnostic_only() -> Self {
        Self {
            auto_heal_enabled: false,
            ..Default::default()
        }
    }

    /// Preset: aggressive healing with fast polling.
    pub fn aggressive() -> Self {
        Self {
            check_interval_ms: 1000,
            max_repair_attempts: 5,
            auto_heal_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = HealingConfig::default();
        assert_eq!(cfg.check_interval_ms, 5000);
        assert_eq!(cfg.max_repair_attempts, 3);
        assert!(cfg.auto_heal_enabled);
    }

    #[test]
    fn test_diagnostic_only_preset() {
        let cfg = HealingConfig::diagnostic_only();
        assert!(!cfg.auto_heal_enabled);
        assert_eq!(cfg.max_repair_attempts, 3);
    }

    #[test]
    fn test_aggressive_preset() {
        let cfg = HealingConfig::aggressive();
        assert_eq!(cfg.check_interval_ms, 1000);
        assert_eq!(cfg.max_repair_attempts, 5);
    }

    #[test]
    fn test_custom_config() {
        let cfg = HealingConfig::new(2000, 10, false);
        assert_eq!(cfg.check_interval_ms, 2000);
        assert_eq!(cfg.max_repair_attempts, 10);
        assert!(!cfg.auto_heal_enabled);
    }

    #[test]
    fn test_diagnostic_only_disables_auto_heal() {
        let cfg = HealingConfig::diagnostic_only();
        assert!(!cfg.auto_heal_enabled);
        assert_eq!(cfg.check_interval_ms, 5000);
    }

    #[test]
    fn test_aggressive_enables_auto_heal() {
        let cfg = HealingConfig::aggressive();
        assert!(cfg.auto_heal_enabled);
    }

    #[test]
    fn test_config_clone() {
        let cfg = HealingConfig::new(1000, 5, true);
        let cloned = cfg.clone();
        assert_eq!(cloned.check_interval_ms, 1000);
        assert_eq!(cloned.max_repair_attempts, 5);
        assert!(cloned.auto_heal_enabled);
    }

    #[test]
    fn test_config_debug() {
        let cfg = HealingConfig::default();
        let debug_str = format!("{:?}", cfg);
        assert!(debug_str.contains("5000"));
        assert!(debug_str.contains("3"));
    }
}
