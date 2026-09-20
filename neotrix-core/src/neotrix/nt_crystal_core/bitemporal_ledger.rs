//! BitemporalLedger — 双时间线审计账本
//!
//! 基于 ECHO (2026) 的双时间线设计：
//! - valid_from/valid_to: 信息何时为真 (有效时间)
//! - transaction_time: 何时写入 (事务时间)
//! - superseded_by: 版本链 (被哪条替代)
//! - source: 来源链 (溯源)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════
// 核心类型
// ═══════════════════════════════════════════════════════════════

/// 条目 ID
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct EntryId(pub String);

impl EntryId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

/// 时间戳 (毫秒 since epoch)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl Timestamp {
    pub fn now() -> Self {
        Self(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        )
    }
}

/// 来源链 — 记忆的溯源信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceChain {
    /// 来源描述
    pub source: String,
    /// 置信度
    pub confidence: f64,
    /// 上游来源 (递归)
    pub parent: Option<Box<ProvenanceChain>>,
}

/// 记忆内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContent {
    /// 文本内容
    pub text: String,
    /// 向量嵌入
    pub embedding: Vec<f64>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

// ═══════════════════════════════════════════════════════════════
// 账本条目
// ═══════════════════════════════════════════════════════════════

/// 双时间线条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// 条目 ID
    pub id: EntryId,
    /// 内容
    pub content: MemoryContent,
    /// 有效开始时间 (信息何时为真)
    pub valid_from: Timestamp,
    /// 有效截止时间 (被替代时设置)
    pub valid_to: Option<Timestamp>,
    /// 事务时间 (何时写入)
    pub transaction_time: Timestamp,
    /// 被哪条替代
    pub superseded_by: Option<EntryId>,
    /// 来源链
    pub source: ProvenanceChain,
    /// 置信度
    pub confidence: f64,
    /// 语义键 (用于查询)
    pub key: String,
}

impl LedgerEntry {
    /// 是否当前有效
    pub fn is_current(&self) -> bool {
        self.valid_to.is_none()
    }

    /// 在给定时间是否有效
    pub fn is_valid_at(&self, time: Timestamp) -> bool {
        self.valid_from <= time && self.valid_to.map_or(true, |t| t > time)
    }
}

// ═══════════════════════════════════════════════════════════════
// 双时间线账本
// ═══════════════════════════════════════════════════════════════

/// 双时间线审计账本
///
/// 支持:
/// - 追加式写入 (不覆盖，只追加新版本)
/// - 当前有效视图 (valid_to = None)
/// - 时间点查询 (某时刻的有效条目)
/// - 版本链追踪 (superseded_by)
/// - 来源溯源 (ProvenanceChain)
pub struct BitemporalLedger {
    /// 所有条目
    pub entries: HashMap<EntryId, LedgerEntry>,
    /// 当前有效视图 (key → EntryId)
    pub current_view: HashMap<String, EntryId>,
    /// 版本链 (EntryId → 下一个版本)
    pub version_chain: HashMap<EntryId, EntryId>,
    /// 操作日志
    pub operation_log: Vec<LedgerOperation>,
}

/// 账本操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerOperation {
    pub op_type: String,
    pub entry_id: EntryId,
    pub timestamp: Timestamp,
    pub details: String,
}

