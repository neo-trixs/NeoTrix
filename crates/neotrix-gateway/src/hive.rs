//! Hive Coordination Protocol — inbox/outbox/blackboard 模式
//!
//! Agent 间通信基础设施: per-agent inbox + outbox + 共享 blackboard + event log。
//! 单提交者 git 风格：每个 agent 写自己的 outbox，GOD agent 仲裁路由。
//!
//! 设计启发: Munder Difflin hive system (blackboard + mailboxes + event log)

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

// ─── Time helper ────────────────────────────────────────────────────────────

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ─── Message ────────────────────────────────────────────────────────────────

/// Inter-agent message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveMessage {
    /// Unique message ID
    pub id: String,
    /// Sender agent ID
    pub from: String,
    /// Receiver agent ID (None = broadcast)
    pub to: Option<String>,
    /// Message type
    pub msg_type: MessageType,
    /// Content payload
    pub content: String,
    /// Priority (0 = normal, 1 = high, 2 = urgent)
    pub priority: u8,
    /// Timestamp (epoch secs)
    pub timestamp: u64,
    /// Optional correlation ID (for request/response threading)
    pub correlation_id: Option<String>,
    /// Optional expiration (epoch secs)
    pub expires_at: Option<u64>,
}

/// Message type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageType {
    /// Task assignment
    Task,
    /// Task result
    Result,
    /// Question / request for info
    Query,
    /// Answer to a query
    Response,
    /// Status update
    Status,
    /// Escalation to human
    Escalation,
    /// Broadcast announcement
    Broadcast,
}

// ─── Mailbox ────────────────────────────────────────────────────────────────

/// Per-agent mailbox (inbox + outbox)
pub struct AgentMailbox {
    /// Agent ID
    pub agent_id: String,
    /// Incoming messages
    pub inbox: VecDeque<HiveMessage>,
    /// Outgoing messages (sent but not yet consumed by router)
    pub outbox: VecDeque<HiveMessage>,
    /// Max inbox size (oldest messages dropped when exceeded)
    pub max_inbox_size: usize,
    /// Max outbox size
    pub max_outbox_size: usize,
    /// Agent capabilities (for capability-based routing)
    pub capabilities: Vec<String>,
    /// Last active timestamp (epoch secs)
    pub last_active: u64,
}

impl AgentMailbox {
    pub fn new(agent_id: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            inbox: VecDeque::new(),
            outbox: VecDeque::new(),
            max_inbox_size: 100,
            max_outbox_size: 50,
            capabilities: Vec::new(),
            last_active: now_secs(),
        }
    }

    /// Receive a message into inbox
    pub fn receive(&mut self, msg: HiveMessage) {
        if self.inbox.len() >= self.max_inbox_size {
            self.inbox.pop_front(); // drop oldest
        }
        self.inbox.push_back(msg);
        self.last_active = now_secs();
    }

    /// Send a message to outbox
    pub fn send(&mut self, msg: HiveMessage) {
        if self.outbox.len() >= self.max_outbox_size {
            self.outbox.pop_front(); // drop oldest
        }
        self.outbox.push_back(msg);
        self.last_active = now_secs();
    }

    /// Read next message from inbox (FIFO)
    pub fn read_next(&mut self) -> Option<HiveMessage> {
        self.inbox.pop_front()
    }

    /// Peek at next message without consuming
    pub fn peek_next(&self) -> Option<&HiveMessage> {
        self.inbox.front()
    }

    /// Drain outbox (for router to consume)
    pub fn drain_outbox(&mut self) -> Vec<HiveMessage> {
        self.outbox.drain(..).collect()
    }

    /// Inbox depth
    pub fn inbox_depth(&self) -> usize {
        self.inbox.len()
    }

    /// Outbox depth
    pub fn outbox_depth(&self) -> usize {
        self.outbox.len()
    }
}

// ─── Blackboard ─────────────────────────────────────────────────────────────

/// Shared blackboard — all agents can read/write key-value entries
pub struct Blackboard {
    entries: HashMap<String, BlackboardEntry>,
    /// Max entries before eviction (LRU by access time)
    pub max_entries: usize,
}

/// A single blackboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackboardEntry {
    pub key: String,
    pub value: String,
    pub author: String,
    pub written_at: u64,
    pub accessed_at: u64,
    pub access_count: u64,
    /// TTL in seconds (None = permanent)
    pub ttl: Option<u64>,
}

