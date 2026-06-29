#![forbid(unsafe_code)]

use std::collections::HashMap;

use super::types::NewsItem;

// ── AIScorer ──

#[derive(Debug, Clone)]
pub struct AIScorer {
    keyword_weights: HashMap<String, f64>,
    source_authority: HashMap<String, f64>,
    min_keywords: usize,
}

impl Default for AIScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl AIScorer {
    pub fn new() -> Self {
        let mut keyword_weights = HashMap::new();
        for (kw, w) in [
            ("ai", 1.5),
            ("machine learning", 1.8),
            ("deep learning", 1.8),
            ("neural", 1.5),
            ("llm", 1.6),
            ("gpt", 1.4),
            ("transformer", 1.5),
            ("robotics", 1.4),
            ("autonomous", 1.3),
            ("quantum", 1.6),
            ("blockchain", 0.8),
            ("cybersecurity", 1.2),
            ("open source", 1.0),
            ("startup", 0.9),
            ("funding", 0.8),
            ("acquisition", 0.7),
            ("breakthrough", 1.2),
            ("research", 1.1),
            ("paper", 0.9),
            ("announcement", 0.6),
            ("release", 0.7),
            ("vulnerability", 1.3),
            ("performance", 0.9),
            ("optimization", 1.0),
            ("architecture", 0.8),
        ] {
            keyword_weights.insert(kw.to_string(), w);
        }

        let mut source_authority = HashMap::new();
        for (src, w) in [
            ("hackernews", 7.0),
            ("reddit", 5.0),
            ("github", 6.5),
            ("rss", 6.0),
            ("telegram", 4.0),
            ("custom", 5.0),
        ] {
            source_authority.insert(src.to_string(), w);
        }

        Self {
            keyword_weights,
            source_authority,
            min_keywords: 1,
        }
    }

    pub fn score(&self, title: &str, source_type: &str, tags: &[String]) -> f64 {
        let title_lower = title.to_lowercase();
        let mut base = self
            .source_authority
            .get(source_type)
            .copied()
            .unwrap_or(5.0);
        let mut matched_keywords = 0;
        let mut total_weight = 0.0;

        for (kw, w) in &self.keyword_weights {
            if title_lower.contains(kw) {
                matched_keywords += 1;
                total_weight += w;
            }
        }

        let has_enough_keywords = matched_keywords >= self.min_keywords;
        if has_enough_keywords {
            let bonus = (total_weight / matched_keywords as f64).min(3.0);
            base += bonus;
        }

        for tag in tags {
            let tag_lower = tag.to_lowercase();
            if let Some(w) = self.keyword_weights.get(&tag_lower) {
                base += w * 0.3;
            }
        }

        base += (title.len() as f64 / 100.0).min(1.0);

        base.clamp(0.0, 10.0)
    }

    pub fn score_item(&self, item: &NewsItem) -> f64 {
        let title_lower = item.title.to_lowercase();
        let mut matched: Vec<String> = Vec::new();
        for kw in self.keyword_weights.keys() {
            if title_lower.contains(kw) {
                matched.push(kw.clone());
            }
        }

        let mut base = 5.0;

        let total_weight: f64 = matched
            .iter()
            .filter_map(|k| self.keyword_weights.get(k))
            .sum();
        if !matched.is_empty() {
            base += (total_weight / matched.len() as f64).min(3.0);
        }

        if let Some(ref summary) = item.summary {
            let summary_lower = summary.to_lowercase();
            for (kw, w) in &self.keyword_weights {
                if summary_lower.contains(kw) {
                    base += w * 0.2;
                }
            }
        }

        base += (item.title.len() as f64 / 100.0).min(1.0);

        base.clamp(0.0, 10.0)
    }
}

// ── StoryDeduplicator ──

#[derive(Debug, Clone)]
pub struct StoryDeduplicator {
    similarity_threshold: f64,
    seen_urls: Vec<String>,
    seen_titles: Vec<String>,
}

impl Default for StoryDeduplicator {
    fn default() -> Self {
        Self::new(0.85)
    }
}

impl StoryDeduplicator {
    pub fn new(threshold: f64) -> Self {
        Self {
            similarity_threshold: threshold,
            seen_urls: Vec::new(),
            seen_titles: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.seen_urls.clear();
        self.seen_titles.clear();
    }

    pub fn is_duplicate(&mut self, item: &NewsItem) -> bool {
        if self.seen_urls.iter().any(|u| u == &item.url) {
            return true;
        }
        for t in &self.seen_titles {
            if cosine_similarity(t, &item.title) > self.similarity_threshold {
                return true;
            }
        }
        self.seen_urls.push(item.url.clone());
        self.seen_titles.push(item.title.clone());
        false
    }

    pub fn deduplicate(&mut self, items: Vec<NewsItem>) -> Vec<NewsItem> {
        self.reset();
        items
            .into_iter()
            .filter(|i| !self.is_duplicate(i))
            .collect()
    }

    pub fn merge_dedup(&mut self, current: Vec<NewsItem>, previous: &[NewsItem]) -> Vec<NewsItem> {
        self.reset();
        for p in previous {
            self.seen_urls.push(p.url.clone());
            self.seen_titles.push(p.title.clone());
        }
        current
            .into_iter()
            .filter(|i| !self.is_duplicate(i))
            .collect()
    }
}

// ── Cosine similarity & tokenization ──

pub fn cosine_similarity(a: &str, b: &str) -> f64 {
    let words_a = tokenize(a);
    let words_b = tokenize(b);
    if words_a.is_empty() || words_b.is_empty() {
        return 0.0;
    }
    let mut all_keys: Vec<&str> = words_a.keys().chain(words_b.keys()).copied().collect();
    all_keys.sort();
    all_keys.dedup();

    let mut dot = 0.0;
    let mut mag_a = 0.0;
    let mut mag_b = 0.0;
    for &key in &all_keys {
        let va = words_a.get(key).copied().unwrap_or(0.0);
        let vb = words_b.get(key).copied().unwrap_or(0.0);
        dot += va * vb;
        mag_a += va * va;
        mag_b += vb * vb;
    }
    let denom = mag_a.sqrt() * mag_b.sqrt();
    if denom < 1e-10 {
        0.0
    } else {
        dot / denom
    }
}

fn tokenize(s: &str) -> HashMap<&str, f64> {
    let mut map = HashMap::new();
    for word in s.split_whitespace() {
        let cleaned: &str = word.trim_matches(|c: char| !c.is_alphanumeric());
        if !cleaned.is_empty() && cleaned.len() > 2 {
            *map.entry(cleaned).or_insert(0.0) += 1.0;
        }
    }
    let len = map.len() as f64;
    if len > 0.0 {
        for v in map.values_mut() {
            *v /= len;
        }
    }
    map
}
