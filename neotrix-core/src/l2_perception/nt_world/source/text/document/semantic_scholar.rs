use crate::l2_perception::nt_world::source::types::*;
use crate::l2_perception::nt_world::source::engine::MediaSource;

pub struct SemanticScholarSource;

impl SemanticScholarSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for SemanticScholarSource {
    fn id(&self) -> &str { "semantic_scholar" }
    fn name(&self) -> &str { "Semantic Scholar" }
    fn media_type(&self) -> MediaType { MediaType::Document }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let q = query.to_string();
        Box::pin(async move {
            let offset = (page - 1) * 20;
            let url = format!("https://api.semanticscholar.org/graph/v1/paper/search?query={}&offset={}&limit=20&fields=title,authors,abstract,url,year,venue", urlencoding::encode(&q), offset);
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let papers = json["data"].as_array().ok_or("No results")?;
            let data: Vec<MediaItem> = papers.iter().filter_map(|p| {
                let id = p["paperId"].as_str()?.to_string();
                let title = p["title"].as_str()?.to_string();
                let author = p["authors"][0]["name"].as_str().unwrap_or("Unknown").to_string();
                let abstract_text = p["abstract"].as_str().unwrap_or("").to_string();
                let year = p["year"].as_i64().map(|y| y.to_string()).unwrap_or_default();
                let venue = p["venue"].as_str().unwrap_or("").to_string();
                let link = p["url"].as_str().map(String::from);
                Some(MediaItem {
                    id,
                    title: format!("{} ({})", title, year),
                    artist: author,
                    album: format!("{} | {}", venue, abstract_text),
                    duration: None,
                    cover_url: link,
                    media_type: MediaType::Document,
                    qualities: vec![Quality::Standard],
                })
            }).collect();
            let total = json["total"].as_u64().unwrap_or(0) as usize;
            Ok(SearchResult { data, total, source: "semantic_scholar".into(), page })
        })
    }
}
