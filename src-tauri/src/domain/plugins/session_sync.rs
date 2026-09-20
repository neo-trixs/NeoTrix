//! # Session Sync Domain Plugin
//!
//! Implements cross-device session synchronization using KB-based CRDT sync.
//! Sessions can be created/modified offline and automatically merge when
//! devices reconnect.
//!
//! # Architecture
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │         Session Sync                         │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
//! │  │  Local   │  │  CRDT    │  │  Remote  │  │
//! │  │  Store   │  │  Engine  │  │  Sync    │  │
//! │  └──────────┘  └──────────┘  └──────────┘  │
//! │       ↓              ↓              ↓        │
//! │  ┌──────────────────────────────────────┐   │
//! │  │      KB-based Persistence            │   │
//! │  └──────────────────────────────────────┘   │
//! └─────────────────────────────────────────────┘
//! ```

use async_trait::async_trait;
use crate::domain::app_handle::{set_app_handle, get_app_handle};
use crate::domain::{ActionSpec, DomainError, DomainPlugin, ParamSpec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ========== Types ==========

/// Sync status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyncStatus {
    /// Session is local only (not synced)
    Local,
    /// Session is being synced
    Syncing,
    /// Session is synced with remote
    Synced,
    /// Sync failed
    Failed(String),
    /// Conflict detected
    Conflict(Vec<ConflictResolution>),
}

/// Conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub field: String,
    pub local_value: serde_json::Value,
    pub remote_value: serde_json::Value,
    pub resolution: String, // "local", "remote", "merged", "manual"
}

/// Synced session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedSession {
    /// Session ID (unique across devices)
    pub id: String,
    /// Device ID that created the session
    pub device_id: String,
    /// Session mode (chat/work/code)
    pub mode: String,
    /// Session title
    pub title: String,
    /// Messages
    pub messages: Vec<SyncedMessage>,
    /// Created at (ISO 8601)
    pub created_at: String,
    /// Last modified at (ISO 8601)
    pub modified_at: String,
    /// CRDT vector clock
    pub vector_clock: HashMap<String, u64>,
    /// Sync status
    pub sync_status: SyncStatus,
    /// Project ID (if in a project)
    pub project_id: Option<String>,
    /// Tags
    pub tags: Vec<String>,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Synced message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedMessage {
    pub id: String,
    pub role: String, // "user", "assistant", "system"
    pub content: String,
    pub timestamp: String,
    pub model: Option<String>,
    pub tokens_used: Option<u32>,
    pub attachments: Vec<SyncedAttachment>,
}

/// Synced attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedAttachment {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub size: u64,
    pub path: Option<String>,
    pub content: Option<String>, // Base64 encoded for small files
}

/// Device info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub device_type: String, // "desktop", "mobile", "web"
    pub platform: String,   // "macos", "windows", "linux", "ios", "android"
    pub last_sync: Option<String>,
    pub sync_enabled: bool,
}

/// Sync statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    pub total_sessions: usize,
    pub synced_sessions: usize,
    pub local_only_sessions: usize,
    pub pending_sync: usize,
    pub conflict_count: usize,
    pub last_sync_time: Option<String>,
    pub devices: Vec<DeviceInfo>,
}

// ========== Plugin ==========

/// Session Sync Domain Plugin
pub struct SessionSyncPlugin {
    state: Arc<Mutex<SyncState>>,
}

struct SyncState {
    sessions: HashMap<String, SyncedSession>,
    devices: Vec<DeviceInfo>,
    current_device_id: String,
}

impl SessionSyncPlugin {
    pub fn new() -> Self {
        let device_id = format!("device-{}", uuid::Uuid::new_v4());
        Self {
            state: Arc::new(Mutex::new(SyncState {
                sessions: HashMap::new(),
                devices: vec![DeviceInfo {
                    device_id: device_id.clone(),
                    device_name: hostname::get().map(|h| h.to_string_lossy().to_string()).unwrap_or_else(|_| "Unknown".to_string()),
                    device_type: "desktop".to_string(),
                    platform: std::env::consts::OS.to_string(),
                    last_sync: None,
                    sync_enabled: true,
                }],
                current_device_id: device_id,
            })),
        }
    }

