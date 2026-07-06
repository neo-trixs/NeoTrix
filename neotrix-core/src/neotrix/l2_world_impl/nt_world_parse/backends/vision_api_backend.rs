use std::path::Path;
use super::super::doc_parser::{DocParser, ParsedDocument, ParseTier};

/// Gemini / GPT-4V Vision API 后端 — Tier 2 GPU
/// 通过 GatewayV2 VLM provider 调用
pub struct VisionApiBackend;

impl DocParser for VisionApiBackend {
    fn parse_pdf(&self, _path: &Path) -> Result<ParsedDocument, String> {
        Err("VisionApiBackend not yet implemented — requires GatewayV2 VLM provider".into())
    }

    fn parse_image(&self, _path: &Path) -> Result<ParsedDocument, String> {
        Err("VisionApiBackend image parsing not yet implemented".into())
    }

    fn supported_formats(&self) -> Vec<&str> {
        vec!["pdf", "png", "jpg", "jpeg", "webp"]
    }

    fn tier(&self) -> ParseTier {
        ParseTier::Tier2Vlm
    }
}
