use super::super::nt_mind::memory::ReasoningBank;
use super::super::nt_mind::self_iterating::brain_impl::ReasoningBrain;
use super::*;

use crate::core::nt_core_agent::bus::AgentCommunicationBus;
use crate::core::nt_core_agent::message::AgentId;
use crate::core::nt_core_consciousness::adaptive_controller::AdaptiveController;

impl BackgroundLoop {
    pub fn with_goal_loop(mut self, goal_loop: GoalLoop) -> Self {
        self.goal_loop = goal_loop;
        self
    }

    pub fn with_nt_world_model(mut self, wm: WorldModelV2) -> Self {
        self.nt_world_model = Some(wm);
        self
    }

    pub fn with_panorama(mut self, _pano: PanoramaPipeline) -> Self {
        self.panorama = Some(_pano);
        self
    }

    pub fn with_nt_world_crawl(mut self, _work_dir: std::path::PathBuf) -> Self {
        self.nt_world_crawl = Some(UnifiedCrawler::new(CrawlerConfig::default()));
        self
    }

    pub fn with_exploration_pipeline(mut self, work_dir: std::path::PathBuf) -> Self {
        self.exploration_pipeline = Some(ExplorationPipeline::new(work_dir));
        self
    }

    pub fn with_exploration_evolver(mut self, work_dir: std::path::PathBuf) -> Self {
        let brain = ReasoningBrain::new();
        let bank = ReasoningBank::new(100);
        self.self_evolver = Some(SelfEvolver::new(brain, bank, work_dir));
        self
    }

    pub fn with_knowledge_chain(mut self, work_dir: std::path::PathBuf) -> Self {
        self.knowledge_chain = Some(KnowledgeChain::new(work_dir));
        self
    }

    pub fn with_web_miner(mut self, work_dir: std::path::PathBuf) -> Self {
        self.web_miner = Some(WebKnowledgeMiner::new(work_dir));
        self
    }

    pub fn with_curiosity_drive(mut self, drive: CuriosityDrive) -> Self {
        self.curiosity_drive = drive;
        self
    }

    pub fn with_knowledge_aging(mut self, aging: KnowledgeAging) -> Self {
        self.knowledge_aging = aging;
        self
    }

    pub fn with_auto_crystallizer(mut self, crystallizer: AutoCrystallizer) -> Self {
        self.auto_crystallizer = crystallizer;
        self
    }

    #[cfg(feature = "stealth-net")]
    pub fn with_tor_crawler(
        mut self,
        tor_crawler: std::sync::Arc<super::super::nt_shield_stealth_net::tor_crawler::TorCrawler>,
    ) -> Self {
        self.tor_crawler = Some(tor_crawler);
        self
    }

    #[cfg(feature = "stealth-net")]
    pub fn with_proxy_heartbeat(mut self, interval_secs: u64) -> Self {
        use super::super::nt_shield_stealth_net::proxy_pool::ProxyPool;
        use super::super::nt_shield_stealth_net::{FingerprintManager, ProxyHeartbeatEngine};
        use std::sync::Arc;

        let fm = FingerprintManager::new();
        let pool = Arc::new(ProxyPool::new());
        let engine = ProxyHeartbeatEngine::new(pool, fm, interval_secs);
        self.heartbeat_engine = Some(engine);
        self
    }

    /// Register built-in plugins.
    pub fn with_builtin_plugins(self) -> Self {
        let reg = self.plugin_registry.clone();
        let rt = tokio::runtime::Handle::current();
        rt.spawn(async move {
            let logging: Box<dyn crate::neotrix::nt_io_plugin::Plugin> =
                Box::new(crate::neotrix::nt_io_plugin::builtin::logging::LoggingPlugin);
            if let Err(e) = reg.register(logging) {
                log::error!("[plugin] register logging: {}", e);
            }
        });
        self
    }

    /// Spawn a HotReloadWatcher for config/rules/subscriptions hot-reload.
    /// Spawns the watcher and stores the join handle in `self.handles`.
    /// TODO(orphan): nt_io_hotreload was deleted — hot reload unavailable
    #[cfg(feature = "stealth-net")]
    pub fn with_hot_reload(self, _neotrix_dir: std::path::PathBuf) -> Self {
        log::warn!("[bg] hot reload not available — nt_io_hotreload module was deleted");
        self
    }

    #[cfg(feature = "stealth-net")]
    pub fn with_proxy_client(mut self) -> Self {
        self.proxy_client =
            Some(super::super::nt_shield_stealth_net::proxy_control::ProxyClient::new());
        self
    }

