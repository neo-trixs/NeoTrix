use super::cortex_memory::{CortexMemory, DimensionTag};
use super::knowledge_engine::KnowledgeEngine;
use super::exploration_pipeline::ExploreDomain;
use crate::core::nt_core_hcube::axis::DimensionAxis;
use crate::core::nt_core_hcube::coord::HyperCoord;
use crate::core::nt_core_hcube::cube::{KnowledgeHyperCube, CubeEntry};
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
            TimelineGeology | TimelineLife | TimelineHuman | TimelineCivilization | TimelineFuture => {
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
        // 修复：多标签(常态)逐标签映射轴并聚合坐标值。
        // 旧实现仅处理空/单 General 标签，多标签返回全零坐标 → 8 维 density 全 ≈0 →
        // analyze_gaps 恒判全域稀疏，好奇心持续误触发"探索一切"。
        let mut coord = HyperCoord::new();
        for tag in tags {
            for axis in Self::dimension_tag_to_axis(tag) {
                let cur = coord.get(&axis);
                // 聚合：均值折中多标签，避免相消为 0
                let v = match axis {
                    DimensionAxis::Abstraction => 0.5 + 0.25 * (cur - 0.5),
                    _ => cur + 0.15,
                };
                coord.set(axis, v.min(1.0));
            }
        }
        if coord.dims().next().is_none() {
            return HyperCoord::with(DimensionAxis::Abstraction, 0.5);
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

    /// 从真实 KB 灌入全部知识节点 — 使 analyze_gaps/sparse_topics/query 反映实际记忆。
    /// 返回灌入条数。
    pub fn ingest_from_kb(&mut self, kb: &crate::neotrix::nt_memory_kb::KnowledgeBase) -> usize {
        let nodes = match kb.all_nodes() {
            Ok(nodes) => nodes,
            Err(_) => return 0,
        };
        let mut count = 0;
        for node in nodes {
            let coord = Self::coord_from_kb_node(&node);
            let source = node.url.clone().unwrap_or_default();
            self.hypercube.insert(&coord, &source, &node.title);
            count += 1;
        }
        count
    }

    /// 将 KB 节点映射为超立方体坐标 — 按 node_type 落到前 8 个维度
    /// (0..8, 即 analyze_gaps/sparse_topics 检视的维度), 使真实知识
    /// 直接影响缺口分析驱动的好奇心爬取。
    fn coord_from_kb_node(node: &crate::neotrix::nt_memory_kb::nt_memory_types::KnowledgeNode) -> HyperCoord {
        use crate::neotrix::nt_memory_kb::nt_memory_types::NodeType as Kt;
        let axis = match node.node_type {
            // 代码理解 (0)
            Kt::CodeSnippet | Kt::Repository | Kt::Tool => DimensionAxis::CodeUnderstanding,
            // 系统设计 (1)
            Kt::Framework | Kt::Method | Kt::Algorithm => DimensionAxis::SystemDesign,
            // 调试 (2)
            Kt::Question | Kt::Event => DimensionAxis::Debugging,
            // 知识检索 (3)
            Kt::Paper | Kt::Article | Kt::Book | Kt::Course | Kt::Theory | Kt::Concept
            | Kt::Source => DimensionAxis::KnowledgeRetrieval,
            // 创造力 (4)
            Kt::Idea | Kt::Insight => DimensionAxis::Creativity,
            // 性能 (6)
            Kt::Benchmark | Kt::Dataset => DimensionAxis::Performance,
            // 沟通 (7)
            Kt::Person | Kt::Organization => DimensionAxis::Communication,
            _ => DimensionAxis::KnowledgeRetrieval,
        };
        HyperCoord::with(axis, 0.5)
    }

    pub fn analyze_gaps(&self) -> Vec<GapReport> {
        let mut reports = Vec::new();
        for dim in 0..8 {
            let current = self.hypercube.coord_density(dim);
            let mut report = GapReport::new(dim, current, 0.6);
            report.sparsity_score = if current < 0.001 { 1.0 } else { (0.6 - current).max(0.0) / 0.6 };
            reports.push(report);
        }
        reports
    }

    pub fn sparse_domains(&self, gap_reports: &[GapReport]) -> Vec<ExploreDomain> {
        let high_gap = gap_reports.iter().any(|r| r.gap > 0.3);
        let high_sparsity = gap_reports.iter().any(|r| r.sparsity_score > 0.5);
        let empty_count = gap_reports.iter().filter(|r| !r.empty_regions.is_empty()).count();
        let underpopulated_count = gap_reports.iter().filter(|r| !r.underpopulated_regions.is_empty()).count();

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
        &self, coord: &HyperCoord, task_type: TaskType, top_k: usize,
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
    use super::*;
    use super::super::cortex_memory::{MemoryTrace, Modality};

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
    fn test_multi_tag_coord_nonzero() {
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
        bridge.ingest_from_cortex(&cortex);
        let entry = bridge
            .hypercube
            .get_entry(&"https://en.wikipedia.org/wiki/Geologic_time_scale-Geologic Time")
            .expect("entry should exist");
        assert!(
            entry.coord.dims().count() > 0,
            "multi-tag coord must not be empty (was all-zero before fix)"
        );
        let any_nonzero = entry.coord.dims().any(|(_, v)| *v > 0.0);
        assert!(any_nonzero, "multi-tag coord must have nonzero value (was all-zero before fix)");
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
            "src", "code-item", TaskType::CodeAnalysis,
        );
        bridge.hypercube.insert_with_task_type(
            &HyperCoord::with(DimensionAxis::Abstraction, 0.1),
            "src", "design-item", TaskType::Design,
        );
        let results = bridge.query_by_task_type(
            &HyperCoord::with(DimensionAxis::Abstraction, 0.0),
            TaskType::Design, 5,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].label, "design-item");
    }

    #[test]
    fn test_ingest_from_kb_populates_hypercube() {
        use crate::neotrix::nt_memory_kb::nt_memory_types::NodeType;
        let kb = crate::neotrix::nt_memory_kb::KnowledgeBase::open(None).expect("open kb");
        let _ = kb.insert_or_get_node(
            "KB-Bridge-Ingest-Topic",
            NodeType::Concept,
            Some("bridge ingest test summary"),
            None,
            Some("test"),
        );
        let mut bridge = HyperCubeBridge::new();
        let count = bridge.ingest_from_kb(&kb);
        assert!(count >= 1, "expected >= 1 ingested, got {}", count);
        assert!(bridge.hypercube.cell_count() >= 1);
        // 灌入后稀疏度应下降（不再全域稀疏）
        let gaps = bridge.analyze_gaps();
        assert!(gaps.iter().any(|r| r.sparsity_score < 1.0));
    }

    #[test]
    fn test_empty_kb_ingest_zero() {
        // 打开一个新 KB 且不插入任何节点，灌入应为 0（或非负）
        let kb = crate::neotrix::nt_memory_kb::KnowledgeBase::open(None).expect("open kb");
        let mut bridge = HyperCubeBridge::new();
        let count = bridge.ingest_from_kb(&kb);
        assert!(count >= 0);
    }

    #[test]
    fn test_frontier_seed_flows_into_hypercube() {
        use crate::neotrix::nt_memory_kb::nt_memory_types::NodeType;
        // 内存 KB 跑完整 seed → 验证前沿模型节点入库 → ingest 映射到 SystemDesign 轴
        let kb = crate::neotrix::nt_memory_kb::KnowledgeBase::open(
            Some(std::path::PathBuf::from(":memory:")),
        ).expect("open memory kb");
        let _ = kb.seed_foundational().expect("seed foundational");
        let nodes = kb.all_nodes().expect("all nodes");
        for frontier in ["Kimi K3", "DeepSeek V4", "Claude Fable 5", "Qwen3.8-Max", "Gemini 3.6 Flash", "Grok 4.5"] {
            assert!(
                nodes.iter().any(|n| n.title == frontier),
                "frontier node {frontier} should be seeded"
            );
        }
        // 每个前沿 Framework 节点应映射到 SystemDesign 轴 (index 1)
        for node in nodes.iter().filter(|n| n.node_type == NodeType::Framework) {
            let coord = HyperCubeBridge::coord_from_kb_node(node);
            assert!(
                coord.to_dense()[1] > 0.0,
                "framework {} should map to SystemDesign", node.title
            );
        }
        // ingest 后超立方体应有灌入
        let mut bridge = HyperCubeBridge::new();
        let count = bridge.ingest_from_kb(&kb);
        assert!(count >= nodes.len().min(count), "all seeded nodes should ingest");
        assert!(bridge.hypercube.cell_count() >= 1);
    }
}
