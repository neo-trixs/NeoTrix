//! # Unified Surface Domain Plugin
//!
//! Implements the ChatGPT/Claude pattern: unified Chat+Work+Code surface
//! with mode switching in a single desktop app.
//!
//! # Architecture
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │         Unified Surface                      │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
//! │  │  Chat    │  │  Work    │  │  Code    │  │
//! │  │  Mode    │  │  Mode    │  │  Mode    │  │
//! │  └──────────┘  └──────────┘  └──────────┘  │
//! │       ↓              ↓              ↓        │
//! │  ┌──────────────────────────────────────┐   │
//! │  │      Mode Router & State Manager     │   │
//! │  └──────────────────────────────────────┘   │
//! └─────────────────────────────────────────────//! ```

use crate::domain::{ActionSpec, DomainError, DomainPlugin, ParamSpec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::AppHandle;

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub fn set_app_handle(app: AppHandle) {
    let _ = APP_HANDLE.set(app);
}

// ========== Types ==========

/// Surface mode (ChatGPT/Claude pattern)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SurfaceMode {
    /// Fast conversational assistance
    Chat,
    /// Long-running tasks, document creation, research
    Work,
    /// Software development, Git integration, testing
    Code,
}

impl Default for SurfaceMode {
    fn default() -> Self {
        Self::Chat
    }
}

impl std::fmt::Display for SurfaceMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SurfaceMode::Chat => write!(f, "Chat"),
            SurfaceMode::Work => write!(f, "Work"),
            SurfaceMode::Code => write!(f, "Code"),
        }
    }
}

/// Mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    /// Mode name
    pub mode: SurfaceMode,
    /// Display label
    pub label: String,
    /// Description
    pub description: String,
    /// Icon name
    pub icon: String,
    /// Available features in this mode
    pub features: Vec<String>,
    /// Default model tier
    pub default_model_tier: String,
    /// Whether to show file panel
    pub show_file_panel: bool,
    /// Whether to show terminal
    pub show_terminal: bool,
    /// Whether to show browser
    pub show_browser: bool,
}

impl Default for ModeConfig {
    fn default() -> Self {
        Self {
            mode: SurfaceMode::Chat,
            label: "Chat".to_string(),
            description: "Fast conversational assistance".to_string(),
            icon: "chat".to_string(),
            features: vec!["chat".to_string(), "search".to_string()],
            default_model_tier: "balanced".to_string(),
            show_file_panel: false,
            show_terminal: false,
            show_browser: false,
        }
    }
}

/// Surface state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceState {
    /// Current mode
    pub current_mode: SurfaceMode,
    /// Mode configurations
    pub modes: HashMap<SurfaceMode, ModeConfig>,
    /// Active session ID
    pub active_session_id: Option<String>,
    /// Panel visibility
    pub panels: PanelVisibility,
    /// History of mode switches
    pub mode_history: Vec<ModeSwitch>,
}

/// Panel visibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelVisibility {
    pub file_panel: bool,
    pub terminal: bool,
    pub browser: bool,
    pub memory: bool,
    pub skills: bool,
    pub automations: bool,
}

impl Default for PanelVisibility {
    fn default() -> Self {
        Self {
            file_panel: false,
            terminal: false,
            browser: false,
            memory: false,
            skills: false,
            automations: false,
        }
    }
}

/// Mode switch event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeSwitch {
    pub from: SurfaceMode,
    pub to: SurfaceMode,
    pub timestamp: String,
    pub reason: Option<String>,
}

/// Session info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceSession {
    pub id: String,
    pub mode: SurfaceMode,
    pub title: String,
    pub created_at: String,
    pub last_active: String,
    pub message_count: usize,
    pub pinned: bool,
    pub project_id: Option<String>,
}

/// Statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceStats {
    pub total_sessions: usize,
    pub sessions_by_mode: HashMap<String, usize>,
    pub total_mode_switches: usize,
    pub avg_session_duration_ms: f64,
}

// ========== Plugin ==========

/// Unified Surface Domain Plugin
pub struct UnifiedSurfacePlugin {
    state: Arc<Mutex<SurfaceState>>,
}

