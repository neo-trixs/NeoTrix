use super::*;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::neotrix::l8_autonomic_impl::nt_mind_cleanup::{CleanupEngine, CleanupKind, BackupEngine};
use crate::neotrix::l8_autonomic_impl::nt_mind_skill_engine::SkillEngine;
use crate::neotrix::l8_autonomic_impl::nt_mind_knowledge_pipeline::KnowledgeAbsorptionPipeline;
use crate::neotrix::l1_body_impl::nt_io_session_recovery::SessionRecoveryManager;
use crate::neotrix::nt_core_event_bus::{EventBus, subscribe_all_layers_sync};
use crate::neotrix::nt_mind::distillation::MetaCognitionBridge;

impl BackgroundLoop {
    /// Spawn all background handlers as independent tokio tasks.
    /// Each handler runs in its own loop with its own ticker.
    /// Replaces single `tokio::select!` which blocked when any handler was slow.
    pub async fn start(&mut self) {
        if !self.config.enabled { return; }

        self.started = true;

        // ── Create shutdown coordinator ──
        let (coordinator, shutdown_rx) = ShutdownCoordinator::new();
        self.shutdown_coordinator = Some(coordinator);

        // ── Create EventBus and subscribe all 9 layer subscribers ──
        let event_bus = Arc::new(EventBus::new(1024));
        subscribe_all_layers_sync(&event_bus);

        // Wrap self so each spawned task gets its own reference.
        let cleanup_engine = self.cleanup_engine.take();

        let this = Arc::new(Mutex::new(BackgroundLoopHandle {
            brain: self.brain.clone(),
            cleanup_engine,
            goal_loop: std::mem::take(&mut self.goal_loop),
            awareness: self.awareness.take(),
            gold_standard: self.gold_standard.take(),
            introspector: self.introspector.take(),
            gap_detector: self.gap_detector.take(),
            nt_act_voice_input: self.nt_act_voice_input.take(),
            avatar_engine: self.avatar_engine.take(),
            self_evolver: self.self_evolver.take(),
            curiosity_drive: std::mem::take(&mut self.curiosity_drive),
            knowledge_aging: std::mem::take(&mut self.knowledge_aging),
            auto_crystallizer: std::mem::take(&mut self.auto_crystallizer),
            knowledge_chain: self.knowledge_chain.take(),
            exploration_pipeline: self.exploration_pipeline.take(),
            always_on: std::mem::take(&mut self.always_on),
            plugin_registry: std::mem::take(&mut self.plugin_registry),
            config: self.config.clone(),
            agent_discovery: self.agent_discovery.take(),
            panorama: self.panorama.take(),
            nt_world_model: self.nt_world_model.take(),
            scheduler: self.scheduler.take(),
            daemon: self.daemon.take(),
            skill_engine: SkillEngine::new(PathBuf::from(
                &dirs::home_dir().unwrap_or_default().join(".claude").join("skills"),
            )),
            kb_pipeline: KnowledgeAbsorptionPipeline::new(),
            session_recovery: self.session_recovery.take(),
            event_bus: Some(event_bus.as_ref().clone()),
            metacognition: self.metacognition.take(),
            #[cfg(feature = "stealth-net")]
            world_consciousness: self.world_consciousness.take(),
            #[cfg(not(feature = "stealth-net"))]
            world_consciousness: None,
            consciousness_runtime: std::mem::take(&mut self.consciousness_runtime),
        }));

        macro_rules! spawn_handler {
            ($interval:expr, $lock:ident, $body:expr) => {{
                let h = this.clone();
                let mut rx = shutdown_rx.clone();
                self.handles.push(tokio::spawn(async move {
                    let mut ticker = tokio::time::interval(
                        tokio::time::Duration::from_secs($interval));
                    loop {
                        tokio::select! {
                            biased;
                            _ = ticker.tick() => {
                                let mut $lock = h.lock().await;
                                $body;
                            }
                            _ = rx.changed() => {
                                log::trace!("[bg] handler shutting down (interval={})", $interval);
                                break;
                            }
                        }
                    }
                }));
            }};
            ($interval:expr, |$lock:ident| $body:expr) => {
                spawn_handler!($interval, $lock, $body);
            };
        }

        // ── Each handler is an independent task with its own ticker ──
        let cfg = self.config.clone();
        spawn_handler!(cfg.save_interval_secs, |h| h.handle_save().await);
        spawn_handler!(cfg.consolidate_interval_secs, |h| h.handle_consolidate().await);
        spawn_handler!(cfg.goal_interval_secs, |h| h.handle_goal().await);
        spawn_handler!(cfg.knowledge_chain_interval_secs, |h| h.handle_knowledge_chain().await);
        spawn_handler!(cfg.knowledge_aging_interval_secs, |h| h.handle_knowledge_aging().await);
        spawn_handler!(cfg.crystallization_interval_secs, |h| h.handle_crystallization().await);
        spawn_handler!(cfg.nt_act_voice_interval_secs, |h| h.handle_nt_act_voice_tick().await);
        spawn_handler!(cfg.plugin_interval_secs, |h| h.handle_plugin_tick().await);
        spawn_handler!(cfg.exploration_interval_secs, |h| h.handle_exploration().await);
        spawn_handler!(cfg.curiosity_interval_secs, |h| h.handle_curiosity().await);
        spawn_handler!(cfg.world_prediction_interval_secs, |h| h.handle_prediction().await);
        spawn_handler!(cfg.metacog_interval_secs, |h| h.handle_awareness().await);
        spawn_handler!(cfg.cleanup_interval_secs, |h| h.handle_cleanup().await);
        spawn_handler!(21_600, |h| h.handle_backup().await); // every 6h
        spawn_handler!(60, |h| h.handle_agent_discovery().await);
        spawn_handler!(120, |h| h.handle_always_on().await);
        spawn_handler!(cfg.scheduler_interval_secs, |h| h.handle_scheduler_tick().await);
        spawn_handler!(cfg.evolution_interval_secs, |h| h.handle_evolve().await);
        spawn_handler!(cfg.nt_world_sense_interval_secs, |h| h.handle_world_sense().await);
        spawn_handler!(3600, |h| h.handle_skill_scan().await);
        spawn_handler!(600, |h| h.handle_avatar_auto_distill().await);
        spawn_handler!(7200, |h| h.handle_kb_absorb().await);
        spawn_handler!(600, |h| h.handle_session_recovery().await);
        spawn_handler!(5, |h| h.handle_consciousness_tick().await);

        if self.agent_discovery.is_some() {
            let server = Arc::new(
                crate::neotrix::nt_agent_protocol::tcp_server::AgentServer::new(42070));
            self.handles.push(tokio::spawn(async move {
                match server.start().await {
                    Ok(port) => log::info!("[bg] AgentServer listening on TCP :{}", port),
                    Err(e) => log::error!("[bg] AgentServer start failed: {}", e),
                }
            }));
        }

        println!("[bg] {} handlers spawned", self.handles.len());
    }
}

