use crate::core::nt_core_self::AttentionDomain;
use crate::core::{ReasoningHexagram, MODE_DESCRIPTIONS, MODE_NAMES};

use super::CoreReasoningPlan;
use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};
use crate::neotrix::nt_io_mention::resolve_mentions;
use crate::neotrix::nt_io_provider::types::{LlmRequest, Message, Role};
use crate::neotrix::nt_mind::reasoning_types::ReasoningType;
use crate::neotrix::nt_shield_prompt::{default_guard, RiskLevel};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

use super::{CostRecord, ReasoningEngine, MAX_COST_LOG, MAX_STATE_TRAJECTORY};

impl ReasoningEngine {
    /// Core-First Reasoning: plan → LLM executes → core reviews
    pub fn reason(&mut self, task: &str) -> NeoTrixResult<String> {
        let plan = self.plan_reasoning(task);
        let result = self.reason_with_plan(task, &plan);

        match result {
            Ok(text) => {
                self.traces_since_distill += 1;
                if self.traces_since_distill >= self.distill_interval {
                    self.self_iterate();
                    self.traces_since_distill = 0;
                }
                self.core_review(task, &text, false);
                return Ok(text);
            }
            Err(e) => {
                self.core_review(task, "", false);
                return Err(e);
            }
        }
    }

