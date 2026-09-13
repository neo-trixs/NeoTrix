//! Governance Compliance Checker — 治理合规检查器
//!
//! 吸收 KB 经验:
//! - 禁止推送到远程仓库
//! - 禁止杀用户未授权的进程
//! - P0 编译闸门
//! - 预提交钩子
//! - 分支保护

use serde::{Deserialize, Serialize};

/// 治理合规检查器
pub struct _GovernanceComplianceChecker {
    rules: Vec<_GovernanceRule>,
    violations: Vec<ComplianceViolation>,
    #[allow(dead_code)]
    config: _ComplianceConfig,
    stats: _ComplianceStats,
}

/// 合规配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _ComplianceConfig {
    pub enable_pre_commit_checks: bool,
    pub enable_pre_push_checks: bool,
    pub enable_runtime_checks: bool,
    pub block_on_violation: bool,
    pub max_violations: usize,
}

impl Default for _ComplianceConfig {
    fn default() -> Self {
        Self {
            enable_pre_commit_checks: true,
            enable_pre_push_checks: true,
            enable_runtime_checks: true,
            block_on_violation: true,
            max_violations: 10,
        }
    }
}

/// 治理规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _GovernanceRule {
    pub rule_id: String,
    pub name: String,
    pub description: String,
    pub category: RuleCategory,
    pub severity: RuleSeverity,
    pub enabled: bool,
    pub check_fn: String,
}

/// 规则分类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuleCategory {
    Security,        // 安全规则
    BuildGate,       // 构建闸门
    ProcessControl,  // 进程控制
    CodeQuality,     // 代码质量
    KnowledgeBase,   // 知识库规则
}

/// 规则严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RuleSeverity {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

/// 合规违反
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceViolation {
    pub violation_id: String,
    pub rule_id: String,
    pub rule_name: String,
    pub message: String,
    pub severity: RuleSeverity,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub resolved: bool,
}

/// 合规统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _ComplianceStats {
    pub total_checks: u64,
    pub passed_checks: u64,
    pub failed_checks: u64,
    pub total_violations: u64,
    pub resolved_violations: u64,
}

/// 检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _ComplianceCheckResult {
    pub passed: bool,
    pub violations: Vec<ComplianceViolation>,
    pub suggestions: Vec<String>,
}

impl _GovernanceComplianceChecker {
    /// Create a new governance compliance checker with default rules.
    ///
    /// Note: Real implementation needs — rules are hardcoded in register_default_rules().
    /// Consider: loading rules from config file, dynamic rule registration via EventBus,
    /// and rule versioning for backward compatibility.
    pub fn new() -> Self {
        let mut checker = Self {
            rules: Vec::new(),
            violations: Vec::new(),
            config: _ComplianceConfig::default(),
            stats: _ComplianceStats {
                total_checks: 0,
                passed_checks: 0,
                failed_checks: 0,
                total_violations: 0,
                resolved_violations: 0,
            },
        };
        checker.register_default_rules();
        checker
    }

    /// Register default governance rules (no_remote_push, no_unauthorized_kill, etc.).
    ///
    /// Note: Real implementation needs — rules are hardcoded with string-based check_fn names.
    /// Consider: rule loading from config file, dynamic rule registration via EventBus,
    /// and rule versioning for backward compatibility.
    fn register_default_rules(&mut self) {
        // 规则1: 禁止推送到远程仓库
        self.rules.push(_GovernanceRule {
            rule_id: "no_remote_push".into(),
            name: "禁止推送到远程仓库".into(),
            description: "用户明确要求禁止将核心代码推送到远程 GitHub 仓库".into(),
            category: RuleCategory::Security,
            severity: RuleSeverity::Critical,
            enabled: true,
            check_fn: "check_no_remote_push".into(),
        });

        // 规则2: 禁止杀未授权进程
        self.rules.push(_GovernanceRule {
            rule_id: "no_unauthorized_kill".into(),
            name: "禁止杀未授权进程".into(),
            description: "禁止在用户未明确点名的情况下杀其他进程".into(),
            category: RuleCategory::ProcessControl,
            severity: RuleSeverity::Critical,
            enabled: true,
            check_fn: "check_no_unauthorized_kill".into(),
        });

        // 规则3: P0 编译闸门 - 提交前
        self.rules.push(_GovernanceRule {
            rule_id: "pre_commit_build_gate".into(),
            name: "P0 编译闸门 (提交前)".into(),
            description: "提交前必须通过 cargo check --tests".into(),
            category: RuleCategory::BuildGate,
            severity: RuleSeverity::High,
            enabled: true,
            check_fn: "check_pre_commit_build".into(),
        });

        // 规则4: P0 编译闸门 - 推送前
        self.rules.push(_GovernanceRule {
            rule_id: "pre_push_build_gate".into(),
            name: "P0 编译闸门 (推送前)".into(),
            description: "推送前必须通过 cargo check --lib".into(),
            category: RuleCategory::BuildGate,
            severity: RuleSeverity::High,
            enabled: true,
            check_fn: "check_pre_push_build".into(),
        });

        // 规则5: 代码质量 - 禁止 unsafe
        self.rules.push(_GovernanceRule {
            rule_id: "no_unsafe_code".into(),
            name: "禁止 unsafe 代码".into(),
            description: "核心代码禁止使用 unsafe (R-P1)".into(),
            category: RuleCategory::CodeQuality,
            severity: RuleSeverity::High,
            enabled: true,
            check_fn: "check_no_unsafe".into(),
        });

        // 规则6: 知识库写入门禁
        self.rules.push(_GovernanceRule {
            rule_id: "kb_write_gate".into(),
            name: "知识库写入门禁".into(),
            description: "AGENTS.md 禁止内联经验表、cycle 正文或增长区".into(),
            category: RuleCategory::KnowledgeBase,
            severity: RuleSeverity::Medium,
            enabled: true,
            check_fn: "check_kb_write_gate".into(),
        });
    }

