//! Thumbnail extraction — Rust-native for images & audio album art.
//!
//! - Images: `image` crate load/resize/thumbnail (zero binary deps)
//! - Audio album art: `symphonia` metadata extraction (behind `audio-decode` feature)
//! - Video first frame: ffmpeg fallback behind `video-decode` feature gate

use std::path::Path;

use image::codecs::png::PngEncoder;
use image::{ImageEncoder, GenericImageView};

use super::detect;

/// Thumbnail extraction result
pub struct Thumbnail {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: ThumbFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbFormat {
    Jpeg,
    Png,
}

/// Extract thumbnail from media file.
///
/// Dispatches by detected `MediaKind`:
/// - **Image**: loads via `image` crate, returns JPEG thumbnail
/// - **Audio**: extracts embedded album art via `symphonia` (`audio-decode` feature)
/// - **Video**: ffmpeg first-frame extraction (`video-decode` feature), else error
pub async fn extract_thumbnail(
    path: &Path,
    _time_offset: Option<&str>,
) -> Result<Thumbnail, ThumbError> {
    let kind = detect::detect_from_path(path);
    if kind.is_audio() {
        #[cfg(feature = "audio-decode")]
        {
            return extract_album_art_symphonia(path).await;
        }
        #[cfg(not(feature = "audio-decode"))]
        {
            return Err(ThumbError::Unsupported(
                "audio album art requires audio-decode feature".into(),
            ));
        }
    } else if kind.is_video() {
        #[cfg(feature = "video-decode")]
        {
            return extract_video_frame_ffmpeg(path, time_offset).await;
        }
        #[cfg(not(feature = "video-decode"))]
        {
            return Err(ThumbError::Unsupported(
                "video thumbnail requires video-decode feature".into(),
            ));
        }
    } else if is_image_path(path) {
        extract_image_thumbnail(path).await
    } else {
        Err(ThumbError::Unsupported(format!(
            "unsupported file type: {}",
            path.display()
        )))
    }
}

/// Load an image file and return a resized thumbnail (max 512px on longest side).
///
/// Uses only the `image` crate — no external binaries.
async fn extract_image_thumbnail(path: &Path) -> Result<Thumbnail, ThumbError> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let img = image::open(&path).map_err(|e| ThumbError::Load(e.to_string()))?;

        let (w, h) = img.dimensions();
        let thumb = if w > 512 || h > 512 {
            let ratio = 512.0 / (w.max(h) as f64);
            let new_w = (w as f64 * ratio) as u32;
            let new_h = (h as f64 * ratio) as u32;
            img.resize(new_w, new_h, image::imageops::FilterType::Lanczos3)
        } else {
            img
        };

        let (tw, th) = thumb.dimensions();
        let mut buf = Vec::new();
        let encoder = PngEncoder::new(&mut buf);
        encoder
            .write_image(
                thumb.to_rgb8().as_raw(),
                tw,
                th,
                image::ExtendedColorType::Rgb8,
            )
            .map_err(|e| ThumbError::Encode(e.to_string()))?;

        Ok(Thumbnail {
            data: buf,
            width: tw,
            height: th,
            format: ThumbFormat::Png,
        })
    })
    .await
    .map_err(|e| ThumbError::Load(format!("task join: {e}")))?
}

/// Extract embedded album art from an audio file using `symphonia`.
///
/// Probes metadata for any attached picture (e.g. ID3v2 APIC, Vorbis comments,
/// MP4 cover). Returns the first image found as JPEG or PNG bytes.
#[cfg(feature = "audio-decode")]
async fn extract_album_art_symphonia(path: &Path) -> Result<Thumbnail, ThumbError> {
    use std::fs::File;
    use std::io::BufReader;

    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let path_clone = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let file = File::open(&path_clone).map_err(|e| ThumbError::Load(e.to_string()))?;
        let source = BufReader::new(file);
        let mss = MediaSourceStream::new(Box::new(source), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path_clone.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let fmt_opts = FormatOptions {
            enable_gapless: true,
            ..Default::default()
        };
        let meta_opts = MetadataOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .map_err(|e| ThumbError::Parse(format!("format probe: {e}")))?;

        let mut reader = probed.format;

        // Get the latest metadata revision and extract the first visual (album art)
        let visual = {
            let mut meta = reader.metadata();
            meta.skip_to_latest()
                .and_then(|rev| rev.visuals().first().cloned())
        };

        let visual = visual.ok_or_else(|| ThumbError::NoThumbnail)?;

        // visual.media_type is a MIME string like "image/jpeg", "image/png"
        let mime = visual.media_type.to_lowercase();

        let (data, format) = if mime.contains("jpeg") || mime.contains("jpg") {
            (visual.data, ThumbFormat::Jpeg)
        } else if mime.contains("png") {
            (visual.data, ThumbFormat::Png)
        } else {
            // BMP, WebP, or unknown — decode via image crate and re-encode as PNG
            let img = image::load_from_memory(&visual.data)
                .map_err(|e| ThumbError::Load(e.to_string()))?;
            let mut buf = Vec::new();
            let encoder = PngEncoder::new(&mut buf);
            let (w, h) = img.dimensions();
            encoder
                .write_image(
                    img.to_rgb8().as_raw(),
                    w,
                    h,
                    image::ExtendedColorType::Rgb8,
                )
                .map_err(|e| ThumbError::Encode(e.to_string()))?;
            return Ok(Thumbnail {
                data: buf,
                width: w,
                height: h,
                format: ThumbFormat::Png,
            });
        };

        Ok(Thumbnail {
            data,
            width: 0,
            height: 0,
            format,
        })
    })
    .await
    .map_err(|e| ThumbError::Load(format!("task join: {e}")))?
}

