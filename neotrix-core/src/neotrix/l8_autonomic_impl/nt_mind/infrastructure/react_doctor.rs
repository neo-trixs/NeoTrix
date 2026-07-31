use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReactRuleCategory {
    StateAndEffects,
    Performance,
    Architecture,
    Security,
    Accessibility,
    DeadCode,
}

impl ReactRuleCategory {
    pub fn name(&self) -> &'static str {
        match self {
            Self::StateAndEffects => "State & Effects",
            Self::Performance => "Performance",
            Self::Architecture => "Architecture",
            Self::Security => "Security",
            Self::Accessibility => "Accessibility",
            Self::DeadCode => "Dead Code",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::StateAndEffects,
            Self::Performance,
            Self::Architecture,
            Self::Security,
            Self::Accessibility,
            Self::DeadCode,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactRule {
    pub id: &'static str,
    pub category: ReactRuleCategory,
    pub severity: RuleSeverity,
    pub description: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactDiagnostic {
    pub rule_id: String,
    pub category: ReactRuleCategory,
    pub severity: RuleSeverity,
    pub file: String,
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryBreakdown {
    pub category: ReactRuleCategory,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactHealthReport {
    pub score: u8,
    pub label: String,
    pub total_diagnostics: usize,
    pub unique_error_rules: usize,
    pub unique_warning_rules: usize,
    pub category_breakdown: Vec<CategoryBreakdown>,
    pub diagnostics: Vec<ReactDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionAnalysis {
    pub is_suppressed: bool,
    pub suppression_hint: Option<String>,
    pub near_miss: Option<String>,
}

impl SuppressionAnalysis {
    pub fn suppressed() -> Self {
        Self { is_suppressed: true, suppression_hint: None, near_miss: None }
    }

    pub fn not_suppressed(hint: String) -> Self {
        Self { is_suppressed: false, suppression_hint: Some(hint), near_miss: None }
    }

    pub fn near_miss(reason: String) -> Self {
        Self { is_suppressed: false, suppression_hint: None, near_miss: Some(reason) }
    }
}

pub struct ReactDoctorEngine;

impl ReactDoctorEngine {
    pub fn calculate_score(diagnostics: &[ReactDiagnostic]) -> ReactHealthReport {
        let total_diagnostics = diagnostics.len();

        let mut error_rules = std::collections::BTreeSet::new();
        let mut warning_rules = std::collections::BTreeSet::new();

        for d in diagnostics {
            match d.severity {
                RuleSeverity::Error => {
                    error_rules.insert(d.rule_id.clone());
                }
                RuleSeverity::Warning => {
                    warning_rules.insert(d.rule_id.clone());
                }
            }
        }

        let unique_error_rules = error_rules.len();
        let unique_warning_rules = warning_rules.len();

        let score_f = 100.0
            - unique_error_rules as f64 * 1.5
            - unique_warning_rules as f64 * 0.75;
        let score_f = score_f.clamp(0.0, 100.0);
        let score = score_f as u8;

        let label = if score >= 75 {
            "Great"
        } else if score >= 50 {
            "Needs work"
        } else {
            "Critical"
        }
        .to_string();

        let mut breakdown = Vec::new();
        for cat in ReactRuleCategory::all() {
            let count = diagnostics
                .iter()
                .filter(|d| {
                    std::mem::discriminant(&d.category) == std::mem::discriminant(&cat)
                })
                .count();
            if count > 0 {
                breakdown.push(CategoryBreakdown { category: cat, count });
            }
        }

        ReactHealthReport {
            score,
            label,
            total_diagnostics,
            unique_error_rules,
            unique_warning_rules,
            category_breakdown: breakdown,
            diagnostics: diagnostics.to_vec(),
        }
    }

    pub fn builtin_rules() -> Vec<ReactRule> {
        vec![
            // State & Effects (6 rules)
            ReactRule {
                id: "no-cascading-set-state",
                category: ReactRuleCategory::StateAndEffects,
                severity: RuleSeverity::Error,
                description: "Avoid cascading setState calls in useEffect that cause re-render chains",
            },
            ReactRule {
                id: "no-derived-useState",
                category: ReactRuleCategory::StateAndEffects,
                severity: RuleSeverity::Warning,
                description: "Derived state should use useMemo instead of redundant useState",
            },
            ReactRule {
                id: "no-fetch-in-effect",
                category: ReactRuleCategory::StateAndEffects,
                severity: RuleSeverity::Error,
                description: "Fetch requests in useEffect should be wrapped in a data-fetching library or custom hook",
            },
            ReactRule {
                id: "no-stale-callback",
                category: ReactRuleCategory::StateAndEffects,
                severity: RuleSeverity::Error,
                description: "Callbacks with stale closures should include updated dependencies",
            },
            ReactRule {
                id: "no-missing-deps",
                category: ReactRuleCategory::StateAndEffects,
                severity: RuleSeverity::Error,
                description: "useEffect/useCallback/useMemo is missing required dependencies",
            },
            ReactRule {
                id: "rerender-state-only-in-handlers",
                category: ReactRuleCategory::StateAndEffects,
                severity: RuleSeverity::Warning,
                description: "State updates that trigger re-renders should be in event handlers, not during render",
            },
            // Performance (3 rules)
            ReactRule {
                id: "no-array-index-as-key",
                category: ReactRuleCategory::Performance,
                severity: RuleSeverity::Warning,
                description: "Using array index as key can cause rendering bugs and poor reconciliation",
            },
            ReactRule {
                id: "no-unnecessary-memo",
                category: ReactRuleCategory::Performance,
                severity: RuleSeverity::Warning,
                description: "useMemo/useCallback wrapping simple computations adds overhead without benefit",
            },
            ReactRule {
                id: "no-large-components",
                category: ReactRuleCategory::Performance,
                severity: RuleSeverity::Warning,
                description: "Components over 300 lines should be split into smaller units",
            },
            // Architecture (3 rules)
            ReactRule {
                id: "no-barrel-import",
                category: ReactRuleCategory::Architecture,
                severity: RuleSeverity::Warning,
                description: "Barrel index.ts imports cause tree-shaking issues and circular dependencies",
            },
            ReactRule {
                id: "no-direct-dom-access",
                category: ReactRuleCategory::Architecture,
                severity: RuleSeverity::Error,
                description: "Direct DOM access (document.querySelector) breaks SSR and React abstraction",
            },
            ReactRule {
                id: "no-hooks-in-conditions",
                category: ReactRuleCategory::Architecture,
                severity: RuleSeverity::Error,
                description: "React hooks must not be called inside conditions, loops, or nested functions",
            },
            // Security (3 rules)
            ReactRule {
                id: "no-dangerous-html",
                category: ReactRuleCategory::Security,
                severity: RuleSeverity::Error,
                description: "dangerouslySetInnerHTML without sanitization opens XSS vulnerabilities",
            },
            ReactRule {
                id: "no-suspicious-link",
                category: ReactRuleCategory::Security,
                severity: RuleSeverity::Warning,
                description: "Anchor tags with href='#' or javascript: URIs should use button elements instead",
            },
            ReactRule {
                id: "no-hardcoded-secrets",
                category: ReactRuleCategory::Security,
                severity: RuleSeverity::Error,
                description: "Hardcoded API keys, tokens, or passwords detected in source code",
            },
            // Accessibility (3 rules)
            ReactRule {
                id: "no-missing-alt",
                category: ReactRuleCategory::Accessibility,
                severity: RuleSeverity::Error,
                description: "Image elements must have alt text for screen readers",
            },
            ReactRule {
                id: "no-missing-aria",
                category: ReactRuleCategory::Accessibility,
                severity: RuleSeverity::Warning,
                description: "Interactive elements should have appropriate ARIA attributes",
            },
            ReactRule {
                id: "no-non-interactive-tabindex",
                category: ReactRuleCategory::Accessibility,
                severity: RuleSeverity::Warning,
                description: "Non-interactive elements should not have positive tabIndex values",
            },
            // Dead Code (3 rules)
            ReactRule {
                id: "unused-import",
                category: ReactRuleCategory::DeadCode,
                severity: RuleSeverity::Warning,
                description: "Unused imports increase bundle size and reduce readability",
            },
            ReactRule {
                id: "unused-component",
                category: ReactRuleCategory::DeadCode,
                severity: RuleSeverity::Warning,
                description: "Exported component is never imported anywhere in the project",
            },
            ReactRule {
                id: "unreachable-code",
                category: ReactRuleCategory::DeadCode,
                severity: RuleSeverity::Error,
                description: "Dead code after early return or conditional branch that can never execute",
            },
        ]
    }

    pub fn detect_react_project(root: &str) -> bool {
        let path = std::path::Path::new(root).join("package.json");
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return false,
        };
        content.contains("\"react\"") || content.contains("'react'")
    }

    pub fn explain_diagnostic(
        diagnostic: &ReactDiagnostic,
        source_lines: &[String],
    ) -> SuppressionAnalysis {
        let line_idx = diagnostic.line.saturating_sub(1);
        if line_idx >= source_lines.len() {
            return SuppressionAnalysis::not_suppressed("Line out of range".to_string());
        }

        let current_line = &source_lines[line_idx];
        let disable_comment = format!("react-doctor-disable-next-line {}", diagnostic.rule_id);

        if line_idx > 0 {
            let prev = &source_lines[line_idx - 1];
            if prev.contains(&disable_comment) {
                return SuppressionAnalysis::suppressed();
            }
            if prev.contains("react-doctor-disable-next-line") {
                let listed = prev.split("react-doctor-disable-next-line").nth(1).unwrap_or("");
                if listed.contains(&diagnostic.rule_id) {
                    return SuppressionAnalysis::suppressed();
                }
                return SuppressionAnalysis::near_miss(format!(
                    "Adjacent suppression lists different rules: {}. Use comma form to add {}",
                    listed.trim(),
                    diagnostic.rule_id
                ));
            }
            if prev.contains("react-doctor-disable-line") && current_line.contains("react-doctor-disable-line") {
                return SuppressionAnalysis::suppressed();
            }
        }

        if current_line.contains(&format!("react-doctor-disable-line {}", diagnostic.rule_id))
            || current_line.contains("react-doctor-disable-next-line")
        {
            return SuppressionAnalysis::suppressed();
        }

        SuppressionAnalysis::not_suppressed(format!(
            "No suppression comment found above line {}. Add: // react-doctor-disable-next-line {}",
            diagnostic.line, diagnostic.rule_id
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_diag(rule_id: &str, category: ReactRuleCategory, severity: RuleSeverity) -> ReactDiagnostic {
        ReactDiagnostic {
            rule_id: rule_id.to_string(),
            category,
            severity,
            file: "src/App.tsx".to_string(),
            line: 10,
            message: format!("violated: {}", rule_id),
        }
    }

    #[test]
    fn test_calculate_score_great() {
        let diags = vec![];
        let report = ReactDoctorEngine::calculate_score(&diags);
        assert_eq!(report.score, 100);
        assert_eq!(report.label, "Great");
        assert_eq!(report.total_diagnostics, 0);
    }

    #[test]
    fn test_calculate_score_needs_work() {
        let mut diags = Vec::new();
        for i in 0..20 {
            diags.push(make_diag(
                &format!("error-{}", i),
                ReactRuleCategory::StateAndEffects,
                RuleSeverity::Error,
            ));
        }
        for i in 0..10 {
            diags.push(make_diag(
                &format!("warning-{}", i),
                ReactRuleCategory::Performance,
                RuleSeverity::Warning,
            ));
        }
        let report = ReactDoctorEngine::calculate_score(&diags);
        // 100 - 20*1.5 - 10*0.75 = 100 - 30 - 7.5 = 62.5
        assert!(report.score >= 50 && report.score <= 74, "score {} should be 50-74", report.score);
        assert_eq!(report.label, "Needs work");
    }

    #[test]
    fn test_calculate_score_critical() {
        let mut diags = Vec::new();
        for i in 0..70 {
            diags.push(make_diag(
                &format!("error-{}", i),
                ReactRuleCategory::StateAndEffects,
                RuleSeverity::Error,
            ));
        }
        let report = ReactDoctorEngine::calculate_score(&diags);
        // 100 - 70*1.5 = 100 - 105 = -5 → clamped to 0
        assert_eq!(report.score, 0);
        assert_eq!(report.label, "Critical");
    }

    #[test]
    fn test_score_clamped() {
        let mut diags = Vec::new();
        for i in 0..100 {
            diags.push(make_diag(
                &format!("error-{}", i),
                ReactRuleCategory::StateAndEffects,
                RuleSeverity::Error,
            ));
        }
        let report = ReactDoctorEngine::calculate_score(&diags);
        assert_eq!(report.score, 0);
        assert_eq!(report.label, "Critical");
    }

    #[test]
    fn test_explain_suppressed() {
        let mut diag = make_diag("no-fetch-in-effect", ReactRuleCategory::StateAndEffects, RuleSeverity::Error);
        diag.line = 2;
        let lines = vec![
            "// react-doctor-disable-next-line no-fetch-in-effect".to_string(),
            "useEffect(() => { fetch('/api') }, [])".to_string(),
        ];
        let analysis = ReactDoctorEngine::explain_diagnostic(&diag, &lines);
        assert!(analysis.is_suppressed);
    }

    #[test]
    fn test_explain_not_suppressed() {
        let mut diag = make_diag("no-fetch-in-effect", ReactRuleCategory::StateAndEffects, RuleSeverity::Error);
        diag.line = 1;
        let lines = vec![
            "useEffect(() => { fetch('/api') }, [])".to_string(),
        ];
        let analysis = ReactDoctorEngine::explain_diagnostic(&diag, &lines);
        assert!(!analysis.is_suppressed);
        assert!(analysis.suppression_hint.is_some());
    }

    #[test]
    fn test_explain_near_miss() {
        let mut diag = make_diag("no-fetch-in-effect", ReactRuleCategory::StateAndEffects, RuleSeverity::Error);
        diag.line = 2;
        let lines = vec![
            "// react-doctor-disable-next-line no-array-index-as-key".to_string(),
            "useEffect(() => { fetch('/api') }, [])".to_string(),
        ];
        let analysis = ReactDoctorEngine::explain_diagnostic(&diag, &lines);
        assert!(!analysis.is_suppressed);
        assert!(analysis.near_miss.is_some());
    }

    #[test]
    fn test_builtin_rules_count() {
        let rules = ReactDoctorEngine::builtin_rules();
        assert!(rules.len() >= 20, "only {} rules", rules.len());
    }

    #[test]
    fn test_all_categories_covered() {
        let rules = ReactDoctorEngine::builtin_rules();
        let mut categories = std::collections::HashSet::new();
        for rule in &rules {
            categories.insert(format!("{:?}", rule.category));
        }
        for cat in ReactRuleCategory::all() {
            assert!(
                categories.contains(&format!("{:?}", cat)),
                "missing rules for {:?}",
                cat
            );
        }
    }
}
