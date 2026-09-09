use crate::l2_perception::nt_world::source::types::*;
use crate::l2_perception::nt_world::source::engine::MediaSource;

pub struct TikTokSource;

impl TikTokSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for TikTokSource {
    fn id(&self) -> &str { "tiktok" }
    fn name(&self) -> &str { "TikTok" }
    fn media_type(&self) -> MediaType { MediaType::Social }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let q = query.to_string();
        Box::pin(async move {
            let url = format!("https://www.tiktok.com/api/search/general/full/?keyword={}&cursor={}", urlencoding::encode(&q), (page - 1) * 20);
            let resp = reqwest::Client::new()
                .get(&url)
                .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let items = json["data"].as_array().ok_or("No results")?;
            let data: Vec<MediaItem> = items.iter().filter_map(|v| {
                let item = v["item"].as_object()?;
                let id = item["id"].as_str()?.to_string();
                let desc = item["desc"].as_str().unwrap_or("Untitled").to_string();
                let author = item["author"]["uniqueId"].as_str()?.to_string();
                let duration = item["video"]["duration"].as_i64().map(|d| std::time::Duration::from_secs(d as u64 / 1000));
                let cover = item["video"]["cover"].as_str().map(String::from);
                Some(MediaItem {
                    id,
                    title: desc,
                    artist: author,
                    album: String::new(),
                    duration,
                    cover_url: cover,
                    media_type: MediaType::Social,
                    qualities: vec![Quality::Standard],
                })
            }).collect();
            let total = items.len();
            Ok(SearchResult { data, total, source: "tiktok".into(), page })
        })
    }
}
