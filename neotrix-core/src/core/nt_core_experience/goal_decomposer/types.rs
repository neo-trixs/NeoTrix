use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum DecomposerGoalStatus {
    Draft,
    InProgress,
    Paused(String),
    Completed(String),
    Failed(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GoalBoundaries {
    pub allowed_paths: Vec<String>,
    pub forbidden_paths: Vec<String>,
    pub max_files_to_modify: usize,
}

impl Default for GoalBoundaries {
    fn default() -> Self {
        Self {
            allowed_paths: vec!["**/*.rs".into(), "**/*.md".into()],
            forbidden_paths: vec![
                "**/target/**".into(),
                "**/.git/**".into(),
                "**/node_modules/**".into(),
            ],
            max_files_to_modify: 10,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IterationPolicy {
    pub max_rounds: usize,
    pub rerun_after_change: bool,
    pub inspect_logs_before_retry: bool,
    pub stop_on_consecutive_failures: usize,
}

impl Default for IterationPolicy {
    fn default() -> Self {
        Self {
            max_rounds: 3,
            rerun_after_change: true,
            inspect_logs_before_retry: true,
            stop_on_consecutive_failures: 2,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GoalContract {
    pub id: String,
    pub raw_request: String,
    pub outcome: String,
    pub verification: Vec<String>,
    pub constraints: Vec<String>,
    pub boundaries: GoalBoundaries,
    pub iteration_policy: IterationPolicy,
    pub stop_conditions: Vec<String>,
    pub pause_conditions: Vec<String>,
    pub created_at: u64,
    pub status: DecomposerGoalStatus,
}

impl GoalContract {
    pub fn new(request: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let id = format!("goal_{:x}", now);
        Self {
            id,
            raw_request: request.to_string(),
            outcome: String::new(),
            verification: Vec::new(),
            constraints: Vec::new(),
            boundaries: GoalBoundaries::default(),
            iteration_policy: IterationPolicy::default(),
            stop_conditions: Vec::new(),
            pause_conditions: Vec::new(),
            created_at: now,
            status: DecomposerGoalStatus::Draft,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum StepStatus {
    Pending,
    InProgress,
    Verified,
    Failed(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GoalStep {
    pub id: usize,
    pub description: String,
    pub verification_hint: String,
    pub status: StepStatus,
    pub error_log: Vec<String>,
}

impl GoalStep {
    pub fn new(id: usize, description: &str, verification_hint: &str) -> Self {
        Self {
            id,
            description: description.to_string(),
            verification_hint: verification_hint.to_string(),
            status: StepStatus::Pending,
            error_log: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StepVerificationResult {
    pub passed: bool,
    pub evidence: String,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct DecompositionContext {
    pub project_type: Option<String>,
    pub existing_files: Vec<String>,
    pub user_constraints: Vec<String>,
    pub domain: Option<String>,
}

impl Default for DecompositionContext {
    fn default() -> Self {
        Self {
            project_type: None,
            existing_files: Vec::new(),
            user_constraints: Vec::new(),
            domain: None,
        }
    }
}
