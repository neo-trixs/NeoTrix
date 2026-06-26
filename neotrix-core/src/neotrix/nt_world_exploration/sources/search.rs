use crate::neotrix::nt_world_exploration::content::{ExplorationSourceType, SourceContent};
use crate::neotrix::nt_world_exploration::source_trait::ExplorationSource;
use crate::neotrix::nt_world_search::WebSearchEngine;

/// 搜索引擎探索源 — 通过 WebSearch 发现 URL
pub struct SearchSource {
    query_queue: Vec<String>,
}

impl SearchSource {
    pub fn new() -> Self {
        Self {
            query_queue: Vec::new(),
        }
    }

    pub fn search(&mut self, query: impl Into<String>) {
        self.query_queue.push(query.into());
    }
}

impl ExplorationSource for SearchSource {
    fn name(&self) -> &'static str {
        "web_search"
    }

    fn confidence(&self) -> f64 {
        0.75
    }

    fn explore(&mut self) -> Result<Vec<SourceContent>, String> {
        let engine = WebSearchEngine::default();
        let mut all = Vec::new();
        for query in self.query_queue.drain(..) {
            if let Ok(results) = engine.search(&query, 10) {
                for r in results {
                    all.push(
                        SourceContent::new(&r.url, &r.snippet, ExplorationSourceType::WebSearch)
                            .with_title(&r.title)
                            .with_url(&r.url),
                    );
                }
            }
        }
        Ok(all)
    }

    fn pending_count(&self) -> usize {
        self.query_queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_queues_query() {
        let mut src = SearchSource::new();
        assert_eq!(src.pending_count(), 0);
        src.search("rust programming");
        assert_eq!(src.pending_count(), 1);
        src.search("ai agents");
        assert_eq!(src.pending_count(), 2);
    }

    #[test]
    fn test_explore_drains_queue() {
        let mut src = SearchSource::new();
        src.search("test query");
        assert_eq!(src.pending_count(), 1);
        let _ = src.explore().unwrap();
        assert_eq!(src.pending_count(), 0);
    }

    #[test]
    fn test_empty_explore() {
        let mut src = SearchSource::new();
        let results = src.explore().unwrap();
        assert!(results.is_empty());
    }
}
