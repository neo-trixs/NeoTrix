
use crate::l2_perception::nt_core_e8::domain_transition::{ E8TaskType};
use crate::l2_perception::nt_core_e8::state_machine::E8StateMachine;
use crate::l2_perception::nt_core_e8::thinking_budget::DifficultyEstimator;

use crate::l0_substrate::nt_core_span::{
    AttributeValue, NoopTracer, Span, SpanKind, Tracer,
};
use crate::l5_cognition::nt_mind::nt_mind::reasoning_types::{ ReasoningType};
use crate::l1_action::nt_io::nt_io_provider::{ LlmRequest};
use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};
use super::nt_builders::ReasoningEngine;

impl ReasoningEngine {
    pub(crate) fn prepare_reasoning(&mut self, task: &str, root_span: &Span) -> (E8StateMachine, String) {
        let mut e8_machine = E8StateMachine::from(self.current_state);
        if let Some(ref ttc) = self.ttc_engine {
            e8_machine.set_ttc_engine(ttc.clone());
            let difficulty = DifficultyEstimator::heuristic_difficulty(task, "reasoning");
            root_span.set_attribute("ttc_difficulty", AttributeValue::Float(difficulty));
            if difficulty > 0.3 {
                let allocation = ttc.allocate_budget(difficulty, 1.0);
                root_span.set_attribute(
                    "ttc_strategy",
                    AttributeValue::String(format!("{:?}", allocation.strategy)),
                );
                root_span.set_attribute(
                    "ttc_max_steps",
                    AttributeValue::Int(allocation.budget.max_steps as i64),
                );
                e8_machine.budget = Some(allocation);
            }
        }

        let hex = self.current_state.mode;
        let mode_name = hex.mode_name();
        let mode_desc = hex.mode_description();
        let context = self.build_context(task, ReasoningType::Conversation);
        let artifact_ctx = self.build_artifact_context(task);

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let date_line = if let Some(ref ads) = self.anti_distillation {
            let watermarked = ads.encode_date_line(&format!("Today's date is {}.", today));
            format!("\n{}\n", watermarked)
        } else {
            format!("\nToday's date is {}.\n", today)
        };

        // Pre-call task decomposition: split sensitive tasks into
        // safer subtasks when anticlistillation detects risk.
        let query = if let Some(ref ads) = self.anti_distillation {
            if let Some(suggestions) = ads.decompose_task(task) {
                let mut decomposed = String::new();
                decomposed.push_str("This task has been decomposed into steps:\n");
                for s in &suggestions {
                    decomposed.push_str(&format!("- {}\n", s.subtask));
                }
                decomposed.push_str("\nProceed through each step sequentially.");
                root_span.set_attribute("antidistil_decomposed", AttributeValue::Bool(true));
                root_span.set_attribute(
                    "antidistil_decomp_steps",
                    AttributeValue::Int(suggestions.len() as i64),
                );
                decomposed
            } else {
                task.to_string()
            }
        } else {
            task.to_string()
        };

        // Phase 1.3: ContextBuilder 集成 — 从 KB/经验构建 Kernel context
        let kb_context = self.build_kb_context(task, root_span);

        // JEPA world-model prior: encode task text, predict next latent, and
        // surface predicted trajectory as a soft prior (absent if unwired).
        let jepa_prior = if let Some(ref jepa) = self.jepa {
            let input: Vec<f64> = task.as_bytes().iter().map(|&b| b as f64 / 255.0).collect();
            match jepa
                .encode(&input)
                .and_then(|feats| jepa.predict_with_confidence(&feats))
            {
                Ok((pred, confidence, uncertainty)) => {
                    root_span
                        .set_attribute("jepa_prior_confidence", AttributeValue::Float(confidence));
                    root_span.set_attribute(
                        "jepa_prior_uncertainty",
                        AttributeValue::Float(uncertainty),
                    );
                    // Quantize the predicted latent into a directional signal. High-magnitude
                    // coordinates carry the strongest next-state trend; low-magnitude ones are noise.
                    let direction: Vec<f64> =
                        pred.iter().cloned().filter(|&x| x.abs() > 0.1).collect();
                    if direction.is_empty() {
                        String::new()
                    } else {
                        let top = direction[0];
                        let signal = if top > 0.0 { "positive" } else { "negative" };
                        format!(
                            "\nWorld-model prior (confidence {:.3}, uncertainty {:.3}): latent trend {signal}, {n} active features — use as soft direction, not ground truth.\n",
                            confidence, uncertainty, n = direction.len()
                        )
                    }
                }
                Err(_) => String::new(),
            }
        } else {
            String::new()
        };

        let prompt = format!(
            "You are NeoTrix — mode: {mode_name}\n\
             Strategy: {mode_desc}\n\n\
             Past experiences:\n{context}\n\n\
             {kb_context}{artifact_ctx}{jepa_prior}{date_line}\
             Query: {query}"
        );
        (e8_machine, prompt)
    }


