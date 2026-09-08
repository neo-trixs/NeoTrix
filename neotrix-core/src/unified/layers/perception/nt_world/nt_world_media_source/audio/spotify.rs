use crate::unified::layers::perception::nt_world::nt_world_media_source::types::*;
use crate::unified::layers::perception::nt_world::nt_world_media_source::engine::MediaSource;

pub struct SpotifySource;

impl SpotifySource {
    pub fn new() -> Self { Self }
}

impl MediaSource for SpotifySource {
    fn id(&self) -> &str { "spotify" }
    fn name(&self) -> &str { "Spotify" }
    fn media_type(&self) -> MediaType { MediaType::Audio }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let query = query.to_string();
        Box::pin(async move {
            let offset = (page - 1) * 20;
            let url = format!("https://api.spotify.com/v1/search?q={}&type=track&limit=20&offset={}", urlencoding::encode(&query), offset);
            let resp = reqwest::Client::new()
                .get(&url)
                .header("Authorization", "Bearer ")
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let items = json["tracks"]["items"].as_array().ok_or("No tracks")?;
            let data: Vec<MediaItem> = items.iter().filter_map(|s| {
                let id = s["id"].as_str()?.to_string();
                let title = s["name"].as_str()?.to_string();
                let artist = s["artists"][0]["name"].as_str()?.unwrap_or("Unknown").to_string();
                let album = s["album"]["name"].as_str()?.unwrap_or("").to_string();
                let duration = s["duration_ms"].as_i64().map(|ms| std::time::Duration::from_millis(ms as u64));
                let cover_url = s["album"]["images"].as_array()?.first()?.get("url")?.as_str().map(String::from);
                Some(MediaItem { id, title, artist, album, duration, cover_url, media_type: MediaType::Audio, qualities: vec![Quality::High, Quality::Standard] })
            }).collect();
            let total = json["tracks"]["total"].as_u64().unwrap_or(0) as usize;
            Ok(SearchResult { data, total, source: "spotify".into(), page })
        })
    }

    fn play_url(&self, item: &MediaItem, quality: Quality) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ViewSource, String>> + Send>> {
        let id = item.id.clone();
        Box::pin(async move {
            let url = format!("https://open.spotify.com/embed/track/{}", id);
            Ok(ViewSource { url, quality, format: "mp3".into(), bitrate: quality.bitrate(), size: 0, source: "spotify".into() })
        })
    }
}
