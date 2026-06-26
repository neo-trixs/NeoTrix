#![allow(dead_code)]

use super::document_parser::{
    source_to_bytes, DocumentError, DocumentMetadata, DocumentParser, DocumentSource,
    ParsedDocument,
};
use super::xlsx_extractor::XlsxExtractor;

pub struct XlsxExtractorBackend {
    max_rows: usize,
    max_sheets: usize,
}

impl XlsxExtractorBackend {
    pub fn new() -> Self {
        Self {
            max_rows: 10000,
            max_sheets: 50,
        }
    }

    pub fn with_max_rows(mut self, n: usize) -> Self {
        self.max_rows = n;
        self
    }

    pub fn with_max_sheets(mut self, n: usize) -> Self {
        self.max_sheets = n;
        self
    }
}

impl DocumentParser for XlsxExtractorBackend {
    fn parse(&self, source: &DocumentSource) -> Result<ParsedDocument, DocumentError> {
        let bytes = source_to_bytes(source)?;

        // Quick magic-byte check for ZIP (PK\x03\x04)
        if bytes.len() < 4 || bytes[..4] != [0x50, 0x4b, 0x03, 0x04] {
            return Err(DocumentError::ParseError(
                "Not a valid ZIP/xlsx file".to_string(),
            ));
        }

        let extractor = XlsxExtractor::new()
            .with_max_rows(self.max_rows)
            .with_max_sheets(self.max_sheets);

        let markdown = extractor
            .to_markdown(&bytes)
            .map_err(|e| DocumentError::ParseError(format!("xlsx extraction failed: {e}")))?;

        let sheet_count = extractor.extract(&bytes).map(|s| s.len()).unwrap_or(0);

        Ok(ParsedDocument {
            markdown,
            tables: vec![],
            images: vec![],
            metadata: DocumentMetadata {
                format: Some("xlsx".to_string()),
                size_bytes: Some(bytes.len()),
                backend_name: "XlsxExtractorBackend".to_string(),
                page_count: Some(sheet_count),
                ..Default::default()
            },
        })
    }

    fn supported_formats(&self) -> Vec<&'static str> {
        vec!["xlsx"]
    }

    fn backend_name(&self) -> &'static str {
        "XlsxExtractorBackend"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_name() {
        let backend = XlsxExtractorBackend::new();
        assert_eq!(backend.backend_name(), "XlsxExtractorBackend");
    }

    #[test]
    fn test_supported_formats() {
        let backend = XlsxExtractorBackend::new();
        assert_eq!(backend.supported_formats(), vec!["xlsx"]);
    }

    #[test]
    fn test_invalid_bytes() {
        let backend = XlsxExtractorBackend::new();
        let result = backend.parse(&DocumentSource::Bytes(b"not an xlsx".to_vec()));
        assert!(result.is_err());
    }

    #[test]
    fn test_from_text_source() {
        let backend = XlsxExtractorBackend::new();
        let result = backend.parse(&DocumentSource::Text("hello".to_string()));
        // Text will fail ZIP magic check
        assert!(result.is_err());
    }
}
