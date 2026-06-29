// REVIVED Task 1 — dead_code removed 2026-06-24

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthorityLevel {
    Autonomous,
    Review,
    Approve,
    Escalate,
    Emergency,
}

#[derive(Debug, Clone)]
pub struct WillState {
    pub current_authority: AuthorityLevel,
    pub consecutive_failures: u32,
    pub last_decision_tick: u64,
    pub override_active: bool,
    pub override_until: u64,
}

#[derive(Debug, Clone)]
pub struct WillReceipt {
    pub id: u64,
    pub action: String,
    pub action_category: String,
    pub authority: AuthorityLevel,
    pub decision: String,
    pub reasoning: String,
    pub tick: u64,
    pub prior_receipt_hash: u64,
    pub receipt_hash: u64,
}

#[derive(Debug, Clone)]
pub struct ReceiptChain {
    pub receipts: Vec<WillReceipt>,
    pub max_len: usize,
}

#[derive(Debug, Clone)]
pub struct AuditReport {
    pub period_start: u64,
    pub period_end: u64,
    pub total_actions: usize,
    pub approved: usize,
    pub rejected: usize,
    pub overridden: usize,
    pub escalated: usize,
    pub authority_distribution: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct WillRule {
    pub category: String,
    pub min_authority: AuthorityLevel,
    pub max_consecutive_failures: u32,
    pub requires_reason: bool,
    pub cooldown_ticks: u64,
    pub last_action_tick: u64,
}

#[derive(Debug, Clone)]
pub struct UnifiedWill {
    pub state: WillState,
    pub chain: ReceiptChain,
    pub rules: Vec<WillRule>,
    pub tick: u64,
    next_id: u64,
}

fn compute_hash(id: u64, action: &str, decision: &str, prior_hash: u64) -> u64 {
    let mut h: u64 = 5381;
    h = h.wrapping_mul(33).wrapping_add(id);
    for b in action.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    for b in decision.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    h = h.wrapping_mul(33).wrapping_add(prior_hash);
    h
}

fn escalate(authority: AuthorityLevel) -> AuthorityLevel {
    match authority {
        AuthorityLevel::Autonomous => AuthorityLevel::Review,
        AuthorityLevel::Review => AuthorityLevel::Approve,
        AuthorityLevel::Approve => AuthorityLevel::Escalate,
        AuthorityLevel::Escalate => AuthorityLevel::Emergency,
        AuthorityLevel::Emergency => AuthorityLevel::Emergency,
    }
}

impl UnifiedWill {
    pub fn new() -> Self {
        let mut will = UnifiedWill {
            state: WillState {
                current_authority: AuthorityLevel::Autonomous,
                consecutive_failures: 0,
                last_decision_tick: 0,
                override_active: false,
                override_until: 0,
            },
            chain: ReceiptChain {
                receipts: Vec::new(),
                max_len: 1000,
            },
            rules: Vec::new(),
            tick: 0,
            next_id: 1,
        };
        will.add_rule(WillRule {
            category: "code_review".to_string(),
            min_authority: AuthorityLevel::Review,
            max_consecutive_failures: 3,
            requires_reason: true,
            cooldown_ticks: 10,
            last_action_tick: 0,
        });
        will.add_rule(WillRule {
            category: "self_modify".to_string(),
            min_authority: AuthorityLevel::Approve,
            max_consecutive_failures: 2,
            requires_reason: true,
            cooldown_ticks: 20,
            last_action_tick: 0,
        });
        will.add_rule(WillRule {
            category: "explore".to_string(),
            min_authority: AuthorityLevel::Autonomous,
            max_consecutive_failures: 5,
            requires_reason: false,
            cooldown_ticks: 0,
            last_action_tick: 0,
        });
        will.add_rule(WillRule {
            category: "communicate".to_string(),
            min_authority: AuthorityLevel::Autonomous,
            max_consecutive_failures: 5,
            requires_reason: false,
            cooldown_ticks: 0,
            last_action_tick: 0,
        });
        will.add_rule(WillRule {
            category: "memory_write".to_string(),
            min_authority: AuthorityLevel::Review,
            max_consecutive_failures: 3,
            requires_reason: true,
            cooldown_ticks: 5,
            last_action_tick: 0,
        });
        will.add_rule(WillRule {
            category: "system".to_string(),
            min_authority: AuthorityLevel::Approve,
            max_consecutive_failures: 1,
            requires_reason: true,
            cooldown_ticks: 50,
            last_action_tick: 0,
        });
        will
    }

    pub fn add_rule(&mut self, rule: WillRule) {
        if let Some(existing) = self.rules.iter_mut().find(|r| r.category == rule.category) {
            *existing = rule;
        } else {
            self.rules.push(rule);
        }
    }

    pub fn evaluate(&self, _action: &str, category: &str) -> AuthorityLevel {
        if self.is_override_active() {
            return self.state.current_authority;
        }
        let rule = self.rules.iter().find(|r| r.category == category);
        match rule {
            Some(r) => {
                let base = r.min_authority;
                if self.state.consecutive_failures >= r.max_consecutive_failures {
                    escalate(base)
                } else {
                    base
                }
            }
            None => AuthorityLevel::Autonomous,
        }
    }

