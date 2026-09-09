use crate::l2_perception::nt_world::source::types::*;
use crate::l2_perception::nt_world::source::engine::MediaSource;
use std::process::Command;

pub struct YtdlpSource;

impl YtdlpSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for YtdlpSource {
    fn id(&self) -> &str { "ytdlp" }
    fn name(&self) -> &str { "yt-dlp" }
    fn media_type(&self) -> MediaType { MediaType::Video }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let q = query.to_string();
        Box::pin(async move {
            let output = Command::new("yt-dlp")
                .args(["--flat-playlist", "--print", "%(id)s|||%(title)s|||%(uploader)s|||%(duration)s|||%(thumbnail)s", &format!("ytsearch{}:{}", page * 5, q)])
                .output()
                .map_err(|e| format!("yt-dlp not found: {}", e))?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            let data: Vec<MediaItem> = stdout.lines().filter_map(|line| {
                let parts: Vec<&str> = line.split("|||").collect();
                if parts.len() < 5 { return None; }
                let id = parts[0].to_string();
                let title = parts[1].to_string();
                let artist = parts[2].to_string();
                let duration = parts[3].parse::<u64>().ok().map(std::time::Duration::from_secs);
                let cover_url = if parts[4].is_empty() { None } else { Some(parts[4].to_string()) };
                Some(MediaItem {
                    id,
                    title,
                    artist,
                    album: String::new(),
                    duration,
                    cover_url,
                    media_type: MediaType::Video,
                    qualities: vec![Quality::Standard],
                })
            }).collect();
            let total = data.len();
            Ok(SearchResult { data, total, source: "ytdlp".into(), page })
        })
    }

    fn play_url(&self, item: &MediaItem, quality: Quality) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ViewSource, String>> + Send>> {
        let id = item.id.clone();
        Box::pin(async move {
            let format_str = match quality {
                Quality::High => "bestvideo[height<=1080]+bestaudio/best[height<=1080]",
                Quality::Standard => "bestvideo[height<=720]+bestaudio/best[height<=720]",
                _ => "best",
            };
            let output = Command::new("yt-dlp")
                .args(["-f", format_str, "--get-url", &format!("https://www.youtube.com/watch?v={}", id)])
                .output()
                .map_err(|e| format!("yt-dlp error: {}", e))?;
            let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if url.is_empty() { return Err("No URL found".into()); }
            let bitrate = quality.bitrate();
            Ok(ViewSource { url, quality, format: "mp4".into(), bitrate, size: 0, source: "ytdlp".into() })
        })
    }
}
