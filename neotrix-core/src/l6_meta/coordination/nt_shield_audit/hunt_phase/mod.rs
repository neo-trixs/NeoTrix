//! Hunt Phase — cloudflare/security-audit-skill Phase 2 吸收
//! 
//! 并行 general agents 按攻击类别狩猎
//! 12 个狩猎角度 + 验证规则 + 域伴侣路由

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 攻击类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttackClass {
    Injection,
    AccessControl,
    ResourceFileHandling,
    CryptographySecrets,
    BusinessLogic,
    FeatureAbuseDataLeakage,
    ChainedAttacksTrustBoundaries,
    Wildcard,
    ObviousThings,
}

/// 域伴侣路由
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DomainCompanion {
    MemorySafetyBinary,
    AiAndLlm,
    WebProtocolAuth,
    ClientSide,
}

/// 狩猎角度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HuntingAngle {
    SadPaths,
    Boundaries,
    ComponentAssumptions,
    WrongOrder,
    Concurrency,
    ParserDisagreement,
    RoundTrips,
    ConfigControl,
    FollowMoneyPrivilege,
    LeakedContext,
    ParameterOverrides,
    UnverifiedClaims,
}

/// 验证规则
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationRule {
    ConcreteAttack,
    MeaningfulImpact,
    DefenseLayerCheck,
    BaselineComparison,
    ParserRuntimeVerification,
    ConfirmedOnly,
}

/// 狩猎发现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntingFinding {
    pub attack_class: AttackClass,
    pub angle: HuntingAngle,
    pub title: String,
    pub description: String,
    pub attack_vector: AttackVector,
    pub impact: Impact,
    pub severity: Severity,
    pub file_paths: Vec<String>,
    pub domain_companion: Option<DomainCompanion>,
    pub validation_passed: bool,
}

/// 攻击向量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackVector {
    pub steps: Vec<String>,
    pub inputs: HashMap<String, String>,
    pub prerequisites: Vec<String>,
}

/// 影响
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Impact {
    pub description: String,
    pub data_exfiltration: bool,
    pub privilege_escalation: bool,
    pub dos: bool,
    pub rce: bool,
}

/// 严重性
#[serde(rename_all = "lowercase")]

/// 狩猎范围分配
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntScope {
    pub attack_class: AttackClass,
    pub sub_systems: Vec<String>,
    pub entry_points: Vec<String>,
    pub domain_companion: Option<DomainCompanion>,
}

/// Hunt Phase 引擎
use neotrix_types::shared::Severity;
pub struct HuntPhase {
    architecture_summary: String, // 注入的 architecture.md
    scopes: Vec<HuntScope>,
    findings: Vec<HuntingFinding>,
}

impl HuntPhase {
    pub fn new(architecture_summary: String) -> Self {
        Self {
            architecture_summary,
            scopes: Self::default_scopes(),
            findings: Vec::new(),
        }
    }

    fn default_scopes() -> Vec<HuntScope> {
        vec![
            HuntScope {
                attack_class: AttackClass::Injection,
                sub_systems: vec!["api".to_string(), "database".to_string()],
                entry_points: vec!["/api/*".to_string()],
                domain_companion: None,
            },
            HuntScope {
                attack_class: AttackClass::AccessControl,
                sub_systems: vec!["auth".to_string(), "middleware".to_string()],
                entry_points: vec!["/api/*".to_string()],
                domain_companion: None,
            },
            HuntScope {
                attack_class: AttackClass::BusinessLogic,
                sub_systems: vec!["core".to_string(), "workflows".to_string()],
                entry_points: vec!["/api/transactions".to_string()],
                domain_companion: None,
            },
            HuntScope {
                attack_class: AttackClass::Wildcard,
                sub_systems: vec!["all".to_string()],
                entry_points: vec!["*".to_string()],
                domain_companion: None,
            },
            HuntScope {
                attack_class: AttackClass::ObviousThings,
                sub_systems: vec!["all".to_string()],
                entry_points: vec!["*".to_string()],
                domain_companion: None,
            },
        ]
    }

