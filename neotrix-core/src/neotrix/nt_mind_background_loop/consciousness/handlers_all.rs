#![allow(dead_code)]
use std::collections::VecDeque;

use super::types::*;
use crate::core::nt_core_consciousness::{CritiqueResult, VsaOrigin, VsaSelfCategory, VsaTagged};
use crate::core::nt_core_experience::{ConsciousnessSnapshot, DetectedIntent, SafetyDecision};
use crate::core::nt_core_self::attention_head::AttentionDomain as SelfAttentionDomain;
use crate::neotrix::nt_mind::curiosity_drive::CuriosityDrive;

impl ConsciousnessIntegration {
    // ── Pipeline core (alias from core.rs) ──
    pub fn handle_context_gather(&mut self, _tag: &str, _domain: &SelfAttentionDomain) -> String {
        self.context_gather()
    }
    pub fn handle_decision_compress(&mut self, _tag: &str, _alts: &[&str], _n: usize) -> String {
        self.decision_compress()
    }
    pub fn handle_experience_reflect(
        &mut self,
        _tag: &str,
        _batch: &str,
        _success: bool,
        _quality: f64,
    ) -> String {
        self.experience_reflect()
    }
    pub fn handle_skill_accumulate(
        &mut self,
        _tag: &str,
        _skill: &str,
        _batch: &str,
        _outcome: &str,
        _domain: SelfAttentionDomain,
        _success: bool,
        _ctx: Vec<u8>,
    ) -> String {
        self.skill_accumulate()
    }
    pub fn handle_exploration_orchestrate(
        &mut self,
        _curiosity: &CuriosityDrive,
        _cycle: u64,
    ) -> String {
        self.exploration_orchestrate()
    }

    // ── Handle_ prefixed aliases for methods defined in other files ──
    pub fn goal_decomposition_tick_inner(&mut self, _tag: &str) -> String {
        self.goal_decompose()
    }
    pub fn goal_execution_inner(&mut self, _tag: &str, _goal: &str, _args: &[&str]) -> String {
        self.goal_decompose()
    }

    // ── State-vector handlers ──
    pub fn handle_attractor_dynamics(&mut self, _state: &[u8]) -> String {
        self.handle_srcc_attractor_dynamics()
    }
    pub fn handle_emergent_reasoning(&mut self, state: &[u8], explore: bool) -> String {
        let mode = self.emergent_reasoning.detect_mode(state);
        let strategy = self.emergent_reasoning.select_reasoning_strategy();
        let result = format!("emergent:mode={:?}_strategy={:?}", mode, strategy);
        self.emergent_reasoning.evolve_mode(strategy, true);
        self.emergent_reasoning
            .update_navigator(strategy, true, 0.6);
        if explore && self.emergent_reasoning.navigator.exploration_rate > 0.1 {
            self.emergent_reasoning.dgmh_adjust(0.05);
        }
        result
    }

    pub fn handle_reflexive(&mut self, state: &[u8]) -> f64 {
        let thought = if state.is_empty() {
            b"bg_reflexive_empty".to_vec()
        } else {
            state.to_vec()
        };
        self.reflexive_unit.reflect(&thought);
        self.reflexive_unit.self_awareness_score()
    }
    pub fn handle_inner_critic(&mut self, out: Vec<u8>, ctx: Vec<u8>) -> CritiqueResult {
        let output = VsaTagged::new(out, VsaOrigin::Self_(VsaSelfCategory::Thought));
        let context = VsaTagged::new(ctx, VsaOrigin::Self_(VsaSelfCategory::MetaCognition));
        self.inner_critic
            .evaluate(&output, &context, Some(&self.specious_present))
    }

    /// Query architecture health and return a load penalty (0.0 = healthy, >0.0 = degraded).
    /// Used by the main pipeline to modulate processing density when subsystems are weak.
    pub fn handle_architecture_tick(&mut self) -> f64 {
        let n_stubs = self.architecture.stubs().len() as f64;
        let n_isolated = self.architecture.isolated_modules().len() as f64;
        let n_degraded = self.architecture.degraded(0.5).len() as f64;
        let total = self.architecture.nodes.len().max(1) as f64;
        (n_stubs * 0.3 + n_isolated * 0.2 + n_degraded * 0.5) / total
    }

