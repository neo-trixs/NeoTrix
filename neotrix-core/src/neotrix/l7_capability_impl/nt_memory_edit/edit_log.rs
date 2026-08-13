//! nt_memory::edit::edit_log — 版本化知识编辑日志
//!
//! 节点: nt_memory::edit::edit_log (L0)
//! Provides: knowledge_editing, edit_logging, edit_rollback, edit_provenance

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

/// 编辑操作类型 (对齐记忆 3D 分类法的写路径)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EditKind {
    Insert, // 新增知识条目
    Update, // 修正既有条目
    Remove, // 删除过期条目
}

/// 单条编辑记录 (带 provenance, 可回滚)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryEdit {
    pub id: u64,
    pub namespace: String,
    pub kind: EditKind,
    pub key: String,
    pub before: Option<String>,
    pub after: Option<String>,
    pub reason: String,
    pub source: String,
    pub applied_at: u64,
}

impl MemoryEdit {
    pub fn insert(
        id: u64,
        namespace: &str,
        key: &str,
        value: &str,
        source: &str,
        reason: &str,
        now: u64,
    ) -> Self {
        Self {
            id,
            namespace: namespace.into(),
            kind: EditKind::Insert,
            key: key.into(),
            before: None,
            after: Some(value.into()),
            reason: reason.into(),
            source: source.into(),
            applied_at: now,
        }
    }

    pub fn update(
        id: u64,
        namespace: &str,
        key: &str,
        before: &str,
        after: &str,
        source: &str,
        reason: &str,
        now: u64,
    ) -> Self {
        Self {
            id,
            namespace: namespace.into(),
            kind: EditKind::Update,
            key: key.into(),
            before: Some(before.into()),
            after: Some(after.into()),
            reason: reason.into(),
            source: source.into(),
            applied_at: now,
        }
    }

    pub fn remove(
        id: u64,
        namespace: &str,
        key: &str,
        before: &str,
        source: &str,
        reason: &str,
        now: u64,
    ) -> Self {
        Self {
            id,
            namespace: namespace.into(),
            kind: EditKind::Remove,
            key: key.into(),
            before: Some(before.into()),
            after: None,
            reason: reason.into(),
            source: source.into(),
            applied_at: now,
        }
    }
}

/// 版本化知识编辑日志
#[derive(Debug, Clone, Default)]
pub struct KnowledgeEditLog {
    entries: Vec<MemoryEdit>,
    store: HashMap<String, String>,
    next_id: u64,
}

impl KnowledgeEditLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// 校验编辑合法性 (护栏入口, 与 guardrail 共用语义):
    /// - Insert 必须有 after; Update 必须同时有 before+after; Remove 必须只有 before。
    /// - key 不允许空串。
    pub fn validate(edit: &MemoryEdit) -> Result<(), NeoTrixError> {
        if edit.key.is_empty() || edit.namespace.is_empty() {
            return Err(NeoTrixError::InvalidInput(
                "编辑 key/namespace 不能为空".into(),
            ));
        }
        match edit.kind {
            EditKind::Insert => {
                if edit.after.is_none() {
                    return Err(NeoTrixError::InvalidInput(
                        "Insert 必须提供 after 值".into(),
                    ));
                }
            }
            EditKind::Update => {
                if edit.before.is_none() || edit.after.is_none() {
                    return Err(NeoTrixError::InvalidInput(
                        "Update 必须同时提供 before+after".into(),
                    ));
                }
            }
            EditKind::Remove => {
                if edit.before.is_none() || edit.after.is_some() {
                    return Err(NeoTrixError::InvalidInput("Remove 只能提供 before".into()));
                }
            }
        }
        Ok(())
    }

    /// 应用一条编辑 (幂等: id 已存在则拒绝)
    pub fn apply(&mut self, edit: MemoryEdit) -> Result<(), NeoTrixError> {
        Self::validate(&edit)?;
        if self.entries.iter().any(|e| e.id == edit.id) {
            return Err(NeoTrixError::InvalidState(format!(
                "编辑 id {} 已存在",
                edit.id
            )));
        }
        let full_key = format!("{}::{}", edit.namespace, edit.key);
        match edit.kind {
            EditKind::Insert => {
                if self.store.contains_key(&full_key) {
                    return Err(NeoTrixError::InvalidState(format!(
                        "key {} 已存在, 用 Update",
                        full_key
                    )));
                }
                self.store.insert(full_key, edit.after.clone().unwrap());
            }
            EditKind::Update => {
                self.store.insert(full_key, edit.after.clone().unwrap());
            }
            EditKind::Remove => {
                if !self.store.contains_key(&full_key) {
                    return Err(NeoTrixError::NotFound(format!(
                        "key {} 不存在, 无法删除",
                        full_key
                    )));
                }
                self.store.remove(&full_key);
            }
        }
        self.entries.push(edit);
        Ok(())
    }

    /// 回滚指定编辑 (逆向应用), 返回逆向操作
    pub fn rollback(&mut self, id: u64) -> Result<MemoryEdit, NeoTrixError> {
        let idx = self
            .entries
            .iter()
            .position(|e| e.id == id)
            .ok_or_else(|| NeoTrixError::NotFound(format!("编辑 id {} 未找到", id)))?;
        let edit = self.entries.remove(idx);
        // 逆向应用 (不追加新日志, 保持历史纯净)
        let full_key = format!("{}::{}", edit.namespace, edit.key);
        match edit.kind {
            EditKind::Insert => {
                self.store.remove(&full_key);
            }
            EditKind::Remove => {
                self.store.insert(full_key, edit.before.clone().unwrap());
            }
            EditKind::Update => {
                // 回到 before
                if let Some(b) = &edit.before {
                    self.store.insert(full_key, b.clone());
                }
            }
        }
        Ok(edit)
    }

    pub fn entries(&self) -> &[MemoryEdit] {
        &self.entries
    }

    pub fn get(&self, namespace: &str, key: &str) -> Option<&str> {
        self.store
            .get(&format!("{}::{}", namespace, key))
            .map(|s| s.as_str())
    }

    pub fn next_id(&self) -> u64 {
        self.next_id
    }
}

