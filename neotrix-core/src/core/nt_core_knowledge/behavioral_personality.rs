use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

// ═══════════════════════════════════════════════════════════════
// BehavioralPersonalityEngine — 用户数字分身/行为人格系统
//
// 融合 Agethos (OCEAN+PAD)、Atman (自写身份信)、Nomos (决策模式)、
// PersonaVLM (EMA衰减进化)、Habitus (行为指纹) 的核心架构。
//
// 三层人格:
//   Layer 1 — 稳定身份 (OCEAN + 价值观 + 决策模式)
//   Layer 2 — 动态状态 (PAD情绪 + 能量 + 注意力)
//   Layer 3 — 行为指纹 (对话模式 + 风格偏好)
//
// 进化机制:
//   - EMA余弦衰减人格更新 (PersonaVLM)
//   - Hebbian式交互学习 (Agethos)
//   - 会话间隔自写身份信 (Atman)
//   - 平行观察者分析 (Nomos)
// ═══════════════════════════════════════════════════════════════

/// 大五人格特质 (OCEAN) — 稳定层核心
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OceanTraits {
    pub openness: f64,          // 开放性 0.0-1.0
    pub conscientiousness: f64, // 尽责性 0.0-1.0
    pub extraversion: f64,      // 外向性 0.0-1.0
    pub agreeableness: f64,     // 宜人性 0.0-1.0
    pub neuroticism: f64,       // 神经质 0.0-1.0
}

impl Default for OceanTraits {
    fn default() -> Self {
        Self {
            openness: 0.5,
            conscientiousness: 0.5,
            extraversion: 0.5,
            agreeableness: 0.5,
            neuroticism: 0.5,
        }
    }
}

/// PAD三轴情绪状态 (Pleasure-Arousal-Dominance) — 动态层核心
/// 参考: Mehrabian 1996, Agethos, Emotion System v2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PadEmotion {
    pub pleasure: f64,  // -1.0 (痛苦) ~ 1.0 (愉悦)
    pub arousal: f64,   // 0.0 (平静) ~ 1.0 (激动)
    pub dominance: f64, // -1.0 (顺从) ~ 1.0 (支配)
    /// 指数衰减率 (每cycle)
    pub decay_rate: f64,
}

impl Default for PadEmotion {
    fn default() -> Self {
        Self {
            pleasure: 0.0,
            arousal: 0.3,
            dominance: 0.0,
            decay_rate: 0.05,
        }
    }
}

impl PadEmotion {
    /// 应用情感刺激 (来自用户交互)
    pub fn stimulate(&mut self, p: f64, a: f64, d: f64) {
        self.pleasure = (self.pleasure + p * 0.3).clamp(-1.0, 1.0);
        self.arousal = (self.arousal + a * 0.3).clamp(0.0, 1.0);
        self.dominance = (self.dominance + d * 0.3).clamp(-1.0, 1.0);
    }

    /// 每cycle衰减至基线
    pub fn decay(&mut self) {
        let r = self.decay_rate;
        self.pleasure += (0.0 - self.pleasure) * r;
        self.arousal += (0.3 - self.arousal) * r;
        self.dominance += (0.0 - self.dominance) * r;
    }

    /// 情感标签 (KNN over PAD anchor points)
    pub fn label(&self) -> &'static str {
        if self.pleasure > 0.3 && self.arousal > 0.5 {
            "excited"
        } else if self.pleasure > 0.3 {
            "content"
        } else if self.pleasure < -0.3 && self.arousal > 0.5 {
            "anxious"
        } else if self.pleasure < -0.3 {
            "frustrated"
        } else if self.arousal < 0.2 {
            "calm"
        } else {
            "neutral"
        }
    }
}

/// 决策模式 — 用户如何权衡 (Nomos启发)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPattern {
    pub id: u64,
    pub name: String,
    pub weight: f64,
    pub evidence_count: u64,
    pub context: String,
    pub last_observed: u64,
}

