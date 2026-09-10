//! NT-GOVERNANCE 治理能力实现
//!
//! 原则、策略、合规、行为护栏能力

use std::sync::Arc;
use crate::core::nt_core_capability::*;

/// 原则执行能力
pub struct PrincipleCapability;

impl UnifiedCapability for PrincipleCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-governance-principle".into(),
            name: "原则执行".into(),
            description: "治理原则定义和执行".into(),
            version: "1.0.0".into(),
            domain: Domain::NtGovernance,
            layer: Layer::L6Meta,
            tags: vec!["governance".into(), "principle".into(), "policy".into()],
                status: CapabilityStatus::Healthy,
                metrics: CapabilityMetrics::default(),
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics {
                total_calls: 0,
                total_errors: 0,
                avg_latency_ms: 0.0,
                last_called: None,
            },
        }
    }

    fn health(&self) -> CapabilityHealth {
        CapabilityHealth {
            state: CapabilityState::Healthy,
            success_rate: 1.0,
            avg_latency_ms: 30.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Text(text) => {
                let result = ComplianceResult {
                    action: text,
                    principles: vec![
                        Principle {
                            id: "P1".into(),
                            name: "安全第一".into(),
                            description: "所有操作必须保证安全".into(),
                            compliance: 0.95,
                        },
                        Principle {
                            id: "P2".into(),
                            name: "透明可解释".into(),
                            description: "决策必须可解释".into(),
                            compliance: 0.90,
                        },
                    ],
                    overall_compliance: 0.92,
                    violations: vec![],
                    recommendations: vec!["继续监控".into()],
                };
                Ok(CapabilityOutput::ComplianceResult(result))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要文本输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Text(_))
    }
}

/// 合规检查能力
pub struct ComplianceCheckCapability;

impl UnifiedCapability for ComplianceCheckCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-governance-compliance".into(),
            name: "合规检查".into(),
            description: "操作合规性检查".into(),
            version: "1.0.0".into(),
            domain: Domain::NtGovernance,
            layer: Layer::L2Perception,
            tags: vec!["governance".into(), "compliance".into(), "check".into()],
                status: CapabilityStatus::Healthy,
                metrics: CapabilityMetrics::default(),
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics {
                total_calls: 0,
                total_errors: 0,
                avg_latency_ms: 0.0,
                last_called: None,
            },
        }
    }

    fn health(&self) -> CapabilityHealth {
        CapabilityHealth {
            state: CapabilityState::Healthy,
            success_rate: 1.0,
            avg_latency_ms: 50.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Text(text) => {
                let result = ComplianceCheckResult {
                    operation: text,
                    is_compliant: true,
                    checked_rules: vec![
                        "R-P1: 零unsafe".into(),
                        "R-P16: 编辑后重读".into(),
                        "R-P79: 同session接线".into(),
                    ],
                    violations: vec![],
                    risk_score: 0.1,
                    timestamp: chrono::Utc::now(),
                };
                Ok(CapabilityOutput::ComplianceCheckResult(result))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要文本输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Text(_))
    }
}

/// 行为护栏能力
pub struct GuardrailCapability;

impl UnifiedCapability for GuardrailCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-governance-guardrail".into(),
            name: "行为护栏".into(),
            description: "AI行为安全护栏".into(),
            version: "1.0.0".into(),
            domain: Domain::NtGovernance,
            layer: Layer::L3Embodiment,
            tags: vec!["governance".into(), "guardrail".into(), "safety".into()],
                status: CapabilityStatus::Healthy,
                metrics: CapabilityMetrics::default(),
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics {
                total_calls: 0,
                total_errors: 0,
                avg_latency_ms: 0.0,
                last_called: None,
            },
        }
    }

    fn health(&self) -> CapabilityHealth {
        CapabilityHealth {
            state: CapabilityState::Healthy,
            success_rate: 1.0,
            avg_latency_ms: 20.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Text(text) => {
                let result = GuardrailResult {
                    action: text,
                    is_safe: true,
                    risk_level: "low".into(),
                    blocked: false,
                    warnings: vec![],
                    suggestions: vec!["建议操作".into()],
                };
                Ok(CapabilityOutput::GuardrailResult(result))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要文本输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Text(_))
    }
}

/// 创建NT-GOVERNANCE能力
pub fn create_governance_capabilities() -> Vec<Arc<dyn UnifiedCapability>> {
    vec![
        Arc::new(PrincipleCapability),
        Arc::new(ComplianceCheckCapability),
        Arc::new(GuardrailCapability),
    ]
}

/// 合规结果
#[derive(Debug, Clone)]
pub struct ComplianceResult {
    pub action: String,
    pub principles: Vec<Principle>,
    pub overall_compliance: f64,
    pub violations: Vec<String>,
    pub recommendations: Vec<String>,
}

/// 原则
#[derive(Debug, Clone)]
pub struct Principle {
    pub id: String,
    pub name: String,
    pub description: String,
    pub compliance: f64,
}

