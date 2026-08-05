use std::path::Path;
use super::super::doc_parser::{DocParser, ParsedDocument, ParseTier, PageResult};

/// PyMuPDF 直接文本提取 — Tier 0 最快路径
/// 使用 lopdf crate 提取纯文本, ~0.01s/page
pub struct PyMuPDFBackend;

impl DocParser for PyMuPDFBackend {
    fn parse_pdf(&self, path: &Path) -> Result<ParsedDocument, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
        let doc = lopdf::Document::load_mem(&bytes)
            .map_err(|e| format!("lopdf parse error: {}", e))?;
        let mut pages = Vec::new();
        let page_count = doc.get_pages().len();

        for (page_num, _page_entry) in doc.get_pages().iter().enumerate() {
            let pn = page_num as u32 + 1;
            let text = doc.extract_text(&[pn]).unwrap_or_default();
            pages.push(PageResult {
                page_num: pn as usize,
                markdown: text.clone(),
                confidence: if text.len() > 50 { 0.85 } else { 0.5 },
                backend_used: "PyMuPDFBackend".into(),
                metadata: Default::default(),
            });
        }

        let full_md = pages.iter().map(|p| p.markdown.clone()).collect::<Vec<_>>().join("\n\n---\n\n");
        let avg_conf = if pages.is_empty() { 0.0 } else { pages.iter().map(|p| p.confidence).sum::<f64>() / pages.len() as f64 };

        Ok(ParsedDocument {
            title: path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string()),
            pages,
            full_markdown: full_md,
            full_json: serde_json::json!({"pages": page_count}),
            avg_confidence: avg_conf,
            metadata: Default::default(),
        })
    }

    fn parse_image(&self, _path: &Path) -> Result<ParsedDocument, String> {
        Err("PyMuPDFBackend does not support image parsing".into())
    }

    fn supported_formats(&self) -> Vec<&str> {
        vec!["pdf"]
    }

    fn tier(&self) -> ParseTier {
        ParseTier::Tier0Fast
    }
}
