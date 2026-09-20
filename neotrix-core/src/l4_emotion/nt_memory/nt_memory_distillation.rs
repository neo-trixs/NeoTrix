#![forbid(unsafe_code)]

//! 层级蒸馏管线 — 基于 TencentDB 层级蒸馏模式
//!
//! L0 原始对话 → L1 原子事实 → L2 场景知识 → L3 人格画像
//!
//! 每层蒸馏产生结构化中间表示，下游可独立消费：
//! - L1 原子事实 → 知识图谱写入 (KB nodes/edges)
//! - L2 场景知识 → 检索增强 (RAG context)
//! - L3 人格画像 → 个性化路由 (GWT salience)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ─── L0: 原始对话 ───────────────────────────────────────────

/// 蒸馏层级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DistillationLevel {
    L0RawConversation,
    L1AtomicFacts,
    L2SceneKnowledge,
    L3PersonalityProfile,
}

/// L0: 原始对话 — 一次完整交互的原始记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawConversation {
    pub id: String,
    pub session_id: String,
    pub messages: Vec<ConversationMessage>,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub metadata: Option<serde_json::Value>,
}

/// 对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: i64,
}

/// 消息角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

// ─── L1: 原子事实 ───────────────────────────────────────────

/// L1: 原子事实 — 从对话中提取的不可再分的断言
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicFact {
    pub id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f64,
    pub source_conv_id: String,
    pub source_msg_idx: usize,
    pub extracted_at: i64,
    pub fact_type: FactType,
}

/// 事实类型分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactType {
    /// 实体属性 (X 是 Y)
    Attribute,
    /// 关系断言 (X 有 Y)
    Relation,
    /// 行为记录 (X 做了 Y)
    Action,
    /// 偏好表达 (X 喜欢 Y)
    Preference,
    /// 上下文状态 (X 处于 Y)
    State,
}

// ─── L2: 场景知识 ───────────────────────────────────────────

/// L2: 场景知识 — 按主题/场景聚合的结构化知识包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneKnowledge {
    pub id: String,
    pub scene_name: String,
    pub scene_type: SceneType,
    pub fact_ids: Vec<String>,
    pub summary: String,
    pub context_window: String,
    pub importance: f64,
    pub created_at: i64,
}

/// 场景类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SceneType {
    /// 技术讨论
    Technical,
    /// 个人偏好
    Preference,
    /// 项目上下文
    Project,
    /// 工作流模式
    Workflow,
    /// 知识探索
    Exploration,
}

// ─── L3: 人格画像 ───────────────────────────────────────────

/// L3: 人格画像 — 从场景知识中提炼的用户/代理特征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityProfile {
    pub id: String,
    pub traits: Vec<Trait>,
    pub preferences: Vec<Preference>,
    pub behavioral_patterns: Vec<BehavioralPattern>,
    pub communication_style: CommunicationStyle,
    pub confidence: f64,
    pub sample_size: usize,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 特征维度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trait {
    pub name: String,
    pub strength: f64,
    pub evidence_count: usize,
}

/// 偏好项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preference {
    pub domain: String,
    pub item: String,
    pub weight: f64,
}

/// 行为模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralPattern {
    pub pattern_name: String,
    pub trigger: String,
    pub response: String,
    pub frequency: f64,
}

/// 沟通风格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStyle {
    pub formality: f64,
    pub verbosity: f64,
    pub technical_depth: f64,
    pub language: String,
}

// ─── 蒸馏管线 ───────────────────────────────────────────────

/// 蒸馏管线统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DistillationStats {
    pub l0_count: usize,
    pub l1_count: usize,
    pub l2_count: usize,
    pub l3_count: usize,
    pub last_distilled_at: Option<i64>,
}

/// 层级蒸馏管线
pub struct HierarchicalDistiller {
    raw_conversations: Vec<RawConversation>,
    atomic_facts: Vec<AtomicFact>,
    scene_knowledge: Vec<SceneKnowledge>,
    personality_profiles: Vec<PersonalityProfile>,
    stats: DistillationStats,
    config: DistillationConfig,
}

