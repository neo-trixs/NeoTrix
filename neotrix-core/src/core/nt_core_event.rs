use serde::{Serialize, Deserialize};

/// Core event enum — type-safe, no `dyn Any` downcasting.
/// Each variant carries typed payload directly.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreEvent {
    #[serde(rename = "task_submitted")]
    TaskSubmitted {
        task: String,
        task_type: String,
        priority: u32,
    },
    #[serde(rename = "agent_feedback")]
    AgentFeedback {
        agent_id: String,
        feedback: String,
        score: f64,
    },
    #[serde(rename = "global_halt")]
    GlobalHalt {
        reason: String,
        source: String,
    },
    #[serde(rename = "external_reward")]
    ExternalReward {
        reward: f64,
        source: String,
    },
    #[serde(rename = "goal_completed")]
    GoalCompleted {
        goal_id: String,
        goal: String,
        iterations: u64,
        score: f64,
    },
    #[serde(rename = "budget_exceeded")]
    BudgetExceeded {
        goal_id: String,
        budget_used: f64,
        max_budget: f64,
    },
    #[serde(rename = "agent_team")]
    AgentTeam {
        agent_id: String,
        action: String,
        timestamp: i64,
    },
    #[serde(rename = "system_error")]
    SystemError {
        component: String,
        error: String,
        severity: String,
    },
    #[serde(rename = "consciousness_critique")]
    ConsciousnessCritique {
        quality: f64,
        relevance: f64,
        consistency: f64,
        timestamp: i64,
    },
}

// ── Backward-compatible type aliases ──────────────────────────────────────
pub type BusEvent = CoreEvent;

// ── Tests ────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_submitted() {
        let e = CoreEvent::TaskSubmitted { task: "t".into(), task_type: "g".into(), priority: 1 };
        assert_eq!(e.task(), "t");
    }

    #[test]
    fn test_agent_feedback() {
        let e = CoreEvent::AgentFeedback { agent_id: "a1".into(), feedback: "good".into(), score: 0.9 };
        assert_eq!(e.agent_id(), "a1");
    }

    #[test]
    fn test_global_halt() {
        let e = CoreEvent::GlobalHalt { reason: "err".into(), source: "test".into() };
        assert_eq!(e.reason(), "err");
    }

    #[test]
    fn test_external_reward() {
        let e = CoreEvent::ExternalReward { reward: 1.0, source: "env".into() };
        assert_eq!(e.reward(), 1.0);
    }

    #[test]
    fn test_goal_completed() {
        let e = CoreEvent::GoalCompleted { goal_id: "g1".into(), goal: "test".into(), iterations: 5, score: 0.8 };
        assert_eq!(e.goal_id(), "g1");
    }

    #[test]
    fn test_budget_exceeded() {
        let e = CoreEvent::BudgetExceeded { goal_id: "g1".into(), budget_used: 100.0, max_budget: 50.0 };
        assert!(e.budget_used() > e.max_budget());
    }

    #[test]
    fn test_agent_team() {
        let e = CoreEvent::AgentTeam { agent_id: "a1".into(), action: "join".into(), timestamp: 1000 };
        assert_eq!(e.action(), "join");
    }

    #[test]
    fn test_system_error() {
        let e = CoreEvent::SystemError { component: "db".into(), error: "timeout".into(), severity: "critical".into() };
        assert_eq!(e.severity(), "critical");
    }

    #[test]
    fn test_consciousness_critique() {
        let e = CoreEvent::ConsciousnessCritique { quality: 0.7, relevance: 0.8, consistency: 0.9, timestamp: 1000 };
        assert!((e.quality() - 0.7).abs() < 0.01);
        assert!((e.relevance() - 0.8).abs() < 0.01);
        assert!((e.consistency() - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_json_roundtrip() {
        let e = CoreEvent::GoalCompleted { goal_id: "g1".into(), goal: "test".into(), iterations: 5, score: 0.8 };
        let json = serde_json::to_string(&e).unwrap();
        let parsed: CoreEvent = serde_json::from_str(&json).unwrap();
        match parsed {
            CoreEvent::GoalCompleted { goal_id, goal: _, iterations, score: _ } => {
                assert_eq!(goal_id, "g1");
                assert_eq!(iterations, 5);
            }
            _ => panic!("wrong variant"),
        }
    }
}

// ── Accessor impls on enum ───────────────────────────────────────────────
// Avoid external pattern matching churn by providing field-level accessors.
impl CoreEvent {
    pub fn task(&self) -> &str { match self { Self::TaskSubmitted { task, .. } => task, _ => "" } }
    pub fn task_type(&self) -> &str { match self { Self::TaskSubmitted { task_type, .. } => task_type, _ => "" } }
    pub fn priority(&self) -> u32 { match self { Self::TaskSubmitted { priority, .. } => *priority, _ => 0 } }
    pub fn agent_id(&self) -> &str { match self { Self::AgentFeedback { agent_id, .. } => agent_id, Self::AgentTeam { agent_id, .. } => agent_id, _ => "" } }
    pub fn feedback(&self) -> &str { match self { Self::AgentFeedback { feedback, .. } => feedback, _ => "" } }
    pub fn score(&self) -> f64 { match self { Self::AgentFeedback { score, .. } => *score, _ => 0.0 } }
    pub fn reason(&self) -> &str { match self { Self::GlobalHalt { reason, .. } => reason, _ => "" } }
    pub fn source(&self) -> &str { match self { Self::GlobalHalt { source, .. } => source, Self::ExternalReward { source, .. } => source, _ => "" } }
    pub fn reward(&self) -> f64 { match self { Self::ExternalReward { reward, .. } => *reward, _ => 0.0 } }
    pub fn goal_id(&self) -> &str { match self { Self::GoalCompleted { goal_id, .. } => goal_id, Self::BudgetExceeded { goal_id, .. } => goal_id, _ => "" } }
    pub fn budget_used(&self) -> f64 { match self { Self::BudgetExceeded { budget_used, .. } => *budget_used, _ => 0.0 } }
    pub fn max_budget(&self) -> f64 { match self { Self::BudgetExceeded { max_budget, .. } => *max_budget, _ => 0.0 } }
    pub fn action(&self) -> &str { match self { Self::AgentTeam { action, .. } => action, _ => "" } }
    pub fn component(&self) -> &str { match self { Self::SystemError { component, .. } => component, _ => "" } }
    pub fn severity(&self) -> &str { match self { Self::SystemError { severity, .. } => severity, _ => "" } }
    pub fn quality(&self) -> f64 { match self { Self::ConsciousnessCritique { quality, .. } => *quality, _ => 0.0 } }
    pub fn relevance(&self) -> f64 { match self { Self::ConsciousnessCritique { relevance, .. } => *relevance, _ => 0.0 } }
    pub fn consistency(&self) -> f64 { match self { Self::ConsciousnessCritique { consistency, .. } => *consistency, _ => 0.0 } }
}
