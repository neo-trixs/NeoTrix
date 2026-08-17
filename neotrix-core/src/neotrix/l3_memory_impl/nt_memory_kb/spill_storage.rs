//! # SpillStorage — 超大结果溢写存储 (deepseek-harness #9 吸收)
//!
//! 吸收自 notes/absorption-20260817-deepseek-harness.md #9 (spill.md):
//! - 超大生成结果不直接占内存, 溢写 (spill) 到持久 backend;
//! - 调用方只拿到**不透明 SpillLocator** (无内容泄露, 仅 backend + byte_len + key);
//! - 按需 retrieve 还原; spill 策略 (阈值) 决定何时溢写而非内联。
//!
//! R-P42: 强化 KB 既有 store 类型路径, 不建平行存储系统 — 本模块是
//! `nt_memory_kb` 内的独立组件, 不修改 KnowledgeBase 结构体 (15 处字面量
//! 构造在外, 禁加字段)。
//!
//! 状态存于模块内 `RwLock<HashMap>`; 生产接线点: KnowledgeBase 大结果
//! 检索/写入路径可将超阈值内容交给 `SpillStorage::store`。

use std::collections::HashMap;
use std::sync::RwLock;

/// 溢写策略配置。
#[derive(Debug, Clone, Copy)]
pub struct SpillConfig {
    /// 达到或超过该字节数 → 溢写; 否则内联返回原内容。
    pub threshold_bytes: usize,
    /// 溢写 backend 标识 (future: "file" / "kv" / "object-store")。
    pub backend: &'static str,
}

impl Default for SpillConfig {
    fn default() -> Self {
        Self {
            // 环境可配置 (R-P11): NEOTRIX_SPILL_THRESHOLD_BYTES
            threshold_bytes: std::env::var("NEOTRIX_SPILL_THRESHOLD_BYTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(64 * 1024),
            backend: "memory",
        }
    }
}

/// 不透明溢写定位符 — 调用方可见的最小信息。
///
/// **不泄露内容**: 只含定位 key、backend、字节长度; 不含原文。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpillLocator {
    key: String,
    backend: String,
    byte_len: usize,
}

impl SpillLocator {
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn backend(&self) -> &str {
        &self.backend
    }
    pub fn byte_len(&self) -> usize {
        self.byte_len
    }
}

/// 存储结果 — 小结果内联, 大结果仅持 locator。
#[derive(Debug, Clone)]
pub enum StoredSpill {
    Inline(Vec<u8>),
    Spilled(SpillLocator),
}

impl StoredSpill {
    pub fn byte_len(&self) -> usize {
        match self {
            Self::Inline(bytes) => bytes.len(),
            Self::Spilled(loc) => loc.byte_len(),
        }
    }

    pub fn is_spilled(&self) -> bool {
        matches!(self, Self::Spilled(_))
    }
}

/// 溢出存储统计。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SpillStats {
    pub stored: u64,
    pub retrieved: u64,
    pub spilled_bytes: u64,
    pub inline_bytes: u64,
}

/// 超大结果溢写存储。
pub struct SpillStorage {
    /// key → 内容 (溢写 backend 为 memory 时的落盘位)。
    blobs: RwLock<HashMap<String, Vec<u8>>>,
    config: SpillConfig,
    stats: RwLock<SpillStats>,
}

impl SpillStorage {
    pub fn new(config: SpillConfig) -> Self {
        Self {
            blobs: RwLock::new(HashMap::new()),
            config,
            stats: RwLock::new(SpillStats::default()),
        }
    }

    /// 无 key 存储: 生成不透明 key (uuid) 后调用 [`store_with_key`]。
    pub fn store(&self, content: &[u8]) -> StoredSpill {
        let key = uuid::Uuid::new_v4().to_string();
        self.store_with_key(&key, content)
    }