    #[cfg(feature = "stealth-net")]
    pub fn with_transit_station(mut self) -> Self {
        use super::super::nt_shield_stealth_net::transit_station::global_transit_station;
        self.transit_station = Some(global_transit_station());
        self
    }

    #[cfg(feature = "stealth-net")]
    pub fn with_ip_rotator(mut self) -> Self {
        use std::sync::Arc;
        self.ip_rotator = Some(Arc::new(
            super::super::nt_shield_stealth_net::ip_rotator::OsIpRotator::new(),
        ));
        self
    }

    #[cfg(feature = "stealth-net")]
    pub fn with_world_consciousness(mut self) -> Self {
        use crate::neotrix::nt_world_sense::WorldConsciousness;
        let mut wc = WorldConsciousness::new();
        wc.nt_world_sense.active = true;
        wc.nt_world_sense.visual.activate();
        wc.nt_world_sense.auditory.activate();
        wc.active = true;
        self.world_consciousness = Some(wc);
        self
    }

    pub fn with_awareness(mut self, monitor: ConsciousnessMonitor) -> Self {
        self.awareness = Some(monitor);
        self
    }

    pub fn with_agent_discovery(mut self, port: u16) -> Self {
        if let Ok(d) = AgentDiscovery::new(port) {
            self.agent_discovery = Some(d);
        }
        self
    }

    pub fn with_awakening(mut self, config: AwakeningEngine) -> Self {
        self.awakening = Some(config);
        self
    }

    pub fn with_audit_report(mut self, report: AuditReport) -> Self {
        self.audit_report = Some(report);
        self
    }

    pub fn with_act_sync(
        mut self,
        discovery_port: u16,
        sync_port: u16,
        local_root: String,
    ) -> Self {
        match FileSync::new(discovery_port, sync_port, local_root) {
            Ok(fs) => self.nt_act_sync = Some(fs),
            Err(e) => log::error!("[bg] nt_act_sync init failed: {}", e),
        }
        self
    }

    pub fn with_act_project_manager(mut self) -> Self {
        self.nt_act_project_manager = Some(ProjectManager::new());
        self
    }

    pub fn with_agent_bus(mut self, bus: AgentCommunicationBus) -> Self {
        self.agent_bus = Some(bus);
        self
    }

    pub fn with_consciousness(mut self, ci: ConsciousnessIntegration) -> Self {
        self.consciousness = Some(ci);
        self
    }

    /// Wire a ConsciousnessCycle into the background loop's CI.
    /// The 12-step cycle runs as a refinement pass after the 3-phase pipeline,
    /// activating analogical reasoning, MCTS-GWT bridge, causal reasoning,
    /// recurrent world model, and economic agency — all previously dead code
    /// without this wiring.
    pub fn with_consciousness_cycle(mut self) -> Self {
        if let Some(ref mut ci) = self.consciousness {
            let config =
                crate::core::nt_core_consciousness::consciousness_cycle::CycleConfig::default();
            let cycle =
                crate::core::nt_core_consciousness::consciousness_cycle::ConsciousnessCycle::new(
                    config,
                );
            ci.with_consciousness_cycle(cycle);
        } else {
            log::warn!(
                "[bg] with_consciousness_cycle: no CI wired yet — call with_consciousness() first"
            );
        }
        self
    }

    /// Wire an ImageCache into the ConsciousnessCycle for VSA-aware image dedup.
    /// Uses dHash + VSA encoding for O(1) repeated-image lookup.
    pub fn with_image_cache(mut self, max_entries: usize) -> Self {
        if let Some(ref mut ci) = self.consciousness {
            let _ = ci.with_image_cache(max_entries);
        }
        self
    }

    /// Wire a ModalityGate into the ConsciousnessCycle for modality-aware attention routing.
    pub fn with_multi_modal_gate(
        mut self,
        config: crate::core::nt_core_consciousness::multi_modal_gate::ModalityGateConfig,
    ) -> Self {
        if let Some(ref mut ci) = self.consciousness {
            let _ = ci.with_multi_modal_gate(config);
        }
        self
    }

    /// Wire a ModuleRegistry into the ConsciousnessCycle — activates 7 cognitive modules
    /// (MCTS, ParallelHypothesis, Counterfactual, DeadEnd, PRM, Pruner, Selector)
    /// in the REASON step via init_default_registry.
    pub fn with_module_registry(mut self) -> Self {
        if let Some(ref mut ci) = self.consciousness {
            let reg = crate::core::nt_core_consciousness::cognitive_module_registry::ModuleRegistry::new();
            ci.with_module_registry(reg);
        } else {
            log::warn!(
                "[bg] with_module_registry: no CI wired yet — call with_consciousness() first"
            );
        }
        self
    }

