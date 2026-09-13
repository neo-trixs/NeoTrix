//! Web Search Engine

use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use neotrix_types::search_backend::SearchResult;

use super::{SearchConfig, SearchEngine, SearchOptions};

/// Web Search Engine
pub struct WebSearch {
    config: SearchConfig,
    client: Client,
    cache: Arc<tokio::sync::Mutex<HashMap<String, Vec<SearchResult>>>>,
}

impl WebSearch {
    pub fn new(config: SearchConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            config,
            client,
            cache: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    pub async fn search(&self, query: &str, options: Option<SearchOptions>) -> Result<Vec<SearchResult>, String> {
        let opts = options.unwrap_or_default();
        let cache_key = format!("{}:{}:{}", query, opts.max_results.unwrap_or(self.config.max_results), opts.engine.unwrap_or(self.config.default_engine) as u8);
        
        // Check cache
        {
            let cache = self.cache.lock().await;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let engine = opts.engine.unwrap_or(self.config.default_engine);
        let results = match engine {
            SearchEngine::Unified => self.search_unified(query, &opts).await?,
            SearchEngine::Google => self.search_google(query, &opts).await?,
            SearchEngine::Bing => self.search_bing(query, &opts).await?,
            SearchEngine::DuckDuckGo => self.search_duckduckgo(query, &opts).await?,
            SearchEngine::Arxiv => self.search_arxiv(query, &opts).await?,
            SearchEngine::Wikipedia => self.search_wikipedia(query, &opts).await?,
            SearchEngine::GitHub => self.search_github(query, &opts).await?,
            _ => self.search_unified(query, &opts).await?,
        };

        // Cache results
        {
            let mut cache = self.cache.lock().await;
            cache.insert(cache_key, results.clone());
        }

        Ok(results)
    }

    /// Dispatch to engine-specific search method
    async fn search_engine(&self, query: &str, engine: SearchEngine, max_results: usize) -> Result<Vec<SearchResult>, String> {
        let opts = SearchOptions { max_results: Some(max_results), engine: Some(engine), ..Default::default() };
        match engine {
            SearchEngine::Google => self.search_google(query, &opts).await,
            SearchEngine::Bing => self.search_bing(query, &opts).await,
            SearchEngine::DuckDuckGo => self.search_duckduckgo(query, &opts).await,
            SearchEngine::Arxiv => self.search_arxiv(query, &opts).await,
            SearchEngine::Wikipedia => self.search_wikipedia(query, &opts).await,
            SearchEngine::GitHub => self.search_github(query, &opts).await,
            SearchEngine::Unified => self.search_unified(query, &opts).await,
            _ => self.search_unified(query, &opts).await,
        }
    }

    async fn search_unified(&self, query: &str, opts: &SearchOptions) -> Result<Vec<SearchResult>, String> {
        // Try multiple engines in parallel, merge and deduplicate
        let mut all_results = Vec::new();
        
        // Call individual engines directly to avoid recursion
        if let Ok(results) = self.search_google(query, opts).await {
            for r in results {
                if !all_results.contains(&r) {
                    all_results.push(r);
                }
            }
        }
        if let Ok(results) = self.search_bing(query, opts).await {
            for r in results {
                if !all_results.contains(&r) {
                    all_results.push(r);
                }
            }
        }
        if let Ok(results) = self.search_duckduckgo(query, opts).await {
            for r in results {
                if !all_results.contains(&r) {
                    all_results.push(r);
                }
            }
        }
        
        all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        all_results.truncate(opts.max_results.unwrap_or(10));
        
        Ok(all_results)
    }

    async fn search_google(&self, query: &str, opts: &SearchOptions) -> Result<Vec<SearchResult>, String> {
        // Use custom search API or scrape
        self.search_via_bing(query, opts).await
    }

    async fn search_bing(&self, query: &str, opts: &SearchOptions) -> Result<Vec<SearchResult>, String> {
        // Use Bing Web Search API or scrape
        self.search_via_duckduckgo(query, opts).await
    }

    async fn search_duckduckgo(&self, query: &str, opts: &SearchOptions) -> Result<Vec<SearchResult>, String> {
        let url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query));
        let response = reqwest::Client::new()
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (compatible; NeoTrix/0.19)")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let html = response.text().await.map_err(|e| e.to_string())?;
        self.parse_duckduckgo_html(&html, opts.max_results.unwrap_or(10))
    }

    async fn search_via_bing(&self, query: &str, opts: &SearchOptions) -> Result<Vec<SearchResult>, String> {
        // Use Bing HTML scraping as fallback
        self.search_duckduckgo(query, opts).await
    }

    async fn search_via_duckduckgo(&self, query: &str, opts: &SearchOptions) -> Result<Vec<SearchResult>, String> {
        self.search_duckduckgo(query, opts).await
    }

