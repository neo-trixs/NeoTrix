//! Record & Replay — Workflow Recording to Reusable Skills
//!
//! Implements the ChatGPT Codex pattern: demonstrate a workflow once and
//! convert it to a reusable skill (SKILL.md + supporting files).
//!
//! # Architecture
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │         Record & Replay                      │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
//! │  │ Recorder │  │ Analyzer │  │ Skill    │  │
//! │  │  Engine  │  │  Engine  │  │ Generator│  │
//! │  └──────────┘  └──────────┘  └──────────┘  │
//! │       ↓              ↓              ↓        │
//! │  ┌──────────────────────────────────────┐   │
//! │  │      Action Capture Layer            │   │
//! │  └──────────────────────────────────────┘   │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! # Safety
//! - Recording is read-only (captures user actions)
//! - Generated skills require user approval before execution
//! - No unsafe code (R-P1)

use std::collections::HashMap;

use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Core Types
// ============================================================================

/// Action type in a recorded workflow
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ActionType {
    /// File edit
    FileEdit { path: String, content: String },
    /// File create
    FileCreate { path: String, content: String },
    /// File delete
    FileDelete { path: String },
    /// Terminal command
    TerminalCommand { command: String, working_dir: Option<String> },
    /// Browser action
    BrowserAction { url: Option<String>, action: String },
    /// MCP tool call
    McpToolCall { server: String, tool: String, params: HashMap<String, String> },
    /// User input (text/prompt)
    UserInput { prompt: String, response: Option<String> },
    /// Wait/delay
    Delay { seconds: f64 },
    /// Conditional branch
    Condition { condition: String, true_actions: Vec<usize>, false_actions: Vec<usize> },
    /// Loop
    Loop { count: usize, body_actions: Vec<usize> },
}

/// Recorded action with metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecordedAction {
    /// Action index in the workflow
    pub index: usize,
    /// The action
    pub action: ActionType,
    /// Timestamp
    pub timestamp: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Whether this action succeeded
    pub success: bool,
    /// Output/error
    pub output: Option<String>,
    /// Screenshot path (if captured)
    pub screenshot: Option<String>,
}

/// Recording session
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecordingSession {
    /// Session ID
    pub id: String,
    /// Session name
    pub name: String,
    /// Description
    pub description: String,
    /// Recorded actions
    pub actions: Vec<RecordedAction>,
    /// Status
    pub status: RecordingStatus,
    /// Started at
    pub started_at: String,
    /// Ended at
    pub ended_at: Option<String>,
    /// Tags
    pub tags: Vec<String>,
}

/// Recording status
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RecordingStatus {
    /// Recording in progress
    Recording,
    /// Recording paused
    Paused,
    /// Recording completed
    Completed,
    /// Recording cancelled
    Cancelled,
}

/// Generated skill from recording
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GeneratedSkill {
    /// Skill ID
    pub id: String,
    /// Skill name
    pub name: String,
    /// SKILL.md content
    pub skill_md: String,
    /// Supporting files
    pub supporting_files: Vec<(String, String)>, // (filename, content)
    /// Source recording ID
    pub source_recording_id: String,
    /// Generated at
    pub generated_at: String,
    /// Whether user has approved
    pub approved: bool,
}

/// Skill generation options
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillGenOptions {
    /// Whether to include screenshots in skill
    pub include_screenshots: bool,
    /// Whether to generate loop constructs
    pub generate_loops: bool,
    /// Whether to generate conditional branches
    pub generate_conditions: bool,
    /// Maximum skill complexity
    pub max_complexity: usize,
    /// Custom metadata to add to frontmatter
    pub metadata: HashMap<String, String>,
}

impl Default for SkillGenOptions {
    fn default() -> Self {
        Self {
            include_screenshots: false,
            generate_loops: true,
            generate_conditions: true,
            max_complexity: 50,
            metadata: HashMap::new(),
        }
    }
}

/// Statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecordReplayStats {
    pub total_recordings: usize,
    pub completed_recordings: usize,
    pub generated_skills: usize,
    pub approved_skills: usize,
}

// ============================================================================
// Record & Replay Engine
// ============================================================================

/// Workflow recording and skill generation engine
pub struct RecordReplayEngine {
    /// Active recording sessions
    sessions: Arc<RwLock<HashMap<String, RecordingSession>>>,
    /// Generated skills
    skills: Arc<RwLock<HashMap<String, GeneratedSkill>>>,
}

