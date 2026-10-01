//! HiveAgentLoop — wraps AgentLoop with Hive coordination
//!
//! Decorates the core AgentLoop with inbox checking, outbox routing,
//! and blackboard access. Non-invasive: core AgentLoop unchanged.
//!
//! Flow per turn:
//!   1. Check inbox for messages from other agents → inject as context
//!   2. Run normal AgentLoop::turn()
//!   3. Collect tool invocations → post results to outbox/blackboard
//!   4. Route pending outbox messages via HiveRouter

use std::sync::{Arc, Mutex};

use super::nt_io_agent_loop::{AgentLoop, ToolInvocation};
use super::nt_io_provider::types::LlmError;
use neotrix_multi_agent::hive::{EventType, HiveEvent, HiveMessage, HiveRouter, MessageType};

/// Wraps AgentLoop with Hive coordination
pub struct HiveAgentLoop {
    /// Inner agent loop
    inner: AgentLoop,
    /// Agent ID for this loop
    agent_id: String,
    /// Shared Hive router (Arc + Mutex for cross-agent access)
    router: Arc<Mutex<HiveRouter>>,
}

/// Result of a coordinated turn
#[derive(Debug, Clone)]
pub struct CoordinatedTurnResult {
    /// The actual response from the inner agent loop
    pub response: String,
    /// Messages received from inbox before this turn
    pub inbox_messages: Vec<HiveMessage>,
    /// Messages sent to outbox during this turn
    pub outbox_messages: Vec<HiveMessage>,
    /// Blackboard entries read
    pub bb_reads: Vec<(String, String)>,
    /// Blackboard entries written
    pub bb_writes: Vec<(String, String)>,
    /// Events logged during this turn
    pub events: usize,
}

impl HiveAgentLoop {
    /// Create a new coordinated agent loop
    pub fn new(inner: AgentLoop, agent_id: &str, router: Arc<Mutex<HiveRouter>>) -> Self {
        // Ensure mailbox exists
        router.lock().unwrap().ensure_mailbox(agent_id);
        Self {
            inner,
            agent_id: agent_id.to_string(),
            router,
        }
    }

    /// Get the agent ID
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    /// Get a reference to the inner AgentLoop
    pub fn inner(&self) -> &AgentLoop {
        &self.inner
    }

    /// Get a mutable reference to the inner AgentLoop
    pub fn inner_mut(&mut self) -> &mut AgentLoop {
        &mut self.inner
    }

    /// Run a coordinated turn:
    /// 1. Drain inbox → build context prefix
    /// 2. Run inner turn
    /// 3. Collect results → post to outbox/blackboard
    pub async fn coordinated_turn(
        &mut self,
        user_input: &str,
    ) -> Result<CoordinatedTurnResult, LlmError> {
        // 1. Drain inbox
        let mut inbox_messages = Vec::new();
        {
            let mut router = self.router.lock().unwrap();
            while let Some(msg) = router.read_next(&self.agent_id) {
                inbox_messages.push(msg);
            }
        }

        // 2. Build enriched input with inbox context
        let enriched_input = if inbox_messages.is_empty() {
            user_input.to_string()
        } else {
            let mut context = String::new();
            for msg in &inbox_messages {
                let sender = &msg.from;
                let content = &msg.content;
                match msg.msg_type {
                    MessageType::Task => {
                        context.push_str(&format!("[来自 {} 的任务] {}\n", sender, content));
                    }
                    MessageType::Query => {
                        context.push_str(&format!("[来自 {} 的问题] {}\n", sender, content));
                    }
                    MessageType::Status => {
                        context.push_str(&format!("[来自 {} 的状态更新] {}\n", sender, content));
                    }
                    MessageType::Escalation => {
                        context.push_str(&format!("[来自 {} 的升级请求] {}\n", sender, content));
                    }
                    _ => {
                        context.push_str(&format!("[来自 {}] {}\n", sender, content));
                    }
                }
            }
            format!("{}\n---\n{}", context, user_input)
        };

        // 3. Run inner turn
        let response = self.inner.turn(&enriched_input).await?;

        // 4. Collect tool invocations
        let recent_tools: Vec<ToolInvocation> = self
            .inner
            .tool_log
            .iter()
            .rev()
            .take(5)
            .cloned()
            .collect();

        // 5. Post results to outbox/blackboard
        let outbox_messages = Vec::new();
        let bb_reads = Vec::new();
        let mut bb_writes = Vec::new();
        let mut events = 0;

        {
            let mut router = self.router.lock().unwrap();

            // Write summary to blackboard
            let summary = if recent_tools.is_empty() {
                format!("完成: {}", truncate(&response, 200))
            } else {
                let tool_names: Vec<&str> = recent_tools
                    .iter()
                    .map(|t| t.name.as_str())
                    .collect();
                format!(
                    "完成: {} (工具: {})",
                    truncate(&response, 150),
                    tool_names.join(", ")
                )
            };
            router.blackboard.write(
                &format!("{}/last_output", self.agent_id),
                &summary,
                &self.agent_id,
            );
            bb_writes.push((
                format!("{}/last_output", self.agent_id),
                summary,
            ));

            // Log completion event
            router.event_log.append(HiveEvent {
                id: format!("turn-{}", now_secs()),
                event_type: EventType::MessageSent,
                agent_id: Some(self.agent_id.clone()),
                message_id: None,
                detail: format!(
                    "turn completed, {} tools, {} inbox msgs",
                    recent_tools.len(),
                    inbox_messages.len()
                ),
                timestamp: now_secs(),
            });
            events += 1;
        }

        Ok(CoordinatedTurnResult {
            response,
            inbox_messages,
            outbox_messages,
            bb_reads,
            bb_writes,
            events,
        })
    }

