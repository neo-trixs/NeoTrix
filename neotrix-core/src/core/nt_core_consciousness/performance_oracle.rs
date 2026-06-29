use std::collections::{HashMap, VecDeque};

use super::consciousness_pipeline::IntegratedResult;
use super::resource_allocator::CognitiveProcess;

#[derive(Debug, Clone)]
pub struct OracleConfig {
    pub window_size: usize,
    pub bottleneck_threshold_ns: u64,
    pub min_samples_for_tuning: usize,
    pub enable_adaptive_budgets: bool,
    pub enable_auto_disable: bool,
    pub convergence_target: f64,
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self {
            window_size: 100,
            bottleneck_threshold_ns: 50_000_000,
            min_samples_for_tuning: 20,
            enable_adaptive_budgets: true,
            enable_auto_disable: false,
            convergence_target: 0.85,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StepMetrics {
    pub step_name: String,
    pub count: usize,
    pub avg_duration_ns: f64,
    pub min_duration_ns: u64,
    pub max_duration_ns: u64,
    pub success_rate: f64,
    pub p95_duration_ns: u64,
    pub is_bottleneck: bool,
}

#[derive(Debug, Clone)]
pub struct PipelineMetrics {
    pub window_cycles: usize,
    pub total_avg_duration_ns: f64,
    pub step_metrics: Vec<StepMetrics>,
    pub bottleneck_steps: Vec<String>,
    pub convergence_rate: f64,
    pub load_shed_rate: f64,
    pub avg_iterations: f64,
    pub allocator_effectiveness: f64,
}

#[derive(Debug, Clone)]
pub struct AdaptiveRecommendation {
    pub recommended_iterations: usize,
    pub recommended_budgets: Vec<(CognitiveProcess, f64)>,
    pub disable_suggestions: Vec<String>,
    pub allocator_weight_adjustments: HashMap<String, f64>,
    pub rationale: String,
}

#[derive(Debug, Clone)]
pub struct HealthDashboard {
    pub overall_health: f64,
    pub step_health: Vec<(String, f64)>,
    pub trend: TrendDirection,
    pub bottlenecks: Vec<String>,
    pub recommendations: Vec<String>,
    pub cycles_analyzed: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
}

impl TrendDirection {
    pub fn name(&self) -> &'static str {
        match self {
            TrendDirection::Improving => "improving",
            TrendDirection::Stable => "stable",
            TrendDirection::Declining => "declining",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceOracle {
    config: OracleConfig,
    results: VecDeque<IntegratedResult>,
    step_durations: HashMap<String, Vec<u64>>,
    step_successes: HashMap<String, Vec<bool>>,
    cycle_durations: Vec<u64>,
    convergence_history: Vec<f64>,
    iteration_history: Vec<usize>,
    load_shed_history: Vec<bool>,
}

impl PerformanceOracle {
    pub fn new(config: OracleConfig) -> Self {
        let window_size = config.window_size;
        Self {
            config,
            results: VecDeque::with_capacity(window_size),
            step_durations: HashMap::new(),
            step_successes: HashMap::new(),
            cycle_durations: Vec::with_capacity(window_size),
            convergence_history: Vec::with_capacity(window_size),
            iteration_history: Vec::with_capacity(window_size),
            load_shed_history: Vec::with_capacity(window_size),
        }
    }

    pub fn config(&self) -> &OracleConfig {
        &self.config
    }

    pub fn observe(&mut self, result: IntegratedResult) {
        if self.results.len() >= self.config.window_size {
            let oldest = self.results.pop_front().unwrap();
            self.remove_oldest(&oldest);
        }

        for step in &result.step_timings {
            self.step_durations
                .entry(step.step_name.clone())
                .or_default()
                .push(step.duration_ns);
            self.step_successes
                .entry(step.step_name.clone())
                .or_default()
                .push(step.success);
        }

        self.cycle_durations.push(result.total_duration_ns);
        self.iteration_history
            .push(result.refinery.metrics.iteration);
        self.convergence_history
            .push(match result.refinery.metrics.convergence_signal {
                super::consciousness_refinery::ConvergenceSignal::Converged => 1.0,
                super::consciousness_refinery::ConvergenceSignal::NotConverged => 0.5,
                _ => 0.0,
            });
        self.load_shed_history.push(result.load_shed_active);

        self.results.push_back(result);
    }

    fn remove_oldest(&mut self, oldest: &IntegratedResult) {
        for step in &oldest.step_timings {
            if let Some(durations) = self.step_durations.get_mut(&step.step_name) {
                durations.remove(0);
            }
            if let Some(successes) = self.step_successes.get_mut(&step.step_name) {
                successes.remove(0);
            }
        }
        self.cycle_durations.remove(0);
        self.iteration_history.remove(0);
        self.convergence_history.remove(0);
        self.load_shed_history.remove(0);
    }

    fn sorted_durations(durations: &[u64]) -> Vec<u64> {
        let mut sorted = durations.to_vec();
        sorted.sort_unstable();
        sorted
    }

    fn p95(durations: &[u64]) -> u64 {
        if durations.is_empty() {
            return 0;
        }
        let sorted = Self::sorted_durations(durations);
        let idx = ((sorted.len() as f64) * 0.95).ceil() as usize - 1;
        sorted[idx.min(sorted.len() - 1)]
    }

    fn mean(values: &[u64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        values.iter().sum::<u64>() as f64 / values.len() as f64
    }

    fn mean_f64(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        values.iter().sum::<f64>() / values.len() as f64
    }

    pub fn step(&self, name: &str) -> Option<StepMetrics> {
        let durations = self.step_durations.get(name)?;
        let successes = self.step_successes.get(name)?;
        if durations.is_empty() {
            return None;
        }

        let success_count = successes.iter().filter(|&&s| s).count();
        let p95 = Self::p95(durations);

        Some(StepMetrics {
            step_name: name.to_string(),
            count: durations.len(),
            avg_duration_ns: Self::mean(durations),
            min_duration_ns: *durations.iter().min().unwrap_or(&0),
            max_duration_ns: *durations.iter().max().unwrap_or(&0),
            success_rate: success_count as f64 / successes.len() as f64,
            p95_duration_ns: p95,
            is_bottleneck: p95 > self.config.bottleneck_threshold_ns,
        })
    }

    pub fn metrics(&self) -> PipelineMetrics {
        let step_names: Vec<String> = {
            let mut names: Vec<String> = self.step_durations.keys().cloned().collect();
            names.sort();
            names
        };

        let step_metrics: Vec<StepMetrics> = step_names
            .iter()
            .filter_map(|name| self.step(name))
            .collect();

        let bottleneck_steps: Vec<String> = step_metrics
            .iter()
            .filter(|m| m.is_bottleneck)
            .map(|m| m.step_name.clone())
            .collect();

        PipelineMetrics {
            window_cycles: self.results.len(),
            total_avg_duration_ns: Self::mean(&self.cycle_durations),
            step_metrics,
            bottleneck_steps,
            convergence_rate: Self::mean_f64(&self.convergence_history),
            load_shed_rate: if self.load_shed_history.is_empty() {
                0.0
            } else {
                self.load_shed_history.iter().filter(|&&b| b).count() as f64
                    / self.load_shed_history.len() as f64
            },
            avg_iterations: if self.iteration_history.is_empty() {
                0.0
            } else {
                self.iteration_history.iter().sum::<usize>() as f64
                    / self.iteration_history.len() as f64
            },
            allocator_effectiveness: self.compute_allocator_effectiveness(),
        }
    }

    fn compute_allocator_effectiveness(&self) -> f64 {
        if self.results.len() < 2 {
            return 0.5;
        }
        let recent: Vec<&IntegratedResult> = self.results.iter().rev().take(10).collect();
        let successful = recent.iter().filter(|r| r.all_passed).count();
        let convolved = recent
            .iter()
            .filter(|r| {
                let sig = r.refinery.metrics.convergence_signal;
                matches!(
                    sig,
                    super::consciousness_refinery::ConvergenceSignal::Converged
                )
            })
            .count();
        let score = (successful as f64 + convolved as f64) / (2.0 * recent.len().max(1) as f64);
        score.clamp(0.0, 1.0)
    }

    pub fn recommend(&self) -> AdaptiveRecommendation {
        let metrics = self.metrics();
        let mut rationale_parts = Vec::new();
        let mut budget_adjustments = Vec::new();
        let mut disable_suggestions = Vec::new();

        // Iterations: base on convergence rate and bottleneck pressure
        let convergence_shortfall =
            (self.config.convergence_target - metrics.convergence_rate).max(0.0);
        let iter_boost = (convergence_shortfall * 5.0).round() as usize;
        let recommended_iterations = (3 + iter_boost).min(10);
        rationale_parts.push(format!(
            "convergence={:.2} target={:.2} → iter={}",
            metrics.convergence_rate, self.config.convergence_target, recommended_iterations
        ));

        // Budget: rebalance toward bottleneck steps
        if self.config.enable_adaptive_budgets {
            for step_metric in &metrics.step_metrics {
                if step_metric.is_bottleneck {
                    let process = match step_metric.step_name.as_str() {
                        "resource_allocation" => CognitiveProcess::Gather,
                        "refinery_loop" => CognitiveProcess::Refine,
                        "dual_path_inference" => CognitiveProcess::DualReason,
                        "spectrum_signal" => CognitiveProcess::Propose,
                        "belief_verification" => CognitiveProcess::Verify,
                        "blackboard_sync" => CognitiveProcess::BlackboardSync,
                        "meta_evolution_assessment" => CognitiveProcess::EvolutionAssess,
                        _ => continue,
                    };
                    let boost = (1.0
                        + step_metric.p95_duration_ns as f64
                            / self.config.bottleneck_threshold_ns as f64)
                        .min(2.0);
                    budget_adjustments.push((process, boost));
                    rationale_parts.push(format!(
                        "boost {} by {:.1}x (bottleneck p95={}ns)",
                        step_metric.step_name, boost, step_metric.p95_duration_ns
                    ));
                }
            }
        }

        // Auto-disable: if a step has 0% success rate over the window
        if self.config.enable_auto_disable {
            for step_metric in &metrics.step_metrics {
                if step_metric.success_rate < 0.05 && step_metric.count >= 5 {
                    disable_suggestions.push(step_metric.step_name.clone());
                    rationale_parts.push(format!(
                        "suggest disable {} (success={:.0}%)",
                        step_metric.step_name,
                        step_metric.success_rate * 100.0
                    ));
                }
            }
        }

        // Allocator weight adjustments (simple heuristic)
        let mut weight_adjustments = HashMap::new();
        if metrics.convergence_rate < 0.6 {
            weight_adjustments.insert("uncertainty_weight".into(), 0.5);
            rationale_parts.push("increase uncertainty_weight for more refine cycles".into());
        }
        if metrics.load_shed_rate > 0.3 {
            weight_adjustments.insert("surprise_weight".into(), 0.2);
            rationale_parts.push("reduce surprise_weight to avoid load_shed storms".into());
        }

        AdaptiveRecommendation {
            recommended_iterations,
            recommended_budgets: budget_adjustments,
            disable_suggestions,
            allocator_weight_adjustments: weight_adjustments,
            rationale: rationale_parts.join("; "),
        }
    }

    pub fn dashboard(&self) -> HealthDashboard {
        let metrics = self.metrics();
        let rec = self.recommend();

        let mut step_health: Vec<(String, f64)> = metrics
            .step_metrics
            .iter()
            .map(|m| {
                let score = m.success_rate * 0.6
                    + (1.0
                        - (m.p95_duration_ns as f64 / self.config.bottleneck_threshold_ns as f64)
                            .min(1.0))
                        * 0.4;
                (m.step_name.clone(), score)
            })
            .collect();
        step_health.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let convergence_score = metrics.convergence_rate;
        let load_score = 1.0 - metrics.load_shed_rate;
        let alloc_score = metrics.allocator_effectiveness;
        let overall_health =
            (convergence_score * 0.4 + load_score * 0.3 + alloc_score * 0.3).clamp(0.0, 1.0);

        // Trend: compare first 30% vs last 30% of window
        let trend = if self.cycle_durations.len() >= 10 {
            let third = self.cycle_durations.len() / 3;
            let early_avg = self.cycle_durations[..third].iter().sum::<u64>() as f64 / third as f64;
            let late_avg = self.cycle_durations[self.cycle_durations.len() - third..]
                .iter()
                .sum::<u64>() as f64
                / third as f64;
            if late_avg < early_avg * 0.9 {
                TrendDirection::Improving
            } else if late_avg > early_avg * 1.1 {
                TrendDirection::Declining
            } else {
                TrendDirection::Stable
            }
        } else {
            TrendDirection::Stable
        };

        let recommendations =
            if rec.recommended_budgets.is_empty() && rec.disable_suggestions.is_empty() {
                vec!["pipeline operating nominally".into()]
            } else {
                let mut recs = Vec::new();
                for (proc, boost) in &rec.recommended_budgets {
                    recs.push(format!("increase budget for {:?} by {:.1}x", proc, boost));
                }
                for step in &rec.disable_suggestions {
                    recs.push(format!("consider disabling {} (consistent failure)", step));
                }
                for (key, val) in &rec.allocator_weight_adjustments {
                    recs.push(format!("set {}={}", key, val));
                }
                recs
            };

        HealthDashboard {
            overall_health,
            step_health,
            trend,
            bottlenecks: metrics.bottleneck_steps,
            recommendations,
            cycles_analyzed: metrics.window_cycles,
        }
    }

    pub fn clear(&mut self) {
        self.results.clear();
        self.step_durations.clear();
        self.step_successes.clear();
        self.cycle_durations.clear();
        self.convergence_history.clear();
        self.iteration_history.clear();
        self.load_shed_history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::consciousness_pipeline::PipelineStepResult;

    fn fake_step(name: &str, duration_ns: u64, success: bool) -> PipelineStepResult {
        PipelineStepResult {
            step_name: name.into(),
            duration_ns,
            success,
            details: String::new(),
        }
    }

    fn fake_result(
        cycle: u64,
        steps: Vec<PipelineStepResult>,
        converged: bool,
        shed: bool,
        iters: usize,
    ) -> IntegratedResult {
        use super::super::consciousness_cycle::{CycleResult, CycleStep};
        use super::super::consciousness_refinery::{ConvergenceSignal, RefineryMetrics};

        IntegratedResult {
            cycle_number: cycle,
            refinery: super::super::consciousness_refinery::RefineryResult {
                final_cycle: CycleResult {
                    cycle_num: cycle as u64,
                    steps_completed: vec![],
                    step_health: vec![],
                    overall_success: true,
                    total_duration_ms: 0,
                    output_state: None,
                    c_score: 0.5,
                    steps_executed: vec![],
                    substrate_concepts: vec![],
                    causal_counterfactuals: vec![],
                    neuromodulator_report: None,
                    dashboard_report: None,
                    phi_metrics: None,
                    meta_insights: vec![],
                    rsi_proposals_count: 0,
                    qualia5: None,
                    extracted_content: None,
                    metabolic_state: "normal".to_string(),
                    irreversible_cost: 0,
                    evaluation_delegated: true,
                },
                metrics: RefineryMetrics {
                    iteration: iters,
                    total_cycles: cycle,
                    convergence_signal: if converged {
                        ConvergenceSignal::Converged
                    } else {
                        ConvergenceSignal::NotConverged
                    },
                    delta_vsa: 0.1,
                    delta_ema: 0.1,
                    cycle_durations_ms: vec![],
                    early_exited: false,
                },
                all_iterations: vec![],
            },
            dual_path: None,
            verification: None,
            spectrum_candidates: vec![],
            blackboard_synced: true,
            episodic_recorded: true,
            load_shed_active: shed,
            allocation: vec![],
            meta_recommendations: vec![],
            step_timings: steps,
            total_duration_ns: 0,
            all_passed: true,
        }
    }

    #[test]
    fn test_oracle_default_config() {
        let config = OracleConfig::default();
        assert_eq!(config.window_size, 100);
        assert!(config.enable_adaptive_budgets);
    }

    #[test]
    fn test_observe_single_result() {
        let mut oracle = PerformanceOracle::new(OracleConfig::default());
        let result = fake_result(
            1,
            vec![
                fake_step("refinery_loop", 10_000_000, true),
                fake_step("belief_verification", 5_000_000, true),
            ],
            true,
            false,
            3,
        );

        oracle.observe(result);
        assert_eq!(oracle.results.len(), 1);
        assert_eq!(oracle.step_durations.len(), 2);
    }

    #[test]
    fn test_step_metrics() {
        let mut oracle = PerformanceOracle::new(OracleConfig::default());
        for i in 0..10 {
            oracle.observe(fake_result(
                i,
                vec![fake_step(
                    "refinery_loop",
                    20_000_000 + i as u64 * 1_000_000,
                    i % 3 != 0,
                )],
                true,
                false,
                3,
            ));
        }

        let sm = oracle.step("refinery_loop").unwrap();
        assert_eq!(sm.count, 10);
        assert!(sm.success_rate > 0.6);
        assert!(sm.avg_duration_ns > 20_000_000.0);
    }

    #[test]
    fn test_metrics_aggregation() {
        let mut oracle = PerformanceOracle::new(OracleConfig::default());
        for i in 0..25 {
            oracle.observe(fake_result(
                i,
                vec![fake_step("refinery_loop", 10_000_000, true)],
                i % 4 != 0,
                i % 10 == 0,
                3usize + (i % 3) as usize,
            ));
        }

        let metrics = oracle.metrics();
        assert_eq!(metrics.window_cycles, 25);
        assert!(metrics.convergence_rate > 0.5);
        assert!(metrics.avg_iterations > 0.0);
    }

    #[test]
    fn test_bottleneck_detection() {
        let mut oracle = PerformanceOracle::new(OracleConfig {
            bottleneck_threshold_ns: 15_000_000,
            ..OracleConfig::default()
        });
        for i in 0..20 {
            oracle.observe(fake_result(
                i,
                vec![
                    fake_step("fast_step", 1_000_000, true),
                    fake_step("slow_step", 30_000_000, true),
                ],
                true,
                false,
                3,
            ));
        }

        let metrics = oracle.metrics();
        assert!(metrics.bottleneck_steps.contains(&"slow_step".into()));
        assert!(!metrics.bottleneck_steps.contains(&"fast_step".into()));
    }

    #[test]
    fn test_recommend_iterations() {
        let mut oracle = PerformanceOracle::new(OracleConfig::default());
        // Low convergence → should recommend more iterations
        for i in 0..30 {
            oracle.observe(fake_result(
                i,
                vec![fake_step("refinery_loop", 10_000_000, true)],
                false,
                false,
                2,
            ));
        }

        let rec = oracle.recommend();
        assert!(rec.recommended_iterations > 3);
        assert!(!rec.rationale.is_empty());
    }

    #[test]
    fn test_recommend_bottleneck_budget_boost() {
        let mut oracle = PerformanceOracle::new(OracleConfig {
            bottleneck_threshold_ns: 10_000_000,
            enable_adaptive_budgets: true,
            ..OracleConfig::default()
        });
        for i in 0..20 {
            oracle.observe(fake_result(
                i,
                vec![fake_step("refinery_loop", 50_000_000, true)],
                true,
                false,
                3,
            ));
        }

        let rec = oracle.recommend();
        assert!(rec
            .recommended_budgets
            .iter()
            .any(|(p, _)| *p == CognitiveProcess::Refine));
    }

    #[test]
    fn test_dashboard_health_score() {
        let mut oracle = PerformanceOracle::new(OracleConfig::default());
        for i in 0..30 {
            oracle.observe(fake_result(
                i,
                vec![
                    fake_step("refinery_loop", 10_000_000, true),
                    fake_step("dual_path_inference", 5_000_000, true),
                    fake_step("belief_verification", 3_000_000, true),
                ],
                true,
                false,
                3,
            ));
        }

        let dash = oracle.dashboard();
        assert!(dash.overall_health > 0.5);
        assert!(!dash.step_health.is_empty());
    }

    #[test]
    fn test_dashboard_detects_decline() {
        let mut oracle = PerformanceOracle::new(OracleConfig::default());
        for i in 0..30 {
            let dur = if i > 15 {
                100_000_000 + i as u64 * 5_000_000
            } else {
                10_000_000
            };
            oracle.observe(fake_result(
                i,
                vec![fake_step("refinery_loop", dur, true)],
                true,
                false,
                3,
            ));
        }

        let dash = oracle.dashboard();
        // The oracle should detect the trend
        assert_eq!(dash.trend, TrendDirection::Declining);
    }

    #[test]
    fn test_dashboard_detects_improvement() {
        let mut oracle = PerformanceOracle::new(OracleConfig::default());
        for i in 0..30 {
            let dur = if i > 15 {
                10_000_000
            } else {
                100_000_000 - i as u64 * 5_000_000
            };
            oracle.observe(fake_result(
                i,
                vec![fake_step("refinery_loop", dur, true)],
                true,
                false,
                3,
            ));
        }

        let dash = oracle.dashboard();
        assert_eq!(dash.trend, TrendDirection::Improving);
    }

    #[test]
    fn test_clear_resets_state() {
        let mut oracle = PerformanceOracle::new(OracleConfig::default());
        for i in 0..10 {
            oracle.observe(fake_result(
                i,
                vec![fake_step("s", 1_000_000, true)],
                true,
                false,
                3,
            ));
        }
        assert_eq!(oracle.results.len(), 10);
        oracle.clear();
        assert_eq!(oracle.results.len(), 0);
        assert!(oracle.step_durations.is_empty());
    }

    #[test]
    fn test_empty_oracle_metrics() {
        let oracle = PerformanceOracle::new(OracleConfig::default());
        let metrics = oracle.metrics();
        assert_eq!(metrics.window_cycles, 0);
        assert!(metrics.step_metrics.is_empty());
    }

    #[test]
    fn test_empty_oracle_recommend() {
        let oracle = PerformanceOracle::new(OracleConfig::default());
        let rec = oracle.recommend();
        assert_eq!(rec.recommended_iterations, 3);
    }

    #[test]
    fn test_empty_oracle_dashboard() {
        let oracle = PerformanceOracle::new(OracleConfig::default());
        let dash = oracle.dashboard();
        assert_eq!(dash.trend, TrendDirection::Stable);
        assert!(dash.bottlenecks.is_empty());
    }

    #[test]
    fn test_p95_calculation() {
        let mut oracle = PerformanceOracle::new(OracleConfig::default());
        for i in 0..20 {
            oracle.observe(fake_result(
                i,
                vec![fake_step("test", 1_000_000 * (i as u64 + 1), true)],
                true,
                false,
                3,
            ));
        }
        let sm = oracle.step("test").unwrap();
        assert!(sm.p95_duration_ns > 1_000_000 * 18);
    }

    #[test]
    fn test_oracle_window_eviction() {
        let mut oracle = PerformanceOracle::new(OracleConfig {
            window_size: 5,
            ..OracleConfig::default()
        });
        for i in 0..10 {
            oracle.observe(fake_result(
                i,
                vec![fake_step("s", 1_000_000, true)],
                true,
                false,
                3,
            ));
        }
        assert_eq!(oracle.results.len(), 5);
    }

    #[test]
    fn test_allocator_effectiveness_score() {
        let mut oracle = PerformanceOracle::new(OracleConfig::default());
        for i in 0..15 {
            oracle.observe(fake_result(
                i,
                vec![fake_step("refinery_loop", 10_000_000, true)],
                true,
                false,
                3,
            ));
        }
        let metrics = oracle.metrics();
        assert!(metrics.allocator_effectiveness > 0.4);
    }

    #[test]
    fn test_trend_insufficient_data_stable() {
        let oracle = PerformanceOracle::new(OracleConfig::default());
        assert_eq!(oracle.dashboard().trend, TrendDirection::Stable);
    }
}
