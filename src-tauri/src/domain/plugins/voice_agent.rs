//! # Voice Agent Domain Plugin
//!
//! Implements voice input/output for agents (ChatGPT/Claude pattern).
//! Users can speak naturally, interrupt, and coordinate tasks.
//!
//! # Architecture
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │         Voice Agent                          │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
//! │  │  STT     │  │  Voice   │  │  TTS     │  │
//! │  │  Engine  │  │  Router  │  │  Engine  │  │
//! │  └──────────┘  └──────────┘  └──────────┘  │
//! │       ↓              ↓              ↓        │
//! │  ┌──────────────────────────────────────┐   │
//! │  │      Voice Session Manager          │   │
//! │  └──────────────────────────────────────┘   │
//! └─────────────────────────────────────────────┘
//! ```

use crate::domain::app_handle::{get_app_handle, set_app_handle};
use crate::domain::{ActionSpec, DomainError, DomainPlugin, ParamSpec};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// ========== Types ==========

/// Voice session status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VoiceStatus {
    /// Voice session is idle
    Idle,
    /// Listening for user input
    Listening,
    /// Processing user input
    Processing,
    /// Speaking response
    Speaking,
    /// Voice session is paused
    Paused,
    /// Error occurred
    Error(String),
}

/// Voice session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSession {
    /// Session ID
    pub id: String,
    /// Current status
    pub status: VoiceStatus,
    /// Language code (e.g., "en-US")
    pub language: String,
    /// Whether voice is enabled
    pub enabled: bool,
    /// Whether auto-send is enabled (send after silence)
    pub auto_send: bool,
    /// Silence timeout in milliseconds
    pub silence_timeout_ms: u64,
    /// Volume level (0.0 - 1.0)
    pub volume: f32,
    /// Whether microphone is muted
    pub muted: bool,
    /// Current transcript
    pub transcript: Option<String>,
    /// Confidence score
    pub confidence: Option<f32>,
    /// Session started at
    pub started_at: String,
    /// Total listening time in milliseconds
    pub total_listening_ms: u64,
    /// Total speaking time in milliseconds
    pub total_speaking_ms: u64,
    /// Interruption count
    pub interruption_count: u32,
}

/// Voice configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    /// STT engine
    pub stt_engine: String,
    /// TTS engine
    pub tts_engine: String,
    /// Voice ID for TTS
    pub voice_id: Option<String>,
    /// Speech rate (0.5 - 2.0)
    pub speech_rate: f32,
    /// Pitch (0.5 - 2.0)
    pub pitch: f32,
    /// Volume (0.0 - 1.0)
    pub volume: f32,
    /// Enable noise reduction
    pub noise_reduction: bool,
    /// Enable echo cancellation
    pub echo_cancellation: bool,
    /// Enable voice activity detection
    pub vad_enabled: bool,
    /// VAD sensitivity (0.0 - 1.0)
    pub vad_sensitivity: f32,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            stt_engine: "whisper".to_string(),
            tts_engine: "system".to_string(),
            voice_id: None,
            speech_rate: 1.0,
            pitch: 1.0,
            volume: 0.8,
            noise_reduction: true,
            echo_cancellation: true,
            vad_enabled: true,
            vad_sensitivity: 0.5,
        }
    }
}

/// Voice command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCommand {
    /// Command ID
    pub id: String,
    /// Transcribed text
    pub text: String,
    /// Confidence
    pub confidence: f32,
    /// Parsed intent (if recognized)
    pub intent: Option<String>,
    /// Entities (if recognized)
    pub entities: HashMap<String, String>,
    /// Timestamp
    pub timestamp: String,
}

/// Voice statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub total_commands: usize,
    pub successful_commands: usize,
    pub failed_commands: usize,
    pub avg_confidence: f32,
    pub total_listening_ms: u64,
    pub total_speaking_ms: u64,
}

// ========== Plugin ==========

/// Voice Agent Domain Plugin
pub struct VoiceAgentPlugin {
    state: Arc<Mutex<VoiceState>>,
}

struct VoiceState {
    sessions: HashMap<String, VoiceSession>,
    config: VoiceConfig,
    commands: Vec<VoiceCommand>,
}

