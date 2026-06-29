use crate::core::nt_core_consciousness::cognitive_load::ThinkingMode;

#[derive(Debug, Clone)]
pub struct PacingSignals {
    pub cognitive_load: f64,
    pub system_load: f64,
    pub cycle_utilization: f64,
}

impl PacingSignals {
    pub fn new(cognitive_load: f64, system_load: f64, cycle_utilization: f64) -> Self {
        Self {
            cognitive_load: cognitive_load.clamp(0.0, 1.0),
            system_load: system_load.clamp(0.0, 1.0),
            cycle_utilization: cycle_utilization.clamp(0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PacingReport {
    pub multiplier: f64,
    pub thinking_mode: ThinkingMode,
    pub skip_heavy_handlers: bool,
    pub heavy_handler_count: usize,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct SelfPacingGovernor {
    current_multiplier: f64,
    hysteresis: f64,
}

impl SelfPacingGovernor {
    pub fn new(hysteresis: f64) -> Self {
        Self {
            current_multiplier: 1.0,
            hysteresis: hysteresis.clamp(0.05, 1.0),
        }
    }

    pub fn new_default() -> Self {
        Self::new(0.3)
    }

    pub fn compute(&mut self, signals: PacingSignals) -> PacingReport {
        let load_factor = signals
            .cognitive_load
            .max(signals.system_load)
            .max(signals.cycle_utilization);

        let raw_multiplier = if load_factor > 0.8 {
            1.0 + 2.0 * (load_factor - 0.8) / 0.2
        } else if load_factor < 0.3 {
            0.5 + 0.5 * (load_factor / 0.3)
        } else {
            1.0
        };

        let clamped = raw_multiplier.clamp(
            self.current_multiplier - self.hysteresis,
            self.current_multiplier + self.hysteresis,
        );

        self.current_multiplier = clamped;

        let thinking_mode = if signals.cognitive_load > 0.65 {
            ThinkingMode::Deep
        } else if signals.cognitive_load < 0.25 {
            ThinkingMode::Fast
        } else {
            ThinkingMode::Balanced
        };

        let skip_heavy_handlers = load_factor > 0.85;

        let heavy_handler_count = if skip_heavy_handlers {
            (load_factor * 10.0).round() as usize
        } else {
            0
        };

        let reason = format!(
            "load_factor={:.3} (cognitive={:.2}, system={:.2}, cycle={:.2}); \
             raw_mult={:.3}; hyst_clamp={:.3}; mode={}; skip_heavy={}",
            load_factor,
            signals.cognitive_load,
            signals.system_load,
            signals.cycle_utilization,
            raw_multiplier,
            clamped,
            thinking_mode.name(),
            skip_heavy_handlers,
        );

        PacingReport {
            multiplier: clamped,
            thinking_mode,
            skip_heavy_handlers,
            heavy_handler_count,
            reason,
        }
    }

    pub fn reset(&mut self) {
        self.current_multiplier = 1.0;
    }
}

pub fn apply_multiplier_to_config(
    base_intervals: &[(u64, &str)],
    multiplier: f64,
) -> Vec<(u64, String, u64)> {
    base_intervals
        .iter()
        .map(|(base, name)| {
            let adapted = ((*base as f64) * multiplier).round().max(1.0) as u64;
            (*base, name.to_string(), adapted)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn governor_starts_at_one() {
        let mut g = SelfPacingGovernor::new_default();
        let s = PacingSignals::new(0.5, 0.5, 0.5);
        let r = g.compute(s);
        assert!((r.multiplier - 1.0).abs() < 0.01);
    }

    #[test]
    fn high_load_produces_large_multiplier() {
        let mut g = SelfPacingGovernor::new_default();
        let s = PacingSignals::new(0.9, 0.9, 0.9);
        let r = g.compute(s);
        assert!(r.multiplier > 1.5, "multiplier={}", r.multiplier);
    }

    #[test]
    fn low_load_produces_small_multiplier() {
        let mut g = SelfPacingGovernor::new_default();
        let s = PacingSignals::new(0.1, 0.1, 0.1);
        let r = g.compute(s);
        assert!(r.multiplier < 1.0, "multiplier={}", r.multiplier);
    }

    #[test]
    fn hysteresis_limits_change_per_cycle() {
        let mut g = SelfPacingGovernor::new(0.1);
        let s1 = PacingSignals::new(0.99, 0.99, 0.99);
        let r1 = g.compute(s1);
        assert!(
            (r1.multiplier - 1.0).abs() <= 0.1001,
            "should be clamped by hysteresis, got {}",
            r1.multiplier
        );

        let s2 = PacingSignals::new(0.01, 0.01, 0.01);
        let r2 = g.compute(s2);
        assert!(
            (r1.multiplier - r2.multiplier).abs() <= 0.1001,
            "second step should also be clamped, diff={}",
            (r1.multiplier - r2.multiplier).abs()
        );
    }

    #[test]
    fn thinking_mode_transitions() {
        let mut g = SelfPacingGovernor::new_default();
        let r = g.compute(PacingSignals::new(0.9, 0.5, 0.5));
        assert_eq!(r.thinking_mode, ThinkingMode::Deep);
        let r = g.compute(PacingSignals::new(0.1, 0.5, 0.5));
        assert_eq!(r.thinking_mode, ThinkingMode::Fast);
        let r = g.compute(PacingSignals::new(0.4, 0.5, 0.5));
        assert_eq!(r.thinking_mode, ThinkingMode::Balanced);
    }

    #[test]
    fn skip_heavy_at_extreme_load() {
        let mut g = SelfPacingGovernor::new_default();
        let r = g.compute(PacingSignals::new(0.86, 0.86, 0.86));
        assert!(r.skip_heavy_handlers);
    }

    #[test]
    fn reset_restores_multiplier() {
        let mut g = SelfPacingGovernor::new_default();
        g.compute(PacingSignals::new(0.99, 0.99, 0.99));
        g.reset();
        assert!((g.current_multiplier - 1.0).abs() < 0.001);
    }

    #[test]
    fn apply_multiplier_to_config_basic() {
        let intervals = &[(10, "fast"), (120, "thinking")];
        let adapted = apply_multiplier_to_config(intervals, 2.0);
        assert_eq!(adapted.len(), 2);
        assert_eq!(adapted[0].0, 10);
        assert_eq!(adapted[0].2, 20);
        assert_eq!(adapted[1].0, 120);
        assert_eq!(adapted[1].2, 240);
    }

    #[test]
    fn apply_multiplier_never_below_one() {
        let intervals = &[(2, "tiny")];
        let adapted = apply_multiplier_to_config(intervals, 0.1);
        assert_eq!(adapted[0].2, 1);
    }

    #[test]
    fn extreme_saturation_maps_to_maximum() {
        let mut g = SelfPacingGovernor::new(1.0);
        let r = g.compute(PacingSignals::new(1.0, 1.0, 1.0));
        assert!((r.multiplier - 3.0).abs() < 0.01);
    }

    #[test]
    fn zero_load_maps_to_minimum() {
        let mut g = SelfPacingGovernor::new(1.0);
        let r = g.compute(PacingSignals::new(0.0, 0.0, 0.0));
        assert!((r.multiplier - 0.5).abs() < 0.01);
    }
}
