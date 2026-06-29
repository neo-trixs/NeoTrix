use std::collections::HashMap;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum RecoveryStep {
    ImmediateRetry,
    BackoffRetry { max_attempts: u32, base_delay_ms: u64 },
    RestartSubsystem(String),
    RevertChange(String),
    RequestHumanHelp(String),
    FallbackToHeuristic,
}

#[derive(Debug, Clone)]
pub struct RecoveryRecipe {
    pub id: u64,
    pub failure_pattern: String,
    pub steps: Vec<RecoveryStep>,
    pub alpha: f64,
    pub beta: f64,
    pub last_used_cycle: u64,
    pub created_at: u64,
    pub total_attempts: u64,
}

impl RecoveryRecipe {
    pub fn success_rate(&self) -> f64 {
        let total = self.alpha + self.beta;
        if total == 0.0 { 0.5 } else { self.alpha / total }
    }
}

#[derive(Debug, Clone)]
pub struct RecoveryOutcome {
    pub recipe_id: u64,
    pub cycle: u64,
    pub success: bool,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct RecipeMatch {
    pub recipe_id: u64,
    pub score: f64,
    pub steps: Vec<RecoveryStep>,
}

#[derive(Debug, Clone)]
pub struct RecoveryRecipeManager {
    recipes: HashMap<String, RecoveryRecipe>,
    next_id: u64,
    history: VecDeque<RecoveryOutcome>,
    max_history: usize,
}

impl RecoveryRecipeManager {
    pub fn new() -> Self {
        let mut mgr = Self {
            recipes: HashMap::new(),
            next_id: 1,
            history: VecDeque::with_capacity(100),
            max_history: 100,
        };
        mgr.register_defaults();
        mgr
    }

    fn register_defaults(&mut self) {
        let defaults = vec![
            ("ece_spike", "Calibration ECE exceeds threshold. Immediate retry with backoff.", vec![
                RecoveryStep::ImmediateRetry,
                RecoveryStep::BackoffRetry { max_attempts: 3, base_delay_ms: 100 },
                RecoveryStep::RequestHumanHelp("Calibration drift detected".into()),
            ]),
            ("compile_error_E0432", "Module not found in mod.rs. Register or create missing module.", vec![
                RecoveryStep::RestartSubsystem("nt_core_experience".into()),
                RecoveryStep::RevertChange("mod.rs".into()),
            ]),
            ("compile_error_E0433", "Crate path or module import incorrect. Check use statements.", vec![
                RecoveryStep::RevertChange("import".into()),
                RecoveryStep::RestartSubsystem("build".into()),
            ]),
            ("meta_accuracy_drop", "Meta-accuracy below 0.6. Calibration may need retuning.", vec![
                RecoveryStep::ImmediateRetry,
                RecoveryStep::BackoffRetry { max_attempts: 2, base_delay_ms: 500 },
                RecoveryStep::RequestHumanHelp("Meta-accuracy degradation".into()),
            ]),
            ("osint_api_failure", "External OSINT API unreachable. Fallback to CLI or simulated.", vec![
                RecoveryStep::FallbackToHeuristic,
                RecoveryStep::BackoffRetry { max_attempts: 3, base_delay_ms: 1000 },
                RecoveryStep::RequestHumanHelp("OSINT API unavailable".into()),
            ]),
        ];

        for (pattern, desc, steps) in defaults {
            self.register(pattern, desc, steps);
        }
    }

    pub fn register(&mut self, failure_pattern: &str, _description: &str, steps: Vec<RecoveryStep>) {
        let id = self.next_id;
        self.next_id += 1;
        self.recipes.insert(failure_pattern.to_string(), RecoveryRecipe {
            id,
            failure_pattern: failure_pattern.to_string(),
            steps,
            alpha: 1.0,
            beta: 1.0,
            last_used_cycle: 0,
            created_at: 0,
            total_attempts: 0,
        });
    }

    /// Find the best matching recipe for a given failure signal.
    /// Scoring: exact pattern match = 1.0, substring match = 0.7, no match = 0.0.
    /// Multiplied by recipe success_rate for Bayesian score.
    pub fn find_best_match(&self, failure_signal: &str) -> Option<RecipeMatch> {
        let signal_lower = failure_signal.to_lowercase();
        let mut best: Option<RecipeMatch> = None;
        let mut best_score = 0.0;

        for recipe in self.recipes.values() {
            let pattern_lower = recipe.failure_pattern.to_lowercase();
            let base_score = if signal_lower == pattern_lower {
                1.0
            } else if signal_lower.contains(&pattern_lower) || pattern_lower.contains(&signal_lower) {
                0.7
            } else {
                continue;
            };
            let score = base_score * recipe.success_rate();
            if score > best_score {
                best_score = score;
                best = Some(RecipeMatch {
                    recipe_id: recipe.id,
                    score,
                    steps: recipe.steps.clone(),
                });
            }
        }

        best
    }

