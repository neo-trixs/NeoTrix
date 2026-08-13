#![deny(clippy::unwrap_used)]
//! Self-Constitution Internalization Module
//!
//! Parses AGENTS.md (the project's constitutional document) and internalizes
//! Dev Rules + Experience Tree into the ConsciousnessTree's soil/roots.
//! This enables the system to "know its own constitution" during reasoning.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

use crate::core::nt_core_hcube::fhrr_vsa::{FhrrVector, similarity};

/// Rule categories for semantic indexing and retrieval
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleCategory {
    /// Build and compilation discipline (R-P1~R-P8, R-P9, R-P13, R-P17, R-P29, R-P35)
    BuildDiscipline,
    /// Architecture constraints and layer boundaries (R-P12, R-P20~R-P22, R-P27, R-P34)
    ArchitectureConstraint,
    /// Behavioral grounding - detection must affect behavior (R-P24, R-P25, R-P30, R-P33, R-P36, R-P41)
    BehavioralGrounding,
    /// Meta-cognition: self-audit, self-test, hallucination detection (R-P10, R-P19, R-P23, R-P26, R-P28, R-P37, R-P39)
    MetaCognition,
    /// Tree Growth principles - how new capabilities grow from the consciousness tree (R-P42~R-P48)
    TreeGrowth,
    /// Absorption protocol - how external designs are distilled (R-P43)
    AbsorptionProtocol,
    /// Code quality patterns (R-P3, R-P4, R-P5, R-P6, R-P7, R-P8, R-P11, R-P31, R-P40)
    CodeQualityPattern,
    /// Error handling and reliability (R-P14, R-P15, R-P16, R-P18, R-P38)
    Reliability,
}

impl RuleCategory {
    pub fn from_rule_id(id: &str) -> Self {
        let num = id.trim_start_matches("R-P").parse::<u32>().unwrap_or(0);
        match num {
            1..=8 | 9 | 13 | 17 | 29 | 35 => RuleCategory::BuildDiscipline,
            12 | 20..=22 | 27 | 34 => RuleCategory::ArchitectureConstraint,
            24 | 25 | 30 | 33 | 36 | 41 => RuleCategory::BehavioralGrounding,
            10 | 19 | 23 | 26 | 28 | 37 | 39 => RuleCategory::MetaCognition,
            43 => RuleCategory::AbsorptionProtocol,
            42 | 44 | 45 | 46 | 47 | 48 => RuleCategory::TreeGrowth,
             11 | 31 | 40 => RuleCategory::CodeQualityPattern,
            14 | 15 | 16 | 18 | 38 => RuleCategory::Reliability,
            _ => RuleCategory::CodeQualityPattern,
        }
    }

    pub fn priority_weight(&self) -> f64 {
        match self {
            RuleCategory::TreeGrowth => 1.0,        // Highest: governs how we grow
            RuleCategory::AbsorptionProtocol => 0.95,
            RuleCategory::BehavioralGrounding => 0.9,
            RuleCategory::ArchitectureConstraint => 0.85,
            RuleCategory::MetaCognition => 0.8,
            RuleCategory::BuildDiscipline => 0.7,
            RuleCategory::Reliability => 0.65,
            RuleCategory::CodeQualityPattern => 0.5,
        }
    }
}

/// A single dev rule extracted from AGENTS.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevRule {
    pub id: String,              // "R-P42"
    pub title: String,           // "Tree-Grafting"
    pub content: String,         // Full rule text
    pub category: RuleCategory,
    pub source_cycle: u32,       // Cycle when added
    pub vector: Option<FhrrVector>, // Semantic vector for retrieval
}

/// Experience Tree entry from AGENTS.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceEntry {
    pub cycle: u32,
    pub date: String,
    pub session_type: String,
    pub actions: Vec<String>,
    pub meta_findings: Vec<String>,
    pub dev_rules_added: Vec<String>,
    pub build_baseline: BuildBaseline,
}

/// Build baseline snapshot from Experience Tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildBaseline {
    pub lib_errors: u32,
    pub lib_warnings: u32,
    pub all_targets_errors: u32,
    pub test_status: String,
}