/// 蒸馏配置
#[derive(Debug, Clone)]
pub struct DistillationConfig {
    /// 原子事实提取: 最小句子长度
    pub min_sentence_len: usize,
    /// 场景聚合: 最小事实数阈值
    pub scene_min_facts: usize,
    /// 人格画像: 最小场景数阈值
    pub profile_min_scenes: usize,
    /// 置信度衰减因子
    pub confidence_decay: f64,
}

impl Default for DistillationConfig {
    fn default() -> Self {
        Self {
            min_sentence_len: 4,
            scene_min_facts: 2,
            profile_min_scenes: 3,
            confidence_decay: 0.95,
        }
    }
}

impl HierarchicalDistiller {
    pub fn new() -> Self {
        Self::with_config(DistillationConfig::default())
    }

    pub fn with_config(config: DistillationConfig) -> Self {
        Self {
            raw_conversations: Vec::new(),
            atomic_facts: Vec::new(),
            scene_knowledge: Vec::new(),
            personality_profiles: Vec::new(),
            stats: DistillationStats::default(),
            config,
        }
    }

    /// 添加原始对话并触发蒸馏
    pub fn ingest_conversation(&mut self, conv: RawConversation) {
        self.raw_conversations.push(conv);
        self.distill_all();
    }

    /// 批量添加原始对话
    pub fn ingest_batch(&mut self, convs: Vec<RawConversation>) {
        self.raw_conversations.extend(convs);
        self.distill_all();
    }

    /// 获取蒸馏统计
    pub fn stats(&self) -> &DistillationStats {
        &self.stats
    }

    /// 获取所有原子事实
    pub fn atomic_facts(&self) -> &[AtomicFact] {
        &self.atomic_facts
    }

    /// 获取所有场景知识
    pub fn scene_knowledge(&self) -> &[SceneKnowledge] {
        &self.scene_knowledge
    }

    /// 获取所有人格画像
    pub fn personality_profiles(&self) -> &[PersonalityProfile] {
        &self.personality_profiles
    }

    // ─── 完整蒸馏流程 ───────────────────────────────────────

    fn distill_all(&mut self) {
        self.atomic_facts.clear();
        self.scene_knowledge.clear();
        self.personality_profiles.clear();

        self.distill_l0_to_l1();
        self.distill_l1_to_l2();
        self.distill_l2_to_l3();

        self.stats.l0_count = self.raw_conversations.len();
        self.stats.l1_count = self.atomic_facts.len();
        self.stats.l2_count = self.scene_knowledge.len();
        self.stats.l3_count = self.personality_profiles.len();
        self.stats.last_distilled_at = Some(now_ts());
    }

    // ─── L0 → L1: 原子事实提取 ──────────────────────────────

    fn distill_l0_to_l1(&mut self) {
        for conv in &self.raw_conversations {
            for (idx, msg) in conv.messages.iter().enumerate() {
                if msg.role == MessageRole::System {
                    continue;
                }
                let facts = extract_atomic_facts(msg, &conv.id, idx);
                self.atomic_facts.extend(facts);
            }
        }
    }

    // ─── L1 → L2: 场景知识聚合 ──────────────────────────────

    fn distill_l1_to_l2(&mut self) {
        // 按 subject 聚合原子事实
        let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, fact) in self.atomic_facts.iter().enumerate() {
            groups.entry(fact.subject.clone()).or_default().push(i);
        }

        for (subject, fact_indices) in groups {
            if fact_indices.len() < self.config.scene_min_facts {
                continue;
            }

            let scene_type = infer_scene_type(&self.atomic_facts, &fact_indices);
            let summary = build_scene_summary(&self.atomic_facts, &fact_indices);
            let importance = compute_scene_importance(&self.atomic_facts, &fact_indices);
            let fact_ids: Vec<String> = fact_indices
                .iter()
                .map(|&i| self.atomic_facts[i].id.clone())
                .collect();

            let knowledge = SceneKnowledge {
                id: format!("scene_{}", uuid_simple()),
                scene_name: subject,
                scene_type,
                fact_ids,
                summary,
                context_window: String::new(),
                importance,
                created_at: now_ts(),
            };
            self.scene_knowledge.push(knowledge);
        }
    }

    // ─── L2 → L3: 人格画像提炼 ──────────────────────────────

    fn distill_l2_to_l3(&mut self) {
        if self.scene_knowledge.len() < self.config.profile_min_scenes {
            return;
        }

        let traits = extract_traits(&self.scene_knowledge);
        let preferences = extract_preferences(&self.scene_knowledge);
        let behavioral_patterns = extract_behavioral_patterns(&self.scene_knowledge);
        let communication_style = infer_communication_style(&self.atomic_facts);
        let confidence = compute_profile_confidence(&self.scene_knowledge);

        let now = now_ts();
        let profile = PersonalityProfile {
            id: format!("profile_{}", uuid_simple()),
            traits,
            preferences,
            behavioral_patterns,
            communication_style,
            confidence,
            sample_size: self.scene_knowledge.len(),
            created_at: now,
            updated_at: now,
        };
        self.personality_profiles.push(profile);
    }
}