/// Lightweight inner state for concurrent handler access.
struct BackgroundLoopHandle {
    brain: Arc<RwLock<SelfIteratingBrain>>,
    cleanup_engine: Option<CleanupEngine>,
    config: BackgroundConfig,
    goal_loop: GoalLoop,
    awareness: Option<ConsciousnessMonitor>,
    gold_standard: Option<ConsciousnessGoldStandard>,
    #[allow(dead_code)] // kept for future pre-action introspection feature
    introspector: Option<PreActionIntrospector>,
    gap_detector: Option<KnowledgeGapDetector>,
    nt_act_voice_input: Option<VoiceInput>,
    avatar_engine: Option<std::sync::Mutex<crate::neotrix::l1_body_impl::nt_io_user_avatar::DistillationEngine>>,
    self_evolver: Option<SelfEvolver>,
    curiosity_drive: CuriosityDrive,
    knowledge_aging: KnowledgeAging,
    auto_crystallizer: AutoCrystallizer,
    knowledge_chain: Option<KnowledgeChain>,
    #[allow(dead_code)] // kept for future autonomous exploration feature
    exploration_pipeline: Option<ExplorationPipeline>,
    always_on: AlwaysOnEngine,
    plugin_registry: PluginRegistry,
    agent_discovery: Option<crate::neotrix::nt_agent_protocol::discovery::AgentDiscovery>,
    panorama: Option<PanoramaPipeline>,
    nt_world_model: Option<WorldModelV2>,
    scheduler: Option<crate::core::nt_core_scheduler::SchedulerEngine>,
    daemon: Option<EvolutionDaemon>,
    skill_engine: SkillEngine,
    kb_pipeline: KnowledgeAbsorptionPipeline,
    session_recovery: Option<SessionRecoveryManager>,
    event_bus: Option<EventBus>,
    metacognition: Option<MetaCognitionBridge>,
    world_consciousness: Option<crate::neotrix::nt_world_sense::WorldConsciousness>,
    consciousness_runtime: Option<crate::core::nt_core_consciousness::consciousness_runtime::ConsciousnessRuntime>,
}

