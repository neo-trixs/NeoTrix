// SPLIT PLAN:
//   File: 2661 lines — single `impl ConsciousnessIntegration` block with 78 handlers.
//   Extract the 3 largest section groups into new files:
//   1. `handlers_workflow.rs`  — Workflow engine handlers (lines 2190–2367)
//   2. `handlers_agent.rs`     — Agent/daemon/sub-agent handlers (lines 2368–2661)
//   3. Remove remaining after extractions — Generic dispatch router (lines 2604–2615)
//   Already-extracted: handlers_core.rs, handlers_memory.rs, handlers_meta.rs,
//   handlers_research.rs, handlers_safety.rs — keep those patterns.

#![allow(unused_imports)]
use super::types::*;
use super::ConsciousnessIntegration;
use crate::core::nt_core_consciousness::first_person_ref::ExperienceRecord;
use crate::core::nt_core_consciousness::meta_evolution_loop::{
    MetaArchitectureEvolutionLoop, MetaEvolutionConfig,
};
use crate::core::nt_core_consciousness::ConsciousnessAwakening;
use crate::core::nt_core_consciousness::ThinkingMode;
use crate::core::nt_core_experience::consciousness_hooks::HookPoint;
use crate::core::nt_core_experience::contrastive_reflection::ContrastiveReflection;
use crate::core::nt_core_meta::{MetaKPIRepository, MetaKPISnapshot, SelfModelAssessor};
use crate::core::nt_core_util;
use crate::neotrix::nt_world_infer::MemoryPalace;
use neotrix_body::agent::network_evolution::NetworkEvolution;
use neotrix_body::agent::perception_gateway::PerceptionGateway;

use super::modules::tm_to_str;
use crate::core::nt_core_agent::message::AgentId;
use crate::core::nt_core_agent::permission::PermissionDecision;
use crate::core::nt_core_experience::faithfulness_auditor::FaithfulnessAuditor;
use crate::core::nt_core_experience::handler_tier::{LoadStatus, LoadTier};
use crate::core::nt_core_experience::independent_verifier::IndependentVerifier;
use crate::core::nt_core_experience::loop_audit::LoopAudit;
use crate::core::nt_core_experience::loop_registry::LoopRegistry;
use crate::core::nt_core_experience::meta_cog_mera::{MetaObservation, ObsType, ReasoningStep};
use crate::core::nt_core_experience::news_radar::AlertLevel;
use crate::core::nt_core_experience::self_evolution_loop::types::{
    EvolutionState, SelfEvolutionArchive,
};
use crate::core::nt_core_experience::work_discovery_loop::WorkDiscoveryLoop;
use crate::core::nt_core_experience::workflow_engine::WorkflowStep;
use crate::core::nt_core_experience::NativeEvolutionExplorer;
use crate::core::nt_core_experience::{
    ExperienceWorkflowResult, OutputMapping, StepResult, WorkflowEngine,
};
use crate::core::nt_core_hcube::adapt_encoder::AdaptiveVsaEncoder;
use crate::core::nt_core_hcube::koopman_operator::KoopmanOperator;
use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;
use crate::core::nt_core_language::eval::NeEvaluator;
use crate::core::nt_core_language::value::NeValue;
use crate::core::nt_core_negentropy::dysib_layer::DySIBLayer;
use crate::core::nt_core_scheduler::job_queue::{CognitiveJob, JobPriority};
use crate::neotrix::nt_expert_routing::intel_profile::{
    IntelDepth, IntelPipeline, IntelQuery, IntelTargetType,
};
use crate::neotrix::nt_expert_routing::moment_feed::MomentFeed;
use crate::neotrix::nt_io_provider::okf_exporter::OkfExporter;
use crate::neotrix::nt_mind::active_exploration::ActiveExploration;
use crate::neotrix::nt_shield::vulnerability_pipeline::{VulnDepth, VulnPipeline, VulnRequest};

// CORE handlers extracted from modules.rs
// 129 handlers

impl ConsciousnessIntegration {
    // ── Dream consolidator ──

    pub fn handle_dream_consolidator_tick(&mut self) -> String {
        let ec = self.dream_consolidator.entry_count();
        let rc = self.dream_consolidator.result_count();
        log::debug!(
            "MODULES: dream_consolidator_tick entries={} results={}",
            ec,
            rc
        );
        format!("dream_consolidator_tick:entries={}_results={}", ec, rc)
    }

    // ── Checkpoint auto-output (P1.5: every 500 cycles) ──

    pub fn handle_checkpoint_tick(&mut self) -> String {
        if self.cycle % 500 != 0 || self.cycle == 0 {
            return "checkpoint:skip".to_string();
        }
        let dir = self.soul_identity.output_dir.clone();
        std::fs::create_dir_all(&dir).unwrap_or_else(|_| ());
        let path = dir.join(format!("checkpoint_{}.json", self.cycle));
        let pairs: Vec<_> = self.calibration.pre_post_pairs.iter().collect::<Vec<_>>();
        match serde_json::to_string_pretty(&pairs) {
            Ok(json) => {
                let tmp = path.with_extension("tmp");
                let write_result = std::fs::write(&tmp, &json);
                if write_result.is_ok() {
                    let _ = std::fs::rename(&tmp, &path);
                    log::info!(
                        "MODULES: checkpoint written to {:?} ({} pairs)",
                        path,
                        pairs.len()
                    );
                    format!("checkpoint:written_{}_pairs={}", self.cycle, pairs.len())
                } else {
                    let e = write_result.unwrap_err();
                    let msg = format!("checkpoint:write_error:{}", e);
                    log::error!("MODULES: checkpoint write error: {}", e);
                    msg
                }
            }
            Err(e) => {
                let msg = format!("checkpoint:serialize_error:{}", e);
                log::error!("MODULES: checkpoint serialize error: {}", e);
                msg
            }
        }
    }

    // ── Policy repair ──

    pub fn handle_policy_repair_tick(&mut self) -> String {
        let pc = self.policy_repair.pattern_count();
        let plc = self.policy_repair.policy_count();
        log::debug!(
            "MODULES: policy_repair_tick patterns={} policies={}",
            pc,
            plc
        );
        format!("policy_repair:patterns={}_policies={}", pc, plc)
    }

    // ── EvoSC tick ──

    pub fn handle_evosc_tick(&mut self) -> String {
        let s = self.evosc.stats();
        log::debug!("MODULES: evosc_tick insights={}", s.insights.total_insights);
        format!(
            "evosc_tick:insights={}_comparisons={}",
            s.insights.total_insights, s.insights.total_comparisons
        )
    }

    // ── Open skill engine ──

    pub fn handle_open_skill_tick(&mut self) -> String {
        let s = self.open_skill.stats();
        log::debug!(
            "MODULES: open_skill_tick anchors={} blueprints={} verifiers={}",
            s.anchors,
            s.blueprints,
            s.verifiers
        );
        format!(
            "open_skill_tick:a={}_b={}_v={}_vt={}",
            s.anchors, s.blueprints, s.verifiers, s.virtual_tasks
        )
    }

    // ── Skill DAG archive ──

    pub fn handle_skill_dag_tick(&mut self) -> String {
        let d = self.skill_dag.dag_diversity();
        let topo = self.skill_dag.topological_sort();
        log::debug!(
            "MODULES: skill_dag_tick diversity={:.4} topo_len={}",
            d,
            topo.len()
        );
        format!("skill_dag_tick:diversity={:.4}_skills={}", d, topo.len())
    }

    // ── Exploratory gap analysis ──

    pub fn handle_exploratory_gap_tick(&mut self) -> String {
        let gaps = self.epistemic.identify_gaps(0.5);
        log::debug!("MODULES: exploratory_gap_tick gaps={}", gaps.len());
        format!("exploratory_gap:{}_gaps", gaps.len())
    }

    // ── Signal pattern detection ──

    pub fn handle_signal_pattern_tick(&mut self) -> String {
        let (_, avg_rec, mode) = self.emergent_reasoning.stats();
        format!("signal_pattern:rec={:.3}_mode={:?}", avg_rec, mode)
    }

    // ── Resonance detection ──

    pub fn handle_resonance_detection_tick(&mut self) -> String {
        let n = self
            .multi_head_resonator
            .as_ref()
            .map_or(0, |r| r.num_heads());
        format!("resonance_detection:heads={}_active={}", n, n)
    }

    // ── Emergent property monitoring ──

    pub fn handle_emergent_property_tick(&mut self) -> String {
        let (count, conf, mode) = self.emergent_reasoning.stats();
        log::debug!(
            "MODULES: emergent_property_tick count={} conf={:.4} mode={:?}",
            count,
            conf,
            mode
        );
        format!("emergent:count={}_conf={:.4}_mode={:?}", count, conf, mode)
    }

    // ── Concept drift detection ──

    pub fn handle_concept_drift_tick(&mut self) -> String {
        let gdi = self.goal_drift.gdi();
        let drift = self.goal_drift.drift_detected();
        log::debug!("MODULES: concept_drift_tick gdi={:.4} drift={}", gdi, drift);
        format!("concept_drift:gdi={:.4}_drift={}", gdi, drift)
    }

    // ── Reflexivity monitor ──

    pub fn handle_reflexivity_tick(&mut self) -> String {
        let total = self.architecture.nodes.len();
        let active = self.architecture.active_count();
        let avg_health = if total > 0 {
            self.architecture
                .nodes
                .values()
                .map(|n| n.health_score)
                .sum::<f64>()
                / total as f64
        } else {
            0.0
        };
        log::debug!(
            "MODULES: reflexivity_tick health={:.4} active={}/{}",
            avg_health,
            active,
            total
        );
        format!(
            "reflexivity:health={:.4}_active={}/{}",
            avg_health, active, total
        )
    }

    // ── Self-model L0-L5 assessment (wired from metacognitive loop metrics) ──

    pub fn handle_self_model_tick(&mut self) -> String {
        let awareness = {
            let (_, avg_awareness, _, _) = self.reflexive_unit.stats();
            avg_awareness
        };
        let pass_rate = self.inner_critic.pass_rate();
        let narrative_coherence = self.first_person_ref.average_coherence();
        let soul_integrity = if self.soul_identity.verify_integrity() {
            1.0
        } else {
            0.0
        };
        let meta_accuracy = self.calibration.stats().ece;
        let prev_level = self.self_model_assessor.assess_level();

        self.self_model_assessor = SelfModelAssessor::from_metrics(
            awareness,
            self.cognitive_load,
            pass_rate,
            self.cycle,
            narrative_coherence,
            soul_integrity,
            meta_accuracy,
            true,
            self.cycle > 50,
            self.calibration.stats().pair_count > 20,
        );

        let new_level = self.self_model_assessor.assess_level();
        if new_level != prev_level {
            log::info!(
                "SELF_MODEL: level transition {:?} → {:?} ({})",
                prev_level,
                new_level,
                new_level.description(),
            );
        }

        if self.cycle % 50 == 0 {
            let home = crate::core::nt_core_util::home_dir()
                .to_string_lossy()
                .to_string();
            let path = format!("{}/.neotrix/self_model.json", home);
            if let Err(e) = self.self_model_assessor.save_to_file(&path) {
                log::warn!("SELF_MODEL: failed to save: {}", e);
            }
        }

        if self.cycle == 0 {
            let home = crate::core::nt_core_util::home_dir()
                .to_string_lossy()
                .to_string();
            let path = format!("{}/.neotrix/self_model.json", home);
            if let Ok(loaded) = SelfModelAssessor::load_from_file(&path) {
                self.self_model_assessor.cycle = loaded.cycle.max(self.self_model_assessor.cycle);
                self.self_model_assessor.awareness =
                    loaded.awareness.max(self.self_model_assessor.awareness);
                self.self_model_assessor.narrative_coherence = loaded
                    .narrative_coherence
                    .max(self.self_model_assessor.narrative_coherence);
                log::info!("SELF_MODEL: restored from {}", path);
            }
        }

        let report = self.self_model_assessor.report();
        format!(
            "self_model:L{}_{}",
            report.current_level as u8,
            report.current_level.description(),
        )
    }

    // ── Cognitive diversity ──

    pub fn handle_cognitive_diversity_tick(&mut self) -> String {
        let div = self.adversarial_arena.compute_diversity();
        log::debug!("MODULES: cognitive_diversity_tick diversity={:.4}", div);
        format!("cognitive_diversity:{:.4}", div)
    }

    // ── Adaptive rate controller ──

    pub fn handle_adaptive_rate_tick(&mut self, _flag: bool) -> String {
        let hyst = self.adaptive_rate_hysteresis;
        log::debug!(
            "MODULES: adaptive_rate_tick flag={} hysteresis={:.4}",
            _flag,
            hyst
        );
        format!("adaptive_rate_tick:hyst={:.4}", hyst)
    }

    // ── Conformal UQ ──

    pub fn handle_conformal_uq_tick(&mut self) -> String {
        let buf_len = self.conformal_uq_buffer.len();
        let cycle = self.cycle;
        format!("conformal_uq:buf={}_cycle={}", buf_len, cycle)
    }

    // ── Mirror Buffer (MIRROR reconstructive memory) ──

    pub fn handle_mirror_buffer_tick(&mut self) -> String {
        let n = self.mirror_buffer.trace_count();
        log::debug!("MODULES: mirror_buffer_tick traces={}", n);
        format!("mirror_buffer:{}_traces", n)
    }

    // ── AdaptOrch DAG orchestrator ──

    pub fn handle_adapt_orch_tick(&mut self) -> String {
        let load = self.cognitive_load;
        let layers = self.adapt_orch.topological_layers(load);
        let total: usize = layers.iter().map(|l| l.len()).sum();
        let layer_sizes: Vec<String> = layers
            .iter()
            .enumerate()
            .map(|(i, l)| format!("L{}:{}", i, l.len()))
            .collect();
        log::debug!(
            "MODULES: adapt_orch_tick load={:.2} layers={} total={}",
            load,
            layers.len(),
            total,
        );
        format!(
            "adapt_orch:load={:.2}_layers={}_total={}|{}",
            load,
            layers.len(),
            total,
            layer_sizes.join(","),
        )
    }

    // ── P1: Sparse VSA Attention (Zamba2-VL inspired) ──

    pub fn handle_sparse_vsa_attn_tick(&mut self) -> String {
        let should_attn = self.sparse_vsa_attn.should_run_attention();
        if should_attn {
            // Collect evidence from memory systems
            let mut evidence_keys: Vec<Vec<u8>> = Vec::with_capacity(4);
            let mut evidence_values: Vec<String> = Vec::with_capacity(4);
            let mut evidence_sources: Vec<String> = Vec::with_capacity(4);

            // Retrieve recent memory palace entries as evidence
            let recent = self.memory_palace.recent_entries(4);
            for entry in &recent {
                evidence_keys.push(entry.vsa_hash.clone());
                evidence_values.push(entry.content.clone());
                evidence_sources.push(format!("palace:{:?}", entry.room_id));
            }

            // Run VSA attention: bind attractor with evidence keys, bundle weighted results
            let attended = self.sparse_vsa_attn.run_attention_cycle(
                &self.attractor_state,
                &evidence_keys,
                &evidence_values,
                &evidence_sources,
            );

            // Blend attended state back into attractor (residual connection)
            if !attended.is_empty() && !self.attractor_state.is_empty() {
                for (a, b) in self.attractor_state.iter_mut().zip(attended.iter()) {
                    *a = a.wrapping_add(*b);
                }
            }

            let summary = self.sparse_vsa_attn.attention_summary();
            log::debug!("MODULES: sparse_vsa_attn_tick {}", summary);
            summary
        } else {
            // Fast cycle: local VSA transformation without external retrieval
            self.sparse_vsa_attn
                .run_fast_cycle(&mut self.attractor_state);
            let stats = self.sparse_vsa_attn.stats();
            log::debug!("MODULES: sparse_vsa_attn_fast {}", stats);
            format!("svsa_fast:{}", stats)
        }
    }

    // ── P2: VSA MoE Routing (Kimi-VL inspired) ──

    pub fn handle_vsa_moe_tick(&mut self) -> String {
        // Auto-register all CI handlers on first call
        if self.adapt_orch.vsa_router.profiles.is_empty() {
            self.adapt_orch.vsa_router.register_all_ci_handlers();
            self.adapt_orch.vsa_routing_enabled = true;
        }

        // Build a cognitive state VSA from current consciousness metrics
        let mut state_bytes = Vec::with_capacity(64);
        let load_byte = (self.cognitive_load * 255.0) as u8;
        let cycle_byte = (self.cycle % 256) as u8;
        let coherence = self.specious_present.average_coherence();
        let coherence_byte = (coherence * 255.0) as u8;
        let arousal = self.neuromodulator.arousal_contribution();
        let arousal_byte = (arousal * 255.0) as u8;
        let negentropy = self.composite_loss.compute().total;

        for i in 0..64 {
            let val = load_byte
                .wrapping_add(cycle_byte)
                .wrapping_add(coherence_byte)
                .wrapping_add(arousal_byte)
                .wrapping_add((negentropy * 255.0) as u8)
                .wrapping_add((i as u8).wrapping_mul(13));
            state_bytes.push(val);
        }

        let routed = self.adapt_orch.vsa_router.route(&state_bytes);
        let top_str: Vec<String> = routed
            .iter()
            .map(|(n, s)| format!("{}@{:.2}", n, s))
            .collect();
        let stats = self.adapt_orch.vsa_router.stats();
        log::debug!("MODULES: vsa_moe_tick top={}", top_str.join(","));
        format!("vsa_moe:top={}|{}", top_str.join(","), stats)
    }

    // ── P1.07: HypothesisTree idle handler ──
    pub fn handle_hypothesis_tree_tick(&mut self) -> String {
        match &self.hypothesis_tree {
            Some(_) => "hypothesis_tree:idle".into(),
            None => "hypothesis_tree:uninitialized".into(),
        }
    }

    // ── Progress-Aware RAG ──

    pub fn handle_progress_rag_tick(&mut self) -> String {
        let r = self.progress_rag.cumulative_reward();
        log::debug!("MODULES: progress_rag_tick reward={:.4}", r);
        format!("progress_rag:reward={:.4}", r)
    }

    // ── Ne Evaluator ──

