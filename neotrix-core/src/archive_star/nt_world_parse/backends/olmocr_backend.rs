use std::path::Path;
use super::super::doc_parser::{DocParser, ParsedDocument, ParseTier};

/// olmOCR 远程 VLM 后端 — Tier 2 GPU
/// HTTP 请求到 olmocr --server 实例或 AI2 托管 API
pub struct OlmocrBackend;

impl DocParser for OlmocrBackend {
    fn parse_pdf(&self, _path: &Path) -> Result<ParsedDocument, String> {
        Err("OlmocrBackend not yet implemented — requires running olmOCR server".into())
    }

    fn parse_image(&self, _path: &Path) -> Result<ParsedDocument, String> {
        Err("OlmocrBackend image parsing not yet implemented".into())
    }

    fn supported_formats(&self) -> Vec<&str> {
        vec!["pdf", "png", "jpg", "jpeg", "tiff"]
    }

    fn tier(&self) -> ParseTier {
        ParseTier::Tier2Vlm
    }
}
