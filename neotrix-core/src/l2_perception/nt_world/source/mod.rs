//! NT-WORLD Media Source - 媒体源生态
//!
//! 媒体数据源的搜索、播放、缓存、插件、自进化
//! 域: NT-WORLD (虚空探索者)
//! 层: L2 Perception

// ============================================================================
// Core modules
// ============================================================================

pub mod types;
pub mod engine;
pub mod api;
pub mod now_playing;
pub mod resource_store;
pub mod lx_script;

pub mod crypto;
pub mod kb_bridge;
pub mod security_bridge;
pub mod evolution_bridge;
pub mod search_aggregator;
pub mod search_cache;
pub mod search_normalizer;
pub mod quality_adaptive;
pub mod offline_index;
pub mod offline_download;
pub mod plugin_interface;
pub mod plugin_loader;
pub mod plugin_sandbox;
pub mod evolution_metrics;
pub mod evolution_decision;
pub mod search_intent;
pub mod multi_cache;
pub mod search_scorer;
pub mod cache_warmer;
pub mod search_analytics;

// ============================================================================
// Provider modules
// ============================================================================

pub mod audio;
pub mod video;
pub mod text;

// ============================================================================
// Governance (existing skeleton types)
// ============================================================================

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// 审计结果
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditOutcome {
    Pass,
    Fail,
    Warn,
    Skip,
}

/// 单条审计记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: SystemTime,
    pub source_id: String,
    pub action: String,
    pub outcome: AuditOutcome,
    pub detail: String,
}

/// 审计轨迹 — 按 source_id 聚合的审计日志
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditTrail {
    entries: Vec<AuditEntry>,
}

impl AuditTrail {
    pub fn new() -> Self { Self { entries: Vec::new() } }
    pub fn record(&mut self, entry: AuditEntry) { self.entries.push(entry); }
    pub fn entries(&self) -> &[AuditEntry] { &self.entries }
    pub fn recent(&self, n: usize) -> &[AuditEntry] {
        let len = self.entries.len();
        if n >= len { &self.entries } else { &self.entries[len - n..] }
    }
    pub fn filter_by_source(&self, source_id: &str) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.source_id == source_id).collect()
    }
}

/// 删除策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeletionStrategy {
    Soft,
    Hard,
    ArchiveThenDelete,
}

/// 单条保留规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionRule {
    pub data_type: String,
    pub max_age: Duration,
    pub strategy: DeletionStrategy,
}

/// 数据保留策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub rules: Vec<RetentionRule>,
    pub default_strategy: DeletionStrategy,
    pub default_max_age: Duration,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            default_strategy: DeletionStrategy::Soft,
            default_max_age: Duration::from_secs(90 * 24 * 3600),
        }
    }
}

impl RetentionPolicy {
    pub fn new() -> Self { Self::default() }
    pub fn add_rule(&mut self, rule: RetentionRule) { self.rules.push(rule); }
    pub fn match_rule(&self, data_type: &str) -> Option<&RetentionRule> {
        self.rules.iter().find(|r| r.data_type == data_type)
    }
    pub fn strategy_for(&self, data_type: &str) -> DeletionStrategy {
        self.match_rule(data_type).map(|r| r.strategy).unwrap_or(self.default_strategy)
    }
    pub fn max_age_for(&self, data_type: &str) -> Duration {
        self.match_rule(data_type).map(|r| r.max_age).unwrap_or(self.default_max_age)
    }
}

/// 受治理管辖的数据记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: String,
    pub source_id: String,
    pub data_type: String,
    pub created_at: SystemTime,
    pub size_bytes: u64,
    pub metadata: std::collections::HashMap<String, String>,
}

impl DataRecord {
    pub fn age(&self) -> Option<Duration> { self.created_at.elapsed().ok() }
    pub fn is_expired(&self, policy: &RetentionPolicy) -> bool {
        let max_age = policy.max_age_for(&self.data_type);
        self.age().map_or(false, |age| age > max_age)
    }
}

// ============================================================================
// Re-exports
// ============================================================================

