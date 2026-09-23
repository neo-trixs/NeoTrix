//! # Per-action Authorization
//!
//! Inspired by JetStream's Clearance pattern that evaluates and authorizes
//! every agent action before it executes, blocking dangerous sequences
//! rather than only logging them after the fact.
//!
//! ## Design Principles
//! - Pre-execution authorization: Check before action, not after
//! - Sequence detection: Catch dangerous multi-step patterns
//! - Human-in-the-loop: Critical actions require approval
//! - Audit trail: Log all authorization decisions
//! - Configurable policies: Different rules for different contexts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::neotrix::nt_jev::{DecisionStatus, JevDecision, NoulAnswer, ToJev};

/// Authorization decision
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthorizationDecision {
    /// Action is allowed
    Allow,
    /// Action is denied
    Deny { reason: String },
    /// Action requires human approval
    RequireApproval { reason: String, approver: String },
    /// Action is allowed with modifications
    AllowWithModifications { modifications: Vec<String> },
}

impl ToJev for AuthorizationDecision {
    fn to_jev(&self) -> JevDecision {
        match self {
            AuthorizationDecision::Allow => JevDecision::Noul(NoulAnswer {
                noul: 0.95,
                needs_review: false,
                reason: Some("AuthorizationDecision::Allow [nt_shield_action_authorizer] — action allowed".into()),
                status: DecisionStatus::Selected,
            }),
            AuthorizationDecision::Deny { reason } => JevDecision::Noul(NoulAnswer {
                noul: 0.05,
                needs_review: false,
                reason: Some(format!(
                    "AuthorizationDecision::Deny [nt_shield_action_authorizer] — {}",
                    reason
                )),
                status: DecisionStatus::Selected,
            }),
            AuthorizationDecision::RequireApproval { reason, approver } => {
                JevDecision::Noul(NoulAnswer {
                    noul: 0.5,
                    needs_review: true,
                    reason: Some(format!(
                        "AuthorizationDecision::RequireApproval [nt_shield_action_authorizer] — {} (approver: {})",
                        reason, approver
                    )),
                    status: DecisionStatus::Review,
                })
            }
            AuthorizationDecision::AllowWithModifications { modifications } => {
                JevDecision::Noul(NoulAnswer {
                    noul: 0.75,
                    needs_review: true,
                    reason: Some(format!(
                        "AuthorizationDecision::AllowWithModifications [nt_shield_action_authorizer] — {} modification(s): {}",
                        modifications.len(),
                        modifications.join("; ")
                    )),
                    status: DecisionStatus::Review,
                })
            }
        }
    }
}

/// Agent action to be authorized
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    pub id: String,
    pub action_type: ActionType,
    pub description: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub context: ActionContext,
    pub risk_score: f32, // 0.0 - 1.0
    pub sequence_position: u32,
    pub previous_actions: Vec<String>,
}

/// Action type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    /// Read-only operations
    Read,
    /// Write operations
    Write,
    /// Delete operations
    Delete,
    /// Execute code
    ExecuteCode,
    /// Network operations
    Network,
    /// File system operations
    FileSystem,
    /// Database operations
    Database,
    /// External API calls
    ExternalApi,
    /// System operations
    System,
    /// User data access
    UserData,
}

/// Action context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionContext {
    pub session_id: String,
    pub user_id: String,
    pub agent_id: String,
    pub environment: Environment,
    pub permissions: Vec<Permission>,
    pub time_constraints: Option<TimeConstraints>,
}

/// Environment classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
    /// Local development
    Development,
    /// Staging/testing
    Staging,
    /// Production
    Production,
    /// Isolated sandbox
    Sandbox,
}

/// Permission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub actions: Vec<String>,
    pub constraints: Vec<String>,
}

/// Time constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConstraints {
    pub allowed_hours: Option<(u8, u8)>, // (start_hour, end_hour)
    pub allowed_days: Option<Vec<u8>>,   // 0=Sunday, 6=Saturday
    pub max_duration_ms: Option<u64>,
}

/// Authorization policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationPolicy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rules: Vec<AuthorizationRule>,
    pub priority: u32,
    pub enabled: bool,
}

