use crate::core::nt_core_time::unix_now;

// Types re-exported from core layer to break circular imports
pub use crate::core::nt_core_data_types::{
    DataSourceRecord, DataSourceType, ExternalDataConnector,
};

impl ExternalDataConnector {
    pub fn collect_all() -> Vec<DataSourceRecord> {
        let mut records = Vec::new();
        if let Ok(r) = Self::fetch_hackernews() {
            records.extend(r);
        }
        if let Ok(r) = Self::fetch_arxiv_latest() {
            records.extend(r);
        }
        if let Ok(r) = Self::fetch_github_trending() {
            records.extend(r);
        }
        if let Ok(r) = Self::fetch_semantic_scholar() {
            records.extend(r);
        }
        records
    }

    pub fn collect_from(source: DataSourceType) -> Vec<DataSourceRecord> {
        match source {
            DataSourceType::HackerNews => Self::fetch_hackernews().unwrap_or_default(),
            DataSourceType::ArXiv => Self::fetch_arxiv_latest().unwrap_or_default(),
            DataSourceType::GitHubTrending => Self::fetch_github_trending().unwrap_or_default(),
            DataSourceType::Wikipedia => Self::fetch_wikipedia_random().unwrap_or_default(),
            DataSourceType::OpenLibrary => Self::fetch_openlibrary_trending().unwrap_or_default(),
            DataSourceType::NewsRss => Self::fetch_news_rss().unwrap_or_default(),
            DataSourceType::SemanticScholar => Self::fetch_semantic_scholar().unwrap_or_default(),
            DataSourceType::YouTube => Vec::new(),
            DataSourceType::Unsplash => Vec::new(),
            DataSourceType::Twitch => Vec::new(),
            DataSourceType::TrendShift => Vec::new(),
            DataSourceType::TikTok => Vec::new(),
            DataSourceType::Spotify => Vec::new(),
            DataSourceType::Pinterest => Vec::new(),
            DataSourceType::Netflix => Vec::new(),
            DataSourceType::Imdb => Vec::new(),
            DataSourceType::Dribbble => Vec::new(),
            DataSourceType::AppleMusic => Vec::new(),
        }
    }

