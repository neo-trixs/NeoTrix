use serde::{Deserialize, Serialize};
use super::source_hierarchy::{KnowledgeLayer, ProvenanceChain, SourceHierarchy};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VsaOrigin {
    Self_(VsaSelfCategory),
    World(VsaWorldCategory),
}

impl VsaOrigin {
    pub fn is_self(&self) -> bool {
        matches!(self, VsaOrigin::Self_(_))
    }

    pub fn is_world(&self) -> bool {
        matches!(self, VsaOrigin::World(_))
    }

    pub fn category_name(&self) -> &'static str {
        match self {
            VsaOrigin::Self_(c) => c.name(),
            VsaOrigin::World(c) => c.name(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VsaSelfCategory {
    Thought,
    Memory,
    Plan,
    Intention,
    Emotion,
    Imagination,
    MetaCognition,
}

impl VsaSelfCategory {
    pub fn name(&self) -> &'static str {
        match self {
            VsaSelfCategory::Thought => "thought",
            VsaSelfCategory::Memory => "memory",
            VsaSelfCategory::Plan => "plan",
            VsaSelfCategory::Intention => "intention",
            VsaSelfCategory::Emotion => "emotion",
            VsaSelfCategory::Imagination => "imagination",
            VsaSelfCategory::MetaCognition => "metacognition",
        }
    }

    pub fn all() -> &'static [VsaSelfCategory] {
        &[
            VsaSelfCategory::Thought,
            VsaSelfCategory::Memory,
            VsaSelfCategory::Plan,
            VsaSelfCategory::Intention,
            VsaSelfCategory::Emotion,
            VsaSelfCategory::Imagination,
            VsaSelfCategory::MetaCognition,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VsaWorldCategory {
    UserInput,
    Sensor,
    WebContent,
    ToolOutput,
    CodeAnalysis,
    FileContent,
    SystemEvent,
}

impl VsaWorldCategory {
    pub fn name(&self) -> &'static str {
        match self {
            VsaWorldCategory::UserInput => "user_input",
            VsaWorldCategory::Sensor => "sensor",
            VsaWorldCategory::WebContent => "web_content",
            VsaWorldCategory::ToolOutput => "tool_output",
            VsaWorldCategory::CodeAnalysis => "code_analysis",
            VsaWorldCategory::FileContent => "file_content",
            VsaWorldCategory::SystemEvent => "system_event",
        }
    }

    pub fn all() -> &'static [VsaWorldCategory] {
        &[
            VsaWorldCategory::UserInput,
            VsaWorldCategory::Sensor,
            VsaWorldCategory::WebContent,
            VsaWorldCategory::ToolOutput,
            VsaWorldCategory::CodeAnalysis,
            VsaWorldCategory::FileContent,
            VsaWorldCategory::SystemEvent,
        ]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VsaTagged {
    pub vector: Vec<u8>,
    pub tag: VsaOrigin,
    pub confidence: f64,
    pub timestamp: std::time::Instant,
    pub salience: f64,
    pub provenance: Option<ProvenanceChain>,
}

impl VsaTagged {
    pub fn new(vector: Vec<u8>, tag: VsaOrigin) -> Self {
        Self {
            vector,
            tag,
            confidence: 1.0,
            timestamp: std::time::Instant::now(),
            salience: 0.5,
            provenance: None,
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn world_input(data: &str) -> Self {
        let bytes: Vec<u8> = data.bytes().collect();
        Self::new(bytes, VsaOrigin::World(VsaWorldCategory::UserInput))
    }

    pub fn with_salience(mut self, salience: f64) -> Self {
        self.salience = salience.clamp(0.0, 1.0);
        self
    }

    pub fn is_self(&self) -> bool {
        self.tag.is_self()
    }

    pub fn is_world(&self) -> bool {
        self.tag.is_world()
    }

    /// Estimate retention priority from confidence and vector density.
    pub fn retention_score(&self) -> f64 {
        let density = self.vector.iter().filter(|&&b| b != 0).count() as f64
            / self.vector.len().max(1) as f64;
        self.confidence * 0.7 + density * 0.3
    }

    /// Attach a provenance chain to this tag.
    pub fn with_provenance(mut self, provenance: ProvenanceChain) -> Self {
        self.provenance = Some(provenance);
        self
    }

    /// Get the knowledge layer of this tag's provenance chain.
    pub fn knowledge_layer(&self) -> Option<&KnowledgeLayer> {
        self.provenance
            .as_ref()
            .and_then(|p| p.layers.last())
            .map(|(layer, _)| layer)
    }

    /// Override retention_score to factor in provenance confidence.
    pub fn retention_score_with_provenance(&self) -> f64 {
        let base = self.retention_score();
        if let Some(ref prov) = self.provenance {
            let layer_conf = prov.effective_confidence();
            0.6 * base + 0.4 * layer_conf
        } else {
            base
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vsa_origin_self() {
        let tag = VsaOrigin::Self_(VsaSelfCategory::Thought);
        assert!(tag.is_self());
        assert!(!tag.is_world());
    }

    #[test]
    fn test_vsa_origin_world() {
        let tag = VsaOrigin::World(VsaWorldCategory::UserInput);
        assert!(!tag.is_self());
        assert!(tag.is_world());
    }

    #[test]
    fn test_vsa_tagged_roundtrip() {
        let vector = vec![1; 256];
        let tag = VsaOrigin::Self_(VsaSelfCategory::Memory);
        let tagged = VsaTagged::new(vector.clone(), tag);
        assert_eq!(tagged.vector, vector);
        assert_eq!(tagged.tag, tag);
        assert!(tagged.is_self());
    }

    #[test]
    fn test_self_categories_distinct() {
        let cats = VsaSelfCategory::all();
        let mut unique = cats.to_vec();
        unique.sort_by_key(|c| *c as u8);
        unique.dedup();
        assert_eq!(cats.len(), unique.len());
    }

    #[test]
    fn test_world_categories_distinct() {
        let cats = VsaWorldCategory::all();
        let mut unique = cats.to_vec();
        unique.sort_by_key(|c| *c as u8);
        unique.dedup();
        assert_eq!(cats.len(), unique.len());
    }

    #[test]
    fn test_confidence_clamping() {
        let vector = vec![0; 10];
        let tag = VsaOrigin::Self_(VsaSelfCategory::Emotion);
        let tagged = VsaTagged::new(vector, tag).with_confidence(1.5);
        assert!((tagged.confidence - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_salience_default() {
        let tagged = VsaTagged::new(vec![1; 16], VsaOrigin::Self_(VsaSelfCategory::Thought));
        assert!((tagged.salience - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_with_salience_clamping() {
        let tagged = VsaTagged::new(vec![1; 16], VsaOrigin::Self_(VsaSelfCategory::Thought))
            .with_salience(1.5);
        assert!((tagged.salience - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_provenance_default_none() {
        let tagged = VsaTagged::new(vec![1; 16], VsaOrigin::Self_(VsaSelfCategory::Thought));
        assert!(tagged.provenance.is_none());
    }

    #[test]
    fn test_with_provenance_attaches_chain() {
        use crate::core::nt_core_consciousness::source_hierarchy::{
            KnowledgeLayer, PerceptionMeta, PerceptionSource, ProvenanceChain,
        };
        let raw = KnowledgeLayer::Raw(PerceptionMeta {
            source_type: PerceptionSource::WebFetch,
            raw_confidence: 0.9,
            timestamp: 1000,
        });
        let chain = ProvenanceChain::new(vec![(raw, 1000)]);
        let tagged = VsaTagged::new(vec![1; 16], VsaOrigin::Self_(VsaSelfCategory::Thought))
            .with_provenance(chain);
        assert!(tagged.provenance.is_some());
    }

    #[test]
    fn test_knowledge_layer_none_without_provenance() {
        let tagged = VsaTagged::new(vec![1; 16], VsaOrigin::Self_(VsaSelfCategory::Thought));
        assert!(tagged.knowledge_layer().is_none());
    }

    #[test]
    fn test_knowledge_layer_returns_topmost() {
        use crate::core::nt_core_consciousness::source_hierarchy::{
            ContextMeta, KnowledgeLayer, PerceptionMeta, PerceptionSource, ProvenanceChain,
        };
        let raw = KnowledgeLayer::Raw(PerceptionMeta {
            source_type: PerceptionSource::WebFetch,
            raw_confidence: 0.9,
            timestamp: 1000,
        });
        let structured = KnowledgeLayer::Structured(ContextMeta {
            source_ids: vec!["src".into()],
            processing_steps: vec!["parse".into()],
            contextual_confidence: 0.8,
        });
        let chain = ProvenanceChain::new(vec![(raw, 1000), (structured, 2000)]);
        let tagged = VsaTagged::new(vec![1; 16], VsaOrigin::Self_(VsaSelfCategory::Thought))
            .with_provenance(chain);
        assert!(matches!(tagged.knowledge_layer(), Some(KnowledgeLayer::Structured(_))));
    }

    #[test]
    fn test_retention_score_with_provenance_factors_in_confidence() {
        use crate::core::nt_core_consciousness::source_hierarchy::{
            ContextMeta, KnowledgeLayer, PerceptionMeta, PerceptionSource, ProvenanceChain,
        };
        let raw = KnowledgeLayer::Raw(PerceptionMeta {
            source_type: PerceptionSource::WebFetch,
            raw_confidence: 0.9,
            timestamp: 1000,
        });
        let structured = KnowledgeLayer::Structured(ContextMeta {
            source_ids: vec!["src".into()],
            processing_steps: vec!["parse".into()],
            contextual_confidence: 0.8,
        });
        let chain = ProvenanceChain::new(vec![(raw, 1000), (structured, 2000)]);
        let tagged = VsaTagged::new(vec![1; 16], VsaOrigin::Self_(VsaSelfCategory::Thought))
            .with_provenance(chain);
        let with_prov = tagged.retention_score_with_provenance();
        // Confirm provenance path differs from base (all-ones vector → density=1.0 → base = 0.7+0.3=1.0)
        // effective_confidence = 0.8 * 0.85 = 0.68
        // with_prov = 0.6 * 1.0 + 0.4 * 0.68 = 0.872
        assert!((with_prov - 0.872).abs() < 1e-9);
    }

    #[test]
    fn test_retention_score_without_provenance_falls_back() {
        let tagged = VsaTagged::new(vec![0; 16], VsaOrigin::Self_(VsaSelfCategory::Thought));
        assert!((tagged.retention_score_with_provenance() - tagged.retention_score()).abs() < 1e-9);
    }
}