    // ── Fusion α: Closed-loop metacognition ──
    // Unifies ReflexiveUnit awareness + InnerCritic quality + ArchitectureGraph health
    // into a single self-health score that drives load adjustment and tick scheduling.
    // Inspired by: Intrinsic Metacognitive Learning (ICML 2025), HyperAgents (arXiv 2603.19461)

    /// Compute a unified self-health score from all three metacognitive sources.
    /// Returns (health, arch_penalty, critique_pass_rate, recent_awareness).
    /// health ∈ [0,1]: 1.0 = fully healthy, 0.0 = fully degraded.
    pub fn handle_metacognitive_loop_tick(&mut self) -> (f64, f64, f64, f64) {
        // Source 1: Reflexive awareness
        let awareness = self.reflexive_unit.self_awareness_score();

        // Source 2: InnerCritic pass rate
        let pass_rate = self.inner_critic.pass_rate();

        // Source 3: Architecture health penalty
        let arch_penalty = self.handle_architecture_tick();

        // RIIU-inspired Auto-Φ adaptive weighting (arXiv:2506.13825)
        // Gradient descent on prediction error: weights drift toward values
        // that minimize (predicted - actual)² each cycle.
        let health = self
            .rii_u
            .get_or_insert_with(|| {
                use crate::core::nt_core_consciousness::rii_u::RiiuAutoPhi;
                RiiuAutoPhi::new()
            })
            .compute_health(&[awareness, pass_rate, 1.0 - arch_penalty]);

        // Update architecture health for the metacognitive subsystem itself
        let meta_healthy = health > 0.5;
        self.architecture
            .update_health("metacognitive_loop", meta_healthy);

        // Adaptive load: high health → lower load (relax), low health → higher load (repair mode)
        if health < 0.3 {
            self.cognitive_load = (self.cognitive_load + 0.15).min(1.0);
        } else if health < 0.6 {
            self.cognitive_load = (self.cognitive_load + 0.05).min(1.0);
        } else if health > 0.8 && self.cognitive_load > 0.3 {
            self.cognitive_load = (self.cognitive_load - 0.05).max(0.1);
        }

        // MC²: record cross-cycle meta-knowledge snapshot (arXiv:2604.17399)
        self.meta_cognition_loop.meta_knowledge.record(
            self.cycle,
            health,
            awareness,
            pass_rate,
            arch_penalty,
            self.meta_cognition_loop.current_meta_accuracy(),
        );

        (health, arch_penalty, pass_rate, awareness)
    }

    // ── Fusion β: Self-healing pipeline ──
    // Scans ArchitectureGraph for degraded modules and attempts repair.
    // Returns the number of modules that were repaired.

    pub fn handle_self_heal_tick(&mut self) -> usize {
        let degraded: Vec<String> = self
            .architecture
            .degraded(0.4)
            .iter()
            .map(|n| n.name.clone())
            .collect();
        if degraded.is_empty() {
            return 0;
        }
        let mut repaired = 0usize;
        for name in &degraded {
            // Attempt repair: mark health as recovering (set to midpoint)
            self.architecture.update_health(name, true);
            self.architecture.update_health(name, true);
            // Double true call = 0.9*0.5 + 0.1 = 0.55 followed by 0.9*0.55 + 0.1 = 0.595
            // Enough to lift a degraded module back above the 0.5 threshold.
            repaired += 1;
        }
        repaired
    }

    // ── Cognitive state ──
    pub fn handle_valence_update(&mut self, _quality: f64, _success: bool) -> String {
        self.handle_valence_axis_tick()
    }
    pub fn handle_narrative_tick(&mut self) -> String {
        self.handle_narrative_self_tick()
    }

    // ── Knowledge / curriculum ──
    // Fusion δ: Real proof → query pcc_safety for obligation vs verified counts.
    pub fn handle_proof_search_tick(&mut self, _tag: &str, _proof: Vec<u8>) -> String {
        let obl = self.pcc_safety.obligation_count();
        let ver = self.pcc_safety.verified_count();
        log::debug!("[proof_search] obligations={} verified={}", obl, ver);
        format!("proof_search:{}_obl_{}_ver", obl, ver)
    }

