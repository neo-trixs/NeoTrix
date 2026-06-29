use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::nt_core_consciousness::consciousness_architecture::GapSeverity;
use crate::core::nt_core_experience::gap_detector_bridge::{
    ArchitectureGapReport, GapClosureSuggestion,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalPriority {
    Critical,
    High,
    Medium,
    Low,
}

impl ProposalPriority {
    pub fn name(&self) -> &'static str {
        match self {
            ProposalPriority::Critical => "critical",
            ProposalPriority::High => "high",
            ProposalPriority::Medium => "medium",
            ProposalPriority::Low => "low",
        }
    }

    pub fn priority_value(&self) -> u8 {
        match self {
            ProposalPriority::Critical => 0,
            ProposalPriority::High => 1,
            ProposalPriority::Medium => 2,
            ProposalPriority::Low => 3,
        }
    }

    pub fn from_gap_severity(severity: GapSeverity) -> Self {
        match severity {
            GapSeverity::Survival => ProposalPriority::Critical,
            GapSeverity::Evolution => ProposalPriority::High,
            GapSeverity::Enhancement => ProposalPriority::Medium,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalStatus {
    Draft,
    Submitted,
    InReview,
    Approved,
    Implemented,
    Rejected,
    RolledBack,
}

impl ProposalStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ProposalStatus::Draft => "draft",
            ProposalStatus::Submitted => "submitted",
            ProposalStatus::InReview => "in_review",
            ProposalStatus::Approved => "approved",
            ProposalStatus::Implemented => "implemented",
            ProposalStatus::Rejected => "rejected",
            ProposalStatus::RolledBack => "rolled_back",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ProposalStatus::Implemented | ProposalStatus::Rejected | ProposalStatus::RolledBack
        )
    }
}

#[derive(Debug, Clone)]
pub struct SealProposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub target_module: String,
    pub priority: ProposalPriority,
    pub estimated_risk: f64,
    pub estimated_impact: f64,
    pub implementation_hint: String,
    pub verification_hint: String,
    pub source_gap: String,
    pub created_at: u64,
    pub status: ProposalStatus,
}

impl SealProposal {
    pub fn risk_label(&self) -> &'static str {
        if self.estimated_risk >= 0.7 {
            "high"
        } else if self.estimated_risk >= 0.3 {
            "medium"
        } else {
            "low"
        }
    }

    pub fn impact_label(&self) -> &'static str {
        if self.estimated_impact >= 0.7 {
            "high"
        } else if self.estimated_impact >= 0.3 {
            "medium"
        } else {
            "low"
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "[{:?}] #{} {} — risk={:.2} impact={:.2} status={:?}",
            self.priority,
            self.id,
            self.title,
            self.estimated_risk,
            self.estimated_impact,
            self.status
        )
    }
}

pub struct SealProposalBridge {
    pub proposals: Vec<SealProposal>,
    pub next_id: u64,
    pub proposals_created: u64,
    pub proposals_implemented: u64,
}

impl SealProposalBridge {
    pub fn new() -> Self {
        SealProposalBridge {
            proposals: Vec::new(),
            next_id: 1,
            proposals_created: 0,
            proposals_implemented: 0,
        }
    }

