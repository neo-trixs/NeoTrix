//! Video thumbnail extraction — generates preview images from video/audio files.
//!
//! Uses ffmpeg to extract first frame, or symmetric timestamps.
//! Falls back to embedded thumbnail (album art for audio).

use std::path::Path;
use tokio::process::Command;

/// Thumbnail extraction result
pub struct Thumbnail {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: ThumbFormat,
}

pub enum ThumbFormat {
    Jpeg,
    Png,
}

/// Extract thumbnail from media file.
/// - Video: extracts first frame at specified time
/// - Audio: extracts embedded album art
/// - Default: 0.5 seconds into the file
pub async fn extract_thumbnail(
    path: &Path,
    time_offset: Option<&str>, // e.g., "00:00:01.000"
) -> Result<Thumbnail, ThumbError> {
    let time = time_offset.unwrap_or("00:00:00.500");

    let output = Command::new("ffmpeg")
        .args([
            "-y", // overwrite
            "-i",
            &path.to_string_lossy(),
            "-ss",
            time,
            "-vframes",
            "1",
            "-f",
            "image2pipe",
            "-vcodec",
            "png",
            "pipe:1",
        ])
        .output()
        .await
        .map_err(|e| ThumbError::Command(format!("ffmpeg not found: {}", e)))?;

    if !output.status.success() {
        // Try extracting album art for audio files
        return extract_album_art(path).await;
    }

    let data = output.stdout;
    if data.is_empty() {
        return Err(ThumbError::Empty);
    }

    // Parse PNG dimensions from IHDR chunk (bytes 16-24)
    let (width, height) = if data.len() > 24 && &data[0..8] == b"\x89PNG\r\n\x1a\n" {
        let w = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
        let h = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
        (w, h)
    } else {
        (0, 0)
    };

    Ok(Thumbnail {
        data,
        width,
        height,
        format: ThumbFormat::Png,
    })
}

/// Extract embedded album art from audio file
async fn extract_album_art(path: &Path) -> Result<Thumbnail, ThumbError> {
    let output = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            &path.to_string_lossy(),
            "-an", // no audio
            "-vcodec",
            "copy",
            "-f",
            "image2pipe",
            "-vcodec",
            "png",
            "pipe:1",
        ])
        .output()
        .await
        .map_err(|e| ThumbError::Command(e.to_string()))?;

    if !output.status.success() || output.stdout.is_empty() {
        return Err(ThumbError::NoThumbnail);
    }

    Ok(Thumbnail {
        data: output.stdout,
        width: 0,
        height: 0,
        format: ThumbFormat::Png,
    })
}

/// Generate multiple thumbnails at different timestamps
pub async fn extract_grid(
    path: &Path,
    count: usize, // number of thumbnails
) -> Result<Vec<Thumbnail>, ThumbError> {
    let duration = get_duration(path).await.unwrap_or(60.0);
    let interval = duration / (count + 1) as f64;

    let mut thumbnails = Vec::new();
    for i in 1..=count {
        let time = interval * i as f64;
        let ts = format!("{:.3}", time);
        if let Ok(thumb) = extract_thumbnail(path, Some(&ts)).await {
            thumbnails.push(thumb);
        }
    }
    Ok(thumbnails)
}

/// Get media duration in seconds
async fn get_duration(path: &Path) -> Result<f64, ThumbError> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            &path.to_string_lossy(),
        ])
        .output()
        .await
        .map_err(|e| ThumbError::Command(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .trim()
        .parse::<f64>()
        .map_err(|_| ThumbError::Parse("invalid duration".into()))
}

#[derive(Debug, thiserror::Error)]
pub enum ThumbError {
    #[error("command error: {0}")]
    Command(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("empty output")]
    Empty,
    #[error("no thumbnail found")]
    NoThumbnail,
}
