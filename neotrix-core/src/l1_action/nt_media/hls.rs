//! HLS m3u8 manifest parser — resolves m3u8 playlists to downloadable segment URLs.
//!
//! Supports:
//! - Master playlists (variant streams with different qualities)
//! - Media playlists (segment lists with EXTINF durations)
//! - Byte-range requests (EXT-X-BYTERANGE)
//! - AES-128 encryption keys (EXT-X-KEY)

use std::time::Duration;

/// Parsed m3u8 manifest
#[derive(Debug, Clone)]
pub enum M3u8Manifest {
    Master(MasterPlaylist),
    Media(MediaPlaylist),
}

/// Master playlist — multiple quality variants
#[derive(Debug, Clone)]
pub struct MasterPlaylist {
    pub version: Option<u32>,
    pub variants: Vec<VariantStream>,
}

#[derive(Debug, Clone)]
pub struct VariantStream {
    pub uri: String,
    pub bandwidth: u64,
    pub resolution: Option<String>,
    pub codecs: Option<String>,
    pub name: Option<String>,
}

/// Media playlist — segment list
#[derive(Debug, Clone)]
pub struct MediaPlaylist {
    pub version: Option<u32>,
    pub target_duration: Option<Duration>,
    pub media_sequence: u64,
    pub segments: Vec<Segment>,
    pub encryption: Option<Encryption>,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub uri: String,
    pub duration: Duration,
    pub byte_range: Option<ByteRange>,
    pub title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ByteRange {
    pub length: u64,
    pub offset: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Encryption {
    pub method: String,
    pub uri: Option<String>,
    pub iv: Option<String>,
}

/// Parse m3u8 content
pub fn parse_m3u8(content: &str) -> Result<M3u8Manifest, HlsError> {
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() || !lines[0].trim().eq_ignore_ascii_case("#EXTM3U") {
        return Err(HlsError::MissingHeader);
    }

    let mut version: Option<u32> = None;
    let mut target_duration: Option<Duration> = None;
    let mut media_sequence: u64 = 0;
    let mut current_encryption: Option<Encryption> = None;
    let mut segments: Vec<Segment> = Vec::new();
    let mut variants: Vec<VariantStream> = Vec::new();

    let mut is_master = false;
    let mut pending_duration: Option<Duration> = None;
    let mut pending_title: Option<String> = None;
    let mut pending_byte_range: Option<ByteRange> = None;

    for line in &lines[1..] {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("#EXT-X-STREAM-INF:") {
            is_master = true;
            let attrs = parse_attributes(line.trim_start_matches("#EXT-X-STREAM-INF:"));
            variants.push(VariantStream {
                uri: String::new(),
                bandwidth: attrs
                    .get("BANDWIDTH")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
                resolution: attrs.get("RESOLUTION").map(|s| s.to_string()),
                codecs: attrs.get("CODECS").map(|s| s.trim_matches('"').to_string()),
                name: attrs.get("NAME").map(|s| s.trim_matches('"').to_string()),
                ..VariantStream { uri: String::new() }
            });
            continue;
        }

        if line.starts_with("#EXT-X-VERSION:") {
            version = line.trim_start_matches("#EXT-X-VERSION:").parse().ok();
            continue;
        }

        if line.starts_with("#EXT-X-TARGETDURATION:") {
            target_duration = line
                .trim_start_matches("#EXT-X-TARGETDURATION:")
                .parse::<f64>()
                .ok()
                .map(Duration::from_secs_f64);
            continue;
        }

        if line.starts_with("#EXT-X-MEDIA-SEQUENCE:") {
            media_sequence = line
                .trim_start_matches("#EXT-X-MEDIA-SEQUENCE:")
                .parse()
                .unwrap_or(0);
            continue;
        }

        if line.starts_with("#EXT-X-KEY:") {
            let attrs = parse_attributes(line.trim_start_matches("#EXT-X-KEY:"));
            current_encryption = Some(Encryption {
                method: attrs.get("METHOD").unwrap_or(&"NONE").to_string(),
                uri: attrs.get("URI").map(|s| s.trim_matches('"').to_string()),
                iv: attrs.get("IV").map(|s| s.to_string()),
            });
            continue;
        }

        if line.starts_with("#EXTINF:") {
            let rest = line.trim_start_matches("#EXTINF:");
            let parts: Vec<&str> = rest.splitn(2, ',').collect();
            pending_duration = parts[0]
                .trim()
                .parse::<f64>()
                .ok()
                .map(Duration::from_secs_f64);
            pending_title = if parts.len() > 1 {
                Some(parts[1].trim().to_string())
            } else {
                None
            };
            continue;
        }

        if line.starts_with("#EXT-X-BYTERANGE:") {
            let rest = line.trim_start_matches("#EXT-X-BYTERANGE:");
            let parts: Vec<&str> = rest.splitn(2, '@').collect();
            let length = parts[0].trim().parse::<u64>().unwrap_or(0);
            let offset = if parts.len() > 1 {
                parts[1].trim().parse::<u64>().ok()
            } else {
                None
            };
            pending_byte_range = Some(ByteRange { length, offset });
            continue;
        }

        if line.starts_with("#EXT-X-ENDLIST") {
            continue;
        }

        if line.starts_with('#') {
            continue;
        }

        if is_master {
            if let Some(last) = variants.last_mut() {
                last.uri = line.to_string();
            }
            continue;
        }

        let duration = pending_duration.take().unwrap_or(Duration::ZERO);
        let title = pending_title.take();
        let byte_range = pending_byte_range.take();

        segments.push(Segment {
            uri: line.to_string(),
            duration,
            byte_range,
            title,
        });
    }

    if is_master {
        Ok(M3u8Manifest::Master(MasterPlaylist { version, variants }))
    } else {
        Ok(M3u8Manifest::Media(MediaPlaylist {
            version,
            target_duration,
            media_sequence,
            segments,
            encryption: current_encryption,
        }))
    }
}

fn parse_attributes(tag_content: &str) -> std::collections::HashMap<String, String> {
    let mut attrs = std::collections::HashMap::new();
    for part in tag_content.split(',') {
        let part = part.trim();
        if let Some((key, value)) = part.split_once('=') {
            attrs.insert(key.trim().to_uppercase(), value.trim().to_string());
        }
    }
    attrs
}

/// Select best variant based on preferred bandwidth
pub fn select_variant<'a>(
    master: &'a MasterPlaylist,
    preferred_bandwidth: Option<u64>,
) -> Option<&'a VariantStream> {
    if master.variants.is_empty() {
        return None;
    }