    pub fn generate_proposals(&mut self, gap_report: &ArchitectureGapReport) -> Vec<SealProposal> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut generated = Vec::new();
        for suggestion in &gap_report.gap_closure_suggestions {
            let proposal = self.propose_for_gap_with_timestamp(suggestion, now);
            self.proposals.push(proposal.clone());
            generated.push(proposal);
        }
        generated
    }

    pub fn propose_for_gap(&mut self, suggestion: &GapClosureSuggestion) -> SealProposal {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.propose_for_gap_with_timestamp(suggestion, now)
    }

    fn propose_for_gap_with_timestamp(
        &mut self,
        suggestion: &GapClosureSuggestion,
        now: u64,
    ) -> SealProposal {
        let id = self.next_id;
        self.next_id += 1;
        self.proposals_created += 1;

        let priority = ProposalPriority::from_gap_severity(suggestion.priority);
        let (estimated_risk, estimated_impact) = match suggestion.priority {
            GapSeverity::Survival => (0.4, 0.8),
            GapSeverity::Evolution => (0.2, 0.5),
            GapSeverity::Enhancement => (0.1, 0.3),
        };

        let title = format!("Close gap: {}", suggestion.gap_id);
        let target_module = suggestion.gap_id.clone();

        SealProposal {
            id,
            title,
            description: suggestion.suggestion.clone(),
            target_module,
            priority,
            estimated_risk,
            estimated_impact,
            implementation_hint: suggestion.suggestion.clone(),
            verification_hint: format!(
                "After implementing {}, run gap detection and confirm the gap is closed",
                suggestion.gap_id
            ),
            source_gap: suggestion.gap_id.clone(),
            created_at: now,
            status: ProposalStatus::Draft,
        }
    }

    pub fn propose_new_capability(&mut self, module_name: &str, description: &str) -> SealProposal {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let id = self.next_id;
        self.next_id += 1;
        self.proposals_created += 1;

        SealProposal {
            id,
            title: format!("Add capability: {}", module_name),
            description: description.to_string(),
            target_module: module_name.to_string(),
            priority: ProposalPriority::Medium,
            estimated_risk: 0.15,
            estimated_impact: 0.4,
            implementation_hint: format!(
                "Implement {} module with VSA-native interface",
                module_name
            ),
            verification_hint: format!(
                "Verify {} registers with architecture and passes integration tests",
                module_name
            ),
            source_gap: "capability_planning".to_string(),
            created_at: now,
            status: ProposalStatus::Draft,
        }
    }

    pub fn approve_proposal(&mut self, id: u64) {
        if let Some(p) = self.proposals.iter_mut().find(|p| p.id == id) {
            if p.status == ProposalStatus::Draft
                || p.status == ProposalStatus::Submitted
                || p.status == ProposalStatus::InReview
            {
                p.status = ProposalStatus::Approved;
            }
        }
    }

    pub fn reject_proposal(&mut self, id: u64) {
        if let Some(p) = self.proposals.iter_mut().find(|p| p.id == id) {
            if !p.status.is_terminal() {
                p.status = ProposalStatus::Rejected;
            }
        }
    }

    pub fn mark_implemented(&mut self, id: u64) {
        if let Some(p) = self.proposals.iter_mut().find(|p| p.id == id) {
            if p.status == ProposalStatus::Approved {
                p.status = ProposalStatus::Implemented;
                self.proposals_implemented += 1;
            }
        }
    }

    pub fn submit_proposal(&mut self, id: u64) {
        if let Some(p) = self.proposals.iter_mut().find(|p| p.id == id) {
            if p.status == ProposalStatus::Draft {
                p.status = ProposalStatus::Submitted;
            }
        }
    }

    pub fn start_review(&mut self, id: u64) {
        if let Some(p) = self.proposals.iter_mut().find(|p| p.id == id) {
            if p.status == ProposalStatus::Submitted {
                p.status = ProposalStatus::InReview;
            }
        }
    }

    pub fn rollback(&mut self, id: u64) {
        if let Some(p) = self.proposals.iter_mut().find(|p| p.id == id) {
            if p.status == ProposalStatus::Implemented {
                p.status = ProposalStatus::RolledBack;
                self.proposals_implemented = self.proposals_implemented.saturating_sub(1);
            }
        }
    }

    pub fn pending_proposals(&self) -> Vec<&SealProposal> {
        self.proposals
            .iter()
            .filter(|p| {
                matches!(
                    p.status,
                    ProposalStatus::Draft | ProposalStatus::Submitted | ProposalStatus::InReview
                )
            })
            .collect()
    }

    pub fn approved_proposals(&self) -> Vec<&SealProposal> {
        self.proposals
            .iter()
            .filter(|p| p.status == ProposalStatus::Approved)
            .collect()
    }

    pub fn proposals_by_priority(&self, priority: ProposalPriority) -> Vec<&SealProposal> {
        self.proposals
            .iter()
            .filter(|p| p.priority == priority)
            .collect()
    }

    pub fn find_by_id(&self, id: u64) -> Option<&SealProposal> {
        self.proposals.iter().find(|p| p.id == id)
    }

    pub fn stats(&self) -> SealProposalBridgeStats {
        let total = self.proposals.len();
        let pending = self.pending_proposals().len();
        let approved = self.approved_proposals().len();
        let implemented = self
            .proposals
            .iter()
            .filter(|p| p.status == ProposalStatus::Implemented)
            .count();
        let rejected = self
            .proposals
            .iter()
            .filter(|p| p.status == ProposalStatus::Rejected)
            .count();
        let critical = self.proposals_by_priority(ProposalPriority::Critical).len();

        SealProposalBridgeStats {
            total_proposals: total,
            pending,
            approved,
            implemented,
            rejected,
            critical_proposals: critical,
        }
    }
}

