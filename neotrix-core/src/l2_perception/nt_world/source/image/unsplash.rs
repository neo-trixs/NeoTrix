use crate::l2_perception::nt_world::source::types::*;
use crate::l2_perception::nt_world::source::engine::MediaSource;

pub struct UnsplashSource;

impl UnsplashSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for UnsplashSource {
    fn id(&self) -> &str { "unsplash" }
    fn name(&self) -> &str { "Unsplash" }
    fn media_type(&self) -> MediaType { MediaType::Image }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let q = query.to_string();
        Box::pin(async move {
            let url = format!("https://api.unsplash.com/search/photos?query={}&page={}&per_page=20", urlencoding::encode(&q), page);
            let resp = reqwest::Client::new()
                .get(&url)
                .header("Authorization", format!("Client-ID {}", std::env::var("UNSPLASH_ACCESS_KEY").unwrap_or_default()))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let results = json["results"].as_array().ok_or("No results")?;
            let data: Vec<MediaItem> = results.iter().filter_map(|p| {
                let id = p["id"].as_str()?.to_string();
                let desc = p["description"].as_str().or(p["alt_description"].as_str()).unwrap_or("Untitled").to_string();
                let author = p["user"]["name"].as_str()?.to_string();
                let src = p["urls"]["regular"].as_str().map(String::from);
                Some(MediaItem {
                    id,
                    title: desc,
                    artist: author,
                    album: String::new(),
                    duration: None,
                    cover_url: src,
                    media_type: MediaType::Image,
                    qualities: vec![Quality::Standard],
                })
            }).collect();
            let total = json["total"].as_u64().unwrap_or(0) as usize;
            Ok(SearchResult { data, total, source: "unsplash".into(), page })
        })
    }
}
