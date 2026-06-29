#![allow(dead_code)]

//! Core types and traits for VLM Document Parsing Evolution (Phase 2).
//!
//! Provides a unified [`DocumentParser`] trait, extensible [`DocumentSource`] enum,
//! structured extraction types ([`ParsedDocument`], [`ExtractedTable`], [`ExtractedFigure`]),
//! and helpers for format detection and byte resolution.
//!
//! # Architecture
//!
//! Backends (MinerU, Docling, Dolphin, Nemotron, etc.) implement [`DocumentParser`].
//! The trait is inference-agnostic — backends can be local or API-based.
//! Consumers call [`parse()`] on a [`DocumentSource`] and receive a [`ParsedDocument`].

use std::fmt;

// ---------------------------------------------------------------------------
// Document source
// ---------------------------------------------------------------------------

/// Source of a document to parse.
#[derive(Debug, Clone)]
pub enum DocumentSource {
    /// Raw bytes with optional format hint.
    Bytes(Vec<u8>),
    /// File path on disk.
    File(String),
    /// URL to fetch.
    Url(String),
    /// Already-read text content (fast path).
    Text(String),
}

// ---------------------------------------------------------------------------
// Extraction results
// ---------------------------------------------------------------------------

/// Result of document parsing — fully structured content extracted from a
/// single document.
#[derive(Debug, Clone)]
pub struct ParsedDocument {
    /// Full document text converted to Markdown.
    pub markdown: String,
    /// Tables extracted during parsing.
    pub tables: Vec<ExtractedTable>,
    /// Figures / images extracted during parsing.
    pub images: Vec<ExtractedFigure>,
    /// Document-level metadata.
    pub metadata: DocumentMetadata,
}

/// A single table extracted from a document.
#[derive(Debug, Clone)]
pub struct ExtractedTable {
    /// Table caption or title (may be empty).
    pub caption: String,
    /// Column header names.
    pub headers: Vec<String>,
    /// Data rows.
    pub rows: Vec<Vec<String>>,
    /// Layout classification of the table.
    pub format: TableFormat,
}

/// Classification of a table's structural layout.
#[derive(Debug, Clone, PartialEq)]
pub enum TableFormat {
    /// Simple grid with uniform columns.
    Simple,
    /// Pivot / cross-tabulation table.
    Pivot,
    /// Merged-cell / complex table.
    Merged,
    /// Undetermined layout.
    Unknown,
}

/// An extracted figure or image from a document.
#[derive(Debug, Clone)]
pub struct ExtractedFigure {
    /// Figure caption or description text.
    pub caption: String,
    /// Spatial bounding box in page coordinates (if available).
    pub bbox: Option<BBox>,
    /// LLM- or OCR-generated description of the figure content.
    pub description: String,
}

/// Axis-aligned bounding box in page coordinates.
#[derive(Debug, Clone, PartialEq)]
pub struct BBox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Metadata collected during document parsing.
#[derive(Debug, Clone, Default)]
pub struct DocumentMetadata {
    /// Document title (may be `None` if not detected).
    pub title: Option<String>,
    /// Author(s).
    pub author: Option<String>,
    /// Total number of pages.
    pub page_count: Option<usize>,
    /// File format extension (e.g. "pdf", "docx").
    pub format: Option<String>,
    /// File size in bytes.
    pub size_bytes: Option<usize>,
    /// Detected language code (e.g. "en", "zh").
    pub language: Option<String>,
    /// Wall-clock time spent extracting, in milliseconds.
    pub extraction_time_ms: u64,
    /// Name of the parser backend that produced this result.
    pub backend_name: String,
    /// Number of formula elements found in this document.
    pub formula_count: usize,
    /// Detected layout type (e.g. "SingleColumn", "TwoColumn").
    pub layout_type: Option<String>,
    /// Number of text columns detected.
    pub column_count: Option<usize>,
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during document parsing.
#[derive(Debug)]
pub enum DocumentError {
    /// The document format is not supported by any available backend.
    UnsupportedFormat(String),
    /// The document could not be parsed (corrupted, malformed, etc.).
    ParseError(String),
    /// A network request failed (URL source only).
    NetworkError(String),
    /// An I/O error occurred reading from disk.
    IoError(std::io::Error),
    /// The requested backend is not compiled in or not reachable.
    BackendUnavailable(String),
    /// The parsing operation exceeded its time budget.
    Timeout(String),
}

impl fmt::Display for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocumentError::UnsupportedFormat(s) => write!(f, "Unsupported format: {}", s),
            DocumentError::ParseError(s) => write!(f, "Parse error: {}", s),
            DocumentError::NetworkError(s) => write!(f, "Network error: {}", s),
            DocumentError::IoError(e) => write!(f, "IO error: {}", e),
            DocumentError::BackendUnavailable(s) => write!(f, "Backend unavailable: {}", s),
            DocumentError::Timeout(s) => write!(f, "Timeout: {}", s),
        }
    }
}

impl std::error::Error for DocumentError {}

