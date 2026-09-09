use crate::l2_perception::nt_world::source::types::*;
use crate::l2_perception::nt_world::source::engine::MediaSource;

pub struct SoundCloudSource;

impl SoundCloudSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for SoundCloudSource {
    fn id(&self) -> &str { "soundcloud" }
    fn name(&self) -> &str { "SoundCloud" }
    fn media_type(&self) -> MediaType { MediaType::Audio }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let query = query.to_string();
        Box::pin(async move {
            let offset = (page - 1) * 20;
            let url = format!("https://api-v2.soundcloud.com/search/tracks?q={}&client_id=&limit=20&offset={}", urlencoding::encode(&query), offset);
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let list = json["collection"].as_array().ok_or("No tracks")?;
            let data: Vec<MediaItem> = list.iter().filter_map(|s| {
                let id = s["id"].as_i64()?.to_string();
                let title = s["title"].as_str()?.to_string();
                let artist = s["user"]["username"].as_str()?.unwrap_or("Unknown").to_string();
                let duration = s["duration"].as_i64().map(|ms| std::time::Duration::from_millis(ms as u64));
                Some(MediaItem { id, title, artist, album: String::new(), duration, cover_url: None, media_type: MediaType::Audio, qualities: vec![Quality::High, Quality::Standard] })
            }).collect();
            let total = json["total_results"].as_u64().unwrap_or(0) as usize;
            Ok(SearchResult { data, total, source: "soundcloud".into(), page })
        })
    }

    fn play_url(&self, item: &MediaItem, quality: Quality) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ViewSource, String>> + Send>> {
        let id = item.id.clone();
        Box::pin(async move {
            let url = format!("https://api.soundcloud.com/tracks/{}/stream?client_id=", id);
            Ok(ViewSource { url, quality, format: "mp3".into(), bitrate: quality.bitrate(), size: 0, source: "soundcloud".into() })
        })
    }
}
