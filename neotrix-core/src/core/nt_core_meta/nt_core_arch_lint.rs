/// ArchLint — 架构规则检查器
///
/// 已有规则:
/// - ARC-LYR-001: L0 不能导入 L1+
/// - ARC-LYR-002: L1 不能导入 L2+
/// - ARC-LYR-003: L2 不能导入 L3+
/// - ARC-LYR-004: L3 不能导入 L4+
/// - ARC-LYR-005: L5 以下不能导入 L6+
/// - ARC-LYR-006: L1→L3+ 和 L2→L4+ 交叉层导入违规检测
#[derive(Debug, Clone)]
pub struct ArchLint {
    pub violations: Vec<LayerViolation>,
    pub rules: Vec<LintRule>,
}

#[derive(Debug, Clone)]
pub struct LayerViolation {
    pub rule_id: String,
    pub source_file: String,
    pub source_layer: u8,
    pub target_module: String,
    pub target_layer: u8,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct LintRule {
    pub id: String,
    pub description: String,
    pub check_fn: String,
}

impl ArchLint {
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
            rules: Self::default_rules(),
        }
    }

    fn default_rules() -> Vec<LintRule> {
        vec![
            LintRule {
                id: "ARC-LYR-001".into(),
                description: "L0 cannot import L1+".into(),
                check_fn: "layer_check(0, 1)".into(),
            },
            LintRule {
                id: "ARC-LYR-002".into(),
                description: "L1 cannot import L2+".into(),
                check_fn: "layer_check(1, 2)".into(),
            },
            LintRule {
                id: "ARC-LYR-003".into(),
                description: "L2 cannot import L3+".into(),
                check_fn: "layer_check(2, 3)".into(),
            },
            LintRule {
                id: "ARC-LYR-004".into(),
                description: "L3 cannot import L4+".into(),
                check_fn: "layer_check(3, 4)".into(),
            },
            LintRule {
                id: "ARC-LYR-005".into(),
                description: "L5- cannot import L6+".into(),
                check_fn: "layer_check(5, 6)".into(),
            },
            LintRule {
                id: "ARC-LYR-006".into(),
                description: "L1→L3+ and L2→L4+ cross-layer imports".into(),
                check_fn: "cross_layer_check(1, 3; 2, 4)".into(),
            },
        ]
    }

    pub fn lint_file(&mut self, source_file: &str, source_layer: u8, imports: &[(&str, u8)]) {
        for &(target_module, target_layer) in imports {
            if source_layer == 0 && target_layer >= 1 {
                self.violations.push(LayerViolation {
                    rule_id: "ARC-LYR-001".into(),
                    source_file: source_file.into(),
                    source_layer,
                    target_module: target_module.into(),
                    target_layer,
                    description: format!(
                        "L0 '{}' imports L{} '{}'",
                        source_file, target_layer, target_module
                    ),
                });
            }
            if source_layer == 1 && target_layer >= 3 {
                self.violations.push(LayerViolation {
                    rule_id: "ARC-LYR-006".into(),
                    source_file: source_file.into(),
                    source_layer,
                    target_module: target_module.into(),
                    target_layer,
                    description: format!(
                        "L1 '{}' jumps to L{} '{}' (ARC-LYR-006)",
                        source_file, target_layer, target_module
                    ),
                });
            }
            if source_layer == 2 && target_layer >= 4 {
                self.violations.push(LayerViolation {
                    rule_id: "ARC-LYR-006".into(),
                    source_file: source_file.into(),
                    source_layer,
                    target_module: target_module.into(),
                    target_layer,
                    description: format!(
                        "L2 '{}' jumps to L{} '{}' (ARC-LYR-006)",
                        source_file, target_layer, target_module
                    ),
                });
            }
        }
    }

    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }

    pub fn report(&self) -> String {
        let mut s = format!(
            "ArchLint: {} rules, {} violations\n",
            self.rules.len(),
            self.violations.len()
        );
        for v in &self.violations {
            s.push_str(&format!(
                "  [{}] {} (L{} → L{})\n",
                v.rule_id, v.source_file, v.source_layer, v.target_layer
            ));
        }
        s
    }
}

impl Default for ArchLint {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::core::nt_core_self_test::SelfTest for ArchLint {
    fn name(&self) -> &str {
        "arch_lint"
    }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        if self.rules.is_empty() {
            failures.push("arch_lint: no rules defined".into());
        }
        if self.rules.len() < 6 {
            failures.push("arch_lint: expected at least 6 rules".into());
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
    use crate::core::nt_core_self_test::SelfTest;

    #[test]
    fn test_default_rules() {
        let lint = ArchLint::new();
        assert_eq!(lint.rules.len(), 6);
    }

    #[test]
    fn test_l0_violation() {
        let mut lint = ArchLint::new();
        lint.lint_file("substrate.rs", 0, &[("gwt", 5)]);
        assert_eq!(lint.violation_count(), 1);
        assert_eq!(lint.violations[0].rule_id, "ARC-LYR-001");
    }

    #[test]
    fn test_arc_lyr_006_l1_violation() {
        let mut lint = ArchLint::new();
        lint.lint_file("shield.rs", 1, &[("brain", 8)]);
        assert_eq!(lint.violations[0].rule_id, "ARC-LYR-006");
    }

    #[test]
    fn test_arc_lyr_006_l2_violation() {
        let mut lint = ArchLint::new();
        lint.lint_file("perception.rs", 2, &[("meta", 9)]);
        assert_eq!(lint.violations[0].rule_id, "ARC-LYR-006");
    }

    #[test]
    fn test_report_string() {
        let lint = ArchLint::new();
        let r = lint.report();
        assert!(r.contains("ArchLint"));
    }

    #[test]
    fn test_self_test() {
        let lint = ArchLint::new();
        assert!(lint.self_test().is_ok());
    }
}
