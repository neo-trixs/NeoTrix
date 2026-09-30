use std::sync::{Arc};

use crate::l2_perception::nt_core_e8::domain_transition::{ E8TaskType};
use crate::l5_cognition::nt_core_ttc::{EffortTier};

use crate::l0_substrate::nt_core_span::{
    AttributeValue, NoopTracer, Span, Tracer,
};
use crate::l5_cognition::l1_facade::ConsciousnessGoldStandard;
use crate::l5_cognition::nt_mind::nt_mind::control_distillation::{
    AlternatingSequence, ControlTrainer, CsppoReport, ControlDistillStep, SftReport,
};
use crate::l5_cognition::nt_mind::nt_mind::reasoning_types::{ReasoningRecord, ReasoningType};
use crate::l0_substrate::nt_core_error::{ NeoTrixResult};
use super::nt_builders::{ReasoningEngine, CONTROL_TRAIN_BATCH, MAX_TRACES};

impl ReasoningEngine {
    pub(crate) fn broadcast_and_finalize(
        &mut self,
        task: &str,
        root_span: Span,
        result: NeoTrixResult<String>,
    ) -> NeoTrixResult<String> {
        // GWT 共振职责已收敛至后台环 (background_loop handlers_consciousness.rs):
        // reasoner 热路径不再承担 GWT resonant_broadcast —— 产物 (winner/entropy) 仅
        // 进 telemetry, 无决策/学习消费, 属 cycle 204 HIGH-2 冗余共振。reasoner 侧
        // gwt 实例保留供未来注入, 但共振计算统一由后台环闭环执行。

        if let Some(ref compressor) = self.trajectory_compressor {
            let orig_len = self.state_trajectory.len();
            self.state_trajectory = compressor.compress_state_trajectory(&self.state_trajectory);
            let saved = orig_len.saturating_sub(self.state_trajectory.len());
            if saved > 0 {
                root_span.set_attribute(
                    "trajectory_compressed_states",
                    AttributeValue::Int(saved as i64),
                );
            }
        }

        if let Some(ref kb) = self.kb {
            if self.state_trajectory.len() >= 2 {
                let tips: Vec<serde_json::Value> = self.state_trajectory.windows(2).enumerate().filter_map(|(i, w)| {
                    if w[0].mode == w[1].mode {
                        Some(serde_json::json!({
                            "tip_type": "Optimization",
                            "source_step_idx": i,
                            "pattern": format!("Mode {:?} repeated consecutively", w[0].mode.mode_name()),
                            "recommendation": "Consolidate consecutive same-mode states".to_string(),
                            "confidence": 0.5,
                            "provenance": format!("engine: state {} -> {}", i, i+1),
                        }))
                    } else {
                        None
                    }
                }).collect();
                let report = serde_json::json!({
                    "tips": tips,
                    "trajectory_id": format!("reason-{}", task.len().min(16)),
                    "total_steps": self.state_trajectory.len(),
                    "success": result.is_ok(),
                });
                if let Err(e) = kb.store_learning_report(&report) {
                    root_span.set_attribute("kb_learn_error", AttributeValue::String(e));
                } else {
                    root_span.set_attribute("kb_learn_stored", AttributeValue::Bool(true));
                }
            }
        }

        // EWHR auto-invoke: analyze trajectory and propose hypothesis descriptions
        if let Some(ref bridge) = self.ewhr_bridge {
            let proposed = bridge.analyze_trajectory(&self.state_trajectory, task);
            if !proposed.is_empty() {
                root_span.set_attribute(
                    "ewhr_hypotheses_proposed",
                    AttributeValue::Int(proposed.len() as i64),
                );
                // Hydrate into HypothesisNetwork if available
                if let Some(ref net_lock) = self.hypothesis_network {
                    if let Ok(mut net) = net_lock.lock() {
                        let added = hydrate_ewhr_hypotheses(
                            &mut net,
                            &proposed,
                            self.state_trajectory.len(),
                        );
                        if added > 0 {
                            root_span.set_attribute(
                                "ewhr_hypotheses_hydrated",
                                AttributeValue::Int(added as i64),
                            );
                        }
                    }
                }
            }
        }

        if let Some(ref t) = self.tracer {
            t.end_span(root_span);
        } else {
            NoopTracer.end_span(root_span);
        }

        // F6 closed loop (周天大阵运转): distill the successful trace into
        // alternating sequences and periodically run SFT + CSPO to update the
        // E8 policy, so reasoning controls learned from real runs feed back
        // into subsequent reason() calls via PRM (R-P79 production wiring).
        if let Ok(response) = &result {
            self.learn_from_trace(task, response);
        }

        self._core_review(task, &result);

        // Return watermarked response if anti-distillation was active, else raw response
        if let Some(watermarked) = self._last_watermarked.take() {
            Ok(watermarked)
        } else {
            result
        }
    }

