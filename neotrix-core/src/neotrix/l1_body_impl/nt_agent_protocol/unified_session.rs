//! 统一会话模型 (G18, novu 吸收) — agent↔渠道统一会话抽象。
//!
//! novu 是事件驱动通知平台, 核心抽象为 workflow / subscriber / event / trigger:
//! 入站消息经渠道归一化为统一事件, 路由到目标 subscriber, 出站按渠道模板
//! 分发, 高频同类事件 digest 合并。本模块将同一模式用于 agent 会话:
//!
//!   - `SessionMessage`: 入站/出站统一消息载体 (渠道无关)。
//!   - `UnifiedSession`: 单个 agent↔渠道会话, 持消息日志 + 状态机。
//!   - `SessionDigest`: digest 合并 — 同主题高频消息按窗口聚合成摘要, 防下游淹没。
//!   - `SessionRouter`: 入站归一 → 按 capability 路由 → 出站分发 (digest 门)。

use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};

/// 会话渠道类型 (novu: provider/channel 抽象)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionChannel {
    /// agent 内部 (TCP/UDP agent 协议)
    Agent,
    /// web/HTTP 入站
    Web,
    /// 自主行动 (nt_act)
    Autonomy,
    /// 其他/自定义
    Other,
}

/// 统一会话消息 — 入站与出站共用载体 (渠道无关, novu event 抽象)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub session_id: String,
    pub agent_id: String,
    pub channel: SessionChannel,
    pub kind: String,
    pub content: String,
    pub topic: String,
    pub ts: i64,
}

impl SessionMessage {
    pub fn new(session_id: impl Into<String>, agent_id: impl Into<String>, kind: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            agent_id: agent_id.into(),
            channel: SessionChannel::Agent,
            kind: kind.into(),
            content: content.into(),
            topic: "general".to_string(),
            ts: crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::now(),
        }
    }

    pub fn with_channel(mut self, c: SessionChannel) -> Self {
        self.channel = c;
        self
    }

    pub fn with_topic(mut self, t: impl Into<String>) -> Self {
        self.topic = t.into();
        self
    }
}

/// 统一会话 — 状态机 + 消息日志 (novu subscriber/session 抽象)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSession {
    pub session_id: String,
    pub agent_id: String,
    pub state: SessionState,
    pub messages: VecDeque<SessionMessage>,
    /// 保留上限 (防无限增长, open-code-review 预算纪律)
    pub max_messages: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    Active,
    Paused,
    Closed,
}

