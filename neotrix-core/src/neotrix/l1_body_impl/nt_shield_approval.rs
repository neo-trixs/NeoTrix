use serde::{Deserialize, Serialize};

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

        let expensive_keywords = ["deploy", "release", "publish", "migrate", "scale"];
        for kw in &expensive_keywords {
            if action_lower.contains(kw) && !context_lower.contains("approved") {
                issues.push(format!("expensive action '{}' requires explicit approval", kw));
                break;
            }
        }

        let known_domains = [
            "api.github.com", "api.openai.com", "registry.npmjs.org",
            "pypi.org", "hub.docker.com", "api.stripe.com",
        ];
        for domain in &known_domains {
            if action_lower.contains(domain) && !context_lower.contains(domain) {
                issues.push(format!("external API call to {} not referenced in context", domain));
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
