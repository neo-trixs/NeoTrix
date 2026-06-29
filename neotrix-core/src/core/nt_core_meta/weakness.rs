use super::self_model::{DebtSeverity, SelfModel, TechDebtItem, TechDebtKind};

/// Multi-dimensional weakness analyzer.
/// Detects code smells, tech debt, architectural gaps, and security risks
/// by applying static patterns to the SelfModel.
#[derive(Debug, Clone)]
pub struct WeaknessAnalyzer {
    pub large_file_threshold: usize,
    pub missing_test_threshold: usize,
    pub unsafe_threshold: usize,
    pub unwrap_threshold: usize,
}

impl Default for WeaknessAnalyzer {
    fn default() -> Self {
        Self {
            large_file_threshold: 800,
            missing_test_threshold: 300,
            unsafe_threshold: 5,
            unwrap_threshold: 20,
        }
    }
}

impl WeaknessAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn analyze(&self, model: &SelfModel) -> WeaknessReport {
        let mut weaknesses = Vec::new();

        weaknesses.extend(self.detect_large_files(model));
        weaknesses.extend(self.detect_missing_tests(model));
        weaknesses.extend(self.detect_excess_unsafe(model));
        weaknesses.extend(self.detect_excess_unwrap(model));
        weaknesses.extend(self.detect_todo_leftovers(model));
        weaknesses.extend(self.detect_orphan_modules(model));
        weaknesses.extend(self.detect_circular_deps(model));
        weaknesses.extend(self.detect_compilation_warnings(model));
        weaknesses.extend(self.detect_test_gaps(model));
        weaknesses.extend(self.detect_tech_debt_accumulation(model));

        let total = weaknesses.len();
        let by_severity = self.count_by_severity(&weaknesses);

