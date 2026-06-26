use super::super::sense_modality::SenseModality;
use super::{ParseError, ParsedContent, SourceParser};
use crate::core::nt_core_input::PdfExtractor;
use crate::core::nt_core_source_cognition::type_detector::DetectedType;

/// Parser for PDF documents — wraps PdfExtractor as a SourceParser.
/// Modality: 眼/Visual — documents are perceived visually before their content
/// becomes mental concepts.
pub struct PdfParser {
    extractor: PdfExtractor,
}

impl PdfParser {
    pub fn new() -> Self {
        Self {
            extractor: PdfExtractor::new(),
        }
    }
}

impl SourceParser for PdfParser {
    fn name(&self) -> &'static str {
        "pdf"
    }

    fn modality(&self) -> SenseModality {
        SenseModality::Visual
    }

    fn can_handle(&self, detected: &DetectedType) -> bool {
        detected.mime == "application/pdf"
    }

    fn parse(&self, data: &[u8]) -> Result<ParsedContent, ParseError> {
        let doc = self
            .extractor
            .extract(data)
            .map_err(|e| ParseError(e.to_string()))?;
        let text: String = doc
            .pages
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<&str>>()
            .join("\n");
        let pages_with_text = doc.pages.iter().filter(|p| !p.text.is_empty()).count();
        let confidence = if doc.pages.is_empty() {
            0.5
        } else {
            pages_with_text as f64 / doc.pages.len() as f64
        };
        Ok(ParsedContent {
            text,
            modality: SenseModality::Visual,
            confidence: confidence.max(0.3),
            metadata: [
                ("pages".into(), doc.pages.len().to_string()),
                ("pages_with_text".into(), pages_with_text.to_string()),
            ]
            .into(),
        })
    }

    fn priority(&self) -> u8 {
        10 // high priority for exact type match
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_parser_can_handle() {
        let p = PdfParser::new();
        let dt = DetectedType::pdf();
        assert!(p.can_handle(&dt));
        let dt2 = DetectedType::text_plain();
        assert!(!p.can_handle(&dt2));
    }

    #[test]
    fn test_pdf_parser_invalid_data_fails() {
        let p = PdfParser::new();
        let result = p.parse(b"not a pdf file");
        assert!(result.is_err());
    }

    #[test]
    fn test_pdf_parser_empty_data_fails() {
        let p = PdfParser::new();
        let result = p.parse(b"");
        assert!(result.is_err());
    }

    #[test]
    fn test_pdf_parser_name_and_modality() {
        let p = PdfParser::new();
        assert_eq!(p.name(), "pdf");
        assert_eq!(p.modality(), SenseModality::Visual);
    }
}
