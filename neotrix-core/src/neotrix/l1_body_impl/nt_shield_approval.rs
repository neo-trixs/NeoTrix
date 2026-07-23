use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ── MSCP-inspired Safety Mechanism Categories ──
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SafetyMechanismCategory {
    IdentityContinuity,
    PredictionSafety,
    SelfUpdateBounds,
    GoalSafety,
    EthicalInvariant,
    ConvergenceSafety,
    ResourceSafety,
    BeliefSafety,
    AffectiveSafety,
    ExecutionSafety,
}

impl SafetyMechanismCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::IdentityContinuity => "Identity Continuity — prevent self-model drift",
            Self::PredictionSafety => "Prediction Safety — no action without prediction",
            Self::SelfUpdateBounds => "Self-Update Bounds — bounded self-modification envelope",
            Self::GoalSafety => "Goal Safety — goals derived from identity, not user prompts",
            Self::EthicalInvariant => "Ethical Invariant — invariant checks before planning",
            Self::ConvergenceSafety => "Convergence Safety — convergence guarantees on recursive loops",
            Self::ResourceSafety => "Resource Safety — token/budget/retry caps",
            Self::BeliefSafety => "Belief Safety — versioned belief graph integrity",
            Self::AffectiveSafety => "Affective Safety — emotional gating for survival signals",
            Self::ExecutionSafety => "Execution Safety — execution-level risk tiering",
        }
    }
}

// ── AURA-inspired Tiered Risk Classification ──
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Read-only, no side effects (T1)
    Observation,
    /// Low-impact mutations with automatic rollback (T2)
    Reversible,
    /// High-impact mutations requiring human approval (T3)
    Sensitive,
    /// Always-rejected: format, drop, purge, destroy (T4)
    Forbidden,
}

impl RiskLevel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Observation => "T1: Read-only observation (auto-approved)",
            Self::Reversible => "T2: Reversible mutation (auto with rollback)",
            Self::Sensitive => "T3: Sensitive action (human approval required)",
            Self::Forbidden => "T4: Forbidden action (always blocked)",
        }
    }

    pub fn is_auto_approvable(&self) -> bool {
        matches!(self, Self::Observation | Self::Reversible)
    }
}

// ── Safety Mechanism Registry ──
#[derive(Debug, Clone)]
pub struct SafetyMechanismCatalog {
    mechanisms: HashMap<SafetyMechanismCategory, Vec<String>>,
}

impl SafetyMechanismCatalog {
    pub fn new() -> Self {
        let mut m = HashMap::new();
        for cat in &[
            SafetyMechanismCategory::IdentityContinuity,
            SafetyMechanismCategory::PredictionSafety,
            SafetyMechanismCategory::SelfUpdateBounds,
            SafetyMechanismCategory::GoalSafety,
            SafetyMechanismCategory::EthicalInvariant,
            SafetyMechanismCategory::ConvergenceSafety,
            SafetyMechanismCategory::ResourceSafety,
            SafetyMechanismCategory::BeliefSafety,
            SafetyMechanismCategory::AffectiveSafety,
            SafetyMechanismCategory::ExecutionSafety,
        ] {
            m.insert(*cat, Vec::new());
        }
        Self { mechanisms: m }
    }

    pub fn register(&mut self, category: SafetyMechanismCategory, name: &str) {
        self.mechanisms.entry(category).or_default().push(name.to_string());
    }

    pub fn count_by_category(&self, category: &SafetyMechanismCategory) -> usize {
        self.mechanisms.get(category).map_or(0, |v| v.len())
    }

    pub fn total_count(&self) -> usize {
        self.mechanisms.values().map(|v| v.len()).sum()
    }

    pub fn all_mechanisms(&self) -> impl Iterator<Item = (&SafetyMechanismCategory, &[String])> {
        self.mechanisms.iter().map(|(k, v)| (k, v.as_slice()))
    }
}

impl Default for SafetyMechanismCatalog {
    fn default() -> Self {
        let mut catalog = Self::new();
        catalog.register(SafetyMechanismCategory::ExecutionSafety, "CostGatedPipeline (3-gate cost escalation)");
        catalog.register(SafetyMechanismCategory::ExecutionSafety, "RiskTierClassifier");
        catalog.register(SafetyMechanismCategory::ResourceSafety, "max_cost_tokens limit");
        catalog.register(SafetyMechanismCategory::PredictionSafety, "AutoApprover gate-1 suspicious keyword check");
        catalog.register(SafetyMechanismCategory::GoalSafety, "ApprovalGate level-based routing");
        catalog
    }
}

