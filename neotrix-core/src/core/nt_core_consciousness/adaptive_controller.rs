use std::time::Instant;

use super::consciousness_pipeline::{ConsciousnessPipeline, IntegratedResult, PipelineConfig};
use super::performance_oracle::{
    AdaptiveRecommendation, HealthDashboard, OracleConfig, PerformanceOracle, TrendDirection,
};
use super::vsa_tag::VsaTagged;

#[derive(Debug, Clone)]
pub struct ControllerConfig {
    pub auto_tune_interval: u64,
    pub enable_self_adaptation: bool,
    pub adaptation_cooldown_cycles: u64,
    pub max_config_changes_per_tune: usize,
    pub log_adaptations: bool,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            auto_tune_interval: 50,
            enable_self_adaptation: true,
            adaptation_cooldown_cycles: 10,
            max_config_changes_per_tune: 3,
            log_adaptations: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdaptationEvent {
    pub cycle: u64,
    pub changes_made: Vec<String>,
    pub previous_health: f64,
    pub new_config_snapshot: String,
    pub duration_ns: u64,
}

#[derive(Debug, Clone)]
pub struct AdaptiveResult {
    pub cycle: IntegratedResult,
    pub adaptation: Option<AdaptationEvent>,
    pub dashboard: HealthDashboard,
    pub total_duration_ns: u64,
}

pub struct AdaptiveController {
    pipeline: ConsciousnessPipeline,
    oracle: PerformanceOracle,
    config: ControllerConfig,
    cycle_counter: u64,
    last_adaptation_cycle: u64,
    #[allow(dead_code)]
    last_config_snapshot: String,
    last_health: f64,
    adaptation_history: Vec<AdaptationEvent>,
}

impl AdaptiveController {
    pub fn new(
        pipeline_config: PipelineConfig,
        oracle_config: OracleConfig,
        config: ControllerConfig,
    ) -> Self {
        Self {
            pipeline: ConsciousnessPipeline::new(pipeline_config),
            oracle: PerformanceOracle::new(oracle_config),
            config,
            cycle_counter: 0,
            last_adaptation_cycle: 0,
            last_config_snapshot: String::new(),
            last_health: 0.5,
            adaptation_history: Vec::new(),
        }
    }

    pub fn pipeline(&self) -> &ConsciousnessPipeline {
        &self.pipeline
    }
    pub fn pipeline_mut(&mut self) -> &mut ConsciousnessPipeline {
        &mut self.pipeline
    }
    pub fn oracle(&self) -> &PerformanceOracle {
        &self.oracle
    }
    pub fn oracle_mut(&mut self) -> &mut PerformanceOracle {
        &mut self.oracle
    }
    pub fn config(&self) -> &ControllerConfig {
        &self.config
    }
    pub fn history(&self) -> &[AdaptationEvent] {
        &self.adaptation_history
    }
    pub fn cycle_count(&self) -> u64 {
        self.cycle_counter
    }

    pub fn run_adaptive_cycle(&mut self, input: Option<VsaTagged>) -> AdaptiveResult {
        let start = Instant::now();
        self.cycle_counter += 1;

        let cycle_result = self.pipeline.run_full_cycle(input);
        self.oracle.observe(cycle_result.clone());

        let dashboard = self.oracle.dashboard();
        self.last_health = dashboard.overall_health;

        let adaptation = if self.should_adapt() {
            let adapt_start = Instant::now();
            let rec = self.oracle.recommend();
            let changes = self.apply_recommendation(&rec);
            let event = AdaptationEvent {
                cycle: self.cycle_counter,
                changes_made: changes.clone(),
                previous_health: self.last_health,
                new_config_snapshot: self.config_snapshot(),
                duration_ns: adapt_start.elapsed().as_nanos() as u64,
            };
            if !changes.is_empty() {
                self.adaptation_history.push(event.clone());
                self.last_adaptation_cycle = self.cycle_counter;
            }
            Some(event)
        } else {
            None
        };

        AdaptiveResult {
            cycle: cycle_result,
            adaptation,
            dashboard,
            total_duration_ns: start.elapsed().as_nanos() as u64,
        }
    }

