use serde::{Serialize, Deserialize};
use crate::neotrix::orchestrator::flow_state::{ConfigState, StateManager};
#[cfg(test)]
use crate::neotrix::orchestrator::flow_state::FlowStateId;

/// Trigger for flow transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlowTrigger {
    /// Fires when flow starts
    Start,
    /// Fires when a specific step completes
    StepComplete(String),
    /// Fires when a condition is met
    Condition(String),
    /// Fires after a duration
    Timeout(std::time::Duration),
}

/// A single flow step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStep {
    pub id: String,
    pub description: String,
    pub trigger: FlowTrigger,
    pub action: String,
}

/// A flow definition — a graph of steps connected by triggers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flow {
    pub name: String,
    pub description: String,
    steps: Vec<FlowStep>,
    edges: Vec<(String, String)>,
}

impl Flow {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            steps: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a step that fires on start.
    pub fn start(mut self, id: &str, description: &str, action: &str) -> Self {
        self.steps.push(FlowStep {
            id: id.to_string(),
            description: description.to_string(),
            trigger: FlowTrigger::Start,
            action: action.to_string(),
        });
        self
    }

    /// Add a step that listens for another step's completion.
    pub fn listen(mut self, id: &str, description: &str, action: &str, after: &str) -> Self {
        self.steps.push(FlowStep {
            id: id.to_string(),
            description: description.to_string(),
            trigger: FlowTrigger::StepComplete(after.to_string()),
            action: action.to_string(),
        });
        self.edges.push((after.to_string(), id.to_string()));
        self
    }

    /// Add a routing step that branches based on a condition.
    pub fn router(mut self, id: &str, description: &str, condition: &str) -> Self {
        self.steps.push(FlowStep {
            id: id.to_string(),
            description: description.to_string(),
            trigger: FlowTrigger::Condition(condition.to_string()),
            action: "route".to_string(),
        });
        self
    }

    pub fn steps(&self) -> &[FlowStep] {
        &self.steps
    }

    pub fn edges(&self) -> &[(String, String)] {
        &self.edges
    }
}

/// Runtime state for a flow execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowRuntime<S: ConfigState> {
    pub flow: Flow,
    pub state: StateManager<S>,
    pub completed_steps: Vec<String>,
    pub current_step: Option<String>,
}

impl<S: ConfigState> FlowRuntime<S> {
    pub fn new(flow: Flow, initial_state: S) -> Self {
        Self {
            flow,
            state: StateManager::new(initial_state),
            completed_steps: Vec::new(),
            current_step: None,
        }
    }

    /// Get the next steps that are ready to execute.
    pub fn ready_steps(&self) -> Vec<&FlowStep> {
        let mut ready = Vec::new();
        for step in &self.flow.steps {
            match &step.trigger {
                FlowTrigger::Start => {
                    if self.completed_steps.is_empty() && self.current_step.is_none() {
                        ready.push(step);
                    }
                }
                FlowTrigger::StepComplete(after) => {
                    if self.completed_steps.contains(after) && !self.completed_steps.contains(&step.id) {
                        ready.push(step);
                    }
                }
                FlowTrigger::Condition(_) => {}
                FlowTrigger::Timeout(_) => {}
            }
        }
        ready
    }

    /// Mark a step as completed.
    pub fn complete_step(&mut self, step_id: &str) {
        self.completed_steps.push(step_id.to_string());
        self.current_step = None;
    }

    pub fn is_complete(&self) -> bool {
        self.flow.steps.iter().all(|s| self.completed_steps.contains(&s.id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct SimpleState {
        id: FlowStateId,
        value: i32,
    }

    impl ConfigState for SimpleState {
        fn state_id(&self) -> FlowStateId { self.id.clone() }
        fn merge(&mut self, other: Self) { self.value = other.value; }
    }

    #[test]
    fn test_flow_builder() {
        let flow = Flow::new("test", "Test flow")
            .start("s1", "Step 1", "analyze")
            .listen("s2", "Step 2", "implement", "s1")
            .listen("s3", "Step 3", "verify", "s2");
        assert_eq!(flow.steps().len(), 3);
        assert_eq!(flow.edges().len(), 2);
    }

    #[test]
    fn test_flow_runtime_ready_steps() {
        let flow = Flow::new("test", "")
            .start("s1", "First", "do_a")
            .listen("s2", "Second", "do_b", "s1");
        let state = SimpleState { id: FlowStateId::new(), value: 0 };
        let runtime = FlowRuntime::new(flow, state);
        let ready = runtime.ready_steps();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "s1");
    }

    #[test]
    fn test_flow_runtime_progression() {
        let flow = Flow::new("test", "")
            .start("s1", "First", "do_a")
            .listen("s2", "Second", "do_b", "s1");
        let state = SimpleState { id: FlowStateId::new(), value: 0 };
        let mut runtime = FlowRuntime::new(flow, state);

        runtime.complete_step("s1");
        assert!(runtime.completed_steps.contains(&"s1".to_string()));

        let ready = runtime.ready_steps();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "s2");
    }

    #[test]
    fn test_flow_is_complete() {
        let flow = Flow::new("test", "")
            .start("s1", "Only step", "do_it");
        let state = SimpleState { id: FlowStateId::new(), value: 0 };
        let mut runtime = FlowRuntime::new(flow, state);
        assert!(!runtime.is_complete());
        runtime.complete_step("s1");
        assert!(runtime.is_complete());
    }

    #[test]
    fn test_router_step() {
        let flow = Flow::new("routing", "")
            .start("s1", "Start", "collect_data")
            .router("decision", "Check condition", "value > 10");
        let steps: Vec<String> = flow.steps().iter().map(|s| s.id.clone()).collect();
        assert!(steps.contains(&"decision".to_string()));
    }

    #[test]
    fn test_flow_with_flowstate() {
        let flow = Flow::new("stateful", "Flow with state")
            .start("init", "Initialize", "setup");
        let state = SimpleState { id: FlowStateId::new(), value: 42 };
        let runtime = FlowRuntime::new(flow, state);
        assert_eq!(runtime.state.current.value, 42);
    }
}
