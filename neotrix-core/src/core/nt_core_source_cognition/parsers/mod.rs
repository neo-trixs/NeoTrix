use super::sense_modality::SenseModality;
use std::collections::HashMap;

/// Content extracted by a SourceParser — normalized text + metadata.
#[derive(Debug, Clone)]
pub struct ParsedContent {
    pub text: String,
    pub modality: SenseModality,
    pub confidence: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error: {}", self.0)
    }
}

impl std::error::Error for ParseError {}

/// SourceParser trait — every sensory parser implements this.
///
/// Stateless or owning-state; thread-safe via Send + Sync.
/// The parser registry (ParserRouter) stores Box<dyn SourceParser>.
pub trait SourceParser: Send + Sync {
    /// Human-readable parser name
    fn name(&self) -> &'static str;
    /// Primary sensory modality this parser produces
    fn modality(&self) -> SenseModality;
    /// Whether this parser can handle a given detected type
    fn can_handle(&self, detected: &super::type_detector::DetectedType) -> bool;
    /// Parse raw bytes into structured content
    fn parse(&self, data: &[u8]) -> Result<ParsedContent, ParseError>;
    /// Priority: lower = higher priority (default 100)
    fn priority(&self) -> u8 {
        100
    }
}

pub mod binary_parser;
pub mod pdf_parser;
pub mod text_parser;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_source_cognition::type_detector::DetectedType;

    struct TestParser;

    impl SourceParser for TestParser {
        fn name(&self) -> &'static str {
            "test"
        }
        fn modality(&self) -> SenseModality {
            SenseModality::Mental
        }
        fn can_handle(&self, detected: &DetectedType) -> bool {
            detected.modality == SenseModality::Mental
        }
        fn parse(&self, data: &[u8]) -> Result<ParsedContent, ParseError> {
            Ok(ParsedContent {
                text: String::from_utf8_lossy(data).to_string(),
                modality: SenseModality::Mental,
                confidence: 1.0,
                metadata: HashMap::new(),
            })
        }
    }

    #[test]
    fn test_parser_trait() {
        let p = TestParser;
        assert_eq!(p.name(), "test");
        assert_eq!(p.modality(), SenseModality::Mental);
        let dt = DetectedType::text_plain();
        assert!(p.can_handle(&dt));
        let result = p.parse(b"hello world").unwrap();
        assert_eq!(result.text, "hello world");
    }
}
