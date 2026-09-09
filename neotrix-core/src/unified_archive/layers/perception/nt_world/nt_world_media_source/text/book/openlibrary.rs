use crate::unified::layers::perception::nt_world::nt_world_media_source::types::*;
use crate::unified::layers::perception::nt_world::nt_world_media_source::engine::MediaSource;

pub struct OpenLibrarySource;

impl OpenLibrarySource {
    pub fn new() -> Self { Self }
}

impl MediaSource for OpenLibrarySource {
    fn id(&self) -> &str { "openlibrary" }
    fn name(&self) -> &str { "Open Library" }
    fn media_type(&self) -> MediaType { MediaType::Book }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let q = query.to_string();
        Box::pin(async move {
            let offset = (page - 1) * 20;
            let url = format!("https://openlibrary.org/search.json?q={}&offset={}&limit=20", urlencoding::encode(&q), offset);
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let docs = json["docs"].as_array().ok_or("No results")?;
            let data: Vec<MediaItem> = docs.iter().filter_map(|d| {
                let key = d["key"].as_str()?.to_string();
                let title = d["title"].as_str()?.to_string();
                let author = d["author_name"][0].as_str().unwrap_or("Unknown").to_string();
                let cover_id = d["cover_i"].as_i64();
                let cover_url = cover_id.map(|id| format!("https://covers.openlibrary.org/b/id/{}-L.jpg", id));
                let first_publish = d["first_publish_year"].as_i64().map(|y| y.to_string()).unwrap_or_default();
                Some(MediaItem {
                    id: key,
                    title: format!("{} ({})", title, first_publish),
                    artist: author,
                    album: d["publisher"][0].as_str().unwrap_or("").to_string(),
                    duration: None,
                    cover_url,
                    media_type: MediaType::Book,
                    qualities: vec![Quality::Standard],
                })
            }).collect();
            let total = json["numFound"].as_u64().unwrap_or(0) as usize;
            Ok(SearchResult { data, total, source: "openlibrary".into(), page })
        })
    }
}