    /// 按策略存储: 超阈值 → 溢写并返回 `Spilled(locator)`;
    /// 未超阈值 → 返回 `Inline(content)` (不上锁落盘)。
    pub fn store_with_key(&self, key: &str, content: &[u8]) -> StoredSpill {
        let mut stats = self.stats.write().unwrap_or_else(|e| e.into_inner());
        if content.len() >= self.config.threshold_bytes {
            self.blobs
                .write()
                .unwrap_or_else(|e| e.into_inner())
                .insert(key.to_string(), content.to_vec());
            stats.stored += 1;
            stats.spilled_bytes += content.len() as u64;
            StoredSpill::Spilled(SpillLocator {
                key: key.to_string(),
                backend: self.config.backend.to_string(),
                byte_len: content.len(),
            })
        } else {
            stats.stored += 1;
            stats.inline_bytes += content.len() as u64;
            StoredSpill::Inline(content.to_vec())
        }
    }

    /// 按 locator 还原内容 (仅 `Spilled` 可还原; `Inline` 内容直接返回)。
    pub fn retrieve(&self, stored: &StoredSpill) -> Option<Vec<u8>> {
        match stored {
            StoredSpill::Inline(bytes) => Some(bytes.clone()),
            StoredSpill::Spilled(loc) => {
                let bytes = self
                    .blobs
                    .read()
                    .unwrap_or_else(|e| e.into_inner())
                    .get(&loc.key)
                    .cloned()?;
                if let Ok(mut stats) = self.stats.write() {
                    stats.retrieved += 1;
                }
                Some(bytes)
            }
        }
    }

    /// 丢弃溢写内容 (删除 backend blob)。
    pub fn delete(&self, locator: &SpillLocator) -> bool {
        self.blobs
            .write()
            .map(|mut b| b.remove(&locator.key).is_some())
            .unwrap_or(false)
    }

    /// 阈值是否触发过 (供测试断言 spill 策略)。
    pub fn config(&self) -> &SpillConfig {
        &self.config
    }

    pub fn stats(&self) -> SpillStats {
        self.stats.read().map(|s| *s).unwrap_or_default()
    }

    pub fn blob_count(&self) -> usize {
        self.blobs.read().map(|b| b.len()).unwrap_or(0)
    }

    /// 溢出层完整性 (C5 自愈): 索引映射 (key → 位置) 与数据层一致, 无悬挂索引。
    /// 悬挂索引 = 索引条目指向已丢失/空的数据 (无法 restore 的条目)。
    pub fn is_consistent(&self) -> bool {
        self.blobs
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .all(|data| !data.is_empty())
    }

    /// 重建索引 (C5 自愈): 扫描数据层, 移除指向缺失数据的悬挂索引条目,
    /// 返回修复动作列表 (无悬挂时返回空列表)。
    pub fn rebuild_index(&mut self) -> Vec<String> {
        let dangling: Vec<String> = self
            .blobs
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .filter(|(_, data)| data.is_empty())
            .map(|(key, _)| key.clone())
            .collect();
        if dangling.is_empty() {
            return Vec::new();
        }
        let mut actions = Vec::new();
        let mut blobs = self.blobs.write().unwrap_or_else(|e| e.into_inner());
        for key in dangling {
            blobs.remove(&key);
            actions.push(format!("removed dangling index entry '{}' (data missing)", key));
        }
        actions
    }
}

/// C5 自愈检测件 (MEMORY, spill_storage): 构造含悬挂索引的存储,
/// rebuild_index 修复后断言 is_consistent。
pub struct SpillStorageHealer;