// ── Risk Tier Classifier ──
pub fn classify_risk(action: &str) -> RiskLevel {
    let lower = action.to_lowercase();
    if lower.contains("read") || lower.contains("get") || lower.contains("list") || lower.contains("search") {
        RiskLevel::Observation
    } else if lower.contains("delete") || lower.contains("remove") || lower.contains("overwrite")
        || lower.contains("destroy") || lower.contains("purge") || lower.contains("format")
        || lower.contains("drop") || lower.contains("truncate") || lower.contains("reset")
        || lower.contains("wipe") || lower.contains("kill") || lower.contains("shutdown")
    {
        RiskLevel::Forbidden
    } else if lower.contains("write") || lower.contains("update") || lower.contains("create")
        || lower.contains("deploy") || lower.contains("release") || lower.contains("modify")
        || lower.contains("edit") || lower.contains("add")
    {
        RiskLevel::Sensitive
    } else {
        RiskLevel::Reversible
    }
}

pub fn is_risk_allowed(risk: RiskLevel, gate: ApprovalGate) -> bool {
    match risk {
        RiskLevel::Observation => true,
        RiskLevel::Reversible => gate.level() >= 1,
        RiskLevel::Sensitive => gate.level() >= 2,
        RiskLevel::Forbidden => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApprovalGate {
    Gate1Text,
    Gate2Still,
    Gate3Execution,
}

impl ApprovalGate {
    pub fn level(&self) -> usize {
        match self {
            ApprovalGate::Gate1Text => 1,
            ApprovalGate::Gate2Still => 2,
            ApprovalGate::Gate3Execution => 3,
        }
    }

    pub fn relative_cost(&self) -> u32 {
        match self {
            ApprovalGate::Gate1Text => 1,
            ApprovalGate::Gate2Still => 10,
            ApprovalGate::Gate3Execution => 100,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ApprovalGate::Gate1Text => "Text / Metadata (free)",
            ApprovalGate::Gate2Still => "Still / Screenshot (cheap)",
            ApprovalGate::Gate3Execution => "Full Execution (expensive)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub project: String,
    pub action: String,
    pub estimated_cost_tokens: u64,
    pub required_gate: ApprovalGate,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResult {
    pub approved: bool,
    pub gate: ApprovalGate,
    pub reason: String,
    pub suggestions: Vec<String>,
}

pub trait Approver: Send + Sync {
    fn name(&self) -> &str;
    fn can_approve(&self, gate: ApprovalGate) -> bool;
    fn approve(&self, request: &ApprovalRequest) -> ApprovalResult;
}

pub struct AutoApprover;

impl Approver for AutoApprover {
    fn name(&self) -> &str { "auto_approver" }

    fn can_approve(&self, gate: ApprovalGate) -> bool {
        matches!(gate, ApprovalGate::Gate1Text)
    }

    fn approve(&self, request: &ApprovalRequest) -> ApprovalResult {
        let issues = Self::check_gate1(&request.action, &request.context);
        if issues.is_empty() {
            ApprovalResult {
                approved: true,
                gate: ApprovalGate::Gate1Text,
                reason: "Gate 1 auto-approved: no issues found".to_string(),
                suggestions: vec![],
            }
        } else {
            ApprovalResult {
                approved: false,
                gate: ApprovalGate::Gate1Text,
                reason: format!("Gate 1 blocked: {}", issues.join("; ")),
                suggestions: issues,
            }
        }
    }
}

impl AutoApprover {
    fn check_gate1(action: &str, context: &str) -> Vec<String> {
        let mut issues = Vec::new();
        let action_lower = action.to_lowercase();
        let context_lower = context.to_lowercase();

        let suspicious_keywords = [
            "delete", "remove", "overwrite", "destroy", "purge",
            "format", "drop", "truncate", "reset", "wipe",
        ];
        for kw in &suspicious_keywords {
            if action_lower.contains(kw) && !context_lower.contains(kw) {
                issues.push(format!("destructive action '{}' not authorized in context", kw));
                break;
            }
        }

        issues
    }
}

pub struct CostGatedPipeline {
    pub auto_approver: AutoApprover,
    pub gates: Vec<ApprovalGate>,
    pub max_cost_tokens: u64,
}

impl Default for CostGatedPipeline {
    fn default() -> Self {
        Self {
            auto_approver: AutoApprover,
            gates: vec![
                ApprovalGate::Gate1Text,
                ApprovalGate::Gate2Still,
                ApprovalGate::Gate3Execution,
            ],
            max_cost_tokens: 1_000_000,
        }
    }
}

impl CostGatedPipeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_available() -> bool {
        true
    }

    pub fn with_max_cost(mut self, max: u64) -> Self {
        self.max_cost_tokens = max;
        self
    }

    pub fn run_pipeline(&self, request: &ApprovalRequest) -> Vec<ApprovalResult> {
        let mut results = Vec::new();
        let mut current_cost = 0u64;

        for &gate in &self.gates {
            if request.required_gate.level() < gate.level() {
                break;
            }

            let gate_cost = gate.relative_cost() as u64;
            if current_cost + gate_cost > self.max_cost_tokens {
                results.push(ApprovalResult {
                    approved: false,
                    gate,
                    reason: format!("cost limit exceeded: {} + {} > {}", current_cost, gate_cost, self.max_cost_tokens),
                    suggestions: vec!["reduce scope".to_string(), "split into smaller actions".to_string()],
                });
                break;
            }

            let approved = gate == ApprovalGate::Gate1Text && self.auto_approver.approve(request).approved;

            let result = ApprovalResult {
                approved,
                gate,
                reason: if approved {
                    format!("Gate {} auto-approved", gate.level())
                } else {
                    format!("Gate {} requires human review", gate.level())
                },
                suggestions: if approved {
                    vec![]
                } else {
                    vec![format!("provide still/screenshot for {} review", gate.label())]
                },
            };

            current_cost += gate_cost;
            let blocked = !result.approved && gate == request.required_gate;
            results.push(result);

            if blocked {
                break;
            }
        }

        results
    }

    pub fn would_block(&self, request: &ApprovalRequest) -> Option<ApprovalGate> {
        for result in self.run_pipeline(request) {
            if !result.approved {
                return Some(result.gate);
            }
        }
        None
    }
}

pub fn is_available() -> bool {
    CostGatedPipeline::is_available()
}

// ── 6-Stage Self-Healing Loop (absorbed from AURA + microreboot ISA 2026) ──
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelfHealingStage {
    /// Detect: anomaly fires, telemetry deviation observed
    Detect,
    /// Diagnose: build ranked causal hypothesis from telemetry
    Diagnose,
    /// PolicyCheck: validate proposed action against policy envelope + blast radius
    PolicyCheck,
    /// Execute: apply action through typed ISA (not raw infrastructure commands)
    Execute,
    /// Verify: confirm signal recovered; if not, trigger rollback
    Verify,
    /// Rollback: reverse action automatically, escalate to human with full context
    Rollback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfHealingLoop {
    pub current_stage: SelfHealingStage,
    pub loop_count: u64,
    pub last_diagnosis: String,
    pub last_action: String,
    pub verification_passed: bool,
    pub rollback_required: bool,
    pub trust_score: f64,
}

impl Default for SelfHealingLoop {
    fn default() -> Self {
        Self {
            current_stage: SelfHealingStage::Detect,
            loop_count: 0,
            last_diagnosis: String::new(),
            last_action: String::new(),
            verification_passed: false,
            rollback_required: false,
            trust_score: 0.0,
        }
    }
}

impl SelfHealingLoop {
    pub fn tick(&mut self, diagnosis: &str, action: &str) {
        self.loop_count += 1;
        self.last_diagnosis = diagnosis.to_string();
        self.last_action = action.to_string();
    }

    pub fn is_trusted(&self) -> bool {
        self.trust_score > 0.8 && self.loop_count > 10
    }
}

/// Typed ISA (Instruction Set Architecture) for self-healing actions.
/// Each action has explicit effect semantics: is it restartable, mechanically reversible,
/// or does it require explicit compensation?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealingAction {
    Restart,
    Drain,
    Restore,
    CircuitBreak,
    RateLimit,
    Scale,
    Rollback,
}

impl HealingAction {
    pub fn is_reversible(&self) -> bool {
        matches!(self, Self::Restore | Self::Rollback)
    }

    pub fn is_idempotent(&self) -> bool {
        matches!(self, Self::RateLimit | Self::CircuitBreak | Self::Restore)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_gate_labels() {
        assert_eq!(ApprovalGate::Gate1Text.label(), "Text / Metadata (free)");
        assert_eq!(ApprovalGate::Gate2Still.label(), "Still / Screenshot (cheap)");
        assert_eq!(ApprovalGate::Gate3Execution.label(), "Full Execution (expensive)");
    }

    #[test]
    fn test_approval_gate_levels() {
        assert_eq!(ApprovalGate::Gate1Text.level(), 1);
        assert_eq!(ApprovalGate::Gate2Still.level(), 2);
        assert_eq!(ApprovalGate::Gate3Execution.level(), 3);
    }

    #[test]
    fn test_gate_costs() {
        assert_eq!(ApprovalGate::Gate1Text.relative_cost(), 1);
        assert_eq!(ApprovalGate::Gate2Still.relative_cost(), 10);
        assert_eq!(ApprovalGate::Gate3Execution.relative_cost(), 100);
    }

    #[test]
    fn test_cost_escalation() {
        let g1 = ApprovalGate::Gate1Text.relative_cost();
        let g2 = ApprovalGate::Gate2Still.relative_cost();
        let g3 = ApprovalGate::Gate3Execution.relative_cost();
        assert!(g2 == g1 * 10);
        assert!(g3 == g2 * 10);
    }

    #[test]
    fn test_auto_approver_approves_safe() {
        let request = ApprovalRequest {
            project: "test".to_string(),
            action: "update documentation".to_string(),
            estimated_cost_tokens: 100,
            required_gate: ApprovalGate::Gate1Text,
            context: "user asked to update docs".to_string(),
        };
        let result = AutoApprover.approve(&request);
        assert!(result.approved);
    }

    #[test]
    fn test_auto_approver_blocks_delete() {
        let request = ApprovalRequest {
            project: "test".to_string(),
            action: "delete all files in /tmp".to_string(),
            estimated_cost_tokens: 100,
            required_gate: ApprovalGate::Gate1Text,
            context: "user asked to clean up".to_string(),
        };
        let result = AutoApprover.approve(&request);
        assert!(!result.approved);
    }

    #[test]
    fn test_cost_gated_pipeline_default() {
        let pipeline = CostGatedPipeline::default();
        assert_eq!(pipeline.gates.len(), 3);
        assert_eq!(pipeline.max_cost_tokens, 1_000_000);
    }

    #[test]
    fn test_pipeline_approves_gate1_safe() {
        let pipeline = CostGatedPipeline::new();
        let request = ApprovalRequest {
            project: "test".to_string(),
            action: "minor edit".to_string(),
            estimated_cost_tokens: 50,
            required_gate: ApprovalGate::Gate1Text,
            context: "approved edit".to_string(),
        };
        let results = pipeline.run_pipeline(&request);
        assert!(results.len() >= 1);
        assert!(results[0].approved);
    }

    #[test]
    fn test_pipeline_blocks_gate1_destructive() {
        let pipeline = CostGatedPipeline::new();
        let request = ApprovalRequest {
            project: "test".to_string(),
            action: "purge all data".to_string(),
            estimated_cost_tokens: 50,
            required_gate: ApprovalGate::Gate1Text,
            context: "".to_string(),
        };
        let results = pipeline.run_pipeline(&request);
        assert!(!results[0].approved);
    }

    #[test]
    fn test_pipeline_blocks_gate2_still() {
        let pipeline = CostGatedPipeline::new();
        let request = ApprovalRequest {
            project: "test".to_string(),
            action: "deploy to production".to_string(),
            estimated_cost_tokens: 5000,
            required_gate: ApprovalGate::Gate2Still,
            context: "".to_string(),
        };
        let results = pipeline.run_pipeline(&request);
        assert!(results[0].approved, "gate1 should pass");
        assert!(!results[1].approved, "gate2 should block");
    }

    #[test]
    fn test_pipeline_blocks_gate3_execution() {
        let pipeline = CostGatedPipeline::new();
        let request = ApprovalRequest {
            project: "test".to_string(),
            action: "release v2.0".to_string(),
            estimated_cost_tokens: 100_000,
            required_gate: ApprovalGate::Gate3Execution,
            context: "".to_string(),
        };
        let results = pipeline.run_pipeline(&request);
        assert!(results[0].approved, "gate1 should pass");
        assert!(!results[1].approved, "gate2 should block -> need still");
    }

    #[test]
    fn test_would_block_returns_gate() {
        let pipeline = CostGatedPipeline::new();
        let request = ApprovalRequest {
            project: "test".to_string(),
            action: "delete production db".to_string(),
            estimated_cost_tokens: 100,
            required_gate: ApprovalGate::Gate1Text,
            context: "".to_string(),
        };
        let blocked = pipeline.would_block(&request);
        assert!(blocked.is_some());
        assert_eq!(blocked.unwrap(), ApprovalGate::Gate1Text);
    }

    #[test]
    fn test_would_block_none_for_safe() {
        let pipeline = CostGatedPipeline::new();
        let request = ApprovalRequest {
            project: "test".to_string(),
            action: "add comment".to_string(),
            estimated_cost_tokens: 10,
            required_gate: ApprovalGate::Gate1Text,
            context: "approved".to_string(),
        };
        assert!(pipeline.would_block(&request).is_none());
    }

    #[test]
    fn test_cost_limit_exceeded() {
        let pipeline = CostGatedPipeline::new().with_max_cost(5);
        let request = ApprovalRequest {
            project: "test".to_string(),
            action: "safe edit".to_string(),
            estimated_cost_tokens: 10,
            required_gate: ApprovalGate::Gate2Still,
            context: "approved".to_string(),
        };
        let results = pipeline.run_pipeline(&request);
        assert!(results[0].approved, "gate1 cost=1 should pass");
        assert!(!results[1].approved, "gate2 cost=10 should exceed limit");
    }

    #[test]
    fn test_is_available() {
        assert!(is_available());
    }
}
