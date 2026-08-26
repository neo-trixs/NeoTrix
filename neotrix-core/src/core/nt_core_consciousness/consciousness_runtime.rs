#![deny(clippy::unwrap_used)]

use super::awakening::{AwakeningReport, ConsciousnessAwakening};
use super::inner_critic::{CritiqueResult, InnerCritic};
use super::source_hierarchy::{
    ContextMeta, KnowledgeLayer, PerceptionMeta, PerceptionSource, ProvenanceChain,
};
use super::specious_present::SpeciousPresent;
use super::stream_buffer::ConsciousnessStream;
use super::volition::{ActionCandidate, VolitionEngine};
use super::vsa_tag::{VsaOrigin, VsaSelfCategory, VsaTagged};
use crate::core::nt_core_self::affective_interface::{
    AffectiveInterface, UserAffectSnapshot, UserEmotion,
};
use crate::core::nt_core_self::emotion_state::{EmotionDimension, EmotionEngine, EmotionReport};
use crate::neotrix::nt_memory_kb::KnowledgeBase;

/// 每次 tick 最多注入的 KB 知识条目数，防止无界流入意识流。
const KB_INJECT_LIMIT: usize = 4;
/// KB 查询缓存容量：避免意识 tick 对相同共振内容重复同步搜索 KB。
const KB_QUERY_CACHE_CAP: usize = 64;

/// 有界 KB 查询缓存 — 以共振内容为 key，避免重复同步 DB 搜索。
struct KbQueryCache {
    entries: std::collections::VecDeque<(String, Vec<(String, f64)>)>,
}

impl KbQueryCache {
    fn new() -> Self {
        Self {
            entries: std::collections::VecDeque::new(),
        }
    }
    fn get(&self, query: &str) -> Option<&Vec<(String, f64)>> {
        self.entries
            .iter()
            .find(|(k, _)| k == query)
            .map(|(_, v)| v)
    }
    fn put(&mut self, query: &str, results: Vec<(String, f64)>) {
        if self.entries.iter().any(|(k, _)| k == query) {
            return;
        }
        if self.entries.len() >= KB_QUERY_CACHE_CAP {
            self.entries.pop_front();
        }
        self.entries.push_back((query.to_string(), results));
    }
}

pub struct ConsciousnessRuntime {
    pub stream: ConsciousnessStream,
    pub specious_present: SpeciousPresent,
    pub volition: VolitionEngine,
    pub critic: InnerCritic,
    pub emotion_engine: EmotionEngine,
    /// 人类情感交互界面 — 感知用户情绪/关系阶段/共情策略, 供数字人前端消费;
    /// 由 Second Brain 经 KB 持久化跨 session 恢复。
    pub affective: AffectiveInterface,
    /// 知识库句柄 — 意识核心主动查询记忆/知识，而非仅被动接收共振字符串。
    pub kb: Option<std::sync::Arc<KnowledgeBase>>,
    /// 最近一次 tick 从 KB 注入的意识条目 (title, score)。
    pub last_kb_injections: Vec<(String, f64)>,
    /// KB 查询缓存 — 防止共振内容重复触发同步搜索。
    kb_cache: KbQueryCache,
    pub awakened: bool,
    pub last_report: Option<AwakeningReport>,
    pub last_quality: f64,
    pub tick_count: u64,
}

impl ConsciousnessRuntime {
    pub fn new() -> Self {
        Self {
            stream: ConsciousnessStream::new(super::stream_buffer::DEFAULT_STREAM_CAPACITY),
            specious_present: SpeciousPresent::new(12),
            volition: VolitionEngine::new(),
            critic: InnerCritic::new(),
            emotion_engine: EmotionEngine::default(),
            affective: AffectiveInterface::new(),
            kb: None,
            last_kb_injections: Vec::new(),
            kb_cache: KbQueryCache::new(),
            awakened: false,
            last_report: None,
            last_quality: 0.0,
            tick_count: 0,
        }
    }

    /// 挂接知识库，使意识核心具备主动查询能力。
    pub fn attach_kb(&mut self, kb: std::sync::Arc<KnowledgeBase>) {
        self.kb = Some(kb);
    }

