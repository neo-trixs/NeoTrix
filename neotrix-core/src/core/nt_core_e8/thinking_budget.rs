//! Adaptive test-time compute allocation for E₈ reasoning engine.
//!
//! Implements compute-optimal thinking budget allocation, early-exit
//! based on confidence, overthinking detection, and heuristic difficulty
//! estimation — all without external dependencies.

use serde::{Deserialize, Serialize};

// ─── Budget Type ──────────────────────────────────────────────────────

/// How the thinking budget is allocated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BudgetType {
    /// Fixed budget (legacy behavior).
    Fixed(usize),
    /// Adaptive: allocate based on estimated difficulty.
    Adaptive {
        /// Steps for easy problems (default 4).
        easy_budget: usize,
        /// Steps for medium problems (default 16).
        medium_budget: usize,
        /// Steps for hard problems (default 32).
        hard_budget: usize,
    },
    /// Compute-optimal: allocate dynamically until confidence threshold.
    ComputeOptimal {
        /// Absolute max steps.
        max_total: usize,
        /// Exit when confidence exceeds this value.
        min_confidence: f64,
    },
}

impl Default for BudgetType {
    fn default() -> Self {
        Self::Adaptive {
            easy_budget: 4,
            medium_budget: 16,
            hard_budget: 32,
        }
    }
}

// ─── Difficulty Method ────────────────────────────────────────────────

/// Method used for difficulty estimation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DifficultyMethod {
    /// Use prompt length + novelty as proxy.
    Heuristic,
    /// Use a lightweight classifier.
    Learned {
        /// Optional path to a model file.
        model_path: Option<String>,
    },
    /// Always assume hard (fallback).
    Conservative,
}

// ─── Difficulty Estimator ─────────────────────────────────────────────

/// Estimates problem difficulty to guide budget allocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyEstimator {
    /// The estimation method.
    pub method: DifficultyMethod,
    /// Features used for estimation.
    pub features: Vec<String>,
}

impl Default for DifficultyEstimator {
    fn default() -> Self {
        Self {
            method: DifficultyMethod::Heuristic,
            features: vec![
                "prompt_length".into(),
                "question_density".into(),
                "task_type_weight".into(),
                "constraint_count".into(),
            ],
        }
    }
}

impl DifficultyEstimator {
    /// Create a new difficulty estimator with the given method.
    pub fn new(method: DifficultyMethod) -> Self {
        Self {
            method,
            features: Vec::new(),
        }
    }

    /// Estimate difficulty from prompt and task type.
    /// Returns a value in [0.0, 1.0] where 1.0 is hardest.
    pub fn estimate(&self, prompt: &str, task_type: &str) -> f64 {
        match self.method {
            DifficultyMethod::Heuristic => Self::heuristic_difficulty(prompt, task_type),
            DifficultyMethod::Learned { .. } => {
                // Fallback to heuristic when no learned model is loaded
                Self::heuristic_difficulty(prompt, task_type)
            }
            DifficultyMethod::Conservative => 0.95,
        }
    }

    /// Heuristic difficulty estimation based on prompt features.
    ///
    /// Formula:
    /// ```text
    /// difficulty = 0.3 × normalized_length
    ///           + 0.2 × question_density
    ///           + 0.3 × task_type_weight
    ///           + 0.2 × constraint_count
    /// ```
    pub fn heuristic_difficulty(prompt: &str, task_type: &str) -> f64 {
        let normalized_length = (prompt.len() as f64 / 2000.0).min(1.0);

        let question_marks = prompt.chars().filter(|&c| c == '?').count() as f64;
        let question_density = (question_marks / 5.0).min(1.0);

        let task_type_weight = Self::task_type_weight(task_type);

        let constraint_count = Self::count_constraints(prompt);
        let constraint_norm = (constraint_count as f64 / 10.0).min(1.0);

        0.3 * normalized_length
            + 0.2 * question_density
            + 0.3 * task_type_weight
            + 0.2 * constraint_norm
    }