impl Blackboard {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            max_entries: 1000,
        }
    }

    /// Write an entry
    pub fn write(&mut self, key: &str, value: &str, author: &str) {
        let now = now_secs();
        if self.entries.len() >= self.max_entries && !self.entries.contains_key(key) {
            // Evict least recently accessed
            if let Some(oldest_key) = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.accessed_at)
                .map(|(k, _)| k.clone())
            {
                self.entries.remove(&oldest_key);
            }
        }
        let entry = self
            .entries
            .entry(key.to_string())
            .or_insert_with(|| BlackboardEntry {
                key: key.to_string(),
                value: value.to_string(),
                author: author.to_string(),
                written_at: now,
                accessed_at: now,
                access_count: 0,
                ttl: None,
            });
        entry.value = value.to_string();
        entry.author = author.to_string();
        entry.written_at = now;
        entry.accessed_at = now;
    }

    /// Read an entry
    pub fn read(&mut self, key: &str) -> Option<BlackboardEntry> {
        let now = now_secs();
        if let Some(entry) = self.entries.get_mut(key) {
            // Check TTL expiry
            if let Some(ttl) = entry.ttl {
                if now > entry.written_at + ttl {
                    self.entries.remove(key);
                    return None;
                }
            }
            entry.accessed_at = now;
            entry.access_count += 1;
            Some(entry.clone())
        } else {
            None
        }
    }

    /// List all keys
    pub fn keys(&self) -> Vec<&str> {
        self.entries.keys().map(|s| s.as_str()).collect()
    }

    /// List entries by prefix
    pub fn list_prefix(&self, prefix: &str) -> Vec<&BlackboardEntry> {
        self.entries
            .values()
            .filter(|e| e.key.starts_with(prefix))
            .collect()
    }

    /// Remove an entry
    pub fn remove(&mut self, key: &str) -> bool {
        self.entries.remove(key).is_some()
    }

    /// Entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for Blackboard {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Event Log ──────────────────────────────────────────────────────────────

/// Append-only event log for audit trail
pub struct EventLog {
    events: VecDeque<HiveEvent>,
    max_events: usize,
}

/// A single event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveEvent {
    pub id: String,
    pub event_type: EventType,
    pub agent_id: Option<String>,
    pub message_id: Option<String>,
    pub detail: String,
    pub timestamp: u64,
}

/// Event type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventType {
    MessageSent,
    MessageDelivered,
    MessageExpired,
    BlackboardWrite,
    BlackboardRead,
    AgentBusy,
    AgentAvailable,
    Escalation,
    CircuitBreakerTripped,
    HumanOverride,
}

impl EventLog {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            max_events: 10000,
        }
    }

    /// Append an event
    pub fn append(&mut self, event: HiveEvent) {
        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Get recent events
    pub fn recent(&self, n: usize) -> Vec<&HiveEvent> {
        self.events.iter().rev().take(n).collect()
    }

    /// Get events by type
    pub fn by_type(&self, event_type: &EventType) -> Vec<&HiveEvent> {
        self.events
            .iter()
            .filter(|e| e.event_type == *event_type)
            .collect()
    }

    /// Get events by agent
    pub fn by_agent(&self, agent_id: &str) -> Vec<&HiveEvent> {
        self.events
            .iter()
            .filter(|e| e.agent_id.as_deref() == Some(agent_id))
            .collect()
    }

    /// Event count
    pub fn len(&self) -> usize {
        self.events.len()
    }
}

impl Default for EventLog {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Hive Router ────────────────────────────────────────────────────────────

/// Central hive router — consumes outboxes, delivers to inboxes, logs events
pub struct HiveRouter {
    /// Per-agent mailboxes
    mailboxes: HashMap<String, AgentMailbox>,
    /// Shared blackboard
    pub blackboard: Blackboard,
    /// Event log
    pub event_log: EventLog,
}

impl HiveRouter {
    pub fn new() -> Self {
        Self {
            mailboxes: HashMap::new(),
            blackboard: Blackboard::new(),
            event_log: EventLog::new(),
        }
    }

    /// Ensure a mailbox exists for an agent
    pub fn ensure_mailbox(&mut self, agent_id: &str) {
        self.mailboxes
            .entry(agent_id.to_string())
            .or_insert_with(|| AgentMailbox::new(agent_id));
    }