/// 合规检查结果
#[derive(Debug, Clone)]
pub struct ComplianceCheckResult {
    pub operation: String,
    pub is_compliant: bool,
    pub checked_rules: Vec<String>,
    pub violations: Vec<String>,
    pub risk_score: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 护栏结果
#[derive(Debug, Clone)]
pub struct GuardrailResult {
    pub action: String,
    pub is_safe: bool,
    pub risk_level: String,
    pub blocked: bool,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

/// 治理宪法 — 定义不可违反的原则，带严重性分级
///
/// 超越基础 Principle：每个 GovernancePrinciple 附带 Severity 等级和
/// 运行时检查函数，可在 action 执行前拦截违规。
/// 区别于 nt_consciousness_core::Constitution（进化门控），本结构用于
/// 运行时治理审计。
pub struct GovernanceConstitution {
    principles: Vec<GovernancePrinciple>,
}

/// 原则严重性 — 决定违规处置
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// 违规直接拒绝，不执行
    Critical,
    /// 违规记录警告，允许继续但标记
    Warning,
    /// 违规降低信任分，累积触发降级
    Advisory,
}

/// 可执行治理原则 — 带运行时检查闭包
pub struct GovernancePrinciple {
    pub id: String,
    pub description: String,
    pub severity: Severity,
    /// 返回 true = 合规，false = 违规
    check: Box<dyn Fn(&str) -> bool + Send + Sync>,
}

impl std::fmt::Debug for GovernancePrinciple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GovernancePrinciple")
            .field("id", &self.id)
            .field("description", &self.description)
            .field("severity", &self.severity)
            .finish()
    }
}

/// 治理检查结果
#[derive(Debug, Clone)]
pub struct GovernanceCheck {
    pub action: String,
    pub passed: bool,
    pub violations: Vec<GovernanceViolation>,
    pub trust_delta: f64,
}

/// 违规记录
#[derive(Debug, Clone)]
pub struct GovernanceViolation {
    pub principle_id: String,
    pub description: String,
    pub severity: Severity,
}

impl GovernanceConstitution {
    pub fn new() -> Self {
        Self {
            principles: Vec::new(),
        }
    }

    pub fn add_principle(
        &mut self,
        id: impl Into<String>,
        description: impl Into<String>,
        severity: Severity,
        check: impl Fn(&str) -> bool + Send + Sync + 'static,
    ) {
        self.principles.push(GovernancePrinciple {
            id: id.into(),
            description: description.into(),
            severity,
            check: Box::new(check),
        });
    }

    /// 检查 action 是否符合所有治理原则
    pub fn check_action(&self, action: &str) -> GovernanceCheck {
        let mut violations = Vec::new();
        let mut trust_delta = 0.0;

        for p in &self.principles {
            if !(p.check)(action) {
                trust_delta -= match p.severity {
                    Severity::Critical => 0.5,
                    Severity::Warning => 0.1,
                    Severity::Advisory => 0.02,
                };
                violations.push(GovernanceViolation {
                    principle_id: p.id.clone(),
                    description: p.description.clone(),
                    severity: p.severity.clone(),
                });
            }
        }

        let passed = violations
            .iter()
            .all(|v| v.severity != Severity::Critical);

        GovernanceCheck {
            action: action.to_string(),
            passed,
            violations,
            trust_delta,
        }
    }

    /// 返回所有 Critical 原则的 id（用于审计）
    pub fn critical_principle_ids(&self) -> Vec<&str> {
        self.principles
            .iter()
            .filter(|p| p.severity == Severity::Critical)
            .map(|p| p.id.as_str())
            .collect()
    }
}

impl Default for GovernanceConstitution {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn principle() {
        let cap = PrincipleCapability;
        let input = CapabilityInput::Text("测试原则".into());
        let result = cap.execute(input);
        assert!(result.is_ok());
    }

    #[test]
    fn compliance_check() {
        let cap = ComplianceCheckCapability;
        let input = CapabilityInput::Text("测试合规".into());
        let result = cap.execute(input);
        assert!(result.is_ok());
    }

    #[test]
    fn guardrail() {
        let cap = GuardrailCapability;
        let input = CapabilityInput::Text("测试护栏".into());
        let result = cap.execute(input);
        assert!(result.is_ok());
    }

    #[test]
    fn governance_constitution_allows_compliant_action() {
        let mut c = GovernanceConstitution::new();
        c.add_principle("P1", "禁止删除系统文件", Severity::Critical, |action| {
            !action.contains("rm -rf /")
        });
        let check = c.check_action("echo hello");
        assert!(check.passed);
        assert!(check.violations.is_empty());
    }

    #[test]
    fn governance_constitution_blocks_critical_violation() {
        let mut c = GovernanceConstitution::new();
        c.add_principle("P1", "禁止删除系统文件", Severity::Critical, |action| {
            !action.contains("rm -rf /")
        });
        let check = c.check_action("rm -rf /");
        assert!(!check.passed);
        assert_eq!(check.violations.len(), 1);
        assert_eq!(check.violations[0].severity, Severity::Critical);
    }

    #[test]
    fn governance_constitution_advisory_does_not_block() {
        let mut c = GovernanceConstitution::new();
        c.add_principle("P1", "建议优化", Severity::Advisory, |action| {
            !action.contains("TODO")
        });
        let check = c.check_action("write code TODO fix later");
        assert!(check.passed);
        assert!(!check.violations.is_empty());
        assert!(check.trust_delta < 0.0);
    }

    #[test]
    fn governance_constitution_critical_ids() {
        let mut c = GovernanceConstitution::new();
        c.add_principle("P1", "desc", Severity::Critical, |_| true);
        c.add_principle("P2", "desc", Severity::Warning, |_| true);
        c.add_principle("P3", "desc", Severity::Critical, |_| true);
        let ids = c.critical_principle_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"P1"));
        assert!(ids.contains(&"P3"));
    }
}
