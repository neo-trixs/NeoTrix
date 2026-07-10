#![forbid(unsafe_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::three_tier::MemoryItem;

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamiliarityWeightedRetrieval {
    pub familiarity_map: HashMap<String, f64>,
    pub recency_bias: f64,
    pub frequency_bias: f64,
}

impl FamiliarityWeightedRetrieval {
    pub fn new(recency_bias: f64, frequency_bias: f64) -> Self {
        Self {
            familiarity_map: HashMap::new(),
            recency_bias,
            frequency_bias,
        }
    }

    pub fn search(
        &self,
        items: &[MemoryItem],
        query: &str,
        max_results: usize,
    ) -> Vec<(usize, f64)> {
        let mut scored: Vec<(usize, f64)> = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let relevance = Self::compute_relevance(item, query);
                let familiarity = item
                    .content
                    .split_whitespace()
                    .filter_map(|w| self.familiarity_map.get(w))
                    .sum::<f64>()
                    * 0.2;
                let recency = if item.last_accessed > 0 {
                    let age = now_secs().saturating_sub(item.last_accessed);
                    (1.0 / (1.0 + age as f64)) * self.recency_bias
                } else {
                    0.0
                };
                let score = relevance + familiarity + recency;
                (i, score)
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(max_results);
        scored
    }

    pub fn update_familiarity(&mut self, item: &MemoryItem) {
        for word in item.content.split_whitespace() {
            let entry = self.familiarity_map.entry(word.to_string()).or_insert(0.0);
            *entry = (*entry + 0.1).max(0.0).min(1.0);
        }
    }

    pub fn compute_relevance(item: &MemoryItem, query: &str) -> f64 {
        Self::keyword_overlap(&item.content, query) + item.importance * 0.3
    }

    pub fn keyword_overlap(content: &str, query: &str) -> f64 {
        if query.is_empty() {
            return 0.0;
        }
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let total = query_words.len() as f64;
        if total == 0.0 {
            return 0.0;
        }
        let matched = query_words
            .iter()
            .filter(|w| content.contains(*w))
            .count() as f64;
        matched / total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::l3_memory_impl::nt_memory_historian::dmn_consolidation::three_tier::{MemoryItem, MemoryTier};

    fn make_item(id: usize, content: &str, importance: f64) -> MemoryItem {
        let mut item = MemoryItem::new(id, content.to_string(), importance, MemoryTier::ShortTerm);
        item.last_accessed = 1000;
        item
    }

    #[test]
    fn test_search_ranking() {
        let items = vec![
            make_item(1, "apple banana cherry", 0.9),
            make_item(2, "banana date", 0.3),
            make_item(3, "apple fig grape", 0.5),
        ];
        let retriever = FamiliarityWeightedRetrieval::new(0.1, 0.1);
        let results = retriever.search(&items, "apple", 3);
        assert!(!results.is_empty());
        assert_eq!(results[0].0, 0);
    }

    #[test]
    fn test_update_familiarity() {
        let mut retriever = FamiliarityWeightedRetrieval::new(0.1, 0.1);
        let item = make_item(1, "hello world hello", 0.5);
        retriever.update_familiarity(&item);
        assert!((retriever.familiarity_map.get("hello").unwrap() - 0.2).abs() < 1e-6);
        assert!((retriever.familiarity_map.get("world").unwrap() - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_compute_relevance() {
        let item = make_item(1, "rust programming language", 0.8);
        let relevance = FamiliarityWeightedRetrieval::compute_relevance(&item, "rust");
        let expected = 1.0 / 1.0 + 0.8 * 0.3;
        assert!((relevance - expected).abs() < 1e-6);
    }

    #[test]
    fn test_keyword_overlap() {
        let score = FamiliarityWeightedRetrieval::keyword_overlap("the quick brown fox", "quick fox");
        assert!((score - 2.0 / 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_keyword_overlap_empty_query() {
        let score = FamiliarityWeightedRetrieval::keyword_overlap("anything", "");
        assert!((score - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_keyword_overlap_no_match() {
        let score =
            FamiliarityWeightedRetrieval::keyword_overlap("aaa bbb ccc", "xxx yyy");
        assert!((score - 0.0).abs() < 1e-6);
    }
}
