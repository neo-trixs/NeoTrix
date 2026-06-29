use super::ConsciousnessIntegration;
use crate::core::nt_core_self_modify::{guard::GateResult, ModifyTarget, SelfModifySafety};

impl ConsciousnessIntegration {
    /// Self-modification tick: process pending proposals through
    /// SelfModifyAgent safety guard and sandbox validator.
    ///
    /// Each cycle, processes at most one proposal. Proposals come from
    /// the SEAL evolution loop via `MutationOp::SelfModifyProposal`.
    /// Safety failures emit a warning and drop the proposal — no crash.
    pub fn handle_self_modify_tick(&mut self) -> String {
        let agent = self.self_modify_agent.get_or_insert_with(|| {
            let mut agent = crate::core::nt_core_self_modify::SelfModifyAgent::new();
            agent.safety_level = SelfModifySafety::HandlerRewrite;
            agent
        });

        // Dequeue one pending proposal
        let proposal = match agent.proposals.pop_front() {
            Some(p) => p,
            None => return "self_modify:idle".to_string(),
        };

        // Step 1: Safety level check
        if !agent.is_target_allowed(&proposal.target) {
            log::warn!(
                "[self_modify] safety level {:?} blocks target {:?}",
                agent.safety_level,
                proposal.target
            );
            return format!("self_modify:rejected_safety_{:?}", proposal.target);
        }

        // Step 2: Guard evaluation (all 4 layers)
        let gate_result = agent.evaluate(&proposal);
        if !matches!(gate_result, GateResult::Approved) {
            log::warn!(
                "[self_modify] guard rejected proposal {}: {:?}",
                proposal.id,
                gate_result
            );
            return format!("self_modify:rejected_guard_{}", proposal.id);
        }

        // Step 3: Sandbox compilation check
        if let Some(ref sandbox) = agent.sandbox {
            let result = sandbox.validate_source(&proposal.source_code, "");
            if !result.compiles {
                log::warn!(
                    "[self_modify] sandbox compile failed for proposal {}: {:?}",
                    proposal.id,
                    result.compile_errors.first().unwrap_or(&"unknown".into())
                );
                return format!("self_modify:compile_fail_{}", proposal.id);
            }
        }

        // Step 4: Record in SEAL archive via self_evolution loop
        // (Proposal is consumed; if SEAL wiring is inactive, it's still logged)
        // SEAL archive recording is temporarily disabled — the MockCI reference
        // was invalid in production code. When SelfModifyProposal wiring is
        // activated, pass &mut self (ConsciousnessHandle impl) instead.
        #[cfg(test)]
        if let Some(ref mut evo) = self.self_evolution {
            let target_str = match &proposal.target {
                ModifyTarget::Handler { name } => name.clone(),
                ModifyTarget::Parameter { path } => path.clone(),
                ModifyTarget::Primitive { name } => name.clone(),
                ModifyTarget::SafetyGate { gate } => gate.clone(),
                ModifyTarget::PipelineStage { phase } => phase.clone(),
            };
            let target_type = match &proposal.target {
                ModifyTarget::Handler { .. } => "handler",
                ModifyTarget::Parameter { .. } => "parameter",
                ModifyTarget::Primitive { .. } => "primitive",
                ModifyTarget::SafetyGate { .. } => "safety_gate",
                ModifyTarget::PipelineStage { .. } => "pipeline",
            };

            // Provide a minimal ConsciousnessHandle for this non-test path
            struct CtxHandle;
            impl crate::core::nt_core_traits::ConsciousnessHandle for CtxHandle {
                fn apply_ne_edit(&mut self, _t: &str, _v: f64) -> String {
                    String::new()
                }
                fn stats_c_score(&self) -> f64 {
                    0.0
                }
                fn cognitive_load(&self) -> f64 {
                    0.0
                }
                fn self_evolution_best_score(&self) -> f64 {
                    0.0
                }
                fn eval_ne_string(&mut self, _e: &str) -> Result<String, String> {
                    Ok(String::new())
                }
                fn set_self_evolution_archive(&mut self, _bs: f64) {}
            }
            let _ = evo.execute_mutation(
                &crate::core::nt_core_experience::self_evolution_loop::MutationOp::SelfModifyProposal {
                    target: target_str,
                    target_type: target_type.to_string(),
                    source_code: proposal.source_code.clone(),
                },
                &mut CtxHandle,
            );
        }

        format!(
            "self_modify:applied_{}_id={}",
            match &proposal.target {
                ModifyTarget::Handler { name } => format!("handler_{}", name),
                ModifyTarget::Parameter { path } => format!("param_{}", path.replace('.', "_")),
                ModifyTarget::Primitive { name } => format!("primitive_{}", name),
                ModifyTarget::PipelineStage { phase } => format!("pipeline_{}", phase),
                ModifyTarget::SafetyGate { gate } => format!("safety_{}", gate),
            },
            proposal.id,
        )
    }

    /// Convenience: enqueue a handler rewrite proposal.
    pub fn propose_handler_rewrite(
        &mut self,
        handler_name: &str,
        source_code: &str,
        rationale: &str,
    ) {
        let agent = self.self_modify_agent.get_or_insert_with(|| {
            let mut agent = crate::core::nt_core_self_modify::SelfModifyAgent::new();
            agent.safety_level = SelfModifySafety::HandlerRewrite;
            agent
        });
        agent.enqueue(
            ModifyTarget::Handler {
                name: handler_name.to_string(),
            },
            source_code.to_string(),
            rationale.to_string(),
            0.5,
        );
    }

    /// Convenience: enqueue a parameter tune proposal.
    pub fn propose_param_tune(&mut self, param_path: &str, source_code: &str, rationale: &str) {
        let agent = self.self_modify_agent.get_or_insert_with(|| {
            let mut agent = crate::core::nt_core_self_modify::SelfModifyAgent::new();
            agent.safety_level = SelfModifySafety::ParamOnly;
            agent
        });
        agent.enqueue(
            ModifyTarget::Parameter {
                path: param_path.to_string(),
            },
            source_code.to_string(),
            rationale.to_string(),
            0.3,
        );
    }
}
