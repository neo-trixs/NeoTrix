use super::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::Mutex;

pub struct ConsciousnessThresholds {
    pub warn_quality: f64,
    pub critical_quality: f64,
    pub eventbus_critical: f64,
}

impl Default for ConsciousnessThresholds {
    fn default() -> Self {
        Self {
            warn_quality: 0.3,
            critical_quality: 0.2,
            eventbus_critical: 0.2,
        }
    }
}

pub static CONSCIOUSNESS_THRESHOLDS: LazyLock<ConsciousnessThresholds> =
    LazyLock::new(ConsciousnessThresholds::default);

// ────────────────────────────────────────────────────────────────
// ConvergencePulse — 分形收敛循环状态机 (Cycle 115/155 模式固化)
// 5 级分形: Artifact → Task → Session → Epic → PR
// 每层迭代推进 gap 关闭, 全部 gap 清空 + 外部验证通过后晋升下一层。
// ────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvergenceLayer {
    Artifact,
    Task,
    Session,
    Epic,
    Pr,
}

impl ConvergenceLayer {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Artifact => "artifact",
            Self::Task => "task",
            Self::Session => "session",
            Self::Epic => "epic",
            Self::Pr => "pr",
        }
    }
    pub fn next(&self) -> Option<ConvergenceLayer> {
        match self {
            Self::Artifact => Some(Self::Task),
            Self::Task => Some(Self::Session),
            Self::Session => Some(Self::Epic),
            Self::Epic => Some(Self::Pr),
            Self::Pr => None,
        }
    }
    pub fn all() -> [ConvergenceLayer; 5] {
        [Self::Artifact, Self::Task, Self::Session, Self::Epic, Self::Pr]
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConvergenceGap {
    pub domain: String,
    pub description: String,
    pub severity: String,
}

#[derive(Debug, Clone)]
pub struct ConvergencePulse {
    pub layer: ConvergenceLayer,
    pub iteration: u32,
    pub gaps: Vec<ConvergenceGap>,
    pub verified: bool,
    pub last_action: String,
    pub updated_at: i64,
}

impl Default for ConvergencePulse {
    fn default() -> Self {
        Self {
            layer: ConvergenceLayer::Artifact,
            iteration: 0,
            gaps: Vec::new(),
            verified: false,
            last_action: String::new(),
            updated_at: 0,
        }
    }
}

impl ConvergencePulse {
    pub fn status_line(&self) -> String {
        if self.gaps.is_empty() {
            return format!("convergence: layer={} iter={} gaps=none verified={}",
                self.layer.name(), self.iteration, self.verified);
        }
        let g = &self.gaps[0];
        format!("convergence: layer={} iter={} gap={}/{} [{}] {} verified={}",
            self.layer.name(), self.iteration, g.domain, g.severity, g.description,
            self.gaps.len(), self.verified)
    }

    /// 当前层是否已完成: 无 gap 且已通过外部验证。
    pub fn layer_complete(&self) -> bool {
        self.gaps.is_empty() && self.verified
    }

    /// 从给定 self_test 结果生成当前层 gap。
    /// 仅当存在 gap 时清除 verified — 无 gap 不重置外部验证结果,
    /// 使外部 cargo check 验证 (D24/P67) 不被内部分析覆盖。
    pub fn gaps_from_self_tests(&mut self, results: &[(String, bool)]) {
        self.gaps = results.iter()
            .filter(|(_, ok)| !*ok)
            .map(|(name, _)| ConvergenceGap {
                domain: self.layer.name().to_string(),
                description: format!("self_test '{}' failing at {} layer", name, self.layer.name()),
                severity: "medium".to_string(),
            })
            .collect();
        if !self.gaps.is_empty() {
            self.verified = false;
        }
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    }

    /// 推进迭代: 若层完成 → 晋升; 否则 iteration++ (自动修复动作占位)。
    pub fn advance(&mut self) -> Option<ConvergenceLayer> {
        if self.layer_complete() {
            let old = self.layer;
            if let Some(nxt) = old.next() {
                self.layer = nxt;
                self.iteration = 0;
                self.verified = false;
                self.last_action = format!("promoted {} → {}", old.name(), nxt.name());
                return Some(nxt);
            }
        }
        self.iteration += 1;
        self.last_action = format!("iter {} at {}: {} open gap(s)",
            self.iteration, self.layer.name(), self.gaps.len());
        None
    }
}

impl crate::core::nt_core_self_test::SelfTest for ConvergencePulse {
    fn name(&self) -> &str {
        "convergence_pulse"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        // 5 级分形层完整
        let layers = ConvergenceLayer::all();
        if layers.len() != 5 {
            failures.push(format!("expected 5 convergence layers, got {}", layers.len()));
        }
        // 晋升链完整: 每层须先通过外部验证 (verified=true) 才能晋升。
        let mut p = ConvergencePulse::default();
        let mut promoted = 0;
        loop {
            p.gaps = Vec::new();
            p.verified = true;
            if p.advance().is_none() { break; }
            promoted += 1;
        }
        if promoted != 4 {
            failures.push(format!("expected 4 promotions artifact→pr, got {}", promoted));
        }
        // gap 存在时不应晋升
        let mut q = ConvergencePulse {
            gaps: vec![ConvergenceGap { domain: "test".into(), description: "open gap".into(), severity: "high".into() }],
            ..Default::default()
        };
        let before = q.layer;
        q.advance();
        if q.layer != before {
            failures.push("should NOT promote when gaps open".into());
        }
        if q.status_line().is_empty() {
            failures.push("status_line() should be non-empty".into());
        }
        if failures.is_empty() { Ok(()) } else { Err(failures) }
    }
}

use crate::core::nt_core_self_constitution::ConstitutionLoader;
use crate::neotrix::l8_autonomic_impl::nt_mind_cleanup::{CleanupEngine, CleanupKind, BackupEngine};
use crate::neotrix::l8_autonomic_impl::nt_mind_skill_engine::SkillEngine;
use crate::neotrix::l8_autonomic_impl::nt_mind_knowledge_pipeline::KnowledgeAbsorptionPipeline;
use crate::neotrix::l1_body_impl::nt_io_session_recovery::SessionRecoveryManager;
use crate::neotrix::nt_core_event_bus::{EventBus, subscribe_all_layers_sync};
use crate::neotrix::nt_mind::distillation::MetaCognitionBridge;
use crate::core::nt_core_event::CoreEvent;
use crate::core::nt_core_scoring_substrate::ScoringSubstrate;
use crate::core::nt_core_state_substrate::StateSubstrate;
use crate::core::nt_core_delegate_engine::DelegateEngine;
use crate::core::nt_core_simulate_engine::SimulateEngine;

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

        // ── Load Constitution at startup ──
        let agents_md_path = std::path::Path::new("AGENTS.md");
        if agents_md_path.exists() {
            match ConstitutionLoader::load_from_file(agents_md_path) {
                Ok(constitution) => {
                    log::info!("[constitution] Loaded {} rules, {} experiences, {} tree-growth, {} absorption",
                        constitution.rules.len(),
                        constitution.experiences.len(),
                        constitution.tree_growth_rules.len(),
                        constitution.absorption_rules.len());
                }
                Err(e) => log::warn!("[constitution] Failed to load AGENTS.md: {}", e),
            }
        } else {
            log::warn!("[constitution] AGENTS.md not found at {}", agents_md_path.display());
        }

        // Wrap self so each spawned task gets its own reference.
        let cleanup_engine = self.cleanup_engine.take();
        let kb = self.kb.clone();

        // Session start event (obsidian-mind SessionStart pattern)
        if let Some(ref kb_ref) = kb {
            let _result: Result<usize, String> = kb_ref.rebuild_skills_library();
            let _ = kb_ref.rebuild_graph_cache();
            let summary = format!("session_start: cycle_{}", chrono::Utc::now().timestamp());
            let title = format!("session-start-{}", chrono::Utc::now().timestamp());
            let _ = kb_ref.insert_or_get_node(&title, crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::NodeType::Session, Some(&summary), None, Some("neotrix"));
            let issues = kb_ref.integrity_check();
            if !issues.is_empty() {
                log::warn!("[session-start] KB integrity issues: {:?}", issues);
            }
        }

        // ── Import knowledge assets at startup ──
        if let Some(ref kb_ref) = kb {
            let assets_path = std::path::Path::new("assets/knowledge_data.json");
            if assets_path.exists() {
                match kb_ref.import_knowledge_assets(assets_path) {
                    Ok(report) => {
                        if report.imported > 0 || report.edges_created > 0 {
                            log::info!("[knowledge-assets] Imported {} nodes, {} edges ({} errors)",
                                report.imported, report.edges_created, report.errors.len());
                        }
                    }
                    Err(e) => log::warn!("[knowledge-assets] Import failed: {}", e),
                }
            } else {
                log::warn!("[knowledge-assets] {} not found, skipping", assets_path.display());
            }
        }

        // ── Import review findings at startup ──
        if let Some(ref kb_ref) = kb {
            let review_path = std::path::Path::new("design/review-findings.json");
            if review_path.exists() {
                match kb_ref.import_review_findings(review_path) {
                    Ok(report) => {
                        if report.imported > 0 {
                            log::info!("[review-findings] Imported {} defects ({} errors)",
                                report.imported, report.errors.len());
                        }
                    }
                    Err(e) => log::warn!("[review-findings] Import failed: {}", e),
                }
            }
        }

        // ── Sync brain state to KB at startup ──
        if let Some(ref kb_ref) = kb {
            let brain_dir = std::path::Path::new(&dirs::home_dir().unwrap_or_default()).join(".neotrix");
            if brain_dir.join("brain.json").exists() {
                match kb_ref.import_brain_state(&brain_dir) {
                    Ok(report) => {
                        if report.imported > 0 {
                            log::info!("[brain-state] Synced {} nodes, {} edges ({} errors)",
                                report.imported, report.edges_created, report.errors.len());
                        }
                    }
                    Err(e) => log::warn!("[brain-state] Sync failed: {}", e),
                }
            }
        }

        // ── Import absorption report at startup ──
        if let Some(ref kb_ref) = kb {
            let abs_path = std::path::Path::new(&dirs::home_dir().unwrap_or_default()).join(".neotrix/absorption_report.json");
            if abs_path.exists() {
                match kb_ref.import_absorption_report(&abs_path) {
                    Ok(report) => {
                        if report.imported > 0 {
                            log::info!("[absorption-report] Imported {} nodes, {} edges ({} errors)",
                                report.imported, report.edges_created, report.errors.len());
                        }
                    }
                    Err(e) => log::warn!("[absorption-report] Import failed: {}", e),
                }
            }
        }

        // ── Import knowledge engine data at startup ──
        if let Some(ref kb_ref) = kb {
            let ke_path = std::path::Path::new(&dirs::home_dir().unwrap_or_default()).join(".neotrix/knowledge_engine.json");
            if ke_path.exists() {
                match kb_ref.import_knowledge_engine(&ke_path) {
                    Ok(report) => {
                        if report.imported > 0 {
                            log::info!("[knowledge-engine] Imported {} entries ({} errors)",
                                report.imported, report.errors.len());
                        }
                    }
                    Err(e) => log::warn!("[knowledge-engine] Import failed: {}", e),
                }
            }
        }

        // ── Import reasoning memories at startup ──
        if let Some(ref kb_ref) = kb {
            let rb_path = std::path::Path::new(&dirs::home_dir().unwrap_or_default()).join(".neotrix/reasoning_bank.json");
            if rb_path.exists() {
                match kb_ref.import_reasoning_memories(&rb_path) {
                    Ok(report) => {
                        if report.imported > 0 {
                            log::info!("[reasoning-memories] Imported {} traces ({} errors)",
                                report.imported, report.errors.len());
                        }
                    }
                    Err(e) => log::warn!("[reasoning-memories] Import failed: {}", e),
                }
            }
        }

        let mut kb_pipeline = KnowledgeAbsorptionPipeline::new();
        if let Some(ref kb_ref) = kb {
            kb_pipeline.attach_kb(kb_ref.clone());
        }

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
            kb_pipeline,
            session_recovery: self.session_recovery.take(),
            event_bus: Some(event_bus.as_ref().clone()),
            metacognition: self.metacognition.take(),
            #[cfg(feature = "stealth-net")]
            world_consciousness: self.world_consciousness.take(),
            #[cfg(not(feature = "stealth-net"))]
            world_consciousness: None,
            consciousness_runtime: std::mem::take(&mut self.consciousness_runtime),
            consciousness_tree: self.consciousness_tree.take(),
            fep_iit_bridge: self.fep_iit_bridge.take(),
cognitive_load: self.cognitive_load.take(),
            bbrain: std::mem::take(&mut self.bbrain),
            cog_eval: crate::core::nt_core_self::metacognitive_evaluator::CognitiveEvaluator::new(),
            second_brain: {
                let mut sb = SecondBrain::new();
        if let Some(ref kb_ref) = self.kb {
                    sb.attach_kb(kb_ref.clone());
                }
                Some(sb)
            },
            kb,
            emotion_restored: std::sync::atomic::AtomicBool::new(false),
            cognitive_mode: 0,
            scoring: ScoringSubstrate::new().with_threshold(0.5),
            state: StateSubstrate::new(),
            delegate: DelegateEngine::new(),
            simulate: SimulateEngine::new(),
            convergence_pulse: ConvergencePulse::default(),
            tool_grounding: crate::core::nt_core_self::self_audit::ToolGroundingMonitor::new(),
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
        macro_rules! emit_event {
            ($h:expr, $event:expr) => {
                if let Some(ref bus) = $h.event_bus {
                    bus.emit($event);
                }
            };
        }

        spawn_handler!(cfg.save_interval_secs, |h| {
            h.handle_save().await;
            emit_event!(h, crate::core::nt_core_event::CoreEvent::TaskSubmitted {
                task: "save".into(), task_type: "storage".into(), priority: 2,
            });
        });
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
        spawn_handler!(7200, |h| {
            h.handle_kb_absorb().await;
        });
        spawn_handler!(86400, |h| h.handle_seed_crawl_queue().await);
        spawn_handler!(600, |h| h.handle_session_recovery().await);
        spawn_handler!(300, |h| h.handle_crawl_queue().await);
        spawn_handler!(3600, |h| h.handle_architecture_audit().await);
        // ── Constitution hot-reload ──
        spawn_handler!(86400, |h| h.handle_constitution_reload().await);
        // 3600s — consciousness evolves at architecture-audit tempo, not real-time
        spawn_handler!(3600, |h| h.handle_consciousness_tick().await);
        // 600s — Second Brain auto-sync (emotion + session notes to KB)
        spawn_handler!(600, |h| h.handle_second_brain_tick().await);
        // Startup: restore emotion state from KB (deferred 5s, then skips)
        spawn_handler!(5, |h| {
            if !h.emotion_restored.load(std::sync::atomic::Ordering::Relaxed) {
                if let Some(kb) = &h.kb {
                    if let Ok(Some(json)) = kb.kv_get("emotion", "engine_state") {
                        if let Ok(engine) = crate::core::nt_core_self::emotion_state::EmotionEngine::from_json(&json) {
                            if let Some(ref mut cr) = h.consciousness_runtime {
                                cr.set_emotion_engine(engine);
                                log::info!("[bg] emotion state restored from KB");
                            }
                        }
                    }
                }
                h.emotion_restored.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        });

        // ── EventBus behavioral consumer (D30 fix) — responds to events with behavioral actions ──
        {
            let mut event_rx = event_bus.subscribe();
            let h = this.clone();
            let mut rx = shutdown_rx.clone();
            self.handles.push(tokio::spawn(async move {
                loop {
                    tokio::select! {
                        biased;
                        result = event_rx.recv() => {
                            match result {
                                Ok(event) => {
                                    let mut handle = h.lock().await;
                                    handle.handle_event_bus_event(event).await;
                                }
                                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                                    log::warn!("[bg] event_bus consumer lagged {} events", n);
                                }
                            }
                        }
                        _ = rx.changed() => {
                            log::trace!("[bg] event_bus consumer shutting down");
                            break;
                        }
                    }
                }
            }));
        }

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
pub struct BackgroundLoopHandle {
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
    consciousness_tree: Option<crate::core::nt_core_consciousness_tree::ConsciousnessTree>,
    fep_iit_bridge: Option<crate::neotrix::nt_core_fep_iit::FEPIITBridge>,
    cognitive_load: Option<crate::core::nt_core_consciousness::CognitiveLoadMonitor>,
    second_brain: Option<SecondBrain>,
    kb: Option<Arc<KnowledgeBase>>,
    emotion_restored: std::sync::atomic::AtomicBool,
    bbrain: crate::neotrix::nt_mind::bbrain_monitor::BMonitor,
    cog_eval: crate::core::nt_core_self::metacognitive_evaluator::CognitiveEvaluator,
    /// 0=Balanced, 1=Deep, 2=Fast — updated by consciousness tick, consumed by batch loops.
    cognitive_mode: u8,
    #[allow(dead_code)]
    scoring: ScoringSubstrate,
    state: StateSubstrate,
    delegate: DelegateEngine,
    simulate: SimulateEngine,
    convergence_pulse: ConvergencePulse,
    tool_grounding: crate::core::nt_core_self::self_audit::ToolGroundingMonitor,
}

impl BackgroundLoopHandle {
    fn try_emit(&self, event: crate::core::nt_core_event::CoreEvent) {
        if let Some(ref bus) = self.event_bus { bus.emit(event); }
    }

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
        // Full autonomous cycle: circuit-breaker guard, terminal-goal
        // reward/dequeue, auto-generation of new goals when none is active,
        // then one pursue_iteration. pursue_all(.., 1) bails early when no
        // active goal exists and never exercises the auto-generation path.
        self.goal_loop.pursue_auto_iteration(&mut b);
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
    /// Log a session event to KB (obsidian-mind SessionStart/Stop pattern).
    /// Creates a node with type=Session with event type and summary.
    #[allow(dead_code)]
    async fn log_session_event(&self, event_type: &str, summary: &str) {
        let Some(ref kb) = self.kb else { return };
        let title = format!("session-{}-{}", event_type, chrono::Utc::now().timestamp());
        let _ = kb.insert_or_get_node(&title, crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::NodeType::Session, Some(summary), None, Some("neotrix"));
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
            Err(e) => {
                log::error!("[bg] kb_absorb failed: {}", e);
                self.try_emit(crate::core::nt_core_event::CoreEvent::SystemError {
                    component: "kb_absorb".into(), error: e.to_string(), severity: "error".into(),
                });
                return;
            }
        };
        log::info!("[bg] kb_absorb: {} sources", report.total_sources);
        self.try_emit(crate::core::nt_core_event::CoreEvent::TaskSubmitted {
            task: "kb_absorb".into(), task_type: "ingestion".into(), priority: 3,
        });
    }
    async fn handle_consciousness_tick(&mut self) {
        // ── Collect real context from brain + KB ──
        let (iteration, caps_mean) = match self.brain.try_read() {
            Ok(b) => {
                let mean = b.brain.capability.arr.iter().sum::<f64>() / 23.0_f64.max(1.0);
                (b.iteration, mean)
            }
            Err(_) => (0, 0.0),
        };
        let (kb_nodes, kb_edges, kb_crawl) = self.kb.as_ref()
            .and_then(|kb| kb.stats().ok())
            .map(|s| (s.total_nodes as u64, s.total_edges as u64, s.crawl_pending as u64))
            .unwrap_or((0, 0, 0));

        // ── Phase 1: ConsciousnessTree Growth Cycle (Soil → Roots → Trunk → Branches → Fruits → Core) ──
        if let Some(ref mut tree) = self.consciousness_tree {
            tree.soil.kb_node_count = kb_nodes;
            tree.soil.kb_edge_count = kb_edges;
            tree.soil.crawl_queue_depth = kb_crawl;
            if let Some(ref monitor) = self.awareness {
                let report = monitor.get_report();
                tree.trunk.phi = report.phi;
                tree.trunk.coherence = report.coherence;
            }
            tree.trunk.gwt_resonance_active = self.panorama.is_some();
            tree.trunk.workspace_size = 23;
            // Branch health is now set from SelfTest results in handle_architecture_audit
            // No simulated fallback here — real data or neutral 0.5 from set_branch_health_from_self_tests
            let growth_report = tree.run_growth_cycle();
            let contract_status = growth_report.phase6_fulfillment
                .as_ref().map(|f| format!("fulfilled={} ({}/{})", f.fulfilled, f.evidence_met, f.evidence_total))
                .unwrap_or_else(|| "n/a".into());
            let drift_status = growth_report.phase7_drift
                .as_ref().map(|d| if d.drift_detected { format!("DRIFT mag={:.3}", d.drift_magnitude) } else { "clean".into() })
                .unwrap_or_else(|| "n/a".into());
            log::info!("[bg] consciousness_tree cycle {}: absorbed={} phi={:.3} fruits={} guidance={} | contract[{}] drift[{}]",
                tree.cycle, growth_report.phase1_absorbed, growth_report.phase2_phi,
                growth_report.phase3_fruits, growth_report.phase4_guidance,
                contract_status, drift_status);
            // Evolution contract → goal loop: enqueue a behavioral goal when drift or unmet contract detected
            if let Some(drift) = &growth_report.phase7_drift {
                if drift.drift_detected {
                    if let Ok(mut brain) = self.brain.try_write() {
                        let action = drift.corrective_actions.first().cloned().unwrap_or_else(|| "Re-evaluate evolution contract".into());
                        self.goal_loop.enqueue_goal(
                            &mut brain,
                            &format!("evolution_drift_recovery: {}", action),
                            None,
                        );
                    }
                }
            }
        }

        // ── Phase 2: Consciousness Runtime Tick with REAL resonance content ──
        if let Some(ref mut cr) = self.consciousness_runtime {
            if !cr.awakened {
                let report = cr.awaken();
                log::info!("[bg] consciousness awakened: step={} coherence={:.3}",
                    report.birth_step, report.initial_coherence);
            }
            let gwt_active = self.panorama.as_ref()
                .map(|p| p.gwt.active_specialists().len()).unwrap_or(0);
            let resonance = format!(
                "[consciousness_tick] iteration={} caps={:.3} kb={} gwt={}",
                iteration, caps_mean, kb_nodes, gwt_active,
            );
            let critique = cr.tick(&resonance);
            if let Some(c) = critique {
                if c.overall_quality < CONSCIOUSNESS_THRESHOLDS.warn_quality {
                    log::warn!("[bg] consciousness: LOW QUALITY ({:.3}) — reasons: {:?}",
                        c.overall_quality, c.reasons);
                    if c.overall_quality < CONSCIOUSNESS_THRESHOLDS.critical_quality {
                        // BEHAVIORAL RESPONSE: enqueue self-review goal on critical quality
                        if let Ok(mut brain) = self.brain.try_write() {
                            self.goal_loop.enqueue_goal(
                                &mut brain,
                                "consciousness_recovery: quality critically low — initiating self-review",
                                None,
                            );
                        }
                    }
                } else if c.overall_quality > 0.7 {
                    log::info!("[bg] consciousness: good quality ({:.3}) selected_action={:?}",
                        c.overall_quality, c.selected_action);
                } else {
                    log::debug!("[bg] consciousness: quality={:.3} relevance={:.3} consistency={:.3}",
                        c.overall_quality, c.relevance_score, c.consistency_score);
                }
                self.try_emit(crate::core::nt_core_event::CoreEvent::ConsciousnessCritique {
                    quality: c.overall_quality,
                    relevance: c.relevance_score,
                    consistency: c.consistency_score,
                    timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default().as_secs() as i64,
                });
                if let Ok(mut brain) = self.brain.try_write() {
                    brain._last_consciousness_quality = c.overall_quality;
                    brain._consciousness_critique_count += 1;
                }
            }
        }
        // Record state metrics from the runtime tick
        self.state.record_metric("phi", self.awareness.as_ref().map(|m| m.get_report().phi).unwrap_or(0.0));
        self.state.record_metric("coherence", self.awareness.as_ref().map(|m| m.get_report().coherence).unwrap_or(0.0));

        // ── Phase 3: FEPIITBridge — compute unified consciousness score ──
        if let Some(ref fep_iit) = self.fep_iit_bridge {
            if let Some(ref monitor) = self.awareness {
                let report = monitor.get_report();
                
                // Free Energy from StateSubstrate (unified: 1 - phi*coherence + load*0.5)
                let fe_val = self.state.free_energy.max(0.0).min(1.0) * 10.0;
                let score = fep_iit.compute_consciousness_score(fe_val, report.phi, report.coherence);
                log::debug!("[bg] fep_iit: unified_score={:.3} phi={:.3} coherence={:.3} fe={:.3}",
                    score, report.phi, report.coherence, fe_val);
            }
        }

        // ── Phase 4: ConsciousnessMonitor — self-observation cycle ──
        if let Some(ref mut monitor) = self.awareness {
            monitor.observe();
            let report = monitor.get_report();
            log::debug!("[bg] consciousness_monitor: level={:.3} phi={:.3} coherence={:.3} health={:.3}",
                report.consciousness, report.phi, report.coherence, report.health);

            // ── CognitiveLoadMonitor: record load from consciousness metrics ──
            if let Some(ref mut clm) = self.cognitive_load {
                let load = (1.0 - report.consciousness.max(0.0).min(1.0)) * 0.6
                    + (1.0 - report.coherence.max(0.0).min(1.0)) * 0.4;
                self.state.record_metric("load", load);
                self.state.tick(); // updates free_energy + thinking mode

                let prev_mode_clm = clm.mode();
                clm.record_step(load);
                let new_state_mode = self.state.active_mode;

                // Update cognitive_mode field for behavioral consumption by other handlers
                self.cognitive_mode = match new_state_mode {
                    crate::core::nt_core_state_substrate::ThinkingMode::Deep => 1,
                    crate::core::nt_core_state_substrate::ThinkingMode::Fast => 2,
                    _ => 0,
                };

                log::debug!("[bg] cognitive_load: mode={:?} load={:.3} free_energy={:.3}",
                    new_state_mode, load, self.state.free_energy);

                // Track mode transitions for behavioral logging
                if prev_mode_clm != clm.mode() {
                    log::info!("[bg] cognitive_load: mode transition (CLM) {:?} -> {:?}",
                        prev_mode_clm, clm.mode());
                }

                // BEHAVIORAL RESPONSE: When deep mode is active, trigger deeper reasoning cycle
                if new_state_mode == crate::core::nt_core_state_substrate::ThinkingMode::Deep {
                    if let Ok(mut brain) = self.brain.try_write() {
                        self.goal_loop.enqueue_goal(
                            &mut brain,
                            "deep_reasoning_available: cognitive budget healthy — initiating extended analysis cycle",
                            None,
                        );
                    }
                    log::info!("[bg] cognitive_load: DEEP mode active — enqueued deep_reasoning goal");
                }
            }
        }

        // ── Phase 4b: BMonitor — observe from consciousness metrics + read report ──
        {
            let phi = self.awareness.as_ref().map(|m| m.get_report().phi).unwrap_or(0.0);
            let coherence = self.awareness.as_ref().map(|m| m.get_report().coherence).unwrap_or(0.0);
            let load = self.state.metric("load").and_then(|m| m.latest()).unwrap_or(0.5);
            self.bbrain.observe_from_metrics(phi, coherence, load);
        }
        if let Some(report) = self.bbrain.latest_report() {
            let trend = self.bbrain.health_trend();
            log::debug!("[bg] bbrain_monitor: health={:.2} trend={:+.2} flags={} intervention={}",
                report.health_score, trend, report.flags.len(), report.needs_intervention);
            if report.needs_intervention {
                log::warn!("[bg] bbrain: intervention needed — score={:.2} flags={:?}",
                    report.health_score, report.flags);
            }
        }

        // ── Phase 4c: CognitiveEvaluator — read persistent metacognitive evaluation ──
        if let Some(report) = self.cog_eval.latest_report() {
            log::debug!("[bg] cognitive_evaluator: id={} stability={:.3} attention={:.2} diversity={:.2} quality={:.2} pressure={:.2} n_flags={}",
                report.evaluation_id, report.stability_score, report.attention_health,
                report.strategy_diversity, report.trace_quality, report.context_pressure,
                report.flags.len());
            if self.cog_eval.has_degraded(0.15) {
                log::warn!("[bg] cognitive_evaluator: stability degraded >0.15");
            }
        }

        // ── Phase 5: ConsciousnessGoldStandard — dual-threshold detection ──
        if let Some(ref mut gs) = self.gold_standard {
            let state = match self.brain.try_read() {
                Ok(b) => {
                    b.brain.capability.arr.to_vec()
                }
                Err(_) => vec![0.0; 23],
            };

            // Get E8 hexagram states from WorldModelV2
            let hexagram_states = self.nt_world_model.as_ref()
                .map(|wm| {
                    wm.e8.current_state.vector.iter().enumerate()
                        .map(|(i, &activation)| crate::neotrix::nt_mind_consciousness_gold_standard::E8HexagramState {
                            index: i as u8,
                            activation: activation.max(0.0).min(1.0),
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let gs_report = gs.evaluate(&state, &hexagram_states);
            log::debug!("[bg] gold_standard: conscious={} phi={:.3} coherence={:.3} trend={:?}",
                gs_report.is_conscious_like, gs_report.phi, gs_report.coherence, gs_report.detection_streak);
        }

        // ── Phase 6: DelegateEngine — delegate cross-domain tasks ──
        if self.delegate.delegate("consciousness_tick", "nt_mind_background_loop", 0).is_none() {
            log::warn!("[bg] delegate: consciousness_tick delegation rejected — max concurrent reached");
        }
        let pending = self.delegate.synchronize();
        let delegated_total = self.delegate.total_tasks();
        if delegated_total > 0 {
            log::debug!("[bg] delegate: {} pending of {} total tasks, success_rate={:.2}",
                pending, delegated_total, self.delegate.success_rate());
        }

        // ── Phase 7: SimulateEngine — run grounding scenario ──
        let ctx = format!("Predict consciousness quality from phi={:.3} coherence={:.3}",
            self.state.metric("phi").and_then(|m| m.latest()).unwrap_or(0.0),
            self.state.metric("coherence").and_then(|m| m.latest()).unwrap_or(0.0));
        let sim_id = self.simulate.create_scenario("consciousness_health", &ctx);
        if self.simulate.simulate(sim_id.clone(), "stable").is_ok() {
            log::debug!("[bg] simulate: scenario={} created", sim_id);
        }

        // ── Phase 8: ConvergencePulse — 分形收敛循环推进 (Cycle 115/155/160) ──
        // 用本 tick 的检测状态生成 gap, 外部验证通过后推进迭代/晋升层级。
        {
            let results = vec![
                ("state_substrate".to_string(), !self.state.active_mode.name().is_empty()),
                ("bbrain".to_string(),
                    self.bbrain.latest_report().map(|r| r.health_score >= 0.0).unwrap_or(false)),
                ("cog_eval".to_string(), self.cog_eval.latest_report().is_some()),
                ("gold_standard".to_string(),
                    self.gold_standard.as_ref().map(|_| true).unwrap_or(false)),
            ];
            self.convergence_pulse.gaps_from_self_tests(&results);
            if self.convergence_pulse.gaps.is_empty() {
                // 外部验证: 运行 cargo check --all-targets 确认构建完整性。
                // 异步 + 120s 超时 (D27): 避免同步阻塞 tokio worker / 无限等待。
                let build_ok = match tokio::time::timeout(
                    std::time::Duration::from_secs(120),
                    tokio::process::Command::new("cargo")
                        .args(["check", "--all-targets", "-p", "neotrix"])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status(),
                ).await {
                    Ok(Ok(status)) => status.success(),
                    _ => {
                        log::warn!("[bg] convergence: external build check timed out or failed");
                        false
                    }
                };
                if build_ok {
                    self.convergence_pulse.verified = true;
                }
            }
            let promoted = self.convergence_pulse.advance();
            if let Some(layer) = promoted {
                log::info!("[bg] convergence: promoted to {} layer (fractal loop)",
                    layer.name());
            }
            log::debug!("[bg] {}", self.convergence_pulse.status_line());
        }

        // ── Phase 9: Auto-Healing — C5 self-healing loop ──
        // 检测 degrade 信号 → 自动响应（enqueue remediation goal / log / circuit-break）
        // 这是分形收敛循环的"修复臂"：检测→诊断→行为修复闭环。
        {
            let deg = self.tool_grounding.degraded_tools();
            if !deg.is_empty() {
                let names: Vec<&str> = deg.iter().map(|(n, _)| n.as_str()).collect();
                log::warn!("[bg] auto-heal: degraded tools detected: {}", names.join(", "));
                if let Ok(mut brain) = self.brain.try_write() {
                    self.goal_loop.enqueue_goal(
                        &mut brain,
                        &format!("[auto-heal] Tools degraded: {}", names.join(", ")),
                        None,
                    );
                }
            }
            // Convergence stalled detection: if same layer for >10 iterations with gaps,
            // escalate to C0-C5 maturity downgrade notification.
            if self.convergence_pulse.iteration > 10 && !self.convergence_pulse.gaps.is_empty() {
                log::warn!("[bg] auto-heal: convergence stalled at {} ({} iters, {} gaps)",
                    self.convergence_pulse.layer.name(),
                    self.convergence_pulse.iteration,
                    self.convergence_pulse.gaps.len());
            }
            // BMonitor health: if cognitive health score < 50, enqueue deep reasoning mode
            // to give the system more time/cycles for recovery.
            if let Some(br) = self.bbrain.latest_report() {
                if br.health_score < 0.5 {
                    log::warn!("[bg] auto-heal: cognitive health low ({:.0}%), adjusting mode to Deep",
                        br.health_score * 100.0);
                    self.state.set_mode(crate::core::nt_core_state_substrate::ThinkingMode::Deep);
                }
            }
        }
    }

    /// EventBus behavioral consumer (D30) — responds to events with brain/KB actions, not just logs.
    async fn handle_event_bus_event(&mut self, event: CoreEvent) {
        match &event {
            CoreEvent::SystemError { severity, component, error } if severity == "critical" => {
                log::error!("[bg] event_bus: CRITICAL {}: {}", component, error);
                if let Ok(mut brain) = self.brain.try_write() {
                    self.goal_loop.enqueue_goal(&mut brain,
                        &format!("event_bus_critical: {} - {}", component, error), None);
                }
            }
            CoreEvent::GlobalHalt { reason, source } => {
                log::error!("[bg] event_bus: GLOBAL HALT {} from {}", reason, source);
                if let Some(ref kb) = self.kb {
                    let _ = kb.kv_set("event_bus", "global_halt", &serde_json::json!({
                        "reason": reason, "source": source,
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default().as_secs(),
                    }).to_string());
                }
                if let Ok(mut brain) = self.brain.try_write() {
                    self.goal_loop.enqueue_goal(&mut brain,
                        &format!("event_bus_recovery: {} - {}", source, reason), None);
                }
            }
            CoreEvent::ConsciousnessCritique { quality, .. } if *quality < CONSCIOUSNESS_THRESHOLDS.eventbus_critical => {
                log::warn!("[bg] event_bus: consciousness CRITICAL ({:.3})", quality);
                if let Some(ref kb) = self.kb {
                    let _ = kb.kv_set("event_bus", "consciousness_critical", &serde_json::json!({
                        "quality": quality,
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default().as_secs(),
                    }).to_string());
                }
            }
            CoreEvent::TaskSubmitted { task, task_type, priority } if *priority >= 3 => {
                log::info!("[bg] event_bus: high-priority task {} ({})", task, task_type);
                // R-P41: 高优先级任务入 KB，供后续 handler 消费，而非纯日志
                if let Some(ref kb) = self.kb {
                    let _ = kb.kv_set("event_bus", "task_submitted", &serde_json::json!({
                        "task": task, "task_type": task_type, "priority": priority,
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default().as_secs(),
                    }).to_string());
                }
            }
            _ => {
                log::trace!("[bg] event_bus: {:?}", event);
            }
        }
    }

    async fn handle_second_brain_tick(&mut self) {
        if let Some(ref mut sb) = self.second_brain {
            if let Some(ref mut cr) = self.consciousness_runtime {
                let report = cr.tick_emotion();
                if let Some(ref mut tree) = self.consciousness_tree {
                    tree.apply_emotion_report(report);
                }
                let note = format!(
                    "consciousness_tick iteration={} quality={:.3}",
                    self.brain.try_read().map(|b| b.iteration).unwrap_or(0),
                    cr.last_quality().unwrap_or(0.0),
                );
                sb.tick(Some(cr.emotion_engine()), Some(&note));
                // Persist emotion state to KB every tick (600s)
                let engine = cr.emotion_engine();
                sb.save_emotion_raw(engine);
            } else {
                sb.tick(None, None);
            }
        }
    }

    async fn handle_architecture_audit(&mut self) {
        use crate::core::nt_core_schema_watchdog::SchemaWatchdog;
        use crate::core::nt_core_self::self_audit::{converge_check, ConvergeCheckFn};
        use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};
        use crate::core::nt_core_meta::scanner::CodeScanner;
        use crate::core::nt_core_gwt::monitor::EntropyMonitor;
        use crate::core::nt_core_consciousness::inner_critic::InnerCritic;
        use crate::core::nt_core_self_review::SelfReviewGate;
        use crate::core::nt_core_meta::nt_core_meta_auditor::MetaAuditor;
        use crate::core::nt_core_meta::nt_core_arch_lint::ArchLint;
        use crate::core::nt_core_meta::monitor::MetaMonitor;
        use crate::core::nt_core_meta::metacognition_loop::MetaCognitiveLoop;
        use crate::core::nt_core_meta::self_model::SelfModel;

        let mut watchdog = SchemaWatchdog::new();
        let mut gaps = 0;
        for (type_name, fields) in &[
            ("KnowledgeNode", vec!["id", "title", "node_type", "content", "summary", "url", "domain", "language", "confidence", "importance", "access_count", "metadata", "created_at", "updated_at"]),
            ("NodeType", vec!["Concept", "Paper", "Repository", "Person", "Event", "Source", "Tool", "Framework", "Algorithm", "Theory", "Method", "Dataset", "Benchmark", "Organization", "Book", "Course", "Article", "CodeSnippet", "Idea", "Question", "Insight", "HarnessProfile", "Image", "EvolutionPattern", "ConversationEvolution", "Textbook", "Resource", "External", "Summary", "Guide", "Skill", "Reference", "WikiPage"]),
        ] {
            let f: Vec<String> = fields.iter().map(|s| s.to_string()).collect();
            if watchdog.detect_drift(type_name, &f).is_some() {
                gaps += 1;
            }
        }
        if gaps > 0 {
            log::warn!("[bg] arch_audit: {} schema gaps detected", gaps);
        } else {
            log::info!("[bg] arch_audit: clean — all schemas match");
        }
        let report = converge_check(".");
        if !report.findings.is_empty() {
            log::warn!("[bg] converge_check: {} ghosts, {} orphans, {} stale",
                report.ghost_count, report.stale_count, report.orphan_count);
        }

        // ── Inline self-test: types WITHOUT persistent fields use fresh instances (acceptable) ──
        let model = SelfModel::new();
        let scanner = CodeScanner::new(".");
        let entropy = EntropyMonitor::new(10, 0.5, 3);
        let meta_auditor = MetaAuditor::new();
        let arch_lint = ArchLint::new();
        let meta_monitor = MetaMonitor::new(model.clone());
        let meta_cog_loop = MetaCognitiveLoop::new(model);

        let mut self_tests = SelfTestRegistry::new();
        self_tests.register(Box::new(crate::core::nt_core_self_test::ExternalVerifier));
        self_tests.register(Box::new(watchdog));
        self_tests.register(Box::new(ConvergeCheckFn));
        self_tests.register(Box::new(scanner));
        self_tests.register(Box::new(entropy));
        self_tests.register(Box::new(InnerCritic::new()));
        self_tests.register(Box::new(SelfReviewGate::new(false)));
        self_tests.register(Box::new(meta_auditor));
        self_tests.register(Box::new(arch_lint));
        self_tests.register(Box::new(meta_monitor));
        self_tests.register(Box::new(meta_cog_loop));
        self_tests.register(Box::new(crate::neotrix::l8_autonomic_impl::nt_mind_self_diagnose::SelfDiagnose));
        self_tests.register(Box::new(crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_svaf_gate::SvafGate::default()));
        self_tests.register(Box::new(crate::core::l7_capability::nt_core_antidistil::DistillationDetector::new()));
        self_tests.register(Box::new(crate::neotrix::l1_body_impl::nt_act_autonomy::oracle_gate::OracleGate::new()));
        self_tests.register(Box::new(crate::neotrix::l1_body_impl::nt_act_code::semantic_entropy::SemanticEntropyGate::new()));
        self_tests.register(Box::new(crate::neotrix::l1_body_impl::nt_act_sandbox::ActionSandbox::new()));
        self_tests.register(Box::new(crate::core::nt_core_consciousness_review::ConsciousnessReview::new()));
        self_tests.register(Box::new(crate::neotrix::l8_autonomic_impl::nt_mind::consciousness_bridge::ConsciousnessBridge::new()));
        self_tests.register(Box::new(crate::neotrix::l1_body_impl::nt_shield::browser_security::BrowserSecurityScanner::new(
            crate::neotrix::l1_body_impl::nt_shield::browser_security::BrowserSecurityConfig::default(),
        )));
        self_tests.register(Box::new(crate::neotrix::l1_body_impl::nt_shield::check_registry::CheckRegistry::new()));
        self_tests.register(Box::new(crate::core::nt_core_telemetry::TelemetryStore::new(100)));

        // NOTE (Cycle 159b): NeoCodexSelfAudit::new() is intentionally NOT
        // registered here. Its Default snapshot reports provider-not-resolvable +
        // catalog-empty (3 permanent failures) → failure_count > 0 on every
        // architecture audit → spurious self-review goals enqueued. The audit is
        // a TUI-side live snapshot (NeoCodexSelfAudit::capture via EvolutionLoop),
        // not a BackgroundLoop detector. R-P26: SelfTest must be wired where its
        // data source exists.

        // ── Absorbed module SelfTests (Cycle 113) ──
        crate::core::nt_core_self_test_integration::register_absorbed_modules(&mut self_tests);

        // ── Substrate + Engine SelfTests (Cycle 119 architecture refactor) ──
        self_tests.register(Box::new(crate::core::nt_core_scoring_substrate::ScoringSubstrate::new().with_threshold(0.5)));
        self_tests.register(Box::new(crate::core::nt_core_state_substrate::StateSubstrate::new()));
        self_tests.register(Box::new(crate::core::nt_core_delegate_engine::DelegateEngine::new()));
        self_tests.register(Box::new(crate::core::nt_core_simulate_engine::SimulateEngine::new()));
        // ── ConvergencePulse SelfTest (Cycle 159c: fractal loop state machine) ──
        self_tests.register(Box::new(ConvergencePulse::default()));
        // ── ToolGroundingMonitor SelfTest — persistent instance (R-P49~R-P53) ──
        self_tests.register(Box::new(self.tool_grounding.clone()));

        if let Some(ref sb) = self.second_brain {
            match sb.self_test() {
                Ok(()) => log::info!("[SELF-TEST] SecondBrain ✅ pass"),
                Err(failures) => log::warn!("[SELF-TEST] SecondBrain ❌ FAIL: {}", failures.join("; ")),
            }
        }

        // ── Inline self-test: types WITH persistent fields — clone or call self_test() directly ──
        // KnowledgeGapDetector
        if let Some(ref gap_detector) = self.gap_detector {
            match gap_detector.self_test() {
                Ok(()) => log::info!("[SELF-TEST] KnowledgeGapDetector ✅ pass"),
                Err(failures) => log::warn!("[SELF-TEST] KnowledgeGapDetector ❌ FAIL: {}", failures.join("; ")),
            }
        } else {
            self_tests.register(Box::new(crate::core::nt_core_meta::knowledge_gap_detector::KnowledgeGapDetector::new()));
        }

        // BMonitor (direct field, not Option)
        match self.bbrain.self_test() {
            Ok(()) => log::info!("[SELF-TEST] BMonitor ✅ pass"),
            Err(failures) => log::warn!("[SELF-TEST] BMonitor ❌ FAIL: {}", failures.join("; ")),
        }

        // CognitiveEvaluator (direct field)
        match self.cog_eval.self_test() {
            Ok(()) => log::info!("[SELF-TEST] CognitiveEvaluator ✅ pass"),
            Err(failures) => log::warn!("[SELF-TEST] CognitiveEvaluator ❌ FAIL: {}", failures.join("; ")),
        }

        // ConsciousnessTree
        if let Some(ref tree) = self.consciousness_tree {
            match tree.self_test() {
                Ok(()) => log::info!("[SELF-TEST] ConsciousnessTree ✅ pass"),
                Err(failures) => log::warn!("[SELF-TEST] ConsciousnessTree ❌ FAIL: {}", failures.join("; ")),
            }
        } else {
            self_tests.register(Box::new(crate::core::nt_core_consciousness_tree::ConsciousnessTree::new()));
        }

        // ConsciousnessRuntime
        if let Some(ref cr) = self.consciousness_runtime {
            match cr.self_test() {
                Ok(()) => log::info!("[SELF-TEST] ConsciousnessRuntime ✅ pass"),
                Err(failures) => log::warn!("[SELF-TEST] ConsciousnessRuntime ❌ FAIL: {}", failures.join("; ")),
            }
        } else {
            self_tests.register(Box::new(crate::core::nt_core_consciousness::consciousness_runtime::ConsciousnessRuntime::new()));
        }

        // ConsciousnessMonitor (awareness)
        if let Some(ref monitor) = self.awareness {
            match monitor.self_test() {
                Ok(()) => log::info!("[SELF-TEST] ConsciousnessMonitor ✅ pass"),
                Err(failures) => log::warn!("[SELF-TEST] ConsciousnessMonitor ❌ FAIL: {}", failures.join("; ")),
            }
        } else {
            let mut cm = crate::neotrix::nt_mind_consciousness_monitor::ConsciousnessMonitor::new();
            cm.observe();
            self_tests.register(Box::new(cm));
        }

        // FEPIITBridge
        if let Some(ref bridge) = self.fep_iit_bridge {
            match bridge.self_test() {
                Ok(()) => log::info!("[SELF-TEST] FEPIITBridge ✅ pass"),
                Err(failures) => log::warn!("[SELF-TEST] FEPIITBridge ❌ FAIL: {}", failures.join("; ")),
            }
        } else {
            self_tests.register(Box::new(crate::neotrix::l5_consciousness_impl::nt_core_fep_iit::bridge::FEPIITBridge::new()));
        }

        // ConsciousnessGoldStandard
        if let Some(ref gs) = self.gold_standard {
            match gs.self_test() {
                Ok(()) => log::info!("[SELF-TEST] ConsciousnessGoldStandard ✅ pass"),
                Err(failures) => log::warn!("[SELF-TEST] ConsciousnessGoldStandard ❌ FAIL: {}", failures.join("; ")),
            }
        } else {
            self_tests.register(Box::new(crate::neotrix::l9_transcendent_impl::nt_mind_consciousness_gold_standard::ConsciousnessGoldStandard::new()));
        }

        // CognitiveLoadMonitor (existing clone pattern)
        if let Some(ref clm) = self.cognitive_load {
            match clm.self_test() {
                Ok(()) => log::info!("[SELF-TEST] CognitiveLoadMonitor ✅ pass"),
                Err(failures) => log::warn!("[SELF-TEST] CognitiveLoadMonitor ❌ FAIL: {}", failures.join("; ")),
            }
        } else {
            self_tests.register(Box::new(crate::core::nt_core_consciousness::CognitiveLoadMonitor::new()));
        }

        // ── L1 Shield + IO SelfTest registrations ──
        // (disabled: these types don't implement SelfTest yet)
        self_tests.register(Box::new(crate::neotrix::nt_memory_kb::nt_memory_commit_tracker::NarrativeConsistencyChecker::new()));

        // ── Run registry self-tests for remaining modules ──
        let results = self_tests.run_all();
        let mut failure_count = 0;
        for r in &results {
            if r.passed {
                log::info!("{}", r.summary());
            } else {
                log::warn!("{}", r.summary());
                failure_count += 1;
            }
        }
        
        // Pass SelfTest results to ConsciousnessTree for real branch health
        if let Some(ref mut tree) = self.consciousness_tree {
            tree.set_branch_health_from_self_tests(&results);
            log::debug!("[bg] consciousness_tree: branch health updated from {} SelfTest results", results.len());
        }
        
        if !report.findings.is_empty() || failure_count > 0 {
            let reason = format!(
                "arch_audit: {} converge issues + {} self-test failures — enqueueing self-review",
                report.findings.len(), failure_count,
            );
            log::warn!("[bg] {}", reason);
            if let Ok(mut brain) = self.brain.try_write() {
                self.goal_loop.enqueue_goal(&mut brain, &reason, None);
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
        
        let base_batch_size: usize = 50;
        let batch_size = match self.cognitive_mode {
            1 => base_batch_size * 2,
            2 => base_batch_size / 2,
            _ => base_batch_size,
        };
        
        let mode_name = match self.cognitive_mode {
            1 => "Deep",
            2 => "Fast",
            _ => "Balanced",
        };
        log::info!("[COGNITIVE MODE] {:?} — batch size adjusted to {}", mode_name, batch_size);
        
        let mut processed = 0;
        loop {
            if processed >= batch_size {
                log::debug!("[bg] crawl_queue: batch limit reached ({})", batch_size);
                break;
            }
            let (id, url) = {
                let kb = match self.kb_pipeline.kb.as_ref() {
                    Some(kb) => kb, None => break,
                };
                let conn = kb.conn.lock().unwrap_or_else(|e| e.into_inner());
                match claim_next_crawl_url(&conn) {
                    Ok(Some(item)) => (item.id, item.url),
                    _ => break,
                }
            };
            log::info!("[bg] crawl claimed: {} (domain tracked)", url);
            match self.kb_pipeline.absorb_url_async(&url).await {
                Ok(report) => {
                    log::info!("[bg] crawl absorbed: {} -> {} nodes (summary: {} chars)", 
                        report.url, report.nodes_created,
                        report.distil_summary.as_ref().map(|s| s.len()).unwrap_or(0));
                    let kb = match self.kb_pipeline.kb.as_ref() {
                        Some(kb) => kb, None => break,
                    };
                    let conn = kb.conn.lock().unwrap_or_else(|e| e.into_inner());
                    let _ = mark_crawl_complete(&conn, &id, true, None);
                }
                Err(e) => {
                    log::warn!("[bg] crawl failed: {}: {:?}", url, e);
                    let kb = match self.kb_pipeline.kb.as_ref() {
                        Some(kb) => kb, None => break,
                    };
                    let conn = kb.conn.lock().unwrap_or_else(|e| e.into_inner());
                    let _ = mark_crawl_complete(&conn, &id, false, Some(&e));
                }
            }
            processed += 1;
        }
        if let Some(kb) = self.kb_pipeline.kb.as_ref() {
            kb.rebuild_bm25();
            kb.rebuild_tech_reserve();
            log::info!("[bg] crawl_queue: BM25 + tech reserve rebuilt after batch ({} processed)", processed);
        }
    }

    async fn handle_constitution_reload(&mut self) {
        use crate::core::nt_core_self_constitution::ConstitutionLoader;
        let path = std::path::Path::new("AGENTS.md");
        if path.exists() {
            match ConstitutionLoader::load_from_file(path) {
                Ok(constitution) => {
                    log::info!("[constitution] Hot-reload: {} rules, {} experiences, {} tree-growth, {} absorption",
                        constitution.rules.len(),
                        constitution.experiences.len(),
                        constitution.tree_growth_rules.len(),
                        constitution.absorption_rules.len());
                    // Note: Global Constitution is LazyLock, so can't be replaced.
                    // In production, use a Mutex<Constitution> for true hot-reload.
                    // This reload validates the file is parseable.
                }
                Err(e) => log::warn!("[constitution] Hot-reload failed: {}", e),
            }
        }
    }

    /// Seed the crawl queue when nearly empty — runs daily.
    async fn handle_seed_crawl_queue(&mut self) {
        use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_store::count_nodes_by_domain;
        use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::enqueue_seed_urls;
        if self.kb_pipeline.kb.is_none() {
            log::warn!("[bg] seed_crawl: kb not attached");
            return;
        }
        let kb = match self.kb_pipeline.kb.as_ref() {
            Some(kb) => kb,
            None => { log::warn!("[bg] seed_crawl: kb disappeared"); return; }
        };
        let conn = match kb.conn.lock() {
            Ok(c) => c,
            Err(e) => { log::warn!("[bg] seed_crawl lock: {}", e); return; }
        };
        let domains = count_nodes_by_domain(&conn).unwrap_or_default();
        let seed_count = domains.len();
        if seed_count == 0 {
            let seed_info: [(&str, i64, &str); 5] = [
                ("rust", 10, "programming"),
                ("machine learning", 10, "ai"),
                ("distributed-systems", 10, "computer-science"),
                ("webassembly", 5, "programming"),
                ("neural-networks", 10, "ai"),
            ];
            let enqueued: Vec<String> = seed_info.iter()
                .map(|(topic, _, _)| format!("https://en.wikipedia.org/api/rest_v1/page/summary/{}", topic))
                .collect();
            let refs: Vec<(&str, i64, &str)> = enqueued.iter().enumerate()
                .map(|(i, url)| (url.as_str(), seed_info[i].1, seed_info[i].2))
                .collect();
            match enqueue_seed_urls(&conn, &refs) {
                Ok(n) => log::info!("[bg] seed_crawl: enqueued {} Wikipedia seed topics", n),
                Err(e) => log::warn!("[bg] seed_crawl: enqueue failed: {}", e),
            }
            drop(conn);
            kb.rebuild_bm25();
            log::info!("[bg] seed_crawl: BM25 rebuilt after seeding");
        } else {
            log::info!("[bg] seed_crawl: {} domains already in KB, auto-seeding complete", seed_count);
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

    use super::ConvergencePulse;
    use crate::core::nt_core_self_test::SelfTest;

    #[test]
    fn test_convergence_pulse_advance_no_gaps() {
        let mut p = ConvergencePulse::default();
        p.gaps = Vec::new();
        p.verified = true;
        let promoted = p.advance();
        assert!(promoted.is_some(), "complete layer should promote");
        assert_eq!(promoted.unwrap().name(), "task");
        assert_eq!(p.iteration, 0, "iteration resets on promotion");
    }

    #[test]
    fn test_convergence_pulse_open_gap_blocks_promotion() {
        let mut p = ConvergencePulse::default();
        p.gaps_from_self_tests(&[("substrate".to_string(), false)]);
        let before = p.layer;
        let promoted = p.advance();
        assert!(promoted.is_none(), "open gap must block promotion");
        assert_eq!(p.layer, before);
        assert_eq!(p.iteration, 1);
        assert!(!p.verified);
    }

    #[test]
    fn test_convergence_pulse_self_test() {
        let p = ConvergencePulse::default();
        let result = p.self_test();
        assert!(result.is_ok(), "default pulse self-test should pass: {:?}", result.err());
    }

    #[test]
    fn test_convergence_gaps_do_not_clobber_external_verification() {
        // P67 regression: 无 gap 时 gaps_from_self_tests 不得覆盖外部 cargo check 的 verified=false
        let mut p = ConvergencePulse::default();
        p.verified = true;
        p.gaps_from_self_tests(&[("substrate".to_string(), true)]);
        assert!(p.verified, "no gaps must not clear external verification");
        let promoted = p.advance();
        assert!(promoted.is_some(), "externally-verified complete layer should promote");
    }
}
