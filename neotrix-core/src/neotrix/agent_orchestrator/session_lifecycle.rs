use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    Spawning,
    Working,
    PrOpen,
    CiFailed,
    ReviewPending,
    ChangesRequested,
    Approved,
    Mergeable,
    Merged,
    Cleanup,
    Done,
    Killed,
    Failed,
}

impl SessionState {
    pub fn label(&self) -> &'static str {
        match self {
            SessionState::Spawning => "spawning",
            SessionState::Working => "working",
            SessionState::PrOpen => "pr_open",
            SessionState::CiFailed => "ci_failed",
            SessionState::ReviewPending => "review_pending",
            SessionState::ChangesRequested => "changes_requested",
            SessionState::Approved => "approved",
            SessionState::Mergeable => "mergeable",
            SessionState::Merged => "merged",
            SessionState::Cleanup => "cleanup",
            SessionState::Done => "done",
            SessionState::Killed => "killed",
            SessionState::Failed => "failed",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, SessionState::Done | SessionState::Killed | SessionState::Failed)
    }

    pub fn is_active(&self) -> bool {
        !self.is_terminal()
    }

    pub fn can_transition_to(&self, next: &SessionState) -> bool {
        if self.is_terminal() { return false; }
        match (self, next) {
            (SessionState::Spawning, SessionState::Working) => true,
            (SessionState::Spawning, SessionState::Killed) => true,
            (SessionState::Spawning, SessionState::Failed) => true,
            (SessionState::Working, SessionState::PrOpen) => true,
            (SessionState::Working, SessionState::Killed) => true,
            (SessionState::Working, SessionState::Failed) => true,
            (SessionState::PrOpen, SessionState::CiFailed) => true,
            (SessionState::PrOpen, SessionState::ReviewPending) => true,
            (SessionState::PrOpen, SessionState::Merged) => true,
            (SessionState::PrOpen, SessionState::Killed) => true,
            (SessionState::CiFailed, SessionState::Working) => true,
            (SessionState::CiFailed, SessionState::Killed) => true,
            (SessionState::CiFailed, SessionState::Failed) => true,
            (SessionState::ReviewPending, SessionState::ChangesRequested) => true,
            (SessionState::ReviewPending, SessionState::Approved) => true,
            (SessionState::ReviewPending, SessionState::Killed) => true,
            (SessionState::ChangesRequested, SessionState::Working) => true,
            (SessionState::ChangesRequested, SessionState::Killed) => true,
            (SessionState::Approved, SessionState::Mergeable) => true,
            (SessionState::Mergeable, SessionState::Merged) => true,
            (SessionState::Merged, SessionState::Cleanup) => true,
            (SessionState::Cleanup, SessionState::Done) => true,
            (SessionState::Cleanup, SessionState::Failed) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: String,
    pub state: SessionState,
    pub branch: String,
    pub worktree_path: Option<String>,
    pub pr_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub metadata: std::collections::HashMap<String, String>,
}

impl AgentSession {
    pub fn new(id: String, branch: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id, state: SessionState::Spawning,
            branch, worktree_path: None,
            pr_url: None, created_at: now.clone(),
            updated_at: now,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn transition(&mut self, next: SessionState) -> Result<(), String> {
        if !self.state.can_transition_to(&next) {
            return Err(format!("Cannot transition from {:?} to {:?}", self.state, next));
        }
        self.state = next;
        self.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_lifecycle_complete() {
        let mut session = AgentSession::new("s1".into(), "feat-x".into());
        assert_eq!(session.state, SessionState::Spawning);
        session.transition(SessionState::Working).expect("valid transition");
        session.transition(SessionState::PrOpen).expect("valid transition");
        session.transition(SessionState::ReviewPending).expect("valid transition");
        session.transition(SessionState::Approved).expect("valid transition");
        session.transition(SessionState::Mergeable).expect("valid transition");
        session.transition(SessionState::Merged).expect("valid transition");
        session.transition(SessionState::Cleanup).expect("valid transition");
        session.transition(SessionState::Done).expect("valid transition");
        assert!(session.state.is_terminal());
    }

    #[test]
    fn test_session_ci_fail_recovery() {
        let mut session = AgentSession::new("s2".into(), "fix".into());
        session.transition(SessionState::Working).expect("valid transition");
        session.transition(SessionState::PrOpen).expect("valid transition");
        session.transition(SessionState::CiFailed).expect("valid transition");
        session.transition(SessionState::Working).expect("valid transition");
        assert_eq!(session.state, SessionState::Working);
    }

    #[test]
    fn test_session_changes_requested() {
        let mut session = AgentSession::new("s3".into(), "feature".into());
        session.transition(SessionState::Working).expect("valid transition");
        session.transition(SessionState::PrOpen).expect("valid transition");
        session.transition(SessionState::ReviewPending).expect("valid transition");
        session.transition(SessionState::ChangesRequested).expect("valid transition");
        session.transition(SessionState::Working).expect("valid transition");
        assert_eq!(session.state, SessionState::Working);
    }

    #[test]
    fn test_invalid_transition() {
        let mut session = AgentSession::new("s4".into(), "bad".into());
        assert!(session.transition(SessionState::Merged).is_err());
    }

    #[test]
    fn test_terminal_blocked() {
        let mut session = AgentSession::new("s5".into(), "done".into());
        session.transition(SessionState::Working).expect("valid transition");
        session.transition(SessionState::Killed).expect("valid transition");
        assert!(session.transition(SessionState::Working).is_err());
    }

    #[test]
    fn test_session_labels() {
        assert_eq!(SessionState::Spawning.label(), "spawning");
        assert_eq!(SessionState::Merged.label(), "merged");
        assert_eq!(SessionState::Killed.label(), "killed");
    }
}