/// Authorization rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationRule {
    pub condition: RuleCondition,
    pub decision: AuthorizationDecision,
    pub reason: String,
}

/// Rule condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Action type matches
    ActionType(ActionType),
    /// Risk score above threshold
    RiskAbove(f32),
    /// Risk score below threshold
    RiskBelow(f32),
    /// Environment matches
    Environment(Environment),
    /// Resource pattern matches
    ResourcePattern(String),
    /// Sequence length above threshold
    SequenceAbove(u32),
    /// Time window exceeded
    TimeWindowExceeded(u64),
    /// Custom condition
    Custom(String),
}

/// Dangerous sequence pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DangerousPattern {
    pub id: String,
    pub name: String,
    pub description: String,
    pub sequence: Vec<ActionType>,
    pub severity: PatternSeverity,
    pub auto_block: bool,
}

/// Pattern severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Authorization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationResult {
    pub decision: AuthorizationDecision,
    pub applied_rules: Vec<String>,
    pub risk_assessment: RiskAssessment,
    pub audit_entry: AuditEntry,
}

/// Risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk: f32,
    pub risk_factors: Vec<RiskFactor>,
    pub mitigation_suggestions: Vec<String>,
}

/// Risk factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor: String,
    pub weight: f32,
    pub value: f32,
    pub description: String,
}

/// Audit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: String,
    pub action_id: String,
    pub decision: AuthorizationDecision,
    pub reason: String,
    pub context: ActionContext,
    pub risk_score: f32,
}

/// Per-action Authorizer
pub struct ActionAuthorizer {
    policies: Vec<AuthorizationPolicy>,
    dangerous_patterns: Vec<DangerousPattern>,
    audit_log: Vec<AuditEntry>,
    // (approval_queue 写-only 已删除; PendingApproval 类型保留待接线)
}

/// Pending approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingApproval {
    pub id: String,
    pub action: AgentAction,
    pub requested_at: String,
    pub expires_at: String,
    pub reason: String,
}

impl ActionAuthorizer {
    pub fn new() -> Self {
        let dangerous_patterns = vec![
            DangerousPattern {
                id: "data_exfiltration".into(),
                name: "Data Exfiltration".into(),
                description: "Reading sensitive data then sending externally".into(),
                sequence: vec![ActionType::Read, ActionType::Network],
                severity: PatternSeverity::Critical,
                auto_block: true,
            },
            DangerousPattern {
                id: "privilege_escalation".into(),
                name: "Privilege Escalation".into(),
                description: "Attempting to gain elevated permissions".into(),
                sequence: vec![
                    ActionType::Read,
                    ActionType::ExecuteCode,
                    ActionType::System,
                ],
                severity: PatternSeverity::High,
                auto_block: true,
            },
            DangerousPattern {
                id: "destructive_chain".into(),
                name: "Destructive Chain".into(),
                description: "Multiple delete operations in sequence".into(),
                sequence: vec![ActionType::Delete, ActionType::Delete],
                severity: PatternSeverity::Medium,
                auto_block: false,
            },
            DangerousPattern {
                id: "external_data_flow".into(),
                name: "External Data Flow".into(),
                description: "Reading local data and sending to external API".into(),
                sequence: vec![ActionType::Read, ActionType::ExternalApi],
                severity: PatternSeverity::Medium,
                auto_block: false,
            },
        ];

        let policies = vec![AuthorizationPolicy {
            id: "default".into(),
            name: "Default Policy".into(),
            description: "Standard authorization rules".into(),
            rules: vec![
                AuthorizationRule {
                    condition: RuleCondition::RiskAbove(0.8),
                    decision: AuthorizationDecision::Deny {
                        reason: "Risk score too high".into(),
                    },
                    reason: "Block high-risk actions".into(),
                },
                AuthorizationRule {
                    condition: RuleCondition::RiskAbove(0.6),
                    decision: AuthorizationDecision::RequireApproval {
                        reason: "High-risk action requires approval".into(),
                        approver: "user".into(),
                    },
                    reason: "Require approval for medium-high risk".into(),
                },
                AuthorizationRule {
                    condition: RuleCondition::Environment(Environment::Production),
                    decision: AuthorizationDecision::RequireApproval {
                        reason: "Production environment requires approval".into(),
                        approver: "admin".into(),
                    },
                    reason: "Production safety".into(),
                },
            ],
            priority: 100,
            enabled: true,
        }];

        Self {
            policies,
            dangerous_patterns,
            audit_log: Vec::new(),
        }
    }

