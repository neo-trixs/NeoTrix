//! yt-dlp video URL extraction — resolves video page URLs to direct download URLs.
//!
//! Shells out to yt-dlp binary for extraction, then routes result through
//! the streaming pipeline (HTTP Range / Stream).

use tokio::process::Command;

/// Extracted video info from yt-dlp
pub struct VideoInfo {
    pub title: String,
    pub url: String,
    pub format_id: Option<String>,
    pub ext: Option<String>,
    pub duration: Option<f64>,
    pub resolution: Option<String>,
    pub filesize: Option<u64>,
    pub webpage_url: String,
    pub thumbnail: Option<String>,
}

/// Extract direct URL from video page using yt-dlp.
/// Returns JSON with url, title, format info.
pub async fn extract_url(video_url: &str) -> Result<VideoInfo, YtError> {
    let output = Command::new("yt-dlp")
        .args([
            "-j",                    // JSON output
            "--no-warnings",
            "--no-playlist",         // single video only
            "--no-check-certificates",
            video_url,
        ])
        .output()
        .await
        .map_err(|e| YtError::Command(format!("yt-dlp not found: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(YtError::Extraction(stderr.to_string()));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| YtError::Parse(e.to_string()))?;

    Ok(VideoInfo {
        title: v["title"].as_str().unwrap_or("unknown").to_string(),
        url: v["url"].as_str().unwrap_or("").to_string(),
        format_id: v["format_id"].as_str().map(String::from),
        ext: v["ext"].as_str().map(String::from),
        duration: v["duration"].as_f64(),
        resolution: v["resolution"].as_str().map(String::from),
        filesize: v["filesize"].as_u64(),
        webpage_url: v["webpage_url"].as_str().unwrap_or(video_url).to_string(),
        thumbnail: v["thumbnail"].as_str().map(String::from),
    })
}

/// List available formats for a video URL
pub async fn list_formats(video_url: &str) -> Result<Vec<FormatInfo>, YtError> {
    let output = Command::new("yt-dlp")
        .args(["-j", "--no-warnings", "--no-playlist", video_url])
        .output()
        .await
        .map_err(|e| YtError::Command(format!("yt-dlp not found: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(YtError::Extraction(stderr.to_string()));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| YtError::Parse(e.to_string()))?;

    let mut formats = Vec::new();
    if let Some(arr) = v["formats"].as_array() {
        for f in arr {
            formats.push(FormatInfo {
                format_id: f["format_id"].as_str().unwrap_or("").to_string(),
                ext: f["ext"].as_str().unwrap_or("").to_string(),
                resolution: f["resolution"].as_str().unwrap_or("").to_string(),
                filesize: f["filesize"].as_u64(),
                vcodec: f["vcodec"].as_str().unwrap_or("").to_string(),
                acodec: f["acodec"].as_str().unwrap_or("").to_string(),
            });
        }
    }
    Ok(formats)
}

pub struct FormatInfo {
    pub format_id: String,
    pub ext: String,
    pub resolution: String,
    pub filesize: Option<u64>,
    pub vcodec: String,
    pub acodec: String,
}

#[derive(Debug, thiserror::Error)]
pub enum YtError {
    #[error("command error: {0}")]
    Command(String),
    #[error("extraction failed: {0}")]
    Extraction(String),
    #[error("parse error: {0}")]
    Parse(String),
}
