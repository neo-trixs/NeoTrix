//! Unified URL router — single source of truth for URL→Transport mapping.
//!
//! Replaces scattered URL parsing in:
//! - `nt_io_download.rs:UrlScheme`
//! - `nt_stream.rs:UrlScheme`
//! - `nt_world_video_pipeline.rs:detect_kind()`
//!
//! All URL handling flows through this router.

use std::path::PathBuf;

/// URL protocol scheme — single fact source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UrlScheme {
    Http,
    Https,
    Magnet,
    Ftp,
    File,
    Data,
    Unknown,
}

impl UrlScheme {
    pub fn parse(url: &str) -> Self {
        if url.starts_with("magnet:") { Self::Magnet }
        else if url.starts_with("https://") { Self::Https }
        else if url.starts_with("http://") { Self::Http }
        else if url.starts_with("ftp://") { Self::Ftp }
        else if url.starts_with("file://") { Self::File }
        else if url.starts_with("data:") { Self::Data }
        else { Self::Unknown }
    }

    /// Is this a network URL (requires download)?
    pub fn is_network(self) -> bool {
        matches!(self, Self::Http | Self::Https | Self::Ftp | Self::Magnet)
    }

    /// Is this a local file?
    pub fn is_local(self) -> bool {
        matches!(self, Self::File | Self::Unknown) // Unknown = treat as local path
    }
}

/// Transport type — what backend handles this URL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    /// HTTP Range parallel download (DownloadEngine)
    HttpRange,
    /// Sequential streaming download (StreamDownload)
    HttpStream,
    /// aria2c JSON-RPC for magnet/torrent
    MagnetRpc,
    /// Direct file copy
    FileCopy,
    /// FIFO pipe (zero disk I/O)
    FifoPipe,
}

/// Media route — full routing decision for a URL
#[derive(Debug, Clone)]
pub struct MediaRoute {
    pub scheme: UrlScheme,
    pub transport: TransportType,
    pub is_huggingface: bool,
    pub filename: Option<String>,
    pub output: PathBuf,
}

/// Route a URL to the appropriate transport.
///
/// Routing rules:
/// - `magnet:*` → MagnetRpc (aria2c RPC)
/// - `http(s)://huggingface.co/*` → HttpRange (parallel, mirror-accelerated)
/// - `http(s)://*.m3u8` → HttpStream (HLS, sequential)
/// - `http(s)://*` → HttpRange (default: parallel download)
/// - `file://*` or local path → FileCopy
/// - `ftp://*` → HttpRange (via reqwest FTP)
pub fn route_url(
    url: &str,
    output_dir: &PathBuf,
    prefer_streaming: bool,
) -> MediaRoute {
    let scheme = UrlScheme::parse(url);
    let is_hf = is_huggingface_url(url);

    let transport = match scheme {
        UrlScheme::Magnet => TransportType::MagnetRpc,

        UrlScheme::Http | UrlScheme::Https => {
            if is_hf {
                // HuggingFace: always parallel download (large files)
                TransportType::HttpRange
            } else if prefer_streaming {
                // Streaming requested: sequential download
                TransportType::HttpStream
            } else {
                // Default: parallel download
                TransportType::HttpRange
            }
        }

        UrlScheme::Ftp => TransportType::HttpRange, // reqwest handles FTP
        UrlScheme::File => TransportType::FileCopy,
        UrlScheme::Data => TransportType::FileCopy,
        UrlScheme::Unknown => TransportType::FileCopy, // local path
    };

    let filename = extract_filename(url);

    let output = if let Some(ref name) = filename {
        output_dir.join(name)
    } else {
        output_dir.join("download")
    };

    MediaRoute {
        scheme,
        transport,
        is_huggingface: is_hf,
        filename,
        output,
    }
}

/// Check if URL is HuggingFace
pub fn is_huggingface_url(url: &str) -> bool {
    url.contains("huggingface.co") || url.contains("hf-mirror.com")
}

/// Extract filename from URL (handles Content-Disposition separately)
pub fn extract_filename(url: &str) -> Option<String> {
    // Remove query params and fragment
    let path = url.split('?').next().unwrap_or(url);
    let path = path.split('#').next().unwrap_or(path);

    // Get last path segment
    let name = path.rsplit('/').next()?;

    if name.is_empty() { return None; }

    // URL-decode
    let decoded = urlencoding::decode(name).ok()?.into_owned();

    Some(decoded)
}

/// Extract filename from Content-Disposition header.
pub fn parse_content_disposition(cd: &str) -> Option<String> {
    // UTF-8'' encoded filename
    if let Some(pos) = cd.find("filename*=UTF-8''") {
        let encoded = &cd[pos + 16..];
        if let Some(name) = encoded.split(';').next() {
            if let Ok(decoded) = urlencoding::decode(name) {
                return Some(decoded.into_owned());
            }
        }
    }
    // RFC 2616 filename="..."
    if let Some(start) = cd.find("filename=\"") {
        let rest = &cd[start + 10..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    // filename=... (no quotes)
    if let Some(start) = cd.find("filename=") {
        let rest = &cd[start + 9..];
        let name = rest.split(';').next()?.trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheme_parsing() {
        assert_eq!(UrlScheme::parse("https://example.com/file.mp4"), UrlScheme::Https);
        assert_eq!(UrlScheme::parse("http://example.com/file.mp4"), UrlScheme::Http);
        assert_eq!(UrlScheme::parse("magnet:?xt=urn:btih:abc"), UrlScheme::Magnet);
        assert_eq!(UrlScheme::parse("ftp://server/file.bin"), UrlScheme::Ftp);
        assert_eq!(UrlScheme::parse("file:///tmp/test.bin"), UrlScheme::File);
        assert_eq!(UrlScheme::parse("/tmp/local.bin"), UrlScheme::Unknown);
    }

    #[test]
    fn test_route_huggingface() {
        let route = route_url(
            "https://huggingface.co/model.gguf/resolve/main/model.gguf",
            &PathBuf::from("/tmp"),
            false,
        );
        assert_eq!(route.transport, TransportType::HttpRange);
        assert!(route.is_huggingface);
    }

    #[test]
    fn test_route_magnet() {
        let route = route_url(
            "magnet:?xt=urn:btih:abc123&dn=model",
            &PathBuf::from("/tmp"),
            false,
        );
        assert_eq!(route.transport, TransportType::MagnetRpc);
    }

    #[test]
    fn test_route_streaming() {
        let route = route_url(
            "https://example.com/video.mp4",
            &PathBuf::from("/tmp"),
            true,
        );
        assert_eq!(route.transport, TransportType::HttpStream);
    }

    #[test]
    fn test_extract_filename() {
        assert_eq!(
            extract_filename("https://example.com/model.gguf?token=abc"),
            Some("model.gguf".to_string())
        );
        assert_eq!(
            extract_filename("https://example.com/path/to/file.bin"),
            Some("file.bin".to_string())
        );
    }

    #[test]
    fn test_content_disposition() {
        assert_eq!(
            parse_content_disposition("attachment; filename*=UTF-8''model.gguf"),
            Some("model.gguf".to_string())
        );
        assert_eq!(
            parse_content_disposition("attachment; filename=\"model.gguf\""),
            Some("model.gguf".to_string())
        );
        assert_eq!(
            parse_content_disposition("attachment; filename=model.gguf; size=123"),
            Some("model.gguf".to_string())
        );
    }
}
