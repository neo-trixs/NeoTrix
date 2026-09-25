// ── NeoCodex agent interaction: side-chat + goals (split from agent.rs, behavior unchanged) ──

use super::NeoCodexAgent;
use super::super::wire::WireEvent;
use crate::l1_action::nt_io::nt_io_provider::types::{Message, Role};

impl NeoCodexAgent {
    /// Record a side-chat message (branched question that must NOT pollute
    /// the main session context). Persisted to the same wire stream but
    /// filtered out of resume_session / get_session_messages.
    pub(crate) fn _record_side_chat(&mut self, content: &str, role: &str) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.wire.record(WireEvent::SideChatMessage {
            content: content.trim().to_string(),
            timestamp: ts,
            role: role.to_string(),
        });
    }

    /// One-shot side-chat answer (P1-2). Unlike the main loop, this must NOT
    /// push into `self.context` / `self.state` — the branch question is
    /// isolated from the session's real conversation. We generate the reply
    /// on a throwaway message list and record both turns as side-chat events.
    pub async fn side_chat_ask(&mut self, content: &str) -> String {
        let provider = match self.provider.to_llm_provider() {
            Some(p) => p,
            None => return "[side chat] no provider configured".to_string(),
        };
        let system = "You are NeoCodex, an AI coding assistant. Answer the user's question concisely and precisely. Respond in markdown.";
        let messages = vec![
            Message::new(Role::System, system),
            Message::new(Role::User, content.trim()),
        ];
        let request = match self.build_request(messages) {
            Some(r) => r,
            None => return "[side chat] provider unavailable".to_string(),
        };
        let mut rx = match provider.stream_complete(&request).await {
            Ok(rx) => rx,
            Err(e) => return format!("[side chat] {}", e),
        };
        let mut answer = String::new();
        while let Some(chunk) = rx.recv().await {
            match chunk {
                Ok(resp) => answer.push_str(&resp.content),
                Err(e) => return format!("[side chat] {}", e),
            }
        }
        if answer.trim().is_empty() {
            "[side chat] empty response".to_string()
        } else {
            answer
        }
    }

    /// Add a goal to the queue (from Kimi Code /goal system)
    pub fn add_goal(&mut self, description: &str, max_iters: u64) {
        self.goals.add(description, max_iters);
        let id = self
            .goals
            .goals
            .back()
            .map(|g| g.id.clone())
            .unwrap_or_default();
        self.state.goal_active = true;
        self.wire.record(WireEvent::GoalUpdate {
            id,
            state: "active".into(),
            description: description.into(),
        });
    }

    /// Check if a goal is complete and advance the queue.
    ///
    /// Also increments the active goal's iteration counter each turn and
    /// resets `goal_active` once the queue drains, so the goal loop can
    /// actually make progress (previously `check_goals` was never called,
    /// `iterations` never incremented, and `goal_active` never reset).
    /// When no goal is active but the queue is non-empty, promotes the head
    /// of the queue (add_goal queues but never promotes).
    pub fn check_goals(&mut self) -> Option<String> {
        if self.goals.active.is_none() && !self.goals.goals.is_empty() {
            self.goals.next();
        }
        if let Some(ref mut goal) = self.goals.active {
            goal.iterations = goal.iterations.saturating_add(1);
            if goal.iterations >= goal.max_iterations {
                self.wire.record(WireEvent::GoalUpdate {
                    id: goal.id.clone(),
                    state: "completed".into(),
                    description: goal.description.clone(),
                });
                let next = self.goals.next().map(|g| g.description);
                if self.goals.active.is_none() && self.goals.goals.is_empty() {
                    self.state.goal_active = false;
                }
                return next;
            }
        }
        None
    }
}
