//! Real-time voice + expression pipeline -- inspired by Open-LLM-VTuber.

/// Voice synthesis backend
#[derive(Debug, Clone)]
pub enum VoiceBackend {
    Local,
    Api,
    Mock,
}

/// Expression type for avatar control
#[derive(Debug, Clone)]
pub enum ExpressionType {
    Neutral,
    Happy,
    Sad,
    Angry,
    Surprised,
    Thinking,
}

/// A frame in the VTuber pipeline
#[derive(Debug, Clone)]
pub struct VtuberFrame {
    pub text: String,
    pub audio_chunk: Option<Vec<u8>>,
    pub expression: ExpressionType,
    pub timestamp_ms: u64,
}

/// VTuber pipeline configuration
#[derive(Debug, Clone)]
pub struct VtuberConfig {
    pub voice_backend: VoiceBackend,
    pub sample_rate: u32,
    pub expression_model: String,
}

impl Default for VtuberConfig {
    fn default() -> Self {
        Self {
            voice_backend: VoiceBackend::Mock,
            sample_rate: 22050,
            expression_model: "default".into(),
        }
    }
}

/// The VTuber pipeline
pub struct VtuberPipeline {
    config: VtuberConfig,
}

impl VtuberPipeline {
    pub fn new(config: VtuberConfig) -> Self {
        Self { config }
    }

    pub fn process_text(&self, text: &str) -> VtuberFrame {
        VtuberFrame {
            text: text.into(),
            audio_chunk: None,
            expression: ExpressionType::Neutral,
            timestamp_ms: 0,
        }
    }
}

impl Default for VtuberPipeline {
    fn default() -> Self {
        Self::new(VtuberConfig::default())
    }
}