impl VoiceAgentPlugin {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(VoiceState {
                sessions: HashMap::new(),
                config: VoiceConfig::default(),
                commands: Vec::new(),
            })),
        }
    }

    /// Start a voice session
    async fn start_session(&self, language: Option<String>) -> Result<VoiceSession, DomainError> {
        let mut state = self.state.lock().await;

        let session_id = format!("voice-{}", uuid::Uuid::new_v4());
        let lang = language.unwrap_or_else(|| "en-US".to_string());

        let session = VoiceSession {
            id: session_id.clone(),
            status: VoiceStatus::Idle,
            language: lang,
            enabled: true,
            auto_send: true,
            silence_timeout_ms: 2000,
            volume: state.config.volume,
            muted: false,
            transcript: None,
            confidence: None,
            started_at: chrono::Utc::now().to_rfc3339(),
            total_listening_ms: 0,
            total_speaking_ms: 0,
            interruption_count: 0,
        };

        state.sessions.insert(session_id, session.clone());
        Ok(session)
    }

    /// Stop a voice session
    async fn stop_session(&self, session_id: &str) -> Result<(), DomainError> {
        let mut state = self.state.lock().await;
        state.sessions.remove(session_id);
        Ok(())
    }

    /// Start listening
    async fn start_listening(&self, session_id: &str) -> Result<VoiceSession, DomainError> {
        let mut state = self.state.lock().await;

        let session = state
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| DomainError {
                code: "SESSION_NOT_FOUND".into(),
                message: format!("Voice session '{}' not found", session_id),
                recoverable: true,
            })?;

        session.status = VoiceStatus::Listening;
        session.transcript = None;
        session.confidence = None;

        Ok(session.clone())
    }

    /// Stop listening
    async fn stop_listening(&self, session_id: &str) -> Result<VoiceSession, DomainError> {
        let mut state = self.state.lock().await;

        let session = state
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| DomainError {
                code: "SESSION_NOT_FOUND".into(),
                message: format!("Voice session '{}' not found", session_id),
                recoverable: true,
            })?;

        session.status = VoiceStatus::Idle;
        Ok(session.clone())
    }

    /// Process voice input (transcribe)
    async fn process_input(
        &self,
        session_id: &str,
        audio_data: Option<String>,
    ) -> Result<VoiceCommand, DomainError> {
        let mut state = self.state.lock().await;

        // Simplified — would use actual STT engine
        let command = VoiceCommand {
            id: format!("cmd-{}", uuid::Uuid::new_v4()),
            text: "Hello, how can I help you?".to_string(),
            confidence: 0.95,
            intent: Some("greeting".to_string()),
            entities: HashMap::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        state.commands.push(command.clone());

        // Update session
        if let Some(session) = state.sessions.get_mut(session_id) {
            session.transcript = Some(command.text.clone());
            session.confidence = Some(command.confidence);
            session.status = VoiceStatus::Processing;
        }

        Ok(command)
    }

    /// Speak response
    async fn speak(&self, session_id: &str, text: &str) -> Result<(), DomainError> {
        let mut state = self.state.lock().await;

        if let Some(session) = state.sessions.get_mut(session_id) {
            session.status = VoiceStatus::Speaking;
            // Simplified — would use actual TTS engine
        }

        Ok(())
    }

    /// Toggle mute
    async fn toggle_mute(&self, session_id: &str) -> Result<VoiceSession, DomainError> {
        let mut state = self.state.lock().await;

        let session = state
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| DomainError {
                code: "SESSION_NOT_FOUND".into(),
                message: format!("Voice session '{}' not found", session_id),
                recoverable: true,
            })?;

        session.muted = !session.muted;
        Ok(session.clone())
    }
}

impl Default for VoiceAgentPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DomainPlugin for VoiceAgentPlugin {
    fn name(&self) -> &str {
        "voice_agent"
    }

