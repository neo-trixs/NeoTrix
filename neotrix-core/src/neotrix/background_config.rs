//! 背景循环配置和遥测计数器
//!
//! 从 background_loop.rs 拆分 (Session 30: 降低主文件 <= 800 行)

use std::sync::atomic::{AtomicU64, Ordering};

/// 背景循环配置
pub struct BackgroundConfig {
    pub save_interval_secs: u64,
    pub consolidate_interval_secs: u64,
    pub evolve_interval_secs: u64,
    pub cleanup_interval_secs: u64,
    pub mine_interval_secs: u64,
    pub goal_interval_secs: u64,
    pub metacog_interval_secs: u64,
    pub thinking_interval_secs: u64,
    pub geo_update_interval_hours: u64,
    pub telemetry_interval_secs: u64,
    pub enabled: bool,
    pub proxy_enabled: bool,
    pub system_proxy_enabled: bool,
    pub geo_auto_update: bool,
    pub agent_protocol_enabled: bool,
    pub nt_world_crawl_interval_secs: u64,
    pub world_prediction_interval_secs: u64,
    pub prediction_interval_secs: u64,
    pub evolution_interval_secs: u64,
    pub panorama_interval_secs: u64,
    pub exploration_interval_secs: u64,
    pub enable_exploration: bool,
    pub curiosity_interval_secs: u64,
    pub knowledge_chain_interval_secs: u64,
    pub knowledge_aging_interval_secs: u64,
    pub crystallization_interval_secs: u64,
    pub enable_auto_crystallize: bool,
    pub tor_crawler_interval_secs: u64,
    pub tor_crawler_search_queries: Vec<String>,
    pub proxy_heartbeat_interval_secs: u64,
    pub nt_world_sense_interval_secs: u64,
    pub nt_act_voice_interval_secs: u64,
    pub always_on_interval_secs: u64,
    pub plugin_interval_secs: u64,
}

impl Default for BackgroundConfig {
    fn default() -> Self {
        Self {
            save_interval_secs: 30,
            consolidate_interval_secs: 60,
            evolve_interval_secs: 120,
            cleanup_interval_secs: 300,
            mine_interval_secs: 600,
            goal_interval_secs: 180,
            metacog_interval_secs: 600,
            thinking_interval_secs: 120,
            geo_update_interval_hours: 6,
            telemetry_interval_secs: 300,
            enabled: true,
            proxy_enabled: true,
            system_proxy_enabled: true,
            geo_auto_update: true,
            agent_protocol_enabled: false,
            nt_world_crawl_interval_secs: 43200,
            world_prediction_interval_secs: 60,
            prediction_interval_secs: 60,
            evolution_interval_secs: 300,
            panorama_interval_secs: 600,
            exploration_interval_secs: 1800,
            enable_exploration: true,
            curiosity_interval_secs: 300,
            knowledge_chain_interval_secs: 3600,
            knowledge_aging_interval_secs: 86400,
            crystallization_interval_secs: 600,
            enable_auto_crystallize: true,
            tor_crawler_interval_secs: 3600,
            tor_crawler_search_queries: vec!["rust".into(), "coding".into(), "AI".into()],
            proxy_heartbeat_interval_secs: 30,
            nt_world_sense_interval_secs: 60,
            nt_act_voice_interval_secs: 5,
            always_on_interval_secs: 60,
            plugin_interval_secs: 30,
        }
    }
}

/// 轻量级遥测计数器
pub struct TelemetryCollector {
    pub seal_loop_count: AtomicU64,
    pub knowledge_mine_count: AtomicU64,
    pub absorb_count: AtomicU64,
    pub error_count: AtomicU64,
    pub started_at: tokio::time::Instant,
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryCollector {
    pub fn new() -> Self {
        Self {
            seal_loop_count: AtomicU64::new(0),
            knowledge_mine_count: AtomicU64::new(0),
            absorb_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            started_at: tokio::time::Instant::now(),
        }
    }

    pub fn snapshot(&self) -> TelemetrySnapshot {
        TelemetrySnapshot {
            uptime_secs: self.started_at.elapsed().as_secs(),
            seal_loops: self.seal_loop_count.load(Ordering::Relaxed),
            knowledge_mines: self.knowledge_mine_count.load(Ordering::Relaxed),
            absorbs: self.absorb_count.load(Ordering::Relaxed),
            errors: self.error_count.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TelemetrySnapshot {
    pub uptime_secs: u64,
    pub seal_loops: u64,
    pub knowledge_mines: u64,
    pub absorbs: u64,
    pub errors: u64,
}
