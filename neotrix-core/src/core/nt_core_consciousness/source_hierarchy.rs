use serde::{Deserialize, Serialize};

/// Raw sensory/perception input — direct, unprocessed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PerceptionSource {
    UserInput,
    WebFetch,
    ToolOutput,
    Sensor,
    SearchResult,
    FileRead,
}

impl PerceptionSource {
    pub fn name(&self) -> &'static str {
        match self {
            PerceptionSource::UserInput => "user_input",
            PerceptionSource::WebFetch => "web_fetch",
            PerceptionSource::ToolOutput => "tool_output",
            PerceptionSource::Sensor => "sensor",
            PerceptionSource::SearchResult => "search_result",
            PerceptionSource::FileRead => "file_read",
        }
    }
}

/// Metadata for the Raw(Perception) knowledge layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerceptionMeta {
    pub source_type: PerceptionSource,
    pub raw_confidence: f64,
    pub timestamp: i64,
}

/// Metadata for the Structured(Context) knowledge layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextMeta {
    pub source_ids: Vec<String>,
    pub processing_steps: Vec<String>,
    pub contextual_confidence: f64,
}

/// Metadata for the Semantic(Meaning) knowledge layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeaningMeta {
    pub abstraction_level: u8,
    pub supporting_facts: Vec<String>,
    pub semantic_confidence: f64,
}

/// Three-layer knowledge source hierarchy.
///
/// Mirrors the Base → Brand → Semantic token hierarchy as:
/// Raw(Perception) → Structured(Context) → Semantic(Meaning).
///
/// - Raw: Direct sensory input, unprocessed (web fetch, tool output, sensor)
/// - Structured: Processed with context, trust-weighted (parsed, categorized)
/// - Semantic: Abstracted meaning for reasoning consumption (inferred concepts)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KnowledgeLayer {
    Raw(PerceptionMeta),
    Structured(ContextMeta),
    Semantic(MeaningMeta),
}

impl KnowledgeLayer {
    /// Effective confidence for this layer with abstraction discount.
    ///
    /// Rules:
    /// - Raw: `raw_confidence` (no discount)
    /// - Structured: `contextual_confidence * 0.85` (processing loss)
    /// - Semantic: `semantic_confidence * 0.7`   (higher abstraction = higher discount)
    pub fn effective_confidence(&self) -> f64 {
        match self {
            KnowledgeLayer::Raw(meta) => meta.raw_confidence,
            KnowledgeLayer::Structured(meta) => meta.contextual_confidence * 0.85,
            KnowledgeLayer::Semantic(meta) => meta.semantic_confidence * 0.7,
        }
    }
}

/// A provenance chain tracking the evolution of knowledge across layers.
///
/// Layers are ordered from earliest (Raw) to latest (current). Each transition
/// represents a processing or abstraction step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProvenanceChain {
    /// Ordered (layer, timestamp_nanos) pairs, from oldest to newest.
    pub layers: Vec<(KnowledgeLayer, i64)>,
    pub earliest: i64,
    pub latest: i64,
    pub depth: usize,
}

impl ProvenanceChain {
    pub fn new(layers: Vec<(KnowledgeLayer, i64)>) -> Self {
        let earliest = layers.iter().map(|(_, ts)| *ts).min().unwrap_or(0);
        let latest = layers.iter().map(|(_, ts)| *ts).max().unwrap_or(0);
        let depth = layers.len();
        Self { layers, earliest, latest, depth }
    }

