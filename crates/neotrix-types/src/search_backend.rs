use serde::{Serialize, Deserialize};

pub trait SearchBackend {
    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, String>;
}

/// Unified search result type for NeoTrix.
/// 
/// This is the canonical SearchResult used across all search domains.
/// Domain-specific search results should use this as the base type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Unique identifier for the result
    pub id: String,
    /// Title or heading of the result
    pub title: String,
    /// URL or path to the result
    pub url: String,
    /// Brief snippet or summary
    pub snippet: String,
    /// Relevance score (0.0 to 1.0)
    pub score: f64,
    /// Source type (web, local, academic, etc.)
    pub source_type: Option<String>,
    /// Timestamp when the result was fetched (Unix timestamp)
    pub fetched_at: Option<u64>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl SearchResult {
    /// Create a new SearchResult with minimal fields
    pub fn new(id: impl Into<String>, title: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            url: url.into(),
            snippet: String::new(),
            score: 0.0,
            source_type: None,
            fetched_at: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set the snippet
    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = snippet.into();
        self
    }

    /// Set the score
    pub fn with_score(mut self, score: f64) -> Self {
        self.score = score;
        self
    }

    /// Set the source type
    pub fn with_source_type(mut self, source_type: impl Into<String>) -> Self {
        self.source_type = Some(source_type.into());
        self
    }

    /// Set the fetched timestamp
    pub fn with_fetched_at(mut self, timestamp: u64) -> Self {
        self.fetched_at = Some(timestamp);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBackend;

    impl SearchBackend for MockBackend {
        fn search(&self, _query: &str, _limit: usize) -> Result<Vec<SearchResult>, String> {
            Ok(vec![SearchResult {
                id: "1".into(),
                title: "Test".into(),
                url: "https://example.com".into(),
                snippet: "test".into(),
                score: 0.9,
                source_type: Some("web".into()),
                fetched_at: None,
                metadata: std::collections::HashMap::new(),
            }])
        }
    }

    #[test]
    fn test_mock_backend_search() {
        let backend = MockBackend;
        let results = backend.search("test", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "1");
    }

    #[test]
    fn test_search_result_builder() {
        let result = SearchResult::new("1", "Test", "https://example.com")
            .with_snippet("A test result")
            .with_score(0.95)
            .with_source_type("web");
        
        assert_eq!(result.id, "1");
        assert_eq!(result.title, "Test");
        assert_eq!(result.url, "https://example.com");
        assert_eq!(result.snippet, "A test result");
        assert_eq!(result.score, 0.95);
        assert_eq!(result.source_type, Some("web".to_string()));
    }
}
