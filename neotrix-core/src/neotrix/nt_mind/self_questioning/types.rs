use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDesc {
    pub name: String,
    pub attributes: Vec<String>,
    pub parent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationDesc {
    pub name: String,
    pub input_params: Vec<String>,
    pub output_effect: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentProfile {
    pub domain: String,
    pub entities: Vec<EntityDesc>,
    pub operations: Vec<OperationDesc>,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationTrajectory {
    pub env_profile: EnvironmentProfile,
    pub states: Vec<String>,
    pub actions: Vec<String>,
    pub observations: Vec<String>,
    pub breadth_phase_steps: usize,
    pub depth_phase_steps: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTask {
    pub id: String,
    pub query: String,
    pub reference_solution: Vec<String>,
    pub difficulty: f64,
    pub style_hints: Vec<String>,
    pub source_trajectory_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuratedTask {
    pub task: GeneratedTask,
    pub passed_dedup: bool,
    pub passed_feasibility: bool,
    pub llm_judge_score: f64,
    pub proxy_reward: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfQuestionConfig {
    pub breadth_steps: usize,
    pub depth_steps: usize,
    pub max_tasks_per_round: usize,
    pub llm_temperature: f64,
    pub dedup_threshold: f64,
    pub min_judge_score: f64,
    pub auto_enqueue_goals: bool,
}

impl Default for SelfQuestionConfig {
    fn default() -> Self {
        Self {
            breadth_steps: 10,
            depth_steps: 5,
            max_tasks_per_round: 5,
            llm_temperature: 1.2,
            dedup_threshold: 0.85,
            min_judge_score: 0.6,
            auto_enqueue_goals: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfQuestionRoundResult {
    pub num_tasks_generated: usize,
    pub num_tasks_curated: usize,
    pub avg_judge_score: f64,
    pub sum_proxy_reward: f64,
}
