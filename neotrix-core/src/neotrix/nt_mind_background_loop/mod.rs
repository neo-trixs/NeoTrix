use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use self::always_on::AlwaysOnEngine;
use super::nt_act_voice::VoiceInput;
use super::nt_expert_routing::WorldModelV2;
use super::nt_io_plugin::registry::PluginRegistry;
use super::nt_memory_knowledge_populator::KnowledgePopulator;
use super::nt_mind::auto_crystallizer::AutoCrystallizer;
use super::nt_mind::bbrain_monitor::BMonitor;
use super::nt_mind::curiosity_drive::CuriosityDrive;
use super::nt_mind::distillation::MetaCognitionBridge;
use super::nt_mind::exploration_pipeline::ExplorationPipeline;
use super::nt_mind::goal_loop::GoalLoop;
use super::nt_mind::knowledge_aging::KnowledgeAging;
use super::nt_mind::knowledge_chain::KnowledgeChain;
use super::nt_mind::panorama_pipeline::PanoramaPipeline;
use super::nt_mind::self_evolver::SelfEvolver;
use super::nt_mind::self_iterating::SelfIteratingBrain;
use super::nt_mind::thinking_bridge::ThinkingBridge;
use super::nt_mind_awakening::AwakeningEngine;
use super::nt_mind_cleanup::CleanupEngine;
use super::nt_mind_evolution_daemon::EvolutionDaemon;
use super::nt_mind_evolution_loop::EvolutionLoop;
use super::nt_shield_audit::{AuditReport, SecurityAuditor};
use super::nt_world_crawl::{CrawlerConfig, UnifiedCrawler};
use crate::core::nt_core_consciousness::adaptive_controller::AdaptiveController;
use crate::core::nt_core_network::NetworkSensor;
use crate::core::nt_core_scheduler::SchedulerEngine;
use crate::core::nt_core_vision::ImagePipeline;

use super::nt_agent_protocol::capabilities::CapabilityRouter;
use super::nt_agent_protocol::discovery::AgentDiscovery;
use super::nt_mind::web_miner::WebKnowledgeMiner;

use super::nt_act_project_manager::ProjectManager;
use super::nt_act_sync::FileSync;
use super::nt_agent_protocol::a2a::A2AServer;
use super::nt_agent_protocol::a2a_grpc::A2AGrpcServer;
use super::nt_mind_consciousness_gold_standard::ConsciousnessGoldStandard;
use super::nt_mind_consciousness_monitor::ConsciousnessMonitor;
use super::nt_shield::agent_anomaly::BehaviorAnomalyDetector;
use super::nt_shield::vsa_guard::VsaGuard;
use super::nt_shield::SecurityManager;
use super::nt_shield_prompt::AdvancedPromptGuard;
use super::nt_world_pred::PredictorState;
use crate::core::nt_core_agent::bus::AgentCommunicationBus;
use crate::core::nt_core_input::NgramVsaEncoder;
use crate::core::nt_core_meta::knowledge_gap_detector::KnowledgeGapDetector;
use crate::core::nt_core_meta::TimerRegistry;
use crate::core::nt_core_self::intra_reflection::PreActionIntrospector;
use crate::core::LoopEngine;
use crate::neotrix::nt_world_pred_hcube::{KnowledgeAugmentedPredictor, ReplayBuffer};

pub use super::nt_mind_background_config::{
    BackgroundConfig, TelemetryCollector, TelemetrySnapshot,
};

pub mod always_on;
mod builder;
pub mod consciousness;
mod handlers;
mod run;

pub use self::consciousness::types_consciousness::{
    GLOBAL_CONSCIOUSNESS_STATS, GLOBAL_STATS_READY,
};
pub use self::consciousness::{ConsciousnessIntegration, ExperienceStats};

