//! Unified media type detection — single source of truth.
//!
//! Three-layer detection: magic bytes → Content-Type header → URL extension.

use std::path::Path;

/// Unified media classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaKind {
    AudioMp3,
    AudioFlac,
    AudioWav,
    AudioOgg,
    AudioAac,
    AudioM4a,
    AudioOpus,
    AudioUnknown,
    VideoMp4,
    VideoWebm,
    VideoMkv,
    VideoAvi,
    VideoMov,
    VideoHls,
    VideoDash,
    VideoUnknown,
    Pdf,
    Docx,
    Xlsx,
    Pptx,
    Markdown,
    Zip,
    Gzip,
    Tar,
    Xz,
    SevenZip,
    Gguf,
    Onnx,
    Safetensors,
    Pickle,
    Json,
    Csv,
    Unknown,
}

impl MediaKind {
    pub fn is_audio(self) -> bool {
        matches!(
            self,
            Self::AudioMp3
                | Self::AudioFlac
                | Self::AudioWav
                | Self::AudioOgg
                | Self::AudioAac
                | Self::AudioM4a
                | Self::AudioOpus
                | Self::AudioUnknown
        )
    }
    pub fn is_video(self) -> bool {
        matches!(
            self,
            Self::VideoMp4
                | Self::VideoWebm
                | Self::VideoMkv
                | Self::VideoAvi
                | Self::VideoMov
                | Self::VideoHls
                | Self::VideoDash
                | Self::VideoUnknown
        )
    }
    pub fn is_media(self) -> bool {
        self.is_audio() || self.is_video()
    }
    pub fn is_model(self) -> bool {
        matches!(
            self,
            Self::Gguf | Self::Onnx | Self::Safetensors | Self::Pickle
        )
    }
    pub fn is_document(self) -> bool {
        matches!(
            self,
            Self::Pdf | Self::Docx | Self::Xlsx | Self::Pptx | Self::Markdown
        )
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::AudioMp3 => "MP3",
            Self::AudioFlac => "FLAC",
            Self::AudioWav => "WAV",
            Self::AudioOgg => "OGG",
            Self::AudioAac => "AAC",
            Self::AudioM4a => "M4A",
            Self::AudioOpus => "Opus",
            Self::AudioUnknown => "Audio",
            Self::VideoMp4 => "MP4",
            Self::VideoWebm => "WebM",
            Self::VideoMkv => "MKV",
            Self::VideoAvi => "AVI",
            Self::VideoMov => "MOV",
            Self::VideoHls => "HLS",
            Self::VideoDash => "DASH",
            Self::VideoUnknown => "Video",
            Self::Pdf => "PDF",
            Self::Docx => "DOCX",
            Self::Xlsx => "XLSX",
            Self::Pptx => "PPTX",
            Self::Markdown => "Markdown",
            Self::Zip => "ZIP",
            Self::Gzip => "GZIP",
            Self::Tar => "TAR",
            Self::Xz => "XZ",
            Self::SevenZip => "7Z",
            Self::Gguf => "GGUF",
            Self::Onnx => "ONNX",
            Self::Safetensors => "SafeTensors",
            Self::Pickle => "Pickle",
            Self::Json => "JSON",
            Self::Csv => "CSV",
            Self::Unknown => "Unknown",
        }
    }
}

/// Detect from magic bytes (most reliable)
pub fn detect_from_bytes(data: &[u8]) -> MediaKind {
    if data.len() < 4 {
        return MediaKind::Unknown;
    }
    if data.starts_with(b"ID3")
        || data.starts_with(b"\xFF\xFB")
        || data.starts_with(b"\xFF\xF3")
        || data.starts_with(b"\xFF\xF2")
    {
        return MediaKind::AudioMp3;
    }
    if data.starts_with(b"fLaC") {
        return MediaKind::AudioFlac;
    }
    if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WAVE" {
        return MediaKind::AudioWav;
    }
    if data.starts_with(b"OggS") {
        return MediaKind::AudioOgg;
    }
    if data.len() > 12 && &data[4..8] == b"ftyp" {
        let brand = &data[8..12];
        if brand == b"qt  " || brand == b"mqt " {
            return MediaKind::VideoMov;
        }
        return MediaKind::VideoMp4;
    }
    if data[0] == 0x1A && data[1] == 0x45 && data[2] == 0xDF && data[3] == 0xA3 {
        return MediaKind::VideoWebm;
    }
    if data.starts_with(b"GGUF") {
        return MediaKind::Gguf;
    }
    if data.starts_with(b"%PDF") {
        return MediaKind::Pdf;
    }
    if data[0] == 0x1F && data[1] == 0x8B {
        return MediaKind::Gzip;
    }
    if data.starts_with(b"\xfd7zXZ") {
        return MediaKind::Xz;
    }
    if data.starts_with(b"7z\xBC\xAF\x27\x1C") {
        return MediaKind::SevenZip;
    }
    MediaKind::Unknown
}

