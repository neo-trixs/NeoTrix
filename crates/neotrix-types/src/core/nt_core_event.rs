use std::any::Any;

pub trait BusEvent: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn BusEvent>;
}

#[derive(Clone, Debug)]
pub struct TaskSubmittedEvent {
    pub task: String,
    pub task_type: String,
    pub priority: u32,
}

impl BusEvent for TaskSubmittedEvent {
    fn as_any(&self) -> &dyn Any { self }
    fn clone_box(&self) -> Box<dyn BusEvent> { Box::new(self.clone()) }
}

#[derive(Clone, Debug)]
pub struct AgentFeedbackEvent {
    pub agent_id: String,
    pub feedback: String,
    pub score: f64,
}

impl BusEvent for AgentFeedbackEvent {
    fn as_any(&self) -> &dyn Any { self }
    fn clone_box(&self) -> Box<dyn BusEvent> { Box::new(self.clone()) }
}

#[derive(Clone, Debug)]
pub struct GlobalHaltEvent {
    pub reason: String,
    pub source: String,
}

impl BusEvent for GlobalHaltEvent {
    fn as_any(&self) -> &dyn Any { self }
    fn clone_box(&self) -> Box<dyn BusEvent> { Box::new(self.clone()) }
}

#[derive(Clone, Debug)]
pub struct ExternalRewardEvent {
    pub reward: f64,
    pub source: String,
}

impl BusEvent for ExternalRewardEvent {
    fn as_any(&self) -> &dyn Any { self }
    fn clone_box(&self) -> Box<dyn BusEvent> { Box::new(self.clone()) }
}

#[derive(Clone, Debug)]
pub struct GoalCompletedEvent {
    pub goal_id: String,
    pub goal: String,
    pub iterations: u64,
    pub score: f64,
}

impl BusEvent for GoalCompletedEvent {
    fn as_any(&self) -> &dyn Any { self }
    fn clone_box(&self) -> Box<dyn BusEvent> { Box::new(self.clone()) }
}

#[derive(Clone, Debug)]
pub struct BudgetExceededEvent {
    pub goal_id: String,
    pub budget_used: f64,
    pub max_budget: f64,
}

impl BusEvent for BudgetExceededEvent {
    fn as_any(&self) -> &dyn Any { self }
    fn clone_box(&self) -> Box<dyn BusEvent> { Box::new(self.clone()) }
}

#[derive(Clone, Debug)]
pub struct AgentTeamEvent {
    pub agent_id: String,
    pub action: String,
    pub timestamp: i64,
}

impl BusEvent for AgentTeamEvent {
    fn as_any(&self) -> &dyn Any { self }
    fn clone_box(&self) -> Box<dyn BusEvent> { Box::new(self.clone()) }
}

#[derive(Clone, Debug)]
pub struct SystemErrorEvent {
    pub component: String,
    pub error: String,
    pub severity: String,
}

impl BusEvent for SystemErrorEvent {
    fn as_any(&self) -> &dyn Any { self }
    fn clone_box(&self) -> Box<dyn BusEvent> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_bus_event<E: BusEvent + Clone + std::fmt::Debug + 'static>(e: &E) {
        let _ = format!("{:?}", e);
        let boxed = e.clone_box();
        assert!(boxed.as_any().downcast_ref::<E>().is_some());
    }

    #[test]
    fn test_task_submitted() {
        let e = TaskSubmittedEvent { task: "t".into(), task_type: "g".into(), priority: 1 };
        assert_eq!(e.task, "t");
        assert_bus_event(&e);
    }

    #[test]
    fn test_agent_feedback() {
        let e = AgentFeedbackEvent { agent_id: "a1".into(), feedback: "good".into(), score: 0.9 };
        assert_eq!(e.agent_id, "a1");
        assert_bus_event(&e);
    }

    #[test]
    fn test_global_halt() {
        let e = GlobalHaltEvent { reason: "err".into(), source: "test".into() };
        assert_eq!(e.reason, "err");
        assert_bus_event(&e);
    }

    #[test]
    fn test_external_reward() {
        let e = ExternalRewardEvent { reward: 1.0, source: "env".into() };
        assert_eq!(e.reward, 1.0);
        assert_bus_event(&e);
    }

    #[test]
    fn test_goal_completed() {
        let e = GoalCompletedEvent { goal_id: "g1".into(), goal: "test".into(), iterations: 5, score: 0.8 };
        assert_eq!(e.goal_id, "g1");
        assert_bus_event(&e);
    }

    #[test]
    fn test_budget_exceeded() {
        let e = BudgetExceededEvent { goal_id: "g1".into(), budget_used: 100.0, max_budget: 50.0 };
        assert!(e.budget_used > e.max_budget);
        assert_bus_event(&e);
    }

    #[test]
    fn test_agent_team() {
        let e = AgentTeamEvent { agent_id: "a1".into(), action: "join".into(), timestamp: 1000 };
        assert_eq!(e.action, "join");
        assert_bus_event(&e);
    }

    #[test]
    fn test_system_error() {
        let e = SystemErrorEvent { component: "db".into(), error: "timeout".into(), severity: "critical".into() };
        assert_eq!(e.severity, "critical");
        assert_bus_event(&e);
    }
}