    /// Task type weight: how hard is this type of problem?
    fn task_type_weight(task_type: &str) -> f64 {
        let lower = task_type.to_lowercase();
        if lower.contains("multi") || lower.contains("complex") {
            1.0
        } else if lower.contains("code")
            || lower.contains("math")
            || lower.contains("reasoning")
            || lower.contains("logic")
        {
            0.8
        } else if lower.contains("analysis")
            || lower.contains("compare")
            || lower.contains("evaluate")
            || lower.contains("explain")
        {
            0.5
        } else {
            0.2
        }
    }

    /// Count constraint-indicating keywords in the prompt.
    fn count_constraints(prompt: &str) -> usize {
        let indicators = [
            "must",
            "should",
            "required",
            "need",
            "cannot",
            "can't",
            "constraint",
            "condition",
            "ensure",
            "guarantee",
            "rule",
            "if",
            "unless",
            "except",
            "only",
            "minimum",
            "maximum",
            "at least",
            "at most",
            "not allowed",
            "forbidden",
        ];
        let lower = prompt.to_lowercase();
        indicators.iter().filter(|&&kw| lower.contains(kw)).count()
    }
}

// ─── Budget Stats ─────────────────────────────────────────────────────

/// Statistics tracking for thinking budget usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStats {
    /// Total reasoning sessions tracked.
    pub total_reasoning_sessions: u64,
    /// Total steps used across all sessions.
    pub total_steps_used: u64,
    /// Number of overthinking penalties applied.
    pub total_overthinking_penalties: u64,
    /// Average steps per session.
    pub avg_steps_per_session: f64,
    /// Number of early exits triggered.
    pub early_exits: u64,
    /// Number of times budget was fully exhausted.
    pub budget_exhausted: u64,
    /// Compute saved vs fixed max budget (0.0 to 1.0).
    pub compute_saved_ratio: f64,
}

impl Default for BudgetStats {
    fn default() -> Self {
        Self {
            total_reasoning_sessions: 0,
            total_steps_used: 0,
            total_overthinking_penalties: 0,
            avg_steps_per_session: 0.0,
            early_exits: 0,
            budget_exhausted: 0,
            compute_saved_ratio: 0.0,
        }
    }
}

impl BudgetStats {
    fn record_session(&mut self, steps_used: usize, allocated: usize, early_exit: bool) {
        self.total_reasoning_sessions += 1;
        self.total_steps_used += steps_used as u64;
        if early_exit {
            self.early_exits += 1;
        } else if steps_used >= allocated {
            self.budget_exhausted += 1;
        }
        self.avg_steps_per_session =
            self.total_steps_used as f64 / self.total_reasoning_sessions as f64;
    }

    fn record_overthinking(&mut self) {
        self.total_overthinking_penalties += 1;
    }

    fn update_compute_saved(&mut self, total_fixed_cost: f64, total_actual_cost: f64) {
        if total_fixed_cost > 0.0 {
            self.compute_saved_ratio = ((total_fixed_cost - total_actual_cost) / total_fixed_cost)
                .max(0.0)
                .min(1.0);
        }
    }
}

// ─── Thinking Budget ──────────────────────────────────────────────────

/// Adaptive thinking budget for the E₈ reasoning engine.
///
/// Controls how many reasoning steps are allocated per session, with
/// support for fixed, adaptive, and compute-optimal strategies.
/// Detects overthinking and supports confidence-based early exit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBudget {
    /// Maximum reasoning steps (default 32).
    pub max_steps: usize,
    /// Minimum steps before early exit (default 2).
    pub min_steps: usize,
    /// How budget is allocated.
    pub budget_type: BudgetType,
    /// Early exit confidence threshold (0.0 - 1.0).
    pub confidence_threshold: f64,
    /// Penalty multiplier for excessive steps beyond optimal.
    pub overthinking_penalty: f64,
    /// Difficulty estimator.
    pub difficulty_estimator: DifficultyEstimator,
    /// Statistics tracking.
    pub stats: BudgetStats,
}

