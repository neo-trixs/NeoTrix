#![allow(dead_code)]

use std::fmt;

use whatlang;

// ---------------------------------------------------------------------------
// DocumentType — top-level classification
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentType {
    ScannedPdf,
    DigitalPdf,
    Image,
    OfficeDoc,
    WebPage,
    PlainText,
    Unknown,
}

impl fmt::Display for DocumentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocumentType::ScannedPdf => write!(f, "scanned_pdf"),
            DocumentType::DigitalPdf => write!(f, "digital_pdf"),
            DocumentType::Image => write!(f, "image"),
            DocumentType::OfficeDoc => write!(f, "office_doc"),
            DocumentType::WebPage => write!(f, "web_page"),
            DocumentType::PlainText => write!(f, "plain_text"),
            DocumentType::Unknown => write!(f, "unknown"),
        }
    }
}

// ---------------------------------------------------------------------------
// ClassificationResult — enriched analysis output
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ClassificationResult {
    pub doc_type: DocumentType,
    pub is_scanned: bool,
    pub has_text_layer: bool,
    pub confidence: f64,
    pub format_hint: Option<String>,
    /// Detected natural language (ISO 639-3 code like "eng", "cmn", "jpn").
    pub language: Option<String>,
}

impl ClassificationResult {
    fn new(doc_type: DocumentType, confidence: f64, format_hint: Option<String>) -> Self {
        let (is_scanned, has_text_layer) = match &doc_type {
            DocumentType::ScannedPdf => (true, false),
            DocumentType::DigitalPdf => (false, true),
            _ => (false, false),
        };
        Self {
            doc_type,
            is_scanned,
            has_text_layer,
            confidence,
            format_hint,
            language: None,
        }
    }
}

// ---------------------------------------------------------------------------
// DocumentClassifier — struct API for integration
// ---------------------------------------------------------------------------

/// Classifies documents by format and content type.
///
/// Provides both lightweight byte-level format detection and
/// enriched analysis with confidence scoring, scanned-vs-digital
/// PDF classification, and file-size heuristics.
#[derive(Clone)]
pub struct DocumentClassifier;

impl DocumentClassifier {
    /// Classify a document from raw bytes and optional filename hint.
    pub fn classify(bytes: &[u8], filename_hint: Option<&str>) -> DocumentType {
        classify(bytes, filename_hint)
    }

    /// Enhanced classification with page-count approximation and deeper PDF analysis.
    pub fn classify_with_analysis(
        bytes: &[u8],
        filename_hint: Option<&str>,
    ) -> ClassificationResult {
        classify_with_analysis(bytes, filename_hint)
    }

    /// Detect the natural language of text content using whatlang.
    ///
    /// Returns the ISO 639-3 language code (e.g. "eng", "cmn", "jpn") if
    /// the input is long enough for reliable detection.
    pub fn detect_language(text: &str) -> Option<String> {
        whatlang::detect(text).map(|info| info.lang().code().to_string())
    }

    /// Detect file format from magic bytes only.
    pub fn detect_format_from_bytes(bytes: &[u8]) -> Option<&'static str> {
        detect_format_from_bytes(bytes)
    }
}

// ---------------------------------------------------------------------------
// Public API (free functions)
// ---------------------------------------------------------------------------

/// Classify a document from raw bytes and optional filename hint.
/// Uses magic bytes for format detection, then heuristics for PDF sub-types.
pub fn classify(bytes: &[u8], filename_hint: Option<&str>) -> DocumentType {
    let format = detect_format_from_bytes(bytes);
    match format {
        Some("pdf") => classify_pdf(bytes),
        Some("png") | Some("jpg") | Some("jpeg") | Some("webp") | Some("gif") => {
            DocumentType::Image
        }
        Some("docx") | Some("xlsx") | Some("pptx") => DocumentType::OfficeDoc,
        Some("html") => DocumentType::WebPage,
        Some("txt") => DocumentType::PlainText,
        Some(_other) => {
            if let Some(hint) = filename_hint {
                classify_by_extension(hint)
            } else {
                DocumentType::Unknown
            }
        }
        None => {
            if let Some(hint) = filename_hint {
                classify_by_extension(hint)
            } else if is_likely_text(bytes) {
                DocumentType::PlainText
            } else {
                DocumentType::Unknown
            }
        }
    }
}

