use super::parser::*;
use super::types::*;
use crate::core::nt_core_hcube::cross_modal::CrossModalAligner;
use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;
use std::collections::HashMap;
use std::path::Path;

/// Bundle multiple binary VSA vectors into one using majority threshold.
/// For each bit position, if sum of 1s across vectors > n/2, output is 1, else 0.
pub fn bundle_sections(vectors: &[Vec<u8>]) -> Vec<u8> {
    if vectors.is_empty() {
        return vec![0; VSA_DIM];
    }

    let dim = vectors[0].len();
    let n = vectors.len();
    let threshold = n as u32 / 2;

    let mut result = Vec::with_capacity(dim);
    for i in 0..dim {
        let sum: u32 = vectors
            .iter()
            .map(|v| v.get(i).copied().unwrap_or(0) as u32)
            .sum();
        result.push(if sum > threshold { 1 } else { 0 });
    }
    result
}

/// Compute Hamming distance between two binary VSA vectors.
pub fn hamming_distance(a: &[u8], b: &[u8]) -> u32 {
    let len = a.len().min(b.len());
    let mut dist = 0u32;
    for i in 0..len {
        if a[i] != b[i] {
            dist += 1;
        }
    }
    dist
}

/// Compute Hamming similarity as 1.0 - normalized Hamming distance.
pub fn hamming_similarity(a: &[u8], b: &[u8]) -> f64 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 0.0;
    }
    1.0 - (hamming_distance(a, b) as f64 / len as f64)
}

/// Main document parsing engine that holds registered parsers.
pub struct DocumentParsingEngine {
    parsers: Vec<Box<dyn DocumentParser>>,
}

impl DocumentParsingEngine {
    pub fn new() -> Self {
        Self {
            parsers: Vec::new(),
        }
    }

    pub fn register_parser(&mut self, parser: Box<dyn DocumentParser>) {
        self.parsers.push(parser);
    }

    /// Parse a file by detecting its format and finding a suitable parser.
    pub fn parse_file(&self, path: &Path) -> Result<ParsedDocument, String> {
        let fmt = DocumentFormat::from_extension(path)
            .ok_or_else(|| format!("unsupported file extension: {}", path.display()))?;

        for parser in &self.parsers {
            if parser.can_parse(path, &fmt) {
                return parser.parse(path);
            }
        }

        Err(format!("no parser registered for format {:?}", fmt))
    }

    /// Parse text directly with a given format.
    pub fn parse_text(&self, text: &str, format: DocumentFormat) -> ParsedDocument {
        for parser in &self.parsers {
            if parser.supported_formats().contains(&format) {
                return parser.parse_text(text, format);
            }
        }

        let text = text.to_string();
        let title = text
            .lines()
            .next()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty());
        let section = Section {
            heading: None,
            level: 0,
            content: text.clone(),
            bounding_box: None,
            subsections: vec![],
        };
        let vsa = text_to_vsa(&text);
        let document = Document {
            format,
            title,
            sections: vec![section],
            metadata: HashMap::new(),
            raw_text: text,
        };
        let words = document.raw_text.split_whitespace().count();
        ParsedDocument {
            document,
            vsa_vectors: vec![vsa.clone()],
            combined_vector: vsa,
            section_count: 1,
            estimated_reading_time: if words > 0 {
                (words as f64 / 200.0) * 60.0
            } else {
                0.0
            },
        }
    }

    /// Detect the format from a file path extension.
    pub fn detect_format(path: &Path) -> Option<DocumentFormat> {
        DocumentFormat::from_extension(path)
    }

    /// Convert a section's content into a VSA vector.
    pub fn section_to_vsa(section: &Section) -> Vec<u8> {
        text_to_vsa(&section.content)
    }
}

impl Default for DocumentParsingEngine {
    fn default() -> Self {
        let mut engine = Self::new();
        engine.register_parser(Box::new(PlainTextParser::new()));
        engine.register_parser(Box::new(MarkdownParser::new()));
        engine.register_parser(Box::new(HtmlParser::new()));
        engine
    }
}

pub fn text_to_vsa(text: &str) -> Vec<u8> {
    let aligner = CrossModalAligner::new(VSA_DIM, 42);
    aligner.text_to_vsa(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_bundle_sections_empty() {
        let result = bundle_sections(&[]);
        assert_eq!(result.len(), VSA_DIM);
        assert!(result.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_bundle_sections_majority() {
        let all_zeros = vec![0u8; VSA_DIM];
        let all_ones = vec![1u8; VSA_DIM];
        let vectors = vec![all_zeros.clone(), all_ones.clone(), all_ones.clone()];
        let result = bundle_sections(&vectors);
        assert_eq!(result.len(), VSA_DIM);
        assert!(result.iter().all(|&b| b == 1));
    }

    #[test]
    fn test_hamming_distance() {
        let a = vec![0u8, 0, 1, 1, 0];
        let b = vec![0u8, 1, 0, 1, 0];
        assert_eq!(hamming_distance(&a, &b), 2);
    }

    #[test]
    fn test_hamming_similarity() {
        let a = vec![0u8, 0, 1, 1, 0];
        let b = vec![0u8, 0, 1, 1, 0];
        assert!((hamming_similarity(&a, &b) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_engine_parse_text_plain() {
        let engine = DocumentParsingEngine::default();
        let result = engine.parse_text("Hello\n\nWorld\n\nTest.", DocumentFormat::PlainText);
        assert!(result.section_count >= 3);
        assert_eq!(result.combined_vector.len(), VSA_DIM);
    }

    #[test]
    fn test_engine_parse_text_markdown() {
        let engine = DocumentParsingEngine::default();
        let result = engine.parse_text("# Title\n\nContent.", DocumentFormat::Markdown);
        assert_eq!(result.document.title.as_deref(), Some("Title"));
        assert_eq!(result.combined_vector.len(), VSA_DIM);
    }

    #[test]
    fn test_engine_parse_text_html() {
        let engine = DocumentParsingEngine::default();
        let result = engine.parse_text("<h1>Title</h1><p>Content.</p>", DocumentFormat::Html);
        assert_eq!(result.document.title.as_deref(), Some("Title"));
    }

    #[test]
    fn test_engine_detect_format() {
        assert_eq!(
            DocumentParsingEngine::detect_format(Path::new("doc.md")),
            Some(DocumentFormat::Markdown)
        );
        assert_eq!(
            DocumentParsingEngine::detect_format(Path::new("page.html")),
            Some(DocumentFormat::Html)
        );
        assert_eq!(
            DocumentParsingEngine::detect_format(Path::new("unknown.xyz")),
            None
        );
    }

    #[test]
    fn test_engine_unknown_format() {
        let engine = DocumentParsingEngine::new();
        let result = engine.parse_text("some text", DocumentFormat::Pdf);
        assert_eq!(result.section_count, 1);
        assert_eq!(result.document.format, DocumentFormat::Pdf);
    }

    #[test]
    fn test_section_to_vsa_non_empty() {
        let section = Section {
            heading: Some("Test".into()),
            level: 1,
            content: "Hello world this is test content.".into(),
            bounding_box: None,
            subsections: vec![],
        };
        let vsa = DocumentParsingEngine::section_to_vsa(&section);
        assert_eq!(vsa.len(), VSA_DIM);
    }

    #[test]
    fn test_engine_default_registers_parsers() {
        let engine = DocumentParsingEngine::default();
        assert!(!engine.parsers.is_empty());
    }
}
