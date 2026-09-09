use crate::unified::layers::perception::nt_world::nt_world_media_source::types::*;
use crate::unified::layers::perception::nt_world::nt_world_media_source::engine::MediaSource;

pub struct MultiLyricSource;

impl MultiLyricSource {
    pub fn new() -> Self { Self }

    /// 解析 LRC 时间轴
    fn parse_lrc_time(time_str: &str) -> Option<i64> {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() == 2 {
            let min = parts[0].parse::<i64>().ok()?;
            let sec = parts[1].parse::<f64>().ok()?;
            Some(min * 60000 + (sec * 1000.0) as i64)
        } else {
            None
        }
    }

    /// 解析逐字歌词 (增强版 LRC)
    fn parse_enhanced_lrc(line: &str) -> Vec<LyricWord> {
        let mut words = Vec::new();
        let mut remaining = line;
        while let Some(start) = remaining.find('<') {
            if let Some(end) = remaining[start..].find('>') {
                let tag = &remaining[start + 1..start + end];
                if let Some(comma) = tag.find(',') {
                    if let Ok(time_ms) = tag[comma + 1..].parse::<i64>() {
                        let text = tag[..comma].to_string();
                        words.push(LyricWord { time_ms, text });
                    }
                }
                remaining = &remaining[start + end + 1..];
            } else {
                break;
            }
        }
        words
    }
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
            let lines: Vec<LyricLine> = synced.lines().filter_map(|line| {
                if let Some(stripped) = line.strip_prefix('[') {
                    if let Some(close) = stripped.find(']') {
                        let time_str = &stripped[..close];
                        let text = stripped[close + 1..].trim().to_string();
                        if let Some(time_ms) = Self::parse_lrc_time(time_str) {
                            return Some(LyricLine { time_ms, text });
                        }
                    }
                }
                None
            }).collect();
            Ok(Lyric { title, artist, lines, source: "multi_lyric".into() })
        })
    }
}

/// 歌词行
#[derive(Debug, Clone)]
pub struct LyricLine {
    pub time_ms: i64,
    pub text: String,
}

/// 歌词中的单个字
#[derive(Debug, Clone)]
pub struct LyricWord {
    pub time_ms: i64,
    pub text: String,
}

/// 歌词
#[derive(Debug, Clone)]
pub struct Lyric {
    pub title: String,
    pub artist: String,
    pub lines: Vec<LyricLine>,
    pub source: String,
}