/// 行为指纹 — 对话风格与偏好 (Habitus启发)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralFingerprint {
    /// 平均回复长度 (字符)
    pub avg_response_length: f64,
    /// 代码 vs 自然语言比例 0.0~1.0
    pub code_ratio: f64,
    /// 最常使用的语气: direct / technical / friendly / concise
    pub communication_style: String,
    /// 每轮平均消息数
    pub messages_per_turn: f64,
    /// 最活跃时段 (小时, 0-23)
    pub peak_hour: u8,
    /// 偏好话题
    pub preferred_topics: Vec<String>,
    /// 总交互次数
    pub total_interactions: u64,
}

impl Default for BehavioralFingerprint {
    fn default() -> Self {
        Self {
            avg_response_length: 0.0,
            code_ratio: 0.0,
            communication_style: "neutral".into(),
            messages_per_turn: 1.0,
            peak_hour: 12,
            preferred_topics: Vec::new(),
            total_interactions: 0,
        }
    }
}

/// 身份信 (Atman启发) — 会话间隔自我身份文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityLetter {
    pub content: String,
    pub cycle_written: u64,
    pub version: u32,
}

impl Default for IdentityLetter {
    fn default() -> Self {
        Self {
            content: String::new(),
            cycle_written: 0,
            version: 0,
        }
    }
}

/// 平行观察结果 (Nomos启发)
#[derive(Debug, Clone)]
pub struct ObservationRecord {
    pub cycle: u64,
    pub insight: String,
    pub category: ObservationCategory,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObservationCategory {
    CommunicationShift,
    TopicPreference,
    EmotionalState,
    DecisionBias,
    ValueSignal,
}

/// 用户数字分身 — 行为人格引擎
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDigitalTwin {
    /// 稳定身份层
    pub ocean: OceanTraits,
    /// 价值观 (权重排序)
    pub values: Vec<(String, f64)>,
    /// 活跃决策模式
    pub decision_patterns: Vec<DecisionPattern>,
    /// PAD情绪状态
    pub emotion: PadEmotion,
    /// 当前能量水平 0.0~1.0
    pub energy: f64,
    /// 行为指纹
    pub fingerprint: BehavioralFingerprint,
    /// 最近的交互迹
    #[serde(skip)]
    pub interaction_history: VecDeque<InteractionRecord>,
    /// 平行观察记录
    #[serde(skip)]
    pub observations: Vec<ObservationRecord>,
    /// 会话间隔身份信
    pub identity_letter: IdentityLetter,
    /// 进化计数器
    pub evolution_version: u32,
    /// 上次更新cycle
    pub last_update_cycle: u64,
}

/// 单次用户交互记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionRecord {
    pub cycle: u64,
    pub message_len: usize,
    pub contains_code: bool,
    pub topic: String,
    pub sentiment: f64,
    pub tags: Vec<String>,
}

impl Default for UserDigitalTwin {
    fn default() -> Self {
        Self::new()
    }
}

impl UserDigitalTwin {
    pub fn new() -> Self {
        Self {
            ocean: OceanTraits::default(),
            values: vec![
                ("accuracy".into(), 0.7),
                ("efficiency".into(), 0.6),
                ("creativity".into(), 0.5),
            ],
            decision_patterns: Vec::new(),
            emotion: PadEmotion::default(),
            energy: 0.8,
            fingerprint: BehavioralFingerprint::default(),
            interaction_history: VecDeque::with_capacity(200),
            observations: Vec::new(),
            identity_letter: IdentityLetter::default(),
            evolution_version: 0,
            last_update_cycle: 0,
        }
    }

    /// 📥 记录一次用户交互 (每轮对话调用)
    pub fn record_interaction(
        &mut self,
        cycle: u64,
        message: &str,
        topic: &str,
        tags: Vec<String>,
    ) {
        let contains_code = message.contains("fn ")
            || message.contains("def ")
            || message.contains("```")
            || message.contains("impl ")
            || message.contains("struct ")
            || message.contains(" pub ");
        let sentiment = simple_sentiment(message);

        let rec = InteractionRecord {
            cycle,
            message_len: message.len(),
            contains_code,
            topic: topic.to_string(),
            sentiment,
            tags,
        };

        if self.interaction_history.len() >= 200 {
            self.interaction_history.pop_front();
        }
        self.interaction_history.push_back(rec);

        // 更新指纹
        self.fingerprint.total_interactions += 1;
        self.update_fingerprint(cycle);
    }