/// Enhanced classification with page-count estimation, file-size heuristics,
/// and deeper PDF image-vs-font analysis.
pub fn classify_with_analysis(bytes: &[u8], filename_hint: Option<&str>) -> ClassificationResult {
    let doc_type = classify(bytes, filename_hint);
    let format_hint = detect_format_from_bytes(bytes)
        .map(|s| s.to_string())
        .or_else(|| filename_hint.map(extract_extension).flatten());

    let confidence = match bytes.len() {
        0 => 0.3,
        1..=64 => 0.5,
        _ => 0.85,
    };

    let language = {
        let sample = if bytes.len() > 4096 {
            &bytes[..4096]
        } else {
            bytes
        };
        std::str::from_utf8(sample)
            .ok()
            .and_then(|s| whatlang::detect(s))
            .map(|info| info.lang().code().to_string())
    };

    let base = ClassificationResult::new(doc_type, confidence, format_hint);

    match base.doc_type {
        DocumentType::ScannedPdf | DocumentType::DigitalPdf => {
            let image_count = count_pdf_images(bytes);
            let font_count = count_pdf_fonts(bytes);
            let (scanned, has_text) = if image_count > 0 && font_count == 0 {
                (true, false)
            } else if font_count > 0 {
                (false, true)
            } else {
                (false, true)
            };
            ClassificationResult {
                doc_type: if scanned {
                    DocumentType::ScannedPdf
                } else {
                    DocumentType::DigitalPdf
                },
                is_scanned: scanned,
                has_text_layer: has_text,
                confidence: base.confidence,
                format_hint: base.format_hint,
                language: language.clone(),
            }
        }
        DocumentType::Image => ClassificationResult {
            is_scanned: true,
            has_text_layer: false,
            confidence: base.confidence,
            language: language.clone(),
            ..base
        },
        _ => ClassificationResult { language, ..base },
    }
}

/// Detect file format from magic bytes only.
/// Returns "pdf", "png", "jpg", "webp", "gif", "docx", "zip", "html", "txt",
/// or `None` if unrecognized.
pub fn detect_format_from_bytes(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() < 4 {
        return None;
    }

    // PDF
    if bytes.starts_with(b"%PDF") {
        return Some("pdf");
    }

    // PNG
    if bytes.len() >= 8 && bytes[..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        return Some("png");
    }

    // JPEG
    if bytes[..3] == [0xFF, 0xD8, 0xFF] {
        return Some("jpg");
    }

    // WEBP (RIFF....WEBP)
    if bytes.len() >= 12
        && bytes[..4] == [0x52, 0x49, 0x46, 0x46]
        && bytes[8..12] == [0x57, 0x45, 0x42, 0x50]
    {
        return Some("webp");
    }

    // GIF
    if bytes[..4] == [0x47, 0x49, 0x46, 0x38] {
        return Some("gif");
    }

    // ZIP-based (DOCX, XLSX, PPTX, JAR, etc.)
    if bytes[..4] == [0x50, 0x4B, 0x03, 0x04] {
        return Some("docx");
    }

    // HTML (check for common HTML signatures in first 4K)
    if bytes.len() >= 4096 {
        let head = &bytes[..4096];
        let head_lower = head.to_ascii_lowercase();
        if head_lower.starts_with(b"<!doctype html")
            || head_lower.starts_with(b"<html")
            || head_lower.windows(7).any(|w| w == b"<html>")
            || head_lower.windows(8).any(|w| w == b"<!doctype")
        {
            return Some("html");
        }
    } else {
        let head_lower = bytes.to_ascii_lowercase();
        if head_lower.starts_with(b"<!doctype html")
            || head_lower.starts_with(b"<html")
            || head_lower.windows(7).any(|w| w == b"<html>")
        {
            return Some("html");
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Classify PDF as Scanned or Digital based on content stream hints.
fn classify_pdf(bytes: &[u8]) -> DocumentType {
    let image_count = count_pdf_images(bytes);
    let font_count = count_pdf_fonts(bytes);

    if image_count > 0 && font_count == 0 {
        DocumentType::ScannedPdf
    } else if font_count > 0 {
        DocumentType::DigitalPdf
    } else {
        // Fallback: look for text operators in content streams
        if has_pdf_text_ops(bytes) {
            DocumentType::DigitalPdf
        } else {
            // Not enough info — assume digital
            DocumentType::DigitalPdf
        }
    }
}

/// Rough count of `/Image` references in PDF objects.
fn count_pdf_images(bytes: &[u8]) -> usize {
    let text = match std::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(_) => return 0,
    };
    text.matches("/Image").count()
}

/// Rough count of `/Font` references in PDF objects.
fn count_pdf_fonts(bytes: &[u8]) -> usize {
    let text = match std::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(_) => return 0,
    };
    text.matches("/Font").count()
}

/// Check if the PDF contains text-showing operators (Tj, TJ, ').
fn has_pdf_text_ops(bytes: &[u8]) -> bool {
    let text = match std::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(_) => return false,
    };
    text.contains("Tj") || text.contains("TJ") || text.contains(" Td")
}

