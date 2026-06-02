use std::collections::VecDeque;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub tech_stack: Vec<String>,
    pub last_opened: i64,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManager {
    pub(crate) projects: Vec<ProjectInfo>,
    pub(crate) recent_projects: VecDeque<String>,
    pub(crate) active_id: Option<String>,
}

pub struct GitIntegration;
