#![allow(dead_code)]

use crate::core::nt_core_input::pdf_extractor::extract_text_from_pdf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocFormat {
    Pdf,
    Docx,
    Pptx,
    Xlsx,
    Html,
    Csv,
    Json,
    Markdown,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub format: DocFormat,
    pub title: Option<String>,
    pub author: Option<String>,
    pub page_count: Option<usize>,
    pub size_bytes: usize,
    pub content: String,
}

pub struct DocPipeline;

impl DocPipeline {
    pub fn process(bytes: &[u8], format_hint: Option<DocFormat>) -> Result<Document, String> {
        let format = match format_hint {
            Some(f) => f,
            None => FormatDetector::from_bytes(bytes).ok_or("unable to detect format")?,
        };
        convert(bytes, format)
    }

    pub fn process_path(path: &str) -> Result<Document, String> {
        let format = FormatDetector::from_path(path).ok_or("unable to detect format from path")?;
        let bytes = std::fs::read(path).map_err(|e| format!("failed to read file: {e}"))?;
        convert(&bytes, format)
    }
}

pub struct FormatDetector;

impl FormatDetector {
    pub fn from_bytes(bytes: &[u8]) -> Option<DocFormat> {
        if bytes.len() < 4 {
            return None;
        }
        if &bytes[..5] == b"%PDF-" {
            return Some(DocFormat::Pdf);
        }
        if bytes.len() > 2 && &bytes[0..2] == b"\x1f\x8b" {
            return Some(DocFormat::Docx);
        }
        if bytes.len() > 4 {
            let head = &bytes[..4];
            if head == [0x50, 0x4b, 0x03, 0x04] {
                let s = String::from_utf8_lossy(bytes);
                if s.contains("word/") || s.contains("docProps/") {
                    return Some(DocFormat::Docx);
                }
                if s.contains("xl/") || s.contains("xl/") {
                    return Some(DocFormat::Xlsx);
                }
                if s.contains("ppt/") || s.contains("ppt/") {
                    return Some(DocFormat::Pptx);
                }
                return Some(DocFormat::Docx);
            }
            if head.starts_with(b"<html") || head.starts_with(b"<!DOC") {
                return Some(DocFormat::Html);
            }
            if head.starts_with(b"{") || head.starts_with(b"[") {
                if bytes.iter().any(|&b| b == b'\n') {
                    let s = String::from_utf8_lossy(bytes);
                    let lines: Vec<&str> = s.lines().collect();
                    if lines.len() > 1 {
                        let first = lines[0].trim();
                        if first == "["
                            || first == "{"
                            || first.starts_with("[")
                            || first.starts_with("{")
                        {
                            return Some(DocFormat::Json);
                        }
                    }
                }
            }
            if bytes.len() > 7 && &bytes[..7] == b"{\\rtf1" {
                return None;
            }
        }
        None
    }

    pub fn from_path(path: &str) -> Option<DocFormat> {
        let ext = path.rsplit('.').next()?.to_lowercase();
        match ext.as_str() {
            "pdf" => Some(DocFormat::Pdf),
            "docx" => Some(DocFormat::Docx),
            "pptx" => Some(DocFormat::Pptx),
            "xlsx" => Some(DocFormat::Xlsx),
            "html" | "htm" => Some(DocFormat::Html),
            "csv" => Some(DocFormat::Csv),
            "json" => Some(DocFormat::Json),
            "md" | "markdown" => Some(DocFormat::Markdown),
            _ => None,
        }
    }
}