impl CapabilityNode for KnowledgeEditLog {
    fn node_id(&self) -> &str {
        "nt_memory::edit::edit_log"
    }
    fn provides(&self) -> Vec<String> {
        vec![
            "knowledge_editing".into(),
            "edit_logging".into(),
            "edit_rollback".into(),
            "edit_provenance".into(),
        ]
    }
    fn requires(&self) -> Vec<String> {
        vec![]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![
            RuneSocket::Crimson,
            RuneSocket::Obsidian,
            RuneSocket::Golden,
        ]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for KnowledgeEditLog {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut log = KnowledgeEditLog::new();
        let now = 1700000000;
        log.apply(MemoryEdit::insert(1, "kv", "k1", "v1", "test", "新增", now))
            .map_err(|e| vec![e.to_string()])?;
        log.apply(MemoryEdit::update(
            2,
            "kv",
            "k1",
            "v1",
            "v2",
            "test",
            "修正",
            now + 1,
        ))
        .map_err(|e| vec![e.to_string()])?;
        assert_eq!(log.get("kv", "k1"), Some("v2"));
        log.rollback(2).map_err(|e| vec![e.to_string()])?;
        assert_eq!(log.get("kv", "k1"), Some("v1"));
        log.rollback(1).map_err(|e| vec![e.to_string()])?;
        assert_eq!(log.get("kv", "k1"), None);
        Ok(())
    }

    fn name(&self) -> &str {
        "nt_memory_edit_log"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_update_rollback() {
        let mut log = KnowledgeEditLog::new();
        log.apply(MemoryEdit::insert(1, "kv", "k1", "v1", "s", "r", 100))
            .unwrap();
        log.apply(MemoryEdit::update(2, "kv", "k1", "v1", "v2", "s", "r", 101))
            .unwrap();
        assert_eq!(log.get("kv", "k1"), Some("v2"));
        log.rollback(2).unwrap();
        assert_eq!(log.get("kv", "k1"), Some("v1"));
        log.rollback(1).unwrap();
        assert_eq!(log.get("kv", "k1"), None);
        assert!(log.entries().is_empty());
    }

    #[test]
    fn test_remove_and_rollback() {
        let mut log = KnowledgeEditLog::new();
        log.apply(MemoryEdit::insert(1, "kv", "k1", "v1", "s", "r", 100))
            .unwrap();
        log.apply(MemoryEdit::remove(2, "kv", "k1", "v1", "s", "过期", 102))
            .unwrap();
        assert_eq!(log.get("kv", "k1"), None);
        log.rollback(2).unwrap();
        assert_eq!(log.get("kv", "k1"), Some("v1"));
    }

    #[test]
    fn test_guardrail_rejects_malformed() {
        // Update 缺 before → 拒绝
        let bad = MemoryEdit {
            id: 1,
            namespace: "kv".into(),
            kind: EditKind::Update,
            key: "k".into(),
            before: None,
            after: Some("v".into()),
            reason: "x".into(),
            source: "x".into(),
            applied_at: 1,
        };
        assert!(KnowledgeEditLog::validate(&bad).is_err());
        // 空 key → 拒绝
        let bad2 = MemoryEdit::insert(2, "kv", "", "v", "s", "r", 1);
        assert!(KnowledgeEditLog::validate(&bad2).is_err());
    }

    #[test]
    fn test_duplicate_id_rejected() {
        let mut log = KnowledgeEditLog::new();
        log.apply(MemoryEdit::insert(1, "kv", "k1", "v1", "s", "r", 100))
            .unwrap();
        assert!(log
            .apply(MemoryEdit::insert(1, "kv", "k2", "v2", "s", "r", 101))
            .is_err());
    }

    #[test]
    fn test_insert_into_existing_key_rejected() {
        let mut log = KnowledgeEditLog::new();
        log.apply(MemoryEdit::insert(1, "kv", "k1", "v1", "s", "r", 100))
            .unwrap();
        assert!(log
            .apply(MemoryEdit::insert(2, "kv", "k1", "v2", "s", "r", 101))
            .is_err());
    }
}
