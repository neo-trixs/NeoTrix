#![allow(dead_code)]

use super::document_classifier::DocumentClassifier;
use super::document_parser::{
    detect_format, DocumentError, DocumentParser, DocumentSource, ParsedDocument,
};
use super::layout_analyzer::LayoutAnalyzer;

type BoxedParser = Box<dyn DocumentParser>;

/// Routes document sources to the appropriate parsing backend.
/// Maintains an ordered list of parsers; first match wins.
pub struct DocumentParserRegistry {
    parsers: Vec<BoxedParser>,
    default_index: usize,
}

// Manual Clone: creates a fresh empty registry (parsers are not deep-cloned).
// This is consistent with ConsciousnessCycle's Clone pattern where ModuleRegistry
// is reconstructed via `ModuleRegistry::new()` instead of deep-cloned.
impl Clone for DocumentParserRegistry {
    fn clone(&self) -> Self {
        Self {
            parsers: Vec::new(), // parsers must be re-registered after clone
            default_index: self.default_index,
        }
    }
}

/// Reconstruct a default registry with standard backends.
/// Used after cloning to restore parsers.
impl DocumentParserRegistry {
    pub fn refill_defaults(&mut self) {
        self.register(Box::new(
            super::pdf_extractor_backend::PdfExtractorBackend::new(),
        ));
        self.register(Box::new(
            super::xlsx_extractor_backend::XlsxExtractorBackend::new(),
        ));
        self.register(Box::new(
            super::docx_extractor_backend::DocxExtractorBackend::new(),
        ));
        self.register(Box::new(
            super::xls_extractor_backend::XlsExtractorBackend::new(),
        ));
    }
}

/// Metadata about a registered backend for introspection.
#[derive(Debug, Clone)]
pub struct BackendInfo {
    pub name: String,
    pub formats: Vec<String>,
}

