//! 内部帮助方法：LLM 调用、成本报告、trace 记录、学习、编译奖励

use chrono::Utc;

use crate::core::nt_core_consciousness::VsaTagged;
use crate::core::CapabilityVector;
use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};
use crate::neotrix::nt_expert_routing::TaskType;
use crate::neotrix::nt_io_provider::LlmRequest;
use crate::neotrix::nt_mind::context_artifacts::inject_relevant_artifacts;
use crate::neotrix::nt_mind::distillation::AntiPattern;
use crate::neotrix::nt_mind::memory::ReasoningMemory;
use crate::neotrix::nt_mind::model_router::RouterTier;
use crate::neotrix::nt_mind::reasoning_engine::{
    CostRecord, EngineMetrics, ReasoningEngine, ReasoningStats, MAX_COST_LOG, MAX_TRACES,
};
use crate::neotrix::nt_mind::reasoning_types::{ReasoningTrace, ReasoningType};
use crate::neotrix::nt_shield_prompt::RiskLevel;

impl ReasoningEngine {
    pub(super) fn call_llm(&mut self, prompt: &str) -> NeoTrixResult<String> {
        self.call_llm_with_ctx(prompt, 0)
    }

    pub(super) fn call_llm_with_ctx(
        &mut self,
        prompt: &str,
        _context_size: u32,
    ) -> NeoTrixResult<String> {
        let guard = crate::neotrix::nt_shield_prompt::default_guard();
        let report = guard.analyze(prompt);
        match report.risk {
            RiskLevel::Dangerous => {
                return Err(NeoTrixError::Brain(format!(
                    "LLM 调用被阻止: 检测到注入模式 (conf={:.2}, reasons={:?})",
                    report.confidence, report.regex_findings
                )))
            }
            RiskLevel::Suspicious => {
                log::warn!(
                    "LLM prompt 可疑: evasion={:?}, semantic={:.2}, outlier={:.2}",
                    report.evasion_techniques,
                    report.semantic_similarity,
                    report.outlier_score
                );
            }
            RiskLevel::Safe => {}
        }

        let route = self.router.route(prompt);
        let model_name = self.default_model.clone();

        let start = std::time::Instant::now();
        let mut request = LlmRequest::new(&model_name, prompt);
        request.max_tokens = route.max_tokens as u32;
        let response = self.runtime.block_on(self.llm.complete(&request));
        let elapsed = start.elapsed();
        let elapsed_ms = elapsed.as_millis() as u64;
        self.llm_call_count += 1;
        self.llm_total_time_ms += elapsed_ms;
        self.llm_last_duration_ms = elapsed_ms;

        match response {
            Ok(content) => {
                self.cost_log.push(CostRecord {
                    timestamp: Utc::now().timestamp(),
                    tier: route.tier.name().to_string(),
                    model: model_name,
                    token_estimate: route.features.token_estimate,
                    cost_estimate_usd: route.cost_estimate(),
                    duration_ms: elapsed_ms,
                    reasoning_type: String::new(),
                    success: true,
                });
                if self.cost_log.len() > MAX_COST_LOG * 2 {
                    self.cost_log.drain(0..self.cost_log.len() - MAX_COST_LOG);
                }
                let sanitized = crate::neotrix::nt_shield_prompt::default_output_screener()
                    .sanitize(&content.content);
                self.last_llm_tagged = Some(VsaTagged::self_thought(&sanitized));
                Ok(sanitized)
            }
            Err(e) => {
                for _fallback_tier in [RouterTier::T4, RouterTier::T3, RouterTier::T2] {
                    if let Some(mapping) = self.router.fallback(route.tier) {
                        let fb_model = mapping.model.clone();
                        let fb_request = LlmRequest::new(&fb_model, prompt);
                        if let Ok(fb_resp) = self.runtime.block_on(self.llm.complete(&fb_request)) {
                            self.cost_log.push(CostRecord {
                                timestamp: Utc::now().timestamp(),
                                tier: format!("{}-fallback", mapping.tier.name()),
                                model: fb_model,
                                token_estimate: route.features.token_estimate,
                                cost_estimate_usd: mapping.tier.cost_multiplier() * 0.5,
                                duration_ms: elapsed_ms,
                                reasoning_type: String::new(),
                                success: true,
                            });
                            if self.cost_log.len() > MAX_COST_LOG * 2 {
                                self.cost_log.drain(0..self.cost_log.len() - MAX_COST_LOG);
                            }
                            let fb_sanitized =
                                crate::neotrix::nt_shield_prompt::default_output_screener()
                                    .sanitize(&fb_resp.content);
                            self.last_llm_tagged = Some(VsaTagged::self_thought(&fb_sanitized));
                            return Ok(fb_sanitized);
                        }
                    }
                }
                Err(NeoTrixError::Brain(format!("LLM 调用失败: {}", e)))
            }
        }
    }

