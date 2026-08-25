//! 意义建构器 — 开放性意义的动态建构与维护
//!
//! 第一性原理拆解:
//! - 意义不是预先给定的,而是在「观察-解释-行动」循环中动态涌现的
//! - 意义建构 = 模式识别 + 价值投射 + 叙事编织
//! - 核心能力: 模式提取 → 价值锚定 → 叙事生成 → 一致性维护

use crate::core::l7_capability::native_bus::{closure_capability, NativeCapability, NativeBusHandle};
use crate::core::nt_core_dao_engine::{DaoEngine, DaoEngineConfig, ExprNode};
use crate::core::nt_core_consciousness_tree::ConsciousnessTree;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};

/// 意义单元 — 最小意义承载单元
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MeaningUnit {
    pub id: String,
    /// 触发模式 (语义指纹)
    pub pattern: MeaningPattern,
    /// 价值锚点
    pub value_anchors: Vec<String>,
    /// 叙事片段
    pub narrative_fragment: String,
    /// 关联语境
    pub context_tags: Vec<String>,
    /// 置信度 [0,1]
    pub confidence: f64,
    /// 创建/更新时间戳
    pub timestamp: i64,
    /// 版本
    pub version: u32,
}

/// 意义模式 — 语义指纹
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MeaningPattern {
    /// 实体关系: (主体, 关系, 客体)
    EntityRelation(String, String, String),
    /// 概念聚类: 关键词集合
    ConceptCluster(Vec<String>),
    /// 时序模式: 事件序列
    TemporalSequence(Vec<String>),
    /// 因果链: 原因 -> 结果
    CausalChain(String, String),
    /// 价值冲突: 价值观 A vs 价值观 B
    ValueConflict(String, String),
    /// 意义缺口: 已知 vs 未知
    MeaningGap(String),
}

/// 意义建构器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeaningConstructorConfig {
    /// 最小模式支持度
    pub min_pattern_support: f64,
    /// 叙事连贯性阈值
    pub narrative_coherence_threshold: f64,
    /// 价值观锚定强度阈值
    pub value_anchor_threshold: f64,
    /// 最大叙事长度
    pub max_narrative_length: usize,
    /// 一致性检查窗口
    pub consistency_window: usize,
}

impl Default for MeaningConstructorConfig {
    fn default() -> Self {
        Self {
            min_pattern_support: 0.3,
            narrative_coherence_threshold: 0.7,
            value_anchor_threshold: 0.6,
            max_narrative_length: 1000,
            consistency_window: 50,
        }
    }
}

/// 意义建构器核心
pub struct MeaningConstructor {
    config: MeaningConstructorConfig,
    /// 意义单元存储 (LRU + 重要性加权)
    units: BTreeMap<String, MeaningUnit>,
    /// 模式索引 (快速检索)
    pattern_index: HashMap<String, Vec<String>>,
    /// 价值观权重 (动态调整)
    value_weights: HashMap<String, f64>,
    /// 叙事缓存 (最近 N 个叙事片段)
    narrative_buffer: VecDeque<String>,
    /// 一致性校验器
    consistency_checker: ConsistencyChecker,
    /// 统计信息
    stats: MeaningStats,
}

impl MeaningConstructor {
    pub fn new(config: MeaningConstructorConfig) -> Self {
        Self {
            config,
            units: BTreeMap::new(),
            pattern_index: HashMap::new(),
            value_weights: HashMap::new(),
            narrative_buffer: VecDeque::with_capacity(1000),
            consistency_checker: ConsistencyChecker::new(),
            stats: MeaningStats::default(),
        }
    }

    /// 从原始观察中提取意义单元
    pub fn extract_meaning(&mut self, observation: &str, context: &MeaningContext) -> Result<Vec<MeaningUnit>, String> {
        let mut units = Vec::new();
        
        // 1. 模式识别
        let patterns = self.recognize_patterns(observation, context)?;
        
        // 2. 价值锚定
        for pattern in patterns {
            if let Some(unit) = self.anchor_value(pattern, context)? {
                units.push(unit);
            }
        }
        
        // 3. 叙事编织
        let narrative = self.weave_narrative(&units)?;
        
        // 4. 一致性检查
        if self.config.narrative_coherence_threshold > 0.0 {
            self.consistency_checker.check(&narrative)?;
        }
        
        // 4. 存储与索引
        for unit in &units {
            self.store_unit(unit.clone())?;
        }
        
        self.stats.total_extracted += units.len() as u64;
        Ok(units)
    }

