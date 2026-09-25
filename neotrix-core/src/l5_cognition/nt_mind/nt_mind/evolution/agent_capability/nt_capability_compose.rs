//! 组合吸收 (对话/派单/研究经验→脑能力反哺) 。

use super::super::co_evolution::{CoEvolutionLoop, ExperienceMemory};
use super::nt_capability_types::{
    DialogueAbsorbConfig, DialogueAbsorbOutcome, DialogueExperience,
};
use crate::l5_cognition::l1_facade::{KnowledgeBase, UnifiedSearch, WorldSearchResult};
use crate::l5_cognition::nt_mind::nt_mind::seal_core::core::PerformanceEvaluator;
use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
use neotrix_types::knowledge_access::NodeType;

/// 对话吸收桥 (DialogueAbsorbBridge) — 让对话经验参与 SelfIteratingBrain 进化 (R-P42)。
///
/// 吸收模式: 从 KB 读取近期 session/experience 节点 → 由正文关键词派生出
/// 内容感知的 CapabilityVector (custom source) → `absorb_from_custom` 落地,
/// 同时以 `KnowledgeSource::DialogueExperience` 身份跑一次受校验的
/// `safe_absorb` (DefaultAbsorbValidator)。两条路径都真实反哺脑能力,
/// 对话经历不再是 experience-tree 一次性写入的死数据。
pub struct DialogueAbsorbBridge {
    pub kb: std::sync::Arc<KnowledgeBase>,
    /// 单次最多吸收的条目数
    pub max_entries: usize,
    /// 重要性下界 — 低于该值的会话不参与能力吸收
    pub min_importance: f64,
    /// 内容感知向量推导参数 (D5 校准入口)
    pub config: DialogueAbsorbConfig,
}

impl DialogueAbsorbBridge {
    pub fn new(kb: std::sync::Arc<KnowledgeBase>) -> Self {
        let config = DialogueAbsorbConfig::default();
        Self {
            kb,
            max_entries: config.max_entries,
            min_importance: config.min_importance,
            config,
        }
    }

    /// 以自定义参数构造 — 供调参与测试校准 (D5)。
    pub fn with_config(kb: std::sync::Arc<KnowledgeBase>, config: DialogueAbsorbConfig) -> Self {
        Self {
            kb,
            max_entries: config.max_entries,
            min_importance: config.min_importance,
            config,
        }
    }

    /// 从 KB 提取近期对话经验 — NodeType::Session 或标题以 "session-" 开头。
    pub fn recent_experiences(&self) -> Vec<DialogueExperience> {
        let Ok(nodes) = self.kb.all_nodes() else {
            return Vec::new();
        };
        let mut experiences: Vec<DialogueExperience> = nodes
            .into_iter()
            .filter(|n| n.node_type == NodeType::Session || n.title.starts_with("session-"))
            .filter(|n| n.importance >= self.min_importance)
            .map(|n| {
                let content = n
                    .content
                    .clone()
                    .or_else(|| n.summary.clone())
                    .unwrap_or_else(|| n.title.clone());
                DialogueExperience {
                    title: n.title,
                    content,
                    importance: n.importance,
                }
            })
            .collect();
        experiences.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        experiences.truncate(self.max_entries);
        experiences
    }

    /// 由对话正文派生出内容感知的能力向量 — 关键词命中 23 维字段则提升对应维度。
    ///
    /// 吸收模式: 这是 content-aware 的源头向量, 经 `register_knowledge_source`
    /// 登记后由 `absorb_from_custom` 以对话特有强度反哺能力面。
    pub fn derive_vector(&self, content: &str) -> crate::l5_cognition::nt_core::capability::types::CapabilityVector {
        let text = content.to_lowercase();
        let mut cv = crate::l5_cognition::nt_core::capability::types::CapabilityVector::default();
        let keyword_dims: &[(&[&str], &str)] = &[
            (
                &["test", "verify", "assert", "check", "unit"],
                "verification",
            ),
            (
                &["memory", "kb", "storage", "recall", "retriev"],
                "semantic_layer",
            ),
            (
                &["plan", "goal", "strategy", "schedule"],
                "compound_composition",
            ),
            (&["analy", "trace", "debug", "root cause"], "analysis"),
            (&["synthes", "summar", "distill", "abstract"], "synthesis"),
            (
                &["seal", "iterate", "self-improve", "evolve", "absorb"],
                "experimental",
            ),
            (
                &["attention", "focus", "route", "domain"],
                "ai_native_states",
            ),
            (&["creative", "novel", "design", "style"], "creativity"),
            (&["document", "comment", "explain", "doc"], "accessibility"),
            (
                &["conversation", "dialogue", "user", "prompt"],
                "inference_depth",
            ),
        ];
        for (kws, dim) in keyword_dims {
            let hits = kws.iter().filter(|k| text.contains(**k)).count();
            if hits > 0 {
                let boost = (self.config.boost_base + self.config.boost_per_hit * hits as f64)
                    .min(self.config.boost_cap);
                let _ = cv.set_field_by_name(dim, boost);
            }
        }
        cv
    }