/// Detect from HTTP Content-Type header
pub fn detect_from_content_type(ct: &str) -> MediaKind {
    let ct = ct.split(';').next().unwrap_or(ct).trim().to_lowercase();
    match ct.as_str() {
        "audio/mpeg" | "audio/mp3" => MediaKind::AudioMp3,
        "audio/flac" | "audio/x-flac" => MediaKind::AudioFlac,
        "audio/wav" | "audio/x-wav" => MediaKind::AudioWav,
        "audio/ogg" | "audio/vorbis" | "audio/opus" => MediaKind::AudioOgg,
        "audio/aac" | "audio/x-aac" => MediaKind::AudioAac,
        "audio/m4a" | "audio/mp4" => MediaKind::AudioM4a,
        "video/mp4" | "video/webm" | "video/x-matroska" | "video/avi" | "video/quicktime" => {
            MediaKind::VideoMp4
        }
        "application/x-mpegurl" | "application/vnd.apple.mpegurl" => MediaKind::VideoHls,
        "application/dash+xml" => MediaKind::VideoDash,
        "application/pdf" => MediaKind::Pdf,
        "application/zip" => MediaKind::Zip,
        "application/gzip" | "application/x-gzip" => MediaKind::Gzip,
        "application/json" | "text/json" => MediaKind::Json,
        "text/csv" => MediaKind::Csv,
        "text/markdown" | "text/x-markdown" => MediaKind::Markdown,
        _ => MediaKind::Unknown,
    }
}

/// Detect from URL extension
pub fn detect_from_url(url: &str) -> MediaKind {
    let path = url.split('?').next().unwrap_or(url);
    let path = path.split('#').next().unwrap_or(path);
    let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "mp3" => MediaKind::AudioMp3,
        "flac" => MediaKind::AudioFlac,
        "wav" => MediaKind::AudioWav,
        "ogg" | "opus" => MediaKind::AudioOgg,
        "aac" => MediaKind::AudioAac,
        "m4a" => MediaKind::AudioM4a,
        "mp4" | "m4v" => MediaKind::VideoMp4,
        "webm" => MediaKind::VideoWebm,
        "mkv" | "matroska" => MediaKind::VideoMkv,
        "avi" => MediaKind::VideoAvi,
        "mov" => MediaKind::VideoMov,
        "m3u8" => MediaKind::VideoHls,
        "mpd" => MediaKind::VideoDash,
        "pdf" => MediaKind::Pdf,
        "docx" => MediaKind::Docx,
        "xlsx" => MediaKind::Xlsx,
        "pptx" => MediaKind::Pptx,
        "md" | "markdown" => MediaKind::Markdown,
        "zip" => MediaKind::Zip,
        "gz" | "gzip" => MediaKind::Gzip,
        "tar" => MediaKind::Tar,
        "xz" => MediaKind::Xz,
        "7z" => MediaKind::SevenZip,
        "gguf" => MediaKind::Gguf,
        "onnx" => MediaKind::Onnx,
        "safetensors" | "st" => MediaKind::Safetensors,
        "pkl" | "pickle" => MediaKind::Pickle,
        "json" | "jsonl" => MediaKind::Json,
        "csv" | "tsv" => MediaKind::Csv,
        _ => MediaKind::Unknown,
    }
}

/// Three-layer detection: magic bytes → Content-Type → URL extension
pub fn detect(
    first_bytes: Option<&[u8]>,
    content_type: Option<&str>,
    url: Option<&str>,
) -> MediaKind {
    if let Some(bytes) = first_bytes {
        let kind = detect_from_bytes(bytes);
        if kind != MediaKind::Unknown {
            return kind;
        }
    }
    if let Some(ct) = content_type {
        let kind = detect_from_content_type(ct);
        if kind != MediaKind::Unknown {
            return kind;
        }
    }
    if let Some(url) = url {
        let kind = detect_from_url(url);
        if kind != MediaKind::Unknown {
            return kind;
        }
    }
    MediaKind::Unknown
}