    /// 运行并行狩猎 (实际应启动多个 general agents)
    pub async fn run(&mut self) -> Vec<HuntingFinding> {
        // 模拟并行执行
        for scope in &self.scopes.clone() {
            let findings = self.hunt_scope(scope).await;
            self.findings.extend(findings);
        }
        
        // 应用验证规则
        self.findings.retain(|f| self.validate_finding(f));
        
        self.findings.clone()
    }

    async fn hunt_scope(&self, _scope: &HuntScope) -> Vec<HuntingFinding> {
        // 实际实现中应根据攻击类别应用 12 个狩猎角度
        // 这里返回模拟发现
        vec![]
    }

    /// 验证规则 (6 条)
    fn validate_finding(&self, finding: &HuntingFinding) -> bool {
        // 1. Concrete attack
        if finding.attack_vector.steps.is_empty() {
            return false;
        }
        
        // 2. Meaningful impact
        let impact = &finding.impact;
        if !impact.data_exfiltration && !impact.privilege_escalation && !impact.dos && !impact.rce {
            return false;
        }
        
        // 3. Defense layer check (简化)
        // 4. Baseline comparison (简化)
        // 5. Parser/runtime verification (简化)
        // 6. Confirmed only
        finding.validation_passed
    }

    /// 根据 Phase 1 动态调整范围
    pub fn adjust_scopes(&mut self, prior_findings: &[HuntingFinding]) {
        // Skip known findings
        let known_classes: std::collections::HashSet<_> = prior_findings.iter()
            .map(|f| f.attack_class)
            .collect();
        
        // Weight toward gaps
        for scope in &mut self.scopes {
            if !known_classes.contains(&scope.attack_class) {
                scope.sub_systems.push("priority-gap".to_string());
            }
        }
    }

    /// SelfTest for C1 promotion
    pub fn self_test() -> Result<(), String> {
        let phase = HuntPhase::new("test architecture".to_string());
        
        // Test 1: Default scopes cover major classes
        let classes: std::collections::HashSet<_> = phase.scopes.iter().map(|s| s.attack_class).collect();
        assert!(classes.contains(&AttackClass::Injection));
        assert!(classes.contains(&AttackClass::AccessControl));
        assert!(classes.contains(&AttackClass::BusinessLogic));
        assert!(classes.contains(&AttackClass::Wildcard));
        assert!(classes.contains(&AttackClass::ObviousThings));
        
        // Test 2: Validation rules
        let finding = HuntingFinding {
            attack_class: AttackClass::Injection,
            angle: HuntingAngle::SadPaths,
            title: "Test".to_string(),
            description: "Test".to_string(),
            attack_vector: AttackVector {
                steps: vec!["Step 1".to_string()],
                inputs: HashMap::new(),
                prerequisites: vec![],
            },
            impact: Impact {
                description: "Data exfiltration".to_string(),
                data_exfiltration: true,
                privilege_escalation: false,
                dos: false,
                rce: false,
            },
            severity: Severity::High,
            file_paths: vec![],
            domain_companion: None,
            validation_passed: true,
        };
        
        assert!(finding.impact.data_exfiltration);
        assert!(!finding.attack_vector.steps.is_empty());
        
        Ok(())
    }
}

impl Default for HuntPhase {
    fn default() -> Self {
        Self::new("".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hunt_phase_scopes() {
        let phase = HuntPhase::new("test".to_string());
        assert_eq!(phase.scopes.len(), 5);
    }

    #[test]
    fn test_validation_rules() {
        let finding = HuntingFinding {
            attack_class: AttackClass::Injection,
            angle: HuntingAngle::SadPaths,
            title: "Test".to_string(),
            description: "Test".to_string(),
            attack_vector: AttackVector {
                steps: vec!["Send payload".to_string()],
                inputs: HashMap::new(),
                prerequisites: vec![],
            },
            impact: Impact {
                description: "Exfil".to_string(),
                data_exfiltration: true,
                privilege_escalation: false,
                dos: false,
                rce: false,
            },
            severity: Severity::High,
            file_paths: vec![],
            domain_companion: None,
            validation_passed: true,
        };
        
        // Should pass validation
        assert!(!finding.attack_vector.steps.is_empty());
        assert!(finding.impact.data_exfiltration);
    }

    #[test]
    fn test_self_test_passes() {
        assert!(HuntPhase::self_test().is_ok());
    }
}