use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static NEXT_MESSAGE_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_INSTANCE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId {
    pub name: String,
    pub version: String,
    pub instance_id: u64,
}

impl AgentId {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            instance_id: 0,
        }
    }

    pub fn with_random_instance(name: &str, version: &str) -> Self {
        let id = NEXT_INSTANCE_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            name: name.to_string(),
            version: version.to_string(),
            instance_id: id,
        }
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/v{}#{}", self.name, self.version, self.instance_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentRole {
    Coordinator,
    Specialist,
    Verifier,
    Researcher,
    Critic,
    Synthesizer,
}

impl AgentRole {
    pub fn name(&self) -> &'static str {
        match self {
            AgentRole::Coordinator => "Coordinator",
            AgentRole::Specialist => "Specialist",
            AgentRole::Verifier => "Verifier",
            AgentRole::Researcher => "Researcher",
            AgentRole::Critic => "Critic",
            AgentRole::Synthesizer => "Synthesizer",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}

impl MessagePriority {
    pub fn rank(&self) -> u8 {
        match self {
            MessagePriority::Low => 0,
            MessagePriority::Normal => 1,
            MessagePriority::High => 2,
            MessagePriority::Critical => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentMessage {
    pub id: u64,
    pub sender: AgentId,
    pub recipients: Vec<AgentId>,
    pub conversation_id: u64,
    pub priority: MessagePriority,
    pub content: MessageContent,
    pub timestamp: Instant,
    pub ttl: Duration,
    pub reply_to: Option<u64>,
}

impl AgentMessage {
    pub fn new(
        sender: AgentId,
        recipients: Vec<AgentId>,
        content: MessageContent,
        priority: MessagePriority,
        ttl: Duration,
    ) -> Self {
        let id = NEXT_MESSAGE_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            id,
            sender,
            recipients,
            conversation_id: 0,
            priority,
            content,
            timestamp: Instant::now(),
            ttl,
            reply_to: None,
        }
    }

    pub fn expired(&self) -> bool {
        self.timestamp.elapsed() >= self.ttl
    }

    pub fn is_broadcast(&self) -> bool {
        self.recipients.is_empty()
    }

    pub fn is_intended_for(&self, agent: &AgentId) -> bool {
        self.recipients.is_empty() || self.recipients.iter().any(|r| r == agent)
    }

    pub fn reply_to_id(mut self, msg_id: u64) -> Self {
        self.reply_to = Some(msg_id);
        self
    }

    pub fn with_conversation(mut self, conversation_id: u64) -> Self {
        self.conversation_id = conversation_id;
        self
    }
}

#[derive(Debug, Clone)]
pub enum MessageContent {
    TaskRequest {
        description: String,
        domain: String,
        constraints: Vec<String>,
    },
    TaskResult {
        task_id: u64,
        output: String,
        confidence: f64,
        artifacts: Vec<String>,
    },
    Query {
        question: String,
        context: Vec<String>,
    },
    Response {
        answer: String,
        sources: Vec<String>,
    },
    Coordination {
        action: CoordinationAction,
        rationale: String,
    },
    StatusUpdate {
        status: AgentStatus,
        progress: f64,
        message: String,
    },
    Error {
        code: u32,
        description: String,
        recoverable: bool,
    },
}

impl MessageContent {
    pub fn kind(&self) -> &'static str {
        match self {
            MessageContent::TaskRequest { .. } => "TaskRequest",
            MessageContent::TaskResult { .. } => "TaskResult",
            MessageContent::Query { .. } => "Query",
            MessageContent::Response { .. } => "Response",
            MessageContent::Coordination { .. } => "Coordination",
            MessageContent::StatusUpdate { .. } => "StatusUpdate",
            MessageContent::Error { .. } => "Error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AgentStatus {
    Idle,
    Busy,
    Blocked,
    Error,
    Offline,
}

impl AgentStatus {
    pub fn name(&self) -> &'static str {
        match self {
            AgentStatus::Idle => "Idle",
            AgentStatus::Busy => "Busy",
            AgentStatus::Blocked => "Blocked",
            AgentStatus::Error => "Error",
            AgentStatus::Offline => "Offline",
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(self, AgentStatus::Idle)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinationAction {
    Delegate,
    RequestHelp,
    Merge,
    Escalate,
    Reassign,
    Cancel,
}

impl CoordinationAction {
    pub fn name(&self) -> &'static str {
        match self {
            CoordinationAction::Delegate => "Delegate",
            CoordinationAction::RequestHelp => "RequestHelp",
            CoordinationAction::Merge => "Merge",
            CoordinationAction::Escalate => "Escalate",
            CoordinationAction::Reassign => "Reassign",
            CoordinationAction::Cancel => "Cancel",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::Hasher;

    #[test]
    fn test_agent_id_new() {
        let id = AgentId::new("alpha", "1.0.0");
        assert_eq!(id.name, "alpha");
        assert_eq!(id.version, "1.0.0");
        assert_eq!(id.instance_id, 0);
    }

    #[test]
    fn test_agent_id_with_random_instance() {
        let a = AgentId::with_random_instance("bot", "2.0");
        let b = AgentId::with_random_instance("bot", "2.0");
        assert_eq!(a.name, b.name);
        assert_eq!(a.version, b.version);
        assert_ne!(a.instance_id, b.instance_id);
    }

    #[test]
    fn test_agent_id_display() {
        let id = AgentId::new("test", "0.1");
        let s = format!("{}", id);
        assert!(s.contains("test"));
        assert!(s.contains("v0.1"));
    }

    #[test]
    fn test_agent_id_hash_eq() {
        let a = AgentId::new("x", "1");
        let b = AgentId::new("x", "1");
        assert_eq!(a, b);

        let mut hasher_a = std::collections::hash_map::DefaultHasher::new();
        let mut hasher_b = std::collections::hash_map::DefaultHasher::new();
        a.hash(&mut hasher_a);
        b.hash(&mut hasher_b);
        assert_eq!(hasher_a.finish(), hasher_b.finish());
    }

    #[test]
    fn test_agent_role_name() {
        assert_eq!(AgentRole::Coordinator.name(), "Coordinator");
        assert_eq!(AgentRole::Specialist.name(), "Specialist");
        assert_eq!(AgentRole::Verifier.name(), "Verifier");
        assert_eq!(AgentRole::Researcher.name(), "Researcher");
        assert_eq!(AgentRole::Critic.name(), "Critic");
        assert_eq!(AgentRole::Synthesizer.name(), "Synthesizer");
    }

    #[test]
    fn test_message_priority_ordering() {
        assert!(MessagePriority::Low < MessagePriority::Normal);
        assert!(MessagePriority::Normal < MessagePriority::High);
        assert!(MessagePriority::High < MessagePriority::Critical);
        assert_eq!(MessagePriority::Low.rank(), 0);
        assert_eq!(MessagePriority::Critical.rank(), 3);
    }

    #[test]
    fn test_agent_message_new() {
        let sender = AgentId::new("alice", "1.0");
        let recipient = AgentId::new("bob", "1.0");
        let content = MessageContent::Query {
            question: "hello".into(),
            context: vec![],
        };
        let msg = AgentMessage::new(
            sender.clone(),
            vec![recipient],
            content,
            MessagePriority::Normal,
            Duration::from_secs(60),
        );
        assert_eq!(msg.sender, sender);
        assert!(!msg.expired());
        assert!(!msg.is_broadcast());
        assert!(msg.id > 0);
        assert!(msg.timestamp.elapsed().as_secs() < 2);
    }

    #[test]
    fn test_broadcast_detection() {
        let sender = AgentId::new("a", "1.0");
        let msg = AgentMessage::new(
            sender,
            vec![],
            MessageContent::StatusUpdate {
                status: AgentStatus::Idle,
                progress: 0.0,
                message: "ready".into(),
            },
            MessagePriority::Low,
            Duration::from_secs(10),
        );
        assert!(msg.is_broadcast());
    }

    #[test]
    fn test_is_intended_for() {
        let alice = AgentId::new("alice", "1.0");
        let bob = AgentId::new("bob", "1.0");
        let charlie = AgentId::new("charlie", "1.0");

        let msg = AgentMessage::new(
            alice.clone(),
            vec![bob.clone()],
            MessageContent::Query {
                question: "?".into(),
                context: vec![],
            },
            MessagePriority::Normal,
            Duration::from_secs(30),
        );
        assert!(msg.is_intended_for(&bob));
        assert!(!msg.is_intended_for(&charlie));
    }

    #[test]
    fn test_expired_message() {
        let sender = AgentId::new("a", "1.0");
        let mut msg = AgentMessage::new(
            sender.clone(),
            vec![],
            MessageContent::Response {
                answer: "ok".into(),
                sources: vec![],
            },
            MessagePriority::Normal,
            Duration::from_nanos(1),
        );
        // Force expiry by setting timestamp far in past
        msg.timestamp = Instant::now() - Duration::from_secs(10);
        assert!(msg.expired());
    }

    #[test]
    fn test_message_fluent_builders() {
        let sender = AgentId::new("a", "1.0");
        let msg = AgentMessage::new(
            sender,
            vec![],
            MessageContent::TaskRequest {
                description: "work".into(),
                domain: "test".into(),
                constraints: vec![],
            },
            MessagePriority::High,
            Duration::from_secs(30),
        )
        .reply_to_id(42)
        .with_conversation(7);

        assert_eq!(msg.reply_to, Some(42));
        assert_eq!(msg.conversation_id, 7);
    }

    #[test]
    fn test_message_content_kind() {
        let c = MessageContent::TaskRequest {
            description: "".into(),
            domain: "".into(),
            constraints: vec![],
        };
        assert_eq!(c.kind(), "TaskRequest");

        let e = MessageContent::Error {
            code: 500,
            description: "fail".into(),
            recoverable: false,
        };
        assert_eq!(e.kind(), "Error");
    }

    #[test]
    fn test_agent_status() {
        assert!(AgentStatus::Idle.is_available());
        assert!(!AgentStatus::Busy.is_available());
        assert!(!AgentStatus::Blocked.is_available());
        assert!(!AgentStatus::Error.is_available());
        assert!(!AgentStatus::Offline.is_available());
        assert_eq!(AgentStatus::Busy.name(), "Busy");
        assert_eq!(AgentStatus::Offline.name(), "Offline");
    }

    #[test]
    fn test_coordination_action_name() {
        assert_eq!(CoordinationAction::Delegate.name(), "Delegate");
        assert_eq!(CoordinationAction::Cancel.name(), "Cancel");
    }

    #[test]
    fn test_message_id_increment() {
        let _id1 = NEXT_MESSAGE_ID.fetch_add(0, Ordering::Relaxed);
        let sender = AgentId::new("t", "1");
        let m1 = AgentMessage::new(
            sender.clone(),
            vec![],
            MessageContent::Query {
                question: "q1".into(),
                context: vec![],
            },
            MessagePriority::Normal,
            Duration::from_secs(1),
        );
        let m2 = AgentMessage::new(
            sender,
            vec![],
            MessageContent::Query {
                question: "q2".into(),
                context: vec![],
            },
            MessagePriority::Normal,
            Duration::from_secs(1),
        );
        assert!(m2.id > m1.id);
        // Reset for other tests
        // (we can't really reset atomic statics without a function)
    }
}