        WeaknessReport {
            timestamp: chrono::Utc::now(),
            weaknesses,
            summary: WeaknessSummary {
                total_count: total,
                critical_count: by_severity.0,
                major_count: by_severity.1,
                minor_count: by_severity.2,
                cosmetic_count: by_severity.3,
            },
        }
    }

    fn detect_large_files(&self, model: &SelfModel) -> Vec<Weakness> {
        model
            .files
            .iter()
            .filter(|f| f.lines > self.large_file_threshold && !f.is_test_file)
            .map(|f| Weakness {
                pattern_id: "LARGE_FILE".into(),
                target_module: Some(f.module.clone()),
                file: Some(f.path.clone()),
                line: None,
                severity: DebtSeverity::Minor,
                description: format!(
                    "File has {} lines (threshold: {})",
                    f.lines, self.large_file_threshold
                ),
                impact: "Reduced maintainability, harder to navigate".into(),
                suggestion: "Split into smaller files with focused responsibilities".into(),
            })
            .collect()
    }

    fn detect_missing_tests(&self, model: &SelfModel) -> Vec<Weakness> {
        model
            .modules
            .iter()
            .filter(|m| {
                !m.has_tests
                    && m.total_lines > self.missing_test_threshold
                    && !m.name.contains("bin")
            })
            .map(|m| Weakness {
                pattern_id: "MISSING_TESTS".into(),
                target_module: Some(m.name.clone()),
                file: Some(m.path.clone()),
                line: None,
                severity: DebtSeverity::Major,
                description: format!(
                    "Module '{}' has {} lines but zero tests",
                    m.name, m.total_lines
                ),
                impact: "Changes in this module cannot be verified automatically".into(),
                suggestion: format!(
                    "Add #[cfg(test)] mod tests with unit tests for all public APIs in {}",
                    m.name
                ),
            })
            .collect()
    }

    fn detect_excess_unsafe(&self, model: &SelfModel) -> Vec<Weakness> {
        model
            .modules
            .iter()
            .filter(|m| m.unsafe_count > self.unsafe_threshold)
            .map(|m| Weakness {
                pattern_id: "EXCESS_UNSAFE".into(),
                target_module: Some(m.name.clone()),
                file: Some(m.path.clone()),
                line: None,
                severity: DebtSeverity::Critical,
                description: format!("Module '{}' has {} unsafe blocks", m.name, m.unsafe_count),
                impact: "Increased risk of undefined behavior and memory bugs".into(),
                suggestion: "Review and minimize unsafe usage; prefer safe abstractions".into(),
            })
            .collect()
    }

    fn detect_excess_unwrap(&self, model: &SelfModel) -> Vec<Weakness> {
        model
            .modules
            .iter()
            .filter(|m| m.unwrap_count > self.unwrap_threshold)
            .map(|m| Weakness {
                pattern_id: "EXCESS_UNWRAP".into(),
                target_module: Some(m.name.clone()),
                file: Some(m.path.clone()),
                line: None,
                severity: DebtSeverity::Major,
                description: format!("Module '{}' has {} unwrap() calls", m.name, m.unwrap_count),
                impact: "Runtime panics if Option/Result is None/Err".into(),
                suggestion: "Replace .unwrap() with .ok(), ? operator, or proper error handling"
                    .into(),
            })
            .collect()
    }

    fn detect_todo_leftovers(&self, model: &SelfModel) -> Vec<Weakness> {
        model
            .modules
            .iter()
            .filter(|m| m.todo_count > 3)
            .map(|m| Weakness {
                pattern_id: "TODO_LEFTOVERS".into(),
                target_module: Some(m.name.clone()),
                file: Some(m.path.clone()),
                line: None,
                severity: DebtSeverity::Minor,
                description: format!(
                    "Module '{}' has {} TODO/todo! markers",
                    m.name, m.todo_count
                ),
                impact: "Incomplete features or unresolved issues".into(),
                suggestion: "Review TODOs, implement remaining work or convert to tracked issues"
                    .into(),
            })
            .collect()
    }

    fn detect_orphan_modules(&self, model: &SelfModel) -> Vec<Weakness> {
        let orphans = model.dep_graph.orphans();
        orphans
            .iter()
            .map(|name| Weakness {
                pattern_id: "ORPHAN_MODULE".into(),
                target_module: Some(name.clone()),
                file: Some(name.clone()),
                line: None,
                severity: DebtSeverity::Major,
                description: format!("Module '{}' has no dependents (orphan)", name),
                impact: "Dead code that wastes maintenance effort".into(),
                suggestion: "Verify if module is still needed; remove or consolidate".into(),
            })
            .collect()
    }

    fn detect_circular_deps(&self, model: &SelfModel) -> Vec<Weakness> {
        let cycles = model.dep_graph.find_cycles();
        cycles
            .iter()
            .map(|cycle| Weakness {
                pattern_id: "CIRCULAR_DEP".into(),
                target_module: Some(cycle.first().cloned().unwrap_or_default()),
                file: None,
                line: None,
                severity: DebtSeverity::Critical,
                description: format!("Circular dependency detected: {}", cycle.join(" -> ")),
                impact: "Tight coupling, prevents independent module evolution".into(),
                suggestion: "Extract shared dependency or invert dependency direction".into(),
            })
            .collect()
    }

    fn detect_compilation_warnings(&self, model: &SelfModel) -> Vec<Weakness> {
        if model.compilation.warnings > 0 {
            model
                .modules
                .iter()
                .map(|m| Weakness {
                    pattern_id: "COMPILATION_WARNING".into(),
                    target_module: Some(m.name.clone()),
                    file: Some(m.path.clone()),
                    line: None,
                    severity: DebtSeverity::Minor,
                    description: format!("Module '{}' has compilation warnings", m.name),
                    impact: "Potential runtime issues or code smells".into(),
                    suggestion: "Run cargo fix to auto-resolve warnings".into(),
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    fn detect_test_gaps(&self, model: &SelfModel) -> Vec<Weakness> {
        model
            .modules
            .iter()
            .filter(|m| m.public_api_count > 0 && m.test_count == 0 && m.total_lines > 100)
            .map(|m| Weakness {
                pattern_id: "TEST_GAP".into(),
                target_module: Some(m.name.clone()),
                file: Some(m.path.clone()),
                line: None,
                severity: DebtSeverity::Minor,
                description: format!(
                    "Module '{}' has {} public APIs but {} tests",
                    m.name, m.public_api_count, m.test_count
                ),
                impact: "Public API surface is untested".into(),
                suggestion: "Add integration tests for public APIs".into(),
            })
            .collect()
    }

    fn detect_tech_debt_accumulation(&self, model: &SelfModel) -> Vec<Weakness> {
        let critical = model
            .tech_debt
            .items
            .iter()
            .filter(|i| i.severity == DebtSeverity::Critical)
            .count();
        if critical > 10 {
            vec![Weakness {
                pattern_id: "DEBT_ACCUMULATION".into(),
                target_module: None,
                file: None,
                line: None,
                severity: DebtSeverity::Critical,
                description: format!("Critical tech debt has accumulated ({} items)", critical),
                impact: "Long-term maintainability at risk".into(),
                suggestion: "Schedule a dedicated tech debt reduction sprint".into(),
            }]
        } else {
            Vec::new()
        }
    }

    fn count_by_severity(&self, weaknesses: &[Weakness]) -> (usize, usize, usize, usize) {
        let mut critical = 0;
        let mut major = 0;
        let mut minor = 0;
        let mut cosmetic = 0;
        for w in weaknesses {
            match w.severity {
                DebtSeverity::Critical => critical += 1,
                DebtSeverity::Major => major += 1,
                DebtSeverity::Minor => minor += 1,
                DebtSeverity::Cosmetic => cosmetic += 1,
            }
        }
        (critical, major, minor, cosmetic)
    }

    pub fn to_tech_debt_items(&self, report: &WeaknessReport) -> Vec<TechDebtItem> {
        report
            .weaknesses
            .iter()
            .map(|w| TechDebtItem {
                file: w.file.clone().unwrap_or_default(),
                line: w.line,
                kind: match w.pattern_id.as_str() {
                    "LARGE_FILE" => TechDebtKind::LargeFile,
                    "MISSING_TESTS" | "TEST_GAP" => TechDebtKind::MissingTests,
                    "EXCESS_UNSAFE" => TechDebtKind::UnsafeBlock,
                    "EXCESS_UNWRAP" => TechDebtKind::UnwrapCall,
                    "TODO_LEFTOVERS" => TechDebtKind::TodoComment,
                    "ORPHAN_MODULE" => TechDebtKind::OrphanModule,
                    "CIRCULAR_DEP" => TechDebtKind::CircularDependency,
                    _ => TechDebtKind::LargeFile,
                },
                description: w.description.clone(),
                severity: w.severity,
                suggested_action: w.suggestion.clone(),
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Weakness {
    pub pattern_id: String,
    pub target_module: Option<String>,
    pub file: Option<String>,
    pub line: Option<usize>,
    pub severity: DebtSeverity,
    pub description: String,
    pub impact: String,
    pub suggestion: String,
}

#[derive(Debug, Clone)]
pub struct WeaknessReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub weaknesses: Vec<Weakness>,
    pub summary: WeaknessSummary,
}

#[derive(Debug, Clone)]
pub struct WeaknessSummary {
    pub total_count: usize,
    pub critical_count: usize,
    pub major_count: usize,
    pub minor_count: usize,
    pub cosmetic_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_meta::self_model::*;

    fn make_test_model() -> SelfModel {
        let mut model = SelfModel::new();
        model.modules.push(ModuleInfo {
            name: "safe_module".into(),
            path: "src/safe/".into(),
            file_count: 1,
            total_lines: 100,
            test_count: 5,
            has_tests: true,
            unsafe_count: 0,
            unwrap_count: 2,
            todo_count: 1,
            public_api_count: 5,
            description: "".into(),
        });
        model.modules.push(ModuleInfo {
            name: "unsafe_module".into(),
            path: "src/unsafe/".into(),
            file_count: 1,
            total_lines: 500,
            test_count: 0,
            has_tests: false,
            unsafe_count: 10,
            unwrap_count: 30,
            todo_count: 5,
            public_api_count: 8,
            description: "".into(),
        });
        model.files.push(FileInfo {
            path: "src/unsafe/big.rs".into(),
            module: "unsafe_module".into(),
            lines: 900,
            is_test_file: false,
            has_unsafe: true,
            has_todos: true,
            pub_fns: 5,
            last_modified: chrono::Utc::now(),
        });
        model
    }

    #[test]
    fn test_analyzer_detects_large_files() {
        let analyzer = WeaknessAnalyzer::new();
        let model = make_test_model();
        let report = analyzer.analyze(&model);
        let large_files: Vec<_> = report
            .weaknesses
            .iter()
            .filter(|w| w.pattern_id == "LARGE_FILE")
            .collect();
        assert!(!large_files.is_empty());
    }

    #[test]
    fn test_analyzer_detects_missing_tests() {
        let analyzer = WeaknessAnalyzer::new();
        let model = make_test_model();
        let report = analyzer.analyze(&model);
        let no_tests: Vec<_> = report
            .weaknesses
            .iter()
            .filter(|w| w.pattern_id == "MISSING_TESTS")
            .collect();
        assert!(!no_tests.is_empty());
    }

    #[test]
    fn test_analyzer_detects_excess_unsafe() {
        let analyzer = WeaknessAnalyzer::new();
        let model = make_test_model();
        let report = analyzer.analyze(&model);
        let unsafe_issues: Vec<_> = report
            .weaknesses
            .iter()
            .filter(|w| w.pattern_id == "EXCESS_UNSAFE")
            .collect();
        assert!(!unsafe_issues.is_empty());
    }

    #[test]
    fn test_analyzer_detects_excess_unwrap() {
        let analyzer = WeaknessAnalyzer::new();
        let model = make_test_model();
        let report = analyzer.analyze(&model);
        let unwrap_issues: Vec<_> = report
            .weaknesses
            .iter()
            .filter(|w| w.pattern_id == "EXCESS_UNWRAP")
            .collect();
        assert!(!unwrap_issues.is_empty());
    }

    #[test]
    fn test_analyzer_detects_todos() {
        let analyzer = WeaknessAnalyzer::new();
        let model = make_test_model();
        let report = analyzer.analyze(&model);
        let todo_issues: Vec<_> = report
            .weaknesses
            .iter()
            .filter(|w| w.pattern_id == "TODO_LEFTOVERS")
            .collect();
        assert!(!todo_issues.is_empty());
    }

    #[test]
    fn test_weakness_report_summary() {
        let analyzer = WeaknessAnalyzer::new();
        let model = make_test_model();
        let report = analyzer.analyze(&model);
        assert_eq!(report.summary.total_count, report.weaknesses.len());
    }

    #[test]
    fn test_to_tech_debt_items() {
        let analyzer = WeaknessAnalyzer::new();
        let model = make_test_model();
        let report = analyzer.analyze(&model);
        let items = analyzer.to_tech_debt_items(&report);
        assert_eq!(items.len(), report.weaknesses.len());
    }
}
