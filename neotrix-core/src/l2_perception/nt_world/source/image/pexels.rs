use crate::l2_perception::nt_world::source::types::*;
use crate::l2_perception::nt_world::source::engine::MediaSource;

pub struct PexelsSource;

impl PexelsSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for PexelsSource {
    fn id(&self) -> &str { "pexels" }
    fn name(&self) -> &str { "Pexels" }
    fn media_type(&self) -> MediaType { MediaType::Image }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let q = query.to_string();
        Box::pin(async move {
            let url = format!("https://api.pexels.com/v1/search?query={}&page={}&per_page=20", urlencoding::encode(&q), page);
            let resp = reqwest::Client::new()
                .get(&url)
                .header("Authorization", std::env::var("PEXELS_API_KEY").unwrap_or_default())
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let photos = json["photos"].as_array().ok_or("No results")?;
            let data: Vec<MediaItem> = photos.iter().filter_map(|p| {
                let id = p["id"].as_i64()?.to_string();
                let alt = p["alt"].as_str().unwrap_or("Untitled").to_string();
                let photographer = p["photographer"].as_str()?.to_string();
                let src = p["src"]["large"].as_str().map(String::from);
                Some(MediaItem {
                    id,
                    title: alt,
                    artist: photographer,
                    album: String::new(),
                    duration: None,
                    cover_url: src,
                    media_type: MediaType::Image,
                    qualities: vec![Quality::Standard],
                })
            }).collect();
            let total = json["total_results"].as_u64().unwrap_or(0) as usize;
            Ok(SearchResult { data, total, source: "pexels".into(), page })
        })
    }
}
