//! # Affective Interface — 人类情感交互界面
//!
//! 面向人类的双向情感引擎: 感知用户情绪 (文本双层检测) → 建模关系阶段
//! (SPT 社会渗透) → 选择共情响应意图 (EDOS 8 意图) → 生成"能量/频率/震动"
//! 三通道表达 (声音韵律 / 文字流式节拍 / 视觉脉动), 并预留摄像头微表情接口。
//!
//! ## 设计原则
//! - **Mirror-then-Guide (镜像引导)**: 先以与用户一致的节奏镜像共情 (entrainment),
//!   再向目标情绪状态平滑引导 (co-regulation)。
//! - **证据分级**: 声音韵律 (pitch/rate/energy → 情绪感知) 证据最强; 文字流式
//!   节奏 (动态打字呈现 → 温暖/信任/共情) 证据强; 视觉运动/色彩表达情绪中强;
//!   脑电夹带 (双耳节拍) 证据弱 → 仅作目标参数不作承诺。
//! - **双层检测**: 词典启发式优先, 词典=中性且 `llm_enabled` 时采用 LLM 精调
//!   提示 (走现有网关, 符合 R-P48 零新第三方依赖)。
//! - **摄像头预留**: `VisualAffectSource` 为契约接口, 本期不采集, AU→情绪映射
//!   已按 FACS 预置, 采集层接入后即生效。

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserEmotion {
    Neutral,
    Joy,
    Sadness,
    Anger,
    Fear,
    Trust,
    Disgust,
    Surprise,
    Anticipation,
}

impl UserEmotion {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Joy => "joy",
            Self::Sadness => "sadness",
            Self::Anger => "anger",
            Self::Fear => "fear",
            Self::Trust => "trust",
            Self::Disgust => "disgust",
            Self::Surprise => "surprise",
            Self::Anticipation => "anticipation",
        }
    }

    /// PAD 三轴基座: (valence, arousal, dominance), 值域 [0,1], 0.5 为中性。
    pub fn pad(&self) -> (f64, f64, f64) {
        match self {
            Self::Neutral => (0.5, 0.5, 0.5),
            Self::Joy => (0.8, 0.7, 0.7),
            Self::Sadness => (0.2, 0.3, 0.3),
            Self::Anger => (0.2, 0.8, 0.8),
            Self::Fear => (0.2, 0.8, 0.2),
            Self::Trust => (0.7, 0.4, 0.7),
            Self::Disgust => (0.3, 0.4, 0.4),
            Self::Surprise => (0.5, 0.8, 0.5),
            Self::Anticipation => (0.7, 0.6, 0.6),
        }
    }
}

/// 词典情绪触发器 — 中文+英文关键词, 命中计数归一为强度。
const LEXICON: &[(UserEmotion, &[&str])] = &[
    (
        UserEmotion::Joy,
        &["开心", "高兴", "太好了", "谢谢", "棒", "喜欢", "好棒", "happy", "great", "love", "thank", "amazing"],
    ),
    (
        UserEmotion::Sadness,
        &["难过", "伤心", "失望", "抱歉", "难受", "沮丧", "sad", "sorry", "unhappy", "depressed"],
    ),
    (
        UserEmotion::Anger,
        &["生气", "愤怒", "讨厌", "烦死", "气死", "angry", "mad", "furious", "hate"],
    ),
    (
        UserEmotion::Fear,
        &["害怕", "担心", "紧张", "慌", "恐惧", "afraid", "scared", "worried", "nervous"],
    ),
    (
        UserEmotion::Trust,
        &["相信", "信任", "放心", "信赖", "trust", "believe", "rely"],
    ),
    (
        UserEmotion::Disgust,
        &["恶心", "反感", "嫌弃", "disgust", "gross", "yuck"],
    ),
    (
        UserEmotion::Surprise,
        &["哇", "天哪", "竟然", "意外", "惊喜", "wow", "unexpected", "surprise"],
    ),
    (
        UserEmotion::Anticipation,
        &["期待", "盼望", "准备出发", "hoping", "looking forward", "expect"],
    ),
];

