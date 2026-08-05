use std::path::Path;
use super::super::doc_parser::{DocParser, ParsedDocument, ParseTier};

/// Text-only LLM 后端 — 使用 Pollinations/Groq 免费 API 做纯文本重构
/// 不支持图片/PDF直接解析, 仅供纯文本重建
pub struct TextOnlyBackend;

impl DocParser for TextOnlyBackend {
    fn parse_pdf(&self, _path: &Path) -> Result<ParsedDocument, String> {
        Err("TextOnlyBackend does not parse PDFs directly; use for text restructuring only".into())
    }

    fn parse_image(&self, _path: &Path) -> Result<ParsedDocument, String> {
        Err("TextOnlyBackend does not support image parsing (no vision capability)".into())
    }

    fn supported_formats(&self) -> Vec<&str> {
        vec!["text", "markdown"]
    }

    fn tier(&self) -> ParseTier {
        ParseTier::Tier0Fast
    }
}
