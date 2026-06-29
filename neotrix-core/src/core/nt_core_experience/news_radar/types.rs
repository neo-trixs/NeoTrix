#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ── PollResult ──

#[derive(Debug, Clone)]
pub struct PollResult {
    pub items: Vec<NewsItem>,
    pub count: usize,
}

// ── NewsSource ──

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum NewsSource {
    HackerNews,
    Reddit { subreddit: String },
    RSS { url: String, name: String },
    GitHub { topic: String },
    Telegram { channel: String },
    Custom { name: String, fetch_fn_desc: String },
}

impl NewsSource {
    pub fn name(&self) -> &str {
        match self {
            NewsSource::HackerNews => "HackerNews",
            NewsSource::Reddit { subreddit } => subreddit,
            NewsSource::RSS { name, .. } => name,
            NewsSource::GitHub { topic } => topic,
            NewsSource::Telegram { channel } => channel,
            NewsSource::Custom { name, .. } => name,
        }
    }

    pub fn source_type(&self) -> &str {
        match self {
            NewsSource::HackerNews => "hackernews",
            NewsSource::Reddit { .. } => "reddit",
            NewsSource::RSS { .. } => "rss",
            NewsSource::GitHub { .. } => "github",
            NewsSource::Telegram { .. } => "telegram",
            NewsSource::Custom { .. } => "custom",
        }
    }
}

// ── NewsItem ──

#[derive(Debug, Clone)]
pub struct NewsItem {
    pub id: u64,
    pub title: String,
    pub url: String,
    pub source_name: String,
    pub summary: Option<String>,
    pub score: f64,
    pub ai_tags: Vec<String>,
    pub published_at: u64,
    pub category: Option<String>,
    pub comments_summary: Option<String>,
    pub enrichment_context: Option<String>,
}

impl NewsItem {
    pub fn new(id: u64, title: &str, url: &str, source_name: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            id,
            title: title.to_string(),
            url: url.to_string(),
            source_name: source_name.to_string(),
            summary: None,
            score: 0.0,
            ai_tags: Vec::new(),
            published_at: now,
            category: None,
            comments_summary: None,
            enrichment_context: None,
        }
    }

    pub fn with_score(mut self, score: f64) -> Self {
        self.score = score.clamp(0.0, 10.0);
        self
    }

    pub fn with_category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.ai_tags.push(tag.to_string());
        self
    }
}

// ── BriefingStats ──

#[derive(Debug, Clone)]
pub struct BriefingStats {
    pub total_articles: usize,
    pub avg_score: f64,
    pub top_category: String,
    pub source_breakdown: HashMap<String, usize>,
}

impl BriefingStats {
    pub fn from_items(items: &[NewsItem]) -> Self {
        let total_articles = items.len();
        let avg_score = if items.is_empty() {
            0.0
        } else {
            items.iter().map(|i| i.score).sum::<f64>() / items.len() as f64
        };

        let mut cat_counts: HashMap<String, usize> = HashMap::new();
        let mut src_counts: HashMap<String, usize> = HashMap::new();
        for item in items {
            if let Some(ref cat) = item.category {
                *cat_counts.entry(cat.clone()).or_insert(0) += 1;
            }
            *src_counts.entry(item.source_name.clone()).or_insert(0) += 1;
        }

        let top_category = cat_counts
            .into_iter()
            .max_by_key(|&(_, c)| c)
            .map(|(c, _)| c)
            .unwrap_or_else(|| "uncategorized".to_string());

        Self {
            total_articles,
            avg_score,
            top_category,
            source_breakdown: src_counts,
        }
    }
}

// ── NewsBriefing ──

#[derive(Debug, Clone)]
pub struct NewsBriefing {
    pub items: Vec<NewsItem>,
    pub generated_at: u64,
    pub total_sources_checked: usize,
    pub summary_stats: BriefingStats,
}

