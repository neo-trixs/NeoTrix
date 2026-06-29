use super::types::*;
use super::sub_hive::SubHiveRegistry;
use crate::core::nt_core_meta::knowledge_gap_detector::{GapReport, KnowledgeGap, GapCategory};

/// Gap-to-spec mapping strategy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpawnStrategy {
    /// Spawn one sub-hive per high-priority gap
    PerGap,
    /// Spawn one sub-hive per gap cluster
    PerCluster,
    /// Spawn one sub-hive per gap category
    PerCategory,
}

/// Controller that analyzes knowledge gaps and spawns sub-hives to fill them.
///
/// Implements the AOrchestra 4-tuple pattern:
///   (instruction, context, tools, model) → on-demand sub-agent creation
///
/// Pipeline: gap analysis → 4-tuple generation → SubHiveSpec → SubHiveRegistry.spawn()
/// Optionally dispatches via A2A AgentCard to remote sub-hives.
pub struct SpawnController {
    registry: SubHiveRegistry,
    strategy: SpawnStrategy,
    max_concurrent_spawns: usize,
    active_spawns: u64,
    total_spawned: u64,
    domain_coverage: std::collections::HashMap<String, usize>,
    min_priority_threshold: f64,
}

impl SpawnController {
    pub fn new(
        registry: SubHiveRegistry,
    ) -> Self {
        SpawnController {
            registry,
            strategy: SpawnStrategy::PerCluster,
            max_concurrent_spawns: 5,
            active_spawns: 0,
            total_spawned: 0,
            domain_coverage: std::collections::HashMap::new(),
            min_priority_threshold: 0.6,
        }
    }

    /// Set the spawn strategy.
    pub fn with_strategy(mut self, strategy: SpawnStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set the minimum exploration priority threshold.
    pub fn with_priority_threshold(mut self, threshold: f64) -> Self {
        self.min_priority_threshold = threshold;
        self
    }

    /// Set max concurrent spawns.
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent_spawns = max;
        self
    }

    /// Analyze gap report and spawn sub-hives to fill gaps.
    /// Returns the list of spawned HiveIds.
    pub fn analyze_and_spawn(&mut self, report: &GapReport) -> Vec<HiveId> {
        let specs = self.gaps_to_specs(report);
        let mut spawned = Vec::new();
        for spec in specs {
            if self.active_spawns >= self.max_concurrent_spawns as u64 {
                break;
            }
            let id = self.spawn_for_spec(spec);
            spawned.push(id);
        }
        spawned
    }

    /// Generate SubHiveSpecs from a GapReport following the chosen strategy.
    pub fn gaps_to_specs(&self, report: &GapReport) -> Vec<SubHiveSpec> {
        match self.strategy {
            SpawnStrategy::PerGap => {
                let gaps: Vec<&KnowledgeGap> = report
                    .gaps
                    .iter()
                    .filter(|g| g.exploration_priority >= self.min_priority_threshold)
                    .collect();
                gaps.into_iter().map(|g| self.spec_for_gap(g)).collect()
            }
            SpawnStrategy::PerCluster => report
                .clusters
                .iter()
                .filter(|c| {
                    let avg_priority = c.gaps.iter().map(|g| g.exploration_priority).sum::<f64>()
                        / c.gaps.len().max(1) as f64;
                    avg_priority >= self.min_priority_threshold
                })
                .map(|c| self.spec_for_cluster(c))
                .collect(),
            SpawnStrategy::PerCategory => {
                let mut seen = std::collections::HashSet::new();
                let mut specs = Vec::new();
                for gap in &report.gaps {
                    if gap.exploration_priority < self.min_priority_threshold {
                        continue;
                    }
                    if seen.insert(gap.category.label().to_string()) {
                        specs.push(self.spec_for_gap(gap));
                    }
                }
                specs
            }
        }
    }

    /// Convert a single KnowledgeGap into an AOrchestra 4-tuple SubHiveSpec.
    pub fn spec_for_gap(&self, gap: &KnowledgeGap) -> SubHiveSpec {
        let domain = self.gap_to_domain(gap);
        let instruction = format!(
            "Research and fill knowledge gap: {}. Priority: {:.2}. Strategy: {}",
            gap.description, gap.exploration_priority, gap.fill_strategy
        );

        SubHiveSpec::new(&instruction, &domain)
            .with_context(vec![
                format!("gap_category: {}", gap.category.label()),
                format!("severity: {:.2}", gap.severity),
                format!("fill_strategy: {}", gap.fill_strategy),
            ])
            .with_tools(gap.suggested_sources.clone())
            .with_budget((gap.exploration_priority * 500.0) as u64 + 100)
    }

    fn spec_for_cluster(&self, cluster: &crate::core::nt_core_meta::knowledge_gap_detector::GapCluster) -> SubHiveSpec {
        let domain = format!("cluster-{}", cluster.id);
        let avg_priority = cluster.gaps.iter().map(|g| g.exploration_priority).sum::<f64>()
            / cluster.gaps.len().max(1) as f64;
        let descriptions: Vec<&str> = cluster.gaps.iter().map(|g| g.description.as_str()).collect();

        SubHiveSpec::new(
            &format!(
                "Explore cluster '{}' (route: {}). {}",
                cluster.centroid_description, cluster.exploration_route, descriptions.join("; ")
            ),
            &domain,
        )
            .with_context(vec![format!("category: {:?}", cluster.category)])
            .with_budget((avg_priority * 800.0) as u64 + 200)
    }

