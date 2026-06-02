use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Mutex;

pub mod pdf;
pub mod docx;
pub mod xml;
pub mod text;

// Re-export for sibling modules
pub(crate) use xml::extract_xml_text;

/// 文件解析结果
#[derive(Debug, Clone)]
pub struct FileParseResult {
    pub filename: String,
    pub mime: String,
    pub text: String,
    pub format: FileFormat,
    pub size_bytes: usize,
    pub parse_success: bool,
    pub spatial_blocks: Vec<SpatialBlock>,
}

/// 检测到的文件格式
#[derive(Debug, Clone, PartialEq)]
pub enum FileFormat {
    PlainText,
    Markdown,
    Code,
    Json,
    Xml,
    Csv,
    Html,
    Pdf,
    Docx,
    Xlsx,
    Pptx,
    Image,
    Audio,
    Video,
    Binary,
    Unknown,
}

/// 全局解析缓存（LRU, 128 条目）
static PARSE_CACHE: Mutex<Option<lru::LruCache<u64, FileParseResult>>> = Mutex::new(None);

fn with_cache<F, R>(f: F) -> R
where
    F: FnOnce(&mut lru::LruCache<u64, FileParseResult>) -> R,
{
    let mut guard = PARSE_CACHE.lock().expect("PARSE_CACHE mutex poisoned");
    let cache = guard.get_or_insert_with(|| lru::LruCache::new(NonZeroUsize::new(128).expect("128 > 0")));
    f(cache)
}

fn cache_hash(filename: &str, data: &[u8]) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    filename.hash(&mut h);
    let sample = data.len().min(1024);
    data[..sample].hash(&mut h);
    h.finish()
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    TextBlock,
    ImageBlock,
    TableBlock,
    HeadingBlock,
}

#[derive(Debug, Clone)]
pub struct SpatialBlock {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub text: String,
    pub block_type: BlockType,
}

fn is_valid_utf8(data: &[u8]) -> bool {
    std::str::from_utf8(data).is_ok()
}

/// 文件类型检测 + 文本提取器
pub struct FileParser;

impl FileParser {
    pub const MAX_EXTRACT_BYTES: usize = 100_000;

    pub fn detect_format(filename: &str, mime: &str, data: &[u8]) -> FileFormat {
        if data.is_empty() {
            return FileFormat::PlainText;
        }
        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "txt" | "text" => return FileFormat::PlainText,
            "md" | "markdown" => return FileFormat::Markdown,
            "json" => return FileFormat::Json,
            "xml" | "plist" | "svg" => return FileFormat::Xml,
            "csv" | "tsv" => return FileFormat::Csv,
            "html" | "htm" => return FileFormat::Html,
            "pdf" => return FileFormat::Pdf,
            "docx" | "docm" => return FileFormat::Docx,
            "xlsx" | "xlsm" | "xls" => return FileFormat::Xlsx,
            "pptx" | "pptm" | "ppsx" => return FileFormat::Pptx,
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" => return FileFormat::Image,
            "mp3" | "wav" | "ogg" | "flac" | "aac" | "m4a" | "wma" => return FileFormat::Audio,
            "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" => return FileFormat::Video,
            "rs" | "py" | "js" | "ts" | "tsx" | "jsx" | "go" | "rb" | "java" | "c" | "cpp" | "h"
            | "hpp" | "swift" | "kt" | "scala" | "sh" | "bash" | "zsh" | "fish" | "r" | "m" | "mm"
            | "sql" | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" | "env" | "dockerfile"
            | "makefile" | "cmake" => return FileFormat::Code,
            _ => {}
        }

        if mime.starts_with("image/") {
            return FileFormat::Image;
        }
        if mime.starts_with("audio/") {
            return FileFormat::Audio;
        }
        if mime.starts_with("video/") {
            return FileFormat::Video;
        }
        if mime == "application/pdf" {
            return FileFormat::Pdf;
        }
        if mime.contains("word") || mime.contains("document") && mime.contains("officedocument") {
            return FileFormat::Docx;
        }
        if mime.contains("spreadsheet") || mime.contains("excel") {
            return FileFormat::Xlsx;
        }
        if mime.contains("presentation") || mime.contains("powerpoint") {
            return FileFormat::Pptx;
        }
        if mime == "application/json" || mime == "application/xml" {
            return FileFormat::PlainText;
        }
        if mime == "text/html" || mime == "application/html" {
            return FileFormat::Html;
        }

        let magic = &data[..4.min(data.len())];
        if magic.starts_with(b"%PDF") {
            return FileFormat::Pdf;
        }
        if magic.starts_with(b"PK") {
            let name_lower = filename.to_lowercase();
            if name_lower.contains(".doc") {
                return FileFormat::Docx;
            }
            if name_lower.contains(".xls") {
                return FileFormat::Xlsx;
            }
            if name_lower.contains(".ppt") {
                return FileFormat::Pptx;
            }
            return FileFormat::Binary;
        }

