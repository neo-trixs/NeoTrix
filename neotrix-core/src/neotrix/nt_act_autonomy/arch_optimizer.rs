use crate::neotrix::nt_act_autonomy::awareness_monitor::AwarenessReport;

#[derive(Debug, Clone, PartialEq)]
pub enum ArchIssueType {
    LargeModule { file: String, lines: usize },
    CircularDependency { modules: Vec<String> },
    MissingAbstraction { description: String },
    BridgeBloat { file: String, lines: usize },
    GodFile { file: String, lines: usize },
}

#[derive(Debug, Clone)]
pub struct ArchSuggestion {
    pub id: String,
    pub issue_type: ArchIssueType,
    pub description: String,
    pub effort: EffortEstimate,
    pub impact: f64,
    pub auto_fixable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EffortEstimate {
    Quick,
    Moderate,
    Large,
    Epic,
}

#[derive(Debug, Clone)]
pub struct ArchOptimizationReport {
    pub suggestions: Vec<ArchSuggestion>,
    pub total_suggestions: u32,
    pub auto_fixable_count: u32,
    pub large_modules: Vec<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct SelfArchitectureOptimizer {
    large_module_threshold: usize,
    bridge_bloat_threshold: usize,
    max_suggestions: usize,
}

impl SelfArchitectureOptimizer {
    pub fn new() -> Self {
        Self {
            large_module_threshold: 800,
            bridge_bloat_threshold: 200,
            max_suggestions: 10,
        }
    }

    pub fn analyze(
        &self,
        files: &[(String, usize)],
        awareness: Option<&AwarenessReport>,
    ) -> ArchOptimizationReport {
        let mut suggestions = Vec::new();

        suggestions.extend(self.detect_large_modules(files));
        suggestions.extend(self.suggest_abstractions(awareness));

        if suggestions.len() > self.max_suggestions {
            suggestions.truncate(self.max_suggestions);
        }

        let auto_fixable_count = suggestions.iter().filter(|s| s.auto_fixable).count() as u32;
        let large_modules: Vec<String> = files
            .iter()
            .filter(|(_, lines)| *lines > self.large_module_threshold)
            .map(|(path, _)| path.clone())
            .collect();

        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        ArchOptimizationReport {
            total_suggestions: suggestions.len() as u32,
            auto_fixable_count,
            large_modules,
            suggestions,
            timestamp,
        }
    }

    fn detect_large_modules(&self, files: &[(String, usize)]) -> Vec<ArchSuggestion> {
        let mut suggestions = Vec::new();
        let mut suggestion_id = 0u32;

        for (file_path, line_count) in files {
            if *line_count > self.large_module_threshold {
                suggestion_id += 1;
                suggestions.push(ArchSuggestion {
                    id: format!("LARGE-{:03}", suggestion_id),
                    issue_type: ArchIssueType::LargeModule {
                        file: file_path.clone(),
                        lines: *line_count,
                    },
                    description: format!(
                        "Module '{}' has {} lines (threshold: {}). Consider splitting into submodules.",
                        file_path, line_count, self.large_module_threshold,
                    ),
                    effort: if *line_count > self.large_module_threshold * 2 {
                        EffortEstimate::Large
                    } else {
                        EffortEstimate::Moderate
                    },
                    impact: if *line_count > self.large_module_threshold * 3 {
                        0.9
                    } else {
                        0.6
                    },
                    auto_fixable: false,
                });
            }

            if *line_count > self.bridge_bloat_threshold
                && *line_count <= self.large_module_threshold
                && file_path.contains("bridge")
            {
                suggestion_id += 1;
                suggestions.push(ArchSuggestion {
                    id: format!("BRIDGE-{:03}", suggestion_id),
                    issue_type: ArchIssueType::BridgeBloat {
                        file: file_path.clone(),
                        lines: *line_count,
                    },
                    description: format!(
                        "Bridge file '{}' has {} lines (threshold: {}). Consider merging with target or inlining delegates.",
                        file_path, line_count, self.bridge_bloat_threshold,
                    ),
                    effort: EffortEstimate::Quick,
                    impact: 0.4,
                    auto_fixable: true,
                });
            }
        }

        suggestions
    }