    /// Dispatch a sub-hive via the registry.
    pub fn spawn_for_spec(&mut self, spec: SubHiveSpec) -> HiveId {
        let domain = spec.domain.clone();
        let id = self.registry.spawn(spec, None);
        self.active_spawns += 1;
        self.total_spawned += 1;
        *self.domain_coverage.entry(domain).or_insert(0) += 1;
        id
    }

    /// Update active spawn count (called when a sub-hive dies).
    pub fn on_sub_hive_destroyed(&mut self) {
        self.active_spawns = self.active_spawns.saturating_sub(1);
    }

    /// Check which domains have been covered.
    pub fn coverage(&self) -> &std::collections::HashMap<String, usize> {
        &self.domain_coverage
    }

    pub fn total_spawned(&self) -> u64 {
        self.total_spawned
    }

    pub fn active_spawns(&self) -> u64 {
        self.active_spawns
    }

    fn gap_to_domain(&self, gap: &KnowledgeGap) -> String {
        match gap.category {
            GapCategory::MissingModule => format!("gap-module-{}", gap.id),
            GapCategory::MissingApi => format!("gap-api-{}", gap.id),
            GapCategory::MissingKnowledgeSource => format!("gap-source-{}", gap.id),
            GapCategory::MissingRelationship => format!("gap-relation-{}", gap.id),
            GapCategory::LowCoverage => format!("gap-coverage-{}", gap.id),
            GapCategory::OutdatedPattern => format!("gap-pattern-{}", gap.id),
            GapCategory::MissingIntegration => format!("gap-integration-{}", gap.id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_meta::knowledge_gap_detector::{
        GapReport, KnowledgeGap, GapCategory,
    };

    fn sample_report() -> GapReport {
        GapReport {
            gaps: vec![
                KnowledgeGap {
                    id: 1,
                    category: GapCategory::MissingModule,
                    description: "No world model predictor module".into(),
                    affected_modules: vec!["nt_mind".into()],
                    severity: 0.85,
                    exploration_priority: 0.9,
                    fill_strategy: "Implement RSSM-style predictor".into(),
                    suggested_sources: vec!["dreamerv3".into()],
                },
                KnowledgeGap {
                    id: 2,
                    category: GapCategory::MissingIntegration,
                    description: "thinking_model missing background_loop wire".into(),
                    affected_modules: vec!["thinking_model".into()],
                    severity: 0.7,
                    exploration_priority: 0.75,
                    fill_strategy: "Wire background_loop to thinking_model".into(),
                    suggested_sources: vec!["design_pattern".into()],
                },
            ],
            clusters: vec![],
            total_gaps: 2,
            high_priority_count: 1,
            exploration_suggestions: vec!["[HIGH] implement world model".into()],
            coherence_score: 0.8,
        }
    }

    fn controller() -> SpawnController {
        SpawnController::new(
            SubHiveRegistry::new(),
        )
    }

    #[test]
    fn test_gaps_to_specs_per_gap() {
        let mut c = controller();
        c.strategy = SpawnStrategy::PerGap;
        let report = sample_report();
        let specs = c.gaps_to_specs(&report);
        assert_eq!(specs.len(), 2, "should create one spec per gap");
        assert!(specs[0].domain.contains("gap-module"));
        assert!(specs[1].domain.contains("gap-integration"));
    }

    #[test]
    fn test_spec_contains_instruction() {
        let c = controller();
        let report = sample_report();
        let specs = c.gaps_to_specs(&report);
        let spec = &specs[0];
        assert!(spec.instruction.contains("knowledge gap"));
        assert!(spec.instruction.contains("No world model"));
    }

    #[test]
    fn test_spawn_for_spec_increases_count() {
        let mut c = controller();
        let report = sample_report();
        let specs = c.gaps_to_specs(&report);
        let initial = c.total_spawned();
        for spec in specs {
            c.spawn_for_spec(spec);
        }
        assert_eq!(c.total_spawned(), initial + 2);
    }

    #[test]
    fn test_priority_filter_respected() {
        let mut c = controller();
        c.strategy = SpawnStrategy::PerGap;
        c.min_priority_threshold = 0.8;
        let report = sample_report();
        let specs = c.gaps_to_specs(&report);
        assert_eq!(specs.len(), 1, "only gap with priority >= 0.8 should pass");
    }

    #[test]
    fn test_max_concurrent_respected() {
        let mut c = controller();
        c.max_concurrent_spawns = 1;
        let report = sample_report();
        let _specs = c.gaps_to_specs(&report);
        let spawned = c.analyze_and_spawn(&sample_report());
        assert!(spawned.len() <= 1);
    }

    #[test]
    fn test_strategy_per_category_dedup() {
        let mut c = controller();
        c.strategy = SpawnStrategy::PerCategory;
        let report = sample_report();
        let specs = c.gaps_to_specs(&report);
        assert_eq!(specs.len(), 2, "two distinct categories → two specs");
    }

    #[test]
    fn test_on_destroyed_decrements() {
        let mut c = controller();
        c.active_spawns = 5;
        c.on_sub_hive_destroyed();
        assert_eq!(c.active_spawns(), 4);
    }

    #[test]
    fn test_coverage_tracking() {
        let mut c = controller();
        let report = sample_report();
        c.analyze_and_spawn(&report);
        assert!(!c.coverage().is_empty());
    }
}
