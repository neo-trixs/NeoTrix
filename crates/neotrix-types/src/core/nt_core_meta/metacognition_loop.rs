use super::self_model::{SelfModel, EvolutionEvent, EventKind};
use super::monitor::{MetaMonitor, HealthCheck, MetaAlert};
use super::weakness::{WeaknessAnalyzer, WeaknessReport};
use super::planner::{EvolutionPlanner, PlannedEvolution};

/// The main metacognitive loop that orchestrates self-awareness.
///
/// Phases:
///   1. SCAN  — Build/update SelfModel from filesystem
///   2. ANALYZE — Run WeaknessAnalyzer on SelfModel
///   3. MONITOR — Generate alerts, check trends
///   4. PLAN   — Prioritize weaknesses into evolution actions
///   5. REPORT — Summarize findings for external consumption
///
/// This is a **synchronous** loop (no tokio dependency) — the `core/` layer
/// is runtime-agnostic. Async scheduling happens in `reasoning_brain/`.
#[derive(Debug, Clone)]
pub struct MetaCognitiveLoop {
    pub self_model: SelfModel,
    pub monitor: MetaMonitor,
    pub analyzer: WeaknessAnalyzer,
    pub planner: EvolutionPlanner,
    pub iteration: usize,
    pub max_iterations: usize,
}

impl MetaCognitiveLoop {
    pub fn new(model: SelfModel) -> Self {
        let monitor = MetaMonitor::new(model.clone());
        Self {
            self_model: model,
            monitor,
            analyzer: WeaknessAnalyzer::new(),
            planner: EvolutionPlanner::new(),
            iteration: 0,
            max_iterations: 100,
        }
    }

    /// Run one full metacognitive cycle.
    pub fn run_cycle(&mut self) -> MetaCycleResult {
        self.iteration += 1;

        let report = self.analyzer.analyze(&self.self_model);
        self.self_model.tech_debt.items = self.analyzer.to_tech_debt_items(&report);
        self.self_model.tech_debt.total_count = report.weaknesses.len();

        self.monitor.weaknesses_to_alerts(&report);
        let health_check = self.monitor.run_check();

        let plans = self.planner.plan_from_report(&report);

        let trend = self.monitor.trend_analysis();

        self.self_model.register_evolution(EvolutionEvent {
            timestamp: chrono::Utc::now(),
            kind: EventKind::MetaCognitionUpdated,
            description: format!("Cycle {}: {} weaknesses found, {} alerts, {} plans generated",
                self.iteration, report.summary.total_count,
                self.monitor.alerts.len(), plans.len()),
            affected_modules: Vec::new(),
        });

        MetaCycleResult {
            iteration: self.iteration,
            health_check,
            report,
            alerts: self.monitor.alerts.clone(),
            plans,
            trend,
            model_snapshot: self.self_model.clone(),
        }
    }

    /// Run multiple cycles, returning the result of each.
    pub fn run_batch(&mut self, cycles: usize) -> Vec<MetaCycleResult> {
        let mut results = Vec::with_capacity(cycles);
        let remaining = self.max_iterations - self.iteration;
        let actual = cycles.min(remaining);
        for _ in 0..actual {
            results.push(self.run_cycle());
        }
        results
    }

    /// Run continuous cycles until a stopping condition is met.
    pub fn run_until(&mut self, mut should_stop: impl FnMut(&MetaCycleResult) -> bool) -> Vec<MetaCycleResult> {
        let mut results = Vec::new();
        while self.iteration < self.max_iterations {
            let result = self.run_cycle();
            if should_stop(&result) {
                break;
            }
            results.push(result);
        }
        results
    }

    pub fn reset(&mut self, model: SelfModel) {
        self.self_model = model.clone();
        self.monitor = MetaMonitor::new(model);
        self.planner = EvolutionPlanner::new();
        self.iteration = 0;
    }

    pub fn status_summary(&self) -> String {
        let trend = self.monitor.trend_analysis();
        format!(
            "MetaCognition Cycle {}/{} | {} weaknesses | {} alerts | {} plans pending | trend: {}",
            self.iteration, self.max_iterations,
            self.self_model.tech_debt.total_count,
            self.monitor.alerts.len(),
            self.planner.pending_count(),
            trend.overall,
        )
    }
}

#[derive(Debug, Clone)]
pub struct MetaCycleResult {
    pub iteration: usize,
    pub health_check: HealthCheck,
    pub report: WeaknessReport,
    pub alerts: Vec<MetaAlert>,
    pub plans: Vec<PlannedEvolution>,
    pub trend: super::monitor::HealthTrend,
    pub model_snapshot: SelfModel,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_loop() -> MetaCognitiveLoop {
        let model = SelfModel::new();
        MetaCognitiveLoop::new(model)
    }

    #[test]
    fn test_loop_run_cycle() {
        let mut metacog = make_test_loop();
        let result = metacog.run_cycle();
        assert_eq!(result.iteration, 1);
        assert!(result.health_check.compilation_ok);
        assert_eq!(metacog.iteration, 1);
    }

    #[test]
    fn test_loop_run_batch() {
        let mut metacog = make_test_loop();
        let results = metacog.run_batch(5);
        assert_eq!(results.len(), 5);
        assert_eq!(metacog.iteration, 5);
    }

    #[test]
    fn test_loop_run_until_stop() {
        let mut metacog = make_test_loop();
        let results = metacog.run_until(|r| r.iteration >= 3);
        assert!(results.len() <= 3);
        assert!(metacog.iteration <= 3);
    }

    #[test]
    fn test_loop_status_summary() {
        let mut metacog = make_test_loop();
        metacog.run_cycle();
        let summary = metacog.status_summary();
        assert!(summary.contains("MetaCognition Cycle"));
        assert!(summary.contains("weaknesses"));
    }

    #[test]
    fn test_loop_reset() {
        let mut metacog = make_test_loop();
        metacog.run_cycle();
        assert_eq!(metacog.iteration, 1);
        metacog.reset(SelfModel::new());
        assert_eq!(metacog.iteration, 0);
    }

    #[test]
    fn test_max_iterations_enforced() {
        let mut metacog = make_test_loop();
        metacog.max_iterations = 2;
        let results = metacog.run_batch(10);
        assert_eq!(results.len(), 2);
    }
}