    /// Send a message from one agent to another
    pub fn send(&mut self, msg: HiveMessage) {
        self.ensure_mailbox(&msg.from);

        if let Some(ref to) = msg.to {
            // Direct message
            self.ensure_mailbox(to);
            let msg_clone = msg.clone();
            if let Some(mailbox) = self.mailboxes.get_mut(to) {
                mailbox.receive(msg_clone);
            }
            self.event_log.append(HiveEvent {
                id: event_id(),
                event_type: EventType::MessageDelivered,
                agent_id: Some(to.clone()),
                message_id: Some(msg.id.clone()),
                detail: format!("{} → {}", msg.from, to),
                timestamp: now_secs(),
            });
        } else {
            // Broadcast to all except sender
            let recipients: Vec<String> = self
                .mailboxes
                .keys()
                .filter(|k| *k != &msg.from)
                .cloned()
                .collect();
            for recipient in &recipients {
                let msg_clone = msg.clone();
                if let Some(mailbox) = self.mailboxes.get_mut(recipient) {
                    mailbox.receive(msg_clone);
                }
            }
            self.event_log.append(HiveEvent {
                id: event_id(),
                event_type: EventType::MessageSent,
                agent_id: Some(msg.from.clone()),
                message_id: Some(msg.id.clone()),
                detail: format!(
                    "broadcast from {} to {} recipients",
                    msg.from,
                    recipients.len()
                ),
                timestamp: now_secs(),
            });
        }
    }

    /// Read next message for an agent
    pub fn read_next(&mut self, agent_id: &str) -> Option<HiveMessage> {
        self.mailboxes
            .get_mut(agent_id)
            .and_then(|m| m.read_next())
    }

    /// Get mailbox depth
    pub fn inbox_depth(&self, agent_id: &str) -> usize {
        self.mailboxes
            .get(agent_id)
            .map(|m| m.inbox_depth())
            .unwrap_or(0)
    }

    /// Get all agent IDs
    pub fn agent_ids(&self) -> Vec<&str> {
        self.mailboxes.keys().map(|s| s.as_str()).collect()
    }

    /// Drain all outboxes and route messages
    pub fn route_pending(&mut self) {
        // Collect all outbox messages first
        let mut pending: Vec<HiveMessage> = Vec::new();
        for mailbox in self.mailboxes.values_mut() {
            pending.extend(mailbox.drain_outbox());
        }
        // Deliver each
        for msg in pending {
            self.send(msg);
        }
    }

    /// Stats
    pub fn stats(&self) -> HiveStats {
        let total_inbox: usize = self.mailboxes.values().map(|m| m.inbox_depth()).sum();
        let total_outbox: usize = self.mailboxes.values().map(|m| m.outbox_depth()).sum();
        HiveStats {
            agents: self.mailboxes.len(),
            total_inbox,
            total_outbox,
            blackboard_entries: self.blackboard.len(),
            events_logged: self.event_log.len(),
        }
    }

    /// Set capabilities for an agent
    pub fn set_capabilities(&mut self, agent_id: &str, capabilities: Vec<String>) {
        self.ensure_mailbox(agent_id);
        if let Some(mailbox) = self.mailboxes.get_mut(agent_id) {
            mailbox.capabilities = capabilities;
        }
    }

    /// Delegate a task to a specific agent
    pub fn delegate_task(
        &mut self,
        from: &str,
        to: &str,
        task_content: &str,
        priority: u8,
    ) -> Result<String, String> {
        self.ensure_mailbox(from);
        self.ensure_mailbox(to);

        let msg_id = msg_id();
        let msg = HiveMessage {
            id: msg_id.clone(),
            from: from.to_string(),
            to: Some(to.to_string()),
            msg_type: MessageType::Task,
            content: task_content.to_string(),
            priority,
            timestamp: now_secs(),
            correlation_id: None,
            expires_at: None,
        };

        self.send(msg);

        self.event_log.append(HiveEvent {
            id: event_id(),
            event_type: EventType::MessageSent,
            agent_id: Some(from.to_string()),
            message_id: Some(msg_id.clone()),
            detail: format!("delegated task from {} to {}", from, to),
            timestamp: now_secs(),
        });

        Ok(msg_id)
    }

