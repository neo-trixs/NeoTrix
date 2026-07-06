use std::path::Path;
use super::super::doc_parser::{DocParser, ParsedDocument, ParseTier};

/// Docling 文档解析后端 — Tier 1 混合
/// Python subprocess 调用 docling, 返回 JSON DoclingDocument 树
pub struct DoclingBackend;

impl DocParser for DoclingBackend {
    fn parse_pdf(&self, _path: &Path) -> Result<ParsedDocument, String> {
        Err("DoclingBackend not yet implemented — requires Python docling package".into())
    }

    fn parse_image(&self, _path: &Path) -> Result<ParsedDocument, String> {
        Err("DoclingBackend image parsing not yet implemented".into())
    }

    fn supported_formats(&self) -> Vec<&str> {
        vec!["pdf", "docx", "pptx", "xlsx", "html", "png", "jpg"]
    }

    fn tier(&self) -> ParseTier {
        ParseTier::Tier1Hybrid
    }
}