    pub fn will_action(&mut self, action: &str, category: &str, reasoning: &str) -> WillReceipt {
        if self.state.override_active && self.tick >= self.state.override_until {
            self.state.override_active = false;
            self.state.current_authority = AuthorityLevel::Autonomous;
        }

        let authority = self.evaluate(action, category);
        let override_active = self.is_override_active();

        let decision = if override_active {
            "overridden".to_string()
        } else if authority == AuthorityLevel::Escalate || authority == AuthorityLevel::Emergency {
            "rejected".to_string()
        } else {
            "approved".to_string()
        };

        let id = self.next_id;
        self.next_id += 1;

        let prior_hash = self
            .chain
            .receipts
            .last()
            .map(|r| r.receipt_hash)
            .unwrap_or(0);
        let receipt_hash = compute_hash(id, action, &decision, prior_hash);

        let receipt = WillReceipt {
            id,
            action: action.to_string(),
            action_category: category.to_string(),
            authority,
            decision,
            reasoning: reasoning.to_string(),
            tick: self.tick,
            prior_receipt_hash: prior_hash,
            receipt_hash,
        };

        if self.chain.receipts.len() >= self.chain.max_len {
            self.chain.receipts.remove(0);
        }
        self.chain.receipts.push(receipt.clone());

        self.tick += 1;
        receipt
    }

    pub fn override_authority(&mut self, level: AuthorityLevel, duration_ticks: u64) {
        self.state.override_active = true;
        self.state.current_authority = level;
        self.state.override_until = self.tick + duration_ticks;
    }

    pub fn is_override_active(&self) -> bool {
        self.state.override_active && self.tick < self.state.override_until
    }

    pub fn record_failure(&mut self) {
        self.state.consecutive_failures += 1;
    }

    pub fn record_success(&mut self) {
        self.state.consecutive_failures = 0;
    }

    pub fn audit(&self, start_tick: u64, end_tick: u64) -> AuditReport {
        let relevant: Vec<&WillReceipt> = self
            .chain
            .receipts
            .iter()
            .filter(|r| r.tick >= start_tick && r.tick <= end_tick)
            .collect();

        let total_actions = relevant.len();
        let approved = relevant.iter().filter(|r| r.decision == "approved").count();
        let rejected = relevant.iter().filter(|r| r.decision == "rejected").count();
        let overridden = relevant
            .iter()
            .filter(|r| r.decision == "overridden")
            .count();
        let escalated = relevant
            .iter()
            .filter(|r| r.authority == AuthorityLevel::Escalate)
            .count();

        let mut authority_distribution: HashMap<String, usize> = HashMap::new();
        for r in &relevant {
            let key = format!("{:?}", r.authority);
            *authority_distribution.entry(key).or_insert(0) += 1;
        }

        AuditReport {
            period_start: start_tick,
            period_end: end_tick,
            total_actions,
            approved,
            rejected,
            overridden,
            escalated,
            authority_distribution,
        }
    }

    pub fn receipts_by_category(&self, category: &str) -> Vec<&WillReceipt> {
        self.chain
            .receipts
            .iter()
            .filter(|r| r.action_category == category)
            .collect()
    }

    pub fn verify_chain_integrity(&self) -> bool {
        let mut prev_hash: u64 = 0;
        for receipt in &self.chain.receipts {
            let expected = compute_hash(receipt.id, &receipt.action, &receipt.decision, prev_hash);
            if receipt.receipt_hash != expected {
                return false;
            }
            if receipt.prior_receipt_hash != prev_hash {
                return false;
            }
            prev_hash = receipt.receipt_hash;
        }
        true
    }