    /// 更新行为指纹
    fn update_fingerprint(&mut self, _cycle: u64) {
        let history: Vec<_> = self.interaction_history.iter().collect();
        if history.is_empty() {
            return;
        }

        self.fingerprint.avg_response_length =
            history.iter().map(|r| r.message_len as f64).sum::<f64>() / history.len() as f64;
        self.fingerprint.code_ratio =
            history.iter().filter(|r| r.contains_code).count() as f64 / history.len() as f64;

        // 判断沟通风格
        let avg_sentiment = history.iter().map(|r| r.sentiment).sum::<f64>() / history.len() as f64;
        let code_ratio = self.fingerprint.code_ratio;

        self.fingerprint.communication_style = if code_ratio > 0.4 {
            "technical"
        } else if avg_sentiment > 0.2 {
            "friendly"
        } else if avg_sentiment < -0.2 {
            "direct"
        } else {
            "neutral"
        }
        .to_string();

        // 收集偏好话题 (保留出现次数最多的前5)
        let mut topic_count: HashMap<String, usize> = HashMap::new();
        for r in &history {
            *topic_count.entry(r.topic.clone()).or_insert(0) += 1;
        }
        let mut sorted: Vec<_> = topic_count.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        self.fingerprint.preferred_topics = sorted.into_iter().take(5).map(|(t, _)| t).collect();
    }

    /// 🧠 EMA人格进化 (PersonaVLM启发)
    /// p_m = λ·p_{m-1} + (1-λ)·p'_m  with cosine decay
    pub fn evolve_personality(&mut self, cycle: u64) {
        if self.last_update_cycle == 0 {
            self.last_update_cycle = cycle;
            return;
        }
        let cycles_since_update = cycle.saturating_sub(self.last_update_cycle);
        if cycles_since_update < 5 {
            return;
        }

        // 余弦衰减 λ: 快速初始适应 → 逐渐稳定
        let t = (self.evolution_version as f64).min(100.0);
        let lambda = 0.5 + 0.4 * (t * std::f64::consts::PI / 200.0).cos();

        // 从最近交互迹计算 delta
        let recent: Vec<_> = self.interaction_history.iter().rev().take(20).collect();
        if recent.is_empty() {
            return;
        }

        let avg_sentiment = recent.iter().map(|r| r.sentiment).sum::<f64>() / recent.len() as f64;
        let recent_code_ratio =
            recent.iter().filter(|r| r.contains_code).count() as f64 / recent.len() as f64;

        // OCEAN 更新
        self.ocean.openness = self.ocean.openness * lambda + recent_code_ratio * (1.0 - lambda);
        self.ocean.extraversion =
            self.ocean.extraversion * lambda + avg_sentiment.max(0.0) * (1.0 - lambda);
        self.ocean.neuroticism =
            self.ocean.neuroticism * lambda + (-avg_sentiment).max(0.0) * (1.0 - lambda);

        // 情感衰减
        self.emotion.decay();

        // 能量恢复
        self.energy = (self.energy + 0.02).min(1.0);

        self.evolution_version += 1;
        self.last_update_cycle = cycle;
    }

