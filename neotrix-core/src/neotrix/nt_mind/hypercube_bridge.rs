use super::cortex_memory::{CortexMemory, DimensionTag};
use super::exploration_pipeline::ExploreDomain;
use super::knowledge_engine::KnowledgeEngine;
use crate::core::nt_core_hcube::axis::DimensionAxis;
use crate::core::nt_core_hcube::coord::HyperCoord;
use crate::core::nt_core_hcube::cube::{CubeEntry, KnowledgeHyperCube};
use crate::core::nt_core_hcube::gap::GapReport;
use crate::core::nt_core_knowledge::TaskType;

pub struct HyperCubeBridge {
    pub cortex: CortexMemory,
    pub knowledge: KnowledgeEngine,
    pub hypercube: KnowledgeHyperCube,
}

impl HyperCubeBridge {
    pub fn new() -> Self {
        Self {
            cortex: CortexMemory::new(10, 100),
            knowledge: KnowledgeEngine::new(100),
            hypercube: KnowledgeHyperCube::new(),
        }
    }

    pub fn dimension_tag_to_axis(tag: &DimensionTag) -> Vec<DimensionAxis> {
        use DimensionTag::*;
        match tag {
            General => DimensionAxis::all().to_vec(),
            TimelineGeology | TimelineLife | TimelineHuman | TimelineCivilization
            | TimelineFuture => {
                vec![DimensionAxis::Time]
            }
            TechAgriculture | TechIndustrial | TechInformation | TechSpace | TechAI => {
                vec![DimensionAxis::Domain]
            }
            KnowledgePhilosophy | KnowledgeScience => vec![DimensionAxis::Abstraction],
            KnowledgeCulture => vec![DimensionAxis::Culture],
            CosmoSpacetime | CosmoMultiverse | CosmoDimension => vec![DimensionAxis::Scale],
            _ => vec![DimensionAxis::Abstraction],
        }
    }

    fn build_coord_from_tags(tags: &[DimensionTag]) -> HyperCoord {
        if tags.is_empty() {
            return HyperCoord::with(DimensionAxis::Abstraction, 0.5);
        }
        let mut coord = HyperCoord::new();
        if tags.len() == 1 && tags[0] == DimensionTag::General {
            for &axis in DimensionAxis::all() {
                coord.set(axis, 0.5);
            }
        }
        coord
    }

    pub fn ingest_from_cortex(&mut self, cortex: &CortexMemory) -> usize {
        let mut count = 0;
        for trace in cortex.all_traces() {
            let coord = Self::build_coord_from_tags(&trace.dimensions);
            self.hypercube.insert(&coord, &trace.source, &trace.title);
            count += 1;
        }
        count
    }

    pub fn analyze_gaps(&self) -> Vec<GapReport> {
        let mut reports = Vec::new();
        for dim in 0..8 {
            let current = self.hypercube.coord_density(dim);
            let mut report = GapReport::new(dim, current, 0.6);
            report.sparsity_score = if current < 0.001 {
                1.0
            } else {
                (0.6 - current).max(0.0) / 0.6
            };
            reports.push(report);
        }
        reports
    }

    pub fn sparse_domains(&self, gap_reports: &[GapReport]) -> Vec<ExploreDomain> {
        let high_gap = gap_reports.iter().any(|r| r.gap > 0.3);
        let high_sparsity = gap_reports.iter().any(|r| r.sparsity_score > 0.5);
        let empty_count = gap_reports
            .iter()
            .filter(|r| !r.empty_regions.is_empty())
            .count();
        let underpopulated_count = gap_reports
            .iter()
            .filter(|r| !r.underpopulated_regions.is_empty())
            .count();

        if high_sparsity || empty_count > 3 {
            return vec![ExploreDomain::General];
        }

        let mut domains = Vec::new();
        if high_gap || underpopulated_count > 2 {
            domains.push(ExploreDomain::Wiki);
        }
        if empty_count > 1 {
            domains.push(ExploreDomain::Papers);
        }
        if domains.is_empty() {
            domains.push(ExploreDomain::General);
        }
        domains
    }

