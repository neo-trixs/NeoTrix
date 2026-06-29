use super::super::sense_modality::SenseModality;
use super::{ParseError, ParsedContent, SourceParser};
use crate::core::nt_core_source_cognition::type_detector::DetectedType;

/// Parser for plain text, code, markup — anything under the Mental modality.
pub struct TextParser;

impl SourceParser for TextParser {
    fn name(&self) -> &'static str {
        "text"
    }

    fn modality(&self) -> SenseModality {
        SenseModality::Mental
    }

    fn can_handle(&self, detected: &DetectedType) -> bool {
        detected.modality == SenseModality::Mental || detected.mime.starts_with("text/")
    }

    fn parse(&self, data: &[u8]) -> Result<ParsedContent, ParseError> {
        let text =
            std::str::from_utf8(data).map_err(|e| ParseError(format!("invalid utf-8: {e}")))?;
        Ok(ParsedContent {
            text: text.to_string(),
            modality: SenseModality::Mental,
            confidence: 1.0,
            metadata: [("charset".into(), "utf-8".into())].into(),
        })
    }

    fn priority(&self) -> u8 {
        50
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_parser_utf8() {
        let p = TextParser;
        let result = p.parse(b"hello world").unwrap();
        assert_eq!(result.text, "hello world");
        assert_eq!(result.modality, SenseModality::Mental);
    }

    #[test]
    fn test_text_parser_non_utf8_fails() {
        let p = TextParser;
        let result = p.parse(b"\xff\xfe\x00\x01");
        assert!(result.is_err());
    }

    #[test]
    fn test_can_handle_mental_modality() {
        let p = TextParser;
        let dt = DetectedType::text_plain();
        assert!(p.can_handle(&dt));
        let dt2 = DetectedType::pdf();
        assert!(!p.can_handle(&dt2));
    }
}