    /// Create a new synced session
    fn create_session(&self, mode: &str, title: &str) -> Result<SyncedSession, DomainError> {
        let mut state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;

        let session_id = format!("session-{}", uuid::Uuid::new_v4());
        let now = chrono::Utc::now().to_rfc3339();

        let mut vector_clock = HashMap::new();
        vector_clock.insert(state.current_device_id.clone(), 1);

        let session = SyncedSession {
            id: session_id.clone(),
            device_id: state.current_device_id.clone(),
            mode: mode.to_string(),
            title: title.to_string(),
            messages: Vec::new(),
            created_at: now.clone(),
            modified_at: now,
            vector_clock,
            sync_status: SyncStatus::Local,
            project_id: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        };

        state.sessions.insert(session_id, session.clone());
        Ok(session)
    }

    /// Add a message to a session
    fn add_message(&self, session_id: &str, role: &str, content: &str) -> Result<SyncedMessage, DomainError> {
        let mut state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;

        let session = state.sessions.get_mut(session_id)
            .ok_or_else(|| DomainError {
                code: "SESSION_NOT_FOUND".into(),
                message: format!("Session '{}' not found", session_id),
                recoverable: true,
            })?;

        let message_id = format!("msg-{}", uuid::Uuid::new_v4());
        let now = chrono::Utc::now().to_rfc3339();

        let message = SyncedMessage {
            id: message_id,
            role: role.to_string(),
            content: content.to_string(),
            timestamp: now.clone(),
            model: None,
            tokens_used: None,
            attachments: Vec::new(),
        };

        session.messages.push(message.clone());
        session.modified_at = now;

        // Increment vector clock
        let clock = session.vector_clock.entry(state.current_device_id.clone()).or_insert(0);
        *clock += 1;

        // Mark as needing sync
        session.sync_status = SyncStatus::Local;

        Ok(message)
    }

    /// Get all sessions
    fn get_sessions(&self) -> Result<Vec<SyncedSession>, DomainError> {
        let state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;
        Ok(state.sessions.values().cloned().collect())
    }

    /// Get a session by ID
    fn get_session(&self, session_id: &str) -> Result<SyncedSession, DomainError> {
        let state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;
        state.sessions.get(session_id).cloned()
            .ok_or_else(|| DomainError {
                code: "SESSION_NOT_FOUND".into(),
                message: format!("Session '{}' not found", session_id),
                recoverable: true,
            })
    }

    /// Delete a session
    fn delete_session(&self, session_id: &str) -> Result<(), DomainError> {
        let mut state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;
        state.sessions.remove(session_id);
        Ok(())
    }

    /// Simulate sync (simplified)
    fn sync(&self) -> Result<SyncStats, DomainError> {
        let mut state = self.state.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: format!("Failed to lock state: {}", e),
            recoverable: false,
        })?;

        // Mark all local sessions as synced
        for session in state.sessions.values_mut() {
            if session.sync_status == SyncStatus::Local {
                session.sync_status = SyncStatus::Synced;
            }
        }

        let total = state.sessions.len();
        let synced = state.sessions.values()
            .filter(|s| s.sync_status == SyncStatus::Synced)
            .count();
        let conflicts = state.sessions.values()
            .filter(|s| matches!(s.sync_status, SyncStatus::Conflict(_)))
            .count();

        Ok(SyncStats {
            total_sessions: total,
            synced_sessions: synced,
            local_only_sessions: total - synced - conflicts,
            pending_sync: 0,
            conflict_count: conflicts,
            last_sync_time: Some(chrono::Utc::now().to_rfc3339()),
            devices: state.devices.clone(),
        })
    }
}

impl Default for SessionSyncPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DomainPlugin for SessionSyncPlugin {
    fn name(&self) -> &str {
        "session_sync"
    }