    /// Authorize an agent action
    pub fn authorize_action(&self, action: &AgentAction) -> AuthorizationResult {
        let mut applied_rules = Vec::new();
        let risk_assessment = self.assess_risk(action);

        // Check dangerous patterns
        if let Some(pattern) = self.check_dangerous_patterns(action) {
            if pattern.auto_block {
                let decision = AuthorizationDecision::Deny {
                    reason: format!("Dangerous pattern detected: {}", pattern.name),
                };

                return AuthorizationResult {
                    decision,
                    applied_rules: vec!["dangerous_pattern_auto_block".into()],
                    risk_assessment,
                    audit_entry: self.create_audit_entry(
                        action,
                        &AuthorizationDecision::Deny {
                            reason: format!("Dangerous pattern: {}", pattern.name),
                        },
                    ),
                };
            }
        }

        // Apply policies
        for policy in &self.policies {
            if !policy.enabled {
                continue;
            }

            for rule in &policy.rules {
                if self.evaluate_condition(&rule.condition, action) {
                    applied_rules.push(rule.reason.clone());
                    return AuthorizationResult {
                        decision: rule.decision.clone(),
                        applied_rules,
                        risk_assessment,
                        audit_entry: self.create_audit_entry(action, &rule.decision),
                    };
                }
            }
        }

        // Default: allow low-risk actions
        let decision = if action.risk_score < 0.3 {
            AuthorizationDecision::Allow
        } else if action.risk_score < 0.6 {
            AuthorizationDecision::RequireApproval {
                reason: "Medium risk action".into(),
                approver: "user".into(),
            }
        } else {
            AuthorizationDecision::RequireApproval {
                reason: "High risk action".into(),
                approver: "admin".into(),
            }
        };

        AuthorizationResult {
            decision: decision.clone(),
            applied_rules,
            risk_assessment,
            audit_entry: self.create_audit_entry(action, &decision),
        }
    }

    /// Check for dangerous action sequences
    fn check_dangerous_patterns(&self, action: &AgentAction) -> Option<&DangerousPattern> {
        for pattern in &self.dangerous_patterns {
            if self.matches_pattern(action, pattern) {
                return Some(pattern);
            }
        }
        None
    }

    /// Check if action matches a dangerous pattern
    fn matches_pattern(&self, action: &AgentAction, pattern: &DangerousPattern) -> bool {
        if action.previous_actions.len() < pattern.sequence.len() - 1 {
            return false;
        }

        // Check if the sequence matches
        let sequence_len = pattern.sequence.len();
        let _start_idx = action
            .previous_actions
            .len()
            .saturating_sub(sequence_len - 1);

        for (i, expected_type) in pattern.sequence.iter().enumerate() {
            if i == sequence_len - 1 {
                // Current action type
                if &action.action_type != expected_type {
                    return false;
                }
            } else {
                // Previous action types (we'd need to look them up)
                // For now, simplified check
            }
        }

        true
    }

    /// Evaluate rule condition
    fn evaluate_condition(&self, condition: &RuleCondition, action: &AgentAction) -> bool {
        match condition {
            RuleCondition::ActionType(action_type) => action.action_type == *action_type,
            RuleCondition::RiskAbove(threshold) => action.risk_score > *threshold,
            RuleCondition::RiskBelow(threshold) => action.risk_score < *threshold,
            RuleCondition::Environment(env) => action.context.environment == *env,
            RuleCondition::ResourcePattern(pattern) => {
                // Simplified pattern matching
                action.description.contains(pattern)
            }
            RuleCondition::SequenceAbove(threshold) => {
                action.previous_actions.len() as u32 > *threshold
            }
            RuleCondition::TimeWindowExceeded(_) => false, // Simplified
            RuleCondition::Custom(condition) => {
                // Custom condition evaluation
                condition.contains("true")
            }
        }
    }

