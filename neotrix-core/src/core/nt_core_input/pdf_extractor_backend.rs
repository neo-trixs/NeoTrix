#![allow(dead_code)]

use super::document_classifier::DocumentClassifier;
use super::document_parser::{
    source_to_bytes, DocumentError, DocumentMetadata, DocumentParser, DocumentSource,
    ParsedDocument,
};
use super::pdf_extractor::PdfExtractor;

pub struct PdfExtractorBackend {
    max_pages: usize,
}

impl PdfExtractorBackend {
    pub fn new() -> Self {
        Self { max_pages: 100 }
    }

    pub fn with_max_pages(mut self, n: usize) -> Self {
        self.max_pages = n;
        self
    }
}

impl DocumentParser for PdfExtractorBackend {
    fn parse(&self, source: &DocumentSource) -> Result<ParsedDocument, DocumentError> {
        let bytes = source_to_bytes(source)?;
        let extractor = PdfExtractor::new().with_max_pages(self.max_pages);
        let pdf = extractor
            .extract(&bytes)
            .map_err(|e| DocumentError::ParseError(format!("PdfExtractor failed: {:?}", e)))?;

        let text: String = pdf
            .pages
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        let page_count = pdf.page_count();

        let language = if !text.is_empty() {
            DocumentClassifier::detect_language(&text)
        } else {
            None
        };

        Ok(ParsedDocument {
            markdown: text,
            tables: vec![],
            images: vec![],
            metadata: DocumentMetadata {
                page_count: Some(page_count),
                format: Some("pdf".to_string()),
                size_bytes: Some(bytes.len()),
                language,
                backend_name: "PdfExtractorBackend".to_string(),
                ..Default::default()
            },
        })
    }

    fn supported_formats(&self) -> Vec<&'static str> {
        vec!["pdf"]
    }

    fn backend_name(&self) -> &'static str {
        "PdfExtractorBackend"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_name() {
        let backend = PdfExtractorBackend::new();
        assert_eq!(backend.backend_name(), "PdfExtractorBackend");
    }

    #[test]
    fn test_supported_formats() {
        let backend = PdfExtractorBackend::new();
        assert!(backend.supported_formats().contains(&"pdf"));
    }

    #[test]
    fn test_with_max_pages() {
        let backend = PdfExtractorBackend::new().with_max_pages(50);
        assert!(backend.supported_formats().contains(&"pdf"));
    }
}