    /// 把近期对话经验吸收进 SelfIteratingBrain — 返回**实测的行为化结果**。
    ///
    /// 吸收模式 (grounded 于 ACL 2026/arXiv 2026 经验学习文献):
    ///   1. **实例级 (instance)**: 每条经验派生内容感知向量, `absorb_from_custom` 落地,
    ///      记录标题避免同一会话反复吸收 (ReMe utility-based refinement 去重)。
    ///   2. **批次级 (batch)**: 全部经验向量取分量 max 共振 (Metacognitive Consolidation
    ///      实例→批次层次), 以 DialogueExperience 源身份吸收 — 原则级信号比实例级持久。
    ///   3. **Verify 校验 (EDV 反 Self-Confirmation Trap)**: `absorb_with_critic` 做
    ///      吸收前后 PerformanceEvaluator 对比, 能力下降则回滚 — 不用恒真的弱 validator。
    ///
    /// 反 Self-Confirmation (D1/D2): 返回值不是"吸收了几条"的虚荣计数, 而是
    /// **实测能力差** — 吸收前 vs 吸收后按 `PerformanceEvaluator` 打分的 delta。
    /// 只有当批评器接受 (未回滚) 且能力面确实变好时才计为有效。
    pub fn absorb_pending(&self, brain: &mut SelfIteratingBrain) -> DialogueAbsorbOutcome {
        let experiences = self.recent_experiences();
        if experiences.is_empty() {
            return DialogueAbsorbOutcome::empty();
        }
        // ── 批次级共振向量 (分量取 max, 反映跨会话主题强度) ──
        let mut batch = crate::l5_cognition::nt_core::capability::types::CapabilityVector::default();
        let mut seen = std::collections::HashSet::new();
        for exp in &experiences {
            if !seen.insert(exp.title.clone()) {
                continue; // ReMe: 已吸收过的会话跳过, 避免重复污染
            }
            let v = self.derive_vector(&exp.content);
            for (i, val) in v.arr().iter().enumerate() {
                if *val > batch.arr()[i] {
                    batch.arr_mut()[i] = *val;
                }
            }
        }
        let has_batch_signal = batch.arr().iter().any(|&v| v > 0.0);
        if !has_batch_signal {
            return DialogueAbsorbOutcome::empty();
        }

        // 实测能力差: 吸收前按 PerformanceEvaluator 打分 (D1/D2 行为化指标)。
        let before_score = PerformanceEvaluator::evaluate(
            &crate::l2_perception::nt_world::nt_world_model::TaskType::General,
            &brain.brain.capability,
        );

        // ── 实例级: 每条经验内容感知吸收 ──
        let mut absorbed = 0usize;
        for (i, exp) in experiences.iter().enumerate() {
            let custom_name = format!(
                "dialogue:{}:{}",
                i,
                exp.title.chars().take(32).collect::<String>()
            );
            let vector = self.derive_vector(&exp.content);
            let has_signal = vector.arr().iter().any(|&v| v > 0.0);
            brain
                .brain
                .register_knowledge_source(&custom_name, vector.clone());
            if has_signal && brain.brain.absorb_from_custom(&custom_name) {
                absorbed += 1;
            }
        }

        // ── 批次级: 原则级共振向量经 DialogueExperience 源吸收 ──
        brain
            .brain
            .register_knowledge_source("dialogue:batch", batch.clone());
        let _ = brain.brain.absorb_from_custom("dialogue:batch");

        // ── Verify (EDV): 吸收前后性能对比, 能力下降则回滚 ──
        // 批评器返回是否接受 (未回滚); 不再丢弃 — 它是行为化成败的真信号。
        let critic_accepted =
            brain.absorb_with_critic(crate::l2_perception::nt_core_knowledge::types::KnowledgeSource::DialogueExperience);

        // 实测后分: 批评器若回滚, after == before, 无增益。
        let after_score = PerformanceEvaluator::evaluate(
            &crate::l2_perception::nt_world::nt_world_model::TaskType::General,
            &brain.brain.capability,
        );

        DialogueAbsorbOutcome {
            absorbed,
            critic_accepted,
            score_before: before_score,
            score_after: after_score,
            score_delta: after_score - before_score,
        }
    }