    /// Validate the layer isolation rule:
    /// - Raw must be first; only Structured may follow.
    /// - Structured must follow Raw; only Semantic may follow.
    /// - Semantic must follow Structured; nothing follows Semantic.
    /// - No layer type may appear after a subsequent type has been seen.
    pub fn validate_chain(&self) -> bool {
        if self.layers.is_empty() {
            return false;
        }
        let mut phase = 0u8; // 0=Raw, 1=Structured, 2=Semantic, 3=done
        for (layer, _) in &self.layers {
            match layer {
                KnowledgeLayer::Raw(_) => {
                    if phase != 0 {
                        return false;
                    }
                    phase = 1;
                }
                KnowledgeLayer::Structured(_) => {
                    if phase == 0 || phase == 2 || phase == 3 {
                        return false;
                    }
                    phase = 2;
                }
                KnowledgeLayer::Semantic(_) => {
                    if phase != 2 {
                        return false;
                    }
                    phase = 3;
                }
            }
        }
        true
    }
}

/// Trait for types that carry source hierarchy information.
pub trait SourceHierarchy: Send + Sync {
    fn layer(&self) -> &KnowledgeLayer;
    fn provenance(&self) -> &ProvenanceChain;
    fn is_semantic(&self) -> bool;
    /// Propagates confidence through layers, applying abstraction discounts.
    fn effective_confidence(&self) -> f64;
}

impl SourceHierarchy for ProvenanceChain {
    fn layer(&self) -> &KnowledgeLayer {
        &self.layers.last().expect("ProvenanceChain is empty").0
    }

    fn provenance(&self) -> &ProvenanceChain {
        self
    }

    fn is_semantic(&self) -> bool {
        matches!(self.layer(), KnowledgeLayer::Semantic(_))
    }

    /// Returns the effective confidence of the topmost (current) layer.
    fn effective_confidence(&self) -> f64 {
        self.layer().effective_confidence()
    }
}

/// Rules governing when a layer is allowed to upgrade.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UpgradeRule {
    /// Only specific perception sources can upgrade.
    Direct(Vec<PerceptionSource>),
    /// Upgrade only if confidence exceeds threshold.
    Inferred { min_confidence: f64 },
    /// Any source can upgrade.
    Automatic,
}

impl UpgradeRule {
    pub fn can_upgrade(&self, source: &PerceptionSource, confidence: f64) -> bool {
        match self {
            UpgradeRule::Direct(sources) => sources.contains(source),
            UpgradeRule::Inferred { min_confidence } => confidence >= *min_confidence,
            UpgradeRule::Automatic => true,
        }
    }
}

