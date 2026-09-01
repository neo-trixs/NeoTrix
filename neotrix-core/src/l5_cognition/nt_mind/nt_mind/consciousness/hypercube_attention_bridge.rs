use crate::core::nt_core_hcube::axis::DimensionAxis;
use crate::core::nt_core_hcube::coord::HyperCoord;
use crate::core::nt_core_hcube::cube::{KnowledgeHyperCube, CubeEntry};
use crate::core::nt_core_self::attention_head::{AttentionDomain, AttentionManager};

#[derive(Debug, Clone)]
pub struct AttentionRecallItem {
    pub domain: AttentionDomain,
    pub domain_activation: f64,
    pub dimension: DimensionAxis,
    pub top_results: Vec<CubeEntry>,
    pub avg_similarity: f64,
}

pub struct AttentionHypercubeBridge {
    pub domain_to_dimension: Vec<(AttentionDomain, DimensionAxis)>,
    pub last_recall: Vec<(String, f64)>,
}

impl AttentionHypercubeBridge {
    pub fn new() -> Self {
        Self {
            domain_to_dimension: vec![
                (AttentionDomain::PatternMatch, DimensionAxis::Abstraction),
                (AttentionDomain::Code, DimensionAxis::CodeUnderstanding),
                (AttentionDomain::Semantic, DimensionAxis::KnowledgeRetrieval),
                (AttentionDomain::Temporal, DimensionAxis::Time),
                (AttentionDomain::Planning, DimensionAxis::SystemDesign),
                (AttentionDomain::SelfReflection, DimensionAxis::Safety),
                (AttentionDomain::ToolUse, DimensionAxis::Performance),
                (AttentionDomain::GoalAlignment, DimensionAxis::Agency),
                (AttentionDomain::RiskAssessment, DimensionAxis::Certainty),
                (AttentionDomain::Creativity, DimensionAxis::Creativity),
            ],
            last_recall: Vec::new(),
        }
    }

    pub fn recall_from_attention(
        &self,
        attention: &AttentionManager,
        hypercube: &KnowledgeHyperCube,
    ) -> Vec<AttentionRecallItem> {
        let active = attention.active_heads();
        if active.is_empty() || hypercube.is_empty() {
            return Vec::new();
        }

        let mut results: Vec<AttentionRecallItem> = Vec::new();

        for head in active {
            let Some(dim) = self.domain_dimension(&head.domain) else { continue };
            let mut coord = HyperCoord::with(dim, 0.9);
            coord.set(DimensionAxis::Abstraction, 0.5);

            let entries = hypercube.query(&coord, 5);
            if entries.is_empty() {
                continue;
            }

            let avg_sim: f64 = entries.iter().map(|e| e.value).sum::<f64>() / entries.len() as f64;

            results.push(AttentionRecallItem {
                domain: head.domain,
                domain_activation: head.activation,
                dimension: dim,
                top_results: entries.into_iter().cloned().collect(),
                avg_similarity: avg_sim,
            });
        }

        results.sort_by(|a, b| b.domain_activation.partial_cmp(&a.domain_activation).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    pub fn domain_dimension(&self, domain: &AttentionDomain) -> Option<DimensionAxis> {
        self.domain_to_dimension.iter()
            .find(|(d, _)| d == domain)
            .map(|(_, axis)| *axis)
    }
}

impl Default for AttentionHypercubeBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bridge() -> AttentionHypercubeBridge {
        AttentionHypercubeBridge::new()
    }

    fn make_manager() -> AttentionManager {
        let mut mgr = AttentionManager::new(0.3);
        mgr.stimulate_domain(AttentionDomain::Code, 0.9);
        mgr.stimulate_domain(AttentionDomain::Planning, 0.7);
        mgr.stimulate_domain(AttentionDomain::Creativity, 0.4);
        mgr
    }

    fn make_cube_with_data() -> KnowledgeHyperCube {
        let mut cube = KnowledgeHyperCube::new();
        let coord = HyperCoord::with(DimensionAxis::CodeUnderstanding, 0.9);
        cube.insert(&coord, "rust-docs", "Rust ownership system");
        cube.insert(&coord, "rust-docs", "Rust trait system");
        let coord2 = HyperCoord::with(DimensionAxis::SystemDesign, 0.9);
        cube.insert(&coord2, "arch-guide", "Clean Architecture patterns");
        cube
    }

    #[test]
    fn test_bridge_new() {
        let bridge = make_bridge();
        assert_eq!(bridge.domain_to_dimension.len(), 10);
        assert!(bridge.last_recall.is_empty());
    }

    #[test]
    fn test_all_10_domains_mapped() {
        let bridge = make_bridge();
        for domain in AttentionDomain::all() {
            assert!(
                bridge.domain_dimension(&domain).is_some(),
                "domain {:?} is missing a dimension mapping",
                domain
            );
        }
    }

    #[test]
    fn test_domain_dimension_mapping() {
        let bridge = make_bridge();

        assert_eq!(bridge.domain_dimension(&AttentionDomain::Code), Some(DimensionAxis::CodeUnderstanding));
        assert_eq!(bridge.domain_dimension(&AttentionDomain::Planning), Some(DimensionAxis::SystemDesign));
        assert_eq!(bridge.domain_dimension(&AttentionDomain::Creativity), Some(DimensionAxis::Creativity));
        assert_eq!(bridge.domain_dimension(&AttentionDomain::PatternMatch), Some(DimensionAxis::Abstraction));
        assert_eq!(bridge.domain_dimension(&AttentionDomain::Semantic), Some(DimensionAxis::KnowledgeRetrieval));
        assert_eq!(bridge.domain_dimension(&AttentionDomain::Temporal), Some(DimensionAxis::Time));
        assert_eq!(bridge.domain_dimension(&AttentionDomain::SelfReflection), Some(DimensionAxis::Safety));
        assert_eq!(bridge.domain_dimension(&AttentionDomain::ToolUse), Some(DimensionAxis::Performance));
        assert_eq!(bridge.domain_dimension(&AttentionDomain::GoalAlignment), Some(DimensionAxis::Agency));
        assert_eq!(bridge.domain_dimension(&AttentionDomain::RiskAssessment), Some(DimensionAxis::Certainty));
    }

    #[test]
    fn test_recall_from_empty_hypercube() {
        let bridge = make_bridge();
        let manager = make_manager();
        let cube = KnowledgeHyperCube::new();
        let results = bridge.recall_from_attention(&manager, &cube);
        assert!(results.is_empty());
    }

    #[test]
    fn test_recall_from_populated_hypercube() {
        let bridge = make_bridge();
        let manager = make_manager();
        let cube = make_cube_with_data();
        let results = bridge.recall_from_attention(&manager, &cube);

        assert!(!results.is_empty());
        for item in &results {
            assert!(item.domain_activation >= 0.0);
            assert!(!item.top_results.is_empty());
        }
        // Code domain (0.9 activation) should be first
        assert_eq!(results[0].domain, AttentionDomain::Code);
    }

    #[test]
    fn test_no_active_heads_returns_empty() {
        let bridge = make_bridge();
        let manager = AttentionManager::new(0.9); // high threshold, no heads active
        let cube = make_cube_with_data();
        let results = bridge.recall_from_attention(&manager, &cube);
        assert!(results.is_empty());
    }

    #[test]
    fn test_default_impl() {
        let bridge: AttentionHypercubeBridge = Default::default();
        assert_eq!(bridge.domain_to_dimension.len(), 10);
    }
}
