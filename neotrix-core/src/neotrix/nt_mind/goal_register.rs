use std::collections::HashMap;

/// A tracked goal with progress quantification.
///
/// Implements the explicit goal-awareness piece of Sutton's Discovery loop:
/// - Goals define what we're trying to achieve (the "Evaluation" criterion)
/// - Progress quantifies how far we are from the goal in real-time
/// - This enables runtime self-evaluation without external reward
#[derive(Debug, Clone)]
pub struct Goal {
    pub name: String,
    pub description: String,
    pub target: f64,
    pub current: f64,
    pub weight: f64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Goal {
    pub fn new(name: &str, description: &str, target: f64, weight: f64, now: u64) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            target,
            current: 0.0,
            weight,
            created_at: now,
            updated_at: now,
        }
    }

    /// Progress toward goal, clamped to [0, 1].
    pub fn progress(&self) -> f64 {
        if self.target == 0.0 {
            return 1.0;
        }
        (self.current / self.target).clamp(0.0, 1.0)
    }

    /// Remaining gap as a fraction of target, clamped to [0, 1].
    pub fn gap(&self) -> f64 {
        1.0 - self.progress()
    }

    /// Whether the goal is fully achieved.
    pub fn is_achieved(&self) -> bool {
        self.current >= self.target
    }
}

/// Registry of active goals with progress tracking.
///
/// Each SEAL iteration checks goals, quantifies progress, and
/// feeds the aggregate progress signal into the reward calculation.
#[derive(Debug, Clone)]
pub struct GoalRegister {
    pub goals: HashMap<String, Goal>,
    pub iteration: u64,
}

impl Default for GoalRegister {
    fn default() -> Self {
        Self::new()
    }
}

impl GoalRegister {
    pub fn new() -> Self {
        Self {
            goals: HashMap::new(),
            iteration: 0,
        }
    }

    /// Register a new goal.
    pub fn register(&mut self, name: &str, description: &str, target: f64, weight: f64) {
        let goal = Goal::new(name, description, target, weight, self.iteration);
        self.goals.insert(name.to_string(), goal);
    }

    /// Update progress toward a named goal.
    pub fn update_progress(&mut self, name: &str, current: f64) {
        if let Some(goal) = self.goals.get_mut(name) {
            goal.current = current;
            goal.updated_at = self.iteration;
        }
    }

    /// Aggregate progress across all goals (weighted).
    pub fn overall_progress(&self) -> f64 {
        let total_weight: f64 = self.goals.values().map(|g| g.weight).sum();
        if total_weight == 0.0 {
            return 0.0;
        }
        let weighted: f64 = self.goals.values().map(|g| g.progress() * g.weight).sum();
        weighted / total_weight
    }

    /// Aggregate remaining gap across all goals.
    pub fn overall_gap(&self) -> f64 {
        1.0 - self.overall_progress()
    }

    /// Number of achieved goals.
    pub fn achieved_count(&self) -> usize {
        self.goals.values().filter(|g| g.is_achieved()).count()
    }

    /// Number of active (not yet achieved) goals.
    pub fn active_count(&self) -> usize {
        self.goals.values().filter(|g| !g.is_achieved()).count()
    }

    /// Advance iteration counter.
    pub fn tick(&mut self) {
        self.iteration += 1;
    }

    /// Reset all goals.
    pub fn reset(&mut self) {
        self.goals.clear();
        self.iteration = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goal_new() {
        let g = Goal::new("test", "a test goal", 100.0, 1.0, 0);
        assert_eq!(g.name, "test");
        assert!((g.progress() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_goal_progress_partial() {
        let mut g = Goal::new("test", "", 100.0, 1.0, 0);
        g.current = 50.0;
        assert!((g.progress() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_goal_progress_complete() {
        let mut g = Goal::new("test", "", 100.0, 1.0, 0);
        g.current = 150.0;
        assert!((g.progress() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_goal_gap() {
        let mut g = Goal::new("test", "", 100.0, 1.0, 0);
        g.current = 30.0;
        assert!((g.gap() - 0.7).abs() < 1e-9);
    }

    #[test]
    fn test_goal_is_achieved() {
        let mut g = Goal::new("test", "", 100.0, 1.0, 0);
        assert!(!g.is_achieved());
        g.current = 100.0;
        assert!(g.is_achieved());
    }

    #[test]
    fn test_register_and_update() {
        let mut reg = GoalRegister::new();
        reg.register("accuracy", "reach 90% accuracy", 90.0, 1.0);
        assert_eq!(reg.active_count(), 1);
        reg.update_progress("accuracy", 85.0);
        assert!((reg.overall_progress() - 85.0 / 90.0).abs() < 1e-9);
    }

    #[test]
    fn test_overall_progress_weighted() {
        let mut reg = GoalRegister::new();
        reg.register("a", "goal a", 100.0, 2.0);
        reg.register("b", "goal b", 100.0, 1.0);
        reg.update_progress("a", 50.0);
        reg.update_progress("b", 100.0);
        let expected = (0.5 * 2.0 + 1.0 * 1.0) / 3.0;
        assert!((reg.overall_progress() - expected).abs() < 1e-9);
    }

    #[test]
    fn test_overall_gap() {
        let mut reg = GoalRegister::new();
        reg.register("x", "", 100.0, 1.0);
        reg.update_progress("x", 40.0);
        assert!((reg.overall_gap() - 0.6).abs() < 1e-9);
    }

    #[test]
    fn test_achieved_count() {
        let mut reg = GoalRegister::new();
        reg.register("a", "", 100.0, 1.0);
        reg.register("b", "", 100.0, 1.0);
        reg.update_progress("a", 100.0);
        assert_eq!(reg.achieved_count(), 1);
        assert_eq!(reg.active_count(), 1);
    }

    #[test]
    fn test_tick() {
        let mut reg = GoalRegister::new();
        assert_eq!(reg.iteration, 0);
        reg.tick();
        assert_eq!(reg.iteration, 1);
    }

    #[test]
    fn test_reset() {
        let mut reg = GoalRegister::new();
        reg.register("a", "", 100.0, 1.0);
        reg.tick();
        reg.reset();
        assert_eq!(reg.iteration, 0);
        assert!(reg.goals.is_empty());
    }
}