/// 词典启发式检测: 返回 (情绪, 强度)。
pub fn lexicon_detect(text: &str) -> (UserEmotion, f64) {
    let lower = text.to_lowercase();
    let mut best = (UserEmotion::Neutral, 0.0f64);
    for (emotion, keys) in LEXICON {
        let hits = keys.iter().filter(|k| lower.contains(**k)).count();
        let score = (hits as f64).min(3.0) / 3.0;
        if score > best.1 {
            best = (*emotion, score);
        }
    }
    best
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAffectConfig {
    pub alpha: f64,
    pub max_history: usize,
    pub llm_enabled: bool,
}

impl Default for UserAffectConfig {
    fn default() -> Self {
        Self {
            alpha: 0.35,
            max_history: 50,
            llm_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAffectObservation {
    pub emotion: UserEmotion,
    pub intensity: f64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAffectSnapshot {
    pub emotion: UserEmotion,
    pub valence: f64,
    pub arousal: f64,
    pub dominance: f64,
    pub intensity: f64,
    pub history_len: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAffectModel {
    pub config: UserAffectConfig,
    pub emotion: UserEmotion,
    pub valence: f64,
    pub arousal: f64,
    pub dominance: f64,
    pub intensity: f64,
    pub history: VecDeque<UserAffectObservation>,
}

impl UserAffectModel {
    pub fn new(config: UserAffectConfig) -> Self {
        Self {
            config,
            emotion: UserEmotion::Neutral,
            valence: 0.5,
            arousal: 0.5,
            dominance: 0.5,
            intensity: 0.5,
            history: VecDeque::new(),
        }
    }

    pub fn default() -> Self {
        Self::new(UserAffectConfig::default())
    }

    /// 双层检测: 词典优先, 词典=中性且启用 LLM 时采用 LLM 精调提示。
    pub fn detect_from_text(
        &mut self,
        text: &str,
        llm_hint: Option<UserEmotion>,
    ) -> UserAffectSnapshot {
        let (lex_emotion, lex_score) = lexicon_detect(text);
        let (emotion, intensity, source) =
            if lex_emotion == UserEmotion::Neutral && self.config.llm_enabled {
                match llm_hint {
                    Some(e) if e != UserEmotion::Neutral => (e, 0.7, "llm"),
                    _ => (lex_emotion, lex_score, "lexicon"),
                }
            } else {
                (lex_emotion, lex_score, "lexicon")
            };
        self.emotion = emotion;
        self.intensity = if intensity > 0.0 { intensity } else { 0.5 };
        let (v, a, d) = emotion.pad();
        let alpha = self.config.alpha;
        self.valence = (alpha * v + (1.0 - alpha) * self.valence).max(0.0).min(1.0);
        self.arousal = (alpha * a + (1.0 - alpha) * self.arousal).max(0.0).min(1.0);
        self.dominance = (alpha * d + (1.0 - alpha) * self.dominance).max(0.0).min(1.0);
        self.history.push_back(UserAffectObservation {
            emotion,
            intensity: self.intensity,
            source: source.into(),
        });
        if self.history.len() > self.config.max_history {
            self.history.pop_front();
        }
        self.snapshot()
    }

    pub fn snapshot(&self) -> UserAffectSnapshot {
        UserAffectSnapshot {
            emotion: self.emotion,
            valence: self.valence,
            arousal: self.arousal,
            dominance: self.dominance,
            intensity: self.intensity,
            history_len: self.history.len(),
        }
    }

    /// 最近 n 条的主导情绪 (多数票)。
    pub fn dominant_recent(&self, n: usize) -> UserEmotion {
        let mut counts = std::collections::HashMap::new();
        for obs in self.history.iter().rev().take(n) {
            *counts.entry(obs.emotion).or_insert(0u32) += 1;
        }
        counts
            .into_iter()
            .max_by_key(|(_, c)| *c)
            .map(|(e, _)| e)
            .unwrap_or(UserEmotion::Neutral)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipStage {
    Stranger,
    Acquaintance,
    Friend,
    Confidant,
    Bond,
}

impl RelationshipStage {
    pub fn order(&self) -> usize {
        match self {
            Self::Stranger => 0,
            Self::Acquaintance => 1,
            Self::Friend => 2,
            Self::Confidant => 3,
            Self::Bond => 4,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Stranger => "stranger",
            Self::Acquaintance => "acquaintance",
            Self::Friend => "friend",
            Self::Confidant => "confidant",
            Self::Bond => "bond",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipConfig {
    pub trust_gain: f64,
    pub affinity_gain: f64,
    pub max_interactions: u64,
}

impl Default for RelationshipConfig {
    fn default() -> Self {
        Self {
            trust_gain: 0.02,
            affinity_gain: 0.03,
            max_interactions: 1000,
        }
    }
}

/// 关系阶段状态 — 社会渗透理论 (SPT) 5 层, 互惠自我披露推进信任/亲密度。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipState {
    pub config: RelationshipConfig,
    pub stage: RelationshipStage,
    pub trust: f64,
    pub affinity: f64,
    pub interactions: u64,
    pub disclosures: u64,
    pub avg_disclosure_depth: f64,
}

impl RelationshipState {
    pub fn new() -> Self {
        Self {
            config: RelationshipConfig::default(),
            stage: RelationshipStage::Stranger,
            trust: 0.0,
            affinity: 0.0,
            interactions: 0,
            disclosures: 0,
            avg_disclosure_depth: 0.0,
        }
    }

    pub fn default() -> Self {
        Self::new()
    }

    /// 每轮对话更新: 用户披露 + 正向情绪 → 信任/亲密度提升, 触发阶段晋升。
    pub fn on_turn(&mut self, user_disclosed: bool, disclosure_depth: f64, positive: f64) {
        self.interactions += 1;
        let depth = disclosure_depth.max(0.0).min(1.0);
        let positive = positive.max(0.0).min(1.0);
        let trust_gain = self.config.trust_gain * (0.6 + 0.4 * positive)
            + if user_disclosed { depth * 0.02 } else { 0.0 };
        self.trust = (self.trust + trust_gain).max(0.0).min(1.0);
        let affinity_gain = self.config.affinity_gain * (0.5 + 0.5 * positive);
        self.affinity = (self.affinity + affinity_gain).max(0.0).min(1.0);
        if user_disclosed {
            self.disclosures += 1;
            let n = self.disclosures as f64;
            self.avg_disclosure_depth = ((n - 1.0) / n) * self.avg_disclosure_depth + depth / n;
        }
        self.promote();
    }

    fn promote(&mut self) {
        let stage = match (self.stage.order(), self.trust) {
            (0, t) if t >= 0.15 => RelationshipStage::Acquaintance,
            (1, t) if t >= 0.4 => RelationshipStage::Friend,
            (2, t) if t >= 0.65 => RelationshipStage::Confidant,
            (3, t) if t >= 0.85 => RelationshipStage::Bond,
            _ => self.stage,
        };
        self.stage = stage;
    }

    /// 关系阶段允许的最大自我披露深度 (SPT 渗透层, 互惠门控)。
    pub fn disclosure_gate(&self) -> f64 {
        match self.stage {
            RelationshipStage::Stranger => 0.1,
            RelationshipStage::Acquaintance => 0.3,
            RelationshipStage::Friend => 0.5,
            RelationshipStage::Confidant => 0.7,
            RelationshipStage::Bond => 0.85,
        }
    }

    /// 互惠门控: 代理自曝深度不得超过当前关系阶段的渗透上限。
    pub fn can_self_disclose(&self, depth: f64) -> bool {
        depth <= self.disclosure_gate()
    }

    pub fn snapshot(&self) -> (RelationshipStage, f64, f64, u64, u64) {
        (
            self.stage,
            self.trust,
            self.affinity,
            self.interactions,
            self.disclosures,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResponseIntent {
    Questioning,
    Agreeing,
    Acknowledging,
    Sympathizing,
    Encouraging,
    Consoling,
    Suggesting,
    Wishing,
    Informative,
}

impl ResponseIntent {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Questioning => "questioning",
            Self::Agreeing => "agreeing",
            Self::Acknowledging => "acknowledging",
            Self::Sympathizing => "sympathizing",
            Self::Encouraging => "encouraging",
            Self::Consoling => "consoling",
            Self::Suggesting => "suggesting",
            Self::Wishing => "wishing",
            Self::Informative => "informative",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpathyConfig {
    pub active_listening: bool,
    pub open_questions: bool,
}

impl Default for EmpathyConfig {
    fn default() -> Self {
        Self {
            active_listening: true,
            open_questions: true,
        }
    }
}

impl EmpathyConfig {
    pub fn warm() -> Self {
        Self::default()
    }

    pub fn minimal() -> Self {
        Self {
            active_listening: false,
            open_questions: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpathyStrategy {
    pub config: EmpathyConfig,
}

impl EmpathyStrategy {
    pub fn new(config: EmpathyConfig) -> Self {
        Self { config }
    }

    pub fn default() -> Self {
        Self::new(EmpathyConfig::default())
    }

    /// EDOS 8 意图 + Informative, 按用户情绪与关系阶段选择。
    pub fn select_intent(&self, user: UserEmotion, stage: RelationshipStage) -> ResponseIntent {
        let bonded = stage.order() >= 3;
        match user {
            UserEmotion::Joy => {
                if bonded {
                    ResponseIntent::Encouraging
                } else {
                    ResponseIntent::Agreeing
                }
            }
            UserEmotion::Sadness => {
                if bonded {
                    ResponseIntent::Consoling
                } else {
                    ResponseIntent::Sympathizing
                }
            }
            UserEmotion::Anger | UserEmotion::Disgust => ResponseIntent::Acknowledging,
            UserEmotion::Fear => ResponseIntent::Consoling,
            UserEmotion::Surprise => ResponseIntent::Questioning,
            UserEmotion::Trust => ResponseIntent::Agreeing,
            UserEmotion::Anticipation => ResponseIntent::Encouraging,
            UserEmotion::Neutral => {
                if bonded {
                    ResponseIntent::Questioning
                } else {
                    ResponseIntent::Informative
                }
            }
        }
    }

    /// 主动倾听 backchannel 短语 (Gracie/共情对话实证: 提升 rapport)。
    pub fn active_listening_phrase(&self) -> Option<&'static str> {
        if self.config.active_listening {
            Some("我在听")
        } else {
            None
        }
    }

    /// 开放式提问 (Wh- 型, 实证比 YN 问题引出更多自我披露)。
    pub fn open_question(&self, user: UserEmotion) -> Option<&'static str> {
        if !self.config.open_questions {
            return None;
        }
        Some(match user {
            UserEmotion::Sadness => "愿意多说一点发生了什么吗？",
            UserEmotion::Anger => "是什么让你这么生气？",
            UserEmotion::Fear => "最让你担心的是哪一部分？",
            UserEmotion::Joy => "是什么让你这么开心？",
            UserEmotion::Surprise => "这之后你是怎么想的？",
            UserEmotion::Neutral => "你今天想聊点什么？",
            _ => "你是怎么看的？",
        })
    }
}

/// "能量/频率/震动"三通道表达配置 — 声音韵律 / 文字流式节拍 / 视觉脉动。
/// 频率参数 (entrainment_hz) 为脑电夹带目标带, 非承诺 (双耳节拍证据不一致)。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RhythmProfile {
    pub voice_pitch: f64,
    pub voice_rate: f64,
    pub voice_energy: f64,
    pub text_cadence_ms: u64,
    pub visual_pulse_hz: f64,
    pub entrainment_hz: f64,
    pub anchor_breathe_ms: u64,
}

impl RhythmProfile {
    /// 平静专注: 低唤醒正价 → alpha 带。
    pub fn calm() -> Self {
        Self {
            voice_pitch: 0.95,
            voice_rate: 0.9,
            voice_energy: 0.4,
            text_cadence_ms: 28,
            visual_pulse_hz: 1.0,
            entrainment_hz: 10.0,
            anchor_breathe_ms: 5000,
        }
    }

    /// 活力激发: 高唤醒正价 → beta 带。
    pub fn energize() -> Self {
        Self {
            voice_pitch: 1.15,
            voice_rate: 1.15,
            voice_energy: 0.7,
            text_cadence_ms: 18,
            visual_pulse_hz: 1.6,
            entrainment_hz: 20.0,
            anchor_breathe_ms: 3000,
        }
    }

    /// 安抚缓和: 低唤醒负价 → theta 带。
    pub fn soothe() -> Self {
        Self {
            voice_pitch: 0.85,
            voice_rate: 0.85,
            voice_energy: 0.35,
            text_cadence_ms: 32,
            visual_pulse_hz: 0.9,
            entrainment_hz: 6.0,
            anchor_breathe_ms: 6000,
        }
    }

    /// Mirror: 以与用户一致的节奏镜像 (entrainment 共情), 并按情绪微调去激化。
    pub fn from_user(emotion: UserEmotion, valence: f64, arousal: f64) -> Self {
        let mut p = if valence >= 0.55 {
            if arousal >= 0.55 {
                Self::energize()
            } else {
                Self::calm()
            }
        } else if arousal >= 0.55 {
            Self::energize()
        } else {
            Self::soothe()
        };
        match emotion {
            UserEmotion::Fear | UserEmotion::Sadness => {
                p.voice_rate = (p.voice_rate - 0.1).max(0.7);
                p.voice_pitch = (p.voice_pitch - 0.08).max(0.7);
            }
            UserEmotion::Anger | UserEmotion::Disgust => {
                p.voice_energy = (p.voice_energy * 0.7).max(0.2);
            }
            _ => {}
        }
        p
    }

    /// Guide: 向目标状态线性渐变 (co-regulation), t∈[0,1] 为引导强度。
    pub fn guide_toward(&self, target: RhythmProfile, t: f64) -> Self {
        let t = t.max(0.0).min(1.0);
        Self {
            voice_pitch: lerp(self.voice_pitch, target.voice_pitch, t),
            voice_rate: lerp(self.voice_rate, target.voice_rate, t),
            voice_energy: lerp(self.voice_energy, target.voice_energy, t),
            text_cadence_ms: lerp_u64(self.text_cadence_ms, target.text_cadence_ms, t),
            visual_pulse_hz: lerp(self.visual_pulse_hz, target.visual_pulse_hz, t),
            entrainment_hz: lerp(self.entrainment_hz, target.entrainment_hz, t),
            anchor_breathe_ms: lerp_u64(self.anchor_breathe_ms, target.anchor_breathe_ms, t),
        }
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn lerp_u64(a: u64, b: u64, t: f64) -> u64 {
    (a as f64 + (b as f64 - a as f64) * t).round() as u64
}

#[derive(Debug, Clone, Copy)]
pub enum GuideMode {
    Auto,
    Mirror,
    Toward(RhythmProfile),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectiveReadout {
    pub user: UserAffectSnapshot,
    pub stage: RelationshipStage,
    pub trust: f64,
    pub affinity: f64,
    pub intent: ResponseIntent,
    pub expression: String,
    pub rhythm: RhythmProfile,
    pub backchannel: Option<String>,
    pub followup_question: Option<String>,
}

/// 面向人类的情感界面 — 感知 → 关系 → 策略 → 三通道表达。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectiveInterface {
    pub user: UserAffectModel,
    pub relationship: RelationshipState,
    pub strategy: EmpathyStrategy,
}

impl AffectiveInterface {
    pub fn new() -> Self {
        Self {
            user: UserAffectModel::default(),
            relationship: RelationshipState::new(),
            strategy: EmpathyStrategy::default(),
        }
    }

    pub fn process_user_input(
        &mut self,
        text: &str,
        llm_hint: Option<UserEmotion>,
        guide: GuideMode,
    ) -> AffectiveReadout {
        let snapshot = self.user.detect_from_text(text, llm_hint);
        let (disclosed, depth) = estimate_disclosure(text);
        self.relationship.on_turn(disclosed, depth, snapshot.valence);
        let intent = self.strategy.select_intent(snapshot.emotion, self.relationship.stage);
        let mirror = RhythmProfile::from_user(snapshot.emotion, snapshot.valence, snapshot.arousal);
        let rhythm = match guide {
            GuideMode::Auto | GuideMode::Mirror => mirror,
            GuideMode::Toward(target) => mirror.guide_toward(target, 0.4),
        };
        let backchannel = if snapshot.intensity >= 0.4 && snapshot.emotion != UserEmotion::Neutral {
            self.strategy
                .active_listening_phrase()
                .map(|s| s.to_string())
        } else {
            None
        };
        let followup = if snapshot.emotion != UserEmotion::Neutral {
            self.strategy.open_question(snapshot.emotion).map(|s| s.to_string())
        } else {
            None
        };
        let expression = expression_for(snapshot.emotion).to_string();
        AffectiveReadout {
            user: snapshot,
            stage: self.relationship.stage,
            trust: self.relationship.trust,
            affinity: self.relationship.affinity,
            intent,
            expression,
            rhythm,
            backchannel,
            followup_question: followup,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// 情绪 → 数字人微表情键 (镜像: 以与用户一致的微表情回应)。
fn expression_for(user: UserEmotion) -> &'static str {
    match user {
        UserEmotion::Joy | UserEmotion::Trust => "smile",
        UserEmotion::Sadness | UserEmotion::Disgust => "frown",
        UserEmotion::Anger => "fury",
        UserEmotion::Fear => "tilt",
        UserEmotion::Surprise => "shock",
        UserEmotion::Anticipation => "look_up",
        UserEmotion::Neutral => "idle",
    }
}

/// 自我披露启发式: 第一人称自述模式 + 长度 → (是否披露, 披露深度)。
pub fn estimate_disclosure(text: &str) -> (bool, f64) {
    let lower = text.to_lowercase();
    let personal = [
        "我觉得",
        "我想",
        "我的",
        "我遇到",
        "我记得",
        "我担心",
        "我害怕",
        "我昨天",
        "i feel",
        "i think",
        "i am",
        "i'm",
        "i was",
        "i have",
    ];
    let has_personal = personal.iter().any(|p| lower.contains(p));
    // CJK 无空格: 空白词数恒为 1, 需按字符折算字数 (≈2 字/词) 否则披露深度被严重低估。
    let whitespace_words = text.split_whitespace().count();
    let cjk_units = (text.chars().filter(|c| !c.is_whitespace()).count() + 1) / 2;
    let words = whitespace_words.max(cjk_units);
    let length_depth = (words as f64 / 40.0).min(0.4);
    let depth = (0.2 + length_depth) * if has_personal { 1.0 } else { 0.4 };
    (has_personal, depth.max(0.0).min(1.0))
}

/// 摄像头微表情采集预留 — 本期只定义契约, 不采集。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualAffect {
    pub au_codes: Vec<u8>,
    pub emotion: Option<UserEmotion>,
    pub confidence: f64,
}

pub trait VisualAffectSource {
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self);
    fn poll(&mut self) -> Option<VisualAffect>;
}

/// FACS 微表情 AU 子集 → 情绪映射 (采集层接入后即生效)。
pub fn au_to_emotion(aus: &[u8]) -> Option<UserEmotion> {
    let has = |au: u8| aus.contains(&au);
    if has(6) && has(12) && has(25) {
        Some(UserEmotion::Joy)
    } else if has(1) && has(4) && has(15) {
        Some(UserEmotion::Sadness)
    } else if has(4) && has(5) && has(24) {
        Some(UserEmotion::Anger)
    } else if has(1) && has(2) && has(4) && has(20) {
        Some(UserEmotion::Fear)
    } else if has(1) && has(2) && has(5) && has(26) {
        Some(UserEmotion::Surprise)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexicon_detect_sadness_and_joy() {
        assert_eq!(lexicon_detect("我很难过").0, UserEmotion::Sadness);
        assert_eq!(lexicon_detect("太开心了 好棒").0, UserEmotion::Joy);
        assert_eq!(lexicon_detect("普通的一句话").0, UserEmotion::Neutral);
    }

    #[test]
    fn test_two_tier_detection_llm_hint() {
        let mut model = UserAffectModel::new(UserAffectConfig {
            llm_enabled: true,
            ..UserAffectConfig::default()
        });
        let snap = model.detect_from_text("今天天气不错", Some(UserEmotion::Anger));
        assert_eq!(snap.emotion, UserEmotion::Anger);
        assert_eq!(model.history.back().unwrap().source, "llm");
        let mut off = UserAffectModel::new(UserAffectConfig {
            llm_enabled: false,
            ..UserAffectConfig::default()
        });
        let snap = off.detect_from_text("今天天气不错", Some(UserEmotion::Anger));
        assert_eq!(snap.emotion, UserEmotion::Neutral);
    }

    #[test]
    fn test_relationship_stage_progression() {
        let mut rel = RelationshipState::new();
        assert_eq!(rel.stage, RelationshipStage::Stranger);
        for _ in 0..8 {
            rel.on_turn(true, 0.5, 0.8);
        }
        assert_eq!(rel.stage, RelationshipStage::Acquaintance);
        assert!(rel.trust > 0.2);
        for _ in 0..12 {
            rel.on_turn(true, 0.6, 0.8);
        }
        assert_eq!(rel.stage, RelationshipStage::Friend);
        assert_eq!(rel.disclosures, 20);
    }

    #[test]
    fn test_reciprocity_gate() {
        let mut rel = RelationshipState::new();
        assert!(!rel.can_self_disclose(0.8));
        for _ in 0..22 {
            rel.on_turn(true, 0.7, 0.7);
        }
        assert_eq!(rel.stage, RelationshipStage::Confidant);
        assert!(rel.can_self_disclose(0.7));
        assert!(!rel.can_self_disclose(0.85));
    }

    #[test]
    fn test_intent_selection() {
        let strategy = EmpathyStrategy::default();
        assert_eq!(
            strategy.select_intent(UserEmotion::Sadness, RelationshipStage::Stranger),
            ResponseIntent::Sympathizing
        );
        assert_eq!(
            strategy.select_intent(UserEmotion::Joy, RelationshipStage::Bond),
            ResponseIntent::Encouraging
        );
        assert_eq!(
            strategy.select_intent(UserEmotion::Neutral, RelationshipStage::Stranger),
            ResponseIntent::Informative
        );
    }

    #[test]
    fn test_rhythm_mirror_and_guide() {
        let sad = RhythmProfile::from_user(UserEmotion::Sadness, 0.2, 0.3);
        assert!(sad.voice_rate < RhythmProfile::calm().voice_rate);
        let joy = RhythmProfile::from_user(UserEmotion::Joy, 0.8, 0.7);
        assert!(joy.text_cadence_ms < RhythmProfile::calm().text_cadence_ms);
        let blended = sad.guide_toward(RhythmProfile::calm(), 0.5);
        assert!((blended.voice_rate - 0.5 * (sad.voice_rate + RhythmProfile::calm().voice_rate)).abs() < 1e-6);
    }

    #[test]
    fn test_process_user_input_readout() {
        let mut iface = AffectiveInterface::new();
        let readout = iface.process_user_input("我很难过，真的很难受", None, GuideMode::Auto);
        assert_eq!(readout.user.emotion, UserEmotion::Sadness);
        assert_eq!(readout.expression, "frown");
        assert_eq!(readout.intent, ResponseIntent::Sympathizing);
        assert!(readout.backchannel.is_some());
        assert!(readout.followup_question.is_some());
        assert_eq!(iface.relationship.interactions, 1);
    }

    #[test]
    fn test_expression_mapping() {
        assert_eq!(expression_for(UserEmotion::Anger), "fury");
        assert_eq!(expression_for(UserEmotion::Joy), "smile");
        assert_eq!(expression_for(UserEmotion::Neutral), "idle");
    }

    #[test]
    fn test_au_to_emotion() {
        assert_eq!(au_to_emotion(&[4, 5, 24]), Some(UserEmotion::Anger));
        assert_eq!(au_to_emotion(&[6, 12, 25]), Some(UserEmotion::Joy));
        assert_eq!(au_to_emotion(&[4, 5]), None);
    }

    /// CJK 无空格句子不得被空白词数低估披露深度 (回归: 8 字中文句子 ≈ 4 词)。
    #[test]
    fn test_disclosure_cjk_word_units() {
        let (personal, depth) = estimate_disclosure("我遇到一件难过的事");
        assert!(personal);
        // 9 字 → 5 词单位 → depth=0.325; 旧实现按空白词数=1 → 仅 0.225。
        assert!(depth > 0.3, "CJK 深度不得被空白词数低估, got {depth}");
        let (_p, d2) = estimate_disclosure("普通一句话");
        assert!(d2 < 0.3, "非个人陈述深度应更低, got {d2}");
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut iface = AffectiveInterface::new();
        iface.process_user_input("我遇到一件烦心事", None, GuideMode::Auto);
        let json = iface.to_json().unwrap();
        let restored = AffectiveInterface::from_json(&json).unwrap();
        assert_eq!(restored.relationship.interactions, 1);
        assert_eq!(restored.user.emotion, iface.user.emotion);
    }

    // ── C2 集成测试: 生产链路 持久化 → 恢复 → 数字人消费 ──
    #[test]
    fn test_end_to_end_persist_restore_consume() {
        // 1) 真实 KB (临时文件) + SecondBrain — 生产持久化路径 (bg loop 每 tick 调 save_affective)。
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_aff_e2e_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = crate::neotrix::nt_memory_kb::KnowledgeBase::open(Some(tmp))
            .expect("open temp KB");
        let kb = std::sync::Arc::new(kb);
        let mut sb = crate::core::nt_core_second_brain::SecondBrain::new();
        sb.attach_kb(kb.clone());

        // 2) 8 轮悲伤披露交互 → 关系推进出 Stranger。
        let mut iface = AffectiveInterface::new();
        for _ in 0..8 {
            iface.process_user_input("我遇到一件难过的事", None, GuideMode::Auto);
        }
        assert_eq!(iface.relationship.interactions, 8);
        assert_ne!(iface.relationship.stage, RelationshipStage::Stranger);

        // 3) 生产持久化。
        sb.save_affective(&iface);

        // 4) 启动恢复路径: 从 KB 读回 → from_json。
        let raw = kb
            .kv_get("emotion", "affective_interface")
            .expect("kv_get")
            .expect("affective interface persisted to KB");
        let restored = AffectiveInterface::from_json(&raw).expect("restore from KB");

        // 5) 连续性: 关系阶段/交互数/用户情绪跨 session 保持。
        assert_eq!(restored.relationship.stage, iface.relationship.stage);
        assert_eq!(restored.relationship.interactions, 8);
        assert_eq!(restored.relationship.disclosures, iface.relationship.disclosures);
        assert_eq!(restored.user.emotion, iface.user.emotion);

        // 6) 数字人生产消费: 恢复的 interface 驱动情感化回复/表情/韵律。
        let mut pipe = crate::neotrix::l1_body_impl::nt_io_digital_human::DigitalHumanPipeline::new(
            crate::neotrix::l1_body_impl::nt_io_digital_human::PersonaConfig::default(),
        );
        pipe.affective = restored;
        pipe.start_session();
        let resp = pipe.process_audio_input("我很难过，真的很难受");
        assert_eq!(pipe.affective.relationship.interactions, 9); // 恢复后继续计数
        assert!(resp.animation.contains("frown") || resp.animation.contains("tilt"));
        assert!(
            resp.tts_text.contains("理解") || resp.tts_text.contains("陪") || resp.tts_text.contains("谢谢")
        );
        assert!(resp.tts_text.contains("我"));
    }
}