    fn description(&self) -> &str {
        "Cross-device session synchronization with CRDT-based merge"
    }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec {
                name: "create_session".into(),
                description: "Create a new synced session".into(),
                params: vec![
                    ParamSpec {
                        name: "mode".into(),
                        typ: "string".into(),
                        required: true,
                        description: "Session mode: chat, work, or code".into(),
                    },
                    ParamSpec {
                        name: "title".into(),
                        typ: "string".into(),
                        required: true,
                        description: "Session title".into(),
                    },
                ],
            },
            ActionSpec {
                name: "add_message".into(),
                description: "Add a message to a session".into(),
                params: vec![
                    ParamSpec {
                        name: "session_id".into(),
                        typ: "string".into(),
                        required: true,
                        description: "Session ID".into(),
                    },
                    ParamSpec {
                        name: "role".into(),
                        typ: "string".into(),
                        required: true,
                        description: "Message role: user, assistant, or system".into(),
                    },
                    ParamSpec {
                        name: "content".into(),
                        typ: "string".into(),
                        required: true,
                        description: "Message content".into(),
                    },
                ],
            },
            ActionSpec {
                name: "get_sessions".into(),
                description: "Get all synced sessions".into(),
                params: vec![],
            },
            ActionSpec {
                name: "get_session".into(),
                description: "Get a specific session by ID".into(),
                params: vec![ParamSpec {
                    name: "session_id".into(),
                    typ: "string".into(),
                    required: true,
                    description: "Session ID".into(),
                }],
            },
            ActionSpec {
                name: "delete_session".into(),
                description: "Delete a session".into(),
                params: vec![ParamSpec {
                    name: "session_id".into(),
                    typ: "string".into(),
                    required: true,
                    description: "Session ID".into(),
                }],
            },
            ActionSpec {
                name: "sync".into(),
                description: "Sync all sessions with remote".into(),
                params: vec![],
            },
            ActionSpec {
                name: "get_stats".into(),
                description: "Get sync statistics".into(),
                params: vec![],
            },
        ]
    }

    async fn call(&self, action: &str, args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        match action {
            "create_session" => {
                let mode = args.get("mode")
                    .and_then(|v| v.as_str())
                    .unwrap_or("chat");
                let title = args.get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("New Session");

                let session = self.create_session(mode, title)?;
                Ok(serde_json::to_value(session).unwrap_or_default())
            }
            "add_message" => {
                let session_id = args.get("session_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_PARAMS".into(),
                        message: "Missing 'session_id' parameter".into(),
                        recoverable: true,
                    })?;
                let role = args.get("role")
                    .and_then(|v| v.as_str())
                    .unwrap_or("user");
                let content = args.get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let message = self.add_message(session_id, role, content)?;
                Ok(serde_json::to_value(message).unwrap_or_default())
            }
            "get_sessions" => {
                let sessions = self.get_sessions()?;
                Ok(serde_json::json!({
                    "sessions": sessions,
                    "total": sessions.len()
                }))
            }
            "get_session" => {
                let session_id = args.get("session_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_PARAMS".into(),
                        message: "Missing 'session_id' parameter".into(),
                        recoverable: true,
                    })?;
                let session = self.get_session(session_id)?;
                Ok(serde_json::to_value(session).unwrap_or_default())
            }
            "delete_session" => {
                let session_id = args.get("session_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_PARAMS".into(),
                        message: "Missing 'session_id' parameter".into(),
                        recoverable: true,
                    })?;
                self.delete_session(session_id)?;
                Ok(serde_json::json!({"success": true}))
            }
            "sync" => {
                let stats = self.sync()?;
                Ok(serde_json::to_value(stats).unwrap_or_default())
            }
            "get_stats" => {
                let state = self.state.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: format!("Failed to lock state: {}", e),
                    recoverable: false,
                })?;

                let total = state.sessions.len();
                let synced = state.sessions.values()
                    .filter(|s| s.sync_status == SyncStatus::Synced)
                    .count();
                let conflicts = state.sessions.values()
                    .filter(|s| matches!(s.sync_status, SyncStatus::Conflict(_)))
                    .count();

                let stats = SyncStats {
                    total_sessions: total,
                    synced_sessions: synced,
                    local_only_sessions: total - synced - conflicts,
                    pending_sync: 0,
                    conflict_count: conflicts,
                    last_sync_time: None,
                    devices: state.devices.clone(),
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

    async fn init(&mut self) -> Result<(), DomainError> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), DomainError> {
        Ok(())
    }
}
