use std::collections::HashMap;
use std::path::Path;
use super::doc_parser::{DocParser, ParsedDocument, ParseTier};

#[cfg(feature = "office")]
use super::backends::office_oxide_backend::OfficeOxideBackend;

fn is_office_ext(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| matches!(e.to_lowercase().as_str(), "docx" | "pptx" | "xlsx"))
        .unwrap_or(false)
}

fn is_pdf_ext(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase() == "pdf")
        .unwrap_or(false)
}

/// ParseGateway — 多后端路由 + 置信度审计 + 自动回退
///
/// 模式: router, not extractor (参考 pdfmux)
/// 策略: Tier 0 → audit → ≥阈值? 返回 : Tier 1 重试 → audit → Tier 2 重试
pub struct ParseGateway {
    backends: HashMap<String, Box<dyn DocParser>>,
    min_confidence: f64,
    retry_enabled: bool,
}

impl Default for ParseGateway {
    fn default() -> Self {
        Self::new()
    }
}

impl ParseGateway {
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut backends: HashMap<String, Box<dyn DocParser>> = HashMap::new();

        #[cfg(feature = "office")]
        backends.insert("office_oxide".into(), Box::new(OfficeOxideBackend));

        Self {
            backends,
            min_confidence: 0.75,
            retry_enabled: true,
        }
    }

    pub fn register_backend(&mut self, name: &str, parser: Box<dyn DocParser>) {
        self.backends.insert(name.to_string(), parser);
    }

    pub fn set_min_confidence(&mut self, threshold: f64) {
        self.min_confidence = threshold.clamp(0.0, 1.0);
    }

    /// Auto-detect file format and route to the right parser.
    pub fn parse(&self, path: &Path) -> Result<ParsedDocument, String> {
        if is_office_ext(path) {
            self.parse_office(path)
        } else if is_pdf_ext(path) {
            self.parse_pdf(path)
        } else {
            Err(format!("unsupported file format: {}", path.display()))
        }
    }

    pub fn parse_pdf(&self, path: &Path) -> Result<ParsedDocument, String> {
        let tier0 = self.select_backend(ParseTier::Tier0Fast)
            .ok_or("No Tier0 backend registered")?;
        let mut doc = tier0.parse_pdf(path)?;

        if self.retry_enabled {
            for page in doc.pages.iter_mut() {
                if page.confidence < self.min_confidence {
                    if let Some(tier1) = self.select_backend(ParseTier::Tier1Hybrid) {
                        if let Ok(retry) = tier1.parse_pdf(path) {
                            if let Some(rp) = retry.pages.iter().find(|p| p.page_num == page.page_num) {
                                if rp.confidence > page.confidence {
                                    *page = rp.clone();
                                }
                            }
                        }
                    }
                }
                if page.confidence < self.min_confidence {
                    if let Some(tier2) = self.select_backend(ParseTier::Tier2Vlm) {
                        if let Ok(retry) = tier2.parse_pdf(path) {
                            if let Some(rp) = retry.pages.iter().find(|p| p.page_num == page.page_num) {
                                if rp.confidence > page.confidence {
                                    *page = rp.clone();
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(doc)
    }

    /// Parse office documents (DOCX, PPTX, XLSX) — no retry tiers needed.
    pub fn parse_office(&self, path: &Path) -> Result<ParsedDocument, String> {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();
        for backend in self.backends.values() {
            if backend.supported_formats().iter().any(|f| *f == ext) {
                return backend.parse_office(path);
            }
        }
        Err(format!("no office parser registered for: {}", path.display()))
    }

    fn select_backend(&self, tier: ParseTier) -> Option<&dyn DocParser> {
        self.backends.values()
            .find(|b| b.tier() == tier)
            .map(|v| &**v)
    }

    pub fn provider_status(&self) -> Vec<(&str, ParseTier)> {
        self.backends.iter()
            .map(|(name, b)| (name.as_str(), b.tier()))
            .collect()
    }
}
