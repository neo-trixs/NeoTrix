#[derive(Debug, Clone)]
pub enum NodeType {
    Planner,
    Worker,
    Critic,
}

#[derive(Debug, Clone)]
pub struct LatentState {
    pub latent_summary: String,
    pub task_state: String,
    pub confidence: f64,
    pub metrics: String,
}

/// Task definition (local, replaces L4 nt_core_parallel::types::Task)
#[derive(Debug, Clone)]
pub struct Task {
    pub agent_id: String,
    pub input: Vec<f64>,
    pub priority: i32,
}

impl Task {
    pub fn new(agent_id: String, input: Vec<f64>, priority: i32) -> Self {
        Self { agent_id, input, priority }
    }
}
