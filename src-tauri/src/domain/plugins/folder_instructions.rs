//! # Folder Instructions Domain Plugin
//!
//! Implements per-folder context injection (Claude/Cursor pattern).
//! Each project folder can have a `.neotrix/` directory with:
//! - `instructions.md` — always loaded when folder is active
//! - `memory.md` — project-specific memory
//! - `skills/` — project-specific skills
//! - `config.toml` — project-specific configuration
//!
//! # Architecture
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │         Folder Instructions                  │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
//! │  │  Scanner │  │  Injector│  │  Merger  │  │
//! │  │          │  │          │  │          │  │
//! │  └──────────┘  └──────────┘  └──────────┘  │
//! │       ↓              ↓              ↓        │
//! │  ┌──────────────────────────────────────┐   │
//! │  │      .neotrix/ Project Config       │   │
//! │  └──────────────────────────────────────┘   │
//! └─────────────────────────────────────────────┘
//! ```

use crate::domain::app_handle::{get_app_handle, set_app_handle};
use crate::domain::{ActionSpec, DomainError, DomainPlugin, ParamSpec};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ========== Types ==========

/// Folder instruction entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderInstruction {
    /// File path relative to .neotrix/
    pub file: String,
    /// Content
    pub content: String,
    /// File type
    pub file_type: String,
    /// Size in bytes
    pub size: u64,
    /// Last modified at
    pub modified_at: String,
}

/// Project context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    /// Project root path
    pub root_path: String,
    /// Project name
    pub name: String,
    /// Whether .neotrix/ directory exists
    pub has_config: bool,
    /// Instructions content
    pub instructions: Option<String>,
    /// Memory content
    pub memory: Option<String>,
    /// Project-specific skills
    pub skills: Vec<ProjectSkill>,
    /// Project config
    pub config: ProjectConfig,
    /// Loaded at timestamp
    pub loaded_at: String,
}

/// Project-specific skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSkill {
    pub name: String,
    pub path: String,
    pub description: String,
    pub enabled: bool,
}

/// Project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Default model for this project
    pub default_model: Option<String>,
    /// Max tokens for this project
    pub max_tokens: Option<u32>,
    /// Temperature for this project
    pub temperature: Option<f32>,
    /// Auto-load instructions on folder open
    pub auto_load: bool,
    /// Additional context files to include
    pub include_files: Vec<String>,
    /// Files to exclude
    pub exclude_files: Vec<String>,
    /// Project-specific environment variables
    pub env: HashMap<String, String>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            default_model: None,
            max_tokens: None,
            temperature: None,
            auto_load: true,
            include_files: vec![],
            exclude_files: vec![],
            env: HashMap::new(),
        }
    }
}

/// Merged context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedContext {
    /// Full merged context string
    pub full_context: String,
    /// Source files that contributed
    pub sources: Vec<String>,
    /// Total token count (estimated)
    pub estimated_tokens: usize,
    /// Context was modified
    pub was_modified: bool,
}

/// Folder instructions statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderStats {
    pub projects_loaded: usize,
    pub total_instructions: usize,
    pub total_memory_entries: usize,
    pub total_skills: usize,
    pub avg_instruction_tokens: usize,
}

// ========== Plugin ==========

/// Folder Instructions Domain Plugin
pub struct FolderInstructionsPlugin {
    state: Arc<Mutex<FolderState>>,
}

struct FolderState {
    /// Loaded project contexts by root path
    projects: HashMap<String, ProjectContext>,
    /// Global instructions (applied to all projects)
    global_instructions: Option<String>,
}

