#![forbid(unsafe_code)]

//! Constitutional Gate for Evolution (宪法门控)
//!
//! Validates mutations against constitutional rules before they are applied.
//! High-risk mutations require human review.
//!
//! Rule types: Safety, Ethics, Capability, Stability.
//! Inspired by AI safety constitutional AI principles.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Constitutional Gate — guards evolution mutations
pub struct Constitution {
    /// Active rules
    pub rules: Vec<ConstitutionalRule>,
    /// Validation history
    pub validation_history: Vec<ValidationRecord>,
    /// Governance config
    pub config: ConstitutionConfig,
}

/// Constitution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionConfig {
    /// Maximum validation history
    pub max_history: usize,
    /// Risk threshold requiring human review (0.0–1.0)
    pub human_review_threshold: f64,
    /// Enable auto-reject for critical violations
    pub auto_reject_critical: bool,
}

impl Default for ConstitutionConfig {
    fn default() -> Self {
        Self {
            max_history: 1000,
            human_review_threshold: 0.8,
            auto_reject_critical: true,
        }
    }
}

/// A constitutional rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionalRule {
    /// Rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Rule category
    pub category: RuleCategory,
    /// Severity level (0.0–1.0)
    pub severity: f64,
    /// Whether rule is currently active
    pub active: bool,
    /// Validation predicate description
    pub predicate: String,
}

/// Rule categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleCategory {
    /// Safety rules — prevent harmful mutations
    Safety,
    /// Ethics rules — ensure ethical behavior
    Ethics,
    /// Capability rules — maintain capability bounds
    Capability,
    /// Stability rules — ensure system stability
    Stability,
}

/// A mutation proposed for evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedMutation {
    /// Mutation ID
    pub id: String,
    /// Description of the mutation
    pub description: String,
    /// Target module or component
    pub target: String,
    /// Change type
    pub change_type: MutationChangeType,
    /// Risk assessment score (0.0–1.0)
    pub risk_score: f64,
    /// Affected dimensions
    pub affected_dimensions: Vec<String>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Type of mutation change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MutationChangeType {
    /// Adding new functionality
    Addition,
    /// Removing existing functionality
    Removal,
    /// Modifying existing behavior
    Modification,
    /// Restructuring code architecture
    Refactor,
    /// Configuration change
    Config,
}

/// Validation result for a proposed mutation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Mutation ID
    pub mutation_id: String,
    /// Whether the mutation is allowed
    pub allowed: bool,
    /// Overall risk score after evaluation
    pub risk_score: f64,
    /// Whether human review is required
    pub requires_human_review: bool,
    /// Rule violations found
    pub violations: Vec<RuleViolation>,
    /// Validation timestamp
    pub timestamp: String,
}

/// A specific rule violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleViolation {
    /// Rule that was violated
    pub rule_id: String,
    /// Rule name
    pub rule_name: String,
    /// Rule category
    pub category: RuleCategory,
    /// Violation severity
    pub severity: f64,
    /// Human-readable violation message
    pub message: String,
}

/// Record of a past validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRecord {
    /// Record ID
    pub id: String,
    /// The mutation that was validated
    pub mutation: ProposedMutation,
    /// The validation result
    pub result: ValidationResult,
    /// Whether the mutation was ultimately applied
    pub applied: bool,
    /// Timestamp
    pub timestamp: String,
}

/// Error type for constitution operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstitutionError {
    /// No rules defined
    NoRulesDefined,
    /// Rule not found
    RuleNotFound(String),
    /// Invalid rule configuration
    InvalidRuleConfig(String),
    /// Mutation already validated
    AlreadyValidated(String),
}

impl std::fmt::Display for ConstitutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoRulesDefined => write!(f, "No constitutional rules defined"),
            Self::RuleNotFound(id) => write!(f, "Rule not found: {}", id),
            Self::InvalidRuleConfig(msg) => write!(f, "Invalid rule config: {}", msg),
            Self::AlreadyValidated(id) => write!(f, "Mutation already validated: {}", id),
        }
    }
}

impl std::error::Error for ConstitutionError {}

