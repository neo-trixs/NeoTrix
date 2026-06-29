use super::super::sense_modality::SenseModality;
use super::{ParseError, ParsedContent, SourceParser};
use crate::core::nt_core_source_cognition::type_detector::DetectedType;
use std::collections::HashMap;

/// Fallback parser for unknown binary data — produces a "scent" signature.
///
/// Even when we cannot extract meaning, consciousness can still sense
/// the presence and statistical properties of the data (鼻/Olfactory modality).
/// This feeds the curiosity system: "I sense something I don't understand."
pub struct BinaryParser;

impl SourceParser for BinaryParser {
    fn name(&self) -> &'static str {
        "binary"
    }

    fn modality(&self) -> SenseModality {
        SenseModality::Olfactory
    }

    fn can_handle(&self, _detected: &DetectedType) -> bool {
        true // universal fallback — always last resort
    }

    fn parse(&self, data: &[u8]) -> Result<ParsedContent, ParseError> {
        let entropy = shannon_entropy(data);
        let hex_sig: String = data.iter().take(64).map(|b| format!("{b:02x}")).collect();
        let printable_count = data
            .iter()
            .filter(|&&b| b.is_ascii_graphic() || b == b' ' || b == b'\n')
            .count();
        let printable_pct = if data.is_empty() {
            0.0
        } else {
            printable_count as f64 / data.len() as f64
        };

        let text = format!(
            "Binary data: {} bytes, entropy={entropy:.4}, printable={printable_pct:.1}%, sig={hex_sig}",
            data.len(),
        );

        Ok(ParsedContent {
            text,
            modality: SenseModality::Olfactory,
            confidence: 0.2,
            metadata: HashMap::new(),
        })
    }

    fn priority(&self) -> u8 {
        200 // lowest — only used when nothing else matches
    }
}

fn shannon_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut freq = [0usize; 256];
    for &b in data {
        freq[b as usize] += 1;
    }
    let len = data.len() as f64;
    freq.iter().filter(|&&c| c > 0).fold(0.0, |acc, &c| {
        let p = c as f64 / len;
        acc - p * p.log2()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_parser_always_handles() {
        let p = BinaryParser;
        let dt = DetectedType::unknown();
        assert!(p.can_handle(&dt));
    }

    #[test]
    fn test_binary_parser_produces_scent() {
        let p = BinaryParser;
        let result = p.parse(b"\x00\x01\x02\xDE\xAD\xBE\xEF").unwrap();
        assert_eq!(result.modality, SenseModality::Olfactory);
        assert!(result.confidence < 0.5);
        assert!(result.text.contains("Binary data"));
        assert!(result.text.contains("entropy="));
    }

    #[test]
    fn test_binary_parser_empty() {
        let p = BinaryParser;
        let result = p.parse(b"").unwrap();
        assert!(result.text.contains("0 bytes"));
    }

    #[test]
    fn test_shannon_entropy_uniform() {
        let e = shannon_entropy(&[0xAA; 100]);
        assert!(e < 0.01); // uniform data → near-zero entropy
    }

    #[test]
    fn test_shannon_entropy_diverse() {
        let e = shannon_entropy(&[0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]);
        assert!(e > 2.5); // diverse data → higher entropy
    }
}
