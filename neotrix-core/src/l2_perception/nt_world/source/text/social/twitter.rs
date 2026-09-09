use crate::l2_perception::nt_world::source::types::*;
use crate::l2_perception::nt_world::source::engine::MediaSource;

pub struct TwitterSource;

impl TwitterSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for TwitterSource {
    fn id(&self) -> &str { "twitter" }
    fn name(&self) -> &str { "Twitter" }
    fn media_type(&self) -> MediaType { MediaType::Social }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let q = query.to_string();
        Box::pin(async move {
            let url = format!("https://api.fxtwitter.com/search?q={}&f=jpeg", urlencoding::encode(&q));
            let resp = reqwest::Client::new()
                .get(&url)
                .header("User-Agent", "NeoTrix/1.0")
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let tweets = json["tweets"].as_array().ok_or("No results")?;
            let data: Vec<MediaItem> = tweets.iter().filter_map(|t| {
                let id = t["id"].as_str()?.to_string();
                let text = t["text"].as_str().unwrap_or("").to_string();
                let author = t["author"]["name"].as_str()?.to_string();
                let photo = t["media"]["photos"][0]["url"].as_str().map(String::from);
                Some(MediaItem {
                    id,
                    title: text.chars().take(100).collect(),
                    artist: author,
                    album: String::new(),
                    duration: None,
                    cover_url: photo,
                    media_type: MediaType::Social,
                    qualities: vec![Quality::Standard],
                })
            }).collect();
            let total = tweets.len();
            Ok(SearchResult { data, total, source: "twitter".into(), page })
        })
    }
}