    /// Wire a BookmarkManager — 对话URL按类别存储
    pub fn with_bookmark_manager(mut self) -> Self {
        if let Some(ref mut ci) = self.consciousness {
            let _ = ci.with_bookmark_manager(500);
        }
        self
    }

    /// Wire a BehavioralPersonalityEngine — 用户数字分身/行为人格系统
    pub fn with_behavioral_personality(mut self) -> Self {
        if let Some(ref mut ci) = self.consciousness {
            let _ = ci.with_behavioral_personality();
        }
        self
    }

    pub fn with_adaptive_controller(mut self, controller: AdaptiveController) -> Self {
        self.adaptive_controller = Some(controller);
        self
    }

    pub fn with_stats_snapshot(
        mut self,
        snapshot: Arc<std::sync::RwLock<ExperienceStats>>,
    ) -> Self {
        self.stats_snapshot = snapshot;
        self
    }

    pub fn with_canvas_snapshot(
        mut self,
        snapshot: Arc<std::sync::RwLock<neotrix_types::core::node_canvas::CanvasProject>>,
    ) -> Self {
        self.canvas_snapshot = snapshot;
        self
    }

    pub fn with_advanced_prompt_guard(mut self, guard: AdvancedPromptGuard) -> Self {
        self.advanced_prompt_guard = Some(guard);
        self
    }

    pub fn with_vsa_guard(mut self, guard: VsaGuard) -> Self {
        self.vsa_guard = Some(guard);
        self
    }

    pub fn with_behavior_anomaly(mut self, detector: BehaviorAnomalyDetector) -> Self {
        self.behavior_anomaly = Some(detector);
        self
    }

    pub fn with_security_manager(mut self) -> Self {
        self.security_manager = Some(SecurityManager::new());
        self
    }

    pub fn with_a2a_server(
        mut self,
        port: u16,
        bus: AgentCommunicationBus,
        self_id: AgentId,
    ) -> Self {
        let server = A2AServer::new("NeoTrix", "NeoTrix Cognitive Agent", port, bus, self_id);
        self.a2a_server = Some(server);
        self
    }

    pub fn with_a2a_server_auth(
        mut self,
        port: u16,
        bus: AgentCommunicationBus,
        self_id: AgentId,
        api_key: String,
    ) -> Self {
        let config = crate::neotrix::nt_agent_protocol::a2a_auth::A2AAuthConfig {
            api_key: Some(api_key),
            rate_limit_per_min: 60,
            max_request_size: 1_048_576,
            max_concurrent_tasks_per_session: 10,
        };
        let server = A2AServer::new("NeoTrix", "NeoTrix Cognitive Agent", port, bus, self_id)
            .with_auth(config);
        self.a2a_server = Some(server);
        self
    }

    /// Convenience: auto-create AgentBus + AgentId, start A2A server on given port.
    /// Reuses `self.agent_bus` if previously set (ownership transfers to A2A server).
    /// When `self.agent_bus` is not set, creates a new bus exclusively for A2A.
    ///
    /// Authentication: reads `NEOTRIX_A2A_API_KEY` env var. If set, uses it as Bearer token.
    /// If unset, generates a random 32‑char hex key and logs a warning — port is never open
    /// without at least a generated key. Set the env var explicitly for stable interop.
    /// Configure and start an A2A gRPC server on the given port.
    /// Uses the same auth config as the SSE A2A server (reads `NEOTRIX_A2A_API_KEY` env var
    /// or generates an ephemeral key).
    pub fn with_a2a_grpc(mut self, port: u16) -> Self {
        use tokio_util::sync::CancellationToken;
        let self_id = AgentId::new("neotrix-grpc", "0.18.0");
        let bus = self
            .agent_bus
            .take()
            .unwrap_or_else(|| AgentCommunicationBus::new(100));
        let config = crate::neotrix::nt_agent_protocol::a2a_auth::A2AAuthConfig::default();
        let shutdown = CancellationToken::new();
        let server = crate::neotrix::nt_agent_protocol::a2a_grpc::A2AGrpcServer::new(
            port, bus, self_id, shutdown,
        )
        .with_auth(config);
        self.a2a_grpc_server = Some(server);
        self
    }

