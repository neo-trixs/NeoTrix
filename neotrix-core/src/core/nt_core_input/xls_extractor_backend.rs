#![allow(dead_code)]

use super::document_parser::{
    source_to_bytes, DocumentError, DocumentMetadata, DocumentParser, DocumentSource,
    ParsedDocument,
};
use super::xls_extractor::XlsExtractor;

pub struct XlsExtractorBackend {
    max_texts: usize,
}

impl XlsExtractorBackend {
    pub fn new() -> Self {
        Self { max_texts: 50000 }
    }

    pub fn with_max_texts(mut self, n: usize) -> Self {
        self.max_texts = n;
        self
    }
}

impl DocumentParser for XlsExtractorBackend {
    fn parse(&self, source: &DocumentSource) -> Result<ParsedDocument, DocumentError> {
        let bytes = source_to_bytes(source)?;

        // Quick magic-byte check for OLE2 (D0CF11E0)
        if bytes.len() < 8 || bytes[..8] != [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1] {
            return Err(DocumentError::ParseError(
                "Not a valid OLE2/xls file".to_string(),
            ));
        }

        let extractor = XlsExtractor::new().with_max_texts(self.max_texts);

        let markdown = extractor
            .to_markdown(&bytes)
            .map_err(|e| DocumentError::ParseError(format!("xls extraction failed: {e}")))?;

        Ok(ParsedDocument {
            markdown,
            tables: vec![],
            images: vec![],
            metadata: DocumentMetadata {
                format: Some("xls".to_string()),
                size_bytes: Some(bytes.len()),
                backend_name: "XlsExtractorBackend".to_string(),
                ..Default::default()
            },
        })
    }

    fn supported_formats(&self) -> Vec<&'static str> {
        vec!["xls"]
    }

    fn backend_name(&self) -> &'static str {
        "XlsExtractorBackend"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_name() {
        let backend = XlsExtractorBackend::new();
        assert_eq!(backend.backend_name(), "XlsExtractorBackend");
    }

    #[test]
    fn test_supported_formats() {
        let backend = XlsExtractorBackend::new();
        assert_eq!(backend.supported_formats(), vec!["xls"]);
    }

    #[test]
    fn test_parse_invalid_bytes() {
        let backend = XlsExtractorBackend::new();
        let result = backend.parse(&DocumentSource::Bytes(b"not an xls".to_vec()));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_non_ole2() {
        let backend = XlsExtractorBackend::new();
        let result = backend.parse(&DocumentSource::Bytes(vec![0u8; 512]));
        assert!(result.is_err());
    }
}
