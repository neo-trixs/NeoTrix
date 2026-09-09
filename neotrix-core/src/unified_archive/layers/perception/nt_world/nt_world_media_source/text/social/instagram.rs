use crate::unified::layers::perception::nt_world::nt_world_media_source::types::*;
use crate::unified::layers::perception::nt_world::nt_world_media_source::engine::MediaSource;

pub struct InstagramSource;

impl InstagramSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for InstagramSource {
    fn id(&self) -> &str { "instagram" }
    fn name(&self) -> &str { "Instagram" }
    fn media_type(&self) -> MediaType { MediaType::Social }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let q = query.to_string();
        Box::pin(async move {
            let url = format!("https://www.instagram.com/web/search/topsearch/?query={}", urlencoding::encode(&q));
            let resp = reqwest::Client::new()
                .get(&url)
                .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let users = json["users"].as_array().ok_or("No results")?;
            let data: Vec<MediaItem> = users.iter().enumerate().skip(((page - 1) * 20) as usize).take(20).filter_map(|(i, u)| {
                let user = u["user"]?;
                let pk = user["pk"].as_i64()?.to_string();
                let username = user["username"].as_str()?.to_string();
                let full_name = user["full_name"].as_str().unwrap_or(&username).to_string();
                let avatar = user["profile_pic_url"].as_str().map(String::from);
                Some(MediaItem {
                    id: pk,
                    title: full_name,
                    artist: username,
                    album: String::new(),
                    duration: None,
                    cover_url: avatar,
                    media_type: MediaType::Social,
                    qualities: vec![Quality::Standard],
                })
            }).collect();
            let total = users.len();
            Ok(SearchResult { data, total, source: "instagram".into(), page })
        })
    }
}
