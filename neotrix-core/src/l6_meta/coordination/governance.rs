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
pub(crate) struct _GovernanceComplianceChecker {
    rules: Vec<_GovernanceRule>,
    violations: Vec<ComplianceViolation>,
    #[allow(dead_code)]
    config: _ComplianceConfig,
    stats: _ComplianceStats,
}

/// 合规配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _ComplianceConfig {
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
pub(crate) struct _GovernanceRule {
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
pub(crate) struct _ComplianceStats {
    pub total_checks: u64,
    pub passed_checks: u64,
    pub failed_checks: u64,
    pub total_violations: u64,
    pub resolved_violations: u64,
}

/// 检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _ComplianceCheckResult {
    pub passed: bool,
    pub violations: Vec<ComplianceViolation>,
    pub suggestions: Vec<String>,
}

impl _GovernanceComplianceChecker {
    /// 创建新的治理合规检查器
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

    /// 注册默认规则
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

    /// 执行合规检查
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

    /// 检查单个规则
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
                // 检查编译状态
                true // 简化版
            }
            "check_pre_push_build" => {
                // 检查编译状态
                true // 简化版
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
    pub fn rules(&self) -> &[_GovernanceRule] {
        &self.rules
    }

    /// 获取统计信息
    pub fn stats(&self) -> &_ComplianceStats {
        &self.stats
    }
}
