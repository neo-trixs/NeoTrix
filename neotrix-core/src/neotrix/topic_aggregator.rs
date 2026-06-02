use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported topic sources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TopicSource {
    Weibo,
    Zhihu,
    Bilibili,
    V2ex,
    HackerNews,
    GitHubTrending,
    Custom,
}

impl TopicSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            TopicSource::Weibo => "weibo",
            TopicSource::Zhihu => "zhihu",
            TopicSource::Bilibili => "bilibili",
            TopicSource::V2ex => "v2ex",
            TopicSource::HackerNews => "hacker_news",
            TopicSource::GitHubTrending => "github_trending",
            TopicSource::Custom => "custom",
        }
    }

    pub fn all() -> Vec<TopicSource> {
        vec![
            TopicSource::Weibo,
            TopicSource::Zhihu,
            TopicSource::Bilibili,
            TopicSource::V2ex,
            TopicSource::HackerNews,
            TopicSource::GitHubTrending,
        ]
    }
}

/// A single hot topic entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicEntry {
    pub id: String,
    pub title: String,
    pub source: TopicSource,
    pub url: Option<String>,
    pub heat: f64,
    pub rank: usize,
    pub previous_rank: Option<usize>,
    pub category: Option<String>,
    pub summary: Option<String>,
    pub fetched_at: u64,
}

impl TopicEntry {
    pub fn rank_change(&self) -> Option<isize> {
        self.previous_rank
            .map(|prev| prev as isize - self.rank as isize)
    }
}

/// Trend analysis result for a topic over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicTrend {
    pub topic_title: String,
    pub source: TopicSource,
    pub heat_history: Vec<f64>,
    pub rank_history: Vec<(u64, usize)>,
    pub heat_velocity: f64,
    pub volatility: f64,
    pub peak_heat: f64,
    pub duration_secs: u64,
}

impl TopicTrend {
    pub fn is_rising(&self) -> bool {
        self.heat_velocity > 0.0
    }

    pub fn is_stable(&self) -> bool {
        self.volatility < 0.1
    }

    pub fn momentum_score(&self) -> f64 {
        self.heat_velocity * (1.0 - self.volatility.min(0.5))
    }
}

/// Aggregator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatorConfig {
    pub sources: Vec<TopicSource>,
    pub fetch_interval_secs: u64,
    pub max_entries_per_source: usize,
    pub heat_decay_factor: f64,
    pub enable_trend_analysis: bool,
    pub trend_window_size: usize,
}

impl Default for AggregatorConfig {
    fn default() -> Self {
        Self {
            sources: TopicSource::all(),
            fetch_interval_secs: 300,
            max_entries_per_source: 50,
            heat_decay_factor: 0.95,
            enable_trend_analysis: true,
            trend_window_size: 12,
        }
    }
}

/// Topic aggregator engine
pub struct TopicAggregator {
    pub config: AggregatorConfig,
    pub entries: Vec<TopicEntry>,
    pub trends: HashMap<String, TopicTrend>,
    pub source_cache: HashMap<TopicSource, Vec<TopicEntry>>,
    pub last_fetch: HashMap<TopicSource, u64>,
}

impl TopicAggregator {
    pub fn new(config: AggregatorConfig) -> Self {
        Self {
            config,
            entries: Vec::new(),
            trends: HashMap::new(),
            source_cache: HashMap::new(),
            last_fetch: HashMap::new(),
        }
    }