impl DocumentParserRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            parsers: Vec::new(),
            default_index: 0,
        }
    }

    /// Register a parser (registered later = higher priority for format matching).
    pub fn register(&mut self, parser: BoxedParser) {
        self.parsers.push(parser);
    }

    /// Register multiple parsers at once.
    pub fn register_all(&mut self, parsers: Vec<BoxedParser>) {
        self.parsers.extend(parsers);
    }

    /// Set the default parser index (used when no format match is found).
    pub fn with_default(mut self, index: usize) -> Self {
        self.default_index = index.min(self.parsers.len().saturating_sub(1));
        self
    }

    /// Find the best parser for a given document format.
    pub fn find_parser(&self, format: &str) -> Option<&BoxedParser> {
        let lower = format.to_lowercase();
        for parser in &self.parsers {
            if parser
                .supported_formats()
                .iter()
                .any(|f| f.eq_ignore_ascii_case(&lower))
            {
                return Some(parser);
            }
        }
        self.parsers.get(self.default_index)
    }

    /// Parse a document by auto-detecting the best backend.
    pub fn parse(&self, source: &DocumentSource) -> Result<ParsedDocument, DocumentError> {
        let format = match source {
            DocumentSource::File(path) => detect_format(path),
            DocumentSource::Url(url) => detect_format(url),
            DocumentSource::Bytes(_) => "unknown",
            DocumentSource::Text(_) => "markdown",
        };

        let parser = self
            .find_parser(format)
            .ok_or_else(|| DocumentError::BackendUnavailable("No parser registered".to_string()))?;

        parser.parse(source)
    }

    /// Parse with a specific format hint.
    pub fn parse_with_hint(
        &self,
        source: &DocumentSource,
        hint: &str,
    ) -> Result<ParsedDocument, DocumentError> {
        let parser = self.find_parser(hint).ok_or_else(|| {
            DocumentError::BackendUnavailable(format!("No parser available for format '{}'", hint))
        })?;
        parser.parse(source)
    }

    /// List all registered backends.
    pub fn list_backends(&self) -> Vec<BackendInfo> {
        self.parsers
            .iter()
            .map(|p| BackendInfo {
                name: p.backend_name().to_string(),
                formats: p
                    .supported_formats()
                    .iter()
                    .map(|f| f.to_string())
                    .collect(),
            })
            .collect()
    }

    /// Number of registered parsers.
    pub fn len(&self) -> usize {
        self.parsers.len()
    }

    /// Returns true if no parsers are registered.
    pub fn is_empty(&self) -> bool {
        self.parsers.is_empty()
    }

    /// Register a VLM backend from explicit parameters.
    /// Returns true if registration succeeded.
    pub fn try_register_vlm(
        &mut self,
        backend_type: &str,
        base_url: &str,
        api_key: Option<&str>,
    ) -> bool {
        match backend_type.to_lowercase().as_str() {
            "docling" => {
                let mut backend = super::vlm_backend::VlmBackend::docling(base_url);
                if let Some(key) = api_key {
                    backend = backend.with_api_key(key);
                }
                self.register(Box::new(backend));
                true
            }
            "dolphin" => {
                let mut backend = super::vlm_backend::VlmBackend::dolphin(base_url);
                if let Some(key) = api_key {
                    backend = backend.with_api_key(key);
                }
                self.register(Box::new(backend));
                true
            }
            "nemotron" => {
                if let Some(key) = api_key {
                    let backend =
                        super::vlm_backend::VlmBackend::nemotron(base_url, key.to_string());
                    self.register(Box::new(backend));
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Try to register a VLM backend from environment variables.
    /// Reads `VLM_BACKEND_TYPE`, `VLM_BACKEND_URL`, `VLM_API_KEY`.
    /// Returns true if a backend was successfully registered.
    pub fn try_register_vlm_from_env(&mut self) -> bool {
        let backend_type = match std::env::var("VLM_BACKEND_TYPE") {
            Ok(t) => t,
            Err(_) => return false,
        };
        let base_url = match std::env::var("VLM_BACKEND_URL") {
            Ok(u) => u,
            Err(_) => return false,
        };
        let api_key = std::env::var("VLM_API_KEY").ok();
        self.try_register_vlm(&backend_type, &base_url, api_key.as_deref())
    }

    /// Parse a document with classification-aware backend routing.
    ///
    /// 1. Classifies the document (scanned/digital/image/etc.)
    /// 2. For ScannedPdf or Image with confidence > 0.5: prefer VlmBackend (skip PdfExtractorBackend)
    /// 3. For DigitalPdf or others: use format-based routing as usual
    /// 4. Falls back to format-based routing if classification-based routing fails
    pub fn classify_and_parse(
        &self,
        source: &DocumentSource,
        bytes: &[u8],
    ) -> Result<ParsedDocument, DocumentError> {
        // 1. Classify
        let classification = DocumentClassifier::classify_with_analysis(bytes, None);
        let is_scanned = classification.doc_type
            == super::document_classifier::DocumentType::ScannedPdf
            && classification.confidence > 0.5;
        let is_image = classification.doc_type == super::document_classifier::DocumentType::Image
            && classification.confidence > 0.5;

        let parsed = if is_scanned || is_image {
            self.try_parse_with_vlm(source)
                .or_else(|_| self.parse(source))
        } else {
            self.parse(source)
        };

        // Add classification metadata to result
        if let Ok(ref doc) = parsed {
            log::info!(
                "[DOC_CLASSIFY] {} (conf={:.2}) \u{2192} {} backend",
                classification.doc_type,
                classification.confidence,
                doc.metadata.backend_name,
            );
        }

        // Post-processing: layout analysis (read-order correction + region detection)
        let parsed = parsed.map(|mut doc| {
            let analyzer = LayoutAnalyzer::default();
            let analysis = analyzer.analyze_and_log(&doc.markdown, &doc.metadata.backend_name);
            doc.markdown = analysis.corrected_text;
            doc.metadata.layout_type = Some(analysis.layout_type);
            doc.metadata.column_count = Some(analysis.column_count);
            doc
        });

        parsed
    }

    /// Try parsing with any registered VLM backend (skips PdfExtractorBackend).
    fn try_parse_with_vlm(&self, source: &DocumentSource) -> Result<ParsedDocument, DocumentError> {
        let format = match source {
            DocumentSource::File(path) => detect_format(path),
            DocumentSource::Url(url) => detect_format(url),
            DocumentSource::Bytes(_) => "pdf",
            DocumentSource::Text(_) => "markdown",
        };

        for parser in &self.parsers {
            let name = parser.backend_name();
            if name != "PdfExtractorBackend"
                && parser
                    .supported_formats()
                    .iter()
                    .any(|f| f.eq_ignore_ascii_case(format))
            {
                return parser.parse(source);
            }
        }
        Err(DocumentError::BackendUnavailable(
            "No VLM backend available for scanned/image document".to_string(),
        ))
    }
}

impl Default for DocumentParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a default registry with all native backends.
/// Optionally registers a VLM backend from environment variables (no-op if not set).
pub fn create_default_registry() -> DocumentParserRegistry {
    let mut registry = DocumentParserRegistry::new();
    registry.register(Box::new(
        super::pdf_extractor_backend::PdfExtractorBackend::new(),
    ));
    registry.register(Box::new(
        super::xlsx_extractor_backend::XlsxExtractorBackend::new(),
    ));
    registry.register(Box::new(
        super::docx_extractor_backend::DocxExtractorBackend::new(),
    ));
    registry.register(Box::new(
        super::xls_extractor_backend::XlsExtractorBackend::new(),
    ));
    // Optionally register VLM backend from env (no-op if env vars not set)
    registry.try_register_vlm_from_env();
    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_input::pdf_extractor_backend::PdfExtractorBackend;

    #[test]
    fn test_empty_registry() {
        let registry = DocumentParserRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_backend() {
        let mut registry = DocumentParserRegistry::new();
        registry.register(Box::new(PdfExtractorBackend::new()));
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_find_parser() {
        let mut registry = DocumentParserRegistry::new();
        registry.register(Box::new(PdfExtractorBackend::new()));
        let parser = registry.find_parser("pdf");
        assert!(parser.is_some());
        assert_eq!(parser.unwrap().backend_name(), "PdfExtractorBackend");
    }

    #[test]
    fn test_list_backends() {
        let mut registry = DocumentParserRegistry::new();
        registry.register(Box::new(PdfExtractorBackend::new()));
        let backends = registry.list_backends();
        assert_eq!(backends.len(), 1);
        assert_eq!(backends[0].name, "PdfExtractorBackend");
        assert!(backends[0].formats.contains(&"pdf".to_string()));
    }

    #[test]
    fn test_default_registry() {
        let registry = create_default_registry();
        // PdfExtractorBackend + XlsxExtractorBackend + DocxExtractorBackend + XlsExtractorBackend
        assert_eq!(registry.len(), 4);
    }

    #[test]
    fn test_try_register_vlm_docling() {
        let mut registry = DocumentParserRegistry::new();
        let ok = registry.try_register_vlm("docling", "http://localhost:5000", None);
        assert!(ok);
        let backends = registry.list_backends();
        assert_eq!(backends.len(), 1);
        assert_eq!(backends[0].name, "DoclingVlm");
    }

    #[test]
    fn test_try_register_vlm_nemotron_no_key() {
        let mut registry = DocumentParserRegistry::new();
        let ok = registry.try_register_vlm("nemotron", "http://localhost:8000", None);
        assert!(!ok);
        assert!(registry.is_empty());
    }
}
