use serde::{Deserialize, Serialize};

/// A reusable web search tool wrapping WebSearchEngine.
/// Provides structured-text results suitable for LLM context injection.
pub struct WebSearchTool {
    engine: WebSearchEngine,
}

impl WebSearchTool {
    pub fn new() -> Self {
        Self { engine: WebSearchEngine::default() }
    }

    /// Execute a web search and return results as formatted text.
    pub fn search(&self, query: &str, count: usize) -> Result<String, String> {
        let results = self.engine.search(query, count)?;
        if results.is_empty() {
            return Ok("No web search results found.".to_string());
        }
        let mut msg = format!("Web search results for \"{}\":\n\n", query);
        for (i, r) in results.iter().enumerate() {
            msg.push_str(&format!("{}. {}\n   URL: {}\n   {}\n\n", i + 1, r.title, r.url, r.snippet));
        }
        Ok(msg.trim().to_string())
    }
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct DuckDuckGoResponse {
    #[serde(default)]
    AbstractText: String,
    #[serde(default)]
    AbstractSource: String,
    #[serde(default)]
    AbstractURL: String,
    #[serde(default)]
    Results: Vec<DuckDuckGoItem>,
    #[serde(default)]
    RelatedTopics: Vec<DuckDuckGoTopic>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct DuckDuckGoItem {
    #[serde(default)]
    Text: String,
    #[serde(default)]
    FirstURL: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(non_snake_case, dead_code)]
enum DuckDuckGoTopic {
    Leaf {
        #[serde(default)]
        Text: String,
        #[serde(default)]
        FirstURL: String,
    },
    Category {
        #[serde(default)]
        Name: String,
        #[serde(default)]
        Topics: Vec<DuckDuckGoItem>,
    },
}

pub struct WebSearchEngine {
    api_base_url: String,
    client: reqwest::blocking::Client,
}

impl Default for WebSearchEngine {
    fn default() -> Self {
        let base = std::env::var("NEOTRIX_SEARCH_API")
            .unwrap_or_else(|_| "https://api.duckduckgo.com".to_string());
        Self::new(&base)
    }
}

impl WebSearchEngine {
    pub fn new(api_base_url: &str) -> Self {
        Self {
            api_base_url: api_base_url.trim_end_matches('/').to_string(),
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn search(&self, query: &str, count: usize) -> Result<Vec<SearchResult>, String> {
        let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
        let url = format!(
            "{}/?q={}&format=json&no_html=1&skip_disambig=1",
            self.api_base_url, encoded
        );

        let resp = self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("API returned status: {}", resp.status()));
        }

        let ddg: DuckDuckGoResponse = resp
            .json()
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let mut results = Vec::new();

        if !ddg.AbstractText.is_empty() {
            let url = if ddg.AbstractURL.is_empty() {
                format!("https://en.wikipedia.org/wiki/{}", query.replace(' ', "_"))
            } else {
                ddg.AbstractURL.clone()
            };
            results.push(SearchResult {
                title: if ddg.AbstractSource.is_empty() {
                    query.to_string()
                } else {
                    format!("{} — {}", query, ddg.AbstractSource)
                },
                url,
                snippet: ddg.AbstractText,
            });
        }

        for item in &ddg.Results {
            if results.len() >= count {
                break;
            }
            let title = item.Text.split(" — ").next().unwrap_or(&item.Text).to_string();
            results.push(SearchResult {
                title,
                url: item.FirstURL.clone(),
                snippet: item.Text.clone(),
            });
        }

        for topic in &ddg.RelatedTopics {
            if results.len() >= count {
                break;
            }
            match topic {
                DuckDuckGoTopic::Leaf { Text, FirstURL } => {
                    let title = Text.split(" — ").next().unwrap_or(Text).to_string();
                    results.push(SearchResult {
                        title,
                        url: FirstURL.clone(),
                        snippet: Text.clone(),
                    });
                }
                DuckDuckGoTopic::Category { Topics, .. } => {
                    for item in Topics {
                        if results.len() >= count {
                            break;
                        }
                        let title = item.Text.split(" — ").next().unwrap_or(&item.Text).to_string();
                        results.push(SearchResult {
                            title,
                            url: item.FirstURL.clone(),
                            snippet: item.Text.clone(),
                        });
                    }
                }
            }
        }

        Ok(results)
    }
}