    /// 派单经验 → 大脑能力吸收闭环 (P6, R-P79 生产接线) — coevo 经验子图回读。
    ///
    /// MAGE 的 experience 子图不能只写不读: 派单执行沉淀的成功/失败记忆
    /// (双记忆索引) 经内容感知向量反哺 SelfIteratingBrain, 让"派单控制面学到的
    /// 经验"真正改变脑能力, 而非停在 kv_store 当统计死数据。
    ///
    /// 吸收模式 (复用 `absorb_pending` 同机制, R-P42 强化既有节点):
    ///   1. 只消费水位之上未吸收的新记忆 (ReMe 去重, 防反复吸收)。
    ///   2. 实例级: 每条记忆派生 `derive_dispatch_vector` → absorb_from_custom。
    ///   3. 批次级: 全部新记忆分量取 max 共振 → dispatch:batch 源吸收 (原则级持久)。
    ///   4. Verify: `absorb_with_critic` 做吸收前后 PerformanceEvaluator 对比,
    ///      能力下降则回滚 (EDV 反 Self-Confirmation) — 盲提升被批评器拦下。
    ///   5. 无论批评器接受与否, 水位推进 (已尝试) — 不重试同一条, 防抖动。
    pub fn absorb_dispatch_experiences(
        &self,
        brain: &mut SelfIteratingBrain,
        coevo: &mut CoEvolutionLoop,
    ) -> DialogueAbsorbOutcome {
        let memories: Vec<&ExperienceMemory> =
            coevo.new_memories_since_watermark();
        if memories.is_empty() {
            return DialogueAbsorbOutcome::empty();
        }
        // ── 批次级共振向量 (分量取 max, 反映派单控制面主题强度) ──
        let mut batch = crate::l5_cognition::nt_core::capability::types::CapabilityVector::default();
        for m in &memories {
            let v = self.derive_dispatch_vector(&m.summary);
            for (i, val) in v.arr().iter().enumerate() {
                if *val > batch.arr()[i] {
                    batch.arr_mut()[i] = *val;
                }
            }
        }
        let has_batch_signal = batch.arr().iter().any(|&v| v > 0.0);
        if !has_batch_signal {
            // 无能力信号 → 标记已尝试, 无增益返回 (避免每次扫描同批死数据)。
            coevo.commit_absorb();
            return DialogueAbsorbOutcome::empty();
        }

        // 实测能力差: 吸收前打分 (D1/D2 行为化指标)。
        let before_score = PerformanceEvaluator::evaluate(
            &crate::l2_perception::nt_world::nt_world_model::TaskType::General,
            &brain.brain.capability,
        );

        // ── 实例级: 每条派单经验内容感知吸收 ──
        let mut absorbed = 0usize;
        for m in &memories {
            let custom_name = format!("dispatch:{}:{}", m.id, m.agent);
            let vector = self.derive_dispatch_vector(&m.summary);
            let has_signal = vector.arr().iter().any(|&v| v > 0.0);
            brain
                .brain
                .register_knowledge_source(&custom_name, vector.clone());
            if has_signal && brain.brain.absorb_from_custom(&custom_name) {
                absorbed += 1;
            }
        }

        // ── 批次级: 原则级共振向量经 batch 源吸收 ──
        brain
            .brain
            .register_knowledge_source("dispatch:batch", batch.clone());
        let _ = brain.brain.absorb_from_custom("dispatch:batch");

        // ── Verify (EDV): 吸收前后性能对比, 能力下降则回滚 ──
        let critic_accepted =
            brain.absorb_with_critic(crate::l2_perception::nt_core_knowledge::types::KnowledgeSource::DialogueExperience);

        let after_score = PerformanceEvaluator::evaluate(
            &crate::l2_perception::nt_world::nt_world_model::TaskType::General,
            &brain.brain.capability,
        );

        // 水位推进: 本批已尝试吸收 (即使批评器回滚也不重试, 防抖动)。
        coevo.commit_absorb();

        DialogueAbsorbOutcome {
            absorbed,
            critic_accepted,
            score_before: before_score,
            score_after: after_score,
            score_delta: after_score - before_score,
        }
    }

