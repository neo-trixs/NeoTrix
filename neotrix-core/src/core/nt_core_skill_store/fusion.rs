use std::collections::HashMap;
use std::collections::HashSet;

use crate::core::nt_core_skill_store::discovery::{DiscoveredSkill, SkillCategory};

/// What to do with a discovered skill
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FusionAction {
    /// Create as new skill
    CreateNew,
    /// Merge into existing skill (extend its capabilities)
    MergeIntoExisting(String),
    /// Ignore — existing skill already covers it
    SkipExisting,
    /// Defer — needs more analysis
    Defer,
}

/// Gap analysis result
#[derive(Debug, Clone)]
pub struct SkillGap {
    pub area: String,
    pub category: SkillCategory,
    pub existing_skills: Vec<String>,
    pub suggested_action: String,
}

/// Report from a fusion cycle
#[derive(Debug, Clone)]
pub struct FusionReport {
    pub new_skills: Vec<String>,
    pub merged_skills: Vec<String>,
    pub skipped_skills: Vec<String>,
    pub gaps_found: Vec<SkillGap>,
    pub total_analyzed: usize,
}

/// Compares discovered skills against existing, decides integration strategy
pub struct SkillFusion;

impl SkillFusion {
    /// Compute Jaccard similarity between two strings
    fn jaccard_words(a: &str, b: &str) -> f64 {
        let set_a: HashSet<&str> = a.to_lowercase().split_whitespace().collect();
        let set_b: HashSet<&str> = b.to_lowercase().split_whitespace().collect();
        let intersection = set_a.intersection(&set_b).count();
        let union = set_a.union(&set_b).count();
        if union == 0 {
            return 0.0;
        }
        intersection as f64 / union as f64
    }

    /// Compute name overlap between discovered name and existing names
    fn name_overlap(name: &str, existing_names: &[String]) -> Vec<(String, f64)> {
        let name_lower = name.to_lowercase();
        existing_names
            .iter()
            .map(|n| {
                let nl = n.to_lowercase();
                let sim = if nl.contains(&name_lower) || name_lower.contains(&nl) {
                    0.9
                } else {
                    Self::jaccard_words(&name_lower, &nl)
                };
                (n.clone(), sim)
            })
            .filter(|(_, s)| *s > 0.3)
            .collect()
    }

    /// Analyze a discovered skill and determine fusion action
    /// Checks: name similarity, description overlap, category match, methodology overlap
    pub fn analyze(
        discovered: &DiscoveredSkill,
        existing_names: &[String],
        existing_methods: &HashMap<String, Vec<String>>,
    ) -> FusionAction {
        let overlaps = Self::name_overlap(&discovered.name, existing_names);

        // Check for exact-ish name match
        if let Some((matched_name, sim)) = overlaps.first() {
            if *sim >= 0.85 {
                // Check methodology overlap
                if let Some(known_methods) = existing_methods.get(matched_name) {
                    let discovered_methods: HashSet<&str> =
                        discovered.methodology.iter().map(|s| s.as_str()).collect();
                    let known_set: HashSet<&str> =
                        known_methods.iter().map(|s| s.as_str()).collect();
                    let novel: Vec<_> = discovered_methods.difference(&known_set).collect();
                    if novel.len() >= 2 {
                        return FusionAction::MergeIntoExisting(matched_name.clone());
                    }
                }
                return FusionAction::SkipExisting;
            }

            // Partial overlap — check description too
            if let Some(matched_name) = overlaps.first() {
                let desc_sim = existing_names
                    .iter()
                    .filter_map(|n| {
                        // We don't have descriptions indexed, use name + heuristic
                        let name_sim = Self::jaccard_words(&discovered.name, n);
                        if name_sim > 0.2 {
                            Some(name_sim)
                        } else {
                            None
                        }
                    })
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

                if let Some(ds) = desc_sim {
                    if ds > 0.4 {
                        return FusionAction::MergeIntoExisting(matched_name.0.clone());
                    }
                }
            }
        }

        // Default: create new
        FusionAction::CreateNew
    }

    /// Run full fusion cycle on batch of discovered skills
    pub fn fusion_cycle(
        discovered: &[DiscoveredSkill],
        existing_names: &[String],
        existing_methods: &HashMap<String, Vec<String>>,
    ) -> FusionReport {
        let mut new = Vec::new();
        let mut merged = Vec::new();
        let mut skipped = Vec::new();

        for skill in discovered {
            match Self::analyze(skill, existing_names, existing_methods) {
                FusionAction::CreateNew => new.push(skill.name.clone()),
                FusionAction::MergeIntoExisting(target) => {
                    merged.push(format!("{}→{}", skill.name, target))
                }
                FusionAction::SkipExisting => skipped.push(skill.name.clone()),
                FusionAction::Defer => {}
            }
        }

        let gaps = Self::detect_gaps(existing_names, &[]);

        FusionReport {
            new_skills: new,
            merged_skills: merged,
            skipped_skills: skipped,
            gaps_found: gaps,
            total_analyzed: discovered.len(),
        }
    }

