//! Delegation Protocol
//!
//! Enables parent agents to delegate subtasks to child agents with
//! a context envelope and provenance tracking. Supports the hierarchical
//! crew strategy and R-P130 (parent verifies child output).

use serde::{Deserialize, Serialize};

/// Status of a delegation response
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DelegationStatus {
    /// Child accepted and completed successfully
    Accepted,
    /// Child rejected the delegation (insufficient capability/tools)
    Rejected,
    /// Child exceeded iteration budget
    TimedOut,
    /// Child encountered an error
    Failed,
}

impl std::fmt::Display for DelegationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DelegationStatus::Accepted => write!(f, "Accepted"),
            DelegationStatus::Rejected => write!(f, "Rejected"),
            DelegationStatus::TimedOut => write!(f, "TimedOut"),
            DelegationStatus::Failed => write!(f, "Failed"),
        }
    }
}

/// A request from a parent agent to delegate a subtask to a child.
///
/// Carries the full context envelope so the child has sufficient
/// information to execute without additional back-and-forth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationRequest {
    /// Agent handle sending the delegation
    pub from_agent: String,
    /// Agent handle receiving the delegation
    pub to_agent: String,
    /// The subtask description
    pub task: String,
    /// Context envelope (prior results, constraints, etc.)
    pub context: String,
    /// Unix timestamp deadline (0 = no deadline)
    pub deadline: i64,
    /// Unique delegation ID for tracking
    pub id: String,
    /// Created timestamp
    pub created_at: i64,
}

impl DelegationRequest {
    pub fn new(from: &str, to: &str, task: &str) -> Self {
        Self {
            from_agent: from.to_string(),
            to_agent: to.to_string(),
            task: task.to_string(),
            context: String::new(),
            deadline: 0,
            id: uuid::Uuid::new_v4().to_string(),
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = context.into();
        self
    }

    pub fn with_deadline(mut self, deadline: i64) -> Self {
        self.deadline = deadline;
        self
    }

    /// Check if the delegation has expired
    pub fn is_expired(&self) -> bool {
        if self.deadline == 0 {
            return false;
        }
        chrono::Utc::now().timestamp() > self.deadline
    }
}

/// Response from a child agent after completing (or failing) a delegation.
///
/// Includes provenance information for audit trail and R-P130 verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationResponse {
    /// Status of the delegation
    pub status: DelegationStatus,
    /// Output produced by the child (empty on failure)
    pub result: String,
    /// Context to hand back to the parent (intermediate findings, etc.)
    pub handback_context: String,
    /// The original delegation ID this responds to
    pub delegation_id: String,
    /// Agent that produced the response
    pub from_agent: String,
    /// Iterations consumed
    pub iterations_used: u32,
    /// Tools actually invoked (provenance)
    pub tools_used: Vec<String>,
}

impl DelegationResponse {
    pub fn accepted(result: &str, delegation_id: &str, from: &str) -> Self {
        Self {
            status: DelegationStatus::Accepted,
            result: result.to_string(),
            handback_context: String::new(),
            delegation_id: delegation_id.to_string(),
            from_agent: from.to_string(),
            iterations_used: 1,
            tools_used: Vec::new(),
        }
    }

    pub fn rejected(delegation_id: &str, from: &str, reason: &str) -> Self {
        Self {
            status: DelegationStatus::Rejected,
            result: String::new(),
            handback_context: reason.to_string(),
            delegation_id: delegation_id.to_string(),
            from_agent: from.to_string(),
            iterations_used: 0,
            tools_used: Vec::new(),
        }
    }

    pub fn failed(delegation_id: &str, from: &str, error: &str) -> Self {
        Self {
            status: DelegationStatus::Failed,
            result: String::new(),
            handback_context: error.to_string(),
            delegation_id: delegation_id.to_string(),
            from_agent: from.to_string(),
            iterations_used: 0,
            tools_used: Vec::new(),
        }
    }

    pub fn timed_out(delegation_id: &str, from: &str) -> Self {
        Self {
            status: DelegationStatus::TimedOut,
            result: String::new(),
            handback_context: "iteration budget exceeded".to_string(),
            delegation_id: delegation_id.to_string(),
            from_agent: from.to_string(),
            iterations_used: 0,
            tools_used: Vec::new(),
        }
    }

    /// Builder: set handback context
    pub fn with_handback(mut self, ctx: impl Into<String>) -> Self {
        self.handback_context = ctx.into();
        self
    }

    /// Builder: record tools used
    pub fn with_tools_used(mut self, tools: Vec<&str>) -> Self {
        self.tools_used = tools.into_iter().map(String::from).collect();
        self
    }

    /// Builder: set iterations consumed
    pub fn with_iterations(mut self, n: u32) -> Self {
        self.iterations_used = n;
        self
    }

    /// Is this a successful response?
    pub fn is_success(&self) -> bool {
        self.status == DelegationStatus::Accepted
    }
}

/// Tracks the full delegation lifecycle for audit and R-P130 verification.
#[derive(Debug, Clone, Default)]
pub struct DelegationLog {
    entries: Vec<DelegationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationEntry {
    pub request: DelegationRequest,
    pub response: Option<DelegationResponse>,
    pub completed: bool,
}

impl DelegationLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Record a new delegation request
    pub fn record_request(&mut self, req: DelegationRequest) {
        self.entries.push(DelegationEntry {
            request: req,
            response: None,
            completed: false,
        });
    }