    /// Auto-record conversation metadata on every reason() call.
    /// Stores task, outcome, E8 mode, specialist winner, error count into KB.
    pub(crate) fn _core_review(&mut self, task: &str, result: &NeoTrixResult<String>) {
        let (outcome, error_ctx) = match result {
            Ok(_) => ("success", None),
            Err(e) => ("error", Some(format!("{}", e))),
        };
        self.record_trace(
            ReasoningType::Conversation,
            task,
            "",
            "",
            error_ctx.as_deref(),
            if result.is_ok() { 1.0 } else { 0.0 },
        );
        let error_count = if result.is_err() { 1 } else { 0 };
        let e8_mode = self.current_state.mode.mode_name().to_string();
        let specialist = self
            .gwt
            .as_ref()
            .and_then(|g| g.last_resonance.as_ref())
            .map(|r| r.winner.to_string())
            .unwrap_or_default();
        if let Some(ref kb) = self.kb {
            use crate::l5_cognition::l1_facade::ConversationRecord;
            let record = ConversationRecord {
                id: format!("conv-{}", self.llm_call_count),
                session_id: String::new(),
                task_description: task.to_string(),
                user_intent: task.to_string(),
                strategy_used: format!("e8_mode:{}", e8_mode),
                e8_mode,
                specialist_winner: specialist,
                actions_taken: Vec::new(),
                obstacles_encountered: Vec::new(),
                fix_patterns: Vec::new(),
                outcome: outcome.to_string(),
                effectiveness: if result.is_ok() { 1.0 } else { 0.0 },
                reasoning_iterations: self.state_trajectory.len() as u32,
                error_count,
                timestamp: chrono::Utc::now().timestamp(),
            };
            if let Err(e) = kb.store_conversation_record(&record) {
                log::warn!("[reasoning] failed to store conversation record: {}", e);
            }
        }
    }


    pub fn record_trace(
        &mut self,
        rt: ReasoningType,
        task: &str,
        prompt: &str,
        response: &str,
        error_info: Option<&str>,
        reward: f64,
    ) {
        let trace = ReasoningRecord {
            id: format!("trace-{}", self.llm_call_count),
            reasoning_type: rt,
            reasoning_method: None,
            perspective_lens: None,
            task: task.to_string(),
            prompt: prompt.to_string(),
            llm_response: response.to_string(),
            error_context: error_info.map(String::from),
            outcome_score: reward,
            success: error_info.is_none(),
            timestamp: chrono::Utc::now().timestamp(),
        };
        if self.traces.len() >= MAX_TRACES {
            self.traces.remove(0);
        }
        self.traces.push(trace);
    }


