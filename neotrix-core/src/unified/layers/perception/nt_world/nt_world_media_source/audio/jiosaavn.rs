use crate::unified::layers::perception::nt_world::nt_world_media_source::types::*;
use crate::unified::layers::perception::nt_world::nt_world_media_source::engine::MediaSource;

pub struct JioSaavnSource;

impl JioSaavnSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for JioSaavnSource {
    fn id(&self) -> &str { "jiosaavn" }
    fn name(&self) -> &str { "JioSaavn" }
    fn media_type(&self) -> MediaType { MediaType::Audio }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let query = query.to_string();
        Box::pin(async move {
            let url = format!("https://www.jiosaavn.com/api.php?__call=autocomplete.get&_format=json&_marker=0&cc=in&includeMetaTags=1&query={}", urlencoding::encode(&query));
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let songs = json["songs"]["data"].as_array().ok_or("No songs")?;
            let data: Vec<MediaItem> = songs.iter().filter_map(|s| {
                let id = s["id"].as_str()?.to_string();
                let title = s["title"].as_str()?.to_string();
                let artist = s["description"].as_str()?.unwrap_or("Unknown").to_string();
                let album = s["album"].as_str()?.unwrap_or("").to_string();
                let duration = s["duration"].as_str().and_then(|d| d.parse::<u64>().ok()).map(|s| std::time::Duration::from_secs(s));
                Some(MediaItem { id, title, artist, album, duration, cover_url: None, media_type: MediaType::Audio, qualities: vec![Quality::High, Quality::Standard] })
            }).collect();
            let total = data.len();
            Ok(SearchResult { data, total, source: "jiosaavn".into(), page })
        })
    }

    fn play_url(&self, item: &MediaItem, quality: Quality) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ViewSource, String>> + Send>> {
        let id = item.id.clone();
        Box::pin(async move {
            let url = format!("https://www.jiosaavn.com/api.php?__call=song.getDetails&cc=in&_marker=0%3F_marker%3D0&p=android&q=0&api_version=4&_format=json&song={}", id);
            Ok(ViewSource { url, quality, format: "mp3".into(), bitrate: quality.bitrate(), size: 0, source: "jiosaavn".into() })
        })
    }
}