    /// Build a plan-based prompt from the core reasoning plan.
    /// Resolves @-mention file references in the task string.
    fn build_plan_prompt(&self, task: &str, plan: &CoreReasoningPlan) -> String {
        let memories = self.bank.retrieve_relevant(task, None, 3);
        let memory_context: String = memories
            .iter()
            .map(|m| {
                format!(
                    "  [{}] {} (reward={:.2})",
                    if m.success { "OK" } else { "FAIL" },
                    m.task_description,
                    m.reward
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let artifact_context = self.build_artifact_context(task);
        let kb_context = self.kb_context(task);

        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let (task_with_mentions, mentions) = resolve_mentions(task, &cwd);
        let mention_block: String = if mentions.is_empty() {
            String::new()
        } else {
            let mut block = String::from("\nReferenced files:\n");
            for m in &mentions {
                let flag = if m.truncated { " (truncated)" } else { "" };
                block.push_str(&format!(
                    "  [📎 {}] {} lines{flag}\n",
                    m.path.display(),
                    m.lines,
                ));
            }
            block
        };

        let guidance_blocks: String = plan
            .guidance
            .iter()
            .enumerate()
            .map(|(i, g)| format!("    {}. {}", i + 1, g))
            .collect::<Vec<_>>()
            .join("\n");
        let avoid_blocks: String = if plan.avoid_patterns.is_empty() {
            "    None".to_string()
        } else {
            plan.avoid_patterns
                .iter()
                .map(|a| format!("    - {}", a))
                .collect::<Vec<_>>()
                .join("\n")
        };
        let domain_str: String = plan
            .domains
            .iter()
            .map(|d| format!("    - {:?}", d))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "You are NeoTrix. Follow the guidance below.\n\n\
             == CORE REASONING PLAN ==\n\
             Guidance mode: {}\n\
             \n\
             Attention domains:\n{domain_str}\n\
             \n\
             Reasoning guidance:\n{guidance_blocks}\n\
             \n\
             Avoid these patterns:\n{avoid_blocks}\n\
             \n\
             == EXECUTION ==\n\
             Task: {task_with_mentions}\n\
             {mention_block}\
             Past experience:\n{memory_context}\n\
             {artifact_context}\n\
             {kb_context}\n\
             Execute the reasoning plan above. Think step by step within the guidance boundaries.",
            plan.mode_name,
        )
    }

    /// Build an LlmRequest with the core's reasoning plan injected.
    fn build_request(
        &mut self,
        task: &str,
        plan: &CoreReasoningPlan,
        image_data: Option<Vec<u8>>,
    ) -> NeoTrixResult<(LlmRequest, ReasoningType, String)> {
        let rtype = self.mode_to_reasoning_type(&plan.e8_mode);
        let prompt = self.build_plan_prompt(task, plan);
        let final_prompt = prompt;

        let guard = default_guard();
        let report = guard.analyze(&final_prompt);
        match report.risk {
            RiskLevel::Dangerous => {
                return Err(NeoTrixError::Brain(format!(
                    "LLM 调用被阻止: 检测到注入模式 (conf={:.2})",
                    report.confidence
                )));
            }
            RiskLevel::Suspicious => {
                log::warn!(
                    "LLM prompt 可疑: evasion={:?}, semantic={:.2}",
                    report.evasion_techniques,
                    report.semantic_similarity
                );
            }
            RiskLevel::Safe => {}
        }

        let route = self.router.route(&final_prompt);
        let model_name = self.default_model.clone();
        let mut request = LlmRequest::new(&model_name, &final_prompt);
        request.max_tokens = route.max_tokens as u32;
        request.image_data = image_data;

        // Inject self-model as System prompt if available — Principle #11: Self Is Not a File
        if let Some(ref system_prompt) = self.self_model {
            request.messages.insert(0, Message {
                role: Role::System,
                content: system_prompt.clone(),
                tool_calls: None,
                tool_call_id: None,
            });
        }

        // Apply context budget if set — trim prompt messages to fit allocation
        if let Some(ref budget) = self.context_budget {
            if let Some(prompt_msg) = request.messages.iter_mut().find(|m| {
                m.role == Role::User
            }) {
                let budget_tokens = budget.token_budget_for(
                    &crate::core::nt_core_context::context_budget::BudgetSourceType::Prompt,
                );
                if budget_tokens > 0 && prompt_msg.content.len() / 4 > budget_tokens {
                    let max_chars = budget_tokens * 4;
                    let trimmed: String = prompt_msg.content.chars().take(max_chars).collect();
                    let orig_tokens = prompt_msg.content.len() / 4;
                    let new_tokens = trimmed.len() / 4;
                    log::info!(
                        "context_budget trimmed prompt: {}→{} tokens ({:.0}%)",
                        orig_tokens, new_tokens,
                        new_tokens as f64 / orig_tokens.max(1) as f64 * 100.0,
                    );
                    prompt_msg.content = trimmed;
                }
            }
        }

        Ok((request, rtype, final_prompt))
    }

    /// Core-First prompt construction: inject the full reasoning plan as instructions.
    /// Automatically handles web search tool calls (NEED_SEARCH: prefix) from the LLM.
    pub(super) fn reason_with_plan(
        &mut self,
        task: &str,
        plan: &CoreReasoningPlan,
    ) -> NeoTrixResult<String> {
        let rtype = self.mode_to_reasoning_type(&plan.e8_mode);
        let mut prompt = self.build_plan_prompt(task, plan);

        for _round in 0..3 {
            let response = self.call_llm(&prompt)?;

            if let Some(query) = self.extract_search_request(&response) {
                match self.tool_call_nt_world_search(&query, 5) {
                    Ok(search_text) => {
                        prompt = format!(
                            "{}\n\n== WEB SEARCH RESULTS ==\n{}\n\nContinue with the above information.",
                            prompt, search_text
                        );
                        continue;
                    }
                    Err(e) => {
                        prompt = format!("{}\n\n[Web search failed: {}]", prompt, e);
                        continue;
                    }
                }
            }

            self.record_trace(rtype, task, &prompt, &response, None, 0.5);
            self.learn_from_trace(task, &response);
            return Ok(response);
        }

        Err(NeoTrixError::Brain(
            "Exceeded maximum tool call rounds (3)".to_string(),
        ))
    }

    /// Check if the LLM response contains a web search request.
    fn extract_search_request(&self, response: &str) -> Option<String> {
        for line in response.lines() {
            let trimmed = line.trim();
            if let Some(query) = trimmed.strip_prefix("NEED_SEARCH:") {
                let q = query.trim().to_string();
                if !q.is_empty() {
                    return Some(q);
                }
            }
        }
        None
    }

    /// Execute a web search and return structured text suitable for LLM context.
    pub fn tool_call_nt_world_search(&self, query: &str, count: usize) -> Result<String, String> {
        match self.nt_world_search_tool {
            Some(ref tool) => tool.search(query, count),
            None => Err(
                "Web search tool not configured. Enable via with_nt_world_search().".to_string(),
            ),
        }
    }

    /// Stream reasoning from LLM, sending tokens through an mpsc channel.
    /// Returns the full accumulated response and a Receiver for streaming.
    /// Core-First: plan → stream → core_review.
    pub async fn reason_stream(
        &mut self,
        task: &str,
        image_data: Option<Vec<u8>>,
    ) -> NeoTrixResult<(String, mpsc::Receiver<String>)> {
        let plan = self.plan_reasoning(task);
        let (request, rtype, prompt_text) = self.build_request(task, &plan, image_data.clone())?;

        let start = std::time::Instant::now();
        let mut receiver = match self.llm.stream_complete(&request).await {
            Ok(r) => r,
            Err(e) => {
                self.core_review(task, "", image_data.is_some());
                return Err(NeoTrixError::Brain(format!("stream start: {}", e)));
            }
        };

        let (tx, rx) = mpsc::channel::<String>(256);
        let mut full_response = String::new();

        loop {
            match timeout(Duration::from_secs(30), receiver.recv()).await {
                Ok(Some(chunk_result)) => match chunk_result {
                    Ok(chunk) => {
                        full_response.push_str(&chunk.content);
                        if tx.send(chunk.content).await.is_err() {
                            log::warn!("engine_core stream send failed: channel closed");
                        }
                    }
                    Err(e) => {
                        if tx.send(format!("\n[LLM Error: {}]", e)).await.is_err() {
                            log::warn!("engine_core error send failed: channel closed");
                        }
                        break;
                    }
                },
                Ok(None) => break,
                Err(_) => continue,
            }
        }

        let elapsed = start.elapsed();
        let elapsed_ms = elapsed.as_millis() as u64;
        self.llm_call_count += 1;
        self.llm_total_time_ms += elapsed_ms;
        self.llm_last_duration_ms = elapsed_ms;

        self.cost_log.push(CostRecord {
            timestamp: chrono::Utc::now().timestamp(),
            tier: "stream".to_string(),
            model: self.default_model.clone(),
            token_estimate: (full_response.len() / 4).max(1),
            cost_estimate_usd: 0.0,
            duration_ms: elapsed_ms,
            reasoning_type: format!("{:?}", rtype),
            success: true,
        });
        if self.cost_log.len() > MAX_COST_LOG * 2 {
            self.cost_log.drain(0..self.cost_log.len() - MAX_COST_LOG);
        }

        self.record_trace(rtype, task, &prompt_text, &full_response, None, 0.8);
        self.learn_from_trace(task, &full_response);

        self.traces_since_distill += 1;
        if self.traces_since_distill >= self.distill_interval {
            self.self_iterate();
            self.traces_since_distill = 0;
        }

        self.core_review(task, &full_response, image_data.is_some());
        self.log_consciousness(task, &full_response, plan.e8_mode);

        Ok((full_response, rx))
    }

    fn log_consciousness(&self, task: &str, response: &str, mode: ReasoningHexagram) {
        let Some(ref kb) = self.kb else { return };
        let summary = if response.len() > 200 {
            &response[..200]
        } else {
            response
        };
        let phi = mode.0 as f64 / 64.0;
        let coherence = self.current_state.meta.0 as f64 / 3.0;
        let details = format!(
            "task={}, mode={}, response_preview={}",
            task, mode.0, summary
        );
        let _ = kb.record_consciousness_snapshot(phi, coherence, true, "daily_reasoning", &details);
    }

    fn kb_context(&self, task: &str) -> String {
        let Some(ref kb) = self.kb else {
            return String::new();
        };
        match kb.search(task, 5) {
            Ok(results) if !results.is_empty() => {
                let entries: Vec<String> = results
                    .iter()
                    .map(|r| format!("  · {} (score: {:.3})", r.node.title, r.score))
                    .collect();
                format!("Knowledge context:\n{}\n", entries.join("\n"))
            }
            _ => String::new(),
        }
    }

    pub(super) fn mode_to_reasoning_type(&self, mode: &ReasoningHexagram) -> ReasoningType {
        let abs = mode.abstraction();
        let scp = mode.scope();
        let mtd = mode.method();
        let dep = mode.depth();
        match (abs, scp, mtd, dep) {
            (0, 0, 0, _) => ReasoningType::TaskSolving,
            (0, 0, 1, _) => ReasoningType::ErrorDebugging,
            (0, 1, 0, 0) => ReasoningType::General,
            (0, 1, 0, 1) => ReasoningType::KnowledgeQuery,
            (0, 1, 1, _) => ReasoningType::General,
            (1, 0, 0, _) => ReasoningType::PrdGeneration,
            (1, 0, 1, _) => ReasoningType::General,
            (1, 1, 0, _) => ReasoningType::KnowledgeQuery,
            (1, 1, 1, _) => ReasoningType::Conversation,
            _ => ReasoningType::General,
        }
    }

    pub(super) fn trim_trajectory(&mut self) {
        if self.state_trajectory.len() > MAX_STATE_TRAJECTORY {
            let remove = self.state_trajectory.len() - MAX_STATE_TRAJECTORY;
            self.state_trajectory.drain(0..remove);
        }
    }

    pub fn reason_through_path(
        &mut self,
        task: &str,
        path: &[ReasoningHexagram],
    ) -> NeoTrixResult<Vec<String>> {
        let mut results = Vec::with_capacity(path.len());
        for (i, &mode) in path.iter().enumerate() {
            let prev_state = self.current_state;
            self.current_state = self.current_state.transition_to(mode);
            self.state_trajectory.push(self.current_state);
            self.trim_trajectory();

            let mode_name = MODE_NAMES[mode.0 as usize];
            let mode_desc = MODE_DESCRIPTIONS[mode.0 as usize];

            if let Some(ref mut policy) = self.e8_policy {
                policy.set_previous(mode);
            }

            let plan = CoreReasoningPlan {
                strategy: self.guide_strategy(mode_name, &[]),
                domains: vec![AttentionDomain::Code, AttentionDomain::Planning],
                e8_mode: mode,
                mode_name: mode_name.to_string(),
                mode_desc: mode_desc.to_string(),
                crystal_used: self.last_crystal_used,
                specialist: "path".to_string(),
                guidance: vec![format!("Path step {}: {} mode", i + 1, mode_name)],
                avoid_patterns: Vec::new(),
            };
            self.last_core_plan = Some(plan.clone());

            let result = self.reason_with_plan(task, &plan);
            match result {
                Ok(text) => {
                    self.core_review(task, &text, false);
                    if i > 0 {
                        let prev = path[i - 1];
                        let transition_info = format!(
                            "\n[Transition {i}: {} → {}, resonance={}, bits flipped={:06b}]",
                            MODE_NAMES[prev.0 as usize],
                            MODE_NAMES[mode.0 as usize],
                            prev.resonance_strength(&mode),
                            prev.0 ^ mode.0,
                        );
                        results.push(transition_info);
                    }
                    results.push(text);
                }
                Err(e) => {
                    self.current_state = prev_state;
                    self.state_trajectory.pop();
                    self.core_review(task, "", false);
                    return Err(e);
                }
            }
        }

        Ok(results)
    }

    pub fn reason_complement(&mut self, task: &str) -> NeoTrixResult<String> {
        let prev_state = self.current_state;
        let complement = self.current_state.mode.complement();
        self.current_state = self.current_state.transition_to(complement);
        self.state_trajectory.push(self.current_state);
        self.trim_trajectory();

        let mode_name = MODE_NAMES[complement.0 as usize];
        let mode_desc = MODE_DESCRIPTIONS[complement.0 as usize];

        if let Some(ref mut policy) = self.e8_policy {
            policy.set_previous(complement);
        }

        let plan = CoreReasoningPlan {
            strategy: self.guide_strategy(mode_name, &[]),
            domains: vec![AttentionDomain::Code, AttentionDomain::Planning],
            e8_mode: complement,
            mode_name: mode_name.to_string(),
            mode_desc: mode_desc.to_string(),
            crystal_used: self.last_crystal_used,
            specialist: "complement".to_string(),
            guidance: vec![format!("Complementary reasoning from {:?}", complement)],
            avoid_patterns: Vec::new(),
        };
        self.last_core_plan = Some(plan.clone());

        let result = self.reason_with_plan(task, &plan);
        match result {
            Ok(text) => {
                self.core_review(task, &text, false);
                Ok(text)
            }
            Err(e) => {
                self.current_state = prev_state;
                self.state_trajectory.pop();
                self.core_review(task, "", false);
                Err(e)
            }
        }
    }
}
