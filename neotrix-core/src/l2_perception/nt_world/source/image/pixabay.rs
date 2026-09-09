use crate::l2_perception::nt_world::source::types::*;
use crate::l2_perception::nt_world::source::engine::MediaSource;

pub struct PixabaySource;

impl PixabaySource {
    pub fn new() -> Self { Self }
}

impl MediaSource for PixabaySource {
    fn id(&self) -> &str { "pixabay" }
    fn name(&self) -> &str { "Pixabay" }
    fn media_type(&self) -> MediaType { MediaType::Image }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let q = query.to_string();
        Box::pin(async move {
            let url = format!("https://pixabay.com/api/?key={}&q={}&page={}&per_page=20", std::env::var("PIXABAY_API_KEY").unwrap_or_default(), urlencoding::encode(&q), page);
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let hits = json["hits"].as_array().ok_or("No results")?;
            let data: Vec<MediaItem> = hits.iter().filter_map(|h| {
                let id = h["id"].as_i64()?.to_string();
                let tags = h["tags"].as_str().unwrap_or("Untitled").to_string();
                let user = h["user"].as_str()?.to_string();
                let src = h["webformatURL"].as_str().map(String::from);
                Some(MediaItem {
                    id,
                    title: tags,
                    artist: user,
                    album: String::new(),
                    duration: None,
                    cover_url: src,
                    media_type: MediaType::Image,
                    qualities: vec![Quality::Standard],
                })
            }).collect();
            let total = json["totalHits"].as_u64().unwrap_or(0) as usize;
            Ok(SearchResult { data, total, source: "pixabay".into(), page })
        })
    }
}
