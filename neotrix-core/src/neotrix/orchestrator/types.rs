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