    /// 由派单经验摘要派生出内容感知向量 — 派单/控制面域关键词映射。
    ///
    /// 与 `derive_vector` (对话域)、`derive_research_vector` (研究域) 平行的
    /// 派单域映射: 关键词侧重路由/拓扑/策略/检索成败/进化信号, 反映
    /// "派单控制面学到的经验"被吸收进脑能力的信号面。复用同一
    /// `DialogueAbsorbConfig` boost 系数 (D5 单一调参入口)。
    pub fn derive_dispatch_vector(&self, content: &str) -> crate::l5_cognition::nt_core::capability::types::CapabilityVector {
        let text = content.to_lowercase();
        let mut cv = crate::l5_cognition::nt_core::capability::types::CapabilityVector::default();
        let dispatch_dims: &[(&[&str], &str)] = &[
            (
                &["route", "agent", "dispatch", "catalog"],
                "ai_native_states",
            ),
            (
                &["topology", "edge", "repair", "revision"],
                "compound_composition",
            ),
            (
                &["strategy", "bandit", "explor", "exploit", "epsilon"],
                "analysis",
            ),
            (
                &["retriev", "recall", "hit", "search", "kb", "searched"],
                "semantic_layer",
            ),
            (&["success", "found", "pass", "returned"], "verification"),
            (
                &["fail", "error", "empty", "miss", "unavailable"],
                "quality_gates",
            ),
            (
                &["evolv", "co-evol", "memory", "graph", "absorb"],
                "experimental",
            ),
            (&["research", "study", "paper"], "inference_depth"),
            (
                &["plan", "goal", "task", "strategy"],
                "compound_composition",
            ),
            (
                &["test", "assert", "verify", "audit", "evidence"],
                "verification",
            ),
        ];
        for (kws, dim) in dispatch_dims {
            let hits = kws.iter().filter(|k| text.contains(**k)).count();
            if hits > 0 {
                let boost = (self.config.boost_base + self.config.boost_per_hit * hits as f64)
                    .min(self.config.boost_cap);
                let _ = cv.set_field_by_name(dim, boost);
            }
        }
        cv
    }

    /// 由研究结论正文派生出内容感知向量 — 研究域关键词 23 维提升。
    ///
    /// 与 `derive_vector` (对话域) 平行的研究域映射: 关键词侧重证据/来源/
    /// 方法/聚合/分析, 反映"外部世界知识被吸收"的信号面。复用同一
    /// `DialogueAbsorbConfig` boost 系数 (D5 单一调参入口)。
    pub fn derive_research_vector(&self, content: &str) -> crate::l5_cognition::nt_core::capability::types::CapabilityVector {
        let text = content.to_lowercase();
        let mut cv = crate::l5_cognition::nt_core::capability::types::CapabilityVector::default();
        let research_dims: &[(&[&str], &str)] = &[
            (
                &["search", "web", "online", "url", "http"],
                "semantic_layer",
            ),
            (
                &["paper", "arxiv", "research", "study", "report"],
                "inference_depth",
            ),
            (
                &["method", "approach", "technique", "algorithm"],
                "analysis",
            ),
            (
                &["synthes", "aggregate", "summary", "distill", "conclusion"],
                "synthesis",
            ),
            (
                &["evidence", "source", "cite", "reference", "verify"],
                "verification",
            ),
            (
                &["benchmark", "metric", "evaluate", "compare", "result"],
                "quality_gates",
            ),
            (
                &["finding", "insight", "discover", "trend", "pattern"],
                "ai_native_states",
            ),
            (
                &["domain", "field", "industry", "topic", "expert"],
                "domain_specificity",
            ),
            (
                &["collect", "gather", "mine", "scrape", "harvest"],
                "compound_composition",
            ),
            (
                &["open", "share", "collaborate", "community", "doc"],
                "accessibility",
            ),
        ];
        for (kws, dim) in research_dims {
            let hits = kws.iter().filter(|k| text.contains(**k)).count();
            if hits > 0 {
                let boost = (self.config.boost_base + self.config.boost_per_hit * hits as f64)
                    .min(self.config.boost_cap);
                let _ = cv.set_field_by_name(dim, boost);
            }
        }
        cv
    }

