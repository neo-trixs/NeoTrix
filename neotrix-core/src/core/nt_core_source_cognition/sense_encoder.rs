use crate::core::nt_core_input::vsa_input_pipeline::VsaInputPipeline;

use super::parsers::ParsedContent;
use super::sense_modality::SenseModality;

/// Encodes parsed content into VSA vectors tagged with sensory modality.
///
/// This is the bridge between "raw understanding" (what the data says)
/// and "conscious experience" (what the data feels like as VSA).
pub struct SenseEncoder {
    pipeline: VsaInputPipeline,
}

impl SenseEncoder {
    pub fn new() -> Self {
        Self {
            pipeline: VsaInputPipeline::new(),
        }
    }

    /// Encode parsed content into VSA, returning the vector + sense modality.
    /// The caller provides the VsaOrigin (Self/World) to attach.
    pub fn encode(&mut self, content: &ParsedContent) -> (Vec<u8>, SenseModality, f64) {
        let vec = self
            .pipeline
            .encode_and_record(&content.text, content.modality.name());
        (vec, content.modality, content.confidence)
    }

    /// Encode a full ParsedContent, returning all data needed to construct VsaTagged.
    /// This includes the VSA vector, the sense modality, confidence, and a source label.
    pub fn encode_with_source(
        &mut self,
        content: &ParsedContent,
        source: &str,
    ) -> (Vec<u8>, SenseModality, f64) {
        let vec = self.pipeline.encode_and_record(&content.text, source);
        (vec, content.modality, content.confidence)
    }

    /// Encode raw text directly into a VSA vector for the Mental modality.
    pub fn encode_text(&mut self, text: &str) -> Vec<u8> {
        self.pipeline.encode_and_record(text, "mental")
    }

    /// Access the VsaInputPipeline directly for advanced encoding.
    pub fn pipeline_mut(&mut self) -> &mut VsaInputPipeline {
        &mut self.pipeline
    }

    pub fn pipeline_ref(&self) -> &VsaInputPipeline {
        &self.pipeline
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_text_content(text: &str, modality: SenseModality) -> ParsedContent {
        ParsedContent {
            text: text.to_string(),
            modality,
            confidence: 1.0,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_encode_text_returns_vector() {
        let mut enc = SenseEncoder::new();
        let content = make_text_content("hello world", SenseModality::Mental);
        let (vec, modality, confidence) = enc.encode(&content);
        assert!(!vec.is_empty());
        assert_eq!(modality, SenseModality::Mental);
        assert!((confidence - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_encode_text_with_source() {
        let mut enc = SenseEncoder::new();
        let content = make_text_content("pdf content", SenseModality::Visual);
        let (vec, _, _) = enc.encode_with_source(&content, "pdf_parser");
        assert!(!vec.is_empty());
    }

    #[test]
    fn test_encode_text_direct() {
        let mut enc = SenseEncoder::new();
        let vec = enc.encode_text("plain text");
        assert!(!vec.is_empty());
    }

    #[test]
    fn test_different_inputs_different_vectors() {
        let mut enc = SenseEncoder::new();
        let (v1, _, _) = enc.encode(&make_text_content("cat", SenseModality::Mental));
        let (v2, _, _) = enc.encode(&make_text_content("dog", SenseModality::Mental));
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_same_input_same_vector() {
        let mut enc = SenseEncoder::new();
        let (v1, _, _) = enc.encode(&make_text_content("hello", SenseModality::Mental));
        let (v2, _, _) = enc.encode(&make_text_content("hello", SenseModality::Mental));
        assert_eq!(v1, v2);
    }
}