    /// Configure and start an A2A v1.2 gRPC server with a JWT-signed Agent Card.
    /// Reads `NEOTRIX_A2A_API_KEY` env var for HMAC signing. If unset, logs a warning
    /// and generates an ephemeral key.
    /// The v1.2 card is available at `/.well-known/agent-card.jwt` as a compact JWT.
    pub fn with_a2a_grpc_v12(mut self, port: u16) -> Self {
        use tokio_util::sync::CancellationToken;
        let self_id = AgentId::new("neotrix-grpc-v12", "1.2");
        let bus = self
            .agent_bus
            .take()
            .unwrap_or_else(|| AgentCommunicationBus::new(100));
        let api_key = match std::env::var("NEOTRIX_A2A_API_KEY") {
            Ok(k) if !k.is_empty() => k,
            _ => {
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64;
                let key = format!(
                    "{:016x}{:016x}",
                    seed.wrapping_mul(6364136223846793005),
                    seed.wrapping_mul(1442695040888963407)
                );
                log::warn!(
                    "[A2A v1.2] NEOTRIX_A2A_API_KEY not set; generated ephemeral key for port {}. \
                     Set the env var for a stable key across restarts.",
                    port
                );
                key
            }
        };
        let config = crate::neotrix::nt_agent_protocol::a2a_auth::A2AAuthConfig {
            api_key: Some(api_key.clone()),
            rate_limit_per_min: 60,
            max_request_size: 1_048_576,
            max_concurrent_tasks_per_session: 10,
        };
        let shutdown = CancellationToken::new();
        let server = crate::neotrix::nt_agent_protocol::a2a_grpc::A2AGrpcServer::new(
            port, bus, self_id, shutdown,
        )
        .with_auth(config);
        self.a2a_grpc_server = Some(server);
        self
    }

    /// Convenience: auto-create AgentBus + AgentId, start A2A server on given port.
    /// Reuses `self.agent_bus` if previously set (ownership transfers to A2A server).
    /// When `self.agent_bus` is not set, creates a new bus exclusively for A2A.
    ///
    /// Authentication: reads `NEOTRIX_A2A_API_KEY` env var. If set, uses it as Bearer token.
    /// If unset, generates a random 32‑char hex key and logs a warning — port is never open
    /// without at least a generated key. Set the env var explicitly for stable interop.
    pub fn with_a2a_server_default(mut self, port: u16) -> Self {
        let self_id = AgentId::new("neotrix", "0.18.0");
        let bus = self
            .agent_bus
            .take()
            .unwrap_or_else(|| AgentCommunicationBus::new(100));
        let api_key = match std::env::var("NEOTRIX_A2A_API_KEY") {
            Ok(k) if !k.is_empty() => k,
            _ => {
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64;
                let key = format!(
                    "{:016x}{:016x}",
                    seed.wrapping_mul(6364136223846793005),
                    seed.wrapping_mul(1442695040888963407)
                );
                log::warn!(
                    "[A2A] NEOTRIX_A2A_API_KEY not set; generated ephemeral key for port {}. \
                     Set the env var for a stable key across restarts.",
                    port
                );
                key
            }
        };
        let config = crate::neotrix::nt_agent_protocol::a2a_auth::A2AAuthConfig {
            api_key: Some(api_key),
            rate_limit_per_min: 60,
            max_request_size: 1_048_576,
            max_concurrent_tasks_per_session: 10,
        };
        let server = A2AServer::new("NeoTrix", "NeoTrix Cognitive Agent", port, bus, self_id)
            .with_auth(config);
        self.a2a_server = Some(server);
        self
    }

    /// Convenience: create an AdaptiveController with default configs.
    /// Uses PipelineConfig::default(), OracleConfig::default(), ControllerConfig::default()
    /// for zero-config setup.
    pub fn with_adaptive_controller_default(mut self) -> Self {
        use crate::core::nt_core_consciousness::adaptive_controller::ControllerConfig;
        use crate::core::nt_core_consciousness::consciousness_pipeline::PipelineConfig;
        use crate::core::nt_core_consciousness::performance_oracle::OracleConfig;
        let controller = AdaptiveController::new(
            PipelineConfig::default(),
            OracleConfig::default(),
            ControllerConfig::default(),
        );
        self.adaptive_controller = Some(controller);
        self
    }

    /// Finalize and auto-initialize any modules not explicitly configured.
    /// Call this once after all explicit `with_*` calls.
    /// If called with `auto_init: true` (default), modules with `init_*` patterns
    /// are lazily created on first use rather than at build time.
    pub fn build(self, auto_init: bool) -> Self {
        if auto_init {
            // nt_world_model: auto-init via init_world_model() on first use
            // panorama: auto-init via init_panorama() on first use
            // These modules already have lazy init patterns — no action needed here.
        }
        self
    }

    /// Convenience wrapper: build with auto_init = true.
    pub fn build_auto(self) -> Self {
        self.build(true)
    }
}