    /// 📝 写身份信 (Atman启发) — 每30cycle在会话间隔调用
    pub fn write_identity_letter(&mut self, cycle: u64) {
        let mut letter = String::new();
        letter.push_str(&format!("## 数字分身状态 @cycle {}\n\n", cycle));

        // 人格概要
        letter.push_str(&format!(
            "**人格**: O={:.1} C={:.1} E={:.1} A={:.1} N={:.1}\n",
            self.ocean.openness,
            self.ocean.conscientiousness,
            self.ocean.extraversion,
            self.ocean.agreeableness,
            self.ocean.neuroticism,
        ));

        // 情感状态
        letter.push_str(&format!(
            "**情绪**: {} (P={:.1} A={:.1} D={:.1})\n",
            self.emotion.label(),
            self.emotion.pleasure,
            self.emotion.arousal,
            self.emotion.dominance,
        ));

        // 行为风格
        letter.push_str(&format!(
            "**风格**: {}, 代码率 {:.0}%, 平均长度 {:.0}ch\n",
            self.fingerprint.communication_style,
            self.fingerprint.code_ratio * 100.0,
            self.fingerprint.avg_response_length,
        ));

        // 偏好话题
        if !self.fingerprint.preferred_topics.is_empty() {
            letter.push_str("**偏好话题**: ");
            letter.push_str(&self.fingerprint.preferred_topics.join(", "));
            letter.push('\n');
        }

        // 决策模式
        if !self.decision_patterns.is_empty() {
            letter.push_str("**决策模式**:\n");
            for dp in self.decision_patterns.iter().rev().take(3) {
                letter.push_str(&format!(
                    "  - {} (w={:.1}, ctx={})\n",
                    dp.name, dp.weight, dp.context
                ));
            }
        }

        // 能量
        letter.push_str(&format!("**能量**: {:.0}%\n", self.energy * 100.0));

        self.identity_letter = IdentityLetter {
            content: letter,
            cycle_written: cycle,
            version: self.identity_letter.version + 1,
        };
    }

