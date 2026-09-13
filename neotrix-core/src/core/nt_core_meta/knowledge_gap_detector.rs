use std::collections::HashMap;
use std::sync::Arc;

use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;

use super::self_model::SelfModel;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum GapCategory {
    #[default]
    MissingModule,
    MissingApi,
    MissingKnowledgeSource,
    MissingRelationship,
    LowCoverage,
    OutdatedPattern,
    MissingIntegration,
    MissingAbstractionLayer,
    WeakConnectivity,
    StaleKnowledge,
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
            GapCategory::MissingAbstractionLayer => "missing_abstraction_layer",
            GapCategory::WeakConnectivity => "weak_connectivity",
            GapCategory::StaleKnowledge => "stale_knowledge",
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
            GapCategory::MissingAbstractionLayer => 0.7,
            GapCategory::WeakConnectivity => 0.5,
            GapCategory::StaleKnowledge => 0.6,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGap {
    pub id: usize,
    pub category: GapCategory,
    pub description: String,
    pub affected_modules: Vec<String>,
    pub severity: f64,
    pub exploration_priority: f64,
    pub fill_strategy: String,
    pub suggested_sources: Vec<String>,
    // KB integration fields
    pub kb_node_ids: Vec<String>,           // Related KB node IDs
    pub abstraction_level: Option<String>,  // Abstraction level from node_dimensions
    pub scale_indicators: Vec<String>,      // Scale indicators from node_dimensions
    pub concern_tags: Vec<String>,          // Concern tags from node_dimensions
    pub temporal_order: Option<i64>,        // Temporal order from node_dimensions
    pub source_gap_report_id: Option<String>, // Link to knowledge_gap_reports table
}

impl Default for KnowledgeGap {
    fn default() -> Self {
        Self {
            id: 0,
            category: GapCategory::default(),
            description: String::new(),
            affected_modules: Vec::new(),
            severity: 0.0,
            exploration_priority: 0.0,
            fill_strategy: String::new(),
            suggested_sources: Vec::new(),
            kb_node_ids: Vec::new(),
            abstraction_level: None,
            scale_indicators: Vec::new(),
            concern_tags: Vec::new(),
            temporal_order: None,
            source_gap_report_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConnectivityStats {
    pub avg_degree: f64,
    pub isolated_nodes: usize,
    pub max_degree: usize,
    pub weak_components: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GapCluster {
    pub id: usize,
    pub category: GapCategory,
    pub gaps: Vec<KnowledgeGap>,
    pub centroid_description: String,
    pub exploration_route: String,
    pub avg_severity: f64,
    pub kb_related_nodes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GapReport {
    pub gaps: Vec<KnowledgeGap>,
    pub clusters: Vec<GapCluster>,
    pub total_gaps: usize,
    pub high_priority_count: usize,
    pub exploration_suggestions: Vec<String>,
    pub coherence_score: f64,
    // KB integration
    pub kb_gap_report_ids: Vec<String>,  // IDs of reports written to knowledge_gap_reports table
    pub abstraction_coverage: HashMap<String, usize>, // abstraction level -> count
    pub connectivity_stats: ConnectivityStats,
}

/// Scans the codebase AND knowledge base for knowledge gaps — missing modules, APIs, patterns, abstraction layers, connectivity.
pub struct KnowledgeGapDetector {
    pub known_sources: Vec<String>,
    pub target_categories: Vec<GapCategory>,
    pub min_severity_threshold: f64,
    // KB integration
    pub kb: Option<Arc<KnowledgeBase>>,
    pub min_connectivity_threshold: f64,
    pub max_temporal_age_days: i64,
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
                GapCategory::MissingAbstractionLayer,
                GapCategory::WeakConnectivity,
                GapCategory::StaleKnowledge,
            ],
            min_severity_threshold: 0.3,
            kb: None,
            min_connectivity_threshold: 5.0,
            max_temporal_age_days: 365,
        }
    }

    pub fn with_kb(mut self, kb: Arc<KnowledgeBase>) -> Self {
        self.kb = Some(kb);
        self
    }

    pub fn with_connectivity_threshold(mut self, threshold: f64) -> Self {
        self.min_connectivity_threshold = threshold;
        self
    }

    pub fn with_temporal_age(mut self, days: i64) -> Self {
        self.max_temporal_age_days = days;
        self
    }

    pub fn add_source(&mut self, source: &str) {
        if !self.known_sources.contains(&source.to_string()) {
            self.known_sources.push(source.to_string());
        }
    }

    /// Full gap detection run: scan codebase → scan KB → cluster → suggest
    pub fn detect_gaps(
        &self,
        model: &SelfModel,
        weaknesses: &[super::weakness::Weakness],
    ) -> GapReport {
        let mut gaps = self.scan_all_gaps(model, weaknesses);

        // KB-based gap detection
        if let Some(ref kb) = self.kb {
            gaps.extend(self.scan_kb_gaps(kb));
        }

        let clusters = self.cluster_gaps(&gaps);
        let suggestions = self.generate_exploration_suggestions(&gaps, &clusters);
        let coherence = self.calculate_coherence(&gaps, &clusters);

        // Compute KB stats
        let (abstraction_coverage, connectivity_stats) = if let Some(ref kb) = self.kb {
            (self.compute_abstraction_coverage(kb), self.compute_connectivity_stats(kb))
        } else {
            (HashMap::new(), ConnectivityStats::default())
        };

        // Write new gap reports to KB
        let kb_gap_report_ids = if let Some(ref kb) = self.kb {
            self.write_gap_reports_to_kb(kb, &gaps)
        } else {
            Vec::new()
        };

        GapReport {
            total_gaps: gaps.len(),
            high_priority_count: gaps.iter().filter(|g| g.exploration_priority > 0.7).count(),
            exploration_suggestions: suggestions,
            coherence_score: coherence,
            gaps,
            clusters,
            kb_gap_report_ids,
            abstraction_coverage,
            connectivity_stats,
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
                            ..Default::default()
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
                        ..Default::default()
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
                            ..Default::default()
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
                            ..Default::default()
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
                ..Default::default()
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
                    ..Default::default()
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
                GapCategory::MissingAbstractionLayer => "abstract+implement",
                GapCategory::WeakConnectivity => "connect+integrate",
                GapCategory::StaleKnowledge => "refresh+reabsorb",
            };

            let avg_sev = cluster_members.iter().map(|g| g.severity).sum::<f64>() / cluster_members.len() as f64;
            let kb_related = cluster_members.iter().map(|g| g.kb_node_ids.len()).sum::<usize>();

            clusters.push(GapCluster {
                id: clusters.len(),
                category: gap.category.clone(),
                centroid_description: centroid,
                gaps: cluster_members,
                exploration_route: route.to_string(),
                avg_severity: avg_sev,
                kb_related_nodes: kb_related,
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

    // ===== KB Integration Methods =====

    /// Scan KB for knowledge gaps: missing abstraction layers, weak connectivity, stale knowledge
    fn scan_kb_gaps(&self, kb: &KnowledgeBase) -> Vec<KnowledgeGap> {
        let mut gaps = Vec::new();
        let mut id = 0;

        // 1. Load existing gap reports to avoid duplicates
        let existing_reports = self.load_existing_gap_reports(kb);

        // 2. Scan for missing abstraction layers
        gaps.extend(self.scan_abstraction_gaps(kb, &mut id));

        // 3. Scan for weak connectivity
        gaps.extend(self.scan_connectivity_gaps(kb, &mut id));

        // 4. Scan for stale knowledge
        gaps.extend(self.scan_stale_knowledge(kb, &mut id));

        // Filter out gaps that already have reports
        gaps.retain(|g| {
            !existing_reports.iter().any(|r| r.description == g.description)
        });

        gaps
    }

    /// Load existing gap reports from KB to avoid duplicates
    fn load_existing_gap_reports(&self, kb: &KnowledgeBase) -> Vec<KnowledgeGap> {
        let conn = match kb.conn.lock() {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let mut stmt = match conn.prepare(
            "SELECT description FROM knowledge_gap_reports WHERE status != 'resolved'"
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        let rows = match stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        }) {
            Ok(r) => r,
            Err(_) => return vec![],
        };

        rows.filter_map(|r| r.ok())
            .map(|desc| KnowledgeGap {
                id: 0,
                category: GapCategory::MissingRelationship,
                description: desc,
                affected_modules: vec![],
                severity: 0.0,
                exploration_priority: 0.0,
                fill_strategy: String::new(),
                suggested_sources: vec![],
                kb_node_ids: vec![],
                abstraction_level: None,
                scale_indicators: vec![],
                concern_tags: vec![],
                temporal_order: None,
                source_gap_report_id: None,
            })
        .collect()
    }

    /// Scan for missing abstraction layers in KB (building_block → pattern → architecture → case_study)
    fn scan_abstraction_gaps(&self, kb: &KnowledgeBase, id: &mut usize) -> Vec<KnowledgeGap> {
        let conn = match kb.conn.lock() {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let mut gaps = Vec::new();

        // Query abstraction level distribution per domain
        let mut stmt = match conn.prepare(
            "SELECT json_extract(metadata, '$.categories') as domain, 
                    json_extract(metadata, '$.abstraction') as abstraction, 
                    COUNT(*) as count
             FROM nodes n
             JOIN node_dimensions nd ON n.id = nd.node_id
             WHERE n.url LIKE 'bytebytego://%' OR n.url LIKE 'easytier://%'
             GROUP BY domain, abstraction"
        ) {
            Ok(s) => s,
            Err(_) => return gaps,
        };

        let rows = match stmt.query_map([], |row| {
            let domain: String = row.get(0)?;
            let abstraction: String = row.get(1)?;
            let count: usize = row.get(2)?;
            Ok((domain, abstraction, count))
        }) {
            Ok(r) => r,
            Err(_) => return gaps,
        };

        let mut domain_abstractions: HashMap<String, HashMap<String, usize>> = HashMap::new();
        for row in rows.flatten() {
            let (domain, abstraction, count) = row;
            domain_abstractions.entry(domain).or_default().insert(abstraction, count);
        }

        let levels = vec!["building_block", "pattern", "architecture", "case_study"];
        for (domain, abs_map) in domain_abstractions {
            for i in 0..levels.len()-1 {
                let current = levels[i];
                let next = levels[i+1];
                let current_count = abs_map.get(current).unwrap_or(&0);
                let next_count = abs_map.get(next).unwrap_or(&0);
                
                // If current level has nodes but next level has very few or zero
                if *current_count > 5 && *next_count < *current_count / 3 {
                    *id += 1;
                    gaps.push(KnowledgeGap {
                        id: *id,
                        category: GapCategory::MissingAbstractionLayer,
                        description: format!(
                            "Domain '{}' has {} {} nodes but only {} {} nodes — missing abstraction layer",
                            domain, current_count, current, next_count, next
                        ),
                        affected_modules: vec![domain.clone()],
                        severity: 0.7,
                        exploration_priority: 0.75,
                        fill_strategy: format!(
                            "Create {} synthesis nodes bridging {} to {} in {}",
                            next, current, next, domain
                        ),
                        suggested_sources: vec!["synthesis".to_string(), "abstraction".to_string()],
                        kb_node_ids: vec![],
                        abstraction_level: Some(current.to_string()),
                        scale_indicators: vec![],
                        concern_tags: vec![],
                        temporal_order: None,
                        source_gap_report_id: None,
                    });
                }
            }
        }

        gaps
    }

    /// Scan for weak connectivity in KB graph
    fn scan_connectivity_gaps(&self, kb: &KnowledgeBase, id: &mut usize) -> Vec<KnowledgeGap> {
        let conn = kb.conn.lock().unwrap_or_else(|e| e.into_inner());
        let mut gaps = Vec::new();

        // Find concepts with degree < threshold
        let mut stmt = conn.prepare(
            "SELECT n.id, n.title, COUNT(e.id) as degree
             FROM nodes n
             LEFT JOIN edges e ON e.source_id = n.id OR e.target_id = n.id
             WHERE n.node_type = 'concept'
             GROUP BY n.id
             HAVING degree < ?"
        ).expect("SQL prepare");

        let rows = stmt.query_map([self.min_connectivity_threshold as usize], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let degree: usize = row.get(2)?;
            Ok((id, title, degree))
        }).expect("SQL query");

        for row in rows.flatten() {
            let (nid, title, degree) = row;
            *id += 1;
            gaps.push(KnowledgeGap {
                id: *id,
                category: GapCategory::WeakConnectivity,
                description: format!(
                    "Concept '{}' has only {} connections (threshold: {})",
                    title, degree, self.min_connectivity_threshold
                ),
                affected_modules: vec!["knowledge_graph".to_string()],
                severity: 0.5,
                exploration_priority: 0.6,
                fill_strategy: format!(
                    "Add edges from '{}' to related concepts via analogy/co-occurrence",
                    title
                ),
                suggested_sources: vec!["connectivity_analysis".to_string()],
                kb_node_ids: vec![nid],
                abstraction_level: None,
                scale_indicators: vec![],
                concern_tags: vec![],
                temporal_order: None,
                source_gap_report_id: None,
            });
        }

        gaps
    }

    /// Scan for stale knowledge (nodes not updated in max_temporal_age_days)
    fn scan_stale_knowledge(&self, kb: &KnowledgeBase, id: &mut usize) -> Vec<KnowledgeGap> {
        let conn = kb.conn.lock().unwrap_or_else(|e| e.into_inner());
        let mut gaps = Vec::new();

        let cutoff = chrono::Utc::now().timestamp() - (self.max_temporal_age_days * 86400);
        let mut stmt = conn.prepare(
            "SELECT n.id, n.title, nd.updated_at
             FROM nodes n
             JOIN node_dimensions nd ON n.id = nd.node_id
             WHERE nd.updated_at < ? AND n.node_type IN ('article', 'concept', 'insight')
              LIMIT 50"
        ).expect("SQL prepare");

        let rows = stmt.query_map([cutoff], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let updated: i64 = row.get(2)?;
            Ok((id, title, updated))
        }).expect("SQL query");

        for row in rows.flatten() {
            let (nid, title, updated) = row;
            let age_days = (chrono::Utc::now().timestamp() - updated) / 86400;
            *id += 1;
            gaps.push(KnowledgeGap {
                id: *id,
                category: GapCategory::StaleKnowledge,
                description: format!(
                    "Node '{}' not updated for {} days (last: {})",
                    title, age_days, chrono::DateTime::from_timestamp(updated, 0).expect("valid timestamp").format("%Y-%m-%d")
                ),
                affected_modules: vec!["knowledge_freshness".to_string()],
                severity: 0.6,
                exploration_priority: 0.55,
                fill_strategy: format!(
                    "Review and update '{}' with current knowledge", title
                ),
                suggested_sources: vec!["freshness_check".to_string()],
                kb_node_ids: vec![nid],
                abstraction_level: None,
                scale_indicators: vec![],
                concern_tags: vec![],
                temporal_order: Some(updated),
                source_gap_report_id: None,
            });
        }

        gaps
    }

    /// Compute abstraction level coverage per domain
    fn compute_abstraction_coverage(&self, kb: &KnowledgeBase) -> HashMap<String, usize> {
        let conn = kb.conn.lock().unwrap_or_else(|e| e.into_inner());
        let mut coverage = HashMap::new();

        let mut stmt = conn.prepare(
            "SELECT json_extract(metadata, '$.abstraction') as abstraction, COUNT(*) as count
             FROM node_dimensions
             WHERE abstraction IS NOT NULL
             GROUP BY abstraction"
        ).expect("SQL operation");

        let rows = stmt.query_map([], |row| {
            let abstraction: String = row.get(0)?;
            let count: usize = row.get(1)?;
            Ok((abstraction, count))
        }).expect("SQL query");

        for row in rows.flatten() {
            coverage.insert(row.0, row.1);
        }

        coverage
    }

    /// Compute connectivity statistics from KB graph
    fn compute_connectivity_stats(&self, kb: &KnowledgeBase) -> ConnectivityStats {
        let conn = kb.conn.lock().unwrap_or_else(|e| e.into_inner());

        let mut stmt = conn.prepare(
            "SELECT AVG(degree) as avg_deg,
                    COUNT(CASE WHEN degree = 0 THEN 1 END) as isolated,
                    MAX(degree) as max_deg
             FROM (
                 SELECT n.id, COUNT(e.id) as degree
                 FROM nodes n
                 LEFT JOIN edges e ON e.source_id = n.id OR e.target_id = n.id
                 GROUP BY n.id
             )"
        ).expect("SQL prepare");

        let stats = stmt.query_row([], |row| {
            Ok(ConnectivityStats {
                avg_degree: row.get(0)?,
                isolated_nodes: row.get(1)?,
                max_degree: row.get(2)?,
                weak_components: 0, // Would need connected components algorithm
            })
        }).unwrap_or_default();

        stats
    }

    /// Write new gap reports to KB knowledge_gap_reports table
    fn write_gap_reports_to_kb(&self, kb: &KnowledgeBase, gaps: &[KnowledgeGap]) -> Vec<String> {
        let conn = kb.conn.lock().unwrap_or_else(|e| e.into_inner());
        let mut report_ids = Vec::new();
        let now = chrono::Utc::now().timestamp();

        for gap in gaps {
            // Skip if already has a source report ID
            if gap.source_gap_report_id.is_some() {
                continue;
            }

            let report_id = Uuid::new_v4().to_string();
            let domain = gap.affected_modules.first().cloned().unwrap_or_else(|| "system-design".to_string());

            conn.execute(
                "INSERT OR IGNORE INTO knowledge_gap_reports 
                 (id, domain, gap_type, description, severity, suggested_actions, related_nodes, created_at, status, resolved_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'pending', NULL)",
                params![
                    report_id,
                    domain,
                    gap.category.label(),
                    gap.description,
                    gap.severity,
                    serde_json::to_string(&gap.suggested_sources).unwrap_or_default(),
                    serde_json::to_string(&gap.kb_node_ids).unwrap_or_default(),
                    now,
                ]
            ).expect("SQL insert");

            report_ids.push(report_id);
        }

        report_ids
    }
}

impl crate::core::nt_core_self_test::SelfTest for KnowledgeGapDetector {
    fn name(&self) -> &str {
        "knowledge_gap_detector"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();

        if self.known_sources.is_empty() {
            failures.push("known_sources is empty".into());
        }
        if self.target_categories.is_empty() {
            failures.push("target_categories is empty".into());
        }
        if self.min_severity_threshold <= 0.0 {
            failures.push("min_severity_threshold must be > 0.0".into());
        }
        if !self
            .target_categories
            .contains(&super::GapCategory::MissingModule)
        {
            failures.push("MissingModule must be in target_categories".into());
        }
        if !self
            .target_categories
            .contains(&super::GapCategory::MissingKnowledgeSource)
        {
            failures.push("MissingKnowledgeSource must be in target_categories".into());
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
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