impl BitemporalLedger {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            current_view: HashMap::new(),
            version_chain: HashMap::new(),
            operation_log: Vec::new(),
        }
    }

    /// 追加新版本 (不覆盖)
    pub fn append(
        &mut self,
        key: &str,
        content: MemoryContent,
        source: ProvenanceChain,
    ) -> EntryId {
        let new_id = EntryId::new();
        let now = Timestamp::now();

        // 标记旧条目为已替代
        if let Some(old_id) = self.current_view.remove(key) {
            if let Some(old) = self.entries.get_mut(&old_id) {
                old.valid_to = Some(now);
                old.superseded_by = Some(new_id.clone());
            }
            self.version_chain.insert(old_id, new_id.clone());
        }

        let entry = LedgerEntry {
            id: new_id.clone(),
            content,
            valid_from: now,
            valid_to: None,
            transaction_time: now,
            superseded_by: None,
            source,
            confidence: 1.0,
            key: key.to_string(),
        };

        self.entries.insert(new_id.clone(), entry);
        self.current_view.insert(key.to_string(), new_id.clone());

        self.operation_log.push(LedgerOperation {
            op_type: "append".to_string(),
            entry_id: new_id.clone(),
            timestamp: now,
            details: format!("New version for key: {}", key),
        });

        new_id
    }

    /// 查询当前有效条目
    pub fn current(&self, key: &str) -> Option<&LedgerEntry> {
        self.current_view
            .get(key)
            .and_then(|id| self.entries.get(id))
    }

    /// 查询某时间点的有效条目
    pub fn at_time(&self, key: &str, time: Timestamp) -> Option<&LedgerEntry> {
        self.entries
            .values()
            .filter(|e| e.key == key)
            .filter(|e| e.is_valid_at(time))
            .max_by_key(|e| e.transaction_time)
    }

    /// 获取版本链
    pub fn version_history(&self, key: &str) -> Vec<&LedgerEntry> {
        let mut history = Vec::new();
        let mut current_id = self.current_view.get(key).cloned();

        while let Some(id) = current_id {
            if let Some(entry) = self.entries.get(&id) {
                history.push(entry);
                // 沿版本链向前 (找 superseded_by 指向这个条目的)
                current_id = self.entries.values()
                    .find(|e| e.superseded_by.as_ref() == Some(&id))
                    .map(|e| e.id.clone());
            } else {
                break;
            }
        }

        history
    }

    /// 获取所有当前有效条目
    pub fn all_current(&self) -> Vec<&LedgerEntry> {
        self.current_view
            .values()
            .filter_map(|id| self.entries.get(id))
            .collect()
    }

    /// 获取统计信息
    pub fn stats(&self) -> LedgerStats {
        let current_count = self.entries.values().filter(|e| e.is_current()).count();
        let total_versions = self.entries.len();

        LedgerStats {
            total_entries: total_versions,
            current_entries: current_count,
            versioned_entries: total_versions - current_count,
            unique_keys: self.current_view.len(),
            operations: self.operation_log.len(),
        }
    }
}

/// 账本统计
#[derive(Debug, Clone)]
pub struct LedgerStats {
    pub total_entries: usize,
    pub current_entries: usize,
    pub versioned_entries: usize,
    pub unique_keys: usize,
    pub operations: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_content(text: &str) -> MemoryContent {
        MemoryContent {
            text: text.to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata: HashMap::new(),
        }
    }

    fn make_source(source: &str) -> ProvenanceChain {
        ProvenanceChain {
            source: source.to_string(),
            confidence: 1.0,
            parent: None,
        }
    }

    #[test]
    fn test_append_and_current() {
        let mut ledger = BitemporalLedger::new();
        let id1 = ledger.append("rust_version", make_content("Rust 1.75"), make_source("release_notes"));
        let id2 = ledger.append("rust_version", make_content("Rust 1.76"), make_source("release_notes"));

        // 当前应该是 1.76
        let current = ledger.current("rust_version").unwrap();
        assert_eq!(current.content.text, "Rust 1.76");
        assert!(current.is_current());

        // 版本链应该有 2 个条目
        let history = ledger.version_history("rust_version");
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_at_time() {
        let mut ledger = BitemporalLedger::new();
        ledger.append("key", make_content("v1"), make_source("s1"));

        // 立即追加 v2
        ledger.append("key", make_content("v2"), make_source("s2"));

        // 查询当前 (应该是 v2)
        let current = ledger.current("key").unwrap();
        assert_eq!(current.content.text, "v2");
    }

    #[test]
    fn test_stats() {
        let mut ledger = BitemporalLedger::new();
        ledger.append("a", make_content("v1"), make_source("s1"));
        ledger.append("b", make_content("v1"), make_source("s1"));
        ledger.append("a", make_content("v2"), make_source("s1"));

        let stats = ledger.stats();
        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.current_entries, 2); // a=v2, b=v1
        assert_eq!(stats.unique_keys, 2);
    }
}