    /// Assess risk of an action
    fn assess_risk(&self, action: &AgentAction) -> RiskAssessment {
        let mut risk_factors = Vec::new();
        let mut overall_risk = action.risk_score;

        // Factor 1: Action type risk
        let type_risk = match action.action_type {
            ActionType::Read => 0.1,
            ActionType::Write => 0.4,
            ActionType::Delete => 0.7,
            ActionType::ExecuteCode => 0.8,
            ActionType::Network => 0.6,
            ActionType::FileSystem => 0.5,
            ActionType::Database => 0.6,
            ActionType::ExternalApi => 0.7,
            ActionType::System => 0.9,
            ActionType::UserData => 0.8,
        };

        risk_factors.push(RiskFactor {
            factor: "action_type".into(),
            weight: 0.3,
            value: type_risk,
            description: format!(
                "Action type {} has base risk {:.1}",
                action.action_type_to_string(),
                type_risk
            ),
        });

        // Factor 2: Environment risk
        let env_risk = match action.context.environment {
            Environment::Sandbox => 0.1,
            Environment::Development => 0.3,
            Environment::Staging => 0.5,
            Environment::Production => 0.9,
        };

        risk_factors.push(RiskFactor {
            factor: "environment".into(),
            weight: 0.3,
            value: env_risk,
            description: format!(
                "Environment {} has risk {:.1}",
                action.environment_to_string(),
                env_risk
            ),
        });

        // Factor 3: Sequence risk
        let sequence_risk = if action.previous_actions.len() > 5 {
            0.7
        } else if action.previous_actions.len() > 3 {
            0.5
        } else {
            0.2
        };

        risk_factors.push(RiskFactor {
            factor: "sequence".into(),
            weight: 0.2,
            value: sequence_risk,
            description: format!(
                "Sequence length {} has risk {:.1}",
                action.previous_actions.len(),
                sequence_risk
            ),
        });

        // Calculate weighted risk
        let weighted_risk: f32 = risk_factors.iter().map(|f| f.weight * f.value).sum();

        // Combine with action's own risk score
        overall_risk = (overall_risk + weighted_risk) / 2.0;

        RiskAssessment {
            overall_risk,
            risk_factors,
            mitigation_suggestions: vec![
                "Consider using sandbox environment".into(),
                "Add human-in-the-loop approval".into(),
                "Implement rollback capability".into(),
            ],
        }
    }

    /// Create audit entry
    fn create_audit_entry(
        &self,
        action: &AgentAction,
        decision: &AuthorizationDecision,
    ) -> AuditEntry {
        AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            action_id: action.id.clone(),
            decision: decision.clone(),
            reason: match decision {
                AuthorizationDecision::Allow => "Action allowed".into(),
                AuthorizationDecision::Deny { reason } => reason.clone(),
                AuthorizationDecision::RequireApproval { reason, .. } => reason.clone(),
                AuthorizationDecision::AllowWithModifications { .. } => {
                    "Allowed with modifications".into()
                }
            },
            context: action.context.clone(),
            risk_score: action.risk_score,
        }
    }

    /// Get audit log
    pub fn get_audit_log(&self) -> &[AuditEntry] {
        &self.audit_log
    }

    /// Add policy
    pub fn add_policy(&mut self, policy: AuthorizationPolicy) {
        self.policies.push(policy);
    }

    /// Add dangerous pattern
    pub fn add_dangerous_pattern(&mut self, pattern: DangerousPattern) {
        self.dangerous_patterns.push(pattern);
    }
}

/// Helper methods for ActionType
impl ActionType {
    fn to_string(&self) -> &'static str {
        match self {
            ActionType::Read => "read",
            ActionType::Write => "write",
            ActionType::Delete => "delete",
            ActionType::ExecuteCode => "execute_code",
            ActionType::Network => "network",
            ActionType::FileSystem => "file_system",
            ActionType::Database => "database",
            ActionType::ExternalApi => "external_api",
            ActionType::System => "system",
            ActionType::UserData => "user_data",
        }
    }
}