    fn http_client() -> &'static reqwest::blocking::Client {
        static CLIENT: std::sync::OnceLock<reqwest::blocking::Client> = std::sync::OnceLock::new();
        CLIENT.get_or_init(|| {
            reqwest::blocking::Client::builder()
                .user_agent("NeoTrix/0.19 (DataConnector)")
                .timeout(std::time::Duration::from_secs(15))
                .no_proxy()
                .build()
                .expect("HTTP client")
        })
    }

    fn fetch_hackernews() -> Result<Vec<DataSourceRecord>, String> {
        let client = Self::http_client();
        let resp = client
            .get("https://hacker-news.firebaseio.com/v0/topstories.json")
            .send()
            .map_err(|e| format!("HN fetch error: {}", e))?;
        let ids: Vec<u64> = resp.json().map_err(|e| format!("HN JSON error: {}", e))?;
        let mut records = Vec::new();
        for id in ids.iter().take(15) {
            if let Ok(item_resp) = client
                .get(&format!(
                    "https://hacker-news.firebaseio.com/v0/item/{}.json",
                    id
                ))
                .send()
            {
                if let Ok(item) = item_resp.json::<serde_json::Value>() {
                    let title = item["title"].as_str().unwrap_or("").to_string();
                    let url = item["url"].as_str().unwrap_or("").to_string();
                    let score = item["score"].as_f64().unwrap_or(0.0);
                    if !title.is_empty() {
                        records.push(DataSourceRecord {
                            title,
                            summary: String::new(),
                            url,
                            source_type: DataSourceType::HackerNews,
                            topics: vec!["technology".to_string(), "trending".to_string()],
                            score,
                            timestamp: unix_now(),
                        });
                    }
                }
            }
        }
        Ok(records)
    }

    fn fetch_arxiv_latest() -> Result<Vec<DataSourceRecord>, String> {
        let client = Self::http_client();
        let resp = client
            .get("https://export.arxiv.org/api/query?search_query=cat:cs.AI&sortBy=submittedDate&sortOrder=descending&max_results=20")
            .send()
            .map_err(|e| format!("arXiv fetch error: {}", e))?;
        let text = resp
            .text()
            .map_err(|e| format!("arXiv text error: {}", e))?;
        let mut records = Vec::new();
        let mut pos = 0;
        while let Some(entry_start) = text[pos..].find("<entry>") {
            let start = pos + entry_start;
            let end = text[start..]
                .find("</entry>")
                .map(|e| start + e + 8)
                .unwrap_or(text.len());
            let entry = &text[start..end];
            let title = Self::extract_xml(entry, "title").unwrap_or_default();
            let summary = Self::extract_xml(entry, "summary").unwrap_or_default();
            let id = Self::extract_xml(entry, "id").unwrap_or_default();
            if !title.is_empty() {
                records.push(DataSourceRecord {
                    title: title.trim().to_string(),
                    summary: summary.trim().chars().take(500).collect(),
                    url: id.trim().to_string(),
                    source_type: DataSourceType::ArXiv,
                    topics: vec!["ai".to_string(), "research".to_string()],
                    score: 1.0,
                    timestamp: unix_now(),
                });
            }
            pos = end;
        }
        Ok(records)
    }

    fn fetch_github_trending() -> Result<Vec<DataSourceRecord>, String> {
        let client = Self::http_client();

        let mut records = Vec::new();

        records.extend(super::github_trending::GitHubTrending::into_feed_records());

        let queries = [
            ("stars:>1000+pushed:>2026-01-01", 15u32),
            ("stars:>500+created:>2026-05-01", 10),
            ("stars:>100+created:>2026-06-01", 10),
        ];

        for (query, per_page) in queries {
            let url = format!(
                "https://api.github.com/search/repositories?q={}&sort=stars&order=desc&per_page={}",
                query, per_page
            );
            if let Ok(resp) = client
                .get(&url)
                .header("Accept", "application/vnd.github.v3+json")
                .send()
            {
                if let Ok(data) = resp.json::<serde_json::Value>() {
                    if let Some(items) = data["items"].as_array() {
                        for item in items {
                            let full_name = item["full_name"].as_str().unwrap_or("").to_string();
                            let description =
                                item["description"].as_str().unwrap_or("").to_string();
                            let html_url = item["html_url"].as_str().unwrap_or("").to_string();
                            let stars = item["stargazers_count"].as_f64().unwrap_or(0.0);
                            let language = item["language"].as_str().unwrap_or("").to_string();
                            let topics: Vec<String> = item["topics"]
                                .as_array()
                                .map(|a| {
                                    a.iter()
                                        .filter_map(|v| v.as_str().map(String::from))
                                        .collect()
                                })
                                .unwrap_or_default();
                            if !full_name.is_empty()
                                && !records
                                    .iter()
                                    .any(|r: &DataSourceRecord| r.title == full_name)
                            {
                                let mut summary = description;
                                if !language.is_empty() {
                                    summary.push_str(&format!(" [{}]", language));
                                }
                                records.push(DataSourceRecord {
                                    title: full_name,
                                    summary,
                                    url: html_url,
                                    source_type: DataSourceType::GitHubTrending,
                                    topics,
                                    score: stars,
                                    timestamp: unix_now(),
                                });
                            }
                        }
                    }
                }
            }
        }

        records.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        records.truncate(25);
        Ok(records)
    }

    fn fetch_wikipedia_random() -> Result<Vec<DataSourceRecord>, String> {
        let client = Self::http_client();
        let topics = [
            "Artificial_intelligence",
            "Machine_learning",
            "Neuroscience",
            "Quantum_computing",
            "Complex_system",
            "Information_theory",
            "Cognitive_science",
            "Evolutionary_biology",
            "Thermodynamics",
            "Category_theory",
        ];
        let mut records = Vec::new();
        for topic in topics {
            let url = format!(
                "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
                topic
            );
            if let Ok(resp) = client.get(&url).send() {
                if let Ok(data) = resp.json::<serde_json::Value>() {
                    let title = data["title"].as_str().unwrap_or(topic);
                    let extract = data["extract"].as_str().unwrap_or("");
                    records.push(DataSourceRecord {
                        title: title.to_string(),
                        summary: extract.chars().take(500).collect(),
                        url: format!("https://en.wikipedia.org/wiki/{}", topic),
                        source_type: DataSourceType::Wikipedia,
                        topics: vec!["knowledge".to_string(), "reference".to_string()],
                        score: 1.0,
                        timestamp: unix_now(),
                    });
                }
            }
        }
        Ok(records)
    }

    fn fetch_semantic_scholar() -> Result<Vec<DataSourceRecord>, String> {
        let client = Self::http_client();
        let query = "https://api.semanticscholar.org/graph/v1/paper/search?query=artificial+intelligence+reasoning&limit=10&fields=title,url,abstract";
        if let Ok(resp) = client.get(query).send() {
            if let Ok(data) = resp.json::<serde_json::Value>() {
                let papers = data["data"].as_array().ok_or("No papers")?;
                let mut records = Vec::new();
                for paper in papers.iter().take(10) {
                    let title = paper["title"].as_str().unwrap_or("").to_string();
                    let url = paper["url"].as_str().unwrap_or("").to_string();
                    let abstract_text = paper["abstract"].as_str().unwrap_or("").to_string();
                    if !title.is_empty() {
                        records.push(DataSourceRecord {
                            title,
                            summary: abstract_text.chars().take(500).collect(),
                            url,
                            source_type: DataSourceType::SemanticScholar,
                            topics: vec!["research".to_string(), "ai".to_string()],
                            score: 1.0,
                            timestamp: unix_now(),
                        });
                    }
                }
                return Ok(records);
            }
        }
        Ok(Vec::new())
    }

    fn fetch_openlibrary_trending() -> Result<Vec<DataSourceRecord>, String> {
        Ok(Vec::new())
    }

    fn fetch_news_rss() -> Result<Vec<DataSourceRecord>, String> {
        Ok(Vec::new())
    }

    fn extract_xml(xml: &str, tag: &str) -> Option<String> {
        let open = format!("<{}>", tag);
        let close = format!("</{}>", tag);
        xml.find(&open).and_then(|s| {
            let start = s + open.len();
            xml[start..]
                .find(&close)
                .map(|e| xml[start..start + e].trim().to_string())
        })
    }

    pub fn ingest_to_kb(
        records: &[DataSourceRecord],
        kb: &crate::neotrix::nt_memory_kb::KnowledgeBase,
    ) -> Result<usize, String> {
        use crate::neotrix::nt_memory_kb::nt_memory_types::{KnowledgeNode, NodeType};
        let mut count = 0;
        for record in records {
            let node_type = match record.source_type {
                DataSourceType::ArXiv | DataSourceType::SemanticScholar => NodeType::Paper,
                DataSourceType::GitHubTrending => NodeType::Repository,
                DataSourceType::Wikipedia => NodeType::Concept,
                DataSourceType::OpenLibrary => NodeType::Book,
                DataSourceType::HackerNews | DataSourceType::NewsRss => NodeType::Article,
                DataSourceType::YouTube => NodeType::Source,
                DataSourceType::Unsplash => NodeType::Image,
                DataSourceType::Twitch => NodeType::Source,
                DataSourceType::TrendShift => NodeType::Article,
                DataSourceType::TikTok => NodeType::Source,
                DataSourceType::Spotify => NodeType::Source,
                DataSourceType::Pinterest => NodeType::Image,
                DataSourceType::Netflix => NodeType::Source,
                DataSourceType::Imdb => NodeType::Source,
                DataSourceType::Dribbble => NodeType::Image,
                DataSourceType::AppleMusic => NodeType::Source,
            };
            let node = KnowledgeNode {
                id: String::new(),
                title: record.title.clone(),
                node_type,
                summary: Some(record.summary.clone()),
                content: None,
                url: Some(record.url.clone()),
                domain: Some(record.source_type.name().to_string()),
                language: "en".to_string(),
                confidence: record.score,
                importance: 1.0,
                created_at: record.timestamp,
                updated_at: record.timestamp,
                access_count: 0,
                metadata: None,
                version: 1,
                superseded_by: None,
            };
            if kb.insert_node(&node).is_ok() {
                count += 1;
            }
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_xml() {
        let xml = "<root><title>Test Title</title></root>";
        assert_eq!(
            ExternalDataConnector::extract_xml(xml, "title"),
            Some("Test Title".to_string())
        );
    }

    #[test]
    fn test_extract_xml_not_found() {
        let xml = "<root><other>content</other></root>";
        assert_eq!(ExternalDataConnector::extract_xml(xml, "title"), None);
    }

    #[test]
    fn test_data_source_type_names() {
        assert_eq!(DataSourceType::HackerNews.name(), "hackernews");
        assert_eq!(DataSourceType::ArXiv.name(), "arxiv");
        assert_eq!(DataSourceType::GitHubTrending.name(), "github_trending");
        assert_eq!(DataSourceType::Wikipedia.name(), "wikipedia");
        assert_eq!(DataSourceType::OpenLibrary.name(), "openlibrary");
        assert_eq!(DataSourceType::NewsRss.name(), "news_rss");
    }

    #[test]
    fn test_data_source_record_creation() {
        let record = DataSourceRecord {
            title: "Test".to_string(),
            summary: "A test record".to_string(),
            url: "https://example.com".to_string(),
            source_type: DataSourceType::Wikipedia,
            topics: vec!["test".to_string()],
            score: 1.0,
            timestamp: 1000,
        };
        assert_eq!(record.title, "Test");
        assert_eq!(record.source_type.name(), "wikipedia");
    }
}
