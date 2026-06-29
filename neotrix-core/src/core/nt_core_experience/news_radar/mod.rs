#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod alert;
pub mod correlator;
pub mod predictor;
pub mod scorer;
pub mod trend;
pub mod types;
pub use self::alert::OpinionFlowReport;
pub use self::correlator::{CrossSourceCorrelator, CrossSourceResult};
pub use self::predictor::{PredictedTrend, SignalPredictor};
pub use self::scorer::{AIScorer, StoryDeduplicator};
pub use self::trend::TrendTracker;
pub use self::types::{
    AlertLevel, BriefingStats, EarlyWarningSignal, HotnessScore, NewsBriefing, NewsItem,
    NewsSource, OpinionAlert, PollResult, SentimentDivergence, SentimentLabel, TopicTrend,
};

// ── NewsRadar ──

#[derive(Debug, Clone)]
pub struct NewsRadar {
    sources: Vec<NewsSource>,
    scorer: AIScorer,
    deduplicator: StoryDeduplicator,
    max_items: usize,
    trend_tracker: TrendTracker,
    signal_predictor: SignalPredictor,
    cross_source_correlator: CrossSourceCorrelator,
    last_briefing: Option<NewsBriefing>,
}

impl NewsRadar {
    pub fn new(sources: Vec<NewsSource>) -> Self {
        Self {
            sources,
            scorer: AIScorer::new(),
            deduplicator: StoryDeduplicator::default(),
            max_items: 20,
            trend_tracker: TrendTracker::new(3600, 24, 0.5),
            signal_predictor: SignalPredictor::new(),
            cross_source_correlator: CrossSourceCorrelator::new(0.75),
            last_briefing: None,
        }
    }

    pub fn with_max_items(mut self, max: usize) -> Self {
        self.max_items = max;
        self
    }

    fn fetch_from_source(&self, source: &NewsSource) -> Vec<NewsItem> {
        let source_name = source.name().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut items = Vec::new();

        match source {
            NewsSource::HackerNews => {
                for i in 0..3 {
                    items.push(NewsItem {
                        id: hash_id(&format!("hn-{}-{}", source_name, i)),
                        title: format!(
                            "Show HN: {}",
                            [
                                "a new open source AI framework",
                                "Rust-based neural network library",
                                "a terminal-based productivity tool"
                            ][i % 3]
                        ),
                        url: format!("https://news.ycombinator.com/item?id={}", i + 1000),
                        source_name: source_name.clone(),
                        summary: Some(
                            "A new open source project gaining traction on HN.".to_string(),
                        ),
                        score: 0.0,
                        ai_tags: vec!["open source".to_string(), "rust".to_string()],
                        published_at: now - i as u64 * 3600,
                        category: Some("technology".to_string()),
                        comments_summary: Some(format!("{} comments", (i + 1) * 15)),
                        enrichment_context: None,
                    });
                }
            }
            NewsSource::Reddit { subreddit } => {
                for i in 0..2 {
                    items.push(NewsItem {
                        id: hash_id(&format!("reddit-{}-{}", subreddit, i)),
                        title: format!(
                            "r/{}: {}",
                            subreddit,
                            [
                                "What are your thoughts on the latest LLM release?",
                                "PSA: Critical vulnerability discovered in popular library"
                            ][i % 2]
                        ),
                        url: format!("https://reddit.com/r/{}/comments/{}", subreddit, i + 500),
                        source_name: source_name.clone(),
                        summary: None,
                        score: 0.0,
                        ai_tags: vec!["discussion".to_string()],
                        published_at: now - i as u64 * 7200,
                        category: Some("discussion".to_string()),
                        comments_summary: None,
                        enrichment_context: None,
                    });
                }
            }
            NewsSource::RSS { url, name } => {
                items.push(NewsItem {
                    id: hash_id(&format!("rss-{}-{}", name, 0)),
                    title: format!("{}: Latest update from {}", name, url),
                    url: url.clone(),
                    source_name: source_name.clone(),
                    summary: Some("A curated RSS feed entry.".to_string()),
                    score: 0.0,
                    ai_tags: vec!["rss".to_string()],
                    published_at: now,
                    category: Some("news".to_string()),
                    comments_summary: None,
                    enrichment_context: None,
                });
            }
            NewsSource::GitHub { topic } => {
                for i in 0..2 {
                    items.push(NewsItem {
                        id: hash_id(&format!("gh-{}-{}", topic, i)),
                        title: format!(
                            "New GitHub repository: {}/{}",
                            topic,
                            ["awesome-tools", "machine-learning-starter"][i % 2]
                        ),
                        url: format!("https://github.com/{}/repo{}", topic, i),
                        source_name: source_name.clone(),
                        summary: Some(format!("A trending repository in the {} space.", topic)),
                        score: 0.0,
                        ai_tags: vec!["github".to_string(), topic.clone()],
                        published_at: now - i as u64 * 1800,
                        category: Some("development".to_string()),
                        comments_summary: None,
                        enrichment_context: Some(format!("{} stars today", (i + 1) * 50)),
                    });
                }
            }
            NewsSource::Telegram { channel } => {
                items.push(NewsItem {
                    id: hash_id(&format!("tg-{}-{}", channel, 0)),
                    title: format!("[{}] Flash news update", channel),
                    url: format!("https://t.me/{}/{}", channel, 42),
                    source_name: source_name.clone(),
                    summary: Some("A breaking news alert from Telegram.".to_string()),
                    score: 0.0,
                    ai_tags: vec!["telegram".to_string(), "flash".to_string()],
                    published_at: now,
                    category: Some("breaking".to_string()),
                    comments_summary: None,
                    enrichment_context: None,
                });
            }
            NewsSource::Custom { name, .. } => {
                items.push(NewsItem {
                    id: hash_id(&format!("custom-{}-{}", name, 0)),
                    title: format!("[{}] Custom intelligence report", name),
                    url: format!("https://custom.news/{}", name),
                    source_name: source_name.clone(),
                    summary: Some("A custom-curated news item.".to_string()),
                    score: 0.0,
                    ai_tags: vec!["custom".to_string()],
                    published_at: now,
                    category: Some("general".to_string()),
                    comments_summary: None,
                    enrichment_context: None,
                });
            }
        }

        items
    }

