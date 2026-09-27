//! `nt_daemon` — wake 去抖 + triage 门 + steer 注入.
//!
//! 协作纪律本地版：per-agent 串行、burst 合并（2.5s debounce）、
//! 轻量 triage 门、同轮 steer 注入。无 Redis/SSE，纯内存 + 可测试纯函数.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// 人/系统对运行中回合的干预.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteerMsg {
    pub agent_id: String,
    pub text: String,
}

/// daemon 门控 (内存态; 多 daemon 场景由 SQLite outbox 兜底).
#[derive(Debug)]
pub struct DaemonGate {
    debounce: Duration,
    last_wake: HashMap<String, Instant>,
    last_text: HashMap<String, String>,
    steer: Vec<SteerMsg>,
}

impl DaemonGate {
    pub fn new() -> Self {
        Self {
            debounce: Duration::from_millis(2500),
            last_wake: HashMap::new(),
            last_text: HashMap::new(),
            steer: Vec::new(),
        }
    }

    /// wake 是否放行 (2.5s 内同 agent 合并为一次).
    pub fn should_wake(&mut self, agent_id: &str) -> bool {
        let now = Instant::now();
        match self.last_wake.get(agent_id) {
            Some(last) if now.duration_since(*last) < self.debounce => false,
            _ => {
                self.last_wake.insert(agent_id.to_owned(), now);
                true
            }
        }
    }

    /// triage 门：空文本/与上次完全重复 → 丢弃（省模型调用）。
    pub fn triage(&mut self, agent_id: &str, text: &str) -> bool {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return false;
        }
        if self.last_text.get(agent_id).is_some_and(|last| last == trimmed) {
            return false;
        }
        self.last_text.insert(agent_id.to_owned(), trimmed.to_owned());
        true
    }

    /// 注入 steer (同轮中断用).
    pub fn push_steer(&mut self, msg: SteerMsg) {
        self.steer.push(msg);
    }

    /// 取走某 agent 的全部 steer.
    pub fn drain_steer(&mut self, agent_id: &str) -> Vec<SteerMsg> {
        let mut hit = Vec::new();
        let mut rest = Vec::new();
        for msg in self.steer.drain(..) {
            if msg.agent_id == agent_id {
                hit.push(msg);
            } else {
                rest.push(msg);
            }
        }
        self.steer = rest;
        hit
    }
}

impl Default for DaemonGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::DaemonGate;

    #[test]
    fn debounce_merges_burst() {
        let mut gate = DaemonGate::new();
        assert!(gate.should_wake("a"));
        assert!(!gate.should_wake("a"));
        assert!(gate.should_wake("b"));
    }

    #[test]
    fn triage_drops_empty_and_dup() {
        let mut gate = DaemonGate::new();
        assert!(!gate.triage("a", "   "));
        assert!(gate.triage("a", "hello"));
        assert!(!gate.triage("a", "hello"));
        assert!(gate.triage("a", "hello2"));
    }
}