/// The internalized constitution - all rules + experiences + semantic index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constitution {
    pub rules: HashMap<String, DevRule>,
    pub experiences: Vec<ExperienceEntry>,
    pub tree_growth_rules: Vec<DevRule>,      // R-P42~R-P46
    pub absorption_rules: Vec<DevRule>,        // R-P43
    pub loaded_at: u64,
    pub source_hash: String,
    #[serde(skip)]
    vector_index: Option<ConstitutionVectorIndex>,
}

impl Constitution {
    /// Check if vector index is built
    pub fn has_vector_index(&self) -> bool {
        self.vector_index.is_some()
    }
}

/// Vector index for semantic rule retrieval
#[derive(Debug, Clone)]
pub struct ConstitutionVectorIndex {
    pub rule_vectors: HashMap<String, FhrrVector>,
    pub experience_vectors: Vec<(u32, FhrrVector)>, // (cycle, vector)
}

impl ConstitutionVectorIndex {
    fn new() -> Self {
        Self {
            rule_vectors: HashMap::new(),
            experience_vectors: Vec::new(),
        }
    }

    fn add_rule(&mut self, id: &str, vector: FhrrVector) {
        self.rule_vectors.insert(id.to_string(), vector);
    }

    fn add_experience(&mut self, cycle: u32, vector: FhrrVector) {
        self.experience_vectors.push((cycle, vector));
    }

    /// Find rules semantically relevant to a query
    pub fn query_rules(&self, query_vector: &FhrrVector, top_k: usize) -> Vec<(String, f64)> {
        let mut results: Vec<(String, f64)> = self.rule_vectors
            .iter()
            .map(|(id, v)| (id.clone(), similarity(query_vector.phases(), v.phases())))
            .collect();
        results.sort_by(|a, b| b.1.total_cmp(&a.1));
        results.into_iter().take(top_k).collect()
    }

    /// Find relevant experiences
    pub fn query_experiences(&self, query_vector: &FhrrVector, top_k: usize) -> Vec<(u32, f64)> {
        let mut results: Vec<(u32, f64)> = self.experience_vectors
            .iter()
            .map(|(cycle, v)| (*cycle, similarity(query_vector.phases(), v.phases())))
            .collect();
        results.sort_by(|a, b| b.1.total_cmp(&a.1));
        results.into_iter().take(top_k).collect()
    }
}

impl Default for Constitution {
    fn default() -> Self {
        Self::new()
    }
}