    fn fetch_all(&self) -> Vec<NewsItem> {
        let mut items = Vec::new();
        for source in &self.sources {
            items.extend(self.fetch_from_source(source));
        }
        items
    }

    pub fn poll_all(&mut self) -> usize {
        let briefing = self.run_cycle();
        briefing.items.len()
    }

    pub fn run_cycle(&mut self) -> NewsBriefing {
        let total_sources = self.sources.len();
        let raw_items = self.fetch_all();

        let deduped = self.deduplicator.deduplicate(raw_items);

        for item in &deduped {
            self.trend_tracker.record_item(item);
        }

        let mut scored: Vec<NewsItem> = deduped
            .into_iter()
            .map(|mut item| {
                item.score = self.scorer.score_item(&item);
                item
            })
            .collect();

        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let threshold = 5.0;
        let filtered: Vec<NewsItem> = scored
            .into_iter()
            .filter(|i| i.score >= threshold)
            .collect();

        let correlated = self.cross_source_correlator.correlate(&filtered);
        if correlated.is_empty() {
            tracing::warn!("[news_radar] cross-source correlation returned no results");
        }

        let limited: Vec<NewsItem> = filtered.into_iter().take(self.max_items).collect();

        let briefing = NewsBriefing::new(limited, total_sources);
        self.last_briefing = Some(briefing.clone());
        briefing
    }

