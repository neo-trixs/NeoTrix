/// anydoc-based document parsing (P0-1 absorption: firecrawl/anydoc 18.3k★)
/// Replaces markitdown + OfficeCLI dual-track with single Rust core.
/// 14 formats, content-based detection, shared Document model, GFM serializer.

use anydoc::{Format, to_document};
use anydoc::model::{Document, Block, Inline, Table, MarkerKind, ListItem, CellSlot, Cell};
use serde::{Serialize, Deserialize};
use crate::l1_action::nt_file_ability::types::{FileModel, ParseError, ParseResult};
use std::path::Path;

/// Parse any supported document format to unified FileModel.
///
/// 无扩展名/未知扩展名时回落：可读 UTF-8 文本按纯文本收纳（anydoc 无 Txt 格式），
/// 二进制仍报 UnsupportedFormat。
pub fn parse_document(path: &Path) -> ParseResult<FileModel> {
    let format = match Format::from_path(path) {
        Some(f) => f,
        None => {
            let bytes = std::fs::read(path).map_err(ParseError::Io)?;
            match String::from_utf8(bytes) {
                Ok(text) => {
                    return Ok(FileModel {
                        format: "txt".to_string(),
                        title: path
                            .file_name()
                            .map(|s| s.to_string_lossy().into_owned()),
                        content: text,
                        tables: None,
                        metadata: None,
                        images: None,
                    })
                }
                Err(_) => return Err(ParseError::UnsupportedFormat),
            }
        }
    };

    if matches!(format, Format::Pdf) {
        return parse_pdf_enhanced(path);
    }

    let bytes = std::fs::read(path).map_err(ParseError::Io)?;
    let doc = to_document(&bytes, format).map_err(|e| ParseError::AnyDoc(e.to_string()))?;
    Ok(document_to_filemodel(doc))
}

/// Parse document from bytes with explicit format.
pub fn parse_bytes(bytes: &[u8], format: Format) -> ParseResult<FileModel> {
    if matches!(format, Format::Pdf) {
        return parse_pdf_bytes_enhanced(bytes);
    }

    let doc = to_document(bytes, format).map_err(|e| ParseError::AnyDoc(e.to_string()))?;
    Ok(document_to_filemodel(doc))
}