impl crate::core::nt_core_self_test::SelfTest for SpillStorageHealer {
    fn name(&self) -> &str {
        "nt_memory_kb::spill_storage_healer"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();

        let store = SpillStorage::new(SpillConfig {
            threshold_bytes: 8,
            backend: "memory",
        });
        if !store.is_consistent() {
            failures.push("empty store must be consistent".into());
        }

        let mut store = SpillStorage::new(SpillConfig {
            threshold_bytes: 8,
            backend: "memory",
        });
        store.store_with_key("healthy", &vec![b'x'; 16]);
        store
            .blobs
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .insert("dangling".to_string(), Vec::new());
        if store.is_consistent() {
            failures.push("dangling index not detected".into());
        }
        let actions = store.rebuild_index();
        if actions.is_empty() {
            failures.push("rebuild_index removed nothing".into());
        }
        if !store.is_consistent() {
            failures.push("store still inconsistent after rebuild".into());
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

impl Default for SpillStorage {
    fn default() -> Self {
        Self::new(SpillConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_config() -> SpillConfig {
        SpillConfig {
            threshold_bytes: 8,
            backend: "memory",
        }
    }

    // (a) 小内容内联, 不落盘
    #[test]
    fn small_content_inline_no_blob() {
        let store = SpillStorage::new(small_config());
        let result = store.store(b"tiny");
        assert!(!result.is_spilled());
        assert_eq!(result.byte_len(), 4);
        assert_eq!(store.blob_count(), 0);
        // Inline 内容 retrieve 直接返回
        assert_eq!(store.retrieve(&result), Some(b"tiny".to_vec()));
    }

    // (b) 超大内容溢写 + locator 往返还原
    #[test]
    fn oversized_spills_and_round_trips() {
        let store = SpillStorage::new(small_config());
        let content = vec![b'x'; 100];
        let result = store.store(&content);
        assert!(result.is_spilled());
        assert_eq!(store.blob_count(), 1);
        let loc = match &result {
            StoredSpill::Spilled(l) => l.clone(),
            _ => panic!("expected Spilled"),
        };
        // locator 不透明: 不含内容
        assert!(!loc.key().contains('x'));
        assert_eq!(loc.backend(), "memory");
        assert_eq!(loc.byte_len(), 100);
        // 还原
        assert_eq!(store.retrieve(&result), Some(content));
    }

    // (c) locator 不泄露内容 (key 随机且不含原文; 字节长度字段仅元数据)
    #[test]
    fn locator_is_opaque() {
        let store = SpillStorage::new(small_config());
        let result = store.store(b"SECRET-CONTENT-PAYLOAD");
        let loc = match &result {
            StoredSpill::Spilled(l) => l,
            _ => panic!("expected Spilled"),
        };
        assert_eq!(loc.byte_len(), 22);
        let debug = format!("{:?}", loc);
        assert!(!debug.contains("SECRET-CONTENT-PAYLOAD"));
    }

    // (d) 阈值边界: 恰好等于阈值 → 溢写; 小于 → 内联
    #[test]
    fn threshold_boundary_honored() {
        let store = SpillStorage::new(small_config()); // threshold = 8
        assert!(!store.store(b"1234567").is_spilled()); // 7 < 8 inline
        assert!(store.store(b"12345678").is_spilled()); // 8 == 8 spill
    }

    // (e) C5 自愈: 正常一致的溢出层 (无悬挂索引)
    #[test]
    fn spill_index_consistent_when_healthy() {
        let mut store = SpillStorage::new(small_config());
        store.store_with_key("k1", &vec![b'a'; 16]);
        store.store_with_key("k2", &vec![b'b'; 16]);
        assert!(store.is_consistent());
        assert!(store.rebuild_index().is_empty());
        assert_eq!(store.blob_count(), 2);
    }

    // (f) C5 自愈: 悬挂索引被 rebuild_index 移除后恢复一致
    #[test]
    fn spill_dangling_index_rebuilt() {
        let mut store = SpillStorage::new(small_config());
        store.store_with_key("k1", &vec![b'a'; 16]);
        store
            .blobs
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .insert("dangling".to_string(), Vec::new());
        assert!(!store.is_consistent());
        let actions = store.rebuild_index();
        assert!(!actions.is_empty());
        assert!(store.is_consistent());
        assert_eq!(store.blob_count(), 1);
    }
}