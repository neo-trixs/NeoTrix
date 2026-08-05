#[derive(Debug, Clone)]
pub struct GuardConfig {
    pub enabled: bool,
    pub metric_name: String,
    pub floor: f64,
    pub relative_floor: bool,
    pub penalty_on_violation: f64,
}

impl Default for GuardConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            metric_name: String::from("accuracy"),
            floor: 0.5,
            relative_floor: false,
            penalty_on_violation: 0.5,
        }
    }
}

#[derive(Debug, Clone)]
pub enum GuardResult {
    Pass,
    Violation { current: f64, floor: f64, delta: f64 },
}

pub fn check_guard_violation(config: &GuardConfig, current: f64, initial: f64) -> GuardResult {
    if !config.enabled {
        return GuardResult::Pass;
    }
    let effective = if config.relative_floor {
        initial * (1.0 - config.floor)
    } else {
        config.floor
    };
    if current < effective - 1e-9 {
        GuardResult::Violation {
            current,
            floor: effective,
            delta: current - effective,
        }
    } else {
        GuardResult::Pass
    }
}

impl GuardConfig {
    pub fn with_floor(floor: f64) -> Self {
        Self {
            enabled: true,
            floor,
            ..Default::default()
        }
    }

    pub fn with_relative(ratio: f64) -> Self {
        Self {
            enabled: true,
            relative_floor: true,
            floor: ratio,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guard_disabled_passes() {
        let cfg = GuardConfig { enabled: false, ..Default::default() };
        assert!(matches!(check_guard_violation(&cfg, 0.1, 0.9), GuardResult::Pass));
    }

    #[test]
    fn test_guard_above_floor_passes() {
        let cfg = GuardConfig::with_floor(0.5);
        assert!(matches!(check_guard_violation(&cfg, 0.6, 0.9), GuardResult::Pass));
    }

    #[test]
    fn test_guard_below_floor_violates() {
        let cfg = GuardConfig::with_floor(0.5);
        assert!(matches!(check_guard_violation(&cfg, 0.3, 0.9), GuardResult::Violation { .. }));
    }

    #[test]
    fn test_guard_relative_floor() {
        let cfg = GuardConfig::with_relative(0.2);
        let result = check_guard_violation(&cfg, 0.7, 1.0);
        assert!(matches!(result, GuardResult::Violation { .. }), "0.7 < 0.8 (1.0*0.8) should violate");
        let result = check_guard_violation(&cfg, 0.7, 0.8);
        assert!(matches!(result, GuardResult::Pass), "0.7 >= 0.64 (0.8*0.8) should pass");
    }

    #[test]
    fn test_violation_delta() {
        let cfg = GuardConfig::with_floor(0.5);
        match check_guard_violation(&cfg, 0.3, 0.9) {
            GuardResult::Violation { current, floor, delta } => {
                assert!((current - 0.3).abs() < 1e-9);
                assert!((floor - 0.5).abs() < 1e-9);
                assert!((delta - (-0.2)).abs() < 1e-9);
            }
            _ => panic!("expected violation"),
        }
    }

    #[test]
    fn test_default_config_disabled() {
        let cfg = GuardConfig::default();
        assert!(!cfg.enabled);
    }
}

