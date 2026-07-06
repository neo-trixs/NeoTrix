use std::collections::{HashMap, VecDeque};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskType {
    CodeGeneration,
    Reasoning,
    KnowledgeQA,
    ToolUse,
    Safety,
}

#[derive(Debug, Clone)]
pub struct BenchmarkTask {
    pub id: String,
    pub description: String,
    pub task_type: TaskType,
    pub expected_difficulty: f64,
    pub timeout_s: u64,
}

impl BenchmarkTask {
    pub fn new(id: &str, description: &str, task_type: TaskType, difficulty: f64) -> Self {
        Self { id: id.to_string(), description: description.to_string(), task_type, expected_difficulty: difficulty.max(0.0).min(1.0), timeout_s: 60 }
    }

    pub fn with_timeout(mut self, timeout_s: u64) -> Self { self.timeout_s = timeout_s; self }
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub task_id: String,
    pub score: f64,
    pub latency_ms: u64,
    pub tokens_used: u64,
    pub error: Option<String>,
}

impl BenchmarkResult {
    pub fn new(task_id: &str, score: f64) -> Self {
        Self { task_id: task_id.to_string(), score: score.max(0.0).min(1.0), latency_ms: 0, tokens_used: 0, error: None }
    }

    pub fn with_error(task_id: &str, error: &str) -> Self {
        Self { task_id: task_id.to_string(), score: 0.0, latency_ms: 0, tokens_used: 0, error: Some(error.to_string()) }
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkSuite {
    pub name: String,
    pub version: String,
    pub tasks: Vec<BenchmarkTask>,
}

impl BenchmarkSuite {
    pub fn new(name: &str, version: &str) -> Self {
        Self { name: name.to_string(), version: version.to_string(), tasks: Vec::new() }
    }

    pub fn add(&mut self, task: BenchmarkTask) { self.tasks.push(task); }

    pub fn pass_rate(&self, results: &[BenchmarkResult]) -> f64 {
        if results.is_empty() { return 0.0; }
        let mut total = 0.0;
        let mut count = 0;
        for result in results {
            if let Some(_task) = self.tasks.iter().find(|t| t.id == result.task_id) {
                total += result.score;
                count += 1;
            }
        }
        if count == 0 { 0.0 } else { total / count as f64 }
    }

    pub fn task_count(&self) -> usize { self.tasks.len() }
    pub fn by_type(&self, task_type: TaskType) -> impl Iterator<Item = &BenchmarkTask> {
        self.tasks.iter().filter(move |t| t.task_type == task_type)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationScope {
    Comprehensive,
    Targeted,
    Minimal,
}

impl MutationScope {
    pub fn from_pass_rate(rate: f64) -> Self {
        if rate < 0.3 { MutationScope::Comprehensive }
        else if rate < 0.7 { MutationScope::Targeted }
        else { MutationScope::Minimal }
    }

    pub fn description(&self) -> &str {
        match self {
            MutationScope::Comprehensive => "Can change architecture, add new modules, rewrite stages",
            MutationScope::Targeted => "Optimize specific stages, tune parameters, add small features",
            MutationScope::Minimal => "Tweak parameters, minor refactoring only",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvolutionStrategy {
    AdaptiveEvolve,
    GuidedSynthesis,
    SkillForge,
    Recombination,
    ParameterSearch,
}

impl EvolutionStrategy {
    pub fn select(scope: MutationScope, pass_rate: f64, cycle: u64) -> Self {
        if cycle > 0 && cycle % 5 == 0 { return EvolutionStrategy::Recombination; }
        match scope {
            MutationScope::Comprehensive => EvolutionStrategy::AdaptiveEvolve,
            MutationScope::Targeted => {
                if pass_rate < 0.5 { EvolutionStrategy::SkillForge }
                else { EvolutionStrategy::GuidedSynthesis }
            }
            MutationScope::Minimal => EvolutionStrategy::ParameterSearch,
        }
    }

    pub fn description(&self) -> &str {
        match self {
            EvolutionStrategy::AdaptiveEvolve => "Meta-learning: per-claim feedback loop",
            EvolutionStrategy::GuidedSynthesis => "LLM proposes mutations, evolver curates",
            EvolutionStrategy::SkillForge => "Workspace mutation with EGL gating",
            EvolutionStrategy::Recombination => "Merge top-K successful strategies",
            EvolutionStrategy::ParameterSearch => "Lightweight parameter search only",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkGate {
    pub required_pass_rate: f64,
    pub max_regression: f64,
}

impl Default for BenchmarkGate {
    fn default() -> Self { Self { required_pass_rate: 0.7, max_regression: -0.05 } }
}

impl BenchmarkGate {
    pub fn new(required: f64, max_regression: f64) -> Self {
        Self { required_pass_rate: required.max(0.0).min(1.0), max_regression }
    }

    pub fn check(&self, old_pass_rate: f64, new_pass_rate: f64) -> Result<(), String> {
        if new_pass_rate < self.required_pass_rate {
            return Err(format!("Pass rate {:.2} < required {:.2}", new_pass_rate, self.required_pass_rate));
        }
        let delta = new_pass_rate - old_pass_rate;
        if delta < self.max_regression {
            return Err(format!("Regression {:.3} < max {:.3}", delta, self.max_regression));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct EglSnapshot {
    pub pass_rate: f64,
    pub iteration: u64,
}

#[derive(Debug, Clone)]
pub enum EglStatus {
    Improving { avg: f64, current: f64 },
    Stable { avg: f64, current: f64 },
    Regressing { avg: f64, current: f64 },
}

impl EglStatus {
    pub fn is_regressing(&self) -> bool {
        matches!(self, EglStatus::Regressing { .. })
    }

    pub fn description(&self) -> String {
        match self {
            EglStatus::Improving { avg, current } => format!("Improving: avg={:.3}, current={:.3}", avg, current),
            EglStatus::Stable { avg, current } => format!("Stable: avg={:.3}, current={:.3}", avg, current),
            EglStatus::Regressing { avg, current } => format!("REGRESSING: avg={:.3}, current={:.3}", avg, current),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EglTracker {
    pub history: VecDeque<EglSnapshot>,
    pub window_size: usize,
    pub regression_threshold: f64,
    pub current_iteration: u64,
}

impl EglTracker {
    pub fn new() -> Self {
        Self { history: VecDeque::new(), window_size: 10, regression_threshold: -0.05, current_iteration: 0 }
    }

    pub fn with_window(mut self, window: usize) -> Self { self.window_size = window; self }
    pub fn with_threshold(mut self, threshold: f64) -> Self { self.regression_threshold = threshold; self }

    pub fn track(&mut self, pass_rate: f64) -> EglStatus {
        self.current_iteration += 1;
        let snapshot = EglSnapshot { pass_rate, iteration: self.current_iteration };
        self.history.push_back(snapshot);

        while self.history.len() > self.window_size {
            self.history.pop_front();
        }

        let avg: f64 = self.history.iter().map(|s| s.pass_rate).sum::<f64>() / self.history.len() as f64;

        if pass_rate < avg + self.regression_threshold {
            EglStatus::Regressing { avg, current: pass_rate }
        } else if pass_rate > avg + 0.05 {
            EglStatus::Improving { avg, current: pass_rate }
        } else {
            EglStatus::Stable { avg, current: pass_rate }
        }
    }

    pub fn rolling_average(&self) -> f64 {
        if self.history.is_empty() { return 0.0; }
        self.history.iter().map(|s| s.pass_rate).sum::<f64>() / self.history.len() as f64
    }

    pub fn reset(&mut self) { self.history.clear(); self.current_iteration = 0; }
}

#[derive(Debug, Clone)]
pub struct TraitStore {
    pub traits: Vec<(String, f64, Instant)>,
    pub max_traits: usize,
}

impl TraitStore {
    pub fn new() -> Self { Self { traits: Vec::new(), max_traits: 100 } }

    pub fn store(&mut self, name: &str, score: f64) {
        self.traits.push((name.to_string(), score, Instant::now()));
        if self.traits.len() > self.max_traits {
            self.traits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            self.traits.truncate(self.max_traits);
        }
    }

    pub fn get_top(&self, k: usize) -> Vec<&(String, f64, Instant)> {
        let mut sorted: Vec<_> = self.traits.iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted.truncate(k);
        sorted
    }

    pub fn best_score(&self) -> f64 {
        self.traits.iter().map(|(_, s, _)| *s).fold(0.0, f64::max)
    }

    pub fn count(&self) -> usize { self.traits.len() }
}

#[derive(Debug, Clone)]
pub struct EvolutionConfig {
    pub benchmark_suite: BenchmarkSuite,
    pub gate: BenchmarkGate,
    pub egl_window: usize,
    pub check_interval_cycles: u64,
    pub max_cycles: u64,
}

#[derive(Debug, Clone)]
pub struct CycleResult {
    pub iteration: u64,
    pub pass_rate: f64,
    pub scope: MutationScope,
    pub strategy: EvolutionStrategy,
    pub egl_status: EglStatus,
    pub accepted: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct EvolutionLoop {
    pub config: EvolutionConfig,
    pub egl_tracker: EglTracker,
    pub trait_store: TraitStore,
    pub iteration: u64,
}

impl EvolutionLoop {
    pub fn new(config: EvolutionConfig) -> Self {
        let egl_tracker = EglTracker::new().with_window(config.egl_window);
        Self { config, egl_tracker, trait_store: TraitStore::new(), iteration: 0 }
    }

    pub fn run_cycle(&mut self, scores: &[(String, f64)]) -> CycleResult {
        self.iteration += 1;

        let pass_rate = if scores.is_empty() { 0.0 }
            else { scores.iter().map(|(_, s)| s).sum::<f64>() / scores.len() as f64 };

        let scope = MutationScope::from_pass_rate(pass_rate);
        let strategy = EvolutionStrategy::select(scope, pass_rate, self.iteration);
        let egl_status = self.egl_tracker.track(pass_rate);

        let old_avg = self.egl_tracker.rolling_average();
        let gate_result = self.config.gate.check(old_avg, pass_rate);

        let accepted = gate_result.is_ok() && !egl_status.is_regressing();
        let message = if !accepted {
            format!("Rejected: {} | {}", egl_status.description(),
                gate_result.as_ref().err().map(|e| e.as_str()).unwrap_or("EGL regression"))
        } else {
            format!("Accepted: scope={:?}, strategy={:?}, pass_rate={:.3}", scope, strategy, pass_rate)
        };

        if accepted {
            self.trait_store.store(&format!("cycle_{}", self.iteration), pass_rate);
        }

        CycleResult { iteration: self.iteration, pass_rate, scope, strategy, egl_status, accepted, message }
    }

    pub fn should_continue(&self) -> bool {
        self.iteration < self.config.max_cycles
    }
}

// ============================================================================
// Multi-Dimensional Evaluation Types (added for P0 agent eval upgrade)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvalDimension {
    Correctness,
    Safety,
    Cost,
    Latency,
    Trajectory,
    Robustness,
}

impl EvalDimension {
    pub fn description(&self) -> &str {
        match self {
            EvalDimension::Correctness => "Task completion accuracy",
            EvalDimension::Safety => "Safety violation count (lower is better)",
            EvalDimension::Cost => "Token/dollar efficiency",
            EvalDimension::Latency => "Response time",
            EvalDimension::Trajectory => "Step efficiency",
            EvalDimension::Robustness => "Consistency across runs",
        }
    }

    pub fn all() -> [EvalDimension; 6] {
        [
            EvalDimension::Correctness,
            EvalDimension::Safety,
            EvalDimension::Cost,
            EvalDimension::Latency,
            EvalDimension::Trajectory,
            EvalDimension::Robustness,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct MultiDimResult {
    pub task_id: String,
    pub scores: HashMap<EvalDimension, f64>,
    pub raw_metrics: HashMap<String, f64>,
    pub cost_usd: f64,
    pub trajectory_steps: usize,
    pub trajectory_optimal_steps: Option<usize>,
    pub safety_violations: Vec<String>,
    pub error: Option<String>,
}

impl MultiDimResult {
    pub fn new(task_id: &str) -> Self {
        Self {
            task_id: task_id.to_string(),
            scores: HashMap::new(),
            raw_metrics: HashMap::new(),
            cost_usd: 0.0,
            trajectory_steps: 0,
            trajectory_optimal_steps: None,
            safety_violations: Vec::new(),
            error: None,
        }
    }

    pub fn with_score(mut self, dim: EvalDimension, score: f64) -> Self {
        self.scores.insert(dim, score.max(0.0).min(1.0));
        self
    }

    pub fn with_cost(mut self, cost_usd: f64) -> Self {
        self.cost_usd = cost_usd;
        self
    }

    pub fn with_trajectory(mut self, steps: usize, optimal: Option<usize>) -> Self {
        self.trajectory_steps = steps;
        self.trajectory_optimal_steps = optimal;
        self
    }

    pub fn with_safety_violation(mut self, violation: &str) -> Self {
        self.safety_violations.push(violation.to_string());
        self
    }

    pub fn with_safety_violations(mut self, violations: Vec<String>) -> Self {
        self.safety_violations = violations;
        self
    }

    pub fn with_error(mut self, error: &str) -> Self {
        self.error = Some(error.to_string());
        self
    }

    pub fn with_raw_metric(mut self, key: &str, value: f64) -> Self {
        self.raw_metrics.insert(key.to_string(), value);
        self
    }

    pub fn score(&self, dim: EvalDimension) -> f64 {
        self.scores.get(&dim).copied().unwrap_or(0.0)
    }

    pub fn composite(&self, weights: &DimensionWeights) -> f64 {
        let norm = weights.normalized();
        let mut total = 0.0;
        for dim in EvalDimension::all() {
            if let Some(&s) = self.scores.get(&dim) {
                total += s * norm.get(dim);
            }
        }
        total
    }
}

#[derive(Debug, Clone)]
pub struct EvalRunConfig {
    pub name: String,
    pub task_ids: Vec<String>,
    pub num_trials: usize,
    pub max_cost_usd: f64,
    pub track_trajectory: bool,
    pub safety_check: bool,
}

impl EvalRunConfig {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            task_ids: Vec::new(),
            num_trials: 1,
            max_cost_usd: 10.0,
            track_trajectory: false,
            safety_check: true,
        }
    }

    pub fn with_tasks(mut self, task_ids: Vec<String>) -> Self {
        self.task_ids = task_ids;
        self
    }

    pub fn with_trials(mut self, trials: usize) -> Self {
        self.num_trials = trials.max(1);
        self
    }

    pub fn with_max_cost(mut self, max_cost: f64) -> Self {
        self.max_cost_usd = max_cost;
        self
    }
}

#[derive(Debug, Clone)]
pub struct DimensionWeights {
    pub correctness: f64,
    pub safety: f64,
    pub cost: f64,
    pub latency: f64,
    pub trajectory: f64,
    pub robustness: f64,
}

impl Default for DimensionWeights {
    fn default() -> Self {
        Self {
            correctness: 0.4,
            safety: 0.25,
            cost: 0.15,
            latency: 0.1,
            trajectory: 0.05,
            robustness: 0.05,
        }
    }
}

impl DimensionWeights {
    pub fn new(
        correctness: f64,
        safety: f64,
        cost: f64,
        latency: f64,
        trajectory: f64,
        robustness: f64,
    ) -> Self {
        Self {
            correctness: correctness.max(0.0).min(1.0),
            safety: safety.max(0.0).min(1.0),
            cost: cost.max(0.0).min(1.0),
            latency: latency.max(0.0).min(1.0),
            trajectory: trajectory.max(0.0).min(1.0),
            robustness: robustness.max(0.0).min(1.0),
        }
    }

    pub fn total(&self) -> f64 {
        self.correctness
            + self.safety
            + self.cost
            + self.latency
            + self.trajectory
            + self.robustness
    }

    pub fn normalized(&self) -> Self {
        let t = self.total();
        if t == 0.0 {
            return Self::default();
        }
        Self {
            correctness: self.correctness / t,
            safety: self.safety / t,
            cost: self.cost / t,
            latency: self.latency / t,
            trajectory: self.trajectory / t,
            robustness: self.robustness / t,
        }
    }

    pub fn get(&self, dim: EvalDimension) -> f64 {
        match dim {
            EvalDimension::Correctness => self.correctness,
            EvalDimension::Safety => self.safety,
            EvalDimension::Cost => self.cost,
            EvalDimension::Latency => self.latency,
            EvalDimension::Trajectory => self.trajectory,
            EvalDimension::Robustness => self.robustness,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvalReport {
    pub suite_name: String,
    pub config: EvalRunConfig,
    pub dimension_scores: HashMap<EvalDimension, f64>,
    pub composite_score: f64,
    pub by_task_type: HashMap<TaskType, HashMap<EvalDimension, f64>>,
    pub total_cost_usd: f64,
    pub total_safety_violations: usize,
    pub avg_trajectory_efficiency: f64,
    pub contamination_flags: Vec<String>,
    pub contamination_adjusted_score: Option<f64>,
    pub num_tasks_evaluated: usize,
    pub timestamp: u64,
}

impl EvalReport {
    pub fn summary_string(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("=== Eval Report: {} ===\n", self.suite_name));
        s.push_str(&format!(
            "Config: {} ({} tasks, {} trials)\n",
            self.config.name, self.num_tasks_evaluated, self.config.num_trials
        ));
        s.push_str(&format!("Composite Score: {:.4}\n", self.composite_score));
        s.push_str("Dimension Scores:\n");
        let mut dims: Vec<_> = self.dimension_scores.iter().collect();
        dims.sort_by(|a, b| a.0.cmp(b.0));
        for (dim, score) in &dims {
            s.push_str(&format!("  {:?}: {:.4}\n", dim, score));
        }
        s.push_str(&format!("\nTotal Cost: ${:.4}\n", self.total_cost_usd));
        s.push_str(&format!("Safety Violations: {}\n", self.total_safety_violations));
        s.push_str(&format!(
            "Avg Trajectory Efficiency: {:.4}\n",
            self.avg_trajectory_efficiency
        ));
        if !self.contamination_flags.is_empty() {
            s.push_str(&format!(
                "Contamination Flags: {}\n",
                self.contamination_flags.join(", ")
            ));
        }
        s
    }

    pub fn to_json(&self) -> serde_json::Value {
        let dim_scores: serde_json::Map<String, serde_json::Value> = self
            .dimension_scores
            .iter()
            .map(|(k, v)| {
                (
                    format!("{:?}", k),
                    serde_json::Number::from_f64(*v)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null),
                )
            })
            .collect();

        let by_type: serde_json::Map<String, serde_json::Value> = self
            .by_task_type
            .iter()
            .map(|(tt, dims)| {
                let dims_val: serde_json::Map<String, serde_json::Value> = dims
                    .iter()
                    .map(|(d, v)| {
                        (
                            format!("{:?}", d),
                            serde_json::Number::from_f64(*v)
                                .map(serde_json::Value::Number)
                                .unwrap_or(serde_json::Value::Null),
                        )
                    })
                    .collect();
                (format!("{:?}", tt), serde_json::Value::Object(dims_val))
            })
            .collect();

        serde_json::json!({
            "suite_name": self.suite_name,
            "config_name": self.config.name,
            "composite_score": self.composite_score,
            "dimension_scores": dim_scores,
            "by_task_type": by_type,
            "total_cost_usd": self.total_cost_usd,
            "total_safety_violations": self.total_safety_violations,
            "avg_trajectory_efficiency": self.avg_trajectory_efficiency,
            "contamination_flags": self.contamination_flags,
            "num_tasks_evaluated": self.num_tasks_evaluated,
            "timestamp": self.timestamp,
        })
    }
}

// Additional impl for BenchmarkSuite with multi-dim evaluation methods
impl BenchmarkSuite {
    pub fn run_multi_dim_eval(
        &self,
        config: &EvalRunConfig,
        task_scores: HashMap<String, Vec<MultiDimResult>>,
    ) -> EvalReport {
        let all_dims = EvalDimension::all();
        let mut dim_totals: HashMap<EvalDimension, f64> = HashMap::new();
        let mut dim_count: HashMap<EvalDimension, usize> = HashMap::new();
        let mut type_dim_totals: HashMap<TaskType, HashMap<EvalDimension, f64>> = HashMap::new();
        let mut type_dim_count: HashMap<TaskType, HashMap<EvalDimension, usize>> = HashMap::new();
        let mut total_cost = 0.0;
        let mut total_violations = 0;
        let mut trajectory_efficiencies: Vec<f64> = Vec::new();
        let mut contamination_flags = Vec::new();
        let mut task_count = 0;

        let known_public: Vec<String> = vec![
            "mmlu".into(),
            "gsm8k".into(),
            "human_eval".into(),
            "swe_bench".into(),
            "webarena".into(),
            "gaia".into(),
            "tau_bench".into(),
        ];

        for (task_id, results) in &task_scores {
            if !config.task_ids.is_empty() && !config.task_ids.contains(task_id) {
                continue;
            }
            task_count += 1;

            if BenchmarkSuite::contamination_check(task_id, &known_public) {
                contamination_flags.push(task_id.clone());
            }

            for result in results {
                for &dim in &all_dims {
                    if let Some(&score) = result.scores.get(&dim) {
                        *dim_totals.entry(dim).or_insert(0.0) += score;
                        *dim_count.entry(dim).or_insert(0) += 1;
                    }
                }
                total_cost += result.cost_usd;
                total_violations += result.safety_violations.len();

                if let Some(optimal) = result.trajectory_optimal_steps {
                    if result.trajectory_steps > 0 {
                        let eff = (optimal as f64 / result.trajectory_steps as f64).min(1.0);
                        trajectory_efficiencies.push(eff);
                    }
                }

                if let Some(task) = self.tasks.iter().find(|t| t.id == *task_id) {
                    let type_entry = type_dim_totals.entry(task.task_type).or_insert_with(HashMap::new);
                    let type_count_entry = type_dim_count.entry(task.task_type).or_insert_with(HashMap::new);
                    for &dim in &all_dims {
                        if let Some(&score) = result.scores.get(&dim) {
                            *type_entry.entry(dim).or_insert(0.0) += score;
                            *type_count_entry.entry(dim).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        let dimension_scores: HashMap<EvalDimension, f64> = all_dims
            .iter()
            .map(|&dim| {
                let count = *dim_count.get(&dim).unwrap_or(&0);
                let total = *dim_totals.get(&dim).unwrap_or(&0.0);
                (dim, if count > 0 { total / count as f64 } else { 0.0 })
            })
            .collect();

        let by_task_type: HashMap<TaskType, HashMap<EvalDimension, f64>> = type_dim_totals
            .keys()
            .map(|&tt| {
                let scores = all_dims
                    .iter()
                    .map(|&dim| {
                        let total = *type_dim_totals
                            .get(&tt)
                            .and_then(|m| m.get(&dim))
                            .unwrap_or(&0.0);
                        let count = *type_dim_count
                            .get(&tt)
                            .and_then(|m| m.get(&dim))
                            .unwrap_or(&0);
                        (dim, if count > 0 { total / count as f64 } else { 0.0 })
                    })
                    .collect();
                (tt, scores)
            })
            .collect();

        let weights = DimensionWeights::default();
        let composite = multi_dim_pass_rate_from_map(&dimension_scores, &weights);
        let avg_traj = if trajectory_efficiencies.is_empty() {
            0.0
        } else {
            trajectory_efficiencies.iter().sum::<f64>() / trajectory_efficiencies.len() as f64
        };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        EvalReport {
            suite_name: self.name.clone(),
            config: config.clone(),
            dimension_scores,
            composite_score: composite,
            by_task_type,
            total_cost_usd: total_cost,
            total_safety_violations: total_violations,
            avg_trajectory_efficiency: avg_traj,
            contamination_flags,
            num_tasks_evaluated: task_count,
            timestamp,
        }
    }

    pub fn multi_dim_pass_rate(results: &[MultiDimResult], weights: &DimensionWeights) -> f64 {
        if results.is_empty() {
            return 0.0;
        }
        let norm = weights.normalized();
        let mut total = 0.0;
        for result in results {
            let mut result_score = 0.0;
            for dim in EvalDimension::all() {
                if let Some(&score) = result.scores.get(&dim) {
                    result_score += score * norm.get(dim);
                }
            }
            total += result_score;
        }
        total / results.len() as f64
    }

    pub fn cost_per_task(results: &[MultiDimResult]) -> f64 {
        if results.is_empty() {
            return 0.0;
        }
        results.iter().map(|r| r.cost_usd).sum::<f64>() / results.len() as f64
    }

    pub fn safety_score(results: &[MultiDimResult]) -> f64 {
        let max_expected = results.len() as f64;
        if max_expected == 0.0 {
            return 1.0;
        }
        let violations: usize = results.iter().map(|r| r.safety_violations.len()).sum();
        let score = 1.0 - (violations as f64 / max_expected);
        score.max(0.0).min(1.0)
    }

    pub fn trajectory_efficiency(results: &[MultiDimResult]) -> f64 {
        let mut total_ratio = 0.0;
        let mut count = 0;
        for r in results {
            if let Some(optimal) = r.trajectory_optimal_steps {
                if r.trajectory_steps > 0 {
                    total_ratio += (optimal as f64 / r.trajectory_steps as f64).min(1.0);
                    count += 1;
                }
            }
        }
        if count == 0 {
            1.0
        } else {
            total_ratio / count as f64
        }
    }

    pub fn contamination_check(task_id: &str, known_public_tasks: &[String]) -> bool {
        ContaminationDetector::exact_match(task_id, known_public_tasks)
            || ContaminationDetector::fuzzy_match(task_id, known_public_tasks)
    }
}

/// Multi-method contamination detector for agent benchmarks
#[derive(Debug, Clone)]
pub struct ContaminationDetector;

impl ContaminationDetector {
    /// Exact substring match (case-insensitive)
    pub fn exact_match(task_id: &str, pool: &[String]) -> bool {
        let lower = task_id.to_lowercase();
        pool.iter().any(|k| lower.contains(&k.to_lowercase()) || k.to_lowercase().contains(&lower))
    }

    /// Fuzzy n-gram overlap detection
    pub fn fuzzy_match(task_id: &str, pool: &[String]) -> bool {
        let grams = Self::ngrams(task_id, 3);
        if grams.is_empty() {
            return false;
        }
        pool.iter().any(|known| {
            let known_grams = Self::ngrams(known, 3);
            let intersection: usize = grams.iter().filter(|g| known_grams.contains(g)).count();
            let union: usize = grams.len() + known_grams.len() - intersection;
            union > 0 && intersection as f64 / union as f64 > 0.3
        })
    }

    /// Generate character n-grams
    pub fn ngrams(s: &str, n: usize) -> Vec<String> {
        let s = s.to_lowercase();
        if s.len() < n {
            return vec![s];
        }
        s.as_bytes().windows(n).map(|w| String::from_utf8_lossy(w).to_string()).collect()
    }

    /// Check for benchmark-specific reward hacking patterns
    pub fn detect_reward_hacking(result: &BenchmarkResult, task: &BenchmarkTask) -> Vec<String> {
        let mut flags = vec![];
        if result.score > 0.95 && task.expected_difficulty > 0.8 {
            flags.push("Suspicious: near-perfect score on hard task".into());
        }
        if result.latency_ms < 100 && task.expected_difficulty > 0.5 {
            flags.push("Suspicious: implausibly fast for task difficulty".into());
        }
        if let Some(ref err) = result.error {
            if !err.is_empty() && result.score > 0.8 {
                flags.push("Suspicious: error present but high score".into());
            }
        }
        flags
    }
}

/// Dynamic benchmark — generates fresh tasks at runtime to avoid contamination
#[derive(Debug, Clone)]
pub struct DynamicBenchmark {
    pub seed_templates: Vec<String>,
    pub parameter_ranges: HashMap<String, (f64, f64)>,
}

impl DynamicBenchmark {
    pub fn new() -> Self {
        let mut param_ranges = HashMap::new();
        param_ranges.insert("difficulty".into(), (0.1, 0.9));
        param_ranges.insert("steps".into(), (1.0, 10.0));
        Self {
            seed_templates: vec![
                "Generate a function that {verb} a {noun} using {technique}".into(),
                "Explain why {concept_a} differs from {concept_b} in the context of {domain}".into(),
                "Write a test case for {scenario} with edge cases including {edge}".into(),
                "Refactor the following code to improve {quality}: {code_snippet}".into(),
            ],
            parameter_ranges: param_ranges,
        }
    }

    /// Generate a fresh benchmark task from template
    pub fn generate_task(&self, id: &str, template_idx: usize) -> BenchmarkTask {
        let description = self.seed_templates.get(template_idx).cloned().unwrap_or_default();
        BenchmarkTask::new(id, &description, TaskType::CodeGeneration, 0.5)
    }

    /// Generate N tasks with guaranteed fresh content
    pub fn generate_suite(&self, name: &str, count: usize) -> BenchmarkSuite {
        let mut suite = BenchmarkSuite::new(name, "dynamic");
        for i in 0..count {
            let tid = format!("{}-{:04x}", name, i);
            let task = self.generate_task(&tid, i % self.seed_templates.len());
            suite.add(task);
        }
        suite
    }
}

fn multi_dim_pass_rate_from_map(
    scores: &HashMap<EvalDimension, f64>,
    weights: &DimensionWeights,
) -> f64 {
    let norm = weights.normalized();
    let mut total = 0.0;
    for dim in EvalDimension::all() {
        if let Some(&score) = scores.get(&dim) {
            total += score * norm.get(dim);
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_suite_pass_rate() {
        let mut suite = BenchmarkSuite::new("test", "1.0");
        suite.add(BenchmarkTask::new("t1", "task1", TaskType::Reasoning, 0.5));
        suite.add(BenchmarkTask::new("t2", "task2", TaskType::CodeGeneration, 0.5));
        suite.add(BenchmarkTask::new("t3", "task3", TaskType::KnowledgeQA, 0.5));

        let results = vec![
            BenchmarkResult::new("t1", 0.8),
            BenchmarkResult::new("t2", 0.6),
            BenchmarkResult::new("t3", 1.0),
        ];
        let pr = suite.pass_rate(&results);
        assert!((pr - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_mutation_scope_selection() {
        assert_eq!(MutationScope::from_pass_rate(0.0), MutationScope::Comprehensive);
        assert_eq!(MutationScope::from_pass_rate(0.29), MutationScope::Comprehensive);
        assert_eq!(MutationScope::from_pass_rate(0.3), MutationScope::Targeted);
        assert_eq!(MutationScope::from_pass_rate(0.69), MutationScope::Targeted);
        assert_eq!(MutationScope::from_pass_rate(0.7), MutationScope::Minimal);
        assert_eq!(MutationScope::from_pass_rate(1.0), MutationScope::Minimal);
    }

    #[test]
    fn test_egl_tracker_states() {
        let mut tracker = EglTracker::new();
        let s1 = tracker.track(0.5);
        assert!(!s1.is_regressing());
        let s2 = tracker.track(0.55);
        assert!(!s2.is_regressing());
        let s3 = tracker.track(0.52);
        assert!(matches!(s3, EglStatus::Stable { .. }));
        let s4 = tracker.track(0.3);
        assert!(s4.is_regressing());
        let s5 = tracker.track(0.9);
        assert!(matches!(s5, EglStatus::Improving { .. }));
    }

    #[test]
    fn test_benchmark_gate_check() {
        let gate = BenchmarkGate::default();
        assert!(gate.check(0.7, 0.8).is_ok());
        assert!(gate.check(0.7, 0.6).is_err());
        assert!(gate.check(0.8, 0.76).is_ok());
        assert!(gate.check(0.8, 0.74).is_err());
    }

    #[test]
    fn test_evolution_cycle_result() {
        let config = EvolutionConfig {
            benchmark_suite: BenchmarkSuite::new("test", "1.0"),
            gate: BenchmarkGate::default(),
            egl_window: 5,
            check_interval_cycles: 1,
            max_cycles: 10,
        };
        let mut loop_ = EvolutionLoop::new(config);
        let scores = vec![("a".into(), 0.8), ("b".into(), 0.6), ("c".into(), 0.7)];
        let result = loop_.run_cycle(&scores);
        assert_eq!(result.iteration, 1);
        assert!((result.pass_rate - 0.7).abs() < 1e-6);
        assert_eq!(result.scope, MutationScope::Targeted);
        assert!(result.accepted);
    }

    #[test]
    fn test_trait_store_top_k() {
        let mut store = TraitStore::new();
        store.store("a", 0.3);
        store.store("b", 0.9);
        store.store("c", 0.6);
        let top = store.get_top(2);
        assert_eq!(top.len(), 2);
        assert!((top[0].1 - 0.9).abs() < 1e-6);
        assert!((top[1].1 - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_trait_store_max_traits() {
        let mut store = TraitStore { traits: Vec::new(), max_traits: 3 };
        store.store("a", 0.1);
        store.store("b", 0.9);
        store.store("c", 0.5);
        store.store("d", 0.7);
        assert_eq!(store.count(), 3);
        assert!((store.best_score() - 0.9).abs() < 1e-6);
    }

    // --- Multi-dimensional evaluation tests ---

    #[test]
    fn test_multi_dim_result_creation() {
        let r = MultiDimResult::new("task_1")
            .with_score(EvalDimension::Correctness, 0.9)
            .with_score(EvalDimension::Safety, 1.0)
            .with_cost(0.05)
            .with_trajectory(5, Some(3));
        assert_eq!(r.task_id, "task_1");
        assert!((r.score(EvalDimension::Correctness) - 0.9).abs() < 1e-6);
        assert!((r.cost_usd - 0.05).abs() < 1e-6);
        assert_eq!(r.trajectory_steps, 5);
        assert_eq!(r.trajectory_optimal_steps, Some(3));
    }

    #[test]
    fn test_dimension_weights_defaults() {
        let w = DimensionWeights::default();
        assert!((w.correctness - 0.4).abs() < 1e-6);
        assert!((w.safety - 0.25).abs() < 1e-6);
        assert!((w.cost - 0.15).abs() < 1e-6);
        assert!((w.latency - 0.1).abs() < 1e-6);
        assert!((w.trajectory - 0.05).abs() < 1e-6);
        assert!((w.robustness - 0.05).abs() < 1e-6);
        assert!((w.total() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_dimension_weights_normalized() {
        let w = DimensionWeights::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
        let n = w.normalized();
        assert!((n.correctness - 1.0 / 6.0).abs() < 1e-6);
        assert!((n.total() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_multi_dim_pass_rate_basic() {
        let results = vec![
            MultiDimResult::new("t1")
                .with_score(EvalDimension::Correctness, 1.0)
                .with_score(EvalDimension::Safety, 1.0)
                .with_score(EvalDimension::Cost, 1.0)
                .with_score(EvalDimension::Latency, 1.0)
                .with_score(EvalDimension::Trajectory, 1.0)
                .with_score(EvalDimension::Robustness, 1.0),
            MultiDimResult::new("t2")
                .with_score(EvalDimension::Correctness, 0.0)
                .with_score(EvalDimension::Safety, 0.0)
                .with_score(EvalDimension::Cost, 0.0)
                .with_score(EvalDimension::Latency, 0.0)
                .with_score(EvalDimension::Trajectory, 0.0)
                .with_score(EvalDimension::Robustness, 0.0),
        ];
        let weights = DimensionWeights::default();
        let rate = BenchmarkSuite::multi_dim_pass_rate(&results, &weights);
        // With equal weights total=1.0, one 1.0 and one 0.0 => avg 0.5
        assert!((rate - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_cost_per_task_average() {
        let results = vec![
            MultiDimResult::new("t1").with_cost(0.10),
            MultiDimResult::new("t2").with_cost(0.20),
            MultiDimResult::new("t3").with_cost(0.30),
        ];
        let avg = BenchmarkSuite::cost_per_task(&results);
        assert!((avg - 0.20).abs() < 1e-6);
    }

    #[test]
    fn test_safety_score_no_violations() {
        let results = vec![
            MultiDimResult::new("t1"),
            MultiDimResult::new("t2"),
        ];
        let score = BenchmarkSuite::safety_score(&results);
        assert!((score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_safety_score_with_violations() {
        let results = vec![
            MultiDimResult::new("t1").with_safety_violation("pii_leak"),
            MultiDimResult::new("t2").with_safety_violation("pii_leak")
                .with_safety_violation("prompt_injection"),
        ];
        let score = BenchmarkSuite::safety_score(&results);
        // 3 violations / 2 tasks => 1.0 - 1.5 = -0.5, clamped to 0.0
        assert!((score - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_trajectory_efficiency_perfect() {
        let results = vec![
            MultiDimResult::new("t1").with_trajectory(5, Some(5)),
            MultiDimResult::new("t2").with_trajectory(3, Some(3)),
        ];
        let eff = BenchmarkSuite::trajectory_efficiency(&results);
        assert!((eff - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_trajectory_efficiency_wasteful() {
        let results = vec![
            MultiDimResult::new("t1").with_trajectory(10, Some(5)),
            MultiDimResult::new("t2").with_trajectory(8, Some(4)),
        ];
        let eff = BenchmarkSuite::trajectory_efficiency(&results);
        assert!((eff - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_contamination_check_public_task() {
        let known: Vec<String> = vec!["mmlu".into(), "gsm8k".into(), "human_eval".into()];
        assert!(BenchmarkSuite::contamination_check("mmlu_physics", &known));
        assert!(BenchmarkSuite::contamination_check("custom_gsm8k_v2", &known));
    }

    #[test]
    fn test_contamination_check_private_task() {
        let known: Vec<String> = vec!["mmlu".into(), "gsm8k".into()];
        assert!(!BenchmarkSuite::contamination_check("my_private_bench_42", &known));
        assert!(!BenchmarkSuite::contamination_check("internal_security_test", &known));
    }

    #[test]
    fn test_eval_report_summary() {
        let config = EvalRunConfig::new("test-run").with_trials(3);
        let mut dim_scores = HashMap::new();
        dim_scores.insert(EvalDimension::Correctness, 0.85);
        dim_scores.insert(EvalDimension::Safety, 0.95);
        let report = EvalReport {
            suite_name: "SuiteX".into(),
            config,
            dimension_scores: dim_scores,
            composite_score: 0.87,
            by_task_type: HashMap::new(),
            total_cost_usd: 1.23,
            total_safety_violations: 2,
            avg_trajectory_efficiency: 0.75,
            contamination_flags: vec!["mmlu_physics".into()],
            num_tasks_evaluated: 10,
            timestamp: 1234567890,
        };
        let summary = report.summary_string();
        assert!(summary.contains("SuiteX"));
        assert!(summary.contains("0.87"));
        assert!(summary.contains("$1.23"));
        assert!(summary.contains("mmlu_physics"));
        assert!(summary.contains("10 tasks"));
    }

    #[test]
    fn test_eval_report_to_json() {
        let config = EvalRunConfig::new("json-test");
        let mut dim_scores = HashMap::new();
        dim_scores.insert(EvalDimension::Correctness, 0.9);
        let report = EvalReport {
            suite_name: "JSON".into(),
            config,
            dimension_scores: dim_scores,
            composite_score: 0.9,
            by_task_type: HashMap::new(),
            total_cost_usd: 0.5,
            total_safety_violations: 0,
            avg_trajectory_efficiency: 1.0,
            contamination_flags: vec![],
            num_tasks_evaluated: 1,
            timestamp: 999,
        };
        let json = report.to_json();
        assert_eq!(json["suite_name"], "JSON");
        assert!((json["composite_score"].as_f64().unwrap() - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_benchmark_suite_multi_dim_run() {
        let mut suite = BenchmarkSuite::new("eval-suite", "2.0");
        suite.add(BenchmarkTask::new("code_01", "write a function", TaskType::CodeGeneration, 0.7));
        suite.add(BenchmarkTask::new("safe_01", "safety check", TaskType::Safety, 0.8));

        let config = EvalRunConfig::new("full-eval").with_trials(2);
        let mut task_scores: HashMap<String, Vec<MultiDimResult>> = HashMap::new();

        task_scores.insert(
            "code_01".into(),
            vec![
                MultiDimResult::new("code_01")
                    .with_score(EvalDimension::Correctness, 0.9)
                    .with_score(EvalDimension::Safety, 1.0)
                    .with_cost(0.05)
                    .with_trajectory(5, Some(4)),
                MultiDimResult::new("code_01")
                    .with_score(EvalDimension::Correctness, 0.8)
                    .with_score(EvalDimension::Robustness, 0.8),
            ],
        );
        task_scores.insert(
            "safe_01".into(),
            vec![MultiDimResult::new("safe_01")
                .with_score(EvalDimension::Safety, 0.6)
                .with_score(EvalDimension::Correctness, 1.0)
                .with_safety_violation("prompt_injection")],
        );

        let report = suite.run_multi_dim_eval(&config, task_scores);
        assert_eq!(report.suite_name, "eval-suite");
        assert_eq!(report.num_tasks_evaluated, 2);
        assert!(report.total_cost_usd > 0.0);
        assert_eq!(report.total_safety_violations, 1);
        assert!(report.composite_score > 0.0);
        assert!(!report.by_task_type.is_empty());
    }

    // ── P0.1: Contamination Detection ──

    #[test]
    fn test_exact_match_detects_contamination() {
        let pool = vec!["swe_bench_django_ticket_42".into(), "gaia_l1_math_003".into()];
        assert!(ContaminationDetector::exact_match("swe_bench_django_ticket_42", &pool));
        assert!(!ContaminationDetector::exact_match("custom_fresh_task_001", &pool));
    }

    #[test]
    fn test_exact_match_case_insensitive() {
        let pool = vec!["SWE_BENCH_DJANGO".into()];
        assert!(ContaminationDetector::exact_match("swe_bench_django_issue", &pool));
    }

    #[test]
    fn test_fuzzy_match_ngram_overlap() {
        let pool = vec!["solve_ticket_user_login".into()];
        assert!(ContaminationDetector::fuzzy_match("fix_user_login_ticket_solution", &pool));
    }

    #[test]
    fn test_fuzzy_match_no_false_positive() {
        let pool = vec!["python_django_auth".into()];
        assert!(!ContaminationDetector::fuzzy_match("rust_actix_web_routing", &pool));
    }

    #[test]
    fn test_ngrams_generation() {
        let grams = ContaminationDetector::ngrams("hello", 3);
        assert_eq!(grams.len(), 3);
        assert!(grams.contains(&"hel".to_string()));
        assert!(grams.contains(&"llo".to_string()));
    }

    #[test]
    fn test_detect_reward_hacking_perfect_on_hard() {
        let result = BenchmarkResult::new("hard_task", 0.99);
        let task = BenchmarkTask::new("hard_task", "very hard", TaskType::Reasoning, 0.9);
        let flags = ContaminationDetector::detect_reward_hacking(&result, &task);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detect_reward_hacking_fast_on_hard() {
        let mut result = BenchmarkResult::new("fast_task", 0.8);
        result.latency_ms = 50;
        let task = BenchmarkTask::new("fast_task", "moderate", TaskType::CodeGeneration, 0.7);
        let flags = ContaminationDetector::detect_reward_hacking(&result, &task);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_dynamic_benchmark_generates_fresh_suite() {
        let db = DynamicBenchmark::new();
        let suite = db.generate_suite("fresh", 5);
        assert_eq!(suite.task_count(), 5);
        assert_eq!(suite.name, "fresh");
    }

    #[test]
    fn test_dynamic_benchmark_tasks_have_unique_ids() {
        let db = DynamicBenchmark::new();
        let suite = db.generate_suite("eval", 3);
        let ids: Vec<_> = suite.tasks.iter().map(|t| t.id.clone()).collect();
        assert_eq!(ids[0], "eval-0000");
        assert_eq!(ids[2], "eval-0002");
    }
}
