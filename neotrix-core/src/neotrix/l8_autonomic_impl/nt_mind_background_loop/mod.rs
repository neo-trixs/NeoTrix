#![deny(clippy::unwrap_used)]

use std::sync::Arc;
use tokio::sync::{RwLock, watch};
use tokio::task::JoinHandle;

use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
use crate::neotrix::nt_mind::knowledge_chain::KnowledgeChain;
use crate::neotrix::nt_mind::goal_loop::GoalLoop;
use crate::neotrix::nt_mind::distillation::MetaCognitionBridge;
use crate::neotrix::nt_mind::bbrain_monitor::BMonitor;
use self::always_on::AlwaysOnEngine;
use crate::neotrix::nt_mind_cleanup::CleanupEngine;
use crate::neotrix::nt_io_plugin::registry::PluginRegistry;
use crate::neotrix::nt_world_model::WorldModelV2;
use crate::neotrix::nt_mind_evolution_daemon::{EvolutionDaemon, EvolutionConfig};
use crate::neotrix::nt_mind::panorama_pipeline::PanoramaPipeline;
use crate::neotrix::nt_mind::exploration_pipeline::ExplorationPipeline;
use crate::neotrix::nt_act_voice::VoiceInput;
use crate::neotrix::l1_body_impl::nt_io_user_avatar::DistillationEngine;
use crate::neotrix::nt_mind::self_evolver::SelfEvolver;
use crate::core::nt_core_scheduler::SchedulerEngine;
use crate::neotrix::nt_mind::curiosity_drive::CuriosityDrive;
use crate::neotrix::nt_mind::knowledge_aging::KnowledgeAging;
use crate::neotrix::nt_mind::auto_crystallizer::AutoCrystallizer;
use crate::neotrix::l1_body_impl::nt_io_session_recovery::SessionRecoveryManager;
use crate::neotrix::nt_memory_kb::KnowledgeBase;

use crate::neotrix::nt_agent_protocol::discovery::AgentDiscovery;

use crate::core::nt_core_second_brain::SecondBrain;
use crate::core::nt_core_self::intra_reflection::PreActionIntrospector;
use crate::core::nt_core_meta::knowledge_gap_detector::KnowledgeGapDetector;
use crate::neotrix::nt_mind_consciousness_gold_standard::ConsciousnessGoldStandard;
use crate::neotrix::nt_mind_consciousness_monitor::ConsciousnessMonitor;
use crate::core::nt_core_consciousness::CognitiveLoadMonitor;
use crate::core::nt_core_gwt::workspace::GlobalWorkspace;

pub use crate::neotrix::nt_mind_background_config::{BackgroundConfig, TelemetryCollector, TelemetrySnapshot};

mod builder;
mod run;
mod handlers;
pub mod always_on;

pub use run::BackgroundLoopHandle;
pub use run::CONSCIOUSNESS_THRESHOLDS;

/// Broadcast shutdown signal to all background handler tasks.
/// Created in `start()`, consumed in `shutdown()`.
pub struct ShutdownCoordinator {
    pub sender: watch::Sender<bool>,
}

impl ShutdownCoordinator {
    pub fn new() -> (Self, watch::Receiver<bool>) {
        let (tx, rx) = watch::channel(false);
        (Self { sender: tx }, rx)
    }
}

