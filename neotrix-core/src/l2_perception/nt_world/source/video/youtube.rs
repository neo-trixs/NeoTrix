use crate::l2_perception::nt_world::source::types::*;
use crate::l2_perception::nt_world::source::engine::MediaSource;

pub struct YouTubeSource;

impl YouTubeSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for YouTubeSource {
    fn id(&self) -> &str { "youtube" }
    fn name(&self) -> &str { "YouTube" }
    fn media_type(&self) -> MediaType { MediaType::Video }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let q = query.to_string();
        Box::pin(async move {
            let url = format!("https://vid.puffyan.us/api/v1/search?q={}&type=video&page={}", urlencoding::encode(&q), page);
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let items = json.as_array().ok_or("No results")?;
            let data: Vec<MediaItem> = items.iter().filter_map(|v| {
                Some(MediaItem {
                    id: v["videoId"].as_str()?.to_string(),
                    title: v["title"].as_str()?.to_string(),
                    artist: v["author"].as_str().map_or("Unknown", |s| s).to_string(),
                    album: String::new(),
                    duration: v["lengthSeconds"].as_i64().map(|s| std::time::Duration::from_secs(s as u64)),
                    cover_url: v["videoThumbnails"][0]["url"].as_str().map(String::from),
                    media_type: MediaType::Video,
                    qualities: vec![Quality::Standard],
                })
            }).collect();
            Ok(SearchResult { data, total: items.len(), source: "youtube".into(), page })
        })
    }
}
