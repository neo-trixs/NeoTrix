#![forbid(unsafe_code)]
//! App Server — 对标 openai/codex app-server: thread / turn / stream / approval
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HarnessThread {
    pub id: String,
    pub project: Option<String>,
    pub created_at: u64,
    pub turn_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessTurn {
    pub id: String,
    pub thread_id: String,
    pub instruction: String,
    pub capability_tag: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalState {
    Pending,
    Approved,
    Denied,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub turn_id: String,
    pub action: String,
    pub target: String,
    pub state: ApprovalState,
    pub created_at: u64,
}

impl ApprovalRequest {
    pub fn new(turn_id: &str, action: &str, target: &str) -> Self {
        Self {
            id: format!("appr-{}", uuid::Uuid::new_v4()),
            turn_id: turn_id.to_string(),
            action: action.to_string(),
            target: target.to_string(),
            state: ApprovalState::Pending,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }
}

#[derive(Debug, Default)]
pub struct ThreadStore {
    threads: HashMap<String, HarnessThread>,
    turns: HashMap<String, HarnessTurn>,
    approvals: HashMap<String, ApprovalRequest>,
}

impl ThreadStore {
    pub fn create_thread(&mut self, project: Option<String>) -> HarnessThread {
        let id = format!("thr-{}", uuid::Uuid::new_v4());
        let t = HarnessThread { id: id.clone(), project, created_at: now_secs(), turn_count: 0 };
        self.threads.insert(id, t.clone());
        t
    }
    pub fn get_thread(&self, id: &str) -> Option<&HarnessThread> {
        self.threads.get(id)
    }
    pub fn list_threads(&self) -> Vec<&HarnessThread> {
        self.threads.values().collect()
    }
    pub fn start_turn(&mut self, thread_id: &str, instruction: &str, tag: &str) -> Option<HarnessTurn> {
        let thr = self.threads.get_mut(thread_id)?;
        thr.turn_count += 1;
        let turn = HarnessTurn {
            id: format!("turn-{}", uuid::Uuid::new_v4()),
            thread_id: thread_id.to_string(),
            instruction: instruction.to_string(),
            capability_tag: tag.to_string(),
            status: "running".to_string(),
        };
        self.turns.insert(turn.id.clone(), turn.clone());
        Some(turn)
    }
    pub fn request_approval(&mut self, turn_id: &str, action: &str, target: &str) -> ApprovalRequest {
        let r = ApprovalRequest::new(turn_id, action, target);
        self.approvals.insert(r.id.clone(), r.clone());
        r
    }
    pub fn resolve_approval(&mut self, id: &str, state: ApprovalState) -> Option<&ApprovalRequest> {
        if let Some(a) = self.approvals.get_mut(id) {
            a.state = state;
            return self.approvals.get(id);
        }
        None
    }
    pub fn pending_approvals(&self) -> Vec<&ApprovalRequest> {
        self.approvals.values().filter(|a| a.state == ApprovalState::Pending).collect()
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn thread_turn_approval_flow() {
        let mut s = ThreadStore::default();
        let thr = s.create_thread(Some("demo".into()));
        let turn = s.start_turn(&thr.id, "合并表格", "xlsx_consolidation").unwrap();
        assert_eq!(turn.capability_tag, "xlsx_consolidation");
        let appr = s.request_approval(&turn.id, "execute", "consolidate_tables");
        assert_eq!(appr.state, ApprovalState::Pending);
        let resolved = s.resolve_approval(&appr.id, ApprovalState::Approved).unwrap();
        assert_eq!(resolved.state, ApprovalState::Approved);
        assert_eq!(s.pending_approvals().len(), 0);
    }
    #[test]
    fn list_threads() {
        let mut s = ThreadStore::default();
        s.create_thread(None);
        s.create_thread(None);
        assert_eq!(s.list_threads().len(), 2);
    }
}