impl UnifiedSurfacePlugin {
    pub fn new() -> Self {
        let mut modes = HashMap::new();

        modes.insert(SurfaceMode::Chat, ModeConfig {
            mode: SurfaceMode::Chat,
            label: "Chat".to_string(),
            description: "Fast conversational assistance and everyday questions".to_string(),
            icon: "chat-bubble".to_string(),
            features: vec![
                "chat".to_string(),
                "search".to_string(),
                "quick-questions".to_string(),
            ],
            default_model_tier: "balanced".to_string(),
            show_file_panel: false,
            show_terminal: false,
            show_browser: false,
        });

        modes.insert(SurfaceMode::Work, ModeConfig {
            mode: SurfaceMode::Work,
            label: "Work".to_string(),
            description: "Research, analyze, create documents, spreadsheets, presentations".to_string(),
            icon: "work".to_string(),
            features: vec![
                "research".to_string(),
                "document-creation".to_string(),
                "analysis".to_string(),
                "file-access".to_string(),
                "browser".to_string(),
                "scheduled-tasks".to_string(),
            ],
            default_model_tier: "strong".to_string(),
            show_file_panel: true,
            show_terminal: false,
            show_browser: true,
        });

        modes.insert(SurfaceMode::Code, ModeConfig {
            mode: SurfaceMode::Code,
            label: "Code".to_string(),
            description: "Software development, Git integration, testing, deployment".to_string(),
            icon: "code".to_string(),
            features: vec![
                "code-editing".to_string(),
                "git-integration".to_string(),
                "testing".to_string(),
                "terminal".to_string(),
                "worktrees".to_string(),
                "code-review".to_string(),
            ],
            default_model_tier: "strong".to_string(),
            show_file_panel: true,
            show_terminal: true,
            show_browser: false,
        });

        Self {
            state: Arc::new(Mutex::new(SurfaceState {
                current_mode: SurfaceMode::Chat,
                modes,
                active_session_id: None,
                panels: PanelVisibility::default(),
                mode_history: Vec::new(),
            })),
        }
    }

    /// Switch mode
    fn switch_mode(&self, target: SurfaceMode, reason: Option<String>) -> Result<SurfaceState, DomainError> {
        let mut state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;

        let from = state.current_mode.clone();

        // Record switch
        state.mode_history.push(ModeSwitch {
            from: from.clone(),
            to: target.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            reason,
        });

        // Update mode
        state.current_mode = target.clone();

        // Update panel visibility based on mode config
        if let Some(config) = state.modes.get(&target) {
            state.panels.file_panel = config.show_file_panel;
            state.panels.terminal = config.show_terminal;
            state.panels.browser = config.show_browser;
        }

        Ok(state.clone())
    }

    /// Get current state
    fn get_state(&self) -> Result<SurfaceState, DomainError> {
        let state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;
        Ok(state.clone())
    }
}

impl Default for UnifiedSurfacePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainPlugin for UnifiedSurfacePlugin {
    fn name(&self) -> &str {
        "unified_surface"
    }

