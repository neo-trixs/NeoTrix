//! Voice Mode commands — inspired by Claude Code's hold-spacebar-for-voice
//!
//! Simulated speech-to-text pipeline with pluggable backend (mock/whisper/api).
//! Supports 10 languages, push-to-talk, voice session management,
//! transcription-to-command mapping, and TTS synthesis.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

// ── Types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VoiceLanguage {
    Zh,
    En,
    Ja,
    Ko,
    Fr,
    De,
    Es,
    Pt,
    Ru,
    Ar,
}

impl VoiceLanguage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Zh => "zh",
            Self::En => "en",
            Self::Ja => "ja",
            Self::Ko => "ko",
            Self::Fr => "fr",
            Self::De => "de",
            Self::Es => "es",
            Self::Pt => "pt",
            Self::Ru => "ru",
            Self::Ar => "ar",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "zh" => Self::Zh,
            "ja" => Self::Ja,
            "ko" => Self::Ko,
            "fr" => Self::Fr,
            "de" => Self::De,
            "es" => Self::Es,
            "pt" => Self::Pt,
            "ru" => Self::Ru,
            "ar" => Self::Ar,
            _ => Self::En,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VoiceStatus {
    Idle,
    Listening,
    Processing,
    Transcribing,
    Speaking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceTranscript {
    pub id: String,
    pub text: String,
    pub language: VoiceLanguage,
    pub confidence: f64,
    pub is_final: bool,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCommand {
    pub transcript_id: String,
    pub raw_text: String,
    pub interpreted_action: String,
    pub confidence: f64,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub enabled: bool,
    pub language: VoiceLanguage,
    pub auto_submit: bool,
    pub wake_word: String,
    pub push_to_talk: bool,
    pub stt_backend: String,
    pub tts_enabled: bool,
    pub tts_voice: String,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            language: VoiceLanguage::En,
            auto_submit: false,
            wake_word: "hey neotrix".into(),
            push_to_talk: true,
            stt_backend: "mock".into(),
            tts_enabled: false,
            tts_voice: "default".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSession {
    pub id: String,
    pub status: VoiceStatus,
    pub started_at: u64,
    pub commands_executed: u32,
    pub duration_secs: u64,
    pub language: VoiceLanguage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceStats {
    pub total_sessions: u32,
    pub commands_executed: u32,
    pub avg_confidence: f64,
    pub top_languages: Vec<(String, u32)>,
    pub top_actions: Vec<(String, u32)>,
}

// ── State ────────────────────────────────────────────────────────────

struct VoiceState {
    sessions: Vec<VoiceSession>,
    transcripts: Vec<VoiceTranscript>,
    commands_log: Vec<(String, String, f64)>, // action, language, confidence
    config: VoiceConfig,
    next_session_num: u32,
    next_transcript_num: u32,
}

impl VoiceState {
    fn new() -> Self {
        Self {
            sessions: Vec::with_capacity(20),
            transcripts: Vec::with_capacity(1000),
            commands_log: Vec::new(),
            config: VoiceConfig::default(),
            next_session_num: 1,
            next_transcript_num: 1,
        }
    }
}

static STATE: LazyLock<Mutex<VoiceState>> = LazyLock::new(|| Mutex::new(VoiceState::new()));

// ── Helpers ──────────────────────────────────────────────────────────

fn short_uid() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let entropy = (now % 99999) as u32;
    format!("{:05x}", entropy)
}

fn timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Generate a realistic fake transcript for the given language.
fn mock_transcript_for_language(lang: &VoiceLanguage) -> &'static str {
    match lang {
        VoiceLanguage::Zh => "打开文件并运行测试",
        VoiceLanguage::En => "open the file and run the tests",
        VoiceLanguage::Ja => "ファイルを開いてテストを実行",
        VoiceLanguage::Ko => "파일을 열고 테스트를 실행",
        VoiceLanguage::Fr => "ouvrir le fichier et exécuter les tests",
        VoiceLanguage::De => "Datei öffnen und Tests ausführen",
        VoiceLanguage::Es => "abrir el archivo y ejecutar las pruebas",
        VoiceLanguage::Pt => "abrir o arquivo e executar os testes",
        VoiceLanguage::Ru => "открыть файл и запустить тесты",
        VoiceLanguage::Ar => "فتح الملف وتشغيل الاختبارات",
    }
}

/// Map a transcript to an action via keyword matching.
fn interpret_action(text: &str) -> String {
    let lower = text.to_lowercase();
    if lower.contains("open file") || lower.contains("打开文件") {
        "file_open".into()
    } else if lower.contains("search") || lower.contains("搜索") {
        "search".into()
    } else if lower.contains("run test") || lower.contains("运行测试") {
        "run_tests".into()
    } else if lower.contains("deploy") || lower.contains("部署") {
        "deploy".into()
    } else if lower.contains("commit") || lower.contains("提交") {
        "git_commit".into()
    } else {
        "unknown".into()
    }
}

// ── Commands ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn voice_start_session(language: Option<String>) -> Result<String, String> {
    let lang = match language {
        Some(l) => VoiceLanguage::from_str(&l),
        None => {
            let state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;
            state.config.language.clone()
        }
    };

    let mut state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;

    if state.sessions.len() >= 20 {
        return Err("maximum 20 sessions reached".into());
    }

    let id = format!("voice-{}", short_uid());
    let session = VoiceSession {
        id: id.clone(),
        status: VoiceStatus::Listening,
        started_at: timestamp_secs(),
        commands_executed: 0,
        duration_secs: 0,
        language: lang,
    };

    state.sessions.push(session);
    state.next_session_num += 1;

    Ok(id)
}

#[tauri::command]
pub fn voice_stop_session(session_id: String) -> Result<VoiceSession, String> {
    let mut state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;

    let session = state
        .sessions
        .iter_mut()
        .find(|s| s.id == session_id)
        .ok_or_else(|| format!("session not found: {}", session_id))?;

    session.status = VoiceStatus::Idle;
    session.duration_secs = timestamp_secs().saturating_sub(session.started_at);

    Ok(session.clone())
}

#[tauri::command]
pub fn voice_session_status(session_id: String) -> Result<VoiceSession, String> {
    let state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;

    state
        .sessions
        .iter()
        .find(|s| s.id == session_id)
        .cloned()
        .ok_or_else(|| format!("session not found: {}", session_id))
}

#[tauri::command]
pub fn voice_send_audio(
    session_id: String,
    audio_data: String,
) -> Result<VoiceTranscript, String> {
    let mut state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;

    let session_idx = state
        .sessions
        .iter()
        .position(|s| s.id == session_id)
        .ok_or_else(|| format!("session not found: {}", session_id))?;

    state.sessions[session_idx].status = VoiceStatus::Processing;
    let lang = state.sessions[session_idx].language.clone();

    let text = if audio_data.is_empty() {
        String::new()
    } else {
        mock_transcript_for_language(&lang).to_string()
    };

    let num = state.next_transcript_num;
    state.next_transcript_num += 1;

    let is_final = state.transcripts.len() % 3 == 2;
    let confidence = if is_final {
        0.92
    } else {
        0.65 + (state.transcripts.len() as f64 % 3.0) * 0.15
    };

    let transcript = VoiceTranscript {
        id: format!("tr-{:05x}", num),
        text,
        language: lang,
        confidence: (confidence * 100.0).round() / 100.0,
        is_final,
        timestamp: timestamp_secs(),
    };

    if state.transcripts.len() >= 1000 {
        state.transcripts.remove(0);
    }
    state.transcripts.push(transcript.clone());

    state.sessions[session_idx].status = VoiceStatus::Listening;

    Ok(transcript)
}

#[tauri::command]
pub fn voice_get_transcription(
    audio_data: String,
    language: Option<String>,
    model: Option<String>,
) -> Result<VoiceTranscript, String> {
    #[allow(unused_variables)]
    let _ = &model;
    let lang = match language {
        Some(l) => VoiceLanguage::from_str(&l),
        None => VoiceLanguage::En,
    };

    let mut state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;

    let text = if audio_data.is_empty() {
        String::new()
    } else {
        mock_transcript_for_language(&lang).to_string()
    };

    let num = state.next_transcript_num;
    state.next_transcript_num += 1;

    let transcript = VoiceTranscript {
        id: format!("tr-{:05x}", num),
        text,
        language: lang,
        confidence: 0.95,
        is_final: true,
        timestamp: timestamp_secs(),
    };

    if state.transcripts.len() >= 1000 {
        state.transcripts.remove(0);
    }
    state.transcripts.push(transcript.clone());

    Ok(transcript)
}

#[tauri::command]
pub fn voice_list_sessions() -> Result<Vec<VoiceSession>, String> {
    let state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;
    Ok(state.sessions.clone())
}

#[tauri::command]
pub fn voice_session_history(session_id: String) -> Result<Vec<VoiceTranscript>, String> {
    let state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;
    let session = state
        .sessions
        .iter()
        .find(|s| s.id == session_id)
        .ok_or_else(|| format!("session not found: {}", session_id))?;

    let session_start = session.started_at;
    let mut history: Vec<VoiceTranscript> = state
        .transcripts
        .iter()
        .filter(|t| t.timestamp >= session_start)
        .cloned()
        .collect();

    if history.len() > 500 {
        history = history[history.len() - 500..].to_vec();
    }

    Ok(history)
}

#[tauri::command]
pub fn voice_synthesize(text: String, voice: Option<String>) -> Result<String, String> {
    // Mock — in production would return base64 audio data
    log::info!(
        "[voice] TTS: text={}, voice={}",
        text,
        voice.unwrap_or_else(|| "default".into())
    );
    Ok("ok".into())
}

#[tauri::command]
pub fn voice_config() -> Result<VoiceConfig, String> {
    let state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;
    Ok(state.config.clone())
}

#[tauri::command]
pub fn voice_set_config(config: VoiceConfig) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;
    state.config = config;
    Ok(())
}