    /// 模式识别 (启发式 + VSA 语义相似度)
    fn recognize_patterns(&self, text: &str, context: &MeaningContext) -> Result<Vec<MeaningPattern>, String> {
        let mut patterns = Vec::new();
        
        // 实体关系抽取 (简化: 依存句法占位)
        // TODO: 接入 NLP pipeline
        
        // 概念聚类 (关键词共现)
        let keywords = self.extract_keywords(text);
        if keywords.len() >= 2 {
            patterns.push(MeaningPattern::ConceptCluster(keywords));
        }
        
        // 时序模式检测
        if let Some(seq) = self.detect_temporal_sequence(text) {
            patterns.push(MeaningPattern::TemporalSequence(seq));
        }
        
        // 因果链检测
        if let Some((cause, effect)) = self.detect_causal_chain(text) {
            patterns.push(MeaningPattern::CausalChain(cause, effect));
        }
        
        // 价值冲突检测
        if let Some((v1, v2)) = self.detect_value_conflict(text) {
            patterns.push(MeaningPattern::ValueConflict(v1, v2));
        }
        
        // 意义缺口检测
        if self.detect_meaning_gap(text) {
            patterns.push(MeaningPattern::MeaningGap("隐含前提缺失".into()));
        }
        
        Ok(patterns)
    }

    fn extract_keywords(&self, text: &str) -> Vec<String> {
        text.split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2)
            .map(|s| s.to_lowercase())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    fn detect_temporal_sequence(&self, text: &str) -> Option<Vec<String>> {
        // 简化: 寻找时序连接词
        let markers = ["然后", "接着", "之后", "随后", "随后", "首先", "其次", "最后"];
        for marker in markers {
            if text.contains(marker) {
                let parts: Vec<String> = text.split(marker).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                if parts.len() >= 2 {
                    return Some(parts);
                }
            }
        }
        None
    }

    fn detect_causal_chain(&self, text: &str) -> Option<(String, String)> {
        let markers = ["因为", "由于", "导致", "引起", "所以", "因此", "从而"];
        for marker in markers {
            if let Some(idx) = text.find(marker) {
                let (cause, effect) = text.split_at(idx);
                return Some((cause.trim().to_string(), effect[marker.len()..].trim().to_string()));
            }
        }
        None
    }

    fn detect_value_conflict(&self, text: &str) -> Option<(String, String)> {
        let conflicts = [
            ("自由", "安全"),
            ("隐私", "便利"),
            ("公平", "效率"),
            ("自主", "保护"),
            ("真相", "和谐"),
        ];
        for (v1, v2) in conflicts {
            if text.contains(v1) && text.contains(v2) {
                return Some((v1.into(), v2.into()));
            }
        }
        None
    }

    fn detect_meaning_gap(&self, text: &str) -> bool {
        // 简化: 检测未量化主张
        text.contains("显然") || text.contains("显然") || text.contains("显然")
    }

    /// 价值锚定
    fn anchor_value(&mut self, pattern: MeaningPattern, context: &MeaningContext) -> Result<Option<MeaningUnit>, String> {
        let value_anchors = self.infer_value_anchors(&pattern, context)?;
        if value_anchors.is_empty() {
            return Ok(None);
        }

        let confidence = self.compute_confidence(&value_anchors);
        if confidence < self.config.value_anchor_threshold {
            return Ok(None);
        }

        let id = format!("mu_{}", uuid::Uuid::new_v4().simple());
        let narrative_fragment = self.generate_fragment(&value_anchors);
        
        let unit = MeaningUnit {
            id,
            pattern,
            value_anchors,
            narrative_fragment,
            context_tags: context.tags.clone(),
            confidence,
            timestamp: crate::core::nt_core_kb_primitives::now(),
            version: 1,
        };
        
        Ok(Some(MeaningUnit { ..Default::default() }))
    }

    fn infer_value_anchors(&self, pattern: &MeaningPattern, context: &MeaningContext) -> Result<Vec<String>, String> {
        let mut anchors = Vec::new();
        // 基于模式类型推断价值锚点
        match pattern {
            MeaningPattern::EntityRelation(_, rel, _) => {
                if rel.contains("保护") || rel.contains("保护") { anchors.push("保护".into()); }
                if rel.contains("尊重") || rel.contains("尊重") { anchors.push("尊重".into()); }
            }
            MeaningPattern::ValueConflict(v1, v2) => {
                anchors.push(v1.clone());
                anchors.push(v2.clone());
            }
            _ => {}
        }
        Ok(anchors)
    }