impl Default for SealProposalBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SealProposalBridgeStats {
    pub total_proposals: usize,
    pub pending: usize,
    pub approved: usize,
    pub implemented: usize,
    pub rejected: usize,
    pub critical_proposals: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_suggestion(gap_id: &str, priority: GapSeverity, effort: &str) -> GapClosureSuggestion {
        GapClosureSuggestion {
            gap_id: gap_id.to_string(),
            suggestion: format!("Implement {}: some implementation detail", gap_id),
            priority,
            estimated_effort: effort.to_string(),
        }
    }

    fn make_report(suggestions: Vec<GapClosureSuggestion>) -> ArchitectureGapReport {
        ArchitectureGapReport {
            total_capabilities: 30,
            total_degraded: 2,
            total_missing: suggestions.len(),
            unregistered_modules: vec![],
            health_discrepancies: vec![],
            gap_closure_suggestions: suggestions,
            overall_health_score: 0.65,
            report_timestamp: 1000,
        }
    }

    #[test]
    fn test_bridge_new() {
        let bridge = SealProposalBridge::new();
        assert_eq!(bridge.next_id, 1);
        assert!(bridge.proposals.is_empty());
        assert_eq!(bridge.proposals_created, 0);
        assert_eq!(bridge.proposals_implemented, 0);
    }

    #[test]
    fn test_propose_for_gap_survival() {
        let mut bridge = SealProposalBridge::new();
        let suggestion = make_suggestion("mcts_reasoning", GapSeverity::Survival, "medium");
        let proposal = bridge.propose_for_gap(&suggestion);

        assert_eq!(proposal.priority, ProposalPriority::Critical);
        assert_eq!(proposal.estimated_risk, 0.4);
        assert_eq!(proposal.estimated_impact, 0.8);
        assert_eq!(proposal.source_gap, "mcts_reasoning");
        assert_eq!(proposal.status, ProposalStatus::Draft);
        assert_eq!(proposal.id, 1);
        assert_eq!(bridge.proposals_created, 1);
    }

    #[test]
    fn test_propose_for_gap_evolution() {
        let mut bridge = SealProposalBridge::new();
        let suggestion = make_suggestion("analogy_reasoning", GapSeverity::Evolution, "medium");
        let proposal = bridge.propose_for_gap(&suggestion);

        assert_eq!(proposal.priority, ProposalPriority::High);
        assert_eq!(proposal.estimated_risk, 0.2);
        assert_eq!(proposal.estimated_impact, 0.5);
        assert_eq!(proposal.source_gap, "analogy_reasoning");
    }

    #[test]
    fn test_propose_for_gap_enhancement() {
        let mut bridge = SealProposalBridge::new();
        let suggestion = make_suggestion("spatial_scene", GapSeverity::Enhancement, "small");
        let proposal = bridge.propose_for_gap(&suggestion);

        assert_eq!(proposal.priority, ProposalPriority::Medium);
        assert_eq!(proposal.estimated_risk, 0.1);
        assert_eq!(proposal.estimated_impact, 0.3);
        assert_eq!(proposal.source_gap, "spatial_scene");
    }

