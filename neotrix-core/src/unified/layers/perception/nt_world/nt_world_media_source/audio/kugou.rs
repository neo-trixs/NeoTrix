use crate::unified::layers::perception::nt_world::nt_world_media_source::types::*;
use crate::unified::layers::perception::nt_world::nt_world_media_source::engine::MediaSource;

pub struct KugouSource;

impl KugouSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for KugouSource {
    fn id(&self) -> &str { "kugou" }
    fn name(&self) -> &str { "酷狗音乐" }
    fn media_type(&self) -> MediaType { MediaType::Audio }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let query = query.to_string();
        Box::pin(async move {
            let url = format!("https://mobilecdn.kugou.com/api/v3/search/song?format=json&keyword={}&page={}&pagesize=30", urlencoding::encode(&query), page);
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let list = json["data"]["info"].as_array().ok_or("No songs")?;
            let data: Vec<MediaItem> = list.iter().filter_map(|s| {
                let id = s["hash"].as_str()?.to_string();
                let title = s["songname"].as_str()?.to_string();
                let artist = s["singername"].as_str()?.unwrap_or("Unknown").to_string();
                let album = s["album_name"].as_str()?.unwrap_or("").to_string();
                let duration = s["duration"].as_i64().map(|s| std::time::Duration::from_secs(s as u64));
                Some(MediaItem { id, title, artist, album, duration, cover_url: None, media_type: MediaType::Audio, qualities: vec![Quality::Flac, Quality::High, Quality::Standard] })
            }).collect();
            let total = json["data"]["total"].as_u64().unwrap_or(0) as usize;
            Ok(SearchResult { data, total, source: "kugou".into(), page })
        })
    }

    fn play_url(&self, item: &MediaItem, quality: Quality) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ViewSource, String>> + Send>> {
        let id = item.id.clone();
        Box::pin(async move {
            let br = match quality {
                Quality::Flac => 2000,
                Quality::High => 320,
                _ => 128,
            };
            let url = format!("https://trackercdn.kugou.com/i/v2/?cmd=25&hash={}&pid=1&behavior=play", id);
            Ok(ViewSource { url, quality, format: "mp3".into(), bitrate: br, size: 0, source: "kugou".into() })
        })
    }
}
