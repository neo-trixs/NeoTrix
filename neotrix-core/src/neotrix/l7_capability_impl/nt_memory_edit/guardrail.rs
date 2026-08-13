//! nt_memory::edit::guardrail — 编辑完整性护栏
//!
//! 节点: nt_memory::edit::guardrail (L1)
//! Provides: edit_guardrail, edit_sanitization
//!
//! 护栏职责: 拒绝畸形/异常编辑 (来源未知、理由缺失、超大 payload),
//! 防止知识库被批量污染 (对齐 memory editing 文献的 integrity guardrail)。

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use crate::neotrix::l7_capability_impl::nt_memory_edit::edit_log::{EditKind, MemoryEdit};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuardrailConfig {
    pub max_payload_len: usize,
    pub require_reason: bool,
    pub require_source: bool,
    pub max_edits_per_cycle: usize,
}

impl Default for GuardrailConfig {
    fn default() -> Self {
        Self {
            max_payload_len: 16 * 1024,
            require_reason: true,
            require_source: true,
            max_edits_per_cycle: 100,
        }
    }
}

/// 编辑护栏
#[derive(Debug, Clone)]
pub struct EditGuardrail {
    config: GuardrailConfig,
}

impl EditGuardrail {
    pub fn new(max_edits_per_cycle: usize) -> Self {
        let mut config = GuardrailConfig::default();
        config.max_edits_per_cycle = max_edits_per_cycle;
        Self { config }
    }

    pub fn config(&self) -> &GuardrailConfig {
        &self.config
    }

    /// 检查单条编辑是否通过护栏
    pub fn check(&self, edit: &MemoryEdit) -> Result<(), NeoTrixError> {
        if self.config.require_reason && edit.reason.trim().is_empty() {
            return Err(NeoTrixError::InvalidInput(
                "编辑缺少 reason (禁止无理由写入)".into(),
            ));
        }
        if self.config.require_source && edit.source.trim().is_empty() {
            return Err(NeoTrixError::InvalidInput(
                "编辑缺少 source (禁止无来源写入)".into(),
            ));
        }
        let payload_len = edit.before.as_ref().map(|s| s.len()).unwrap_or(0)
            + edit.after.as_ref().map(|s| s.len()).unwrap_or(0);
        if payload_len > self.config.max_payload_len {
            return Err(NeoTrixError::InvalidInput(format!(
                "编辑 payload {}B 超限 (max {}B)",
                payload_len, self.config.max_payload_len
            )));
        }
        Ok(())
    }

    /// 批量过滤: 返回合法编辑 (丢弃不合规, 不 panic)
    pub fn sanitize<'a>(&self, edits: &[&'a MemoryEdit]) -> Vec<&'a MemoryEdit> {
        edits
            .iter()
            .copied()
            .filter(|e| self.check(e).is_ok())
            .collect()
    }

    /// 批量注入逃逸: 某来源在单 cycle 内编辑数超限 → 整体拒绝 (防污染)
    pub fn check_batch(&self, edits: &[MemoryEdit]) -> Result<(), NeoTrixError> {
        if edits.len() > self.config.max_edits_per_cycle {
            return Err(NeoTrixError::InvalidInput(format!(
                "批量编辑 {} 条超限 (max {})",
                edits.len(),
                self.config.max_edits_per_cycle
            )));
        }
        Ok(())
    }
}

impl CapabilityNode for EditGuardrail {
    fn node_id(&self) -> &str {
        "nt_memory::edit::guardrail"
    }
    fn provides(&self) -> Vec<String> {
        vec!["edit_guardrail".into(), "edit_sanitization".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["knowledge_editing".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Golden, RuneSocket::Obsidian]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for EditGuardrail {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let g = EditGuardrail::new(10);
        let ok = MemoryEdit::insert(1, "kv", "k", "v", "source", "reason", 1);
        assert!(g.check(&ok).is_ok(), "合规编辑应通过");
        let no_reason = MemoryEdit {
            id: 2,
            namespace: "kv".into(),
            kind: EditKind::Insert,
            key: "k".into(),
            before: None,
            after: Some("v".into()),
            reason: "  ".into(),
            source: "s".into(),
            applied_at: 1,
        };
        assert!(g.check(&no_reason).is_err(), "无理由应拒绝");
        let batch = vec![ok.clone(); 11];
        assert!(g.check_batch(&batch).is_err(), "批量超限应拒绝");
        Ok(())
    }

    fn name(&self) -> &str {
        "nt_memory_edit_guardrail"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_requires_provenance() {
        let g = EditGuardrail::new(10);
        let no_source = MemoryEdit {
            id: 1,
            namespace: "kv".into(),
            kind: EditKind::Insert,
            key: "k".into(),
            before: None,
            after: Some("v".into()),
            reason: "r".into(),
            source: "".into(),
            applied_at: 1,
        };
        assert!(g.check(&no_source).is_err());
    }

    #[test]
    fn test_payload_limit() {
        let g = EditGuardrail::new(10);
        let huge = MemoryEdit::insert(1, "kv", "k", &"x".repeat(20_000), "s", "r", 1);
        assert!(g.check(&huge).is_err());
    }

    #[test]
    fn test_sanitize_filters_bad() {
        let g = EditGuardrail::new(10);
        let good = MemoryEdit::insert(1, "kv", "k1", "v1", "s", "r", 1);
        let bad = MemoryEdit {
            id: 2,
            namespace: "kv".into(),
            kind: EditKind::Insert,
            key: "k2".into(),
            before: None,
            after: Some("v2".into()),
            reason: "".into(),
            source: "s".into(),
            applied_at: 1,
        };
        let filtered = g.sanitize(&[&good, &bad]);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }

    #[test]
    fn test_batch_limit() {
        let g = EditGuardrail::new(3);
        let e = MemoryEdit::insert(1, "kv", "k", "v", "s", "r", 1);
        assert!(g.check_batch(&[e.clone(), e.clone(), e.clone()]).is_ok());
        assert!(g.check_batch(&vec![e.clone(); 4]).is_err());
    }
}