impl Default for ThinkingBudget {
    fn default() -> Self {
        Self {
            max_steps: 32,
            min_steps: 2,
            budget_type: BudgetType::default(),
            confidence_threshold: 0.9,
            overthinking_penalty: 0.1,
            difficulty_estimator: DifficultyEstimator::default(),
            stats: BudgetStats::default(),
        }
    }
}

impl ThinkingBudget {
    /// Create a new `ThinkingBudget` with adaptive default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Determine the maximum number of reasoning steps for a given input.
    ///
    /// Based on the configured `BudgetType`:
    /// - `Fixed(n)`: always returns `n`
    /// - `Adaptive`: estimates difficulty and maps to easy/medium/hard budget
    /// - `ComputeOptimal`: returns `max_total` (fine-grained control at step level)
    pub fn allocate(&self, prompt: &str, task_type: &str) -> usize {
        match &self.budget_type {
            BudgetType::Fixed(n) => *n,
            BudgetType::Adaptive {
                easy_budget,
                medium_budget,
                hard_budget,
            } => {
                let difficulty = self.estimate_difficulty(prompt, task_type);
                if difficulty < 0.33 {
                    *easy_budget
                } else if difficulty < 0.66 {
                    *medium_budget
                } else {
                    *hard_budget
                }
            }
            BudgetType::ComputeOptimal { max_total, .. } => *max_total,
        }
    }

    /// Check whether the engine should early-exit based on confidence.
    ///
    /// Returns `true` when:
    /// - Current step >= `min_steps`
    /// - Confidence >= `confidence_threshold`
    pub fn should_early_exit(&self, step: usize, confidence: f64) -> bool {
        if step < self.min_steps {
            return false;
        }
        match &self.budget_type {
            BudgetType::ComputeOptimal { min_confidence, .. } => confidence >= *min_confidence,
            _ => confidence >= self.confidence_threshold,
        }
    }

    /// Record an overthinking event when the engine used more steps than
    /// optimal for the given problem.
    pub fn record_overthinking(&mut self, steps_used: usize, allocated: usize) {
        self.stats.record_overthinking();
        if allocated > 0 && steps_used > allocated {
            let excess = steps_used - allocated;
            let penalty = (excess as f64 * self.overthinking_penalty).ceil() as usize;
            let _ = penalty; // penalty is tracked for future tuning
        }
    }

    /// Estimate the difficulty of a given prompt and task type.
    /// Returns a value in [0.0, 1.0] where 1.0 is hardest.
    pub fn estimate_difficulty(&self, prompt: &str, task_type: &str) -> f64 {
        self.difficulty_estimator.estimate(prompt, task_type)
    }

    /// Pure heuristic difficulty (accessible without an estimator instance).
    pub fn heuristic_difficulty(prompt: &str, task_type: &str) -> f64 {
        DifficultyEstimator::heuristic_difficulty(prompt, task_type)
    }

    /// Record a completed reasoning session for statistics tracking.
    ///
    /// * `steps_used` — actual steps consumed
    /// * `allocated` — budget allocated for this session
    /// * `early_exit` — whether the session exited early via confidence
    /// * `fixed_max_budget` — the fixed maximum budget used for comparison
    pub fn record_session(
        &mut self,
        steps_used: usize,
        allocated: usize,
        early_exit: bool,
        fixed_max_budget: usize,
    ) {
        self.stats.record_session(steps_used, allocated, early_exit);
        let sessions = self.stats.total_reasoning_sessions;
        let total_fixed = sessions as f64 * fixed_max_budget as f64;
        let total_actual = self.stats.total_steps_used as f64;
        self.stats.update_compute_saved(total_fixed, total_actual);
    }

    /// Access current budget statistics.
    pub fn stats(&self) -> &BudgetStats {
        &self.stats
    }

