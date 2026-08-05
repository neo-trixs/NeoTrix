use std::path::Path;
use std::collections::HashMap;

/// 文档解析统一 trait
pub trait DocParser: Send + Sync {
    fn parse_pdf(&self, path: &Path) -> Result<ParsedDocument, String>;
    fn parse_image(&self, path: &Path) -> Result<ParsedDocument, String>;
    fn parse_office(&self, path: &Path) -> Result<ParsedDocument, String> {
        let _ = path;
        Err("office parsing not supported by this backend".into())
    }
    fn supported_formats(&self) -> Vec<&str>;
    fn tier(&self) -> ParseTier;
}

/// 解析后端层级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseTier {
    Tier0Fast,    // PyMuPDF / text-only LLM, 0.01s/page
    Tier1Hybrid,  // surya + Marker/Docling, 0.1-1s/page
    Tier2Vlm,     // olmOCR / Gemini Vision, 1-10s/page
}

/// 单页解析结果
#[derive(Debug, Clone)]
pub struct PageResult {
    pub page_num: usize,
    pub markdown: String,
    pub confidence: f64,
    pub backend_used: String,
    pub metadata: HashMap<String, String>,
}

/// 完整文档解析结果
#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub title: Option<String>,
    pub pages: Vec<PageResult>,
    pub full_markdown: String,
    pub full_json: serde_json::Value,
    pub avg_confidence: f64,
    pub metadata: HashMap<String, String>,
}
