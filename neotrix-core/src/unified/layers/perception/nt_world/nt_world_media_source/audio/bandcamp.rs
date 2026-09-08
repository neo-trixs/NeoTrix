use crate::unified::layers::perception::nt_world::nt_world_media_source::types::*;
use crate::unified::layers::perception::nt_world::nt_world_media_source::engine::MediaSource;
use scraper::{Html, Selector};

pub struct BandcampSource;

impl BandcampSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for BandcampSource {
    fn id(&self) -> &str { "bandcamp" }
    fn name(&self) -> &str { "Bandcamp" }
    fn media_type(&self) -> MediaType { MediaType::Audio }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let query = query.to_string();
        Box::pin(async move {
            let url = format!("https://bandcamp.com/api/bcsearch_public_api/1/autocomplete_elastic",);
            let body = serde_json::json!({
                "search_text": query,
                "search_filter": "t",
                "full_page": true,
                "fan_id": null
            });
            let resp = reqwest::Client::new()
                .post(&url)
                .json(&body)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let items = json["auto"]["results"].as_array().ok_or("No results")?;
            let data: Vec<MediaItem> = items.iter().filter_map(|s| {
                let id = s["id"].as_i64()?.to_string();
                let title = s["title"].as_str()?.to_string();
                let artist = s["band_name"].as_str()?.unwrap_or("Unknown").to_string();
                let url_str = s["url"].as_str()?;
                Some(MediaItem { id, title, artist, album: String::new(), duration: None, cover_url: None, media_type: MediaType::Audio, qualities: vec![Quality::High, Quality::Standard] })
            }).collect();
            let total = data.len();
            Ok(SearchResult { data, total, source: "bandcamp".into(), page })
        })
    }

    fn play_url(&self, item: &MediaItem, quality: Quality) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ViewSource, String>> + Send>> {
        let id = item.id.clone();
        Box::pin(async move {
            let url = format!("https://bandcamp.com/EmbeddedPlayer/track/id={}", id);
            Ok(ViewSource { url, quality, format: "mp3".into(), bitrate: quality.bitrate(), size: 0, source: "bandcamp".into() })
        })
    }
}