/// Fallback classification by filename extension.
fn classify_by_extension(filename: &str) -> DocumentType {
    let lower = filename.to_lowercase();
    if lower.ends_with(".pdf") {
        // Can't distinguish scanned vs digital from extension alone
        DocumentType::DigitalPdf
    } else if lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".webp")
        || lower.ends_with(".gif")
        || lower.ends_with(".bmp")
        || lower.ends_with(".tiff")
        || lower.ends_with(".tif")
    {
        DocumentType::Image
    } else if lower.ends_with(".docx") || lower.ends_with(".xlsx") || lower.ends_with(".pptx") {
        DocumentType::OfficeDoc
    } else if lower.ends_with(".html") || lower.ends_with(".htm") {
        DocumentType::WebPage
    } else if lower.ends_with(".txt") || lower.ends_with(".md") {
        DocumentType::PlainText
    } else {
        DocumentType::Unknown
    }
}

/// Extract the lowercase extension from a filename.
fn extract_extension(filename: &str) -> Option<String> {
    let rpos = filename.rfind('.')?;
    if rpos + 1 < filename.len() {
        Some(filename[rpos + 1..].to_lowercase())
    } else {
        None
    }
}

/// Heuristic: check if bytes look like printable text.
fn is_likely_text(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    let printable = bytes
        .iter()
        .filter(|&&b| b.is_ascii_graphic() || b == b' ' || b == b'\n' || b == b'\r' || b == b'\t')
        .count();
    let ratio = printable as f64 / bytes.len() as f64;
    ratio > 0.85
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Magic byte helpers for tests
    fn pdf_bytes() -> Vec<u8> {
        b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog >>\nendobj\n".to_vec()
    }

    fn pdf_with_font() -> Vec<u8> {
        b"%PDF-1.4\n1 0 obj\n<< /Type /Font /Subtype /TrueType >>\nendobj\nstream\nBT\n/F1 12 Tf\n(Hello) Tj\nET\nendstream\n".to_vec()
    }

    fn pdf_with_image() -> Vec<u8> {
        b"%PDF-1.4\n1 0 obj\n<< /Type /XObject /Subtype /Image >>\nendobj\nstream\n\xFF\xD8\xFF\xE0\x00\x10JFIF\nendstream\n".to_vec()
    }

    fn png_bytes() -> Vec<u8> {
        vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
        ]
    }

    fn jpeg_bytes() -> Vec<u8> {
        vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46]
    }

    fn webp_bytes() -> Vec<u8> {
        let mut b = vec![0x52, 0x49, 0x46, 0x46];
        b.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // file size placeholder
        b.extend_from_slice(&[0x57, 0x45, 0x42, 0x50]);
        b
    }

    fn gif_bytes() -> Vec<u8> {
        b"GIF89a\x01\x00\x01\x00".to_vec()
    }

    fn zip_bytes() -> Vec<u8> {
        vec![0x50, 0x4B, 0x03, 0x04, 0x14, 0x00, 0x00, 0x00]
    }

    fn html_bytes() -> Vec<u8> {
        b"<!DOCTYPE html>\n<html lang=\"en\">\n<head><title>Test</title></head>\n<body><p>Hello</p></body>\n</html>\n".to_vec()
    }

    // -----------------------------------------------------------------------
    // Magic byte detection
    // -----------------------------------------------------------------------

    #[test]
    fn test_detect_png_from_bytes() {
        let bytes = png_bytes();
        assert_eq!(detect_format_from_bytes(&bytes), Some("png"));
    }

    #[test]
    fn test_detect_jpeg_from_bytes() {
        let bytes = jpeg_bytes();
        assert_eq!(detect_format_from_bytes(&bytes), Some("jpg"));
    }

    #[test]
    fn test_detect_pdf_from_bytes() {
        let bytes = pdf_bytes();
        assert_eq!(detect_format_from_bytes(&bytes), Some("pdf"));
    }

    #[test]
    fn test_detect_webp_from_bytes() {
        let bytes = webp_bytes();
        assert_eq!(detect_format_from_bytes(&bytes), Some("webp"));
    }

    #[test]
    fn test_detect_gif_from_bytes() {
        let bytes = gif_bytes();
        assert_eq!(detect_format_from_bytes(&bytes), Some("gif"));
    }

    #[test]
    fn test_detect_docx_from_bytes() {
        let bytes = zip_bytes();
        assert_eq!(detect_format_from_bytes(&bytes), Some("docx"));
    }

    #[test]
    fn test_detect_html_from_bytes() {
        let bytes = html_bytes();
        assert_eq!(detect_format_from_bytes(&bytes), Some("html"));
    }

    // -----------------------------------------------------------------------
    // classify() — top-level dispatch
    // -----------------------------------------------------------------------

    #[test]
    fn test_classify_png() {
        let bytes = png_bytes();
        assert_eq!(classify(&bytes, None), DocumentType::Image);
    }

    #[test]
    fn test_classify_jpeg() {
        let bytes = jpeg_bytes();
        assert_eq!(classify(&bytes, None), DocumentType::Image);
    }

    #[test]
    fn test_classify_pdf_with_font_as_digital() {
        let bytes = pdf_with_font();
        assert_eq!(classify(&bytes, None), DocumentType::DigitalPdf);
    }

    #[test]
    fn test_classify_pdf_with_image_as_scanned() {
        let bytes = pdf_with_image();
        assert_eq!(classify(&bytes, None), DocumentType::ScannedPdf);
    }

    #[test]
    fn test_classify_empty_bytes_with_filename_hint() {
        let bytes: Vec<u8> = vec![];
        assert_eq!(
            classify(&bytes, Some("report.pdf")),
            DocumentType::DigitalPdf
        );
        assert_eq!(classify(&bytes, Some("photo.png")), DocumentType::Image);
        assert_eq!(
            classify(&bytes, Some("document.docx")),
            DocumentType::OfficeDoc
        );
        assert_eq!(classify(&bytes, Some("page.html")), DocumentType::WebPage);
        assert_eq!(classify(&bytes, Some("readme.md")), DocumentType::PlainText);
    }

    #[test]
    fn test_classify_unknown_format() {
        let bytes = b"\x00\x01\x02\x03\x04\x05";
        assert_eq!(classify(bytes, None), DocumentType::Unknown);
    }

    #[test]
    fn test_classify_plain_text_content() {
        let bytes = b"Hello world\nThis is just text content.\nNothing special.\n";
        assert_eq!(classify(bytes, None), DocumentType::PlainText);
    }

    // -----------------------------------------------------------------------
    // classify_with_analysis() — enriched results
    // -----------------------------------------------------------------------

    #[test]
    fn test_classify_with_analysis_confidence_non_empty() {
        let bytes = pdf_with_font();
        let result = classify_with_analysis(&bytes, None);
        assert_eq!(result.doc_type, DocumentType::DigitalPdf);
        assert!(!result.is_scanned);
        assert!(result.has_text_layer);
        assert!((result.confidence - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_classify_with_analysis_scanned_pdf() {
        let bytes = pdf_with_image();
        let result = classify_with_analysis(&bytes, None);
        assert_eq!(result.doc_type, DocumentType::ScannedPdf);
        assert!(result.is_scanned);
        assert!(!result.has_text_layer);
    }

    #[test]
    fn test_classify_with_analysis_empty_bytes_low_confidence() {
        let bytes: Vec<u8> = vec![];
        let result = classify_with_analysis(&bytes, Some("doc.pdf"));
        assert_eq!(result.confidence, 0.3);
        assert!((result.confidence - 0.3).abs() < 0.01);
    }

    // -----------------------------------------------------------------------
    // Format hint / extension helpers
    // -----------------------------------------------------------------------

    #[test]
    fn test_format_hint_present_in_analysis() {
        let bytes = png_bytes();
        let result = classify_with_analysis(&bytes, Some("screenshot.png"));
        assert_eq!(result.format_hint.as_deref(), Some("png"));
    }

    #[test]
    fn test_classify_image_from_bytes_only() {
        let bytes = gif_bytes();
        assert_eq!(classify(&bytes, None), DocumentType::Image);
    }

    #[test]
    fn test_html_detection_via_classify() {
        let bytes = html_bytes();
        assert_eq!(classify(&bytes, None), DocumentType::WebPage);
    }

    // -----------------------------------------------------------------------
    // Edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_classify_short_buffer() {
        let bytes = b"%P";
        assert_eq!(classify(bytes, None), DocumentType::Unknown);
    }

    #[test]
    fn test_classify_with_analysis_unknown_confidence() {
        let bytes = b"\x00\x01\x02\x03";
        let result = classify_with_analysis(bytes, None);
        assert_eq!(result.doc_type, DocumentType::Unknown);
        assert_eq!(result.confidence, 0.5);
    }

    #[test]
    fn test_detect_format_too_short() {
        assert_eq!(detect_format_from_bytes(b"abc"), None);
        assert_eq!(detect_format_from_bytes(b""), None);
    }
}
