use crate::unified::layers::perception::nt_world::nt_world_media_source::types::*;
use crate::unified::layers::perception::nt_world::nt_world_media_source::engine::MediaSource;

pub struct VimeoSource;

impl VimeoSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for VimeoSource {
    fn id(&self) -> &str { "vimeo" }
    fn name(&self) -> &str { "Vimeo" }
    fn media_type(&self) -> MediaType { MediaType::Video }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let q = query.to_string();
        Box::pin(async move {
            let url = format!("https://api.vimeo.com/videos?query={}&page={}&per_page=20", urlencoding::encode(&q), page);
            let resp = reqwest::Client::new()
                .get(&url)
                .header("User-Agent", "NeoTrix/1.0")
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let data_list = json["data"].as_array().ok_or("No results")?;
            let data: Vec<MediaItem> = data_list.iter().filter_map(|v| {
                let id = v["uri"].as_str()?.trim_start_matches("/videos/").to_string();
                let title = v["name"].as_str()?.to_string();
                let author = v["user"]["name"].as_str()?.unwrap_or("Unknown").to_string();
                let duration = v["duration"].as_i64().map(|s| std::time::Duration::from_secs(s as u64));
                let cover = v["pictures"]["sizes"].as_array()?.last()?.get("link")?.as_str().map(String::from);
                Some(MediaItem {
                    id,
                    title,
                    artist: author,
                    album: String::new(),
                    duration,
                    cover_url: cover,
                    media_type: MediaType::Video,
                    qualities: vec![Quality::Standard],
                })
            }).collect();
            let total = json["total"].as_u64().unwrap_or(0) as usize;
            Ok(SearchResult { data, total, source: "vimeo".into(), page })
        })
    }
}