impl FolderInstructionsPlugin {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(FolderState {
                projects: HashMap::new(),
                global_instructions: None,
            })),
        }
    }

    /// Load project context from a directory
    fn load_project(&self, root_path: &str) -> Result<ProjectContext, DomainError> {
        let mut state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;

        let name = std::path::Path::new(root_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Simplified — would actually read .neotrix/ directory
        let instructions = Some(format!(
            "# Project Instructions for {}\n\nThis project uses NeoTrix for AI-assisted development.\n\n## Guidelines\n- Follow Rust best practices\n- Use nt_ prefix for all modules\n- Keep functions small and focused",
            name
        ));

        let context = ProjectContext {
            root_path: root_path.to_string(),
            name,
            has_config: true,
            instructions,
            memory: Some("# Project Memory\n\nNo entries yet.".to_string()),
            skills: Vec::new(),
            config: ProjectConfig::default(),
            loaded_at: chrono::Utc::now().to_rfc3339(),
        };

        state
            .projects
            .insert(root_path.to_string(), context.clone());
        Ok(context)
    }

    /// Unload a project
    fn unload_project(&self, root_path: &str) -> Result<(), DomainError> {
        let mut state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;
        state.projects.remove(root_path);
        Ok(())
    }

    /// Get merged context for a project
    fn get_context(&self, root_path: &str) -> Result<MergedContext, DomainError> {
        let state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;

        let project = state.projects.get(root_path).ok_or_else(|| DomainError {
            code: "PROJECT_NOT_FOUND".into(),
            message: format!("Project '{}' not loaded", root_path),
            recoverable: true,
        })?;

        let mut parts = Vec::new();
        let mut sources = Vec::new();

        if let Some(ref global) = state.global_instructions {
            parts.push(global.clone());
            sources.push("global".to_string());
        }

        if let Some(ref instructions) = project.instructions {
            parts.push(instructions.clone());
            sources.push(format!("{}/.neotrix/instructions.md", root_path));
        }

        if let Some(ref memory) = project.memory {
            parts.push(memory.clone());
            sources.push(format!("{}/.neotrix/memory.md", root_path));
        }

        let full_context = parts.join("\n\n---\n\n");
        let estimated_tokens = full_context.split_whitespace().count() * 4 / 3; // rough estimate

        Ok(MergedContext {
            full_context,
            sources,
            estimated_tokens,
            was_modified: false,
        })
    }

    /// List all loaded projects
    fn list_projects(&self) -> Result<Vec<ProjectContext>, DomainError> {
        let state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;
        Ok(state.projects.values().cloned().collect())
    }

    /// Set global instructions
    fn set_global_instructions(&self, instructions: &str) -> Result<(), DomainError> {
        let mut state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;
        state.global_instructions = Some(instructions.to_string());
        Ok(())
    }

    /// Get stats
    fn get_stats(&self) -> Result<FolderStats, DomainError> {
        let state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;

        let total_instructions = state
            .projects
            .values()
            .filter(|p| p.instructions.is_some())
            .count();
        let total_memory = state
            .projects
            .values()
            .filter(|p| p.memory.is_some())
            .count();
        let total_skills: usize = state.projects.values().map(|p| p.skills.len()).sum();

        Ok(FolderStats {
            projects_loaded: state.projects.len(),
            total_instructions,
            total_memory_entries: total_memory,
            total_skills,
            avg_instruction_tokens: 0,
        })
    }
}

impl Default for FolderInstructionsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DomainPlugin for FolderInstructionsPlugin {
    fn name(&self) -> &str {
        "folder_instructions"
    }

    fn description(&self) -> &str {
        "Per-folder context injection — .neotrix/ project instructions, memory, and skills"
    }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec {
                name: "load_project".into(),
                description: "Load project context from a directory".into(),
                params: vec![ParamSpec {
                    name: "root_path".into(),
                    r#type: "string".into(),
                    optional: false,
                    description: "Project root path".into(),
                }],
                ..Default::default()
            },
            ActionSpec {
                name: "unload_project".into(),
                description: "Unload a project context".into(),
                params: vec![ParamSpec {
                    name: "root_path".into(),
                    r#type: "string".into(),
                    optional: false,
                    description: "Project root path".into(),
                }],
                ..Default::default()
            },
            ActionSpec {
                name: "get_context".into(),
                description: "Get merged context for a project".into(),
                params: vec![ParamSpec {
                    name: "root_path".into(),
                    r#type: "string".into(),
                    optional: false,
                    description: "Project root path".into(),
                }],
                ..Default::default()
            },
            ActionSpec {
                name: "list_projects".into(),
                description: "List all loaded projects".into(),
                params: vec![],
                ..Default::default()
            },
            ActionSpec {
                name: "set_global_instructions".into(),
                description: "Set global instructions applied to all projects".into(),
                params: vec![ParamSpec {
                    name: "instructions".into(),
                    r#type: "string".into(),
                    optional: false,
                    description: "Instructions content".into(),
                }],
                ..Default::default()
            },
            ActionSpec {
                name: "get_stats".into(),
                description: "Get folder instructions statistics".into(),
                params: vec![],
                ..Default::default()
            },
        ]
    }

    async fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            "load_project" => {
                let root_path =
                    args.get("root_path")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_PARAMS".into(),
                            message: "Missing 'root_path'".into(),
                            recoverable: true,
                        })?;
                let ctx = self.load_project(root_path)?;
                Ok(serde_json::to_value(ctx).unwrap_or_default())
            }
            "unload_project" => {
                let root_path =
                    args.get("root_path")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_PARAMS".into(),
                            message: "Missing 'root_path'".into(),
                            recoverable: true,
                        })?;
                self.unload_project(root_path)?;
                Ok(serde_json::json!({"success": true}))
            }
            "get_context" => {
                let root_path =
                    args.get("root_path")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_PARAMS".into(),
                            message: "Missing 'root_path'".into(),
                            recoverable: true,
                        })?;
                let ctx = self.get_context(root_path)?;
                Ok(serde_json::to_value(ctx).unwrap_or_default())
            }
            "list_projects" => {
                let projects = self.list_projects()?;
                Ok(serde_json::json!({"projects": projects, "total": projects.len()}))
            }
            "set_global_instructions" => {
                let instructions = args
                    .get("instructions")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                self.set_global_instructions(instructions)?;
                Ok(serde_json::json!({"success": true}))
            }
            "get_stats" => {
                let stats = self.get_stats()?;
                Ok(serde_json::to_value(stats).unwrap_or_default())
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }

    async fn init(&mut self) -> Result<(), DomainError> {
        Ok(())
    }
    async fn shutdown(&mut self) -> Result<(), DomainError> {
        Ok(())
    }
}
