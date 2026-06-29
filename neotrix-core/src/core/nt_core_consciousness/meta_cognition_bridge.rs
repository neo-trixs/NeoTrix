// Phase 4: MetaCognition Bridge — Splice SelfUnderstanding → ArchitectureSelfModel
// Closes the meta-cognition loop: self-knowledge graph → architecture health assessment

use crate::core::nt_core_experience::self_understanding::SelfUnderstandingEngine;
use crate::core::nt_core_self::architecture_governor::ArchitectureSelfModel;

#[derive(Debug)]
pub struct MetaCognitionBridge {
    pub self_understanding: SelfUnderstandingEngine,
    pub architecture_model: ArchitectureSelfModel,
    pub last_bridge_tick: u64,
}

impl MetaCognitionBridge {
    pub fn new() -> Self {
        let mut sue = SelfUnderstandingEngine::new();
        sue.register_core_modules();
        Self {
            self_understanding: sue,
            architecture_model: ArchitectureSelfModel::new(),
            last_bridge_tick: 0,
        }
    }

    pub fn run_bridge(&mut self, cycle: u64) -> BridgeReport {
        let entities = self.self_understanding.export_layer_map();
        let insights = self.architecture_model.ingest_entities(&entities);
        let diagnosis = self.self_understanding.diagnose();
        let report = self.architecture_model.generate_report();

        self.last_bridge_tick = cycle;

        BridgeReport {
            total_entities: diagnosis.total_entities,
            total_edges: diagnosis.total_edges,
            layer_gaps: diagnosis.layer_gaps,
            new_insights: insights.len(),
            architecture_report: report,
        }
    }

    pub fn explain(&self, entity_id: &str) -> String {
        self.self_understanding.explain(entity_id)
    }

    pub fn detect_smells(&mut self) -> Vec<String> {
        self.architecture_model
            .detect_code_smells()
            .into_iter()
            .map(|i| format!("{}: {} (sev={:.2})", i.module, i.description, i.severity))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct BridgeReport {
    pub total_entities: usize,
    pub total_edges: usize,
    pub layer_gaps: Vec<String>,
    pub new_insights: usize,
    pub architecture_report: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_new() {
        let bridge = MetaCognitionBridge::new();
        assert!(bridge.self_understanding.graph.entities.len() >= 8);
        assert_eq!(bridge.architecture_model.module_count(), 0);
    }

    #[test]
    fn test_bridge_run_populates_modules() {
        let mut bridge = MetaCognitionBridge::new();
        let report = bridge.run_bridge(1);
        assert!(report.total_entities >= 8);
        assert!(bridge.architecture_model.module_count() >= 8);
        assert!(!report.architecture_report.is_empty());
    }

    #[test]
    fn test_bridge_detect_gaps() {
        let mut bridge = MetaCognitionBridge::new();
        bridge.run_bridge(1);
        let smells = bridge.detect_smells();
        // Should find at least underutilized modules (invocations=0)
        assert!(smells.len() >= 1);
    }

    #[test]
    fn test_bridge_explain() {
        let bridge = MetaCognitionBridge::new();
        let explanation = bridge.explain("hcube");
        assert!(explanation.contains("HyperCube"));
    }
}
