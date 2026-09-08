use crate::unified::layers::perception::nt_world::nt_world_media_source::types::*;
use crate::unified::layers::perception::nt_world::nt_world_media_source::engine::MediaSource;

pub struct NeteaseSource;

impl NeteaseSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for NeteaseSource {
    fn id(&self) -> &str { "netease" }
    fn name(&self) -> &str { "网易云音乐" }
    fn media_type(&self) -> MediaType { MediaType::Audio }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let query = query.to_string();
        Box::pin(async move {
            let url = format!("https://music.163.com/api/search/get?s={}&type=1&limit=30&offset={}", urlencoding::encode(&query), (page - 1) * 30);
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let songs = json["result"]["songs"].as_array().ok_or("No songs")?;
            let data: Vec<MediaItem> = songs.iter().filter_map(|s| {
                let id = s["id"].as_i64()?.to_string();
                let title = s["name"].as_str()?.to_string();
                let artist = s["artists"][0]["name"].as_str()?.unwrap_or("Unknown").to_string();
                let album = s["album"]["name"].as_str()?.unwrap_or("").to_string();
                let duration = s["duration"].as_i64().map(|ms| std::time::Duration::from_millis(ms as u64));
                Some(MediaItem { id, title, artist, album, duration, cover_url: None, media_type: MediaType::Audio, qualities: vec![Quality::Flac, Quality::High, Quality::Standard] })
            }).collect();
            let total = json["result"]["songCount"].as_u64().unwrap_or(0) as usize;
            Ok(SearchResult { data, total, source: "netease".into(), page })
        })
    }

    fn play_url(&self, item: &MediaItem, quality: Quality) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ViewSource, String>> + Send>> {
        let id = item.id.clone();
        Box::pin(async move {
            let bitrate = quality.bitrate();
            let url = format!("https://music.163.com/song/media/outer/url?id={}", id);
            Ok(ViewSource { url, quality, format: "mp3".into(), bitrate, size: 0, source: "netease".into() })
        })
    }
}
