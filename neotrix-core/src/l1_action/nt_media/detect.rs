//! Unified media type detection — single source of truth.
//!
//! Replaces scattered Content-Type detection in:
//! - `nt_http.rs:489-503`
//! - `nt_world_video_pipeline.rs:1684-1713`
//! - `osint/http.rs:140-153`
//! - inline `detect_kind()` calls
//!
//! Architecture:
//! ```text
//! ┌──────────────────────────────────────────┐
//! │  L1 Magic Bytes (most reliable)          │
//! │  Reads first 8KB, matches file signatures│
//! ├──────────────────────────────────────────┤
//! │  L2 HTTP Content-Type header             │
║ │  Fallback when magic bytes insufficient  │
//! ├──────────────────────────────────────────┤
//! │  L3 URL extension                        │
║ │  Last resort                             │
//! └──────────────────────────────────────────┘
//! ```

use std::path::Path;

/// Unified media classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaKind {
    // Audio
    AudioMp3,
    AudioFlac,
    AudioWav,
    AudioOgg,
    AudioAac,
    AudioM4a,
    AudioOpus,
    AudioUnknown,
    // Video
    VideoMp4,
    VideoWebm,
    VideoMkv,
    VideoAvi,
    VideoMov,
    VideoHls,      // .m3u8
    VideoDash,     // .mpd
    VideoUnknown,
    // Documents
    Pdf,
    Docx,
    Xlsx,
    Pptx,
    Markdown,
    // Archives
    Zip,
    Gzip,
    Tar,
    Xz,
    SevenZip,
    // Models / Binary
    Gguf,
    Onnx,
    Safetensors,
    Pickle,
    // Code
    Json,
    Csv,
    // Unknown
    Unknown,
}

impl MediaKind {
    /// Is this a playable audio format?
    pub fn is_audio(self) -> bool {
        matches!(self, Self::AudioMp3 | Self::AudioFlac | Self::AudioWav |
                       Self::AudioOgg | Self::AudioAac | Self::AudioM4a |
                       Self::AudioOpus | Self::AudioUnknown)
    }

    /// Is this a playable video format?
    pub fn is_video(self) -> bool {
        matches!(self, Self::VideoMp4 | Self::VideoWebm | Self::VideoMkv |
                       Self::VideoAvi | Self::VideoMov | Self::VideoHls |
                       Self::VideoDash | Self::VideoUnknown)
    }

    /// Is this a playable media (audio or video)?
    pub fn is_media(self) -> bool {
        self.is_audio() || self.is_video()
    }

    /// Is this a model/binary file?
    pub fn is_model(self) -> bool {
        matches!(self, Self::Gguf | Self::Onnx | Self::Safetensors | Self::Pickle)
    }

    /// Is this a document?
    pub fn is_document(self) -> bool {
        matches!(self, Self::Pdf | Self::Docx | Self::Xlsx | Self::Pptx | Self::Markdown)
    }