impl RecordReplayEngine {
    /// Create a new engine
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            skills: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a new recording session
    pub async fn start_recording(
        &self,
        name: String,
        description: String,
    ) -> Result<String, RecordReplayError> {
        let session_id = format!("rec-{}", uuid::Uuid::new_v4());
        let session = RecordingSession {
            id: session_id.clone(),
            name,
            description,
            actions: Vec::new(),
            status: RecordingStatus::Recording,
            started_at: chrono::Utc::now().to_rfc3339(),
            ended_at: None,
            tags: Vec::new(),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);

        Ok(session_id)
    }

    /// Record an action in a session
    pub async fn record_action(
        &self,
        session_id: &str,
        action: ActionType,
    ) -> Result<(), RecordReplayError> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            if session.status != RecordingStatus::Recording {
                return Err(RecordReplayError::NotRecording(session_id.to_string()));
            }

            let index = session.actions.len();
            let recorded = RecordedAction {
                index,
                action,
                timestamp: chrono::Utc::now().to_rfc3339(),
                duration_ms: 0,
                success: true,
                output: None,
                screenshot: None,
            };

            session.actions.push(recorded);
            Ok(())
        } else {
            Err(RecordReplayError::NotFound(session_id.to_string()))
        }
    }

    /// Stop recording
    pub async fn stop_recording(&self, session_id: &str) -> Result<(), RecordReplayError> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = RecordingStatus::Completed;
            session.ended_at = Some(chrono::Utc::now().to_rfc3339());
            Ok(())
        } else {
            Err(RecordReplayError::NotFound(session_id.to_string()))
        }
    }

    /// Generate a skill from a recording
    pub async fn generate_skill(
        &self,
        session_id: &str,
        options: SkillGenOptions,
    ) -> Result<GeneratedSkill, RecordReplayError> {
        let session = {
            let sessions = self.sessions.read().await;
            sessions.get(session_id).cloned()
                .ok_or(RecordReplayError::NotFound(session_id.to_string()))?
        };

        if session.status != RecordingStatus::Completed {
            return Err(RecordReplayError::NotCompleted(session_id.to_string()));
        }

        // Generate SKILL.md
        let skill_md = self.generate_skill_md(&session, &options);

        // Generate supporting files
        let supporting_files = self.generate_supporting_files(&session, &options);

        let skill_id = format!("skill-{}", uuid::Uuid::new_v4());
        let skill = GeneratedSkill {
            id: skill_id.clone(),
            name: session.name.clone(),
            skill_md,
            supporting_files,
            source_recording_id: session_id.to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            approved: false,
        };

        let mut skills = self.skills.write().await;
        skills.insert(skill_id, skill.clone());

        Ok(skill)
    }

    /// Generate SKILL.md content
    fn generate_skill_md(&self, session: &RecordingSession, options: &SkillGenOptions) -> String {
        let mut md = String::new();

        // Frontmatter
        md.push_str("---\n");
        md.push_str(&format!("name: {}\n", session.name.to_lowercase().replace(' ', "-")));
        md.push_str(&format!("description: Auto-generated from recording: {}\n", session.description));
        md.push_str("tags: recorded, auto-generated\n");
        if !options.metadata.is_empty() {
            for (key, value) in &options.metadata {
                md.push_str(&format!("{}: {}\n", key, value));
            }
        }
        md.push_str("---\n\n");

        // Title
        md.push_str(&format!("# {}\n\n", session.name));

        // Description
        md.push_str(&format!("{}\n\n", session.description));

        // Steps
        md.push_str("## Steps\n\n");
        for (i, action) in session.actions.iter().enumerate() {
            md.push_str(&format!("### Step {}\n\n", i + 1));
            match &action.action {
                ActionType::FileEdit { path, .. } => {
                    md.push_str(&format!("Edit file: `{}`\n\n", path));
                }
                ActionType::FileCreate { path, .. } => {
                    md.push_str(&format!("Create file: `{}`\n\n", path));
                }
                ActionType::FileDelete { path } => {
                    md.push_str(&format!("Delete file: `{}`\n\n", path));
                }
                ActionType::TerminalCommand { command, working_dir } => {
                    md.push_str(&format!("Run command: `{}`\n", command));
                    if let Some(dir) = working_dir {
                        md.push_str(&format!("Working directory: `{}`\n", dir));
                    }
                    md.push_str("\n");
                }
                ActionType::BrowserAction { url, action } => {
                    if let Some(url) = url {
                        md.push_str(&format!("Open URL: `{}`\n", url));
                    }
                    md.push_str(&format!("Browser action: {}\n\n", action));
                }
                ActionType::McpToolCall { server, tool, params } => {
                    md.push_str(&format!("Call MCP tool `{}` on server `{}`\n", tool, server));
                    if !params.is_empty() {
                        md.push_str(&format!("Parameters: `{:?}`\n", params));
                    }
                    md.push_str("\n");
                }
                ActionType::UserInput { prompt, .. } => {
                    md.push_str(&format!("User input: {}\n\n", prompt));
                }
                ActionType::Delay { seconds } => {
                    md.push_str(&format!("Wait {} seconds\n\n", seconds));
                }
                _ => {}
            }
        }

        md
    }

    /// Generate supporting files from recording
    fn generate_supporting_files(
        &self,
        session: &RecordingSession,
        _options: &SkillGenOptions,
    ) -> Vec<(String, String)> {
        let mut files = Vec::new();

        // Extract file contents from recording
        for action in &session.actions {
            match &action.action {
                ActionType::FileCreate { path, content } => {
                    let filename = path.split('/').last().unwrap_or("file.txt").to_string();
                    files.push((filename, content.clone()));
                }
                ActionType::FileEdit { path, content } => {
                    let filename = path.split('/').last().unwrap_or("file.txt").to_string();
                    files.push((filename, content.clone()));
                }
                _ => {}
            }
        }

        files
    }

    /// Approve a generated skill
    pub async fn approve_skill(&self, skill_id: &str) -> Result<(), RecordReplayError> {
        let mut skills = self.skills.write().await;
        if let Some(skill) = skills.get_mut(skill_id) {
            skill.approved = true;
            Ok(())
        } else {
            Err(RecordReplayError::NotFound(skill_id.to_string()))
        }
    }

    /// Get a skill by ID
    pub async fn get_skill(&self, skill_id: &str) -> Option<GeneratedSkill> {
        let skills = self.skills.read().await;
        skills.get(skill_id).cloned()
    }

    /// List all skills
    pub async fn list_skills(&self) -> Vec<GeneratedSkill> {
        let skills = self.skills.read().await;
        skills.values().cloned().collect()
    }

    /// Get statistics
    pub async fn stats(&self) -> RecordReplayStats {
        let sessions = self.sessions.read().await;
        let skills = self.skills.read().await;

        RecordReplayStats {
            total_recordings: sessions.len(),
            completed_recordings: sessions.values()
                .filter(|s| s.status == RecordingStatus::Completed)
                .count(),
            generated_skills: skills.len(),
            approved_skills: skills.values()
                .filter(|s| s.approved)
                .count(),
        }
    }
}

