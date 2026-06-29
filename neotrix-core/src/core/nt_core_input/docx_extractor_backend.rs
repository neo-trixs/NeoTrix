#![allow(dead_code)]

use super::document_parser::{
    source_to_bytes, DocumentError, DocumentMetadata, DocumentParser, DocumentSource,
    ParsedDocument,
};
use super::docx_extractor::DocxExtractor;

pub struct DocxExtractorBackend {
    max_paragraphs: usize,
    include_headers: bool,
}

impl DocxExtractorBackend {
    pub fn new() -> Self {
        Self {
            max_paragraphs: 10000,
            include_headers: false,
        }
    }

    pub fn with_max_paragraphs(mut self, n: usize) -> Self {
        self.max_paragraphs = n;
        self
    }

    pub fn with_headers(mut self, yes: bool) -> Self {
        self.include_headers = yes;
        self
    }
}

impl DocumentParser for DocxExtractorBackend {
    fn parse(&self, source: &DocumentSource) -> Result<ParsedDocument, DocumentError> {
        let bytes = source_to_bytes(source)?;

        // Quick magic-byte check for ZIP (PK\x03\x04)
        if bytes.len() < 4 || bytes[..4] != [0x50, 0x4b, 0x03, 0x04] {
            return Err(DocumentError::ParseError(
                "Not a valid ZIP/docx file".to_string(),
            ));
        }

        let extractor = DocxExtractor::new()
            .with_max_paragraphs(self.max_paragraphs)
            .with_headers(self.include_headers);

        let markdown = extractor
            .to_markdown(&bytes)
            .map_err(|e| DocumentError::ParseError(format!("docx extraction failed: {e}")))?;

        Ok(ParsedDocument {
            markdown,
            tables: vec![],
            images: vec![],
            metadata: DocumentMetadata {
                format: Some("docx".to_string()),
                size_bytes: Some(bytes.len()),
                backend_name: "DocxExtractorBackend".to_string(),
                ..Default::default()
            },
        })
    }

    fn supported_formats(&self) -> Vec<&'static str> {
        vec!["docx"]
    }

    fn backend_name(&self) -> &'static str {
        "DocxExtractorBackend"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_name() {
        let backend = DocxExtractorBackend::new();
        assert_eq!(backend.backend_name(), "DocxExtractorBackend");
    }

    #[test]
    fn test_supported_formats() {
        let backend = DocxExtractorBackend::new();
        assert_eq!(backend.supported_formats(), vec!["docx"]);
    }

    #[test]
    fn test_parse_invalid_zip() {
        let backend = DocxExtractorBackend::new();
        let result = backend.parse(&DocumentSource::Bytes(b"not a zip".to_vec()));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DocumentError::ParseError(_)));
    }

    #[test]
    fn test_with_headers() {
        let backend = DocxExtractorBackend::new().with_headers(true);
        assert!(backend.include_headers);
    }
}