    fn description(&self) -> &str {
        "Voice input/output for agents (ChatGPT/Claude pattern)"
    }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec {
                name: "start_session".into(),
                description: "Start a new voice session".into(),
                params: vec![ParamSpec {
                    name: "language".into(),
                    r#type: "string".into(),
                    optional: true,
                    description: "Language code (e.g., en-US)".into(),
                }],
                ..Default::default()
            },
            ActionSpec {
                name: "stop_session".into(),
                description: "Stop a voice session".into(),
                params: vec![ParamSpec {
                    name: "session_id".into(),
                    r#type: "string".into(),
                    optional: false,
                    description: "Voice session ID".into(),
                }],
                ..Default::default()
            },
            ActionSpec {
                name: "start_listening".into(),
                description: "Start listening for voice input".into(),
                params: vec![ParamSpec {
                    name: "session_id".into(),
                    r#type: "string".into(),
                    optional: false,
                    description: "Voice session ID".into(),
                }],
                ..Default::default()
            },
            ActionSpec {
                name: "stop_listening".into(),
                description: "Stop listening for voice input".into(),
                params: vec![ParamSpec {
                    name: "session_id".into(),
                    r#type: "string".into(),
                    optional: false,
                    description: "Voice session ID".into(),
                }],
                ..Default::default()
            },
            ActionSpec {
                name: "process_input".into(),
                description: "Process voice input and transcribe".into(),
                params: vec![
                    ParamSpec {
                        name: "session_id".into(),
                        r#type: "string".into(),
                        optional: false,
                        description: "Voice session ID".into(),
                    },
                    ParamSpec {
                        name: "audio_data".into(),
                        r#type: "string".into(),
                        optional: true,
                        description: "Base64 encoded audio data".into(),
                    },
                ],
                ..Default::default()
            },
            ActionSpec {
                name: "speak".into(),
                description: "Speak a response using TTS".into(),
                params: vec![
                    ParamSpec {
                        name: "session_id".into(),
                        r#type: "string".into(),
                        optional: false,
                        description: "Voice session ID".into(),
                    },
                    ParamSpec {
                        name: "text".into(),
                        r#type: "string".into(),
                        optional: false,
                        description: "Text to speak".into(),
                    },
                ],
                ..Default::default()
            },
            ActionSpec {
                name: "toggle_mute".into(),
                description: "Toggle microphone mute".into(),
                params: vec![ParamSpec {
                    name: "session_id".into(),
                    r#type: "string".into(),
                    optional: false,
                    description: "Voice session ID".into(),
                }],
                ..Default::default()
            },
            ActionSpec {
                name: "get_stats".into(),
                description: "Get voice statistics".into(),
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
            "start_session" => {
                let language = args
                    .get("language")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let session = self.start_session(language).await?;
                Ok(serde_json::to_value(session).unwrap_or_default())
            }
            "stop_session" => {
                let session_id =
                    args.get("session_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_PARAMS".into(),
                            message: "Missing 'session_id' parameter".into(),
                            recoverable: true,
                        })?;
                self.stop_session(session_id).await?;
                Ok(serde_json::json!({"success": true}))
            }
            "start_listening" => {
                let session_id =
                    args.get("session_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_PARAMS".into(),
                            message: "Missing 'session_id' parameter".into(),
                            recoverable: true,
                        })?;
                let session = self.start_listening(session_id).await?;
                Ok(serde_json::to_value(session).unwrap_or_default())
            }
            "stop_listening" => {
                let session_id =
                    args.get("session_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_PARAMS".into(),
                            message: "Missing 'session_id' parameter".into(),
                            recoverable: true,
                        })?;
                let session = self.stop_listening(session_id).await?;
                Ok(serde_json::to_value(session).unwrap_or_default())
            }
            "process_input" => {
                let session_id =
                    args.get("session_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_PARAMS".into(),
                            message: "Missing 'session_id' parameter".into(),
                            recoverable: true,
                        })?;
                let audio_data = args
                    .get("audio_data")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let command = self.process_input(session_id, audio_data).await?;
                Ok(serde_json::to_value(command).unwrap_or_default())
            }
            "speak" => {
                let session_id =
                    args.get("session_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_PARAMS".into(),
                            message: "Missing 'session_id' parameter".into(),
                            recoverable: true,
                        })?;
                let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
                self.speak(session_id, text).await?;
                Ok(serde_json::json!({"success": true}))
            }
            "toggle_mute" => {
                let session_id =
                    args.get("session_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_PARAMS".into(),
                            message: "Missing 'session_id' parameter".into(),
                            recoverable: true,
                        })?;
                let session = self.toggle_mute(session_id).await?;
                Ok(serde_json::to_value(session).unwrap_or_default())
            }
            "get_stats" => {
                let state = self.state.lock().await;

                let total_sessions = state.sessions.len();
                let active = state
                    .sessions
                    .values()
                    .filter(|s| matches!(s.status, VoiceStatus::Listening | VoiceStatus::Speaking))
                    .count();
                let total_commands = state.commands.len();
                let successful = state.commands.iter().filter(|c| c.confidence > 0.5).count();

                let stats = VoiceStats {
                    total_sessions,
                    active_sessions: active,
                    total_commands,
                    successful_commands: successful,
                    failed_commands: total_commands - successful,
                    avg_confidence: if total_commands > 0 {
                        state.commands.iter().map(|c| c.confidence).sum::<f32>()
                            / total_commands as f32
                    } else {
                        0.0
                    },
                    total_listening_ms: state.sessions.values().map(|s| s.total_listening_ms).sum(),
                    total_speaking_ms: state.sessions.values().map(|s| s.total_speaking_ms).sum(),
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