    /// Human-readable label
    pub fn label(self) -> &'static str {
        match self {
            Self::AudioMp3 => "MP3", Self::AudioFlac => "FLAC", Self::AudioWav => "WAV",
            Self::AudioOgg => "OGG", Self::AudioAac => "AAC", Self::AudioM4a => "M4A",
            Self::AudioOpus => "Opus", Self::AudioUnknown => "Audio",
            Self::VideoMp4 => "MP4", Self::VideoWebm => "WebM", Self::VideoMkv => "MKV",
            Self::VideoAvi => "AVI", Self::VideoMov => "MOV", Self::VideoHls => "HLS",
            Self::VideoDash => "DASH", Self::VideoUnknown => "Video",
            Self::Pdf => "PDF", Self::Docx => "DOCX", Self::Xlsx => "XLSX",
            Self::Pptx => "PPTX", Self::Markdown => "Markdown",
            Self::Zip => "ZIP", Self::Gzip => "GZIP", Self::Tar => "TAR",
            Self::Xz => "XZ", Self::SevenZip => "7Z",
            Self::Gguf => "GGUF", Self::Onnx => "ONNX",
            Self::Safetensors => "SafeTensors", Self::Pickle => "Pickle",
            Self::Json => "JSON", Self::Csv => "CSV",
            Self::Unknown => "Unknown",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// L1 Magic Bytes — most reliable detection
// ═══════════════════════════════════════════════════════════════════════════

/// Detect media kind from magic bytes (first 8KB of content).
/// Single fact source for all Content-Type detection in NeoTrix.
pub fn detect_from_bytes(data: &[u8]) -> MediaKind {
    if data.len() < 4 { return MediaKind::Unknown; }

    // Audio signatures
    if data.starts_with(b"ID3") || data.starts_with(b"\xFF\xFB") || data.starts_with(b"\xFF\xF3") || data.starts_with(b"\xFF\xF2") {
        return MediaKind::AudioMp3;
    }
    if data.starts_with(b"fLaC") { return MediaKind::AudioFlac; }
    if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WAVE" { return MediaKind::AudioWav; }
    if data.starts_with(b"OggS") { return MediaKind::AudioOgg; } // could be Vorbis/Opus
    if data.starts_with(b"Usac") || data.starts_with(b"ftyp") { return MediaKind::AudioAac; }

    // Video signatures
    if data.len() > 12 {
        // MP4/MOV: ftyp box at offset 4
        if &data[4..8] == b"ftyp" {
            let brand = &data[8..12];
            if brand == b"qt  " || brand == b"mqt " { return MediaKind::VideoMov; }
            return MediaKind::VideoMp4;
        }
        // WebM/Matroska: EBML header
        if data[0] == 0x1A && data[1] == 0x45 && data[2] == 0xDF && data[3] == 0xA3 {
            return MediaKind::VideoWebm;
        }
    }
    if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"AVI " {
        return MediaKind::VideoAvi;
    }

    // Model signatures
    if data.starts_with(b"GGUF") { return MediaKind::Gguf; }
    if data.starts_with(b"PK\x03\x04") {
        // ZIP — could be SafeTensors, ONNX, DOCX, etc.
        // Check for specific signatures within
        return MediaKind::Zip;
    }

    // Compressed
    if data[0] == 0x1F && data[1] == 0x8B { return MediaKind::Gzip; }
    if data.starts_with(b"\xfd7zXZ") { return MediaKind::Xz; }
    if data.starts_with(b"7z\xBC\xAF\x27\x1C") { return MediaKind::SevenZip; }

    // Documents
    if data.starts_with(b"%PDF") { return MediaKind::Pdf; }
    if data.starts_with(b"{") || data.starts_with(b"[") { return MediaKind::Json; }

    // Text that might be CSV
    if data.starts_with(b",") || data.starts_with(b"\"") { return MediaKind::Csv; }

    MediaKind::Unknown
}

/// Detect media kind from HTTP Content-Type header.
pub fn detect_from_content_type(ct: &str) -> MediaKind {
    let ct = ct.split(';').next().unwrap_or(ct).trim().to_lowercase();
    match ct.as_str() {
        "audio/mpeg" | "audio/mp3" => MediaKind::AudioMp3,
        "audio/flac" | "audio/x-flac" => MediaKind::AudioFlac,
        "audio/wav" | "audio/x-wav" => MediaKind::AudioWav,
        "audio/ogg" | "audio/vorbis" | "audio/opus" => MediaKind::AudioOgg,
        "audio/aac" | "audio/x-aac" => MediaKind::AudioAac,
        "audio/m4a" | "audio/mp4" => MediaKind::AudioM4a,
        "video/mp4" | "video/webm" | "video/x-matroska" | "video/avi" | "video/quicktime" => MediaKind::VideoMp4,
        "application/x-mpegurl" | "application/vnd.apple.mpegurl" => MediaKind::VideoHls,
        "application/dash+xml" => MediaKind::VideoDash,
        "application/pdf" => MediaKind::Pdf,
        "application/zip" => MediaKind::Zip,
        "application/gzip" | "application/x-gzip" => MediaKind::Gzip,
        "application/json" | "text/json" => MediaKind::Json,
        "text/csv" => MediaKind::Csv,
        "text/markdown" | "text/x-markdown" => MediaKind::Markdown,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => MediaKind::Docx,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => MediaKind::Xlsx,
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => MediaKind::Pptx,
        _ => MediaKind::Unknown,
    }
}

/// Detect media kind from URL extension.
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

/// Three-layer detection: magic bytes → Content-Type header → URL extension.
/// Returns the most reliable detection available.
pub fn detect(
    first_bytes: Option<&[u8]>,
    content_type: Option<&str>,
    url: Option<&str>,
) -> MediaKind {
    // L1: Magic bytes (most reliable)
    if let Some(bytes) = first_bytes {
        let kind = detect_from_bytes(bytes);
        if kind != MediaKind::Unknown { return kind; }
    }

    // L2: HTTP Content-Type header
    if let Some(ct) = content_type {
        let kind = detect_from_content_type(ct);
        if kind != MediaKind::Unknown { return kind; }
    }

    // L3: URL extension (fallback)
    if let Some(url) = url {
        let kind = detect_from_url(url);
        if kind != MediaKind::Unknown { return kind; }
    }

    MediaKind::Unknown
}

// ═══════════════════════════════════════════════════════════════════════════
// File-based detection
// ═══════════════════════════════════════════════════════════════════════════

/// Detect media kind from a file path (extension only, no I/O).
pub fn detect_from_path(path: &Path) -> MediaKind {
    detect_from_url(&path.to_string_lossy())
}

/// Detect media kind by reading first bytes from a file.
pub async fn detect_from_file(path: &Path) -> MediaKind {
    match tokio::fs::read(path).await {
        Ok(data) => {
            let first = &data[..data.len().min(8192)];
            detect_from_bytes(first)
        }
        Err(_) => detect_from_path(path),
    }
}

/// Probe URL headers and first bytes for detection.
/// Makes a HEAD request (or GET with Range for first bytes) to classify remote content.
pub async fn detect_remote(
    client: &reqwest::Client,
    url: &str,
) -> (MediaKind, Option<String>) {
    // HEAD request for Content-Type
    if let Ok(resp) = client.head(url).send().await {
        let ct = resp.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        if let Some(ref ct_val) = ct {
            let kind = detect_from_content_type(ct_val);
            if kind != MediaKind::Unknown {
                return (kind, ct.clone());
            }
        }

        // Fallback to URL extension
        let kind = detect_from_url(url);
        return (kind, ct);
    }

    // GET first bytes if HEAD fails
    if let Ok(resp) = client.get(url)
        .header("Range", "bytes=0-8191")
        .send()
        .await
    {
        let ct = resp.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        if let Ok(bytes) = resp.bytes().await {
            let kind = detect_from_bytes(&bytes[..bytes.len().min(8192)]);
            if kind != MediaKind::Unknown {
                return (kind, ct);
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
        assert_eq!(detect_from_bytes(b"RIFF\x00\x00\x00\x00WAVE"), MediaKind::AudioWav);
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
        assert_eq!(detect_from_content_type("video/mp4; charset=utf-8"), MediaKind::VideoMp4);
        assert_eq!(detect_from_content_type("application/x-mpegurl"), MediaKind::VideoHls);
        assert_eq!(detect_from_content_type("application/pdf"), MediaKind::Pdf);
    }

    #[test]
    fn test_url_extension() {
        assert_eq!(detect_from_url("https://example.com/song.mp3"), MediaKind::AudioMp3);
        assert_eq!(detect_from_url("https://example.com/video.mp4?token=abc"), MediaKind::VideoMp4);
        assert_eq!(detect_from_url("https://example.com/model.gguf"), MediaKind::Gguf);
        assert_eq!(detect_from_url("https://example.com/playlist.m3u8"), MediaKind::VideoHls);
    }

    #[test]
    fn test_three_layer_priority() {
        // Magic bytes win over Content-Type
        assert_eq!(detect(Some(b"fLaC"), Some("application/octet-stream"), None), MediaKind::AudioFlac);
        // Content-Type wins over URL
        assert_eq!(detect(None, Some("audio/mpeg"), Some("https://example.com/file.bin")), MediaKind::AudioMp3);
        // URL fallback
        assert_eq!(detect(None, None, Some("https://example.com/song.mp3")), MediaKind::AudioMp3);
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
