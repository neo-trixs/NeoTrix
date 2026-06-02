//! 多视角推理 + Cascade 推理

use chrono::Utc;

use crate::neotrix::error::NeoTrixResult;
use crate::neotrix::nt_mind::reasoning_engine::ReasoningEngine;
use crate::neotrix::nt_mind::reasoning_types::{
    CascadeConfig, CascadeResult, ReasoningType, ReasoningMethod,
    PerspectiveLens, ReasoningTrace,
};
use crate::neotrix::nt_mind::core::CapabilityVector;

impl ReasoningEngine {
    pub fn reason_multi_perspective(&mut self, task: &str, methods: &[ReasoningMethod]) -> Vec<ReasoningTrace> {
        let mode = self.select_mode(task);
        self.current_state = self.current_state.transition_to(mode);
        self.state_trajectory.push(self.current_state);

        let mut traces = Vec::new();
        for method in methods {
            let prompt = format!(
                "[Reasoning Mode: {}, Method: {:?}]\n{}\n\nTask: {}",
                mode.mode_name(),
                method,
                method.description(),
                task
            );
            match self.call_llm(&prompt) {
                Ok(response) => {
                    let score = (response.len() as f64 / 1000.0).clamp(0.1, 0.9);
                    let trace = ReasoningTrace {
                        id: uuid::Uuid::new_v4().to_string(),
                        reasoning_type: ReasoningType::TaskSolving,
                        reasoning_method: Some(*method),
                        perspective_lens: None,
                        task: task.to_string(),
                        prompt,
                        llm_response: response,
                        error_context: None,
                        outcome_score: score,
                        success: score > 0.5,
                        timestamp: Utc::now().timestamp(),
                    };
                    let score_ = trace.outcome_score;
                    self.traces.push(trace.clone());
                    traces.push(trace);
                    self.update_capability_from_trace(score_);
                }
                Err(_) => continue,
            }
        }
        self.observer_analyze(task);
        traces
    }

    pub fn reason_with_lenses(&mut self, task: &str, lenses: &[PerspectiveLens]) -> Vec<ReasoningTrace> {
        let mode = self.select_mode(task);
        self.current_state = self.current_state.transition_to(mode);
        self.state_trajectory.push(self.current_state);

        let mut traces = Vec::new();
        for lens in lenses {
            let prompt = format!(
                "[Perspective: {:?}]\n{}\n\nAnalyze from this perspective:\n{}",
                lens,
                lens.description(),
                task
            );
            match self.call_llm(&prompt) {
                Ok(response) => {
                    let score = (response.len() as f64 / 800.0).clamp(0.1, 0.9);
                    let trace = ReasoningTrace {
                        id: uuid::Uuid::new_v4().to_string(),
                        reasoning_type: ReasoningType::Conversation,
                        reasoning_method: None,
                        perspective_lens: Some(*lens),
                        task: task.to_string(),
                        prompt,
                        llm_response: response,
                        error_context: None,
                        outcome_score: score,
                        success: score > 0.5,
                        timestamp: Utc::now().timestamp(),
                    };
                    let score_ = trace.outcome_score;
                    self.traces.push(trace.clone());
                    traces.push(trace);
                    self.update_capability_from_trace(score_);
                }
                Err(_) => continue,
            }
        }
        self.observer_analyze(task);
        traces
    }

    fn update_capability_from_trace(&mut self, score: f64) {
        let inf_idx = CapabilityVector::index_from_name("inference_depth").unwrap_or(8);
        let cur = self.brain.capability.arr()[inf_idx];
        self.brain.capability.arr_mut()[inf_idx] = (cur + score * 0.02).min(1.0);
        self.brain.capability.normalize();
    }

    pub fn reason_cascade(&mut self, task: &str, config: Option<CascadeConfig>) -> NeoTrixResult<CascadeResult> {
        let mode = self.select_mode(task);
        self.current_state = self.current_state.transition_to(mode);
        self.state_trajectory.push(self.current_state);

        let cfg = config.unwrap_or_default();
        if !cfg.enabled {
            let deep = self.reason(task)?;
            self.observer_analyze(task);
            return Ok(CascadeResult {
                fast_response: deep.clone(),
                escalated: false,
                deep_response: None,
                confidence: 1.0,
                final_output: deep,
            });
        }

        let fast_prompt = format!(
            "Classify this task's complexity (simple/complex) and provide a quick answer.\n\
             Keep response under {} tokens.\n\nTask: {}",
            cfg.fast_max_tokens, task
        );
        let fast_response = self.call_llm_with_ctx(&fast_prompt, cfg.fast_context_size)?;

        let confidence = if fast_response.len() < 100 {
            0.85
        } else if fast_response.len() > 500 {
            0.4
        } else {
            0.6
        };

        if confidence >= cfg.confidence_threshold {
            self.observer_analyze(task);
            return Ok(CascadeResult {
                fast_response: fast_response.clone(),
                escalated: false,
                deep_response: None,
                confidence,
                final_output: fast_response,
            });
        }

        let deep_prompt = format!(
            "Task: {}\n\nPreliminary analysis: {}\n\nProvide a comprehensive solution.",
            task, fast_response
        );
        let deep_response = self.call_llm_with_ctx(&deep_prompt, cfg.deep_context_size)?;

        self.observer_analyze(task);
        Ok(CascadeResult {
            fast_response,
            escalated: true,
            deep_response: Some(deep_response.clone()),
            confidence,
            final_output: deep_response,
        })
    }
}