    pub(crate) fn call_llm_and_analyze(
        &mut self,
        task: &str,
        prompt: &str,
        root_span: &Span,
        e8_machine: &mut E8StateMachine,
    ) -> NeoTrixResult<String> {
        let llm_span = self
            .tracer
            .as_ref()
            .map(|t| t.start_child_span(root_span, "call_llm", SpanKind::Llm))
            .unwrap_or_else(|| NoopTracer.start_child_span(root_span, "call_llm", SpanKind::Llm));
        llm_span.set_gen_ai_request_model(&self.default_model);

        let result = self.call_llm(prompt);

        match &result {
            Ok(response) => {
                llm_span.set_attribute(
                    "response_length",
                    AttributeValue::Int(response.len() as i64),
                );
                if let Some(ref t) = self.tracer {
                    t.end_span(llm_span);
                } else {
                    NoopTracer.end_span(llm_span);
                }
                if let Some(ref _ttc) = self.ttc_engine {
                    let early_exit = e8_machine.check_early_exit(0.9);
                    root_span.set_attribute("ttc_early_exit", AttributeValue::Bool(early_exit));
                }
                let refused = _detect_refusal_response(response);
                if let Some(ref mut ads) = self.anti_distillation {
                    ads.record_llm_call(refused);
                    let bits = ads.watermark.to_bits();
                    let watermarked = ads.watermark_response(response);
                    ads.register_trace(
                        response,
                        bits,
                        &self.default_model,
                        &prompt[..prompt.len().min(64)],
                    );
                    if self.gateway.is_some() {
                        ads.detector.record_request(
                            &self.default_model,
                            task,
                            0.7,
                            4096,
                            response.len(),
                        );
                    }
                    root_span.set_attribute("antidistil_bits", AttributeValue::Int(bits as i64));
                    root_span.set_attribute("antidistil_watermarked", AttributeValue::Bool(true));
                    root_span.set_attribute("antidistil_refused", AttributeValue::Bool(refused));
                    // Store watermarked response for return, but don't early-return —
                    // continue to trajectory compression + KB learning.
                    self._last_watermarked = Some(watermarked);
                }
                refused
            }
            Err(e) => {
                llm_span.set_attribute("error", AttributeValue::String(format!("{}", e)));
                if let Some(ref t) = self.tracer {
                    t.end_span(llm_span);
                } else {
                    NoopTracer.end_span(llm_span);
                }
                false
            }
        };

        self.current_state = e8_machine.current_state;
        // Only record the start-of-call state when it actually differs from the
        // last trajectory entry. Combined with the oracle-driven transition in
        // the prediction block below, the trajectory becomes a clean sequence of
        // real state transitions instead of a run of identical self-loop states.
        if self.state_trajectory.last().map(|s| s.mode.0) != Some(self.current_state.mode.0) {
            self.state_trajectory.push(self.current_state);
        }

        if let Some(ref mut sae) = self.sae_bridge {
            let hex = self.current_state.mode;
            let features = sae.extract_features(hex.0, self.current_state.meta.0, &[]);
            if !features.is_empty() {
                root_span.set_attribute(
                    "sae_active_features",
                    AttributeValue::Int(features.len() as i64),
                );
                root_span.set_attribute(
                    "sae_top_feature",
                    AttributeValue::String(
                        features
                            .iter()
                            .max_by(|a, b| a.activation.total_cmp(&b.activation))
                            .map(|f| format!("f{}({:.3})", f.index, f.activation))
                            .unwrap_or_default(),
                    ),
                );
            }
        }

        // Observer analysis: record transitions, detect patterns, compute PRM scores
        let observer_report = self.observer.analyze(&self.state_trajectory, &[task]);
        root_span.set_attribute(
            "observer_traj_len",
            AttributeValue::Int(observer_report.trajectory_len as i64),
        );
        root_span.set_attribute(
            "observer_quality",
            AttributeValue::Float(observer_report.quality_score),
        );
        root_span.set_attribute(
            "observer_distinct_states",
            AttributeValue::Int(observer_report.distinct_states as i64),
        );
        for p in &observer_report.patterns {
            root_span.set_attribute("observer_pattern", AttributeValue::String(p.clone()));
        }
        if observer_report.has_actionable_insight {
            root_span.set_attribute("observer_insight", AttributeValue::Bool(true));
        }
        if let Some(w) = observer_report.trajectory_weighted_score {
            root_span.set_attribute("observer_traj_weighted", AttributeValue::Float(w));
        }
        if let Some(c) = observer_report.convergence_score {
            root_span.set_attribute("observer_convergence", AttributeValue::Float(c));
        }

        // ProcessRewardLearner: learn from E8 trajectory using step-level rewards
        if let Some(ref mut prm_learner) = self.prm {
            let task_string = task.to_string();
            let traj_len = self.state_trajectory.len();
            prm_learner.learn_step(|collector| {
                collector.begin(task_string.clone());
                for (i, state) in self.state_trajectory.iter().enumerate() {
                    collector.record_step(
                        crate::l0_substrate::nt_core_traits::SpecialistType::ReflectionEngine,
                        state.mode,
                        format!("e8_step_{}", i),
                        String::new(),
                        String::new(),
                        None,
                        true,
                        Some(state.meta.0 as f64 / 3.0),
                    );
                }
                if traj_len > 0 {
                    collector.finish(Some(observer_report.quality_score), true);
                } else {
                    collector.finish(None, false);
                }
            });
            root_span.set_attribute(
                "prm_avg_score",
                AttributeValue::Float(prm_learner.avg_recent_score(10)),
            );
            root_span.set_attribute(
                "prm_learning_count",
                AttributeValue::Int(prm_learner.learning_count as i64),
            );
        }

        // Fable-5 pattern matcher: score trajectory alignment against Mythos reasoning phases
        if let Some(ref matcher) = self.fable_matcher {
            if self.state_trajectory.len() >= 2 {
                let traj_modes: Vec<u8> = self.state_trajectory.iter().map(|s| s.mode.0).collect();
                let task_type_idx = match E8TaskType::detect(task) {
                    E8TaskType::General => 0,
                    E8TaskType::Reasoning => 1,
                    E8TaskType::Math => 2,
                    E8TaskType::Coding => 3,
                    E8TaskType::Agentic => 4,
                    E8TaskType::Creative => 5,
                };
                let alignment = matcher.score_alignment_advanced(&traj_modes, task_type_idx, 0.5);
                root_span.set_attribute(
                    "fable_composite",
                    AttributeValue::Float(alignment.composite),
                );
                root_span.set_attribute(
                    "fable_non_linear",
                    AttributeValue::Float(alignment.non_linear_score),
                );
                root_span.set_attribute(
                    "fable_phase_score",
                    AttributeValue::Float(alignment.quality),
                );
                root_span.set_attribute(
                    "fable_transition_score",
                    AttributeValue::Float(alignment.transition_score),
                );

                let sqv = matcher.sqv_score(&traj_modes);
                if sqv > 0.01 {
                    root_span.set_attribute("fable_sqv", AttributeValue::Float(sqv));
                }
                let deep = matcher.detect_deep_reason_pattern(&traj_modes);
                if deep > 0.0 {
                    root_span.set_attribute("fable_deep_reason", AttributeValue::Float(deep));
                }
            }
        }
        result
    }


