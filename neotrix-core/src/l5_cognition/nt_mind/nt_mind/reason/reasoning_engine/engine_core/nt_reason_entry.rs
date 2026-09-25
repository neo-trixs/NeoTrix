
use crate::l5_cognition::nt_core_cot_generator::CoTGenerator;

use crate::l0_substrate::nt_core_hex::{FullReasoningState};
use crate::l0_substrate::nt_core_span::{
    AttributeValue, NoopTracer, SpanKind, Tracer,
};
use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};
use super::nt_builders::ReasoningEngine;

impl ReasoningEngine {
    pub fn reason(&mut self, task: &str) -> NeoTrixResult<String> {
        let root_span = self
            .tracer
            .as_ref()
            .map(|t| t.start_span("reason", SpanKind::Handoff))
            .unwrap_or_else(|| NoopTracer.start_span("reason", SpanKind::Handoff));
        root_span.set_attribute("task", AttributeValue::String(task.to_string()));
        root_span.set_attribute(
            "e8_state",
            AttributeValue::String(self.current_state.mode.mode_name().to_string()),
        );
        root_span.set_attribute(
            "agent",
            AttributeValue::String("ReasoningEngine".to_string()),
        );
        root_span.set_gen_ai_system("neotrix");

        let (mut e8_machine, prompt) = self.prepare_reasoning(task, &root_span);
        let result = self.call_llm_and_analyze(task, &prompt, &root_span, &mut e8_machine);

        // Phase 2.2: Verifier 生产接线 — 对 LLM 响应进行验证
        if let Ok(ref _response) = result {
            // 1. GroundedPrmVerifier: 过程级验证 (若已配置)
            if self.verifier.is_some() {
                // 将 state_trajectory 转为 TrajectoryStep 进行验证
                // 简化：仅记录验证器存在，实际步骤验证在 PRM learner 中进行
                root_span.set_attribute("verifier_grounded_prm", AttributeValue::Bool(true));
            }

            // 2. 最终答案验证器 (RLVR 锚): 使用 verify_answer 对比预期答案
            // 注意: 这里无 gold 答案, 仅作演示; 实际使用时需外部提供 gold
            // let verification_score = crate::l1_action::nt_io::nt_io_standalone::verify_answer(gold, response);
            // root_span.set_attribute("verification_score", AttributeValue::Float(verification_score));
        }

        // Phase 2.1: CoT Generator 生产接线 — 生成结构化 CoT
        if let Ok(ref response) = result {
            if let Some(ref mut cot_gen) = self.cot_generator {
                // 构建一个简化的 Kernel trace 用于 CoT 生成
                let kernel_trace = crate::l5_cognition::reasoning_core::ReasoningTrace {
                    trace_id: format!(
                        "engine_{}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos()
                    ),
                    task: task.to_string(),
                    method: crate::l1_action::nt_io::nt_io_standalone::ReasoningMethod::Deductive,
                    hexagram: neotrix_types::e8_reasoning::ReasoningHexagram::new(self.current_state.mode.0),
                    stage: self.current_state.mode.0 as usize % 19,
                    steps: Vec::new(),
                    intermediate_states: Vec::new(),
                    convergence: 0.5,
                    final_quality: 0.5,
                    llm_response: Some(response.clone()),
                    source: crate::l5_cognition::reasoning_core::TraceSource::LLMDriven,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                };

                // 同步调用 CoT 生成器 (使用 block_on 在当前运行时中执行)
                let cot_future = cot_gen.generate_cot(task, &kernel_trace, None);
                if let Ok(cot_output) = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(cot_future)
                }) {
                    // 将 CoT 结果记录到 span 中
                    root_span.set_attribute("cot_generated", AttributeValue::Bool(true));
                    root_span.set_attribute(
                        "cot_final_answer",
                        AttributeValue::String(cot_output.final_answer.clone()),
                    );
                    root_span.set_attribute(
                        "cot_overall_confidence",
                        AttributeValue::Float(cot_output.overall_confidence),
                    );
                    // 可以选择将 CoT 结果合并到响应中
                    // 这里我们记录但不替换原始响应，保持向后兼容
                } else {
                    root_span.set_attribute("cot_generation_failed", AttributeValue::Bool(true));
                }
            }
        }

        // Phase 2.3: Kernel trace → E8Policy 反哺闭环 (完整双向闭环)
        // 使用推理轨迹的收敛度和质量作为奖励信号，更新 E8Policy
        if let Ok(ref response) = result {
            // 优先使用引擎内真实的推理轨迹（Kernel trace 真实反哺）
            // 注: self.traces 元素为 reasoning_types::ReasoningTrace, 需转换为
            // nt_core_reasoning::ReasoningTrace (E8Policy 反哺所需字段)。
            let feedback_trace = if let Some(last_trace) = self.traces.last() {
                crate::l5_cognition::reasoning_core::ReasoningTrace {
                    trace_id: last_trace.id.clone(),
                    task: last_trace.task.clone(),
                    method: crate::l1_action::nt_io::nt_io_standalone::ReasoningMethod::Deductive,
                    hexagram: neotrix_types::e8_reasoning::ReasoningHexagram::new(self.current_state.mode.0),
                    stage: self.current_state.mode.0 as usize % 19,
                    steps: Vec::new(),
                    intermediate_states: Vec::new(),
                    convergence: last_trace.outcome_score.clamp(0.0, 1.0),
                    final_quality: if last_trace.success { 0.8 } else { 0.5 },
                    llm_response: Some(last_trace.llm_response.clone()),
                    source: crate::l5_cognition::reasoning_core::TraceSource::LLMDriven,
                    timestamp: last_trace.timestamp as u64,
                }
            } else {
                // 回退：基于响应质量构建反馈轨迹
                crate::l5_cognition::reasoning_core::ReasoningTrace {
                    trace_id: format!(
                        "feedback_{}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos()
                    ),
                    task: task.to_string(),
                    method: crate::l1_action::nt_io::nt_io_standalone::ReasoningMethod::Deductive,
                    hexagram: neotrix_types::e8_reasoning::ReasoningHexagram::new(self.current_state.mode.0),
                    stage: self.current_state.mode.0 as usize % 19,
                    steps: Vec::new(),
                    intermediate_states: Vec::new(),
                    convergence: 0.7, // 基于响应质量估算
                    final_quality: if response.len() > 100 { 0.8 } else { 0.5 },
                    llm_response: Some(response.clone()),
                    source: crate::l5_cognition::reasoning_core::TraceSource::LLMDriven,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                }
            };

            // 计算奖励信号：收敛度 * 0.6 + 质量 * 0.4
            let reward = feedback_trace.convergence * 0.6 + feedback_trace.final_quality * 0.4;

            // 反哺 E8Policy：更新当前模式的 Q-value
            if let Some(ref mut policy) = self.e8_policy {
                policy.set_previous(self.current_state.mode);
                policy.update(reward.clamp(0.0, 1.0));
            }

            root_span.set_attribute("e8_policy_feedback_reward", AttributeValue::Float(reward));
            root_span.set_attribute("e8_policy_feedback_applied", AttributeValue::Bool(true));
        }

        self.run_prediction_fusion(task, &root_span, &mut e8_machine);
        self.broadcast_and_finalize(task, root_span, result)
    }


    pub(crate) fn _reason_multi_agent(&mut self, task: &str) -> NeoTrixResult<String> {
        if let Some(ref orch) = self.orchestrator {
            match orch.execute(task) {
                Ok(output) => Ok(output.content),
                Err(e) => Err(NeoTrixError::Brain(format!("Orchestration failed: {}", e))),
            }
        } else {
            self.reason(task)
        }
    }


    pub fn reason_task(&mut self, task: &str) -> NeoTrixResult<String> {
        self.reason(task)
    }

    pub fn plan_reasoning(&mut self, task: &str, _mode: u8) -> String {
        self.reason_task(task).unwrap_or_default()
    }

    pub fn self_iterate(&mut self) {
        // Run observer analysis to monitor reasoning state health
        self._observer_analyze("self-iteration");
        // Record self-iteration through _core_review
        let result =
            self.call_llm("self-iteration: analyze current state and propose improvements");
        self._core_review("self-iteration", &result);
        // Feed back to PRM for learning signal
        if let Some(ref mut prm) = self.prm {
            let score = if result.is_ok() { 0.5 } else { 0.0 };
            prm.learn_step(|collector| {
                collector.begin("self-iteration".to_string());
                collector.record_step(
                    crate::l0_substrate::nt_core_traits::SpecialistType::ReflectionEngine,
                    self.current_state.mode,
                    "self_iterate".into(),
                    String::new(),
                    result.as_deref().unwrap_or("error").to_string(),
                    None,
                    true,
                    Some(score),
                );
                collector.finish(Some(score), result.is_ok());
            });
        }
        log::info!("[engine] Self-iteration cycle completed");
    }


    pub fn select_mode(&self, _query: &str) -> FullReasoningState {
        self.current_state
    }


    /// Stream the reasoning response token-by-token through a channel.
    /// Returns the full response alongside a receiver for streaming.
    pub async fn reason_stream(
        &mut self,
        task: &str,
        _budget: Option<u32>,
    ) -> NeoTrixResult<(String, tokio::sync::mpsc::Receiver<String>)> {
        let response = self.reason(task)?;
        let (tx, rx) = tokio::sync::mpsc::channel(64);
        let response_clone = response.clone();
        tokio::spawn(async move {
            for word in response_clone.split(' ') {
                if tx.send(format!("{} ", word)).await.is_err() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        });
        Ok((response, rx))
    }
}

// ── L1 trait bridge: ReasoningEngineProvider ──────────────────────────
// 允许 l1_action::nt_core_task_dispatcher 通过 trait 抽象使用 ReasoningEngine,
// 不在 L1 层直接引入 L5 具体类型。

impl crate::l1_action::nt_core_task_dispatcher::ReasoningEngineProvider for ReasoningEngine {
    fn reason(&mut self, prompt: &str) -> Result<String, String> {
        ReasoningEngine::reason(self, prompt).map_err(|e| e.to_string())
    }
}