pub struct BackgroundLoop {
    pub brain: Arc<RwLock<SelfIteratingBrain>>,
    pub config: BackgroundConfig,
    pub cleanup_engine: Option<CleanupEngine>,
    pub knowledge_chain: Option<KnowledgeChain>,
    pub telemetry: Arc<TelemetryCollector>,
    pub goal_loop: GoalLoop,
    pub metacognition: Option<MetaCognitionBridge>,
    pub bbrain: BMonitor,
    pub daemon: Option<EvolutionDaemon>,
    pub exploration_pipeline: Option<ExplorationPipeline>,
    pub nt_world_model: Option<WorldModelV2>,
    pub panorama: Option<PanoramaPipeline>,
    pub agent_discovery: Option<AgentDiscovery>,
    pub self_evolver: Option<SelfEvolver>,
    pub curiosity_drive: CuriosityDrive,
    pub knowledge_aging: KnowledgeAging,
    pub auto_crystallizer: AutoCrystallizer,
    pub introspector: Option<PreActionIntrospector>,
    pub gap_detector: Option<KnowledgeGapDetector>,
    pub awareness: Option<ConsciousnessMonitor>,
    pub gold_standard: Option<ConsciousnessGoldStandard>,
    #[cfg(feature = "stealth-net")]
    pub nt_shield_manager: Option<crate::neotrix::nt_shield_stealth_net::nt_shield_manager::StealthManager>,
    #[cfg(feature = "stealth-net")]
    pub tor_crawler: Option<std::sync::Arc<crate::neotrix::nt_shield_stealth_net::tor_crawler::TorCrawler>>,
    #[cfg(feature = "stealth-net")]
    pub heartbeat_engine: Option<crate::neotrix::nt_shield_stealth_net::ProxyHeartbeatEngine>,
    #[cfg(feature = "stealth-net")]
    pub world_consciousness: Option<crate::neotrix::nt_world_sense::WorldConsciousness>,
    #[cfg(feature = "stealth-net")]
    pub proxy_client: Option<crate::neotrix::nt_shield_stealth_net::proxy_control::ProxyClient>,
    pub nt_act_voice_input: Option<VoiceInput>,
    pub avatar_engine: Option<std::sync::Mutex<DistillationEngine>>,
    pub scheduler: Option<SchedulerEngine>,
    pub handles: Vec<JoinHandle<()>>,
    /// Broadcasts shutdown signal to all handler tasks.
    /// `Some` after `start()`, `None` after `shutdown()`.
    pub shutdown_coordinator: Option<ShutdownCoordinator>,
    /// Whether `start()` has been called. When true, most fields have been
    /// moved into `BackgroundLoopHandle` and `BackgroundLoop` should only
    /// be used for shutdown/bookkeeping (handles, config, brain).
    pub started: bool,
    pub always_on: AlwaysOnEngine,
    pub plugin_registry: PluginRegistry,
    pub session_recovery: Option<SessionRecoveryManager>,
    pub consciousness_runtime: Option<crate::core::nt_core_consciousness::consciousness_runtime::ConsciousnessRuntime>,
    pub consciousness_tree: Option<crate::core::nt_core_consciousness_tree::ConsciousnessTree>,
    pub fep_iit_bridge: Option<crate::neotrix::nt_core_fep_iit::FEPIITBridge>,
    pub cognitive_load: Option<CognitiveLoadMonitor>,
    pub second_brain: Option<SecondBrain>,
    pub kb: Option<Arc<KnowledgeBase>>,
    /// 统一的 GlobalWorkspace 单例 —— 被 engine、panorama、consciousness_bridge 共享。
    /// 消除 panorama 独立 new(13.0) 与 engine new(0.5) 的双实例分裂。
    pub gwt: Option<GlobalWorkspace>,
}