    match preferred_bandwidth {
        Some(bw) => master
            .variants
            .iter()
            .min_by_key(|v| (v.bandwidth as i64 - bw as i64).unsigned_abs()),
        None => master.variants.iter().max_by_key(|v| v.bandwidth),
    }
}

/// Convert manifest to download URLs
pub fn to_download_urls(manifest: &M3u8Manifest, base_url: &str) -> Vec<String> {
    let base = base_url
        .rsplit_once('/')
        .map(|(b, _)| b)
        .unwrap_or(base_url);
    match manifest {
        M3u8Manifest::Master(master) => {
            let mut urls = Vec::new();
            for variant in &master.variants {
                urls.push(resolve_uri(base, &variant.uri));
            }
            urls
        }
        M3u8Manifest::Media(media) => {
            let mut urls = Vec::new();
            for seg in &media.segments {
                urls.push(resolve_uri(base, &seg.uri));
            }
            urls
        }
    }
}

fn resolve_uri(base: &str, uri: &str) -> String {
    if uri.starts_with("http://") || uri.starts_with("https://") {
        return uri.to_string();
    }
    if uri.starts_with('/') {
        return uri.to_string();
    }
    format!("{}/{}", base, uri)
}

#[derive(Debug, thiserror::Error)]
pub enum HlsError {
    #[error("missing #EXTM3U header")]
    MissingHeader,
    #[error("invalid tag: {0}")]
    InvalidTag(String),
    #[error("parse error: {0}")]
    Parse(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_master_playlist() {
        let content = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-STREAM-INF:BANDWIDTH=800000,RESOLUTION=640x360,CODECS="avc1.42e00a,mp4a.40.2",NAME="Low"
low/low.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=2000000,RESOLUTION=1280x720,CODECS="avc1.4d401f,mp4a.40.2",NAME="Medium"
mid/mid.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=5000000,RESOLUTION=1920x1080,CODECS="avc1.640028,mp4a.40.2",NAME="High"
high/high.m3u8
"#;
        let manifest = parse_m3u8(content).unwrap();
        match manifest {
            M3u8Manifest::Master(master) => {
                assert_eq!(master.version, Some(3));
                assert_eq!(master.variants.len(), 3);
                assert_eq!(master.variants[0].bandwidth, 800000);
                assert_eq!(master.variants[0].resolution, Some("640x360".to_string()));
                assert_eq!(master.variants[0].name, Some("Low".to_string()));
                assert_eq!(master.variants[1].uri, "mid/mid.m3u8");
                assert_eq!(master.variants[2].bandwidth, 5000000);
            }
            _ => panic!("Expected master playlist"),
        }
    }

    #[test]
    fn test_parse_media_playlist() {
        let content = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:10
#EXT-X-MEDIA-SEQUENCE:0
#EXT-X-KEY:METHOD=AES-128,URI="https://example.com/key",IV=0x00000000000000000000000000000001
#EXTINF:9.009,
http://media.example.com/first.ts
#EXTINF:9.009,
http://media.example.com/second.ts
#EXTINF:3.003,
http://media.example.com/third.ts
#EXT-X-ENDLIST
"#;
        let manifest = parse_m3u8(content).unwrap();
        match manifest {
            M3u8Manifest::Media(media) => {
                assert_eq!(media.version, Some(3));
                assert_eq!(media.target_duration, Some(Duration::from_secs(10)));
                assert_eq!(media.media_sequence, 0);
                assert_eq!(media.segments.len(), 3);
                assert_eq!(media.segments[0].duration, Duration::from_secs_f64(9.009));
                assert_eq!(media.segments[0].uri, "http://media.example.com/first.ts");
                assert!(media.encryption.is_some());
                let enc = media.encryption.unwrap();
                assert_eq!(enc.method, "AES-128");
                assert_eq!(enc.uri, Some("https://example.com/key".to_string()));
            }
            _ => panic!("Expected media playlist"),
        }
    }

    #[test]
    fn test_parse_empty_content() {
        assert!(parse_m3u8("").is_err());
    }

    #[test]
    fn test_parse_missing_header() {
        assert!(parse_m3u8("#EXT-X-VERSION:3\n").is_err());
    }

    #[test]
    fn test_select_variant_preferred() {
        let master = MasterPlaylist {
            version: None,
            variants: vec![
                VariantStream {
                    uri: "low.m3u8".into(),
                    bandwidth: 800000,
                    resolution: None,
                    codecs: None,
                    name: None,
                },
                VariantStream {
                    uri: "high.m3u8".into(),
                    bandwidth: 5000000,
                    resolution: None,
                    codecs: None,
                    name: None,
                },
            ],
        };
        let selected = select_variant(&master, Some(1000000)).unwrap();
        assert_eq!(selected.uri, "low.m3u8");
    }

    #[test]
    fn test_select_variant_best() {
        let master = MasterPlaylist {
            version: None,
            variants: vec![
                VariantStream {
                    uri: "low.m3u8".into(),
                    bandwidth: 800000,
                    resolution: None,
                    codecs: None,
                    name: None,
                },
                VariantStream {
                    uri: "high.m3u8".into(),
                    bandwidth: 5000000,
                    resolution: None,
                    codecs: None,
                    name: None,
                },
            ],
        };
        let selected = select_variant(&master, None).unwrap();
        assert_eq!(selected.uri, "high.m3u8");
    }

    #[test]
    fn test_to_download_urls_media() {
        let media = MediaPlaylist {
            version: None,
            target_duration: None,
            media_sequence: 0,
            segments: vec![
                Segment {
                    uri: "seg1.ts".into(),
                    duration: Duration::from_secs(9),
                    byte_range: None,
                    title: None,
                },
                Segment {
                    uri: "/absolute/seg2.ts".into(),
                    duration: Duration::from_secs(9),
                    byte_range: None,
                    title: None,
                },
            ],
            encryption: None,
        };
        let urls = to_download_urls(
            &M3u8Manifest::Media(media),
            "https://example.com/path/playlist.m3u8",
        );
        assert_eq!(urls[0], "https://example.com/path/seg1.ts");
        assert_eq!(urls[1], "/absolute/seg2.ts");
    }

    #[test]
    fn test_to_download_urls_master() {
        let master = MasterPlaylist {
            version: None,
            variants: vec![VariantStream {
                uri: "low/low.m3u8".into(),
                bandwidth: 800000,
                resolution: None,
                codecs: None,
                name: None,
            }],
        };
        let urls = to_download_urls(
            &M3u8Manifest::Master(master),
            "https://example.com/master.m3u8",
        );
        assert_eq!(urls[0], "https://example.com/low/low.m3u8");
    }
}