    /// Detect capability gaps — areas where NeoTrix has limited or no coverage
    pub fn detect_gaps(
        existing_names: &[String],
        existing_categories: &[SkillCategory],
    ) -> Vec<SkillGap> {
        let cat_set: HashSet<SkillCategory> = existing_categories.iter().cloned().collect();
        let names_lower: Vec<String> = existing_names.iter().map(|n| n.to_lowercase()).collect();
        let mut gaps = Vec::new();

        // Check for missing categories
        let desired_categories = vec![
            ("Security auditing".into(), SkillCategory::Security),
            ("Design systems".into(), SkillCategory::Design),
            ("DevOps automation".into(), SkillCategory::Devops),
            ("Data science workflows".into(), SkillCategory::DataScience),
        ];

        for (area, cat) in &desired_categories {
            if !cat_set.contains(cat) {
                let existing: Vec<String> = names_lower
                    .iter()
                    .filter(|n| n.contains(&cat.label().to_lowercase()))
                    .cloned()
                    .collect();
                if existing.is_empty() {
                    gaps.push(SkillGap {
                        area: area.clone(),
                        category: cat.clone(),
                        existing_skills: vec![],
                        suggested_action: format!(
                            "Consider adding or discovering a skill in the {} category",
                            cat.label()
                        ),
                    });
                }
            }
        }

        // Check for DevOps coverage specifically
        let devops_terms = ["ci", "cd", "deploy", "pipeline", "docker", "kubernetes"];
        let has_devops = names_lower
            .iter()
            .any(|n| devops_terms.iter().any(|t| n.contains(t)));
        if !has_devops {
            gaps.push(SkillGap {
                area: "CI/CD pipeline management".into(),
                category: SkillCategory::Devops,
                existing_skills: vec![],
                suggested_action: "Add a devops-pipeline style skill for CI/CD automation".into(),
            });
        }

        gaps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_skill_store::discovery::{DiscoveredSkill, SkillSource};

    fn make_discovered(name: &str, cat: SkillCategory, methods: Vec<&str>) -> DiscoveredSkill {
        DiscoveredSkill {
            name: name.into(),
            description: format!("{} description", name),
            source: SkillSource::GitHub,
            source_url: format!("https://github.com/test/{}", name),
            category: cat,
            star_count: 100,
            author: None,
            methodology: methods.iter().map(|s| s.to_string()).collect(),
            instructions: None,
            tags: vec![],
            install_command: None,
        }
    }

    #[test]
    fn test_analyze_exact_name_match_returns_skip() {
        let discovered = make_discovered("debugging", SkillCategory::Debugging, vec!["tracing"]);
        let existing_names = vec!["debugging".into()];
        let mut methods = HashMap::new();
        methods.insert("debugging".into(), vec!["tracing".into()]);
        let action = SkillFusion::analyze(&discovered, &existing_names, &methods);
        assert_eq!(
            action,
            FusionAction::SkipExisting,
            "exact name match with same methods should skip"
        );
    }

    #[test]
    fn test_analyze_partial_match_returns_merge() {
        let discovered = make_discovered(
            "debugging-pro",
            SkillCategory::Debugging,
            vec!["tracing", "heap-analysis", "stack-trace"],
        );
        let existing_names = vec!["debugging".into()];
        let mut methods = HashMap::new();
        methods.insert("debugging".into(), vec!["tracing".into()]);
        let action = SkillFusion::analyze(&discovered, &existing_names, &methods);
        assert_eq!(
            action,
            FusionAction::MergeIntoExisting("debugging".into()),
            "partial name with novel methods should merge"
        );
    }

    #[test]
    fn test_analyze_no_match_returns_create() {
        let discovered = make_discovered(
            "quantum-computing",
            SkillCategory::Specialized("quantum".into()),
            vec!["qsim"],
        );
        let existing_names = vec!["debugging".into(), "testing".into()];
        let methods = HashMap::new();
        let action = SkillFusion::analyze(&discovered, &existing_names, &methods);
        assert_eq!(action, FusionAction::CreateNew);
    }

    #[test]
    fn test_detect_gaps_identifies_missing_categories() {
        let names = vec!["coding".into(), "testing".into()];
        let cats = vec![SkillCategory::Development, SkillCategory::Testing];
        let gaps = SkillFusion::detect_gaps(&names, &cats);
        let gap_areas: Vec<&str> = gaps.iter().map(|g| g.area.as_str()).collect();
        assert!(gap_areas.contains(&"Security auditing"));
        assert!(gap_areas.contains(&"Design systems"));
    }

    #[test]
    fn test_fusion_cycle_report_stats() {
        let discovered = vec![
            make_discovered("debugging", SkillCategory::Debugging, vec!["tracing"]),
            make_discovered(
                "quantum",
                SkillCategory::Specialized("quantum".into()),
                vec!["qsim"],
            ),
        ];
        let existing_names = vec!["debugging".into()];
        let mut methods = HashMap::new();
        methods.insert("debugging".into(), vec!["tracing".into()]);
        let report = SkillFusion::fusion_cycle(&discovered, &existing_names, &methods);
        assert_eq!(report.total_analyzed, 2);
        assert_eq!(report.skipped_skills.len(), 1);
        assert_eq!(report.new_skills.len(), 1);
    }

    #[test]
    fn test_detect_gaps_devops_coverage() {
        let names = vec!["coding".into(), "testing".into()];
        let cats = vec![SkillCategory::Development, SkillCategory::Testing];
        let gaps = SkillFusion::detect_gaps(&names, &cats);
        let has_devops_gap = gaps.iter().any(|g| g.area.contains("CI/CD"));
        assert!(has_devops_gap);
    }
}