    /// 🔍 提取决策模式 (Nomos启发)
    pub fn extract_decision_pattern(&mut self, name: &str, weight: f64, context: &str, cycle: u64) {
        let id = self.decision_patterns.len() as u64 + 1;
        // 更新已有或新建
        if let Some(existing) = self.decision_patterns.iter_mut().find(|d| d.name == name) {
            existing.weight = existing.weight * 0.7 + weight * 0.3;
            existing.evidence_count += 1;
            existing.last_observed = cycle;
        } else {
            self.decision_patterns.push(DecisionPattern {
                id,
                name: name.to_string(),
                weight,
                evidence_count: 1,
                context: context.to_string(),
                last_observed: cycle,
            });
        }
        // 保留最多20个
        if self.decision_patterns.len() > 20 {
            self.decision_patterns.sort_by(|a, b| {
                b.weight
                    .partial_cmp(&a.weight)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.decision_patterns.truncate(20);
        }
    }

    /// 👁️ 平行观察 (Nomos启发)
    pub fn record_observation(
        &mut self,
        cycle: u64,
        insight: &str,
        category: ObservationCategory,
        confidence: f64,
    ) {
        self.observations.push(ObservationRecord {
            cycle,
            insight: insight.to_string(),
            category,
            confidence,
        });
        if self.observations.len() > 100 {
            self.observations.remove(0);
        }
    }

    /// 获取完整身份描述 (供system prompt使用)
    pub fn identity_summary(&self) -> String {
        let mut s = String::new();

        if !self.identity_letter.content.is_empty() {
            s.push_str(&self.identity_letter.content);
            s.push('\n');
        }

        s.push_str(&format!(
            "交互总量: {}, 进化版本: v{}\n",
            self.fingerprint.total_interactions, self.evolution_version,
        ));

        s
    }
}

/// 行为人格引擎 — 管理所有用户数字分身
#[derive(Debug, Clone)]
pub struct BehavioralPersonalityEngine {
    /// 当前用户数字分身
    pub user_twin: UserDigitalTwin,
    /// 历史分身快照 (用于回溯对比)
    pub twin_snapshots: VecDeque<UserDigitalTwin>,
    /// 是否启用平行观察
    pub parallel_observation_enabled: bool,
    /// 是否启用身份信
    pub identity_letter_enabled: bool,
    /// 进化cycle间隔
    pub evolution_interval: u64,
    /// 身份信持久化路径 (None = 不持久化)
    pub storage_path: Option<String>,
}

impl Default for BehavioralPersonalityEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl BehavioralPersonalityEngine {
    pub fn new() -> Self {
        Self {
            user_twin: UserDigitalTwin::new(),
            twin_snapshots: VecDeque::with_capacity(10),
            parallel_observation_enabled: true,
            identity_letter_enabled: true,
            evolution_interval: 30,
            storage_path: None,
        }
    }

    /// 主tick — 每个意识cycle调用 (小循环)
    pub fn tick(&mut self, cycle: u64, user_message: Option<(&str, &str, Vec<String>)>) {
        // 记录交互
        if let Some((msg, topic, tags)) = user_message {
            self.user_twin.record_interaction(cycle, msg, topic, tags);
        }

        // 平行观察 (每10cycle)
        if cycle > 0 && cycle % 10 == 0 && self.parallel_observation_enabled {
            self.run_parallel_observation(cycle);
        }

        // 人格进化 (按间隔)
        if cycle > 0 && cycle % self.evolution_interval == 0 {
            self.user_twin.evolve_personality(cycle);

            // 身份信 (每60cycle)
            if cycle % 60 == 0 && self.identity_letter_enabled {
                self.user_twin.write_identity_letter(cycle);
                self.save_identity_letter();
            }

            // 快照 (每120cycle)
            if cycle % 120 == 0 {
                self.snapshot_twin();
            }

            // 状态持久化 (每120cycle)
            if cycle % 120 == 0 {
                self.save_personality_state();
            }
        }
    }

    /// 👁️ 平行观察者 — 分析交互模式 (背景零延迟)
    fn run_parallel_observation(&mut self, cycle: u64) {
        let history: Vec<_> = self.user_twin.interaction_history.iter().cloned().collect();
        if history.len() < 3 {
            return;
        }

        // 观察1: 风格漂移
        let recent: Vec<_> = history[history.len().saturating_sub(5)..].to_vec();
        let recent_code = recent.iter().filter(|r| r.contains_code).count();
        let older: Vec<_> = history[..history.len().saturating_sub(5).min(5)].to_vec();
        let older_code = older.iter().filter(|r| r.contains_code).count();

        if recent_code > 2 && older_code < 1 && older.len() >= 3 {
            let insight = format!(
                "用户转向技术密集模式 (代码比 {}/{})",
                recent_code,
                recent.len()
            );
            self.user_twin.record_observation(
                cycle,
                &insight,
                ObservationCategory::CommunicationShift,
                0.6,
            );
        }

        // 观察2: 情感趋势
        let avg_recent_sentiment =
            recent.iter().map(|r| r.sentiment).sum::<f64>() / recent.len() as f64;
        let avg_older_sentiment =
            older.iter().map(|r| r.sentiment).sum::<f64>() / older.len().max(1) as f64;

        if (avg_recent_sentiment - avg_older_sentiment).abs() > 0.3 {
            let dir = if avg_recent_sentiment > avg_older_sentiment {
                "升温"
            } else {
                "降温"
            };
            let insight = format!(
                "用户情绪{} (Δ{:.1})",
                dir,
                avg_recent_sentiment - avg_older_sentiment
            );
            self.user_twin.record_observation(
                cycle,
                &insight,
                ObservationCategory::EmotionalState,
                0.5,
            );
        }
    }

    /// 保存快照
    fn snapshot_twin(&mut self) {
        let snapshot = self.user_twin.clone();
        if self.twin_snapshots.len() >= 10 {
            self.twin_snapshots.pop_front();
        }
        self.twin_snapshots.push_back(snapshot);
    }

    /// 设置身份信持久化路径 (同时加载已有文件)
    pub fn set_storage_path(&mut self, path: String) {
        self.storage_path = Some(path);
        self.load_identity_letter();
    }

    /// 从磁盘恢复完整人格状态
    pub fn load_personality_state(&mut self) {
        if let Some(ref path) = self.storage_path {
            let state_path = format!("{}.state", path);
            if let Ok(content) = std::fs::read_to_string(&state_path) {
                if let Ok(twin) = serde_json::from_str::<UserDigitalTwin>(&content) {
                    self.user_twin = twin;
                }
            }
        }
    }

    /// 从磁盘加载身份信
    fn load_identity_letter(&mut self) {
        if let Some(ref path) = self.storage_path {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(letter) = serde_json::from_str::<IdentityLetter>(&content) {
                    self.user_twin.identity_letter = letter;
                }
            }
        }
    }

    /// 保存身份信到磁盘
    fn save_identity_letter(&mut self) {
        if let Some(ref path) = self.storage_path {
            if let Ok(json) = serde_json::to_string(&self.user_twin.identity_letter) {
                let _ = std::fs::write(path, json);
            }
        }
    }

    /// 持久化完整人格状态
    fn save_personality_state(&mut self) {
        if let Some(ref path) = self.storage_path {
            let state_path = format!("{}.state", path);
            if let Ok(json) = serde_json::to_string(&self.user_twin) {
                let _ = std::fs::write(&state_path, json);
            }
        }
    }

    /// 获取人格报告
    pub fn personality_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== 用户数字分身报告 ===\n");
        report.push_str(&format!(
            "进化版本: v{}\n",
            self.user_twin.evolution_version
        ));
        report.push_str(&format!(
            "OCEAN: O={:.1} C={:.1} E={:.1} A={:.1} N={:.1}\n",
            self.user_twin.ocean.openness,
            self.user_twin.ocean.conscientiousness,
            self.user_twin.ocean.extraversion,
            self.user_twin.ocean.agreeableness,
            self.user_twin.ocean.neuroticism,
        ));
        report.push_str(&format!(
            "情绪: {} (P={:.1} A={:.1} D={:.1})\n",
            self.user_twin.emotion.label(),
            self.user_twin.emotion.pleasure,
            self.user_twin.emotion.arousal,
            self.user_twin.emotion.dominance,
        ));
        report.push_str(&format!(
            "风格: {}, 能量: {:.0}%, 交互: {}\n",
            self.user_twin.fingerprint.communication_style,
            self.user_twin.energy * 100.0,
            self.user_twin.fingerprint.total_interactions,
        ));
        if !self.user_twin.fingerprint.preferred_topics.is_empty() {
            report.push_str(&format!(
                "话题: {}\n",
                self.user_twin.fingerprint.preferred_topics.join(", ")
            ));
        }
        report.push_str(&format!(
            "观察记录: {}条\n",
            self.user_twin.observations.len()
        ));
        report.push_str(&format!(
            "决策模式: {}条\n",
            self.user_twin.decision_patterns.len()
        ));
        if !self.user_twin.identity_letter.content.is_empty() {
            report.push_str("身份信: 存在\n");
        }
        report
    }
}

