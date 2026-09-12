//! HTTP-native video URL extraction — resolves video page URLs to direct download URLs.
//!
//! No external binaries. Uses oEmbed, site-specific APIs, and OpenGraph meta tags.

use regex::Regex;
use serde::Deserialize;

/// Extracted video info from HTTP APIs.
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
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("extraction failed: {0}")]
    Extraction(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("unsupported site: {0}")]
    Unsupported(String),
}

// ---------------------------------------------------------------------------
// oEmbed response
// ---------------------------------------------------------------------------
#[derive(Deserialize)]
struct OEmbedResponse {
    title: Option<String>,
    thumbnail_url: Option<String>,
    #[serde(rename = "type")]
    oembed_type: Option<String>,
}

async fn fetch_oembed(url: &str, client: &reqwest::Client) -> Option<OEmbedResponse> {
    let oembed_url = format!("https://oembed?url={}", urlencoding::encode(url));
    client
        .get(&oembed_url)
        .send()
        .await
        .ok()?
        .json::<OEmbedResponse>()
        .await
        .ok()
}

// ---------------------------------------------------------------------------
// HTML meta tag extraction
// ---------------------------------------------------------------------------
fn extract_meta(html: &str, property: &str) -> Option<String> {
    let re = Regex::new(&format!(
        r#"<meta[^>]+(?:property|name)=["']{}["'][^>]+content=["']([^"']+)["']"#,
        regex::escape(property)
    ))
    .ok()?;
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

fn extract_og_video(html: &str) -> Option<String> {
    extract_meta(html, "og:video").or_else(|| extract_meta(html, "og:video:url"))
}

fn extract_og_title(html: &str) -> Option<String> {
    extract_meta(html, "og:title")
}

fn extract_og_image(html: &str) -> Option<String> {
    extract_meta(html, "og:image")
}

// ---------------------------------------------------------------------------
// YouTube extraction
// ---------------------------------------------------------------------------
fn extract_video_id(url: &str) -> Option<String> {
    // Standard: https://www.youtube.com/watch?v=VIDEO_ID
    if let Some(id) = url.split("v=").nth(1) {
        let id = id.split('&').next().unwrap_or(id);
        if !id.is_empty() {
            return Some(id.to_string());
        }
    }
    // Short: https://youtu.be/VIDEO_ID
    if let Some(path) = url.strip_prefix("https://youtu.be/") {
        let id = path.split('?').next().unwrap_or(path);
        if !id.is_empty() {
            return Some(id.to_string());
        }
    }
    // Embedded: https://www.youtube.com/embed/VIDEO_ID
    if let Some(rest) = url.strip_prefix("https://www.youtube.com/embed/") {
        let id = rest.split('?').next().unwrap_or(rest);
        if !id.is_empty() {
            return Some(id.to_string());
        }
    }
    None
}

async fn extract_youtube(
    video_url: &str,
    client: &reqwest::Client,
) -> Result<VideoInfo, YtError> {
    let video_id =
        extract_video_id(video_url).ok_or_else(|| YtError::Extraction("invalid YouTube URL".into()))?;

    // 1. Try oEmbed for metadata
    let mut title = String::new();
    let mut thumbnail = None;
    if let Some(oembed) = fetch_oembed(video_url, client).await {
        title = oembed.title.unwrap_or_default();
        thumbnail = oembed.thumbnail_url;
    }

    // 2. Fetch page HTML to extract player response
    let page_url = format!("https://www.youtube.com/watch?v={}", video_id);
    let html = client
        .get(&page_url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .send()
        .await?
        .text()
        .await?;

    // Try to find ytInitialPlayerResponse
    let re = Regex::new(r"ytInitialPlayerResponse\s*=\s*(\{.+?\})\s*;")
        .map_err(|e| YtError::Parse(e.to_string()))?;
    let player_json = re
        .captures(&html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str());

    if let Some(json_str) = player_json {
        let v: serde_json::Value =
            serde_json::from_str(json_str).map_err(|e| YtError::Parse(e.to_string()))?;

        // Extract streaming data
        if let Some(streaming) = v.get("streamingData") {
            // Try adaptive formats first (higher quality)
            if let Some(formats) = streaming.get("adaptiveFormats").and_then(|f| f.as_array()) {
                if let Some(best) = formats.iter().find(|f| {
                    f.get("mimeType")
                        .and_then(|m| m.as_str())
                        .map(|m| m.starts_with("video/"))
                        .unwrap_or(false)
                        && f.get("url").is_some()
                }) {
                    let url = best["url"].as_str().unwrap_or("").to_string();
                    if !url.is_empty() {
                        return Ok(VideoInfo {
                            title: if title.is_empty() {
                                v.get("videoDetails")
                                    .and_then(|d| d.get("title"))
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("unknown")
                                    .to_string()
                            } else {
                                title
                            },
                            url,
                            format_id: best["itag"].as_i64().map(|i| i.to_string()),
                            ext: best["mimeType"]
                                .as_str()
                                .and_then(|m| m.split(';').next())
                                .and_then(|m| m.split('/').last())
                                .map(String::from),
                            duration: v
                                .get("videoDetails")
                                .and_then(|d| d.get("lengthSeconds"))
                                .and_then(|s| s.as_str())
                                .and_then(|s| s.parse::<f64>().ok()),
                            resolution: best["qualityLabel"].as_str().map(String::from),
                            filesize: best.get("contentLength").and_then(|s| {
                                s.as_str().and_then(|s| s.parse::<u64>().ok())
                            }),
                            webpage_url: video_url.to_string(),
                            thumbnail: thumbnail.or_else(|| {
                                v.get("videoDetails")
                                    .and_then(|d| d.get("thumbnail"))
                                    .and_then(|t| t.get("thumbnails"))
                                    .and_then(|t| t.as_array())
                                    .and_then(|t| t.last())
                                    .and_then(|t| t.get("url"))
                                    .and_then(|u| u.as_str())
                                    .map(String::from)
                            }),
                        });
                    }
                }
            }

            // Fallback to regular formats
            if let Some(formats) = streaming.get("formats").and_then(|f| f.as_array()) {
                if let Some(best) = formats
                    .iter()
                    .rev()
                    .find(|f| f.get("url").is_some())
                {
                    let url = best["url"].as_str().unwrap_or("").to_string();
                    if !url.is_empty() {
                        return Ok(VideoInfo {
                            title: if title.is_empty() {
                                v.get("videoDetails")
                                    .and_then(|d| d.get("title"))
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("unknown")
                                    .to_string()
                            } else {
                                title
                            },
                            url,
                            format_id: best["itag"].as_i64().map(|i| i.to_string()),
                            ext: best["mimeType"]
                                .as_str()
                                .and_then(|m| m.split(';').next())
                                .and_then(|m| m.split('/').last())
                                .map(String::from),
                            duration: v
                                .get("videoDetails")
                                .and_then(|d| d.get("lengthSeconds"))
                                .and_then(|s| s.as_str())
                                .and_then(|s| s.parse::<f64>().ok()),
                            resolution: best["qualityLabel"].as_str().map(String::from),
                            filesize: best.get("contentLength").and_then(|s| {
                                s.as_str().and_then(|s| s.parse::<u64>().ok())
                            }),
                            webpage_url: video_url.to_string(),
                            thumbnail: thumbnail.or_else(|| {
                                v.get("videoDetails")
                                    .and_then(|d| d.get("thumbnail"))
                                    .and_then(|t| t.get("thumbnails"))
                                    .and_then(|t| t.as_array())
                                    .and_then(|t| t.last())
                                    .and_then(|t| t.get("url"))
                                    .and_then(|u| u.as_str())
                                    .map(String::from)
                            }),
                        });
                    }
                }
            }
        }

        // Return metadata even without stream URL (playback may require signature)
        return Ok(VideoInfo {
            title: if title.is_empty() {
                v.get("videoDetails")
                    .and_then(|d| d.get("title"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("unknown")
                    .to_string()
            } else {
                title
            },
            url: String::new(),
            format_id: None,
            ext: None,
            duration: v
                .get("videoDetails")
                .and_then(|d| d.get("lengthSeconds"))
                .and_then(|s| s.as_str())
                .and_then(|s| s.parse::<f64>().ok()),
            resolution: None,
            filesize: None,
            webpage_url: video_url.to_string(),
            thumbnail: thumbnail.or_else(|| {
                v.get("videoDetails")
                    .and_then(|d| d.get("thumbnail"))
                    .and_then(|t| t.get("thumbnails"))
                    .and_then(|t| t.as_array())
                    .and_then(|t| t.last())
                    .and_then(|t| t.get("url"))
                    .and_then(|u| u.as_str())
                    .map(String::from)
            }),
        });
    }

    Err(YtError::Extraction(
        "could not extract ytInitialPlayerResponse from YouTube page".into(),
    ))
}

// ---------------------------------------------------------------------------
// Bilibili extraction
// ---------------------------------------------------------------------------
fn extract_bvid(url: &str) -> Option<String> {
    // https://www.bilibili.com/video/BV1xx411c7mD
    let re = Regex::new(r"(BV[a-zA-Z0-9]+)").ok()?;
    re.captures(url).and_then(|c| c.get(1)).map(|m| m.as_str().to_string())
}

async fn extract_bilibili(
    video_url: &str,
    client: &reqwest::Client,
) -> Result<VideoInfo, YtError> {
    let bvid =
        extract_bvid(video_url).ok_or_else(|| YtError::Extraction("invalid Bilibili URL".into()))?;

    let api_url = format!("https://api.bilibili.com/x/web-interface/view?bvid={}", bvid);
    let resp: serde_json::Value = client
        .get(&api_url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .send()
        .await?
        .json()
        .await?;

    let data = resp
        .get("data")
        .ok_or_else(|| YtError::Extraction("missing data field in Bilibili response".into()))?;

    let title = data["title"].as_str().unwrap_or("unknown").to_string();
    let duration = data["duration"].as_f64();
    let thumbnail = data["pic"].as_str().map(String::from);
    let cid = data["cid"].as_i64();
    let aid = data["aid"].as_i64();

    // Try to get playurl for direct streaming URL
    if let (Some(aid_val), Some(cid_val)) = (aid, cid) {
        let play_url = format!(
            "https://api.bilibili.com/x/player/playurl?avid={}&cid={}&qn=64&fnval=1",
            aid_val, cid_val
        );
        if let Ok(play_resp) = client
            .get(&play_url)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
            .send()
            .await
        {
            if let Ok(play_json) = play_resp.json::<serde_json::Value>().await {
                if let Some(durl) = play_json
                    .get("data")
                    .and_then(|d| d.get("durl"))
                    .and_then(|d| d.as_array())
                    .and_then(|d| d.first())
                {
                    let url = durl["url"].as_str().unwrap_or("").to_string();
                    if !url.is_empty() {
                        return Ok(VideoInfo {
                            title,
                            url,
                            format_id: Some("durl-0".into()),
                            ext: Some("flv".into()),
                            duration,
                            resolution: None,
                            filesize: durl["size"].as_u64(),
                            webpage_url: video_url.to_string(),
                            thumbnail,
                        });
                    }
                }
            }
        }
    }

    Ok(VideoInfo {
        title,
        url: String::new(),
        format_id: None,
        ext: None,
        duration,
        resolution: None,
        filesize: None,
        webpage_url: video_url.to_string(),
        thumbnail,
    })
}

// ---------------------------------------------------------------------------
// Generic fallback: OpenGraph extraction
// ---------------------------------------------------------------------------
async fn extract_generic(
    video_url: &str,
    client: &reqwest::Client,
) -> Result<VideoInfo, YtError> {
    let html = client
        .get(video_url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .send()
        .await?
        .text()
        .await?;

    let url = extract_og_video(&html).unwrap_or_default();
    let title = extract_og_title(&html).unwrap_or_else(|| "unknown".into());
    let thumbnail = extract_og_image(&html);

    Ok(VideoInfo {
        title,
        url,
        format_id: None,
        ext: None,
        duration: None,
        resolution: None,
        filesize: None,
        webpage_url: video_url.to_string(),
        thumbnail,
    })
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------
fn detect_site(url: &str) -> &str {
    if url.contains("youtube.com") || url.contains("youtu.be") {
        "youtube"
    } else if url.contains("bilibili.com") {
        "bilibili"
    } else {
        "generic"
    }
}

/// Extract direct URL from video page using HTTP APIs only (no yt-dlp binary).
/// Tries oEmbed, site-specific APIs, then OpenGraph meta tags.
pub async fn extract_url(video_url: &str) -> Result<VideoInfo, YtError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;

    match detect_site(video_url) {
        "youtube" => extract_youtube(video_url, &client).await,
        "bilibili" => extract_bilibili(video_url, &client).await,
        _ => extract_generic(video_url, &client).await,
    }
}

/// List available formats for a video URL.
/// Returns structured format list from site APIs where available.
pub async fn list_formats(video_url: &str) -> Result<Vec<FormatInfo>, YtError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;

    match detect_site(video_url) {
        "youtube" => list_youtube_formats(video_url, &client).await,
        _ => Ok(Vec::new()),
    }
}

async fn list_youtube_formats(
    video_url: &str,
    client: &reqwest::Client,
) -> Result<Vec<FormatInfo>, YtError> {
    let video_id = extract_video_id(video_url)
        .ok_or_else(|| YtError::Extraction("invalid YouTube URL".into()))?;

    let page_url = format!("https://www.youtube.com/watch?v={}", video_id);
    let html = client
        .get(&page_url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .send()
        .await?
        .text()
        .await?;

    let re = Regex::new(r"ytInitialPlayerResponse\s*=\s*(\{.+?\})\s*;")
        .map_err(|e| YtError::Parse(e.to_string()))?;
    let json_str = re
        .captures(&html)
        .and_then(|c| c.get(1))
        .ok_or_else(|| YtError::Extraction("no player response found".into()))?;

    let v: serde_json::Value =
        serde_json::from_str(json_str.as_str()).map_err(|e| YtError::Parse(e.to_string()))?;

    let mut formats = Vec::new();

    if let Some(streaming) = v.get("streamingData") {
        // Adaptive formats
        if let Some(arr) = streaming.get("adaptiveFormats").and_then(|f| f.as_array()) {
            for f in arr {
                formats.push(FormatInfo {
                    format_id: f["itag"].as_i64().map(|i| i.to_string()).unwrap_or_default(),
                    ext: f["mimeType"]
                        .as_str()
                        .and_then(|m| m.split(';').next())
                        .and_then(|m| m.split('/').last())
                        .unwrap_or("")
                        .to_string(),
                    resolution: f["qualityLabel"].as_str().unwrap_or("").to_string(),
                    filesize: f.get("contentLength").and_then(|s| {
                        s.as_str().and_then(|s| s.parse::<u64>().ok())
                    }),
                    vcodec: f["mimeType"]
                        .as_str()
                        .map(|m| if m.starts_with("video/") { "video" } else { "" })
                        .unwrap_or("")
                        .to_string(),
                    acodec: f["mimeType"]
                        .as_str()
                        .map(|m| if m.starts_with("audio/") { "audio" } else { "" })
                        .unwrap_or("")
                        .to_string(),
                });
            }
        }

        // Muxed formats
        if let Some(arr) = streaming.get("formats").and_then(|f| f.as_array()) {
            for f in arr {
                formats.push(FormatInfo {
                    format_id: f["itag"].as_i64().map(|i| i.to_string()).unwrap_or_default(),
                    ext: f["mimeType"]
                        .as_str()
                        .and_then(|m| m.split(';').next())
                        .and_then(|m| m.split('/').last())
                        .unwrap_or("")
                        .to_string(),
                    resolution: f["qualityLabel"].as_str().unwrap_or("").to_string(),
                    filesize: f.get("contentLength").and_then(|s| {
                        s.as_str().and_then(|s| s.parse::<u64>().ok())
                    }),
                    vcodec: "video".to_string(),
                    acodec: "audio".to_string(),
                });
            }
        }
    }

    Ok(formats)
}