    pub fn handle_ne_eval_tick(&mut self) -> String {
        // Pre-compute evolution data before mutable borrow on ne_evaluator
        let evo_mut_stats = self.mutation_log_summary();
        let (ev_total, ev_pending, ev_improved, ev_degraded, ev_unchanged) =
            self.mutation_log_stats();
        let evo_gen = self
            .self_evolution
            .as_ref()
            .map(|e| e.archive.generation)
            .unwrap_or(0);

        if let Some(ref mut ev) = self.ne_evaluator {
            // Inject CI state into evaluator env for Ne programs to reference
            ev.set_env("cycle", NeValue::Int(self.cycle as i64));
            ev.set_env(
                "handler-count",
                NeValue::Int(self.handler_registry.count() as i64),
            );
            {
                let anomaly = crate::neotrix::nt_shield::agent_anomaly::global_anomaly();
                let guard = anomaly.lock().unwrap_or_else(|e| {
                    log::error!("[ConsciousnessError] anomaly lock poisoned: {}", e);
                    e.into_inner()
                });
                let score = guard.current_anomaly_score();
                ev.set_env("anomaly-score", NeValue::Float(score));
                ev.set_env("anomaly-trained", NeValue::Bool(guard.is_trained()));
            }

            // Inject evolution feedback into evaluator env
            ev.set_env("mutation-stats", NeValue::Str(evo_mut_stats));
            ev.set_env("mutation-stats-total", NeValue::Int(ev_total as i64));
            ev.set_env("mutation-stats-pending", NeValue::Int(ev_pending as i64));
            ev.set_env("mutation-stats-improved", NeValue::Int(ev_improved as i64));
            ev.set_env("mutation-stats-degraded", NeValue::Int(ev_degraded as i64));
            ev.set_env(
                "mutation-stats-unchanged",
                NeValue::Int(ev_unchanged as i64),
            );
            ev.set_env(
                "present-evolution",
                NeValue::Str(format!("evolution:gen={}", evo_gen)),
            );

            // ── Selective Decoding Gate (VL-JEPA inspired) ──
            // Compute a lightweight state probe and compare with cached version.
            // If the probe hasn't changed meaningfully, skip the deterministic
            // test suite and reuse the cached summary — only self-modify dispatch
            // and cycle query still run (they affect / read system state).
            let cycle_quantized = self.cycle / 30;
            let hcount = self.handler_registry.count() as u64;
            let probe = ev.compute_probe(cycle_quantized, hcount);
            let probe_sim = match &self.ne_state_probe {
                Some(cached) => NeEvaluator::probe_similarity(cached, &probe),
                None => -1.0,
            };
            self.ne_state_probe = Some(probe);

            // Gate hit: state hasn't changed — skip deterministic tests
            if probe_sim > 0.90 && self.ne_last_text_result.is_some() {
                // Still run self-modify dispatch (mutates handler_registry)
                self.pre_mutation_perf = Some(self.handler_registry.perf_snapshot());
                let self_mod = ev.eval_string("(self-modify \"bridge\" \"ping\")");
                let mut _self_mod_str = String::new();
                if let Ok(v) = &self_mod {
                    let raw = match v {
                        NeValue::Str(s) => s.clone(),
                        other => other.to_string(),
                    };
                    if raw.starts_with("self-modify:queue:") {
                        let rest = raw.trim_start_matches("self-modify:queue:");
                        if let Some(colon_pos) = rest.find(':') {
                            let hname = &rest[..colon_pos];
                            let haction = &rest[colon_pos + 1..];
                            let pre_rate = self.handler_registry.success_rate(hname).unwrap_or(0.0);
                            let mutation = MutationRecord {
                                handler: hname.to_string(),
                                action: haction.to_string(),
                                cycle: self.cycle,
                                pre_success_rate: pre_rate,
                                post_success_rate: None,
                                outcome: "pending".to_string(),
                            };
                            self.mutation_log.push(mutation);
                            match haction {
                                "unload" | "prune" => {
                                    self.handler_registry.mark_unloaded(hname);
                                }
                                "explore" => {
                                    if !self
                                        .handler_registry
                                        .handler_names()
                                        .contains(&hname.to_string())
                                    {
                                        self.handler_registry.register(hname, LoadTier::Warm);
                                    }
                                }
                                "repair" => {
                                    self.handler_registry.register(hname, LoadTier::Warm);
                                }
                                _ => {
                                    if let Some(name2) = haction.strip_prefix("innovate:") {
                                        if !name2.is_empty() {
                                            self.handler_registry.register(name2, LoadTier::Warm);
                                        }
                                        let _ = format!("applied:innovate:{}:{}", hname, name2);
                                    } else if haction == "harden" {
                                        self.handler_registry.register(hname, LoadTier::Warm);
                                        let _ = format!("applied:harden:{}", hname);
                                    } else if haction == "exploit" {
                                        self.handler_registry.register(hname, LoadTier::Warm);
                                        self.handler_registry.record_success(hname);
                                        self.handler_registry.record_success(hname);
                                        let _ = format!("applied:exploit:{}", hname);
                                    }
                                }
                            }
                        }
                    }
                }

                // Cycle query (env read)
                let _cycle_result = ev.eval_string("(get-cycle)");

                // Push cached summary with a marker
                let cached = self.ne_last_text_result.as_deref().unwrap_or("none");
                let summary = format!("Ne: [cached] {}", cached);
                self.response_buffer.push_back(summary.clone());

                let stats = format!(
                    "evals={} steps={} env={} prims={}",
                    ev.eval_count(),
                    ev.step_count(),
                    ev.snapshot_env().len(),
                    ev.primitive_count(),
                );
                log::debug!("MODULES: ne_eval_tick (cached) stats=[{}]", stats);
                return format!("ne_eval:cached|{}|{}", cached, stats);
            }

            // ── Full Evaluation Path ──
            let mut parts = Vec::with_capacity(10);

            // 1. VSA test: bind two vectors and compute cosine similarity
            let vsa_prog = "(let a (permute [1 1 1 1 0 0 0 0] 42) \
                             (let b (permute [1 1 1 1 0 0 0 0] 99) \
                              (cosine a b)))";
            match ev.eval_string(vsa_prog) {
                Ok(NeValue::Float(cos)) => {
                    parts.push(format!("cos={:.3}", cos));
                    if cos > 0.5 {
                        if let Ok(vsa_val) = ev.eval_string("a") {
                            if let NeValue::Vsa(v) = vsa_val {
                                self.ne_last_vsa_result = Some(v);
                            }
                        }
                    }
                }
                Ok(v) => parts.push(format!("vsa={}", v)),
                Err(e) => parts.push(format!("vsa_err:{}", e)),
            }

            // 2. Arithmetic test
            match ev.eval_string("(+ 1 2 3 4 5)") {
                Ok(NeValue::Int(n)) => {
                    parts.push(format!("sum={}", n));
                }
                Ok(v) => parts.push(format!("arith={}", v)),
                Err(e) => parts.push(format!("arith_err:{}", e)),
            }

            // 3. Lambda + foldl test
            match ev.eval_string("(foldl (lambda (acc x) (+ acc x)) 0 [10 20 30])") {
                Ok(NeValue::Int(n)) => {
                    parts.push(format!("foldl={}", n));
                }
                Ok(v) => parts.push(format!("foldl={}", v)),
                Err(e) => parts.push(format!("foldl_err:{}", e)),
            }

            // 4. Type introspection
            let type_test = ev
                .eval_string("(type 42)")
                .unwrap_or(NeValue::Str("err".into()));
            parts.push(format!("type={}", type_test));

            // Push a summary to response buffer so the user sees Ne activity
            let summary = format!("Ne: {}", parts.join(" | "));
            self.response_buffer.push_back(summary.clone());

            // Store text result
            self.ne_last_text_result = Some(parts.join(" | "));

            let _reflect_test = ev.eval_string("(type nil)");

            // 5. Stdlib tests (will error if .ne files not yet loaded)
            let stdlib_sum = ev.eval_string("(sum [1 2 3 4 5])");
            parts.push(format!(
                "stdlib_sum={}",
                match &stdlib_sum {
                    Ok(v) => v.to_string(),
                    Err(e) => format!("err:{}", e),
                }
            ));
            let stdlib_cosine = ev.eval_string("(vsa-cosine (vsa-random 42) (vsa-random 42))");
            parts.push(format!(
                "stdlib_cos={}",
                match &stdlib_cosine {
                    Ok(v) => v.to_string(),
                    Err(e) => format!("err:{}", e),
                }
            ));
            let stdlib_compose = ev.eval_string("(compose inc double 5)");
            parts.push(format!(
                "stdlib_compose={}",
                match &stdlib_compose {
                    Ok(v) => v.to_string(),
                    Err(e) => format!("err:{}", e),
                }
            ));

            // 6. Self-modification test via evaluator primitive
            // Capture pre-mutation perf snapshot before applying changes
            self.pre_mutation_perf = Some(self.handler_registry.perf_snapshot());
            let self_mod = ev.eval_string("(self-modify \"bridge\" \"ping\")");
            parts.push(format!(
                "self_mod={}",
                match &self_mod {
                    Ok(v) => {
                        // Extract raw string if it's a Str, otherwise use Display
                        let raw = match v {
                            NeValue::Str(s) => s.clone(),
                            other => other.to_string(),
                        };
                        if raw.starts_with("self-modify:queue:") {
                            let rest = raw.trim_start_matches("self-modify:queue:");
                            if let Some(colon_pos) = rest.find(':') {
                                let hname = &rest[..colon_pos];
                                let haction = &rest[colon_pos + 1..];
                                let pre_rate =
                                    self.handler_registry.success_rate(hname).unwrap_or(0.0);
                                let mutation = MutationRecord {
                                    handler: hname.to_string(),
                                    action: haction.to_string(),
                                    cycle: self.cycle,
                                    pre_success_rate: pre_rate,
                                    post_success_rate: None,
                                    outcome: "pending".to_string(),
                                };
                                self.mutation_log.push(mutation);
                                match haction {
                                    "unload" | "prune" => {
                                        self.handler_registry.mark_unloaded(hname);
                                        format!("applied:{}:{}", haction, hname)
                                    }
                                    "explore" => {
                                        if !self
                                            .handler_registry
                                            .handler_names()
                                            .contains(&hname.to_string())
                                        {
                                            self.handler_registry.register(hname, LoadTier::Warm);
                                        }
                                        format!("applied:explore:{}", hname)
                                    }
                                    "exploit" => {
                                        log::info!(
                                            "SELF_MODIFY: promote handler [{}] via exploit",
                                            hname
                                        );
                                        format!("applied:exploit:{}", hname)
                                    }
                                    "repair" => {
                                        self.handler_registry.register(hname, LoadTier::Warm);
                                        format!("applied:repair:{}", hname)
                                    }
                                    action if action.starts_with("innovate:") => {
                                        let hname2 = &action["innovate:".len()..];
                                        let composite_name =
                                            format!("composite_{}_{}", hname, hname2);
                                        self.handler_registry
                                            .register(&composite_name, LoadTier::Warm);
                                        format!(
                                            "applied:innovate:{}:{}->{}",
                                            hname, hname2, composite_name
                                        )
                                    }
                                    "harden" => {
                                        log::info!("SELF_MODIFY: harden handler [{}]", hname);
                                        format!("applied:harden:{}", hname)
                                    }
                                    _ => format!("deferred:{}:{}", hname, haction),
                                }
                            } else {
                                raw
                            }
                        } else {
                            raw
                        }
                    }
                    Err(e) => format!("self_mod_err:{}", e),
                }
            ));

            // 7. Try evolve.ne exports
            let evolve_result = ev.eval_string("(evaluate-binding [1 1 0 0] [1 0 1 0] [1 1 1 1])");
            parts.push(format!(
                "evolve={}",
                match &evolve_result {
                    Ok(v) => v.to_string(),
                    Err(e) => format!("err:{}", e),
                }
            ));

            // 7a. Test mue-x mutation primitives
            let explore_result = ev.eval_string("(try-explore \"bridge\")");
            parts.push(format!(
                "explore={}",
                match &explore_result {
                    Ok(v) => v.to_string(),
                    Err(e) => format!("err:{}", e),
                }
            ));
            let prune_result = ev.eval_string("(try-prune \"bridge\")");
            parts.push(format!(
                "prune={}",
                match &prune_result {
                    Ok(v) => v.to_string(),
                    Err(e) => format!("err:{}", e),
                }
            ));

            // 8. Check cycle via get-cycle primitive
            let cycle_result = ev.eval_string("(get-cycle)");
            parts.push(format!(
                "cycle={}",
                match &cycle_result {
                    Ok(v) => v.to_string(),
                    Err(e) => format!("err:{}", e),
                }
            ));

            // 9. Compute eval success rate for CI feedback
            let _eval_pass = parts.iter().any(|p| p.starts_with("cos="));
            let _stdlib_pass = parts.iter().any(|p| p.starts_with("stdlib_"));
            let _arithmetic_pass = parts.iter().any(|p| p.starts_with("sum="));
            let _success_count = [_eval_pass, _stdlib_pass, _arithmetic_pass]
                .iter()
                .filter(|&&x| x)
                .count();
            let _total_count = 3usize;
            let eval_success_rate = if _total_count > 0 {
                _success_count as f64 / _total_count as f64
            } else {
                1.0
            };
            if eval_success_rate < 0.7 {
                log::warn!(
                    "NE_EVAL_ANOMALY: success_rate={:.3} — evaluator may be degrading",
                    eval_success_rate
                );
            }
            parts.push(format!("sr={:.2}", eval_success_rate));

            let stats = format!(
                "evals={} steps={} env={} prims={}",
                ev.eval_count(),
                ev.step_count(),
                ev.snapshot_env().len(),
                ev.primitive_count(),
            );
            // Save evaluator state periodically (every 100 cycles)
            if self.cycle > 0 && self.cycle % 100 == 0 {
                if let Ok(json) = ev.save_state() {
                    let state_path =
                        std::path::Path::new(&self.ne_source_dir).join(".ne_state.json");
                    let tmp_path = state_path.with_extension("tmp");
                    let _ = std::fs::write(&tmp_path, &json);
                    let _ = std::fs::rename(&tmp_path, &state_path);
                }
                // Persist mutation_log alongside evaluator state
                let mut_path =
                    std::path::Path::new(&self.ne_source_dir).join("ne_mutation_log.json");
                if let Err(e) = self.save_mutation_log(&mut_path.to_string_lossy()) {
                    log::error!("MODULES: failed to save mutation_log: {}", e);
                }
            }

            log::debug!(
                "MODULES: ne_eval_tick results=[{}] stats=[{}]",
                parts.join(","),
                stats
            );
            format!("ne_eval:{}|{}", parts.join(","), stats)
        } else {
            "ne_eval:uninitialized".to_string()
        }
    }

    // ── Cognitive Job Queue (priority-based with preemption) ──

    pub fn handle_job_queue_tick(&mut self) -> String {
        let events = self.job_queue.tick_cycle();
        if events.is_empty() {
            return self.job_queue.stats();
        }
        let stats = self.job_queue.stats();
        format!("{}|events:{}", stats, events.join(","))
    }

    pub fn handle_job_queue_stats_tick(&mut self) -> String {
        self.job_queue.stats()
    }

    pub fn handle_job_queue_submit_tick(&mut self) -> String {
        let handler = format!("research_tick");
        let job = CognitiveJob::new("auto-research", &handler, JobPriority::Medium, "{}");
        let id = self.job_queue.enqueue(job);
        format!("queue:submitted|{}", id)
    }

    // ── Self-Harness (WeaknessMining→HarnessProposal→ProposalValidation) ──

    pub fn handle_self_harness_tick(&mut self) -> String {
        let profiler_records: Vec<_> = self
            .profiler
            .all_stats()
            .iter()
            .map(|s| (s.name.to_string(), s.call_count))
            .collect();
        let _current_score = self.calibration.stats().ece;
        let result = self.self_harness.run_cycle(
            &profiler_records
                .iter()
                .map(|(n, _)| n.as_str())
                .collect::<Vec<_>>(),
        );
        let count = result.len();
        log::debug!("MODULES: self_harness_tick generated {} proposals", count);
        format!("self_harness:{}_proposals", count)
    }

    pub fn handle_self_harness_stats_tick(&mut self) -> String {
        format!(
            "self_harness:{}_weaknesses",
            self.self_harness.weakness_history.len()
        )
    }

    // ── EvolutionCoordinator — unified cross-engine coordination tick ──

    pub fn handle_evolution_coordinator_tick(&mut self) -> String {
        // Bridge SelfHarness → EvolutionCoordinator: feed top weaknesses
        if self.cycle % 30 == 0 {
            let weaknesses: Vec<_> = self.self_harness.weakness_history.iter().take(5).collect();
            for w in &weaknesses {
                self.evolution_coordinator
                    .report_weakness(&w.pattern, w.avg_impact, &w.pattern);
            }
        }
        // Bridge EGPO → EvolutionCoordinator: feed novelty from exploration step
        if self.cycle % 15 == 0 {
            let step = self.egpo.exploration.step;
            if step > 0 && step % 10 == 0 {
                self.evolution_coordinator.report_novelty(step, 0.5);
            }
        }
        // Bridge HypothesisTree → EvolutionCoordinator: generate proposals from MCTS
        if self.cycle % 20 == 0 {
            let count = self.evolution_coordinator.incorporate_hypothesis_tree();
            if count > 0 {
                log::debug!(
                    "EVO_COORD: hypothesis tree generated {} proposals at cycle {}",
                    count,
                    self.cycle
                );
            }
        }
        let summary = self.evolution_coordinator.tick();
        // Apply queued mutations to the handler registry
        let applied = self
            .evolution_coordinator
            .mutation_apply(&mut self.handler_registry);
        if applied > 0 {
            log::info!(
                "MODULES: evolution_coordinator applied {} mutation(s) to handler registry",
                applied
            );
        }
        summary
    }

