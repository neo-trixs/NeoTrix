use super::*;
use crate::core::nt_core_gate::{GateDecision, ToolRegistry};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::Mutex;

#[path = "handlers_core.rs"]
mod handlers_core;
#[path = "handlers_consciousness.rs"]
mod handlers_consciousness;
#[path = "handlers_maintenance.rs"]
mod handlers_maintenance;

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
use crate::neotrix::l8_autonomic_impl::nt_mind_hook::{HookEvent, MindHookRegistry, LogHook};
use crate::neotrix::l8_autonomic_impl::nt_mind_knowledge_pipeline::KnowledgeAbsorptionPipeline;
use crate::neotrix::l1_body_impl::nt_io_session_recovery::SessionRecoveryManager;
use crate::neotrix::nt_core_event_bus::{EventBus, subscribe_all_layers_sync};
use crate::neotrix::nt_mind::distillation::MetaCognitionBridge;
use crate::core::nt_core_event::CoreEvent;
use crate::core::nt_core_state_substrate::StateSubstrate;
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
            skill_engine: {
                let mut hooks = MindHookRegistry::default();
                hooks.register(HookEvent::SkillLoaded, Box::new(LogHook::new("bg-loop")));
                hooks.register(HookEvent::SkillUnloaded, Box::new(LogHook::new("bg-loop")));
                SkillEngine::new(PathBuf::from(
                    &dirs::home_dir().unwrap_or_default().join(".claude").join("skills"),
                )).with_hooks(hooks)
            },
            kb_pipeline,
            session_recovery: self.session_recovery.take(),
            event_bus: Some(event_bus.as_ref().clone()),
            metacognition: self.metacognition.take(),
            #[cfg(feature = "stealth-net")]
            world_consciousness: self.world_consciousness.take(),
            #[cfg(not(feature = "stealth-net"))]
            world_consciousness: None,
            consciousness_runtime: {
                let mut cr = std::mem::take(&mut self.consciousness_runtime);
                if let Some(ref kb_ref) = self.kb {
                    if let Some(ref mut runtime) = cr {
                        runtime.attach_kb(kb_ref.clone());
                    }
                }
                cr
            },
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
            state: StateSubstrate::new(),
            simulate: SimulateEngine::new(),
            convergence_pulse: ConvergencePulse::default(),
            tool_grounding: crate::core::nt_core_self::self_audit::ToolGroundingMonitor::new(),
            /// 门控注册表 — 默认只读工具, 运行时可扩展。
            gate_registry: Some(ToolRegistry::from_read_only(&["get", "query", "read", "search"])),
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
    gap_detector: Option<KnowledgeGapDetector>,
    nt_act_voice_input: Option<VoiceInput>,
    avatar_engine: Option<std::sync::Mutex<crate::neotrix::l1_body_impl::nt_io_user_avatar::DistillationEngine>>,
    self_evolver: Option<SelfEvolver>,
    curiosity_drive: CuriosityDrive,
    knowledge_aging: KnowledgeAging,
    auto_crystallizer: AutoCrystallizer,
    knowledge_chain: Option<KnowledgeChain>,
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
    state: StateSubstrate,
    simulate: SimulateEngine,
    convergence_pulse: ConvergencePulse,
    tool_grounding: crate::core::nt_core_self::self_audit::ToolGroundingMonitor,
    /// 门控注册表 — 背景循环工具执行前置检查用。
    gate_registry: Option<ToolRegistry>,
}

impl BackgroundLoopHandle {
    fn try_emit(&self, event: crate::core::nt_core_event::CoreEvent) {
        if let Some(ref bus) = self.event_bus { bus.emit(event); }
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