    pub fn learn_from_trace(&mut self, task: &str, response: &str) {
        if let Some(ref mut prm) = self.prm {
            let _task_type = E8TaskType::detect(task);
            let substantive = if response.len() > 100 { 0.8 } else { 0.3 };
            let tier_credit = match self.last_effort_tier {
                Some(EffortTier::Low) => 1.0,
                Some(EffortTier::Medium) => 0.9,
                Some(EffortTier::High) => 0.8,
                Some(EffortTier::XHigh) => 0.7,
                Some(EffortTier::Max) => 0.6,
                None => 0.85,
            };
            let mut step_reward = substantive * tier_credit;
            // ── P0 RLVR 锚: 本地 kernel self-consistency 一致性信号 ──
            // 无需外部 gold 答案的自我验证奖励: 对 task 做多路径聚合推理,
            // 一致性分数越高 → 推理越可信 → 奖励加成。对齐主流推理模型的
            // self-consistency / majority vote 机制 (R-P79 生产接线)。
            let sc = {
                use crate::l5_cognition::l1_facade::{text_to_vector, ReasoningKernel};
                let kernel = ReasoningKernel::new(self.current_state.mode.0 as usize % 19);
                let query = text_to_vector(task, 128);
                kernel.self_consistency(&query, 3)
            };
            let sc_score = sc.consistency * 0.6 + sc.avg_confidence * 0.4;
            step_reward = step_reward * 0.7 + sc_score * 0.3;
            log::trace!(
                "[sc-consistency] method={:?} consistency={:.3} avg_conf={:.3} blended_reward={:.3}",
                sc.majority_method, sc.consistency, sc.avg_confidence, step_reward
            );
            prm.learn_step(|collector| {
                collector.begin(task.to_string());
                collector.record_step(
                    crate::l0_substrate::nt_core_traits::SpecialistType::ReflectionEngine,
                    self.current_state.mode,
                    "learn_from_trace".into(),
                    task.to_string(),
                    response.to_string(),
                    None,
                    true,
                    Some(step_reward),
                );
                collector.finish(Some(step_reward), true);
            });
        }
        // F6 wiring: distill control segments from the trace for CSPO training (R-P36 behavioral grounding)
        if let Some(seq) = self._distill_trace(task, response) {
            log::debug!(
                "[control-distill] distilled {} segments (quality={:.3})",
                seq.segments.len(),
                seq.outcome_quality
            );
        }
        // F6 closed loop: once a batch of alternating sequences has accumulated,
        // consume them via SFT + CSPO and write the updated policy back into PRM.
        // Throttled here (in addition to reason()) so training also runs on the
        // offline learn_from_trace path.
        if self.train_batch >= CONTROL_TRAIN_BATCH {
            if let Some((sft, csppo)) = self._train_from_distilled() {
                log::debug!(
                    "[control-train] SFT(c={},r={}) CSPO(reward={:.3},masked={})",
                    sft.control_updates,
                    sft.reason_updates,
                    csppo.total_control_reward,
                    csppo.masked_steps,
                );
            }
        }
    }

    /// 把单条推理 response 蒸馏为交替序列 (Reason ↔ Control)，供 CSPO/SFT 训练消费。
    /// 按换行/句读切分步骤；无法解析时返回 None (失败静默，不影响主推理路径)。
    pub(crate) fn _distill_trace(
        &mut self,
        task: &str,
        response: &str,
    ) -> Option<AlternatingSequence> {
        let distiller = self.control_distiller.as_ref()?;
        let steps = _split_response_into_steps(response);
        if steps.is_empty() {
            return None;
        }
        let id = format!(
            "distill_{}_{}",
            self.traces.len(),
            chrono::Utc::now().timestamp()
        );
        let seq = distiller
            .extract_alternating_sequence(id, task, response, &steps, response)
            .ok()?;
        if self.distilled_sequences.len() >= MAX_TRACES {
            self.distilled_sequences.remove(0);
        }
        self.distilled_sequences.push(seq.clone());
        self.train_batch += 1;
        Some(seq)
    }

    /// F6 训练闭环：消费蒸馏序列，SFT + CSPO 更新 E8 policy。
    ///
    /// 从当前 `prm.policy` 克隆构造临时训练器（单策略权威，避免平行状态，
    /// R-P42），运行 SFT（阶段 1）+ CSPO（阶段 2）后写回 `prm.policy`。
    /// 节流由 `reason()` 主流程按 `train_batch` 阈值触发。
    pub(crate) fn _train_from_distilled(&mut self) -> Option<(SftReport, CsppoReport)> {
        if self.distilled_sequences.is_empty() || self.prm.is_none() {
            return None;
        }
        let seqs = std::mem::take(&mut self.distilled_sequences);
        let policy = self.prm.as_ref().map(|p| p.policy.clone())?;
        let gold = Arc::new(ConsciousnessGoldStandard::new());
        let mut trainer = ControlTrainer::new(policy, gold);
        let sft = trainer.sft(&seqs).ok()?;
        let csppo = trainer.csppo(&seqs).ok()?;
        if let Some(ref mut prm) = self.prm {
            prm.policy = trainer.policy.clone();
            prm.learning_count += 1;
        }
        self.train_batch = 0;
        log::info!(
            "[control-train] batch={} sft(control={},reason={}) csppo(reward={:.3},masked={})",
            seqs.len(),
            sft.control_updates,
            sft.reason_updates,
            csppo.total_control_reward,
            csppo.masked_steps,
        );
        Some((sft, csppo))
    }