/// Deduce the initial knowledge layer from a perception source and confidence.
///
/// - UserInput / SearchResult → Semantic if confidence > 0.7 else Structured
/// - WebFetch / ToolOutput / FileRead → Raw
/// - Sensor → Raw
pub fn deduce_layer(source: &PerceptionSource, confidence: f64) -> KnowledgeLayer {
    match source {
        PerceptionSource::UserInput | PerceptionSource::SearchResult => {
            if confidence > 0.7 {
                KnowledgeLayer::Semantic(MeaningMeta {
                    abstraction_level: 2,
                    supporting_facts: vec![source.name().to_string()],
                    semantic_confidence: confidence,
                })
            } else {
                KnowledgeLayer::Structured(ContextMeta {
                    source_ids: vec![source.name().to_string()],
                    processing_steps: vec!["initial_classification".to_string()],
                    contextual_confidence: confidence,
                })
            }
        }
        PerceptionSource::WebFetch
        | PerceptionSource::ToolOutput
        | PerceptionSource::FileRead => KnowledgeLayer::Raw(PerceptionMeta {
            source_type: source.clone(),
            raw_confidence: confidence,
            timestamp: 0,
        }),
        PerceptionSource::Sensor => KnowledgeLayer::Raw(PerceptionMeta {
            source_type: source.clone(),
            raw_confidence: confidence,
            timestamp: 0,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw_layer(conf: f64) -> KnowledgeLayer {
        KnowledgeLayer::Raw(PerceptionMeta {
            source_type: PerceptionSource::WebFetch,
            raw_confidence: conf,
            timestamp: 1000,
        })
    }

    fn structured_layer(conf: f64) -> KnowledgeLayer {
        KnowledgeLayer::Structured(ContextMeta {
            source_ids: vec!["src1".into()],
            processing_steps: vec!["parse".into()],
            contextual_confidence: conf,
        })
    }

    fn semantic_layer(conf: f64) -> KnowledgeLayer {
        KnowledgeLayer::Semantic(MeaningMeta {
            abstraction_level: 2,
            supporting_facts: vec!["fact1".into()],
            semantic_confidence: conf,
        })
    }

    #[test]
    fn test_layer_creation_raw() {
        let meta = PerceptionMeta {
            source_type: PerceptionSource::WebFetch,
            raw_confidence: 0.9,
            timestamp: 42,
        };
        let layer = KnowledgeLayer::Raw(meta);
        assert_eq!(layer.effective_confidence(), 0.9);
    }

    #[test]
    fn test_layer_creation_structured() {
        let meta = ContextMeta {
            source_ids: vec!["a".into(), "b".into()],
            processing_steps: vec!["filter".into(), "classify".into()],
            contextual_confidence: 0.85,
        };
        let layer = KnowledgeLayer::Structured(meta);
        assert!((layer.effective_confidence() - 0.7225).abs() < 1e-9);
    }

    #[test]
    fn test_layer_creation_semantic() {
        let meta = MeaningMeta {
            abstraction_level: 3,
            supporting_facts: vec!["derived".into()],
            semantic_confidence: 0.8,
        };
        let layer = KnowledgeLayer::Semantic(meta);
        assert!((layer.effective_confidence() - 0.56).abs() < 1e-9);
    }

    #[test]
    fn test_confidence_propagation() {
        let r = raw_layer(1.0);
        let s = structured_layer(1.0);
        let m = semantic_layer(1.0);
        assert!((r.effective_confidence() - 1.0).abs() < 1e-9);
        assert!((s.effective_confidence() - 0.85).abs() < 1e-9);
        assert!((m.effective_confidence() - 0.70).abs() < 1e-9);
    }

    #[test]
    fn test_chain_validation_valid() {
        let chain = ProvenanceChain::new(vec![
            (raw_layer(0.9), 1000),
            (structured_layer(0.8), 2000),
            (semantic_layer(0.7), 3000),
        ]);
        assert!(chain.validate_chain());
    }

    #[test]
    fn test_chain_validation_raw_only() {
        let chain = ProvenanceChain::new(vec![(raw_layer(0.9), 1000)]);
        assert!(chain.validate_chain());
    }

    #[test]
    fn test_chain_validation_raw_structured() {
        let chain = ProvenanceChain::new(vec![
            (raw_layer(0.9), 1000),
            (structured_layer(0.8), 2000),
        ]);
        assert!(chain.validate_chain());
    }

    #[test]
    fn test_chain_validation_invalid_skip() {
        let chain = ProvenanceChain::new(vec![
            (raw_layer(0.9), 1000),
            (semantic_layer(0.7), 3000),
        ]);
        assert!(!chain.validate_chain());
    }

    #[test]
    fn test_chain_validation_invalid_reorder() {
        let chain = ProvenanceChain::new(vec![
            (structured_layer(0.8), 2000),
            (raw_layer(0.9), 1000),
        ]);
        assert!(!chain.validate_chain());
    }

    #[test]
    fn test_chain_validation_empty() {
        let chain = ProvenanceChain::new(vec![]);
        assert!(!chain.validate_chain());
    }

    #[test]
    fn test_isolation_rule_raw_first() {
        let chain = ProvenanceChain::new(vec![
            (structured_layer(0.8), 2000),
        ]);
        assert!(!chain.validate_chain());
    }

    #[test]
    fn test_isolation_rule_semantic_after_raw() {
        let chain = ProvenanceChain::new(vec![
            (raw_layer(0.9), 1000),
            (semantic_layer(0.7), 3000),
        ]);
        assert!(!chain.validate_chain());
    }

    #[test]
    fn test_upgrade_rule_direct() {
        let rule = UpgradeRule::Direct(vec![
            PerceptionSource::WebFetch,
            PerceptionSource::FileRead,
        ]);
        assert!(rule.can_upgrade(&PerceptionSource::WebFetch, 0.5));
        assert!(!rule.can_upgrade(&PerceptionSource::UserInput, 0.9));
    }

    #[test]
    fn test_upgrade_rule_inferred() {
        let rule = UpgradeRule::Inferred { min_confidence: 0.75 };
        assert!(rule.can_upgrade(&PerceptionSource::WebFetch, 0.8));
        assert!(!rule.can_upgrade(&PerceptionSource::UserInput, 0.5));
    }

    #[test]
    fn test_upgrade_rule_automatic() {
        let rule = UpgradeRule::Automatic;
        assert!(rule.can_upgrade(&PerceptionSource::WebFetch, 0.0));
        assert!(rule.can_upgrade(&PerceptionSource::SearchResult, 0.0));
    }

    #[test]
    fn test_deduce_layer_raw() {
        let layer = deduce_layer(&PerceptionSource::WebFetch, 0.9);
        assert!(matches!(layer, KnowledgeLayer::Raw(_)));
        assert!((layer.effective_confidence() - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_deduce_layer_semantic_high_conf() {
        let layer = deduce_layer(&PerceptionSource::UserInput, 0.9);
        assert!(matches!(layer, KnowledgeLayer::Semantic(_)));
    }

    #[test]
    fn test_deduce_layer_structured_low_conf() {
        let layer = deduce_layer(&PerceptionSource::UserInput, 0.5);
        assert!(matches!(layer, KnowledgeLayer::Structured(_)));
    }

    #[test]
    fn test_deduce_layer_search_result() {
        let layer = deduce_layer(&PerceptionSource::SearchResult, 0.8);
        assert!(matches!(layer, KnowledgeLayer::Semantic(_)));
    }

    #[test]
    fn test_deduce_layer_tool_output() {
        let layer = deduce_layer(&PerceptionSource::ToolOutput, 0.9);
        assert!(matches!(layer, KnowledgeLayer::Raw(_)));
    }

    #[test]
    fn test_deduce_layer_sensor() {
        let layer = deduce_layer(&PerceptionSource::Sensor, 0.6);
        assert!(matches!(layer, KnowledgeLayer::Raw(_)));
    }

    #[test]
    fn test_source_hierarchy_trait() {
        let chain = ProvenanceChain::new(vec![
            (raw_layer(0.9), 1000),
            (structured_layer(0.8), 2000),
        ]);
        assert!(!chain.is_semantic());
        assert!((chain.effective_confidence() - 0.68).abs() < 1e-9);
    }

    #[test]
    fn test_effective_confidence_by_layer() {
        let r = raw_layer(0.8);
        let s = structured_layer(0.8);
        let m = semantic_layer(0.8);
        assert!((r.effective_confidence() - 0.80).abs() < 1e-9);
        assert!((s.effective_confidence() - 0.68).abs() < 1e-9);
        assert!((m.effective_confidence() - 0.56).abs() < 1e-9);
    }

    #[test]
    fn test_provenance_chain_depths() {
        let chain = ProvenanceChain::new(vec![
            (raw_layer(0.9), 1000),
            (structured_layer(0.8), 2000),
            (semantic_layer(0.7), 3000),
        ]);
        assert_eq!(chain.depth, 3);
        assert_eq!(chain.earliest, 1000);
        assert_eq!(chain.latest, 3000);
    }

    #[test]
    fn test_perception_source_name() {
        assert_eq!(PerceptionSource::WebFetch.name(), "web_fetch");
        assert_eq!(PerceptionSource::UserInput.name(), "user_input");
        assert_eq!(PerceptionSource::ToolOutput.name(), "tool_output");
        assert_eq!(PerceptionSource::Sensor.name(), "sensor");
        assert_eq!(PerceptionSource::SearchResult.name(), "search_result");
        assert_eq!(PerceptionSource::FileRead.name(), "file_read");
    }
}