pub struct BackgroundLoop {
    pub brain: Arc<RwLock<SelfIteratingBrain>>,
    pub config: BackgroundConfig,
    pub cleanup_engine: Option<CleanupEngine>,
    pub knowledge_chain: Option<KnowledgeChain>,
    pub telemetry: Arc<TelemetryCollector>,
    pub goal_loop: GoalLoop,
    pub metacognition: MetaCognitionBridge,
    pub thinking: ThinkingBridge,
    pub bbrain: BMonitor,
    pub evolution: EvolutionLoop,
    pub daemon: Option<EvolutionDaemon>,
    pub nt_world_crawl: Option<UnifiedCrawler>,
    pub exploration_pipeline: Option<ExplorationPipeline>,
    pub nt_world_model: Option<WorldModelV2>,
    pub panorama: Option<PanoramaPipeline>,
    pub agent_discovery: Option<AgentDiscovery>,
    pub capability_router: Option<CapabilityRouter>,
    pub self_evolver: Option<SelfEvolver>,
    pub curiosity_drive: CuriosityDrive,
    pub knowledge_aging: KnowledgeAging,
    pub auto_crystallizer: AutoCrystallizer,
    pub web_miner: Option<WebKnowledgeMiner>,
    pub introspector: Option<PreActionIntrospector>,
    pub gap_detector: Option<KnowledgeGapDetector>,
    pub awareness: Option<ConsciousnessMonitor>,
    pub gold_standard: Option<ConsciousnessGoldStandard>,
    pub adaptive_controller: Option<AdaptiveController>,
    #[cfg(feature = "stealth-net")]
    pub nt_shield_manager: Option<super::nt_shield_stealth_net::nt_shield_manager::StealthManager>,
    #[cfg(feature = "stealth-net")]
    pub tor_crawler: Option<std::sync::Arc<super::nt_shield_stealth_net::tor_crawler::TorCrawler>>,
    #[cfg(feature = "stealth-net")]
    pub heartbeat_engine: Option<super::nt_shield_stealth_net::ProxyHeartbeatEngine>,
    #[cfg(feature = "stealth-net")]
    pub rotation_coordinator:
        Option<std::sync::Arc<super::nt_shield_stealth_net::RotationCoordinator>>,
    #[cfg(feature = "stealth-net")]
    pub world_consciousness: Option<crate::neotrix::nt_world_sense::WorldConsciousness>,
    #[cfg(feature = "stealth-net")]
    pub proxy_client: Option<super::nt_shield_stealth_net::proxy_control::ProxyClient>,
    #[cfg(feature = "stealth-net")]
    pub transit_station:
        Option<std::sync::Arc<super::nt_shield_stealth_net::transit_station::TransitStation>>,
    #[cfg(feature = "stealth-net")]
    pub ip_rotator: Option<std::sync::Arc<super::nt_shield_stealth_net::ip_rotator::OsIpRotator>>,
    pub nt_act_voice_input: Option<VoiceInput>,
    pub handles: Vec<JoinHandle<()>>,
    pub always_on: AlwaysOnEngine,
    pub plugin_registry: PluginRegistry,
    pub consciousness: Option<ConsciousnessIntegration>,
    pub vsa_encoder: Option<NgramVsaEncoder>,
    pub advanced_prompt_guard: Option<AdvancedPromptGuard>,
    pub vsa_guard: Option<VsaGuard>,
    pub behavior_anomaly: Option<BehaviorAnomalyDetector>,
    pub a2a_server: Option<A2AServer>,
    pub a2a_grpc_server: Option<A2AGrpcServer>,
    pub scheduler: SchedulerEngine,
    pub network_sensor: Option<NetworkSensor>,
    pub image_pipeline: Option<ImagePipeline>,
    pub awakening: Option<AwakeningEngine>,
    pub audit_report: Option<AuditReport>,
    pub nt_act_sync: Option<FileSync>,
    pub nt_act_project_manager: Option<ProjectManager>,
    pub nt_world_pred_hcube: Option<KnowledgeAugmentedPredictor>,
    pub nt_world_replay_buffer: Option<ReplayBuffer>,
    pub nt_world_predictor: Option<PredictorState>,
    pub timer_registry: TimerRegistry,
    pub agent_bus: Option<AgentCommunicationBus>,
    pub security_manager: Option<SecurityManager>,
    pub loop_engine: LoopEngine,
    pub event_scheduler: crate::core::nt_core_scheduler::event_driven::EventDrivenScheduler,
    pub event_rx: Option<
        tokio::sync::mpsc::Receiver<
            crate::core::nt_core_scheduler::event_driven::ConsciousnessEvent,
        >,
    >,
    pub event_sender: Option<
        tokio::sync::mpsc::Sender<crate::core::nt_core_scheduler::event_driven::ConsciousnessEvent>,
    >,
    pub dropped_events: AtomicU64,
    pub health_patrol_ok: bool,
    pub thinking_in_progress: bool,
    pub thinking_cycle_counter: u8,
    pub shutdown: crate::core::nt_core_shutdown::GracefulShutdown,
    /// External input queue for the consciousness pipeline.
    /// Write with `bg.ci_pending_input.lock().unwrap().push("text".to_string())`.
    pub ci_pending_input: Arc<Mutex<Vec<String>>>,
    /// External output buffer — the background loop pushes drained CI response
    /// text here so that external consumers (Tauri bridge) can read it without
    /// owning the CI instance.
    pub ci_response_output: Arc<Mutex<Vec<String>>>,
    /// Shared stats snapshot updated after each consciousness cycle.
    /// External consumers (e.g., Tauri bridge) can read the latest stats
    /// without owning the CI instance.
    pub stats_snapshot: Arc<std::sync::RwLock<ExperienceStats>>,
    /// Shared canvas project snapshot for E8 reasoning graph visualization.
    /// Updated after canvas handler runs each cycle.
    pub canvas_snapshot: Arc<std::sync::RwLock<neotrix_types::core::node_canvas::CanvasProject>>,
}

