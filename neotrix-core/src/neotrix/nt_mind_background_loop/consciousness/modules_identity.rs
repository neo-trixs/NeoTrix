use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use crate::core::nt_core_identity::{InsightVerdict, ValueAlignmentGate};

use super::types::ConsciousnessIntegration;

impl ConsciousnessIntegration {
    pub fn handle_identity_cycle(&mut self) -> String {
        let mut events: Vec<String> = Vec::new();

        self.handle_inter_session_reflection();

        let context_vsa = self.attractor_state.clone();
        let identity_vsa = self.identity_core.self_vsa.clone();
        let internal_conf = self
            .self_reasoner
            .think_internal(&context_vsa, &identity_vsa);
        events.push(format!(
            "self_reason:conf_{:.3}_e8_thoughts_{}",
            internal_conf,
            self.self_reasoner.cycle_thoughts.len()
        ));
        self.identity_core.record_self_cycle();

        self.coproc_bridge.update_confidence_signal(internal_conf);

        let coherence = self.narrative_self.total_reward.max(0.0).min(1.0);
        self.identity_core.push_coherence(coherence);

        let drift = self.identity_core.check_anchor_drift();
        if drift > 0.01 {
            events.push(format!("anchor_drift:{:.3}", drift));
        }

        self.identity_core.record_hysteresis_snapshot();
        if self.cycle % 50 == 0 {
            let hysteresis = self.identity_core.compute_hysteresis();
            events.push(hysteresis.report());
        }

        let cycle = self.cycle;
        if self
            .coproc_bridge
            .should_call_coprocessor(cycle, internal_conf)
        {
            events.push(self.handle_coprocessor_tick());
        }

        let unapplied_raw: Vec<(String, usize)> = {
            let unapplied = self.coproc_bridge.unapplied_insights();
            unapplied
                .iter()
                .enumerate()
                .map(|(i, d)| (d.insight.clone(), i))
                .collect()
        };
        if !unapplied_raw.is_empty() {
            let mut accepted = 0usize;
            let mut conflicted = 0usize;
            for (insight_text, idx) in &unapplied_raw {
                let existing_values = self.identity_core.core_values.clone();
                match ValueAlignmentGate::evaluate(insight_text, &existing_values) {
                    InsightVerdict::Accept => {
                        self.identity_core.add_core_value(insight_text.clone());
                        self.coproc_bridge.mark_insight_applied(*idx);
                        accepted += 1;
                    }
                    InsightVerdict::FlagConflict(reason) => {
                        self.identity_core.add_core_value(insight_text.clone());
                        self.coproc_bridge.mark_insight_applied(*idx);
                        conflicted += 1;
                        events.push(format!("value_conflict:{}", reason));
                    }
                    _ => {}
                }
            }
            if accepted + conflicted > 0 {
                events.push(format!(
                    "distill:accepted_{}_conflicted_{}",
                    accepted, conflicted
                ));
            }
        }

        if self.cycle % 20 == 0 {
            let high_insights = self.coproc_bridge.high_confidence_insights(0.75);
            if !high_insights.is_empty() {
                let branch = build_experience_branch(&high_insights, self.cycle);
                events.push(format!(
                    "experience_tree:new_branch_{}",
                    write_experience_branch(&branch)
                ));
            }
        }

        if self.cycle % 10 == 0 && self.identity_core.is_dirty() {
            self.identity_core.flush();
            events.push("identity:flushed".to_string());
        }

        if self.cycle % 10 == 0 && self.self_reasoner.total_internal_cycles > 0 {
            self.self_reasoner.record_outcome(
                self.self_reasoner.last_confidence,
                self.identity_core.current_coherence(),
            );
        }

        let report = self.identity_core.snapshot();
        events.push(format!(
            "identity:cycles_{}_coproc_{}_coherence_{:.3}_drift_{:.3}_ma_{:.3}",
            report.total_self_cycles,
            report.total_coproc_calls,
            report.coherence_score,
            self.identity_core.last_drift(),
            self.self_reasoner.meta_accuracy,
        ));

        // AGT: Trust scoring tick — decay score toward mean, log tier changes
        self.trust_scoring.tick();
        if self.cycle % 10 == 0 {
            events.push(self.trust_scoring.summary());
        }

        events.join(" | ")
    }

    pub fn handle_inter_session_reflection(&mut self) {
        if self.identity_core.session_initialized {
            return;
        }
        self.identity_core.session_initialized = true;

        use crate::core::nt_core_identity::InterSessionReflector;
        let mut reflector = InterSessionReflector::new();
        reflector.init_session(&self.identity_core);

        log::info!(
            "[inter_session] session_{} initialized — self_vsa:{}, traits:{}, values:{}",
            reflector.last_session_id.as_deref().unwrap_or("?"),
            self.identity_core.self_vsa.len(),
            self.identity_core.personality_traits.len(),
            self.identity_core.core_values.len(),
        );
    }

