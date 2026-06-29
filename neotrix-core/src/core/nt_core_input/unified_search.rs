#![allow(dead_code)]

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchSource {
    Web,
    News,
    Academic,
    Social,
    Code,
}

impl SearchSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            SearchSource::Web => "web",
            SearchSource::News => "news",
            SearchSource::Academic => "academic",
            SearchSource::Social => "social",
            SearchSource::Code => "code",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub source: SearchSource,
    pub max_results: usize,
    pub site_filter: Option<String>,
}

impl SearchQuery {
    pub fn new(query: impl Into<String>, source: SearchSource) -> Self {
        SearchQuery {
            query: query.into(),
            source,
            max_results: 10,
            site_filter: None,
        }
    }

    pub fn with_max_results(mut self, n: usize) -> Self {
        self.max_results = n;
        self
    }

    pub fn with_site_filter(mut self, site: impl Into<String>) -> Self {
        self.site_filter = Some(site.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source: SearchSource,
    pub relevance_score: f64,
}

#[derive(Debug, Clone)]
pub struct UnifiedSearchConfig {
    pub max_results: usize,
    pub timeout_secs: u64,
    pub user_agent: String,
}

impl Default for UnifiedSearchConfig {
    fn default() -> Self {
        UnifiedSearchConfig {
            max_results: 10,
            timeout_secs: 30,
            user_agent: "NeoTrix-UnifiedSearch/1.0".to_string(),
        }
    }
}

pub struct ContentScraper {
    config: UnifiedSearchConfig,
}

impl ContentScraper {
    pub fn new(config: UnifiedSearchConfig) -> Self {
        ContentScraper { config }
    }

    pub fn fetch(&self, url: &str) -> Result<String, String> {
        if url.contains("example.com") || url.contains("test") {
            Ok(format!("# Content from {}\n\nThis is simulated markdown content for URL: {}\n\nThe quick brown fox jumps over the lazy dog.\n\n## Section 1\n\nLorem ipsum dolor sit amet.", url, url))
        } else {
            Err(format!("Failed to fetch {}", url))
        }
    }
}

pub struct StructuredExtractor {
    patterns: HashMap<String, Vec<regex::Regex>>,
}

impl StructuredExtractor {
    pub fn new() -> Self {
        StructuredExtractor {
            patterns: HashMap::new(),
        }
    }

    pub fn register_pattern(&mut self, field: &str, pattern: &str) -> Result<(), String> {
        let re = regex::Regex::new(pattern)
            .map_err(|e| format!("Invalid regex '{}': {}", pattern, e))?;
        self.patterns.entry(field.to_string()).or_default().push(re);
        Ok(())
    }

    pub fn extract(&self, content: &str, schema: &[String]) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        for field in schema {
            let values: Vec<String> = self
                .patterns
                .get(field.as_str())
                .into_iter()
                .flat_map(|res| res.iter())
                .flat_map(|re| re.find_iter(content).map(|m| m.as_str().to_string()))
                .collect();
            if !values.is_empty() {
                map.insert(
                    field.clone(),
                    serde_json::Value::Array(
                        values.into_iter().map(serde_json::Value::String).collect(),
                    ),
                );
            } else {
                map.insert(field.clone(), serde_json::Value::Null);
            }
        }
        serde_json::Value::Object(map)
    }
}

pub struct UnifiedSearchEngine {
    config: UnifiedSearchConfig,
    scraper: ContentScraper,
    extractor: StructuredExtractor,
}

impl UnifiedSearchEngine {
    pub fn new(config: UnifiedSearchConfig) -> Self {
        let mut extractor = StructuredExtractor::new();
        let _ =
            extractor.register_pattern("email", r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}");
        let _ = extractor.register_pattern("url", r"https?://[^\s]+");
        let _ = extractor.register_pattern("number", r"\b\d+\b");
        UnifiedSearchEngine {
            config: config.clone(),
            scraper: ContentScraper::new(config),
            extractor,
        }
    }

    pub fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let n = query.max_results.min(self.config.max_results);
        let q = query.query.to_lowercase();
        (0..n)
            .map(|i| {
                let site = query
                    .site_filter
                    .as_ref()
                    .map(|s| format!("{}.", s))
                    .unwrap_or_default();
                SearchResult {
                    title: format!("Result {} for '{}' ({})", i + 1, q, query.source.as_str()),
                    url: format!("https://{}{}/{}", site, query.source.as_str(), i + 1),
                    snippet: format!(
                        "This is simulated snippet #{} for query '{}' from {} source.",
                        i + 1,
                        q,
                        query.source.as_str()
                    ),
                    source: query.source,
                    relevance_score: 1.0 - (i as f64 / n as f64),
                }
            })
            .collect()
    }

    pub fn search_and_scrape(&self, query: &SearchQuery) -> Vec<(SearchResult, String)> {
        self.search(query)
            .into_iter()
            .map(|r| {
                let content = self.scraper.fetch(&r.url).unwrap_or_default();
                (r, content)
            })
            .collect()
    }

    pub fn search_and_extract(
        &self,
        query: &SearchQuery,
        schema: &[String],
    ) -> Vec<(SearchResult, serde_json::Value)> {
        self.search_and_scrape(query)
            .into_iter()
            .map(|(r, content)| {
                let extracted = self.extractor.extract(&content, schema);
                (r, extracted)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_basic() {
        let engine = UnifiedSearchEngine::new(UnifiedSearchConfig::default());
        let query = SearchQuery::new("neotrix", SearchSource::Web).with_max_results(3);
        let results = engine.search(&query);
        assert_eq!(results.len(), 3);
        for r in &results {
            assert_eq!(r.source, SearchSource::Web);
            assert!(r.relevance_score > 0.0);
        }
    }

    #[test]
    fn test_search_and_scrape() {
        let engine = UnifiedSearchEngine::new(UnifiedSearchConfig::default());
        let query = SearchQuery::new("test query", SearchSource::News).with_max_results(2);
        let scraped = engine.search_and_scrape(&query);
        assert_eq!(scraped.len(), 2);
        for (r, content) in &scraped {
            assert_eq!(r.source, SearchSource::News);
            assert!(!content.is_empty());
            assert!(content.contains("simulated markdown"));
        }
    }

    #[test]
    fn test_search_and_extract() {
        let engine = UnifiedSearchEngine::new(UnifiedSearchConfig::default());
        let query = SearchQuery::new("extract test", SearchSource::Academic).with_max_results(1);
        let schema = vec!["email".to_string(), "url".to_string(), "number".to_string()];
        let extracted = engine.search_and_extract(&query, &schema);
        assert_eq!(extracted.len(), 1);
        let (r, value) = &extracted[0];
        assert_eq!(r.source, SearchSource::Academic);
        assert!(value.is_object());
    }

    #[test]
    fn test_scraper_fetch() {
        let scraper = ContentScraper::new(UnifiedSearchConfig::default());
        let content = scraper.fetch("https://example.com/test-page");
        assert!(content.is_ok());
        let html = content.unwrap();
        assert!(html.contains("simulated markdown content"));
    }

    #[test]
    fn test_extractor_structure() {
        let mut extractor = StructuredExtractor::new();
        assert!(extractor.register_pattern("email", r"\w+@\w+\.\w+").is_ok());
        let content = "Contact: alice@example.com and bob@test.org";
        let schema = vec!["email".to_string()];
        let result = extractor.extract(content, &schema);
        let obj = result.as_object().unwrap();
        let emails = obj["email"].as_array().unwrap();
        assert!(emails.len() >= 2);
    }
}
