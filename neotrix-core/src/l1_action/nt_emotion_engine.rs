/// Emotion Engine trait — L1 抽象，解耦 L4 实现
///
/// 定义情感引擎的接口，L4 提供具体实现。
/// 遵循依赖倒置原则：L1 定义 trait，L4 实现 trait。

/// 情感类型
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

/// 情感引擎 trait — 情感检测和状态管理的抽象
pub trait EmotionEngineTrait: Send + Sync {
    /// 从文本检测情感
    fn detect_from_text(&mut self, text: &str) -> Emotion;

    /// 设置情感强度
    fn set_intensity(&mut self, intensity: f64);

    /// 获取当前情感
    fn current_emotion(&self) -> Emotion;

    /// 获取情感强度
    fn intensity(&self) -> f64;
}

/// 从表情字符串映射到情感类型
pub fn emotion_from_expression(expression: &str) -> Emotion {
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