    pub(crate) fn _observer_analyze(&mut self, task: &str) {
        // Use error recovery to wrap the observer analysis with retry + circuit breaker + fallback
        let report = self
            .observer_error_recovery
            .execute(|| Ok(self.observer.analyze(&self.state_trajectory, &[task])))
            .unwrap_or_else(|_| {
                log::warn!("[observer] Error recovery exhausted, using degraded report");
                self.observer.analyze(&self.state_trajectory, &[task])
            });
        if report.has_actionable_insight {
            if let Some(ref mut prm) = self.prm {
                let bonus = report.quality_score * 0.1;
                let current_mode = self.current_state.mode.0 as usize;
                prm.policy.mode_values[current_mode.min(63)] =
                    (prm.policy.mode_values[current_mode.min(63)] + bonus).min(1.0);
            }
            // Feed richer observer data to E8 policy: step attention weights, convergence
            if let Some(ref mut prm) = self.prm {
                if let Some(ref attn) = report.step_attention {
                    for (mode_idx, weight) in attn.iter().enumerate() {
                        if mode_idx < prm.policy.mode_values.len() {
                            prm.policy.mode_values[mode_idx] =
                                (prm.policy.mode_values[mode_idx] + weight * 0.05).min(1.0);
                        }
                    }
                }
            }
        }
        // Log critical patterns
        for cp in &report.critical_patterns {
            log::info!("[observer] Critical pattern detected: {}", cp);
        }
    }

    pub(crate) fn _infer_reasoning_type(task: &str) -> ReasoningType {
        let lower = task.to_lowercase();
        let math_keywords = [
            "solve",
            "calculate",
            "compute",
            "equation",
            "math",
            "algebra",
            "calculus",
            "derivative",
            "integral",
        ];
        let coding_keywords = [
            "implement",
            "function",
            "bug",
            "test",
            "code",
            "compile",
            "refactor",
            "debug",
            "api",
            "class",
            "struct",
        ];
        let reasoning_keywords = [
            "why",
            "explain",
            "analyze",
            "compare",
            "reason",
            "evaluate",
            "hypothesis",
            "infer",
            "deduce",
        ];
        let knowledge_keywords = [
            "what is",
            "define",
            "meaning of",
            "tell me about",
            "information on",
            "search for",
        ];
        for k in math_keywords {
            if lower.contains(k) {
                return ReasoningType::TaskSolving;
            }
        }
        for k in coding_keywords {
            if lower.contains(k) {
                return ReasoningType::General;
            }
        }
        for k in reasoning_keywords {
            if lower.contains(k) {
                return ReasoningType::TaskSolving;
            }
        }
        for k in knowledge_keywords {
            if lower.contains(k) {
                return ReasoningType::KnowledgeQuery;
            }
        }
        ReasoningType::Conversation
    }
}

/// EWHR 提议 → HypothesisNetwork 落点。
///
/// 将 `analyze_trajectory` 返回的候选假说字符串 (Vec<String>) 转化为
/// HypothesisNetwork 节点。幂等: 同一 (trajectory_len, index) 的 id 已存在则跳过,
/// 避免每轮重复落点。返回实际新增数量。
pub(crate) fn hydrate_ewhr_hypotheses(
    net: &mut crate::l4_emotion::nt_memory::nt_memory_historian::nt_evidence_hypothesis::HypothesisNetwork,
    proposed: &[String],
    tick: usize,
) -> usize {
    let mut added = 0usize;
    for (i, proposal) in proposed.iter().enumerate() {
        let id = format!("ewhr_{}_{}", tick, i);
        if net.get_hypothesis(&id).is_some() {
            continue;
        }
        let title = if proposal.chars().count() > 32 {
            let truncated: String = proposal.chars().take(30).collect();
            format!("{}…", truncated)
        } else {
            proposal.clone()
        };
        net.propose_hypothesis(&id, &title, proposal, 0.5);
        added += 1;
    }
    added
}

/// 把推理 response 文本切分为步骤序列，供 ControlDistiller 检测 takeover 点。
/// 按换行分段；若不足 2 段则按句号/分号切分。每步携带近似 token 数。
pub fn _split_response_into_steps(response: &str) -> Vec<ControlDistillStep> {
    let mut segments: Vec<String> = response
        .split('\n')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if segments.len() < 2 {
        segments = response
            .split(['.', ';', '。', '；'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
    }
    segments
        .iter()
        .enumerate()
        .map(|(i, text)| ControlDistillStep {
            step_idx: i,
            text: text.clone(),
            e8_mode: None,
            token_count: text.len() / 4,
        })
        .collect()
}
