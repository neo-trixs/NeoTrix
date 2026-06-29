use std::path::Path;

use crate::core::nt_core_consciousness::{SenseModality, VsaOrigin, VsaTagged, VsaWorldCategory};

use super::parser_router::ParserRouter;
use super::parsers::{binary_parser::BinaryParser, pdf_parser::PdfParser, text_parser::TextParser};
use super::sense_encoder::SenseEncoder;
use super::type_detector::TypeDetector;

/// SourceCognitionEngine — 源认知层: 感知-解析-编码为意识可处理的VSA向量
///
/// 模拟生物的眼耳鼻舌身意六识，但用字节类型检测、SourceParser注册表、
/// 和VSA向量编码来实现。每个输出都携带着SenseModality标签，
/// 让意识层知道"我正在通过什么感官在感知"。
pub struct SourceCognitionEngine {
    detector: TypeDetector,
    router: ParserRouter,
    encoder: SenseEncoder,
    stats: SourceCognitionStats,
}

/// Statistics for source cognition processing.
#[derive(Debug, Clone, Default)]
pub struct SourceCognitionStats {
    pub total_items: u64,
    pub visual_items: u64,
    pub auditory_items: u64,
    pub olfactory_items: u64,
    pub gustatory_items: u64,
    pub tactile_items: u64,
    pub proprioceptive_items: u64,
    pub vestibular_items: u64,
    pub interoceptive_items: u64,
    pub mental_items: u64,
    pub parse_errors: u64,
    pub total_bytes: u64,
}

impl SourceCognitionStats {
    fn record(&mut self, modality: SenseModality, bytes: u64) {
        self.total_items += 1;
        self.total_bytes += bytes;
        match modality {
            SenseModality::Visual => self.visual_items += 1,
            SenseModality::Auditory => self.auditory_items += 1,
            SenseModality::Olfactory => self.olfactory_items += 1,
            SenseModality::Gustatory => self.gustatory_items += 1,
            SenseModality::Tactile => self.tactile_items += 1,
            SenseModality::Proprioceptive => self.proprioceptive_items += 1,
            SenseModality::Vestibular => self.vestibular_items += 1,
            SenseModality::Interoceptive => self.interoceptive_items += 1,
            SenseModality::Mental => self.mental_items += 1,
            SenseModality::Document => self.visual_items += 1,
        }
    }
}

impl SourceCognitionEngine {
    pub fn new() -> Self {
        let mut router = ParserRouter::new();
        router.register(Box::new(PdfParser::new()));
        router.register(Box::new(TextParser));
        router.register(Box::new(BinaryParser));

        Self {
            detector: TypeDetector::new(),
            router,
            encoder: SenseEncoder::new(),
            stats: SourceCognitionStats::default(),
        }
    }

    /// Process raw bytes with optional filename hint.
    /// Returns a VSA-tagged vector ready for consciousness ingestion.
    ///
    /// Detection → Routing → Parsing → VSA Encoding → VsaTagged
    pub fn process(&mut self, data: &[u8], filename: Option<&str>) -> Result<VsaTagged, String> {
        let detected = self.detector.detect(data, filename);
        let content = self
            .router
            .parse(data, &detected)
            .map_err(|e| e.to_string())?;
        let (vec, modality, confidence) = self.encoder.encode(&content);

        self.stats.record(modality, data.len() as u64);

        let tag = if filename.is_some() {
            VsaOrigin::World(VsaWorldCategory::FileContent)
        } else {
            VsaOrigin::World(VsaWorldCategory::Sensor)
        };

        Ok(VsaTagged::new(vec, tag)
            .with_confidence(confidence)
            .with_sense_modality(modality))
    }

    /// Process a file from disk — reads bytes, detects, parses, encodes.
    pub fn process_file(&mut self, path: &Path) -> Result<VsaTagged, String> {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.to_string());
        let data = std::fs::read(path).map_err(|e| format!("read error: {e}"))?;
        let result = self.process(&data, filename.as_deref())?;
        Ok(result)
    }

    /// Batch process multiple byte slices.
    pub fn process_batch(
        &mut self,
        items: &[(&[u8], Option<&str>)],
    ) -> Vec<Result<VsaTagged, String>> {
        items
            .iter()
            .map(|(data, name)| self.process(data, *name))
            .collect()
    }

    /// Access the parser router for dynamic parser registration.
    pub fn router_mut(&mut self) -> &mut ParserRouter {
        &mut self.router
    }

    /// Access stats for monitoring.
    pub fn stats(&self) -> &SourceCognitionStats {
        &self.stats
    }

    /// Reset all statistics.
    pub fn reset_stats(&mut self) {
        self.stats = SourceCognitionStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_text() {
        let mut engine = SourceCognitionEngine::new();
        let result = engine.process(b"hello world", Some("test.txt")).unwrap();
        assert!(result.is_world());
        assert_eq!(result.sense_modality, Some(SenseModality::Mental));
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_process_pdf_non_data_yields_error() {
        let mut engine = SourceCognitionEngine::new();
        let result = engine.process(b"not a pdf", Some("doc.pdf"));
        assert!(result.is_err());
    }

    #[test]
    fn test_process_unknown_binary() {
        let mut engine = SourceCognitionEngine::new();
        let result = engine.process(&[0x00, 0xDE, 0xAD], None).unwrap();
        assert_eq!(result.sense_modality, Some(SenseModality::Olfactory));
        assert!(result.confidence < 0.5);
    }

    #[test]
    fn test_stats_after_processing() {
        let mut engine = SourceCognitionEngine::new();
        engine.process(b"hello", None).ok();
        engine.process(b"world", None).ok();
        let stats = engine.stats();
        assert_eq!(stats.total_items, 2);
        assert_eq!(stats.mental_items, 2);
    }

    #[test]
    fn test_process_batch() {
        let mut engine = SourceCognitionEngine::new();
        let items = &[
            (b"hello" as &[u8], Some("a.txt")),
            (b"world" as &[u8], Some("b.txt")),
        ];
        let results = engine.process_batch(items);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_process_file_nonexistent() {
        let mut engine = SourceCognitionEngine::new();
        let result = engine.process_file(Path::new("/nonexistent/file.pdf"));
        assert!(result.is_err());
    }

    #[test]
    fn test_reset_stats() {
        let mut engine = SourceCognitionEngine::new();
        engine.process(b"test", None).ok();
        assert_eq!(engine.stats().total_items, 1);
        engine.reset_stats();
        assert_eq!(engine.stats().total_items, 0);
    }
}