// ─── 提取函数 ───────────────────────────────────────────────

/// 从消息中提取原子事实 (基于规则的轻量提取, 无需 LLM)
fn extract_atomic_facts(
    msg: &ConversationMessage,
    conv_id: &str,
    msg_idx: usize,
) -> Vec<AtomicFact> {
    let mut facts = Vec::new();
    let sentences = split_sentences(&msg.content);
    let ts = now_ts();

    for sent in &sentences {
        if sent.len() < 4 {
            continue;
        }

        let (fact_type, subject, predicate, object) = match classify_sentence(sent) {
            Some(triple) => triple,
            None => continue,
        };

        facts.push(AtomicFact {
            id: format!("fact_{}", uuid_simple()),
            subject,
            predicate,
            object,
            confidence: 0.7,
            source_conv_id: conv_id.to_string(),
            source_msg_idx: msg_idx,
            extracted_at: ts,
            fact_type,
        });
    }
    facts
}

/// 简单句子分割
fn split_sentences(text: &str) -> Vec<String> {
    text.split(['。', '！', '？', '.', '!', '?', '\n'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// 基于规则的句子分类 → (FactType, subject, predicate, object)
fn classify_sentence(sent: &str) -> Option<(FactType, String, String, String)> {
    // 模式 1: "X 是 Y" / "X 为 Y"
    if let Some(pos) = sent.find("是") {
        let (left, right) = sent.split_at(pos);
        let subject = left.trim().to_string();
        let rest = right[1..].trim().to_string();
        if !subject.is_empty() && !rest.is_empty() {
            return Some((FactType::Attribute, subject, "是".into(), rest));
        }
    }
    if let Some(pos) = sent.find("为") {
        let (left, right) = sent.split_at(pos);
        let subject = left.trim().to_string();
        let rest = right[1..].trim().to_string();
        if !subject.is_empty() && !rest.is_empty() {
            return Some((FactType::Attribute, subject, "为".into(), rest));
        }
    }

    // 模式 2: "X 有 Y" / "X 包含 Y"
    for keyword in &["有", "包含", "使用", "采用", "支持"] {
        if let Some(pos) = sent.find(keyword) {
            let (left, right) = sent.split_at(pos);
            let subject = left.trim().to_string();
            let kw_len = keyword.len();
            let rest = right[kw_len..].trim().to_string();
            if !subject.is_empty() && !rest.is_empty() {
                let fact_type = if *keyword == "使用" || *keyword == "采用" {
                    FactType::Action
                } else {
                    FactType::Relation
                };
                return Some((fact_type, subject, keyword.to_string(), rest));
            }
        }
    }

    // 模式 3: "X 喜欢 Y" / "X 偏好 Y"
    for keyword in &["喜欢", "偏好", "倾向", "偏好于"] {
        if let Some(pos) = sent.find(keyword) {
            let (left, right) = sent.split_at(pos);
            let subject = left.trim().to_string();
            let kw_len = keyword.len();
            let rest = right[kw_len..].trim().to_string();
            if !subject.is_empty() && !rest.is_empty() {
                return Some((FactType::Preference, subject, keyword.to_string(), rest));
            }
        }
    }

    // 模式 4: 英文 "X is Y" / "X has Y"
    let lower = sent.to_lowercase();
    for (keyword, fact_type) in &[(" is ", FactType::Attribute), (" has ", FactType::Relation), (" uses ", FactType::Action), (" likes ", FactType::Preference)] {
        if let Some(pos) = lower.find(keyword) {
            let subject = sent[..pos].trim().to_string();
            let rest = sent[pos + keyword.len()..].trim().to_string();
            if !subject.is_empty() && !rest.is_empty() {
                return Some((*fact_type, subject, keyword.trim().to_string(), rest));
            }
        }
    }

    None
}

/// 推断场景类型
fn infer_scene_type(facts: &[AtomicFact], indices: &[usize]) -> SceneType {
    let type_counts = indices.iter().fold(
        HashMap::new(),
        |mut acc, &i| {
            *acc.entry(facts[i].fact_type).or_insert(0) += 1;
            acc
        },
    );

    let dominant = type_counts
        .iter()
        .max_by_key(|&(_, &count)| count)
        .map(|(&ft, _)| ft)
        .unwrap_or(FactType::Attribute);
    dominant.to_scene_type()
}

impl FactType {
    fn to_scene_type(&self) -> SceneType {
        match self {
            FactType::Attribute | FactType::Relation => SceneType::Technical,
            FactType::Preference => SceneType::Preference,
            FactType::Action => SceneType::Workflow,
            FactType::State => SceneType::Project,
        }
    }
}

/// 构建场景摘要
fn build_scene_summary(facts: &[AtomicFact], indices: &[usize]) -> String {
    let mut parts: Vec<String> = indices
        .iter()
        .map(|&i| format!("{}{}{}", facts[i].subject, facts[i].predicate, facts[i].object))
        .collect();
    parts.truncate(5);
    parts.join("; ")
}

/// 计算场景重要性 (基于事实数量和置信度)
fn compute_scene_importance(facts: &[AtomicFact], indices: &[usize]) -> f64 {
    let count = indices.len() as f64;
    let avg_conf: f64 = indices
        .iter()
        .map(|&i| facts[i].confidence)
        .sum::<f64>()
        / count.max(1.0);
    (count.ln() * avg_conf).min(1.0)
}

/// 从场景知识提取特征
fn extract_traits(scenes: &[SceneKnowledge]) -> Vec<Trait> {
    let mut trait_counts: HashMap<String, usize> = HashMap::new();
    for scene in scenes {
        *trait_counts.entry(scene.scene_name.clone()).or_insert(0) += 1;
    }
    trait_counts
        .into_iter()
        .map(|(name, count)| Trait {
            strength: (count as f64 / scenes.len() as f64).min(1.0),
            evidence_count: count,
            name,
        })
        .collect()
}

/// 从场景知识提取偏好
fn extract_preferences(scenes: &[SceneKnowledge]) -> Vec<Preference> {
    scenes
        .iter()
        .filter(|s| s.scene_type == SceneType::Preference)
        .map(|s| Preference {
            domain: s.scene_name.clone(),
            item: s.summary.clone(),
            weight: s.importance,
        })
        .collect()
}

/// 提取行为模式
fn extract_behavioral_patterns(scenes: &[SceneKnowledge]) -> Vec<BehavioralPattern> {
    scenes
        .iter()
        .filter(|s| s.scene_type == SceneType::Workflow)
        .map(|s| BehavioralPattern {
            pattern_name: s.scene_name.clone(),
            trigger: String::new(),
            response: s.summary.clone(),
            frequency: s.importance,
        })
        .collect()
}

/// 推断沟通风格
fn infer_communication_style(facts: &[AtomicFact]) -> CommunicationStyle {
    let total = facts.len().max(1) as f64;
    let technical = facts
        .iter()
        .filter(|f| f.fact_type == FactType::Attribute || f.fact_type == FactType::Relation)
        .count() as f64
        / total;
    CommunicationStyle {
        formality: 0.5,
        verbosity: 0.5,
        technical_depth: technical,
        language: "auto".to_string(),
    }
}

/// 计算画像置信度
fn compute_profile_confidence(scenes: &[SceneKnowledge]) -> f64 {
    if scenes.is_empty() {
        return 0.0;
    }
    let avg = scenes.iter().map(|s| s.importance).sum::<f64>() / scenes.len() as f64;
    let size_bonus = (scenes.len() as f64).ln() * 0.1;
    (avg + size_bonus).min(1.0)
}

// ─── 工具函数 ───────────────────────────────────────────────

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// 简单唯一 ID (无 uuid 依赖, 内存蒸馏不需要全局唯一)
fn uuid_simple() -> String {
    let ts = now_ts() as u64;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;
    format!("{:x}-{:x}", ts, nanos)
}

// ─── 测试 ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_conv() -> RawConversation {
        RawConversation {
            id: "conv_1".into(),
            session_id: "sess_1".into(),
            messages: vec![
                ConversationMessage {
                    role: MessageRole::User,
                    content: "我喜欢使用 Rust 编程语言。".into(),
                    timestamp: 1000,
                },
                ConversationMessage {
                    role: MessageRole::Assistant,
                    content: "Rust 是系统级编程语言。它支持内存安全。".into(),
                    timestamp: 1001,
                },
                ConversationMessage {
                    role: MessageRole::User,
                    content: "我偏好函数式编程风格。".into(),
                    timestamp: 1002,
                },
            ],
            started_at: 1000,
            ended_at: Some(1002),
            metadata: None,
        }
    }

    #[test]
    fn test_full_distillation_pipeline() {
        let mut distiller = HierarchicalDistiller::new();
        distiller.ingest_conversation(make_test_conv());

        let stats = distiller.stats();
        assert_eq!(stats.l0_count, 1);
        assert!(stats.l1_count > 0, "应提取到原子事实");
        assert!(stats.l2_count > 0, "应聚合出场景知识");

        let facts = distiller.atomic_facts();
        assert!(facts.iter().any(|f| f.fact_type == FactType::Preference));
        assert!(facts.iter().any(|f| f.fact_type == FactType::Attribute));
    }

    #[test]
    fn test_extract_atomic_facts_attribute() {
        let msg = ConversationMessage {
            role: MessageRole::User,
            content: "Rust 是系统级编程语言。".into(),
            timestamp: 1000,
        };
        let facts = extract_atomic_facts(&msg, "c1", 0);
        assert!(!facts.is_empty());
        assert_eq!(facts[0].fact_type, FactType::Attribute);
    }

    #[test]
    fn test_extract_atomic_facts_preference() {
        let msg = ConversationMessage {
            role: MessageRole::User,
            content: "我喜欢函数式编程。".into(),
            timestamp: 1000,
        };
        let facts = extract_atomic_facts(&msg, "c1", 0);
        assert!(!facts.is_empty());
        assert_eq!(facts[0].fact_type, FactType::Preference);
    }

    #[test]
    fn test_classify_english() {
        let sent = "The user likes Rust";
        let result = classify_sentence(sent);
        assert!(result.is_some());
        let (ft, _, _, _) = result.unwrap();
        assert_eq!(ft, FactType::Preference);
    }

    #[test]
    fn test_scene_aggregation() {
        let mut distiller = HierarchicalDistiller::new();
        let conv = RawConversation {
            id: "c1".into(),
            session_id: "s1".into(),
            messages: vec![
                ConversationMessage {
                    role: MessageRole::User,
                    content: "我使用 VS Code。我偏好 Vim 快捷键。".into(),
                    timestamp: 1000,
                },
            ],
            started_at: 1000,
            ended_at: None,
            metadata: None,
        };
        distiller.ingest_conversation(conv);
        let scenes = distiller.scene_knowledge();
        assert!(!scenes.is_empty(), "应有场景知识产出");
    }

    #[test]
    fn test_empty_conversation() {
        let mut distiller = HierarchicalDistiller::new();
        distiller.ingest_conversation(RawConversation {
            id: "empty".into(),
            session_id: "s".into(),
            messages: vec![],
            started_at: 0,
            ended_at: None,
            metadata: None,
        });
        let stats = distiller.stats();
        assert_eq!(stats.l1_count, 0);
        assert_eq!(stats.l2_count, 0);
        assert_eq!(stats.l3_count, 0);
    }

    #[test]
    fn test_stats_default() {
        let stats = DistillationStats::default();
        assert_eq!(stats.l0_count, 0);
        assert!(stats.last_distilled_at.is_none());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut distiller = HierarchicalDistiller::new();
        distiller.ingest_conversation(make_test_conv());
        let json = serde_json::to_string(distiller.stats()).unwrap();
        let parsed: DistillationStats = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.l0_count, 1);
    }
}
