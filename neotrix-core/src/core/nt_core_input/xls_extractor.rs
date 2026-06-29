#![allow(dead_code)]

use super::office_common::{extract_text_from_biff, Ole2Reader};

/// xls (OLE2/BIFF) extraction engine.
/// Reads raw .xls bytes and extracts text content via:
/// 1. OLE2 compound document parsing
/// 2. BIFF record traversal over the Workbook stream
/// 3. Cell text extraction (shared strings, labels, formulas)
#[derive(Debug, Clone)]
pub struct XlsExtractor {
    max_texts: usize,
}

impl Default for XlsExtractor {
    fn default() -> Self {
        Self { max_texts: 50000 }
    }
}

impl XlsExtractor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_texts(mut self, n: usize) -> Self {
        self.max_texts = n;
        self
    }

    /// Extract text content from an .xls file.
    /// Returns a list of text values found in cells.
    pub fn extract(&self, data: &[u8]) -> Result<Vec<String>, String> {
        let reader = Ole2Reader::new(data)?;
        let workbook_stream = reader.read_workbook()?;
        let texts = extract_text_from_biff(&workbook_stream);
        let mut result = texts;
        result.truncate(self.max_texts);
        Ok(result)
    }

    /// Extract text as markdown (one line per cell text).
    pub fn to_markdown(&self, data: &[u8]) -> Result<String, String> {
        let texts = self.extract(data)?;
        let mut md = String::new();
        for t in &texts {
            md.push_str(t);
            md.push('\n');
        }
        Ok(md)
    }

    /// Get metadata from the OLE2 compound document.
    pub fn metadata(&self, data: &[u8]) -> Result<XlsMetadata, String> {
        let reader = Ole2Reader::new(data)?;
        let streams = reader.stream_names();
        let text_count = self.extract(data).map(|t| t.len()).unwrap_or(0);
        Ok(XlsMetadata {
            stream_count: streams.len(),
            stream_names: streams,
            text_count,
            directory_entry_count: reader.dir_entries.len(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct XlsMetadata {
    pub stream_count: usize,
    pub stream_names: Vec<String>,
    pub text_count: usize,
    pub directory_entry_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extractor_defaults() {
        let ex = XlsExtractor::new();
        assert_eq!(ex.max_texts, 50000);
    }

    #[test]
    fn test_invalid_xls() {
        let ex = XlsExtractor::new();
        let result = ex.extract(b"not an ole2 file");
        assert!(result.is_err());
    }

    #[test]
    fn test_non_ole2_magic() {
        let ex = XlsExtractor::new();
        let result = ex.extract(&[0u8; 512]);
        assert!(result.is_err());
    }
}