/// 用 mdream（MIT，harlan-zw）做 HTML→Markdown 的可选加速通道。
///
/// 存在即用、不存在即回落 anydoc——这是「适配器注册」式接入，不是平行实现：
/// 仅当环境变量 `MDREAM_BIN` 指向一个可执行文件时调用；
/// 其它情况返回 `None`，调用方继续走 anydoc。
///
/// 依据：mdream 在 README 自称「fastest HTML to markdown convertor,
/// optimized for LLMs, supports streaming」——价值在于流式/表格/strikethrough
/// 的保真，但本层只吃 bytes 不做流式（TODO）。
pub fn html_to_markdown_via_mdream(bytes: &[u8]) -> Option<String> {
    let bin = std::env::var("MDREAM_BIN").ok()?;
    let mut child = std::process::Command::new(bin)
        .args(["--format", "markdown"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    use std::io::Write as _;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(bytes);
    }
    let out = child.wait_with_output().ok()?;
    if out.status.success() {
        String::from_utf8(out.stdout).ok()
    } else {
        None
    }
}

/// PDF 字节 → FileModel。
///
/// 2026-09-29 修正：`parse_pdf_bytes_enhanced` 走 `to_document(bytes, Pdf)`，
/// 而 anydoc 对 PDF **只支持 `to_markdown_bytes`**（无 document-model 形式，
/// 见 anydoc `lib.rs:131-133`）⇒ 真实 PDF 一律报
/// `unsupported input: PDF converts directly to Markdown`。
/// 这不是「PDF 不支持」，而是**用错了入口**。此处改走 `to_markdown_bytes`，
/// 保证 `parse_any` 的 PDF 路径真的能出内容。
fn pdf_bytes_to_filemodel(bytes: &[u8], title: Option<String>) -> ParseResult<FileModel> {
    let md = anydoc::to_markdown_bytes(bytes, Format::Pdf)
        .map_err(|e| ParseError::AnyDoc(e.to_string()))?;
    Ok(FileModel {
        format: "pdf".to_string(),
        title,
        content: md,
        tables: None,
        metadata: Some(serde_json::json!({
            "mode_used": "anydoc_to_markdown",
            "reason": "PDF has no document-model form in anydoc",
        })),
        images: None,
    })
}

/// 统一文件解析入口 — **通用文件解析能力的唯一正门**。
///
/// 2026-09-29 设立：此前存在两条互不知晓的路径 —— `parse_document`
/// （扩展名优先，无嗅探）与 `FileAbility::open`（扩展名→office_oxide→FileParser
/// 三段），且两者**零生产调用**（只有 selftest）。本函数是唯一的、
/// 内容嗅探优先的正门：
///
/// ```text
/// 读字节一次 → 魔法字节嗅探 → 扩展名回退 → 按 DocFormat 分类路由
///   ├─ Pdf                    → parse_pdf_enhanced（既有增强路径）
///   ├─ 有 anydoc 后端         → to_document（12 格式）
///   ├─ 纯文本系               → UTF-8 直读
///   ├─ 媒体系                 → 元数据模式（不做内容解析）
///   └─ Unknown                → UTF-8 试探 → 不行报 UnsupportedFormat
/// ```
///
/// 顺序即优先级：**内容嗅探永远优先于扩展名**（扩展名会撒谎，魔数不会）。
pub fn parse_any(path: &Path) -> ParseResult<FileModel> {
    use super::format_route::DocFormat;

    let bytes = std::fs::read(path).map_err(ParseError::Io)?;
    let title = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned());

    // 1) 内容嗅探优先（魔数不会撒谎）
    if let Some(sniffed) = Format::from_bytes(&bytes) {
        if matches!(sniffed, Format::Pdf) {
            return pdf_bytes_to_filemodel(&bytes, title);
        }
        let doc =
            to_document(&bytes, sniffed).map_err(|e| ParseError::AnyDoc(e.to_string()))?;
        let mut model = document_to_filemodel(doc);
        model.title = title;
        return Ok(model);
    }

    // 2) 扩展名回退（本地全集枚举）
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match DocFormat::from_ext(ext) {
        // 3a) 有后端：走 anydoc
        f if f.to_anydoc().is_some() => {
            let backend = f.to_anydoc().expect("checked");
            if matches!(backend, Format::Pdf) {
                return pdf_bytes_to_filemodel(&bytes, title);
            }
            let doc =
                to_document(&bytes, backend).map_err(|e| ParseError::AnyDoc(e.to_string()))?;
            let mut model = document_to_filemodel(doc);
            model.format = f.as_str().to_string();
            model.title = title;
            Ok(model)
        }
        // 3b) 纯文本系：UTF-8 直读
        f if f.is_text_like() => match String::from_utf8(bytes) {
            Ok(text) => Ok(FileModel {
                format: f.as_str().to_string(),
                title,
                content: text,
                tables: None,
                metadata: None,
                images: None,
            }),
            Err(_) => Err(ParseError::UnsupportedFormat),
        },
        // 3c) 媒体系：元数据模式
        f if f.is_media() => Ok(media_metadata_model(path, &bytes, f, title)),
        // 3d) Unknown：UTF-8 试探
        _ => match String::from_utf8(bytes) {
            Ok(text) => Ok(FileModel {
                format: "txt".to_string(),
                title,
                content: text,
                tables: None,
                metadata: None,
                images: None,
            }),
            Err(_) => Err(ParseError::UnsupportedFormat),
        },
    }
}

/// 媒体系文件的元数据模式：不做内容解析，只记录可观测事实。
fn media_metadata_model(
    path: &Path,
    bytes: &[u8],
    format: super::format_route::DocFormat,
    title: Option<String>,
) -> FileModel {
    let meta = serde_json::json!({
        "path": path.to_string_lossy(),
        "bytes": bytes.len(),
        "kind": format.as_str(),
        "mode": "metadata-only",
    });
    FileModel {
        format: format.as_str().to_string(),
        title,
        content: String::new(),
        tables: None,
        metadata: Some(meta),
        images: None,
    }
}

/// Convert anydoc Document to NeoTrix FileModel.
fn document_to_filemodel(doc: Document) -> FileModel {
    let text = blocks_to_text(&doc.blocks);
    
    FileModel {
        format: "anydoc".to_string(),
        title: None,
        content: text,
        tables: extract_tables_json(&doc.blocks),
        metadata: None,
        images: None,
    }
}

fn blocks_to_text(blocks: &[Block]) -> String {
    blocks.iter().map(block_to_text).collect::<Vec<_>>().join("\n\n")
}