#[tauri::command]
pub fn voice_test_microphone() -> Result<bool, String> {
    // Mock — always returns true
    Ok(true)
}

#[tauri::command]
pub fn voice_stats() -> Result<VoiceStats, String> {
    let state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;

    let total_sessions = state.sessions.len() as u32;
    let commands_executed: u32 = state.sessions.iter().map(|s| s.commands_executed).sum();

    let avg_confidence = if state.transcripts.is_empty() {
        0.0
    } else {
        let sum: f64 = state.transcripts.iter().map(|t| t.confidence).sum();
        (sum / state.transcripts.len() as f64 * 100.0).round() / 100.0
    };

    let mut lang_map: HashMap<String, u32> = HashMap::new();
    for t in &state.transcripts {
        *lang_map.entry(t.language.as_str().to_string()).or_insert(0) += 1;
    }
    let mut top_languages: Vec<(String, u32)> = lang_map.into_iter().collect();
    top_languages.sort_by(|a, b| b.1.cmp(&a.1));
    top_languages.truncate(5);

    let mut action_map: HashMap<String, u32> = HashMap::new();
    for (action, _, _) in &state.commands_log {
        *action_map.entry(action.clone()).or_insert(0) += 1;
    }
    let mut top_actions: Vec<(String, u32)> = action_map.into_iter().collect();
    top_actions.sort_by(|a, b| b.1.cmp(&a.1));
    top_actions.truncate(5);

    Ok(VoiceStats {
        total_sessions,
        commands_executed,
        avg_confidence,
        top_languages,
        top_actions,
    })
}