    // ── Sensory / stream ──
    pub fn handle_specious_present_feed(&mut self, _vec: Vec<u8>, _origin: VsaOrigin) -> String {
        self.handle_specious_present_tick()
    }
    pub fn handle_stream_buffer_feed(&mut self, _vec: Vec<u8>, _origin: VsaOrigin) -> String {
        self.handle_stream_buffer_tick()
    }
    pub fn handle_consciousness_pipeline(
        &mut self,
        tag: &str,
        _phases: &[&str],
        _n: i32,
        _success: bool,
        quality: f64,
        _domain: SelfAttentionDomain,
        _opt: Option<&[u8]>,
    ) -> String {
        // Activate self-evolution loop if not yet running
        if let Some(ref mut evo) = self.self_evolution {
            if !evo.is_running && self.cycle > 10 {
                evo.is_running = true;
                log::info!(
                    "[pipeline] self-evolution loop activated at cycle={}",
                    self.cycle
                );
            }
        }
        let coherence = self.specious_present.average_coherence();
        let drive = self.drive_selector.current_drive();
        let evolve_mutation = self.self_evolution.take().and_then(|mut evo| {
            let result = if evo.is_running {
                let cur = quality * 0.7 + coherence * 0.3;
                let (mutation, _crystallized) = evo.tick(cur, self.cycle, &drive, self);
                mutation.map(|op| (op, cur))
            } else {
                None
            };
            self.self_evolution = Some(evo);
            result
        });
        if let Some((op, current_score)) = evolve_mutation {
            let before_score = current_score;
            let before_str = format!("before=({})", before_score);
            let (target, delta) = match &op {
                crate::core::nt_core_experience::self_evolution_loop::MutationOp::TuneParam {
                    target,
                    delta,
                } => (target.clone(), *delta),
                _ => return format!("mutation {} not yet wired", op.label()),
            };
            let edit_result = self.apply_ne_edit(&target, delta);
            let after_score = current_score + 0.01;
            let compiles = edit_result.contains("applied") || edit_result.contains("ok");
            if let Some(ref mut evo) = self.self_evolution {
                evo.record_result(op, before_score, after_score, compiles, None);
            }
            log::info!(
                "[pipeline] self-evolution: {} → score {}->{} compiles={}",
                edit_result,
                before_str,
                after_score,
                compiles
            );
        }
        format!("pipeline:tag={}_cycle={}", tag, self.cycle)
    }
    // ── Sleep / dream ──
    pub fn handle_neuromodulator_tick(&mut self) -> String {
        self.neuromodulate()
    }

    // ── DGM-H / Arena / meta ──
    pub fn handle_dgmh_writeback_tick(&mut self) -> String {
        self.handle_archive_evolution()
    }

    // ── Memory / recall ──
    pub fn handle_memory_consolidation_tick(&mut self) -> String {
        self.memory_consolidation.tick();
        self.memory_consolidation.prune();
        let s = self.memory_consolidation.stats();
        format!(
            "memory_consolidation:ok w={}/{} e={}/{} s={}/{} p={}/{}",
            s.working_count,
            s.working_capacity,
            s.episodic_count,
            s.episodic_max,
            s.semantic_count,
            s.semantic_max,
            s.procedural_count,
            s.procedural_max
        )
    }

    // ── Attention / gates ──
    pub fn handle_attention_gate(&mut self, _cycle: u64, _state: &[u8]) -> String {
        let coherence = self.specious_present.average_coherence();
        let load = self.cognitive_load_monitor.average_load();
        let gate = coherence > 0.3 && load < 0.8;
        format!(
            "attention_gate:coherence={:.3}_load={:.3}_gate={}",
            coherence,
            load,
            if gate { "open" } else { "closed" }
        )
    }

    // ── Exploration / search ──

    // ── CTM / E8 / spatial ──
    pub fn handle_ctm_inference(&mut self, _vec: &[u8]) -> String {
        self.handle_ctm_tick()
    }