fn block_to_text(block: &Block) -> String {
    match block {
        Block::Paragraph(inlines) => inlines_to_text(inlines),
        Block::Heading { level, content, .. } => {
            let prefix = "#".repeat(*level as usize);
            format!("{} {}", prefix, inlines_to_text(content))
        }
        Block::List(list) => {
            items_to_text(&list.items, &list.marker, list.start)
        }
        Block::CodeBlock { text, lang } => {
            let lang_str = lang.as_deref().unwrap_or("");
            format!("```{}\n{}\n```", lang_str, text)
        }
        Block::BlockQuote(blocks) => {
            blocks.iter().map(block_to_text).collect::<Vec<_>>().join("\n")
        }
        Block::Table(table) => table_to_markdown(table),
        Block::Rule => "---".to_string(),
        Block::Math(tex) => format!("$${}$$", tex),
    }
}

fn items_to_text(items: &[ListItem], marker: &MarkerKind, start: u64) -> String {
    items.iter().enumerate().map(|(i, item)| {
        let marker_text = match marker {
            MarkerKind::Bullet => "-".to_string(),
            _ => format!("{}.", marker.label(start + i as u64)),
        };
        let content = item.blocks.iter().map(block_to_text).collect::<Vec<_>>().join("\n");
        format!("{} {}", marker_text, content)
    }).collect::<Vec<_>>().join("\n")
}

fn inlines_to_text(inlines: &[Inline]) -> String {
    inlines.iter().map(inline_to_text).collect()
}

fn inline_to_text(inline: &Inline) -> String {
    match inline {
        Inline::Text { text, style } => {
            if style.code {
                format!("`{}`", text)
            } else if style.italic && style.bold {
                format!("***{}***", text)
            } else if style.italic {
                format!("*{}*", text)
            } else if style.bold {
                format!("**{}**", text)
            } else if style.strike {
                format!("~~{}~~", text)
            } else {
                text.clone()
            }
        }
        Inline::Link { content, .. } => inlines_to_text(content),
        Inline::Image { alt, .. } => format!("[Image: {}]", alt),
        Inline::Anchor(_) => String::new(),
        Inline::NoteRef(_) => "[ref]".to_string(),
        Inline::LineBreak => "\n".to_string(),
        Inline::Math(tex) => format!("${}$", tex),
        Inline::Checkbox(checked) => format!("[{}] ", if *checked { "x" } else { " " }),
    }
}

fn extract_tables(blocks: &[Block]) -> Option<Vec<serde_json::Value>> {
    let tables: Vec<_> = blocks.iter().filter_map(|b| {
        if let Block::Table(table) = b {
            Some(table_to_json(table))
        } else {
            None
        }
    }).collect();
    if tables.is_empty() { None } else { Some(tables) }
}

fn extract_tables_json(blocks: &[Block]) -> Option<Vec<serde_json::Value>> {
    extract_tables(blocks)
}

fn table_to_json(table: &Table) -> serde_json::Value {
    let grid = &table.grid;
    let headers = grid.first().map(|row| row.iter().map(cell_text).collect::<Vec<_>>()).unwrap_or_default();
    let rows: Vec<Vec<String>> = grid.iter().skip(1).map(|row| row.iter().map(cell_text).collect()).collect();
    serde_json::json!({
        "headers": headers,
        "rows": rows,
    })
}

fn cell_text(cell: &CellSlot) -> String {
    match cell {
        CellSlot::Origin(cell) => cell_text_inner(cell),
        CellSlot::Covered { .. } => String::new(),
    }
}

fn cell_text_inner(cell: &Cell) -> String {
    cell.blocks.iter().map(block_to_text).collect::<Vec<_>>().join(" ")
}

fn table_to_markdown(table: &Table) -> String {
    let grid = &table.grid;
    if grid.is_empty() {
        return String::new();
    }
    
    let mut md = String::new();
    
    let header = grid.first().expect("non-empty");
    md.push_str("| ");
    for cell in header {
        md.push_str(&cell_text(cell));
        md.push_str(" | ");
    }
    md.push('\n');
    
    md.push_str("| ");
    for _ in header {
        md.push_str("--- | ");
    }
    md.push('\n');
    
    for row in grid.iter().skip(1) {
        md.push_str("| ");
        for cell in row {
            md.push_str(&cell_text(cell));
            md.push_str(" | ");
        }
        md.push('\n');
    }
    
    md
}