    /// Delegate a task to the best matching agent based on capabilities
    pub fn delegate_to_best(
        &mut self,
        from: &str,
        task_content: &str,
        required_capabilities: &[String],
    ) -> Result<(String, String), String> {
        // Find agents that have all required capabilities
        let candidates: Vec<String> = self
            .mailboxes
            .iter()
            .filter(|(id, mb)| {
                *id != from
                    && required_capabilities
                        .iter()
                        .all(|cap| mb.capabilities.contains(cap))
            })
            .map(|(id, _)| id.clone())
            .collect();

        if candidates.is_empty() {
            return Err(format!(
                "no agent found with capabilities: {:?}",
                required_capabilities
            ));
        }

        // Pick the candidate with smallest inbox (least busy)
        let best = candidates
            .iter()
            .min_by_key(|id| self.mailboxes[*id].inbox_depth())
            .unwrap()
            .clone();

        let msg_id = self.delegate_task(from, &best, task_content, 0)?;
        Ok((best, msg_id))
    }

    /// Collect a Result message from an agent's inbox
    pub fn collect_result(&mut self, agent_id: &str) -> Option<HiveMessage> {
        if let Some(mailbox) = self.mailboxes.get_mut(agent_id) {
            // Peek and find first Result message without consuming non-Result messages
            let pos = mailbox
                .inbox
                .iter()
                .position(|m| m.msg_type == MessageType::Result);
            if let Some(idx) = pos {
                return mailbox.inbox.remove(idx);
            }
        }
        None
    }