impl NewsBriefing {
    pub fn new(items: Vec<NewsItem>, total_sources_checked: usize) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let stats = BriefingStats::from_items(&items);
        Self {
            items,
            generated_at: now,
            total_sources_checked,
            summary_stats: stats,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

// ── Sentiment Label ──

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum SentimentLabel {
    VeryPositive,
    Positive,
    Neutral,
    Negative,
    VeryNegative,
    Mixed,
}

impl SentimentLabel {
    pub fn score(&self) -> f64 {
        match self {
            SentimentLabel::VeryPositive => 1.0,
            SentimentLabel::Positive => 0.6,
            SentimentLabel::Neutral => 0.0,
            SentimentLabel::Negative => -0.4,
            SentimentLabel::VeryNegative => -0.8,
            SentimentLabel::Mixed => 0.1,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            SentimentLabel::VeryPositive => "very-positive",
            SentimentLabel::Positive => "positive",
            SentimentLabel::Neutral => "neutral",
            SentimentLabel::Negative => "negative",
            SentimentLabel::VeryNegative => "very-negative",
            SentimentLabel::Mixed => "mixed",
        }
    }

    /// Heuristic keyword-based sentiment analysis.
    pub fn from_text(text: &str) -> Self {
        let lower = text.to_lowercase();
        let pos = [
            "breakthrough",
            "impressive",
            "excellent",
            "milestone",
            "innovation",
            "success",
            "remarkable",
            "promising",
            "soaring",
            "surge",
        ];
        let neg = [
            "critical",
            "vulnerability",
            "attack",
            "breach",
            "crisis",
            "failure",
            "decline",
            "threat",
            "crash",
            "disaster",
        ];
        let strong_pos = [
            "revolutionary",
            "unprecedented",
            "historic",
            "game-changing",
            "breakthrough",
        ];
        let strong_neg = [
            "catastrophic",
            "devastating",
            "emergency",
            "fatal",
            "collapse",
        ];

        let mut pos_score = 0;
        let mut neg_score = 0;
        for w in &pos {
            if lower.contains(w) {
                pos_score += 1;
            }
        }
        for w in &neg {
            if lower.contains(w) {
                neg_score += 1;
            }
        }
        for w in &strong_pos {
            if lower.contains(w) {
                pos_score += 2;
            }
        }
        for w in &strong_neg {
            if lower.contains(w) {
                neg_score += 2;
            }
        }

        if pos_score > 0 && neg_score > 0 {
            SentimentLabel::Mixed
        } else if pos_score >= 3 || strong_pos.iter().any(|w| lower.contains(w)) {
            SentimentLabel::VeryPositive
        } else if pos_score > 0 {
            SentimentLabel::Positive
        } else if neg_score >= 3 || strong_neg.iter().any(|w| lower.contains(w)) {
            SentimentLabel::VeryNegative
        } else if neg_score > 0 {
            SentimentLabel::Negative
        } else {
            SentimentLabel::Neutral
        }
    }
}

// ── Topic Trend ──

#[derive(Debug, Clone)]
pub struct TopicTrend {
    pub topic: String,
    pub window_counts: Vec<u64>,
    pub window_timestamps: Vec<u64>,
    pub velocity: f64,
    pub acceleration: f64,
    pub is_rising: bool,
    pub cross_source_count: usize,
    pub sentiment_trend: Vec<SentimentLabel>,
}

impl TopicTrend {
    pub fn new(topic: &str) -> Self {
        Self {
            topic: topic.to_string(),
            window_counts: vec![1],
            window_timestamps: Vec::new(),
            velocity: 0.0,
            acceleration: 0.0,
            is_rising: false,
            cross_source_count: 1,
            sentiment_trend: Vec::new(),
        }
    }
}

// ── Alert Level ──

/// Alert severity level (from BettaFish hotness escalation pattern)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum AlertLevel {
    Green = 0,
    Yellow = 1,
    Orange = 2,
    Red = 3,
}

impl AlertLevel {
    pub fn name(&self) -> &'static str {
        match self {
            AlertLevel::Green => "green",
            AlertLevel::Yellow => "yellow",
            AlertLevel::Orange => "orange",
            AlertLevel::Red => "red",
        }
    }

    pub fn from_score(score: f64) -> Self {
        if score >= 8.0 {
            AlertLevel::Red
        } else if score >= 5.0 {
            AlertLevel::Orange
        } else if score >= 2.5 {
            AlertLevel::Yellow
        } else {
            AlertLevel::Green
        }
    }
}

// ── Hotness Score ──

/// Composite hotness score (inspired by BettaFish multi-factor hotness + TrendRadar ranking)
#[derive(Debug, Clone)]
pub struct HotnessScore {
    pub composite: f64,
    pub velocity_factor: f64,
    pub divergence_factor: f64,
    pub cross_source_factor: f64,
    pub sentiment_momentum: f64,
}