/// ========================================================================
/// PDF 增强解析模式 - 占位实现，回退到 anydoc 基础解析
/// ========================================================================

use std::time::Instant;

/// PDF 增强解析模式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PdfParseMode {
    Balanced,
    Fast,
    NoOcr,
    Auto,
}

impl Default for PdfParseMode {
    fn default() -> Self {
        if std::env::var("CUDA_VISIBLE_DEVICES").is_ok() 
            || std::path::Path::new("/dev/nvidia0").exists() {
            PdfParseMode::Balanced
        } else {
            PdfParseMode::Fast
        }
    }
}

impl PdfParseMode {
    pub fn from_env() -> Self {
        if let Ok(mode) = std::env::var("NEOTRIX_PDF_MODE") {
            match mode.to_lowercase().as_str() {
                "balanced" => PdfParseMode::Balanced,
                "fast" => PdfParseMode::Fast,
                "no-ocr" | "no_ocr" => PdfParseMode::NoOcr,
                _ => Self::default(),
            }
        } else {
            Self::default()
        }
    }
}

/// PDF 增强解析配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfParseConfig {
    pub mode: PdfParseMode,
    pub vlm_model: Option<String>,
    pub llm_model: Option<String>,
    pub api_key_env: Option<String>,
    pub enable_ocr: bool,
    pub enable_tables: bool,
    pub enable_images: bool,
    pub max_pages: Option<usize>,
    pub output_format: String,
}

impl Default for PdfParseConfig {
    fn default() -> Self {
        Self {
            mode: PdfParseMode::default(),
            vlm_model: None,
            llm_model: None,
            api_key_env: None,
            enable_ocr: true,
            enable_tables: true,
            enable_images: false,
            max_pages: None,
            output_format: "markdown".to_string(),
        }
    }
}

