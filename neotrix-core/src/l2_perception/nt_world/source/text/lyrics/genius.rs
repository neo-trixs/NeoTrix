use crate::l2_perception::nt_world::source::types::*;
use crate::l2_perception::nt_world::source::engine::MediaSource;

pub struct GeniusSource;

impl GeniusSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for GeniusSource {
    fn id(&self) -> &str { "genius" }
    fn name(&self) -> &str { "Genius" }
    fn media_type(&self) -> MediaType { MediaType::Lyrics }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let q = query.to_string();
        Box::pin(async move {
            let url = format!("https://genius.com/api/search?q={}", urlencoding::encode(&q));
            let resp = reqwest::Client::new()
                .get(&url)
                .header("User-Agent", "NeoTrix/1.0")
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let hits = json["response"]["hits"].as_array().ok_or("No results")?;
            let data: Vec<MediaItem> = hits.iter().filter_map(|h| {
                let result = h["result"].as_object()?;
                let id = result["id"].as_i64()?.to_string();
                let title = result["title"].as_str()?.to_string();
                let artist = result["primary_artist"]["name"].as_str()?.to_string();
                let url = result["url"].as_str().map(String::from);
                Some(MediaItem {
                    id,
                    title,
                    artist,
                    album: String::new(),
                    duration: None,
                    cover_url: url,
                    media_type: MediaType::Lyrics,
                    qualities: vec![Quality::Standard],
                })
            }).collect();
            let total = hits.len();
            Ok(SearchResult { data, total, source: "genius".into(), page })
        })
    }

    fn lyric(&self, item: &MediaItem) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Lyric, String>> + Send>> {
        let id = item.id.clone();
        let title = item.title.clone();
        let artist = item.artist.clone();
        Box::pin(async move {
            let url = format!("https://genius.com/api/songs/{}", id);
            let resp = reqwest::Client::new()
                .get(&url)
                .header("User-Agent", "NeoTrix/1.0")
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let lyrics_url = json["response"]["song"]["url"].as_str().ok_or("No lyrics URL")?;
            let resp = reqwest::Client::new()
                .get(lyrics_url)
                .header("User-Agent", "NeoTrix/1.0")
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let html = resp.text().await.map_err(|e| e.to_string())?;
            let lyrics_container = html.split("data-lyrics-container=\"true\">").nth(1).ok_or("No lyrics found")?;
            let raw = lyrics_container.split("</div>").next().unwrap_or("");
            let cleaned = raw.replace("<br>", "\n").replace("<br/>", "\n");
            let stripped: String = cleaned.chars().filter(|c| *c != '<' && *c != '>').collect();
            let lines: Vec<LyricLine> = stripped.lines().enumerate().map(|(i, line)| {
                LyricLine { timestamp: Some(std::time::Duration::from_millis((i as u64) * 4000)), text: line.trim().to_string() }
            }).filter(|l| !l.text.is_empty()).collect();
            Ok(Lyric { title: Some(title), artist: Some(artist), lines, source: "genius".into() })
        })
    }
}
