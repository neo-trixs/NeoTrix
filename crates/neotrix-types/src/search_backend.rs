pub trait SearchBackend {
    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, String>;
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub score: f64,
    pub snippet: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBackend;

    impl SearchBackend for MockBackend {
        fn search(&self, _query: &str, _limit: usize) -> Result<Vec<SearchResult>, String> {
            Ok(vec![SearchResult {
                id: "1".into(),
                score: 0.9,
                snippet: "test".into(),
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
}