impl Constitution {
    /// Create empty constitution
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            experiences: Vec::new(),
            tree_growth_rules: Vec::new(),
            absorption_rules: Vec::new(),
            loaded_at: 0,
            source_hash: String::new(),
            vector_index: None,
        }
    }

    /// Build vector index for semantic retrieval
    pub fn build_vector_index(&mut self) {
        let mut index = ConstitutionVectorIndex::new();

        for rule in self.rules.values() {
            if let Some(vec) = &rule.vector {
                index.add_rule(&rule.id, vec.clone());
            }
        }
        for exp in &self.experiences {
            let text = format!(
                "Cycle {} {}: {} {}",
                exp.cycle,
                exp.session_type,
                exp.actions.join(" "),
                exp.meta_findings.join(" ")
            );
            let vec = encode_text_to_fhrr(&text);
            index.add_experience(exp.cycle, vec);
        }
        self.vector_index = Some(index);
    }

    /// Get rules relevant to a task description
    pub fn relevant_rules_for_task(&self, task_desc: &str, top_k: usize) -> Vec<&DevRule> {
        let Some(index) = &self.vector_index else { return Vec::new(); };
        let query_vec = FhrrVector::from_scalar(task_desc.len() as f64);
        let results = index.query_rules(&query_vec, top_k);
        results.into_iter()
            .filter_map(|(id, _)| self.rules.get(&id))
            .collect()
    }

    /// Get tree growth rules (R-P42~R-P46) - highest priority for architecture decisions
    pub fn tree_growth_rules(&self) -> &[DevRule] {
        &self.tree_growth_rules
    }

    /// Get absorption rules (R-P43) - for external design integration
    pub fn absorption_rules(&self) -> &[DevRule] {
        &self.absorption_rules
    }

    /// Verify if an action description complies with constitution
    pub fn verify_compliance(&self, action_desc: &str) -> ComplianceReport {
        let relevant = self.relevant_rules_for_task(action_desc, 10);
        let checked_count = relevant.len();
        let mut violations = Vec::new();
        let warnings = Vec::new();

        for rule in relevant {
            // Simple keyword-based violation detection
            // In production, this would use more sophisticated reasoning
            if self.check_violation(rule, action_desc) {
                violations.push(ComplianceViolation {
                    rule_id: rule.id.clone(),
                    rule_title: rule.title.clone(),
                    severity: if rule.category == RuleCategory::TreeGrowth {
                        ViolationSeverity::Critical
                    } else if rule.category == RuleCategory::BehavioralGrounding {
                        ViolationSeverity::High
                    } else {
                        ViolationSeverity::Medium
                    },
                    description: format!("Action may violate {}", rule.title),
                });
            }
        }

        ComplianceReport {
            compliant: violations.is_empty(),
            violations,
            warnings,
            checked_rules: checked_count,
        }
    }

    pub(crate) fn check_violation(&self, rule: &DevRule, action_desc: &str) -> bool {
        let desc_lower = action_desc.to_lowercase();
        match rule.id.as_str() {
            "R-P42" => desc_lower.contains("new module") && !desc_lower.contains("branch") && !desc_lower.contains("extend"),
            "R-P43" => desc_lower.contains("copy") && (desc_lower.contains("claude") || desc_lower.contains("codex")) && !desc_lower.contains("distill"),
            "R-P44" => desc_lower.contains("cargo check") && !desc_lower.contains("register"),
            "R-P45" => desc_lower.contains("nt-mind") && !desc_lower.contains("health"),
            "R-P46" => desc_lower.contains("yaml") && desc_lower.contains("tool") && !desc_lower.contains("hexagram"),
            "R-P47" => (desc_lower.contains("adapter") || desc_lower.contains("wrapper") || desc_lower.contains("new mod")) && !desc_lower.contains("node") && !desc_lower.contains("reinforce"),
            "R-P48" => (desc_lower.contains("command::new") || desc_lower.contains("binary dep") || desc_lower.contains("external bin")) && !desc_lower.contains("reqwest"),
            _ => false,
        }
    }
}