impl BackgroundLoop {
    pub fn new(brain: Arc<RwLock<SelfIteratingBrain>>) -> Self {
        let (ev_sched, ev_rx) =
            crate::core::nt_core_scheduler::event_driven::EventDrivenScheduler::new(4096);
        let ev_sender = ev_sched.sender();
        Self {
            cleanup_engine: Some(CleanupEngine::new()),
            knowledge_chain: None,
            telemetry: Arc::new(TelemetryCollector::new()),
            goal_loop: GoalLoop::new(),
            metacognition: MetaCognitionBridge::new("."),
            thinking: ThinkingBridge::new("."),
            bbrain: BMonitor::new(),
            evolution: EvolutionLoop::new(),
            daemon: None,
            nt_world_crawl: None,
            exploration_pipeline: None,
            nt_world_model: None,
            panorama: None,
            agent_discovery: None,
            capability_router: None,
            self_evolver: None,
            curiosity_drive: CuriosityDrive::new(),
            knowledge_aging: KnowledgeAging::new(),
            auto_crystallizer: AutoCrystallizer::new(),
            web_miner: None,
            introspector: Some(PreActionIntrospector::new()),
            gap_detector: Some(KnowledgeGapDetector::new()),
            awareness: Some(ConsciousnessMonitor::new()),
            gold_standard: Some(ConsciousnessGoldStandard::new()),
            adaptive_controller: None,
            #[cfg(feature = "stealth-net")]
            nt_shield_manager: Some(
                super::nt_shield_stealth_net::nt_shield_manager::StealthManager::new(5),
            ),
            #[cfg(feature = "stealth-net")]
            tor_crawler: None,
            #[cfg(feature = "stealth-net")]
            heartbeat_engine: None,
            #[cfg(feature = "stealth-net")]
            rotation_coordinator: None,
            #[cfg(feature = "stealth-net")]
            world_consciousness: None,
            #[cfg(feature = "stealth-net")]
            proxy_client: None,
            #[cfg(feature = "stealth-net")]
            transit_station: None,
            #[cfg(feature = "stealth-net")]
            ip_rotator: None,
            nt_act_voice_input: Some(VoiceInput::new()),
            handles: Vec::new(),
            always_on: AlwaysOnEngine::new(),
            plugin_registry: PluginRegistry::new(),
            config: BackgroundConfig::default(),
            brain,
            consciousness: None,
            vsa_encoder: None,
            advanced_prompt_guard: Some(AdvancedPromptGuard::new()),
            vsa_guard: Some(VsaGuard::new()),
            behavior_anomaly: Some(BehaviorAnomalyDetector::new()),
            a2a_server: None,
            a2a_grpc_server: None,
            scheduler: SchedulerEngine::new(),
            network_sensor: None,
            image_pipeline: None,
            awakening: Some(AwakeningEngine::default()),
            audit_report: None,
            nt_act_sync: None,
            nt_act_project_manager: None,
            nt_world_pred_hcube: Some(KnowledgeAugmentedPredictor::new()),
            nt_world_replay_buffer: Some(ReplayBuffer::new(1000)),
            nt_world_predictor: Some(PredictorState::new(
                crate::neotrix::nt_world_pred::PredictorConfig::default(),
            )),
            timer_registry: TimerRegistry::new(),
            agent_bus: None,
            security_manager: Some(SecurityManager::new()),
            loop_engine: LoopEngine::new(),
            event_scheduler: ev_sched,
            event_rx: Some(ev_rx),
            event_sender: Some(ev_sender),
            dropped_events: AtomicU64::new(0),
            health_patrol_ok: false,
            thinking_in_progress: false,
            thinking_cycle_counter: 0,
            shutdown: crate::core::nt_core_shutdown::GracefulShutdown::new(30),
            ci_pending_input: Arc::new(Mutex::new(Vec::new())),
            ci_response_output: Arc::new(Mutex::new(Vec::new())),
            stats_snapshot: Arc::new(std::sync::RwLock::new(ExperienceStats {
                c_score: 0.0,
                sp_coherence: 0.0,
                nm_da: 0.0,
                nm_ne: 0.0,
                nm_ht: 0.0,
                nm_ach: 0.0,
                critic_pass_rate: 0.0,
                load_mode: 0,
                vsa_buffer_size: 0,
                text_feed_total: 0,
                reflexivity: 0.0,
                emotion: "init".to_string(),
                critic_issued: 0,
                cycle: 0,
                last_critique: crate::core::nt_core_consciousness::CritiqueResult::perfect(),
            })),
            canvas_snapshot: Arc::new(std::sync::RwLock::new(
                neotrix_types::core::node_canvas::CanvasProject::new("E8 Reasoning Graph"),
            )),
        }
    }
}

impl Drop for BackgroundLoop {
    fn drop(&mut self) {
        for handle in self.handles.drain(..) {
            handle.abort();
        }
    }
}
