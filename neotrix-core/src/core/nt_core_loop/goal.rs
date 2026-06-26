#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoopGoal {
    pub description: String,
    pub min_quality: f64,
    pub min_coherence: f64,
    pub max_iterations: u64,
    pub iterations_used: u64,
    pub achieved: bool,
}

impl LoopGoal {
    pub fn new(
        description: &str,
        min_quality: f64,
        min_coherence: f64,
        max_iterations: u64,
    ) -> Self {
        Self {
            description: description.to_string(),
            min_quality,
            min_coherence,
            max_iterations,
            iterations_used: 0,
            achieved: false,
        }
    }

    pub fn evaluate(&mut self, quality: f64, coherence: f64) -> LoopGoalStatus {
        self.iterations_used += 1;

        if quality >= self.min_quality && coherence >= self.min_coherence {
            self.achieved = true;
            return LoopGoalStatus::Achieved;
        }

        if self.iterations_used >= self.max_iterations {
            return LoopGoalStatus::MaxIterationsExceeded;
        }

        LoopGoalStatus::InProgress {
            remaining: self.max_iterations - self.iterations_used,
            quality_gap: (self.min_quality - quality).max(0.0),
            coherence_gap: (self.min_coherence - coherence).max(0.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoopGoalStatus {
    Achieved,
    InProgress {
        remaining: u64,
        quality_gap: f64,
        coherence_gap: f64,
    },
    MaxIterationsExceeded,
}

pub struct GoalRegistry {
    pub goals: Vec<LoopGoal>,
}

impl GoalRegistry {
    pub fn new() -> Self {
        Self { goals: Vec::new() }
    }

    pub fn register(&mut self, goal: LoopGoal) {
        self.goals.push(goal);
    }

    pub fn active_goal(&self) -> Option<&LoopGoal> {
        self.goals.iter().find(|g| !g.achieved)
    }

    pub fn active_goal_mut(&mut self) -> Option<&mut LoopGoal> {
        self.goals.iter_mut().find(|g| !g.achieved)
    }

    pub fn stats(&self) -> GoalRegistryStats {
        GoalRegistryStats {
            total: self.goals.len(),
            achieved: self.goals.iter().filter(|g| g.achieved).count(),
            active: self.goals.iter().filter(|g| !g.achieved).count(),
        }
    }
}

pub struct GoalRegistryStats {
    pub total: usize,
    pub achieved: usize,
    pub active: usize,
}

impl Default for GoalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goal_new_unachieved() {
        let g = LoopGoal::new("test", 0.8, 0.7, 5);
        assert!(!g.achieved);
        assert_eq!(g.max_iterations, 5);
        assert_eq!(g.iterations_used, 0);
    }

    #[test]
    fn test_goal_evaluate_achieved() {
        let mut g = LoopGoal::new("test", 0.8, 0.7, 5);
        let status = g.evaluate(0.9, 0.8);
        assert_eq!(status, LoopGoalStatus::Achieved);
        assert!(g.achieved);
    }

    #[test]
    fn test_goal_evaluate_in_progress() {
        let mut g = LoopGoal::new("test", 0.8, 0.7, 5);
        let status = g.evaluate(0.5, 0.5);
        match status {
            LoopGoalStatus::InProgress {
                remaining,
                quality_gap,
                coherence_gap,
            } => {
                assert_eq!(remaining, 4);
                assert!((quality_gap - 0.3).abs() < 0.01);
                assert!((coherence_gap - 0.2).abs() < 0.01);
            }
            _ => panic!("expected InProgress"),
        }
    }

    #[test]
    fn test_goal_max_iterations_exceeded() {
        let mut g = LoopGoal::new("test", 0.8, 0.7, 2);
        g.evaluate(0.5, 0.5);
        let status = g.evaluate(0.5, 0.5);
        assert_eq!(status, LoopGoalStatus::MaxIterationsExceeded);
    }

    #[test]
    fn test_goal_registry() {
        let mut reg = GoalRegistry::new();
        reg.register(LoopGoal::new("a", 0.8, 0.7, 5));
        reg.register(LoopGoal::new("b", 0.8, 0.7, 5));
        assert!(reg.active_goal().is_some());
        let stats = reg.stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.active, 2);
    }
}