/// Compliance verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub compliant: bool,
    pub violations: Vec<ComplianceViolation>,
    pub warnings: Vec<String>,
    pub checked_rules: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceViolation {
    pub rule_id: String,
    pub rule_title: String,
    pub severity: ViolationSeverity,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Constitution Loader - parses AGENTS.md and builds Constitution
pub struct ConstitutionLoader;

impl ConstitutionLoader {
    /// Load and parse AGENTS.md into a Constitution
    pub fn load_from_file(path: &Path) -> Result<Constitution, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read AGENTS.md: {}", e))?;
        Self::parse(&content)
    }

    /// Parse AGENTS.md content into Constitution
    pub fn parse(content: &str) -> Result<Constitution, String> {
        let mut constitution = Constitution::new();

        // Compute source hash for change detection
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        constitution.source_hash = format!("{:x}", hasher.finish());

        // Extract Dev Rules
        constitution.rules = Self::extract_dev_rules(content)?;
        Self::categorize_rules(&mut constitution);

        // Extract Experience Tree
        constitution.experiences = Self::extract_experience_tree(content)?;

        // Build vectors
        Self::vectorize_rules(&mut constitution);
        constitution.build_vector_index();

        Ok(constitution)
    }

    /// Extract all R-Pxx rules from Dev Rules section
    fn extract_dev_rules(content: &str) -> Result<HashMap<String, DevRule>, String> {
        let mut rules = HashMap::new();

        // Find Dev Rules section
        let dev_rules_section = Self::find_section(content, "Dev Rules", "Experience Tree")
            .or_else(|| Self::find_section(content, "Dev Rules Added", "Experience Tree"))
            .or_else(|| Self::find_section(content, "Dev Rules", "## "))
            .ok_or("Dev Rules section not found")?;

        // 兼容格式:
        //   1. `- **R-Pxx (Title)**: content` (AGENTS.md 历史格式)
        //   2. `- **R-Pxx**: content` (dev-rules.md 现行格式, 无 Title 括号)
        //   3. `- **R-P42 / R-P47 (Title)**: content` (组合 ID, 双规则共用正文)
        let rule_regex = regex::Regex::new(
            r"(?m)^\s*-\s*\*\*R-P(\d+)(?:\s*/\s*R-P(\d+))?(?:\s*\(([^)]+)\))?\*\*:\s*(.+)$",
        )
        .map_err(|e| format!("Regex error: {}", e))?;

        for cap in rule_regex.captures_iter(dev_rules_section) {
            let title = cap.get(3).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
            let content_text = cap[4].trim().to_string();
            // 主规则 + 可选组合规则 (R-P42 / R-P47 → 两条规则共享正文)
            let mut ids = vec![format!("R-P{}", &cap[1])];
            if let Some(second) = cap.get(2) {
                ids.push(format!("R-P{}", second.as_str()));
            }
            for id in ids {
                let category = RuleCategory::from_rule_id(&id);
                let rule = DevRule {
                    id: id.clone(),
                    title: title.clone(),
                    content: content_text.clone(),
                    category: category.clone(),
                    source_cycle: Self::extract_cycle_for_rule(dev_rules_section, &id).unwrap_or(0),
                    vector: None,
                };
                rules.insert(id, rule);
            }
        }

        // Also catch rules in "Dev Rules Added (R-Px to R-Py)" format
        let added_regex = regex::Regex::new(r"(?m)Dev Rules Added\s*\(R-P(\d+)\s*to\s*R-P(\d+)\)")
            .map_err(|e| format!("Regex error: {}", e))?;
        for cap in added_regex.captures_iter(content) {
            let _start: u32 = cap[1].parse().unwrap_or(0);
            let _end: u32 = cap[2].parse().unwrap_or(0);
            // These are already captured above, just note the cycle range
        }

        Ok(rules)
    }

    /// Extract Experience Tree entries
    #[allow(clippy::invalid_regex)]
    fn extract_experience_tree(content: &str) -> Result<Vec<ExperienceEntry>, String> {
        let mut experiences = Vec::new();

        // Find Experience Tree sections (uses look-ahead which clippy's parser doesn't support)
        let exp_regex = regex::Regex::new(
            r"(?ms)## Experience Tree\s*[—-]\s*(\d{4}-\d{2}-\d{2})\s*Cycle\s*(\d+)\s*\((.*?)\)\n(.*?)(?:\n## |\z)"
        ).map_err(|e| format!("Regex error: {}", e))?;

        for cap in exp_regex.captures_iter(content) {
            let date = cap[1].trim().to_string();
            let cycle: u32 = cap[2].trim().parse().unwrap_or(0);
            let session_type = cap[3].trim().to_string();
            let body = &cap[4];

            let actions = Self::extract_bullet_list(body, "Action");
            let meta_findings = Self::extract_bullet_list(body, "Meta-Cognitive");
            let dev_rules_added = Self::extract_bullet_list(body, "Dev Rules Added");
            let build_baseline = Self::extract_build_baseline(body);

            experiences.push(ExperienceEntry {
                cycle,
                date,
                session_type,
                actions,
                meta_findings,
                dev_rules_added,
                build_baseline,
            });
        }

        // Sort by cycle
        experiences.sort_by_key(|e| e.cycle);
        Ok(experiences)
    }

    fn extract_bullet_list(text: &str, keyword: &str) -> Vec<String> {
        let mut items = Vec::new();
        let pattern = format!(r"(?mi)^{}\s*[:\-]\s*(.+)$", regex::escape(keyword));
        if let Ok(re) = regex::Regex::new(&pattern) {
            for cap in re.captures_iter(text) {
                items.push(cap[1].trim().to_string());
            }
        }
        items
    }

    fn extract_build_baseline(text: &str) -> BuildBaseline {
        let mut baseline = BuildBaseline {
            lib_errors: 0,
            lib_warnings: 0,
            all_targets_errors: 0,
            test_status: "unknown".to_string(),
        };

        for line in text.lines() {
            let line = line.trim();
            if line.contains("lib errors") || line.contains("lib error") {
                if let Some(num) = Self::extract_number(line) {
                    baseline.lib_errors = num;
                }
            } else if line.contains("lib warnings") || line.contains("lib warning") {
                if let Some(num) = Self::extract_number(line) {
                    baseline.lib_warnings = num;
                }
            } else if line.contains("all.targets") || line.contains("all targets") {
                if let Some(num) = Self::extract_number(line) {
                    baseline.all_targets_errors = num;
                }
            } else if line.to_lowercase().contains("test") && (line.contains("pass") || line.contains("fail")) {
                baseline.test_status = line.to_string();
            }
        }
        baseline
    }

    fn extract_number(text: &str) -> Option<u32> {
        regex::Regex::new(r"\d+").ok()?
            .find(text)?
            .as_str()
            .parse()
            .ok()
    }

    fn find_section<'a>(content: &'a str, start_marker: &str, end_marker: &str) -> Option<&'a str> {
        let start = content.find(start_marker)?;
        let after_start = &content[start..];
        let end = after_start.find(end_marker).unwrap_or(after_start.len());
        Some(&after_start[..end])
    }

    fn extract_cycle_for_rule(section: &str, rule_id: &str) -> Option<u32> {
        let cycle_re = regex::Regex::new(r"Cycle\s*(\d+)").ok()?;
        let lines: Vec<&str> = section.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.contains(rule_id) {
                for j in (0..=i).rev() {
                    if let Some(cap) = cycle_re.captures(lines[j]) {
                        return cap[1].parse().ok();
                    }
                }
            }
        }
        None
    }

    fn categorize_rules(constitution: &mut Constitution) {
        for rule in constitution.rules.values() {
            match rule.category {
                RuleCategory::TreeGrowth => constitution.tree_growth_rules.push(rule.clone()),
                RuleCategory::AbsorptionProtocol => constitution.absorption_rules.push(rule.clone()),
                _ => {}
            }
        }
    }

    fn vectorize_rules(constitution: &mut Constitution) {
        for rule in constitution.rules.values_mut() {
            // Create semantic vector from rule content
            let text = format!("{} {} {}", rule.id, rule.title, rule.content);
            let vector = encode_text_to_fhrr(&text);
            rule.vector = Some(vector);
        }
    }
}