impl From<std::io::Error> for DocumentError {
    fn from(e: std::io::Error) -> Self {
        DocumentError::IoError(e)
    }
}

// ---------------------------------------------------------------------------
// Parser trait
// ---------------------------------------------------------------------------

/// A parser that can extract structured content from documents.
///
/// Implementors are [`Send`] so they can be dispatched across threads or
/// stored in an `Arc<dyn DocumentParser>`.
pub trait DocumentParser: Send + Sync {
    /// Parse a document from the given source.
    fn parse(&self, source: &DocumentSource) -> Result<ParsedDocument, DocumentError>;

    /// Return the list of formats this parser supports (e.g. `["pdf", "png", "jpg"]`).
    fn supported_formats(&self) -> Vec<&'static str>;

    /// Human-readable name for this backend.
    fn backend_name(&self) -> &'static str;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve a [`DocumentSource`] into raw bytes.
///
/// # Errors
///
/// Returns [`DocumentError::NetworkError`] for `Url` sources (not implemented
/// in the base layer — use a VLM backend that handles HTTP).
pub fn source_to_bytes(source: &DocumentSource) -> Result<Vec<u8>, DocumentError> {
    match source {
        DocumentSource::Bytes(bytes) => Ok(bytes.clone()),
        DocumentSource::File(path) => Ok(std::fs::read(path)?),
        DocumentSource::Url(_url) => Err(DocumentError::NetworkError(
            "URL fetching not implemented in base. Use VlmBackend.".to_string(),
        )),
        DocumentSource::Text(text) => Ok(text.as_bytes().to_vec()),
    }
}

/// Detect document format from a file extension or magic-bytes prefix string.
///
/// This is a heuristic — for authoritative detection, use `infer` or similar.
pub fn detect_format(path_or_bytes: &str) -> &'static str {
    let lower = path_or_bytes.to_lowercase();
    if lower.ends_with(".pdf") {
        "pdf"
    } else if lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".webp")
    {
        "image"
    } else if lower.ends_with(".docx") {
        "docx"
    } else if lower.ends_with(".xlsx") {
        "xlsx"
    } else if lower.ends_with(".xls") {
        "xls"
    } else if lower.ends_with(".html") || lower.ends_with(".htm") {
        "html"
    } else if lower.ends_with(".md") {
        "markdown"
    } else if lower.ends_with(".csv") {
        "csv"
    } else {
        "unknown"
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_to_bytes_from_bytes() {
        let data = vec![1, 2, 3];
        let source = DocumentSource::Bytes(data.clone());
        let result = source_to_bytes(&source).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_source_to_bytes_from_text() {
        let source = DocumentSource::Text("hello".to_string());
        let result = source_to_bytes(&source).unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn test_source_to_bytes_url_returns_error() {
        let source = DocumentSource::Url("http://example.com/doc.pdf".to_string());
        let result = source_to_bytes(&source);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DocumentError::NetworkError(_)
        ));
    }

    #[test]
    fn test_detect_format_pdf() {
        assert_eq!(detect_format("report.pdf"), "pdf");
        assert_eq!(detect_format("a/b/c.PDF"), "pdf");
    }

    #[test]
    fn test_detect_format_image() {
        assert_eq!(detect_format("photo.png"), "image");
        assert_eq!(detect_format("photo.jpg"), "image");
        assert_eq!(detect_format("photo.jpeg"), "image");
        assert_eq!(detect_format("photo.webp"), "image");
    }

    #[test]
    fn test_detect_format_office() {
        assert_eq!(detect_format("doc.docx"), "docx");
        assert_eq!(detect_format("sheet.xlsx"), "xlsx");
        assert_eq!(detect_format("old.xls"), "xls");
    }

    #[test]
    fn test_detect_format_web() {
        assert_eq!(detect_format("page.html"), "html");
        assert_eq!(detect_format("page.htm"), "html");
    }

    #[test]
    fn test_detect_format_markdown_and_csv() {
        assert_eq!(detect_format("readme.md"), "markdown");
        assert_eq!(detect_format("data.csv"), "csv");
    }

    #[test]
    fn test_detect_format_unknown() {
        assert_eq!(detect_format("archive.zip"), "unknown");
        assert_eq!(detect_format(""), "unknown");
    }

    #[test]
    fn test_document_error_display() {
        let err = DocumentError::UnsupportedFormat("foo".into());
        assert_eq!(format!("{}", err), "Unsupported format: foo");

        let err = DocumentError::ParseError("bar".into());
        assert_eq!(format!("{}", err), "Parse error: bar");

        let err = DocumentError::NetworkError("timeout".into());
        assert_eq!(format!("{}", err), "Network error: timeout");
    }

    #[test]
    fn test_document_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let doc_err: DocumentError = io_err.into();
        assert!(matches!(doc_err, DocumentError::IoError(_)));
    }

    #[test]
    fn test_parsed_document_default_metadata() {
        let meta = DocumentMetadata::default();
        assert_eq!(meta.extraction_time_ms, 0);
        assert_eq!(meta.backend_name, "");
    }
}