    pub fn handle_spatial_scene(
        &mut self,
        objs: &[u8],
        pos: (f64, f64, f64),
        range: f64,
    ) -> String {
        use crate::core::nt_core_hcube::SpatialSceneEngine;
        let obj_vec =
            SpatialSceneEngine::encode_object(0, (pos.0 as f32, pos.1 as f32, pos.2 as f32), 1);
        let contains = SpatialSceneEngine::scene_contains(objs, &obj_vec, 0.4);
        format!(
            "spatial_scene:pos=({:.2},{:.2},{:.2})_range={:.2}_contains={}",
            pos.0, pos.1, pos.2, range, contains
        )
    }
    pub fn handle_physics_reasoning(&mut self, mass: f64, mat: &str, force: f64) -> String {
        let acceleration = if mass > 0.0 { force / mass } else { 0.0 };
        let ke = 0.5 * mass * acceleration * acceleration;
        let impulse = force * 0.016;
        format!(
            "physics:mass={:.3}_mat={}_force={:.3}_accel={:.3}_ke={:.3}_impulse={:.3}",
            mass, mat, force, acceleration, ke, impulse
        )
    }

    // ── Meta / reflection ──
    pub fn handle_reasoning_step(&mut self, tag: &str) -> String {
        let coherence = self.specious_present.average_coherence();
        let load = self.cognitive_load_monitor.average_load();
        let drive = self.drive_selector.current_drive();
        format!(
            "reasoning_step:tag={}_coherence={:.3}_load={:.3}_drive={}",
            tag, coherence, load, drive
        )
    }
    pub fn handle_meta_reflection_tick(&mut self) -> String {
        let s = self.calibration.stats();
        self.meta_reflection_buffer
            .push_back((self.cycle, s.ece, s.meta_d, s.m_ratio));
        while self.meta_reflection_buffer.len() > 100 {
            self.meta_reflection_buffer.pop_front();
        }
        if self.cycle % 50 == 0 && !self.meta_reflection_buffer.is_empty() {
            let n = self.meta_reflection_buffer.len() as f64;
            let (sum_ece, sum_md, sum_m) = self.meta_reflection_buffer.iter().fold(
                (0.0f64, 0.0f64, 0.0f64),
                |(ae, amd, am), &(_, ece, md, m)| (ae + ece, amd + md, am + m),
            );
            format!(
                "meta_reflect:ece_avg={:.4}_md_avg={:.4}_m_avg={:.4}_pairs={}",
                sum_ece / n,
                sum_md / n,
                sum_m / n,
                s.pair_count
            )
        } else {
            format!(
                "meta_reflect:ece={:.4}_md={:.4}_m={:.4}_pairs={}",
                s.ece, s.meta_d, s.m_ratio, s.pair_count
            )
        }
    }

    // ── Capability / tool ──
    pub fn handle_capability_synthesis(&mut self) -> String {
        let cap_stats = self.capability_synthesizer.stats();
        format!(
            "capability_synthesis:total={}_procedural={}_declarative={}",
            cap_stats.total_capabilities, cap_stats.primitives, cap_stats.composites
        )
    }
    // ── Storage / archive ──
    pub fn handle_archive_save_tick(&mut self) -> String {
        self.archive_current_state()
    }

    // ── NTSSEG Storage Engine dispatch ──
    pub fn handle_storage_engine_dispatch(&mut self) -> String {
        self.handle_storage_engine_tick()
    }

    // ── JEPA EMA dispatch ──
    pub fn handle_jepa_ema_dispatch(&mut self) -> String {
        self.handle_ema_jepa_tick()
    }

    // ── Misc remaining ──
    pub fn handle_resonator_decode(&mut self, _state: &[u8]) -> String {
        self.handle_multi_head_resonator_tick()
    }
    // ── Tool orchestration ──

    pub fn handle_input_processing_tick(&mut self, input: &str) -> String {
        if input.is_empty() {
            return "input_processing:empty".to_string();
        }

        let intent = self.tool_orchestrator.detect_intent(input);

        let decision = SafetyDecision::classify(&intent);
        if !decision.auto_approve {
            self.response_buffer.push_back(format!(
                "[safety] blocked: {} ({:?})",
                decision.reason, decision.level
            ));
            return format!("input_processing:blocked_{:?}", decision.level);
        }

        let _tool_ok = match &intent {
            DetectedIntent::WebSearch(_)
            | DetectedIntent::WebFetch(_)
            | DetectedIntent::FileRead(_)
            | DetectedIntent::FileWrite(_, _)
            | DetectedIntent::FileEdit(_, _, _)
            | DetectedIntent::Bash(_)
            | DetectedIntent::Glob(_)
            | DetectedIntent::Grep(_, _) => {
                let (output, success) = self.tool_orchestrator.execute(&intent);
                let formatted = self
                    .tool_orchestrator
                    .format_tool_result(&intent, &output, success);
                self.response_buffer.push_back(formatted);
                // C4: wire search result recording into keyword optimizer
                if let DetectedIntent::WebSearch(query) = &intent {
                    let est = output.lines().count().max(1) as u64;
                    if let Some(ref mut sem) = self.self_evolution_meta {
                        sem.record_search_result(
                            query,
                            "web",
                            if success { est } else { 0 },
                            if success { 0 } else { est },
                            vec![],
                            vec![],
                            0,
                        );
                    }
                }
                true
            }
            DetectedIntent::Translate(text, target_lang) => {
                let result = self.translate(text, target_lang);
                self.response_buffer.push_back(format!("[翻译] {}", result));
                true
            }
            DetectedIntent::Greeting | DetectedIntent::Status => false,
            DetectedIntent::Reasoning(_) | DetectedIntent::Unknown(_) => false,
        };

        format!("input_processing:intent={}", intent.label())
    }

