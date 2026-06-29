//! Data source types extracted from neotrix layer to break circular import.

use std::collections::HashMap;

/// Generic key-value registry — the canonical building block for all 48+ registries.
///
/// Bounded by `max_items` (default 10000) to prevent unbounded growth. When the
/// registry exceeds `max_items` on insert, the oldest 20% of entries are evicted.
///
/// Use this directly for simple `HashMap<K, V>` registries, or embed it in
/// domain-specific registry structs that require extra fields or methods.
#[derive(Debug, Clone)]
pub struct GenericRegistry<K, V> {
    pub items: HashMap<K, V>,
    max_items: usize,
}

impl<K: std::hash::Hash + Eq + Clone, V> GenericRegistry<K, V> {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            max_items: 10000,
        }
    }

    pub fn with_max_items(max_items: usize) -> Self {
        Self {
            items: HashMap::new(),
            max_items,
        }
    }

    pub fn max_items(&self) -> usize {
        self.max_items
    }

    pub fn set_max_items(&mut self, max_items: usize) {
        self.max_items = max_items;
    }

    pub fn register(&mut self, key: K, value: V) -> Option<V> {
        let result = self.items.insert(key, value);
        if self.items.len() > self.max_items {
            self.drain_oldest((self.max_items * 20 / 100).max(1));
        }
        result
    }

    fn drain_oldest(&mut self, count: usize) {
        let keys: Vec<K> = self.items.keys().take(count).cloned().collect();
        for key in &keys {
            self.items.remove(key);
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.items.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.items.get_mut(key)
    }

    pub fn contains(&self, key: &K) -> bool {
        self.items.contains_key(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.items.remove(key)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.items.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.items.values()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.items.iter()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn into_inner(self) -> HashMap<K, V> {
        self.items
    }
}

/// Record of data from an external source
#[derive(Debug, Clone)]
pub struct DataSourceRecord {
    pub title: String,
    pub summary: String,
    pub url: String,
    pub source_type: DataSourceType,
    pub topics: Vec<String>,
    pub score: f64,
    pub timestamp: i64,
}

/// Type of external data source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataSourceType {
    HackerNews,
    ArXiv,
    GitHubTrending,
    Wikipedia,
    OpenLibrary,
    NewsRss,
    SemanticScholar,
    YouTube,
    Unsplash,
    Twitch,
    TrendShift,
    TikTok,
    Spotify,
    Pinterest,
    Netflix,
    Imdb,
    Dribbble,
    AppleMusic,
}

impl DataSourceType {
    pub fn name(&self) -> &'static str {
        match self {
            DataSourceType::HackerNews => "hackernews",
            DataSourceType::ArXiv => "arxiv",
            DataSourceType::GitHubTrending => "github_trending",
            DataSourceType::Wikipedia => "wikipedia",
            DataSourceType::OpenLibrary => "openlibrary",
            DataSourceType::NewsRss => "news_rss",
            DataSourceType::SemanticScholar => "semantic_scholar",
            DataSourceType::YouTube => "youtube",
            DataSourceType::Unsplash => "unsplash",
            DataSourceType::Twitch => "twitch",
            DataSourceType::TrendShift => "trendshift",
            DataSourceType::TikTok => "tiktok",
            DataSourceType::Spotify => "spotify",
            DataSourceType::Pinterest => "pinterest",
            DataSourceType::Netflix => "netflix",
            DataSourceType::Imdb => "imdb",
            DataSourceType::Dribbble => "dribbble",
            DataSourceType::AppleMusic => "apple_music",
        }
    }
}

/// External data connector (type only — implementation in neotrix layer)
pub struct ExternalDataConnector;

// ── ToolResult (canonical definition — re-exported via nt_core_agent::tool_result) ──

/// Generic tool execution result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: String,
    pub duration_ms: u64,
}

impl ToolResult {
    pub fn ok(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
            error: String::new(),
            duration_ms: 0,
        }
    }
    pub fn err(error: impl Into<String>) -> Self {
        Self {
            success: false,
            output: String::new(),
            error: error.into(),
            duration_ms: 0,
        }
    }
    pub fn unwrap(self) -> String {
        assert!(
            self.success,
            "ToolResult::unwrap() called on failure: {}",
            self.error
        );
        self.output
    }
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_registry() {
        let mut reg: GenericRegistry<String, i32> = GenericRegistry::with_max_items(10);
        assert!(reg.is_empty());
        reg.register("a".into(), 42);
        assert_eq!(reg.len(), 1);
        assert_eq!(*reg.get(&"a".into()).unwrap(), 42);
        assert!(reg.contains(&"a".into()));
        assert!(!reg.contains(&"b".into()));
    }

    #[test]
    fn test_tool_result() {
        let ok = ToolResult::ok("done");
        assert!(ok.success);
        assert_eq!(ok.output, "done");
        let err = ToolResult::err("failed");
        assert!(!err.success);
        assert_eq!(err.error, "failed");
        let with_dur = ok.with_duration(100);
        assert_eq!(with_dur.duration_ms, 100);
    }

    #[test]
    fn test_data_source_type_names() {
        assert_eq!(DataSourceType::HackerNews.name(), "hackernews");
        assert_eq!(DataSourceType::ArXiv.name(), "arxiv");
        assert_eq!(DataSourceType::GitHubTrending.name(), "github_trending");
    }
}