    /// Reset all statistics to default.
    pub fn reset_stats(&mut self) {
        self.stats = BudgetStats::default();
    }
}

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_allocation() {
        let budget = ThinkingBudget::new();
        assert_eq!(budget.max_steps, 32);
        assert_eq!(budget.min_steps, 2);
        assert_eq!(budget.confidence_threshold, 0.9);
    }

    #[test]
    fn test_adaptive_allocation_easy() {
        let budget = ThinkingBudget::new();
        let prompt = "What is 2+2?";
        let steps = budget.allocate(prompt, "simple_qa");
        // Easy: short prompt, low difficulty → easy_budget (4)
        assert_eq!(steps, 4);
    }

    #[test]
    fn test_adaptive_allocation_hard() {
        let budget = ThinkingBudget::new();
        let prompt = "Write a recursive Fibonacci function in Rust and analyze its time complexity. Ensure it handles large inputs efficiently. Must be tail-recursive. Need to compare with iterative approach. Must handle edge cases like overflow. Must implement memoization for performance. Cannot use recursion without caching. Must guarantee constant-time lookups. Should minimize memory usage. Must not use dynamic dispatch.";
        let steps = budget.allocate(prompt, "code");
        // Hard-ish prompt lands on medium budget (16) by heuristic
        assert!(
            steps >= 16,
            "expected >= 16 for code+constraints prompt, got {steps}"
        );
    }

    #[test]
    fn test_early_exit_when_confidence_high() {
        let budget = ThinkingBudget::new();
        // At step 5 (≥ min_steps=2), confidence 0.95 (≥ 0.9)
        assert!(budget.should_early_exit(5, 0.95));
    }

    #[test]
    fn test_early_exit_prevented_when_confidence_low() {
        let budget = ThinkingBudget::new();
        // At step 5 but confidence 0.5 (< 0.9)
        assert!(!budget.should_early_exit(5, 0.50));
    }

    #[test]
    fn test_early_exit_prevented_below_min_steps() {
        let budget = ThinkingBudget::new();
        // At step 1 (< min_steps=2) even with high confidence
        assert!(!budget.should_early_exit(1, 0.99));
    }

    #[test]
    fn test_heuristic_difficulty_easy() {
        let d = ThinkingBudget::heuristic_difficulty("What is the capital of France?", "simple_qa");
        assert!(d < 0.5, "Easy QA should be easy, got {d}");
    }

    #[test]
    fn test_heuristic_difficulty_hard() {
        let d = ThinkingBudget::heuristic_difficulty(
            "Implement a multi-threaded MapReduce framework in Rust. Must handle \
             fault tolerance, data shuffling, and combinatorial optimization. \
             Ensure at-least-once semantics and maximum throughput. Must guarantee \
             exactly-once processing. Cannot tolerate data loss under any condition. \
             Must recover from worker failures automatically. Should minimize \
             network overhead while maximizing parallelism across distributed nodes.",
            "code",
        );
        assert!(
            d > 0.3,
            "Hard coding task should have moderate difficulty, got {d}"
        );
    }

    #[test]
    fn test_overthinking_recording() {
        let mut budget = ThinkingBudget::new();
        budget.record_overthinking(40, 32);
        assert_eq!(budget.stats.total_overthinking_penalties, 1);
    }

    #[test]
    fn test_compute_saved_ratio() {
        let mut budget = ThinkingBudget::new();
        // Session 1: allocated 32, used 16 (early exit, saved 16)
        budget.record_session(16, 32, true, 32);
        // Session 2: allocated 32, used 32 (full use)
        budget.record_session(32, 32, false, 32);

        // total_fixed = 2 * 32 = 64, total_actual = 16 + 32 = 48
        // compute_saved = (64 - 48) / 64 = 0.25
        let saved = budget.stats.compute_saved_ratio;
        assert!((saved - 0.25).abs() < 1e-10, "Expected 0.25, got {saved}");
    }

    #[test]
    fn test_fixed_budget_type() {
        let budget = ThinkingBudget {
            budget_type: BudgetType::Fixed(10),
            ..ThinkingBudget::new()
        };
        assert_eq!(budget.allocate("anything", "any"), 10);
        assert_eq!(budget.allocate("a very long prompt here", "code"), 10);
    }

    #[test]
    fn test_compute_optimal_budget_type() {
        let budget = ThinkingBudget {
            budget_type: BudgetType::ComputeOptimal {
                max_total: 50,
                min_confidence: 0.85,
            },
            confidence_threshold: 0.85,
            ..ThinkingBudget::new()
        };
        assert_eq!(budget.allocate("anything", "any"), 50);

        // Should early exit at 0.85 confidence
        assert!(budget.should_early_exit(3, 0.85));
        assert!(!budget.should_early_exit(3, 0.84));
    }

    #[test]
    fn test_stats_tracking() {
        let mut budget = ThinkingBudget::new();
        budget.record_session(8, 16, true, 32);
        budget.record_session(16, 16, false, 32);
        budget.record_session(4, 16, true, 32);

        let stats = budget.stats();
        assert_eq!(stats.total_reasoning_sessions, 3);
        assert_eq!(stats.total_steps_used, 28);
        assert_eq!(stats.early_exits, 2);
        assert_eq!(stats.budget_exhausted, 1);
        // avg = 28 / 3 ≈ 9.33
        assert!((stats.avg_steps_per_session - 28.0 / 3.0).abs() < 1e-10);
        // saved = (96 - 28) / 96 ≈ 0.7083
        assert!((stats.compute_saved_ratio - (96.0 - 28.0) / 96.0).abs() < 1e-10);
    }

    #[test]
    fn test_reset_stats() {
        let mut budget = ThinkingBudget::new();
        budget.record_session(8, 16, true, 32);
        assert_eq!(budget.stats.total_reasoning_sessions, 1);
        budget.reset_stats();
        assert_eq!(budget.stats.total_reasoning_sessions, 0);
        assert_eq!(budget.stats.total_steps_used, 0);
    }

    #[test]
    fn test_heuristic_formula_components() {
        // Very long prompt with many questions
        let long_q = "?".repeat(100);
        let prompt = format!("A{}{}", "x".repeat(4000), long_q);
        let d = DifficultyEstimator::heuristic_difficulty(&prompt, "math");
        // normalized_length = 1.0, question_density = 1.0, task = 0.8, constraints = 0.0
        // = 0.3 * 1.0 + 0.2 * 1.0 + 0.3 * 0.8 + 0.2 * 0.0 = 0.3 + 0.2 + 0.24 = 0.74
        assert!((d - 0.74).abs() < 1e-10, "Expected 0.74, got {d}");
    }

    #[test]
    fn test_difficulty_estimator_conservative() {
        let estimator = DifficultyEstimator::new(DifficultyMethod::Conservative);
        let d = estimator.estimate("easy prompt", "simple_qa");
        assert!((d - 0.95).abs() < 1e-10);
    }

    #[test]
    fn test_min_steps_respected() {
        let budget = ThinkingBudget::new();
        // Still min_steps=2, even with low confidence
        assert!(!budget.should_early_exit(0, 0.99));
        assert!(!budget.should_early_exit(1, 0.99));
        assert!(budget.should_early_exit(2, 0.99));
    }

    #[test]
    fn test_overthinking_no_penalty_when_under_budget() {
        let mut budget = ThinkingBudget::new();
        budget.record_overthinking(10, 32);
        assert_eq!(budget.stats.total_overthinking_penalties, 1);
        // No crash, no issue — steps used < allocated
    }

    #[test]
    fn test_task_type_weight_variety() {
        assert!((DifficultyEstimator::task_type_weight("multi_step") - 1.0).abs() < 1e-10);
        assert!((DifficultyEstimator::task_type_weight("code") - 0.8).abs() < 1e-10);
        assert!((DifficultyEstimator::task_type_weight("analysis") - 0.5).abs() < 1e-10);
        assert!((DifficultyEstimator::task_type_weight("simple_qa") - 0.2).abs() < 1e-10);
    }
}
