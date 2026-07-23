use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Emotion {
    Neutral,
    Happy,
    Sad,
    Angry,
    Surprised,
    Confused,
    Thinking,
}

impl Emotion {
    pub fn animation_key(&self) -> &'static str {
        match self {
            Emotion::Neutral => "idle",
            Emotion::Happy => "smile",
            Emotion::Sad => "frown",
            Emotion::Angry => "fury",
            Emotion::Surprised => "shock",
            Emotion::Confused => "tilt",
            Emotion::Thinking => "look_up",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AsrConfig {
    pub engine: String,
    pub language: String,
    pub sample_rate: u32,
    pub streaming: bool,
    pub vad_enabled: bool,
}

impl Default for AsrConfig {
    fn default() -> Self {
        Self {
            engine: "funasr".into(),
            language: "zh".into(),
            sample_rate: 16000,
            streaming: true,
            vad_enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AsrResult {
    pub text: String,
    pub confidence: f64,
    pub is_final: bool,
    pub language: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct TtsConfig {
    pub engine: String,
    pub voice: String,
    pub speed: f64,
    pub pitch: f64,
    pub emotion: Emotion,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            engine: "edge".into(),
            voice: "zh-CN-XiaoxiaoNeural".into(),
            speed: 1.0,
            pitch: 1.0,
            emotion: Emotion::Neutral,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TtsResult {
    pub audio_data: Vec<u8>,
    pub duration_ms: u64,
    pub text: String,
    pub emotion: Emotion,
}

#[derive(Debug, Clone)]
pub struct PersonaConfig {
    pub name: String,
    pub description: String,
    pub knowledge_base: Vec<String>,
    pub qa_pairs: HashMap<String, String>,
    pub personality_traits: Vec<String>,
    pub wake_words: Vec<String>,
    pub interrupt_enabled: bool,
}

impl Default for PersonaConfig {
    fn default() -> Self {
        let mut qa = HashMap::new();
        qa.insert("你是谁".into(), "我是NeoTrix数字助手".into());
        qa.insert("hello".into(), "Hello! How can I help you?".into());
        Self {
            name: "Neo".into(),
            description: "AI Digital Assistant".into(),
            knowledge_base: vec!["general knowledge".into()],
            qa_pairs: qa,
            personality_traits: vec!["helpful".into(), "friendly".into()],
            wake_words: vec!["hey neo".into(), "neo".into()],
            interrupt_enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmotionEngine {
    current: Emotion,
    intensity: f64,
    history: VecDeque<(Emotion, Instant)>,
}

impl EmotionEngine {
    pub fn new() -> Self {
        Self {
            current: Emotion::Neutral,
            intensity: 0.5,
            history: VecDeque::new(),
        }
    }

    pub fn detect_from_text(&mut self, text: &str) -> Emotion {
        let lower = text.to_lowercase();
        let emotion = if lower.contains("happy") || lower.contains("great") || lower.contains("thank") {
            Emotion::Happy
        } else if lower.contains("sad") || lower.contains("sorry") || lower.contains("bad") {
            Emotion::Sad
        } else if lower.contains("angry") || lower.contains("mad") || lower.contains("furious") {
            Emotion::Angry
        } else if lower.contains("wow") || lower.contains("amazing") || lower.contains("unexpected") {
            Emotion::Surprised
        } else if lower.contains("hmm") || lower.contains("maybe") || lower.chars().any(|c| c == '?') {
            Emotion::Confused
        } else {
            Emotion::Neutral
        };
        self.current = emotion;
        self.history.push_back((emotion, Instant::now()));
        if self.history.len() > 100 {
            self.history.pop_front();
        }
        emotion
    }

    pub fn set_intensity(&mut self, intensity: f64) {
        self.intensity = intensity.max(0.0).min(1.0);
    }

    pub fn current_emotion(&self) -> Emotion {
        self.current
    }

    pub fn intensity(&self) -> f64 {
        self.intensity
    }
}

pub struct AvatarController {
    pub expression: Emotion,
    pub animation: String,
    pub lip_sync: bool,
    pub blink_interval_ms: u64,
    last_blink: Instant,
}

impl AvatarController {
    pub fn new() -> Self {
        Self {
            expression: Emotion::Neutral,
            animation: "idle".into(),
            lip_sync: true,
            blink_interval_ms: 4000,
            last_blink: Instant::now(),
        }
    }

    pub fn set_emotion(&mut self, emotion: Emotion) {
        self.expression = emotion;
        self.animation = emotion.animation_key().to_string();
    }

    pub fn should_blink(&mut self) -> bool {
        if self.last_blink.elapsed() > Duration::from_millis(self.blink_interval_ms) {
            self.last_blink = Instant::now();
            true
        } else {
            false
        }
    }

    pub fn get_state(&self) -> AvatarState {
        AvatarState {
            expression: self.expression,
            animation: self.animation.clone(),
            lip_sync: self.lip_sync,
            blinking: self.last_blink.elapsed() < Duration::from_millis(200),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AvatarState {
    pub expression: Emotion,
    pub animation: String,
    pub lip_sync: bool,
    pub blinking: bool,
}

pub struct DigitalHumanPipeline {
    pub persona: PersonaConfig,
    pub asr_config: AsrConfig,
    pub tts_config: TtsConfig,
    pub emotion: EmotionEngine,
    pub avatar: AvatarController,
    session_active: bool,
    session_start: Option<Instant>,
    utterance_count: u64,
}

impl DigitalHumanPipeline {
    pub fn new(persona: PersonaConfig) -> Self {
        Self {
            persona,
            asr_config: AsrConfig::default(),
            tts_config: TtsConfig::default(),
            emotion: EmotionEngine::new(),
            avatar: AvatarController::new(),
            session_active: false,
            session_start: None,
            utterance_count: 0,
        }
    }

    pub fn start_session(&mut self) {
        self.session_active = true;
        self.session_start = Some(Instant::now());
        self.utterance_count = 0;
    }

    pub fn end_session(&mut self) {
        self.session_active = false;
    }

    pub fn is_active(&self) -> bool {
        self.session_active
    }

    pub fn process_audio_input(&mut self, text: &str) -> PipelineResponse {
        self.utterance_count += 1;
        let emotion = self.emotion.detect_from_text(text);
        self.avatar.set_emotion(emotion);
        let reply = self.generate_reply(text);
        self.tts_config.emotion = emotion;
        PipelineResponse {
            reply: reply.clone(),
            emotion,
            animation: emotion.animation_key().to_string(),
            asr_confidence: 0.92,
            tts_text: reply,
            session_duration: self.session_start.map(|s| s.elapsed()).unwrap_or(Duration::ZERO),
        }
    }

    pub fn generate_reply(&self, input: &str) -> String {
        let trimmed = input.trim().to_lowercase();
        if let Some(answer) = self.persona.qa_pairs.get(&trimmed) {
            return answer.clone();
        }
        for (q, a) in &self.persona.qa_pairs {
            if trimmed.contains(&q.to_lowercase()) {
                return a.clone();
            }
        }
        format!("I heard: '{}'. Let me think about that...", input)
    }

    pub fn process_asr_result(&self, result: &AsrResult) -> String {
        if result.is_final {
            format!("ASR({}): {} [conf={:.2}]", result.language, result.text, result.confidence)
        } else {
            format!("ASR(partial): {}", result.text)
        }
    }

    pub fn session_stats(&self) -> SessionStats {
        SessionStats {
            active: self.session_active,
            utterance_count: self.utterance_count,
            duration: self.session_start.map(|s| s.elapsed()).unwrap_or(Duration::ZERO),
            current_emotion: self.emotion.current_emotion(),
            has_persona: !self.persona.name.is_empty(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PipelineResponse {
    pub reply: String,
    pub emotion: Emotion,
    pub animation: String,
    pub asr_confidence: f64,
    pub tts_text: String,
    pub session_duration: Duration,
}

#[derive(Debug, Clone)]
pub struct SessionStats {
    pub active: bool,
    pub utterance_count: u64,
    pub duration: Duration,
    pub current_emotion: Emotion,
    pub has_persona: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotion_detection() {
        let mut ee = EmotionEngine::new();
        assert_eq!(ee.detect_from_text("thank you very much"), Emotion::Happy);
        assert_eq!(ee.detect_from_text("I am so angry"), Emotion::Angry);
        assert_eq!(ee.detect_from_text("ordinary text"), Emotion::Neutral);
    }

    #[test]
    fn test_emotion_intensity_clamping() {
        let mut ee = EmotionEngine::new();
        ee.set_intensity(1.5);
        assert!((ee.intensity() - 1.0).abs() < 0.01);
        ee.set_intensity(-0.5);
        assert!((ee.intensity() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_persona_qa() {
        let pipeline = DigitalHumanPipeline::new(PersonaConfig::default());
        let reply = pipeline.generate_reply("你是谁");
        assert_eq!(reply, "我是NeoTrix数字助手");
        let fallback = pipeline.generate_reply("unknown text");
        assert!(fallback.contains("unknown text"));
    }

    #[test]
    fn test_session_lifecycle() {
        let mut pipeline = DigitalHumanPipeline::new(PersonaConfig::default());
        assert!(!pipeline.is_active());
        pipeline.start_session();
        assert!(pipeline.is_active());
        let resp = pipeline.process_audio_input("hello");
        assert_eq!(resp.emotion, Emotion::Neutral);
        pipeline.end_session();
        assert!(!pipeline.is_active());
    }

    #[test]
    fn test_avatar_emotion_mapping() {
        let mut avatar = AvatarController::new();
        assert_eq!(avatar.animation, "idle");
        avatar.set_emotion(Emotion::Happy);
        assert_eq!(avatar.animation, "smile");
        avatar.set_emotion(Emotion::Surprised);
        assert_eq!(avatar.animation, "shock");
    }

    #[test]
    fn test_emotion_animation_keys() {
        assert_eq!(Emotion::Neutral.animation_key(), "idle");
        assert_eq!(Emotion::Confused.animation_key(), "tilt");
        assert_eq!(Emotion::Thinking.animation_key(), "look_up");
    }

    #[test]
    fn test_asr_result_processing() {
        let pipeline = DigitalHumanPipeline::new(PersonaConfig::default());
        let result = AsrResult {
            text: "hello world".into(),
            confidence: 0.95,
            is_final: true,
            language: "en".into(),
            duration_ms: 1200,
        };
        let processed = pipeline.process_asr_result(&result);
        assert!(processed.contains("ASR"));
        assert!(processed.contains("0.95"));
    }

    #[test]
    fn test_session_stats() {
        let mut pipeline = DigitalHumanPipeline::new(PersonaConfig::default());
        pipeline.start_session();
        pipeline.process_audio_input("test");
        let stats = pipeline.session_stats();
        assert!(stats.active);
        assert_eq!(stats.utterance_count, 1);
    }
}