    /// Execute compliance check for all enabled rules against a check_type string.
    ///
    /// Note: Real implementation needs — the check_type is a free-form string matched
    /// against rule patterns. Consider: structured check contexts (git diff, command
    /// AST), rule dependency ordering, and parallel rule evaluation for performance.
    pub fn check(&mut self, check_type: &str) -> _ComplianceCheckResult {
        self.stats.total_checks += 1;

        let mut violations = Vec::new();
        let mut suggestions = Vec::new();

        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }

            let passed = self.check_rule(rule, check_type);
            if !passed {
                let violation = ComplianceViolation {
                    violation_id: uuid::Uuid::new_v4().to_string(),
                    rule_id: rule.rule_id.clone(),
                    rule_name: rule.name.clone(),
                    message: format!("违反规则: {}", rule.description),
                    severity: rule.severity.clone(),
                    timestamp: chrono::Utc::now(),
                    resolved: false,
                };

                self.stats.total_violations += 1;
                violations.push(violation);
            }
        }

        if violations.is_empty() {
            self.stats.passed_checks += 1;
        } else {
            self.stats.failed_checks += 1;
            suggestions.push("修复违规后再继续".into());
        }

        _ComplianceCheckResult {
            passed: violations.is_empty(),
            violations,
            suggestions,
        }
    }

    /// Evaluate a single governance rule against the check_type.
    ///
    /// Build gate rules (`check_pre_commit_build`, `check_pre_push_build`) expect
    /// the caller to provide build evidence in `check_type` (e.g., "cargo_check_passed").
    /// If no build evidence is present, the rule fails — it does not fabricate a pass.
    ///
    /// Note: The actual `cargo check` / `cargo test` execution is NOT performed here.
    /// This checker only validates that the caller has declared build status.
    /// Real implementation needs: integration with build system for compile-time checks,
    /// or structured check contexts (git diff, command AST).
    fn check_rule(&self, rule: &_GovernanceRule, check_type: &str) -> bool {
        match rule.check_fn.as_str() {
            "check_no_remote_push" => {
                // 检查是否有 git push 命令
                !check_type.contains("push")
            }
            "check_no_unauthorized_kill" => {
                // 检查是否有 kill 命令
                !check_type.contains("kill")
            }
            "check_pre_commit_build" => {
                // 需要调用方提供编译通过的证据
                check_type.contains("cargo_check_passed") || check_type.contains("build_ok")
            }
            "check_pre_push_build" => {
                // 需要调用方提供编译通过的证据
                check_type.contains("cargo_check_passed") || check_type.contains("build_ok")
            }
            "check_no_unsafe" => {
                // 检查 unsafe 代码
                !check_type.contains("unsafe")
            }
            "check_kb_write_gate" => {
                // 检查 KB 写入
                !check_type.contains("experience") || !check_type.contains("inline")
            }
            _ => true,
        }
    }

    /// 解决违规
    ///
    /// Note: Real implementation needs — resolution is tracked but not persisted.
    /// Consider: resolution reason logging, audit trail, and integration with
    /// compliance reporting for governance dashboards.
    pub(crate) fn _resolve_violation(&mut self, violation_id: &str) -> bool {
        if let Some(violation) = self.violations.iter_mut().find(|v| v.violation_id == violation_id) {
            violation.resolved = true;
            self.stats.resolved_violations += 1;
            true
        } else {
            false
        }
    }

    /// 获取所有规则
    ///
    /// Note: Real implementation needs — returns reference to in-memory vector.
    /// Consider: filtering by category/severity, rule versioning, and persistence
    /// to KB for cross-session rule tracking.
    pub fn rules(&self) -> &[_GovernanceRule] {
        &self.rules
    }

    /// 获取统计信息
    ///
    /// Note: Real implementation needs — stats are computed from in-memory state.
    /// For production: maintain running aggregates for O(1) access, and expose
    /// metrics via EventBus for telemetry integration.
    pub fn stats(&self) -> &_ComplianceStats {
        &self.stats
    }
}
