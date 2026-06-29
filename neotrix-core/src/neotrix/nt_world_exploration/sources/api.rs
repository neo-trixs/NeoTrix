use crate::neotrix::nt_world_exploration::content::{ExplorationSourceType, SourceContent};
use crate::neotrix::nt_world_exploration::source_trait::ExplorationSource;
use std::sync::LazyLock;

static API_CLIENT: LazyLock<reqwest::blocking::Client> = LazyLock::new(|| {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("NeoTrix/0.18")
        .build()
        .unwrap_or_else(|e| {
            log::warn!("API exploration HTTP client init failed: {e}, using default client");
            reqwest::blocking::Client::new()
        })
});

/// API 探索源 — 通过公开 API 注入知识
///
/// 支持: OpenLibrary, GitHub Search, Wikipedia
pub struct ApiSource {
    pub mode: ApiMode,
    query_queue: Vec<String>,
}

pub enum ApiMode {
    OpenLibrary,
    GitHub,
    Wikipedia,
}

impl ApiSource {
    pub fn new(mode: ApiMode) -> Self {
        Self {
            mode,
            query_queue: Vec::new(),
        }
    }

    pub fn enqueue(&mut self, query: impl Into<String>) {
        self.query_queue.push(query.into());
    }

    pub fn enqueue_many(&mut self, queries: Vec<String>) {
        self.query_queue.extend(queries);
    }
}

impl ExplorationSource for ApiSource {
    fn name(&self) -> &'static str {
        match self.mode {
            ApiMode::OpenLibrary => "api_openlibrary",
            ApiMode::GitHub => "api_github",
            ApiMode::Wikipedia => "api_wikipedia",
        }
    }

    fn confidence(&self) -> f64 {
        match self.mode {
            ApiMode::OpenLibrary => 0.8,
            ApiMode::GitHub => 0.9,
            ApiMode::Wikipedia => 0.85,
        }
    }

    fn explore(&mut self) -> Result<Vec<SourceContent>, String> {
        let mut all = Vec::new();
        // Atomic snapshot: drain queue before processing to avoid data loss on panic
        let queries = std::mem::take(&mut self.query_queue);

        for query in queries {
            let encoded = url_encode(&query);
            let results = match self.mode {
                ApiMode::OpenLibrary => {
                    let url = format!("https://openlibrary.org/search.json?q={}&limit=30", encoded);
                    let resp = API_CLIENT.get(&url).send();
                    match resp {
                        Ok(r) if r.status().is_success() => {
                            if let Ok(json) = r.json::<serde_json::Value>() {
                                json["docs"]
                                    .as_array()
                                    .map(|docs| {
                                        docs.iter()
                                            .filter_map(|doc| {
                                                let title = doc["title"].as_str()?;
                                                let key = doc["key"].as_str().unwrap_or("");
                                                let author = doc["author_name"][0]
                                                    .as_str()
                                                    .unwrap_or("Unknown");
                                                Some(
                                                    SourceContent::new(
                                                        key,
                                                        &format!(
                                                            "Author: {} | Title: {}",
                                                            author, title
                                                        ),
                                                        ExplorationSourceType::ApiOpenLibrary,
                                                    )
                                                    .with_title(title)
                                                    .with_author(author)
                                                    .with_url(format!(
                                                        "https://openlibrary.org{}",
                                                        key
                                                    )),
                                                )
                                            })
                                            .collect::<Vec<_>>()
                                    })
                                    .unwrap_or_default()
                            } else {
                                vec![]
                            }
                        }
                        _ => vec![],
                    }
                }
                ApiMode::GitHub => {
                    let url = format!(
                        "https://api.github.com/search/repositories?q={}&per_page=20",
                        encoded
                    );
                    let resp = API_CLIENT.get(&url).send();
                    match resp {
                        Ok(r) if r.status().is_success() => {
                            if let Ok(json) = r.json::<serde_json::Value>() {
                                json["items"]
                                    .as_array()
                                    .map(|items| {
                                        items
                                            .iter()
                                            .filter_map(|item| {
                                                let name = item["full_name"].as_str()?;
                                                let desc =
                                                    item["description"].as_str().unwrap_or("");
                                                let stars =
                                                    item["stargazers_count"].as_u64().unwrap_or(0);
                                                let url = item["html_url"].as_str().unwrap_or("");
                                                Some(
                                                    SourceContent::new(
                                                        name,
                                                        &format!("{} | ⭐{}", desc, stars),
                                                        ExplorationSourceType::ApiGithub,
                                                    )
                                                    .with_title(name)
                                                    .with_url(url),
                                                )
                                            })
                                            .collect::<Vec<_>>()
                                    })
                                    .unwrap_or_default()
                            } else {
                                vec![]
                            }
                        }
                        _ => vec![],
                    }
                }
                ApiMode::Wikipedia => {
                    let url = format!(
                        "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
                        encoded
                    );
                    let resp = API_CLIENT.get(&url).send();
                    match resp {
                        Ok(r) if r.status().is_success() => {
                            if let Ok(json) = r.json::<serde_json::Value>() {
                                let title = json["title"].as_str().unwrap_or(&query);
                                let extract = json["extract"].as_str().unwrap_or("");
                                let page_url = json["content_urls"]["desktop"]["page"]
                                    .as_str()
                                    .unwrap_or("");
                                if !extract.is_empty() {
                                    vec![SourceContent::new(
                                        query.clone(),
                                        extract,
                                        ExplorationSourceType::ApiWikipedia,
                                    )
                                    .with_title(title)
                                    .with_url(page_url)]
                                } else {
                                    vec![]
                                }
                            } else {
                                vec![]
                            }
                        }
                        _ => vec![],
                    }
                }
            };
            all.extend(results);
        }

        Ok(all)
    }

    fn pending_count(&self) -> usize {
        self.query_queue.len()
    }
}

fn url_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("hello world"), "hello+world");
        assert_eq!(url_encode("rust & ai"), "rust+%26+ai");
        assert_eq!(url_encode("normal"), "normal");
    }

    #[test]
    fn test_enqueue_and_drain() {
        let mut src = ApiSource::new(ApiMode::OpenLibrary);
        assert_eq!(src.pending_count(), 0);
        src.enqueue("test query");
        assert_eq!(src.pending_count(), 1);
        let _ = src.explore().unwrap();
        assert_eq!(src.pending_count(), 0);
    }

    #[test]
    fn test_enqueue_many() {
        let mut src = ApiSource::new(ApiMode::GitHub);
        let queries = vec!["rust".into(), "ai".into()];
        src.enqueue_many(queries);
        assert_eq!(src.pending_count(), 2);
    }

    #[test]
    fn test_wikipedia_handles_missing_page() {
        let mut src = ApiSource::new(ApiMode::Wikipedia);
        src.enqueue("this_page_does_not_exist_xyzzy");
        let _ = src.explore().unwrap();
    }
}
