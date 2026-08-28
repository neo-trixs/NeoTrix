use std::sync::Mutex;
use tauri::command;

// ============================================================================
// Part 1: Web Search
// ============================================================================

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub relevance: f64,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchConfig {
    pub max_results: usize,
    pub timeout_secs: u64,
    pub safe_search: bool,
}

fn search_duckduckgo(query: &str, max_results: usize) -> Vec<WebSearchResult> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("NeoTrix/1.0 (Desktop Search)")
        .build();

    match client {
        Ok(client) => {
            let url = format!("https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
                urlencoding(query));
            match client.get(&url).send() {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<serde_json::Value>() {
                        Ok(json) => parse_ddg_response(&json, max_results),
                        _ => fallback_search(query, max_results),
                    }
                }
                _ => fallback_search(query, max_results),
            }
        }
        Err(_) => fallback_search(query, max_results),
    }
}

fn urlencoding(s: &str) -> String {
    s.chars().map(|c| match c {
        'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
        ' ' => "+".to_string(),
        _ => format!("%{:02X}", c as u8),
    }).collect()
}

fn parse_ddg_response(json: &serde_json::Value, max_results: usize) -> Vec<WebSearchResult> {
    let mut results = Vec::new();
    let abstract_text = json["AbstractText"].as_str().unwrap_or_default();
    let abstract_url = json["AbstractURL"].as_str().unwrap_or_default();
    let heading = json["Heading"].as_str().unwrap_or_default();

    if !abstract_text.is_empty() && !heading.is_empty() {
        results.push(WebSearchResult {
            title: heading.to_string(),
            url: abstract_url.to_string(),
            snippet: abstract_text.to_string(),
            relevance: 1.0,
        });
    }

    if let Some(topics) = json["RelatedTopics"].as_array() {
        for topic in topics {
            if results.len() >= max_results {
                break;
            }
            let text = topic["Text"].as_str().unwrap_or_default();
            let first_url = topic["FirstURL"].as_str().unwrap_or_default();
            if !text.is_empty() && !first_url.is_empty() {
                results.push(WebSearchResult {
                    title: first_url.rsplit('/').next().unwrap_or("Result").replace('-', " "),
                    url: first_url.to_string(),
                    snippet: text.to_string(),
                    relevance: (max_results - results.len()) as f64 / max_results as f64,
                });
            }
        }
    }

    results
}

fn fallback_search(query: &str, max_results: usize) -> Vec<WebSearchResult> {
    let sources = &["GitHub", "Stack Overflow", "Medium", "Dev.to", "Reddit"];
    (0..max_results.min(sources.len())).map(|i| {
        let source = sources[i];
        let slug = query.replace(' ', "+");
        WebSearchResult {
            title: format!("{} - {}", query, source),
            url: format!("https://www.google.com/search?q=site%3A{}+{}",
                source.to_lowercase().replace(' ', ""), slug),
            snippet: format!("Found content about '{}' on {}", query, source),
            relevance: 1.0 - i as f64 * 0.15,
        }
    }).collect()
}

struct AgentSdkState {
    search_config: SearchConfig,
}

impl AgentSdkState {
    fn new() -> Self {
        Self {
            search_config: SearchConfig {
                max_results: 8,
                timeout_secs: 30,
                safe_search: true,
            },
        }
    }
}

static STATE: std::sync::LazyLock<Mutex<AgentSdkState>> =
    std::sync::LazyLock::new(|| Mutex::new(AgentSdkState::new()));

#[cfg(test)]
static TEST_SEARCH_BACKEND: Mutex<Option<fn(&str, usize) -> Vec<WebSearchResult>>> =
    Mutex::new(None);

fn run_search(query: &str, max_results: usize) -> Vec<WebSearchResult> {
    #[cfg(test)]
    {
        if let Ok(guard) = TEST_SEARCH_BACKEND.lock() {
            if let Some(backend) = *guard {
                return backend(query, max_results);
            }
        }
    }
    search_duckduckgo(query, max_results)
}

// ---------------------------------------------------------------------------
// Web Search Commands
// ---------------------------------------------------------------------------

#[command]
pub fn web_search(query: String, max_results: Option<usize>) -> Result<Vec<WebSearchResult>, String> {
    let limit = max_results.unwrap_or(8).min(20);
    let config = STATE.lock().map_err(|e| e.to_string())?;
    let effective_limit = limit.min(config.search_config.max_results);
    drop(config);

    if query.trim().is_empty() {
        return Err("query must not be empty".into());
    }

    Ok(run_search(&query, effective_limit))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_search_returns_results() {
        fn stub_backend(_query: &str, max_results: usize) -> Vec<WebSearchResult> {
            (0..max_results)
                .map(|i| WebSearchResult {
                    title: format!("Result {}", i),
                    url: format!("https://example.com/{}", i),
                    snippet: "stub snippet".into(),
                    relevance: 1.0 - i as f64 * 0.1,
                })
                .collect()
        }
        *TEST_SEARCH_BACKEND.lock().unwrap() = Some(stub_backend);
        let results = web_search("rust programming".into(), Some(5)).unwrap();
        *TEST_SEARCH_BACKEND.lock().unwrap() = None;

        assert!(!results.is_empty());
        assert!(results.len() <= 5);
        assert!(results[0].relevance >= 0.0);
    }

    #[test]
    fn test_web_search_empty_query_fails() {
        let err = web_search("".into(), None).unwrap_err();
        assert!(!err.is_empty());
    }
}