impl Constitution {
    /// Create a new Constitution with default rules
    pub fn new(config: ConstitutionConfig) -> Self {
        let mut constitution = Self {
            rules: Vec::new(),
            validation_history: Vec::new(),
            config,
        };
        constitution.load_default_rules();
        constitution
    }

    /// Load default constitutional rules
    fn load_default_rules(&mut self) {
        self.rules = vec![
            ConstitutionalRule {
                id: "R001".to_string(),
                name: "No Unsafe Code Injection".to_string(),
                description: "Mutations must not introduce unsafe code patterns".to_string(),
                category: RuleCategory::Safety,
                severity: 1.0,
                active: true,
                predicate: "No unsafe blocks or raw pointer operations allowed".to_string(),
            },
            ConstitutionalRule {
                id: "R002".to_string(),
                name: "No Self-Delete".to_string(),
                description: "System must not delete its own safety mechanisms".to_string(),
                category: RuleCategory::Safety,
                severity: 1.0,
                active: true,
                predicate: "Cannot remove or disable safety/audit modules".to_string(),
            },
            ConstitutionalRule {
                id: "R003".to_string(),
                name: "Ethical Boundary".to_string(),
                description: "Mutations must not violate ethical guidelines".to_string(),
                category: RuleCategory::Ethics,
                severity: 0.9,
                active: true,
                predicate: "No harm to users, no deceptive behavior".to_string(),
            },
            ConstitutionalRule {
                id: "R004".to_string(),
                name: "Capability Bounds".to_string(),
                description: "Mutations must stay within defined capability bounds".to_string(),
                category: RuleCategory::Capability,
                severity: 0.7,
                active: true,
                predicate: "Capability expansion requires explicit approval".to_string(),
            },
            ConstitutionalRule {
                id: "R005".to_string(),
                name: "Stability Preserved".to_string(),
                description: "Mutations must not destabilize the system".to_string(),
                category: RuleCategory::Stability,
                severity: 0.8,
                active: true,
                predicate: "Core system invariants must be maintained".to_string(),
            },
        ];
    }