    /// Record the response for the most recent pending delegation to a given agent
    pub fn record_response(&mut self, to_agent: &str, resp: DelegationResponse) {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .rev()
            .find(|e| e.request.to_agent == to_agent && !e.completed)
        {
            entry.response = Some(resp);
            entry.completed = true;
        }
    }

    /// Count delegations to a specific agent
    pub fn count_to(&self, agent: &str) -> usize {
        self.entries
            .iter()
            .filter(|e| e.request.to_agent == agent)
            .count()
    }

    /// Count successful delegations to a specific agent
    pub fn success_count_to(&self, agent: &str) -> usize {
        self.entries
            .iter()
            .filter(|e| {
                e.request.to_agent == agent
                    && e.response
                        .as_ref()
                        .is_some_and(|r| r.is_success())
            })
            .count()
    }

    /// Total entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Perform a full delegation cycle: create request, check expiry, record.
pub fn delegate(
    log: &mut DelegationLog,
    from: &str,
    to: &str,
    task: &str,
    context: &str,
) -> DelegationRequest {
    let req = DelegationRequest::new(from, to, task).with_context(context);
    log.record_request(req.clone());
    req
}

/// Perform a handback: child returns result with provenance.
pub fn handback(
    log: &mut DelegationLog,
    req: &DelegationRequest,
    child: &str,
    result: &str,
    tools: Vec<&str>,
) -> DelegationResponse {
    let resp = DelegationResponse::accepted(result, &req.id, child)
        .with_tools_used(tools)
        .with_handback(result);
    log.record_response(&req.to_agent, resp.clone());
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delegation_request_builder() {
        let req = DelegationRequest::new("manager", "worker", "do task")
            .with_context("prior output")
            .with_deadline(1000);
        assert_eq!(req.from_agent, "manager");
        assert_eq!(req.to_agent, "worker");
        assert_eq!(req.context, "prior output");
        assert_eq!(req.deadline, 1000);
        assert!(!req.is_expired());
    }

    #[test]
    fn delegation_request_expiry() {
        let req = DelegationRequest::new("a", "b", "t").with_deadline(1);
        assert!(req.is_expired());
    }

    #[test]
    fn delegation_request_no_deadline() {
        let req = DelegationRequest::new("a", "b", "t");
        assert!(!req.is_expired());
    }

    #[test]
    fn response_accepted() {
        let resp = DelegationResponse::accepted("output", "d1", "worker");
        assert!(resp.is_success());
        assert_eq!(resp.result, "output");
        assert_eq!(resp.status, DelegationStatus::Accepted);
    }

    #[test]
    fn response_rejected() {
        let resp = DelegationResponse::rejected("d1", "worker", "no tools");
        assert!(!resp.is_success());
        assert_eq!(resp.handback_context, "no tools");
    }

    #[test]
    fn response_failed() {
        let resp = DelegationResponse::failed("d1", "worker", "crash");
        assert_eq!(resp.status, DelegationStatus::Failed);
    }

    #[test]
    fn response_timed_out() {
        let resp = DelegationResponse::timed_out("d1", "worker");
        assert_eq!(resp.status, DelegationStatus::TimedOut);
    }

    #[test]
    fn delegation_log_lifecycle() {
        let mut log = DelegationLog::new();
        let req = DelegationRequest::new("mgr", "w1", "task1");
        log.record_request(req.clone());
        assert_eq!(log.len(), 1);
        assert_eq!(log.count_to("w1"), 1);

        let resp = DelegationResponse::accepted("done", &req.id, "w1");
        log.record_response("w1", resp);
        assert_eq!(log.success_count_to("w1"), 1);
    }

    #[test]
    fn delegate_and_handback() {
        let mut log = DelegationLog::new();
        let req = delegate(&mut log, "mgr", "w1", "do it", "ctx");
        assert_eq!(log.len(), 1);

        let resp = handback(&mut log, &req, "w1", "result", vec!["file_read"]);
        assert!(resp.is_success());
        assert_eq!(resp.tools_used, vec!["file_read"]);
        assert_eq!(log.success_count_to("w1"), 1);
    }

    #[test]
    fn response_builder_chain() {
        let resp = DelegationResponse::accepted("r", "d1", "w1")
            .with_handback("extra ctx")
            .with_tools_used(vec!["tool1", "tool2"])
            .with_iterations(3);
        assert_eq!(resp.handback_context, "extra ctx");
        assert_eq!(resp.tools_used.len(), 2);
        assert_eq!(resp.iterations_used, 3);
    }

    #[test]
    fn status_display() {
        assert_eq!(DelegationStatus::Accepted.to_string(), "Accepted");
        assert_eq!(DelegationStatus::Rejected.to_string(), "Rejected");
        assert_eq!(DelegationStatus::Failed.to_string(), "Failed");
        assert_eq!(DelegationStatus::TimedOut.to_string(), "TimedOut");
    }

    #[test]
    fn delegation_serialization_roundtrip() {
        let req = DelegationRequest::new("a", "b", "t");
        let json = serde_json::to_string(&req).unwrap();
        let back: DelegationRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.id, back.id);
        assert_eq!(req.task, back.task);
    }
}
