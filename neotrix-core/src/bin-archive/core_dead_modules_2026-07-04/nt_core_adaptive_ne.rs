use std::collections::{HashMap, VecDeque};

/// Configuration for the adaptive reasoning engine.
#[derive(Debug, Clone)]
pub struct AdaptiveNeConfig {
    /// Probability of exploring a random solver instead of the best (ε).
    pub exploration_rate: f64,
    /// Learning rate for LoH choice weight updates (α).
    pub learning_rate: f64,
    /// Maximum number of recent outcomes to retain.
    pub history_size: usize,
}

impl Default for AdaptiveNeConfig {
    fn default() -> Self {
        Self {
            exploration_rate: 0.1,
            learning_rate: 0.05,
            history_size: 100,
        }
    }
}

/// Available reasoning solver strategies.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReasoningSolver {
    E8StateMachine,
    SymbolicPatternMatch,
    BeamSearch { beam_width: usize },
    MctsSearch { simulations: usize },
    FastDirect,
    KbRetrieval,
    CodeExecution,
    MultiAgent { agents: usize },
    ExternalSolver(String),
}

impl ReasoningSolver {
    pub fn name(&self) -> &str {
        match self {
            ReasoningSolver::E8StateMachine => "E8StateMachine",
            ReasoningSolver::SymbolicPatternMatch => "SymbolicPatternMatch",
            ReasoningSolver::BeamSearch { .. } => "BeamSearch",
            ReasoningSolver::MctsSearch { .. } => "MctsSearch",
            ReasoningSolver::FastDirect => "FastDirect",
            ReasoningSolver::KbRetrieval => "KbRetrieval",
            ReasoningSolver::CodeExecution => "CodeExecution",
            ReasoningSolver::MultiAgent { .. } => "MultiAgent",
            ReasoningSolver::ExternalSolver(_) => "ExternalSolver",
        }
    }
}

/// Learned performance profile for a single solver.
#[derive(Debug, Clone)]
pub struct SolverProfile {
    pub solver: ReasoningSolver,
    pub effectiveness: f64,
    pub avg_latency_ms: f64,
    pub avg_cost_tokens: f64,
    pub best_domains: Vec<String>,
    pub worst_domains: Vec<String>,
    pub uses_count: u64,
}

impl SolverProfile {
    pub fn new(solver: ReasoningSolver) -> Self {
        Self {
            solver,
            effectiveness: 0.5,
            avg_latency_ms: 100.0,
            avg_cost_tokens: 500.0,
            best_domains: Vec::new(),
            worst_domains: Vec::new(),
            uses_count: 0,
        }
    }
}

/// Classification of the current reasoning problem.
#[derive(Debug, Clone)]
pub struct ProblemClass {
    pub domain: String,
    pub complexity: f64,
    pub has_verification: bool,
    pub required_precision: f64,
    pub context_size: usize,
    pub keywords: Vec<String>,
}

/// LoH (Logic of Hypotheses) choice weight system.
///
/// Maintains a set of weights over solvers and provides
/// stochastic selection and gradient-based updates.
#[derive(Debug, Clone)]
pub struct LoHChoiceWeights {
    /// Weights for each solver index, in [0.0, 1.0].
    pub weights: Vec<f64>,
    /// Temperature for softmax (higher = more uniform).
    pub temperature: f64,
}

impl LoHChoiceWeights {
    pub fn new(n_solvers: usize) -> Self {
        Self {
            weights: vec![0.5; n_solvers],
            temperature: 1.0,
        }
    }

    pub fn with_temperature(n_solvers: usize, temperature: f64) -> Self {
        Self {
            weights: vec![0.5; n_solvers],
            temperature,
        }
    }

    /// Select an index via softmax over weights.
    pub fn select(&self) -> usize {
        let max_w = self.weights.iter().cloned().fold(0.0_f64, f64::max);
        let scaled: Vec<f64> = self
            .weights
            .iter()
            .map(|w| ((w - max_w) / self.temperature).exp())
            .collect();
        let sum: f64 = scaled.iter().sum();
        if sum <= 0.0 {
            return 0;
        }
        let mut rng_val = fast_rng_fraction() * sum;
        for (i, p) in scaled.iter().enumerate() {
            rng_val -= p;
            if rng_val <= 0.0 {
                return i;
            }
        }
        self.weights.len() - 1
    }

