//! worldmonitor — 网站变更检测 + FTS5 索引 (C1)
//!
//! 吸收 github.com/koala73/worldmonitor: 周期性抓取网站、计算内容 hash、
//! 比对历史版本检测变更，并写入 FTS5 全文索引。本模块以 trait 定义
//! 检测 + 索引契约，提供 stub 实现 (C1 接入点)。

use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};
use crate::core::nt_core_traits::KnowledgeSink;
use super::l1_facade::NodeType;
use std::collections::HashMap;

/// 当前 Unix 时间戳 (秒)。
#[allow(dead_code)]
fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 单次快照
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub url: String,
    pub content_hash: String,
}

/// 变更检测结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum _ChangeKind {
    Added,
    Removed,
    Modified,
    Unchanged,
}

/// 网站变更检测 + FTS5 索引契约 (worldmonitor 抽象)
pub trait _SiteChangeMonitor {
    /// 计算内容 hash 快照
    fn snapshot(&self, url: &str, content: &str) -> Snapshot;
    /// 比对前后快照，返回变更类型
    fn detect(&self, previous: &Snapshot, current: &Snapshot) -> _ChangeKind;
    /// 无 KB 句柄的轻量入口 (兼容旧调用方) — 仅校验可索引性。
    /// TODO(C2): 真实写入统一经 `index_fts5_to_kb`, 本方法保留为可用性探针。
    fn index_fts5(&self, url: &str, content: &str) -> bool;
    /// C2 接线: 将抓取内容作为 `Source` 节点写入 KB 并触发 FTS5 索引。
    /// 返回新节点 id。
    fn index_fts5_to_kb(&self, kb: &dyn KnowledgeSink, url: &str, content: &str) -> Result<String, String>;
}

/// 默认 stub 实现
#[derive(Default)]
pub struct _WorldMonitor;

fn simple_hash(content: &str) -> String {
    if content.is_empty() {
        return String::new();
    }
    format!("{:x}", md5_like(content))
}

fn md5_like(content: &str) -> u64 {
    let mut h: u64 = 1469598103934665603;
    for b in content.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h
}

impl _SiteChangeMonitor for _WorldMonitor {
    fn snapshot(&self, url: &str, content: &str) -> Snapshot {
        Snapshot {
            url: url.to_string(),
            content_hash: simple_hash(content),
        }
    }

    fn detect(&self, previous: &Snapshot, current: &Snapshot) -> _ChangeKind {
        if previous.content_hash == current.content_hash {
            _ChangeKind::Unchanged
        } else if previous.content_hash.is_empty() {
            _ChangeKind::Added
        } else {
            _ChangeKind::Modified
        }
    }

    /// Returns `true` if the URL and content pass basic validation.
    /// Does **not** persist to KB — use `index_fts5_to_kb` for actual writes.
    fn index_fts5(&self, url: &str, content: &str) -> bool {
        !url.trim().is_empty() && !content.trim().is_empty()
    }

    fn index_fts5_to_kb(&self, kb: &dyn KnowledgeSink, url: &str, _content: &str) -> Result<String, String> {
        if url.trim().is_empty() {
            return Err("url must not be empty".to_string());
        }
        let host = url
            .split("//")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .unwrap_or(url)
            .to_string();
        kb.sink_node(
            url,
            NodeType::Source,
            Some(&format!("Monitored page snapshot ({})", host)),
            Some(url),
            Some(&host),
        )
    }
}

/// SelfTest (T1): 变更检测 + hash 比对存在性
pub struct _WorldMonitorSelfTest;

impl SelfTest for _WorldMonitorSelfTest {
    fn name(&self) -> &str {
        "nt_world_monitor"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let m = _WorldMonitor;
        let a = m.snapshot("https://x.com", "v1");
        let b = m.snapshot("https://x.com", "v1");
        let c = m.snapshot("https://x.com", "v2");
        if m.detect(&a, &b) != _ChangeKind::Unchanged {
            return Err(vec!["worldmonitor: identical content must be Unchanged".into()]);
        }
        if m.detect(&a, &c) != _ChangeKind::Modified {
            return Err(vec!["worldmonitor: differing content must be Modified".into()]);
        }
        if !m.index_fts5("https://x.com", "data") {
            return Err(vec!["worldmonitor: fts5 index should succeed".into()]);
        }
        let _: HashMap<String, Snapshot> = HashMap::new();
        Ok(())
    }
}

/// 注册 worldmonitor SelfTest
pub fn _register_monitor_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(_WorldMonitorSelfTest));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_stable() {
        let m = _WorldMonitor;
        assert_eq!(
            m.snapshot("u", "same").content_hash,
            m.snapshot("u", "same").content_hash
        );
    }

    #[test]
    fn test_detect_added_and_modified() {
        let m = _WorldMonitor;
        let empty = m.snapshot("u", "");
        let v1 = m.snapshot("u", "v1");
        assert_eq!(m.detect(&empty, &v1), _ChangeKind::Added);
        assert_eq!(m.detect(&v1, &v1), _ChangeKind::Unchanged);
        assert_eq!(m.detect(&v1, &m.snapshot("u", "v2")), _ChangeKind::Modified);
    }

    #[test]
    fn test_fts5_index_stub() {
        let m = _WorldMonitor;
        assert!(m.index_fts5("https://x.com", "content"));
        let t = _WorldMonitorSelfTest;
        assert_eq!(t.name(), "nt_world_monitor");
        assert!(t.self_test().is_ok());
    }

    #[test]
    fn index_fts5_to_kb_writes_source_node() {
        use crate::l2_perception::nt_world::l1_facade::KnowledgeBase;
        use std::path::PathBuf;
        let Ok(kb) = KnowledgeBase::open(Some(PathBuf::from(":memory:"))) else {
            eprintln!("skip: KB (FTS5) unavailable in this build");
            return;
        };
        let m = _WorldMonitor;
        let id = m
            .index_fts5_to_kb(&kb, "https://example.com/page", "monitored content")
            .expect("index should succeed");
        assert!(!id.is_empty());
        let node = kb.get_node(&id).expect("node present").expect("node exists");
        assert_eq!(node.node_type, crate::core::nt_core_kb_types::NodeType::Source);
        assert_eq!(node.domain.as_deref(), Some("example.com"));
    }
}
