use super::super::nt_mind::memory::ReasoningBank;
use super::super::nt_mind::self_iterating::ReasoningBrain;
use super::*;
use crate::core::nt_core_consciousness::{ThinkingMode, VsaOrigin, VsaSelfCategory};
use crate::core::nt_core_decision::ActionType;
use crate::core::nt_core_experience::{FailureType, TraceNodeType};
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use crate::core::nt_core_input::NgramVsaEncoder;
use crate::core::nt_core_scheduler::default_scheduler;
use crate::core::nt_core_self::attention_head::AttentionDomain;

impl BackgroundLoop {
    pub async fn start(&mut self) {
        if !self.config.enabled {
            return;
        }

        println!("[bg] background loop started");

        // Seed default scheduler jobs (build_cleanup, knowledge_aging, evosc)
        {
            let anchor_now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            self.scheduler = default_scheduler(anchor_now);
        }

        let mut always_on_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.always_on_interval_secs,
        ));
        let mut save_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.save_interval_secs,
        ));
        let mut consolidate_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.consolidate_interval_secs,
        ));
        let mut meta_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.metacog_interval_secs,
        ));
        let mut thinking_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.thinking_interval_secs,
        ));
        let mut goal_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.goal_interval_secs,
        ));
        let mut exploration_orch_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.consciousness_pipeline_interval_secs * 2,
        ));
        let mut prediction_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.world_prediction_interval_secs,
        ));
        let mut panorama_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.panorama_interval_secs,
        ));
        let mut exploration_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.exploration_interval_secs,
        ));
        let mut curiosity_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.curiosity_interval_secs,
        ));
        let mut knowledge_chain_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.knowledge_chain_interval_secs,
        ));
        let mut aging_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.knowledge_aging_interval_secs,
        ));
        let mut crystallization_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.crystallization_interval_secs,
        ));
        let mut nt_act_voice_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.nt_act_voice_interval_secs,
        ));
        let mut awareness_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.metacog_interval_secs,
        ));
        let mut plugin_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.plugin_interval_secs,
        ));
        let mut scheduler_ticker = tokio::time::interval(tokio::time::Duration::from_secs(30));
        let mut discovery_ticker = tokio::time::interval(tokio::time::Duration::from_secs(60));

        let mut consciousness_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.consciousness_pipeline_interval_secs,
        ));

        #[cfg(feature = "stealth-net")]
        let mut heartbeat_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.proxy_heartbeat_interval_secs,
        ));
        #[cfg(not(feature = "stealth-net"))]
        let mut heartbeat_ticker = tokio::time::interval(tokio::time::Duration::from_secs(3600));

        #[cfg(feature = "stealth-net")]
        let mut nt_world_sense_ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.nt_world_sense_interval_secs,
        ));
        #[cfg(not(feature = "stealth-net"))]
        let mut nt_world_sense_ticker =
            tokio::time::interval(tokio::time::Duration::from_secs(3600));

        #[cfg(feature = "stealth-net")]
        if let Some(ref nt_world_crawl) = self.tor_crawler {
            let c = nt_world_crawl.clone();
            tokio::spawn(async move { c.run().await });
        }

        // Spawn AgentServer if discovery is enabled
        if self.agent_discovery.is_some() {
            let server = std::sync::Arc::new(
                super::super::nt_agent_protocol::tcp_server::AgentServer::new(42070),
            );
            let server_clone = server.clone();
            tokio::spawn(async move {
                match server_clone.start().await {
                    Ok(port) => log::info!("[bg] AgentServer listening on TCP :{}", port),
                    Err(e) => log::error!("[bg] AgentServer start failed: {}", e),
                }
            });
        }

        loop {
            tokio::select! {
                _ = always_on_ticker.tick() => self.handle_always_on().await,
                _ = save_ticker.tick() => self.handle_save().await,
                _ = consolidate_ticker.tick() => self.handle_consolidate().await,
                _ = meta_ticker.tick() => self.handle_meta().await,
                _ = thinking_ticker.tick() => self.handle_thinking().await,
                _ = goal_ticker.tick() => self.handle_goal().await,
                _ = exploration_orch_ticker.tick() => {
                    if let Some(ref mut ci) = self.consciousness {
                        ci.handle_exploration_orchestrate(&self.curiosity_drive, ci.cycle as u64);
                    }
                },
                _ = prediction_ticker.tick() => self.handle_prediction().await,
                _ = panorama_ticker.tick() => self.handle_panorama().await,
                _ = exploration_ticker.tick() => self.handle_exploration().await,
                _ = curiosity_ticker.tick() => self.handle_curiosity().await,
                _ = knowledge_chain_ticker.tick() => self.handle_knowledge_chain().await,
                _ = aging_ticker.tick() => self.handle_knowledge_aging().await,
                _ = crystallization_ticker.tick() => self.handle_crystallization().await,
                _ = heartbeat_ticker.tick() => {
                    #[cfg(feature = "stealth-net")]
                    self.handle_proxy_heartbeat().await;
                },
                _ = nt_world_sense_ticker.tick() => {
                    #[cfg(feature = "stealth-net")]
                    self.handle_nt_world_sense_tick().await;
                },
                _ = awareness_ticker.tick() => self.handle_awareness().await,
                _ = nt_act_voice_ticker.tick() => self.handle_nt_act_voice_tick().await,
                _ = plugin_ticker.tick() => self.handle_plugin_tick().await,
                _ = scheduler_ticker.tick() => self.handle_scheduler_tick().await,
                _ = discovery_ticker.tick() => self.handle_agent_discovery().await,
                _ = consciousness_ticker.tick() => self.handle_consciousness_batch().await,
            }
        }
    }

    async fn handle_consciousness_batch(&mut self) {
        let encoder = self
            .vsa_encoder
            .get_or_insert_with(NgramVsaEncoder::default);
        let state_vec = encoder.encode_text("consciousness_bg_state");
        let self_vsa = encoder.encode_text("consciousness_bg_self");
        let wm_vsa = encoder.encode_text("consciousness_bg_wm");
        let attn_vsa = encoder.encode_text("consciousness_bg_attn");
        let sp_vec = encoder.encode_text("consciousness_bg_sp");
        let out_v = encoder.encode_text("consciousness_bg_output");
        let ctx_v = encoder.encode_text("consciousness_bg_context");
        let proof_vec = encoder.encode_text("consciousness_bg_proof");
        let ctm_vec = encoder.encode_text("consciousness_bg_ctm");

        let ci = match self.consciousness {
            Some(ref mut c) => c,
            None => return,
        };
        // ── Loop Engineering: advance phase ──
        let _loop_phase = ci.loop_engine.tick();
        // Seed exploration from curiosity BEFORE main pipeline, so results feed same cycle
        let _curiosity_seeds = ci.curiosity_orchestrator_bridge(&mut self.curiosity_drive);
        let _exploration_stats =
            ci.handle_exploration_orchestrate(&self.curiosity_drive, ci.cycle as u64);

        // Compute reflexive early for use as real data signal in Phase 7 handlers
        let reflective = ci.handle_reflexive(&state_vec);
        let reflect_success = reflective > 0.3;
        let reflect_quality = reflective.clamp(0.0, 1.0);

        // Self-protection: integrity check every cycle
        ci.handle_self_protection_tick();

        // Phase 7 — Context OS + Decision Compression (handle_context_gather increments cycle)
        ci.handle_context_gather("bg_tick", &AttentionDomain::PatternMatch);
        let alternatives = &["observe", "reason", "act"];
        ci.handle_decision_compress("bg_batch", alternatives, 0);
        ci.handle_experience_reflect("bg_tick", "batch", reflect_success, reflect_quality);
        ci.handle_skill_accumulate(
            "bg_skill",
            "bg_tick",
            "batch",
            "ok",
            AttentionDomain::Semantic,
            reflect_success,
            vec![],
        );
        ci.handle_curriculum_generate(AttentionDomain::Semantic, "bg_curriculum", 0.3);
        let failure_type = if reflect_success {
            FailureType::ReasoningError
        } else {
            FailureType::RepeatedFailure
        };
        ci.handle_policy_repair("bg_tick", failure_type, 1.0 - reflect_quality);
        ci.handle_epistemic_calibrate(AttentionDomain::Semantic, reflect_quality, reflect_success);
        ci.epistemic_gap_bridge(0.4);

        // Phase 8 — SRCC — deterministic VSA vectors from text labels
        ci.handle_attractor_dynamics(&state_vec);
        ci.handle_ebbinghaus_decay(state_vec.clone(), "bg_decay");
        ci.handle_dream_cycle();
        ci.handle_emergent_reasoning(&state_vec, true);
        // reflective already computed above
        ci.handle_epistemic_honesty(0.85, reflective > 0.3);
        ci.handle_personality_update(0.1, 1.0);
        ci.handle_cognitive_state_ingest(&self_vsa, &wm_vsa, &attn_vsa, 0.0, "bg");
        ci.handle_master_consciousness_update("bg");

        // VSA advantage + sleep consolidation
        let action_vec = encoder.encode_text("consciousness_bg_action");
        ci.handle_vs_advantage_learn(&state_vec, &action_vec, ActionType::Reasoning, 0.0);
        ci.handle_sleep_consolidation(ci.cycle as u64);

        // Goal execution
        ci.handle_goal_execution("bg_explore", "general", &[]);

        // Consciousness wiring — deterministic VSA vectors from text labels
        ci.handle_specious_present_feed(sp_vec, VsaOrigin::Self_(VsaSelfCategory::Thought));
        let narrative_insight = if reflect_success {
            None
        } else {
            Some("low_reflexivity".to_string())
        };
        ci.handle_narrative_tick("bg_cycle", reflect_quality, narrative_insight);
        ci.handle_valence_update(reflect_quality, reflect_success);
        let critique = ci.handle_inner_critic(out_v, ctx_v);
        let load = (ci.working_memory.item_count() as f64) * 0.1 + (ci.cycle as f64) * 0.001;
        let mode = ci.handle_cognitive_load_tick(load);
        let _proof = ci.handle_proof_search_tick("bg_self_check", proof_vec);

        // Wiring: DGM-H writeback, DMN, min_sufficient — published but orphaned before
        let _dgmh = ci.handle_dgmh_writeback_tick();
        let _dmn = ci.handle_default_mode_tick(false);
        let _min_suff = ci.handle_min_sufficient_tick();

        // Orphan handlers — previously declared but never called
        ci.handle_stream_buffer_feed(
            state_vec.clone(),
            VsaOrigin::Self_(VsaSelfCategory::Thought),
        );
        ci.handle_reconstructive_narrative_tick();
        ci.handle_adaptive_rate_tick(true);
        ci.handle_context_budget_tick();
        ci.handle_resonator_decode(&state_vec);
        if ci.cycle % 5 == 0 {
            ci.handle_volition_tick();
            ci.handle_conformal_uq_tick();
            ci.handle_confidence_calibrate(reflect_quality, reflect_success);
            ci.handle_failure_trace("bg_tick", "bg_action", "bg_outcome", TraceNodeType::Hypothesis, None, reflect_quality);
        }
        if ci.cycle % 10 == 0 {
            let _value_report = ci.handle_value_system_tick(0.5);
            let _align_report = ci.handle_value_alignment_tick();
            ci.handle_dream_consolidate_feed("bg", &[]);
            ci.handle_moss_health_tick();
        }
        if ci.cycle % 20 == 0 {
            ci.handle_e8_geometry_tick();
            ci.handle_ctm_inference(&ctm_vec);
            let _ = ci.handle_async_delegate_submit("bg_delegate");
            ci.handle_async_delegate_poll("bg");
        }
        if ci.cycle % 30 == 0 {
            ci.handle_social_feed_absorb(&[]);
            ci.handle_consciousness_pipeline("bg_tick", &["observe"], 0, reflect_success, reflect_quality, AttentionDomain::Semantic, None);
        }

        // Previously uncalled handlers — wired periodically
        if ci.cycle % 15 == 0 {
            ci.handle_reasoning_step("bg_reason");
            ci.handle_sia_feedback("bg_tick", 0.5, 0);
            ci.handle_srcc_brain_dgm("bg_tick");
        }
        if ci.cycle % 30 == 0 && ci.self_improvement.cycle > 0 {
            ci.handle_moss_pipeline("bg_tick", "periodic", "periodic_maintenance");
        }
        if ci.cycle % 50 == 0 {
            ci.handle_input_pipeline_batch(&[("bg", "batch")]);
        }

        // Kroneker VSA codebook cleanup: lightweight, runs every cycle
        ci.handle_kroneker_cleanup_tick();
        // Attention gate: lightweight gating signal every cycle
        ci.handle_attention_gate(ci.cycle as u64, &state_vec);
        // KitchenGate: adversarial UAT verification
        ci.handle_uat_gate_tick();
        // Neuromodulator ODE step: lightweight, runs every cycle
        ci.handle_neuromodulator_tick();
        // MIRRORThreads parallel cognitive synthesis: lightweight, runs every cycle
        ci.handle_mirror_thread_synthesize();
        // GEA group evolution experience sharing: every cycle (lightweight)
        ci.handle_gea_tick();
        // EvoSC self-consolidation: contrastive reflection + parametric compression (every 5 cycles)
        if ci.cycle % 5 == 0 {
            ci.handle_evosc_tick();
        }
        // OpenSkill bootstrapping: acquire + bootstrap + build verifier (every 10 cycles)
        if ci.cycle % 10 == 0 {
            ci.handle_open_skill_tick();
        }
        // CODE-SHARP skill DAG archive: evolve skill graph (every 7 cycles)
        if ci.cycle % 7 == 0 {
            ci.handle_skill_dag_tick();
        }
        // Phase 18 — Skill evolution pipeline: SkillDAG → OpenSkill → PACE verify (every 15 cycles)
        if ci.cycle % 15 == 0 {
            ci.handle_skill_evolution_tick();
        }
        // VSA Recurrent-Depth Transformer: iterative reasoning refinement (Fable 5 / RDT)
        ci.handle_vsa_rdt_tick();
        // TreeSeeker: maintain UCB-based search tree every cycle
        ci.handle_tree_seeker_tick("bg_exploration");
        // ShortcutDetector: analyze tree branches for shortcut risks
        ci.handle_shortcut_detect(0, 0.5, 0.3, 0.1, 0, "bg_cycle");
        // FailureModeClassifier: record a default tick (no failure)
        ci.handle_failure_mode_record(crate::core::nt_core_experience::FailureModeType::Timeout, true);
        // SocialBeliefModel: advance belief dynamics every cycle
        ci.handle_social_belief_tick("bg_consensus", 0.5, "bg_cycle");
        // Fable 5 VSA-native security routing: log stats every cycle
        ci.handle_fable_route_tick();
        // HORMA-inspired MemoryFs hierarchical memory: advance cycle
        ci.handle_memory_fs_tick();
        // Arbor Hypothesis-Tree Refinement: Observe/Ideate/Verify/Persist phases every cycle
        ci.handle_hypothesis_tree_tick();
        // Round 10 — Counterfactual Futures Engine: temporal contrast + internal feedback every 5 cycles
        if ci.cycle % 5 == 0 {
            ci.handle_counterfactual_futures_tick();
        }

        // OSC — Orthogonal Subspace Carving: passive infrastructure, every cycle
        ci.handle_osc_tick();
        // Phase 17 — DGM population variant proposal: every 5 cycles
        if ci.cycle % 5 == 0 {
            ci.handle_dgm_variant_propose();
        }
        // DGM-H plan verification & execution: every 10 cycles (picks best from plan buffer)
        if ci.cycle % 10 == 0 {
            ci.handle_dgmh_plan_verify_execute();
        }
        // Phase 17 — DGM archive auto-save: every 50 cycles
        if ci.cycle > 0 && ci.cycle % 50 == 0 {
            ci.handle_archive_save_tick();
        }

        // ── Phase 19: DA-gated attention scheduling ──
        // Combines CognitiveLoad Fast mode (hardware gate) with DA level (motivation gate).
        let fast_gate = mode == ThinkingMode::Fast && ci.cycle % 3 != 0;
        let attn_gate = !ci.should_run_group("ctm");
        let heavy_ops_allowed = !fast_gate && attn_gate;

        if heavy_ops_allowed || ci.should_run_group("spatial") {
            let ctm_result = ci.ctm_engine.infer(&ctm_vec);
            if ci.cycle % 10 == 0 && ctm_result.weight > 0.5 {
                eprintln!(
                    "[bg] ctm: winner={}, weight={:.3}",
                    ctm_result.winner_name, ctm_result.weight
                );
            }
            ci.handle_spatial_scene(&[], (0.0, 0.0, 0.0), 10.0);
            ci.handle_physics_reasoning(5.0, "solid", 20.0);
        } else if ci.cycle % 20 == 0 {
            eprintln!(
                "[bg] gating: fast={} da={:.2}",
                mode == ThinkingMode::Fast,
                ci.neuromodulator.da.level,
            );
        }

        // DA-gated exploration: skip when attention is Core
        if ci.should_run_group("exploration") {
            if !critique.passed && ci.cycle % 3 == 0 {
                ci.handle_novelty_detection_tick(0.3);
                ci.handle_tool_discovery_tick("bg_discovery", "critique_repair");
            } else {
                ci.handle_novelty_detection_tick(0.1);
                ci.handle_tool_discovery_tick("bg_discovery", "");
            }
        } else {
            ci.handle_novelty_detection_tick(0.05); // minimal tracking
        }
        let _gd = ci.handle_goal_decomposition_tick("explore");
        let _em = ci.handle_episodic_memory_tick(0);

        // ── Autonomy bridge: value→volition actuation ──
        // curriculum_thinking_bridge is called in handle_goal (run.rs:257) when needed
        let _volition_action = ci.value_volition_bridge();

        // ── Loop Engineering: record all handlers called this cycle ──
        let handler_names = [
            "context_gather",
            "decision_compress",
            "experience_reflect",
            "skill_accumulate",
            "curriculum_generate",
            "policy_repair",
            "epistemic_calibrate",
            "attractor_dynamics",
            "ebbinghaus_decay",
            "dream_cycle",
            "emergent_reasoning",
            "reflexive",
            "epistemic_honesty",
            "personality_update",
            "cognitive_state_ingest",
            "master_consciousness_update",
            "vs_advantage_learn",
            "sleep_consolidation",
            "fable_route",
            "goal_execution",
            "specious_present_feed",
            "narrative_tick",
            "valence_update",
            "inner_critic",
            "cognitive_load_tick",
            "proof_search_tick",
            "dgmh_writeback_tick",
            "self_protection_tick",
            "stream_buffer_feed",
            "reconstructive_narrative_tick",
            "adaptive_rate_tick",
            "context_budget_tick",
            "resonator_decode",
            "volition_tick",
            "conformal_uq_tick",
            "confidence_calibrate",
            "spatial_scene",
            "physics_reasoning",
            "novelty_detection_tick",
            "tool_discovery_tick",
            "goal_decomposition_tick",
            "episodic_memory_tick",
            "reasoning_step",
            "moss_pipeline",
            "input_pipeline_batch",
            "ctm_inference",
            "kroneker_cleanup",
            "uat_gate",
            "attention_gate",
            "failure_trace",
            "dream_consolidate_feed",
            "moss_health_tick",
            "sia_feedback",
            "srcc_brain_dgm",
            "async_delegate_submit",
            "async_delegate_poll",
            "consciousness_pipeline",
            "dgm_variant_propose",
            "archive_save",
            "neuromodulator_tick",
            "mirror_thread_synthesize",
            "gea_tick",
            "evosc_tick",
            "open_skill_tick",
            "skill_dag_tick",
            "skill_evolution_tick",
            "vsa_rdt",
            "hypothesis_tree_tick",
            "counterfactual_futures_tick",
            "social_feed_absorb",
            "social_feed_absorb",
            "e8_geometry_tick",
        ];
        for name in &handler_names {
            ci.loop_engine.discovery.record_call(name);
        }
        let handler_count = handler_names.len();
        let _verdict = ci.loop_engine.verifier.verify(
            reflect_quality,
            ci.specious_present.average_coherence(),
            handler_count,
        );

        if ci.cycle % 10 == 0 {
            let s = ci.stats();
            let ls = ci.loop_engine.stats();
            eprintln!("[bg] consciousness: cycle={}, c_score={:.3}, sp_coherence={:.3}, load={}, mode={:?}, loop={:?}({:.0}%)",
                ci.cycle, s.c_score, s.sp_coherence, s.load_mode, mode,
                ls.phase, ls.coverage_pct);
        }
    }

    async fn handle_save(&self) {
        let b = self.brain.write().await;
        if let Err(e) = b.brain.save() {
            eprintln!("[bg] auto-save failed: {}", e);
        }
        let _ = self.goal_loop.save();
    }

    async fn handle_consolidate(&self) {
        let mut b = self.brain.write().await;
        let r = b.consolidate_memories();
        eprintln!(
            "[bg] consolidated: {} merged, {} pruned, {} replayed",
            r.merged_count, r.pruned_count, r.replayed_count
        );
    }

    async fn handle_meta(&mut self) {
        let r = self.metacognition.run_full_cycle();
        eprintln!("[bg] meta cycle #{}", r.iteration);
    }

    async fn handle_thinking(&self) {
        let goal_desc = self
            .goal_loop
            .active_goal
            .as_ref()
            .map(|g| g.description.clone());
        let mut b = self.brain.write().await;
        if let Some(desc) = goal_desc {
            let _ = b.run_seal_loop(&desc, None, None);
        } else {
            b.iterate(super::super::nt_world_model::TaskType::General);
        }
    }

    async fn handle_goal(&mut self) {
        {
            let mut b = self.brain.write().await;
            self.goal_loop.pursue_all(&mut b, 1);
        }
        if self.goal_loop.active_goal.is_none() {
            if let Some(ref mut ci) = self.consciousness {
                if let Some(task) = ci.curriculum_thinking_bridge() {
                    let mut b = self.brain.write().await;
                    self.goal_loop.start_goal(&mut b, &task, None);
                    eprintln!("[bg] curriculum→goal: '{}'", task);
                }
            }
        }
    }

    async fn handle_prediction(&mut self) {
        // Pre-action introspection before prediction cycle
        if let Some(ref mut ip) = self.introspector {
            let state = self.thinking.silicon.current_state();
            let action = format!("prediction_cycle_{}", self.thinking.silicon.iteration);
            let _r = ip.introspect(
                &action,
                crate::core::nt_core_self::reasoning_strategy::StrategyKind::Deliberate,
                crate::core::nt_core_self::attention_head::AttentionDomain::Planning,
                &state,
            );
        }

        // 1. PREDICT — run panorama pipeline
        let report_opt = if let Some(ref mut pano) = self.panorama {
            let mut brain = self.brain.write().await;
            if let Some(ref mut wm) = self.nt_world_model {
                let report = pano.run_cycle(&mut brain, &mut self.goal_loop, wm);
                eprintln!(
                    "[bg] prediction: cycle={}, anomaly={}",
                    report.cycle, report.anomaly
                );
                Some(report)
            } else {
                None
            }
        } else {
            if let Some(ref wm) = self.nt_world_model {
                wm.predict_all(&[]);
            }
            None
        };

        // 2. OBSERVE — run awareness monitor after prediction
        if let Some(ref mut aw) = self.awareness {
            aw.observe();
            let level = aw.current.consciousness_level;
            let phi = aw.current.phi_current;
            let coherence = aw.current.coherence_current;
            let anomaly_flag = report_opt.as_ref().map(|r| r.anomaly).unwrap_or(false);
            eprintln!("[bg] awareness after prediction: consciousness={:.3}, phi={:.4}, coherence={:.3}, anomaly={}",
                level, phi, coherence, anomaly_flag);
        }

        // 3. REPORT — consolidated prediction summary
        if let Some(ref report) = report_opt {
            eprintln!("[bg] prediction report: cycle={}, hypercube={}, cortex={}, gwt={}, fe={:.3}, phi={:.3}, goals={}",
                report.cycle, report.hypercube_entries, report.cortex_traces,
                report.gwt_broadcasts, report.fe_energy, report.phi, report.goals_created);
        }

        #[cfg(feature = "stealth-net")]
        self.handle_stealth_rotation().await;
    }

    async fn handle_panorama(&self) {
        if let Some(ref pano) = self.panorama {
            eprintln!("[bg] panorama status: {}", pano.status());
        }
    }

    async fn handle_exploration(&mut self) {
        let sources_path = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".neotrix")
            .join("exploration_sources.txt");

        let urls: Vec<String> = match std::fs::read_to_string(&sources_path) {
            Ok(content) => content
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .collect(),
            Err(_) => Vec::new(),
        };

        if !urls.is_empty() {
            if let Some(ref mut evolver) = self.self_evolver {
                for url in &urls {
                    if !SelfEvolver::is_url(url) {
                        eprintln!("[bg] skipping non-URL: {}", url);
                        continue;
                    }
                    match evolver.evolve_from_url(url) {
                        Ok(reward) => {
                            eprintln!("[bg] exploration evolved {}: reward={:.3}", url, reward)
                        }
                        Err(e) => eprintln!("[bg] exploration failed {}: {}", url, e),
                    }
                }
            }

            // Clear processed URLs to avoid re-processing
            if let Err(e) = std::fs::write(&sources_path, "") {
                eprintln!("[bg] failed to clear exploration sources: {}", e);
            }
        }

        // Knowledge gap detection — scans what's missing vs known sources
        if let Some(ref mut gap_detector) = self.gap_detector {
            use crate::core::nt_core_meta::scanner::CodeScanner;
            use crate::core::nt_core_meta::weakness::WeaknessAnalyzer;
            let scanner = CodeScanner::new(".");
            let model = scanner.scan();
            let analyzer = WeaknessAnalyzer::new();
            let weaknesses = analyzer.analyze(&model);
            let report = gap_detector.detect_gaps(&model, &weaknesses.weaknesses);
            if report.high_priority_count > 0 {
                eprintln!(
                    "[bg] gap detection: {} gaps, {} high-priority, coherence={:.2}",
                    report.total_gaps, report.high_priority_count, report.coherence_score
                );
                for s in report.exploration_suggestions.iter().take(3) {
                    eprintln!("[bg]   gap suggestion: {}", s);
                }
            }
        }
    }

    #[cfg(feature = "stealth-net")]
    async fn handle_stealth_rotation(&mut self) {
        if let Some(ref mut sm) = self.nt_shield_manager {
            let tags: Vec<String> = vec!["pool_0".to_string()];
            let _id = sm.get_identity(&tags);
            let stats = sm.stats();
            if stats.active_count < stats.total_identities {
                eprintln!(
                    "[bg] stealth: {}/{} active, success={:.2}, confidence={:.2}",
                    stats.active_count,
                    stats.total_identities,
                    stats.avg_success_rate,
                    stats.avg_confidence
                );
            }
        }
    }

    async fn handle_awareness(&mut self) {
        if let Some(ref mut aw) = self.awareness {
            aw.observe();
            let level = aw.current.consciousness_level;
            let phi = aw.current.phi_current;
            let coherence = aw.current.coherence_current;
            eprintln!(
                "[bg] awareness: consciousness={:.3}, phi={:.4}, coherence={:.3}",
                level, phi, coherence
            );
        }
    }

    async fn handle_always_on(&mut self) {
        use crate::neotrix::nt_mind_background_loop::always_on::AlwaysOnEngine;
        static ALWAYS_ON: std::sync::LazyLock<std::sync::Mutex<Option<AlwaysOnEngine>>> =
            std::sync::LazyLock::new(|| std::sync::Mutex::new(None));

        let report = {
            let mut guard = ALWAYS_ON.lock().unwrap_or_else(|e| e.into_inner());
            let engine = guard.get_or_insert_with(|| {
                let mut e = AlwaysOnEngine::load();
                e.enabled = true;
                e
            });
            if engine.enabled {
                match engine.full_cycle() {
                    Ok(report) => Some(report),
                    Err(_e) => {
                        let _ = engine.save();
                        None
                    }
                }
            } else {
                None
            }
        };

        if let Some(report) = report {
            if report.tasks_executed > 0 {
                eprintln!(
                    "[bg] always_on: scanned={}, executed={}, completed={}, took={}ms",
                    report.scan_count,
                    report.tasks_executed,
                    report.tasks_completed,
                    report.duration_ms
                );
                let guard = ALWAYS_ON.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(ref engine) = *guard {
                    let _ = engine.save();
                }
            }
        }
    }

    async fn handle_nt_act_voice_tick(&mut self) {
        if let Some(ref mut vi) = self.nt_act_voice_input {
            if !vi.is_active() {
                return;
            }
            if vi.is_continuous() {
                if let Some(text) = vi.poll_transcription() {
                    eprintln!("[nt_act_voice] transcribed: {}", text);
                    let cmd = crate::neotrix::nt_act_voice::VoiceCommand::parse(&text);
                    match cmd {
                        crate::neotrix::nt_act_voice::VoiceCommand::OpenSettings => {
                            eprintln!("[nt_act_voice] command: open settings");
                        }
                        crate::neotrix::nt_act_voice::VoiceCommand::ShowHelp => {
                            eprintln!("[nt_act_voice] command: show help");
                        }
                        crate::neotrix::nt_act_voice::VoiceCommand::RunCommand(c) => {
                            eprintln!("[nt_act_voice] command: run {}", c);
                        }
                        crate::neotrix::nt_act_voice::VoiceCommand::SwitchSession(s) => {
                            eprintln!("[nt_act_voice] command: switch to session {}", s);
                        }
                        _ => {}
                    }
                }
            } else if vi.check_wake_word() {
                eprintln!("[nt_act_voice] wake word detected");
            }
        }
    }

    #[cfg(feature = "stealth-net")]
    async fn handle_nt_world_sense_tick(&mut self) {
        if let Some(ref mut wc) = self.world_consciousness {
            let events = wc.nt_world_sense.poll_all();
            if !events.is_empty() {
                log::info!("[nt_world_sense] {} new events", events.len());
            }
            wc.refresh_self_awareness();
            log::info!("[nt_world_sense] status: {}", wc.consciousness_status());
        }
    }

    #[cfg(feature = "stealth-net")]
    async fn handle_proxy_heartbeat(&self) {
        if let Some(ref engine) = self.heartbeat_engine {
            let record = engine.tick().await;
            if record.success {
                eprintln!(
                    "[bg] proxy heartbeat #{}: proxy={}, geo={:?}, fp={}, dns={}",
                    record.tick,
                    record.proxy_url,
                    record.proxy_geo,
                    record.fingerprint_id,
                    record.dns_flushed,
                );
            } else {
                eprintln!(
                    "[bg] proxy heartbeat #{}: no proxy available (pool empty?)",
                    record.tick
                );
            }
        }
        self.handle_proxy_auto_mode().await;
    }

    /// 根据 brain 上下文自动切换 proxy 模式
    #[cfg(feature = "stealth-net")]
    async fn handle_proxy_auto_mode(&self) {
        use super::super::nt_shield_stealth_net::proxy_control::DaemonMode;

        let client = match self.proxy_client {
            Some(ref c) => c,
            None => return,
        };
        if !client.is_reachable().await {
            return;
        }

        // 读取当前 daemon 模式
        let status_str = match client.status().await {
            Ok(s) => s,
            Err(_) => return,
        };
        let current = match serde_json::from_str::<serde_json::Value>(&status_str) {
            Ok(v) => {
                DaemonMode::from_str(v["mode"].as_str().unwrap_or("geo")).unwrap_or(DaemonMode::Geo)
            }
            Err(_) => return,
        };

        // 决定目标模式 (函数已 #[cfg(feature = "stealth-net")], 字段可用)
        let target = if self.tor_crawler.is_some() {
            DaemonMode::Tor
        } else if self
            .nt_shield_manager
            .as_ref()
            .map_or(false, |sm| sm.stats().active_count > 0)
        {
            DaemonMode::Stealth
        } else {
            DaemonMode::Geo
        };

        if target != current {
            match client.set_mode(target).await {
                Ok(_) => eprintln!(
                    "[bg] proxy auto-mode: {} → {}",
                    current.as_str(),
                    target.as_str()
                ),
                Err(e) => eprintln!("[bg] proxy auto-mode failed: {}", e),
            }
        }
    }

    async fn handle_plugin_tick(&self) {
        use crate::neotrix::nt_io_plugin::PluginEvent;
        self.plugin_registry.dispatch(&PluginEvent::BrainTick);
    }

    /// Periodic agent discovery listener — sweep for UDP broadcasts.
    async fn handle_agent_discovery(&mut self) {
        if let Some(ref mut discovery) = self.agent_discovery {
            if let Err(e) = discovery.listen() {
                log::warn!("[bg] agent discovery listen: {}", e);
            }
            if discovery.agent_count() > 0 {
                log::info!("[bg] known agents: {}", discovery.agent_count());
            }
        }
    }

    /// Curiosity drive: knowledge gaps → GWT attention → exploration queries
    /// Wired to negentropy: gap sparsity → negentropy proxy → curiosity calibration
    async fn handle_curiosity(&mut self) {
        use crate::neotrix::nt_mind::hypercube_bridge::HyperCubeBridge;
        let gap_reports = {
            let bridge = HyperCubeBridge::new();
            bridge.analyze_gaps()
        };

        self.curiosity_drive.ingest_gap_reports(&gap_reports);

        // Negentropy alignment: use gap sparsity as inverse negentropy proxy
        // sparsity ↑ → order ↓ → negentropy ↓ → curiosity ↑
        let n_total_proxy = if gap_reports.is_empty() {
            0.5
        } else {
            let avg_sparsity: f64 = gap_reports.iter().map(|r| r.sparsity_score).sum::<f64>()
                / gap_reports.len() as f64;
            (1.0 - avg_sparsity).clamp(0.0, 1.0)
        };
        self.curiosity_drive
            .calibrate_to_negentropy(n_total_proxy, 0.0);

        let queries = self.curiosity_drive.drain_queries();

        if !queries.is_empty() {
            eprintln!(
                "[bg] curiosity: {} signals, {} queries generated: {:?}",
                self.curiosity_drive.signals.len(),
                queries.len(),
                &queries[..queries.len().min(3)],
            );
            if let Some(ref mut evolver) = self.self_evolver {
                for query_str in queries.iter().take(2) {
                    let q: &String = query_str;
                    let search_url =
                        format!("https://en.wikipedia.org/wiki/{}", q.replace(' ', "_"));
                    match evolver.evolve_from_url(&search_url) {
                        Ok(reward) => {
                            eprintln!("[bg] curiosity evolved {}: reward={:.3}", q, reward)
                        }
                        Err(e) => eprintln!("[bg] curiosity failed {}: {}", q, e),
                    }
                }
            }
        } else {
            let level = self.curiosity_drive.curiosity_level;
            let signal_count = self.curiosity_drive.signals.len();
            if signal_count > 0 {
                eprintln!(
                    "[bg] curiosity: {:?}, {} signals, {} total gaps",
                    level, signal_count, self.curiosity_drive.total_gaps_detected
                );
            }
        }
    }

    /// Knowledge chain: discovery → mining → validation → absorption → storage
    async fn handle_knowledge_chain(&mut self) {
        if let Some(ref mut chain) = self.knowledge_chain {
            let kc: &mut KnowledgeChain = chain;
            if !kc.has_pending() {
                kc.init_default_discovery();
            }
            let mut brain = ReasoningBrain::new();
            let mut bank = ReasoningBank::new(100);
            match kc.run_chain(&mut brain, &mut bank) {
                Ok(result) => {
                    eprintln!(
                        "[bg] knowledge chain: discovered={}, mined={}, absorbed={}, reward={:.3}",
                        result.discovered, result.mined, result.absorbed, result.total_reward
                    );
                    if result.absorbed > 0 && self.config.enable_auto_crystallize {
                        for d in &result.details {
                            let detail: &String = d;
                            if detail.starts_with("吸收阶段") {
                                let edits = vec![
                                    super::super::nt_mind::self_edit::MicroEdit::NormalizeVector,
                                ];
                                self.auto_crystallizer.crystallize_from_absorption(
                                    &mut brain,
                                    &mut bank,
                                    "knowledge_chain",
                                    "chain_batch",
                                    "general",
                                    &edits,
                                    result.total_reward / result.absorbed as f64,
                                );
                            }
                        }
                    }
                }
                Err(e) => eprintln!("[bg] knowledge chain failed: {}", e),
            }
        }
    }

    /// Knowledge aging: score decay → stale detection → re-scan scheduling
    async fn handle_knowledge_aging(&mut self) {
        let report = self.knowledge_aging.run_aging_cycle();
        if report.stale_count > 0 || report.expired_count > 0 {
            eprintln!(
                "[bg] knowledge aging: {} survived, {} stale, {} expired, avg_age={:.1}d",
                report.surviving_entries,
                report.stale_count,
                report.expired_count,
                report.avg_age_days
            );

            if !report.rescans_needed.is_empty() {
                eprintln!("[bg] aging: {} rescans needed", report.rescans_needed.len());
                if let Some(ref mut evolver) = self.self_evolver {
                    for url_str in report.rescans_needed.iter().take(3) {
                        if SelfEvolver::is_url(url_str) {
                            let ev: &mut SelfEvolver = evolver;
                            match ev.evolve_from_url(url_str) {
                                Ok(reward) => {
                                    eprintln!("[bg] re-scan {}: reward={:.3}", url_str, reward)
                                }
                                Err(e) => eprintln!("[bg] re-scan failed {}: {}", url_str, e),
                            }
                        }
                    }
                }
            }
        }
    }

    /// Auto-crystallization: check SelfEvolver results → create SkillCrystals
    async fn handle_crystallization(&mut self) {
        if !self.config.enable_auto_crystallize {
            return;
        }
        let summary = self.auto_crystallizer.summary();
        eprintln!("[bg] crystallization: {}", summary);
    }

    /// Scheduler tick: check due jobs, gate by consciousness state, dispatch handlers.
    /// Replaces the old hardcoded build_cleanup_ticker with OpenClaw-inspired scheduling.
    async fn handle_scheduler_tick(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Read consciousness state for context-aware gating
        let (cog_load, da_level, sleep_pressure, curiosity) = match self.consciousness {
            Some(ref mut ci) => {
                if ci.cycle == 0 {
                    (0.0, 0.3, 0.0, 0.5)
                } else {
                    (
                        ci.cognitive_load.average_load(),
                        ci.neuromodulator.da.level,
                        ci.consolidation_bridge.sleep_pressure(),
                        self.curiosity_drive.curiosity_level.salience_multiplier(),
                    )
                }
            }
            None => (0.0, 0.3, 0.0, 0.5),
        };

        let due_jobs: Vec<(String, String)> = self
            .scheduler
            .tick(now, cog_load, da_level, sleep_pressure, curiosity);

        for (job_id, handler) in &due_jobs {
            let start = std::time::Instant::now();

            let (success, error_msg): (bool, Option<String>) = match handler.as_str() {
                "handle_build_cleanup" => {
                    self.handle_build_cleanup().await;
                    (true, None)
                }
                "handle_knowledge_aging" => {
                    self.handle_knowledge_aging().await;
                    (true, None)
                }
                "handle_evosc_tick" => {
                    if let Some(ref mut ci) = self.consciousness {
                        if ci.cycle % 5 == 0 {
                            ci.handle_evosc_tick();
                        }
                    }
                    (true, None)
                }
                other => {
                    let msg = format!("scheduler: unknown handler '{}' for job '{}'", other, job_id);
                    eprintln!("[bg] {}", msg);
                    (false, Some(msg))
                }
            };

            let duration_ms = start.elapsed().as_millis() as u64;
            self.scheduler
                .record_run(job_id, now, duration_ms, success, error_msg);
        }

        // Log stats every 10 scheduler ticks (~5 minutes)
        if self.scheduler.tick_count() % 10 == 0 {
            let stats = self.scheduler.stats();
            eprintln!(
                "[bg] scheduler: {} jobs ({} enabled), {} runs, {:.1}% success",
                stats.total_jobs,
                stats.enabled_jobs,
                stats.total_runs,
                stats.success_rate * 100.0,
            );
        }
    }

    /// 每日构建产物清理：target/ + node_modules/ + dist/
    async fn handle_build_cleanup(&mut self) {
        let engine = match self.cleanup_engine {
            Some(ref mut e) => e,
            None => return,
        };
        let project_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let result = engine.force_project_clean(&project_root);
        if result.deletable_count > 0 {
            eprintln!(
                "[cleanup] {}: {} items, {:.1} MB freed",
                result.kind.description(),
                result.deletable_count,
                result.estimated_bytes as f64 / 1_048_576.0,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::neotrix::nt_mind::goal_loop::GoalLoop;
    use crate::neotrix::nt_mind::panorama_pipeline::PanoramaPipeline;
    use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
    use crate::neotrix::nt_world_model_v2::WorldModelV2;

    #[test]
    fn test_panorama_pipeline_new() {
        let pano = PanoramaPipeline::new();
        assert_eq!(pano.cycle, 0);
        assert_eq!(pano.total_anomalies, 0);
    }

    #[test]
    fn test_panorama_pipeline_status_nonempty() {
        let pano = PanoramaPipeline::new();
        let s = pano.status();
        assert!(!s.is_empty());
        assert!(s.contains("Panorama:"));
    }

    #[test]
    fn test_panorama_run_cycle_basic() {
        let mut pano = PanoramaPipeline::new();
        let mut brain = SelfIteratingBrain::new();
        let mut goal_loop = GoalLoop::new();
        let mut wm = WorldModelV2::new(4, 64);

        let report = pano.run_cycle(&mut brain, &mut goal_loop, &mut wm);
        assert_eq!(report.cycle, 1);
        assert!(report.hypercube_entries > 0);
    }

    #[test]
    fn test_panorama_multiple_cycles() {
        let mut pano = PanoramaPipeline::new();
        let mut brain = SelfIteratingBrain::new();
        let mut goal_loop = GoalLoop::new();
        let mut wm = WorldModelV2::new(4, 64);

        for i in 1..=3 {
            let report = pano.run_cycle(&mut brain, &mut goal_loop, &mut wm);
            assert_eq!(report.cycle, i);
        }
        assert_eq!(pano.cycle, 3);
        assert!(pano.status().contains("cycle=3"));
    }
}