    /// LoH gradient update: `w[i] += lr * reward * (1 - w[i])`.
    pub fn update(&mut self, selected_idx: usize, reward: f64, learning_rate: f64) {
        if selected_idx >= self.weights.len() {
            return;
        }
        let delta = learning_rate * reward * (1.0 - self.weights[selected_idx]);
        self.weights[selected_idx] = (self.weights[selected_idx] + delta).max(0.0).min(1.0);
    }
}

/// Outcome of a single solver execution.
#[derive(Debug, Clone)]
pub struct SolverOutcome {
    pub domain: String,
    pub solver: ReasoningSolver,
    pub success: bool,
    pub latency_ms: f64,
    pub cost_tokens: f64,
}

/// Adaptive Neuro-Symbolic Reasoning Engine.
///
/// Dynamically selects between solvers based on learned profiles,
/// domain affinity, and LoH-style choice weighting.
pub struct AdaptiveReasoningEngine {
    /// Profiles for each registered solver.
    pub solver_profiles: Vec<SolverProfile>,
    /// Domain → solver index affinity scores.
    pub domain_solver_affinity: HashMap<String, Vec<f64>>,
    /// Recent selection history.
    pub selection_history: VecDeque<SolverOutcome>,
    /// Engine configuration.
    pub config: AdaptiveNeConfig,
    /// LoH choice weights.
    pub choice_weights: LoHChoiceWeights,
}

impl Default for AdaptiveReasoningEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AdaptiveReasoningEngine {
    pub fn new() -> Self {
        let solvers = vec![
            ReasoningSolver::E8StateMachine,
            ReasoningSolver::SymbolicPatternMatch,
            ReasoningSolver::BeamSearch { beam_width: 5 },
            ReasoningSolver::MctsSearch { simulations: 100 },
            ReasoningSolver::FastDirect,
            ReasoningSolver::KbRetrieval,
            ReasoningSolver::CodeExecution,
            ReasoningSolver::MultiAgent { agents: 3 },
        ];

        let n = solvers.len();
        let profiles: Vec<SolverProfile> = solvers
            .into_iter()
            .map(SolverProfile::new)
            .collect();

        Self {
            solver_profiles: profiles,
            domain_solver_affinity: HashMap::new(),
            selection_history: VecDeque::with_capacity(100),
            config: AdaptiveNeConfig::default(),
            choice_weights: LoHChoiceWeights::new(n),
        }
    }

    pub fn with_config(config: AdaptiveNeConfig) -> Self {
        let mut engine = Self::new();
        engine.config = config;
        engine.choice_weights = LoHChoiceWeights::new(engine.solver_profiles.len());
        engine
    }

    /// Classify a task string into a ProblemClass using heuristics.
    pub fn classify_problem(
        task: &str,
        context: &HashMap<String, String>,
    ) -> ProblemClass {
        let task_lower = task.to_lowercase();
        let context_size = task.len();

        let has_verification = task_lower.contains("verify")
            || task_lower.contains("check")
            || task_lower.contains("validate")
            || task_lower.contains("test")
            || task_lower.contains("prove");

        let required_precision = if task_lower.contains("exact")
            || task_lower.contains("precise")
            || task_lower.contains("formal")
            || task_lower.contains("proof")
        {
            0.9
        } else if task_lower.contains("approximate")
            || task_lower.contains("estimate")
            || task_lower.contains("guess")
        {
            0.3
        } else {
            0.6
        };

        let complexity = if context_size > 2000 {
            0.8
        } else if context_size > 500 {
            0.5
        } else {
            0.3
        };

        let domain = context
            .get("domain")
            .cloned()
            .unwrap_or_else(|| classify_domain(&task_lower));

        let keywords = extract_keywords(&task_lower);

        ProblemClass {
            domain,
            complexity,
            has_verification,
            required_precision,
            context_size,
            keywords,
        }
    }

    /// Select the best solver for a given problem (ε-greedy).
    pub fn select_solver(&self, problem: &ProblemClass) -> ReasoningSolver {
        if self.solver_profiles.is_empty() {
            return ReasoningSolver::FastDirect;
        }

        // ε-greedy: explore random solver
        if fast_rng_fraction() < self.config.exploration_rate {
            let idx = (fast_rng_fraction() * self.solver_profiles.len() as f64) as usize;
            return self.solver_profiles[idx].solver.clone();
        }

        // Score each solver: weight * effectiveness * domain affinity
        let affinities = self
            .domain_solver_affinity
            .get(&problem.domain);

        let mut best_idx = 0;
        let mut best_score = f64::NEG_INFINITY;

        for (i, profile) in self.solver_profiles.iter().enumerate() {
            let weight = self
                .choice_weights
                .weights
                .get(i)
                .copied()
                .unwrap_or(0.5);
            let affinity = affinities
                .and_then(|a| a.get(i))
                .copied()
                .unwrap_or(0.5);
            let score = weight * profile.effectiveness * affinity;
            if score > best_score {
                best_score = score;
                best_idx = i;
            }
        }

        self.solver_profiles[best_idx].solver.clone()
    }