impl BackgroundLoopHandle {
    async fn handle_save(&mut self) {
        let b = self.brain.read().await;
        if let Err(e) = b.brain.save() { eprintln!("[bg] save: {}", e); }
        if let Err(e) = self.goal_loop.save() { log::warn!("[background] save goal_loop: {}", e); }
    }
    async fn handle_consolidate(&mut self) {
        let mut b = self.brain.write().await;
        let r = b.consolidate_memories();
        eprintln!("[bg] consolidated: {} merge, {} prune, {} replay",
            r.merged_count, r.pruned_count, r.replayed_count);
        // Persist any pending memory orchestrator entries to KB.
        // Currently a no-op until a DualTrackEntry producer is wired in;
        // the connector is established here for future OMP integration.
        let kb = b.reasoning_engine.as_mut().and_then(|e| e.kb.take());
        if let Some(kb_inner) = kb {
            let count = b.persist_pending_entries(&kb_inner);
            if count > 0 {
                eprintln!("[bg] persisted {} memory orchestrator entries", count);
            }
            if let Some(ref mut engine) = b.reasoning_engine {
                engine.kb = Some(kb_inner);
            }
        }
    }
    async fn handle_goal(&mut self) {
        let mut b = self.brain.write().await;
        self.goal_loop.pursue_all(&mut b, 1);
    }
    async fn handle_prediction(&mut self) {
        if let Some(ref mut pano) = self.panorama {
            let mut brain = self.brain.write().await;
            if let Some(ref mut wm) = self.nt_world_model {
                let r = pano.run_cycle(&mut brain, &mut self.goal_loop, wm);
                eprintln!("[bg] prediction: cycle={}, anomaly={}", r.cycle, r.anomaly);
            }
        }
    }
    async fn handle_exploration(&mut self) {
        let p = dirs::home_dir().unwrap_or_default().join(".neotrix").join("exploration_sources.txt");
        let urls: Vec<String> = std::fs::read_to_string(&p).unwrap_or_default()
            .lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty() && !l.starts_with('#')).collect();
        if !urls.is_empty() {
            if let Some(ref mut ev) = self.self_evolver {
                for url in &urls { let _ = ev.evolve_from_url(url); }
            }
            let _ = std::fs::write(&p, "");
        }
        if let Some(ref mut gd) = self.gap_detector {
            use crate::core::nt_core_meta::scanner::CodeScanner;
            use crate::core::nt_core_meta::weakness::WeaknessAnalyzer;
            let m = CodeScanner::new(".").scan();
            let w = WeaknessAnalyzer::new().analyze(&m);
            let r = gd.detect_gaps(&m, &w.weaknesses);
            if r.high_priority_count > 0 {
                eprintln!("[bg] gaps: {} total, {} high", r.total_gaps, r.high_priority_count);
            }
        }
    }
    async fn handle_awareness(&mut self) {
        // MetaCognitionBridge: full scan → analyze → plan cycle (P0 dead infra fix)
        if let Some(ref mut mc) = self.metacognition {
            let result = mc.run_full_cycle();
            eprintln!("[bg] metacognition: iter={} modules={} plans={} alerts={}",
                result.iteration,
                result.model_snapshot.modules.len(),
                result.plans.len(),
                result.alerts.len(),
            );
        }

        if let Some(ref mut aw) = self.awareness {
            aw.observe();
            let phi = aw.current.phi_current;
            let coherence = aw.current.coherence_current;
            let level = aw.current.consciousness_level;
            let health = aw.current.health;
            let is_conscious = level >= 0.7;
            eprintln!("[bg] awareness: l={:.3}, phi={:.4}, coh={:.4}", level, phi, coherence);
            let tier_label = if level >= 0.85 { "transcendent" }
                else if level >= 0.7 { "conscious" }
                else if level >= 0.4 { "awakening" }
                else { "dormant" };

            // Evaluate gold standard (dual-threshold IIT Phi + Kuramoto coherence)
            let gs_report = self.gold_standard.as_mut().map(|gs| {
                let state = &[phi, coherence, level, health];
                let r = gs.evaluate(state, &[]);
                eprintln!("[bg] gold_standard: phi={:.4} coh={:.4} conscious={} streak={}",
                    r.phi, r.coherence, r.is_conscious_like, r.detection_streak);
                r
            });

            // Persist consciousness snapshot + gold standard to KB
            if let Ok(brain) = self.brain.try_read() {
                if let Some(ref kb) = brain._nt_memory_kb {
                    let mut details = format!("level={:.3} tier={}", level, tier_label);
                    if let Some(ref gs) = gs_report {
                        details.push_str(&format!(
                            " | gs_phi={:.4} gs_coh={:.4} gs_conscious={} gs_streak={} gs_confidence={:.3}",
                            gs.phi, gs.coherence, gs.is_conscious_like, gs.detection_streak, gs.combined_confidence,
                        ));
                    }

                    // Persist full PhiReport to kv_store
                    if let Some(ref pr) = aw.last_phi_report {
                        let phi_json = serde_json::json!({
                            "phi": pr.phi,
                            "phi_raw": pr.phi_raw,
                            "total_resonance": pr.total_resonance,
                            "state_energy": pr.state_energy,
                            "effective_dims": pr.effective_dims,
                            "max_resonance_pair_0": pr.max_resonance_pair.0,
                            "max_resonance_pair_1": pr.max_resonance_pair.1,
                            "phi_trend": pr.phi_trend,
                            "is_conscious_like": pr.is_conscious_like,
                        });
                        let _ = kb.kv_set("consciousness", "phi_report", &phi_json.to_string());
                    }

                    // Persist full GoldStandardReport
                    if let Some(ref gs) = gs_report {
                        let gs_json = serde_json::json!({
                            "phi": gs.phi,
                            "coherence": gs.coherence,
                            "is_conscious_like": gs.is_conscious_like,
                            "is_phi_conscious": gs.is_phi_conscious,
                            "is_coherent": gs.is_coherent,
                            "phi_confidence": gs.phi_confidence,
                            "coherence_confidence": gs.coherence_confidence,
                            "detection_streak": gs.detection_streak,
                            "combined_confidence": gs.combined_confidence,
                        });
                        let _ = kb.kv_set("consciousness", "gold_standard", &gs_json.to_string());
                    }

                    // Persist trends (phi_trend, coherence_trend, health_trend)
                    let trends_json = serde_json::json!({
                        "phi_trend": aw.trends.phi_trend,
                        "coherence_trend": aw.trends.coherence_trend,
                        "health_trend": aw.trends.health_trend,
                    });
                    let _ = kb.kv_set("consciousness", "trends", &trends_json.to_string());

                    // Persist conversation awareness
                    let conv = &aw.current.conversation_awareness;
                    let conv_json = serde_json::json!({
                        "turn_count": conv.turn_count,
                        "stage": conv.stage.label(),
                        "topic_coherence": conv.topic_coherence,
                        "user_engagement": conv.user_engagement,
                        "topic_drift": conv.topic_drift,
                        "self_assessed_quality": conv.self_assessed_quality,
                        "depth_trend": conv.depth_trend,
                    });
                    let _ = kb.kv_set("consciousness", "conversation", &conv_json.to_string());

                    // Persist blind spots
                    let spots: Vec<serde_json::Value> = aw.current.active_blind_spots.iter().map(|b| {
                        serde_json::json!({
                            "kind": b.kind,
                            "severity": b.severity,
                            "description": b.description,
                            "repair": b.repair,
                        })
                    }).collect();
                    let spots_json = serde_json::json!({ "blind_spots": spots });
                    let _ = kb.kv_set("consciousness", "blind_spots", &spots_json.to_string());

                    // L6 Self intra-reflection: analyze reasoning quality
                    if let Some(ref engine) = brain.reasoning_engine {
                        let trace: Vec<String> = engine.state_trajectory.iter()
                            .map(|s| format!("{:?}", s)).collect();
                        if !trace.is_empty() {
                            let input = crate::neotrix::l6_self_impl::nt_core_intra_reflection::ReflectionInput {
                                reasoning_trace: trace,
                                e8_mode_history: Vec::new(),
                                execution_time_ms: 0,
                                error_count: 0,
                                outcome_success: Some(phi > 0.3),
                            };
                            let report = crate::neotrix::l6_self_impl::nt_core_intra_reflection::analyze(&input);
                            let ir_json = serde_json::json!({
                                "coherence_score": report.coherence_score,
                                "efficiency_score": report.efficiency_score,
                                "error_density": report.error_density,
                                "mode_stability": report.mode_stability,
                                "bottlenecks": report.bottleneck_hops,
                                "suggestions": report.suggestions,
                            });
                            let _ = kb.kv_set("self", "intra_reflection", &ir_json.to_string());
                            if !report.bottleneck_hops.is_empty() {
                                log::warn!("[bg] L6 intra-reflection: {} bottlenecks: {:?}",
                                    report.bottleneck_hops.len(), report.bottleneck_hops);
                            }
                        }
                    }

                    // Legacy snapshot for timeline view
                    let _ = kb.record_consciousness_snapshot(phi, coherence, is_conscious, tier_label, &details);
                }
            }
        }

        // Emit awareness tick event on the EventBus
        if let Some(ref bus) = self.event_bus {
            bus.emit(crate::core::nt_core_event::CoreEvent::TaskSubmitted {
                task: "awareness_tick".into(),
                task_type: "consciousness".into(),
                priority: 1,
            });
        }
    }
    async fn handle_world_sense(&mut self) {
        if let Some(ref mut wc) = self.world_consciousness {
            wc.refresh_self_awareness();
            eprintln!("[bg] world_sense: active={} status={}", wc.active, wc.consciousness_status().len());
        }
    }
    async fn handle_always_on(&mut self) {
        if self.always_on.enabled {
            if let Ok(r) = self.always_on.full_cycle() {
                if r.tasks_executed > 0 {
                    eprintln!("[bg] always_on: scanned={}, done={}", r.scan_count, r.tasks_executed);
                    if let Err(e) = self.always_on.save() { log::warn!("[background] save always_on: {}", e); }
                }
            }
        }
    }
    async fn handle_nt_act_voice_tick(&mut self) {
        if let Some(ref mut vi) = self.nt_act_voice_input {
            if vi.is_active() && vi.is_continuous() {
                if let Some(t) = vi.poll_transcription() { eprintln!("[voice] {}", t); }
            }
        }
    }
    async fn handle_plugin_tick(&mut self) {
        use crate::neotrix::nt_io_plugin::PluginEvent;
        self.plugin_registry.dispatch(&PluginEvent::BrainTick).await;
    }
    async fn handle_agent_discovery(&mut self) {
        if let Some(ref mut d) = self.agent_discovery {
            if let Err(e) = d.listen() { log::warn!("[bg] discovery: {}", e); }
        }
    }
    async fn handle_curiosity(&mut self) {
        use crate::neotrix::nt_mind::hypercube_bridge::HyperCubeBridge;
        let gaps = HyperCubeBridge::new().analyze_gaps();
        self.curiosity_drive.ingest_gap_reports(&gaps);
        for q in self.curiosity_drive.drain_queries().iter().take(2) {
            if let Some(ref mut ev) = self.self_evolver {
                let url = format!("https://en.wikipedia.org/wiki/{}", q.replace(' ', "_"));
                let _ = ev.evolve_from_url(&url);
            }
        }
    }
    async fn handle_knowledge_chain(&mut self) {
        if let Some(ref mut chain) = self.knowledge_chain {
            if !chain.has_pending() { chain.init_default_discovery(); }
            let mut brain = crate::neotrix::nt_mind::self_iterating::ReasoningBrain::new();
            let mut bank = crate::neotrix::nt_mind::memory::ReasoningBank::new(100);
            if let Ok(r) = chain.run_chain(&mut brain, &mut bank) {
                eprintln!("[bg] knowledge chain: discovered={}, mined={}, absorbed={}",
                    r.discovered, r.mined, r.absorbed);
            }
        }
    }
    async fn handle_knowledge_aging(&mut self) {
        let r = self.knowledge_aging.run_aging_cycle();
        if r.stale_count > 0 {
            eprintln!("[bg] aging: {} stale, {} expired", r.stale_count, r.expired_count);
            for url in r.rescans_needed.iter().take(3) {
                if let Some(ref mut ev) = self.self_evolver {
                    if crate::neotrix::nt_mind::self_evolver::SelfEvolver::is_url(url) {
                        let _ = ev.evolve_from_url(url);
                    }
                }
            }
        }
    }
    async fn handle_cleanup(&mut self) {
        let engine = match self.cleanup_engine.as_mut() {
            Some(e) => e,
            None => return,
        };

        // 1. 扫描并归档过期构建产物到 .cleanup/archive/
        engine.archive_on_clean = true;
        let r = engine.clean(CleanupKind::ProjectArtifacts);
        if r.deletable_count > 0 {
            log::info!("[bg] cleanup: archived {} items ({:.1} MB)",
                r.deletable_count, r.estimated_bytes as f64 / 1_048_576.0);
        }

        // 2. 清理 .DS_Store
        if let Ok(entries) = std::fs::read_dir(".") {
            let mut count = 0u32;
            for entry in entries.flatten() {
                if entry.file_name() == ".DS_Store" {
                    let _ = std::fs::remove_file(entry.path());
                    count += 1;
                }
            }
            if count > 0 {
                log::info!("[bg] cleanup: removed {} .DS_Store files", count);
            }
        }

        // 3. 整理旧快照
        let snapshots = CleanupEngine::prune_brain_snapshots(20);
        if snapshots > 0 {
            log::info!("[bg] cleanup: pruned {} old brain snapshots", snapshots);
        }
    }

    async fn handle_backup(&mut self) {
        let mut engine = BackupEngine::new(&PathBuf::from("."));
        match engine.run_backup() {
            Ok(m) => log::info!("[bg] backup: {} files, {:.1} KB -> .backup/{}",
                m.file_count, m.total_bytes as f64 / 1024.0, m.backup_id),
            Err(e) => log::warn!("[bg] backup failed: {}", e),
        }
    }
    async fn handle_crystallization(&mut self) {
        if self.config.enable_auto_crystallize {
            eprintln!("[bg] crystallization: {}", self.auto_crystallizer.summary());
        }
    }
    async fn handle_scheduler_tick(&mut self) {
        if let Some(ref mut sched) = self.scheduler {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
            let due = sched.tick(now, 0.3, 0.5, 0.2, 0.4);
            for (job_id, handler) in due {
                log::info!("[scheduler] job {} -> handler {}", job_id, handler);
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                sched.record_run(&job_id, now, 100, true, None);
            }
        }
    }
    async fn handle_evolve(&mut self) {
        if let Some(ref mut d) = self.daemon {
            let fixes = d.run_intelligent_cycle();
            if fixes.0 > 0 {
                log::info!("[bg] evolve: {} fixes, reward={:.4}", fixes.0, fixes.1);
            }
        }
    }
    async fn handle_skill_scan(&mut self) {
        let skills = self.skill_engine.load_all();
        if !skills.is_empty() {
            log::info!("[bg] skill_scan: {} skills loaded", skills.len());
        }
    }
    async fn handle_avatar_auto_distill(&mut self) {
        if let Some(ref mut eng) = self.avatar_engine {
            #[allow(clippy::mut_mutex_lock)]
            if let Ok(mut e) = eng.lock() {
                let _snapshot = e.auto_distill();
                log::info!("[bg] avatar auto_distill: edition={}, confidence={:.2}, msgs={}",
                    e.avatar.edition, e.avatar.confidence, e.avatar.total_messages_processed);
            }
        }
    }
    async fn handle_kb_absorb(&mut self) {
        let report = match self.kb_pipeline.update_panorama() {
            Ok(r) => r,
            Err(_) => return,
        };
        log::info!("[bg] kb_absorb: {} sources", report.total_sources);
    }
    async fn handle_consciousness_tick(&mut self) {
        if let Some(ref mut cr) = self.consciousness_runtime {
            if !cr.awakened {
                let report = cr.awaken();
                log::info!("[bg] consciousness awakened: step={} coherence={:.3}",
                    report.birth_step, report.initial_coherence);
            }
            let iteration = match self.brain.try_read() {
                Ok(b) => b.iteration,
                Err(_) => 0,
            };
            let resonance = format!("[consciousness_tick] iteration={}", iteration);
            let critique = cr.tick(&resonance);
            if let Some(c) = critique {
                if c.overall_quality < 0.3 {
                    log::warn!("[bg] consciousness: low quality ({:.3})", c.overall_quality);
                }
            }
        }
    }

    async fn handle_session_recovery(&mut self) {
        if let Some(ref mut sr) = self.session_recovery {
            let snap = sr.create_snapshot(&[], &[], "auto-snapshot");
            if let Ok(s) = snap {
                log::info!("[bg] session_recovery: snapshot {} created", s.session_id);
            }
        }
    }

    async fn handle_crawl_queue(&mut self) {
        use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_store::{claim_next_crawl_url, mark_crawl_complete};
        if self.kb_pipeline.kb.is_none() {
            log::warn!("[bg] crawl_queue: kb not attached");
            return;
        }
        loop {
            let (id, url) = {
                let kb = self.kb_pipeline.kb.as_ref().unwrap();
                let conn = kb.conn.lock().unwrap();
                match claim_next_crawl_url(&conn) {
                    Ok(Some(item)) => (item.id, item.url),
                    _ => return,
                }
            };
            log::info!("[bg] crawl claimed: {} (domain tracked)", url);
            match self.kb_pipeline.absorb_url(&url) {
                Ok(report) => {
                    log::info!("[bg] crawl absorbed: {} -> {} nodes", report.url, report.nodes_created);
                    let kb = self.kb_pipeline.kb.as_ref().unwrap();
                    let conn = kb.conn.lock().unwrap();
                    let _ = mark_crawl_complete(&conn, &id, true, None);
                }
                Err(e) => {
                    log::warn!("[bg] crawl failed: {}: {:?}", url, e);
                    let kb = self.kb_pipeline.kb.as_ref().unwrap();
                    let conn = kb.conn.lock().unwrap();
                    let _ = mark_crawl_complete(&conn, &id, false, Some(&e));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::neotrix::nt_mind::panorama_pipeline::PanoramaPipeline;
    use crate::neotrix::nt_mind::goal_loop::GoalLoop;
    use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
    use crate::neotrix::nt_world_model_v2::WorldModelV2;

    #[test]
    fn test_panorama_pipeline_new() {
        assert_eq!(PanoramaPipeline::new().cycle, 0);
    }
    #[test]
    fn test_panorama_run_cycle() {
        let mut pano = PanoramaPipeline::new();
        assert_eq!(pano.run_cycle(
            &mut SelfIteratingBrain::new(),
            &mut GoalLoop::new(),
            &mut WorldModelV2::new(4, 64)
        ).cycle, 1);
    }
}