    pub fn filter_by_category<'a>(
        &self,
        brief: &'a NewsBriefing,
        category: &str,
    ) -> Vec<&'a NewsItem> {
        let cat_lower = category.to_lowercase();
        brief
            .items
            .iter()
            .filter(|item| {
                item.category
                    .as_deref()
                    .map(|c| c.to_lowercase() == cat_lower)
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn to_briefing_text(&self, brief: &NewsBriefing) -> String {
        let mut output = String::new();
        output.push_str(&format!(
            "# 📡 News Briefing — {}\n\n",
            format_timestamp(brief.generated_at)
        ));
        output.push_str(&format!(
            "**Sources checked:** {} | **Articles:** {} | **Avg score:** {:.1}\n\n",
            brief.total_sources_checked,
            brief.summary_stats.total_articles,
            brief.summary_stats.avg_score,
        ));
        output.push_str(&format!(
            "**Top category:** {} | **Source breakdown:** ",
            brief.summary_stats.top_category,
        ));
        let mut srcs: Vec<(&String, &usize)> =
            brief.summary_stats.source_breakdown.iter().collect();
        srcs.sort_by(|a, b| b.1.cmp(a.1));
        for (i, (src, count)) in srcs.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            output.push_str(&format!("{} ({})", src, count));
        }
        output.push_str("\n\n---\n\n");

        let mut sorted = brief.items.clone();
        sorted.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (i, item) in sorted.iter().enumerate() {
            output.push_str(&format!("### {}. {}\n", i + 1, item.title));
            output.push_str(&format!(
                "**Score:** {:.1}/10 | **Source:** {} | **Category:** {}\n",
                item.score,
                item.source_name,
                item.category.as_deref().unwrap_or("uncategorized"),
            ));
            if let Some(ref summary) = item.summary {
                output.push_str(&format!("> {}\n", summary));
            }
            output.push_str(&format!("🔗 {}\n", item.url));
            if !item.ai_tags.is_empty() {
                output.push_str(&format!("🏷️ `{}`\n", item.ai_tags.join("`, `")));
            }
            if let Some(ref cc) = item.comments_summary {
                output.push_str(&format!("💬 {}\n", cc));
            }
            if let Some(ref ec) = item.enrichment_context {
                output.push_str(&format!("📊 {}\n", ec));
            }
            output.push('\n');
        }

        output
    }

    pub fn update_scores(&mut self, items: &mut [NewsItem]) {
        for item in items.iter_mut() {
            item.score = self.scorer.score_item(item);
        }
    }

    pub fn merge_with_previous(
        &self,
        current: NewsBriefing,
        previous: &NewsBriefing,
    ) -> NewsBriefing {
        let merged_total = current
            .total_sources_checked
            .max(previous.total_sources_checked);
        let mut deduper = self.deduplicator.clone();
        let merged_items = deduper.merge_dedup(current.items, &previous.items);
        let limited: Vec<NewsItem> = merged_items.into_iter().take(self.max_items).collect();
        NewsBriefing::new(limited, merged_total)
    }

    // ── Opinion Flow / Prediction API ──

    /// Generate consolidated opinion flow report with alerts.
    /// Inspired by BettaFish micro-opinion multi-agent analysis + TrendRadar hotness
    /// + deep-early-warnings critical transition detection.
    pub fn extended_opinion_flow(&mut self) -> OpinionFlowReport {
        if self.last_briefing.is_none() {
            self.run_cycle();
        }
        let trends: Vec<TopicTrend> = self.trend_tracker.all_topics().values().cloned().collect();
        let cross_source = self.cross_source_correlator.correlate(
            &self
                .last_briefing
                .as_ref()
                .map(|b| b.items.clone())
                .unwrap_or_default(),
        );
        self.signal_predictor
            .predict(self.trend_tracker.all_topics(), &cross_source);

        let mut sentiment_summary: HashMap<String, usize> = HashMap::new();
        for trend in &trends {
            for s in &trend.sentiment_trend {
                *sentiment_summary.entry(s.name().to_string()).or_insert(0) += 1;
            }
        }

        let alerts = self.alerts_from_trends(&trends, &cross_source);

        OpinionFlowReport {
            trends,
            cross_source,
            predictions: self.signal_predictor.predictions.clone(),
            signal_confidence: self.signal_predictor.signal_confidence(),
            briefing: self
                .last_briefing
                .clone()
                .unwrap_or_else(|| NewsBriefing::new(Vec::new(), 0)),
            sentiment_summary,
            alerts,
        }
    }

    /// Generate BettaFish-style multi-factor alerts from trend data.
    fn alerts_from_trends(
        &self,
        trends: &[TopicTrend],
        cross_source: &[CrossSourceResult],
    ) -> Vec<OpinionAlert> {
        self::alert::alerts_from_trends(trends, cross_source, &self.signal_predictor.predictions)
    }

    pub fn trend_tracker(&self) -> &TrendTracker {
        &self.trend_tracker
    }

    pub fn signal_predictor(&self) -> &SignalPredictor {
        &self.signal_predictor
    }

    pub fn last_briefing(&self) -> Option<&NewsBriefing> {
        self.last_briefing.as_ref()
    }
}