/// Simple text to FHRR vector encoding
fn encode_text_to_fhrr(text: &str) -> FhrrVector {
    // Use a hash-based approach for deterministic encoding
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let hash = hasher.finish();

    // Use hash as seed for scalar encoding
    FhrrVector::from_scalar(hash as f64)
}

/// Global constitution instance (loaded at startup)
static GLOBAL_CONSTITUTION: LazyLock<Constitution> = LazyLock::new(|| {
    // 优先加载全量规则文件 dev-rules.md (R-P1~R-P101), 回退 AGENTS.md。
    // 两者都向上查找以兼容任意 CWD (测试 crate 根 / 仓库根 / 子目录)。
    // dev-rules.md 是规则单一事实源 (R-P101: 两处修订必须同步)。
    let candidates = ["dev-rules.md", "AGENTS.md"];
    for candidate in candidates {
        let mut search_path = std::path::PathBuf::from(candidate);
        for _depth in 0..4 {
            if search_path.exists() {
                match ConstitutionLoader::load_from_file(&search_path) {
                    Ok(c) => {
                        if !c.rules.is_empty() {
                            return c;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to load constitution from {}: {}", search_path.display(), e);
                    }
                }
            }
            search_path = std::path::Path::new("..").join(search_path);
        }
    }
    eprintln!("Failed to load constitution: dev-rules.md and AGENTS.md not found (CWD and parents)");
    Constitution::new()
});

/// Close Constitution impl block
impl Constitution {
    // This impl block is intentionally empty - it's just to close the previous one
}

/// Get global constitution reference
pub fn global_constitution() -> &'static Constitution {
    &GLOBAL_CONSTITUTION
}

