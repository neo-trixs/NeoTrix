use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastGoal {
    pub objective: String,
    pub status: _FastGoalStatus,
    pub summary: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum _FastGoalStatus {
    Active,
    Completed,
    Abandoned,
}

impl FastGoal {
    pub fn new(objective: impl Into<String>) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            objective: objective.into(),
            status: _FastGoalStatus::Active,
            summary: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub(crate) fn _to_context_block(&self) -> Vec<String> {
        let mut lines = vec![
            "[Runtime Context — Goal]".into(),
            format!("Objective: {}", self.objective),
            format!("Status: {:?}", self.status),
        ];
        if let Some(ref summary) = self.summary {
            lines.push(format!("Progress: {}", summary));
        }
        lines.push("[/Runtime Context]".into());
        lines
    }

    pub fn is_active(&self) -> bool {
        self.status == _FastGoalStatus::Active
    }

    pub fn complete(&mut self, summary: Option<String>) {
        self.status = _FastGoalStatus::Completed;
        self.summary = summary;
        self.updated_at = chrono::Utc::now().timestamp();
    }

    pub(crate) fn _abandon(&mut self) {
        self.status = _FastGoalStatus::Abandoned;
        self.updated_at = chrono::Utc::now().timestamp();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_goal_creation() {
        let goal = FastGoal::new("analyze project architecture");
        assert!(goal.is_active());
        assert_eq!(goal.objective, "analyze project architecture");
    }

    #[test]
    fn test_fast_goal_context_block() {
        let mut goal = FastGoal::new("fix performance");
        goal.summary = Some("identified 3 bottlenecks".into());
        let ctx = goal._to_context_block();
        assert!(ctx[0].contains("Runtime Context"));
        assert!(ctx.iter().any(|l| l.contains("fix performance")));
    }

    #[test]
    fn test_fast_goal_completion() {
        let mut goal = FastGoal::new("add tests");
        goal.complete(Some("added 10 tests".into()));
        assert!(!goal.is_active());
        assert_eq!(goal.status, _FastGoalStatus::Completed);
    }
}