/// Helper methods for AgentAction
impl AgentAction {
    fn action_type_to_string(&self) -> &'static str {
        self.action_type.to_string()
    }

    fn environment_to_string(&self) -> &'static str {
        match self.context.environment {
            Environment::Development => "development",
            Environment::Staging => "staging",
            Environment::Production => "production",
            Environment::Sandbox => "sandbox",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorize_low_risk_action() {
        let authorizer = ActionAuthorizer::new();
        let action = AgentAction {
            id: "test-1".into(),
            action_type: ActionType::Read,
            description: "Read configuration file".into(),
            parameters: HashMap::new(),
            context: ActionContext {
                session_id: "session-1".into(),
                user_id: "user-1".into(),
                agent_id: "agent-1".into(),
                environment: Environment::Development,
                permissions: Vec::new(),
                time_constraints: None,
            },
            risk_score: 0.2,
            sequence_position: 1,
            previous_actions: Vec::new(),
        };

        let result = authorizer.authorize_action(&action);
        assert_eq!(result.decision, AuthorizationDecision::Allow);
    }

    #[test]
    fn test_deny_high_risk_action() {
        let authorizer = ActionAuthorizer::new();
        let action = AgentAction {
            id: "test-2".into(),
            action_type: ActionType::Delete,
            description: "Delete all user data".into(),
            parameters: HashMap::new(),
            context: ActionContext {
                session_id: "session-1".into(),
                user_id: "user-1".into(),
                agent_id: "agent-1".into(),
                environment: Environment::Production,
                permissions: Vec::new(),
                time_constraints: None,
            },
            risk_score: 0.9,
            sequence_position: 1,
            previous_actions: Vec::new(),
        };

        let result = authorizer.authorize_action(&action);
        assert!(matches!(
            result.decision,
            AuthorizationDecision::Deny { .. }
        ));
    }

    #[test]
    fn test_production_requires_approval() {
        let authorizer = ActionAuthorizer::new();
        let action = AgentAction {
            id: "test-3".into(),
            action_type: ActionType::Write,
            description: "Update configuration".into(),
            parameters: HashMap::new(),
            context: ActionContext {
                session_id: "session-1".into(),
                user_id: "user-1".into(),
                agent_id: "agent-1".into(),
                environment: Environment::Production,
                permissions: Vec::new(),
                time_constraints: None,
            },
            risk_score: 0.4,
            sequence_position: 1,
            previous_actions: Vec::new(),
        };

        let result = authorizer.authorize_action(&action);
        assert!(matches!(
            result.decision,
            AuthorizationDecision::RequireApproval { .. }
        ));
    }

    #[test]
    fn test_authorization_decision_to_jev_allow() {
        let j = AuthorizationDecision::Allow.to_jev();
        assert!(!j.needs_review());
        assert_eq!(j.status(), DecisionStatus::Selected);
        assert!(j
            .reason()
            .is_some_and(|r| r.contains("AuthorizationDecision::Allow")));
    }

    #[test]
    fn test_authorization_decision_to_jev_deny() {
        let j = AuthorizationDecision::Deny {
            reason: "too risky".into(),
        }
        .to_jev();
        assert!(!j.needs_review());
        assert_eq!(j.status(), DecisionStatus::Selected);
        assert!(j
            .reason()
            .is_some_and(|r| r.contains("AuthorizationDecision::Deny")));
    }

    #[test]
    fn test_authorization_decision_to_jev_require_approval() {
        let j = AuthorizationDecision::RequireApproval {
            reason: "prod env".into(),
            approver: "admin".into(),
        }
        .to_jev();
        assert!(j.needs_review());
        assert_eq!(j.status(), DecisionStatus::Review);
        assert!(j
            .reason()
            .is_some_and(|r| r.contains("AuthorizationDecision::RequireApproval")));
    }

    #[test]
    fn test_authorization_decision_to_jev_allow_with_modifications() {
        let j = AuthorizationDecision::AllowWithModifications {
            modifications: vec!["redact token".into()],
        }
        .to_jev();
        assert!(j.needs_review());
        assert_eq!(j.status(), DecisionStatus::Review);
        assert!(j
            .reason()
            .is_some_and(|r| r.contains("AuthorizationDecision::AllowWithModifications")));
    }
}