impl Default for RecordReplayEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Errors
// ============================================================================

/// Record & Replay errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum RecordReplayError {
    #[error("recording session not found: {0}")]
    NotFound(String),

    #[error("session not recording: {0}")]
    NotRecording(String),

    #[error("session not completed: {0}")]
    NotCompleted(String),

    #[error("skill generation failed: {0}")]
    GenerationFailed(String),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_and_generate_skill() {
        let engine = RecordReplayEngine::new();

        // Start recording
        let session_id = engine.start_recording(
            "Deploy App".to_string(),
            "Deploy application to production".to_string(),
        ).await.unwrap();

        // Record actions
        engine.record_action(&session_id, ActionType::TerminalCommand {
            command: "cargo build --release".to_string(),
            working_dir: None,
        }).await.unwrap();

        engine.record_action(&session_id, ActionType::FileEdit {
            path: "config.toml".to_string(),
            content: "[deploy]\nenv = \"production\"".to_string(),
        }).await.unwrap();

        engine.record_action(&session_id, ActionType::TerminalCommand {
            command: "cargo deploy".to_string(),
            working_dir: None,
        }).await.unwrap();

        // Stop recording
        engine.stop_recording(&session_id).await.unwrap();

        // Generate skill
        let skill = engine.generate_skill(&session_id, SkillGenOptions::default()).await.unwrap();

        assert!(!skill.skill_md.is_empty());
        assert!(skill.skill_md.contains("# Deploy App"));
    }

    #[tokio::test]
    async fn test_recording_stats() {
        let engine = RecordReplayEngine::new();
        let stats = engine.stats().await;
        assert_eq!(stats.total_recordings, 0);
        assert_eq!(stats.generated_skills, 0);
    }
}