    /// Send a message to another agent
    pub fn send_to(
        &self,
        to: &str,
        msg_type: MessageType,
        content: &str,
    ) {
        let msg = HiveMessage {
            id: format!("msg-{}", now_secs()),
            from: self.agent_id.clone(),
            to: Some(to.to_string()),
            msg_type,
            content: content.to_string(),
            priority: 0,
            timestamp: now_secs(),
            correlation_id: None,
            expires_at: None,
        };
        self.router.lock().unwrap().send(msg);
    }

    /// Broadcast a message to all agents
    pub fn broadcast(&self, msg_type: MessageType, content: &str) {
        let msg = HiveMessage {
            id: format!("bc-{}", now_secs()),
            from: self.agent_id.clone(),
            to: None,
            msg_type,
            content: content.to_string(),
            priority: 1,
            timestamp: now_secs(),
            correlation_id: None,
            expires_at: None,
        };
        self.router.lock().unwrap().send(msg);
    }

    /// Read a blackboard entry
    pub fn bb_read(&self, key: &str) -> Option<String> {
        self.router
            .lock()
            .unwrap()
            .blackboard
            .read(key)
            .map(|e| e.value)
    }

    /// Write a blackboard entry
    pub fn bb_write(&self, key: &str, value: &str) {
        self.router
            .lock()
            .unwrap()
            .blackboard
            .write(key, value, &self.agent_id);
    }

    /// Get inbox depth
    pub fn inbox_depth(&self) -> usize {
        self.router.lock().unwrap().inbox_depth(&self.agent_id)
    }

    /// Get router stats
    pub fn router_stats(&self) -> neotrix_multi_agent::hive::HiveStats {
        self.router.lock().unwrap().stats()
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    // 2026-09-30 副本漂移审计（scripts/ops/nt_fn_drift.py，UNIT-DIVERGENCE）修正：
    // 原来是 `&s[..max]` —— **按字节切**，而 `max` 落在非字符边界时
    // `str` 切片索引**直接 panic**。调用点是 `truncate(&response, 200)`，
    // response 是模型输出；一个汉字 3 字节，200 不是 3 的倍数
    // ⇒ 中文回复有约 2/3 概率在此 panic。全仓 8 份 truncate 副本里
    // `nt_memory_integration::truncate_chars` 与 `panoramic::truncate` 本来就是
    // 边界安全的，只有这份是字节切。
    let mut end = max.min(s.len());
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

use crate::l0_substrate::nt_core_time::now_secs;

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use neotrix_multi_agent::hive::{HiveMessage, MessageType};

    // Note: Full integration tests require LlmProvider mock
    // These tests verify the wrapper construction and message helpers

    #[test]
    fn test_send_to() {
        let router = Arc::new(Mutex::new(HiveRouter::new()));
        // We can't easily create a real AgentLoop without a backend,
        // so test the router directly
        {
            let mut r = router.lock().unwrap();
            r.ensure_mailbox("agent-a");
            r.ensure_mailbox("agent-b");
        }

        let msg = HiveMessage {
            id: "test-1".to_string(),
            from: "agent-a".to_string(),
            to: Some("agent-b".to_string()),
            msg_type: MessageType::Task,
            content: "do something".to_string(),
            priority: 0,
            timestamp: 0,
            correlation_id: None,
            expires_at: None,
        };
        router.lock().unwrap().send(msg);
        assert_eq!(router.lock().unwrap().inbox_depth("agent-b"), 1);
    }

    #[test]
    fn test_blackboard_roundtrip() {
        let router = Arc::new(Mutex::new(HiveRouter::new()));
        {
            let mut r = router.lock().unwrap();
            r.blackboard.write("agent-1/status", "busy", "agent-1");
        }
        let val = router.lock().unwrap().blackboard.read("agent-1/status").map(|e| e.value);
        assert_eq!(val.as_deref(), Some("busy"));
    }

    /// Regression (2026-09-30, 副本漂移审计): `&s[..max]` 按字节切，
    /// max 落在非字符边界时 `str` 索引 panic。调用点是 truncate(&response, 200)，
    /// 而 200 不是 3 的倍数 ⇒ 中文模型回复曾有约 2/3 概率在此 panic。
    #[test]
    fn truncate_never_panics_on_cjk() {
        assert_eq!(truncate("中文回复", 200), "中文回复");
        // 3 字节汉字 + 切点 5：字节 5 是「回」的首字节，非边界
        let out = truncate("中文回复", 5);
        assert!(out.len() <= 5, "must not exceed the byte budget");
        assert!(out.chars().count() >= 1);
        assert_eq!(truncate("abc", 1), "a");
        assert_eq!(truncate("abc", 0), "");
    }
}