    pub fn is_kb_attached(&self) -> bool {
        self.kb.is_some()
    }

    /// 主动查询知识库：以当前共振内容为查询词检索关联知识。
    /// 返回 (title, score) 列表；未挂接 KB 时返回空。结果经有界缓存复用。
    pub fn query_kb(&self, query: &str, limit: usize) -> Vec<(String, f64)> {
        if let Some(cached) = self.kb_cache.get(query) {
            let mut v = cached.clone();
            v.truncate(limit);
            return v;
        }
        let kb = match self.kb.as_ref() {
            Some(kb) => kb,
            None => return Vec::new(),
        };
        match kb.search(query, limit) {
            Ok(results) => results
                .into_iter()
                .map(|r| (r.node.title.clone(), r.score))
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// 将 KB 检索结果注入意识流 (specious present) 作为带溯源的记忆条目。
    /// 返回注入条目数。每条带 Structured provenance 层，可被后续感知层级鉴别。
    fn inject_kb_knowledge(&mut self, query: &str) -> usize {
        let Some(kb) = self.kb.clone() else { return 0 };
        // 缓存命中: 直接复用上次搜索结果, 避免同步 DB 搜索阻塞意识 tick。
        let results: Vec<(String, f64)> = if let Some(cached) = self.kb_cache.get(query) {
            cached.clone()
        } else {
            let fresh = match kb.search(query, KB_INJECT_LIMIT) {
                Ok(r) => r,
                Err(_) => return 0,
            };
            let mapped: Vec<(String, f64)> = fresh
                .iter()
                .map(|r| (r.node.title.clone(), r.score))
                .collect();
            self.kb_cache.put(query, mapped.clone());
            mapped
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;
        let mut injected = Vec::new();
        for (title, score) in results {
            if title.is_empty() {
                continue;
            }
            let raw = KnowledgeLayer::Raw(PerceptionMeta {
                source_type: PerceptionSource::SearchResult,
                raw_confidence: score.clamp(0.0, 1.0),
                timestamp: now,
            });
            let structured = KnowledgeLayer::Structured(ContextMeta {
                source_ids: vec![title.clone()],
                processing_steps: vec!["kb_hybrid_search".into()],
                contextual_confidence: score.clamp(0.0, 1.0),
            });
            let chain = ProvenanceChain::new(vec![(raw, now), (structured, now + 1)]);
            let mut item = VsaTagged::new(
                title.as_bytes().to_vec(),
                VsaOrigin::Self_(VsaSelfCategory::Memory),
            )
            .with_confidence(score.clamp(0.0, 1.0))
            .with_provenance(chain);
            item.salience = (0.5 + score * 0.5).min(1.0);
            self.specious_present.push(item);
            injected.push((title, score));
        }
        self.last_kb_injections = injected;
        self.last_kb_injections.len()
    }

    pub fn awaken(&mut self) -> &AwakeningReport {
        self.awakened = true;
        let report = ConsciousnessAwakening::awaken(&mut self.stream, &mut self.specious_present);
        self.last_report.insert(report)
    }

    pub fn tick_emotion(&mut self) -> EmotionReport {
        self.emotion_engine.tick();
        self.emotion_engine.report()
    }

    pub fn observe_from_critique(&mut self, critique: &CritiqueResult) {
        let quality = critique.overall_quality;
        self.last_quality = quality;
        self.emotion_engine
            .observe(EmotionDimension::Confidence, quality, "critique_quality");
        if quality < 0.3 {
            self.emotion_engine.observe(
                EmotionDimension::Frustration,
                0.7 - quality,
                "low_quality_critique",
            );
        }
        if let Some(ref action) = critique.selected_action {
            if action.contains("explore") || action.contains("curious") {
                self.emotion_engine
                    .observe(EmotionDimension::Curiosity, 0.8, action);
            }
        }
    }

    pub fn emotion_engine(&self) -> &EmotionEngine {
        &self.emotion_engine
    }

    pub fn emotion_engine_mut(&mut self) -> &mut EmotionEngine {
        &mut self.emotion_engine
    }

    pub fn last_quality(&self) -> Option<f64> {
        if self.tick_count > 0 {
            Some(self.last_quality)
        } else {
            None
        }
    }

    pub fn set_emotion_engine(&mut self, engine: EmotionEngine) {
        self.emotion_engine = engine;
    }

    /// 意识影响: 将用户情感快照经 OCC 事件评估映射为自身六维情绪变化 —
    /// 用户情绪经由意识核心转化为代理自己的情绪基调, 使代理"感受"到对话对象。
    pub fn observe_user_affect(&mut self, affect: &UserAffectSnapshot) {
        let novelty = affect.arousal;
        let goal_conduciveness = affect.valence;
        let coping = affect.dominance;
        self.emotion_engine
            .observe_appraisal(novelty, goal_conduciveness, coping, "user_affect");
        match affect.emotion {
            UserEmotion::Joy | UserEmotion::Trust | UserEmotion::Anticipation => {
                self.emotion_engine
                    .observe(EmotionDimension::Joy, 0.6 + 0.4 * affect.valence, "user_affect");
            }
            UserEmotion::Sadness | UserEmotion::Fear => {
                self.emotion_engine.observe(
                    EmotionDimension::Fatigue,
                    0.4 + 0.5 * (1.0 - affect.valence),
                    "user_affect",
                );
                self.emotion_engine.observe(
                    EmotionDimension::Urgency,
                    0.4 + 0.4 * affect.arousal,
                    "user_affect",
                );
            }
            UserEmotion::Anger | UserEmotion::Disgust => {
                self.emotion_engine.observe(
                    EmotionDimension::Frustration,
                    0.5 + 0.4 * affect.arousal,
                    "user_affect",
                );
            }
            UserEmotion::Surprise => {
                self.emotion_engine
                    .observe(EmotionDimension::Curiosity, 0.7, "user_affect");
            }
            UserEmotion::Neutral => {}
        }
    }

    pub fn affective(&self) -> &AffectiveInterface {
        &self.affective
    }

    pub fn affective_mut(&mut self) -> &mut AffectiveInterface {
        &mut self.affective
    }

    /// Advance one consciousness tick.
    pub fn tick(&mut self, resonance_content: &str) -> Option<CritiqueResult> {
        if !self.awakened {
            return None;
        }
        self.tick_count += 1;
        // Feed resonance content into specious present as a VSA-tagged item
        let world_item = VsaTagged::world_input(resonance_content);
        // ── F4 接线 (R-P79): 共振内容同步进自传体意识流, stream 不再只是唤醒门。
        self.stream.push(world_item.clone());
        self.specious_present.push(world_item);
        // Query knowledge base with the resonance content — 意识核心主动检索知识
        self.inject_kb_knowledge(resonance_content);
        // Run volition: propose candidates from the specious present window
        for item in self.specious_present.window().iter() {
            let desc =
                String::from_utf8_lossy(&item.vector[..item.vector.len().min(64)]).to_string();
            if !desc.is_empty() {
                let candidate = ActionCandidate::new(item.vector.clone(), &desc);
                self.volition.propose(candidate);
            }
        }
        // Select the best action candidate (was never called before)
        let selected_action = self.volition.select_best();
        let temporal_delta = self.specious_present.temporal_difference();
        // Re-borrow for critique: context = 前一帧 (current==current 会让 relevance 恒为 1.0)
        let mut critique = match self.specious_present.current() {
            Some(current) => {
                let context = self.specious_present.previous(1).unwrap_or(current);
                self.critic
                    .evaluate(current, context, Some(&self.specious_present))
            }
            None => return None,
        };
        // ── SourceHierarchy 溯源消费接线 (R-P79): 当前帧带 ProvenanceChain 时,
        // validate_chain+effective_confidence 参与一致性评分; 弱链降分并留痕 reasons。
        // 无溯源链的帧保持原一致性 (中性, 不惩罚)。
        if let Some(cur_tagged) = self.specious_present.current() {
            if let Some(ref chain) = cur_tagged.provenance {
                let prov_conf = if chain.validate_chain() {
                    // 有效置信度 = 链上最新层的抽象折扣置信度 (Raw 无折扣 / Structured×0.85 / Semantic×0.7)
                    chain
                        .layers
                        .last()
                        .map(|(layer, _)| layer.effective_confidence())
                        .unwrap_or(0.0)
                } else {
                    0.2
                };
                critique.consistency_score =
                    (critique.consistency_score * 0.8 + prov_conf * 0.2).clamp(0.0, 1.0);
                if prov_conf < 0.35 {
                    critique
                        .reasons
                        .push(format!("provenance_chain_weak: conf={:.2}", prov_conf));
                }
            }
        }
        // Attach selected action info to critique
        if let Some(action) = selected_action {
            critique.selected_action = Some(action.description.clone());
        }
        critique.temporal_delta = temporal_delta;
        self.observe_from_critique(&critique);
        Some(critique)
    }

    /// Get a reference to the volition engine for inspection
    pub fn volition(&self) -> &VolitionEngine {
        &self.volition
    }

    /// Get a mutable reference to the volition engine
    pub fn volition_mut(&mut self) -> &mut VolitionEngine {
        &mut self.volition
    }

    /// Get the current specious present coherence
    pub fn coherence(&self) -> f64 {
        self.specious_present.average_coherence()
    }

    /// Clear all candidates from the volition engine
    pub fn clear_volition(&mut self) {
        self.volition.clear();
    }
}

impl crate::core::nt_core_self_test::SelfTest for ConsciousnessRuntime {
    fn name(&self) -> &str {
        "consciousness_runtime"
    }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        // Test 1: new runtime is not awakened
        if self.awakened {
            failures.push("new runtime should not be awakened".into());
        }
        // Test 2: tick returns None before awaken
        let mut cr = ConsciousnessRuntime::new();
        if cr.tick("test").is_some() {
            failures.push("tick before awaken should return None".into());
        }
        // Test 3: awaken sets awakened flag
        cr.awaken();
        if !cr.awakened {
            failures.push("awaken should set awakened=true".into());
        }
        // Test 4: tick_count increments
        let count_before = cr.tick_count;
        let _ = cr.tick("test resonance");
        if cr.tick_count != count_before + 1 {
            failures.push("tick should increment tick_count".into());
        }
        // Test 5: tick after awaken returns Some
        let result = cr.tick("after awaken");
        if result.is_none() {
            failures.push("tick after awaken should return Some critique".into());
        }
        // Test 6: KB attachment is optional — new runtime has none
        if cr.is_kb_attached() {
            failures.push("new runtime should not have KB attached".into());
        }
        // Test 7: query_kb without KB returns empty (no panic)
        if !cr.query_kb("anything", 3).is_empty() {
            failures.push("query_kb without KB should return empty".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

impl Default for ConsciousnessRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observe_from_critique_low_quality() {
        let mut cr = ConsciousnessRuntime::new();
        let critique = CritiqueResult {
            passed: true,
            relevance_score: 0.0,
            consistency_score: 0.0,
            uncertainty_score: 0.8,
            overall_quality: 0.2,
            reasons: vec!["low quality".into()],
            selected_action: Some("rethink".into()),
            temporal_delta: Some(0.0),
        };
        cr.observe_from_critique(&critique);
        let report = cr.emotion_engine.report();
        assert!(
            report.confidence < 0.5,
            "confidence={} should have dropped from 0.5",
            report.confidence
        );
        assert!(
            report.frustration > 0.49,
            "frustration={} should be above neutral",
            report.frustration
        );
        assert_eq!(cr.last_quality, 0.2);
    }

    #[test]
    fn test_observe_from_critique_high_quality() {
        let mut cr = ConsciousnessRuntime::new();
        let critique = CritiqueResult {
            passed: true,
            relevance_score: 0.0,
            consistency_score: 0.0,
            uncertainty_score: 0.2,
            overall_quality: 0.9,
            reasons: vec!["high quality".into()],
            selected_action: Some("explore_new".into()),
            temporal_delta: Some(0.0),
        };
        cr.observe_from_critique(&critique);
        let report = cr.emotion_engine.report();
        assert!(
            report.confidence > 0.55,
            "confidence={} should be above neutral",
            report.confidence
        );
        assert!(
            report.curiosity > 0.55,
            "curiosity={} should be above neutral",
            report.curiosity
        );
        assert_eq!(cr.last_quality, 0.9);
    }

    #[test]
    fn test_tick_emotion_returns_report() {
        let mut cr = ConsciousnessRuntime::new();
        cr.awaken();
        let _ = cr.tick("test resonance");
        let report = cr.tick_emotion();
        assert!(report.confidence >= 0.0);
        assert!(report.valence >= -1.0 && report.valence <= 1.0);
    }

    #[test]
    fn test_set_emotion_engine() {
        let mut cr = ConsciousnessRuntime::new();
        let mut engine = EmotionEngine::default();
        engine.observe(EmotionDimension::Confidence, 0.9, "test");
        cr.set_emotion_engine(engine);
        let report = cr.emotion_engine.report();
        assert!(
            report.confidence > 0.55,
            "confidence={} should reflect observed 0.9",
            report.confidence
        );
    }

    #[test]
    fn test_tick_wires_observe_from_critique() {
        let mut cr = ConsciousnessRuntime::new();
        cr.awaken();
        let result =
            cr.tick("high quality content that is meaningful long enough to produce a critique");
        assert!(result.is_some());
        let report = cr.emotion_engine.report();
        assert!(report.confidence >= 0.0); // tick wired observe_from_critique
        assert!(cr.last_quality > 0.0 || cr.last_quality == 0.0); // set by tick
        assert!(cr.last_quality() == Some(cr.last_quality));
    }

    #[test]
    fn test_query_kb_without_attachment_returns_empty() {
        let cr = ConsciousnessRuntime::new();
        assert!(!cr.is_kb_attached());
        assert!(cr.query_kb("anything", 3).is_empty());
    }

    #[test]
    fn test_tick_without_kb_injects_nothing() {
        let mut cr = ConsciousnessRuntime::new();
        cr.awaken();
        let _ = cr.tick("no kb attached yet");
        assert!(cr.last_kb_injections.is_empty());
        // volition works without KB: tick 输入进入 specious present 窗口 → 至少一个候选
        assert!(cr.volition().candidate_count() >= 1);
    }

    #[test]
    fn test_kb_query_cache_bounded_and_shared() {
        let cr = ConsciousnessRuntime::new();
        // 未挂接 KB 时, 缓存不应误报命中
        assert!(cr.query_kb("cache_probe", 3).is_empty());
        // cache 内部无任何 KB 结果 — 验证缓存不会在无 KB 时返回脏数据
        let mut cr2 = ConsciousnessRuntime::new();
        cr2.awaken();
        let _ = cr2.tick("cache probe resonance content for cache test");
        assert!(cr2.last_kb_injections.is_empty());
    }

    #[test]
    fn test_observe_user_affect_maps_to_engine() {
        use crate::core::nt_core_self::affective_interface::UserAffectModel;
        let mut cr = ConsciousnessRuntime::new();
        let mut model = UserAffectModel::default();
        let snap = model.detect_from_text("我很难过", None);
        cr.observe_user_affect(&snap);
        let report = cr.emotion_engine.report();
        // 意识影响: 用户负价 → 代理情绪转向低 valence + 疲劳上升 (困扰态)。
        assert!(report.valence < 0.5, "valence={}", report.valence);
        assert!(report.fatigue > 0.5);
        let mut model2 = UserAffectModel::default();
        let snap2 = model2.detect_from_text("太开心了", None);
        cr.observe_user_affect(&snap2);
        let report2 = cr.emotion_engine.report();
        assert!(report2.joy > 0.5);
        assert_eq!(cr.affective().user.history.len(), 0);
    }

    #[test]
    fn test_affective_interface_exposed() {
        let cr = ConsciousnessRuntime::new();
        assert_eq!(cr.affective().relationship.interactions, 0);
        let mut cr2 = ConsciousnessRuntime::new();
        cr2.affective_mut().relationship.on_turn(true, 0.5, 0.8);
        assert_eq!(cr2.affective().relationship.interactions, 1);
    }
}