    fn compute_confidence(&self, anchors: &[String]) -> f64 {
        if anchors.is_empty() { return 0.0; }
        let mut weight = 0.0;
        for a in anchors {
            weight += self.value_weights.get(a).copied().unwrap_or(0.5);
        }
        (weight / anchors.len() as f64).min(1.0)
    }

    fn generate_fragment(&self, anchors: &[String]) -> String {
        anchors.join(" · ")
    }

    fn weave_narrative(&mut self, units: &[MeaningUnit]) -> Result<String, String> {
        // 简化: 按时间序拼接
        let mut fragments: Vec<String> = units.iter().map(|u| u.narrative_fragment.clone()).collect();
        if fragments.len() > self.config.max_narrative_length {
            fragments.truncate(self.config.max_narrative_length);
        }
        let narrative = fragments.join(" → ");
        self.narrative_buffer.push_back(fragments.join(" → "));
        if self.narrative_buffer.len() > 100 {
            self.narrative_buffer.pop_front();
        }
        Ok(fragments.join(" → "))
    }

    fn store_unit(&mut self, unit: MeaningUnit) -> Result<(), String> {
        // 索引
        self.pattern_index.entry(format!("{:?}", unit.pattern)).or_default().push(unit.id.clone());
        self.units.insert(unit.id.clone(), unit);
        Ok(())
    }

    pub fn query(&self, query: &str, limit: usize) -> Vec<MeaningUnit> {
        let keywords = self.extract_keywords(query);
        let mut candidates: Vec<_> = self.units.values().collect();
        candidates.sort_by(|a, b| {
            let score_a = self.compute_relevance(a, &query);
            let score_b = self.compute_relevance(b, &query);
            score_b.partial_cmp(&score_a).unwrap()
        });
        candidates.into_iter().take(limit).cloned().collect()
    }

    fn compute_relevance(&self, unit: &MeaningUnit, query: &str) -> f64 {
        let query_words: HashSet<_> = unit.pattern.to_string().split_whitespace().collect();
        let query_words: HashSet<_> = query.split_whitespace().collect();
        let inter = unit.pattern.to_string().split_whitespace().filter(|w| query.contains(w)).count();
        inter as f64 / query_words.len().max(1) as f64
    }

    pub fn stats(&self) -> MeaningStats {
        self.stats.clone()
    }
}

/// 语境信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MeaningContext {
    pub tags: Vec<String>,
    pub domain: Option<String>,
    pub timestamp: i64,
}

/// 一致性校验器
#[derive(Debug, Clone)]
struct ConsistencyChecker {
    contradictions: Vec<(String, String)>,
}

impl ConsistencyChecker {
    fn new() -> Self {
        Self { contradictions: Vec::new() }
    }

    fn check(&mut self, narrative: &str) -> Result<(), String> {
        // 简化: 检测明显矛盾
        if narrative.contains("既") && narrative.contains("不") {
            return Err("检测到潜在矛盾".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeaningStats {
    pub total_extracted: u64,
    pub patterns_recognized: u64,
    pub narratives_generated: u64,
    pub conflicts_detected: u64,
}

/// 意义建构器能力 — 包装为 NativeCapability
pub fn meaning_constructor_capability(config: MeaningConstructorConfig) -> Arc<dyn crate::core::l7_capability::native_bus::NativeCapability> {
    let constructor = Arc::new(RwLock::new(MeaningConstructor::new(config)));
    closure_capability(
        "meaning.constructor",
        "意义建构器",
        r#"{"observation": "string", "context": {"tags": "array", "domain": "string"}}"#,
        r#"{"units": "array"}"#,
        true,
        move |input| {
            let mc = constructor.clone();
            let observation = input.get("observation").and_then(|v| v.as_str()).unwrap_or("");
            let context = input.get("context").cloned().unwrap_or_default();
            let mut mc = mc.write().unwrap();
            let units = mc.extract_meaning(observation, &serde_json::from_value(context).unwrap_or_default())?;
            Ok(serde_json::json!({ "units": units }))
        })
    }
}
}