fn convert(bytes: &[u8], format: DocFormat) -> Result<Document, String> {
    let size = bytes.len();
    match format {
        DocFormat::Pdf => {
            let text =
                extract_text_from_pdf(bytes).map_err(|e| format!("PDF extraction failed: {e}"))?;
            let title = MetadataExtractor::extract_title(&text);
            let author = MetadataExtractor::extract_author(&text);
            let page_count = MetadataExtractor::extract_page_count(&text);
            let content = MarkdownRenderer::render(&text);
            Ok(Document {
                format,
                title,
                author,
                page_count,
                size_bytes: size,
                content,
            })
        }
        DocFormat::Docx
        | DocFormat::Pptx
        | DocFormat::Xlsx
        | DocFormat::Html
        | DocFormat::Csv
        | DocFormat::Json => {
            let raw = extract_readable_text(bytes);
            let title = MetadataExtractor::extract_title(&raw);
            let author = MetadataExtractor::extract_author(&raw);
            let content = MarkdownRenderer::render(&raw);
            Ok(Document {
                format,
                title,
                author,
                page_count: None,
                size_bytes: size,
                content,
            })
        }
        DocFormat::Markdown => {
            let raw = String::from_utf8_lossy(bytes).to_string();
            let title = MetadataExtractor::extract_title(&raw);
            let author = MetadataExtractor::extract_author(&raw);
            Ok(Document {
                format,
                title,
                author,
                page_count: None,
                size_bytes: size,
                content: raw,
            })
        }
    }
}

fn extract_readable_text(bytes: &[u8]) -> String {
    let mut result = String::new();
    let mut pending = String::new();
    for &b in bytes {
        if b.is_ascii_graphic() || b == b' ' || b == b'\n' || b == b'\r' || b == b'\t' {
            pending.push(b as char);
        } else {
            let c = b as char;
            if c.is_alphabetic() {
                pending.push(c);
            } else if c.is_whitespace() {
                pending.push(' ');
            } else if pending.ends_with(' ') {
                continue;
            } else {
                pending.push(' ');
            }
        }
        if pending.len() >= 3 {
            let tail = &pending[pending.len() - 3..];
            if tail == "   " {
                continue;
            }
        }
        if pending.len() > 1_000_000 {
            break;
        }
    }
    result.push_str(pending.trim());
    result
}

pub struct MarkdownRenderer;

impl MarkdownRenderer {
    pub fn render(text: &str) -> String {
        let mut out = String::with_capacity(text.len() + 256);
        let mut in_code_block = false;
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                out.push('\n');
                continue;
            }
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                out.push_str("```\n");
                continue;
            }
            if in_code_block {
                out.push_str(line);
                out.push('\n');
                continue;
            }
            if trimmed.starts_with("# ")
                || trimmed.starts_with("## ")
                || trimmed.starts_with("### ")
                || trimmed.starts_with("#### ")
                || trimmed.starts_with("##### ")
                || trimmed.starts_with("###### ")
            {
                out.push_str(trimmed);
                out.push('\n');
                continue;
            }
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
                out.push_str(trimmed);
                out.push('\n');
                continue;
            }
            if trimmed.starts_with(|c: char| c.is_ascii_digit()) && trimmed.contains(". ") {
                out.push_str(trimmed);
                out.push('\n');
                continue;
            }
            if trimmed.starts_with("> ") {
                out.push_str(trimmed);
                out.push('\n');
                continue;
            }
            if trimmed.starts_with("|") && trimmed.ends_with("|") {
                out.push_str(trimmed);
                out.push('\n');
                continue;
            }
            out.push_str(trimmed);
            out.push('\n');
        }
        out.trim().to_string()
    }
}

pub struct MetadataExtractor;