    pub fn query(&self, coord: &HyperCoord, top_k: usize) -> Vec<CubeEntry> {
        self.hypercube
            .query(coord, top_k)
            .into_iter()
            .cloned()
            .collect()
    }

    pub fn query_by_task_type(
        &self,
        coord: &HyperCoord,
        task_type: TaskType,
        top_k: usize,
    ) -> Vec<CubeEntry> {
        self.hypercube
            .query_by_task_type(coord, task_type, top_k)
            .into_iter()
            .cloned()
            .collect()
    }

    pub fn prune_low_access(&mut self, min_access: u64) -> usize {
        self.hypercube.prune_low_access(min_access)
    }
}

impl Default for HyperCubeBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::cortex_memory::{MemoryTrace, Modality};
    use super::*;

    #[test]
    fn test_dimension_tag_mapping() {
        let result = HyperCubeBridge::dimension_tag_to_axis(&DimensionTag::TimelineGeology);
        assert_eq!(result, vec![DimensionAxis::Time]);

        let result = HyperCubeBridge::dimension_tag_to_axis(&DimensionTag::TechAI);
        assert_eq!(result, vec![DimensionAxis::Domain]);

        let result = HyperCubeBridge::dimension_tag_to_axis(&DimensionTag::KnowledgeScience);
        assert_eq!(result, vec![DimensionAxis::Abstraction]);

        let result = HyperCubeBridge::dimension_tag_to_axis(&DimensionTag::KnowledgeCulture);
        assert_eq!(result, vec![DimensionAxis::Culture]);

        let result = HyperCubeBridge::dimension_tag_to_axis(&DimensionTag::CosmoSpacetime);
        assert_eq!(result, vec![DimensionAxis::Scale]);

        let result = HyperCubeBridge::dimension_tag_to_axis(&DimensionTag::General);
        assert_eq!(result.len(), DimensionAxis::count());
    }

    #[test]
    fn test_bridge_new() {
        let bridge = HyperCubeBridge::new();
        assert_eq!(bridge.hypercube.cell_count(), 0);
    }

    #[test]
    fn test_empty_bridge_high_sparsity() {
        let bridge = HyperCubeBridge::new();
        let report = bridge.analyze_gaps();
        assert!(report.iter().all(|r| r.sparsity_score > 0.8));
    }

    #[test]
    fn test_ingest_from_cortex() {
        let mut cortex = CortexMemory::new(10, 100);
        let trace = MemoryTrace::new(
            "Geologic Time",
            "https://en.wikipedia.org/wiki/Geologic_time_scale",
            "summary about geologic eras",
            Modality::Text,
            vec![DimensionTag::TimelineGeology, DimensionTag::TechAI],
        );
        cortex.store(trace);

        let mut bridge = HyperCubeBridge::new();
        let count = bridge.ingest_from_cortex(&cortex);
        assert_eq!(count, 1);
        assert_eq!(bridge.hypercube.cell_count(), 1);
    }

    #[test]
    fn test_query_after_ingest() {
        let mut bridge = HyperCubeBridge::new();
        let mut cortex = CortexMemory::new(10, 100);
        cortex.store(MemoryTrace::new(
            "Test Entry",
            "https://example.com",
            "test summary",
            Modality::Text,
            vec![DimensionTag::General],
        ));
        bridge.ingest_from_cortex(&cortex);

        let coord = HyperCoord::with(DimensionAxis::Abstraction, 0.5);
        let results = bridge.query(&coord, 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source, "https://example.com");
    }

    #[test]
    fn test_bridge_query_by_task_type() {
        use crate::core::nt_core_knowledge::TaskType;
        let mut bridge = HyperCubeBridge::new();
        bridge.hypercube.insert_with_task_type(
            &HyperCoord::with(DimensionAxis::Abstraction, 0.9),
            "src",
            "code-item",
            TaskType::CodeAnalysis,
        );
        bridge.hypercube.insert_with_task_type(
            &HyperCoord::with(DimensionAxis::Abstraction, 0.1),
            "src",
            "design-item",
            TaskType::Design,
        );
        let results = bridge.query_by_task_type(
            &HyperCoord::with(DimensionAxis::Abstraction, 0.0),
            TaskType::Design,
            5,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].label, "design-item");
    }
}