impl UnifiedSession {
    pub fn new(session_id: impl Into<String>, agent_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            agent_id: agent_id.into(),
            state: SessionState::Active,
            messages: VecDeque::new(),
            max_messages: 256,
        }
    }

    /// 追加入站消息; 超上限弹出最旧 (滑动窗口)。
    pub fn ingest(&mut self, msg: SessionMessage) {
        if self.state != SessionState::Active {
            return;
        }
        self.messages.push_back(msg);
        while self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }
    }

    pub fn pause(&mut self) {
        self.state = SessionState::Paused;
    }

    pub fn close(&mut self) {
        self.state = SessionState::Closed;
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

/// digest 合并窗口 (novu digest): 同 session+topic 的消息在窗口内聚合。
#[derive(Debug, Clone, Default)]
pub struct SessionDigest {
    /// 窗口秒数
    pub window_secs: i64,
    /// 聚合桶: (session_id, topic) → 桶内容
    buckets: HashMap<(String, String), DigestBucket>,
}

#[derive(Debug, Clone)]
struct DigestBucket {
    first_ts: i64,
    last_ts: i64,
    count: usize,
    last_content: String,
}

impl SessionDigest {
    pub fn new(window_secs: i64) -> Self {
        Self {
            window_secs,
            buckets: HashMap::new(),
        }
    }

    /// 尝试消化一条消息: 命中窗口内同桶则合并 (返回 None), 否则开启新桶 (返回 Some)。
    /// 陈旧桶 (超出窗口) 由 `flush_stale` 清出。
    pub fn try_digest(&mut self, msg: &SessionMessage, now: i64) -> Option<SessionMessage> {
        let key = (msg.session_id.clone(), msg.topic.clone());
        if let Some(bucket) = self.buckets.get_mut(&key) {
            if now - bucket.last_ts <= self.window_secs {
                bucket.last_ts = now;
                bucket.count += 1;
                bucket.last_content = msg.content.clone();
                return None;
            }
            self.flush(&key, now);
        }
        self.buckets.insert(
            key.clone(),
            DigestBucket {
                first_ts: now,
                last_ts: now,
                count: 1,
                last_content: msg.content.clone(),
            },
        );
        Some(msg.clone())
    }

    /// 清出单个桶并生成 digest 摘要消息。
    fn flush(&mut self, key: &(String, String), now: i64) {
        if let Some(bucket) = self.buckets.remove(key) {
            let (session_id, topic) = key.clone();
            let digest_msg = SessionMessage::new(
                session_id.clone(),
                "digest".to_string(),
                "digest",
                format!(
                    "[{}] {} msgs ({}) → {}",
                    topic,
                    bucket.count,
                    bucket.last_ts - bucket.first_ts,
                    bucket.last_content
                ),
            )
            .with_topic(format!("{}#digest", topic));
            self.buckets.insert(
                (session_id, format!("{}#digest", topic)),
                DigestBucket {
                    first_ts: now,
                    last_ts: now,
                    count: 1,
                    last_content: digest_msg.content.clone(),
                },
            );
        }
    }

    /// 清出所有超窗桶; 返回新生成的 digest 消息。
    pub fn flush_stale(&mut self, now: i64) -> Vec<SessionMessage> {
        let keys: Vec<(String, String)> = self
            .buckets
            .keys()
            .filter(|k| now - self.buckets.get(*k).map(|b| b.last_ts).unwrap_or(now) > self.window_secs)
            .cloned()
            .collect();
        let mut out = Vec::new();
        for key in keys {
            self.flush(&key, now);
        }
        out
    }

    pub fn bucket_count(&self) -> usize {
        self.buckets.len()
    }
}

/// 统一会话路由器 (novu workflow 抽象): 入站归一 → capability 路由 → 出站分发。
#[derive(Debug, Default)]
pub struct SessionRouter {
    sessions: HashMap<String, UnifiedSession>,
    digest: SessionDigest,
}

impl SessionRouter {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            digest: SessionDigest::new(60),
        }
    }

    /// 入站归一: 任意渠道消息 → 归入对应 session (无则创建), 经 digest 门返回
    /// 可出站消息。返回 (入站归一计数, digest 合并数, 出站消息列表)。
    pub fn route(&mut self, msg: SessionMessage, capability: &str) -> (usize, usize, Vec<SessionMessage>) {
        let session = self
            .sessions
            .entry(msg.session_id.clone())
            .or_insert_with(|| UnifiedSession::new(msg.session_id.clone(), msg.agent_id.clone()));
        if !capability.is_empty() && session.agent_id != msg.agent_id {
            session.agent_id = msg.agent_id.clone();
        }
        let before = session.message_count();
        session.ingest(msg.clone());
        let ingested = session.message_count().saturating_sub(before);
        let now = crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::now();
        let mut out = Vec::new();
        if let Some(emitted) = self.digest.try_digest(&msg, now) {
            out.push(emitted);
        }
        let digested = if out.is_empty() { 1 } else { 0 };
        (ingested, digested, out)
    }

    pub fn session(&self, id: &str) -> Option<&UnifiedSession> {
        self.sessions.get(id)
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn flush_digests(&mut self) -> Vec<SessionMessage> {
        let now = crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::now();
        self.digest.flush_stale(now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(now: i64) -> SessionMessage {
        SessionMessage::new("s1", "agent-a", "task", format!("msg {}", now)).with_topic("t1")
    }

    #[test]
    fn test_unified_session_ingest_and_cap() {
        let mut s = UnifiedSession::new("s1", "a1");
        s.max_messages = 3;
        for i in 0..5 {
            s.ingest(SessionMessage::new("s1", "a1", "m", format!("{}", i)));
        }
        assert_eq!(s.message_count(), 3, "sliding window cap");
    }

    #[test]
    fn test_session_state_gating() {
        let mut s = UnifiedSession::new("s1", "a1");
        s.pause();
        s.ingest(SessionMessage::new("s1", "a1", "m", "x"));
        assert_eq!(s.message_count(), 0, "paused session must not ingest");
        s.close();
        assert_eq!(s.state, SessionState::Closed);
    }

    #[test]
    fn test_digest_merges_same_topic_in_window() {
        let mut d = SessionDigest::new(60);
        let m1 = t(1000);
        assert!(d.try_digest(&m1, 1000).is_some(), "first message opens bucket");
        assert!(d.try_digest(&t(1020), 1020).is_none(), "second merges into bucket");
        assert_eq!(d.bucket_count(), 1);
    }

    #[test]
    fn test_router_creates_session_and_routes() {
        let mut r = SessionRouter::new();
        let (ingested, digested, out) = r.route(SessionMessage::new("s1", "a1", "task", "hello"), "crawl");
        assert_eq!(ingested, 1);
        assert_eq!(digested, 0, "first message not digested");
        assert_eq!(out.len(), 1);
        assert!(r.session("s1").is_some());
        assert_eq!(r.session_count(), 1);
    }

    #[test]
    fn test_router_digests_high_frequency_same_topic() {
        let mut r = SessionRouter::new();
        let m1 = SessionMessage::new("s1", "a1", "task", "m1").with_topic("t1");
        let (_, _, out1) = r.route(m1, "crawl");
        assert_eq!(out1.len(), 1, "first emitted");
        let m2 = SessionMessage::new("s1", "a1", "task", "m2").with_topic("t1");
        let (_, digested, out2) = r.route(m2, "crawl");
        assert_eq!(digested, 1, "second merged into digest bucket");
        assert!(out2.is_empty(), "no outbound while merging");
    }
}