pub type GoalId = u64;

#[derive(Debug, Clone)]
pub enum GoalStatus {
    Active,
    InProgress,
    Completed { outcome: String },
    Failed { reason: String },
    Abandoned,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum GoalPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub struct Goal {
    pub id: GoalId,
    pub description: String,
    pub priority: GoalPriority,
    pub status: GoalStatus,
    pub created_at: u64,
    pub entropy_source: Option<String>,
    pub target_metric: Option<String>,
    pub target_value: Option<f64>,
    pub current_value: Option<f64>,
}

impl Goal {
    pub fn progress(&self) -> f64 {
        match (self.current_value, self.target_value) {
            (Some(current), Some(target)) if target > 0.0 => (current / target).clamp(0.0, 1.0),
            _ => 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GoalProgress {
    pub goal_id: GoalId,
    pub progress: f64,
    pub eta_steps: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct GoalSynthesizer {
    next_id: u64,
    goals: Vec<Goal>,
    max_active_goals: usize,
}

impl Default for GoalSynthesizer {
    fn default() -> Self {
        Self::new()
    }
}

impl GoalSynthesizer {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            goals: Vec::new(),
            max_active_goals: 5,
        }
    }

    pub fn with_max_active(mut self, max: usize) -> Self {
        self.max_active_goals = max.max(1);
        self
    }

    pub fn synthesize_from_curiosity(
        &mut self,
        curiosity_signal: f64,
        source_description: &str,
    ) -> Option<GoalId> {
        if curiosity_signal <= 0.5 {
            return None;
        }
        if self.active_goal_count() >= self.max_active_goals {
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;
        let priority = if curiosity_signal > 0.9 {
            GoalPriority::Critical
        } else if curiosity_signal > 0.7 {
            GoalPriority::High
        } else {
            GoalPriority::Medium
        };
        self.goals.push(Goal {
            id,
            description: format!("Explore {}", source_description),
            priority,
            status: GoalStatus::Active,
            created_at: Self::now(),
            entropy_source: Some(source_description.to_string()),
            target_metric: Some("knowledge_gain".to_string()),
            target_value: Some(curiosity_signal),
            current_value: Some(0.0),
        });
        Some(id)
    }

    pub fn synthesize_from_entropy(&mut self, entropy: f64, domain: &str) -> Option<GoalId> {
        if entropy <= 0.7 {
            return None;
        }
        if self.active_goal_count() >= self.max_active_goals {
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;
        let priority = if entropy > 0.9 {
            GoalPriority::Critical
        } else {
            GoalPriority::High
        };
        self.goals.push(Goal {
            id,
            description: format!("Reduce uncertainty in {}", domain),
            priority,
            status: GoalStatus::Active,
            created_at: Self::now(),
            entropy_source: Some(domain.to_string()),
            target_metric: Some("prediction_accuracy".to_string()),
            target_value: Some(1.0 - entropy),
            current_value: Some(0.0),
        });
        Some(id)
    }

    pub fn update_progress(&mut self, id: GoalId, value: f64) {
        if let Some(goal) = self.goals.iter_mut().find(|g| g.id == id) {
            goal.current_value = Some(value);
            if let Some(target) = goal.target_value {
                if value >= target {
                    goal.status = GoalStatus::Completed {
                        outcome: format!(
                            "Reached target {:.3} (metric: {})",
                            value,
                            goal.target_metric.as_deref().unwrap_or("unknown")
                        ),
                    };
                } else {
                    goal.status = GoalStatus::InProgress;
                }
            }
        }
    }

    pub fn active_goals(&self) -> Vec<&Goal> {
        self.goals
            .iter()
            .filter(|g| matches!(g.status, GoalStatus::Active | GoalStatus::InProgress))
            .collect()
    }

    fn active_goal_count(&self) -> usize {
        self.goals
            .iter()
            .filter(|g| matches!(g.status, GoalStatus::Active | GoalStatus::InProgress))
            .count()
    }

    pub fn abandon(&mut self, id: GoalId, reason: &str) {
        if let Some(goal) = self.goals.iter_mut().find(|g| g.id == id) {
            goal.status = GoalStatus::Abandoned;
            goal.entropy_source = Some(reason.to_string());
        }
    }

    pub fn prioritize(&mut self, id: GoalId, new_priority: GoalPriority) {
        if let Some(goal) = self.goals.iter_mut().find(|g| g.id == id) {
            goal.priority = new_priority;
        }
    }

    pub fn summary(&self) -> String {
        if self.goals.is_empty() {
            return "No goals synthesized.".to_string();
        }
        let mut lines = Vec::new();
        for goal in &self.goals {
            let status_str = match &goal.status {
                GoalStatus::Active => "ACTIVE".to_string(),
                GoalStatus::InProgress => {
                    let p = goal.progress();
                    format!("IN_PROGRESS ({:.0}%)", p * 100.0)
                }
                GoalStatus::Completed { outcome } => format!("COMPLETED: {}", outcome),
                GoalStatus::Failed { reason } => format!("FAILED: {}", reason),
                GoalStatus::Abandoned => "ABANDONED".to_string(),
            };
            lines.push(format!(
                "#{} [{}] {} — {}",
                goal.id,
                format!("{:?}", goal.priority),
                goal.description,
                status_str,
            ));
        }
        lines.join("\n")
    }

    pub fn all_goals(&self) -> &[Goal] {
        &self.goals
    }

    pub fn get_goal(&self, id: GoalId) -> Option<&Goal> {
        self.goals.iter().find(|g| g.id == id)
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_synthesizer_is_empty() {
        let s = GoalSynthesizer::new();
        assert_eq!(s.active_goal_count(), 0);
        assert!(s.active_goals().is_empty());
        assert_eq!(s.summary(), "No goals synthesized.");
    }

    #[test]
    fn test_synthesize_curiosity_below_threshold() {
        let mut s = GoalSynthesizer::new();
        let id = s.synthesize_from_curiosity(0.3, "test domain");
        assert!(id.is_none());
        assert_eq!(s.active_goal_count(), 0);
    }

    #[test]
    fn test_synthesize_curiosity_above_threshold() {
        let mut s = GoalSynthesizer::new();
        let id = s.synthesize_from_curiosity(0.7, "novel pattern");
        assert!(id.is_some());
        assert_eq!(s.active_goal_count(), 1);
        let goal = s.get_goal(id.unwrap()).unwrap();
        assert_eq!(goal.description, "Explore novel pattern");
        assert_eq!(goal.priority, GoalPriority::High);
        assert!(matches!(goal.status, GoalStatus::Active));
    }

    #[test]
    fn test_synthesize_curiosity_critical() {
        let mut s = GoalSynthesizer::new();
        let id = s.synthesize_from_curiosity(0.95, "critical anomaly");
        assert!(id.is_some());
        let goal = s.get_goal(id.unwrap()).unwrap();
        assert_eq!(goal.priority, GoalPriority::Critical);
    }

    #[test]
    fn test_synthesize_entropy_below_threshold() {
        let mut s = GoalSynthesizer::new();
        let id = s.synthesize_from_entropy(0.5, "visual cortex");
        assert!(id.is_none());
        assert_eq!(s.active_goal_count(), 0);
    }

    #[test]
    fn test_synthesize_entropy_above_threshold() {
        let mut s = GoalSynthesizer::new();
        let id = s.synthesize_from_entropy(0.85, "language model");
        assert!(id.is_some());
        let goal = s.get_goal(id.unwrap()).unwrap();
        assert_eq!(goal.description, "Reduce uncertainty in language model");
        assert_eq!(goal.priority, GoalPriority::High);
    }

    #[test]
    fn test_synthesize_entropy_critical() {
        let mut s = GoalSynthesizer::new();
        let id = s.synthesize_from_entropy(0.95, "world model");
        assert!(id.is_some());
        let goal = s.get_goal(id.unwrap()).unwrap();
        assert_eq!(goal.priority, GoalPriority::Critical);
    }

    #[test]
    fn test_update_progress_completes_goal() {
        let mut s = GoalSynthesizer::new();
        let id = s.synthesize_from_curiosity(0.8, "test").unwrap();
        s.update_progress(id, 0.8);
        let goal = s.get_goal(id).unwrap();
        assert!(matches!(goal.status, GoalStatus::Completed { .. }));
        assert_eq!(s.active_goal_count(), 0);
    }

    #[test]
    fn test_update_progress_partial() {
        let mut s = GoalSynthesizer::new();
        let id = s.synthesize_from_curiosity(0.8, "test").unwrap();
        s.update_progress(id, 0.3);
        let goal = s.get_goal(id).unwrap();
        assert!(matches!(goal.status, GoalStatus::InProgress));
        assert_eq!(goal.current_value, Some(0.3));
    }

    #[test]
    fn test_abandon_goal() {
        let mut s = GoalSynthesizer::new();
        let id = s.synthesize_from_curiosity(0.8, "test").unwrap();
        s.abandon(id, "irrelevant");
        let goal = s.get_goal(id).unwrap();
        assert!(matches!(goal.status, GoalStatus::Abandoned));
        assert_eq!(s.active_goal_count(), 0);
    }

    #[test]
    fn test_prioritize() {
        let mut s = GoalSynthesizer::new();
        let id = s.synthesize_from_curiosity(0.6, "test").unwrap();
        assert_eq!(s.get_goal(id).unwrap().priority, GoalPriority::Medium);
        s.prioritize(id, GoalPriority::Critical);
        assert_eq!(s.get_goal(id).unwrap().priority, GoalPriority::Critical);
    }

    #[test]
    fn test_max_active_goals_enforced() {
        let mut s = GoalSynthesizer::new().with_max_active(2);
        assert!(s.synthesize_from_curiosity(0.8, "a").is_some());
        assert!(s.synthesize_from_curiosity(0.8, "b").is_some());
        assert!(s.synthesize_from_curiosity(0.8, "c").is_none());
    }

    #[test]
    fn test_summary_format() {
        let mut s = GoalSynthesizer::new();
        s.synthesize_from_curiosity(0.6, "test pattern");
        let summary = s.summary();
        assert!(summary.contains("#1"));
        assert!(summary.contains("[Medium]"));
        assert!(summary.contains("Explore test pattern"));
        assert!(summary.contains("ACTIVE"));
    }

    #[test]
    fn test_goal_progress_calculation() {
        let mut s = GoalSynthesizer::new();
        let id = s.synthesize_from_curiosity(0.8, "test").unwrap();
        let goal = s.get_goal(id).unwrap();
        assert_eq!(goal.progress(), 0.0);
        s.update_progress(id, 0.4);
        let goal = s.get_goal(id).unwrap();
        assert!((goal.progress() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_id_increment() {
        let mut s = GoalSynthesizer::new();
        let id1 = s.synthesize_from_curiosity(0.8, "a").unwrap();
        let id2 = s.synthesize_from_curiosity(0.8, "b").unwrap();
        assert_eq!(id2, id1 + 1);
    }

    #[test]
    fn test_default_max_active() {
        let s = GoalSynthesizer::new();
        assert_eq!(s.max_active_goals, 5);
    }

    #[test]
    fn test_all_goals_returns_all() {
        let mut s = GoalSynthesizer::new();
        let id1 = s.synthesize_from_curiosity(0.8, "a").unwrap();
        let id2 = s.synthesize_from_entropy(0.8, "b").unwrap();
        assert_eq!(s.all_goals().len(), 2);
    }

    #[test]
    fn test_goal_progress_struct() {
        let gp = GoalProgress {
            goal_id: 1,
            progress: 0.75,
            eta_steps: Some(3),
        };
        assert_eq!(gp.progress, 0.75);
        assert_eq!(gp.eta_steps, Some(3));
    }

    #[test]
    fn test_goal_clone() {
        let g = Goal {
            id: 1,
            description: "test".into(),
            priority: GoalPriority::High,
            status: GoalStatus::Active,
            created_at: 100,
            entropy_source: None,
            target_metric: None,
            target_value: None,
            current_value: None,
        };
        let g2 = g.clone();
        assert_eq!(g.id, g2.id);
    }

    #[test]
    fn test_synthesizer_clone() {
        let mut s = GoalSynthesizer::new();
        s.synthesize_from_curiosity(0.8, "a");
        let s2 = s.clone();
        assert_eq!(s2.all_goals().len(), 1);
    }
}