impl BackgroundLoop {
    pub fn new(brain: Arc<RwLock<SelfIteratingBrain>>) -> Self {
        Self {
            cleanup_engine: Some(CleanupEngine::new()),
            knowledge_chain: None,
            telemetry: Arc::new(TelemetryCollector::new()),
            goal_loop: GoalLoop::new(),
            metacognition: Some(MetaCognitionBridge::new(".")),
            bbrain: BMonitor::new(),
            daemon: Some(EvolutionDaemon::new(EvolutionConfig::default())),
            exploration_pipeline: None,
            nt_world_model: None,
            panorama: None,
            agent_discovery: None,
            self_evolver: None,
            curiosity_drive: CuriosityDrive::new(),
            knowledge_aging: KnowledgeAging::new(),
            auto_crystallizer: AutoCrystallizer::new(),
            introspector: Some(PreActionIntrospector::new()),
            gap_detector: Some(KnowledgeGapDetector::new()),
            awareness: Some(ConsciousnessMonitor::new()),
            gold_standard: Some(ConsciousnessGoldStandard::new()),
            #[cfg(feature = "stealth-net")]
            nt_shield_manager: Some(crate::neotrix::nt_shield_stealth_net::nt_shield_manager::StealthManager::new(5)),
            #[cfg(feature = "stealth-net")]
            tor_crawler: None,
            #[cfg(feature = "stealth-net")]
            heartbeat_engine: None,
            #[cfg(feature = "stealth-net")]
            world_consciousness: None,
            #[cfg(feature = "stealth-net")]
            proxy_client: None,
            nt_act_voice_input: Some(VoiceInput::new()),
            avatar_engine: Some(std::sync::Mutex::new(DistillationEngine::new())),
            scheduler: Some(crate::core::nt_core_scheduler::default_scheduler(
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
            )),
            handles: Vec::new(),
            shutdown_coordinator: None,
            always_on: AlwaysOnEngine::new(),
            plugin_registry: PluginRegistry::new(),
            config: BackgroundConfig::default(),
            brain,
            started: false,
            session_recovery: Some(SessionRecoveryManager::new("bg-loop")),
            consciousness_runtime: Some(crate::core::nt_core_consciousness::consciousness_runtime::ConsciousnessRuntime::new()),
            consciousness_tree: Some(crate::core::nt_core_consciousness_tree::ConsciousnessTree::new()),
            fep_iit_bridge: Some(crate::neotrix::nt_core_fep_iit::FEPIITBridge::new()),
            cognitive_load: Some(CognitiveLoadMonitor::new()),
            second_brain: Some(SecondBrain::new()),
            kb: None,
            gwt: Some(GlobalWorkspace::new(13.0)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tokio::sync::RwLock;

    use super::BackgroundLoop;
    use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;

    #[test]
    fn test_background_loop_wires_consciousness_core() {
        let bg = BackgroundLoop::new(Arc::new(RwLock::new(SelfIteratingBrain::new())));
        assert!(bg.gwt.is_some(), "GWT 单例必须存在 (F1 接线)");
        assert!(bg.consciousness_runtime.is_some(), "ConsciousnessRuntime 阵眼必须接线");
        assert!(bg.consciousness_tree.is_some(), "ConsciousnessTree 必须接线");
        assert!(bg.fep_iit_bridge.is_some(), "FEPIIT bridge 必须接线");
        assert!(bg.daemon.is_some(), "evolution daemon 必须接线");
        assert!(!bg.started, "new() 不得自动 start");
    }

    #[test]
    fn test_background_loop_gwt_singleton_shared_by_panorama() {
        let brain = Arc::new(RwLock::new(SelfIteratingBrain::new()));
        let bg = BackgroundLoop::new(brain.clone());
        let pre_pano_gwt = bg.gwt.is_some();
        assert!(pre_pano_gwt, "new() 必须预置 GWT 单例");
        let pano = crate::neotrix::nt_mind::panorama_pipeline::PanoramaPipeline::new();
        let bg = bg.with_panorama(pano);
        // B1: with_panorama 会把共享 gwt 注入 pano (self.gwt.take), 故 bg.gwt 转 None、pano 持有它
        assert!(bg.panorama.is_some(), "panorama 已注入");
        assert!(bg.gwt.is_none(), "gwt 已被注入 pano (B1 单例转移)");
        // 注入路径被证明执行过: bg.gwt 的 Some→None 转移即 B1 生效
        assert!(bg.panorama.as_ref().map_or(false, |p| p.cycle == 0),
            "panorama 实例保留");
    }
}
