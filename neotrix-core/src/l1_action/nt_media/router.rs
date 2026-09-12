//! Unified URL router — single source of truth for URL→Transport mapping.

use std::path::PathBuf;

/// URL protocol scheme
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
        if url.starts_with("magnet:") {
            Self::Magnet
        } else if url.starts_with("https://") {
            Self::Https
        } else if url.starts_with("http://") {
            Self::Http
        } else if url.starts_with("ftp://") {
            Self::Ftp
        } else if url.starts_with("file://") {
            Self::File
        } else if url.starts_with("data:") {
            Self::Data
        } else {
            Self::Unknown
        }
    }
    pub fn is_network(self) -> bool {
        matches!(self, Self::Http | Self::Https | Self::Ftp | Self::Magnet)
    }
}

/// Transport type — what backend handles this URL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    HttpRange,
    HttpStream,
    MagnetRpc,
    FileCopy,
    FifoPipe,
}

/// Media route — full routing decision
#[derive(Debug, Clone)]
pub struct MediaRoute {
    pub scheme: UrlScheme,
    pub transport: TransportType,
    pub is_huggingface: bool,
    pub filename: Option<String>,
    pub output: PathBuf,
}

/// Route a URL to the appropriate transport
pub fn route_url(url: &str, output_dir: &PathBuf, prefer_streaming: bool) -> MediaRoute {
    let scheme = UrlScheme::parse(url);
    let is_hf = is_huggingface_url(url);

    let transport = match scheme {
        UrlScheme::Magnet => TransportType::MagnetRpc,
        UrlScheme::Http | UrlScheme::Https => {
            if is_hf {
                TransportType::HttpRange
            } else if prefer_streaming {
                TransportType::HttpStream
            } else {
                TransportType::HttpRange
            }
        }
        UrlScheme::Ftp => TransportType::HttpRange,
        UrlScheme::File => TransportType::FileCopy,
        UrlScheme::Data => TransportType::FileCopy,
        UrlScheme::Unknown => TransportType::FileCopy,
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

pub fn is_huggingface_url(url: &str) -> bool {
    url.contains("huggingface.co") || url.contains("hf-mirror.com")
}

pub fn extract_filename(url: &str) -> Option<String> {
    let path = url.split('?').next().unwrap_or(url);
    let path = path.split('#').next().unwrap_or(path);
    let name = path.rsplit('/').next()?;
    if name.is_empty() {
        return None;
    }
    Some(urlencoding::decode(name).ok()?.into_owned())
}

pub fn parse_content_disposition(cd: &str) -> Option<String> {
    if let Some(pos) = cd.find("filename*=UTF-8''") {
        let encoded = &cd[pos + 16..];
        if let Some(name) = encoded.split(';').next() {
            if let Ok(decoded) = urlencoding::decode(name) {
                return Some(decoded.into_owned());
            }
        }
    }
    if let Some(start) = cd.find("filename=\"") {
        let rest = &cd[start + 10..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
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
        assert_eq!(
            UrlScheme::parse("https://example.com/file.mp4"),
            UrlScheme::Https
        );
        assert_eq!(
            UrlScheme::parse("magnet:?xt=urn:btih:abc"),
            UrlScheme::Magnet
        );
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
    }
}
