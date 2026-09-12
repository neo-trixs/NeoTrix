//! L4 Emotion Layer Traits
//!
//! 情感层合约: 核心情感引擎 (nt_feel core)
//! 吸收来源: Open-LLM-VTuber (情绪识别, 角色人格, 语音交互)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// 统一 EmotionLabel 事实源: core::nt_core_self::emotion_state::EmotionLabel
pub use crate::core::nt_core_self::emotion_state::EmotionLabel;

/// 情绪信号 — 从文本/语音/视觉检测
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _EmotionSignal {
    pub label: EmotionLabel,
    pub intensity: f64, // 0.0 - 1.0
    pub source: _SignalSource,
    pub raw_input: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 信号来源
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum _SignalSource {
    Text,
    Voice,
    Visual,
    MultiModal,
}

/// 角色人格 — Open-LLM-VTuber 吸收
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _CharacterPersona {
    pub name: String,
    pub personality_traits: Vec<String>,
    pub response_style: _ResponseStyle,
    pub emotional_baseline: HashMap<EmotionLabel, f64>,
    pub voice_config: Option<_VoiceConfig>,
}

/// 响应风格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _ResponseStyle {
    pub formality: f64,  // 0.0 casual - 1.0 formal
    pub enthusiasm: f64, // 0.0 calm - 1.0 excited
    pub empathy: f64,    // 0.0 analytical - 1.0 empathetic
    pub humor: f64,      // 0.0 serious - 1.0 playful
}

/// 语音配置 — Open-LLM-VTuber 吸收
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _VoiceConfig {
    pub tts_provider: String,
    pub stt_provider: String,
    pub voice_id: Option<String>,
    pub language: String,
    pub speed: f64,
    pub pitch: f64,
}

/// 情感调节结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _RegulationResult {
    pub original: _EmotionSignal,
    pub regulated: _EmotionSignal,
    pub strategy: _RegulationStrategy,
    pub reason: String,
}

/// 调节策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum _RegulationStrategy {
    Amplify,
    Dampen,
    Redirect,
    Suppress,
    Express,
}

/// 情感层核心合约 (Open-LLM-VTuber 吸收)
pub(crate) trait _EmotionLayer: Send + Sync {
    /// 初始化情感层
    fn initialize(&mut self) -> Result<(), String>;

    /// 从文本检测情绪 (基础, FeelEngine 已有)
    fn detect_from_text(&self, text: &str) -> _EmotionSignal;

    /// 从语音检测情绪 (Open-LLM-VTuber 吸收)
    fn detect_from_voice(&self, audio: &[u8]) -> Result<_EmotionSignal, String>;

    /// 从视觉检测情绪 (Open-LLM-VTuber 吸收)
    fn detect_from_visual(&self, image: &[u8]) -> Result<_EmotionSignal, String>;

    /// 情绪调节 (FeelEngine 已有, 扩展)
    fn regulate(&self, signal: _EmotionSignal) -> _RegulationResult;

    /// 应用角色人格 (Open-LLM-VTuber 吸收)
    fn apply_persona(&self, signal: _EmotionSignal, persona: &_CharacterPersona) -> _EmotionSignal;

    /// TTS 输出 — 情绪驱动语音合成 (Open-LLM-VTuber 吸收)
    fn synthesize_speech(
        &self,
        text: &str,
        emotion: &_EmotionSignal,
        voice: &_VoiceConfig,
    ) -> Result<Vec<u8>, String>;

    /// STT 输入 — 语音转文本 (Open-LLM-VTuber 吸收)
    fn transcribe_speech(&self, audio: &[u8], language: &str) -> Result<String, String>;

    /// 获取情感状态快照
    fn snapshot(&self) -> _EmotionSnapshot;
}

/// 情感状态快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _EmotionSnapshot {
    pub current_emotion: EmotionLabel,
    pub intensity: f64,
    pub social_state: HashMap<String, f64>,
    pub attention_signals: Vec<String>,
    pub last_update: chrono::DateTime<chrono::Utc>,
}
