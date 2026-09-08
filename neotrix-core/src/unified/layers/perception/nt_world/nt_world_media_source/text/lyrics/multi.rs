use crate::unified::layers::perception::nt_world::nt_world_media_source::types::*;
use crate::unified::layers::perception::nt_world::nt_world_media_source::engine::MediaSource;

pub struct MultiLyricSource;

impl MultiLyricSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for MultiLyricSource {
    fn id(&self) -> &str { "multi_lyric" }
    fn name(&self) -> &str { "Multi-Lyric" }
    fn media_type(&self) -> MediaType { MediaType::Lyrics }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let q = query.to_string();
        Box::pin(async move {
            let url = format!("https://lrclib.net/api/search?q={}", urlencoding::encode(&q));
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let items = json.as_array().ok_or("No results")?;
            let data: Vec<MediaItem> = items.iter().filter_map(|v| {
                let id = v["id"].as_i64()?.to_string();
                let title = v["trackName"].as_str()?.to_string();
                let artist = v["artistName"].as_str()?.to_string();
                let album = v["albumName"].as_str().unwrap_or("").to_string();
                let duration = v["duration"].as_i64().map(|s| std::time::Duration::from_secs(s as u64));
                Some(MediaItem {
                    id,
                    title,
                    artist,
                    album,
                    duration,
                    cover_url: None,
                    media_type: MediaType::Lyrics,
                    qualities: vec![Quality::Standard],
                })
            }).collect();
            let total = items.len();
            Ok(SearchResult { data, total, source: "multi_lyric".into(), page })
        })
    }

    fn lyric(&self, item: &MediaItem) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Lyric, String>> + Send>> {
        let id = item.id.clone();
        let title = item.title.clone();
        let artist = item.artist.clone();
        Box::pin(async move {
            let url = format!("https://lrclib.net/api/{}", id);
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let synced = json["syncedLyrics"].as_str().or(json["plainLyrics"].as_str()).ok_or("No lyrics")?;
            let lines: Vec<LyricLine> = synced.lines().enumerate().map(|(i, line)| {
                if let Some(stripped) = line.strip_prefix('[') {
                    if let Some(close) = stripped.find(']') {
                        let time_part = &stripped[..close];
                        let text = stripped[close + 1..].trim().to_string();
                        let parts: Vec<&str> = time_part.split(':').collect();
                        if parts.len() == 2 {
                            if let (Ok(min), Ok(sec)) = (parts[0].parse::<i64>(), parts[1].parse::<f64>()) {
                                return LyricLine { time_ms: min * 60000 + (sec * 1000.0) as i64, text };
                            }
                        }
                    }
                }
                LyricLine { time_ms: (i as i64) * 4000, text: line.to_string() }
            }).collect();
            Ok(Lyric { title, artist, lines, source: "multi_lyric".into() })
        })
    }
}
