use crate::l2_perception::nt_world::source::types::*;
use crate::l2_perception::nt_world::source::engine::MediaSource;

pub struct AnnasArchiveSource;

impl AnnasArchiveSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for AnnasArchiveSource {
    fn id(&self) -> &str { "annas_archive" }
    fn name(&self) -> &str { "Anna's Archive" }
    fn media_type(&self) -> MediaType { MediaType::Book }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let q = query.to_string();
        Box::pin(async move {
            let url = format!("https://annas-archive.org/api/v1/search?q={}&page={}", urlencoding::encode(&q), page);
            let resp = reqwest::Client::new()
                .get(&url)
                .header("User-Agent", "NeoTrix/1.0")
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let docs = json["docs"].as_array().ok_or("No results")?;
            let data: Vec<MediaItem> = docs.iter().filter_map(|d| {
                let id = d["id"].as_str()?.to_string();
                let title = d["title"].as_str()?.to_string();
                let author = d["author"].as_str().unwrap_or("Unknown").to_string();
                let cover = d["cover_url"].as_str().map(String::from);
                let file_size = d["file_size"].as_i64().map(|s| format!("{} MB", s / 1048576)).unwrap_or_default();
                Some(MediaItem {
                    id,
                    title,
                    artist: author,
                    album: file_size,
                    duration: None,
                    cover_url: cover,
                    media_type: MediaType::Book,
                    qualities: vec![Quality::Standard],
                })
            }).collect();
            let total = json["total"].as_u64().unwrap_or(0) as usize;
            Ok(SearchResult { data, total, source: "annas_archive".into(), page })
        })
    }
}