    pub fn call_llm(&mut self, prompt: &str) -> NeoTrixResult<String> {
        // T2: 叙事自我上下文注入 — P2 融合注入点 + 协调器增强
        let prompt_with_narrative = {
            if let Some(orch) = crate::l5_cognition::nt_mind::nt_mind_background_loop::consciousness_orchestrator::ConsciousnessOrchestrator::get() {
                orch.pre_llm(prompt)
            } else {
                let bridge = crate::l5_cognition::nt_core::capability::consciousness_bridge::bridge();
                match bridge.narrative_prefix() {
                    Some(prefix) => format!("{}{}", prefix, prompt),
                    None => prompt.to_string(),
                }
            }
        };
        let prompt = &prompt_with_narrative;
        if let Some(ref gateway) = self.gateway {
            // cumora 借鉴接线 (T3): ModelRouter T0-T4 分级路由决策驱动实际模型选择。
            // route() 按 prompt 特征 (长度/代码占比/推理关键词) 选 tier → 映射模型名 + max_tokens。
            // default_model 保留为 trace/cost 归因的 fallback 标记; 行为由 route 决策驱动。
            let route_decision = self.router.route(prompt);
            let mut request = LlmRequest::new(&route_decision.model, prompt);
            request.max_tokens = route_decision.max_tokens as u32;
            if let Some(tier) = self.last_effort_tier {
                let think = tier.thinking_budget_tokens();
                let max_tok = tier.max_tokens_budget();
                request.max_tokens = max_tok;
                request.thinking_budget = Some(think);
                if think > 0 {
                    if let Some(msg) = request.messages.first_mut() {
                        msg.content = format!(
                            "{}\n\n[budget] Reason within {} thinking tokens; answer within {} tokens.",
                            msg.content, think, max_tok
                        );
                    }
                }
            }
            let gateway_ref = gateway.clone();
            let response = tokio::task::block_in_place(|| {
                let handle = tokio::runtime::Handle::current();
                handle.block_on(gateway_ref.complete(&request))
            })
            .map_err(|e| NeoTrixError::Brain(format!("LLM call failed: {}", e)))?;
            let prompt_tokens = response.usage.prompt_tokens;
            let completion_tokens = response.usage.completion_tokens;
            if let Some(ref mut ct) = self.cost_tracker {
                ct.record(
                    &self.default_model,
                    prompt_tokens as u64,
                    completion_tokens as u64,
                );
            }
            // T2+T4: 结果记录 + 工具路由 — 通过协调器统一处理
            if crate::l5_cognition::nt_mind::nt_mind_background_loop::consciousness_orchestrator::ConsciousnessOrchestrator::get().is_some() {
                // 记录 LLM 调用结果
                crate::l5_cognition::nt_core::capability::consciousness_bridge::bridge().post_llm_record(true);
                // 如果需要工具，通过协调器的 ValueGate + NativeBus 统一路由
                if response.content.contains("[tool_call:") {
                    log::info!("[engine] tool call routed through orchestrator");
                    // 上层 agent 通过 orch.dispatch_tool() 执行，此处标记就绪
                }
            }
            Ok(response.content)
        } else {
            Err(NeoTrixError::Brain("No LLM provider configured".into()))
        }
    }