#[tauri::command]
pub fn voice_execute_command(transcript: String) -> Result<String, String> {
    let action = interpret_action(&transcript);

    let mut state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;

    state.commands_log.push((
        action.clone(),
        "en".into(),
        0.9,
    ));

    // Update the most recent session's command count
    if let Some(session) = state.sessions.last_mut() {
        session.commands_executed += 1;
    }

    log::info!("[voice] action={} from transcript=\"{}\"", action, transcript);

    Ok(action)
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_start_and_stop_session() {
        let id = voice_start_session(Some("zh".into())).unwrap();
        assert!(id.starts_with("voice-"));

        let session = voice_stop_session(id.clone()).unwrap();
        assert_eq!(session.status, VoiceStatus::Idle);
        assert_eq!(session.language, VoiceLanguage::Zh);
    }

    #[test]
    fn test_voice_send_audio_and_get_transcript() {
        let id = voice_start_session(None).unwrap();
        let transcript = voice_send_audio(id, "fake-base64-data".into()).unwrap();
        assert!(transcript.id.starts_with("tr-"));
        assert!(!transcript.text.is_empty());
        assert!(transcript.confidence > 0.0);
    }

    #[test]
    fn test_voice_get_direct_transcription() {
        let result = voice_get_transcription(
            "audio-data".into(),
            Some("fr".into()),
            Some("mock".into()),
        )
        .unwrap();
        assert_eq!(result.language, VoiceLanguage::Fr);
        assert!(result.is_final);
    }

    #[test]
    fn test_voice_execute_command_mapping() {
        let result = voice_execute_command("open file main.rs".into()).unwrap();
        assert_eq!(result, "file_open");

        let result = voice_execute_command("运行测试".into()).unwrap();
        assert_eq!(result, "run_tests");

        let result = voice_execute_command("deploy to production".into()).unwrap();
        assert_eq!(result, "deploy");

        let result = voice_execute_command("commit changes".into()).unwrap();
        assert_eq!(result, "git_commit");

        let result = voice_execute_command("random text".into()).unwrap();
        assert_eq!(result, "unknown");
    }
}