    #[test]
    fn test_generate_proposals_from_report() {
        let mut bridge = SealProposalBridge::new();
        let suggestions = vec![
            make_suggestion("mcts_reasoning", GapSeverity::Survival, "medium"),
            make_suggestion("analogy_reasoning", GapSeverity::Evolution, "medium"),
            make_suggestion("spatial_scene", GapSeverity::Enhancement, "small"),
        ];
        let report = make_report(suggestions);

        let generated = bridge.generate_proposals(&report);
        assert_eq!(generated.len(), 3);
        assert_eq!(bridge.proposals.len(), 3);
        assert_eq!(bridge.proposals_created, 3);

        assert_eq!(generated[0].priority, ProposalPriority::Critical);
        assert_eq!(generated[1].priority, ProposalPriority::High);
        assert_eq!(generated[2].priority, ProposalPriority::Medium);
    }

    #[test]
    fn test_propose_new_capability() {
        let mut bridge = SealProposalBridge::new();
        let proposal = bridge.propose_new_capability("key_vault", "Encrypted credential storage");

        assert_eq!(proposal.priority, ProposalPriority::Medium);
        assert_eq!(proposal.target_module, "key_vault");
        assert!(proposal
            .description
            .contains("Encrypted credential storage"));
        assert_eq!(proposal.id, 1);
        assert_eq!(bridge.proposals_created, 1);
    }

    #[test]
    fn test_approve_proposal() {
        let mut bridge = SealProposalBridge::new();
        let suggestion = make_suggestion("mcts_reasoning", GapSeverity::Survival, "medium");
        let proposal = bridge.propose_for_gap(&suggestion);

        assert_eq!(proposal.status, ProposalStatus::Draft);
        bridge.approve_proposal(1);
        assert_eq!(
            bridge.find_by_id(1).unwrap().status,
            ProposalStatus::Approved
        );
    }

    #[test]
    fn test_reject_proposal() {
        let mut bridge = SealProposalBridge::new();
        bridge.propose_for_gap(&make_suggestion(
            "mcts_reasoning",
            GapSeverity::Survival,
            "medium",
        ));
        bridge.reject_proposal(1);
        assert_eq!(
            bridge.find_by_id(1).unwrap().status,
            ProposalStatus::Rejected
        );
    }

    #[test]
    fn test_mark_implemented() {
        let mut bridge = SealProposalBridge::new();
        bridge.propose_for_gap(&make_suggestion(
            "mcts_reasoning",
            GapSeverity::Survival,
            "medium",
        ));
        bridge.approve_proposal(1);
        bridge.mark_implemented(1);

        assert_eq!(
            bridge.find_by_id(1).unwrap().status,
            ProposalStatus::Implemented
        );
        assert_eq!(bridge.proposals_implemented, 1);
    }

    #[test]
    fn test_mark_implemented_requires_approved() {
        let mut bridge = SealProposalBridge::new();
        bridge.propose_for_gap(&make_suggestion(
            "mcts_reasoning",
            GapSeverity::Survival,
            "medium",
        ));
        bridge.mark_implemented(1);
        assert_eq!(bridge.find_by_id(1).unwrap().status, ProposalStatus::Draft);
        assert_eq!(bridge.proposals_implemented, 0);
    }

    #[test]
    fn test_submit_and_review_flow() {
        let mut bridge = SealProposalBridge::new();
        bridge.propose_for_gap(&make_suggestion(
            "mcts_reasoning",
            GapSeverity::Survival,
            "medium",
        ));

        assert_eq!(bridge.find_by_id(1).unwrap().status, ProposalStatus::Draft);
        bridge.submit_proposal(1);
        assert_eq!(
            bridge.find_by_id(1).unwrap().status,
            ProposalStatus::Submitted
        );
        bridge.start_review(1);
        assert_eq!(
            bridge.find_by_id(1).unwrap().status,
            ProposalStatus::InReview
        );
        bridge.approve_proposal(1);
        assert_eq!(
            bridge.find_by_id(1).unwrap().status,
            ProposalStatus::Approved
        );
    }

