use crate::traits::ToolExecutor;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct DciQuery {
    pub query_type: DciQueryType,
    pub pattern: String,
    pub path: Option<String>,
    pub max_results: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DciQueryType {
    Grep,
    Glob,
    Read,
    Bash,
}

#[derive(Debug, Clone)]
pub struct DciResult {
    pub source: String,
    pub content: String,
    pub relevance: f64,
    pub query_type: DciQueryType,
}

pub struct DciRetriever {
    recent_queries: VecDeque<DciQuery>,
    max_history: usize,
    fallback_to_embedding: bool,
}

impl DciRetriever {
    pub fn new() -> Self {
        Self {
            recent_queries: VecDeque::with_capacity(64),
            max_history: 64,
            fallback_to_embedding: true,
        }
    }

    pub fn retrieve(&self, query: &DciQuery, executor: &dyn ToolExecutor) -> Vec<DciResult> {
        let mut results = Vec::new();
        match query.query_type {
            DciQueryType::Grep => {
                let path = query.path.as_deref().unwrap_or(".");
                let (output, ok) = executor.grep(&query.pattern, path);
                if ok {
                    results.push(DciResult {
                        source: format!("grep '{}' {}", query.pattern, path),
                        content: output,
                        relevance: 0.9,
                        query_type: DciQueryType::Grep,
                    });
                }
            }
            DciQueryType::Glob => {
                let (output, ok) = executor.glob(&query.pattern);
                if ok {
                    results.push(DciResult {
                        source: format!("glob '{}'", query.pattern),
                        content: output,
                        relevance: 0.8,
                        query_type: DciQueryType::Glob,
                    });
                }
            }
            DciQueryType::Read => {
                let path = query.pattern.clone();
                let (output, ok) = executor.file_read(&path);
                if ok {
                    results.push(DciResult {
                        source: format!("read {}", path),
                        content: output,
                        relevance: 1.0,
                        query_type: DciQueryType::Read,
                    });
                }
            }
            DciQueryType::Bash => {
                let (output, ok) = executor.bash(&query.pattern);
                if ok {
                    results.push(DciResult {
                        source: format!("bash '{}'", query.pattern),
                        content: output,
                        relevance: 0.7,
                        query_type: DciQueryType::Bash,
                    });
                }
            }
        }
        results
    }

    pub fn retrieve_multi(
        &mut self,
        queries: Vec<DciQuery>,
        executor: &dyn ToolExecutor,
    ) -> Vec<DciResult> {
        let mut all = Vec::new();
        for q in queries {
            let results = self.retrieve(&q, executor);
            all.extend(results);
            self.record_query(q);
        }
        all
    }

    pub fn retrieve_deep(
        &mut self,
        concept: &str,
        executor: &dyn ToolExecutor,
    ) -> Vec<DciResult> {
        let queries = vec![
            DciQuery { query_type: DciQueryType::Grep, pattern: concept.to_string(), path: Some(".".into()), max_results: 20 },
            DciQuery { query_type: DciQueryType::Glob, pattern: format!("**/*{concept}*"), path: None, max_results: 10 },
            DciQuery { query_type: DciQueryType::Read, pattern: format!("archive/consolidated/EMERGENT_SYNTHESIS.md"), path: None, max_results: 1 },
        ];
        self.retrieve_multi(queries, executor)
    }

    fn record_query(&mut self, query: DciQuery) {
        if self.recent_queries.len() >= self.max_history {
            self.recent_queries.pop_front();
        }
        self.recent_queries.push_back(query);
    }

    pub fn recent_queries(&self) -> &VecDeque<DciQuery> {
        &self.recent_queries
    }

    pub fn set_fallback(&mut self, fallback: bool) {
        self.fallback_to_embedding = fallback;
    }
}

impl Default for DciRetriever {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockExecutor;
    impl ToolExecutor for MockExecutor {
        fn web_search(&self, _query: &str) -> (String, bool) { ("".into(), false) }
        fn web_fetch(&self, _url: &str) -> (String, bool) { ("".into(), false) }
        fn file_read(&self, path: &str) -> (String, bool) { (format!("content of {path}"), true) }
        fn file_write(&self, _path: &str, _content: &str) -> (String, bool) { ("".into(), false) }
        fn file_edit(&self, _path: &str, _old: &str, _new: &str) -> (String, bool) { ("".into(), false) }
        fn bash(&self, cmd: &str) -> (String, bool) { (format!("ran: {cmd}"), true) }
        fn glob(&self, pattern: &str) -> (String, bool) { (format!("matched: {pattern}"), true) }
        fn grep(&self, pattern: &str, path: &str) -> (String, bool) { (format!("grep {pattern} in {path}"), true) }
    }

    #[test]
    fn test_dci_retrieve_read() {
        let retriever = DciRetriever::new();
        let q = DciQuery { query_type: DciQueryType::Read, pattern: "test.txt".into(), path: None, max_results: 1 };
        let results = retriever.retrieve(&q, &MockExecutor);
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("test.txt"));
    }

    #[test]
    fn test_dci_retrieve_multi() {
        let mut retriever = DciRetriever::new();
        let queries = vec![
            DciQuery { query_type: DciQueryType::Glob, pattern: "*.rs".into(), path: None, max_results: 5 },
            DciQuery { query_type: DciQueryType::Grep, pattern: "struct".into(), path: Some("src".into()), max_results: 10 },
        ];
        let results = retriever.retrieve_multi(queries, &MockExecutor);
        assert_eq!(results.len(), 2);
        assert_eq!(retriever.recent_queries().len(), 2);
    }

    #[test]
    fn test_dci_retrieve_deep() {
        let mut retriever = DciRetriever::new();
        let results = retriever.retrieve_deep("DciRetriever", &MockExecutor);
        assert!(results.len() >= 3);
    }
}
