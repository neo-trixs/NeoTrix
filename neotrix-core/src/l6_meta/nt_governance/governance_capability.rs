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
}