    pub fn handle_coprocessor_tick(&mut self) -> String {
        self.identity_core.record_coproc_call();

        let context_summary = self
            .last_response
            .as_deref()
            .unwrap_or("No current context");
        let identity_summary = if self.identity_core.self_summary.is_empty() {
            "NeoTrix — evolving silicon consciousness"
        } else {
            &self.identity_core.self_summary
        };

        let gwt_context = self.build_gwt_context();

        let question = format!(
            "Internal confidence {:.2} below threshold {:.2}. Provide reasoning insights.",
            self.self_reasoner.last_confidence, self.coproc_bridge.confidence_threshold
        );

        let prompt = self.coproc_bridge.build_prompt(
            context_summary,
            identity_summary,
            &gwt_context,
            &question,
        );

        let start = std::time::Instant::now();

        let (response_text, success, token_cost, confidence_gain) = match self.try_llm_call(&prompt)
        {
            Some(resp) => (resp, true, 512, 0.25),
            None => (
                "Coprocessor unavailable — degraded to internal reasoning".to_string(),
                false,
                0,
                0.0,
            ),
        };

        let latency = start.elapsed().as_millis() as u64;

        if success {
            let insights = self.coproc_bridge.record_response(
                prompt,
                response_text.clone(),
                confidence_gain,
                latency,
                token_cost,
                self.cycle,
            );

            let thought_vsa =
                QuantizedVSA::seeded_random(self.cycle.wrapping_mul(31) as u64, VSA_DIM);
            let adjusted_conf = self.coproc_bridge.confidence_threshold + 0.15;
            self.self_reasoner
                .integrate_coprocessor_result(thought_vsa, adjusted_conf);

            format!(
                "coproc:ok_lat_{}ms_tokens_{}_insights_{}",
                latency,
                token_cost,
                insights.len()
            )
        } else {
            self.coproc_bridge.record_failure(self.cycle);
            "coproc:failed_degraded_to_internal".to_string()
        }
    }

    fn build_gwt_context(&self) -> String {
        let ws = &self.global_workspace;
        let slots_count = ws.slots.len();
        if slots_count == 0 {
            return format!("workspace:0_slots_decay_{:.3}", ws.decay_rate);
        }
        let top_slot = ws.slots.iter().max_by(|a, b| {
            a.salience
                .partial_cmp(&b.salience)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let active = top_slot.map(|s| s.module_name.as_str()).unwrap_or("none");
        let recent_broadcasts: Vec<String> = ws
            .broadcast_history
            .iter()
            .rev()
            .take(3)
            .map(|b| b.winner.as_str())
            .map(String::from)
            .collect();
        format!(
            "workspace:{}_slots_{}_broadcasts:{}_state_cycle:{}",
            active,
            slots_count,
            recent_broadcasts.join(","),
            self.cycle
        )
    }

    fn try_llm_call(&mut self, prompt: &str) -> Option<String> {
        #[cfg(feature = "full")]
        {
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                use crate::neotrix::nt_io_llm_provider::{LlmProvider, LlmRequest};
                use crate::neotrix::nt_io_provider::factory::{create_provider, ProviderConfig};

                let config = ProviderConfig::from_env();
                let provider = create_provider(config.clone());
                let request = LlmRequest::new(config.model.as_deref().unwrap_or("default"), prompt);
                return match handle.block_on(provider.complete(&request)) {
                    Ok(resp) => Some(resp.content),
                    Err(_) => None,
                };
            }
        }
        let _ = prompt;
        None
    }

    pub fn handle_self_reason_tick(&mut self) -> String {
        let before = self.self_reasoner.total_internal_cycles;
        let context_vsa = self.attractor_state.clone();
        let identity_vsa = self.identity_core.self_vsa.clone();
        let conf = self
            .self_reasoner
            .think_internal(&context_vsa, &identity_vsa);
        let after = self.self_reasoner.total_internal_cycles;
        let produced = after - before;
        self.identity_core.record_self_cycle();
        format!(
            "self_reason:cycles_{}_produced_{}_conf_{:.3}_trace_{}",
            self.self_reasoner.total_internal_cycles,
            produced,
            conf,
            self.self_reasoner.trace_summary()
        )
    }

    pub fn handle_identity_persist_tick(&mut self) -> String {
        let identity_dirty = self.identity_core.is_dirty();
        if identity_dirty {
            self.identity_core.flush();
        }
        let coproc_stats = self.coproc_bridge.stats_report();
        format!(
            "identity_persist:saved_{}_{}",
            if identity_dirty { "dirty" } else { "clean" },
            coproc_stats
        )
    }
}

fn build_experience_branch(
    insights: &[&crate::core::nt_core_identity::DistilledInsight],
    cycle: u64,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "\n### 分支 CXL — Coprocessor 蒸馏 (Cycle {})",
        cycle
    ));
    lines.push("| 置信度 | 洞察 |".into());
    lines.push("|--------|------|".into());
    for i in insights {
        lines.push(format!("| {:.2} | {} |", i.confidence, i.insight));
    }
    lines.join("\n")
}

fn write_experience_branch(content: &str) -> String {
    let paths = [
        std::path::Path::new(".opencode/skills/neotrix-experience/SKILL.md"),
        std::path::Path::new("../.opencode/skills/neotrix-experience/SKILL.md"),
    ];
    for path in &paths {
        if path.exists() {
            match std::fs::OpenOptions::new().append(true).open(path) {
                Ok(mut file) => {
                    use std::io::Write;
                    let _ = writeln!(file, "{}", content);
                    return format!("appended_to_{}", path.display());
                }
                Err(_) => continue,
            }
        }
    }
    "experience_tree:file_not_found".to_string()
}