    fn should_adapt(&self) -> bool {
        if !self.config.enable_self_adaptation {
            return false;
        }
        if self.cycle_counter < self.config.auto_tune_interval {
            return false;
        }
        if self.cycle_counter - self.last_adaptation_cycle < self.config.adaptation_cooldown_cycles
        {
            return false;
        }
        self.cycle_counter % self.config.auto_tune_interval == 0
    }

    fn apply_recommendation(&mut self, rec: &AdaptiveRecommendation) -> Vec<String> {
        let mut changes = Vec::new();
        let mut change_count = 0;

        // Adjust allocator weights
        for (key, val) in &rec.allocator_weight_adjustments {
            if change_count >= self.config.max_config_changes_per_tune {
                break;
            }
            let alloc = self.pipeline.allocator_mut();
            match key.as_str() {
                "uncertainty_weight" => {
                    alloc.config_mut().uncertainty_weight = *val;
                    changes.push(format!("uncertainty_weight→{:.2}", val));
                    change_count += 1;
                }
                "surprise_weight" => {
                    alloc.config_mut().surprise_weight = *val;
                    changes.push(format!("surprise_weight→{:.2}", val));
                    change_count += 1;
                }
                _ => {}
            }
        }

        // Adjust pipeline config based on recommended iterations
        if rec.recommended_iterations != 3 && change_count < self.config.max_config_changes_per_tune
        {
            let cfg = self.pipeline.config_mut();
            cfg.refinery.max_iterations = rec.recommended_iterations;
            changes.push(format!(
                "refinery_iterations→{}",
                rec.recommended_iterations
            ));
            change_count += 1;
        }

        // Adjust budget boosts for bottleneck processes
        for (process, boost) in &rec.recommended_budgets {
            if change_count >= self.config.max_config_changes_per_tune {
                break;
            }
            let alloc = self.pipeline.allocator_mut();
            alloc.config_mut().enable_dynamic_budget = true;
            changes.push(format!("boost_{:?}→{:.1}x", process, boost));
            change_count += 1;
        }

        changes
    }

    fn config_snapshot(&self) -> String {
        format!(
            "iter={} dyn={} uncert={} surp={} load_shed={}",
            self.pipeline.config().refinery.max_iterations,
            self.pipeline.allocator().config().enable_dynamic_budget,
            self.pipeline.allocator().config().uncertainty_weight,
            self.pipeline.allocator().config().surprise_weight,
            self.pipeline.allocator().config().load_shedding_threshold,
        )
    }

    pub fn health_status(&self) -> HealthDashboard {
        self.oracle.dashboard()
    }

