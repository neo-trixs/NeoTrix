#[derive(Debug, Clone)]
pub enum NodeType {
    Planner,
    Worker,
    Critic,
}

#[derive(Debug, Clone)]
pub struct LatentState {
    pub latent_summary: String,
    pub task_state: String,
    pub confidence: f64,
    pub metrics: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandoverRole {
    Planner,
    Builder,
    Reviewer,
    Verifier,
    Coordinator,
    Specialist(String),
}

#[derive(Debug, Clone)]
pub struct HandoverMessage {
    pub from: HandoverRole,
    pub to: HandoverRole,
    pub summary: String,
    pub baton: String,
    pub context: String,
    pub artifacts: Vec<String>,
    pub blockers: Vec<String>,
    pub confidence: f64,
    pub token_cost_estimate: Option<f64>,
}

impl HandoverMessage {
    pub fn new(from: HandoverRole, to: HandoverRole, summary: &str, baton: &str) -> Self {
        Self {
            from,
            to,
            summary: summary.to_string(),
            baton: baton.to_string(),
            context: String::new(),
            artifacts: Vec::new(),
            blockers: Vec::new(),
            confidence: 0.5,
            token_cost_estimate: None,
        }
    }

    pub fn with_context(mut self, context: &str) -> Self {
        self.context = context.to_string();
        self
    }

    pub fn with_artifact(mut self, artifact: &str) -> Self {
        self.artifacts.push(artifact.to_string());
        self
    }

    pub fn with_blocker(mut self, blocker: &str) -> Self {
        self.blockers.push(blocker.to_string());
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }
}

#[derive(Debug, Clone)]
pub struct AgentOutputContract {
    pub summary: String,
    pub changes: Vec<String>,
    pub evidence: Vec<String>,
    pub risks: Vec<String>,
    pub blockers: Vec<String>,
    pub handover: Option<HandoverMessage>,
}

impl AgentOutputContract {
    pub fn new(summary: &str) -> Self {
        Self {
            summary: summary.to_string(),
            changes: Vec::new(),
            evidence: Vec::new(),
            risks: Vec::new(),
            blockers: Vec::new(),
            handover: None,
        }
    }

    pub fn with_change(mut self, change: &str) -> Self {
        self.changes.push(change.to_string());
        self
    }

    pub fn with_evidence(mut self, evidence: &str) -> Self {
        self.evidence.push(evidence.to_string());
        self
    }

    pub fn with_risk(mut self, risk: &str) -> Self {
        self.risks.push(risk.to_string());
        self
    }

    pub fn with_blocker(mut self, blocker: &str) -> Self {
        self.blockers.push(blocker.to_string());
        self
    }

    pub fn with_handover(mut self, handover: HandoverMessage) -> Self {
        self.handover = Some(handover);
        self
    }

    pub fn format(&self) -> String {
        let mut output = format!("SUMMARY\n{}\n\n", self.summary);
        if !self.changes.is_empty() {
            output.push_str("CHANGES\n");
            for c in &self.changes {
                output.push_str(&format!("  - {}\n", c));
            }
            output.push('\n');
        }
        if !self.evidence.is_empty() {
            output.push_str("EVIDENCE\n");
            for e in &self.evidence {
                output.push_str(&format!("  - {}\n", e));
            }
            output.push('\n');
        }
        if !self.risks.is_empty() {
            output.push_str("RISKS\n");
            for r in &self.risks {
                output.push_str(&format!("  - {}\n", r));
            }
            output.push('\n');
        }
        if !self.blockers.is_empty() {
            output.push_str("BLOCKERS\n");
            for b in &self.blockers {
                output.push_str(&format!("  - {}\n", b));
            }
            output.push('\n');
        }
        if let Some(ref h) = self.handover {
            output.push_str(&format!(
                "HANDOVER\n  from: {:?} → to: {:?}\n  baton: {}\n",
                h.from, h.to, h.baton
            ));
        }
        output
    }
}

#[derive(Debug, Clone)]
pub struct HandoverRegistry {
    handovers: Vec<HandoverMessage>,
    max_history: usize,
}

impl HandoverRegistry {
    pub fn new(max_history: usize) -> Self {
        Self {
            handovers: Vec::new(),
            max_history,
        }
    }

    pub fn record(&mut self, handover: HandoverMessage) {
        self.handovers.push(handover);
        if self.handovers.len() > self.max_history {
            self.handovers.remove(0);
        }
    }