/// Detect from file path (extension only)
pub fn detect_from_path(path: &Path) -> MediaKind {
    detect_from_url(&path.to_string_lossy())
}

/// Detect from file contents (reads first 8KB)
pub async fn detect_from_file(path: &Path) -> MediaKind {
    match tokio::fs::read(path).await {
        Ok(data) => {
            let first = &data[..data.len().min(8192)];
            detect_from_bytes(first)
        }
        Err(_) => detect_from_path(path),
    }
}

/// Probe URL headers and first bytes for detection
pub async fn detect_remote(client: &reqwest::Client, url: &str) -> (MediaKind, Option<String>) {
    if let Ok(resp) = client.head(url).send().await {
        let ct = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        if let Some(ref ct_val) = ct {
            let kind = detect_from_content_type(ct_val);
            if kind != MediaKind::Unknown {
                return (kind, ct.clone());
            }
        }
        let kind = detect_from_url(url);
        return (kind, ct);
    }
    (detect_from_url(url), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_bytes() {
        assert_eq!(detect_from_bytes(b"ID3\x04\x00\x00"), MediaKind::AudioMp3);
        assert_eq!(detect_from_bytes(b"fLaC"), MediaKind::AudioFlac);
        assert_eq!(
            detect_from_bytes(b"RIFF\x00\x00\x00\x00WAVE"),
            MediaKind::AudioWav
        );
        assert_eq!(detect_from_bytes(b"OggS"), MediaKind::AudioOgg);
        assert_eq!(detect_from_bytes(b"%PDF-1.4"), MediaKind::Pdf);
        assert_eq!(detect_from_bytes(b"GGUF"), MediaKind::Gguf);
        assert_eq!(detect_from_bytes(b"\x1F\x8B\x08"), MediaKind::Gzip);
        assert_eq!(detect_from_bytes(b"\x1A\x45\xDF\xA3"), MediaKind::VideoWebm);
    }

    #[test]
    fn test_mp4_ftyp() {
        let mut data = vec![0u8; 16];
        data[4..8].copy_from_slice(b"ftyp");
        data[8..12].copy_from_slice(b"isom");
        assert_eq!(detect_from_bytes(&data), MediaKind::VideoMp4);
    }

    #[test]
    fn test_content_type() {
        assert_eq!(detect_from_content_type("audio/mpeg"), MediaKind::AudioMp3);
        assert_eq!(
            detect_from_content_type("video/mp4; charset=utf-8"),
            MediaKind::VideoMp4
        );
        assert_eq!(
            detect_from_content_type("application/x-mpegurl"),
            MediaKind::VideoHls
        );
    }

    #[test]
    fn test_url_extension() {
        assert_eq!(
            detect_from_url("https://example.com/song.mp3"),
            MediaKind::AudioMp3
        );
        assert_eq!(
            detect_from_url("https://example.com/video.mp4?token=abc"),
            MediaKind::VideoMp4
        );
        assert_eq!(
            detect_from_url("https://example.com/model.gguf"),
            MediaKind::Gguf
        );
    }

    #[test]
    fn test_three_layer_priority() {
        assert_eq!(
            detect(Some(b"fLaC"), Some("application/octet-stream"), None),
            MediaKind::AudioFlac
        );
        assert_eq!(
            detect(
                None,
                Some("audio/mpeg"),
                Some("https://example.com/file.bin")
            ),
            MediaKind::AudioMp3
        );
        assert_eq!(
            detect(None, None, Some("https://example.com/song.mp3")),
            MediaKind::AudioMp3
        );
    }

    #[test]
    fn test_media_kind_categories() {
        assert!(MediaKind::AudioMp3.is_audio());
        assert!(!MediaKind::AudioMp3.is_video());
        assert!(MediaKind::AudioMp3.is_media());
        assert!(MediaKind::VideoMp4.is_video());
        assert!(MediaKind::VideoMp4.is_media());
        assert!(MediaKind::Gguf.is_model());
        assert!(!MediaKind::Gguf.is_media());
        assert!(MediaKind::Pdf.is_document());
    }
}