impl HotnessScore {
    pub fn new(
        velocity: f64,
        acceleration: f64,
        divergence: f64,
        cross_source_count: usize,
        sentiment_momentum: f64,
    ) -> Self {
        let vf =
            (velocity.abs().min(20.0) / 20.0) * 0.3 + (acceleration.abs().min(10.0) / 10.0) * 0.15;
        let df = divergence.min(1.0) * 0.25;
        let cf = ((cross_source_count as f64).min(10.0) / 10.0) * 0.25;
        let sm = sentiment_momentum.clamp(-1.0, 1.0).abs() * 0.2;
        let composite = (vf + df + cf + sm) * 10.0;

        Self {
            composite: composite.clamp(0.0, 10.0),
            velocity_factor: vf * 10.0,
            divergence_factor: df * 10.0,
            cross_source_factor: cf * 10.0,
            sentiment_momentum: sm,
        }
    }
}

// ── Sentiment Divergence ──

/// Sentiment divergence (polarization detection):
/// When both positive and negative sentiment surge simultaneously,
/// it signals opinion polarization → potential tipping point.
#[derive(Debug, Clone)]
pub struct SentimentDivergence {
    pub positive_ratio: f64,
    pub negative_ratio: f64,
    pub divergence_index: f64,
    pub is_polarizing: bool,
}

impl SentimentDivergence {
    pub fn compute(sentiments: &[SentimentLabel]) -> Self {
        let n = sentiments.len() as f64;
        if n == 0.0 {
            return Self {
                positive_ratio: 0.0,
                negative_ratio: 0.0,
                divergence_index: 0.0,
                is_polarizing: false,
            };
        }
        let pos = sentiments
            .iter()
            .filter(|s| matches!(s, SentimentLabel::VeryPositive | SentimentLabel::Positive))
            .count() as f64;
        let neg = sentiments
            .iter()
            .filter(|s| matches!(s, SentimentLabel::VeryNegative | SentimentLabel::Negative))
            .count() as f64;
        let pos_r = pos / n;
        let neg_r = neg / n;
        let divergence = 1.0 - (pos_r - neg_r).abs();
        Self {
            positive_ratio: pos_r,
            negative_ratio: neg_r,
            divergence_index: divergence,
            is_polarizing: divergence > 0.5 && pos_r > 0.2 && neg_r > 0.2,
        }
    }
}

// ── Early Warning Signal ──

/// Deep-early-warning inspired signal:
/// Tracks variance and "autocorrelation" of sentiment velocity as leading indicators
/// of opinion tipping points (inspired by PNAS 2021 deep-early-warnings).
#[derive(Debug, Clone)]
pub struct EarlyWarningSignal {
    pub variance: f64,
    pub pseudo_autocorr: f64,
    pub is_pre_tipping: bool,
}

impl EarlyWarningSignal {
    pub fn compute(velocity_history: &[f64]) -> Self {
        let n = velocity_history.len();
        if n < 3 {
            return Self {
                variance: 0.0,
                pseudo_autocorr: 0.0,
                is_pre_tipping: false,
            };
        }
        let mean = velocity_history.iter().sum::<f64>() / n as f64;
        let variance = velocity_history
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / n as f64;
        let mut covar = 0.0;
        for i in 0..n - 1 {
            covar += (velocity_history[i] - mean) * (velocity_history[i + 1] - mean);
        }
        let autocorr = if variance > 1e-10 {
            covar / variance / (n - 1) as f64
        } else {
            0.0
        };
        let is_pre_tipping = variance > 1.0 && autocorr > 0.3;
        Self {
            variance,
            pseudo_autocorr: autocorr,
            is_pre_tipping,
        }
    }
}

// ── Opinion Alert ──

/// Consolidated opinion alert combining all signals
#[derive(Debug, Clone)]
pub struct OpinionAlert {
    pub topic: String,
    pub level: AlertLevel,
    pub hotness: HotnessScore,
    pub divergence: SentimentDivergence,
    pub early_warning: EarlyWarningSignal,
    pub prediction_confidence: f64,
    pub sources_covering: Vec<String>,
    pub trigger_reason: String,
}

// ── Common stop words ──

pub const STOP_WORDS: &[&str] = &[
    "this", "that", "with", "from", "have", "been", "were", "they", "them", "their", "what",
    "when", "where", "which", "while", "there", "about", "after", "before", "would", "could",
    "should", "also", "than", "then", "just", "like", "more", "some", "into", "over", "such",
    "only", "other", "very", "well",
];