    pub fn suggest_escalation(&self) -> Option<String> {
        for rule in &self.rules {
            if self.state.consecutive_failures >= rule.max_consecutive_failures {
                return Some(format!(
                    "Escalation suggested for '{}': {} consecutive failures (max {})",
                    rule.category, self.state.consecutive_failures, rule.max_consecutive_failures,
                ));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_will_autonomous() {
        let will = UnifiedWill::new();
        assert_eq!(will.state.current_authority, AuthorityLevel::Autonomous);
        assert_eq!(will.state.consecutive_failures, 0);
        assert!(!will.state.override_active);
        assert_eq!(will.tick, 0);
        assert_eq!(will.chain.receipts.len(), 0);
    }

    #[test]
    fn test_will_action_autonomous_approved() {
        let mut will = UnifiedWill::new();
        let receipt = will.will_action("scan_network", "explore", "routine exploration");
        assert_eq!(receipt.authority, AuthorityLevel::Autonomous);
        assert_eq!(receipt.decision, "approved");
        assert_eq!(receipt.action, "scan_network");
        assert_eq!(receipt.action_category, "explore");
    }

    #[test]
    fn test_will_action_review_categories() {
        let mut will = UnifiedWill::new();
        let receipt = will.will_action("refactor_module", "code_review", "needs review");
        assert_eq!(receipt.authority, AuthorityLevel::Review);
        assert_eq!(receipt.decision, "approved");
        assert!(receipt.receipt_hash != 0);
    }

    #[test]
    fn test_receipt_chain_integrity() {
        let mut will = UnifiedWill::new();
        will.will_action("explore", "explore", "first");
        will.will_action("commit", "code_review", "second");
        will.will_action("modify", "self_modify", "third");
        will.will_action("message", "communicate", "fourth");
        assert!(will.verify_chain_integrity());
        assert_eq!(will.chain.receipts.len(), 4);
    }

    #[test]
    fn test_override_authority() {
        let mut will = UnifiedWill::new();
        will.override_authority(AuthorityLevel::Emergency, 100);
        assert!(will.is_override_active());
        assert_eq!(will.state.current_authority, AuthorityLevel::Emergency);

        let receipt = will.will_action("critical", "system", "emergency override");
        assert_eq!(receipt.decision, "overridden");
        assert_eq!(receipt.authority, AuthorityLevel::Emergency);
    }

    #[test]
    fn test_override_expiry() {
        let mut will = UnifiedWill::new();
        will.override_authority(AuthorityLevel::Emergency, 1);
        assert!(will.is_override_active());
        will.will_action("test", "explore", "triggers tick advance");
        assert!(!will.is_override_active());
    }

    #[test]
    fn test_consecutive_failures_escalation() {
        let mut will = UnifiedWill::new();
        let base = will.evaluate("write_memory", "memory_write");
        assert_eq!(base, AuthorityLevel::Review);

        will.record_failure();
        will.record_failure();
        will.record_failure();

        let escalated = will.evaluate("write_memory", "memory_write");
        assert_eq!(escalated, AuthorityLevel::Approve);
    }

    #[test]
    fn test_audit_report_counts() {
        let mut will = UnifiedWill::new();
        will.will_action("a", "explore", "");
        will.will_action("b", "explore", "");
        will.will_action("c", "code_review", "");

        let report = will.audit(0, 10);
        assert_eq!(report.total_actions, 3);
        assert_eq!(report.approved, 3);
        assert_eq!(report.rejected, 0);
        assert_eq!(report.overridden, 0);
        assert_eq!(report.escalated, 0);
    }

    #[test]
    fn test_receipts_by_category() {
        let mut will = UnifiedWill::new();
        will.will_action("scan", "explore", "");
        will.will_action("refactor", "code_review", "");
        will.will_action("probe", "explore", "");
        will.will_action("message", "communicate", "");

        let explore = will.receipts_by_category("explore");
        assert_eq!(explore.len(), 2);

        let review = will.receipts_by_category("code_review");
        assert_eq!(review.len(), 1);

        let unknown = will.receipts_by_category("unknown");
        assert_eq!(unknown.len(), 0);
    }

    #[test]
    fn test_suggest_escalation() {
        let mut will = UnifiedWill::new();
        assert!(will.suggest_escalation().is_none());

        will.record_failure();
        will.record_failure();

        let suggestion = will.suggest_escalation();
        assert!(suggestion.is_some());
        let msg = suggestion.unwrap();
        assert!(msg.contains("system"));
        assert!(msg.contains("2"));
    }

    #[test]
    fn test_multiple_rules() {
        let will = UnifiedWill::new();

        assert_eq!(will.evaluate("x", "explore"), AuthorityLevel::Autonomous);
        assert_eq!(
            will.evaluate("x", "communicate"),
            AuthorityLevel::Autonomous
        );
        assert_eq!(will.evaluate("x", "code_review"), AuthorityLevel::Review);
        assert_eq!(will.evaluate("x", "memory_write"), AuthorityLevel::Review);
        assert_eq!(will.evaluate("x", "self_modify"), AuthorityLevel::Approve);
        assert_eq!(will.evaluate("x", "system"), AuthorityLevel::Approve);
    }

    #[test]
    fn test_receipt_id_monotonic() {
        let mut will = UnifiedWill::new();
        let r1 = will.will_action("first", "explore", "");
        let r2 = will.will_action("second", "explore", "");
        let r3 = will.will_action("third", "explore", "");
        assert!(r1.id < r2.id && r2.id < r3.id);
    }

    #[test]
    fn test_record_success_resets_failures() {
        let mut will = UnifiedWill::new();
        will.record_failure();
        will.record_failure();
        will.record_failure();
        assert_eq!(will.state.consecutive_failures, 3);
        will.record_success();
        assert_eq!(will.state.consecutive_failures, 0);
    }

    #[test]
    fn test_verify_chain_corruption() {
        let mut will = UnifiedWill::new();
        will.will_action("a", "explore", "");
        will.will_action("b", "explore", "");
        will.will_action("c", "explore", "");
        assert!(will.verify_chain_integrity());

        will.chain.receipts[1].action = "corrupted".to_string();
        assert!(!will.verify_chain_integrity());
    }

    #[test]
    fn test_reject_escalated_action() {
        let mut will = UnifiedWill::new();
        will.record_failure();
        will.record_failure();

        let receipt = will.will_action("dangerous_op", "system", "too many failures");
        assert_eq!(receipt.decision, "rejected");
        assert_eq!(receipt.authority, AuthorityLevel::Emergency);
    }
}
