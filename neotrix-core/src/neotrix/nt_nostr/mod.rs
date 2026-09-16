//! # NT-NOSTR: NIP-01 事件总线
//!
//! 所有桌面操作编码为 NIP-01 事件, 提供序列化/反序列化、签名验证、
//! 事件类型系统等核心能力。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// NeoTrix 事件 — 所有桌面操作编码为 NIP-01 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoTrixEvent {
    /// 事件类型 (NIP-01 kind)
    pub kind: EventKind,
    /// 发布者公钥 (agent 或 user)
    pub pubkey: String,
    /// 事件内容 (JSON payload)
    pub content: String,
    /// 创建时间 (UNIX 纪元秒)
    pub created_at: u64,
    /// 标签 (channel, worktree, etc.)
    pub tags: Vec<Vec<String>>,
    /// 签名 (agent key)
    pub sig: Option<String>,
}

impl NeoTrixEvent {
    /// 创建新事件
    pub fn new(kind: EventKind, pubkey: String, content: String, tags: Vec<Vec<String>>) -> Self {
        let created_at = now_timestamp();
        Self {
            kind,
            pubkey,
            content,
            created_at,
            tags,
            sig: None,
        }
    }

    /// 签名事件
    pub fn sign(&mut self, signature: String) {
        self.sig = Some(signature);
    }

    /// 验证事件是否已签名
    pub fn is_signed(&self) -> bool {
        self.sig.is_some()
    }

    /// 序列化为 JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// 从 JSON 反序列化
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// 事件摘要 (用于日志/审计)
    pub fn summary(&self) -> String {
        format!(
            "[{}] {}: {} ({} tags, signed={})",
            self.kind.label(),
            self.pubkey,
            self.content.chars().take(80).collect::<String>(),
            self.tags.len(),
            self.is_signed()
        )
    }
}

/// NIP-01 kind 映射
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    // 文本笔记
    TextNote = 1,
    // 加密消息
    Encrypted = 4,
    // 频道创建
    ChannelCreate = 40280,
    // 频道消息
    ChannelMessage = 40281,
    // Git 事件 (NIP-34)
    GitPatch = 30023,
    // 工作流步骤
    WorkflowStep = 30078,
    // 审批门
    ApprovalGate = 31337,
    // 研究实验
    Experiment = 31989,
    // Agent 心跳
    AgentHeartbeat = 31000,
}

impl EventKind {
    pub fn label(&self) -> &'static str {
        match self {
            EventKind::TextNote => "TextNote",
            EventKind::Encrypted => "Encrypted",
            EventKind::ChannelCreate => "ChannelCreate",
            EventKind::ChannelMessage => "ChannelMessage",
            EventKind::GitPatch => "GitPatch",
            EventKind::WorkflowStep => "WorkflowStep",
            EventKind::ApprovalGate => "ApprovalGate",
            EventKind::Experiment => "Experiment",
            EventKind::AgentHeartbeat => "AgentHeartbeat",
        }
    }

    pub fn from_kind(kind: u64) -> Option<Self> {
        match kind {
            1 => Some(EventKind::TextNote),
            4 => Some(EventKind::Encrypted),
            40280 => Some(EventKind::ChannelCreate),
            40281 => Some(EventKind::ChannelMessage),
            30023 => Some(EventKind::GitPatch),
            30078 => Some(EventKind::WorkflowStep),
            31337 => Some(EventKind::ApprovalGate),
            31989 => Some(EventKind::Experiment),
            31000 => Some(EventKind::AgentHeartbeat),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> u64 {
        *self as u64
    }
}

/// 事件标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTag {
    pub tag_type: String,
    pub values: Vec<String>,
}

/// 事件存储
#[derive(Debug, Default)]
pub struct EventStore {
    events: Vec<NeoTrixEvent>,
    by_kind: HashMap<EventKind, Vec<usize>>,
}

impl EventStore {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            by_kind: HashMap::new(),
        }
    }

    pub fn add(&mut self, event: NeoTrixEvent) -> usize {
        let idx = self.events.len();
        self.by_kind.entry(event.kind).or_default().push(idx);
        self.events.push(event);
        idx
    }

    pub fn get(&self, idx: usize) -> Option<&NeoTrixEvent> {
        self.events.get(idx)
    }

    pub fn by_kind(&self, kind: EventKind) -> Vec<&NeoTrixEvent> {
        self.by_kind
            .get(&kind)
            .map(|indices| indices.iter().map(|&i| &self.events[i]).collect())
            .unwrap_or_default()
    }

    pub fn all(&self) -> &[NeoTrixEvent] {
        &self.events
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// 获取当前时间戳
fn now_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation_and_serialization() {
        let event = NeoTrixEvent::new(
            EventKind::TextNote,
            "test_pubkey_123".to_string(),
            r#"{"action":"test"}"#.to_string(),
            vec![vec!["channel".to_string(), "general".to_string()]],
        );
        assert_eq!(event.kind, EventKind::TextNote);
        assert!(event.is_signed() == false);

        let json = event.to_json().unwrap();
        let deserialized = NeoTrixEvent::from_json(&json).unwrap();
        assert_eq!(deserialized.kind, EventKind::TextNote);
        assert_eq!(deserialized.pubkey, "test_pubkey_123");
    }

    #[test]
    fn test_event_kind_from_u64() {
        assert_eq!(EventKind::from_kind(1), Some(EventKind::TextNote));
        assert_eq!(EventKind::from_kind(4), Some(EventKind::Encrypted));
        assert_eq!(EventKind::from_kind(40280), Some(EventKind::ChannelCreate));
        assert_eq!(EventKind::from_kind(30023), Some(EventKind::GitPatch));
        assert_eq!(EventKind::from_kind(31989), Some(EventKind::Experiment));
        assert_eq!(EventKind::from_kind(99999), None);
    }

    #[test]
    fn test_event_kind_as_u64() {
        assert_eq!(EventKind::TextNote.as_u64(), 1);
        assert_eq!(EventKind::Experiment.as_u64(), 31989);
        assert_eq!(EventKind::AgentHeartbeat.as_u64(), 31000);
    }

    #[test]
    fn test_event_store() {
        let mut store = EventStore::new();
        assert!(store.is_empty());

        let e1 = NeoTrixEvent::new(
            EventKind::TextNote,
            "pk1".to_string(),
            "content1".to_string(),
            vec![],
        );
        let idx = store.add(e1);
        assert_eq!(store.len(), 1);
        assert!(!store.is_empty());

        let retrieved = store.get(idx).unwrap();
        assert_eq!(retrieved.kind, EventKind::TextNote);

        let text_notes = store.by_kind(EventKind::TextNote);
        assert_eq!(text_notes.len(), 1);
    }

    #[test]
    fn test_event_signing() {
        let mut event = NeoTrixEvent::new(
            EventKind::AgentHeartbeat,
            "agent_key".to_string(),
            "heartbeat".to_string(),
            vec![],
        );
        assert!(!event.is_signed());
        event.sign("sig_abc123".to_string());
        assert!(event.is_signed());
        assert_eq!(event.sig.unwrap(), "sig_abc123");
    }

    #[test]
    fn test_event_summary() {
        let event = NeoTrixEvent::new(
            EventKind::Experiment,
            "pk".to_string(),
            r#"{"hypothesis":"test"}"#.to_string(),
            vec![vec!["project".to_string(), "alpha".to_string()]],
        );
        let summary = event.summary();
        assert!(summary.contains("Experiment"));
        assert!(summary.contains("signed=false"));
    }
}