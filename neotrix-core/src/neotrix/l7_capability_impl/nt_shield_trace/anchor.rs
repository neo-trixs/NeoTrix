//! nt_shield::trace::anchor — 推理迹锚定 (完整性摘要)
//!
//! 节点: nt_shield::trace::anchor (L1)
//! Provides: trace_integrity, trace_verification
//!
//! 对推理迹生成稳定摘要, 事后校验是否被篡改 (防中间人改写推理链路)。

#![forbid(unsafe_code)]

use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};

/// 轻量 FNV-1a 摘要 (无外部依赖, 与 build_hygiene 同风格)
fn fnv1a(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnchoredTrace {
    pub id: u64,
    pub content: String,
    pub digest: u64,
}

/// 推理迹锚定器
#[derive(Debug, Clone, Default)]
pub struct TraceAnchor {
    traces: Vec<AnchoredTrace>,
    next_id: u64,
}

impl TraceAnchor {
    pub fn new() -> Self {
        Self::default()
    }

    /// 锚定一条推理迹, 返回 id
    pub fn anchor(&mut self, content: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.traces.push(AnchoredTrace {
            id,
            content: content.into(),
            digest: fnv1a(content),
        });
        id
    }

    /// 校验给定内容是否与锚定内容一致
    pub fn verify(&self, id: u64, content: &str) -> bool {
        self.traces
            .iter()
            .find(|t| t.id == id)
            .map(|t| t.digest == fnv1a(content) && t.content == content)
            .unwrap_or(false)
    }

    pub fn digest_of(&self, id: u64) -> Option<u64> {
        self.traces.iter().find(|t| t.id == id).map(|t| t.digest)
    }

    pub fn anchored_count(&self) -> usize {
        self.traces.len()
    }
}

impl CapabilityNode for TraceAnchor {
    fn node_id(&self) -> &str {
        "nt_shield::trace::anchor"
    }
    fn provides(&self) -> Vec<String> {
        vec!["trace_integrity".into(), "trace_verification".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["reasoning_trace_protection".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Alabaster, RuneSocket::Obsidian]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for TraceAnchor {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut a = TraceAnchor::new();
        let id = a.anchor("seed: 推理迹 v1");
        assert!(a.verify(id, "seed: 推理迹 v1"), "原样应通过");
        assert!(!a.verify(id, "seed: 推理迹 v2"), "篡改应失败");
        assert!(!a.verify(999, "anything"));
        assert_eq!(a.anchored_count(), 1);
        Ok(())
    }

    fn name(&self) -> &str {
        "nt_shield_trace_anchor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anchor_and_verify() {
        let mut a = TraceAnchor::new();
        let id = a.anchor("agent reasoning step 1");
        assert!(a.verify(id, "agent reasoning step 1"));
        assert!(!a.verify(id, "agent reasoning step 2"));
    }

    #[test]
    fn test_unknown_id_fails() {
        let a = TraceAnchor::new();
        assert!(!a.verify(42, "x"));
        assert!(a.digest_of(42).is_none());
    }

    #[test]
    fn test_digest_stable() {
        let mut a = TraceAnchor::new();
        let id1 = a.anchor("same content");
        let id2 = a.anchor("same content");
        assert_eq!(a.digest_of(id1), a.digest_of(id2));
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_deterministic_fnv() {
        let mut a = TraceAnchor::new();
        let id = a.anchor("abc");
        let expected = fnv1a("abc");
        assert_eq!(a.digest_of(id), Some(expected));
    }
}