        if is_valid_utf8(data) {
            let sample = String::from_utf8_lossy(data);
            let s = sample.trim();
            if s.len() > 10 {
                if s.starts_with('{') || s.starts_with('[') {
                    return FileFormat::Json;
                }
                if s.starts_with('<') && s.contains("html") {
                    return FileFormat::Html;
                }
                if s.starts_with('<') {
                    return FileFormat::Xml;
                }
                if s.chars().filter(|&c| c == '\n').count() > 2 && s.contains("|") {
                    return FileFormat::Csv;
                }
            }

            if mime.starts_with("text/") {
                return FileFormat::PlainText;
            }

            return FileFormat::PlainText;
        }

        FileFormat::Binary
    }

    /// 带缓存的解析入口（推荐使用）
    pub fn parse(filename: &str, mime: &str, data: &[u8]) -> FileParseResult {
        let hash = cache_hash(filename, data);
        {
            let guard = PARSE_CACHE.lock().expect("PARSE_CACHE mutex poisoned");
            if let Some(cache) = guard.as_ref() {
                if let Some(cached) = cache.peek(&hash) {
                    return cached.clone();
                }
            }
        }
        let result = Self::extract_text(filename, mime, data);
        with_cache(|c| {
            c.put(hash, result.clone());
        });
        result
    }

    /// 无缓存解析入口（静态方法，保持向后兼容）
    pub fn extract_text(filename: &str, mime: &str, data: &[u8]) -> FileParseResult {
        let format = Self::detect_format(filename, mime, data);
        let size = data.len();
        let data = if data.len() > Self::MAX_EXTRACT_BYTES {
            &data[..Self::MAX_EXTRACT_BYTES]
        } else {
            data
        };
        let (text, success) = match &format {
            FileFormat::Pdf => (Self::extract_pdf_text(data), true),
            FileFormat::Docx => {
                let extracted = Self::extract_docx_text(data);
                if !extracted.is_empty() {
                    (extracted, true)
                } else {
                    (format!("[NeoTrix 文件解析]\n格式: {:?}\n文件名: {}\n大小: {} bytes\n\n文件已接收，NeoTrix 正在学习解析此格式。",
                        format, filename, size),
                     false)
                }
            }
            FileFormat::Xlsx => {
                let extracted = Self::extract_xlsx_text(data);
                if !extracted.is_empty() {
                    (extracted, true)
                } else {
                    (format!("[NeoTrix 文件解析]\n格式: {:?}\n文件名: {}\n大小: {} bytes\n\n文件已接收，NeoTrix 正在学习解析此格式。",
                        format, filename, size),
                     false)
                }
            }
            FileFormat::Pptx => {
                let extracted = Self::extract_pptx_text(data);
                if !extracted.is_empty() {
                    (extracted, true)
                } else {
                    (format!("[NeoTrix 文件解析]\n格式: {:?}\n文件名: {}\n大小: {} bytes\n\n文件已接收，NeoTrix 正在学习解析此格式。",
                        format, filename, size),
                     false)
                }
            }
            fmt if fmt == &FileFormat::Binary || fmt == &FileFormat::Audio || fmt == &FileFormat::Video => {
                (format!("[NeoTrix 文件引用]\n格式: {:?}\n文件名: {}\n大小: {} bytes\n\n此文件为二进制格式，已存储为引用。NeoTrix 持续学习中。",
                    fmt, filename, size),
                 false)
            }
            _ => {
                match String::from_utf8(data.to_vec()) {
                    Ok(s) => (s, true),
                    Err(_) => {
                        (format!("[NeoTrix 文件解析]\n文件名: {}\n大小: {} bytes\n\n内容无法解码为 UTF-8 文本。",
                            filename, size),
                         false)
                    }
                }
            }
        };
        FileParseResult {
            filename: filename.to_string(),
            mime: mime.to_string(),
            text,
            format,
            size_bytes: size,
            parse_success: success,
            spatial_blocks: Vec::new(),
        }
    }

    /// 清空解析缓存
    pub fn clear_cache() {
        let mut guard = PARSE_CACHE.lock().expect("PARSE_CACHE mutex poisoned");
        *guard = None;
    }

    pub fn parse_with_layout(filename: &str, mime: &str, data: &[u8]) -> FileParseResult {
        let format = Self::detect_format(filename, mime, data);
        let size = data.len();
        let data = if data.len() > Self::MAX_EXTRACT_BYTES {
            &data[..Self::MAX_EXTRACT_BYTES]
        } else {
            data
        };

        let (text, spatial_blocks, success) = match &format {
            FileFormat::Pdf => {
                let blocks = Self::extract_pdf_spatial(data);
                let text = blocks
                    .iter()
                    .map(|b| b.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
                (text, blocks, true)
            }
            _ => {
                let result = Self::extract_text(filename, mime, data);
                (result.text, Vec::new(), result.parse_success)
            }
        };

        FileParseResult {
            filename: filename.to_string(),
            mime: mime.to_string(),
            text,
            format,
            size_bytes: size,
            parse_success: success,
            spatial_blocks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_zip_xml(entry_name: &str, xml_content: &str) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            zip.start_file(entry_name, options).expect("zip start_file");
            zip.write_all(xml_content.as_bytes()).expect("zip write_all");
            zip.finish().expect("zip finish");
        }
        buf
    }

    #[test]
    fn test_detect_format_by_extension() {
        assert_eq!(FileParser::detect_format("test.rs", "text/plain", b"fn main() {}"), FileFormat::Code);
        assert_eq!(FileParser::detect_format("doc.md", "text/markdown", b"# Hello"), FileFormat::Markdown);
        assert_eq!(FileParser::detect_format("data.json", "application/json", b"{}"), FileFormat::Json);
        assert_eq!(FileParser::detect_format("doc.pdf", "application/pdf", b"%PDF-1.4"), FileFormat::Pdf);
        assert_eq!(FileParser::detect_format("doc.docx", "application/octet-stream", &[0x50, 0x4B, 0x03, 0x04]), FileFormat::Docx);
        assert_eq!(FileParser::detect_format("img.png", "image/png", &[0x89, 0x50, 0x4E, 0x47]), FileFormat::Image);
        assert_eq!(FileParser::detect_format("deck.pptx", "application/octet-stream", &[0x50, 0x4B, 0x03, 0x04]), FileFormat::Pptx);
    }

    #[test]
    fn test_detect_format_by_magic_bytes() {
        assert_eq!(FileParser::detect_format("unknown", "application/octet-stream", b"%PDF-1.4"), FileFormat::Pdf);
        let mut zip_header = vec![0x50, 0x4B, 0x03, 0x04];
        zip_header.extend_from_slice(b"some_content_here");
        assert_eq!(FileParser::detect_format("unknown.zip", "application/zip", &zip_header), FileFormat::Binary);
    }

    #[test]
    fn test_detect_format_by_mime() {
        assert_eq!(FileParser::detect_format("f", "text/html", b"<html></html>"), FileFormat::Html);
        assert_eq!(FileParser::detect_format("f", "text/plain", b"hello world"), FileFormat::PlainText);
        assert_eq!(FileParser::detect_format("f", "application/pdf", b"junk"), FileFormat::Pdf);
    }

    #[test]
    fn test_extract_plain_text() {
        let result = FileParser::extract_text("hello.txt", "text/plain", b"Hello, World!");
        assert!(result.parse_success);
        assert_eq!(result.text, "Hello, World!");
        assert_eq!(result.format, FileFormat::PlainText);
    }

    #[test]
    fn test_extract_code_file() {
        let result = FileParser::extract_text("main.rs", "text/x-rust", b"fn main() { println!(\"hello\"); }");
        assert!(result.parse_success);
        assert_eq!(result.format, FileFormat::Code);
        assert!(result.text.contains("fn main"));
    }

    #[test]
    fn test_extract_pdf_basic() {
        let result = FileParser::extract_text("doc.pdf", "application/pdf", b"%PDF-1.4\nBT\n(Hello World) Tj\nET");
        assert!(result.parse_success);
        assert!(result.text.contains("Hello World"));
    }

    #[test]
    fn test_size_limit() {
        let big = vec![b'a'; FileParser::MAX_EXTRACT_BYTES + 1000];
        let result = FileParser::extract_text("big.txt", "text/plain", &big);
        assert!(result.parse_success);
        assert!(result.text.len() <= FileParser::MAX_EXTRACT_BYTES + 100);
    }

    #[test]
    fn test_binary_file_no_panic() {
        let binary = vec![0x00, 0xFF, 0x01, 0x02, 0x03, 0x04];
        let result = FileParser::extract_text("data.bin", "application/octet-stream", &binary);
        assert!(!result.parse_success);
        assert_eq!(result.format, FileFormat::Binary);
    }

    #[test]
    fn test_docx_extraction() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>Hello from DOCX</w:t></w:r></w:p>
    <w:p><w:r><w:t>Second paragraph</w:t></w:r></w:p>
  </w:body>
</w:document>"#;
        let zip_data = make_zip_xml("word/document.xml", xml);
        let result = FileParser::extract_text("test.docx", "application/vnd.openxmlformats-officedocument.wordprocessingml.document", &zip_data);
        assert!(result.parse_success, "DOCX parse failed: {:?}", result.text);
        assert!(result.text.contains("Hello from DOCX"), "Missing text: {}", result.text);
        assert!(result.text.contains("Second paragraph"), "Missing text: {}", result.text);
    }

    #[test]
    fn test_pptx_extraction() {
        let xml = r#"<?xml version="1.0"?>
<p:sp xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:txBody>
    <a:p><a:r><a:t>Slide One Title</a:t></a:r></a:p>
    <a:p><a:r><a:t>Slide One Content</a:t></a:r></a:p>
  </p:txBody>
</p:sp>"#;
        let zip_data = make_zip_xml("ppt/slides/slide1.xml", xml);
        let result = FileParser::extract_text("test.pptx", "application/vnd.openxmlformats-officedocument.presentationml.presentation", &zip_data);
        assert!(result.parse_success, "PPTX parse failed: {:?}", result.text);
        assert!(result.text.contains("Slide One"), "Missing text: {}", result.text);
        assert!(result.text.contains("Slide 1"), "Missing slide number: {}", result.text);
    }

    #[test]
    fn test_cached_parse() {
        let data = b"cached content test";
        let r1 = FileParser::parse("cache_test.txt", "text/plain", data);
        let r2 = FileParser::parse("cache_test.txt", "text/plain", data);
        assert!(r1.parse_success);
        assert_eq!(r1.text, r2.text);
        FileParser::clear_cache();
    }

    #[test]
    fn test_extract_xml_text_helper() {
        let xml = b"<root><item>Hello</item><item>World</item></root>";
        let text = extract_xml_text(xml, &[b"item"]);
        assert_eq!(text, "Hello World");
    }

    #[test]
    fn test_extract_xml_text_nested() {
        let xml = b"<doc><p><span>nested</span> text</p></doc>";
        let text = extract_xml_text(xml, &[b"span"]);
        assert_eq!(text, "nested");
    }

    #[test]
    fn test_unknown_format_fallback() {
        let data = b"just some random text without extension";
        let result = FileParser::extract_text("no_ext", "text/plain", data);
        assert!(result.parse_success);
        assert_eq!(result.text, "just some random text without extension");
    }

    #[test]
    fn test_spatial_block_creation() {
        let block = SpatialBlock {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 12.0,
            text: "Hello".to_string(),
            block_type: BlockType::TextBlock,
        };
        assert_eq!(block.x, 10.0);
        assert_eq!(block.y, 20.0);
        assert_eq!(block.text, "Hello");
        assert_eq!(block.block_type, BlockType::TextBlock);
        assert_eq!(BlockType::ImageBlock as u8, 1);
        assert_eq!(BlockType::TableBlock as u8, 2);
        assert_eq!(BlockType::HeadingBlock as u8, 3);
    }

    #[test]
    fn test_pdf_spatial_extraction_with_tm_operator() {
        let pdf = b"%PDF-1.4\nBT\n1 0 0 1 100 700 Tm\n(Hello World) Tj\nET";
        let blocks = FileParser::extract_pdf_spatial(pdf);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].text, "Hello World");
        assert!((blocks[0].x - 100.0).abs() < 0.001);
        assert!((blocks[0].y - 700.0).abs() < 0.001);
    }

    #[test]
    fn test_grid_projection_order() {
        let pdf = b"%PDF-1.4\nBT\n1 0 0 1 100 700 Tm\n(First) Tj\n1 0 0 1 200 700 Tm\n(Second) Tj\n1 0 0 1 100 600 Tm\n(Third) Tj\nET";
        let blocks = FileParser::extract_pdf_spatial(pdf);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].text, "First");
        assert_eq!(blocks[1].text, "Second");
        assert_eq!(blocks[2].text, "Third");
    }

    #[test]
    fn test_parse_with_layout_sets_spatial_blocks() {
        let pdf = b"%PDF-1.4\nBT\n1 0 0 1 100 700 Tm\n(Layout Text) Tj\nET";
        let result = FileParser::parse_with_layout("doc.pdf", "application/pdf", pdf);
        assert!(result.parse_success);
        assert!(!result.spatial_blocks.is_empty());
        assert_eq!(result.spatial_blocks[0].text, "Layout Text");
    }

    #[test]
    fn test_xlsx_shared_strings() {
        let ss_xml = r#"<?xml version="1.0"?>
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <si><t>Apple</t></si>
  <si><t>Banana</t></si>
  <si><t>Cherry</t></si>
</sst>"#;
        let zip_data = make_zip_xml("xl/sharedStrings.xml", ss_xml);
        let result = FileParser::extract_text("test.xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", &zip_data);
        assert!(result.parse_success, "XLSX parse failed: {:?}", result.text);
        assert!(result.text.contains("Apple"), "Missing Apple: {}", result.text);
        assert!(result.text.contains("Banana"), "Missing Banana: {}", result.text);
        assert!(result.text.contains("Cherry"), "Missing Cherry: {}", result.text);
    }
}