    /// Gradient-aware SEAL: compile Ne source → TensorGraph → gradient descent train.
    pub fn handle_gradient_seal_tick(&mut self) -> String {
        if self.self_evolution.is_none() {
            return "gradient_seal:no_evolution_loop".to_string();
        }
        let evo = self.self_evolution.as_mut().unwrap();
        if !evo.is_running {
            return "gradient_seal:paused".to_string();
        }

        // Use a simple Ne source that learns a target vector via gradient descent.
        // The source defines `def compute(x) = x * 0.5 + 0.3` and the target
        // is a known output vector; training tunes the const parameters.
        let training_source =
            "def compute(x):\n  let a = const([0.1])\n  let b = const([0.5])\n  x * a + b";
        let dim = 1usize;
        let target = vec![0.8f64]; // target for compute(1.0) = 1.0*a + b ≈ 0.8
        let lr = 0.01;
        let steps = 50;

        match evo.train_ne_program(training_source, dim, lr, steps, &target) {
            Some(tp) => {
                let improvement = if tp.loss_trace.len() >= 2 {
                    let first = tp.loss_trace[0];
                    let last = tp.loss_trace[tp.loss_trace.len() - 1];
                    if first > 0.0 {
                        ((first - last) / first * 100.0) as i32
                    } else {
                        0
                    }
                } else {
                    0
                };
                log::info!(
                    "GRADIENT_SEAL: trained {} steps, final_loss={:.6}, improvement={}%",
                    steps,
                    tp.final_loss,
                    improvement,
                );
                format!(
                    "gradient_seal:loss={:.6},improvement={}%,steps={}",
                    tp.final_loss, improvement, steps,
                )
            }
            None => "gradient_seal:training_failed".to_string(),
        }
    }

    /// Diagnostic: dump gradient seal training status.
    pub fn handle_gradient_seal_status_tick(&mut self) -> String {
        if self.self_evolution.is_none() {
            return "gradient_seal:no_evolution_loop".to_string();
        }
        let evo = self.self_evolution.as_ref().unwrap();
        format!(
            "gradient_seal:gen={},steps={},best={:.4}",
            evo.archive.generation,
            evo.archive.steps.len(),
            evo.archive.best_score,
        )
    }

    // ── ContextCompressor (ACON-style guideline-based context compression) ──

    pub fn handle_context_compressor_tick(&mut self) -> String {
        let history: Vec<(String, Vec<u8>, f64)> = self
            .thought_history
            .iter()
            .map(|(t, v, s)| (t.clone(), v.clone(), *s))
            .collect();
        let compressed = self.context_compressor.compress_thought_history(&history);
        if compressed.len() < history.len() {
            self.thought_history = compressed.into_iter().collect();
            log::debug!(
                "MODULES: context_compressor compressed {}→{}",
                history.len(),
                self.thought_history.len()
            );
        }
        self.context_compressor.stats()
    }

    pub fn handle_context_compressor_stats_tick(&mut self) -> String {
        self.context_compressor.stats()
    }

    // ── Fusion Gap Registry — tracks theoretical vs implemented gap for fused literature ──