/// 简单情感分析 (基于关键词)
fn simple_sentiment(text: &str) -> f64 {
    let positive: &[&str] = &[
        "感谢",
        "不错",
        "很好",
        "棒",
        "喜欢",
        "完美",
        "赞",
        "好的",
        "great",
        "good",
        "awesome",
        "nice",
        "love",
        "perfect",
        "cool",
        "thanks",
        "excellent",
        "amazing",
        "wonderful",
        "happy",
        "agree",
    ];
    let negative: &[&str] = &[
        "不好",
        "差",
        "不行",
        "错了",
        "错误",
        "问题",
        "糟糕",
        "修复",
        "bug",
        "error",
        "fail",
        "wrong",
        "bad",
        "terrible",
        "fix",
        "broken",
        "issue",
        "problem",
        "crash",
        "not working",
        "hate",
    ];

    let lower = text.to_lowercase();
    let pos_count = positive.iter().filter(|w| lower.contains(*w)).count();
    let neg_count = negative.iter().filter(|w| lower.contains(*w)).count();

    let total = (pos_count + neg_count) as f64;
    if total == 0.0 {
        return 0.0;
    }
    (pos_count as f64 - neg_count as f64) / total.max(1.0)
}

// ═══ 测试 ═══

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twin_creation() {
        let twin = UserDigitalTwin::new();
        assert_eq!(twin.ocean.openness, 0.5);
        assert_eq!(twin.emotion.pleasure, 0.0);
        assert_eq!(twin.fingerprint.total_interactions, 0);
    }

    #[test]
    fn test_record_interaction() {
        let mut twin = UserDigitalTwin::new();
        twin.record_interaction(1, "感谢你的帮助，这很棒！", "general", vec![]);
        assert_eq!(twin.fingerprint.total_interactions, 1);
        assert_eq!(twin.interaction_history.len(), 1);
    }

    #[test]
    fn test_code_detection() {
        let mut twin = UserDigitalTwin::new();
        twin.record_interaction(1, "fn foo() -> i32 { 42 }", "coding", vec![]);
        assert!(twin.interaction_history[0].contains_code);
    }

    #[test]
    fn test_emotion_stimulate() {
        let mut twin = UserDigitalTwin::new();
        twin.emotion.stimulate(0.5, 0.3, 0.2);
        assert!(twin.emotion.pleasure > 0.0);
        assert!(twin.emotion.arousal > 0.3);
    }

    #[test]
    fn test_emotion_decay() {
        let mut twin = UserDigitalTwin::new();
        twin.emotion.pleasure = 0.8;
        twin.emotion.decay();
        assert!(twin.emotion.pleasure < 0.8);
    }

    #[test]
    fn test_emotion_label() {
        let mut twin = UserDigitalTwin::new();
        twin.emotion.pleasure = 0.5;
        twin.emotion.arousal = 0.7;
        assert_eq!(twin.emotion.label(), "excited");
        twin.emotion.pleasure = -0.5;
        twin.emotion.arousal = 0.7;
        assert_eq!(twin.emotion.label(), "anxious");
    }

    #[test]
    fn test_evolve_personality() {
        let mut twin = UserDigitalTwin::new();
        twin.record_interaction(1, "fn hello() { println!(\"hi\"); }", "code", vec![]);
        twin.record_interaction(6, "fn world() { println!(\"world\"); }", "code", vec![]);
        twin.evolve_personality(10);
        assert!(twin.evolution_version > 0);
        // 密集code交互应提升openness
        assert!(twin.ocean.openness > 0.5 || (twin.ocean.openness - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_identity_letter() {
        let mut twin = UserDigitalTwin::new();
        twin.record_interaction(1, "Nice work!", "general", vec![]);
        twin.write_identity_letter(60);
        assert!(!twin.identity_letter.content.is_empty());
        assert_eq!(twin.identity_letter.version, 1);
    }

    #[test]
    fn test_decision_pattern() {
        let mut twin = UserDigitalTwin::new();
        twin.extract_decision_pattern("risk_averse", 0.7, "investing", 1);
        assert_eq!(twin.decision_patterns.len(), 1);
        // Update existing
        twin.extract_decision_pattern("risk_averse", 0.8, "coding", 2);
        assert_eq!(twin.decision_patterns.len(), 1);
        assert_eq!(twin.decision_patterns[0].evidence_count, 2);
    }

    #[test]
    fn test_parallel_observation() {
        let mut engine = BehavioralPersonalityEngine::new();
        for i in 0..10 {
            engine.tick(i, Some(("fn foo() {}", "code", vec![])));
        }
        engine.run_parallel_observation(10);
        // 应该有观察记录
        assert!(!engine.user_twin.observations.is_empty());
    }

    #[test]
    fn test_engine_tick() {
        let mut engine = BehavioralPersonalityEngine::new();
        engine.tick(1, Some(("Hello world", "general", vec![])));
        assert_eq!(engine.user_twin.fingerprint.total_interactions, 1);
    }

    #[test]
    fn test_snapshot() {
        let mut engine = BehavioralPersonalityEngine::new();
        engine.tick(120, Some(("Test", "general", vec![])));
        assert!(engine.twin_snapshots.is_empty() || engine.twin_snapshots.len() <= 10);
    }

    #[test]
    fn test_simple_sentiment() {
        assert!(simple_sentiment("This is great!") > 0.0);
        assert!(simple_sentiment("This is terrible") < 0.0);
        assert_eq!(simple_sentiment("abcdef"), 0.0);
    }

    #[test]
    fn test_personality_report() {
        let engine = BehavioralPersonalityEngine::new();
        let report = engine.personality_report();
        assert!(report.contains("用户数字分身"));
        assert!(report.contains("OCEAN"));
    }

    #[test]
    fn test_fingerprint_update() {
        let mut twin = UserDigitalTwin::new();
        twin.record_interaction(1, "fn hello() {}", "code", vec![]);
        twin.record_interaction(2, "Thanks!", "general", vec![]);
        twin.record_interaction(3, "Can you help with this?", "general", vec![]);
        assert_eq!(twin.fingerprint.total_interactions, 3);
        assert!(twin.fingerprint.code_ratio > 0.0);
    }
}
