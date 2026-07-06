use std::path::Path;
use super::super::doc_parser::{DocParser, ParsedDocument, ParseTier};

/// Marker PDF-to-Markdown 后端 — Tier 1 混合
/// Python subprocess 调用 marker_single, --use_llm 可选
pub struct MarkerBackend;

impl DocParser for MarkerBackend {
    fn parse_pdf(&self, _path: &Path) -> Result<ParsedDocument, String> {
        Err("MarkerBackend not yet implemented — requires Python marker package".into())
    }

    fn parse_image(&self, _path: &Path) -> Result<ParsedDocument, String> {
        Err("MarkerBackend image parsing not yet implemented".into())
    }

    fn supported_formats(&self) -> Vec<&str> {
        vec!["pdf"]
    }

    fn tier(&self) -> ParseTier {
        ParseTier::Tier1Hybrid
    }
}