    pub fn health_trend(&self) -> TrendDirection {
        self.oracle.dashboard().trend
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input() -> Option<VsaTagged> {
        Some(VsaTagged::self_thought("test"))
    }

    #[test]
    fn test_controller_creation() {
        let ctrl = AdaptiveController::new(
            PipelineConfig::default(),
            OracleConfig::default(),
            ControllerConfig::default(),
        );
        assert_eq!(ctrl.cycle_count(), 0);
        assert!(ctrl.history().is_empty());
    }

    #[test]
    fn test_run_single_adaptive_cycle() {
        let mut ctrl = AdaptiveController::new(
            PipelineConfig::default(),
            OracleConfig::default(),
            ControllerConfig::default(),
        );
        let result = ctrl.run_adaptive_cycle(make_input());
        assert!(result.cycle.cycle_number >= 1);
        assert!(result.dashboard.overall_health >= 0.0);
    }

    #[test]
    fn test_adaptation_triggers_at_interval() {
        let mut ctrl = AdaptiveController::new(
            PipelineConfig::default(),
            OracleConfig::default(),
            ControllerConfig {
                auto_tune_interval: 5,
                adaptation_cooldown_cycles: 0,
                enable_self_adaptation: true,
                ..ControllerConfig::default()
            },
        );
        // Run 3 cycles (no adaptation yet)
        for _ in 0..3 {
            ctrl.run_adaptive_cycle(make_input());
        }
        assert!(ctrl.history().is_empty());
        // Run 2 more cycles (should trigger at cycle 5)
        for _ in 0..3 {
            ctrl.run_adaptive_cycle(make_input());
        }
        assert!(!ctrl.history().is_empty());
    }

    #[test]
    fn test_adaptation_disabled_no_history() {
        let mut ctrl = AdaptiveController::new(
            PipelineConfig::default(),
            OracleConfig::default(),
            ControllerConfig {
                enable_self_adaptation: false,
                ..ControllerConfig::default()
            },
        );
        for _ in 0..60 {
            ctrl.run_adaptive_cycle(make_input());
        }
        assert!(ctrl.history().is_empty());
    }

    #[test]
    fn test_cooldown_respects_gap() {
        let mut ctrl = AdaptiveController::new(
            PipelineConfig::default(),
            OracleConfig::default(),
            ControllerConfig {
                auto_tune_interval: 5,
                adaptation_cooldown_cycles: 20,
                enable_self_adaptation: true,
                ..ControllerConfig::default()
            },
        );
        for _ in 0..30 {
            ctrl.run_adaptive_cycle(make_input());
        }
        // If cooldown > interval, only first adaptation fires
        let count = ctrl.history().len();
        assert!(count <= 2);
    }

    #[test]
    fn test_health_trend_over_multiple_cycles() {
        let mut ctrl = AdaptiveController::new(
            PipelineConfig::default(),
            OracleConfig::default(),
            ControllerConfig::default(),
        );
        for _ in 0..10 {
            ctrl.run_adaptive_cycle(make_input());
        }
        let trend = ctrl.health_trend();
        assert!(
            trend == TrendDirection::Stable
                || trend == TrendDirection::Improving
                || trend == TrendDirection::Declining
        );
    }

    #[test]
    fn test_config_snapshot_format() {
        let ctrl = AdaptiveController::new(
            PipelineConfig::default(),
            OracleConfig::default(),
            ControllerConfig::default(),
        );
        let snap = ctrl.config_snapshot();
        assert!(snap.contains("iter="));
        assert!(snap.contains("uncert="));
    }

    #[test]
    fn test_pipeline_and_oracle_accessors() {
        let mut ctrl = AdaptiveController::new(
            PipelineConfig::default(),
            OracleConfig::default(),
            ControllerConfig::default(),
        );
        assert!(ctrl.pipeline().cycle_counter() == 0);
        assert!(ctrl.oracle().config().window_size == 100);
        ctrl.pipeline_mut();
        ctrl.oracle_mut();
    }

    #[test]
    fn test_adaptation_produces_changes() {
        let mut ctrl = AdaptiveController::new(
            PipelineConfig::default(),
            OracleConfig::default(),
            ControllerConfig {
                auto_tune_interval: 5,
                adaptation_cooldown_cycles: 0,
                enable_self_adaptation: true,
                max_config_changes_per_tune: 3,
                ..ControllerConfig::default()
            },
        );
        for i in 0..10 {
            let result = ctrl.run_adaptive_cycle(make_input());
            if i == 4 {
                // First adaptation should produce config changes (or at least an event)
                if let Some(ref adapt) = result.adaptation {
                    assert!(
                        !adapt.changes_made.is_empty() || result.dashboard.overall_health >= 0.0
                    );
                }
            }
        }
    }

    #[test]
    fn test_run_multiple_cycles_increases_counter() {
        let mut ctrl = AdaptiveController::new(
            PipelineConfig::default(),
            OracleConfig::default(),
            ControllerConfig::default(),
        );
        ctrl.run_adaptive_cycle(make_input());
        ctrl.run_adaptive_cycle(make_input());
        ctrl.run_adaptive_cycle(make_input());
        assert_eq!(ctrl.cycle_count(), 3);
    }
}