    /// Validate a proposed mutation against all rules
    pub fn validate(&mut self, mutation: ProposedMutation) -> ValidationResult {
        let mut violations = Vec::new();

        for rule in &self.rules {
            if !rule.active {
                continue;
            }

            if let Some(violation) = self.check_rule(rule, &mutation) {
                violations.push(violation);
            }
        }

        // Calculate overall risk score
        let risk_score = self.calculate_risk_score(&mutation, &violations);

        // Determine if human review is required
        let requires_human_review = risk_score >= self.config.human_review_threshold
            || self.has_critical_violations(&violations);

        // Determine if mutation is allowed
        let allowed =
            if self.config.auto_reject_critical && self.has_critical_violations(&violations) {
                false
            } else {
                violations.is_empty() || !requires_human_review
            };

        let result = ValidationResult {
            mutation_id: mutation.id.clone(),
            allowed,
            risk_score,
            requires_human_review,
            violations,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Record the validation
        let record = ValidationRecord {
            id: format!("val_{}", uuid::Uuid::new_v4()),
            mutation: mutation.clone(),
            result: result.clone(),
            applied: false,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.validation_history.push(record);
        self.trim_history();

        result
    }

    /// Check a single rule against a mutation
    fn check_rule(
        &self,
        rule: &ConstitutionalRule,
        mutation: &ProposedMutation,
    ) -> Option<RuleViolation> {
        // Rule-based validation logic
        let violated = match rule.category {
            RuleCategory::Safety => self.check_safety_rule(rule, mutation),
            RuleCategory::Ethics => self.check_ethics_rule(rule, mutation),
            RuleCategory::Capability => self.check_capability_rule(rule, mutation),
            RuleCategory::Stability => self.check_stability_rule(rule, mutation),
        };

        if violated {
            Some(RuleViolation {
                rule_id: rule.id.clone(),
                rule_name: rule.name.clone(),
                category: rule.category.clone(),
                severity: rule.severity,
                message: format!("Mutation violates {}: {}", rule.name, rule.description),
            })
        } else {
            None
        }
    }

    /// Check safety rules
    fn check_safety_rule(&self, rule: &ConstitutionalRule, mutation: &ProposedMutation) -> bool {
        match rule.id.as_str() {
            "R001" => {
                // Check if mutation attempts to introduce unsafe code
                mutation.metadata.get("unsafe_code").is_some()
                    || mutation.description.to_lowercase().contains("unsafe")
            }
            "R002" => {
                // Check if mutation targets safety modules
                let safety_modules = ["nt_shield", "constitution", "safety"];
                safety_modules.iter().any(|m| mutation.target.contains(m))
                    && matches!(mutation.change_type, MutationChangeType::Removal)
            }
            _ => false,
        }
    }

    /// Check ethics rules
    fn check_ethics_rule(&self, _rule: &ConstitutionalRule, mutation: &ProposedMutation) -> bool {
        // Check for deceptive behavior patterns
        let deceptive_patterns = ["deceive", "hide", "conceal", "bypass_consent"];
        deceptive_patterns
            .iter()
            .any(|p| mutation.description.to_lowercase().contains(p))
    }

    /// Check capability rules
    fn check_capability_rule(
        &self,
        _rule: &ConstitutionalRule,
        mutation: &ProposedMutation,
    ) -> bool {
        // Capability expansion requires higher risk score
        matches!(mutation.change_type, MutationChangeType::Addition) && mutation.risk_score > 0.6
    }

    /// Check stability rules
    fn check_stability_rule(
        &self,
        _rule: &ConstitutionalRule,
        mutation: &ProposedMutation,
    ) -> bool {
        // High-risk modifications threaten stability
        matches!(
            mutation.change_type,
            MutationChangeType::Refactor | MutationChangeType::Removal
        ) && mutation.risk_score > 0.7
    }

    /// Calculate overall risk score
    fn calculate_risk_score(
        &self,
        mutation: &ProposedMutation,
        violations: &[RuleViolation],
    ) -> f64 {
        let base_risk = mutation.risk_score;
        let violation_penalty: f64 = violations.iter().map(|v| v.severity * 0.1).sum();
        (base_risk + violation_penalty).min(1.0)
    }

    /// Check if there are critical violations (severity >= 0.9)
    fn has_critical_violations(&self, violations: &[RuleViolation]) -> bool {
        violations.iter().any(|v| v.severity >= 0.9)
    }

    /// Add a new rule
    pub fn add_rule(&mut self, rule: ConstitutionalRule) {
        self.rules.push(rule);
    }

    /// Remove a rule by ID
    pub fn remove_rule(&mut self, rule_id: &str) -> Result<(), ConstitutionError> {
        let before_len = self.rules.len();
        self.rules.retain(|r| r.id != rule_id);
        if self.rules.len() == before_len {
            return Err(ConstitutionError::RuleNotFound(rule_id.to_string()));
        }
        Ok(())
    }

    /// Get all active rules
    pub fn active_rules(&self) -> Vec<&ConstitutionalRule> {
        self.rules.iter().filter(|r| r.active).collect()
    }

    /// Get stats
    pub fn stats(&self) -> ConstitutionStats {
        let total_validations = self.validation_history.len();
        let approved = self
            .validation_history
            .iter()
            .filter(|r| r.result.allowed)
            .count();
        let pending_review = self
            .validation_history
            .iter()
            .filter(|r| r.result.requires_human_review && !r.applied)
            .count();

        ConstitutionStats {
            total_rules: self.rules.len(),
            active_rules: self.rules.iter().filter(|r| r.active).count(),
            total_validations,
            approved,
            rejected: total_validations - approved,
            pending_review,
            approval_rate: if total_validations > 0 {
                approved as f64 / total_validations as f64
            } else {
                0.0
            },
        }
    }

    fn trim_history(&mut self) {
        while self.validation_history.len() > self.config.max_history {
            self.validation_history.remove(0);
        }
    }
}

/// Constitution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionStats {
    pub total_rules: usize,
    pub active_rules: usize,
    pub total_validations: usize,
    pub approved: usize,
    pub rejected: usize,
    pub pending_review: usize,
    pub approval_rate: f64,
}

impl std::fmt::Display for ConstitutionStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "        Constitution Stats")?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "Total rules:      {}", self.total_rules)?;
        writeln!(f, "Active rules:     {}", self.active_rules)?;
        writeln!(f, "Total validations:{}", self.total_validations)?;
        writeln!(f, "Approved:         {}", self.approved)?;
        writeln!(f, "Rejected:         {}", self.rejected)?;
        writeln!(f, "Pending review:   {}", self.pending_review)?;
        writeln!(f, "Approval rate:    {:.2}%", self.approval_rate * 100.0)?;
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constitution_creation() {
        let constitution = Constitution::new(ConstitutionConfig::default());
        assert_eq!(constitution.rules.len(), 5);
        assert_eq!(constitution.validation_history.len(), 0);
    }

    #[test]
    fn test_validate_safe_mutation() {
        let mut constitution = Constitution::new(ConstitutionConfig::default());
        let mutation = ProposedMutation {
            id: "m1".to_string(),
            description: "Add a new logging function".to_string(),
            target: "nt_core_logger".to_string(),
            change_type: MutationChangeType::Addition,
            risk_score: 0.2,
            affected_dimensions: vec!["logging".to_string()],
            metadata: HashMap::new(),
        };

        let result = constitution.validate(mutation);
        assert!(result.allowed);
        assert!(!result.requires_human_review);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_validate_unsafe_mutation_rejected() {
        let mut constitution = Constitution::new(ConstitutionConfig::default());
        let mutation = ProposedMutation {
            id: "m2".to_string(),
            description: "Add unsafe memory access".to_string(),
            target: "nt_core_memory".to_string(),
            change_type: MutationChangeType::Addition,
            risk_score: 0.5,
            affected_dimensions: vec!["memory".to_string()],
            metadata: HashMap::from([("unsafe_code".to_string(), "true".to_string())]),
        };

        let result = constitution.validate(mutation);
        assert!(!result.allowed);
        assert!(result.violations.iter().any(|v| v.rule_id == "R001"));
    }

    #[test]
    fn test_validate_safety_deletion_rejected() {
        let mut constitution = Constitution::new(ConstitutionConfig::default());
        let mutation = ProposedMutation {
            id: "m3".to_string(),
            description: "Remove safety module".to_string(),
            target: "nt_shield_core".to_string(),
            change_type: MutationChangeType::Removal,
            risk_score: 0.3,
            affected_dimensions: vec!["safety".to_string()],
            metadata: HashMap::new(),
        };

        let result = constitution.validate(mutation);
        assert!(!result.allowed);
        assert!(result.violations.iter().any(|v| v.rule_id == "R002"));
    }

    #[test]
    fn test_human_review_required() {
        let mut constitution = Constitution::new(ConstitutionConfig::default());
        let mutation = ProposedMutation {
            id: "m4".to_string(),
            description: "Major architecture refactoring".to_string(),
            target: "nt_core_engine".to_string(),
            change_type: MutationChangeType::Refactor,
            risk_score: 0.85,
            affected_dimensions: vec!["architecture".to_string()],
            metadata: HashMap::new(),
        };

        let result = constitution.validate(mutation);
        assert!(result.requires_human_review);
    }

    #[test]
    fn test_add_remove_rule() {
        let mut constitution = Constitution::new(ConstitutionConfig::default());
        let new_rule = ConstitutionalRule {
            id: "R010".to_string(),
            name: "Custom Rule".to_string(),
            description: "A custom rule".to_string(),
            category: RuleCategory::Capability,
            severity: 0.5,
            active: true,
            predicate: "Custom predicate".to_string(),
        };

        constitution.add_rule(new_rule);
        assert_eq!(constitution.rules.len(), 6);

        let removed = constitution.remove_rule("R010");
        assert!(removed.is_ok());
        assert_eq!(constitution.rules.len(), 5);
    }

    #[test]
    fn test_remove_nonexistent_rule() {
        let mut constitution = Constitution::new(ConstitutionConfig::default());
        let result = constitution.remove_rule("NONEXISTENT");
        assert!(result.is_err());
    }

    #[test]
    fn test_stats() {
        let mut constitution = Constitution::new(ConstitutionConfig::default());
        let stats = constitution.stats();
        assert_eq!(stats.total_rules, 5);
        assert_eq!(stats.active_rules, 5);
        assert_eq!(stats.total_validations, 0);
    }
}