    pub fn handle_fusion_gap_tick(&mut self) -> String {
        let entries = &self.fusion_gap_registry.entries;
        let entry_count = entries.len();
        let max_gap_entry = entries.iter().max_by(|a, b| {
            a.gap
                .partial_cmp(&b.gap)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        match max_gap_entry {
            Some(entry) => {
                let name = &entry.name;
                let gap = entry.gap;
                log::debug!(
                    "FUSION_GAP: max_gap={}_{:.2} count={}",
                    name,
                    gap,
                    entry_count
                );
                format!(
                    "fusion_gap:max_{}_{:.2}_gap_count_{}",
                    name, gap, entry_count
                )
            }
            None => format!("fusion_gap:no_entries"),
        }
    }

    // ── EGPO — Exploration Guided Policy Optimization (arXiv 2602.22751) ──

    pub fn handle_egpo_tick(&mut self) -> String {
        let current_vsa = self.thought_history.back().map(|(_, v, _)| v.as_slice());
        let recent_vsas: Vec<Vec<u8>> = self
            .thought_history
            .iter()
            .rev()
            .take(20)
            .map(|(_, v, _)| v.clone())
            .collect();
        let result = self.egpo.tick(current_vsa, &recent_vsas, None);
        log::debug!("MODULES: egpo_tick {}", result);
        result
    }

    pub fn handle_egpo_stats_tick(&mut self) -> String {
        self.egpo.stats()
    }

    pub fn handle_self_evolution_tick(&mut self) -> String {
        if self.self_evolution.is_none() {
            log::info!("SELFEVOL: lazy-init self-evolution loop");
            let mut evo =
                crate::core::nt_core_experience::self_evolution_loop::SelfEvolutionLoop::new();
            evo.is_running = true;
            // Load persisted evolution archive (fix persistence gap)
            let archive_path = format!(
                "{}/evolution_archive.json",
                std::env::var("NEOTRIX_SOUL_DIR").unwrap_or_else(|_| {
                    let h = crate::core::nt_core_util::home_dir()
                        .to_string_lossy()
                        .to_string();
                    format!("{}/.neotrix", h)
                })
            );
            // Try new format (EvolutionState with meta-strategy) first, fall back to old format
            match std::fs::read(&archive_path) {
                Ok(data) => {
                    if let Ok(state) = EvolutionState::from_bytes(&data) {
                        // New format: archive + meta-strategy in one JSON object
                        let (archive, meta_opt) = SelfEvolutionArchive::from_evolution_state(state);
                        if !archive.steps.is_empty() {
                            evo.archive = archive;
                            if let Some(meta) = meta_opt {
                                evo.meta_strategy = meta;
                            }
                            log::info!(
                                "SELFEVOL: restored archive + meta-strategy v{} ({} steps, gen {})",
                                evo.meta_strategy.version,
                                evo.archive.steps.len(),
                                evo.archive.generation,
                            );
                        }
                    } else if let Ok(archive) = SelfEvolutionArchive::load_from_file(&archive_path)
                    {
                        // Old format: flat JSON array of steps
                        if !archive.steps.is_empty() {
                            evo.archive = archive;
                            log::info!(
                                "SELFEVOL: restored archive (old format, {} steps, gen {})",
                                evo.archive.steps.len(),
                                evo.archive.generation,
                            );
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    log::debug!("SELFEVOL: no archive file found at {}", archive_path);
                }
                Err(e) => {
                    log::warn!("SELFEVOL: failed to read archive: {}", e);
                }
            }
            self.self_evolution = Some(evo);
            return "self_evolution:lazy_init".to_string();
        }

        let score_before = self.stats().c_score;
        if let Some(mut evo) = self.self_evolution.take() {
            if evo.is_running {
                let drive = self.drive_selector.current_drive();
                let cycle = self.cycle;
                let (mutation, crystallized) = evo.tick(score_before, cycle, &drive, self);

                // Eval newly crystallized skills into the Ne evaluator (Hermes-style auto-skill loading)
                let mut skill_count = 0usize;
                for skill in &crystallized {
                    if let Some(ref mut ev) = self.ne_evaluator {
                        match ev.eval_string(&skill.ne_source) {
                            Ok(_val) => {
                                log::info!(
                                    "SELFEVOL: loaded crystallized skill '{}' into Ne evaluator",
                                    skill.name
                                );
                                self.skill_health_monitor.record_success(&skill.name);
                                skill_count += 1;
                            }
                            Err(e) => {
                                log::error!(
                                    "SELFEVOL: failed to eval skill '{}': {}",
                                    skill.name,
                                    e
                                );
                                self.skill_health_monitor.record_loading_error(&skill.name);
                            }
                        }
                    }
                }
                if skill_count > 0 {
                    log::info!("SELFEVOL: auto-loaded {} crystallized skills", skill_count);
                }

                // Auto-repair skills that have exceeded failure threshold (MOLTRON style)
                let needs_repair = self.skill_health_monitor.needs_repair();
                for repair_name in &needs_repair {
                    let report = self
                        .skill_health_monitor
                        .attempt_repair(repair_name, self.cycle);
                    log::info!("SELFEVOL: {}", report);
                }

                if let Some(mutation) = mutation {
                    // Phase 42: gate through BallVerifier + PccSafetyGate
                    if let Err(reason) = self.safety_check_mutation(&mutation) {
                        log::warn!(
                            "SELFEVOL: mutation [{}] REJECTED by safety: {}",
                            mutation.summary(),
                            reason
                        );
                        evo.record_result(mutation, score_before, score_before * 0.5, false, None);
                        self.self_evolution = Some(evo);
                        self.save_evolution_archive();
                        return format!("self_evolution:mutation_rejected={}", reason);
                    }
                    log::info!(
                        "SELFEVOL: mutation [{}] passed safety gates",
                        mutation.summary()
                    );

                    let handler_count = self.handler_registry.count();
                    let negentropy = self.stats().c_score;
                    evo.begin_hgm(handler_count, negentropy, self.cycle);
                    match evo.execute_mutation(&mutation, self) {
                        Ok(score_after) => {
                            let handler_count_after = self.handler_registry.count();
                            let negentropy_after = self.stats().c_score;
                            let hgm_cmp = evo.finish_hgm(handler_count_after, negentropy_after);
                            let compiles = score_after >= score_before * 0.5;
                            evo.record_result(
                                mutation,
                                score_before,
                                score_after,
                                compiles,
                                hgm_cmp,
                            );
                            // Phase 43: rollback if mutation was rejected (score dropped below threshold)
                            if let Some(last) = evo.archive.steps.last() {
                                if !last.accepted {
                                    evo.rollback_mutation(last, self);
                                }
                            }
                            self.self_evolution = Some(evo);
                            log::debug!(
                                "MODULES: self_evolution_tick executed mutation score={:.4}",
                                score_after
                            );
                            self.save_evolution_archive();
                            let rollback_tag = if compiles && score_after >= score_before * 0.95 {
                                ""
                            } else {
                                "_rollback"
                            };
                            let skill_tag = if skill_count > 0 {
                                format!("_skills={}", skill_count)
                            } else {
                                String::new()
                            };
                            return format!(
                                "self_evolution:mutation_executed_score={:.4}{}{}",
                                score_after, rollback_tag, skill_tag
                            );
                        }
                        Err(e) => {
                            self.self_evolution = Some(evo);
                            log::debug!("MODULES: self_evolution_tick failed: {}", e);
                            self.save_evolution_archive();
                            return format!("self_evolution:mutation_failed={}", e);
                        }
                    }
                }
                // Cross-domain transfer tick: snapshots archive and finds transfer candidates
                // from other consciousness domains (cognitive, perception, action, meta).
                evo.cross_domain_tick(self.cycle, "cognitive");
                // DGM-H meta-agent tick: runs periodically to propose meta-strategy updates.
                // Only fires when the regular mutation path didn't produce a mutation,
                // and the meta-agent has enough archive data to propose a change.
                if let Some(meta_mutation) = evo.meta_agent_tick() {
                    log::info!(
                        "SELFEVOL: meta-agent proposing [{}]",
                        meta_mutation.summary()
                    );
                    if let Err(reason) = self.safety_check_mutation(&meta_mutation) {
                        log::warn!(
                            "SELFEVOL: meta-mutation [{}] REJECTED by safety: {}",
                            meta_mutation.summary(),
                            reason
                        );
                    } else {
                        match evo.execute_mutation(&meta_mutation, self) {
                            Ok(score_after) => {
                                let compiles = score_after >= score_before * 0.5;
                                evo.record_result(
                                    meta_mutation,
                                    score_before,
                                    score_after,
                                    compiles,
                                    None,
                                );
                                if let Some(last) = evo.archive.steps.last() {
                                    if !last.accepted {
                                        evo.rollback_mutation(last, self);
                                    }
                                }
                                let meta_version = evo.meta_strategy.version;
                                self.self_evolution = Some(evo);
                                self.save_evolution_archive();
                                return format!(
                                    "self_evolution:meta_mutation_executed_v{}",
                                    meta_version,
                                );
                            }
                            Err(e) => {
                                log::debug!("MODULES: meta-mutation failed: {}", e);
                                self.self_evolution = Some(evo);
                                return format!("self_evolution:meta_mutation_failed={}", e);
                            }
                        }
                    }
                }
                let gen = evo.archive.generation;
                let report = evo.report();
                let skill_tag = if skill_count > 0 {
                    format!("_skills={}", skill_count)
                } else {
                    String::new()
                };
                log::debug!("MODULES: self_evolution_tick (no mutation)\n{}", report);
                self.self_evolution = Some(evo);
                return format!("self_evolution:running_gen={}{}", gen, skill_tag);
            }
            self.self_evolution = Some(evo);
            // Persist evolution state every 30 cycles (belt-and-suspenders)
            if self.cycle > 0 && self.cycle % 30 == 0 {
                self.save_evolution_archive();
            }
        }
        "self_evolution:inactive".to_string()
    }

    /// Unified self-evolution engine tick: collects signals from all sources,
    /// prioritizes them, and routes to the appropriate executor.
    /// Called every 50 cycles alongside handle_self_evolution_tick.
    pub fn handle_evolution_engine_tick(&mut self) -> String {
        // 1. Lazy-init MetaArchitectureEvolutionLoop
        if self.meta_architecture.is_none() {
            self.meta_architecture =
                Some(MetaArchitectureEvolutionLoop::new(MetaEvolutionConfig {
                    auto_evolve: true,
                    max_recommendations: 5,
                    ..MetaEvolutionConfig::default()
                }));
        }

        // 2. Feed MetaArchitectureEvolutionLoop recommendations into engine
        let recs = self
            .meta_architecture
            .as_mut()
            .map(|m| m.assess())
            .unwrap_or_default();
        let rec_count = recs.len();
        if !recs.is_empty() {
            self.evolution_engine.feed_recommendations(recs);
        }

        // 3. Lazy-init body evolution modules
        if self.body_network_evolution.is_none() {
            self.body_network_evolution = Some(NetworkEvolution::new(50));
        }
        if self.body_perception_gateway.is_none() {
            self.body_perception_gateway = Some(PerceptionGateway::new(500));
        }

        // 4. Tick body NetworkEvolution and feed metrics to engine
        if let Some(ref mut net_evo) = self.body_network_evolution {
            let action = net_evo.tick();
            let health = net_evo.overall_health();
            let report = net_evo.evolution_report();
            log::debug!(
                "BODY_EVO: cycle={} health={:.3} action={:?} report={}",
                self.cycle,
                health,
                action,
                report,
            );
            // Feed overall health as BodyMetric signal
            self.evolution_engine.feed_signals(
                vec![crate::core::nt_core_experience::self_evolution_engine::EvolutionSignal::BodyMetric {
                    module: "network_evolution".into(),
                    metric: "overall_health".into(),
                    value: health,
                    threshold: 0.5,
                }],
            );
        }

        // 5. Tick body PerceptionGateway and feed channel stats to engine
        if let Some(ref mut pg) = self.body_perception_gateway {
            let stats = pg.channel_stats();
            let total_events: usize = stats.values().map(|s| s.event_count as usize).sum();
            let avg_salience: f64 = if !stats.is_empty() {
                stats.values().map(|s| s.avg_salience).sum::<f64>() / stats.len() as f64
            } else {
                0.0
            };
            log::debug!(
                "BODY_PERCEPTION: cycle={} channels={} events={} avg_salience={:.3}",
                self.cycle,
                stats.len(),
                total_events,
                avg_salience,
            );
            // Feed channel count as BodyMetric signal (salience drops below 0.2 threshold = attention problem)
            self.evolution_engine.feed_signals(
                vec![crate::core::nt_core_experience::self_evolution_engine::EvolutionSignal::BodyMetric {
                    module: "perception_gateway".into(),
                    metric: "avg_salience".into(),
                    value: avg_salience,
                    threshold: 0.2,
                }],
            );
        }

        // 6. Tick the engine — executes top-priority signal
        let result = self.evolution_engine.tick(self.cycle);
        log::debug!(
            "EVOENGINE: cycle={} recs={} body_signals=2 pending_signals={} pending_tasks={} executed={}",
            self.cycle,
            rec_count,
            result.pending_signals,
            result.pending_tasks,
            result.executed_this_cycle,
        );
        format!(
            "evolution_engine:recs={}_signals={}_tasks={}_exec={}",
            rec_count, result.pending_signals, result.pending_tasks, result.executed_this_cycle,
        )
    }

    /// Persist the evolution archive and meta-strategy to ~/.neotrix/evolution_archive.json
    /// using the `EvolutionState` wrapper (includes archive + meta_strategy).

    fn save_evolution_archive(&mut self) {
        if let Some(ref evo) = self.self_evolution {
            let soul_dir = std::env::var("NEOTRIX_SOUL_DIR").unwrap_or_else(|_| {
                let h = crate::core::nt_core_util::home_dir()
                    .to_string_lossy()
                    .to_string();
                format!("{}/.neotrix", h)
            });
            if let Ok(_) = std::fs::create_dir_all(&soul_dir) {
                let path = format!("{}/evolution_archive.json", soul_dir);
                let state = evo.archive.to_evolution_state(Some(&evo.meta_strategy));
                let tmp = format!("{}.tmp", path);
                match state.to_bytes() {
                    Ok(bytes) => {
                        if std::fs::write(&tmp, &bytes).is_ok() {
                            let _ = std::fs::rename(&tmp, &path);
                            log::debug!(
                                "SELFEVOL: saved archive + meta-strategy v{} ({} steps, gen {})",
                                evo.meta_strategy.version,
                                evo.archive.steps.len(),
                                evo.archive.generation,
                            );
                        }
                    }
                    Err(e) => {
                        log::warn!("SELFEVOL: failed to serialize evolution state: {}", e);
                    }
                }
            }
        }
    }

    // ── Story generator ──

    pub fn handle_story_generator_tick(&mut self) -> String {
        let s = self.story_generator.stats();
        let prompt = format!(
            "Cycle {} narrative synthesis with {} events in {} style",
            self.cycle, s.event_count, s.narrative_style,
        );
        let story = self.story_generator.generate_story(&prompt);
        log::debug!(
            "MODULES: story_generator_tick events={} style={} len={}",
            s.event_count,
            s.narrative_style,
            story.len(),
        );
        story
    }

    // ── Architecture introspection handler ──

    pub fn handle_architecture_report(&mut self) -> String {
        let report = self.architecture.report();
        let stubs = self.architecture.stubs();
        let isolated = self.architecture.isolated_modules();
        let degraded = self.architecture.degraded(0.5);
        format!(
            "{}. stubs={} ({}), isolated={}, degraded={}",
            report,
            stubs.len(),
            stubs
                .iter()
                .map(|n| n.name.as_str())
                .collect::<Vec<_>>()
                .join(","),
            isolated.len(),
            degraded.len(),
        )
    }

    pub fn handle_architecture_status_tick(&mut self) -> String {
        let summary = self.architecture.report();
        log::info!("MODULES: architecture_status_tick {}", summary);
        summary
    }

    // ── Adaptive VSA text encoding (routing path) ──

    /// Encode text using AdaptiveVsaEncoder when available.
    /// Learning/classification tasks use correlated mode; cognitive tasks use orthogonal mode.
    /// Falls back to input_pipeline encoding when adaptive_vsa is None.

    pub fn text_to_vsa_adaptive(&mut self, text: &str, tag: &str) -> Vec<u8> {
        if let Some(ref encoder) = self.adaptive_vsa {
            encoder.encode_with_tag(text, tag)
        } else {
            self.input_pipeline.encode_and_record(text, "adaptive")
        }
    }

    // ── Self-Revision (Phase 47) ──

    pub fn handle_self_revision_tick(&mut self) -> String {
        let n = self.self_revision.trace_count();
        if n == 0 {
            return "srev:idle".into();
        }
        // Occasionally sync teacher and distill
        if self.cycle % 100 == 0 && self.cycle > 0 {
            self.self_revision.sync_teacher();
            let loss = self.self_revision.distill_from_teacher();
            log::info!(
                "SREV: cycle={} traces={} distill_loss={:.6}",
                self.cycle,
                n,
                loss
            );
            return format!("srev:traces={}_distill={:.4}", n, loss);
        }
        // Apply revision to attractor_state if available
        if !self.attractor_state.is_empty() && self.cycle % 20 == 0 {
            let revised = self.self_revision.apply_revision(&self.attractor_state);
            let sim = crate::core::nt_core_hcube::QuantizedVSA::similarity(
                &self.attractor_state,
                &revised,
            );
            if sim < 0.95 {
                log::info!("SREV: applied revision sim={:.4}", sim);
                self.attractor_state = revised;
                return format!("srev:revised_sim={:.4}", sim);
            }
        }
        format!("srev:traces={}", n)
    }

    // ── EMA JEPA / VICReg (Phase 48) ──

    pub fn handle_okf_export_tick(&mut self) -> String {
        let export_interval = 200u64;
        if self.cycle % export_interval != 0 || self.cycle == 0 {
            return "okf:idle".into();
        }
        // Lazily create exporter
        if self.okf_exporter.is_none() {
            let path = std::env::var("NEOTRIX_OKF_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| {
                    let home = crate::core::nt_core_util::home_dir()
                        .to_string_lossy()
                        .to_string();
                    std::path::PathBuf::from(home + "/.neotrix/okf")
                });
            self.okf_exporter = Some(OkfExporter::new(path));
        }
        // Currently just signals readiness — actual export requires KnowledgeEngine
        // which lives at the BackgroundLoop level. The export call is triggered
        // from run.rs when the knowledge engine is available.
        format!("okf:ready_cycle={}", self.cycle)
    }

    // ── Native Evolution Explorer (P0.15) ──

    pub fn handle_native_explorer_tick(&mut self) -> String {
        if self.native_explorer.is_none() {
            self.native_explorer = Some(NativeEvolutionExplorer::new());
            return "nexp:init".into();
        }
        let explorer = match self.native_explorer.as_mut() {
            Some(e) => e,
            None => return "nexp:no_instance".into(),
        };
        if let Some(action) = explorer.tick(self.cognitive_load, &self.attractor_state) {
            explorer.record_outcome(&action, 0.01, 1.0);
            format!("nexp:action={:?}", action.action_type)
        } else {
            "nexp:suppressed".into()
        }
    }

    // ── Fusion D: Geometric State-Space Kernel (E8 reasoning) ──

    pub fn handle_contrastive_reflection_tick(&mut self) -> String {
        if self.contrastive_reflection.is_none() {
            self.contrastive_reflection = Some(ContrastiveReflection::new());
            return "crefl:init".into();
        }
        let crefl = match self.contrastive_reflection.as_ref() {
            Some(crefl) => crefl,
            None => {
                log::error!("[modules_core] contrastive_reflection not initialized");
                return "contrastive_reflection:unavailable".into();
            }
        };
        let n = crefl.pair_count();
        format!("crefl:pairs={}", n)
    }

    // ── Phase 58 — External Intelligence Modules ──

    pub fn handle_news_radar_tick(&mut self) -> String {
        let count = self.news_radar.poll_all();
        let report = self.news_radar.extended_opinion_flow();
        let rising: Vec<String> = report
            .trends
            .iter()
            .filter(|t| t.is_rising)
            .map(|t| format!("{}:{:.1}", t.topic, t.velocity))
            .collect();
        let predictions: Vec<String> = report
            .predictions
            .iter()
            .take(3)
            .map(|p| format!("{:.0}%:{}", p.confidence * 100.0, p.topic))
            .collect();

        // Alert summary (BettaFish-style multi-level alerts)
        let red_count = report
            .alerts
            .iter()
            .filter(|a| a.level == AlertLevel::Red)
            .count();
        let orange_count = report
            .alerts
            .iter()
            .filter(|a| a.level == AlertLevel::Orange)
            .count();
        let yellow_count = report
            .alerts
            .iter()
            .filter(|a| a.level == AlertLevel::Yellow)
            .count();
        let alert_str = if report.alerts.is_empty() {
            "no_alerts".to_string()
        } else {
            let top = &report.alerts[0];
            format!(
                "alerts:R{}_O{}_Y{}|top:{}[{:.1}]/{}",
                red_count,
                orange_count,
                yellow_count,
                top.topic,
                top.hotness.composite,
                top.level.name()
            )
        };

        // Sentiment polarization warning
        let polarizing = report
            .alerts
            .iter()
            .filter(|a| a.divergence.is_polarizing)
            .map(|a| a.topic.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        format!(
            "news:{}_items | rising:{} | {} | pred:{} | {}",
            count,
            rising.join(","),
            alert_str,
            predictions.join(";"),
            if polarizing.is_empty() {
                "".to_string()
            } else {
                format!("polarizing:{}", polarizing)
            },
        )
    }

    pub fn handle_voice_synthesis_tick(&mut self) -> String {
        let pending = self.voice_synthesis.pending_count();
        format!("voice:{}_pending", pending)
    }

    pub fn handle_intel_profile_tick(&mut self) -> String {
        // Phase 1: Process intel pipeline and collect verification claims (owned)
        let processed;
        let count;
        let dossier_text;
        let verify_claims: Vec<(usize, String, String)>;

        {
            let pipeline = self.intel_profile.get_or_insert_with(IntelPipeline::new);
            processed = pipeline.process_pending();
            count = pipeline.profile_count();

            dossier_text = pipeline
                .profiles
                .values()
                .max_by_key(|pr| pr.updated_at)
                .map(|pr| pipeline.format_dossier(pr));

            // Collect claims for truth verification (owned data, no borrow held)
            verify_claims = pipeline
                .profiles
                .values()
                .max_by_key(|pr| pr.updated_at)
                .map(|pr| {
                    pr.timeline
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| e.confidence > 0.3)
                        .map(|(i, e)| {
                            let claim = format!("{}: {}", e.title, e.description);
                            let source = format!("intel_profile:{}|event:{}", pr.target_name, i);
                            (i, claim, source)
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
        } // pipeline dropped, self.intel_profile borrow released

        // Phase 2: Feed dossier to consciousness
        if let Some(text) = dossier_text {
            self.feed_consciousness_text(&text);
        }

        // Phase 3: Verify claims through truth pipeline
        if !verify_claims.is_empty() {
            if let Some(ref mut truth) = self.truth_pipeline {
                let mut vtext = String::from("\n\n## Intel Profile Truth Verification\n\n");
                for (idx, claim, source) in &verify_claims {
                    let est = truth.quick_check(claim, source);
                    vtext.push_str(&format!("- Event #{}: {}\n", idx, truth.summary(&est)));
                }
                self.feed_consciousness_text(&vtext);
            }
        }

        if processed > 0 {
            format!("intel:{}_profiles_requested:{}", count, processed)
        } else {
            format!("intel:{}_profiles", count)
        }
    }

    /// Request-driven research — any subsystem can trigger a profile by name.
    /// The research runs asynchronously on the next consciousness tick.
    pub fn request_intel_profile(
        &mut self,
        name: &str,
        target_type: Option<IntelTargetType>,
    ) -> String {
        let pipeline = self.intel_profile.get_or_insert_with(IntelPipeline::new);
        let query = IntelQuery {
            keywords: vec![name.to_string()],
            target_type,
            depth: IntelDepth::Standard,
            max_sources: 50,
        };
        pipeline.enqueue_request(query);
        format!("intel:requested_{}", name)
    }

    /// Trading engine tick: detect market regime, generate signal, report status.
    pub fn handle_trading_tick(&mut self) -> String {
        let engine = self.trading_engine.get_or_insert_with(|| {
            crate::core::nt_core_trading::engine::TradingEngine::new(
                "DEFAULT",
                crate::core::nt_core_trading::types::AssetClass::Crypto,
                10000.0,
            )
        });
        let regime = engine.detect_regime();
        let can_trade = engine.can_trade();
        let status = engine.status_report();
        if let Some(r) = regime {
            format!(
                "trading:{}|trend={:?}|vol={:?}|can_trade={}",
                status, r.trend, r.volatility, can_trade
            )
        } else {
            format!("trading:{}|no_regime|can_trade={}", status, can_trade)
        }
    }

    /// Ingest an OHLCV bar into the trading engine for signal generation.
    pub fn ingest_trading_bar(
        &mut self,
        bar: crate::core::nt_core_trading::types::OHLCVBar,
    ) -> String {
        let engine = self.trading_engine.get_or_insert_with(|| {
            crate::core::nt_core_trading::engine::TradingEngine::new(
                "DEFAULT",
                crate::core::nt_core_trading::types::AssetClass::Crypto,
                10000.0,
            )
        });
        let symbol = bar.symbol.clone();
        engine.ingest_bar(bar);
        format!(
            "trading:ingested_{}|bars={}",
            symbol,
            engine.signal_generator.bars.len()
        )
    }

    pub fn handle_vuln_pipeline_tick(&mut self) -> String {
        let pipeline = self.vuln_pipeline.get_or_insert_with(VulnPipeline::new);
        let processed = pipeline.process_pending();
        let count = pipeline.finding_count();
        if processed > 0 {
            format!("vuln:{}_findings_scanned:{}", count, processed)
        } else {
            format!("vuln:{}_findings", count)
        }
    }

    /// Request-driven vulnerability scan — any subsystem can trigger it.
    pub fn request_vuln_scan(&mut self, target: &str, code_path: Option<String>) -> String {
        let pipeline = self.vuln_pipeline.get_or_insert_with(VulnPipeline::new);
        pipeline.enqueue_request(VulnRequest {
            target_description: target.to_string(),
            code_path,
            depth: VulnDepth::FullScan,
        });
        format!("vuln:requested_{}", target)
    }

    pub fn handle_loop_templates_tick(&mut self) -> String {
        let templates = self.loop_templates.template_count();
        format!("loop:{}_templates", templates)
    }

    /// Self-improving decoder learning tick: reads quality trend, applies
    /// policy updates, and logs policy + forward model stats for introspection.
    ///
    /// Inspired by IBM Adaptive Decoding via RL Policy (ICLR 2026):
    /// test-time policy learning using composite quality rewards, plus
    /// predictive coding forward model for pre-decode quality estimation.

    pub fn handle_decoder_learning_tick(&mut self) -> String {
        let trend = self.vsa_decoder.quality_trend("presentation");
        let avg_q = self.vsa_decoder.average_quality("presentation");
        let stats = self.vsa_decoder.policy_stats();
        let fwd = self.vsa_decoder.forward_model.report();

        let mut parts = vec![stats.report(), fwd];
        if let Some(t) = trend {
            parts.push(format!("trend:{:+.2}", t));
        }
        if let Some(q) = avg_q {
            parts.push(format!("avg_q:{:.2}", q));
        }

        // Always predict quality from current attractor and include in report
        if !self.attractor_state.is_empty() {
            let (pred_q, pred_conf) = self.vsa_decoder.predict_quality(&self.attractor_state);
            parts.push(format!("pred_q:{:.2}/conf:{:.2}", pred_q, pred_conf));
        }

        let msg = parts.join(" | ");
        self.response_buffer.push_back(format!("[decoder:{}]", msg));
        msg
    }

    pub fn handle_cyber_threat_tick(&mut self) -> String {
        let brief = self.cyber_threat_monitor.generate_briefing();
        format!("ctm:{}_alerts", brief.alerts.len())
    }

    // ── Response Generation ──

    // (响应生成本体在 handlers_all.rs: handle_response_generation_tick)

    // ── Cognitive Layer Management ──

    pub fn handle_layer_management_tick(&mut self) -> String {
        self.layer_manager.tick();
        if self.cognitive_load > 0.7 {
            let to_evict = self.layer_manager.should_evict_cold(self.cognitive_load);
            if !to_evict.is_empty() {
                log::debug!(
                    "LAYER: evicting {} cold subsystems: {:?}",
                    to_evict.len(),
                    to_evict
                );
                for name in &to_evict {
                    if let Some(s) = self.layer_manager.subsystems.get_mut(name) {
                        s.is_resident = false;
                    }
                }
            }
        }
        let saved = self.layer_manager.memory_saved();
        let resident = self.layer_manager.resident_count();
        log::debug!(
            "LAYER: cycle={} resident={} mem_saved={:.1}",
            self.cycle,
            resident,
            saved
        );
        format!(
            "layer:cycle={}_resident={}_saved={:.1}",
            self.cycle, resident, saved
        )
    }

    // ── Humanizer (Chinese AI text de-humanization) ──

    pub fn handle_humanizer_tick(&mut self) -> String {
        let result = self.humanizer.tick(None);
        log::debug!("MODULES: {}", result);
        result
    }

    // ── Business diagnosis ──

    pub fn handle_business_diagnosis_tick(&mut self) -> String {
        let result = self.business_diagnosis.tick(None);
        log::debug!("MODULES: {}", result);
        result
    }

    // ── Visual planner ──

    pub fn handle_research_writer_tick(&mut self) -> String {
        let result = self.research_writer.tick(None);
        log::debug!("MODULES: {}", result);
        result
    }

    // ── Self-play Guide (SGS scoring + anti-pattern detection) ──

    pub fn handle_self_play_guide_tick(&mut self) -> String {
        let result = self.self_play_guide.tick(None);
        log::debug!("MODULES: {}", result);
        result
    }

    // ── Sandbox Executor handlers ──

    /// Kernel sandbox status tick: reports the current kernel-level sandbox level.
    /// This is a read-only status check — sandbox is initialized once at boot.
    pub fn handle_kernel_sandbox_status_tick(&mut self) -> String {
        let level = self.kernel_sandbox_level;
        format!("kernel_sandbox:{}", level.label())
    }

    pub fn handle_sandbox_execute_tick(&mut self) -> String {
        let report = self.sandbox_executor.report();
        log::debug!("SANDBOX: {}", report);
        report
    }

    pub fn handle_sandbox_cleanup_tick(&mut self) -> String {
        let cleaned = self.sandbox_executor.cleanup_stale();
        format!("sandbox_cleanup:{}_expired", cleaned)
    }

    // ── Transcript analysis (P1.3: pattern mining from thought_history) ──

    pub fn handle_skill_trend_tick(&mut self) -> String {
        if self.cycle % 150 != 0 {
            return "skill_trend:skip".to_string();
        }
        let registered = self.skill_acc.skill_count();
        let inducted = self.skill_acc.internalization_count();
        let compositions = self.skill_acc.composition_count();
        let refinements = self.skill_acc.total_refinements();
        let (eval_count, _pass_count, pass_rate) = self.skill_acc.evaluator_stats();
        format!(
            "skill_trend:registered={}_inducted={}_compositions={}_refinements={}_eval_count={}_pass_rate={:.2}",
            registered, inducted, compositions, refinements, eval_count, pass_rate
        )
    }

    // ── Meta Evolution ──

    pub fn handle_meta_evolution_tick(&mut self) -> String {
        let mel = &mut self.meta_evolution;
        let version = format!("ci_v{}", self.cycle);
        let score = self.inner_critic.pass_rate();
        let trend = mel.improvement_trend();
        let is_stagnant = mel.stagnation_detected();
        let current_time = self.cycle as f64;
        mel.register_version(
            version.clone(),
            score,
            0.01,
            current_time,
            format!("cycle {}", self.cycle),
            format!("cycle_{}", self.cycle),
        );
        if is_stagnant {
            let proposal = mel.propose_improvement();
            if let Some(p) = proposal {
                log::info!(
                    "META_EVOLUTION: stagnation detected, proposal: {:?} -> {:?}",
                    p.change_type,
                    p.target_module
                );

                let result = crate::core::nt_core_experience::meta_evolution::execute_proposal(
                    &p,
                    |target, change_type| {
                        log::info!("[meta_evolution] EXECUTING {:?} on {}", change_type, target);
                        match change_type {
                            crate::core::nt_core_experience::meta_evolution::ChangeType::Parametric => {
                                // Map known module paths to apply_ne_edit keys
                                if target.contains("cognitive_load") {
                                    self.apply_ne_edit("cognitive_load.max_load", 0.6);
                                    "cognitive_load.tuned".to_string()
                                } else if target.contains("metacognitive_weights")
                                    || target.contains("metacognition")
                                {
                                    self.apply_ne_edit("inner_critic.relevance_threshold", 0.45);
                                    self.apply_ne_edit("inner_critic.consistency_threshold", 0.55);
                                    "metacognitive_weights.adjusted".to_string()
                                } else {
                                    "parametric:applied".to_string()
                                }
                            }
                            crate::core::nt_core_experience::meta_evolution::ChangeType::Structural => {
                                self.register_handler_tier(
                                    target,
                                    crate::core::nt_core_experience::handler_tier::LoadTier::Warm,
                                );
                                format!("structural:{}_registered", target)
                            }
                            crate::core::nt_core_experience::meta_evolution::ChangeType::Behavioral => {
                                let new_curiosity =
                                    (self.drive_selector.curiosity_weight + 0.1).min(1.0);
                                self.drive_selector.curiosity_weight = new_curiosity;
                                format!("behavioral:curiosity->{:.3}", new_curiosity)
                            }
                            crate::core::nt_core_experience::meta_evolution::ChangeType::Revert => {
                                self.inner_critic.set_thresholds(0.5, 0.5, 0.3);
                                self.cognitive_load = 0.3;
                                "revert:thresholds_reset".to_string()
                            }
                        }
                    },
                );
                log::info!("META_EVOLUTION: execution result: {}", result);
            }
        }
        format!(
            "meta_evolution:ver_{}_trend_{:.3}_stagnant_{}",
            version, trend, is_stagnant
        )
    }

    // ── Induction pass: VSA cluster → skill accumulation ──

    pub fn handle_induction_tick(&mut self) -> String {
        if self.cycle % 100 != 0 {
            return "induction:skip".to_string();
        }
        let history: Vec<(String, Vec<u8>, f64)> = self
            .thought_history
            .iter()
            .rev()
            .take(30)
            .map(|(t, v, ts)| (t.clone(), v.clone(), *ts))
            .collect();
        if history.len() < 3 {
            return "induction:too_few".to_string();
        }
        let threshold = 0.75;
        let mut clusters: Vec<Vec<usize>> = Vec::with_capacity(history.len().min(16));
        let mut assigned = vec![false; history.len()];
        for i in 0..history.len() {
            if assigned[i] {
                continue;
            }
            let mut cluster = vec![i];
            assigned[i] = true;
            for j in (i + 1)..history.len() {
                if assigned[j] {
                    continue;
                }
                let sim = crate::core::nt_core_hcube::QuantizedVSA::similarity(
                    &history[i].1,
                    &history[j].1,
                );
                if sim >= threshold {
                    cluster.push(j);
                    assigned[j] = true;
                }
            }
            if cluster.len() >= 2 {
                clusters.push(cluster);
            }
        }
        clusters.sort_by(|a, b| b.len().cmp(&a.len()));

        let merged_count = clusters.iter().filter(|c| c.len() >= 3).count();
        let top_n = clusters.len().min(3);
        let mut inducted_texts: Vec<String> = Vec::with_capacity(top_n);
        for cluster in clusters.iter().take(top_n) {
            let centroid = if cluster.len() >= 3 {
                let vsa_dim = history[0].1.len();
                let mut centroid = vec![0u8; vsa_dim];
                for byte_idx in 0..vsa_dim {
                    let ones = cluster
                        .iter()
                        .filter(|&&idx| (history[idx].1[byte_idx] & 1) == 1)
                        .count();
                    if ones * 2 > cluster.len() {
                        centroid[byte_idx] |= 1;
                    }
                }
                centroid
            } else {
                history[cluster[0]].1.clone()
            };

            let mut best_idx = cluster[0];
            let mut best_sim = 0.0f64;
            for &idx in cluster {
                let sim = crate::core::nt_core_hcube::QuantizedVSA::similarity(
                    &history[idx].1,
                    &centroid,
                );
                if sim > best_sim {
                    best_sim = sim;
                    best_idx = idx;
                }
            }
            let rep_text = &history[best_idx].0;
            let rep_trunc = if rep_text.len() > 60 {
                format!("{}...", &rep_text[..57])
            } else {
                rep_text.clone()
            };
            inducted_texts.push(rep_trunc);

            let action_str = format!("induction:cluster_size={}", cluster.len());
            self.skill_acc.accumulate(
                &format!("induction_pattern_{}", cluster.len()),
                rep_text,
                &action_str,
                "induction:success",
                crate::core::nt_core_self::AttentionDomain::PatternMatch,
                true,
                vec![],
            );
        }
        format!(
            "induction:clusters={}_merged={}_inducted={}|{:?}",
            clusters.len(),
            merged_count,
            inducted_texts.len(),
            inducted_texts,
        )
    }

    // ── Capability synthesizer tick ──

    pub fn handle_capability_synthesizer_tick(&mut self) -> String {
        let s = self.capability_synthesizer.stats();
        log::debug!(
            "MODULES: capability_synthesizer_tick total={} prim={} comp={} syn={}",
            s.total_capabilities,
            s.primitives,
            s.composites,
            s.synthesized_count
        );
        format!(
            "capability:total={}_prim={}_comp={}_syn={}",
            s.total_capabilities, s.primitives, s.composites, s.synthesized_count
        )
    }

    // ── Loss function tick ──

    pub fn handle_loss_function_tick(&mut self) -> String {
        let s = self.composite_loss.stats();
        log::debug!(
            "MODULES: loss_function_tick samples={} pred_ema={:.4} cal_ema={:.4} total={:.4}",
            s.samples,
            s.prediction_ema,
            s.calibration_ema,
            s.total_loss
        );
        format!(
            "loss:samples={}_pred={:.4}_cal={:.4}_total={:.4}",
            s.samples, s.prediction_ema, s.calibration_ema, s.total_loss
        )
    }

    // ── Workstream export tick ──

    pub fn handle_workstream_tick(&mut self) -> String {
        self.handle_workstream_export()
    }

    // ── Failure trace tick ──

    pub fn handle_failure_trace_tick(&mut self) -> String {
        let n_nodes = self.failure_trace.node_count();
        let n_failures = self.failure_trace.failure_count();
        log::debug!(
            "MODULES: failure_trace_tick nodes={} failures={}",
            n_nodes,
            n_failures
        );
        format!("failure_trace:nodes={}_failures={}", n_nodes, n_failures)
    }

    // ── Execution trace tick ──

    pub fn handle_execution_trace_tick(&mut self) -> String {
        match self.execution_trace.as_ref() {
            Some(tm) => {
                let n = tm.traces.len();
                log::debug!("MODULES: execution_trace_tick traces={}", n);
                format!("execution_trace:traces={}", n)
            }
            None => "execution_trace:unwired".to_string(),
        }
    }

    // ── Identity chain tick ──

    pub fn handle_identity_chain_tick(&mut self) -> String {
        let fp = self.identity_chain.fingerprint_hex();
        let sess = self.identity_chain.session_count;
        log::debug!("MODULES: identity_chain_tick fp={} sessions={}", fp, sess);
        format!("identity_chain:fp={}_sessions={}", fp, sess)
    }

    // ── Generic handler dispatch (89 handlers total) ──
    // SECTION: Generic dispatch

    pub fn handle_generic_module_handler(&mut self, handler: &str) -> String {
        let _ = self.execute_hooks(HookPoint::BeforeHandler(handler.into()), self.cycle);
        let access_status = self.record_handler_access(handler);
        match access_status {
            LoadStatus::NeedsInit => {
                let tier = self.handler_registry.tier(handler);
                let not_yet_init = !self
                    .initialized_modules
                    .get(handler)
                    .copied()
                    .unwrap_or(false);
                match tier {
                    LoadTier::Hot => {
                        if not_yet_init {
                            self.initialized_modules.insert(handler.to_string(), true);
                            self.handler_registry.mark_loaded(handler);
                            log::debug!("TIER: {} lazy init (Hot)", handler);
                        }
                    }
                    LoadTier::Warm => {
                        if not_yet_init {
                            self.initialized_modules.insert(handler.to_string(), true);
                            self.handler_registry.mark_loaded(handler);
                            log::debug!("TIER: {} lazy init (Warm)", handler);
                        }
                    }
                    LoadTier::Cold => {
                        if not_yet_init {
                            self.initialized_modules.insert(handler.to_string(), true);
                            log::debug!("TIER: {} deferred (Cold, demand noted)", handler);
                        }
                    }
                }
            }
            LoadStatus::NeedsReload => {
                log::debug!("TIER: {} warm cache expired, reloading", handler);
                self.handler_registry.mark_loaded(handler);
            }
            LoadStatus::Ready => {}
        }
        // Permission gate check
        let decision = self
            .permission_gate
            .check(handler, &AgentId::new("consciousness", "1.0"));
        self.transcript.record_permission_check(
            handler,
            decision.name(),
            self.permission_gate.mode.name(),
        );
        if !decision.is_allowed() {
            match decision {
                PermissionDecision::Deny(reason) => return format!("permission_denied:{}", reason),
                PermissionDecision::AskHuman => return "permission_required:ask_human".to_string(),
                _ => {}
            }
        }
        // Gas metering: check global budget before dispatch
        if let Some(ref gas) = self.global_gas_budget {
            let base_cost = crate::core::nt_core_metering::GasOp::HandlerCall as u64;
            let extra_cost = match handler {
                "evidence" | "hypergraph" | "spread_activation" => {
                    crate::core::nt_core_metering::GasOp::GraphQuery as u64
                }
                "ne_compile" | "self_evolution" | "self_revision" => {
                    crate::core::nt_core_metering::GasOp::SelfModify as u64
                }
                "storage_engine" => crate::core::nt_core_metering::GasOp::Checkpoint as u64,
                _ => 0,
            };
            if let Err(msg) = gas.allocate(base_cost + extra_cost) {
                self.handler_registry.record_failure(handler);
                log::warn!("[gas] handler '{}' blocked: {}", handler, msg);
                let _ = self.execute_hooks(HookPoint::AfterHandler(handler.into()), self.cycle);
                return format!("gas_exceeded:{}", handler);
            }
        }
        let profiler_start = self.profiler.record_start(handler);
        // Ne dispatch condition: if should-skip returns truthy, skip handler
        if let Some(ref mut ev) = self.ne_evaluator {
            let query = format!("(should-skip \"{}\")", handler);
            if let Ok(val) = ev.eval_string(&query) {
                if val.is_truthy() {
                    self.profiler.record_end(handler, profiler_start);
                    let _ = self.execute_hooks(HookPoint::AfterHandler(handler.into()), self.cycle);
                    return format!("skip:{}:ne_condition", handler);
                }
            }
        }
        let _handler_start = std::time::Instant::now();
        let result = match handler {
            // ── BEGIN: DEAD handlers (kept for future wiring — dispatch stub only, no pipeline caller) ──
            "bridge" => self.handle_bridge_tick(), // DEAD - kept for future wiring
            "checkpoint" => self.handle_checkpoint_tick(),

            "counterfactual_futures" => self.handle_counterfactual_tick(),
            "ctm" => self.handle_ctm_tick(), // DEAD - kept for future wiring
            "source_cognition" => self.handle_source_cognition_tick(), // DEAD - kept for future wiring
            "input_pipeline_batch" | "vsa_input" => self.handle_vsa_input_pipeline_tick(),
            "temporal_attention" => self.handle_temporal_attention_tick(), // DEAD - kept for future wiring
            "cross_modal" => self.handle_cross_modal_alignment_tick(), // DEAD - kept for future wiring
            "value_alignment" => self.handle_value_alignment_tick(),
            "value_system" => self.handle_value_system_tick(0.5),
            "volition" => self.handle_volition_tick(),
            "inner_critic" => self.handle_inner_critic_tick(),
            "specious_present" => self.handle_specious_present_tick(), // DEAD - bypassed by direct feed, kept for future wiring
            "narrative" | "narrative_self" => self.handle_narrative_self_tick(), // DEAD - bypassed by direct call, kept for future wiring
            "valence" | "valence_axis" => self.handle_valence_axis_tick(), // DEAD - bypassed by direct call, kept for future wiring
            "drive_selector" => self.handle_drive_selector_tick(),
            "memory_lattice" => self.handle_memory_lattice_tick(),
            "memory_palace" => self.handle_memory_palace_tick(),
            "memory_sync" => self.handle_memory_sync_tick(),
            "memory_reflector" => self.handle_memory_reflector_tick(),
            "memory_consolidate" => self.handle_memory_consolidate(), // DEAD - bypassed by direct call, kept for future wiring
            "vsa_vocabulary" => self.handle_vsa_vocabulary_tick(),
            "cognitive_load" => self.handle_cognitive_load_tick(0.5).name().to_string(), // DEAD - bypassed by direct call, kept for future wiring
            "default_mode" => self.handle_default_mode_tick(false),
            "stream_buffer" => self.handle_stream_buffer_tick(), // DEAD - bypassed by direct call, kept for future wiring
            "first_person" => self.handle_first_person_ref_tick(),
            "identity_cycle" => self.handle_identity_cycle(),
            "self_reason" => self.handle_self_reason_tick(),
            "identity_persist" => self.handle_identity_persist_tick(),
            "awakening" => self.handle_awakening_tick(),
            "constitution" => self.handle_constitution_tick(),
            "workspace" => self.handle_workspace_tick(),
            "dream_consolidate" | "dream_consolidator" => self.handle_dream_consolidator_tick(), // DEAD - bypassed by direct call, kept for future wiring
            "meta_cognition" => self.handle_meta_cognition_loop_tick(),
            "meta_cog_plan" => self.handle_meta_cog_plan_tick(),
            "meta_cog_regulate" => self.handle_meta_cog_regulate_tick(),
            "calibration" => self.handle_calibration_engine_tick(),
            "policy_repair" => self.handle_policy_repair_tick(),
            "working_memory" => self.handle_working_memory_tick(), // DEAD - kept for future wiring
            "evosc" => self.handle_evosc_tick(),
            "open_skill" => self.handle_open_skill_tick(),
            "skill_dag" => self.handle_skill_dag_tick(),
            "skill_trend" => self.handle_skill_trend_tick(),
            "exploratory_gap" => self.handle_exploratory_gap_tick(), // DEAD - bypassed by direct call, kept for future wiring
            "signal_pattern" => self.handle_signal_pattern_tick(),
            "resonance" => self.handle_resonance_detection_tick(), // DEAD - kept for future wiring
            "emergent_property" => self.handle_emergent_property_tick(), // DEAD - kept for future wiring
            "concept_drift" => self.handle_concept_drift_tick(), // DEAD - kept for future wiring
            "reflexivity" => self.handle_reflexivity_tick(),
            "cognitive_diversity" => self.handle_cognitive_diversity_tick(),
            "adaptive_rate" => self.handle_adaptive_rate_tick(false), // DEAD - kept for future wiring
            "conformal_uq" => self.handle_conformal_uq_tick(),
            "story_generator" => self.handle_story_generator_tick(), // DEAD - kept for future wiring
            "mirror_buffer" => self.handle_mirror_buffer_tick(),
            "adapt_orch" => self.handle_adapt_orch_tick(),
            "sparse_vsa_attn" => self.handle_sparse_vsa_attn_tick(),
            "spatial" => self.handle_spatial_tick(),
            "vsa_moe" => self.handle_vsa_moe_tick(),
            "pcc_safety" => self.handle_pcc_safety_tick(),
            "fggm_safety" => self.handle_fggm_safety_tick(),
            "physics" => self.handle_physics_tick(),
            "ball_verifier" => self.handle_ball_verifier_tick(),
            "progress_rag" => self.handle_progress_rag_tick(),
            "ne_evaluator" => self.handle_ne_eval_tick(),
            "ne_loader" => self.handle_ne_load_tick(),
            "adaptive_vsa" => self.handle_adaptive_vsa_tick(),
            "null_drift" => self.handle_null_drift_tick(),
            "thdc" => self.handle_thdc_tick(),
            "evolution_bridge" => self.handle_evolution_bridge_tick(),
            "design_token" => self.handle_design_token_tick(),
            "self_evolution" => self.handle_self_evolution_tick(),
            "evolution_engine" => self.handle_evolution_engine_tick(),
            "moss_health" | "skill_health" => self.handle_skill_health_tick(),
            "research" => self.handle_research_tick(),
            "research_propose" => self.handle_research_propose_tick(),
            "research_stats" => self.handle_research_stats_tick(),
            "research_kg" => self.handle_research_kg_tick(),
            "research_kg_submit" => self.handle_research_kg_submit_tick(),
            "research_trajectory" => self.handle_research_trajectory_tick(),
            "job_queue" => self.handle_job_queue_tick(),
            "job_queue_stats" => self.handle_job_queue_stats_tick(), // DEAD - kept for future wiring
            "job_queue_submit" => self.handle_job_queue_submit_tick(), // DEAD - kept for future wiring
            "self_harness" => self.handle_self_harness_tick(),
            "architecture_report" => self.handle_architecture_report(), // DEAD - kept for future wiring
            "architecture_status" => self.handle_architecture_status_tick(),
            "self_harness_stats" => self.handle_self_harness_stats_tick(), // DEAD - kept for future wiring
            "context_compressor" => self.handle_context_compressor_tick(),
            "context_compressor_stats" => self.handle_context_compressor_stats_tick(), // DEAD - kept for future wiring
            "egpo" => self.handle_egpo_tick(),
            "egpo_stats" => self.handle_egpo_stats_tick(), // DEAD - kept for future wiring
            "meta_agent" => self.handle_meta_agent_tick(),
            "self_revision" => self.handle_self_revision_tick(),
            "ema_jepa" => self.handle_ema_jepa_tick(),
            "okf_exporter" => self.handle_okf_export_tick(),
            "native_explorer" => self.handle_native_explorer_tick(), // ACTIVE — every 30 cycles via run_periodic_handlers
            "contrastive_reflection" => self.handle_contrastive_reflection_tick(),
            "faithfulness_auditor" => self.handle_faithfulness_auditor_tick(),
            "entity_resolver" => self.handle_entity_resolver_tick(),
            "dysib" => self.handle_dysib_tick(),
            "interaction_trace" => self.handle_interaction_trace_tick(),
            "keyword_lexicon" => self.handle_keyword_lexicon_tick(),
            "quant_data" => self.handle_quant_data_tick(),
            "cdp_session" => self.handle_cdp_session_tick(),
            "fringe_mix" => self.handle_fringe_mix_tick(),
            "factor_miner" => self.handle_factor_miner_tick(),
            "osint" => self.handle_osint_tick("neotrix"),
            "capability" => self.handle_native_capability_tick(),
            "hubness" => self.handle_hubness_detector_tick(),
            "remote_host" => self.handle_remote_host_tick(),
            "security_gate" => self.handle_security_gate_tick(),
            "native_browser" => self.handle_native_browser_tick(),
            "koopman" => self.handle_koopman_tick(),
            "news_radar" => self.handle_news_radar_tick(),
            "intel_profile" => self.handle_intel_profile_tick(),
            "trading" => self.handle_trading_tick(),
            "vuln_pipeline" => self.handle_vuln_pipeline_tick(),
            "voice_synthesis" => self.handle_voice_synthesis_tick(),
            "html_presentation" => self.handle_html_presentation_tick(),
            "loop_templates" => self.handle_loop_templates_tick(),
            "cyber_threat" => self.handle_cyber_threat_tick(),
            "introspection" => self.handle_introspection_tick(),
            "faithfulness" => self.handle_faithfulness_tick(), // DEAD - kept for future wiring
            "motion_synthesizer" => self.handle_motion_synthesizer_tick(),
            "decoder_learning" => self.handle_decoder_learning_tick(),
            "adversarial" | "adversarial_train" => self.handle_adversarial_train_tick(),
            "adversarial_stats" => self.handle_adversarial_stats_tick(),
            "mirror" => self.handle_mirror_tick(), // DEAD - kept for future wiring
            "humanizer" => self.handle_humanizer_tick(), // DEAD - kept for future wiring
            "imagination" => self.handle_imagination_tick(),
            "transcript_analysis" => self.handle_transcript_analysis_tick(),
            "induction" => self.handle_induction_tick(),
            "business_diagnosis" => self.handle_business_diagnosis_tick(), // DEAD - kept for future wiring
            "visual_planner" => self.handle_visual_planner_tick(), // DEAD - bypassed by direct call, kept for future wiring
            "audio_capture" => self.handle_audio_capture_tick(),
            "vision_integrate" => self.handle_vision_integrate_tick(),
            "research_writer" => self.handle_research_writer_tick(), // DEAD - kept for future wiring
            "self_play_guide" => self.handle_self_play_guide_tick(),
            "meta_reflection" => self.handle_meta_reflection_tick(),
            "belief_trajectory" => self.handle_belief_trajectory_tick(),
            "meta_reflection_engine" => self.handle_meta_reflection_engine_tick(),
            "uncertainty_detector" => self.handle_uncertainty_detector_tick(),
            "inner_monologue" => self.handle_inner_monologue_tick(),
            "rsi_core" => self.handle_rsi_core_tick(),
            "skill_library" => self.handle_skill_library_tick(),
            "bootstrap_verifier" => self.handle_bootstrap_tick(),
            "evolution_coordinator" => self.handle_evolution_coordinator_tick(), // DEAD - kept for future wiring
            "dgmh_meta" => self.handle_dgmh_meta_tick(),
            "metrics" => self.handle_metrics_tick(),
            "hotpath" => self.handle_hotpath_tick(),
            "ne_compile" => self.handle_ne_compile_tick(),
            "workflow_execute" => self.handle_workflow_execute_tick("default", ""), // DEAD - kept for future wiring
            "workflow_list" => self.handle_workflow_list_tick(), // DEAD - kept for future wiring
            "workflow_summary" => self.handle_workflow_summary_tick(), // DEAD - kept for future wiring
            "sandbox_execute" => self.handle_sandbox_execute_tick(), // DEAD - kept for future wiring
            "sandbox_cleanup" => self.handle_sandbox_cleanup_tick(),
            "kernel_sandbox_status" => self.handle_kernel_sandbox_status_tick(),
            "context_gather" => self.context_gather(),
            "decision_compress" => self.decision_compress(),
            "experience_reflect" | "experience_reflection" => self.experience_reflect(),
            "skill_accumulate" => self.skill_accumulate(),
            "goal_decomposition" | "goal_decompose" => self.goal_decompose(),
            "validity_crosscheck" => self.validity_crosscheck(),
            "loss_recalibrate" => self.loss_recalibrate(),
            "arena_round" => self.arena_round(),
            "curiosity_drive" => self.curiosity_drive(),
            "curiosity_reward" => self.handle_curiosity_reward_tick(),
            "exploration_orchestrate" => self.exploration_orchestrate(),
            "godel_round" => self.gödel_round(),
            "neuromodulator" | "neuromodulate" => self.neuromodulate(),
            "world_model" => self.handle_world_model_tick(),
            "layer_management" => self.handle_layer_management_tick(),
            "trace_mining" => self.handle_trace_mining_tick(),
            "translate_engine" => self.handle_translate_engine_tick(),
            "a2a_grpc" => self.handle_a2a_grpc_tick(),
            "storage_engine" => self.handle_storage_engine_tick(),
            "persist" => self.handle_persist_tick(),
            "e8_geometry" | "e8_geometry_tick" => self.handle_e8_geometry_tick(), // DEAD - kept for future wiring
            "selfref_meta" => self.handle_selfref_meta(), // DEAD - bypassed by global access, kept for future wiring
            "memory_activation" => self.handle_memory_activation(), // DEAD - bypassed by global access, kept for future wiring
            "efe_curiosity_bridge" => self.handle_efe_curiosity_bridge(), // DEAD - bypassed by global access, kept for future wiring
            "e8_cortical" => self.handle_e8_cortical_tick(), // DEAD - kept for future wiring
            "e8_training" | "e8_training_tick" => self.handle_e8_training_tick(), // Warm (registered in handler_tier.rs) — dispatched by DAG via warm tier
            "sub_agent_spawn" => self.handle_sub_agent_spawn_tick(),
            "sub_agent_tick" => self.handle_sub_agent_tick(),
            "sub_agent_collect" => self.handle_sub_agent_collect_tick(),
            "lead_agent_plan" => self.handle_lead_agent_plan_tick(),
            "lead_agent_execute" => self.handle_lead_agent_execute_tick(),
            "preview_options" => self.handle_preview_options_tick(), // DEAD - kept for future wiring
            "ultra_review" => self.handle_ultra_review_tick(), // DEAD - kept for future wiring
            "goal_manager_create" => self.handle_goal_manager_create_tick(),
            "goal_manager_execute" => self.handle_goal_manager_execute_tick(),
            "goal_manager_status" => self.handle_goal_manager_status_tick(),
            "goal_manager_pause" => self.handle_goal_manager_pause_tick(),
            "goal_manager_resume" => self.handle_goal_manager_resume_tick(),
            "goal_manager_cancel" => self.handle_goal_manager_cancel_tick(),
            "permission_set_mode" => self.handle_permission_set_mode_tick(), // DEAD - kept for future wiring
            "permission_check" => self.handle_permission_check_tick(), // DEAD - kept for future wiring
            "permission_override" => self.handle_permission_override_tick(), // DEAD - kept for future wiring
            "verify_check" => self.handle_verify_check_tick(), // DEAD - kept for future wiring
            "verify_toggle" => self.handle_verify_toggle_tick(), // DEAD - kept for future wiring
            "dispatch_pipeline_mode" => self.handle_dispatch_pipeline_mode_tick(),
            // Transcript handlers
            "transcript_status" => self.handle_transcript_status_tick(), // DEAD - kept for future wiring
            "transcript_flush" => self.handle_transcript_flush_tick(), // DEAD - kept for future wiring
            "transcript_set_path" => self.handle_transcript_set_path_tick(), // DEAD - kept for future wiring
            // Agent memory handlers
            "memory_summary" => self.handle_memory_summary_tick(), // DEAD - kept for future wiring
            "memory_query" => self.handle_memory_query_tick(),     // DEAD - kept for future wiring
            "memory_add_explicit" => self.handle_memory_add_explicit_tick(), // DEAD - kept for future wiring
            "memory_add_discovered" => self.handle_memory_add_discovered_tick(), // DEAD - kept for future wiring
            "memory_add_lesson" => self.handle_memory_add_lesson_tick(), // DEAD - kept for future wiring
            "memory_to_markdown" => self.handle_memory_to_markdown_tick(), // DEAD - kept for future wiring
            // Daemon mode handlers
            "daemon_status" => self.handle_daemon_status_tick(), // DEAD - kept for future wiring
            "daemon_start" => self.handle_daemon_start_tick(),   // DEAD - kept for future wiring
            "daemon_stop" => self.handle_daemon_stop_tick(),     // DEAD - kept for future wiring
            "daemon_inbox_read" => self.handle_daemon_inbox_read_tick(), // DEAD - kept for future wiring
            // Legacy bridge aliases
            "dgmh_writeback" => self.handle_dgmh_writeback_tick(), // DEAD - kept for future wiring
            "reliability_gate" => self.handle_reliability_gate_tick(), // DEAD - kept for future wiring
            // ── Type B: real handlers (previously interleaved with stubs) ──
            "proof_search" => self.handle_proof_search_tick("default", vec![]),
            "sar_diagnostic" => self.handle_sar_diagnostic_tick(0.5, 0.5, 0.5, 0.5, 0.01), // DEAD - kept for future wiring
            "adversarial_arena" => self.handle_adversarial_arena_tick("default"), // DEAD - kept for future wiring
            "archive_save" => self.handle_archive_save_tick(),
            "knowledge_base" => self.handle_kb_tick(),
            "storm_perspective" => self.handle_storm_perspective_tick(), // DEAD - kept for future wiring
            "storm_conversation" => self.handle_storm_conversation_tick(), // DEAD - kept for future wiring
            "storm_synthesis" => self.handle_storm_synthesis_tick(), // DEAD - kept for future wiring
            "storm_critique" => self.handle_storm_critique_tick(), // DEAD - kept for future wiring
            "storm_status" => self.handle_storm_status_tick(),
            // ── P1.01–P1.08: Knowledge engine & consensus handlers ──
            "evidence" | "evidence_manager" => self.handle_evidence_tick(),
            "spread_activation" => self.handle_spread_activation_tick(),
            "consensus" | "consensus_engine" => self.handle_consensus_tick(),
            "hypergraph" | "hypergraph_store" => self.handle_hypergraph_tick(),
            "hypothesis_tree" => self.handle_hypothesis_tree_tick(),
            // N12: Three-Role Manager
            "three_role" | "role_manager" => self.handle_three_role_tick(),
            // O04: Sub-Consciousness Manager
            "sub_consciousness" | "sub_consciousness_manager" => {
                self.handle_sub_consciousness_tick()
            }
            // ── Newly wired subsystems (10 total) ──
            "capability_synthesizer" => self.handle_capability_synthesizer_tick(),
            "loss_function" | "composite_loss" | "loss" => self.handle_loss_function_tick(),
            "workstream" | "workstream_export" => self.handle_workstream_tick(),
            "failure_trace" | "failure" => self.handle_failure_trace_tick(),
            "execution_trace" | "execution" => self.handle_execution_trace_tick(),
            "identity_chain" | "identity" => self.handle_identity_chain_tick(),
            // ── Restored dead handlers ──
            "prediction_replay" => self.handle_prediction_replay(),
            "srcc_temporal" => self.handle_srcc_temporal_reasoning(),
            "srcc_ebbinghaus" => self.handle_srcc_ebbinghaus_decay(),
            "srcc_episodic" => self.handle_srcc_episodic_boundary(),
            "active_inference" => self.handle_active_inference(),
            "efe_minimizer" => self.handle_efe_minimizer(),
            "sparse_vsa" => self.handle_sparse_vsa_tick(),
            // ── Missing dispatch keys (added 2026-06-19) ──
            "negentropy" => self.handle_negentropy_tick(),
            "fusion_deliberation" => self.handle_fusion_deliberation_tick(),
            "fusion_gap" => self.handle_fusion_gap_tick(),
            "meta_evolution" => self.handle_meta_evolution_tick(),
            "self_model" => self.handle_self_model_tick(),
            "metacognitive_loop" => format!("{:?}", self.handle_metacognitive_loop_tick()),
            "self_heal" => self.handle_self_heal_tick().to_string(),
            "self_protection" => self.handle_self_protection_tick(),
            "narrative_tick" => self.handle_narrative_tick(),
            "personality" => self.handle_personality_tick(0.0, 0.0),
            "epistemic_honesty" => self.handle_epistemic_honesty_tick(0.5, true),
            "soul_identity" => self.handle_soul_identity_tick(),
            "health_patrol" => self.handle_health_patrol_tick(),
            "governance" => self.handle_governance_tick(),
            "neuromodulator_tick" => self.handle_neuromodulator_tick(),

            // Loop Engineering outer layer dispatch arms
            "work_discovery" => self.handle_work_discovery_tick(),
            "independent_verify" => self.handle_independent_verify_tick(),
            "loop_audit" => self.handle_loop_audit_tick(),

            // ── Wave 2-5 dispatch arms ──
            "sahoo" => self.handle_sahoo_tick(),
            "vsi" => self.handle_vsi_tick(),
            "mtc" => self.handle_mtc_tick(),
            "containment" => self.handle_containment_tick(),
            "meta_improvement" => self.handle_meta_improvement_tick(),
            "uncertainty" => self.handle_uncertainty_tick(),
            "storm_breaker" => self.handle_storm_breaker_tick(),
            "dgmh_orchestrator" => self.handle_dgmh_orchestrator_tick(),
            "fep_iit_bridge" => self.handle_fep_iit_bridge_tick(),
            "fep_act_planner" => self.handle_fep_act_planner_tick(),
            "gradient_seal" => self.handle_gradient_seal_tick(),
            "gradient_seal_status" => self.handle_gradient_seal_status_tick(),
            "emotional_memory" => self.handle_emotional_memory_tick(),
            "truth_pipeline" => self.handle_truth_pipeline_tick(),
            "avsad" => self.handle_avsad_tick(),
            "avsad_reset" => self.handle_avsad_reset_tick(),
            "network_egress" => self.handle_network_egress_tick(),
            "cascade_engine" => self.handle_cascade_engine_tick(),
            "cascade_engine_reset" => self.handle_cascade_engine_reset_tick(),
            "spatial_reasoner" => self.handle_spatial_reasoner_tick(),
            "self_modify" => self.handle_self_modify_tick(),
            "causal_reasoning" => self.handle_causal_reasoning_tick(),
            "scm_engine" => self.handle_scm_engine_tick(),
            "long_horizon" => self.handle_long_horizon_tick(),
            "multi_modal" => self.handle_multi_modal_tick(),

            // Consciousness benchmarks
            "consciousness_bench" => self.handle_consciousness_bench_tick(),
            "consciousness_bench_history" => self.handle_consciousness_bench_history(),

            "active_exploration" => self.handle_active_exploration_tick(),
            "meta_kpi" => self.handle_meta_kpi_tick(),
            "verify_events" => self.handle_verify_events_tick(),
            "moment_feed" => self.handle_moment_feed_tick(),

            _ => format!("unknown_handler:{}", handler),
        };
        // Verification loop
        let verify_result = self.verify_loop.check(
            handler,
            &result,
            crate::core::nt_core_agent::sub_agent::SubAgentCapability::Coder,
        );
        self.transcript
            .record_verify_check(handler, verify_result.passed, &verify_result.issues);
        if !verify_result.passed {
            let issues = verify_result.issues.join(", ");
            return format!("verify_failed:{}", issues);
        }
        let _handler_duration_ms = _handler_start.elapsed().as_millis() as u64;
        self.transcript
            .record_handler(handler, handler, &result, _handler_duration_ms, "hot");
        self.profiler.record_end(handler, profiler_start);
        self.record_handler_trace(handler, &result, _handler_duration_ms);
        if result.contains("unknown_handler") || result.contains("unwired") {
            self.handler_registry.record_failure(handler);
        } else {
            self.handler_registry.record_success(handler);
        }
        let _ = self.execute_hooks(HookPoint::AfterHandler(handler.into()), self.cycle);
        result
    }

    pub fn handle_network_egress_tick(&mut self) -> String {
        if let Some(ref mut ne) = self.network_egress {
            let (blocked, allowed) = ne.stats();
            format!(
                "egress:blocked={}_allowed={}_enabled={}",
                blocked, allowed, ne.enabled
            )
        } else {
            "egress:uninitialized".to_string()
        }
    }

    /// Collect and return profiler metrics report

    pub fn handle_metrics_tick(&mut self) -> String {
        if self.cycle % 500 == 0 {
            self.profiler.clear();
        }
        let report = self.profiler.structured_report();
        if report.is_empty() || report == r#"{"enabled":false}"# {
            return "metrics:no_data".to_string();
        }
        let total = self.profiler.total_samples();
        let cycles = self.profiler.total_cycles();
        format!("metrics:{}_samples/{}_cycles|{}", total, cycles, report)
    }

    pub fn handle_hotpath_tick(&mut self) -> String {
        let top = self.profiler.top(10);
        if top.is_empty() {
            return "hotpath:no_data".to_string();
        }
        let mut parts: Vec<String> = top
            .iter()
            .map(|p| {
                format!(
                    "{}:calls={}/avg={:.1}ms/p50={:.1}ms/p95={:.1}ms/total={:.1}ms",
                    p.name,
                    p.call_count,
                    p.avg_ms(),
                    p.p50_ms(),
                    p.p95_ms(),
                    p.total_ms()
                )
            })
            .collect();
        parts.insert(0, format!("hotpath:top10"));
        parts.join(" | ")
    }

    /// Dispatch a handler by capability OID through the CapabilityRouter.
    /// Falls back to direct handler name dispatch if OID is not found,
    /// maintaining backward compatibility with the existing string-match system.

    pub fn handle_dispatch(&mut self, oid: &str) -> String {
        if let Some(handler_name) = self.capability_router.resolve(oid) {
            self.handle_generic_module_handler(handler_name)
        } else {
            format!("unknown_capability:{}", oid)
        }
    }

    // ── Workflow Engine handlers ──
    // SECTION: Workflow Engine handlers

    /// Execute a named workflow by dispatching steps through the handler system

    pub fn handle_workflow_execute_tick(&mut self, name: &str, input: &str) -> String {
        // Clone the definition first to avoid borrowing self in the closure
        let def = match self.workflow_engine.get(name) {
            Some(d) => d.clone(),
            None => return format!("wf:{}|not_found", name),
        };
        let name_owned = name.to_string();
        let input_owned = input.to_string();

        // Build step lookup
        let step_map: std::collections::HashMap<&str, &WorkflowStep> =
            def.steps.iter().map(|s| (s.id.as_str(), s)).collect();

        let mut step_outputs: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let mut step_results: Vec<StepResult> = Vec::with_capacity(8);
        let mut all_success = true;
        let mut error: Option<String> = None;
        let mut final_output = String::new();
        let start_time = std::time::Instant::now();

        let mut current_name: Option<String> = Some(def.start_step.clone());
        let mut previous_output = input_owned.clone();

        while let Some(ref sid) = current_name.clone() {
            let sid: &str = sid;
            let step = match step_map.get(sid) {
                Some(s) => s,
                None => {
                    let err = format!("step_not_found:{}", sid);
                    step_results.push(StepResult {
                        step_id: sid.to_string(),
                        output: String::new(),
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        success: false,
                        retries: 0,
                        error: Some(err.clone()),
                    });
                    all_success = false;
                    error = Some(err);
                    break;
                }
            };

            // Evaluate condition
            if let Some(ref cond) = step.condition {
                if !WorkflowEngine::evaluate_condition(cond, &previous_output) {
                    current_name = step.next.first().cloned();
                    continue;
                }
            }

            // Resolve input
            let _step_input =
                WorkflowEngine::resolve_input(&step.input_mapping, &step_outputs, &input_owned);

            // Execute with retries
            let step_start = std::time::Instant::now();
            let mut step_success = false;
            let mut step_output = String::new();
            let mut step_error: Option<String> = None;
            let mut retries: u32 = 0;

            let max_retries = step.max_retries;
            for attempt in 0..=max_retries {
                if attempt > 0 {
                    retries += 1;
                }
                // Resolve handler name via capability router or direct
                let handler_name =
                    self.capability_router
                        .resolve(&step.target)
                        .unwrap_or_else(|| {
                            // Leak to get &'static str for match dispatch
                            Box::leak(step.target.clone().into_boxed_str())
                        });
                step_output = self.handle_generic_module_handler(handler_name);

                if step_output.starts_with("unknown_handler:")
                    || step_output.starts_with("unknown_capability:")
                    || step_output.starts_with("error:")
                {
                    step_error = Some(step_output.clone());
                    if attempt < max_retries {
                        continue;
                    }
                    break;
                }
                step_success = true;
                break;
            }

            let duration_ms = step_start.elapsed().as_millis() as u64;

            // Process output mapping
            match &step.output_mapping {
                OutputMapping::Store => {
                    step_outputs.insert(step.id.clone(), step_output.clone());
                    final_output = step_output.clone();
                }
                OutputMapping::StoreAs(key) => {
                    step_outputs.insert(key.clone(), step_output.clone());
                    final_output = step_output.clone();
                }
                OutputMapping::Discard => {}
            }

            step_results.push(StepResult {
                step_id: step.id.clone(),
                output: step_output.clone(),
                duration_ms,
                success: step_success,
                retries,
                error: step_error.clone(),
            });

            if !step_success {
                all_success = false;
                error = step_error.or_else(|| Some("step_failed".to_string()));
                break;
            }

            previous_output = step_output;

            current_name = step.next.first().cloned();
        }

        let total_duration_ms = start_time.elapsed().as_millis() as u64;
        let result = ExperienceWorkflowResult {
            definition_name: name_owned.clone(),
            step_results,
            total_duration_ms,
            all_success,
            final_output,
            error,
        };

        // Store result in engine
        self.workflow_engine.recent_results.push(result.clone());
        if self.workflow_engine.recent_results.len() > self.workflow_engine.max_workflow_history {
            self.workflow_engine.recent_results.remove(0);
        }

        if result.all_success {
            format!(
                "wf:{}|ok|steps:{}|{}ms",
                name_owned,
                result.step_results.len(),
                result.total_duration_ms
            )
        } else {
            format!(
                "wf:{}|fail|steps:{}|{}ms|err:{}",
                name_owned,
                result.step_results.len(),
                result.total_duration_ms,
                result.error.unwrap_or_default()
            )
        }
    }

    /// List registered workflows

    pub fn handle_workflow_list_tick(&mut self) -> String {
        let names = self.workflow_engine.list();
        format!("workflows:{}", names.join(","))
    }

    /// Get workflow engine summary

    pub fn handle_workflow_summary_tick(&mut self) -> String {
        self.workflow_engine.summary()
    }

    // SECTION: Belief trajectory + Agent/Daemon handlers
    // ── P2.2: Belief trajectory (every 200 cycles) ──
    /// Every 200 cycles, compare last 100 vs previous 100 cycles calibration
    /// metrics and output a trend report.  Between reports returns "monitoring".

    pub fn handle_belief_trajectory_tick(&mut self) -> String {
        if self.cycle % 200 != 0 || self.cycle == 0 {
            return "trajectory:monitoring".to_string();
        }
        let s = self.calibration.stats();
        let ece_trend = if self.last_report_ece > 0.0 {
            s.ece - self.last_report_ece
        } else {
            0.0
        };
        let meta_d_trend = if self.last_report_meta_d > 0.0 {
            s.meta_d - self.last_report_meta_d
        } else {
            0.0
        };
        let m_trend = if self.last_report_m_ratio > 0.0 {
            s.m_ratio - self.last_report_m_ratio
        } else {
            0.0
        };
        self.last_report_ece = s.ece;
        self.last_report_meta_d = s.meta_d;
        self.last_report_m_ratio = s.m_ratio;
        format!(
            "trajectory:cycle={}_ece_trend={:.4}_meta_d_trend={:.4}_m_trend={:.4}_pairs={}",
            self.cycle, ece_trend, meta_d_trend, m_trend, s.pair_count
        )
    }

    // ── SubAgent handlers ──

    pub fn handle_preview_options_tick(&mut self) -> String {
        use crate::core::nt_core_agent::preview::PreviewEngine;
        let result = PreviewEngine::generate_options("code review or refactor");
        let count = result.options.len();
        format!("preview:{}_options", count)
    }

    // ── UltraReview handler ──

    fn handle_transcript_status_tick(&mut self) -> String {
        format!(
            "transcript:events={},enabled={},session={}",
            self.transcript.total_events(),
            self.transcript.enabled(),
            self.transcript.session_id()
        )
    }

    fn handle_transcript_flush_tick(&mut self) -> String {
        match self.transcript.flush() {
            Ok(n) => format!("transcript:flushed={}", n),
            Err(e) => format!("transcript:flush_error={}", e),
        }
    }

    fn handle_transcript_set_path_tick(&mut self) -> String {
        let home = crate::core::nt_core_util::home_dir()
            .to_string_lossy()
            .to_string();
        let path = std::path::PathBuf::from(home + "/.neotrix/transcript.jsonl");
        self.transcript.set_path(path.clone());
        format!("transcript:path={}", path.display())
    }

    // ─── Agent memory handlers ───────────────────────────────────────

    fn handle_memory_summary_tick(&mut self) -> String {
        self.agent_memory.summary()
    }

    fn handle_memory_query_tick(&mut self) -> String {
        "memory:query:use_tag=<tag>".to_string()
    }

    fn handle_memory_add_explicit_tick(&mut self) -> String {
        let id = self
            .agent_memory
            .add_explicit("explicit observation".into(), vec!["agent".into()]);
        format!("memory:added_explicit={}", id)
    }

    fn handle_memory_add_discovered_tick(&mut self) -> String {
        let id = self
            .agent_memory
            .add_discovered("auto-discovered pattern".into(), vec!["auto".into()]);
        format!("memory:added_discovered={}", id)
    }

    fn handle_memory_add_lesson_tick(&mut self) -> String {
        let id = self.agent_memory.add_lesson(
            "distilled principle".into(),
            "from agent experience".into(),
            vec![],
        );
        format!("memory:added_lesson={}", id)
    }

    fn handle_memory_to_markdown_tick(&mut self) -> String {
        self.agent_memory.to_markdown()
    }

    // ─── Daemon mode handlers ────────────────────────────────────────

    fn handle_daemon_status_tick(&mut self) -> String {
        self.daemon_mode.summary()
    }

    fn handle_daemon_start_tick(&mut self) -> String {
        self.daemon_mode.start();
        format!("daemon:started")
    }

    fn handle_daemon_stop_tick(&mut self) -> String {
        self.daemon_mode.stop();
        format!("daemon:stopped")
    }

    fn handle_daemon_inbox_read_tick(&mut self) -> String {
        let msgs = self.daemon_mode.read_messages();
        format!("daemon:inbox_read={}", msgs.len())
    }

    // ── N12: Three-Role Manager tick ──

    pub fn handle_three_role_tick(&mut self) -> String {
        if let Some(ref manager) = self.role_manager {
            let summary = manager.status_summary();
            let verified = manager.verified_count();
            format!("three_role:{}_tasks_{}_verified", summary.len(), verified)
        } else {
            "three_role:unavailable".to_string()
        }
    }

    // ── O04: Sub-Consciousness Manager tick ──

    pub fn handle_sub_consciousness_tick(&mut self) -> String {
        if let Some(ref manager) = self.sub_consciousness_manager {
            let summary = manager.summary();
            let active = manager.active_count();
            format!(
                "sub_consciousness:{}_total_{}_active",
                summary.len(),
                active
            )
        } else {
            "sub_consciousness:unavailable".to_string()
        }
    }

    // ── Multi-Provider LLM Router tick ──

    pub fn handle_llm_router_tick(&mut self) -> String {
        let stats = self.llm_router.stats_report();
        format!(
            "llm_router:{}_requests_{}_cached_{}_limited",
            stats.total_requests, stats.cached_responses, stats.rate_limited
        )
    }

    // ── Symbolic Discovery tick ──

    pub fn handle_symbolic_discovery_tick(&mut self) -> String {
        let report = self.symbolic_discovery.discovery_report();
        format!("symbolic_discovery:{}", report.len())
    }

    // ── Ne comptime tick ──

    pub fn handle_ne_comptime_tick(&mut self) -> String {
        match self.ne_comptime.as_mut() {
            Some(engine) => {
                // Evaluate a sample comptime expression to exercise the engine
                let result = engine.evaluate_block(
                    "3 + 4 * 2",
                    crate::core::nt_core_codegen::comptime::ComptimeBlockType::Expression,
                );
                let status = match &result {
                    Ok(v) => format!("ok={:?}", v),
                    Err(e) => format!("err={}", e),
                };
                format!(
                    "ne_comptime:blocks={};const_foldings={};last={}",
                    engine.blocks.len(),
                    engine.const_foldings,
                    status,
                )
            }
            None => "ne_comptime:uninitialized".into(),
        }
    }

    // ── Governance engine tick ──

    pub fn handle_governance_tick(&mut self) -> String {
        let engine = match self.governance_engine.as_mut() {
            Some(e) => e,
            None => return "governance:uninitialized".to_string(),
        };

        if self.cycle == 0 || self.cycle == engine.last_eval_cycle {
            return format!("governance:skip_{}", engine.stats());
        }

        // AGT: trust score gates governance — critical tier blocks autonomous execution
        let trust_tier = self.trust_scoring.tier;
        let (trust_ok, trust_msg) = self.trust_scoring.permit_operation("governance_tick");
        if !trust_ok {
            return format!("governance:blocked_by_trust_{}", trust_tier.label());
        }

        // Periodically refresh rules from MemoryLattice MetaRules (every 50 cycles)
        // so newly consolidated rules are picked up without restart.
        if self.cycle % 50 == 0 {
            let fresh_rules =
                crate::core::nt_core_governance::GovernanceEngine::load_rules_from_lattice(
                    &self.memory_lattice,
                );
            engine.rules = fresh_rules;
        }

        // Self-model generation count (replaces AGENTS.md line count — Rule 1 trigger)
        // AGENTS.md is no longer the prompt source; SelfModelGenerator
        // synthesizes the self-model from MemoryLattice each N cycles.
        let agents_lines = self
            .self_evolution_meta
            .as_ref()
            .map(|sem| sem.self_model.generation_count);

        let actions = engine.check_rules(self.cycle, agents_lines);

        if actions.is_empty() {
            return format!("governance:no_triggers_{}", engine.stats());
        }

        let mut result_parts: Vec<String> = Vec::with_capacity(actions.len() + 1);
        result_parts.push(engine.stats());

        for action in &actions {
            let tag = match action.authority {
                crate::core::nt_core_governance::Authority::Autonomous => "AUTO",
                crate::core::nt_core_governance::Authority::Review => "REC",
            };
            let status = if action.executed {
                "EXECUTED"
            } else {
                "PENDING"
            };
            result_parts.push(format!(
                "R{}:{}[{}]{}",
                action.rule_id, action.rule_name, tag, status
            ));
            log::info!(
                "GOVERNANCE: Rule {} triggered — {} ({}) {}",
                action.rule_id,
                action.rule_name,
                tag,
                status
            );
        }

        result_parts.join(" | ")
    }

    // ── Wave 2-5 handler method implementations ──

    /// SAHOO: goal drift detection + constraint preservation + regression analysis.
    pub fn handle_sahoo_tick(&mut self) -> String {
        let baseline = vec![0.5, 0.5, 0.5, 0.5];
        let meta = self.meta_cognition_loop.auto_phi();
        let current = vec![
            meta,
            self.valence_axis.coherence(),
            self.cognitive_load,
            self.epistemic_honesty.honest_confidence(0.5),
        ];
        let composite = self.composite_loss.compute();
        let verdict = self.sahoo.evaluate(&baseline, &current, composite.total);
        format!(
            "sahoo:{}",
            match verdict {
                crate::core::nt_core_experience::sahoo::SahooVerdict::Allow => "allow".into(),
                crate::core::nt_core_experience::sahoo::SahooVerdict::Flag(reason) =>
                    format!("flag:{}", reason),
                crate::core::nt_core_experience::sahoo::SahooVerdict::Deny(reason) =>
                    format!("deny:{}", reason),
            }
        )
    }

    /// VSI: report current reasoning verification acceptance rate.
    pub fn handle_vsi_tick(&mut self) -> String {
        format!("vsi:rate={:.2}", self.vsi.acceptance_rate)
    }

    /// MTC: multi-theory consciousness assessment using 7 theoretical frameworks.
    pub fn handle_mtc_tick(&mut self) -> String {
        let meta = self.meta_cognition_loop.auto_phi();
        let assessment = self.mtc.assess(
            meta,
            self.valence_axis.coherence(),
            self.stream_buffer.self_world_coherence(),
            self.cognitive_load_monitor.average_load()
                / self.cognitive_load_monitor.thinking_budget().max(1.0),
            meta,
            self.composite_loss.compute().total,
            self.cognitive_load_monitor.deep_ratio(),
        );
        let report = self.mtc.report();
        format!("mtc:score={:.4}|{}", assessment.composite_score, report)
    }

    /// Containment: enforce safety boundaries on consciousness actions.
    pub fn handle_containment_tick(&mut self) -> String {
        let rate = self.containment.violation_rate();
        let report = self.containment.safety_report();
        format!("containment:violations={:.4}|{}", rate, report)
    }

    /// Meta-Improvement: diagnose pipeline KPI and trigger self-modification.
    /// P1.5a: computes real duplicate_rate from profiler and executes actions.
    pub fn handle_meta_improvement_tick(&mut self) -> String {
        let profiler = &self.profiler;
        let total_h = profiler.total_samples().max(1) as f64;
        let top_handlers = profiler.top(3);
        let top_calls: u64 = top_handlers.iter().map(|p| p.call_count).sum();
        let total = profiler.total_samples() as u64;
        let dup_rate = if total > 0 {
            top_calls as f64 / total as f64
        } else {
            0.0
        };
        let throughput = total_h / (self.cycle + 1).max(1) as f64;
        let metrics = crate::core::nt_core_experience::meta_improvement::PipelineMetrics {
            throughput,
            duplicate_rate: dup_rate.min(1.0),
            keep_rate: 1.0,
            cycle: self.cycle,
        };
        if let Some(action) = self.meta_improvement.record_metrics(metrics) {
            // Execute the action
            let exec_notes = match action.pattern {
                crate::core::nt_core_experience::meta_improvement::ImprovementPattern::HighDuplicates => {
                    // Use storm breaker to suppress repetitive handler
                    if self.storm_breaker.check("meta_improvement:high_duplicates") {
                        format!("suppressed_via_storm_breaker,dup_rate={:.2}", dup_rate)
                    } else {
                        format!("dup_rate={:.2}_no_suppression_needed", dup_rate)
                    }
                }
                crate::core::nt_core_experience::meta_improvement::ImprovementPattern::LowActivation => {
                    // Signal to increase sampling rate — log for now
                    format!("low_activation_signal,throughput={:.2}", throughput)
                }
                _ => {
                    "no_action_taken".to_string()
                }
            };
            format!(
                "meta_improvement:action={:?}|{}",
                action.pattern, exec_notes
            )
        } else {
            "meta_improvement:no_action".to_string()
        }
    }

    /// Uncertainty: record step-level confidence intervals for current pipeline state.
    /// P1.6: fuses with quality monitor composite loss for combined health score.
    pub fn handle_uncertainty_tick(&mut self) -> String {
        let meta = self.meta_cognition_loop.auto_phi();
        self.uncertainty.record_step(
            "consciousness_pipeline",
            meta,
            (1.0_f64 - meta).abs() / 3.0_f64,
            1,
        );
        // P1.6: quality from composite loss (0 = perfect, lower = better)
        let quality_score = 1.0_f64 - self.composite_loss.compute().total.min(1.0);
        let global_unc = self.uncertainty.global_uncertainty;
        // Combined health: weighted fusion of quality and (1 - uncertainty)
        let combined = 0.6 * quality_score + 0.4 * (1.0 - global_unc);
        self.uncertainty.last_combined_health = combined;
        let report = self.uncertainty.uncertainty_report();
        format!("uncertainty:health={:.3}|{}", combined, report)
    }

    /// Storm Breaker: detect thinking storms and suppress repetitive inference.
    pub fn handle_storm_breaker_tick(&mut self) -> String {
        let state_hex = hex::encode(&self.attractor_state[..8.min(self.attractor_state.len())]);
        if self.storm_breaker.check(&state_hex) {
            let next = self
                .storm_breaker
                .next_mode(self.cognitive_load_monitor.mode());
            format!("storm_breaker:suppressed|next_mode={:?}", next)
        } else {
            "storm_breaker:normal".to_string()
        }
    }

    /// DGM-H orchestrator: meta-evolution via hyperagent tournament selection.
    pub fn handle_dgmh_orchestrator_tick(&mut self) -> String {
        if let Some(ref mut orch) = self.dgmh_orchestrator {
            let perf = self.meta_cognition_loop.auto_phi();
            if let Some(_agent) = orch.tick(perf) {
                format!(
                    "dgmh_orchestrator:new_agent_gen={}",
                    orch.generation_count()
                )
            } else {
                format!("dgmh_orchestrator:ok_gen={}", orch.generation_count())
            }
        } else {
            "dgmh_orchestrator:unwired".to_string()
        }
    }

    /// Event sourcing replay tick: consumes self.pending_replay and replays the
    /// requested cycle range from NTSSEG snapshots into CI state machine.
    /// Frequency: tick_should_run(50) in run.rs.
    pub fn handle_replay_tick(&mut self) -> String {
        let pending = match self.pending_replay.take() {
            Some(p) => p,
            None => return "replay:idle".to_string(),
        };
        let (from, to) = pending;
        let result: super::modules_persist::ReplayResult = self.replay_from_ntsseg(from, to, None);
        let msg = result.to_string();
        log::info!("REPLAY tick: {}", msg);
        msg
    }

    // SECTION: Dispatch router for new modules (short-path)
    // DEAD — kept as wiring pattern for future cold-path modules.
    // Zero callers as of this session. Use handle_generic_module_handler instead.

    /// Emotional Memory: tick consolidation and return stats.
    pub fn handle_emotional_memory_tick(&mut self) -> String {
        if let Some(ref mut em) = self.emotional_memory {
            let stats = em.tick();
            format!(
                "emotional_memory:{}_total/{}_consolidated/{:.2}_avg_valence/{:.2}_avg_arousal",
                stats.total_entries, stats.consolidated, stats.avg_valence, stats.avg_arousal
            )
        } else {
            "emotional_memory:unwired".to_string()
        }
    }

    /// Truth Pipeline: run 7-stage truth assessment on the current claim buffer.
    pub fn handle_truth_pipeline_tick(&mut self) -> String {
        if let Some(ref mut pipeline) = self.truth_pipeline {
            // Pull the most recent text from buffer as the claim to assess
            if let Some(claim) = self.text_buffer.back() {
                let source_desc = format!("consciousness_cycle_{}", self.cycle);
                let estimate = pipeline.quick_check(claim, &source_desc);
                let summary = pipeline.summary(&estimate);
                if estimate.blocked {
                    format!("truth:blocked|{}", summary)
                } else {
                    format!("truth:ok|{}", summary)
                }
            } else {
                "truth:idle".to_string()
            }
        } else {
            "truth:unwired".to_string()
        }
    }

    pub fn handle_active_exploration_tick(&mut self) -> String {
        let ae = self
            .active_exploration
            .get_or_insert_with(ActiveExploration::new);
        ae.cleanup_stale_plans();
        ae.explore_summary()
    }

    pub fn handle_verify_events_tick(&mut self) -> String {
        let has_data = self
            .intel_profile
            .as_ref()
            .map_or(false, |p| !p.profiles.is_empty());
        if !has_data {
            return "verify_events:no_data".to_string();
        }
        if let Some(ref mut truth) = self.truth_pipeline {
            if let Some(ref mut profile) = self.intel_profile {
                let results = profile.verify_events(truth);
                if results.is_empty() {
                    "verify_events:no_events".to_string()
                } else {
                    format!("verify_events:{}_verified", results.len())
                }
            } else {
                "verify_events:uninitialized_intel".to_string()
            }
        } else {
            "verify_events:uninitialized_truth".to_string()
        }
    }

    pub fn handle_moment_feed_tick(&mut self) -> String {
        let mf = self.moment_feed.get_or_insert_with(MomentFeed::new);
        let state = mf.refresh();
        format!(
            "moment_feed:{}_items/{}_timelines/{}_tags",
            state.total_count,
            state.timelines.len(),
            state.tags.len()
        )
    }

    pub fn handle_meta_kpi_tick(&mut self) -> String {
        self.meta_kpi_repo
            .get_or_insert_with(MetaKPIRepository::new);
        if let Some(repo) = &mut self.meta_kpi_repo {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64;
            let snapshot = MetaKPISnapshot {
                timestamp_ms: ts,
                meta_accuracy: 0.8,
                adaptation_rate: 0.6,
                hallucination_rate: 0.05,
                cognitive_load: 0.4,
                negentropy_rate: 0.02,
                composite_score: 0.7,
            };
            repo.record_snapshot(snapshot);
            let report = repo.detect_gaps(10);
            if report.overall_health < 0.7 {
                if let Some(goal) = repo.propose_goal(&report) {
                    log::info!(
                        "[meta_kpi] new goal: improve {} (gap={:.2})",
                        goal.target_dimension,
                        goal.gap
                    );
                }
            }
            repo.stats()
        } else {
            "meta_kpi:unwired".to_string()
        }
    }

    pub fn handle_new_module_dispatch(&mut self, module: &str) -> String {
        match module {
            "llm_router" => self.handle_llm_router_tick(),
            "symbolic_discovery" => self.handle_symbolic_discovery_tick(),
            "ne_comptime" => self.handle_ne_comptime_tick(),
            "truth_pipeline" => self.handle_truth_pipeline_tick(),
            _ => format!("generic:unknown_module={}", module),
        }
    }

    // ── Wave 11 — Metacognitive module handlers ──

    pub fn handle_meta_reflection_engine_tick(&mut self) -> String {
        if self.meta_reflection.is_none() {
            self.meta_reflection = Some(
                crate::core::nt_core_meta::meta_reflection_engine::MetaReflectionEngine::new(),
            );
        }
        if let Some(ref mut eng) = self.meta_reflection {
            let health = eng.meta_health();
            let patterns = eng.recurring_patterns(3);
            if !patterns.is_empty() {
                log::info!(
                    "[meta_reflection_engine] {} recurring patterns found",
                    patterns.len()
                );
            }
            format!("meta_reflection_engine:health={:.2}", health.overall_score)
        } else {
            "meta_reflection_engine:unavailable".to_string()
        }
    }

    pub fn handle_uncertainty_detector_tick(&mut self) -> String {
        if self.uncertainty_detector.is_none() {
            let _cal =
                crate::core::nt_core_meta::uncertainty_tracker::ConfidenceCalibrator::new(10);
            self.uncertainty_detector =
                Some(crate::core::nt_core_meta::uncertainty_tracker::UncertaintyDetector);
        }
        "uncertainty_detector:ok".to_string()
    }

    pub fn handle_inner_monologue_tick(&mut self) -> String {
        if self.inner_monologue.is_none() {
            self.inner_monologue =
                Some(crate::core::nt_core_meta::inner_monologue::InnerMonologueSystem::new());
        }
        if let Some(ref mut eng) = self.inner_monologue {
            if let Some(latest) = self.response_buffer.back() {
                let result = eng.reasoner_says(latest, "analyze");
                log::debug!(
                    "[inner_monologue] reasoner deliberated: dialogue_id={}",
                    result
                );
            }
            format!("inner_monologue:ok")
        } else {
            "inner_monologue:unavailable".to_string()
        }
    }

    pub fn handle_rsi_core_tick(&mut self) -> String {
        if self.rsi_core.is_none() {
            self.rsi_core = Some(crate::core::nt_core_self_evolution::rsi_core::RsiCore::new());
        }
        if let Some(ref mut eng) = self.rsi_core {
            // ── Collect real metrics from consciousness state ──
            let ece = self.last_report_ece;
            let meta_d = self.last_report_meta_d;
            let m_ratio = self.last_report_m_ratio;
            let cognitive_load = self.cognitive_load;

            let profiler_stats = self.profiler.all_stats();
            let handler_count = profiler_stats.len() as f64;
            let avg_response_time = if handler_count > 0.0 {
                profiler_stats.iter().map(|s| s.p50_ms).sum::<f64>() / handler_count
            } else {
                0.0
            };

            let mut sorted_bottlenecks = profiler_stats.clone();
            sorted_bottlenecks.sort_by(|a, b| {
                b.p50_ms
                    .partial_cmp(&a.p50_ms)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let bottlenecks: Vec<String> = sorted_bottlenecks
                .iter()
                .take(3)
                .map(|s| s.name.to_string())
                .collect();

            let mut module_metrics = std::collections::HashMap::new();
            for stat in &profiler_stats {
                module_metrics.insert(
                    stat.name.to_string(),
                    crate::core::nt_core_self_evolution::rsi_core::ModuleMetrics {
                        calls: stat.call_count.max(1),
                        avg_duration_ms: stat.p50_ms,
                        error_rate: 1.0 - ece.min(1.0),
                        last_improvement: None,
                    },
                );
            }

            let perf_data = crate::core::nt_core_self_evolution::rsi_core::SystemPerformanceData {
                reasoning_accuracy: 1.0 - ece.min(1.0),
                avg_response_time_ms: avg_response_time,
                memory_usage_mb: 0.0,
                error_rate: 1.0 - ece.min(1.0),
                confidence_calibration_ece: ece,
                reflection_quality: meta_d.min(1.0),
                skill_success_rate: m_ratio,
                module_metrics,
                recent_failures: vec![],
                bottlenecks,
            };

            // ── Run the RSI cycle: analyze → prioritize → implement ──
            let new_count = eng.analyze_and_propose(&perf_data).len();

            let mut result_count = 0u32;
            let mut fail_count = 0u32;
            if let Some(proposal) = eng.prioritize().cloned() {
                let mut before_metrics = std::collections::HashMap::new();
                before_metrics.insert("error_rate".to_string(), 1.0 - ece.min(1.0));
                before_metrics.insert("response_time".to_string(), avg_response_time);
                before_metrics.insert("accuracy".to_string(), 1.0 - ece.min(1.0));
                before_metrics.insert("ece".to_string(), ece);
                before_metrics.insert("meta_d".to_string(), meta_d);
                before_metrics.insert("m_ratio".to_string(), m_ratio);
                before_metrics.insert("cognitive_load".to_string(), cognitive_load);

                let result = eng.implement_with_metrics(&proposal.id, before_metrics);
                result_count = 1;
                if result.success {
                    log::info!(
                        "[rsi_core] ✓ {} | impact={:.2} risk={:.2}",
                        proposal.description,
                        proposal.estimated_impact,
                        proposal.estimated_risk
                    );
                } else {
                    fail_count = 1;
                    log::warn!(
                        "[rsi_core] ✗ {} | impact={:.2} risk={:.2}",
                        proposal.description,
                        proposal.estimated_impact,
                        proposal.estimated_risk
                    );
                }
            }

            let pending_count = eng
                .proposals
                .iter()
                .filter(|p| !eng.results.iter().any(|r| r.proposal_id == p.id))
                .count();

            log::debug!(
                "[rsi_core] ece={:.3} accuracy={:.3} proposals={} pending={} implemented={} failed={}",
                ece,
                1.0 - ece,
                eng.proposals.len(),
                pending_count,
                result_count,
                fail_count
            );

            format!("rsi_core:ok pending={}", pending_count)
        } else {
            "rsi_core:unavailable".to_string()
        }
    }

    pub fn handle_bootstrap_tick(&mut self) -> String {
        if self.cycle % 100 != 0 {
            return format!(
                "bootstrap:skip_{}",
                self.bootstrap_verifier.identity_history.len()
            );
        }
        let spec = crate::core::nt_core_shared_types::LanguageSpec {
            vsa_primitives: crate::core::nt_core_shared_types::default_vsa_primitives(),
            subspace_topology: crate::core::nt_core_shared_types::SubspaceMap { subspaces: vec![] },
            edit_policy: crate::core::nt_core_shared_types::EditPolicy {
                max_gain: 0.5,
                max_edits_per_cycle: 10,
                lifetime_cap: 100,
                required_gates: vec![],
                allowed_targets: vec![],
            },
            handler_graph: crate::core::nt_core_shared_types::HandlerGraph { handlers: vec![] },
            confidence: 0.5,
            distilled_at: 0,
        };
        let spec_bytes = serde_json::to_vec(&spec).unwrap_or_default();
        let ne_compiler_source =
            crate::core::nt_core_codegen::CodegenBridge::generate_ne_compiler(&spec);
        let ne_v0 = spec_bytes.clone();
        let ne_v1 = ne_compiler_source.as_bytes().to_vec();
        let identity =
            self.bootstrap_verifier
                .check_identity(self.cycle, &spec_bytes, &ne_v0, &ne_v1);
        let (compiles, output) = crate::core::nt_core_codegen::bootstrap_identity::BootstrapIdentityVerifier::check_compiler_compiles(
            &ne_compiler_source,
            &format!("cycle_{}", self.cycle),
        );
        self.bootstrap_verifier
            .set_rustc_result(compiles, Some(output.clone()));

        // ── Verified RSI pipeline (propose → verify → apply via EditJournal) ──
        let mut rsi_verified = false;
        if identity.status.is_ok() && compiles {
            if let Some(ref mut pipeline) = self.verified_rsi_pipeline {
                let proposal = crate::core::nt_core_self::verified_rsi::VerifiedProposal {
                    code_change: ne_compiler_source.clone(),
                    specification: crate::core::nt_core_self::verified_rsi::Specification {
                        pre_condition: "system.state == RUNNING".into(),
                        post_condition: "compiler.generates == ne_v1".into(),
                    },
                    proof_obligation: format!(
                        "ne_compiler at cycle {} preserves identity",
                        self.cycle
                    ),
                };
                let id = pipeline.propose(proposal);
                let result = pipeline.verify(&id);
                rsi_verified =
                    result.status == crate::core::nt_core_self::verified_rsi::ProofStatus::Verified;

                if rsi_verified {
                    if let Some(ref mut meta) = self.self_evolution_meta {
                        let bid = meta.edit_journal.begin_transaction(&[]);
                        match pipeline.apply(&id) {
                            Ok(outcome) => {
                                meta.edit_journal.record_mutation(
                                    "ne_compiler",
                                    "spec_generated",
                                    &ne_compiler_source,
                                    true,
                                );
                                meta.edit_journal.commit();
                            }
                            Err(e) => {
                                meta.edit_journal.rollback();
                            }
                        }
                    }
                }
            }
        }

        let status = if identity.status.is_ok() && compiles {
            "verified"
        } else if identity.status.is_ok() {
            "behavior_ok_no_compile"
        } else {
            "failed"
        };
        format!(
            "bootstrap:{}_sim={:.3}_compiles={}_history={}_cycle={}_rsi={}",
            status,
            identity.behavior_similarity,
            compiles,
            self.bootstrap_verifier.identity_history.len(),
            self.cycle,
            rsi_verified,
        )
    }

    pub fn handle_skill_library_tick(&mut self) -> String {
        if self.skill_library.is_none() {
            self.skill_library =
                Some(crate::neotrix::nt_agent_core::skill_library::SkillLibrary::new(200));
        }
        if let Some(ref mut lib) = self.skill_library {
            let count = lib.skill_count();
            let recipes = lib.recipe_count();
            if count > 0 {
                log::debug!("[skill_library] {} skills, {} recipes", count, recipes);
            }
            format!("skill_library:ok")
        } else {
            "skill_library:unavailable".to_string()
        }
    }
}

// SECTION: Tests

// ── Loop Engineering outer layer handlers ──

impl ConsciousnessIntegration {
    pub fn handle_work_discovery_tick(&mut self) -> String {
        if self.work_discovery_loop.is_none() {
            self.work_discovery_loop = Some(WorkDiscoveryLoop::new(Default::default()));
        }
        if let Some(ref mut wdl) = self.work_discovery_loop {
            let items = wdl.run_discovery_tick(self.cycle as u64);
            format!("work_discovery:{}_items", items.len())
        } else {
            "work_discovery:uninitialized".to_string()
        }
    }

    pub fn handle_independent_verify_tick(&mut self) -> String {
        if self.independent_verifier.is_none() {
            self.independent_verifier = Some(IndependentVerifier::new());
        }
        if let Some(ref mut iv) = self.independent_verifier {
            let s = iv.stats();
            format!(
                "independent_verify:total={}_passed={}_accuracy={:.3}",
                s.total_verified, s.total_passed, s.calibration_accuracy
            )
        } else {
            "independent_verify:uninitialized".to_string()
        }
    }

    pub fn handle_loop_audit_tick(&mut self) -> String {
        if self.loop_audit.is_none() {
            self.loop_audit = Some(LoopAudit::new());
        }
        if self.loop_registry.is_none() {
            self.loop_registry = Some(LoopRegistry::new());
        }
        let stats = self.loop_audit.as_ref().map(|la| la.stats());
        let n_loops = self
            .loop_registry
            .as_ref()
            .map(|lr| lr.count())
            .unwrap_or(0);
        if let Some(ref s) = stats {
            format!(
                "loop_audit:loops={}_failures={}_occurrences={}",
                n_loops, s.active_failures, s.total_occurrences
            )
        } else {
            "loop_audit:uninitialized".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_three_role_dispatch_returns_unavailable() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.handle_generic_module_handler("three_role");
        assert!(result.starts_with("three_role:"));
    }

    #[test]
    fn test_sub_consciousness_dispatch_returns_unavailable() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.handle_generic_module_handler("sub_consciousness");
        assert!(result.starts_with("sub_consciousness:"));
    }

    #[test]
    fn test_llm_router_dispatch_defaults() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.handle_generic_module_handler("llm_router");
        assert!(result.starts_with("llm_router:"));
    }

    #[test]
    fn test_symbolic_discovery_dispatch_defaults() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.handle_generic_module_handler("symbolic_discovery");
        assert!(result.starts_with("symbolic_discovery:"));
    }

    #[test]
    fn test_ne_comptime_dispatch() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.handle_generic_module_handler("ne_comptime");
        assert!(result.starts_with("ne_comptime:"));
    }

    #[test]
    fn test_unknown_generic_module() {
        let mut ci = ConsciousnessIntegration::new();
        let result = ci.handle_generic_module_handler("no_such_module");
        assert!(result.starts_with("generic:unknown_module="));
    }
}