/// Extract a video frame via ffmpeg (feature-gated).
#[cfg(feature = "video-decode")]
async fn extract_video_frame_ffmpeg(
    path: &Path,
    time_offset: Option<&str>,
) -> Result<Thumbnail, ThumbError> {
    let time = time_offset.unwrap_or("00:00:00.500");
    let output = tokio::process::Command::new("ffmpeg")
        .args([
            "-y",
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
        .map_err(|e| ThumbError::Command(format!("ffmpeg not found: {e}")))?;

    if !output.status.success() {
        return Err(ThumbError::Command(format!(
            "ffmpeg exited {}",
            output.status
        )));
    }

    let data = output.stdout;
    if data.is_empty() {
        return Err(ThumbError::Empty);
    }

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

/// Generate multiple thumbnails at different timestamps.
///
/// For images, returns a single thumbnail (grid count ignored).
/// For audio/video with `video-decode`/`audio-decode` features, samples evenly.
pub async fn extract_grid(
    path: &Path,
    _count: usize,
) -> Result<Vec<Thumbnail>, ThumbError> {
    let kind = detect::detect_from_path(path);

    if is_image_path(path) || !kind.is_media() {
        let thumb = extract_thumbnail(path, None).await?;
        return Ok(vec![thumb]);
    }

    // For audio: single album art
    if kind.is_audio() {
        let thumb = extract_thumbnail(path, None).await?;
        return Ok(vec![thumb]);
    }

    // For video with ffmpeg: extract at evenly spaced timestamps
    #[cfg(feature = "video-decode")]
    {
        let duration = get_duration_ffprobe(path).await.unwrap_or(60.0);
        let interval = duration / (count + 1) as f64;
        let mut thumbnails = Vec::new();
        for i in 1..=count {
            let time = interval * i as f64;
            let ts = format!("{time:.3}");
            if let Ok(thumb) = extract_video_frame_ffmpeg(path, Some(&ts)).await {
                thumbnails.push(thumb);
            }
        }
        return Ok(thumbnails);
    }

    #[cfg(not(feature = "video-decode"))]
    {
        Err(ThumbError::Unsupported(
            "video grid requires video-decode feature".into(),
        ))
    }
}

/// Get media duration in seconds via ffprobe (only when `video-decode` enabled).
#[cfg(feature = "video-decode")]
async fn get_duration_ffprobe(path: &Path) -> Result<f64, ThumbError> {
    let output = tokio::process::Command::new("ffprobe")
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

/// Check if a `MediaKind` is an image file.
///
/// Since `detect_from_path` maps image extensions to `Unknown`, we check the
/// actual extension via `is_image_path` for a reliable result.
impl detect::MediaKind {
    pub fn is_image(self) -> bool {
        matches!(
            self,
            Self::Unknown
        )
    }
}

/// Check if a file path looks like an image by extension.
fn is_image_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| {
            matches!(
                ext.to_lowercase().as_str(),
                "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tiff" | "tif" | "ico"
                    | "svg" | "avif" | "heic" | "heif"
            )
        })
        .unwrap_or(false)
}

#[derive(Debug, thiserror::Error)]
pub enum ThumbError {
    #[error("command error: {0}")]
    Command(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("load error: {0}")]
    Load(String),
    #[error("encode error: {0}")]
    Encode(String),
    #[error("empty output")]
    Empty,
    #[error("no thumbnail found")]
    NoThumbnail,
    #[error("unsupported: {0}")]
    Unsupported(String),
}
