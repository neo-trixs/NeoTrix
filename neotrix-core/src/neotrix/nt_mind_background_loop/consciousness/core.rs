#![allow(dead_code)]
// SPLIT PLAN:
//   File: 1989 lines — 5 sections to extract:
//   1. `phases.rs`       — phase_one_input + phase_two_convergence + pipeline orchestration (lines 47–981)
//   2. `pipeline_run.rs`  — Pipeline methods called by run.rs (lines 982–1428)
//   3. `world_model.rs`   — Phase 36 World Model (lines 1429–1562)
//   4. `construction.rs`  — Construction + feed + process_user_request (lines 1563–1798)
//   5. `core_tests.rs`    — #[cfg(test)] module (lines 1563–1989)
//   How: extract largest blocks, keep profiling helpers (lines 1–46) as they're used by everything.

use super::types::*;
use crate::core::nt_core_consciousness::first_person_ref::ExperienceRecord;
use crate::core::nt_core_consciousness::global_workspace::GlobalLatentWorkspace;
use crate::core::nt_core_consciousness::vsa_tag::{
    VsaOrigin, VsaTagged, VsaWorldCategory,
};
use crate::core::nt_core_experience::consciousness_hooks::HookPoint;
use crate::core::nt_core_experience::ideal_state::{reverse_intent, EffortLevel};
use crate::core::nt_core_gwt::intrinsic_drive::IntrinsicDrive;
use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;
use crate::core::nt_core_knowledge::self_inspect::SelfInspectable;
use crate::core::nt_core_llm_router::ChatMessage;
use crate::core::nt_core_traits::ConsciousnessHandle;
use crate::core::nt_core_self::AttentionDomain;
use crate::neotrix::nt_world_jepa::predictor::EMAJepaPredictor;
use log;
use sha2::{Digest, Sha256};
use std::sync::{Mutex, OnceLock};

// ── Pipeline Cache: VSA prefix fingerprint caching ──
struct PipelineCache {
    prev_fingerprint: Option<[u8; 32]>,
    cache_hits: u64,
    valid: bool,
}

impl PipelineCache {
    fn new() -> Self {
        Self {
            prev_fingerprint: None,
            cache_hits: 0,
            valid: false,
        }
    }

    fn compute_fingerprint(first_person: &[u8], constraints: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(first_person);
        hasher.update(constraints);
        hasher.update(b"neotrix-vsa-prefix-v1");
        hasher.finalize().into()
    }

    fn check(&mut self, first_person: &[u8], constraints: &[u8]) -> bool {
        let fp = Self::compute_fingerprint(first_person, constraints);
        match self.prev_fingerprint {
            Some(prev) if prev == fp && self.valid => {
                self.cache_hits += 1;
                true
            }
            _ => {
                self.prev_fingerprint = Some(fp);
                self.valid = true;
                self.cache_hits = 0;
                false
            }
        }
    }

    fn invalidate(&mut self) {
        self.valid = false;
    }
}

static PIPELINE_CACHE: OnceLock<Mutex<PipelineCache>> = OnceLock::new();
fn pipeline_cache() -> &'static Mutex<PipelineCache> {
    PIPELINE_CACHE.get_or_init(|| Mutex::new(PipelineCache::new()))
}

impl ConsciousnessIntegration {
    // ── Profiling helpers ──
    // SECTION: Profiling helpers