    pub fn ingest(&mut self, source: TopicSource, entries: Vec<TopicEntry>) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.source_cache.insert(source, entries.clone());
        self.last_fetch.insert(source, now);
        for entry in entries {
            if let Some(pos) = self.entries.iter().position(|e| e.id == entry.id) {
                let prev_rank = self.entries[pos].rank;
                self.entries[pos] = TopicEntry {
                    previous_rank: Some(prev_rank),
                    ..entry
                };
            } else {
                self.entries.push(entry);
            }
        }
        self.entries.sort_by(|a, b| {
            b.heat
                .partial_cmp(&a.heat)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if self.entries.len() > self.config.max_entries_per_source * self.config.sources.len() {
            self.entries
                .truncate(self.config.max_entries_per_source * self.config.sources.len());
        }
    }

    pub fn analyze_trends(&mut self) {
        if !self.config.enable_trend_analysis {
            return;
        }
        for entry in &self.entries {
            let key = format!("{}:{}", entry.source.as_str(), entry.title);
            let trend = self.trends.entry(key).or_insert(TopicTrend {
                topic_title: entry.title.clone(),
                source: entry.source,
                heat_history: Vec::new(),
                rank_history: Vec::new(),
                heat_velocity: 0.0,
                volatility: 0.0,
                peak_heat: entry.heat,
                duration_secs: 0,
            });
            trend.heat_history.push(entry.heat);
            trend.rank_history.push((entry.fetched_at, entry.rank));
            if entry.heat > trend.peak_heat {
                trend.peak_heat = entry.heat;
            }
            if trend.heat_history.len() >= 2 {
                let n = trend.heat_history.len();
                let recent = trend.heat_history[n - 1];
                let prev = trend.heat_history[n - 2];
                trend.heat_velocity = recent - prev;
                let mean = trend.heat_history.iter().sum::<f64>() / n as f64;
                let variance = trend
                    .heat_history
                    .iter()
                    .map(|v| (v - mean).powi(2))
                    .sum::<f64>()
                    / n as f64;
                trend.volatility = variance.sqrt();
            }
            if trend.heat_history.len() > self.config.trend_window_size {
                trend.heat_history.remove(0);
            }
        }
    }

    pub fn top_topics(&self, n: usize) -> Vec<&TopicEntry> {
        self.entries.iter().take(n).collect()
    }

    pub fn top_rising(&self, n: usize) -> Vec<(&str, f64)> {
        let mut rising: Vec<(&str, f64)> = self
            .trends
            .iter()
            .filter(|(_, t)| t.is_rising())
            .map(|(k, t)| (k.as_str(), t.momentum_score()))
            .collect();
        rising.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        rising.truncate(n);
        rising
    }

    pub fn source_staleness(&self, source: &TopicSource) -> Option<u64> {
        self.last_fetch.get(source).map(|last| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now - last
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(source: TopicSource, title: &str, heat: f64, rank: usize) -> TopicEntry {
        TopicEntry {
            id: format!("{}:{}", source.as_str(), title),
            title: title.into(),
            source,
            url: None,
            heat,
            rank,
            previous_rank: None,
            category: None,
            summary: None,
            fetched_at: 1000,
        }
    }

    #[test]
    fn test_ingest_and_sort() {
        let mut agg = TopicAggregator::new(AggregatorConfig::default());
        let entries = vec![
            make_entry(TopicSource::Weibo, "News A", 90.0, 1),
            make_entry(TopicSource::Weibo, "News B", 50.0, 2),
        ];
        agg.ingest(TopicSource::Weibo, entries);
        assert_eq!(agg.entries.len(), 2);
        assert_eq!(agg.top_topics(1)[0].title, "News A");
    }

    #[test]
    fn test_trend_analysis() {
        let mut agg = TopicAggregator::new(AggregatorConfig::default());
        agg.ingest(
            TopicSource::HackerNews,
            vec![make_entry(TopicSource::HackerNews, "Trend Topic", 30.0, 10)],
        );
        agg.analyze_trends();
        agg.ingest(
            TopicSource::HackerNews,
            vec![make_entry(TopicSource::HackerNews, "Trend Topic", 80.0, 5)],
        );
        agg.analyze_trends();
        let rising = agg.top_rising(5);
        assert!(!rising.is_empty());
    }

    #[test]
    fn test_source_staleness() {
        let agg = TopicAggregator::new(AggregatorConfig::default());
        assert!(agg.source_staleness(&TopicSource::Weibo).is_none());
    }

    #[test]
    fn test_rank_change() {
        let entry = TopicEntry {
            previous_rank: Some(3),
            ..make_entry(TopicSource::Zhihu, "Test", 50.0, 1)
        };
        assert_eq!(entry.rank_change(), Some(2));
    }

    #[test]
    fn test_topic_source_all() {
        let sources = TopicSource::all();
        assert!(sources.contains(&TopicSource::Weibo));
        assert!(sources.contains(&TopicSource::GitHubTrending));
    }
}
