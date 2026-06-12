use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use self::always_on::AlwaysOnEngine;
use super::nt_act_voice::VoiceInput;
use super::nt_io_plugin::registry::PluginRegistry;
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
use super::nt_mind_cleanup::CleanupEngine;
use super::nt_mind_evolution_daemon::EvolutionDaemon;
use crate::core::nt_core_scheduler::SchedulerEngine;
use super::nt_mind_evolution_loop::EvolutionLoop;
use super::nt_world_crawl::{CrawlerConfig, UnifiedCrawler};
use super::nt_world_model::WorldModelV2;

use super::nt_agent_protocol::capabilities::CapabilityRouter;
use super::nt_agent_protocol::discovery::AgentDiscovery;
use super::nt_mind::web_miner::WebKnowledgeMiner;

use super::nt_mind_consciousness_gold_standard::ConsciousnessGoldStandard;
use super::nt_mind_consciousness_monitor::ConsciousnessMonitor;
use crate::core::nt_core_input::NgramVsaEncoder;
use crate::core::nt_core_meta::knowledge_gap_detector::KnowledgeGapDetector;
use crate::core::nt_core_self::intra_reflection::PreActionIntrospector;

pub use super::nt_mind_background_config::{
    BackgroundConfig, TelemetryCollector, TelemetrySnapshot,
};

pub mod always_on;
mod builder;
mod consciousness;
mod handlers;
mod run;

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
    #[cfg(feature = "stealth-net")]
    pub nt_shield_manager: Option<super::nt_shield_stealth_net::nt_shield_manager::StealthManager>,
    #[cfg(feature = "stealth-net")]
    pub tor_crawler: Option<std::sync::Arc<super::nt_shield_stealth_net::tor_crawler::TorCrawler>>,
    #[cfg(feature = "stealth-net")]
    pub heartbeat_engine: Option<super::nt_shield_stealth_net::ProxyHeartbeatEngine>,
    #[cfg(feature = "stealth-net")]
    pub world_consciousness: Option<crate::neotrix::nt_world_sense::WorldConsciousness>,
    #[cfg(feature = "stealth-net")]
    pub proxy_client: Option<super::nt_shield_stealth_net::proxy_control::ProxyClient>,
    pub nt_act_voice_input: Option<VoiceInput>,
    pub handles: Vec<JoinHandle<()>>,
    pub always_on: AlwaysOnEngine,
    pub plugin_registry: PluginRegistry,
    pub consciousness: Option<ConsciousnessIntegration>,
    pub vsa_encoder: Option<NgramVsaEncoder>,
    pub scheduler: SchedulerEngine,
}

impl BackgroundLoop {
    pub fn new(brain: Arc<RwLock<SelfIteratingBrain>>) -> Self {
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
            #[cfg(feature = "stealth-net")]
            nt_shield_manager: Some(
                super::nt_shield_stealth_net::nt_shield_manager::StealthManager::new(5),
            ),
            #[cfg(feature = "stealth-net")]
            tor_crawler: None,
            #[cfg(feature = "stealth-net")]
            heartbeat_engine: None,
            #[cfg(feature = "stealth-net")]
            world_consciousness: None,
            #[cfg(feature = "stealth-net")]
            proxy_client: None,
            nt_act_voice_input: Some(VoiceInput::new()),
            handles: Vec::new(),
            always_on: AlwaysOnEngine::new(),
            plugin_registry: PluginRegistry::new(),
            config: BackgroundConfig::default(),
            brain,
            consciousness: None,
            vsa_encoder: None,
            scheduler: SchedulerEngine::new(),
        }
    }
}
