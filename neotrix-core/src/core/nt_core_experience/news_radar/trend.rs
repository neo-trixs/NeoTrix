#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::types::{NewsItem, SentimentLabel, TopicTrend, STOP_WORDS};

#[derive(Debug, Clone)]
pub struct TrendTracker {
    window_duration_secs: u64,
    max_windows: usize,
    rising_threshold: f64,
    topics: HashMap<String, TopicTrend>,
    current_window_start: u64,
    current_window_topics: HashMap<String, (u64, Vec<SentimentLabel>)>,
    cycle: u64,
}

impl TrendTracker {
    pub fn new(window_duration_secs: u64, max_windows: usize, rising_threshold: f64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            window_duration_secs,
            max_windows,
            rising_threshold,
            topics: HashMap::new(),
            current_window_start: now,
            current_window_topics: HashMap::new(),
            cycle: 0,
        }
    }

    pub fn record_item(&mut self, item: &NewsItem) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if now - self.current_window_start >= self.window_duration_secs {
            self.flush_window(now);
        }
        let lower_title = item.title.to_lowercase();
        let words: Vec<&str> = lower_title
            .split_whitespace()
            .filter(|w| w.len() > 3 && !STOP_WORDS.contains(w))
            .collect();
        let sentiment = SentimentLabel::from_text(&item.title);

        for word in words {
            let entry = self
                .current_window_topics
                .entry(word.to_string())
                .or_insert_with(|| (0, Vec::new()));
            entry.0 += 1;
            entry.1.push(sentiment);
        }
    }

    fn flush_window(&mut self, now: u64) {
        let window_ts = self.current_window_start;
        for (topic, (count, sentiments)) in self.current_window_topics.drain() {
            let trend = self
                .topics
                .entry(topic)
                .or_insert_with(|| TopicTrend::new(""));
            trend.window_counts.push(count);
            trend.window_timestamps.push(window_ts);
            trend.sentiment_trend.extend(sentiments);

            while trend.window_counts.len() > self.max_windows {
                trend.window_counts.remove(0);
                trend.window_timestamps.remove(0);
            }

            let n = trend.window_counts.len();
            if n >= 2 {
                let recent = trend.window_counts[n - 1] as f64;
                let prev = trend.window_counts[n - 2] as f64;
                trend.velocity = recent - prev;
                trend.is_rising = trend.velocity > self.rising_threshold;
                if n >= 3 {
                    let prev_prev = trend.window_counts[n - 3] as f64;
                    trend.acceleration = (recent - prev) - (prev - prev_prev);
                }
            }
        }
        self.current_window_start = now;
        self.cycle += 1;
    }

    pub fn rising_topics(&self, min_velocity: f64) -> Vec<&TopicTrend> {
        self.topics
            .values()
            .filter(|t| t.is_rising && t.velocity >= min_velocity)
            .collect()
    }

    pub fn top_trends(&self, n: usize) -> Vec<String> {
        let mut sorted: Vec<&TopicTrend> = self.topics.values().collect();
        sorted.sort_by(|a, b| {
            b.velocity
                .partial_cmp(&a.velocity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.iter().take(n).map(|t| t.topic.clone()).collect()
    }

    pub fn all_topics(&self) -> &HashMap<String, TopicTrend> {
        &self.topics
    }
}