    /// Record an outcome and update profiles & LoH weights.
    pub fn record_outcome(
        &mut self,
        problem: &ProblemClass,
        solver: &ReasoningSolver,
        success: bool,
        latency_ms: f64,
        cost_tokens: f64,
    ) {
        let reward = if success { 1.0 } else { 0.0 };

        // Trim history if needed
        while self.selection_history.len() >= self.config.history_size {
            self.selection_history.pop_front();
        }

        // Find solver index
        if let Some(solver_idx) = self
            .solver_profiles
            .iter()
            .position(|p| p.solver == *solver)
        {
            let profile = &mut self.solver_profiles[solver_idx];
            let n = profile.uses_count as f64;

            // Update stats (online exponential moving average)
            profile.effectiveness = ((profile.effectiveness * n) + reward) / (n + 1.0);
            profile.avg_latency_ms = ((profile.avg_latency_ms * n) + latency_ms) / (n + 1.0);
            profile.avg_cost_tokens = ((profile.avg_cost_tokens * n) + cost_tokens) / (n + 1.0);
            profile.uses_count += 1;

            // Update domain affinity
            let domain = problem.domain.clone();
            let entry = self
                .domain_solver_affinity
                .entry(domain)
                .or_insert_with(|| vec![0.5; self.solver_profiles.len()]);
            if solver_idx < entry.len() {
                entry[solver_idx] = ((entry[solver_idx] * n) + reward) / (n + 1.0);
            }

            // LoH update
            self.choice_weights
                .update(solver_idx, reward, self.config.learning_rate);
        }

        self.selection_history.push_back(SolverOutcome {
            domain: problem.domain.clone(),
            solver: solver.clone(),
            success,
            latency_ms,
            cost_tokens,
        });
    }