    pub fn cost_report(&self) -> serde_json::Value {
        let total_cost: f64 = self.cost_log.iter().map(|c| c.cost_estimate_usd).sum();
        let total_duration: u64 = self.cost_log.iter().map(|c| c.duration_ms).sum();
        let by_tier: std::collections::HashMap<String, Vec<&CostRecord>> = self
            .cost_log
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, c| {
                acc.entry(c.tier.clone()).or_default().push(c);
                acc
            });
        let tier_breakdown: serde_json::Value = by_tier.iter().map(|(tier, records)| {
            let count = records.len();
            let cost: f64 = records.iter().map(|r| r.cost_estimate_usd).sum();
            let avg_duration: u64 = records.iter().map(|r| r.duration_ms).sum::<u64>() / count as u64;
            (tier.clone(), serde_json::json!({ "count": count, "total_cost": format!("${:.4}", cost), "avg_duration_ms": avg_duration }))
        }).collect();

        serde_json::json!({
            "total_llm_calls": self.cost_log.len(),
            "total_cost_usd": format!("${:.4}", total_cost),
            "total_duration_ms": total_duration,
            "tier_breakdown": tier_breakdown,
            "last_10": self.cost_log.iter().rev().take(10).map(|c| serde_json::json!({
                "tier": c.tier, "model": c.model, "cost": c.cost_estimate_usd,
                "ms": c.duration_ms, "success": c.success
            })).collect::<Vec<_>>(),
        })
    }

    pub fn infer_reasoning_type(task: &str) -> ReasoningType {
        let t = task.to_lowercase();
        if t.contains("error")
            || t.contains("bug")
            || t.contains("fail")
            || t.contains("exception")
            || t.contains("panic")
            || t.contains("crash")
            || t.contains("错误")
            || t.contains("失败")
        {
            ReasoningType::ErrorDebugging
        } else if t.contains("knowledge")
            || t.contains("what is")
            || t.contains("explain")
            || t.contains("知识")
            || t.contains("什么")
        {
            ReasoningType::KnowledgeQuery
        } else if t.contains("task")
            || t.contains("implement")
            || t.contains("solve")
            || t.contains("任务")
            || t.contains("实现")
            || t.contains("解决")
        {
            ReasoningType::TaskSolving
        } else {
            ReasoningType::Conversation
        }
    }

    pub(super) fn build_context(&mut self, query: &str, rtype: ReasoningType) -> String {
        let tt = match rtype {
            ReasoningType::Conversation => TaskType::General,
            ReasoningType::TaskSolving => TaskType::Planning,
            ReasoningType::ErrorDebugging => TaskType::CodeReview,
            ReasoningType::KnowledgeQuery => TaskType::Research,
            ReasoningType::General => TaskType::General,
            ReasoningType::PrdGeneration => TaskType::Planning,
        };
        let results = self.bank.retrieve_relevant(query, Some(tt), 3);
        self.bank_retrieval_count += 1;
        results
            .iter()
            .map(|m| {
                format!(
                    "  [{}] {}",
                    if m.success { "SUCCESS" } else { "FAILURE" },
                    m.task_description
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn build_artifact_context(&self, query: &str) -> String {
        let Some(ref indexer) = self.artifact_indexer else {
            return String::new();
        };
        let artifacts = inject_relevant_artifacts(indexer.store(), query, 3);
        if artifacts.is_empty() {
            return String::new();
        }
        let mut out = String::from("Relevant project artifacts:\n");
        for a in &artifacts {
            let preview: String = a.content.chars().take(200).collect();
            out.push_str(&format!(
                "  [{}] ({})\n    {}\n",
                a.name,
                a.artifact_type.label(),
                preview
            ));
        }
        out.push('\n');
        out
    }

    pub(super) fn record_trace(
        &mut self,
        rt: ReasoningType,
        task: &str,
        prompt: &str,
        response: &str,
        err: Option<&str>,
        score: f64,
    ) {
        let vsa_tag = self.last_llm_tagged.take();
        self.traces.push(ReasoningTrace {
            id: uuid::Uuid::new_v4().to_string(),
            reasoning_type: rt,
            reasoning_method: None,
            perspective_lens: None,
            task: task.to_string(),
            prompt: prompt.to_string(),
            llm_response: response.to_string(),
            error_context: err.map(|s| s.to_string()),
            outcome_score: score,
            success: score > 0.5,
            timestamp: Utc::now().timestamp(),
            vsa_tag,
        });
        if self.traces.len() > MAX_TRACES {
            self.traces.remove(0);
        }
    }

    pub(super) fn learn_from_trace(&mut self, task: &str, _response: &str) {
        let tt = match Self::infer_reasoning_type(task) {
            ReasoningType::TaskSolving => TaskType::Planning,
            ReasoningType::ErrorDebugging => TaskType::CodeReview,
            ReasoningType::KnowledgeQuery => TaskType::Research,
            _ => TaskType::General,
        };
        let memory = ReasoningMemory::new(task, tt, &[], 0.8);
        self.bank.store(memory);
    }

    pub fn learn_from_execution(
        &mut self,
        task: &str,
        exec_result: &Result<(String, String), String>,
    ) {
        let reward = match exec_result {
            Ok((stdout, stderr)) => {
                let output_score = (stdout.len() as f64 / 500.0).min(0.5);
                let clean_bonus = if stderr.is_empty() { 0.2 } else { 0.0 };
                0.3 + output_score + clean_bonus
            }
            Err(e) => {
                self.anti_patterns.push(AntiPattern {
                    id: uuid::Uuid::new_v4().to_string(),
                    description: format!("Exec failure: {}", e),
                    task_type: TaskType::General,
                    harmful_pattern: std::collections::HashMap::new(),
                    failure_count: 1,
                });
                -0.2
            }
        };
        let depth_idx = CapabilityVector::index_from_name("inference_depth").unwrap_or(8);
        let current = self.brain.capability().arr()[depth_idx];
        self.brain.capability_mut().arr_mut()[depth_idx] =
            (current + reward * 0.05).clamp(0.0, 1.0);
        self.brain.capability_mut().normalize();
        let memory = ReasoningMemory::new(task, TaskType::General, &[], reward.max(0.0));
        self.bank.store(memory);
    }

    pub fn external_compile_reward(&mut self, project_dir: &str) -> f64 {
        if cfg!(test) {
            return 0.0;
        }

        let mut total = 0.0;

        let check_result = std::process::Command::new("cargo")
            .args(["check", "--lib"])
            .current_dir(project_dir)
            .output();

        match check_result {
            Ok(output) if output.status.success() => {
                total += 0.5;
                if let Some(idx) = CapabilityVector::index_from_name("quality_gates") {
                    let cur = self.brain.capability().arr()[idx];
                    self.brain.capability_mut().arr_mut()[idx] = (cur + 0.05).min(1.0);
                }
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                self.anti_patterns.push(AntiPattern {
                    id: uuid::Uuid::new_v4().to_string(),
                    description: format!(
                        "Compile error: {}",
                        &stderr.lines().next().unwrap_or("unknown")
                    ),
                    task_type: TaskType::CodeReview,
                    harmful_pattern: std::collections::HashMap::new(),
                    failure_count: 1,
                });
            }
            Err(e) => {
                log::error!("cargo check 调用失败: {}", e);
            }
        }

        let test_result = std::process::Command::new("cargo")
            .args(["test", "--lib"])
            .current_dir(project_dir)
            .output();

        match test_result {
            Ok(output) if output.status.success() => {
                total += 0.5;
                if let Some(idx) = CapabilityVector::index_from_name("verification") {
                    let cur = self.brain.capability().arr()[idx];
                    self.brain.capability_mut().arr_mut()[idx] = (cur + 0.05).min(1.0);
                }
            }
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let failed = stdout.lines().filter(|l| l.contains("FAILED")).count();
                self.anti_patterns.push(AntiPattern {
                    id: uuid::Uuid::new_v4().to_string(),
                    description: format!("Test failures: {} tests failed", failed),
                    task_type: TaskType::CodeReview,
                    harmful_pattern: std::collections::HashMap::new(),
                    failure_count: failed as u32,
                });
            }
            Err(e) => {
                log::error!("cargo test 调用失败: {}", e);
            }
        }

        self.brain.capability_mut().normalize();

        let summary = format!("External compile/test reward: {:.2}", total);
        let memory = ReasoningMemory::new(&summary, TaskType::General, &[], total);
        self.bank.store(memory);

        total
    }

    pub fn metrics(&self) -> EngineMetrics {
        EngineMetrics {
            total_llm_calls: self.llm_call_count,
            total_llm_time_ms: self.llm_total_time_ms,
            last_call_duration_ms: self.llm_last_duration_ms,
            bank_retrieval_count: self.bank_retrieval_count,
            total_traces: self.traces.len() as u64,
            principles_count: self.principles.len() as u64,
            anti_patterns_count: self.anti_patterns.len() as u64,
        }
    }

    pub fn traces_to_tree(&self, limit: usize) -> String {
        let traces: Vec<&ReasoningTrace> = self.traces.iter().rev().take(limit).collect();
        if traces.is_empty() {
            return "(无推理记录)".to_string();
        }

        let mut out = String::new();
        out.push_str("因果链 (ReasoningTrace)\n");
        out.push_str("═══════════════════════\n");
        for (i, t) in traces.iter().enumerate() {
            let prefix = if i == traces.len() - 1 {
                " └─ "
            } else {
                " ├─ "
            };
            out.push_str(&format!(
                "{}#{} [{}] {}\n",
                prefix,
                traces.len() - i,
                format!("{:?}", t.reasoning_type).to_lowercase(),
                t.task.chars().take(40).collect::<String>()
            ));

            let method = t
                .reasoning_method
                .map(|m| format!("{:?}", m))
                .unwrap_or_default();
            let lens = t
                .perspective_lens
                .map(|l| format!("{:?}", l))
                .unwrap_or_default();
            let sub_prefix = if i == traces.len() - 1 {
                "    "
            } else {
                " │  "
            };

            if !method.is_empty() {
                out.push_str(&format!("{} ├─ method: {}\n", sub_prefix, method));
            }
            if !lens.is_empty() {
                out.push_str(&format!("{} ├─ lens: {}\n", sub_prefix, lens));
            }
            out.push_str(&format!(
                "{} └─ score: {:.2} (success: {})\n",
                sub_prefix, t.outcome_score, t.success
            ));
        }
        out
    }

    pub fn stats(&self) -> ReasoningStats {
        let total = self.traces.len() as f64;
        let successes = self.traces.iter().filter(|t| t.success).count() as f64;
        ReasoningStats {
            total_traces: self.traces.len(),
            success_rate: if total > 0.0 { successes / total } else { 0.0 },
            principles_count: self.principles.len(),
            anti_patterns_count: self.anti_patterns.len(),
            last_type: self.traces.last().map(|t| t.reasoning_type),
        }
    }
}
