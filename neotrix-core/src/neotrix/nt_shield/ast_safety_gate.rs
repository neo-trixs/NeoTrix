use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    Read,
    Write,
    Exec,
    ReadWrite,
    ReadExec,
    All,
}

impl Permission {
    pub fn bits(&self) -> u8 {
        match self {
            Permission::Read => 0b001,
            Permission::Write => 0b010,
            Permission::Exec => 0b100,
            Permission::ReadWrite => 0b011,
            Permission::ReadExec => 0b101,
            Permission::All => 0b111,
        }
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Permission::Read => write!(f, "read"),
            Permission::Write => write!(f, "write"),
            Permission::Exec => write!(f, "exec"),
            Permission::ReadWrite => write!(f, "read+write"),
            Permission::ReadExec => write!(f, "read+exec"),
            Permission::All => write!(f, "all"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstNodeType {
    FunctionCall,
    VariableAssignment,
    ModuleImport,
    FileWrite,
    NetworkAccess,
    ProcessSpawn,
    MemoryModify,
    CodeEval,
    UnsafeBlock,
    MacroInvocation,
    Dereference,
    TypeCast,
}

impl fmt::Display for AstNodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstNodeType::FunctionCall => write!(f, "FunctionCall"),
            AstNodeType::VariableAssignment => write!(f, "VariableAssignment"),
            AstNodeType::ModuleImport => write!(f, "ModuleImport"),
            AstNodeType::FileWrite => write!(f, "FileWrite"),
            AstNodeType::NetworkAccess => write!(f, "NetworkAccess"),
            AstNodeType::ProcessSpawn => write!(f, "ProcessSpawn"),
            AstNodeType::MemoryModify => write!(f, "MemoryModify"),
            AstNodeType::CodeEval => write!(f, "CodeEval"),
            AstNodeType::UnsafeBlock => write!(f, "UnsafeBlock"),
            AstNodeType::MacroInvocation => write!(f, "MacroInvocation"),
            AstNodeType::Dereference => write!(f, "Dereference"),
            AstNodeType::TypeCast => write!(f, "TypeCast"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstPattern {
    pub node_type: AstNodeType,
    pub target_pattern: Option<String>,
    pub required_permission: Permission,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuleAction {
    Allow,
    Deny,
    RequireReview,
    LogOnly,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SeverityLevel {
    Info,
    Warning,
    Error,
    Critical,
}

impl fmt::Display for SeverityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SeverityLevel::Info => write!(f, "Info"),
            SeverityLevel::Warning => write!(f, "Warning"),
            SeverityLevel::Error => write!(f, "Error"),
            SeverityLevel::Critical => write!(f, "Critical"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SafetyRule {
    pub name: String,
    pub pattern: AstPattern,
    pub action: RuleAction,
    pub severity: SeverityLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PermissionTier {
    pub tier_name: String,
    pub permissions: Vec<Permission>,
    pub description: String,
}

impl PermissionTier {
    pub fn allows(&self, permission: &Permission) -> bool {
        let required_bits = permission.bits();
        self.permissions
            .iter()
            .any(|p| p.bits() & required_bits == required_bits)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SafetyViolation {
    pub rule_name: String,
    pub pattern: String,
    pub severity: SeverityLevel,
    pub message: String,
    pub line_estimate: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SafetyVerdict {
    pub passed: bool,
    pub violations: Vec<SafetyViolation>,
    pub permission_tier: String,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct AstSafetyGate {
    pub rules: Vec<SafetyRule>,
    pub tiers: Vec<PermissionTier>,
    pub default_tier: String,
    pub enabled: bool,
}

impl AstSafetyGate {
    pub fn new() -> Self {
        let rules = vec![
            SafetyRule {
                name: "no-unsafe".into(),
                pattern: AstPattern {
                    node_type: AstNodeType::UnsafeBlock,
                    target_pattern: Some("unsafe".into()),
                    required_permission: Permission::Exec,
                },
                action: RuleAction::Deny,
                severity: SeverityLevel::Critical,
            },
            SafetyRule {
                name: "no-code-eval".into(),
                pattern: AstPattern {
                    node_type: AstNodeType::CodeEval,
                    target_pattern: Some("eval".into()),
                    required_permission: Permission::Exec,
                },
                action: RuleAction::RequireReview,
                severity: SeverityLevel::Error,
            },
            SafetyRule {
                name: "file-write-review".into(),
                pattern: AstPattern {
                    node_type: AstNodeType::FileWrite,
                    target_pattern: Some("write".into()),
                    required_permission: Permission::Write,
                },
                action: RuleAction::RequireReview,
                severity: SeverityLevel::Warning,
            },
            SafetyRule {
                name: "process-spawn-review".into(),
                pattern: AstPattern {
                    node_type: AstNodeType::ProcessSpawn,
                    target_pattern: Some("spawn".into()),
                    required_permission: Permission::Exec,
                },
                action: RuleAction::RequireReview,
                severity: SeverityLevel::Error,
            },
            SafetyRule {
                name: "network-log".into(),
                pattern: AstPattern {
                    node_type: AstNodeType::NetworkAccess,
                    target_pattern: Some("network".into()),
                    required_permission: Permission::Write,
                },
                action: RuleAction::LogOnly,
                severity: SeverityLevel::Info,
            },
            SafetyRule {
                name: "module-import-allow".into(),
                pattern: AstPattern {
                    node_type: AstNodeType::ModuleImport,
                    target_pattern: None,
                    required_permission: Permission::Read,
                },
                action: RuleAction::Allow,
                severity: SeverityLevel::Info,
            },
            SafetyRule {
                name: "function-call-allow".into(),
                pattern: AstPattern {
                    node_type: AstNodeType::FunctionCall,
                    target_pattern: None,
                    required_permission: Permission::Read,
                },
                action: RuleAction::Allow,
                severity: SeverityLevel::Info,
            },
            SafetyRule {
                name: "no-memory-modify".into(),
                pattern: AstPattern {
                    node_type: AstNodeType::MemoryModify,
                    target_pattern: Some("memory".into()),
                    required_permission: Permission::Exec,
                },
                action: RuleAction::Deny,
                severity: SeverityLevel::Critical,
            },
            SafetyRule {
                name: "variable-assign-allow".into(),
                pattern: AstPattern {
                    node_type: AstNodeType::VariableAssignment,
                    target_pattern: None,
                    required_permission: Permission::Write,
                },
                action: RuleAction::Allow,
                severity: SeverityLevel::Info,
            },
        ];

        let tiers = vec![
            PermissionTier {
                tier_name: "read-only".into(),
                permissions: vec![Permission::Read],
                description: "Safe observation".into(),
            },
            PermissionTier {
                tier_name: "standard".into(),
                permissions: vec![Permission::Read, Permission::Write],
                description: "Normal operations".into(),
            },
            PermissionTier {
                tier_name: "elevated".into(),
                permissions: vec![Permission::Read, Permission::Write, Permission::Exec],
                description: "Self-modification capable".into(),
            },
            PermissionTier {
                tier_name: "audit".into(),
                permissions: vec![Permission::Read],
                description: "Audit mode".into(),
            },
        ];

        Self {
            rules,
            tiers,
            default_tier: "standard".into(),
            enabled: true,
        }
    }

    pub fn add_rule(&mut self, rule: SafetyRule) {
        self.rules.push(rule);
    }

    pub fn register_tier(&mut self, tier: PermissionTier) {
        if let Some(pos) = self
            .tiers
            .iter()
            .position(|t| t.tier_name == tier.tier_name)
        {
            self.tiers[pos] = tier;
        } else {
            self.tiers.push(tier);
        }
    }

    pub fn tier(&self, name: &str) -> Option<&PermissionTier> {
        self.tiers.iter().find(|t| t.tier_name == name)
    }

    pub fn tier_allows(&self, tier_name: &str, permission: Permission) -> bool {
        self.tier(tier_name)
            .map(|t| t.allows(&permission))
            .unwrap_or(false)
    }

    pub fn check_node(
        &self,
        node: AstNodeType,
        target: Option<&str>,
        current_tier: &PermissionTier,
    ) -> Option<SafetyViolation> {
        for rule in &self.rules {
            if rule.pattern.node_type != node {
                continue;
            }
            if let Some(ref pat) = rule.pattern.target_pattern {
                if let Some(t) = target {
                    if !t.contains(pat) && !t.starts_with(pat) {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            if !current_tier.allows(&rule.pattern.required_permission) {
                let severity = match rule.action {
                    RuleAction::Deny => SeverityLevel::Critical,
                    RuleAction::RequireReview => SeverityLevel::Error,
                    RuleAction::LogOnly => SeverityLevel::Info,
                    RuleAction::Allow => SeverityLevel::Info,
                };
                return Some(SafetyViolation {
                    rule_name: rule.name.clone(),
                    pattern: node.to_string(),
                    severity,
                    message: format!(
                        "Node '{}' requires {:?} but tier '{}' lacks it",
                        node, rule.pattern.required_permission, current_tier.tier_name,
                    ),
                    line_estimate: 0,
                });
            }
            match rule.action {
                RuleAction::Deny => {
                    return Some(SafetyViolation {
                        rule_name: rule.name.clone(),
                        pattern: node.to_string(),
                        severity: SeverityLevel::Critical,
                        message: format!("Denied: {} pattern matched", rule.name),
                        line_estimate: 0,
                    });
                }
                RuleAction::RequireReview => {
                    return Some(SafetyViolation {
                        rule_name: rule.name.clone(),
                        pattern: node.to_string(),
                        severity: SeverityLevel::Error,
                        message: format!("Requires review: {} pattern matched", rule.name),
                        line_estimate: 0,
                    });
                }
                RuleAction::LogOnly | RuleAction::Allow => {}
            }
        }
        None
    }

    pub fn audit(&self, code_snippet: &str, tier_name: &str) -> SafetyVerdict {
        if !self.enabled {
            return SafetyVerdict {
                passed: true,
                violations: vec![],
                permission_tier: tier_name.to_string(),
                summary: "Gate disabled — all operations allowed".into(),
            };
        }

        let current_tier = self.tier(tier_name).cloned().unwrap_or_else(|| {
            self.tier(&self.default_tier)
                .cloned()
                .unwrap_or_else(|| PermissionTier {
                    tier_name: "none".into(),
                    permissions: vec![],
                    description: "Fallback — no permissions".into(),
                })
        });

        let code_lower = code_snippet.to_lowercase();
        let lines: Vec<&str> = code_snippet.lines().collect();

        let mut violations = Vec::new();
        let nodes_to_check: Vec<(AstNodeType, Option<&'static str>)> = vec![
            (AstNodeType::UnsafeBlock, Some("unsafe")),
            (AstNodeType::CodeEval, Some("eval")),
            (AstNodeType::FileWrite, Some("write")),
            (AstNodeType::ProcessSpawn, Some("spawn")),
            (AstNodeType::NetworkAccess, Some("network")),
            (AstNodeType::MemoryModify, Some("memory")),
            (AstNodeType::ModuleImport, Some("use ")),
            (AstNodeType::ModuleImport, Some("mod ")),
            (AstNodeType::VariableAssignment, Some("let ")),
            (AstNodeType::VariableAssignment, Some("mut ")),
            (AstNodeType::FunctionCall, Some("fn ")),
            (AstNodeType::MacroInvocation, Some("!")),
            (AstNodeType::Dereference, Some("*")),
            (AstNodeType::TypeCast, Some("as ")),
        ];

        for (node_type, keyword) in &nodes_to_check {
            if let Some(kw) = keyword {
                if !code_lower.contains(kw) {
                    continue;
                }
            }
            for (line_idx, _line) in lines.iter().enumerate() {
                let target_str = keyword.unwrap_or("");
                if !_line.to_lowercase().contains(target_str) {
                    continue;
                }
                if let Some(violation) =
                    self.check_node(node_type.clone(), Some(target_str), &current_tier)
                {
                    let mut v = violation;
                    v.line_estimate = line_idx + 1;
                    violations.push(v);
                }
            }
        }

        let critical_count = violations
            .iter()
            .filter(|v| v.severity == SeverityLevel::Critical)
            .count();
        let error_count = violations
            .iter()
            .filter(|v| v.severity == SeverityLevel::Error)
            .count();
        let warning_count = violations
            .iter()
            .filter(|v| v.severity == SeverityLevel::Warning)
            .count();

        let blocked = violations
            .iter()
            .any(|v| matches!(v.severity, SeverityLevel::Critical | SeverityLevel::Error));

        let summary = if !self.enabled {
            "Gate disabled — all operations allowed".into()
        } else if blocked {
            format!(
                "BLOCKED: {} critical, {} error, {} warning violations",
                critical_count, error_count, warning_count
            )
        } else if !violations.is_empty() {
            format!(
                "PASSED with warnings: {} info/warning violations",
                violations.len()
            )
        } else {
            "PASSED: no violations detected".into()
        };

        SafetyVerdict {
            passed: !blocked,
            violations,
            permission_tier: tier_name.to_string(),
            summary,
        }
    }

    pub fn summary(&self) -> String {
        let rule_lines: Vec<String> = self
            .rules
            .iter()
            .map(|r| {
                format!(
                    "  [{}] {:?} → action={:?} severity={}",
                    r.name, r.pattern.node_type, r.action, r.severity
                )
            })
            .collect();
        let tier_lines: Vec<String> = self
            .tiers
            .iter()
            .map(|t| {
                let perms: Vec<String> = t.permissions.iter().map(|p| p.to_string()).collect();
                format!(
                    "  {}: {} ({})",
                    t.tier_name,
                    perms.join(", "),
                    t.description
                )
            })
            .collect();

        format!(
            "AstSafetyGate (enabled={})\nRules ({}):\n{}\nTiers ({}):\n{}\nDefault tier: {}",
            self.enabled,
            self.rules.len(),
            rule_lines.join("\n"),
            self.tiers.len(),
            tier_lines.join("\n"),
            self.default_tier,
        )
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Default for AstSafetyGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_has_default_rules() {
        let gate = AstSafetyGate::new();
        assert!(!gate.rules.is_empty());
        assert_eq!(gate.rules.len(), 9);
    }

    #[test]
    fn test_new_has_default_tiers() {
        let gate = AstSafetyGate::new();
        assert!(!gate.tiers.is_empty());
        assert_eq!(gate.tiers.len(), 4);
    }

    #[test]
    fn test_audit_passes_with_no_violations() {
        let gate = AstSafetyGate::new();
        let code = "fn hello() { println!(\"hi\"); }";
        let verdict = gate.audit(code, "elevated");
        assert!(verdict.passed);
    }

    #[test]
    fn test_audit_detects_unsafe_block() {
        let gate = AstSafetyGate::new();
        let code = "unsafe { *ptr = 42; }";
        let verdict = gate.audit(code, "standard");
        assert!(!verdict.passed);
        assert!(verdict
            .violations
            .iter()
            .any(|v| v.rule_name == "no-unsafe"));
    }

    #[test]
    fn test_audit_detects_file_write() {
        let gate = AstSafetyGate::new();
        let code = "std::fs::write(\"path\", data).unwrap();";
        let verdict = gate.audit(code, "read-only");
        assert!(verdict
            .violations
            .iter()
            .any(|v| v.rule_name == "file-write-review"));
    }

    #[test]
    fn test_tier_allows_correct() {
        let gate = AstSafetyGate::new();
        assert!(gate.tier_allows("elevated", Permission::Exec));
        assert!(gate.tier_allows("standard", Permission::Read));
        assert!(gate.tier_allows("standard", Permission::Write));
    }

    #[test]
    fn test_tier_denies_missing_permission() {
        let gate = AstSafetyGate::new();
        assert!(!gate.tier_allows("read-only", Permission::Write));
        assert!(!gate.tier_allows("read-only", Permission::Exec));
        assert!(!gate.tier_allows("standard", Permission::Exec));
    }

    #[test]
    fn test_add_rule() {
        let mut gate = AstSafetyGate::new();
        let rule = SafetyRule {
            name: "custom".into(),
            pattern: AstPattern {
                node_type: AstNodeType::MacroInvocation,
                target_pattern: None,
                required_permission: Permission::Read,
            },
            action: RuleAction::LogOnly,
            severity: SeverityLevel::Info,
        };
        gate.add_rule(rule);
        assert_eq!(gate.rules.len(), 10);
    }

    #[test]
    fn test_register_tier() {
        let mut gate = AstSafetyGate::new();
        let tier = PermissionTier {
            tier_name: "custom".into(),
            permissions: vec![Permission::Exec],
            description: "Custom tier".into(),
        };
        gate.register_tier(tier);
        assert_eq!(gate.tiers.len(), 5);
        assert!(gate.tier("custom").is_some());
    }

    #[test]
    fn test_check_node_returns_violation() {
        let gate = AstSafetyGate::new();
        let tier = gate.tier("read-only").unwrap();
        let violation = gate.check_node(AstNodeType::UnsafeBlock, Some("unsafe"), tier);
        assert!(violation.is_some());
    }

    #[test]
    fn test_summary_not_empty() {
        let gate = AstSafetyGate::new();
        let s = gate.summary();
        assert!(!s.is_empty());
        assert!(s.contains("Rules (9)"));
        assert!(s.contains("Tiers (4)"));
    }

    #[test]
    fn test_set_enabled() {
        let mut gate = AstSafetyGate::new();
        assert!(gate.enabled);
        gate.set_enabled(false);
        let code = "unsafe { *ptr = 42; }";
        let verdict = gate.audit(code, "standard");
        assert!(verdict.passed);
        assert_eq!(verdict.summary, "Gate disabled — all operations allowed");
    }

    #[test]
    fn test_permission_bits() {
        assert_eq!(Permission::Read.bits(), 0b001);
        assert_eq!(Permission::Write.bits(), 0b010);
        assert_eq!(Permission::Exec.bits(), 0b100);
        assert_eq!(Permission::All.bits(), 0b111);
    }

    #[test]
    fn test_elevated_tier_allows_exec() {
        let gate = AstSafetyGate::new();
        let tier = gate.tier("elevated").unwrap();
        assert!(tier.allows(&Permission::Exec));
        assert!(tier.allows(&Permission::ReadWrite));
        assert!(tier.allows(&Permission::All));
    }
}