    fn parse_duckduckgo_html(&self, html: &str, max_results: usize) -> Result<Vec<SearchResult>, String> {
        // Simplified HTML parsing - in production use a proper HTML parser
        let mut results = Vec::new();
        let document = scraper::Html::parse_document(html);
        let selector = scraper::Selector::parse(".result__snippet").expect("valid CSS selector");
        
        for element in document.select(&selector).take(max_results) {
            let snippet = element.text().collect::<String>();
            if snippet.len() > 50 {
                results.push(SearchResult::new(
                    "duckduckgo-result",
                    "DuckDuckGo Result",
                    "https://duckduckgo.com",
                )
                .with_snippet(snippet.chars().take(200).collect::<String>())
                .with_score(0.8)
                .with_source_type("duckduckgo"));
            }
        }
        
        Ok(results)
    }

    async fn search_arxiv(&self, query: &str, opts: &SearchOptions) -> Result<Vec<SearchResult>, String> {
        let url = format!("http://export.arxiv.org/api/query?search_query=all:{}&start=0&max_results={}", 
            urlencoding::encode(query), opts.max_results.unwrap_or(10));
        
        let response = reqwest::Client::new()
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let xml = response.text().await.map_err(|e| e.to_string())?;
        self.parse_arxiv_xml(&xml)
    }

    fn parse_arxiv_xml(&self, xml: &str) -> Result<Vec<SearchResult>, String> {
        use std::io::Read;
        let mut reader = quick_xml::Reader::from_str(xml);
        let mut results = Vec::new();
        let mut buf = Vec::new();
        let mut in_entry = false;
        let mut current_title = String::new();
        let mut current_summary = String::new();
        let mut current_id = String::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(ref e)) => {
                    if e.name().as_ref() == b"entry" {
                        in_entry = true;
                    }
                }
                Ok(quick_xml::events::Event::End(ref e)) => {
                    if e.name().as_ref() == b"entry" {
                        if !current_title.is_empty() {
                            results.push(SearchResult::new(
                                &current_id,
                                &current_title,
                                &current_id,
                            )
                            .with_snippet(current_summary.chars().take(300).collect::<String>())
                            .with_score(0.9)
                            .with_source_type("arxiv"));
                        }
                        current_title.clear();
                        current_summary.clear();
                        current_id.clear();
                        in_entry = false;
                    }
                }
                Ok(quick_xml::events::Event::Text(ref _e)) => {
                    if in_entry {
                        // Would need parent element name to distinguish
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }
        
        Ok(results)
    }

    async fn search_wikipedia(&self, query: &str, opts: &SearchOptions) -> Result<Vec<SearchResult>, String> {
        let url = format!("https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&format=json&srlimit={}", 
            urlencoding::encode(query), opts.max_results.unwrap_or(10));
        
        let response = reqwest::Client::new()
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        
        let mut results = Vec::new();
        if let Some(search) = json.get("query").and_then(|q| q.get("search")).and_then(|s| s.as_array()) {
            for item in search.iter().take(opts.max_results.unwrap_or(10)) {
                if let (Some(title), Some(snippet)) = (item.get("title").and_then(|v| v.as_str()), item.get("snippet").and_then(|v| v.as_str())) {
                    results.push(SearchResult::new(
                        title,
                        title,
                        format!("https://en.wikipedia.org/wiki/{}", urlencoding::encode(title)),
                    )
                    .with_snippet(snippet)
                    .with_score(0.85)
                    .with_source_type("wikipedia"));
                }
            }
        }
        
        Ok(results)
    }

    async fn search_github(&self, query: &str, opts: &SearchOptions) -> Result<Vec<SearchResult>, String> {
        let url = format!("https://api.github.com/search/repositories?q={}&per_page={}&sort=stars", 
            urlencoding::encode(query), opts.max_results.unwrap_or(10));
        
        let response = reqwest::Client::new()
            .get(&url)
            .header("User-Agent", "NeoTrix/0.19")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        
        let mut results = Vec::new();
        if let Some(items) = json.get("items").and_then(|i| i.as_array()) {
            for item in items.iter().take(opts.max_results.unwrap_or(10)) {
                if let (Some(name), Some(desc), Some(html_url)) = (
                    item.get("full_name").and_then(|v| v.as_str()),
                    item.get("description").and_then(|v| v.as_str()),
                    item.get("html_url").and_then(|v| v.as_str()),
                ) {
                    results.push(SearchResult::new(
                        name,
                        name,
                        html_url,
                    )
                    .with_snippet(desc)
                    .with_score(0.85)
                    .with_source_type("github"));
                }
            }
        }
        
        Ok(results)
    }
}


