use crate::l2_perception::nt_world::source::types::*;
use crate::l2_perception::nt_world::source::engine::MediaSource;

pub struct WikimediaSource;

impl WikimediaSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for WikimediaSource {
    fn id(&self) -> &str { "wikimedia" }
    fn name(&self) -> &str { "Wikimedia Commons" }
    fn media_type(&self) -> MediaType { MediaType::Image }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let q = query.to_string();
        Box::pin(async move {
            let offset = (page - 1) * 20;
            let url = format!("https://commons.wikimedia.org/w/api.php?action=query&list=search&srsearch={}&srnamespace=6&sroffset={}&srlimit=20&format=json", urlencoding::encode(&q), offset);
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let search = json["query"]["search"].as_array().ok_or("No results")?;
            let data: Vec<MediaItem> = search.iter().filter_map(|s| {
                let title = s["title"].as_str()?.to_string();
                let pageid = s["pageid"].as_i64()?.to_string();
                let snippet = s["snippet"].as_str().unwrap_or("").to_string();
                let encoded_title = urlencoding::encode(&title);
                let image_url = format!("https://commons.wikimedia.org/wiki/Special:FilePath/{}", encoded_title);
                Some(MediaItem {
                    id: pageid,
                    title,
                    artist: "Wikimedia".to_string(),
                    album: snippet,
                    duration: None,
                    cover_url: Some(image_url),
                    media_type: MediaType::Image,
                    qualities: vec![Quality::Standard],
                })
            }).collect();
            let total = json["query"]["searchinfo"]["totalhits"].as_u64().unwrap_or(0) as usize;
            Ok(SearchResult { data, total, source: "wikimedia".into(), page })
        })
    }
}
