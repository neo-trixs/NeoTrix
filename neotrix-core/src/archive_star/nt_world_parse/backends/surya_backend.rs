use std::path::Path;
use super::super::doc_parser::{DocParser, ParsedDocument, ParseTier};

/// surya OCR+layout 后端 — Tier 1 混合
/// Python subprocess 调用 surya OCR + 布局检测
pub struct SuryaBackend;

impl DocParser for SuryaBackend {
    fn parse_pdf(&self, _path: &Path) -> Result<ParsedDocument, String> {
        Err("SuryaBackend not yet implemented — requires Python surya package".into())
    }

    fn parse_image(&self, _path: &Path) -> Result<ParsedDocument, String> {
        Err("SuryaBackend image parsing not yet implemented".into())
    }

    fn supported_formats(&self) -> Vec<&str> {
        vec!["pdf", "png", "jpg", "jpeg", "tiff"]
    }

    fn tier(&self) -> ParseTier {
        ParseTier::Tier1Hybrid
    }
}