    fn description(&self) -> &str {
        "Unified Chat+Work+Code surface with mode switching (ChatGPT/Claude pattern)"
    }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec {
                name: "switch_mode".into(),
                description: "Switch between Chat/Work/Code modes".into(),
                params: vec![ParamSpec {
                    name: "mode".into(),
                    typ: "string".into(),
                    required: true,
                    description: "Target mode: chat, work, or code".into(),
                }, ParamSpec {
                    name: "reason".into(),
                    typ: "string".into(),
                    required: false,
                    description: "Reason for switching".into(),
                }],
            },
            ActionSpec {
                name: "get_state".into(),
                description: "Get current surface state".into(),
                params: vec![],
            },
            ActionSpec {
                name: "get_mode_config".into(),
                description: "Get configuration for a specific mode".into(),
                params: vec![ParamSpec {
                    name: "mode".into(),
                    typ: "string".into(),
                    required: true,
                    description: "Mode to get config for".into(),
                }],
            },
            ActionSpec {
                name: "toggle_panel".into(),
                description: "Toggle panel visibility".into(),
                params: vec![ParamSpec {
                    name: "panel".into(),
                    typ: "string".into(),
                    required: true,
                    description: "Panel name: file_panel, terminal, browser, memory, skills, automations".into(),
                }],
            },
            ActionSpec {
                name: "get_sessions".into(),
                description: "Get sessions for current mode".into(),
                params: vec![],
            },
            ActionSpec {
                name: "get_mode_history".into(),
                description: "Get mode switch history".into(),
                params: vec![],
            },
            ActionSpec {
                name: "get_stats".into(),
                description: "Get surface statistics".into(),
                params: vec![],
            },
        ]
    }

    fn call(&self, action: &str, args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        match action {
            "switch_mode" => {
                let mode_str = args.get("mode")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_PARAMS".into(),
                        message: "Missing 'mode' parameter".into(),
                        recoverable: true,
                    })?;

                let mode = match mode_str.to_lowercase().as_str() {
                    "chat" => SurfaceMode::Chat,
                    "work" => SurfaceMode::Work,
                    "code" => SurfaceMode::Code,
                    _ => return Err(DomainError {
                        code: "INVALID_MODE".into(),
                        message: format!("Invalid mode: {}. Use chat, work, or code", mode_str),
                        recoverable: true,
                    }),
                };

                let reason = args.get("reason").and_then(|v| v.as_str()).map(|s| s.to_string());
                let state = self.switch_mode(mode, reason)?;
                Ok(serde_json::to_value(state).unwrap_or_default())
            }
            "get_state" => {
                let state = self.get_state()?;
                Ok(serde_json::to_value(state).unwrap_or_default())
            }
            "get_mode_config" => {
                let mode_str = args.get("mode")
                    .and_then(|v| v.as_str())
                    .unwrap_or("chat");

                let mode = match mode_str.to_lowercase().as_str() {
                    "chat" => SurfaceMode::Chat,
                    "work" => SurfaceMode::Work,
                    "code" => SurfaceMode::Code,
                    _ => SurfaceMode::Chat,
                };

                let state = self.get_state()?;
                if let Some(config) = state.modes.get(&mode) {
                    Ok(serde_json::to_value(config).unwrap_or_default())
                } else {
                    Err(DomainError {
                        code: "MODE_NOT_FOUND".into(),
                        message: format!("Mode '{}' not found", mode_str),
                        recoverable: true,
                    })
                }
            }
            "toggle_panel" => {
                let panel = args.get("panel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_PARAMS".into(),
                        message: "Missing 'panel' parameter".into(),
                        recoverable: true,
                    })?;

                let mut state = self.state.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: format!("Failed to lock state: {}", e),
                    recoverable: false,
                })?;

                match panel {
                    "file_panel" => state.panels.file_panel = !state.panels.file_panel,
                    "terminal" => state.panels.terminal = !state.panels.terminal,
                    "browser" => state.panels.browser = !state.panels.browser,
                    "memory" => state.panels.memory = !state.panels.memory,
                    "skills" => state.panels.skills = !state.panels.skills,
                    "automations" => state.panels.automations = !state.panels.automations,
                    _ => return Err(DomainError {
                        code: "INVALID_PANEL".into(),
                        message: format!("Invalid panel: {}", panel),
                        recoverable: true,
                    }),
                }

                Ok(serde_json::to_value(state.clone()).unwrap_or_default())
            }
            "get_sessions" => {
                // Simplified — would query session manager
                Ok(serde_json::json!({
                    "sessions": [],
                    "total": 0
                }))
            }
            "get_mode_history" => {
                let state = self.get_state()?;
                Ok(serde_json::to_value(state.mode_history).unwrap_or_default())
            }
            "get_stats" => {
                let state = self.get_state()?;
                let mut sessions_by_mode = HashMap::new();
                // Simplified stats
                sessions_by_mode.insert("chat".to_string(), 0);
                sessions_by_mode.insert("work".to_string(), 0);
                sessions_by_mode.insert("code".to_string(), 0);

                let stats = SurfaceStats {
                    total_sessions: 0,
                    sessions_by_mode,
                    total_mode_switches: state.mode_history.len(),
                    avg_session_duration_ms: 0.0,
                };
                Ok(serde_json::to_value(stats).unwrap_or_default())
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }

    fn init(&mut self) -> Result<(), DomainError> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), DomainError> {
        Ok(())
    }
}
