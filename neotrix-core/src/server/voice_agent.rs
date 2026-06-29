use serde::{Deserialize, Serialize};

/// Supported TTS providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TtsProvider {
    ElevenLabs,
    Deepgram,
    OpenAi,
    Doubao,
}

impl TtsProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            TtsProvider::ElevenLabs => "elevenlabs",
            TtsProvider::Deepgram => "deepgram",
            TtsProvider::OpenAi => "openai",
            TtsProvider::Doubao => "doubao",
        }
    }
}

/// Supported ASR providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AsrProvider {
    Deepgram,
    Whisper,
    Doubao,
}

impl AsrProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            AsrProvider::Deepgram => "deepgram",
            AsrProvider::Whisper => "whisper",
            AsrProvider::Doubao => "doubao",
        }
    }
}

/// Voice agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceAgentConfig {
    pub tts_provider: TtsProvider,
    pub asr_provider: AsrProvider,
    pub nt_act_voice_id: String,
    pub welcome_message: Option<String>,
    pub interruption_supported: bool,
    pub vad_threshold: f64,
    pub sample_rate: u32,
}

impl Default for VoiceAgentConfig {
    fn default() -> Self {
        Self {
            tts_provider: TtsProvider::OpenAi,
            asr_provider: AsrProvider::Whisper,
            nt_act_voice_id: "default".into(),
            welcome_message: Some("Hello, I'm your AI nt_act_voice agent.".into()),
            interruption_supported: true,
            vad_threshold: 0.5,
            sample_rate: 16000,
        }
    }
}

/// Voice session state
#[derive(Debug, Clone, PartialEq)]
pub enum VoiceSessionState {
    Idle,
    Listening,
    Processing,
    Speaking,
    Paused,
}

/// A single nt_act_voice interaction turn
#[derive(Debug, Clone)]
pub struct VoiceTurn {
    pub turn_id: String,
    pub user_transcript: String,
    pub agent_response: String,
    pub audio_input_duration_ms: u64,
    pub audio_output_duration_ms: u64,
    pub started_at: u64,
    pub ended_at: u64,
}

/// Voice session tracking
pub struct VoiceSession {
    pub session_id: String,
    pub config: VoiceAgentConfig,
    pub state: VoiceSessionState,
    pub turns: Vec<VoiceTurn>,
    pub start_time: u64,
    pub persona_id: Option<String>,
}

impl VoiceSession {
    pub fn new(config: VoiceAgentConfig) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            config,
            state: VoiceSessionState::Idle,
            turns: Vec::new(),
            start_time: now,
            persona_id: None,
        }
    }

    pub fn with_persona(mut self, persona_id: &str) -> Self {
        self.persona_id = Some(persona_id.to_string());
        self
    }

    pub fn transition_to(&mut self, new_state: VoiceSessionState) {
        self.state = new_state;
    }

    pub fn record_turn(
        &mut self,
        user_text: &str,
        agent_text: &str,
        audio_in_ms: u64,
        audio_out_ms: u64,
    ) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.turns.push(VoiceTurn {
            turn_id: uuid::Uuid::new_v4().to_string(),
            user_transcript: user_text.to_string(),
            agent_response: agent_text.to_string(),
            audio_input_duration_ms: audio_in_ms,
            audio_output_duration_ms: audio_out_ms,
            started_at: now,
            ended_at: now,
        });
    }

    pub fn turn_count(&self) -> usize {
        self.turns.len()
    }

    pub fn total_audio_duration_ms(&self) -> u64 {
        self.turns
            .iter()
            .map(|t| t.audio_input_duration_ms + t.audio_output_duration_ms)
            .sum()
    }
}

/// Voice agent manager
pub struct VoiceAgentManager {
    pub sessions: Vec<VoiceSession>,
    pub default_config: VoiceAgentConfig,
}

impl VoiceAgentManager {
    pub fn new(config: VoiceAgentConfig) -> Self {
        Self {
            sessions: Vec::new(),
            default_config: config,
        }
    }

    pub fn create_session(
        &mut self,
        config_override: Option<VoiceAgentConfig>,
    ) -> &mut VoiceSession {
        let config = config_override.unwrap_or_else(|| self.default_config.clone());
        let session = VoiceSession::new(config);
        self.sessions.push(session);
        self.sessions.last_mut().expect("session was just pushed but pop returned None - logic error in session stack")
    }

    pub fn get_session(&mut self, session_id: &str) -> Option<&mut VoiceSession> {
        self.sessions
            .iter_mut()
            .find(|s| s.session_id == session_id)
    }

    pub fn active_sessions(&self) -> Vec<&VoiceSession> {
        self.sessions
            .iter()
            .filter(|s| s.state != VoiceSessionState::Idle)
            .collect()
    }

    pub fn remove_session(&mut self, session_id: &str) {
        self.sessions.retain(|s| s.session_id != session_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nt_act_voice_agent_config_default() {
        let config = VoiceAgentConfig::default();
        assert_eq!(config.tts_provider, TtsProvider::OpenAi);
        assert!(config.interruption_supported);
        assert_eq!(config.sample_rate, 16000);
    }

    #[test]
    fn test_nt_act_voice_session_lifecycle() {
        let mut session = VoiceSession::new(VoiceAgentConfig::default());
        assert_eq!(session.state, VoiceSessionState::Idle);
        session.transition_to(VoiceSessionState::Listening);
        assert_eq!(session.state, VoiceSessionState::Listening);
        session.transition_to(VoiceSessionState::Speaking);
        assert_eq!(session.state, VoiceSessionState::Speaking);
    }

    #[test]
    fn test_record_turn() {
        let mut session = VoiceSession::new(VoiceAgentConfig::default());
        session.record_turn("Hello", "Hi there!", 1000, 2000);
        assert_eq!(session.turn_count(), 1);
        assert_eq!(session.total_audio_duration_ms(), 3000);
    }

    #[test]
    fn test_nt_act_voice_agent_manager() {
        let mut manager = VoiceAgentManager::new(VoiceAgentConfig::default());
        let session = manager.create_session(None);
        assert_eq!(session.state, VoiceSessionState::Idle);
        assert_eq!(manager.sessions.len(), 1);
    }

    #[test]
    fn test_session_persona() {
        let session = VoiceSession::new(VoiceAgentConfig::default()).with_persona("alice");
        assert_eq!(session.persona_id, Some("alice".into()));
    }

    #[test]
    fn test_tts_provider_as_str() {
        assert_eq!(TtsProvider::ElevenLabs.as_str(), "elevenlabs");
        assert_eq!(TtsProvider::Deepgram.as_str(), "deepgram");
    }

    #[test]
    fn test_asr_provider_as_str() {
        assert_eq!(AsrProvider::Deepgram.as_str(), "deepgram");
        assert_eq!(AsrProvider::Whisper.as_str(), "whisper");
    }
}