    pub fn recent(&self) -> &[HandoverMessage] {
        let start = self.handovers.len().saturating_sub(5);
        &self.handovers[start..]
    }

    pub fn all(&self) -> &[HandoverMessage] {
        &self.handovers
    }

    pub fn last_to_role(&self, role: &HandoverRole) -> Option<&HandoverMessage> {
        self.handovers.iter().rev().find(|h| h.to == *role)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handover_message_creation() {
        let msg = HandoverMessage::new(
            HandoverRole::Planner,
            HandoverRole::Builder,
            "Plan is ready",
            "Implement the Foo struct",
        );
        assert_eq!(msg.summary, "Plan is ready");
        assert_eq!(msg.baton, "Implement the Foo struct");
    }

    #[test]
    fn test_handover_message_builder() {
        let msg = HandoverMessage::new(
            HandoverRole::Planner,
            HandoverRole::Builder,
            "Design complete",
            "Build it",
        )
        .with_context("Project context")
        .with_artifact("design.md")
        .with_blocker("Need API key")
        .with_confidence(0.85);

        assert!(msg.context.contains("Project context"));
        assert_eq!(msg.artifacts.len(), 1);
        assert_eq!(msg.blockers.len(), 1);
        assert!((msg.confidence - 0.85).abs() < 1e-10);
    }

    #[test]
    fn test_agent_output_contract_creation() {
        let contract = AgentOutputContract::new("Implemented feature X")
            .with_change("src/foo.rs")
            .with_evidence("tests pass")
            .with_risk("edge case in bar")
            .with_blocker("needs review");

        assert_eq!(contract.summary, "Implemented feature X");
        assert_eq!(contract.changes.len(), 1);
        assert_eq!(contract.evidence.len(), 1);
        assert_eq!(contract.risks.len(), 1);
        assert_eq!(contract.blockers.len(), 1);
    }

    #[test]
    fn test_agent_output_contract_with_handover() {
        let handover = HandoverMessage::new(
            HandoverRole::Builder,
            HandoverRole::Reviewer,
            "Done",
            "Check the edge cases",
        );
        let contract = AgentOutputContract::new("Built feature").with_handover(handover);
        assert!(contract.handover.is_some());
    }

    #[test]
    fn test_agent_output_contract_format() {
        let contract = AgentOutputContract::new("Feature complete")
            .with_change("src/lib.rs")
            .with_risk("performance");
        let formatted = contract.format();
        assert!(formatted.contains("SUMMARY"));
        assert!(formatted.contains("Feature complete"));
        assert!(formatted.contains("CHANGES"));
        assert!(formatted.contains("src/lib.rs"));
        assert!(formatted.contains("RISKS"));
        assert!(formatted.contains("performance"));
    }

    #[test]
    fn test_handover_registry_records_and_recalls() {
        let mut registry = HandoverRegistry::new(10);
        let msg = HandoverMessage::new(
            HandoverRole::Planner,
            HandoverRole::Builder,
            "Plan ready",
            "Implement",
        );
        registry.record(msg);
        assert_eq!(registry.recent().len(), 1);
    }

    #[test]
    fn test_handover_registry_max_history() {
        let mut registry = HandoverRegistry::new(3);
        for i in 0..5 {
            let msg = HandoverMessage::new(
                HandoverRole::Planner,
                HandoverRole::Builder,
                &format!("Plan {}", i),
                "bat",
            );
            registry.record(msg);
        }
        assert_eq!(registry.all().len(), 3);
    }

    #[test]
    fn test_handover_registry_last_to_role() {
        let mut registry = HandoverRegistry::new(10);
        registry.record(HandoverMessage::new(
            HandoverRole::Planner,
            HandoverRole::Builder,
            "Go",
            "build",
        ));
        let last = registry.last_to_role(&HandoverRole::Builder);
        assert!(last.is_some());
        let not_found = registry.last_to_role(&HandoverRole::Reviewer);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_handover_confidence_clamped() {
        let msg = HandoverMessage::new(
            HandoverRole::Specialist("coder".into()),
            HandoverRole::Verifier,
            "Done",
            "check",
        )
        .with_confidence(2.0);
        assert!((msg.confidence - 1.0).abs() < 1e-10);

        let msg2 = msg.with_confidence(-0.5);
        assert!((msg2.confidence - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_handover_role_equality() {
        let a = HandoverRole::Specialist("coder".into());
        let b = HandoverRole::Specialist("coder".into());
        let c = HandoverRole::Specialist("tester".into());
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(HandoverRole::Planner, HandoverRole::Builder);
    }
}