    /// Get status summary for all agents
    pub fn agent_status_summary(&self) -> Vec<AgentStatus> {
        self.mailboxes
            .values()
            .map(|mb| AgentStatus {
                agent_id: mb.agent_id.clone(),
                inbox_depth: mb.inbox_depth(),
                last_active: mb.last_active,
                capabilities: mb.capabilities.clone(),
            })
            .collect()
    }
}

impl Default for HiveRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Stats ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveStats {
    pub agents: usize,
    pub total_inbox: usize,
    pub total_outbox: usize,
    pub blackboard_entries: usize,
    pub events_logged: usize,
}

/// Agent status summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub agent_id: String,
    pub inbox_depth: usize,
    pub last_active: u64,
    pub capabilities: Vec<String>,
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn event_id() -> String {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("evt-{:016x}", t)
}

fn msg_id() -> String {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("msg-{:016x}", t)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mailbox_send_receive() {
        let mut mailbox = AgentMailbox::new("agent-1");
        let msg = HiveMessage {
            id: "m1".to_string(),
            from: "agent-2".to_string(),
            to: Some("agent-1".to_string()),
            msg_type: MessageType::Task,
            content: "do something".to_string(),
            priority: 0,
            timestamp: now_secs(),
            correlation_id: None,
            expires_at: None,
        };
        mailbox.receive(msg);
        assert_eq!(mailbox.inbox_depth(), 1);
        let received = mailbox.read_next().unwrap();
        assert_eq!(received.content, "do something");
        assert_eq!(mailbox.inbox_depth(), 0);
    }

    #[test]
    fn test_mailbox_overflow() {
        let mut mailbox = AgentMailbox::new("a1");
        mailbox.max_inbox_size = 3;
        for i in 0..5 {
            mailbox.receive(HiveMessage {
                id: format!("m{}", i),
                from: "x".to_string(),
                to: Some("a1".to_string()),
                msg_type: MessageType::Status,
                content: i.to_string(),
                priority: 0,
                timestamp: now_secs(),
                correlation_id: None,
                expires_at: None,
            });
        }
        assert_eq!(mailbox.inbox_depth(), 3); // oldest dropped
    }

    #[test]
    fn test_blackboard_write_read() {
        let mut bb = Blackboard::new();
        bb.write("key1", "value1", "agent-1");
        assert_eq!(bb.read("key1").unwrap().value, "value1");
        assert_eq!(bb.read("key1").unwrap().author, "agent-1");
        assert_eq!(bb.len(), 1);
    }

    #[test]
    fn test_blackboard_prefix_list() {
        let mut bb = Blackboard::new();
        bb.write("agent-1/status", "busy", "a1");
        bb.write("agent-1/cost", "0.5", "a1");
        bb.write("agent-2/status", "idle", "a2");
        let entries = bb.list_prefix("agent-1/");
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_event_log() {
        let mut log = EventLog::new();
        log.append(HiveEvent {
            id: "e1".to_string(),
            event_type: EventType::MessageSent,
            agent_id: Some("a1".to_string()),
            message_id: Some("m1".to_string()),
            detail: "test".to_string(),
            timestamp: now_secs(),
        });
        assert_eq!(log.len(), 1);
        assert_eq!(log.by_agent("a1").len(), 1);
        assert_eq!(log.by_agent("a2").len(), 0);
    }

    #[test]
    fn test_hive_router_send_deliver() {
        let mut router = HiveRouter::new();
        router.ensure_mailbox("a1");
        router.ensure_mailbox("a2");

        router.send(HiveMessage {
            id: "m1".to_string(),
            from: "a1".to_string(),
            to: Some("a2".to_string()),
            msg_type: MessageType::Task,
            content: "hello".to_string(),
            priority: 0,
            timestamp: now_secs(),
            correlation_id: None,
            expires_at: None,
        });

        assert_eq!(router.inbox_depth("a2"), 1);
        assert_eq!(router.inbox_depth("a1"), 0);
    }

    #[test]
    fn test_hive_router_broadcast() {
        let mut router = HiveRouter::new();
        router.ensure_mailbox("a1");
        router.ensure_mailbox("a2");
        router.ensure_mailbox("a3");

        router.send(HiveMessage {
            id: "m1".to_string(),
            from: "a1".to_string(),
            to: None, // broadcast
            msg_type: MessageType::Broadcast,
            content: "attention all".to_string(),
            priority: 1,
            timestamp: now_secs(),
            correlation_id: None,
            expires_at: None,
        });

        assert_eq!(router.inbox_depth("a1"), 0); // sender doesn't get it
        assert_eq!(router.inbox_depth("a2"), 1);
        assert_eq!(router.inbox_depth("a3"), 1);
    }

    #[test]
    fn test_hive_router_stats() {
        let mut router = HiveRouter::new();
        router.ensure_mailbox("a1");
        router.ensure_mailbox("a2");
        let stats = router.stats();
        assert_eq!(stats.agents, 2);
        assert_eq!(stats.blackboard_entries, 0);
    }

    #[test]
    fn test_delegate_task() {
        let mut router = HiveRouter::new();
        router.ensure_mailbox("coordinator");
        router.ensure_mailbox("worker");

        let result = router.delegate_task("coordinator", "worker", "analyze data", 1);
        assert!(result.is_ok());

        let msg_id = result.unwrap();
        // Worker should have 1 message in inbox
        assert_eq!(router.inbox_depth("worker"), 1);
        // Coordinator should have 0
        assert_eq!(router.inbox_depth("coordinator"), 0);

        // Verify the message content
        let msg = router.read_next("worker").unwrap();
        assert_eq!(msg.id, msg_id);
        assert_eq!(msg.from, "coordinator");
        assert_eq!(msg.msg_type, MessageType::Task);
        assert_eq!(msg.content, "analyze data");
        assert_eq!(msg.priority, 1);
    }

    #[test]
    fn test_collect_result() {
        let mut router = HiveRouter::new();
        router.ensure_mailbox("worker");
        router.ensure_mailbox("coordinator");

        // Worker sends a result back
        router.send(HiveMessage {
            id: "r1".to_string(),
            from: "worker".to_string(),
            to: Some("coordinator".to_string()),
            msg_type: MessageType::Result,
            content: "analysis complete".to_string(),
            priority: 0,
            timestamp: now_secs(),
            correlation_id: None,
            expires_at: None,
        });

        // Coordinator should be able to collect the result
        let result = router.collect_result("coordinator");
        assert!(result.is_some());
        let msg = result.unwrap();
        assert_eq!(msg.msg_type, MessageType::Result);
        assert_eq!(msg.content, "analysis complete");

        // No more results
        assert!(router.collect_result("coordinator").is_none());
    }

    #[test]
    fn test_agent_status_summary() {
        let mut router = HiveRouter::new();
        router.set_capabilities("a1", vec!["reasoning".to_string(), "code".to_string()]);
        router.set_capabilities("a2", vec!["search".to_string()]);

        // Send a message to a1 to change its inbox depth
        router.send(HiveMessage {
            id: "m1".to_string(),
            from: "a2".to_string(),
            to: Some("a1".to_string()),
            msg_type: MessageType::Task,
            content: "help".to_string(),
            priority: 0,
            timestamp: now_secs(),
            correlation_id: None,
            expires_at: None,
        });

        let summary = router.agent_status_summary();
        assert_eq!(summary.len(), 2);

        let a1_status = summary.iter().find(|s| s.agent_id == "a1").unwrap();
        assert_eq!(a1_status.inbox_depth, 1);
        assert_eq!(a1_status.capabilities, vec!["reasoning", "code"]);

        let a2_status = summary.iter().find(|s| s.agent_id == "a2").unwrap();
        assert_eq!(a2_status.inbox_depth, 0);
        assert_eq!(a2_status.capabilities, vec!["search"]);
    }
}