fn hash_id(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    h
}

fn format_timestamp(ts: u64) -> String {
    let secs = ts % 60;
    let mins = (ts / 60) % 60;
    let hours = (ts / 3600) % 24;
    let days = ts / 86400;
    format!("Day {} {:02}:{:02}:{:02} UTC", days, hours, mins, secs)
}

// ── Default NewsRadar ──

impl Default for NewsRadar {
    fn default() -> Self {
        Self::new(vec![
            NewsSource::HackerNews,
            NewsSource::Reddit {
                subreddit: "machinelearning".to_string(),
            },
            NewsSource::RSS {
                url: "https://feeds.arxiv.org/cs.AI".to_string(),
                name: "arXiv AI".to_string(),
            },
            NewsSource::GitHub {
                topic: "machine-learning".to_string(),
            },
        ])
    }
}

// ── Public API re-exports (aliases) ──

pub use self::scorer::AIScorer as AIScorerPublic;
pub use self::scorer::StoryDeduplicator as StoryDeduplicatorPublic;
pub use self::types::BriefingStats as BriefingStatsPublic;
pub use self::types::NewsBriefing as NewsBriefingPublic;
pub use self::types::NewsItem as NewsItemPublic;
pub use self::types::NewsSource as NewsSourcePublic;
pub use self::NewsRadar as NewsRadarPublic;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_item(id: u64, title: &str, url: &str, source: &str) -> NewsItem {
        NewsItem::new(id, title, url, source)
    }

    fn sample_radar() -> NewsRadar {
        NewsRadar::new(vec![
            NewsSource::HackerNews,
            NewsSource::Reddit {
                subreddit: "rust".to_string(),
            },
            NewsSource::GitHub {
                topic: "ai".to_string(),
            },
        ])
    }

    // 1
    #[test]
    fn test_source_creation() {
        let hn = NewsSource::HackerNews;
        assert_eq!(hn.name(), "HackerNews");
        assert_eq!(hn.source_type(), "hackernews");

        let reddit = NewsSource::Reddit {
            subreddit: "rust".to_string(),
        };
        assert_eq!(reddit.name(), "rust");
        assert_eq!(reddit.source_type(), "reddit");

        let rss = NewsSource::RSS {
            url: "https://example.com/rss".to_string(),
            name: "MyFeed".to_string(),
        };
        assert_eq!(rss.name(), "MyFeed");
        assert_eq!(rss.source_type(), "rss");

        let gh = NewsSource::GitHub {
            topic: "quantum".to_string(),
        };
        assert_eq!(gh.name(), "quantum");
        assert_eq!(gh.source_type(), "github");

        let tg = NewsSource::Telegram {
            channel: "news_channel".to_string(),
        };
        assert_eq!(tg.name(), "news_channel");
        assert_eq!(tg.source_type(), "telegram");

        let custom = NewsSource::Custom {
            name: "AlphaIntel".to_string(),
            fetch_fn_desc: "scrape custom API".to_string(),
        };
        assert_eq!(custom.name(), "AlphaIntel");
        assert_eq!(custom.source_type(), "custom");
    }

    // 2
    #[test]
    fn test_item_scoring() {
        let scorer = AIScorer::new();

        let item_high = sample_item(
            1,
            "Breakthrough in deep learning and AI research",
            "https://example.com/ai",
            "hackernews",
        );
        let score_high = scorer.score_item(&item_high);
        assert!(score_high >= 6.0, "Expected high score, got {}", score_high);

        let item_low = sample_item(
            2,
            "Local weather forecast for tomorrow",
            "https://example.com/weather",
            "rss",
        );
        let score_low = scorer.score_item(&item_low);
        assert!(score_low < 7.0, "Expected lower score, got {}", score_low);

        assert!(
            score_high > score_low,
            "AI article should score higher than weather"
        );
    }

    // 3
    #[test]
    fn test_scorer_with_source_authority() {
        let scorer = AIScorer::new();

        let title = "Rust programming language release";
        let hn_score = scorer.score(title, "hackernews", &[]);
        let tg_score = scorer.score(title, "telegram", &[]);
        assert!(
            hn_score >= tg_score,
            "HN authority should be higher than Telegram"
        );
    }

    // 4
    #[test]
    fn test_deduplication_by_url() {
        let mut deduper = StoryDeduplicator::new(0.85);
        let a = sample_item(1, "AI breakthrough", "https://example.com/1", "hn");
        let b = sample_item(
            2,
            "AI breakthrough duplicate",
            "https://example.com/1",
            "hn",
        );

        assert!(!deduper.is_duplicate(&a));
        assert!(deduper.is_duplicate(&b));
    }

    // 5
    #[test]
    fn test_deduplication_by_title_similarity() {
        let mut deduper = StoryDeduplicator::new(0.85);
        let items = vec![
            sample_item(
                1,
                "New AI model breaks records on benchmark tests",
                "https://example.com/a",
                "hn",
            ),
            sample_item(
                2,
                "New AI model breaks records on benchmark tests",
                "https://example.com/b",
                "hn",
            ),
            sample_item(
                3,
                "Completely unrelated weather update",
                "https://example.com/c",
                "rss",
            ),
        ];
        let deduped = deduper.deduplicate(items);
        assert_eq!(
            deduped.len(),
            2,
            "Should deduplicate the two identical titles"
        );
    }

    // 6
    #[test]
    fn test_briefing_generation() {
        let mut radar = sample_radar();
        let briefing = radar.run_cycle();
        assert!(briefing.total_sources_checked >= 3);
        assert!(!briefing.is_empty(), "Briefing should contain items");
        for item in &briefing.items {
            assert!(
                item.score >= 0.0 && item.score <= 10.0,
                "Score out of range: {}",
                item.score
            );
        }
    }

    // 7
    #[test]
    fn test_category_filtering() {
        let items = vec![
            sample_item(1, "AI news A", "https://x.com/a", "hn").with_category("technology"),
            sample_item(2, "Sports update B", "https://x.com/b", "rss").with_category("sports"),
            sample_item(3, "AI news C", "https://x.com/c", "hn").with_category("technology"),
        ];
        let brief = NewsBriefing::new(items, 2);
        let radar = sample_radar();

        let tech_items = radar.filter_by_category(&brief, "technology");
        assert_eq!(tech_items.len(), 2);

        let sports_items = radar.filter_by_category(&brief, "sports");
        assert_eq!(sports_items.len(), 1);

        let missing = radar.filter_by_category(&brief, "science");
        assert!(missing.is_empty());
    }

    // 8
    #[test]
    fn test_merge_with_previous() {
        let radar = sample_radar();
        let items_a = vec![
            sample_item(1, "Story one", "https://x.com/1", "hn"),
            sample_item(2, "Story two", "https://x.com/2", "hn"),
        ];
        let items_b = vec![
            sample_item(2, "Story two (duplicate)", "https://x.com/2b", "hn"),
            sample_item(3, "Story three", "https://x.com/3", "hn"),
        ];
        let prev = NewsBriefing::new(items_a, 2);
        let current = NewsBriefing::new(items_b, 2);

        let merged = radar.merge_with_previous(current, &prev);
        assert_eq!(merged.items.len(), 2, "Should merge non-duplicate items");
        let titles: Vec<&str> = merged.items.iter().map(|i| i.title.as_str()).collect();
        assert!(titles.contains(&"Story one"), "Should keep previous item");
        assert!(
            titles.contains(&"Story three"),
            "Should include new unique item"
        );
    }

    // 9
    #[test]
    fn test_score_update() {
        let mut radar = sample_radar();
        let mut items = vec![
            sample_item(
                1,
                "Major breakthrough in quantum machine learning",
                "https://x.com/q",
                "hackernews",
            ),
            sample_item(
                2,
                "What I ate for breakfast today",
                "https://x.com/b",
                "github",
            ),
        ];
        radar.update_scores(&mut items);
        assert!(
            items[0].score > items[1].score,
            "Quantum ML should score higher than breakfast"
        );
        for item in &items {
            assert!(item.score >= 0.0 && item.score <= 10.0);
        }
    }

    // 10
    #[test]
    fn test_markdown_briefing_output() {
        let mut radar = sample_radar();
        let briefing = radar.run_cycle();
        let text = radar.to_briefing_text(&briefing);
        assert!(text.contains("News Briefing"), "Should contain heading");
        assert!(text.contains("**Score:"), "Should contain score labels");
        assert!(text.contains("🔗"), "Should contain link icon");
        assert!(text.contains("Source:"), "Should contain source labels");
        assert!(text.contains("Articles:"), "Should contain article count");
    }

    // 11
    #[test]
    fn test_empty_input() {
        let mut radar = NewsRadar::new(vec![]);
        let briefing = radar.run_cycle();
        assert!(briefing.is_empty());
        assert_eq!(briefing.total_sources_checked, 0);

        let text = radar.to_briefing_text(&briefing);
        assert!(text.contains("Articles: **0**") || text.contains("Articles:"));

        let stats = BriefingStats::from_items(&[]);
        assert_eq!(stats.total_articles, 0);
        assert!((stats.avg_score - 0.0).abs() < 1e-6);
    }

    // 12
    #[test]
    fn test_briefing_stats() {
        let items = vec![
            sample_item(1, "AI story", "https://x.com/1", "hn")
                .with_category("tech")
                .with_score(8.0),
            sample_item(2, "Dev story", "https://x.com/2", "github")
                .with_category("tech")
                .with_score(6.0),
            sample_item(3, "Science story", "https://x.com/3", "rss")
                .with_category("science")
                .with_score(7.0),
        ];
        let stats = BriefingStats::from_items(&items);
        assert_eq!(stats.total_articles, 3);
        assert!((stats.avg_score - 7.0).abs() < 1e-6);
        assert_eq!(stats.top_category, "tech");
        assert_eq!(*stats.source_breakdown.get("hn").unwrap(), 1);
    }

    // 13
    #[test]
    fn test_max_items_limit() {
        let sources = (0..10)
            .map(|i| NewsSource::Custom {
                name: format!("src{}", i),
                fetch_fn_desc: "test".to_string(),
            })
            .collect();
        let mut radar = NewsRadar::new(sources).with_max_items(3);
        let briefing = radar.run_cycle();
        assert!(briefing.items.len() <= 3, "Should limit to max_items");
    }

    // 14
    #[test]
    fn test_cosine_similarity_edge_cases() {
        use super::scorer::cosine_similarity;

        assert!((cosine_similarity("hello world", "hello world") - 1.0).abs() < 1e-6);
        assert!((cosine_similarity("hello world", "world hello") - 1.0).abs() < 1e-6);
        assert!((cosine_similarity("abc def", "ghi jkl")).abs() < 1e-6);
        assert!((cosine_similarity("", "") - 0.0).abs() < 1e-6);
        assert!((cosine_similarity("a", "b") - 0.0).abs() < 1e-6);
    }

    // 15
    #[test]
    fn test_run_cycle_threshold_filtering() {
        struct LowScoreScorer(AIScorer);
        let mut radar = NewsRadar::new(vec![NewsSource::Custom {
            name: "filtered-source".to_string(),
            fetch_fn_desc: "test".to_string(),
        }]);
        let mut items = radar.fetch_all();
        radar.update_scores(&mut items);
        for item in &items {
            assert!(item.score >= 0.0 && item.score <= 10.0);
        }
    }
}