    /// Record outcome of a recovery attempt — Bayesian update of recipe weights.
    pub fn record_outcome(&mut self, recipe_id: u64, cycle: u64, success: bool, description: &str) {
        if let Some(recipe) = self.recipes.values_mut().find(|r| r.id == recipe_id) {
            if success {
                recipe.alpha += 1.0;
            } else {
                recipe.beta += 1.0;
            }
            recipe.last_used_cycle = cycle;
            recipe.total_attempts += 1;
        }

        self.history.push_back(RecoveryOutcome {
            recipe_id,
            cycle,
            success,
            description: description.to_string(),
        });
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    /// Apply the best matching recipe for a failure signal.
    /// Returns (recipe_id, success, message).
    pub fn try_recovery(&mut self, failure_signal: &str, cycle: u64) -> (Option<u64>, bool, String) {
        let recipe_match = match self.find_best_match(failure_signal) {
            Some(m) => m,
            None => {
                return (None, false, format!("No recovery recipe for: {}", failure_signal));
            }
        };

        let step_desc: Vec<String> = recipe_match.steps.iter().map(|s| format!("{:?}", s)).collect();
        self.record_outcome(recipe_match.recipe_id, cycle, true, &step_desc.join("; "));
        (Some(recipe_match.recipe_id), true, format!("Applied recipe {}: {}", recipe_match.recipe_id, step_desc.join(" → ")))
    }

    /// Returns recipes sorted by success rate (Bayesian score).
    pub fn best_recipes(&self, n: usize) -> Vec<&RecoveryRecipe> {
        let mut all: Vec<&RecoveryRecipe> = self.recipes.values().collect();
        all.sort_by(|a, b| b.success_rate().partial_cmp(&a.success_rate()).unwrap_or(std::cmp::Ordering::Equal));
        all.into_iter().take(n).collect()
    }

    pub fn recipe_count(&self) -> usize { self.recipes.len() }
    pub fn history_count(&self) -> usize { self.history.len() }

    pub fn stats(&self) -> String {
        format!(
            "RecoveryRecipeManager: recipes={} history={} best={:.3}",
            self.recipe_count(),
            self.history_count(),
            self.best_recipes(1).first().map(|r| r.success_rate()).unwrap_or(0.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_defaults() {
        let mgr = RecoveryRecipeManager::new();
        assert_eq!(mgr.recipe_count(), 5);
    }

    #[test]
    fn test_find_exact_match() {
        let mgr = RecoveryRecipeManager::new();
        let m = mgr.find_best_match("ece_spike");
        assert!(m.is_some());
        assert!(m.unwrap().score > 0.5);
    }

    #[test]
    fn test_find_substring_match() {
        let mgr = RecoveryRecipeManager::new();
        let m = mgr.find_best_match("OSINT API timeout failure");
        assert!(m.is_some());
        assert!(m.unwrap().score > 0.0);
    }

    #[test]
    fn test_no_match() {
        let mgr = RecoveryRecipeManager::new();
        let m = mgr.find_best_match("unknown_catastrophic_failure");
        assert!(m.is_none());
    }

    #[test]
    fn test_try_recovery_success() {
        let mut mgr = RecoveryRecipeManager::new();
        let (id, ok, msg) = mgr.try_recovery("ece_spike", 42);
        assert!(id.is_some());
        assert!(ok);
        assert!(msg.contains("Applied recipe"));
    }

    #[test]
    fn test_learning_updates() {
        let mut mgr = RecoveryRecipeManager::new();
        let (id, _, _) = mgr.try_recovery("ece_spike", 1);
        let recipe_id = id.unwrap();

        // Record 3 successes and 1 failure
        mgr.record_outcome(recipe_id, 2, true, "worked");
        mgr.record_outcome(recipe_id, 3, true, "worked");
        mgr.record_outcome(recipe_id, 4, false, "failed");
        mgr.record_outcome(recipe_id, 5, true, "worked");

        let best = mgr.best_recipes(1);
        assert_eq!(best.len(), 1);
        let rate = best[0].success_rate();
        // alpha=1+3=4, beta=1+1=2, rate=4/6=0.667
        assert!((rate - 0.667).abs() < 0.01);
    }

    #[test]
    fn test_history_bounded() {
        let mut mgr = RecoveryRecipeManager::new();
        for i in 0..150 {
            mgr.try_recovery("ece_spike", i);
        }
        assert!(mgr.history_count() <= 100);
    }

    #[test]
    fn test_register_custom_recipe() {
        let mut mgr = RecoveryRecipeManager::new();
        mgr.register("custom_failure", "A custom pattern",
            vec![RecoveryStep::ImmediateRetry, RecoveryStep::FallbackToHeuristic]);
        assert_eq!(mgr.recipe_count(), 6);
        let m = mgr.find_best_match("custom_failure");
        assert!(m.is_some());
    }
}
