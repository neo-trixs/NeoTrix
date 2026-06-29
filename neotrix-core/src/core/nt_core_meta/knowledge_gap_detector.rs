use super::self_model::SelfModel;

#[derive(Debug, Clone, PartialEq)]
pub enum GapCategory {
    MissingModule,
    MissingApi,
    MissingKnowledgeSource,
    MissingRelationship,
    LowCoverage,
    OutdatedPattern,
    MissingIntegration,
}

impl GapCategory {
    pub fn label(&self) -> &str {
        match self {
            GapCategory::MissingModule => "missing_module",
            GapCategory::MissingApi => "missing_api",
            GapCategory::MissingKnowledgeSource => "missing_knowledge_source",
            GapCategory::MissingRelationship => "missing_relationship",
            GapCategory::LowCoverage => "low_coverage",
            GapCategory::OutdatedPattern => "outdated_pattern",
            GapCategory::MissingIntegration => "missing_integration",
        }
    }

    pub fn severity(&self) -> f64 {
        match self {
            GapCategory::MissingModule => 0.9,
            GapCategory::MissingApi => 0.7,
            GapCategory::MissingKnowledgeSource => 0.6,
            GapCategory::MissingRelationship => 0.5,
            GapCategory::LowCoverage => 0.4,
            GapCategory::OutdatedPattern => 0.3,
            GapCategory::MissingIntegration => 0.8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KnowledgeGap {
    pub id: usize,
    pub category: GapCategory,
    pub description: String,
    pub affected_modules: Vec<String>,
    pub severity: f64,
    pub exploration_priority: f64,
    pub fill_strategy: String,
    pub suggested_sources: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GapCluster {
    pub id: usize,
    pub category: GapCategory,
    pub gaps: Vec<KnowledgeGap>,
    pub centroid_description: String,
    pub exploration_route: String,
}

#[derive(Debug, Clone)]
pub struct GapReport {
    pub gaps: Vec<KnowledgeGap>,
    pub clusters: Vec<GapCluster>,
    pub total_gaps: usize,
    pub high_priority_count: usize,
    pub exploration_suggestions: Vec<String>,
    pub coherence_score: f64,
}

/// Scans the codebase for knowledge gaps — missing modules, APIs, patterns.
pub struct KnowledgeGapDetector {
    pub known_sources: Vec<String>,
    pub target_categories: Vec<GapCategory>,
    pub min_severity_threshold: f64,
}

impl Default for KnowledgeGapDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeGapDetector {
    pub fn new() -> Self {
        Self {
            known_sources: vec![
                "dreamerv3".to_string(),
                "jepa".to_string(),
                "mirror".to_string(),
                "camoufox".to_string(),
                "cua".to_string(),
                "playwright".to_string(),
                "opencode".to_string(),
                "mcp".to_string(),
            ],
            target_categories: vec![
                GapCategory::MissingModule,
                GapCategory::MissingApi,
                GapCategory::MissingKnowledgeSource,
                GapCategory::MissingIntegration,
            ],
            min_severity_threshold: 0.3,
        }
    }

    pub fn add_source(&mut self, source: &str) {
        if !self.known_sources.contains(&source.to_string()) {
            self.known_sources.push(source.to_string());
        }
    }

    /// Full gap detection run: scan → cluster → suggest
    pub fn detect_gaps(
        &self,
        model: &SelfModel,
        weaknesses: &[super::weakness::Weakness],
    ) -> GapReport {
        let gaps = self.scan_all_gaps(model, weaknesses);
        let clusters = self.cluster_gaps(&gaps);
        let suggestions = self.generate_exploration_suggestions(&gaps, &clusters);
        let coherence = self.calculate_coherence(&gaps, &clusters);

        GapReport {
            total_gaps: gaps.len(),
            high_priority_count: gaps.iter().filter(|g| g.exploration_priority > 0.7).count(),
            exploration_suggestions: suggestions,
            coherence_score: coherence,
            gaps,
            clusters,
        }
    }

    fn scan_all_gaps(
        &self,
        model: &SelfModel,
        weaknesses: &[super::weakness::Weakness],
    ) -> Vec<KnowledgeGap> {
        let mut gaps = Vec::new();
        let mut id = 0;

        for category in &self.target_categories {
            let found = self.scan_category(category, model, weaknesses, &mut id);
            gaps.extend(found);
        }

        let additional = self.scan_known_source_gaps(model, &mut id);
        gaps.extend(additional);

        let integration = self.scan_integration_gaps(model, &mut id);
        gaps.extend(integration);

        gaps.sort_by(|a, b| {
            b.exploration_priority
                .partial_cmp(&a.exploration_priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        gaps
    }

    fn scan_category(
        &self,
        category: &GapCategory,
        model: &SelfModel,
        _weaknesses: &[super::weakness::Weakness],
        id: &mut usize,
    ) -> Vec<KnowledgeGap> {
        let mut gaps = Vec::new();

        match category {
            GapCategory::MissingModule => {
                let expected = vec![
                    "intra_reflection",
                    "knowledge_gap_detector",
                    "world_model_predictor",
                    "stealth_manager",
                ];
                for name in expected {
                    let exists = model.modules.iter().any(|m| m.name.contains(name));
                    if !exists {
                        *id += 1;
                        gaps.push(KnowledgeGap {
                            id: *id,
                            category: GapCategory::MissingModule,
                            description: format!("Module '{}' not found in codebase", name),
                            affected_modules: vec!["core".to_string()],
                            severity: 0.85,
                            exploration_priority: 0.9,
                            fill_strategy: format!(
                                "Implement {} with JEPA/DreamerV3 patterns",
                                name
                            ),
                            suggested_sources: vec!["dreamerv3".to_string(), "jepa".to_string()],
                        });
                    }
                }

                let module_names: Vec<String> =
                    model.modules.iter().map(|m| m.name.clone()).collect();
                if !module_names.iter().any(|n| n.contains("predict")) {
                    *id += 1;
                    gaps.push(KnowledgeGap {
                        id: *id,
                        category: GapCategory::MissingModule,
                        description: "No prediction/world_model module found".to_string(),
                        affected_modules: vec!["nt_mind".to_string()],
                        severity: 0.8,
                        exploration_priority: 0.85,
                        fill_strategy: "Implement RSSM-style world model prediction".to_string(),
                        suggested_sources: vec!["dreamerv3".to_string(), "jepa".to_string()],
                    });
                }
            }
            GapCategory::LowCoverage => {
                for module in &model.modules {
                    if !module.has_tests && module.total_lines > 200 {
                        *id += 1;
                        gaps.push(KnowledgeGap {
                            id: *id,
                            category: GapCategory::LowCoverage,
                            description: format!(
                                "{} ({} lines) has no tests",
                                module.name, module.total_lines
                            ),
                            affected_modules: vec![module.name.clone()],
                            severity: 0.5,
                            exploration_priority: 0.6,
                            fill_strategy: format!(
                                "Add {} test stubs for {}",
                                module.test_count, module.name
                            ),
                            suggested_sources: vec!["standard".to_string()],
                        });
                    }
                }
            }
            GapCategory::MissingKnowledgeSource => {
                let current = self.known_sources.clone();
                let expected = vec![
                    "dreamerv3_rssm",
                    "jepa_vicreg",
                    "camoufox_stealth",
                    "cua_browser",
                    "mcp_tools",
                ];
                for name in expected {
                    if !current
                        .iter()
                        .any(|s| s.contains(&name[..name.len().min(6)]))
                    {
                        *id += 1;
                        gaps.push(KnowledgeGap {
                            id: *id,
                            category: GapCategory::MissingKnowledgeSource,
                            description: format!("Knowledge source '{}' not registered", name),
                            affected_modules: vec!["nt_mind".to_string()],
                            severity: 0.6,
                            exploration_priority: 0.7,
                            fill_strategy: format!(
                                "Register {} as KnowledgeSource with seed knowledge",
                                name
                            ),
                            suggested_sources: vec![name.to_string()],
                        });
                    }
                }
            }
            _ => {}
        }

        gaps
    }

    fn scan_known_source_gaps(&self, model: &SelfModel, id: &mut usize) -> Vec<KnowledgeGap> {
        let mut gaps = Vec::new();

        let known: Vec<String> = self
            .known_sources
            .iter()
            .flat_map(|s| model.modules.iter().map(move |m| (s, m)))
            .filter(|(s, m)| m.name.to_lowercase().contains(&s.to_lowercase()))
            .map(|(s, _)| s.clone())
            .collect();

        let missing: Vec<&String> = self
            .known_sources
            .iter()
            .filter(|s| !known.contains(s))
            .collect();

        for source in missing {
            *id += 1;
            gaps.push(KnowledgeGap {
                id: *id,
                category: GapCategory::MissingKnowledgeSource,
                description: format!("Knowledge source '{}' has no corresponding module", source),
                affected_modules: vec!["unknown".to_string()],
                severity: 0.5,
                exploration_priority: 0.5,
                fill_strategy: format!("Research and implement module for {}", source),
                suggested_sources: vec![source.to_string()],
            });
        }

        gaps
    }

    fn scan_integration_gaps(&self, model: &SelfModel, id: &mut usize) -> Vec<KnowledgeGap> {
        let mut gaps = Vec::new();
        let module_names: Vec<String> = model.modules.iter().map(|m| m.name.clone()).collect();

        let pairs = vec![
            ("thinking_model", "background_loop"),
            ("intra_reflection", "metacognition"),
            ("knowledge_gap", "exploration"),
            ("stealth_manager", "identity_rotator"),
        ];

        for (a, b) in pairs {
            let a_exists = module_names.iter().any(|n| n.contains(a));
            let b_exists = module_names.iter().any(|n| n.contains(b));
            if a_exists && !b_exists {
                *id += 1;
                gaps.push(KnowledgeGap {
                    id: *id,
                    category: GapCategory::MissingIntegration,
                    description: format!("{} exists but {} is missing — integration gap", a, b),
                    affected_modules: vec![a.to_string()],
                    severity: 0.7,
                    exploration_priority: 0.75,
                    fill_strategy: format!("Implement {} and wire to {}", b, a),
                    suggested_sources: vec!["design_pattern".to_string()],
                });
            }
        }

        gaps
    }

    fn cluster_gaps(&self, gaps: &[KnowledgeGap]) -> Vec<GapCluster> {
        let mut clusters: Vec<GapCluster> = Vec::new();
        let mut assigned: Vec<bool> = vec![false; gaps.len()];

        for (i, gap) in gaps.iter().enumerate() {
            if assigned[i] {
                continue;
            }

            let mut cluster_members = vec![gap.clone()];
            assigned[i] = true;

            for (j, other) in gaps.iter().enumerate() {
                if i != j && !assigned[j] && gap.category == other.category {
                    let topic_sim = self.topic_similarity(&gap.description, &other.description);
                    if topic_sim > 0.4 {
                        cluster_members.push(other.clone());
                        assigned[j] = true;
                    }
                }
            }

            let centroid = cluster_members
                .first()
                .map(|g| g.description.clone())
                .unwrap_or_default();

            let route = match gap.category {
                GapCategory::MissingModule => "implement",
                GapCategory::MissingKnowledgeSource => "research+register",
                GapCategory::MissingIntegration => "wire",
                GapCategory::LowCoverage => "test",
                GapCategory::MissingApi => "design+implement",
                GapCategory::OutdatedPattern => "refactor",
                GapCategory::MissingRelationship => "analyze",
            };

            clusters.push(GapCluster {
                id: clusters.len(),
                category: gap.category.clone(),
                centroid_description: centroid,
                gaps: cluster_members,
                exploration_route: route.to_string(),
            });
        }

        clusters.sort_by(|a, b| {
            let sa = a.gaps.iter().map(|g| g.severity).sum::<f64>() / a.gaps.len() as f64;
            let sb = b.gaps.iter().map(|g| g.severity).sum::<f64>() / b.gaps.len() as f64;
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });

        clusters
    }

    fn generate_exploration_suggestions(
        &self,
        gaps: &[KnowledgeGap],
        clusters: &[GapCluster],
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        for cluster in clusters {
            let avg_priority = cluster
                .gaps
                .iter()
                .map(|g| g.exploration_priority)
                .sum::<f64>()
                / cluster.gaps.len() as f64;
            if avg_priority > 0.7 {
                suggestions.push(format!(
                    "[HIGH] {} — {} ({:.1} avg priority): {}",
                    cluster.centroid_description,
                    cluster.exploration_route,
                    avg_priority,
                    cluster
                        .gaps
                        .iter()
                        .map(|g| g.fill_strategy.clone())
                        .collect::<Vec<_>>()
                        .join("; ")
                ));
            }
        }

        for gap in gaps.iter().filter(|g| g.exploration_priority > 0.8) {
            let already = suggestions.iter().any(|s| s.contains(&gap.description));
            if !already {
                suggestions.push(format!(
                    "[CRITICAL] {} → {}",
                    gap.description, gap.fill_strategy
                ));
            }
        }

        suggestions
    }

    fn calculate_coherence(&self, gaps: &[KnowledgeGap], clusters: &[GapCluster]) -> f64 {
        if gaps.is_empty() {
            return 1.0;
        }
        if clusters.is_empty() {
            return 0.0;
        }

        let unique_categories = clusters.len();
        let total = gaps.len();

        let cluster_efficiency = total as f64 / unique_categories.max(1) as f64;
        let optimal = 3.0;
        let score = 1.0 - (cluster_efficiency - optimal).abs() / optimal.max(cluster_efficiency);
        score.clamp(0.0, 1.0)
    }

    fn topic_similarity(&self, a: &str, b: &str) -> f64 {
        let words_a: Vec<&str> = a.split_whitespace().collect();
        let words_b: Vec<&str> = b.split_whitespace().collect();

        let common = words_a.iter().filter(|w| words_b.contains(w)).count();
        let max_len = words_a.len().max(words_b.len()).max(1);
        common as f64 / max_len as f64
    }

    pub fn exploration_plan(&self, report: &GapReport) -> Vec<String> {
        let mut plan = Vec::new();
        for cluster in &report.clusters {
            let priority = cluster
                .gaps
                .iter()
                .map(|g| g.exploration_priority)
                .sum::<f64>()
                / cluster.gaps.len() as f64;
            plan.push(format!(
                "[p={:.2}] {} → {} ({} gaps)",
                priority,
                cluster.centroid_description,
                cluster.exploration_route,
                cluster.gaps.len()
            ));
        }
        plan
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_meta::self_model::ModuleInfo;

    fn sample_model() -> SelfModel {
        let mut model = SelfModel::new();
        model.modules = vec![
            ModuleInfo {
                name: "thinking_model".to_string(),
                total_lines: 350,
                has_tests: false,
                ..Default::default()
            },
            ModuleInfo {
                name: "background_loop".to_string(),
                total_lines: 180,
                has_tests: true,
                ..Default::default()
            },
        ];
        model
    }

    #[test]
    fn test_detect_missing_modules() {
        let detector = KnowledgeGapDetector::new();
        let model = sample_model();
        let weaknesses = Vec::new();
        let report = detector.detect_gaps(&model, &weaknesses);

        let missing_mods: Vec<_> = report
            .gaps
            .iter()
            .filter(|g| g.category == GapCategory::MissingModule)
            .collect();
        assert!(!missing_mods.is_empty(), "should find missing modules");
    }

    #[test]
    fn test_cluster_gaps() {
        let detector = KnowledgeGapDetector::new();
        let model = sample_model();
        let weaknesses = Vec::new();
        let report = detector.detect_gaps(&model, &weaknesses);

        assert!(!report.clusters.is_empty(), "should produce clusters");
    }

    #[test]
    fn test_exploration_suggestions() {
        let detector = KnowledgeGapDetector::new();
        let model = sample_model();
        let weaknesses = Vec::new();
        let report = detector.detect_gaps(&model, &weaknesses);

        let suggestions = detector.exploration_plan(&report);
        assert!(
            !suggestions.is_empty(),
            "should produce an exploration plan"
        );
    }

    #[test]
    fn test_topic_similarity() {
        let detector = KnowledgeGapDetector::new();
        let sim = detector.topic_similarity("world model prediction", "world model module");
        assert!(sim > 0.4, "similar phrases should score > 0.4, got {}", sim);
    }

    #[test]
    fn test_coherence_score() {
        let detector = KnowledgeGapDetector::new();
        let model = sample_model();
        let weaknesses = Vec::new();
        let report = detector.detect_gaps(&model, &weaknesses);

        assert!((0.0..=1.0).contains(&report.coherence_score));
    }
}
