use crate::l2_perception::nt_world::source::types::*;
use crate::l2_perception::nt_world::source::engine::MediaSource;

pub struct PipedSource;

impl PipedSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for PipedSource {
    fn id(&self) -> &str { "piped" }
    fn name(&self) -> &str { "Piped" }
    fn media_type(&self) -> MediaType { MediaType::Audio }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let query = query.to_string();
        Box::pin(async move {
            let url = format!("https://pipedapi.kavin.rocks/search?q={}&filter=music_songs", urlencoding::encode(&query));
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let list = json["items"].as_array().ok_or("No items")?;
            let start = ((page - 1) * 20) as usize;
            let end = (start + 20).min(list.len());
            let data: Vec<MediaItem> = list[start..end].iter().filter_map(|s| {
                let id = s["url"].as_str()?.trim_start_matches("/watch?v=").to_string();
                let title = s["title"].as_str()?.to_string();
                let artist = s["uploaderName"].as_str()?.unwrap_or("Unknown").to_string();
                let duration = s["duration"].as_i64().map(|s| std::time::Duration::from_secs(s as u64));
                Some(MediaItem { id, title, artist, album: String::new(), duration, cover_url: None, media_type: MediaType::Audio, qualities: vec![Quality::High, Quality::Standard] })
            }).collect();
            let total = list.len();
            Ok(SearchResult { data, total, source: "piped".into(), page })
        })
    }

    fn play_url(&self, item: &MediaItem, quality: Quality) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ViewSource, String>> + Send>> {
        let id = item.id.clone();
        Box::pin(async move {
            let url = format!("https://pipedapi.kavin.rocks/streams/{}", id);
            Ok(ViewSource { url, quality, format: "mp3".into(), bitrate: quality.bitrate(), size: 0, source: "piped".into() })
        })
    }
}
