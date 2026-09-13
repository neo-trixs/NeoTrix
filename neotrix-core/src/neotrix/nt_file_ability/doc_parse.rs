/// anydoc-based document parsing (P0-1 absorption: firecrawl/anydoc 18.3k★)
/// Replaces markitdown + OfficeCLI dual-track with single Rust core.
/// 14 formats, content-based detection, shared Document model, GFM serializer.

use anydoc::{Format, to_document};
use anydoc::model::{Document, Block, Inline, Table, MarkerKind, ListItem, CellSlot, Cell};
use serde::{Serialize, Deserialize};
use crate::neotrix::nt_file_ability::types::{FileModel, ParseError, ParseResult};
use std::path::Path;

/// Parse any supported document format to unified FileModel.
pub fn parse_document(path: &Path) -> ParseResult<FileModel> {
    let format = Format::from_path(path).ok_or_else(|| ParseError::UnsupportedFormat)?;
    
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
    let headers = grid.first().map(|row| row.iter().map(|c| cell_text(c)).collect::<Vec<_>>()).unwrap_or_default();
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