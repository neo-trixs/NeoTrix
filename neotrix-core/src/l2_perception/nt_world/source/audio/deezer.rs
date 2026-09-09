use crate::l2_perception::nt_world::source::types::*;
use crate::l2_perception::nt_world::source::engine::MediaSource;

pub struct DeezerSource;

impl DeezerSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for DeezerSource {
    fn id(&self) -> &str { "deezer" }
    fn name(&self) -> &str { "Deezer" }
    fn media_type(&self) -> MediaType { MediaType::Audio }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let query = query.to_string();
        Box::pin(async move {
            let url = format!("https://api.deezer.com/search?q={}&limit=30&index={}", urlencoding::encode(&query), (page - 1) * 30);
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let list = json["data"].as_array().ok_or("No tracks")?;
            let data: Vec<MediaItem> = list.iter().filter_map(|s| {
                let id = s["id"].as_i64()?.to_string();
                let title = s["title"].as_str()?.to_string();
                let artist = s["artist"]["name"].as_str().map_or("Unknown", |s| s).to_string();
                let album = s["album"]["title"].as_str().map_or("", |s| s).to_string();
                let duration = s["duration"].as_i64().map(|s| std::time::Duration::from_secs(s as u64));
                let cover_url = s["album"]["cover_medium"].as_str().map(String::from);
                Some(MediaItem { id, title, artist, album, duration, cover_url, media_type: MediaType::Audio, qualities: vec![Quality::Flac, Quality::High, Quality::Standard] })
            }).collect();
            let total = json["total"].as_u64().unwrap_or(0) as usize;
            Ok(SearchResult { data, total, source: "deezer".into(), page })
        })
    }

    fn play_url(&self, item: &MediaItem, quality: Quality) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ViewSource, String>> + Send>> {
        let id = item.id.clone();
        Box::pin(async move {
            let url = format!("https://api.deezer.com/track/{}", id);
            Ok(ViewSource { url, quality, format: "mp3".into(), bitrate: quality.bitrate(), size: 0, source: "deezer".into() })
        })
    }
}