    fn suggest_abstractions(&self, awareness: Option<&AwarenessReport>) -> Vec<ArchSuggestion> {
        let mut suggestions = Vec::new();

        let report = match awareness {
            Some(r) => r,
            None => return suggestions,
        };

        for focus_dim in &report.recommended_focus {
            let dim_lower = focus_dim.to_lowercase();
            let (description, effort, impact) = if dim_lower.contains("compound_composition")
                || dim_lower.contains("design")
            {
                (
                    "Missing abstraction layer for UI composition. Consider extracting a compound component factory.",
                    EffortEstimate::Moderate,
                    0.7,
                )
            } else if dim_lower.contains("tailwind") || dim_lower.contains("styling") {
                (
                    "Styling concerns mixed with logic. Consider extracting a design token system or theme layer.",
                    EffortEstimate::Moderate,
                    0.6,
                )
            } else if dim_lower.contains("accessibility") || dim_lower.contains("aria") {
                (
                    "Accessibility logic scattered across components. Consider creating a centralized accessibility hook/utility module.",
                    EffortEstimate::Moderate,
                    0.7,
                )
            } else if dim_lower.contains("nt_shield") || dim_lower.contains("audit") {
                (
                    "Security auditing logic is ad-hoc. Consider extracting a dedicated nt_shield audit module.",
                    EffortEstimate::Large,
                    0.85,
                )
            } else if dim_lower.contains("testing") || dim_lower.contains("verification") {
                (
                    "Verification logic is tightly coupled with business logic. Consider extracting a test utility harness.",
                    EffortEstimate::Moderate,
                    0.5,
                )
            } else if dim_lower.contains("secret") || dim_lower.contains("detection") {
                (
                    "Secret detection logic is inline. Consider extracting a dedicated scanning pipeline module.",
                    EffortEstimate::Moderate,
                    0.75,
                )
            } else {
                continue;
            };

            suggestions.push(ArchSuggestion {
                id: format!("ABS-{:03}", suggestions.len() + 1),
                issue_type: ArchIssueType::MissingAbstraction {
                    description: description.to_string(),
                },
                description: description.to_string(),
                effort,
                impact,
                auto_fixable: false,
            });
        }

        suggestions
    }

    pub fn format_report(&self, report: &ArchOptimizationReport) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let age_secs = now.saturating_sub(report.timestamp);

        let header = format!(
            "Architecture Optimization Report\n\
             Total suggestions: {} | Auto-fixable: {} | {}s ago\n\
             Large modules: {} -> {:?}\n\n",
            report.total_suggestions,
            report.auto_fixable_count,
            age_secs,
            report.large_modules.len(),
            if report.large_modules.is_empty() {
                vec!["(none)".to_string()]
            } else {
                report.large_modules.clone()
            },
        );

        let mut body = String::from("Suggestions:\n");
        for (i, s) in report.suggestions.iter().enumerate() {
            body.push_str(&format!(
                "  {}. [{}] {} (effort={:?}, impact={:.2}, auto={})\n     {}\n",
                i + 1,
                s.id,
                match &s.issue_type {
                    ArchIssueType::LargeModule { file, lines } =>
                        format!("LargeModule({}, {} lines)", file, lines),
                    ArchIssueType::CircularDependency { modules } =>
                        format!("CircularDependency({:?})", modules),
                    ArchIssueType::MissingAbstraction { description } =>
                        format!("MissingAbstraction: {}", description),
                    ArchIssueType::BridgeBloat { file, lines } =>
                        format!("BridgeBloat({}, {} lines)", file, lines),
                    ArchIssueType::GodFile { file, lines } =>
                        format!("GodFile({}, {} lines)", file, lines),
                },
                s.effort,
                s.impact,
                s.auto_fixable,
                s.description,
            ));
        }

        if report.suggestions.is_empty() {
            body.push_str("  (no suggestions)\n");
        }

        format!("{}{}", header, body)
    }

    pub fn generate_fix_plan(&self, suggestion: &ArchSuggestion) -> Vec<String> {
        match &suggestion.issue_type {
            ArchIssueType::LargeModule { file, .. } => {
                vec![
                    format!("split_into_submodules({})", file),
                    format!("create_mod_rs({})", file),
                ]
            }
            ArchIssueType::GodFile { file, .. } => {
                vec![
                    format!("extract_responsibilities({})", file),
                    "create_separate_modules()".to_string(),
                ]
            }
            ArchIssueType::BridgeBloat { file, .. } => {
                vec![
                    format!("merge_with_target({})", file),
                    "inline_simple_delegates()".to_string(),
                ]
            }
            ArchIssueType::CircularDependency { modules } => {
                vec![
                    format!("extract_common_abstraction({:?})", modules),
                    "apply_dependency_inversion()".to_string(),
                ]
            }
            ArchIssueType::MissingAbstraction { .. } => {
                vec![
                    "extract_interface_trait()".to_string(),
                    "create_new_module_stub()".to_string(),
                ]
            }
        }
    }
}

impl Default for SelfArchitectureOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_file(path: &str, lines: usize) -> (String, usize) {
        (path.to_string(), lines)
    }