    #[test]
    fn test_rollback() {
        let mut bridge = SealProposalBridge::new();
        bridge.propose_for_gap(&make_suggestion(
            "mcts_reasoning",
            GapSeverity::Survival,
            "medium",
        ));
        bridge.approve_proposal(1);
        bridge.mark_implemented(1);
        assert_eq!(bridge.proposals_implemented, 1);

        bridge.rollback(1);
        assert_eq!(
            bridge.find_by_id(1).unwrap().status,
            ProposalStatus::RolledBack
        );
        assert_eq!(bridge.proposals_implemented, 0);
    }

    #[test]
    fn test_pending_proposals() {
        let mut bridge = SealProposalBridge::new();
        bridge.propose_for_gap(&make_suggestion("mcts", GapSeverity::Survival, "medium"));
        bridge.propose_for_gap(&make_suggestion(
            "analogy",
            GapSeverity::Evolution,
            "medium",
        ));
        bridge.approve_proposal(2);

        let pending = bridge.pending_proposals();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, 1);

        let approved = bridge.approved_proposals();
        assert_eq!(approved.len(), 1);
        assert_eq!(approved[0].id, 2);
    }

    #[test]
    fn test_proposals_by_priority() {
        let mut bridge = SealProposalBridge::new();
        bridge.propose_for_gap(&make_suggestion("mcts", GapSeverity::Survival, "medium"));
        bridge.propose_for_gap(&make_suggestion(
            "analogy",
            GapSeverity::Evolution,
            "medium",
        ));

        let critical = bridge.proposals_by_priority(ProposalPriority::Critical);
        let high = bridge.proposals_by_priority(ProposalPriority::High);
        assert_eq!(critical.len(), 1);
        assert_eq!(high.len(), 1);
    }

    #[test]
    fn test_stats() {
        let mut bridge = SealProposalBridge::new();
        bridge.propose_for_gap(&make_suggestion("mcts", GapSeverity::Survival, "medium"));
        bridge.propose_for_gap(&make_suggestion(
            "analogy",
            GapSeverity::Evolution,
            "medium",
        ));
        bridge.approve_proposal(1);
        bridge.mark_implemented(1);

        let stats = bridge.stats();
        assert_eq!(stats.total_proposals, 2);
        assert_eq!(stats.pending, 1);
        assert_eq!(stats.approved, 0);
        assert_eq!(stats.implemented, 1);
        assert_eq!(stats.rejected, 0);
        assert_eq!(stats.critical_proposals, 1);
    }

    #[test]
    fn test_proposal_summary() {
        let mut bridge = SealProposalBridge::new();
        let proposal =
            bridge.propose_for_gap(&make_suggestion("mcts", GapSeverity::Survival, "medium"));
        let summary = proposal.summary();
        assert!(summary.contains("Critical"));
        assert!(summary.contains("mcts"));
        assert!(summary.contains("risk=0.40"));
        assert!(summary.contains("impact=0.80"));
    }

    #[test]
    fn test_risk_impact_labels() {
        let mut bridge = SealProposalBridge::new();
        let p1 = bridge.propose_for_gap(&make_suggestion("a", GapSeverity::Survival, "m"));
        assert_eq!(p1.risk_label(), "medium");
        assert_eq!(p1.impact_label(), "high");

        let mut low_risk = SealProposal {
            estimated_risk: 0.05,
            estimated_impact: 0.15,
            ..p1.clone()
        };
        assert_eq!(low_risk.risk_label(), "low");
        assert_eq!(low_risk.impact_label(), "low");

        let mut high_risk = SealProposal {
            estimated_risk: 0.85,
            estimated_impact: 0.95,
            ..p1
        };
        assert_eq!(high_risk.risk_label(), "high");
        assert_eq!(high_risk.impact_label(), "high");
    }

    #[test]
    fn test_proposal_status_terminal() {
        assert!(!ProposalStatus::Draft.is_terminal());
        assert!(!ProposalStatus::Submitted.is_terminal());
        assert!(!ProposalStatus::InReview.is_terminal());
        assert!(!ProposalStatus::Approved.is_terminal());
        assert!(ProposalStatus::Implemented.is_terminal());
        assert!(ProposalStatus::Rejected.is_terminal());
        assert!(ProposalStatus::RolledBack.is_terminal());
    }

    #[test]
    fn test_proposal_priority_mapping() {
        assert_eq!(
            ProposalPriority::from_gap_severity(GapSeverity::Survival),
            ProposalPriority::Critical
        );
        assert_eq!(
            ProposalPriority::from_gap_severity(GapSeverity::Evolution),
            ProposalPriority::High
        );
        assert_eq!(
            ProposalPriority::from_gap_severity(GapSeverity::Enhancement),
            ProposalPriority::Medium
        );
    }

    #[test]
    fn test_priority_value_ordering() {
        assert!(
            ProposalPriority::Critical.priority_value() < ProposalPriority::High.priority_value()
        );
        assert!(
            ProposalPriority::High.priority_value() < ProposalPriority::Medium.priority_value()
        );
        assert!(ProposalPriority::Medium.priority_value() < ProposalPriority::Low.priority_value());
    }

    #[test]
    fn test_generate_proposals_id_sequence() {
        let mut bridge = SealProposalBridge::new();
        let s1 = make_suggestion("mcts", GapSeverity::Survival, "m");
        let s2 = make_suggestion("analogy", GapSeverity::Evolution, "m");
        let s3 = make_suggestion("spatial", GapSeverity::Enhancement, "s");
        let report = make_report(vec![s1, s2, s3]);

        let proposals = bridge.generate_proposals(&report);
        assert_eq!(proposals[0].id, 1);
        assert_eq!(proposals[1].id, 2);
        assert_eq!(proposals[2].id, 3);
    }

    #[test]
    fn test_empty_report_generates_no_proposals() {
        let mut bridge = SealProposalBridge::new();
        let report = make_report(vec![]);
        let proposals = bridge.generate_proposals(&report);
        assert!(proposals.is_empty());
        assert_eq!(bridge.proposals_created, 0);
    }

    #[test]
    fn test_reject_terminal_proposal_noop() {
        let mut bridge = SealProposalBridge::new();
        bridge.propose_for_gap(&make_suggestion("mcts", GapSeverity::Survival, "m"));
        bridge.approve_proposal(1);
        bridge.mark_implemented(1);
        bridge.reject_proposal(1);
        assert_eq!(
            bridge.find_by_id(1).unwrap().status,
            ProposalStatus::Implemented
        );
    }

    #[test]
    fn test_approve_skips_terminal() {
        let mut bridge = SealProposalBridge::new();
        bridge.propose_for_gap(&make_suggestion("mcts", GapSeverity::Survival, "m"));
        bridge.reject_proposal(1);
        bridge.approve_proposal(1);
        assert_eq!(
            bridge.find_by_id(1).unwrap().status,
            ProposalStatus::Rejected
        );
    }

    #[test]
    fn test_stats_empty_bridge() {
        let bridge = SealProposalBridge::new();
        let stats = bridge.stats();
        assert_eq!(stats.total_proposals, 0);
        assert_eq!(stats.pending, 0);
        assert_eq!(stats.approved, 0);
        assert_eq!(stats.implemented, 0);
        assert_eq!(stats.rejected, 0);
        assert_eq!(stats.critical_proposals, 0);
    }

    #[test]
    fn test_find_by_id_nonexistent() {
        let bridge = SealProposalBridge::new();
        assert!(bridge.find_by_id(42).is_none());
    }

    #[test]
    fn test_proposal_status_labels() {
        assert_eq!(ProposalStatus::Draft.label(), "draft");
        assert_eq!(ProposalStatus::Submitted.label(), "submitted");
        assert_eq!(ProposalStatus::InReview.label(), "in_review");
        assert_eq!(ProposalStatus::Approved.label(), "approved");
        assert_eq!(ProposalStatus::Implemented.label(), "implemented");
        assert_eq!(ProposalStatus::Rejected.label(), "rejected");
        assert_eq!(ProposalStatus::RolledBack.label(), "rolled_back");
    }

    #[test]
    fn test_proposal_priority_names() {
        assert_eq!(ProposalPriority::Critical.name(), "critical");
        assert_eq!(ProposalPriority::High.name(), "high");
        assert_eq!(ProposalPriority::Medium.name(), "medium");
        assert_eq!(ProposalPriority::Low.name(), "low");
    }
}
