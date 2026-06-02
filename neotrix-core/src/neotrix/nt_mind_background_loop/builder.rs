use super::*;
use super::super::nt_mind::self_iterating::brain_impl::ReasoningBrain;
use super::super::nt_mind::memory::ReasoningBank;

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
    pub fn with_tor_crawler(mut self, tor_crawler: std::sync::Arc<super::super::nt_shield_stealth_net::tor_crawler::TorCrawler>) -> Self {
        self.tor_crawler = Some(tor_crawler);
        self
    }

    #[cfg(feature = "stealth-net")]
    pub fn with_proxy_heartbeat(mut self, interval_secs: u64) -> Self {
        use super::super::nt_shield_stealth_net::{ProxyHeartbeatEngine, FingerprintManager};
        use super::super::nt_shield_stealth_net::proxy_pool::ProxyPool;
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
        let _ = rt.block_on(async {
            let logging: Box<dyn crate::neotrix::nt_io_plugin::Plugin> =
                Box::new(crate::neotrix::nt_io_plugin::builtin::logging::LoggingPlugin);
            if let Err(e) = reg.register(logging).await {
                log::error!("[plugin] register logging: {}", e);
            }
        });
        self
    }

    /// Spawn a HotReloadWatcher for config/rules/subscriptions hot-reload.
    /// Spawns the watcher and stores the join handle in `self.handles`.
    #[cfg(feature = "stealth-net")]
    pub fn with_hot_reload(mut self, neotrix_dir: std::path::PathBuf) -> Self {
        match crate::neotrix::hotreload::default_watcher(neotrix_dir, None, None) {
            Ok(mut watcher) => match watcher.spawn() {
                Ok(handle) => self.handles.push(handle),
                Err(e) => log::error!("[bg] hotreload spawn failed: {}", e),
            },
            Err(e) => log::error!("[bg] hotreload init failed: {}", e),
        }
        self
    }

    #[cfg(feature = "stealth-net")]
    pub fn with_proxy_client(mut self) -> Self {
        self.proxy_client = Some(super::super::nt_shield_stealth_net::proxy_control::ProxyClient::new());
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
}