    #[allow(dead_code)]
    fn empty_awareness() -> AwarenessReport {
        AwarenessReport {
            gaps: vec![],
            total_gap: 0.0,
            critical_count: 0,
            significant_count: 0,
            recommended_focus: vec![],
            overall_health: 1.0,
        }
    }

    fn awareness_with_focus(dims: &[&str]) -> AwarenessReport {
        AwarenessReport {
            gaps: vec![],
            total_gap: 0.0,
            critical_count: 0,
            significant_count: 0,
            recommended_focus: dims.iter().map(|s| s.to_string()).collect(),
            overall_health: 0.5,
        }
    }

    #[test]
    fn no_large_modules() {
        let optimizer = SelfArchitectureOptimizer::new();
        let files = vec![
            make_file("src/main.rs", 100),
            make_file("src/lib.rs", 50),
        ];
        let report = optimizer.analyze(&files, None);
        assert_eq!(report.total_suggestions, 0);
        assert!(report.large_modules.is_empty());
    }

    #[test]
    fn detects_large_module() {
        let optimizer = SelfArchitectureOptimizer::new();
        let files = vec![
            make_file("src/main.rs", 100),
            make_file("src/huge.rs", 900),
        ];
        let report = optimizer.analyze(&files, None);
        assert_eq!(report.total_suggestions, 1);
        assert_eq!(report.large_modules.len(), 1);
        assert_eq!(report.large_modules[0], "src/huge.rs");
    }

    #[test]
    fn bridge_bloat_detection() {
        let optimizer = SelfArchitectureOptimizer::new();
        let files = vec![
            make_file("src/bridge/mod.rs", 250),
            make_file("src/processor_bridge.rs", 180),
        ];
        let report = optimizer.analyze(&files, None);
        let bridge_suggestions: Vec<_> = report
            .suggestions
            .iter()
            .filter(|s| matches!(s.issue_type, ArchIssueType::BridgeBloat { .. }))
            .collect();
        assert_eq!(bridge_suggestions.len(), 1);
        assert!(bridge_suggestions[0].auto_fixable);
    }

    #[test]
    fn fix_plan_for_large_module() {
        let optimizer = SelfArchitectureOptimizer::new();
        let suggestion = ArchSuggestion {
            id: "LARGE-001".into(),
            issue_type: ArchIssueType::LargeModule {
                file: "src/core.rs".into(),
                lines: 1200,
            },
            description: "Module is too large".into(),
            effort: EffortEstimate::Moderate,
            impact: 0.7,
            auto_fixable: false,
        };
        let plan = optimizer.generate_fix_plan(&suggestion);
        assert_eq!(plan.len(), 2);
        assert!(plan[0].starts_with("split_into_submodules"));
        assert!(plan[1].starts_with("create_mod_rs"));
    }

    #[test]
    fn report_format() {
        let optimizer = SelfArchitectureOptimizer::new();
        let files = vec![make_file("src/big.rs", 1000)];
        let report = optimizer.analyze(&files, None);
        let formatted = optimizer.format_report(&report);
        assert!(formatted.contains("LargeModule"));
        assert!(formatted.contains("big.rs"));
        assert!(formatted.contains("1000 lines"));
    }

    #[test]
    fn awareness_affects_suggestions() {
        let optimizer = SelfArchitectureOptimizer::new();
        let files = vec![make_file("src/small.rs", 50)];
        let awareness = awareness_with_focus(&["nt_shield_audit", "secret_detection"]);
        let report = optimizer.analyze(&files, Some(&awareness));
        assert!(report.total_suggestions >= 2);
        let abstraction_count = report
            .suggestions
            .iter()
            .filter(|s| matches!(s.issue_type, ArchIssueType::MissingAbstraction { .. }))
            .count();
        assert_eq!(abstraction_count, 2);
    }

    #[test]
    fn empty_files_list() {
        let optimizer = SelfArchitectureOptimizer::new();
        let report = optimizer.analyze(&[], None);
        assert_eq!(report.total_suggestions, 0);
        assert!(report.large_modules.is_empty());
    }

    #[test]
    fn auto_fixable_count() {
        let optimizer = SelfArchitectureOptimizer::new();
        let files = vec![
            make_file("src/huge.rs", 1000),
            make_file("src/bridge/mod.rs", 250),
        ];
        let report = optimizer.analyze(&files, None);
        assert!(report.auto_fixable_count >= 1);
        assert_eq!(report.total_suggestions, 2);
    }
}