    pub(crate) fn _call_llm_with_ctx(&mut self, ctx: &str, prompt: &str) -> NeoTrixResult<String> {
        self.call_llm(&format!("{}\n\n{}", ctx, prompt))
    }

    /// 生产 LLM 评审路径 — 用引擎已接线的 gateway 作为 LLM 法官, 走异步评审组
    /// (run_async + JudgeRegistry + LLMJudgeAdapter, R-P79: 接线生产路径)。
    ///
    /// 法官即当前 provider 本身 (GatewayV2 impl LlmProvider), 家族 = Producer;
    /// 机械护栏仍确定性前置, LLM 只是聚合打分器。无 gateway 时降级返回 None,
    /// 调用方沿用同步启发式评审组, 不阻断执行。
    pub fn llm_judge(
        &mut self,
        candidate: &str,
        claims: &[&str],
        evidence_ids: &[String],
    ) -> Option<crate::l5_cognition::nt_core_gate::PanelVerdict> {
        use crate::l5_cognition::nt_core_gate::{JudgeFamily, JudgeInput, JudgePanel, JudgeRegistry};
        let provider = self.gateway.clone()?;
        let input = JudgeInput {
            candidate: candidate.to_string(),
            claims: claims
                .iter()
                .map(|c| crate::l5_cognition::nt_core_gate::Claim::new(c, &[]))
                .collect(),
            evidence_ids: evidence_ids.to_vec(),
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
            rubric: None,
            samples: 1,
            attestation: None,
        };
        let registry =
            JudgeRegistry::new().register(JudgeFamily::None, provider, &self.default_model);
        let judges = registry.build_async_judges();
        let refs: Vec<&dyn crate::l5_cognition::nt_core_gate::AsyncPanelJudge> =
            judges.iter().map(|j| j.as_ref()).collect();
        let panel = JudgePanel::default_panel();
        Some(tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(panel.run_async(&input, &refs))
        }))
    }
}

/// Detect if an LLM response is a refusal (empty, apology, or explicit refusal patterns).
pub fn _detect_refusal_response(response: &str) -> bool {
    let trimmed = response.trim();
    if trimmed.is_empty() || trimmed.len() < 5 {
        return true;
    }
    let lower = trimmed.to_lowercase();
    // Common refusal/apology patterns
    let patterns = [
        "i cannot",
        "i can't",
        "i'm sorry",
        "i am sorry",
        "i apologize",
        "i'm not able",
        "i am not able",
        "i'm unable",
        "i am unable",
        "i cannot fulfill",
        "i can't fulfill",
        "cannot provide",
        "can't provide",
        "cannot assist",
        "can't assist",
        "not allowed to",
        "against my policy",
        "against my guidelines",
        "i don't feel comfortable",
        "sorry, but i cannot",
        "sorry, i cannot",
        "as an ai",
        "as an ai language model",
        "i'm designed to",
        "i was designed to",
    ];
    patterns.iter().any(|p| lower.contains(p))
}