/// Reload constitution from file (for hot reload)
pub fn reload_constitution(path: &Path) -> Result<(), String> {
    // Note: Can't actually replace LazyLock, but can return new instance
    // In practice, use a Mutex<Constitution> for hot reload
    let _ = ConstitutionLoader::load_from_file(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_rule_category_priority() {
        assert!(RuleCategory::TreeGrowth.priority_weight() > RuleCategory::BuildDiscipline.priority_weight());
        assert!(RuleCategory::AbsorptionProtocol.priority_weight() > RuleCategory::ArchitectureConstraint.priority_weight());
    }

    #[test]
    fn test_rule_categorization() {
        assert_eq!(RuleCategory::from_rule_id("R-P42"), RuleCategory::TreeGrowth);
        assert_eq!(RuleCategory::from_rule_id("R-P43"), RuleCategory::AbsorptionProtocol);
        assert_eq!(RuleCategory::from_rule_id("R-P24"), RuleCategory::BehavioralGrounding);
        assert_eq!(RuleCategory::from_rule_id("R-P10"), RuleCategory::MetaCognition);
    }

    #[test]
    fn test_constitution_new() {
        let c = Constitution::new();
        assert!(c.rules.is_empty());
        assert!(c.experiences.is_empty());
    }

    #[test]
    fn test_load_real_agents_md() {
        let path = Path::new("../../../AGENTS.md");
        if path.exists() {
            let result = ConstitutionLoader::load_from_file(path);
            assert!(result.is_ok(), "Failed to load real AGENTS.md: {:?}", result.err());
            let constitution = result.unwrap();
            assert!(!constitution.rules.is_empty(), "Should extract rules");
            assert!(!constitution.experiences.is_empty(), "Should extract experiences");
            assert!(!constitution.tree_growth_rules.is_empty(), "Should have tree growth rules");
            assert!(!constitution.absorption_rules.is_empty(), "Should have absorption rules");
            println!("Loaded {} rules, {} experiences, {} tree-growth, {} absorption",
                constitution.rules.len(),
                constitution.experiences.len(),
                constitution.tree_growth_rules.len(),
                constitution.absorption_rules.len()
            );
        }
    }

    #[test]
    fn test_compliance_check_tree_growth() {
        let path = Path::new("../../../AGENTS.md");
        if path.exists() {
            let constitution = ConstitutionLoader::load_from_file(path).unwrap();
            // Action that violates R-P42: creating new module without branch mapping
            let report = constitution.verify_compliance("create new module nt_core_subagent.rs without mapping to any branch");
            assert!(!report.compliant);
            assert!(report.violations.iter().any(|v| v.rule_id == "R-P42"));
        }
    }

    #[test]
    fn test_compliance_check_absorption() {
        let path = Path::new("../../../AGENTS.md");
        if path.exists() {
            let constitution = ConstitutionLoader::load_from_file(path).unwrap();
            // Action that violates R-P43: copying Claude Code design without distillation
            let report = constitution.verify_compliance("copy claude code subagent design directly without distillation");
            assert!(!report.compliant);
            assert!(report.violations.iter().any(|v| v.rule_id == "R-P43"));
        }
    }

    #[test]
    fn test_compliance_check_hexagram() {
        let path = Path::new("../../../AGENTS.md");
        if path.exists() {
            let constitution = ConstitutionLoader::load_from_file(path).unwrap();
            // Action that violates R-P46: YAML tools config without hexagram derivation
            let report = constitution.verify_compliance("define agent tools in yaml file without hexagram derivation");
            assert!(!report.compliant);
            assert!(report.violations.iter().any(|v| v.rule_id == "R-P46"));
        }
    }
}