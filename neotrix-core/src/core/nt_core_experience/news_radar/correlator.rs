#![forbid(unsafe_code)]

use super::scorer::cosine_similarity;
use super::types::NewsItem;

// ── CrossSourceResult ──

#[derive(Debug, Clone)]
pub struct CrossSourceResult {
    pub story_id: String,
    pub sources: Vec<(String, String)>,
    pub canonical_title: String,
    pub avg_score: f64,
    pub coverage_count: usize,
    pub first_seen: u64,
    pub last_seen: u64,
}

#[derive(Debug, Clone)]
pub struct CrossSourceCorrelator {
    similarity_threshold: f64,
    stories: Vec<CrossSourceResult>,
}

impl CrossSourceCorrelator {
    pub fn new(threshold: f64) -> Self {
        Self {
            similarity_threshold: threshold,
            stories: Vec::new(),
        }
    }

    pub fn correlate(&mut self, items: &[NewsItem]) -> Vec<CrossSourceResult> {
        let mut results: Vec<CrossSourceResult> = Vec::new();
        for item in items {
            let mut matched = false;
            for story in &mut results {
                if cosine_similarity(&story.canonical_title, &item.title)
                    > self.similarity_threshold
                {
                    story
                        .sources
                        .push((item.source_name.clone(), item.url.clone()));
                    story.coverage_count += 1;
                    story.avg_score = (story.avg_score * (story.coverage_count - 1) as f64
                        + item.score)
                        / story.coverage_count as f64;
                    story.last_seen = item.published_at;
                    matched = true;
                    break;
                }
            }
            if !matched {
                results.push(CrossSourceResult {
                    story_id: format!("story-{}", hash_id(&item.title)),
                    sources: vec![(item.source_name.clone(), item.url.clone())],
                    canonical_title: item.title.clone(),
                    avg_score: item.score,
                    coverage_count: 1,
                    first_seen: item.published_at,
                    last_seen: item.published_at,
                });
            }
        }
        results.sort_by(|a, b| b.coverage_count.cmp(&a.coverage_count));
        self.stories = results.clone();
        results
    }

    pub fn multi_source_stories(&self, min_sources: usize) -> Vec<&CrossSourceResult> {
        self.stories
            .iter()
            .filter(|s| s.coverage_count >= min_sources)
            .collect()
    }
}

fn hash_id(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    h
}