    /// Get top-K solver suggestions for a problem.
    pub fn suggest_hybrid(
        &self,
        problem: &ProblemClass,
    ) -> Vec<(ReasoningSolver, f64)> {
        let affinities = self
            .domain_solver_affinity
            .get(&problem.domain);

        let mut scores: Vec<(usize, f64)> = self
            .solver_profiles
            .iter()
            .enumerate()
            .map(|(i, profile)| {
                let weight = self
                    .choice_weights
                    .weights
                    .get(i)
                    .copied()
                    .unwrap_or(0.5);
                let affinity = affinities
                    .and_then(|a| a.get(i))
                    .copied()
                    .unwrap_or(0.5);
                let score = weight * profile.effectiveness * affinity;
                (i, score)
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scores
            .into_iter()
            .map(|(i, score)| (self.solver_profiles[i].solver.clone(), score))
            .collect()
    }

    /// Find the best solver for a given domain.
    pub fn best_solver_for_domain(&self, domain: &str) -> Option<ReasoningSolver> {
        let affinities = self.domain_solver_affinity.get(domain)?;
        let best_idx = affinities
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))?
            .0;
        Some(self.solver_profiles[best_idx].solver.clone())
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Simple deterministic "random" fraction in [0.0, 1.0) for reproducibility.
fn fast_rng_fraction() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;
    let seed = nanos.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let x = seed >> 33;
    (x as f64) / (1u64 << 31) as f64
}

/// Classify a domain from task text heuristics.
fn classify_domain(task_lower: &str) -> String {
    if task_lower.contains("math") || task_lower.contains("arithmetic") || task_lower.contains("equation") {
        "mathematics".to_string()
    } else if task_lower.contains("code") || task_lower.contains("program") || task_lower.contains("compile") {
        "programming".to_string()
    } else if task_lower.contains("logic") || task_lower.contains("reasoning") || task_lower.contains("deduce") {
        "logic".to_string()
    } else if task_lower.contains("search") || task_lower.contains("find") || task_lower.contains("retrieve") {
        "retrieval".to_string()
    } else if task_lower.contains("plan") || task_lower.contains("schedule") || task_lower.contains("strategy") {
        "planning".to_string()
    } else {
        "general".to_string()
    }
}

/// Extract keywords from task text.
fn extract_keywords(task_lower: &str) -> Vec<String> {
    let stop_words = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "could",
        "should", "may", "might", "shall", "can", "need", "dare", "ought",
        "used", "to", "of", "in", "for", "on", "with", "at", "by", "from",
        "as", "into", "through", "during", "before", "after", "above", "below",
        "between", "out", "off", "over", "under", "again", "further", "then",
        "once", "here", "there", "when", "where", "why", "how", "all", "each",
        "every", "both", "few", "more", "most", "other", "some", "such", "no",
        "nor", "not", "only", "own", "same", "so", "than", "too", "very",
        "just", "because", "but", "and", "or", "if", "while", "that", "this",
        "these", "those", "it", "its", "which", "who", "whom", "what",
    ];
    task_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3 && !stop_words.contains(w))
        .map(|w| w.to_string())
        .collect()
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = AdaptiveReasoningEngine::new();
        assert_eq!(engine.solver_profiles.len(), 8);
        assert_eq!(engine.choice_weights.weights.len(), 8);
        assert!((engine.config.exploration_rate - 0.1).abs() < 1e-9);
        assert!((engine.config.learning_rate - 0.05).abs() < 1e-9);
        assert_eq!(engine.config.history_size, 100);
    }

    #[test]
    fn test_engine_creation_with_config() {
        let config = AdaptiveNeConfig {
            exploration_rate: 0.2,
            learning_rate: 0.1,
            history_size: 50,
        };
        let engine = AdaptiveReasoningEngine::with_config(config.clone());
        assert!((engine.config.exploration_rate - 0.2).abs() < 1e-9);
        assert!((engine.config.learning_rate - 0.1).abs() < 1e-9);
        assert_eq!(engine.config.history_size, 50);
    }

    #[test]
    fn test_classify_problem_mathematics() {
        let context = HashMap::new();
        let problem = AdaptiveReasoningEngine::classify_problem(
            "solve the equation 2x + 3 = 7",
            &context,
        );
        assert_eq!(problem.domain, "mathematics");
        assert!((problem.complexity - 0.3).abs() < 1e-9);
    }

    #[test]
    fn test_classify_problem_with_verification() {
        let context = HashMap::new();
        let problem = AdaptiveReasoningEngine::classify_problem(
            "verify that the algorithm terminates",
            &context,
        );
        assert!(problem.has_verification);
        assert_eq!(problem.domain, "general");
    }

    #[test]
    fn test_classify_problem_with_context_domain() {
        let mut context = HashMap::new();
        context.insert("domain".to_string(), "programming".to_string());
        let problem = AdaptiveReasoningEngine::classify_problem(
            "implement a sorting algorithm",
            &context,
        );
        assert_eq!(problem.domain, "programming");
    }

    #[test]
    fn test_select_solver_returns_solver() {
        let engine = AdaptiveReasoningEngine::new();
        let context = HashMap::new();
        let problem = AdaptiveReasoningEngine::classify_problem("test task", &context);
        let solver = engine.select_solver(&problem);
        // Should always return one of the known solvers
        let names: Vec<&str> = engine
            .solver_profiles
            .iter()
            .map(|p| p.solver.name())
            .collect();
        assert!(names.contains(&solver.name()));
    }

    #[test]
    fn test_record_outcome_updates_profile() {
        let mut engine = AdaptiveReasoningEngine::new();
        let context = HashMap::new();
        let problem = AdaptiveReasoningEngine::classify_problem("math problem", &context);

        let solver = ReasoningSolver::E8StateMachine;
        engine.record_outcome(&problem, &solver, true, 50.0, 200.0);

        let profile = engine
            .solver_profiles
            .iter()
            .find(|p| p.solver == solver)
            .expect("solver should exist");
        assert!((profile.effectiveness - 1.0).abs() < 1e-9);
        assert!((profile.avg_latency_ms - 50.0).abs() < 1e-9);
        assert!((profile.avg_cost_tokens - 200.0).abs() < 1e-9);
        assert_eq!(profile.uses_count, 1);
    }

    #[test]
    fn test_record_outcome_multiple_updates() {
        let mut engine = AdaptiveReasoningEngine::new();
        let context = HashMap::new();
        let problem = AdaptiveReasoningEngine::classify_problem("math problem", &context);

        let solver = ReasoningSolver::E8StateMachine;
        engine.record_outcome(&problem, &solver, true, 50.0, 200.0);
        engine.record_outcome(&problem, &solver, false, 100.0, 500.0);

        let profile = engine
            .solver_profiles
            .iter()
            .find(|p| p.solver == solver)
            .expect("solver should exist");
        assert!((profile.effectiveness - 0.5).abs() < 1e-9);
        assert!((profile.avg_latency_ms - 75.0).abs() < 1e-9);
        assert_eq!(profile.uses_count, 2);
    }

    #[test]
    fn test_record_outcome_updates_domain_affinity() {
        let mut engine = AdaptiveReasoningEngine::new();
        let context = HashMap::new();
        let problem = AdaptiveReasoningEngine::classify_problem("math problem", &context);

        engine.record_outcome(&problem, &ReasoningSolver::E8StateMachine, true, 50.0, 200.0);

        let affinities = engine.domain_solver_affinity.get("mathematics");
        assert!(affinities.is_some());
        let affinity = affinities.unwrap();
        let e8_idx = engine
            .solver_profiles
            .iter()
            .position(|p| p.solver == ReasoningSolver::E8StateMachine)
            .unwrap();
        assert!((affinity[e8_idx] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_loh_choice_weights_update() {
        let mut loh = LoHChoiceWeights::new(5);
        assert_eq!(loh.weights.len(), 5);
        assert!((loh.weights[0] - 0.5).abs() < 1e-9);

        loh.update(0, 1.0, 0.1);
        assert!(loh.weights[0] > 0.5);
        assert!(loh.weights[0] <= 1.0);
    }

    #[test]
    fn test_loh_choice_weights_no_change_on_bad_reward() {
        let mut loh = LoHChoiceWeights::new(3);
        let w0 = loh.weights[0];
        loh.update(0, 0.0, 0.1);
        assert!((loh.weights[0] - w0).abs() < 1e-9);
    }

    #[test]
    fn test_suggest_hybrid_returns_ranked_solvers() {
        let mut engine = AdaptiveReasoningEngine::new();
        let context = HashMap::new();
        let problem = AdaptiveReasoningEngine::classify_problem("code problem", &context);

        // Give E8StateMachine a good outcome in this domain
        engine.record_outcome(&problem, &ReasoningSolver::E8StateMachine, true, 10.0, 50.0);

        let suggestions = engine.suggest_hybrid(&problem);
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions.len(), engine.solver_profiles.len());

        // First suggestion should be E8StateMachine (highest score)
        assert_eq!(suggestions[0].0, ReasoningSolver::E8StateMachine);
        assert!(suggestions[0].1 > 0.0);

        // Scores should be non-increasing
        for i in 1..suggestions.len() {
            assert!(suggestions[i - 1].1 >= suggestions[i].1 - 1e-9);
        }
    }

    #[test]
    fn test_best_solver_for_domain() {
        let mut engine = AdaptiveReasoningEngine::new();
        let context = HashMap::new();
        let problem = AdaptiveReasoningEngine::classify_problem("solve equation", &context);

        // Record successful outcomes for E8StateMachine in mathematics
        engine.record_outcome(&problem, &ReasoningSolver::E8StateMachine, true, 10.0, 50.0);

        let best = engine.best_solver_for_domain("mathematics");
        assert!(best.is_some());
        assert_eq!(best.unwrap(), ReasoningSolver::E8StateMachine);

        // Unknown domain returns None
        let unknown = engine.best_solver_for_domain("unknown_domain");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_loh_select_returns_valid_index() {
        let loh = LoHChoiceWeights::new(8);
        for _ in 0..100 {
            let idx = loh.select();
            assert!(idx < 8);
        }
    }

    #[test]
    fn test_history_size_bounded() {
        let config = AdaptiveNeConfig {
            history_size: 5,
            ..AdaptiveNeConfig::default()
        };
        let mut engine = AdaptiveReasoningEngine::with_config(config);
        let context = HashMap::new();

        for i in 0..10 {
            let problem = AdaptiveReasoningEngine::classify_problem(
                &format!("task {}", i),
                &context,
            );
            engine.record_outcome(&problem, &ReasoningSolver::FastDirect, true, 1.0, 1.0);
        }

        assert!(engine.selection_history.len() <= 5);
    }

    #[test]
    fn test_solver_profile_new() {
        let profile = SolverProfile::new(ReasoningSolver::BeamSearch { beam_width: 10 });
        assert!((profile.effectiveness - 0.5).abs() < 1e-9);
        assert_eq!(profile.uses_count, 0);
        assert!(profile.best_domains.is_empty());
        assert!(profile.worst_domains.is_empty());
    }
}