impl MetadataExtractor {
    pub fn extract_title(text: &str) -> Option<String> {
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") {
                return Some(trimmed[2..].trim().to_string());
            }
            if trimmed.starts_with("title:") {
                let val = trimmed[6..].trim().trim_matches('"').to_string();
                if !val.is_empty() {
                    return Some(val);
                }
            }
            if trimmed.starts_with("Title:") {
                let val = trimmed[6..].trim().trim_matches('"').to_string();
                if !val.is_empty() {
                    return Some(val);
                }
            }
        }
        for line in text.lines().take(5) {
            let trimmed = line.trim();
            if !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && trimmed.len() > 10
                && trimmed.len() < 200
                && trimmed.chars().filter(|&c| c == ' ').count() <= 15
            {
                return Some(trimmed.to_string());
            }
        }
        None
    }

    pub fn extract_author(text: &str) -> Option<String> {
        for line in text.lines() {
            let trimmed = line.trim().to_lowercase();
            if trimmed.starts_with("author:") || trimmed.starts_with("by ") {
                let val = if trimmed.starts_with("author:") {
                    line.trim()[7..].trim().trim_matches('"').to_string()
                } else {
                    line.trim()[3..].trim().trim_matches('"').to_string()
                };
                if !val.is_empty() && val.len() < 100 {
                    return Some(val);
                }
            }
        }
        None
    }

    pub fn extract_page_count(text: &str) -> Option<usize> {
        let pages: Vec<&str> = text.split("---").collect();
        if pages.len() > 1 {
            Some(pages.len())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_from_bytes() {
        assert_eq!(
            FormatDetector::from_bytes(b"%PDF-1.4"),
            Some(DocFormat::Pdf)
        );
        assert_eq!(
            FormatDetector::from_bytes(b"<html>\n<body>\n</body>\n</html>"),
            Some(DocFormat::Html)
        );
        assert_eq!(
            FormatDetector::from_bytes(b"<!DOCTYPE html>"),
            Some(DocFormat::Html)
        );
        assert_eq!(FormatDetector::from_bytes(b"hello world"), None);
    }

    #[test]
    fn test_detect_format_from_extension() {
        assert_eq!(
            FormatDetector::from_path("report.pdf"),
            Some(DocFormat::Pdf)
        );
        assert_eq!(FormatDetector::from_path("doc.docx"), Some(DocFormat::Docx));
        assert_eq!(
            FormatDetector::from_path("slide.pptx"),
            Some(DocFormat::Pptx)
        );
        assert_eq!(
            FormatDetector::from_path("sheet.xlsx"),
            Some(DocFormat::Xlsx)
        );
        assert_eq!(
            FormatDetector::from_path("page.html"),
            Some(DocFormat::Html)
        );
        assert_eq!(FormatDetector::from_path("data.csv"), Some(DocFormat::Csv));
        assert_eq!(
            FormatDetector::from_path("data.json"),
            Some(DocFormat::Json)
        );
        assert_eq!(
            FormatDetector::from_path("readme.md"),
            Some(DocFormat::Markdown)
        );
        assert_eq!(FormatDetector::from_path("image.png"), None);
    }

    #[test]
    fn test_pdf_delegation() {
        let pdf_bytes = b"%PDF-1.4\n1 0 obj\n<< /Length 44 >>\nstream\nBT\n/F1 12 Tf\n(Hello from PDF) Tj\nET\nendstream\nendobj\n%%EOF\n";
        let doc = DocPipeline::process(pdf_bytes, Some(DocFormat::Pdf)).unwrap();
        assert_eq!(doc.format, DocFormat::Pdf);
        assert!(
            doc.content.contains("Hello from PDF"),
            "content: {}",
            doc.content
        );
    }

    #[test]
    fn test_html_as_markdown() {
        let html = b"<html><body><h1>Title</h1><p>Hello world</p></body></html>";
        let doc = DocPipeline::process(html, Some(DocFormat::Html)).unwrap();
        assert_eq!(doc.format, DocFormat::Html);
        assert!(doc.content.contains("Title") || doc.content.contains("Hello"));
    }

    #[test]
    fn test_format_unknown() {
        let result = DocPipeline::process(b"\x00\x01\x02\x03", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_markdown_renderer_headings() {
        let input = "# Heading 1\nSome text\n## Heading 2\n- list item\n";
        let rendered = MarkdownRenderer::render(input);
        assert!(rendered.contains("# Heading 1"));
        assert!(rendered.contains("## Heading 2"));
        assert!(rendered.contains("- list item"));
    }
}