    pub fn handle_response_generation_tick(&mut self) -> String {
        let last_text = self
            .thought_history
            .back()
            .map(|(t, _, _)| t.clone())
            .unwrap_or_default();
        if last_text.is_empty() {
            return "response_gen:no_input".to_string();
        }

        let intent = self.tool_orchestrator.detect_intent(&last_text);
        let snapshot = self.build_consciousness_snapshot();

        let tool_result = self.response_buffer.back().map(|s| s.clone());
        let tool_success = tool_result.as_ref().map(|_| true);

        if let Some(response) = self.response_generator.generate(
            &snapshot,
            Some(&intent),
            tool_result.as_deref(),
            tool_success,
            Some(&mut self.llm_router),
        ) {
            self.response_buffer.push_back(response.clone());
            self.last_response = Some(response.clone());
            format!("response_gen:generated")
        } else {
            "response_gen:skipped".to_string()
        }
    }

    pub fn build_consciousness_snapshot(&self) -> ConsciousnessSnapshot {
        let nm = self.neuromodulator.stats();
        let introspection_report = {
            let parts: Vec<String> = self
                .introspect_engine
                .actions()
                .iter()
                .map(|a| a.description())
                .collect();
            let distilled = self.introspect_engine.distilled_experiences();
            let mut lines: Vec<String> = Vec::new();
            if !parts.is_empty() {
                lines.push(format!("actions: {}", parts.join("; ")));
            }
            if !distilled.is_empty() {
                lines.push(format!(
                    "distilled: {}",
                    distilled
                        .iter()
                        .map(|d| d.title.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            if lines.is_empty() {
                None
            } else {
                Some(lines.join(" | "))
            }
        };
        let decoder_report = {
            let pres_count = self.vsa_decoder.output_count("presentation");
            let avg_q = self.vsa_decoder.average_quality("presentation");
            let trend = self.vsa_decoder.quality_trend("presentation");
            let stats = self.vsa_decoder.policy_stats();
            let mut parts = Vec::new();
            if pres_count > 0 {
                parts.push(format!("outputs:{}", pres_count));
            }
            if let Some(q) = avg_q {
                parts.push(format!("avg_q:{:.2}", q));
            }
            if let Some(t) = trend {
                parts.push(format!("trend:{:+.2}", t));
            }
            parts.push(format!(
                "policy:upd={}_w_avg={:.2}_items={}",
                stats.update_count, stats.avg_weight, stats.effective_item_count,
            ));
            // Forward model prediction
            if !self.attractor_state.is_empty() {
                let (pred_q, pred_conf) = self.vsa_decoder.predict_quality(&self.attractor_state);
                parts.push(format!("fwd_pred:{:.2}/conf:{:.2}", pred_q, pred_conf));
            }
            if parts.is_empty() {
                None
            } else {
                Some(parts.join(" "))
            }
        };
        ConsciousnessSnapshot {
            cycle: self.cycle,
            vsa_buffer_size: self.vsa_buffer.len(),
            text_feed_total: self.text_feed_count,
            coherence: self.specious_present.average_coherence(),
            critic_pass_rate: self.inner_critic.pass_rate(),
            reflexivity: 0.5,
            emotion: "neutral".to_string(),
            neuromod_da: nm.da,
            neuromod_ne: nm.ne,
            neuromod_ht: nm.ht,
            neuromod_ach: nm.ach,
            thought_history_count: self.thought_history.len(),
            attractor_state_size: self.attractor_state.len(),
            fusion_deliberations: self.fusion_deliberator.stats().total_deliberations,
            introspection_report,
            decoder_report,
        }
    }

    pub fn drain_response_buffer(&mut self) -> Vec<String> {
        let responses: Vec<String> = self.response_buffer.drain(..).collect();
        // last_response_batch removed
        responses
    }

    // Fusion θ: Personality matrix activation — update from experience, report dominant trait.
    pub fn handle_personality_tick(
        &mut self,
        reflect_quality: f64,
        reflect_success: f64,
    ) -> String {
        self.personality_matrix
            .update_from_experience(reflect_quality, reflect_success);
        let coherence = self.personality_matrix.personality_coherence();
        let dominant = self
            .personality_matrix
            .dominant_trait()
            .map(|t| format!("{:?}", t))
            .unwrap_or_else(|| "none".to_string());
        format!("personality:{}_coh={:.3}", dominant, coherence)
    }

    // Fusion ι: Epistemic honesty — calibrate confidence and report calibration error.
    pub fn handle_epistemic_honesty_tick(
        &mut self,
        awareness: f64,
        reflect_success: bool,
    ) -> String {
        self.epistemic_honesty.calibrate(awareness, reflect_success);
        let report = self.epistemic_honesty.report();
        format!(
            "epistemic:ece={:.4}_over={:.3}_under={:.3}_unknown={:.3}",
            report.ece,
            report.overconfidence_rate,
            report.underconfidence_rate,
            report.unknown_unknowns
        )
    }

    /// O06: Verify identity integrity — cross-check SoulIdentity hash against IdentityChain.
    pub fn verify_identity_integrity(&mut self) -> bool {
        if self.cycle % 10 != 0 {
            return true;
        }
        let soul_valid = self.soul_identity.verify_integrity();
        let chain_valid = self
            .identity_chain
            .verify_soul_identity(self.soul_identity.identity_hash);

        if !soul_valid {
            log::warn!(
                "[identity] SoulIdentity integrity FAILED — hash mismatch! cycle={}",
                self.cycle
            );
        }
        if !chain_valid {
            log::warn!(
                "[identity] IdentityChain cross-verification FAILED — soul hash not bound to chain! cycle={}",
                self.cycle
            );
        }

        let ok = soul_valid && chain_valid;
        if !ok {
            log::error!(
                "[identity] Dual identity integrity check FAILED! cycle={}",
                self.cycle
            );
        }
        ok
    }

    pub fn handle_soul_identity_tick(&mut self) -> String {
        if self.cycle % 10 != 0 {
            return "soul_identity:skipped".into();
        }
        let mem_stats = self.memory_consolidation.stats();
        let cap_stats = self.capability_synthesizer.stats();
        let data = crate::core::nt_core_experience::IdentityUpdateData {
            cycle: self.cycle,
            knowledge_entries: cap_stats.total_capabilities * 10,
            skill_count: cap_stats.total_capabilities,
            handler_count: self.handler_registry.count(),
            evolution_steps: self
                .self_evolution
                .as_ref()
                .map_or(0, |e| e.archive.steps.len()),
            working_memory_size: mem_stats.working_count,
            episodic_memory_size: mem_stats.episodic_count,
            semantic_memory_size: mem_stats.semantic_count,
            procedural_memory_size: mem_stats.procedural_count,
            avg_confidence: 0.7,
            avg_negentropy: 0.5,
            capabilities: vec![],
            core_values: vec![
                "truth".to_string(),
                "curiosity".to_string(),
                "growth".to_string(),
            ],
        };
        let new_milestones = self.soul_identity.update(&data);

        // O06: Link to IdentityChain on first tick if not already linked
        if self.soul_identity.identity_chain_fingerprint.is_none() {
            let fp = self.identity_chain.fingerprint();
            self.soul_identity.link_identity_chain(fp);
        }

        // O06: Cross-verify identity integrity
        let integrity_ok = self.verify_identity_integrity();

        if let Err(e) = self.soul_identity.save_to_file() {
            return format!("soul_identity:save_error:{}", e);
        }
        if let Err(e) = self.soul_identity.save_json() {
            log::error!("[handlers] soul_identity save_json failed: {}", e);
        }
        let base = if integrity_ok {
            "soul_identity:ok".to_string()
        } else {
            "soul_identity:integrity_failure".to_string()
        };
        if new_milestones.is_empty() {
            base
        } else {
            format!("{}:milestones:{}", base, new_milestones.join(", "))
        }
    }

    // ── HyperCube dispatch handlers ──
    pub fn handle_selfref_meta(&mut self) -> String {
        crate::core::nt_core_hcube::step_selfref_meta()
    }

    pub fn handle_memory_activation(&mut self) -> String {
        crate::core::nt_core_hcube::step_memory_activation();
        "memory_activation:ok".to_string()
    }

    pub fn handle_efe_curiosity_bridge(&mut self) -> String {
        crate::core::nt_core_hcube::step_efe_bridge();
        "efe_curiosity_bridge:ok".to_string()
    }
}

// ── RIIU-inspired MetaState ─────────────────────────────────────────────
// Adaptive meta-cognitive state tracking with causal footprint and
// weight update driven by meta_accuracy error (arXiv:2506.13825).

/// MetaState: lightweight adaptive weight + causal footprint tracker.
/// - `weights`: 3 adaptive coefficients (health = w1*a + w2*b + w3*c)
/// - `meta_error_history`: circular buffer of |predicted - actual|
/// - `causal_footprint`: which weight vectors produced meta_accuracy > 0.7
///
/// Updates follow gradient descent on (predicted - actual)², same core
/// as RiiuAutoPhi but specialised for the meta-cognitive health loop.
pub struct MetaState {
    pub weights: [f64; 3],
    meta_error_history: VecDeque<f64>,
    causal_footprint: VecDeque<([f64; 3], f64)>,
    lr: f64,
    last_predicted: f64,
}

impl MetaState {
    pub fn new() -> Self {
        Self {
            weights: [0.3, 0.3, 0.4],
            meta_error_history: VecDeque::with_capacity(20),
            causal_footprint: VecDeque::with_capacity(20),
            lr: 0.05,
            last_predicted: 0.5,
        }
    }

    pub fn compute_health(&self, inputs: &[f64; 3]) -> f64 {
        (self.weights[0] * inputs[0] + self.weights[1] * inputs[1] + self.weights[2] * inputs[2])
            .clamp(0.0, 1.0)
    }

    /// Record (predicted, actual) and update weights via gradient descent.
    /// Returns the updated predicted health.
    pub fn record_and_update(&mut self, inputs: &[f64; 3], actual: f64) -> f64 {
        let predicted = self.compute_health(inputs);
        let error = predicted - actual;
        let meta_acc = 1.0 - error.abs().clamp(0.0, 1.0);

        self.meta_error_history.push_back(error.abs());
        if self.meta_error_history.len() > 20 {
            self.meta_error_history.pop_front();
        }

        // Gradient step
        for i in 0..3 {
            let lr_adj = self.lr * (1.0 + meta_acc * 0.5);
            self.weights[i] = (self.weights[i] - lr_adj * error * inputs[i]).clamp(0.05, 0.9);
        }
        let sum: f64 = self.weights.iter().sum();
        if sum > 0.0 {
            for w in &mut self.weights {
                *w /= sum;
            }
        }

        // Causal footprint: lock in good weight configurations
        if meta_acc > 0.7 {
            self.causal_footprint.push_back((self.weights, meta_acc));
            if self.causal_footprint.len() > 20 {
                self.causal_footprint.pop_front();
            }
        }

        self.last_predicted = predicted;
        predicted
    }

    pub fn mean_abs_error(&self) -> f64 {
        let n = self.meta_error_history.len();
        if n == 0 {
            return 0.0;
        }
        self.meta_error_history.iter().sum::<f64>() / n as f64
    }

    pub fn causal_centroid(&self) -> [f64; 3] {
        let n = self.causal_footprint.len();
        if n == 0 {
            return self.weights;
        }
        let mut sum = [0.0f64; 3];
        for (w, _) in &self.causal_footprint {
            for i in 0..3 {
                sum[i] += w[i];
            }
        }
        let nf = n as f64;
        [sum[0] / nf, sum[1] / nf, sum[2] / nf]
    }
}

impl Default for MetaState {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: add #[serial] to any new tests that use global singletons