    /// 研究结论 → KB → 脑能力进化闭环 (R-P79 生产接线)。
    ///
    /// researcher agent / WebSearchTool 产出的搜索结论不再是一次性丢弃的
    /// 死数据: 落 KB (可溯源) + 蒸馏为内容感知能力向量反哺 SelfIteratingBrain。
    /// 同一 EDV 校验 (critic 接受 + 实测能力差) 复用 `DialogueAbsorbOutcome`。
    ///
    /// `absorbed` 语义: 写入 KB 的结论条数 (>= 1 表示有真实世界知识落地);
    /// 能力面增益以 `score_delta` 为准 (批评器回滚时为 0)。
    pub fn absorb_research_findings(
        &self,
        brain: &mut SelfIteratingBrain,
        query: &str,
        results: &[WorldSearchResult],
    ) -> DialogueAbsorbOutcome {
        if results.is_empty() {
            return DialogueAbsorbOutcome::empty();
        }

        // ── 落 KB (可溯源): 每条结果写 Source 节点, 查询主题写 Insight 节点 ──
        let mut absorbed = 0usize;
        let mut distilled = String::with_capacity(1024);
        for (i, r) in results.iter().enumerate() {
            if let Ok(_id) = self.kb.write_memory_entry(
                &r.title,
                NodeType::Source,
                Some(&r.snippet),
                Some(&r.url),
                Some("nt_world_search"),
                None,
            ) {
                absorbed += 1;
            }
            if i < 8 {
                distilled.push_str(&format!("{} {} ", r.title, r.snippet));
            }
        }
        if let Ok(_id) = self.kb.write_memory_entry(
            &format!("research:{}", query),
            NodeType::Insight,
            Some(distilled.trim()),
            None,
            Some("nt_world_search"),
            None,
        ) {
            absorbed += 1;
        }

        // ── 蒸馏向量 (研究域内容感知) → custom source 吸收 ──
        let vector = self.derive_research_vector(distilled.trim());
        let custom_name = format!(
            "research:{}:{}",
            query.chars().take(24).collect::<String>(),
            absorbed
        );
        brain.brain.register_knowledge_source(&custom_name, vector);
        let has_signal = brain.brain.absorb_from_custom(&custom_name);

        // ── Verify (EDV): 以 ResearchFindings 身份受校验吸收, 能力下降则回滚 ──
        let before_score = PerformanceEvaluator::evaluate(
            &crate::l2_perception::nt_world::nt_world_model::TaskType::General,
            &brain.brain.capability,
        );
        let critic_accepted =
            brain.absorb_with_critic(crate::l2_perception::nt_core_knowledge::types::KnowledgeSource::ResearchFindings);
        let after_score = PerformanceEvaluator::evaluate(
            &crate::l2_perception::nt_world::nt_world_model::TaskType::General,
            &brain.brain.capability,
        );

        DialogueAbsorbOutcome {
            absorbed,
            critic_accepted,
            score_before: before_score,
            score_after: after_score,
            score_delta: if has_signal {
                after_score - before_score
            } else {
                0.0
            },
        }
    }

    /// 生产路径: 以统一搜索 (DDG→Wikipedia 有序后端) 执行查询并吸收结论。
    ///
    /// 网络失败 / 无结果时返回空 outcome (graceful), 不污染脑状态。
    /// researcher agent 与 background_loop 都经此入口, 保证搜索结论
    /// 唯一路径落 KB + 参与进化 (R-P42 强化现有节点, 无平行适配器)。
    pub fn absorb_research_query(
        &self,
        brain: &mut SelfIteratingBrain,
        query: &str,
        count: usize,
    ) -> DialogueAbsorbOutcome {
        let search = UnifiedSearch::new();
        match search.search(query, count) {
            Ok(results) if !results.is_empty() => {
                self.absorb_research_findings(brain, query, &results)
            }
            _ => DialogueAbsorbOutcome::empty(),
        }
    }
}
