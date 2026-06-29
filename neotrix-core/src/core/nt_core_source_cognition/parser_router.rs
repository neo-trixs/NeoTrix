use super::parsers::{ParseError, ParsedContent, SourceParser};
use super::type_detector::DetectedType;

/// Registry of SourceParsers with priority-based routing.
///
/// Routing strategy:
/// 1. Exact MIME match — parser explicitly registered for this MIME type
/// 2. Modality match — any parser that shares the detected sensory modality
/// 3. Universal fallback — BinaryParser (always catches)
pub struct ParserRouter {
    parsers: Vec<Box<dyn SourceParser>>,
}

impl ParserRouter {
    pub fn new() -> Self {
        Self {
            parsers: Vec::new(),
        }
    }

    pub fn register(&mut self, parser: Box<dyn SourceParser>) {
        self.parsers.push(parser);
    }

    pub fn parsers(&self) -> &[Box<dyn SourceParser>] {
        &self.parsers
    }

    /// Find the best parser for a detected type.
    /// Returns None if no parser matches at all (should not happen with BinaryParser registered).
    pub fn route(&self, detected: &DetectedType) -> Option<&dyn SourceParser> {
        let mut best: Option<(&dyn SourceParser, u8)> = None;

        for p in &self.parsers {
            if !p.can_handle(detected) {
                continue;
            }
            let priority = p.priority();
            let is_better = match best {
                None => true,
                Some((_, best_prio)) => {
                    // Higher priority (lower number) wins
                    priority < best_prio
                }
            };
            if is_better {
                best = Some((p.as_ref(), priority));
            }
        }

        best.map(|(p, _)| p)
    }

    /// Route and parse in one step.
    pub fn parse(&self, data: &[u8], detected: &DetectedType) -> Result<ParsedContent, ParseError> {
        match self.route(detected) {
            Some(parser) => parser.parse(data),
            None => Err(ParseError("no parser available for this data".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_source_cognition::parsers::binary_parser::BinaryParser;
    use crate::core::nt_core_source_cognition::parsers::pdf_parser::PdfParser;
    use crate::core::nt_core_source_cognition::parsers::text_parser::TextParser;

    #[test]
    fn test_router_empty() {
        let router = ParserRouter::new();
        let dt = DetectedType::text_plain();
        assert!(router.route(&dt).is_none());
    }

    #[test]
    fn test_router_selects_best_priority() {
        let mut router = ParserRouter::new();
        router.register(Box::new(TextParser));
        router.register(Box::new(BinaryParser));

        let dt = DetectedType::text_plain();
        let parser = router.route(&dt).unwrap();
        assert_eq!(parser.name(), "text");
    }

    #[test]
    fn test_router_pdf_selects_pdf_parser() {
        let mut router = ParserRouter::new();
        router.register(Box::new(BinaryParser));
        router.register(Box::new(TextParser));
        router.register(Box::new(PdfParser::new()));

        let dt = DetectedType::pdf();
        let parser = router.route(&dt).unwrap();
        assert_eq!(parser.name(), "pdf");
    }

    #[test]
    fn test_router_unknown_falls_to_binary() {
        let mut router = ParserRouter::new();
        router.register(Box::new(BinaryParser));
        router.register(Box::new(TextParser));

        let dt = DetectedType::unknown();
        let parser = router.route(&dt).unwrap();
        assert_eq!(parser.name(), "binary");
    }

    #[test]
    fn test_router_parse_text() {
        let mut router = ParserRouter::new();
        router.register(Box::new(TextParser));
        router.register(Box::new(BinaryParser));

        let dt = DetectedType::text_plain();
        let result = router.parse(b"hello", &dt).unwrap();
        assert_eq!(result.text, "hello");
    }
}