pub use types::*;
pub use engine::MediaEngine;
pub use api::{MediaApi, SourceInfo, media_api};
pub use crate::l1_action::nt_media::playback::{
    EnginePlaybackState, PlaybackController, PlaybackEngine, PlaybackHistory, PlaybackQueue,
    PlaybackRetry, PlaybackState, PlayMode, RepeatMode,
};
pub use now_playing::{NowPlaying, PlayerDisplay};
pub use resource_store::ResourceStore;
pub use lx_script::LxScriptSource;
pub use crypto::*;
pub use kb_bridge::KbBridge;
pub use security_bridge::SecurityBridge;
pub use evolution_bridge::EvolutionBridge;
pub use search_aggregator::SearchAggregator;
pub use search_cache::SearchCache;
pub use search_normalizer::SearchNormalizer;
pub use quality_adaptive::*;
pub use offline_index::{OfflineIndex, OfflineEntry};
pub use offline_download::{OfflineDownloader, DownloadResult, OfflineError};
pub use plugin_interface::MediaSourcePlugin;
pub use plugin_loader::PluginLoader;
pub use plugin_sandbox::PluginSandbox;
pub use evolution_metrics::EvolutionMetrics;
pub use evolution_decision::{EvolutionAction, decide_evolution};
pub use search_intent::{SearchIntent, classify_intent};
pub use multi_cache::MultiLevelCache;
pub use search_scorer::{score_result, fuzzy_match, rank_results};
pub use cache_warmer::CacheWarmer;
pub use search_analytics::SearchAnalytics;

/// 构建默认媒体引擎 (注册所有29个源)
pub fn build_default_engine() -> MediaEngine {
    use std::sync::Arc;

    let mut engine = MediaEngine::new();

    engine.add_source(Arc::new(audio::netease::NeteaseSource::new()), 1);
    engine.add_source(Arc::new(audio::kuwo::KuwoSource::new()), 2);
    engine.add_source(Arc::new(audio::kugou::KugouSource::new()), 3);
    engine.add_source(Arc::new(audio::migu::MiguSource::new()), 4);
    engine.add_source(Arc::new(audio::soundcloud::SoundCloudSource::new()), 5);
    engine.add_source(Arc::new(audio::spotify::SpotifySource::new()), 6);
    engine.add_source(Arc::new(audio::piped::PipedSource::new()), 7);
    engine.add_source(Arc::new(audio::jiosaavn::JioSaavnSource::new()), 8);
    engine.add_source(Arc::new(audio::deezer::DeezerSource::new()), 9);
    engine.add_source(Arc::new(audio::bandcamp::BandcampSource::new()), 10);
    engine.add_source(Arc::new(audio::qqmusic::QQMusicSource::new()), 11);

    engine.add_source(Arc::new(video::youtube::YouTubeSource::new()), 12);
    engine.add_source(Arc::new(video::bilibili::BilibiliSource::new()), 13);
    engine.add_source(Arc::new(video::vimeo::VimeoSource::new()), 14);

    engine.add_source(Arc::new(text::document::arxiv::ArxivSource::new()), 19);
    engine.add_source(Arc::new(text::document::semantic_scholar::SemanticScholarSource::new()), 20);
    engine.add_source(Arc::new(text::book::openlibrary::OpenLibrarySource::new()), 21);
    engine.add_source(Arc::new(text::book::annas_archive::AnnasArchiveSource::new()), 22);
    engine.add_source(Arc::new(text::lyrics::lrclib::LrclibSource::new()), 23);
    engine.add_source(Arc::new(text::lyrics::multi::MultiLyricSource::new()), 24);
    engine.add_source(Arc::new(text::lyrics::genius::GeniusSource::new()), 25);
    engine.add_source(Arc::new(text::social::tiktok::TikTokSource::new()), 26);
    engine.add_source(Arc::new(text::social::instagram::InstagramSource::new()), 27);
    engine.add_source(Arc::new(text::social::twitter::TwitterSource::new()), 28);
    engine.add_source(Arc::new(text::social::ytdlp::YtdlpSource::new()), 29);

    engine
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_trail_record_and_filter() {
        let mut trail = AuditTrail::new();
        trail.record(AuditEntry {
            timestamp: SystemTime::now(),
            source_id: "s1".into(),
            action: "fetch".into(),
            outcome: AuditOutcome::Pass,
            detail: "ok".into(),
        });
        assert_eq!(trail.entries().len(), 1);
        assert_eq!(trail.filter_by_source("s1").len(), 1);
    }

    #[test]
    fn test_retention_policy_defaults() {
        let policy = RetentionPolicy::new();
        assert_eq!(policy.default_strategy, DeletionStrategy::Soft);
        assert_eq!(policy.strategy_for("unknown"), DeletionStrategy::Soft);
    }
}