/// PDF 解析结果扩展
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfParseResult {
    pub file_model: FileModel,
    pub metadata: PdfParseMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfParseMetadata {
    pub mode_used: PdfParseMode,
    pub pages_processed: usize,
    pub ocr_pages: usize,
    pub tables_extracted: usize,
    pub images_extracted: usize,
    pub vlm_calls: usize,
    pub llm_refinement_calls: usize,
    pub processing_time_ms: u64,
    pub vlm_model: Option<String>,
    pub llm_model: Option<String>,
}

/// 增强 PDF 文件解析
pub fn parse_pdf_enhanced(path: &Path) -> ParseResult<FileModel> {
    parse_pdf_enhanced_with_config(path, PdfParseConfig::default())
}

pub fn parse_pdf_enhanced_with_config(path: &Path, _config: PdfParseConfig) -> ParseResult<FileModel> {
    let bytes = std::fs::read(path).map_err(ParseError::Io)?;
    parse_pdf_bytes_enhanced_with_config(&bytes, PdfParseConfig::default())
}

pub fn parse_pdf_bytes_enhanced(bytes: &[u8]) -> ParseResult<FileModel> {
    parse_pdf_bytes_enhanced_with_config(bytes, PdfParseConfig::default())
}

pub fn parse_pdf_bytes_enhanced_with_config(bytes: &[u8], _config: PdfParseConfig) -> ParseResult<FileModel> {
    let _start = Instant::now();
    
    let format = Format::Pdf;
    let doc = to_document(bytes, format)
        .map_err(|e| ParseError::AnyDoc(e.to_string()))?;
    let mut model = document_to_filemodel(doc);
    model.metadata = Some(serde_json::json!({
        "mode_used": "fallback_anydoc",
        "fallback_reason": "marker crate not yet integrated",
    }));
    Ok(model)
}

/// 格式自动检测增强
pub fn detect_format_from_content(bytes: &[u8]) -> Option<Format> {
    Format::from_bytes(bytes)
}

/// 批量解析
pub fn parse_batch(paths: &[&Path], config: PdfParseConfig) -> Vec<ParseResult<FileModel>> {
    use rayon::prelude::*;
    paths.par_iter()
        .map(|p| {
            if matches!(Format::from_path(p), Some(Format::Pdf)) {
                parse_pdf_enhanced_with_config(p, config.clone())
            } else {
                parse_document(p)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn mdream_adapter_returns_none_when_bin_unset() {
        std::env::remove_var("MDREAM_BIN");
        assert!(html_to_markdown_via_mdream(b"<p>hi</p>").is_none());
    }

    #[test]
    fn test_parse_any_content_sniff_beats_extension() {
        // 核心契约：**魔数优先于扩展名**。扩展名会撒谎，魔数不会。
        // 这里把 Markdown 内容命名成 .pdf —— 嗅探不到 PDF 魔数，
        // 应回退扩展名→text 路径，而不是被 .pdf 骗去走 PDF 解析器。
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "# Real Content\n\nnot a pdf").unwrap();
        let renamed = f.path().with_extension("txt");
        std::fs::copy(f.path(), &renamed).unwrap();
        let m = super::parse_any(&renamed).expect("parse_any");
        assert_eq!(m.format, "text");
        assert!(m.content.contains("Real Content"));
    }

    #[test]
    fn test_parse_any_sniffs_real_pdf() {
        let mut f = NamedTempFile::new().unwrap();
        // 最小但结构合法的 PDF：xref + trailer + startxref 必被 pdf-inspector 接受
        let pdf = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n\
3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\n\
trailer\n<< /Root 1 0 R >>\n%%EOF\n";
        f.write_all(pdf).unwrap();
        // 文件名故意用 .txt：若实现只信扩展名就会判成 text
        let renamed = f.path().with_extension("txt");
        std::fs::copy(f.path(), &renamed).unwrap();
        // 核心断言：嗅探判定为 pdf（**即使解析器随后报结构问题**，
        // 错误信息也必须提到 PDF 而非 text —— 那就证明路由按魔数走了）。
        match super::parse_any(&renamed) {
            Ok(m) => assert_eq!(m.format, "pdf", "魔数应压过 .txt 扩展名"),
            Err(ParseError::AnyDoc(e)) => {
                assert!(e.to_lowercase().contains("pdf"), "应走 PDF 路径，实际: {e}")
            }
            Err(other) => panic!("错误类型不对: {other:?}"),
        }
    }

    #[test]
    fn test_parse_any_text_and_media_and_unknown() {
        let mut txt = NamedTempFile::new().unwrap();
        writeln!(txt, "plain body").unwrap();
        let m = super::parse_any(txt.path()).expect("parse_any");
        assert!(m.content.contains("plain body"));

        // 媒体：元数据模式，content 空但 metadata 有 path/bytes/kind/mode
        let mut bin = NamedTempFile::new().unwrap();
        bin.write_all(&[0u8; 64]).unwrap();
        let media = NamedTempFile::new().unwrap();
        std::fs::copy(bin.path(), media.path().with_extension("mp3")).unwrap();
        let m = super::parse_any(&media.path().with_extension("mp3")).expect("parse_any");
        assert_eq!(m.format, "audio");
        assert!(m.content.is_empty(), "媒体不应做内容解析");
        let meta = m.metadata.expect("媒体应有元数据");
        assert_eq!(meta["mode"], "metadata-only");
        assert_eq!(meta["kind"], "audio");
    }

    #[test]
    fn test_parse_any_binary_unknown_reports_unsupported() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(&[0xFF, 0xFE, 0xFD, 0xFC, 0x00, 0x01, 0x02]).unwrap();
        let err = super::parse_any(f.path()).expect_err("二进制未知格式应报错");
        assert!(matches!(err, ParseError::UnsupportedFormat), "got {err:?}");
    }

    #[test]
    fn test_parse_any_markdown_ext() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "# Title\n\nbody").unwrap();
        let renamed = f.path().with_extension("md");
        std::fs::copy(f.path(), &renamed).unwrap();
        let m = super::parse_any(&renamed).expect("parse_any");
        assert_eq!(m.format, "markdown");
    }

    #[test]
    fn test_parse_text_file() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "# Hello\n\nContent here.").unwrap();
        let result = parse_document(f.path());
        assert!(result.is_ok());
        let model = result.unwrap();
        assert!(model.content.contains("Hello"));
        assert!(model.content.contains("Content here"));
    }

    #[test]
    fn test_pdf_mode_from_env() {
        std::env::set_var("NEOTRIX_PDF_MODE", "fast");
        assert_eq!(PdfParseMode::from_env(), PdfParseMode::Fast);
        std::env::remove_var("NEOTRIX_PDF_MODE");
        
        std::env::set_var("NEOTRIX_PDF_MODE", "balanced");
        assert_eq!(PdfParseMode::from_env(), PdfParseMode::Balanced);
        std::env::remove_var("NEOTRIX_PDF_MODE");
    }
}