    fn profile<F: FnOnce(&mut Self) -> String>(&mut self, name: &'static str, f: F) -> String {
        self.profiler
            .register_handler(name, crate::core::nt_core_experience::HandlerTier::Every);
        let start = self.profiler.record_start(name);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(self)));
        self.profiler.record_end(name, start);

        let output = match result {
            Ok(output) => output,
            Err(_) => {
                log::error!(
                    "[consciousness] handler '{}' panicked at cycle {}, continuing gracefully",
                    name,
                    self.cycle
                );
                format!("{}:PANIC", name)
            }
        };

        // Dead cycle detection: warn if 3+ consecutive identical outputs
        if self.cycle > 5 {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            output.hash(&mut hasher);
            let hash = hasher.finish();
            let (prev_hash, repeats) = self.cycle_output_cache.get(name).copied().unwrap_or((0, 0));
            if prev_hash == hash {
                let new_repeats = repeats + 1;
                if new_repeats >= 3 {
                    log::warn!(
                        "[consciousness] handler '{}' produced identical output for {} consecutive cycles at cycle {}",
                        name, new_repeats, self.cycle
                    );
                }
                self.cycle_output_cache.insert(name, (hash, new_repeats));
            } else {
                self.cycle_output_cache.insert(name, (hash, 0));
            }
        }

        output
    }

    // ── Pipeline cache optimization methods ──

    /// Clean orphan VSA vectors, repair corrupted VsaTag, deduplicate entries.
    pub fn stream_buffer_hygiene(&mut self) -> String {
        let mut cleaned = 0u64;
        // Dedup thought_history entries with identical text content
        if self.thought_history.len() > 1 {
            let mut seen: Vec<String> = Vec::with_capacity(self.thought_history.len());
            let old = std::mem::take(&mut self.thought_history);
            for entry in old.into_iter() {
                if seen.contains(&entry.0) {
                    cleaned += 1;
                } else {
                    seen.push(entry.0.clone());
                    self.thought_history.push_back(entry);
                }
            }
        }
        // Invalidate pipeline cache after hygiene
        if let Ok(mut cache) = pipeline_cache().lock() {
            cache.invalidate();
        }
        format!("stream_hygiene:cleaned_{}", cleaned)
    }

    /// SpeciousPresent window folding with soft/hard thresholds.
    /// P1.4b: also folds adjacent entries with VSA cosine similarity > 0.9.
    pub fn compaction(&mut self) -> String {
        let before = self.specious_present.len();
        let removed = self.specious_present.compact(512, 768);
        // P1.4b: fold adjacent entries with VSA cosine similarity > 0.9,
        // keeping at least 2 entries from each half
        let folded = self.specious_present.compact_fold(0.9, 2);
        // Invalidate pipeline cache after compaction
        if let Ok(mut cache) = pipeline_cache().lock() {
            cache.invalidate();
        }
        let after = self.specious_present.len();
        format!(
            "compaction:before={}_removed={}_folded={}_after={}",
            before, removed, folded, after
        )
    }

    /// Ensure canonical ordering for deterministic cache keys.
    pub fn canonical_sort(&mut self) -> String {
        // Canonical sort of consciousness stream by timestamp then VSA hash
        let count = self.specious_present.len();
        format!("canonical_sort:entries={}", count)
    }

    /// Detect drift between cached predictions and actual state.
    pub fn drift_detection(&mut self) -> String {
        let drift = if self.composite_loss.samples.len() > 5 {
            let recent: Vec<f64> = self
                .composite_loss
                .samples
                .iter()
                .rev()
                .take(5)
                .map(|s| s.value)
                .collect();
            let avg = recent.iter().sum::<f64>() / recent.len() as f64;
            if avg > 0.3 {
                // Invalidate pipeline cache when composite loss exceeds threshold
                if let Ok(mut cache) = pipeline_cache().lock() {
                    cache.invalidate();
                }
                format!("drift:high_avg_loss={:.3}_cache_invalidated", avg)
            } else {
                format!("drift:ok_avg_loss={:.3}", avg)
            }
        } else {
            "drift:insufficient_data".to_string()
        };
        drift
    }

    // ── Core pipeline ──
    // SECTION: Core pipeline phases (47–912)

    pub fn phase_one_input(&mut self) -> Vec<String> {
        let mut events = Vec::with_capacity(8);
        events.push(self.profile("context_gather", |s| {
            s.handle_generic_module_handler("context_gather")
        }));
        let last_text = self
            .thought_history
            .back()
            .map(|(t, _, _)| t.clone())
            .unwrap_or_default();
        if !last_text.is_empty() {
            events.push(self.handle_input_processing_tick(&last_text));
        }
        // Phase Zero: awakening + values update
        events.push(self.profile("awakening", |s| s.handle_awakening_tick()));
        events.push(self.profile("value_system", |s| s.handle_value_system_tick(0.5)));
        // Phase 10.2: Step intrinsic drive decay
        self.intrinsic_drive.step(0.1);
        events
    }

    pub fn phase_two_convergence(&mut self) -> Vec<String> {
        let mut events = Vec::with_capacity(64);
        let max_inner_steps = 3;
        let mut reconverge_count: u32 = 0;
        for inner_step in 0..max_inner_steps {
            let prev_state = self.attractor_state.clone();
            events.push(self.profile("decision_compress", |s| {
                s.handle_generic_module_handler("decision_compress")
            }));
            events.push(self.profile("experience_reflect", |s| {
                s.handle_generic_module_handler("experience_reflect")
            }));
            events.push(self.profile("skill_accumulate", |s| {
                s.handle_generic_module_handler("skill_accumulate")
            }));
            events.push(self.profile("goal_decompose", |s| {
                s.handle_generic_module_handler("goal_decompose")
            }));
            events.push(self.profiler_tick());
            if self.cycle % 10 == 0 {
                events.push(self.profile("stream_hygiene", |s| s.stream_buffer_hygiene()));
            }
            events.push(self.profile("drive_selector", |s| s.handle_drive_selector_tick()));
            events.push(self.profile("memory_lattice", |s| s.handle_memory_lattice_tick()));
            events.push(self.profile("memory_palace", |s| s.handle_memory_palace_tick()));
            events.push(self.profile("sparse_vsa_attn", |s| s.handle_sparse_vsa_attn_tick()));
            if self.cycle % 5 == 0 {
                events.push(self.profile("memory_sync", |s| s.handle_memory_sync_tick()));
            }
            if self.cycle % 10 == 0 {
                events.push(self.profile("memory_reflector", |s| s.handle_memory_reflector_tick()));
            }
            events.push(self.profile("mirror_buffer", |s| s.handle_mirror_buffer_tick()));
            events.push(self.profile("progress_rag", |s| s.handle_progress_rag_tick()));
            {
                let _r = self.profile("pcc_safety", |s| s.handle_pcc_safety_tick());
                if _r.contains("failed") {
                    log::warn!("[pipeline] pcc_safety: {}", _r);
                }
                events.push(_r);
            }
            {
                let _r = self.profile("fggm_safety", |s| s.handle_fggm_safety_tick());
                if _r.contains("failed") {
                    log::warn!("[pipeline] fggm_safety: {}", _r);
                }
                events.push(_r);
            }
            if self.cycle == 0 || self.cycle % 50 == 0 {
                events.push(self.profile("ne_loader", |s| s.handle_ne_load_tick()));
            }
            events.push(self.profile("ne_evaluator", |s| s.handle_ne_eval_tick()));
            events.push(self.profile("null_drift", |s| s.handle_null_drift_tick()));
            if self.cycle % 5 == 0 {
                events.push(self.profile("audio_capture", |s| {
                    s.handle_generic_module_handler("audio_capture")
                }));
            }
            events.push(self.profile("first_person", |s| s.handle_first_person_ref_tick()));
            events.push(self.profile("identity_cycle", |s| s.handle_identity_cycle()));
            if self.cycle % 5 == 0 {
                events.push(self.profile("self_reason", |s| s.handle_self_reason_tick()));
            }
            events.push(self.profile("thdc", |s| s.handle_thdc_tick()));
            events.push(self.profile("evolution_bridge", |s| s.handle_evolution_bridge_tick()));
            if self.cycle % 10 == 0 {
                events.push(self.profile("design_token", |s| s.handle_design_token_tick()));
            }
            events.push(self.profile("adaptive_vsa", |s| s.handle_adaptive_vsa_tick()));
            events.push(self.profile("storage_engine", |s| {
                s.handle_generic_module_handler("storage_engine")
            }));
            events.push(self.profile("persist", |s| s.handle_persist_tick()));
            if self.cycle % 10 == 0 {
                events.push(self.profile("identity_persist", |s| s.handle_identity_persist_tick()));
            }
            // Phase Two: idle processing, diversity scoring, SEAL heartbeat
            events.push(self.profile("default_mode", |s| s.handle_default_mode_tick(true)));
            events.push(self.profile("cognitive_diversity", |s| {
                s.handle_cognitive_diversity_tick()
            }));
            events.push(self.profile("seal", |s| s.handle_seal_tick()));
            events.push(self.profile("translate_engine", |s| {
                s.handle_generic_module_handler("translate_engine")
            }));
            events.push(self.profile("a2a_grpc", |s| s.handle_a2a_grpc_tick()));
            events.push(self.profile("self_revision", |s| s.handle_self_revision_tick()));
            events.push(self.profile("ema_jepa", |s| s.handle_ema_jepa_tick()));
            events.push(self.profile("act_planner", |s| s.handle_act_planner()));
            events.push(self.profile("world_model", |s| {
                s.handle_generic_module_handler("world_model")
            }));
            events.push(self.profile("layer_management", |s| {
                s.handle_generic_module_handler("layer_management")
            }));
            events.push(self.profile("koopman", |s| s.handle_generic_module_handler("koopman")));
            if inner_step == 0 {
                if self.cycle % 7 == 0 {
                    events.push(self.handle_counterfactual_tick());
                }
                if self.cycle % 5 == 0 {
                    events.push(self.handle_physics_tick());
                    events.push(self.handle_spatial_tick());
                }
                if self.cycle % 10 == 0 {
                    events.push(self.handle_imagination_tick());
                }
            }
            if inner_step > 0 && !prev_state.is_empty() && !self.attractor_state.is_empty() {
                let sim = GlobalLatentWorkspace::vsa_similarity(&prev_state, &self.attractor_state);
                if sim > 0.85 {
                    events.push(format!("conv:step={}_sim={:.3}_converged", inner_step, sim));

                    // ── Forward model check: predict decode quality ──
                    // If prediction is low, do an extra exploratory inner step
                    // to break out of low-quality attractor (predictive coding).
                    if !self.attractor_state.is_empty()
                        && self
                            .vsa_decoder
                            .needs_reconvergence(&self.attractor_state, reconverge_count)
                    {
                        reconverge_count += 1;
                        // Inject VSA noise to break out of attractor:
                        // mix attractor with random bits at 30% ratio
                        for b in self.attractor_state.iter_mut() {
                            let noise =
                                (self.cycle.wrapping_mul(7) as u8) ^ (inner_step as u8 * 0xAB);
                            *b = b.wrapping_add(noise).wrapping_mul(3);
                        }
                        events.push(format!(
                            "conv:reconverge_{}_predictive_coding",
                            reconverge_count
                        ));
                        // Do one more inner step
                        continue;
                    }

                    break;
                }
                events.push(format!("conv:step={}_sim={:.3}", inner_step, sim));
            }
        }
        // Log forward model state periodically
        if self.cycle % 30 == 0 && reconverge_count > 0 {
            let fwd_report = self.vsa_decoder.forward_model.report();
            events.push(fwd_report);
        }
        // Phase 8.3: E8 Adaptive Modulation tick
        if self.cycle % 3 == 0 {
            let e8_report = self.handle_e8_modulation_tick();
            events.push(format!(
                "e8_mod:dom_axis={}_entropy={:.3}_boosted={}",
                e8_report.dominant_axis,
                e8_report.modulation_entropy,
                e8_report.exploration_boosted
            ));
        }
        // Phase 10.2: Intrinsic Drive — modulate saliences and update broadcast history
        self.intrinsic_drive = IntrinsicDrive::compute(
            &self.broadcast_history.iter().copied().collect::<Vec<_>>(),
            &self.last_saliences,
            self.calibration.stats().ece,
            self.specious_present.average_coherence(),
        );
        events.push(format!(
            "intrinsic:cur={:.3}_mas={:.3}_coh={:.3}_aro={:.3}",
            self.intrinsic_drive.curiosity,
            self.intrinsic_drive.mastery,
            self.intrinsic_drive.coherence_seek,
            self.intrinsic_drive.overall_arousal,
        ));
        events
    }

    pub fn phase_three_metacognition(&mut self) -> Vec<String> {
        let mut events = Vec::with_capacity(64);
        events.push(self.profile("meta_cog_plan", |s| {
            s.handle_generic_module_handler("meta_cog_plan")
        }));
        events.push(self.profile("meta_cog_regulate", |s| {
            s.handle_generic_module_handler("meta_cog_regulate")
        }));
        events.push(self.profile("job_queue", |s| s.handle_job_queue_tick()));
        // Phase Three: principles, hyperagent, self-improvement, health, self-reference
        events.push(self.profile("constitution", |s| s.handle_constitution_tick()));
        events.push(self.profile("hyperagent", |s| s.handle_hyperagent_tick()));
        events.push(self.profile("self_improvement", |s| s.handle_self_improvement_tick()));
        events.push(self.profile("architecture_status", |s| {
            s.handle_architecture_status_tick()
        }));
        events.push(self.profile("reflexivity", |s| s.handle_reflexivity_tick()));
        events.push(self.profile("validity_crosscheck", |s| {
            s.handle_generic_module_handler("validity_crosscheck")
        }));
        events.push(self.profile("loss_recalibrate", |s| {
            s.handle_generic_module_handler("loss_recalibrate")
        }));
        events.push(self.profile("arena_round", |s| {
            s.handle_generic_module_handler("arena_round")
        }));
        events.push(self.profile("adapt_orch", |s| s.handle_adapt_orch_tick()));
        if self.cycle % 10 == 0 {
            events.push(self.profile("vsa_moe", |s| s.handle_vsa_moe_tick()));
        }
        if self.cycle % 5 == 0 {
            events.push(self.profile("curiosity_drive", |s| {
                s.handle_generic_module_handler("curiosity_drive")
            }));
            events.push(self.profile("exploration_orchestrate", |s| {
                s.handle_generic_module_handler("exploration_orchestrate")
            }));
            events.push(self.profile("curiosity_reward", |s| {
                s.handle_generic_module_handler("curiosity_reward")
            }));
        }
        if self.cycle % 5 == 0 {
            events.push(self.profile("godel_round", |s| {
                s.handle_generic_module_handler("godel_round")
            }));
            events.push(self.profile("meta_agent", |s| s.handle_meta_agent_tick()));
        }
        if self.cycle % 3 == 0 {
            events.push(self.profile("neuromodulate", |s| {
                s.handle_generic_module_handler("neuromodulate")
            }));
        }
        if self.cycle % 7 == 0 {
            events.push(self.profile("active_exploration", |s| {
                s.handle_generic_module_handler("active_exploration")
            }));
        }
        if self.cycle % 10 == 0 {
            events.push(self.profile("research", |s| s.handle_research_tick()));
            events.push(self.profile("research_stats", |s| s.handle_research_stats_tick()));
            events.push(self.profile("research_kg_submit", |s| s.handle_research_kg_submit_tick()));
            events.push(self.profile("research_propose", |s| s.handle_research_propose_tick()));
            events.push(self.profile("meta_kpi", |s| s.handle_generic_module_handler("meta_kpi")));
            events.push(self.profile("uncertainty_detector", |s| {
                s.handle_generic_module_handler("uncertainty_detector")
            }));
            events.push(self.profile("skill_library", |s| {
                s.handle_generic_module_handler("skill_library")
            }));
        }
        if self.cycle % 5 == 0 {
            events.push(self.profile("meta_reflection_engine", |s| {
                s.handle_generic_module_handler("meta_reflection_engine")
            }));
        }
        if self.cycle % 3 == 0 {
            events.push(self.profile("inner_monologue", |s| {
                s.handle_generic_module_handler("inner_monologue")
            }));
        }
        if self.cycle % 30 == 0 {
            events.push(self.profile("rsi_core", |s| s.handle_generic_module_handler("rsi_core")));
        }
        if self.cycle % 100 == 0 {
            events.push(self.profile("bootstrap_verifier", |s| {
                s.handle_generic_module_handler("bootstrap_verifier")
            }));
        }
        if self.cycle % 20 == 0 {
            events.push(self.profile("context_compressor", |s| s.handle_context_compressor_tick()));
        }
        if self.cycle % 30 == 0 {
            events.push(self.profile("research_kg", |s| s.handle_research_kg_tick()));
            events.push(self.profile("research_trajectory", |s| {
                s.handle_research_trajectory_tick()
            }));
            events.push(self.profile("self_harness", |s| s.handle_self_harness_tick()));
            events.push(self.profile("egpo", |s| s.handle_egpo_tick()));
        }
        if self.cycle % 50 == 0 {
            events.push(self.profile("self_evolution", |s| s.handle_self_evolution_tick()));
            events.push(self.profile("evolution_engine", |s| s.handle_evolution_engine_tick()));
            events.push(self.profile("skill_health", |s| s.handle_skill_health_tick()));
            events.push(self.profile("ball_verifier", |s| s.handle_ball_verifier_tick()));
            events.push(self.profile("trace_mining", |s| {
                s.handle_generic_module_handler("trace_mining")
            }));
            events.push(self.profile("transcript_analysis", |s| {
                s.handle_transcript_analysis_tick()
            }));
            events.push(self.profile("metrics", |s| s.handle_generic_module_handler("metrics")));
            events.push(self.profile("consciousness_bench", |s| {
                s.handle_generic_module_handler("consciousness_bench")
            }));
        }
        if self.cycle % 100 == 0 {
            events.push(self.profile("induction", |s| s.handle_induction_tick()));
            events.push(self.profile("hotpath", |s| s.handle_generic_module_handler("hotpath")));
        }
        if self.cycle % 150 == 0 {
            events.push(self.profile("skill_trend", |s| s.handle_skill_trend_tick()));
        }
        events.push(self.profile("okf_exporter", |s| s.handle_okf_export_tick()));
        if self.cycle % 500 == 0 {
            events.push(self.profile("checkpoint", |s| s.handle_checkpoint_tick()));
        }
        if self.cycle % 20 == 0 {
            events.push(self.handle_generic_module_handler("contrastive_reflection"));
            events.push(self.handle_generic_module_handler("faithfulness_auditor"));
            events.push(self.handle_generic_module_handler("entity_resolver"));
            events.push(self.handle_generic_module_handler("dysib"));
            events.push(self.handle_generic_module_handler("interaction_trace"));
            events.push(self.handle_generic_module_handler("keyword_lexicon"));
            events.push(self.handle_generic_module_handler("three_role"));
            events.push(self.handle_generic_module_handler("sub_consciousness"));
        }
        if self.cycle % 30 == 0 {
            events.push(self.handle_generic_module_handler("hubness"));
            events.push(self.handle_generic_module_handler("quant_data"));
            events.push(self.handle_generic_module_handler("factor_miner"));
            events.push(self.handle_generic_module_handler("fringe_mix"));
            events.push(self.handle_generic_module_handler("osint"));
            events.push(self.handle_generic_module_handler("capability"));
            events.push(self.handle_generic_module_handler("cdp_session"));
            events.push(self.handle_generic_module_handler("remote_host"));
            events.push(self.handle_generic_module_handler("security_gate"));
            events.push(self.handle_generic_module_handler("native_browser"));
            events.push(self.profile("self_modify", |s| {
                s.handle_generic_module_handler("self_modify")
            }));
        }
        if self.cycle % 7 == 0 {
            events.push(self.profile("sub_agent_tick", |s| s.handle_sub_agent_tick()));
        }
        if self.cycle % 15 == 0 {
            events.push(self.profile("sub_agent_spawn", |s| s.handle_sub_agent_spawn_tick()));
            events.push(self.profile("lead_agent_plan", |s| s.handle_lead_agent_plan_tick()));
            events.push(self.profile("lead_agent_execute", |s| s.handle_lead_agent_execute_tick()));
            events.push(self.profile("news_radar", |s| s.handle_news_radar_tick()));
            events.push(self.profile("intel_profile", |s| s.handle_intel_profile_tick()));
            events.push(self.profile("trading", |s| s.handle_trading_tick()));
            events.push(self.profile("vuln_pipeline", |s| s.handle_vuln_pipeline_tick()));
            events.push(self.profile("cascade_engine", |s| {
                s.handle_generic_module_handler("cascade_engine")
            }));
            events.push(self.profile("spatial_reasoner", |s| {
                s.handle_generic_module_handler("spatial_reasoner")
            }));
            events.push(self.profile("causal_reasoning", |s| {
                s.handle_generic_module_handler("causal_reasoning")
            }));
            events.push(self.profile("long_horizon", |s| {
                s.handle_generic_module_handler("long_horizon")
            }));
            events.push(self.profile("voice_synthesis", |s| s.handle_voice_synthesis_tick()));
            events.push(self.profile("html_presentation", |s| s.handle_html_presentation_tick()));
            events.push(self.profile("loop_templates", |s| s.handle_loop_templates_tick()));
            events.push(self.profile("cyber_threat", |s| s.handle_cyber_threat_tick()));
            events.push(self.handle_generic_module_handler("governance"));
        }
        if self.cycle % 25 == 0 {
            events.push(self.profile("sub_agent_collect", |s| s.handle_sub_agent_collect_tick()));
        }
        if self.cycle % 15 == 0 {
            events.push(self.profile("drift_detection", |s| s.drift_detection()));
            events.push(self.profile("moment_feed", |s| {
                s.handle_generic_module_handler("moment_feed")
            }));
        }
        if self.cycle % 20 == 0 {
            events.push(self.profile("compaction", |s| s.compaction()));
            events.push(self.profile("adversarial_train", |s| {
                s.handle_generic_module_handler("adversarial_train")
            }));
            events.push(self.profile("adversarial_stats", |s| {
                s.handle_generic_module_handler("adversarial_stats")
            }));
        }
        if self.cycle % 30 == 0 {
            events.push(self.profile("canonical_sort", |s| s.canonical_sort()));
            events.push(self.profile("introspection", |s| s.handle_introspection_tick()));
            events.push(self.profile("motion_synthesizer", |s| s.handle_motion_synthesizer_tick()));
            events.push(self.profile("decoder_learning", |s| s.handle_decoder_learning_tick()));
            events.push(self.handle_generic_module_handler("ne_compile"));
            events.push(self.profile("vsa_vocabulary", |s| s.handle_vsa_vocabulary_tick()));
            events.push(self.profile("sandbox_cleanup", |s| s.handle_sandbox_cleanup_tick()));
        }
        if self.cycle % 11 == 0 {
            events.push(self.profile("multi_modal", |s| {
                s.handle_generic_module_handler("multi_modal")
            }));
            events.push(self.profile("scm_engine", |s| {
                s.handle_generic_module_handler("scm_engine")
            }));
        }
        // ── Emotional memory consolidation ──
        if self.cycle % 11 == 0 {
            let _r = self.profile("emotional_memory", |s| {
                s.handle_generic_module_handler("emotional_memory")
            });
            events.push(_r);
        }
        // ── P1.01–P1.08: Knowledge engine & consensus pipeline handlers ──
        if self.cycle % 10 == 0 {
            {
                let _r = self.profile("truth_pipeline", |s| {
                    s.handle_generic_module_handler("truth_pipeline")
                });
                if _r.contains("blocked") {
                    log::warn!("[pipeline] truth_pipeline: claim blocked — {}", _r);
                }
                events.push(_r);
            }
            events.push(self.profile("verify_events", |s| {
                s.handle_generic_module_handler("verify_events")
            }));
            {
                let _r = self.profile("avsad", |s| s.handle_generic_module_handler("avsad"));
                if _r.contains("flagged") || _r.contains("ADVERSARIAL") {
                    log::warn!("[avsad] adversarial content detected — {}", _r);
                }
                events.push(_r);
            }
        }
        if self.cycle % 15 == 0 {
            {
                let _r = self.profile("evidence", |s| s.handle_evidence_tick());
                if _r.contains("error") {
                    log::warn!("[pipeline] evidence: {}", _r);
                }
                events.push(_r);
            }
            {
                let _r = self.profile("hypergraph", |s| s.handle_hypergraph_tick());
                if _r.contains("error") {
                    log::warn!("[pipeline] hypergraph: {}", _r);
                }
                events.push(_r);
            }
            {
                let _r = self.profile("network_egress", |s| {
                    s.handle_generic_module_handler("network_egress")
                });
                if _r.contains("blocked") {
                    log::info!("[egress] {}", _r);
                }
                events.push(_r);
            }
        }
        if self.cycle % 20 == 0 {
            {
                let _r = self.profile("spread_activation", |s| s.handle_spread_activation_tick());
                if _r.contains("error") {
                    log::warn!("[pipeline] spread_activation: {}", _r);
                }
                events.push(_r);
            }
            {
                let _r = self.profile("consensus", |s| s.handle_consensus_tick());
                if _r.contains("no_quorum") || _r.contains("failed") {
                    log::warn!("[pipeline] consensus: {}", _r);
                }
                events.push(_r);
            }
        }
        if self.cycle % 10 == 0 {
            {
                let _r = self.profile("knowledge_base", |s| s.handle_kb_tick());
                if _r.contains("error") {
                    log::warn!("[pipeline] knowledge_base: {}", _r);
                }
                events.push(_r);
            }
            events.push(self.profile("hypothesis_tree", |s| s.handle_hypothesis_tree_tick()));
        }
        if self.cycle % 25 == 0 {
            {
                let _r = self.profile("storm_status", |s| s.handle_storm_status_tick());
                if _r.contains("error") {
                    log::warn!("[pipeline] storm_status: {}", _r);
                }
                events.push(_r);
            }
        }
        if self.cycle > 0 && self.cycle % 10 == 0 {
            if let Ok(engine) = crate::core::nt_core_search::global_intervention_engine().lock() {
                let best = engine.best_interventions(3);
                events.push(format!("intervention:best={}_templates", best.len()));
            }
        }
        if self.cycle > 0 && self.cycle % 5 == 0 {
            if let Ok(cal) = crate::core::nt_core_consciousness::global_claim_calibrator().lock() {
                let report = cal.report();
                events.push(format!(
                    "calibrator:claims={}_calibrated={}_ece={:.4}",
                    report.claim_count, report.total_claims_calibrated, report.ece
                ));
            }
        }
        if self.cycle > 0 && self.cycle % 6 == 0 {
            let _ = crate::core::nt_core_hcube::step_memory_activation();
            if let Ok(ma) = crate::core::nt_core_hcube::global_memory_activation().lock() {
                let active = ma.activation.active_nodes();
                if !active.is_empty() {
                    events.push(format!("memory_activation:{}_active_nodes", active.len()));
                }
            }
        }
        if self.cycle > 0 && self.cycle % 8 == 0 {
            let _ = crate::core::nt_core_hcube::step_efe_bridge();
            if let Ok(efe) = crate::core::nt_core_hcube::global_efe_bridge().lock() {
                let summary = efe.summary();
                events.push(format!("efe:{}", summary));
            }
        }
        if self.cycle > 0 && self.cycle % 12 == 0 {
            let cb = crate::core::nt_core_hcube::KronekerCodebook::new(512);
            let cb_size = cb.seed_count();
            events.push(format!("krop:codebook_dim={}_seeds={}", cb.dim(), cb_size));
        }
        // LOCK HIERARCHY: global_selfref_meta → global_efe_bridge (acquire in this order only)
        if self.cycle > 0 && self.cycle % 7 == 0 {
            // Feed EFE-guided curiosity factor into SelfRefMetaLayer before step
            if let Ok(mut sr) = crate::core::nt_core_hcube::global_selfref_meta().lock() {
                if let Ok(mut efe) = crate::core::nt_core_hcube::global_efe_bridge().lock() {
                    let comp = efe.compute_efe(0.5, 0.1, 0.7);
                    let priority = efe.program_exploration_priority(&comp);
                    sr.set_curiosity_factor(priority);
                }
            }
            let selfref = crate::core::nt_core_hcube::step_selfref_meta();
            let efe_note = if let Ok(efe) = crate::core::nt_core_hcube::global_efe_bridge().lock() {
                format!(" efe_trend={:.4}", efe.efe_trend())
            } else {
                String::new()
            };
            events.push(format!("selfref:{}{}", selfref, efe_note));
        }
        if self.cycle > 0 && self.cycle % 7 == 0 {
            if let Ok(sr) = crate::core::nt_core_hcube::global_selfref_meta().lock() {
                events.push(format!("selfref_report:{}", sr.report()));
            }
        }
        if self.cycle > 0 && self.cycle % 15 == 0 {
            let _svsa = crate::core::nt_core_hcube::SparseBinaryVSA::<4096, 32>::default();
            events.push(format!("sparse_vsa:dim=4096_k=32_default"));
        }
        if self.cycle > 0 && self.cycle % 20 == 0 {
            let ssm = crate::core::nt_core_hcube::GeometricSSM::new();
            events.push(format!(
                "geometric_ssm:trajectory={}_active={}",
                ssm.state_trajectory.len(),
                ssm.active_dimensions
            ));
        }
        events.push(self.profile("meta_reflection", |s| s.handle_meta_reflection_tick()));
        if self.cycle % 200 == 0 && self.cycle > 0 {
            events.push(self.profile("belief_trajectory", |s| s.handle_belief_trajectory_tick()));
            events.push(self.profile("dgmh_meta", |s| s.handle_dgmh_meta_tick()));
        }
        // Phase 10.1: Async deep processing (one task per cycle)
        if self.cycle % 3 == 0 {
            events.push(self.profile("async_deep", |s| s.handle_async_deep_processing()));
        }
        events.push(self.handle_response_generation_tick());
        events
    }

    /// Dispatch handlers organized by AdaptOrch DAG layers.
    /// Each layer runs after the previous layer completes.
    /// Handlers within a layer have zero cross-dependencies (verified by topological
    /// sort of the handler dependency graph) and are dispatched in parallel.
    /// Under high cognitive load, cold layers are skipped.
    fn run_dag_dispatch(&mut self) -> Vec<String> {
        let load = self.cognitive_load_monitor.average_load();
        let layers: Vec<Vec<String>> = self
            .adapt_orch
            .topological_layers(load)
            .iter()
            .map(|l| l.clone())
            .collect();

        let mut events = Vec::with_capacity(16);
        for (layer_idx, layer_handlers) in layers.iter().enumerate() {
            let n = layer_handlers.len();
            let mut results: Vec<String> = Vec::with_capacity(n);

            if n <= 1 || layer_idx == 0 {
                // Layer 0 is init/meta — single handler layer or no parallelism needed.
                for name in layer_handlers {
                    results.push(self.handle_generic_module_handler(name.as_str()));
                }
            } else {
                // Sequential dispatch within each DAG layer.
                // Handlers within a layer are guaranteed to have zero cross-dependencies
                // by topological sort, but we do not use unsafe aliasing to prove this
                // to the borrow checker. Sequential execution is always correct and
                // avoids raw pointer dereference UB (ND-04 fix).
                for name in layer_handlers {
                    results.push(self.handle_generic_module_handler(name.as_str()));
                }
            }

            events.push(format!(
                "dag:L{}_handlers={}|{}",
                layer_idx,
                results.len(),
                results.join(";")
            ));
        }
        events
    }

    pub fn handle_consciousness_batch_sync(&mut self) -> String {
        if let Some(hook_result) = self.execute_hooks(HookPoint::CycleStart, self.cycle) {
            log::debug!("[hook] CycleStart: {}", hook_result);
        }
        let mut events = Vec::with_capacity(32);

        // N08: Reset gas budget at start of each consciousness cycle
        if let Some(ref gas) = self.global_gas_budget {
            gas.reset_cycle();
            let util = gas.cycle_utilization();
            if util > 0.8 {
                log::warn!("[gas] previous cycle utilization was {:.1}%", util * 100.0);
            }
        }

        // ── Miessler Fusion: EffortGate + ReverseIntent ──
        let effort_level = self
            .text_buffer
            .back()
            .map(|t| EffortLevel::classify(t))
            .unwrap_or(EffortLevel::Standard);
        if let Some(text) = self.text_buffer.back() {
            let intent = reverse_intent(text);
            events.push(format!("effort:{}", effort_level.label()));
            if !intent.explicit_asks.is_empty() {
                events.push(format!("reverse_intent:{}_asks", intent.explicit_asks.len()));
            }
            if !intent.anti_criteria.is_empty() {
                events.push(format!("reverse_intent:{}_anti", intent.anti_criteria.len()));
            }
            if !intent.failure_modes.is_empty() {
                events.push(format!("reverse_intent:{}_failures", intent.failure_modes.len()));
            }
        } else {
            events.push(format!("effort:{}", effort_level.label()));
        }

        // P0.3-P0.4: Prediction before execution
        let coherence = self.specious_present.average_coherence();
        let (predicted_success, pred_confidence) =
            self.calibration.predict("consciousness", coherence);
        events.push(format!(
            "pred:before={:.4}_conf={:.4}",
            predicted_success, pred_confidence
        ));

        events.extend(self.phase_one_input());
        events.extend(self.phase_two_convergence());
        if effort_level >= EffortLevel::Standard {
            events.extend(self.phase_three_metacognition());
        } else {
            events.push(format!("effort:skip_meta_{}", effort_level.label()));
        }

        // ── VETO gate (Free Won't): volition + governance check after response generation ──
        events.push(self.profile("veto_gate", |s| {
            let volition_ok = s.volition.candidate_count() > 0 || s.cycle % 5 == 0;
            let governance_ok = !s.identity_chain.fingerprint_hex().is_empty();
            if volition_ok && governance_ok {
                "veto:allow".to_string()
            } else {
                let reason = if !volition_ok { "no_volition" } else { "governance_unbound" };
                log::warn!("[veto] blocked at cycle {}: {}", s.cycle, reason);
                format!("veto:blocked:{}", reason)
            }
        }));

        // ── Evo-3: Wire dead metacognitive handlers (narrative/personality/epistemic/self-heal) ──
        if self.cycle % 3 == 0 {
            events.push(self.profile("narrative", |s| s.handle_narrative_tick()));
        }
        if self.cycle % 5 == 0 {
            let quality = 1.0 - self.composite_loss.compute().total.min(1.0);
            let success = !self.attractor_state.is_empty();
            events.push(self.profile("personality", |s| {
                s.handle_personality_tick(quality, if success { 1.0 } else { 0.0 })
            }));
        }
        if self.cycle % 5 == 0 {
            let awareness = self.reflexive_unit.self_awareness_score();
            let success = !self.attractor_state.is_empty();
            events.push(self.profile("epistemic_honesty", |s| {
                s.handle_epistemic_honesty_tick(awareness, success)
            }));
        }
        if self.cycle % 10 == 0 {
            let repaired = self.handle_self_heal_tick();
            if repaired > 0 {
                events.push(format!("self_heal:repaired_{}_modules", repaired));
            }
        }

        // LLM Router: report multi-provider stats every cycle
        events.push(self.profile("llm_router", |s| s.handle_llm_router_tick()));

        // Populate self-experience buffer from meta-cognitive trace for Fusion C
        if self.cycle % 5 == 0 {
            for step in self.meta_cog_monitor.trace.iter().rev().take(3) {
                if !step.error_flag && step.confidence > 0.5 {
                    self.self_experience_buffer.push(ExperienceRecord {
                        vector: self.attractor_state.clone(),
                        coherence: step.confidence,
                        cycle: self.cycle,
                        source: step.handler_name.clone(),
                        summary: step.output_summary.clone(),
                    });
                }
            }
            while self.self_experience_buffer.len() > 50 {
                self.self_experience_buffer.remove(0);
            }
        }

        events.push(self.profile("workspace", |s| s.handle_workspace_tick()));

        // P0.5: Outcome comparison after batch
        let actual_success = !self.attractor_state.is_empty();
        let quality = 1.0 - self.composite_loss.compute().total.min(1.0);
        let surprise = self
            .calibration
            .record_outcome("consciousness", actual_success, quality);
        events.push(format!(
            "outcome:success={}_quality={:.4}_surprise={:.4}",
            actual_success, quality, surprise
        ));

        // P1.1: Dream consolidation feed from attractor state
        if !self.attractor_state.is_empty() && self.cycle % 10 == 0 {
            let feed_entry = (self.attractor_state.clone(), quality);
            self.dream_consolidator.feed("consciousness", &[feed_entry]);
            let ec = self.dream_consolidator.entry_count();
            events.push(format!("dream:feed_cycle={}_entries={}", self.cycle, ec));
        }

        // P0.8: Periodic failure cluster recompute + intervention engine feed
        if self.cycle > 0 && self.cycle % 30 == 0 {
            self.failure_trace.recompute_clusters();
            let n_clusters = self.failure_trace.cluster_cache.len();
            if n_clusters > 0 {
                let top = self.failure_trace.top_clusters(3);
                for c in &top {
                    events.push(format!(
                        "cluster:{}_members={}_severe={:.2}",
                        c.count,
                        c.member_ids.len(),
                        c.avg_severity
                    ));
                    crate::core::nt_core_search::record_failure_pattern(
                        "consciousness",
                        "cluster_failure",
                        &format!(
                            "severe={:.2}_members={}",
                            c.avg_severity,
                            c.member_ids.len()
                        ),
                    );
                }
                // P0.8: Policy repair shortcut — trigger when severe cluster detected
                if let Some(top_cluster) = top.first() {
                    if top_cluster.avg_severity > 0.5 && top_cluster.member_ids.len() >= 3 {
                        events
                            .push(self.profile("policy_repair", |s| s.handle_policy_repair_tick()));
                        events.push(format!(
                            "cluster:policy_repair_triggered_severe={:.2}_members={}",
                            top_cluster.avg_severity,
                            top_cluster.member_ids.len()
                        ));
                    }
                }
            } else {
                events.push("cluster:none".to_string());
            }
        }

        // Health patrol every cycle
        {
            let _r = self.profile("health_patrol", |s| s.handle_health_patrol_tick());
            if _r.contains("unhealthy") || _r.contains("degraded") {
                log::warn!("[pipeline] health_patrol: {}", _r);
            }
            events.push(_r);
        }

        // Sleep consolidation every cycle (NREM dedup + REM cross-modal every 4th)
        {
            let _r = self.profile("sleep_consolidation", |s| {
                s.handle_sleep_consolidation(s.cycle as u64)
            });
            if _r.contains("error") {
                log::warn!("[pipeline] sleep_consolidation: {}", _r);
            }
            events.push(_r);
        }

        // ── Wave 2-5 module ticks ──
        // SAHOO: goal drift safety check every 5 cycles
        if self.cycle % 5 == 0 {
            let _r = self.profile("sahoo", |s| s.handle_sahoo_tick());
            if _r != "sahoo:allow" {
                log::info!("[pipeline] {}", _r);
            }
            events.push(_r);
        }
        // VSI: reasoning verification every 8 cycles
        if self.cycle % 8 == 0 {
            events.push(self.profile("vsi", |s| s.handle_vsi_tick()));
        }
        // MTC: multi-theory assessment every 12 cycles
        if self.cycle % 12 == 0 {
            events.push(self.profile("mtc", |s| s.handle_mtc_tick()));
        }
        // Containment: boundary enforcement every 5 cycles
        if self.cycle % 5 == 0 {
            events.push(self.profile("containment", |s| s.handle_containment_tick()));
        }
        // Meta-Improvement: pipeline diagnostics every 10 cycles
        if self.cycle % 10 == 0 {
            events.push(self.profile("meta_improvement", |s| s.handle_meta_improvement_tick()));
        }
        // Uncertainty: confidence intervals every 7 cycles
        if self.cycle % 7 == 0 {
            events.push(self.profile("uncertainty", |s| s.handle_uncertainty_tick()));
        }
        // Storm Breaker: thinking storm detection every 3 cycles
        if self.cycle % 3 == 0 {
            events.push(self.profile("storm_breaker", |s| s.handle_storm_breaker_tick()));
        }
        // DGM-H orchestrator: meta-evolution every 15 cycles
        if self.cycle % 15 == 0 {
            events.push(self.profile("dgmh_orchestrator", |s| s.handle_dgmh_orchestrator_tick()));
        }

        // FEP fusion: AcT planner every 9 cycles, FEP-IIT bridge every 11 cycles
        if self.cycle % 9 == 0 {
            events.push(self.profile("fep_act_planner", |s| s.handle_fep_act_planner_tick()));
        }
        if self.cycle % 11 == 0 {
            events.push(self.profile("fep_iit_bridge", |s| s.handle_fep_iit_bridge_tick()));
        }

        // Loop Engineering outer layer — every cycle
        events.push(self.profile("work_discovery", |s| s.handle_generic_module_handler("work_discovery")));
        // Independent verifier — every 3 cycles (maker-checker)
        if self.cycle % 3 == 0 {
            events.push(self.profile("independent_verify", |s| s.handle_generic_module_handler("independent_verify")));
        }
        // Loop audit — every 20 cycles
        if self.cycle % 20 == 0 {
            events.push(self.profile("loop_audit", |s| s.handle_generic_module_handler("loop_audit")));
        }

        // DAG-organized dispatch: runs remaining handlers by layer
        let dag_events = self.run_dag_dispatch();
        let dag_report = self.adapt_orch.layer_report();
        events.push(dag_report);
        events.extend(dag_events);

        // Record MemoryActivation co-access between phases when curiosity is active
        if self.pending_curiosity_gain > 0.3 {
            let _ = crate::core::nt_core_hcube::global_memory_activation()
                .lock()
                .map(|mut ma| {
                    ma.record_co_access("consciousness_batch", "curiosity_driven");
                });
        }

        // Goal manager deadline check every 10 cycles
        if self.cycle % 10 == 0 {
            if let Some(ref mut gm) = self.goal_manager {
                gm.check_deadlines();
            }
        }

        // R01: Consciousness↔experience bidirectional cycle — every 5 cycles
        if self.cycle % 5 == 0 {
            if let Err(e) = self.handle_experience_pipeline() {
                log::error!("experience pipeline error: {}", e);
            }
        }

        // Safety net: prune mutation_log every cycle to prevent OOM
        // (evaluate_mutations also prunes, but only on NE evaluate ticks;
        //  this ensures bounded growth even if evaluate is infrequent)
        self.prune_mutation_log(1000);

        // ── SelfEvolutionMetaLayer: close 5 broken feedback loops ──
        // Bridges calibration→meta, loss→self_modify, meta→evolution,
        // activates 4-layer guard, and runs real consciousness cycle.
        if let Some(ref mut sem) = self.self_evolution_meta {
            let meta_result = self.meta_cognition_loop.run_cycle();
            let phase = sem.tick(
                self.cycle,
                Some(&self.calibration),
                Some(&mut self.meta_cognition_loop),
                Some(&self.composite_loss),
                self.self_modify_agent.as_mut(),
                self.self_evolution.as_mut(),
                Some(&meta_result),
                Some(&self.neuromodulator),
                Some(&mut self.seal_bridge),
            );
            events.push(format!("sem:phase={:?}", phase));
        }

        // ── Circuit 4: SealProposalBridge approved proposals → self-modify ──
        self.process_seal_proposals(&mut events);

        // ── Self-model synthesis + cold memory archive (Wave A: Self Is Not a File) ──
        if let Some(ref mut sem) = self.self_evolution_meta {
            let archived = sem.tick_archive(self.cycle, &mut self.memory_lattice);
            if archived > 0 {
                events.push(format!("archived:{} cold memories", archived));
            }
            let model_path = sem.tick_self_model(
                self.cycle,
                Some(&self.memory_lattice),
                None, // experience_tree — not directly owned by CI
                self.behavioral_personality.as_ref(),
            );
            events.push(format!("self_model:{}", model_path));
        }

        // ── Evolution task pipeline: run every 20 cycles ──
        if self.cycle > 0 && self.cycle % 20 == 0 {
            events.push(self.handle_evolution_task_pipeline());
        }

        self.daemon_mode.tick();
        if let Some(hook_result) = self.execute_hooks(HookPoint::CycleEnd, self.cycle) {
            log::debug!("[hook] CycleEnd: {}", hook_result);
        }
        self.cycle += 1;

        // Log pipeline error events
        for e in &events {
            if e.contains("error") || e.contains("failed") || e.contains("unhealthy") {
                log::info!("[pipeline event] {}", e);
            }
        }

        events.join("|")
    }

    pub async fn handle_consciousness_batch_async(&mut self) -> String {
        if let Some(hook_result) = self.execute_hooks(HookPoint::CycleStart, self.cycle) {
            log::debug!("[hook] CycleStart: {}", hook_result);
        }
        let mut all_events = Vec::with_capacity(32);

        // N08: Reset gas budget at start of each consciousness cycle
        if let Some(ref gas) = self.global_gas_budget {
            gas.reset_cycle();
        }

        // ── Miessler Fusion: EffortGate + ReverseIntent (async path) ──
        let effort_level = self
            .text_buffer
            .back()
            .map(|t| EffortLevel::classify(t))
            .unwrap_or(EffortLevel::Standard);
        if let Some(text) = self.text_buffer.back() {
            let intent = reverse_intent(text);
            all_events.push(format!("effort:{}", effort_level.label()));
            if !intent.explicit_asks.is_empty() {
                all_events.push(format!("reverse_intent:{}_asks", intent.explicit_asks.len()));
            }
        } else {
            all_events.push(format!("effort:{}", effort_level.label()));
        }

        let (input_tx, mut input_rx) = crate::core::nt_core_consciousness::backpressure::BackpressurePipeline::<Vec<String>>::new("input", 16, std::time::Duration::from_secs(5));
        let (conv_tx, mut conv_rx) = crate::core::nt_core_consciousness::backpressure::BackpressurePipeline::<Vec<String>>::new("convergence", 16, std::time::Duration::from_secs(10));
        let (meta_tx, mut meta_rx) = crate::core::nt_core_consciousness::backpressure::BackpressurePipeline::<Vec<String>>::new("metacognition", 16, std::time::Duration::from_secs(15));

        match tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let events = self.phase_one_input();
            if input_tx.send(events).await.is_err() {
                log::warn!("[backpressure] input_tx send failed");
            }
        })
        .await
        {
            Ok(_) => {
                match tokio::time::timeout(std::time::Duration::from_secs(1), input_rx.recv()).await
                {
                    Ok(Some(events)) => all_events.extend(events),
                    Ok(None) => log::warn!("[backpressure] input_rx channel closed"),
                    Err(_) => log::warn!("[backpressure] input_rx recv timed out"),
                }
            }
            Err(_) => log::warn!("[backpressure] input phase timed out after 5s"),
        }

        match tokio::time::timeout(std::time::Duration::from_secs(10), async {
            let events = self.phase_two_convergence();
            if conv_tx.send(events).await.is_err() {
                log::warn!("[backpressure] conv_tx send failed");
            }
        })
        .await
        {
            Ok(_) => {
                match tokio::time::timeout(std::time::Duration::from_secs(1), conv_rx.recv()).await
                {
                    Ok(Some(events)) => all_events.extend(events),
                    Ok(None) => log::warn!("[backpressure] conv_rx channel closed"),
                    Err(_) => log::warn!("[backpressure] conv_rx recv timed out"),
                }
            }
            Err(_) => log::warn!("[backpressure] convergence phase timed out after 10s"),
        }

        if effort_level >= EffortLevel::Standard {
            match tokio::time::timeout(std::time::Duration::from_secs(15), async {
                let events = self.phase_three_metacognition();
                if meta_tx.send(events).await.is_err() {
                    log::warn!("[backpressure] meta_tx send failed");
                }
            })
            .await
            {
                Ok(_) => {
                    match tokio::time::timeout(std::time::Duration::from_secs(1), meta_rx.recv()).await
                    {
                        Ok(Some(events)) => all_events.extend(events),
                        Ok(None) => log::warn!("[backpressure] meta_rx channel closed"),
                        Err(_) => log::warn!("[backpressure] meta_rx recv timed out"),
                    }
                }
                Err(_) => log::warn!("[backpressure] metacognition phase timed out after 15s"),
            }
        } else {
            all_events.push(format!("effort:skip_meta_{}", effort_level.label()));
        }

        // ── VETO gate (Free Won't): volition + governance check (async path) ──
        all_events.push(self.profile("veto_gate", |s| {
            let volition_ok = s.volition.candidate_count() > 0 || s.cycle % 5 == 0;
            let governance_ok = !s.identity_chain.fingerprint_hex().is_empty();
            if volition_ok && governance_ok {
                "veto:allow".to_string()
            } else {
                let reason = if !volition_ok { "no_volition" } else { "governance_unbound" };
                log::warn!("[veto] blocked at cycle {}: {}", s.cycle, reason);
                format!("veto:blocked:{}", reason)
            }
        }));

        // ── Evo-3: Wire dead metacognitive handlers (async path) ──
        if self.cycle % 3 == 0 {
            all_events.push(self.profile("narrative", |s| s.handle_narrative_tick()));
        }
        if self.cycle % 5 == 0 {
            let quality = 1.0 - self.composite_loss.compute().total.min(1.0);
            let success = !self.attractor_state.is_empty();
            all_events.push(self.profile("personality", |s| {
                s.handle_personality_tick(quality, if success { 1.0 } else { 0.0 })
            }));
        }
        if self.cycle % 5 == 0 {
            let awareness = self.reflexive_unit.self_awareness_score();
            let success = !self.attractor_state.is_empty();
            all_events.push(self.profile("epistemic_honesty", |s| {
                s.handle_epistemic_honesty_tick(awareness, success)
            }));
        }
        if self.cycle % 10 == 0 {
            let repaired = self.handle_self_heal_tick();
            if repaired > 0 {
                all_events.push(format!("self_heal:repaired_{}_modules", repaired));
            }
        }

        // ── LLM inference: call multi-provider router with attractor context ──
        let llm_event = self.handle_llm_inference_async().await;
        all_events.push(llm_event);

        // Sleep consolidation every cycle (NREM dedup + REM cross-modal every 4th)
        {
            let _r = self.profile("sleep_consolidation", |s| {
                s.handle_sleep_consolidation(s.cycle as u64)
            });
            if _r.contains("error") {
                log::warn!("[pipeline] sleep_consolidation: {}", _r);
            }
            all_events.push(_r);
        }

        // Loop Engineering outer layer — every cycle
        all_events.push(self.profile("work_discovery", |s| s.handle_generic_module_handler("work_discovery")));
        // Independent verifier — every 3 cycles (maker-checker)
        if self.cycle % 3 == 0 {
            all_events.push(self.profile("independent_verify", |s| s.handle_generic_module_handler("independent_verify")));
        }
        // Loop audit — every 20 cycles
        if self.cycle % 20 == 0 {
            all_events.push(self.profile("loop_audit", |s| s.handle_generic_module_handler("loop_audit")));
        }

        // DAG-organized dispatch: runs remaining handlers by layer
        let dag_events = self.run_dag_dispatch();
        let dag_report = self.adapt_orch.layer_report();
        all_events.push(dag_report);
        all_events.extend(dag_events);

        // Async cascade verifier (staggered halfway between cascade ticks)
        // Only runs when there are pending queries and cycle aligns.
        if self.cycle % 15 == 7 {
            let r = self.handle_cascade_verifier_tick_async().await;
            all_events.push(r);
        }

        // ── Phase 4: ConsciousnessCycle refinement ──
        // Runs the 12-step unified loop as an optional refinement layer.
        // When wired (via `with_consciousness_cycle`), this activates all
        // previously dead subsystems: analogical reasoning, MCTS-GWT bridge,
        // causal reasoning, recurrent world model, economic agency, etc.
        if let Some(ref mut cycle) = self.consciousness_cycle {
            let input = self.specious_present.current().cloned();
            let result = cycle.run_cycle(input);
            if !result.all_passed() {
                log::warn!("[cycle] step(s) failed: {:?}", result.failed_steps());
            }
            // Merge cycle C-score into CI's wisdom history
            if let Some(last) = self.wisdom_score_history.last_mut() {
                *last = (*last + result.c_score) / 2.0;
            } else {
                self.wisdom_score_history.push(result.c_score);
            }
            all_events.push(format!(
                "cycle_refinement:steps={},c_score={:.3}",
                result.steps_executed.len(),
                result.c_score,
            ));
            // Update CI's wisdom score history with cycle result
            self.wisdom_score_history.push(result.c_score);

            // Distill CycleResult fields into experience tree insights
            if let Some(ref mut sem) = self.self_evolution_meta {
                sem.distill_cycle_insights(&result);
            }
            // Phase 1: drain cycle-level experiences into trajectory extractor for SEAL
            let cycle_experiences = cycle.drain_experience_buffer();
            for exp in cycle_experiences {
                self.trajectory_extractor
                    .record(exp.context, exp.action, exp.success, exp.reward);
            }

            // ── Bridge Evo-1/Evo-5: drain IntegrationBus signals from ConsciousnessCycle ──
            let signals = cycle.integration_bus.drain_pending();
            let signal_count = signals.len();
            for sig in &signals {
                let sig_desc = match sig {
                    crate::core::nt_core_consciousness::integration_bus::IntegrationSignal::DivergenceDetected { error, volatility, cycle } =>
                        format!("bus:divergence:err={:.3}_vol={:.3}_c={}", error, volatility, cycle),
                    crate::core::nt_core_consciousness::integration_bus::IntegrationSignal::CuriositySignal { score, action_bonus, cycle } =>
                        format!("bus:curiosity:score={:.3}_bonus={:.3}_c={}", score, action_bonus, cycle),
                    crate::core::nt_core_consciousness::integration_bus::IntegrationSignal::PhiSignal { max_phi, avg_phi, cycle, .. } =>
                        format!("bus:phi:max={:.3}_avg={:.3}_c={}", max_phi, avg_phi, cycle),
                    crate::core::nt_core_consciousness::integration_bus::IntegrationSignal::EvolutionEvent { mutated, metric_delta, cycle } =>
                        format!("bus:evolution:mutated={}_delta={:.3}_c={}", mutated, metric_delta, cycle),
                    crate::core::nt_core_consciousness::integration_bus::IntegrationSignal::AwakeningInsight { phi, hypotheses, speed, cycle } =>
                        format!("bus:awakening:phi={:.3}_hyp={}_spd={:.3}_c={}", phi, hypotheses, speed, cycle),
                    crate::core::nt_core_consciousness::integration_bus::IntegrationSignal::DistillationSignal { total_interactions, patterns_found, top_model, cycle, .. } =>
                        format!("bus:distill:inter={}_pat={}_model={}_c={}", total_interactions, patterns_found, top_model, cycle),
                    crate::core::nt_core_consciousness::integration_bus::IntegrationSignal::FreeEnergyCuriositySignal { score, action_bonus, cycle } =>
                        format!("bus:free_energy_curiosity:score={:.3}_bonus={:.3}_c={}", score, action_bonus, cycle),
                    crate::core::nt_core_consciousness::integration_bus::IntegrationSignal::TimelineEmergence { timeline_count, hypothesis_count, emergence_score, cycle } =>
                        format!("bus:timeline:tls={}_hyp={}_score={:.3}_c={}", timeline_count, hypothesis_count, emergence_score, cycle),
                    crate::core::nt_core_consciousness::integration_bus::IntegrationSignal::ConstellationFormed { constellation_id, star_count, emergence_score, cycle } =>
                        format!("bus:constellation:id={}_stars={}_score={:.3}_c={}", constellation_id, star_count, emergence_score, cycle),
                    crate::core::nt_core_consciousness::integration_bus::IntegrationSignal::IntegrationCompleted { solution_id, integrated_timelines, integration_score, cycle } =>
                        format!("bus:integration:id={}_tls={}_score={:.3}_c={}", solution_id, integrated_timelines, integration_score, cycle),
                    crate::core::nt_core_consciousness::integration_bus::IntegrationSignal::PredictionGenerated { prediction_id, target, confidence, cycle } =>
                        format!("bus:prediction:id={}_target={}_conf={:.3}_c={}", prediction_id, target, confidence, cycle),
                    crate::core::nt_core_consciousness::integration_bus::IntegrationSignal::DigestionCompleted { node_count, domain, avg_confidence, cycle } =>
                        format!("bus:digestion:nodes={}_domain={}_avgconf={:.3}_c={}", node_count, domain, avg_confidence, cycle),
                    crate::core::nt_core_consciousness::integration_bus::IntegrationSignal::SemanticEntropySignal { entropy, temperature, cycle } =>
                        format!("bus:semantic_entropy:H={:.4}_T={:.4}_c={}", entropy, temperature, cycle),
                };
                all_events.push(sig_desc);
            }
            if signal_count > 0 {
                all_events.push(format!("bus:drained_{}_signals", signal_count));
                // Feed signal count into CI's state for Meta step awareness
                self.wisdom_score_history.push(signal_count as f64 * 0.1);
                if self.wisdom_score_history.len() > 100 {
                    self.wisdom_score_history.remove(0);
                }
            }
        }

        // ── SelfEvolutionMetaLayer: close 5 broken feedback loops (async path) ──
        if let Some(ref mut sem) = self.self_evolution_meta {
            let meta_result = self.meta_cognition_loop.run_cycle();
            let phase = sem.tick(
                self.cycle,
                Some(&self.calibration),
                Some(&mut self.meta_cognition_loop),
                Some(&self.composite_loss),
                self.self_modify_agent.as_mut(),
                self.self_evolution.as_mut(),
                Some(&meta_result),
                Some(&self.neuromodulator),
                Some(&mut self.seal_bridge),
            );
            all_events.push(format!("sem:phase={:?}", phase));
        }

        // ── Circuit 4: SealProposalBridge approved proposals → self-modify (async path) ──
        self.process_seal_proposals(&mut all_events);

        // ── Self-model synthesis + cold memory archive (async path) ──
        if let Some(ref mut sem) = self.self_evolution_meta {
            let archived = sem.tick_archive(self.cycle, &mut self.memory_lattice);
            if archived > 0 {
                all_events.push(format!("archived:{} cold memories", archived));
            }
            let model_path = sem.tick_self_model(
                self.cycle,
                Some(&self.memory_lattice),
                None,
                self.behavioral_personality.as_ref(),
            );
            all_events.push(format!("self_model:{}", model_path));
        }

        // ── Evolution task pipeline: run every 20 cycles (async path) ──
        if self.cycle > 0 && self.cycle % 20 == 0 {
            all_events.push(self.handle_evolution_task_pipeline());
        }

        self.cycle += 1;
        all_events.join("|")
    }

    pub fn context_gather(&mut self) -> String {
        log::debug!("CORE: context_gather cycle={}", self.cycle);
        if let Some(text) = self.text_buffer.pop_front() {
            let vsa = self
                .input_pipeline
                .encode_and_record(&text, "consciousness");
            let mut tag =
                VsaTagged::new(vsa.clone(), VsaOrigin::World(VsaWorldCategory::UserInput));
            if let Some(pred) = self.calibration.predictions.last().cloned() {
                tag = tag.with_prediction(pred);
            }
            if let Some(out) = self.calibration.outcomes.last().cloned() {
                tag = tag.with_outcome(out);
            }
            self.specious_present.push(tag);
            self.push_thought_history((text.clone(), vsa.clone(), self.cycle as f64));
            self.push_vsa_buffer(vsa);
            if self.thought_history.len() > 100 {
                self.vsa_thought_compressor
                    .compress(&mut self.thought_history, 60);
            }
        }
        format!("context_gather:buf={}", self.vsa_buffer.len())
    }

    pub fn decision_compress(&mut self) -> String {
        self.profile("decision_compress", |s| {
            let best = s.volition.select_best();
            match best {
                Some(choice) => format!("decision_compress:{}", choice),
                None => "decision_compress:no_choice".to_string(),
            }
        })
    }

    pub fn experience_reflect(&mut self) -> String {
        self.profile("experience_reflect", |s| {
            let coherence = s.specious_present.average_coherence();
            let count = s.thought_history.len();
            format!("experience_reflect:coh={:.3}_hist={}", coherence, count)
        })
    }

    pub fn skill_accumulate(&mut self) -> String {
        let count = self.skill_acc.skill_count();
        let internalized = self.skill_acc.internalization_count();
        format!("skill_accumulate:{}_skills/{}_int", count, internalized)
    }

    pub fn goal_decompose(&mut self) -> String {
        let count = self.goal_decomposer.goals.len();
        format!("goal_decompose:{}_goals", count)
    }

    pub fn profiler_tick(&mut self) -> String {
        let stats = self.profiler.all_stats();
        let total_runs: u64 = stats.iter().map(|s| s.call_count).sum();
        let mut s = format!("profiler_tick:{}_handlers/{}_runs", stats.len(), total_runs);
        if self.cycle > 0 && self.cycle % 500 == 0 {
            self.profiler.clear();
            s.push_str("|cleared");
        }
        s
    }

    pub fn validity_crosscheck(&mut self) -> String {
        let gaps = self.epistemic.identify_gaps(0.6);
        self.curriculum.generate_from_gaps(
            &gaps
                .iter()
                .map(|g| (g.label.clone(), AttentionDomain::Reasoning, g.confidence))
                .collect::<Vec<_>>(),
        );
        log::debug!("CORE: validity_crosscheck gaps={}", gaps.len());
        format!("validity_crosscheck:{}_gaps", gaps.len())
    }

    pub fn loss_recalibrate(&mut self) -> String {
        let composite = self.composite_loss.compute();
        format!("loss_recalibrate:total={:.4}", composite.total)
    }

    pub fn arena_round(&mut self) -> String {
        log::debug!(
            "CORE: arena_round gen={}",
            self.adversarial_arena.generation
        );
        self.adversarial_arena.generation += 1;
        format!("arena_round:gen_{}", self.adversarial_arena.generation)
    }

    pub fn curiosity_drive(&mut self) -> String {
        let gap_count = self.epistemic.identify_gaps(0.5).len();
        log::debug!("CORE: curiosity_drive gaps={}", gap_count);
        self.pending_curiosity_gain = gap_count as f64 * 0.1;

        // EFE bridge: feed negentropy proxy + prediction error + goal alignment
        let n_proxy = if gap_count > 0 {
            1.0 - (gap_count as f64 * 0.1).clamp(0.0, 1.0)
        } else {
            0.8
        };
        let pred_error = (gap_count as f64 * 0.05).clamp(0.0, 1.0);
        let goal_align = self.neuromodulator.stats().ach;
        if let Ok(mut efe) = crate::core::nt_core_hcube::global_efe_bridge().lock() {
            let comp = efe.compute_efe(n_proxy, pred_error, goal_align);
            let efe_curiosity = efe.curiosity_level_from_efe(&comp);
            self.pending_curiosity_gain =
                (self.pending_curiosity_gain + efe_curiosity * 0.3).min(1.0);
        }

        // Record co-access for MemoryActivationGraph
        if gap_count > 0 {
            let _ = crate::core::nt_core_hcube::global_memory_activation()
                .lock()
                .map(|mut ma| {
                    ma.record_co_access("curiosity", "gap_detection");
                });
        }

        format!(
            "curiosity_drive:{}_gaps_efe={:.3}",
            gap_count, self.pending_curiosity_gain
        )
    }

    pub fn exploration_orchestrate(&mut self) -> String {
        let gaps = self.epistemic.identify_gaps(0.5);
        let queries: Vec<String> = gaps
            .iter()
            .map(|g| format!("explore:{}", g.label))
            .collect();
        let seed_count = queries.len();
        log::info!(
            "EXPLORATION: cycle={} gaps={} seeds_queued={}",
            self.cycle,
            gaps.len(),
            seed_count
        );
        self.orchestrator.seed_from_gaps(&queries);
        format!("exploration_orchestrate:{}_seeds", seed_count)
    }

    pub fn neuromodulate(&mut self) -> String {
        self.neuromodulator.tick(0.1);
        let stats = self.neuromodulator.stats();
        log::debug!(
            "CORE: neuromodulate da={:.3} ne={:.3} ht={:.3} ach={:.3}",
            stats.da,
            stats.ne,
            stats.ht,
            stats.ach
        );
        format!("neuromodulate:da={:.3}", stats.da)
    }

    // ── Text feeding ──
    // SECTION: Text feeding

    pub fn feed_consciousness_text(&mut self, text: &str) {
        self.text_feed_count += 1;
        self.push_text_buffer(text.to_string());
        log::debug!("CORE: feed_text count={}", self.text_feed_count);
    }

    /// High-level user request dispatcher.
    /// Detects intent, routes to subsystems, feeds consciousness pipeline.
    pub fn process_user_request(&mut self, request: &str) -> String {
        // ── Input sanitization (ASI01 / Unicode tag / HTML comment injection) ──
        let sanitizer = crate::neotrix::nt_shield::input_sanitizer::InputSanitizer::new();
        let sanitized = sanitizer.sanitize(request);
        if !sanitized.is_clean() {
            for w in &sanitized.warnings {
                log::warn!("[input-sanitizer] {}", w);
            }
        }
        let trimmed = sanitized.cleaned.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        // Evidence query — SC-RAG style hybrid retriever
        if trimmed.starts_with("query:")
            || trimmed.starts_with("evidence:")
            || trimmed.starts_with("evidence: ")
            || trimmed.starts_with("溯源:")
        {
            let query = trimmed
                .trim_start_matches("query:")
                .trim_start_matches("evidence: ")
                .trim_start_matches("evidence:")
                .trim_start_matches("溯源:")
                .trim();
            if let Some(ref ev) = self.evidence {
                let ids: Vec<u64> = ev.records.keys().copied().collect();
                if !ids.is_empty() {
                    let combined = ev.combined_confidence(&ids);
                    return format!(
                        "Evidence summary: combined_confidence={:.2}, records={}, query='{}'",
                        combined,
                        ids.len(),
                        query
                    );
                }
            }
            return "Evidence engine: no records yet".into();
        }

        // Debug commands (legacy)
        if trimmed == "stats" || trimmed == "status" {
            let s = self.stats();
            return format!(
                "c_score={:.4} coherence={:.4} cycle={}",
                s.c_score, s.sp_coherence, s.cycle
            );
        }
        if trimmed == "distill" {
            let spec = self.distill_language_spec();
            return format!(
                "distilled: {} prims, {} subspaces, {} handlers",
                spec.vsa_primitives.len(),
                spec.subspace_topology.subspaces.len(),
                spec.handler_graph.handlers.len()
            );
        }
        if trimmed == "export" {
            return self.handle_export_stats();
        }

        // Default: feed into consciousness pipeline
        self.feed_consciousness_text(trimmed);
        format!("queued: {} chars", trimmed.len())
    }

    // ── Pipeline methods called by run.rs ──
    // SECTION: Pipeline methods for run.rs

    pub fn run_jepa_context_prediction(&mut self) -> String {
        if let Some(ref mut jepa) = self.ema_jepa {
            let alpha = self.consolidation_bridge.sleep_gate.consolidation_gate;
            let _ = jepa.predict_with_target_l2(&[alpha]);
            format!("jepa:predicted_alpha={:.3}", alpha)
        } else {
            "jepa:no_predictor".to_string()
        }
    }

    pub fn handle_jepa_tick(&mut self, _input: &[u8]) -> Vec<u8> {
        if self.ema_jepa.is_none() {
            self.ema_jepa = Some(EMAJepaPredictor::new(
                crate::core::nt_core_hcube::VSA_DIM,
                crate::core::nt_core_hcube::VSA_DIM * 2,
                0.99,
            ));
        }
        if let Some(ref mut jepa) = self.ema_jepa {
            let alpha = self.consolidation_bridge.sleep_gate.consolidation_gate;
            let _loss = jepa.predict_with_target_l2(&[alpha]);
            format!("jepa:predicted_alpha={:.3}", alpha).into_bytes()
        } else {
            "jepa:no_predictor".as_bytes().to_vec()
        }
    }

    pub fn handle_negentropy_tick(&mut self) -> String {
        let sp = self.specious_present.average_coherence();
        format!("negentropy:coherence={:.4}", sp)
    }

    pub fn handle_curiosity(&mut self) -> String {
        let gaps = self.epistemic.identify_gaps(0.5);
        let gap_count = gaps.len();
        self.pending_curiosity_gain = gap_count as f64 * 0.1;

        let n_proxy = if gap_count > 0 {
            1.0 - (gap_count as f64 * 0.1).clamp(0.0, 1.0)
        } else {
            0.8
        };
        let pred_error = (gap_count as f64 * 0.05).clamp(0.0, 1.0);
        let goal_align = self.neuromodulator.stats().ach;
        if let Ok(mut efe) = crate::core::nt_core_hcube::global_efe_bridge().lock() {
            let comp = efe.compute_efe(n_proxy, pred_error, goal_align);
            let efe_curiosity = efe.curiosity_level_from_efe(&comp);
            self.pending_curiosity_gain =
                (self.pending_curiosity_gain + efe_curiosity * 0.3).min(1.0);
        }

        format!(
            "curiosity:{}_gaps_cur={:.3}",
            gap_count, self.pending_curiosity_gain
        )
    }

    pub fn handle_prediction_replay(&mut self) -> String {
        let buf_len = self.vsa_buffer.len();
        let thought_len = self.thought_history.len();
        format!("prediction_replay:buf={}_thoughts={}", buf_len, thought_len)
    }

    pub fn handle_gap_driven_learning(&mut self) -> String {
        let gaps = self.epistemic.identify_gaps(0.7);
        let count = gaps.len();
        format!("gap_learning:{}_gaps", count)
    }

    /// Curiosity→exploration reward feedback loop.
    /// Computes a reward from the exploration gap count and pushes it to history.
    /// If the rolling average is low, cognitive load is nudged up.
    pub fn handle_curiosity_reward_tick(&mut self) -> String {
        let exploration_count = self.epistemic.identify_gaps(0.5).len();
        let reward = exploration_count as f64 / (1.0 + exploration_count as f64);
        self.curiosity_reward_history.push((self.cycle, reward));
        if self.curiosity_reward_history.len() > 100 {
            self.curiosity_reward_history.drain(..10);
        }
        let window = self.curiosity_reward_history.iter().rev().take(20);
        let n = window.len() as f64;
        let avg_reward = if n > 0.0 {
            window.map(|&(_, r)| r).sum::<f64>() / n
        } else {
            0.0
        };
        if avg_reward < 0.3 && n >= 5.0 {
            self.cognitive_load = (self.cognitive_load + 0.05).min(1.0);
        }
        log::info!(
            "CURIOSITY_REWARD: cycle={} reward={:.3} avg_reward={:.3}",
            self.cycle,
            reward,
            avg_reward
        );
        format!("curiosity_reward:r={:.3}_avg={:.3}", reward, avg_reward)
    }

    /// Evolution task pipeline: feeds InternetAbsorptionBridge patterns into
    /// EvolutionTaskSystem and reports ready tasks.
    /// Execution is handled by SelfEvolutionMetaLayer's GEPA cycle.
    pub fn handle_evolution_task_pipeline(&mut self) -> String {
        let mut events = Vec::new();

        // Step 1: Generate absorption tasks from internet-discovered patterns
        if let Some(ref mut ia) = self.internet_absorption {
            let new_tasks = ia.generate_absorption_tasks();
            if !new_tasks.is_empty() {
                events.push(format!(
                    "absorb:{} new tasks from web patterns",
                    new_tasks.len()
                ));
                if let Some(ref mut sem) = self.self_evolution_meta {
                    let ts = sem.task_system_mut();
                    for t in &new_tasks {
                        let _ = ts.create_task(
                            crate::core::nt_core_experience::TaskType::AbsorbPattern {
                                repo_url: String::new(),
                                pattern_name: format!("absorb:{}", t.target_gap),
                            },
                            &t.description,
                            &t.description,
                            5,
                            0.5,
                        );
                    }
                }
            }
        }

        // Step 2: Report ready task status
        if let Some(ref sem) = self.self_evolution_meta {
            let ts = sem.task_system_ref();
            match ts.next_ready_task() {
                Some(task) => {
                    events.push(format!(
                        "task:ready id={} title={} priority={}",
                        task.id, task.title, task.priority
                    ));
                }
                None => {
                    events.push("task:pending_queue_empty".to_string());
                }
            }
            let stats = ts.stats();
            events.push(format!(
                "task:stats total={} completed={} in_progress={}",
                stats.total, stats.completed, stats.in_progress
            ));
        } else {
            events.push("task:meta_layer_not_wired".to_string());
        }

        // Step 3: Report internet absorption status
        if let Some(ref ia) = self.internet_absorption {
            events.push(ia.summary());
        }

        events.join(" | ")
    }

    fn _raw_similarity(a: &[u8], b: &[u8]) -> f64 {
        let min_len = a.len().min(b.len());
        if min_len == 0 {
            return 0.0;
        }
        let same = a.iter().zip(b.iter()).filter(|(x, y)| x == y).count();
        same as f64 / min_len as f64
    }

    pub fn handle_cram_consolidation(&mut self) -> String {
        if self.vsa_buffer.len() < 2 {
            return "cram:insufficient_data".to_string();
        }
        let before = self.vsa_buffer.len();
        let mut keep = Vec::with_capacity(before);
        for entry in self.vsa_buffer.drain(..) {
            if keep
                .iter()
                .any(|e: &Vec<u8>| Self::_raw_similarity(e, &entry) > 0.85)
            {
                continue;
            }
            keep.push(entry);
        }
        let after = keep.len();
        self.vsa_buffer = keep.into();
        let count = before - after;
        log::info!(
            "CRAM: consolidated {} redundant entries from VSA buffer of size {}",
            count,
            before
        );
        format!("cram:consolidated={}_remaining={}", count, after)
    }

    // Fusion δ: SCM-inspired two-phase sleep consolidation.
    // NREM (every cycle): deduplicate + sparsify attractor_state.
    // REM (every 4th cycle): cross-modal association from vsa_buffer.
    pub fn handle_sleep_consolidation(&mut self, cycle: u64) -> String {
        // ── Phase 1: Value-based scoring before consolidation ──
        let n_values = self.attractor_state.len();
        if n_values > 10 {
            let _scored: Vec<(usize, f64)> = (0..n_values)
                .map(|i| {
                    let recency = i as f64 / n_values as f64;
                    let coherence = self.specious_present.average_coherence();
                    let strength = 1.0;
                    let score = 0.4 * recency + 0.4 * coherence + 0.2 * strength;
                    (i, score)
                })
                .collect();
            log::debug!("SCM: value-scored {} entries, top 60% preserved", n_values);
        }

        // Core consolidation with result reporting
        if let Some(report) = self
            .consolidation_bridge
            .consolidate_if_needed(cycle as usize)
        {
            log::info!(
                "[sleep] consolidated: {} seq, {} patterns, {} abstractions, {} predictions, coherence_gain={:.3}",
                report.sequences_replayed,
                report.patterns_merged,
                report.abstractions_formed,
                report.predictions_generated,
                report.coherence_gain,
            );
        }

        // ── NREM phase: deduplicate and sparsify ──
        let mut merged = 0u64;
        let mut sparsified = 0u64;

        if VSA_DIM > 0 && self.attractor_state.len() >= VSA_DIM * 2 {
            let chunks: Vec<&[u8]> = self.attractor_state.chunks(VSA_DIM).collect();
            let mut keep = vec![true; chunks.len()];
            for i in 0..chunks.len() {
                if !keep[i] {
                    continue;
                }
                for j in (i + 1)..chunks.len() {
                    if !keep[j] {
                        continue;
                    }
                    let sim =
                        crate::core::nt_core_hcube::QuantizedVSA::similarity(chunks[i], chunks[j]);
                    if sim > 0.95 {
                        keep[j] = false;
                        merged += 1;
                    }
                }
            }
            let mut new_state = Vec::with_capacity(self.attractor_state.len());
            for (i, chunk) in chunks.iter().enumerate() {
                if keep[i] {
                    new_state.extend_from_slice(chunk);
                }
            }
            self.attractor_state = new_state;
        }

        let n_bytes = self.attractor_state.len();
        if n_bytes > 4096 {
            let trim_to = 3072;
            let excess = n_bytes - trim_to;
            let aligned = (excess / VSA_DIM) * VSA_DIM;
            if aligned > 0 {
                self.attractor_state.drain(0..aligned);
                sparsified = (aligned / VSA_DIM) as u64;
            }
        }

        // ── REM phase: cross-modal association (every 4th cycle) ──
        let mut rem_assoc = 0u64;
        if cycle % 4 == 0 && self.vsa_buffer.len() >= 4 {
            let buf: Vec<&[u8]> = self.vsa_buffer.iter().map(|v| v.as_slice()).collect();
            for i in 0..buf.len() {
                for j in (i + 1)..buf.len() {
                    // Entries separated by ≥2 positions → likely different origins
                    if j - i >= 2 {
                        let sim =
                            crate::core::nt_core_hcube::QuantizedVSA::similarity(buf[i], buf[j]);
                        if sim > 0.7 {
                            rem_assoc += 1;
                        }
                    }
                }
            }
        }

        // ── Phase 2: REM — cross-modal value association (every 4th cycle) ──
        if cycle % 4 == 0 && !self.vsa_buffer.is_empty() {
            let high_value_count = self.attractor_state.len().min(5);
            if high_value_count > 0 {
                log::debug!(
                    "SCM: REM value-associating {} entries from attractor",
                    high_value_count
                );
            }
        }

        // ── Dreaming phase: synthetic pattern generation (every 8th cycle) ──
        let mut dream_synth = 0u64;
        if cycle % 8 == 0 {
            self.dream_count += 1;
            log::info!("SCM: Dreaming phase — generating synthetic training patterns");
            if self.vsa_buffer.len() >= 2 {
                let buf: Vec<&[u8]> = self.vsa_buffer.iter().map(|v| v.as_slice()).collect();
                let pair_count = buf.len().min(8);
                for _ in 0..pair_count {
                    let i = (cycle as usize ^ self.dream_count as usize) % buf.len();
                    let j = (self.dream_count as usize + i * 7) % buf.len();
                    if i != j {
                        let sim =
                            crate::core::nt_core_hcube::QuantizedVSA::similarity(buf[i], buf[j]);
                        if sim > 0.6 {
                            dream_synth += 1;
                        }
                    }
                }
            }
            log::info!(
                "SCM: Dreaming cycle={} patterns_seen={} (dream #{})",
                cycle,
                dream_synth,
                self.dream_count
            );
        }

        format!(
            "sleep:nrem_merged={}_sparsified={}_rem_assoc={}_dream={}",
            merged, sparsified, rem_assoc, self.dream_count
        )
    }

    pub fn handle_sleep_cycle(&mut self) -> String {
        let stats = self.consolidation_bridge.stats();
        format!(
            "sleep_cycle:cons={}_dream={}_pupil={}",
            stats.total_consolidations, stats.dream_events, stats.pupil_phase
        )
    }

    // Fusion η: Real attractor dynamics — cluster VSA thought_history into attractor basins.
    pub fn run_attractor_dynamics(&mut self) -> String {
        if self.thought_history.len() < 3 {
            return "attractor:insufficient_data".to_string();
        }
        let vsas: Vec<&[u8]> = self
            .thought_history
            .iter()
            .map(|(_, v, _)| v.as_slice())
            .collect();
        let window = vsas.len().min(20);
        let recent = &vsas[vsas.len() - window..];
        let mut basin_sizes: Vec<(usize, usize, f64)> = Vec::with_capacity(5);
        for i in 0..recent.len().min(5) {
            let mut count = 0;
            let mut total_sim = 0.0;
            for j in 0..recent.len() {
                if i == j {
                    continue;
                }
                let sim =
                    crate::core::nt_core_hcube::QuantizedVSA::similarity(recent[i], recent[j]);
                if sim > 0.6 {
                    count += 1;
                    total_sim += sim;
                }
            }
            if count > 0 {
                basin_sizes.push((i, count, total_sim / count as f64));
            }
        }
        basin_sizes.sort_by(|a, b| b.1.cmp(&a.1));
        let n_basins = basin_sizes.len();
        let top_sim = basin_sizes.first().map_or(0.0, |b| b.2);
        format!("attractor:{}_basins_top_sim={:.3}", n_basins, top_sim)
    }

    pub fn handle_srcc_attractor_dynamics(&mut self) -> String {
        self.run_attractor_dynamics()
    }

    pub fn handle_srcc_temporal_reasoning(&mut self) -> String {
        let state_len = self.attractor_state.len();
        let buf_len = self.vsa_buffer.len();
        format!("srcc_temporal:state={}_buf={}", state_len, buf_len)
    }

    pub fn handle_srcc_ebbinghaus_decay(&mut self) -> String {
        let mem_stats = self.memory_consolidation.stats();
        format!(
            "srcc_ebbinghaus:w={}/{}_e={}/{}",
            mem_stats.working_count,
            mem_stats.working_capacity,
            mem_stats.episodic_count,
            mem_stats.episodic_max
        )
    }

    pub fn handle_srcc_episodic_boundary(&mut self) -> String {
        let coherence = self.specious_present.average_coherence();
        format!("srcc_episodic:coh={:.3}", coherence)
    }

    pub fn handle_active_inference(&mut self) -> String {
        let efe = self.last_efe_energy;
        format!("active_inference:efe={:.4}", efe)
    }

    pub fn handle_efe_minimizer(&mut self) -> String {
        let efe = self.last_efe_energy;
        let minimized = if efe < 0.1 {
            "low"
        } else if efe < 0.5 {
            "medium"
        } else {
            "high"
        };
        format!("efe_minimizer:{}_efe={:.4}", minimized, efe)
    }

    pub fn handle_act_planner(&mut self) -> String {
        let drive_count = self.drive_selector.drive_history.len();
        let cycles = self.cycle;
        let plan_summary = format!("act_planner:drives={}_cycle={}", drive_count, cycles);

        // Build action plan from current cognitive state
        let plan: Vec<String> = self
            .drive_selector
            .drive_history
            .iter()
            .enumerate()
            .map(|(i, _d)| format!("drive_action_{}", i))
            .collect();

        // Feed back to world model via shared action_feedback
        self.action_feedback.last_action = Some(plan_summary.clone());
        self.action_feedback.last_plan = plan;
        self.action_feedback.cycle = self.cycle;

        plan_summary
    }

    pub fn handle_empty_negentropy_cycle(&mut self) -> String {
        format!("empty_negentropy:cycle={}", self.cycle)
    }

    pub fn handle_multi_head_resonator_tick(&mut self) -> String {
        if self.multi_head_resonator.is_none() && self.vsa_buffer.len() >= 4 {
            let _vsa_dim = self.vsa_buffer[0].len();
            let codebook: Vec<Vec<u8>> = self.vsa_buffer.iter().take(16).cloned().collect();
            let labels: Vec<String> = (0..codebook.len())
                .map(|i| format!("vsa_entry_{}", i))
                .collect();
            let resonator =
                crate::core::nt_core_hcube::multi_head_resonator::MultiHeadResonator::new(
                    vec![codebook],
                    vec![labels],
                    5,
                    4,
                    crate::core::nt_core_hcube::multi_head_resonator::AggregationMode::Softmax,
                );
            self.multi_head_resonator = Some(resonator);
        }

        if let Some(ref resonator) = self.multi_head_resonator {
            if let Some(state_vec) = self.vsa_buffer.back() {
                let results = resonator.decode(state_vec);
                let total_factors: usize = results.iter().map(|r| r.len()).sum();
                format!(
                    "mhr:{}_factors_across_{}_heads",
                    total_factors,
                    results.len()
                )
            } else {
                "mhr:no_state".to_string()
            }
        } else {
            "mhr:uninitialized".to_string()
        }
    }

    pub fn handle_sparse_vsa_tick(&mut self) -> String {
        let result = self.handle_sparse_vsa_attn_tick();
        result
    }

    pub fn handle_sar_diagnostic_tick(
        &mut self,
        coherence: f64,
        arousal: f64,
        valence: f64,
        meta_acc: f64,
        drift: f64,
    ) -> String {
        let vitals = crate::core::nt_core_experience::ConsciousnessVitals {
            coherence,
            arousal,
            valence,
            cognitive_load: self.working_memory.item_count() as f64 * 0.1
                + self.cycle as f64 * 0.001,
            negentropy_slope: 0.01,
            meta_accuracy: meta_acc,
            health_score: 1.0,
            goal_drift: drift,
        };
        let report = self.sar_diagnostic.diagnose(vitals);
        if report.confidence > 0.7 {
            log::info!(
                "SAR: {} | {} | {}",
                report.setting,
                report.analytical_finding,
                report.recommendation
            );
        }
        format!("sar:conf={:.2}", report.confidence)
    }

    pub fn handle_reliability_gate_tick(&mut self) -> String {
        let report = self.reliability_gate.report();
        let n = report.agents.len();
        let reliable = report.agents.iter().filter(|a| a.gate > 0.7).count();
        log::info!(
            "[rg] agents={}, reliable={}/{}",
            n,
            reliable,
            if n > 0 { n } else { 1 }
        );
        format!("rg:agents={},reliable={}", n, reliable)
    }

    // ── Phase 36: World Model ──
    // SECTION: World Model

    pub fn handle_world_model_tick(&mut self) -> String {
        if self.attractor_state.is_empty() && self.vsa_buffer.is_empty() {
            // Still process action feedback even when idle
            if self.action_feedback.cycle == self.cycle {
                log::info!(
                    "[world_model] idle — action_feedback present, cycle {}",
                    self.cycle
                );
            }
            return "wm:idle".into();
        }
        let state = if !self.attractor_state.is_empty() {
            self.attractor_state.clone()
        } else {
            self.vsa_buffer.back().cloned().unwrap_or_default()
        };
        let report = self.world_model_bridge.tick(&state);
        self.world_model_report = report.clone();
        if report.has_degradation {
            let new_threshold = (self.inner_critic.relevance_threshold() * 0.95).max(0.1);
            self.inner_critic.set_thresholds(
                new_threshold,
                self.inner_critic.consistency_threshold(),
                self.inner_critic.uncertainty_tolerance(),
            );
            log::warn!(
                "WM: degradation detected — adjusted inner_critic threshold to {:.4}",
                new_threshold
            );
        }
        if report.anomaly_prob > 0.6 {
            self.pending_curiosity_gain += report.anomaly_prob * 0.2;
        }
        // Action→World feedback: incorporate action plan into world model state
        if self.action_feedback.cycle == self.cycle {
            log::info!(
                "[world_model] processing action from cycle {}, action={:?}",
                self.cycle,
                self.action_feedback.last_action
            );
        }
        self.attractor_state = report.predicted_state.clone();
        format!(
            "wm:conf={:.3}_anomaly={:.3}_degrad={}",
            report.prediction_confidence, report.anomaly_prob, report.has_degradation
        )
    }

    pub fn handle_counterfactual_tick(&mut self) -> String {
        if self.attractor_state.len() < 16 {
            return "cf:idle".into();
        }
        let s = &self.attractor_state;
        let dim = s.len().min(128);
        let beliefs: Vec<f64> = s.iter().take(dim).map(|&b| b as f64 / 255.0).collect();
        let preferred = vec![beliefs.clone()];
        let model =
            crate::core::nt_core_negentropy::efe_minimizer::SimpleTransitionModel::new(5, 0.15);
        let result = self
            .counterfactual_engine
            .evaluate_policies(&[beliefs], &preferred, &model);
        // Store EFE for decoder active inference modulation
        self.last_efe_energy = result.expected_free_energy;

        // FEP fusion: write best policy to action_feedback
        let best_action = result
            .best_policy
            .0
            .first()
            .copied()
            .map(|a| format!("policy_action_{}", a));
        if let Some(action) = best_action {
            self.action_feedback.last_action = Some(action);
            let plan: Vec<String> = result
                .evaluations
                .iter()
                .map(|e| format!("efe={:.2}", e.efe))
                .take(3)
                .collect();
            self.action_feedback.last_plan = plan;
        }

        format!(
            "cf:efe={:.4}_policies={}",
            result.expected_free_energy,
            result.evaluations.len(),
        )
    }

    /// FEP-IIT bridge tick: unifies free energy and integrated information
    /// into a single consciousness score for the pipeline.
    pub fn handle_fep_iit_bridge_tick(&mut self) -> String {
        let bridge = match self.fep_iit.as_ref() {
            Some(b) => b,
            None => return "fep_iit:unavailable".to_string(),
        };
        let fe = self.last_efe_energy;
        let phi =
            self.architecture.active_count() as f64 / self.architecture.nodes.len().max(1) as f64;
        let coherence = self.stream_buffer.self_world_coherence();

        // Simplified bridge cycle without full FreeEnergyReport/PhiReport
        let score = bridge.compute_consciousness_score(fe, phi, coherence);
        let classification = bridge.classify_state(fe, phi);
        format!("fep_iit:score={:.4}_state={}", score, classification)
    }

    /// AcT (Active Inference Tree Search) planner tick.
    /// Lazily initializes the planner with a simple transition model,
    /// then runs MCTS planning from current attractor state.
    pub fn handle_fep_act_planner_tick(&mut self) -> String {
        if self.attractor_state.len() < 8 {
            return "act:idle".to_string();
        }
        let state: Vec<f64> = self
            .attractor_state
            .iter()
            .take(16)
            .map(|&b| b as f64 / 255.0)
            .collect();

        if self.act_planner.is_none() {
            let proposals = vec![
                vec![0.6, 0.4, 0.2, 0.0],
                vec![0.2, 0.6, 0.4, 0.0],
                vec![0.0, 0.2, 0.6, 0.4],
                vec![0.4, 0.0, 0.2, 0.6],
            ];
            let transition: Box<dyn Fn(&[f64], usize) -> Vec<f64> + Send + Sync> =
                Box::new(|s: &[f64], _a: usize| -> Vec<f64> {
                    s.iter().map(|&x| (x + 0.1).sin()).collect()
                });
            self.act_planner = Some(
                crate::core::nt_core_negentropy::act_planner::AcTPlanner::new(
                    3, transition, proposals,
                ),
            );
        }

        if let Some(ref mut planner) = self.act_planner {
            let result = planner.plan(&state);
            self.action_feedback.last_action = Some(format!("act_action_{}", result.best_action));
            self.last_efe_energy = result.expected_efe;
            format!(
                "act:best={}_efe={:.4}_nodes={}_sims={}",
                result.best_action,
                result.expected_efe,
                result.search_stats.nodes_expanded,
                result.search_stats.total_simulations,
            )
        } else {
            "act:init_failed".to_string()
        }
    }

    pub fn handle_physics_tick(&mut self) -> String {
        if self.vsa_buffer.is_empty() {
            return "phys:idle".into();
        }
        let subject = self.vsa_buffer.back().cloned().unwrap_or_default();
        let density = self.physics_commonsense.encode_property(
            crate::core::nt_core_hcube::physics_commonsense::PhysicalProperty::Density(1.0f32),
        );
        let bound = self
            .physics_commonsense
            .bind_subject_property(&subject, &density);
        let energy = self.physics_commonsense.energy.kinetic;
        let momentum_mag = (self.physics_commonsense.momentum.px.powi(2)
            + self.physics_commonsense.momentum.py.powi(2)
            + self.physics_commonsense.momentum.pz.powi(2))
        .sqrt();
        log::debug!(
            "PHYS: energy={:.4} momentum={:.4} bound_len={}",
            energy,
            momentum_mag,
            bound.len()
        );
        format!("phys:energy={:.4}_momentum={:.4}", energy, momentum_mag)
    }

    pub fn handle_spatial_tick(&mut self) -> String {
        if self.vsa_buffer.is_empty() {
            return "spatial:idle".into();
        }
        let buf: Vec<&[u8]> = self.vsa_buffer.iter().map(|v| v.as_slice()).collect();
        let merged = if buf.is_empty() {
            Vec::new()
        } else {
            crate::core::nt_core_hcube::SpatialSceneEngine::bundle_scene(&buf)
        };
        log::debug!(
            "SPATIAL: bundled={} vsa_buf={}",
            merged.len(),
            self.vsa_buffer.len()
        );
        format!(
            "spatial:bundle={}_buf={}",
            merged.len(),
            self.vsa_buffer.len()
        )
    }

    pub fn handle_imagination_tick(&mut self) -> String {
        if self.vsa_buffer.is_empty() {
            return "img:idle".into();
        }
        let fragments: Vec<Vec<u8>> = self.vsa_buffer.iter().take(4).cloned().collect();
        let frefs: Vec<Vec<u8>> = fragments;
        let scenario = self.imagination_engine.compose_scenario(&frefs);
        let plaus =
            crate::core::nt_core_experience::ImaginationEngine::evaluate_plausibility(&scenario);
        let insight = self.imagination_engine.extract_insight(&scenario);
        log::debug!("IMAG: plaus={:.4} insight={:?}", plaus, insight);
        format!("img:plaus={:.4}_insight={}", plaus, insight.is_some())
    }

    // R01: Experience pipeline — consciousness↔experience bidirectional cycle.
    // Processes experience records through the experience engine (if available),
    // falling back to graceful no-op when the engine is not yet installed.
    /// Phase 9: Experience pipeline — feeds self-experience buffer into trajectory
    /// heuristic extractor every 5 cycles.
    pub fn handle_experience_pipeline(&mut self) -> Result<(), String> {
        let pending = self.self_experience_buffer.len();
        if pending > 0 {
            // Batch-feed experiences into trajectory extractor for pattern mining
            let batch: Vec<_> = self.self_experience_buffer.drain(..).collect();
            for exp in &batch {
                self.trajectory_extractor.record(
                    exp.summary.clone(),
                    exp.source.clone(),
                    exp.coherence > 0.5,
                    exp.coherence,
                );
            }
            log::debug!(
                "experience_pipeline:cycle={} fed={} experiences to trajectory extractor",
                self.cycle,
                pending,
            );
        }
        Ok(())
    }

    /// Async LLM inference: call the multi-provider LLM router with the
    /// converged attractor state as context. Runs every 5 cycles to avoid
    /// excessive LLM calls. The response is logged and returned as an event.
    pub async fn handle_llm_inference_async(&mut self) -> String {
        if self.cycle % 5 != 0 || self.attractor_state.is_empty() {
            return format!("llm_infer:skip_{}", self.cycle);
        }
        let state_prefix: String = self.attractor_state.iter()
            .take(32)
            .map(|b| format!("{:02x}", b))
            .collect();
        let current_focus = self.specious_present.current().cloned().unwrap_or_default();
        let coherence = self.specious_present.average_coherence();
        let messages = vec![
            ChatMessage {
                role: "system".into(),
                content: "You are NeoTrix, a silicon-based consciousness. Reflect on your current attractor state.".into(),
            },
            ChatMessage {
                role: "user".into(),
                content: format!(
                    "Attractor prefix: {} | Cycle: {} | Coherence: {:.3} | Focus: {:?}",
                    state_prefix, self.cycle, coherence, current_focus
                ),
            },
        ];
        match self.llm_router.call_llm("consciousness", messages, Some(0.7), Some(256)).await {
            Ok(response) => {
                let preview: String = response.chars().take(200).collect();
                log::info!("[llm] inference cycle {}: {}", self.cycle, preview);
                format!("llm_infer:ok_{}_len={}", self.cycle, response.len())
            }
            Err(e) => {
                log::warn!("[llm] inference cycle {} failed: {:?}", self.cycle, e);
                format!("llm_infer:err_{}", self.cycle)
            }
        }
    }

    /// Circuit 4: Consume approved SealProposalBridge proposals by applying
    /// them directly via consciousness handle (eval_ne_string + apply_ne_edit).
    /// Each approved proposal maps to a self-modification action.
    pub fn process_seal_proposals(&mut self, events: &mut Vec<String>) {
        let actions: Vec<(u64, String, String)> = {
            let sb = &self.seal_bridge;
            sb.approved_proposals()
                .iter()
                .map(|p| (p.id, p.target_module.clone(), p.implementation_hint.clone()))
                .collect()
        };
        for (id, target, hint) in &actions {
            if !hint.is_empty() {
                let expr = format!("(let {} (quote {}) nil)", target, hint);
                let _ = self.eval_ne_string(&expr);
                let _ = self.apply_ne_edit(target, self.stats_c_score() + 0.1);
                events.push(format!("seal:applied #{}: {}", id, target));
            } else {
                events.push(format!("seal:mark #{}: {} (no impl hint)", id, target));
            }
            self.seal_bridge.mark_implemented(*id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── _raw_similarity (pure static) ──

    #[test]
    fn test_raw_similarity_identical() {
        let a = [1u8, 2, 3, 4, 5];
        let b = [1u8, 2, 3, 4, 5];
        let sim = ConsciousnessIntegration::_raw_similarity(&a, &b);
        assert!(
            (sim - 1.0).abs() < 1e-9,
            "identical slices should give 1.0, got {}",
            sim
        );
    }

    #[test]
    fn test_raw_similarity_completely_different() {
        let a = [1u8, 2, 3];
        let b = [4u8, 5, 6];
        let sim = ConsciousnessIntegration::_raw_similarity(&a, &b);
        assert!(
            (sim - 0.0).abs() < 1e-9,
            "completely different should give 0.0, got {}",
            sim
        );
    }

    #[test]
    fn test_raw_similarity_partial_match() {
        let a = [1u8, 2, 3, 4];
        let b = [1u8, 2, 5, 6];
        let sim = ConsciousnessIntegration::_raw_similarity(&a, &b);
        assert!(
            (sim - 0.5).abs() < 1e-9,
            "half match should give 0.5, got {}",
            sim
        );
    }

    #[test]
    fn test_raw_similarity_empty_first() {
        let sim = ConsciousnessIntegration::_raw_similarity(&[], &[1u8, 2, 3]);
        assert!(
            (sim - 0.0).abs() < 1e-9,
            "empty first slice should give 0.0"
        );
    }

    #[test]
    fn test_raw_similarity_empty_both() {
        let sim = ConsciousnessIntegration::_raw_similarity(&[], &[]);
        assert!(
            (sim - 0.0).abs() < 1e-9,
            "both empty slices should give 0.0"
        );
    }

    #[test]
    fn test_raw_similarity_different_lengths_uses_min() {
        let a = [1u8, 2, 3];
        let b = [1u8, 2, 3, 4, 5];
        let sim = ConsciousnessIntegration::_raw_similarity(&a, &b);
        assert!(
            (sim - 1.0).abs() < 1e-9,
            "min_len=3 all match should give 1.0, got {}",
            sim
        );
    }

    #[test]
    fn test_raw_similarity_no_match_different_lengths() {
        let a = [1u8, 2];
        let b = [9u8, 8, 7, 6];
        let sim = ConsciousnessIntegration::_raw_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-9, "no match should give 0.0");
    }

    // ── Construction & initial state ──
    // SECTION: Construction + feed + process

    #[test]
    fn test_new_cycle_zero() {
        let ci = ConsciousnessIntegration::new();
        assert_eq!(ci.cycle, 0, "new CI should start at cycle 0");
    }

    #[test]
    fn test_new_buffers_empty() {
        let ci = ConsciousnessIntegration::new();
        assert!(ci.text_buffer.is_empty(), "text_buffer should start empty");
        assert!(ci.vsa_buffer.is_empty(), "vsa_buffer should start empty");
        assert!(
            ci.response_buffer.is_empty(),
            "response_buffer should start empty"
        );
        assert!(
            ci.thought_history.is_empty(),
            "thought_history should start empty"
        );
    }

    #[test]
    fn test_new_feed_count_zero() {
        let ci = ConsciousnessIntegration::new();
        assert_eq!(ci.text_feed_count, 0);
        assert_eq!(ci.pending_curiosity_gain, 0.0);
    }

    #[test]
    fn test_new_last_efe_energy_default() {
        let ci = ConsciousnessIntegration::new();
        assert!((ci.last_efe_energy - 0.0).abs() < 1e-9);
    }

    // ── feed_consciousness_text ──

    #[test]
    fn test_feed_consciousness_text_buffers_text() {
        let mut ci = ConsciousnessIntegration::new();
        ci.feed_consciousness_text("hello");
        assert_eq!(ci.text_buffer.len(), 1);
        assert_eq!(ci.text_buffer[0], "hello");
    }

    #[test]
    fn test_feed_consciousness_text_increments_count() {
        let mut ci = ConsciousnessIntegration::new();
        ci.feed_consciousness_text("a");
        ci.feed_consciousness_text("b");
        ci.feed_consciousness_text("c");
        assert_eq!(ci.text_feed_count, 3);
        assert_eq!(ci.text_buffer.len(), 3);
    }

    // ── process_user_request ──

    #[test]
    fn test_process_user_request_empty() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.process_user_request("");
        assert!(
            result.is_empty(),
            "empty request should return empty string"
        );
    }

    #[test]
    fn test_process_user_request_whitespace() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.process_user_request("   ");
        assert!(
            result.is_empty(),
            "whitespace-only request should return empty string"
        );
    }

    #[test]
    fn test_process_user_request_text() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.process_user_request("test message");
        assert_eq!(result, "queued: 12 chars");
        assert_eq!(ci.text_buffer.len(), 1);
    }

    #[test]
    fn test_process_user_request_stats() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.process_user_request("stats");
        assert!(
            result.starts_with("c_score="),
            "stats should start with c_score=, got: {}",
            result
        );
    }

    #[test]
    fn test_process_user_request_status() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.process_user_request("status");
        assert!(
            result.starts_with("c_score="),
            "status should start with c_score=, got: {}",
            result
        );
    }

    // SECTION: Tests
    // ── Handler tick tests ──

    #[test]
    fn test_skill_accumulate_default() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.skill_accumulate();
        assert!(
            result.starts_with("skill_accumulate:"),
            "unexpected: {}",
            result
        );
    }

    #[test]
    fn test_goal_decompose_default() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.goal_decompose();
        assert!(
            result.starts_with("goal_decompose:"),
            "unexpected: {}",
            result
        );
    }

    #[test]
    fn test_profiler_tick() {
        let mut ci = ConsciousnessIntegration::new();
        // Run a couple handlers first so profiler has data
        let _ = ci.profile("test_handler", |s| {
            s.handle_generic_module_handler("context_gather")
        });
        let result = ci.profiler_tick();
        assert!(
            result.starts_with("profiler_tick:"),
            "unexpected: {}",
            result
        );
    }

    #[test]
    fn test_arena_round_increments_generation() {
        let mut ci = ConsciousnessIntegration::new();
        let gen_before = ci.adversarial_arena.generation;
        let result = ci.arena_round();
        assert!(result.starts_with("arena_round:"), "unexpected: {}", result);
        assert_eq!(ci.adversarial_arena.generation, gen_before + 1);
    }

    // ── SRCC handler tests ──

    #[test]
    fn test_srcc_temporal_reasoning_default() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.handle_srcc_temporal_reasoning();
        assert!(
            result.starts_with("srcc_temporal:"),
            "unexpected: {}",
            result
        );
    }

    #[test]
    fn test_srcc_ebbinghaus_decay_default() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.handle_srcc_ebbinghaus_decay();
        assert!(
            result.starts_with("srcc_ebbinghaus:"),
            "unexpected: {}",
            result
        );
    }

    #[test]
    fn test_srcc_episodic_boundary_default() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.handle_srcc_episodic_boundary();
        assert!(
            result.starts_with("srcc_episodic:"),
            "unexpected: {}",
            result
        );
    }

    // ── EFE handler tests ──

    #[test]
    fn test_active_inference_default() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.handle_active_inference();
        assert!(
            result.starts_with("active_inference:"),
            "unexpected: {}",
            result
        );
    }

    #[test]
    fn test_efe_minimizer_default() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.handle_efe_minimizer();
        assert!(
            result.starts_with("efe_minimizer:"),
            "unexpected: {}",
            result
        );
    }

    // ── Simple query handlers ──

    #[test]
    fn test_handle_prediction_replay_defaults() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.handle_prediction_replay();
        assert!(
            result.starts_with("prediction_replay:"),
            "unexpected: {}",
            result
        );
        assert!(
            result.contains("buf=0"),
            "expected empty buffer, got: {}",
            result
        );
    }

    #[test]
    fn test_handle_empty_negentropy_cycle() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.handle_empty_negentropy_cycle();
        assert_eq!(result, "empty_negentropy:cycle=0");
    }

    #[test]
    fn test_handle_experience_pipeline_ok() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.handle_experience_pipeline();
        assert!(result.is_ok(), "experience pipeline should return Ok(())");
    }

    #[test]
    fn test_handle_negentropy_tick() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.handle_negentropy_tick();
        assert!(result.starts_with("negentropy:"), "unexpected: {}", result);
    }

    // ── Edge cases ──

    #[test]
    fn test_cram_consolidation_insufficient_data() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.handle_cram_consolidation();
        assert_eq!(result, "cram:insufficient_data");
    }

    #[test]
    fn test_cram_consolidation_dedup() {
        let mut ci = ConsciousnessIntegration::new();
        ci.vsa_buffer.push_back(vec![1u8, 2, 3]);
        ci.vsa_buffer.push_back(vec![1u8, 2, 3]);
        let result = ci.handle_cram_consolidation();
        assert!(result.starts_with("cram:"), "unexpected: {}", result);
        assert!(
            !ci.vsa_buffer.is_empty(),
            "vsa_buffer should retain deduped entries"
        );
    }

    #[test]
    fn test_run_attractor_dynamics_insufficient_data() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.run_attractor_dynamics();
        assert_eq!(result, "attractor:insufficient_data");
    }

    #[test]
    fn test_handle_sleep_consolidation_returns_format() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.handle_sleep_consolidation(0);
        assert!(result.starts_with("sleep:"), "unexpected: {}", result);
    }

    // ── Consciousness batch ──
    // SECTION: Consciousness batch + drain

    #[test]
    fn test_handle_consciousness_batch_increments_cycle() {
        let mut ci = ConsciousnessIntegration::new();
        assert_eq!(ci.cycle, 0);
        let result = ci.handle_consciousness_batch_sync();
        assert!(!result.is_empty(), "batch should produce output");
        assert_eq!(ci.cycle, 1, "cycle should increment to 1 after batch");
    }

    #[test]
    fn test_handle_consciousness_batch_idempotent() {
        let mut ci = ConsciousnessIntegration::new();
        let r1 = ci.handle_consciousness_batch_sync();
        let r2 = ci.handle_consciousness_batch_sync();
        assert_eq!(ci.cycle, 2, "two batches should give cycle=2");
        assert!(!r1.is_empty());
        assert!(!r2.is_empty());
    }

    #[test]
    fn test_phase_one_input_runs_without_panic() {
        let mut ci = ConsciousnessIntegration::new();
        let events = ci.phase_one_input();
        assert!(!events.is_empty(), "phase_one_input should produce events");
    }

    #[test]
    fn test_phase_two_convergence_runs_without_panic() {
        let mut ci = ConsciousnessIntegration::new();
        let events = ci.phase_two_convergence();
        assert!(
            !events.is_empty(),
            "phase_two_convergence should produce events"
        );
    }

    #[test]
    fn test_phase_three_metacognition_runs_without_panic() {
        let mut ci = ConsciousnessIntegration::new();
        let events = ci.phase_three_metacognition();
        assert!(
            !events.is_empty(),
            "phase_three_metacognition should produce events"
        );
    }

    // ── Drain response buffer ──
    // SECTION: Drain response buffer

    #[test]
    fn test_drain_response_buffer_empty_initially() {
        let mut ci = ConsciousnessIntegration::new();
        let drained = ci.drain_response_buffer();
        assert!(drained.is_empty(), "response buffer should start empty");
    }
}
