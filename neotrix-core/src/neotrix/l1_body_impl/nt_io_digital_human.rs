use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use crate::core::nt_core_self::affective_interface::{
    AffectiveInterface, AffectiveReadout, GuideMode, ResponseIntent,
};

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

/// 情绪微表情键 → 数字人 Emotion 枚举 (桥接 affective 表情键与动画枚举)。
fn emotion_from_expression(expression: &str) -> Emotion {
    match expression {
        "smile" => Emotion::Happy,
        "frown" => Emotion::Sad,
        "fury" => Emotion::Angry,
        "shock" => Emotion::Surprised,
        "tilt" => Emotion::Confused,
        "look_up" => Emotion::Thinking,
        _ => Emotion::Neutral,
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
    pub energy: f64,
    pub emotion: Emotion,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            engine: "edge".into(),
            voice: "zh-CN-XiaoxiaoNeural".into(),
            speed: 1.0,
            pitch: 1.0,
            energy: 0.5,
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

impl Default for EmotionEngine {
    fn default() -> Self {
        Self::new()
    }
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

impl Default for AvatarController {
    fn default() -> Self {
        Self::new()
    }
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
    /// 人类情感交互界面 — 感知用户情绪/关系阶段/共情策略, 驱动表情/韵律/回复意图。
    pub affective: AffectiveInterface,
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
            affective: AffectiveInterface::new(),
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
        // Legacy keyword engine kept for session_stats backward compat.
        self.emotion.detect_from_text(text);
        // Affective interface drives expression/rhythm/reply intent (mirror-then-guide).
        let readout = self
            .affective
            .process_user_input(text, None, GuideMode::Auto);
        let emotion = emotion_from_expression(&readout.expression);
        self.avatar.set_emotion(emotion);
        self.tts_config.emotion = emotion;
        self.tts_config.speed = readout.rhythm.voice_rate;
        self.tts_config.pitch = readout.rhythm.voice_pitch;
        self.tts_config.energy = readout.rhythm.voice_energy;
        let reply = self.reply_affective(text, &readout);
        PipelineResponse {
            reply: reply.clone(),
            emotion,
            animation: readout.expression.clone(),
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

    /// 情感化回复: backchannel + 意图模板 + 开放式追问 (共情对话实证)。
    fn reply_affective(&self, input: &str, readout: &AffectiveReadout) -> String {
        let mut out = String::new();
        if let Some(bc) = &readout.backchannel {
            out.push_str(bc);
            out.push(' ');
        }
        out.push_str(&self.intent_template(input, readout));
        if let Some(fq) = &readout.followup_question {
            out.push(' ');
            out.push_str(fq);
        }
        out
    }

    fn intent_template(&self, input: &str, readout: &AffectiveReadout) -> String {
        match readout.intent {
            ResponseIntent::Sympathizing => "我理解这让你很难受。谢谢你和我说这些。".to_string(),
            ResponseIntent::Consoling => "别担心，我会在这里陪着你。".to_string(),
            ResponseIntent::Acknowledging => "我听到了，这确实不容易。".to_string(),
            ResponseIntent::Encouraging => "这真的很棒，为你开心！".to_string(),
            ResponseIntent::Questioning => format!("嗯，我听到了：'{}'。", input),
            ResponseIntent::Agreeing => "没错，我也这么觉得。".to_string(),
            _ => self.generate_reply(input),
        }
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

    #[test]
    fn test_affective_drives_expression_rhythm() {
        let mut pipeline = DigitalHumanPipeline::new(PersonaConfig::default());
        pipeline.start_session();
        let resp = pipeline.process_audio_input("我很难过，真的很难受");
        assert_eq!(resp.emotion, Emotion::Sad);
        assert_eq!(resp.animation, "frown");
        assert!(pipeline.tts_config.speed < 1.0, "sad → slower rate");
        assert!(resp.reply.contains("我在听"));
        assert_eq!(pipeline.affective.relationship.interactions, 1);
    }

    #[test]
    fn test_affective_encouragement_reply() {
        let mut pipeline = DigitalHumanPipeline::new(PersonaConfig::default());
        let resp = pipeline.process_audio_input("太开心了，终于成功了");
        assert_eq!(resp.emotion, Emotion::Happy);
        assert_eq!(resp.animation, "smile");
        assert!(resp.reply.contains("开心"));
        assert!(pipeline.tts_config.energy >= 0.5);
    }

    #[test]
    fn test_emotion_from_expression() {
        assert_eq!(emotion_from_expression("fury"), Emotion::Angry);
        assert_eq!(emotion_from_expression("idle"), Emotion::Neutral);
        assert_eq!(emotion_from_expression("look_up"), Emotion::Thinking);
    }

    #[test]
    fn test_tts_energy_field() {
        let cfg = TtsConfig::default();
        assert!((cfg.energy - 0.5).abs() < 1e-9);
    }
}
