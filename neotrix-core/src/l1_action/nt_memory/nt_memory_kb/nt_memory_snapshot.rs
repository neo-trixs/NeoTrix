//! KB 快照 / diff 工具 (dbx absorb, G5) — 完整知识图谱的可复现快照 + 变更比较。
//!
//! dbx 提供 schema diff / backup 能力; 此处对齐: `KbSnapshot` 捕获全量节点/边/统计,
//! `diff_snapshots` 给出 added / removed / changed 三向清单, 供巡检、迁移、回滚评估。
//! 纯数据模块 — 唯一副作用是文件读写, 逻辑可单测。

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::{KnowledgeBase, KnowledgeEdge, KnowledgeNode, KnowledgeStats};

/// 快照文件格式标识。
pub const SNAPSHOT_FORMAT: &str = "neotrix-kb-snapshot";
/// 快照格式版本 (节点/边 schema 变更时递增)。
pub const SNAPSHOT_VERSION: u32 = 1;

/// 一次 KB 全量快照。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbSnapshot {
    pub format: String,
    pub version: u32,
    pub captured_at_ms: i64,
    pub stats: KnowledgeStats,
    pub nodes: Vec<KnowledgeNode>,
    pub edges: Vec<KnowledgeEdge>,
}

impl KbSnapshot {
    /// 内容指纹 — 节点 (id+updated_at) + 边 (id) 的确定性摘要, 用于零拷贝变更探测。
    pub fn checksum(&self) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        for n in &self.nodes {
            n.id.hash(&mut hasher);
            n.updated_at.hash(&mut hasher);
        }
        for e in &self.edges {
            e.id.hash(&mut hasher);
        }
        format!("{:016x}", hasher.finish())
    }
}

/// 变更清单中单个节点的轻量表示 (不携带大字段, 保持 diff 可读)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffNode {
    pub id: String,
    pub node_type: String,
    pub title: String,
    /// 变化的字段名 (title/summary/content/url/domain/importance/confidence/...)
    pub fields_changed: Vec<String>,
}

/// 变更清单中单条边的轻量表示。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
}

/// 两个快照的差异。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KbDiff {
    pub nodes_added: Vec<DiffNode>,
    pub nodes_removed: Vec<DiffNode>,
    pub nodes_changed: Vec<DiffNode>,
    pub edges_added: Vec<DiffEdge>,
    pub edges_removed: Vec<DiffEdge>,
}

impl KbDiff {
    pub fn is_empty(&self) -> bool {
        self.nodes_added.is_empty()
            && self.nodes_removed.is_empty()
            && self.nodes_changed.is_empty()
            && self.edges_added.is_empty()
            && self.edges_removed.is_empty()
    }
}

/// 捕获当前 KB 全量快照。
pub fn snapshot_kb(kb: &KnowledgeBase) -> Result<KbSnapshot, String> {
    let stats = kb.stats()?;
    let nodes = kb.all_nodes()?;
    let edges = kb.all_edges()?;
    let captured_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    Ok(KbSnapshot {
        format: SNAPSHOT_FORMAT.into(),
        version: SNAPSHOT_VERSION,
        captured_at_ms,
        stats,
        nodes,
        edges,
    })
}

/// 快照落盘 (父目录不存在时自动创建)。
pub fn snapshot_to_file(snap: &KbSnapshot, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建快照目录失败: {}", e))?;
    }
    let json = serde_json::to_string_pretty(snap)
        .map_err(|e| format!("序列化快照失败: {}", e))?;
    std::fs::write(path, json).map_err(|e| format!("写快照文件失败: {}", e))
}

/// 从文件加载快照 (校验格式标识与版本)。
pub fn snapshot_from_file(path: &Path) -> Result<KbSnapshot, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("读快照文件失败: {}", e))?;
    let snap: KbSnapshot = serde_json::from_str(&text)
        .map_err(|e| format!("解析快照失败: {}", e))?;
    if snap.format != SNAPSHOT_FORMAT {
        return Err(format!("非 KB 快照文件 (format='{}')", snap.format));
    }
    if snap.version > SNAPSHOT_VERSION {
        return Err(format!(
            "快照版本 {} 高于当前支持 {}",
            snap.version, SNAPSHOT_VERSION
        ));
    }
    Ok(snap)
}

/// 计算两个快照的差异 (base 为旧, other 为新)。
pub fn diff_snapshots(base: &KbSnapshot, other: &KbSnapshot) -> KbDiff {
    let mut diff = KbDiff::default();

    let base_nodes: HashMap<&str, &KnowledgeNode> =
        base.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
    let other_nodes: HashMap<&str, &KnowledgeNode> =
        other.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    let base_ids: HashSet<&str> = base_nodes.keys().copied().collect();
    let other_ids: HashSet<&str> = other_nodes.keys().copied().collect();

    for id in other_ids.difference(&base_ids) {
        let n = other_nodes.get(*id).copied().unwrap();
        diff.nodes_added.push(diff_node(n, vec!["(new)".into()]));
    }
    for id in base_ids.difference(&other_ids) {
        let n = base_nodes.get(*id).copied().unwrap();
        diff.nodes_removed.push(diff_node(n, vec!["(removed)".into()]));
    }
    for id in base_ids.intersection(&other_ids) {
        let b = base_nodes.get(*id).copied().unwrap();
        let o = other_nodes.get(*id).copied().unwrap();
        let changed = changed_node_fields(b, o);
        if !changed.is_empty() {
            diff.nodes_changed.push(diff_node(o, changed));
        }
    }

    let base_edges: HashMap<&str, &KnowledgeEdge> =
        base.edges.iter().map(|e| (e.id.as_str(), e)).collect();
    let other_edges: HashMap<&str, &KnowledgeEdge> =
        other.edges.iter().map(|e| (e.id.as_str(), e)).collect();

    let base_edge_ids: HashSet<&str> = base_edges.keys().copied().collect();
    let other_edge_ids: HashSet<&str> = other_edges.keys().copied().collect();

    for id in other_edge_ids.difference(&base_edge_ids) {
        diff.edges_added.push(diff_edge(other_edges.get(*id).copied().unwrap()));
    }
    for id in base_edge_ids.difference(&other_edge_ids) {
        diff.edges_removed.push(diff_edge(base_edges.get(*id).copied().unwrap()));
    }

    diff
}

fn diff_node(n: &KnowledgeNode, fields_changed: Vec<String>) -> DiffNode {
    DiffNode {
        id: n.id.clone(),
        node_type: n.node_type.as_str().into(),
        title: n.title.clone(),
        fields_changed,
    }
}

fn diff_edge(e: &KnowledgeEdge) -> DiffEdge {
    DiffEdge {
        id: e.id.clone(),
        source_id: e.source_id.clone(),
        target_id: e.target_id.clone(),
        relation_type: e.relation_type.as_str().into(),
    }
}

fn changed_node_fields(b: &KnowledgeNode, o: &KnowledgeNode) -> Vec<String> {
    let mut changed = Vec::new();
    if b.title != o.title {
        changed.push("title".into());
    }
    if b.summary != o.summary {
        changed.push("summary".into());
    }
    if b.content != o.content {
        changed.push("content".into());
    }
    if b.url != o.url {
        changed.push("url".into());
    }
    if b.domain != o.domain {
        changed.push("domain".into());
    }
    if (b.confidence - o.confidence).abs() > 1e-9 {
        changed.push("confidence".into());
    }
    if (b.importance - o.importance).abs() > 1e-9 {
        changed.push("importance".into());
    }
    if b.node_type != o.node_type {
        changed.push("node_type".into());
    }
    changed
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    fn node(id: &str, title: &str, content: &str) -> KnowledgeNode {
        KnowledgeNode {
            id: id.into(),
            node_type: crate::core::nt_core_kb_types::NodeType::Concept,
            title: title.into(),
            summary: None,
            content: Some(content.into()),
            url: None,
            domain: None,
            language: "en".into(),
            confidence: 1.0,
            importance: 0.5,
            created_at: 0,
            updated_at: 1,
            access_count: 0,
            metadata: None,
            temporal: None,
            supersedes: None,
            source_episode: None,
            parent_id: None,
            depth: 0,
            cluster_id: None,
            recall_weight: 1.0,
        }
    }

    fn edge(id: &str, src: &str, tgt: &str) -> KnowledgeEdge {
        KnowledgeEdge {
            id: id.into(),
            source_id: src.into(),
            target_id: tgt.into(),
            relation_type: crate::core::nt_core_kb_types::RelationType::RelatedTo,
            weight: 1.0,
            description: None,
            created_at: 0,
            metadata: None,
        }
    }

    fn snapshot_of(nodes: Vec<KnowledgeNode>, edges: Vec<KnowledgeEdge>) -> KbSnapshot {
        KbSnapshot {
            format: SNAPSHOT_FORMAT.into(),
            version: SNAPSHOT_VERSION,
            captured_at_ms: 0,
            stats: KnowledgeStats::default(),
            nodes,
            edges,
        }
    }

    #[test]
    fn test_diff_add_remove_change() {
        let base = snapshot_of(
            vec![node("a", "A", "v1"), node("b", "B", "x"), node("c", "C", "z")],
            vec![edge("e1", "a", "b")],
        );
        let mut b2 = node("b", "B", "x");
        b2.content = Some("changed".into());
        let other = snapshot_of(
            vec![node("a", "A", "v2"), b2, node("d", "D", "new")],
            vec![edge("e2", "a", "d")],
        );

        let diff = diff_snapshots(&base, &other);
        assert_eq!(diff.nodes_added.len(), 1, "d added");
        assert_eq!(diff.nodes_added[0].id, "d");
        assert_eq!(diff.nodes_removed.len(), 1, "c removed");
        assert_eq!(diff.nodes_removed[0].id, "c");
        assert_eq!(diff.nodes_changed.len(), 2, "a (content) + b (content) changed");
        let b_change = diff
            .nodes_changed
            .iter()
            .find(|d| d.id == "b")
            .expect("b must be reported changed");
        assert!(b_change.fields_changed.contains(&"content".to_string()));
        assert_eq!(diff.edges_added.len(), 1, "e2 added");
        assert_eq!(diff.edges_added[0].id, "e2");
        assert_eq!(diff.edges_removed.len(), 1, "e1 removed");
        assert_eq!(diff.edges_removed[0].id, "e1");
    }

    #[test]
    fn test_diff_identical_is_empty() {
        let base = snapshot_of(
            vec![node("a", "A", "v1")],
            vec![edge("e1", "a", "a")],
        );
        let other = base.clone();
        assert!(diff_snapshots(&base, &other).is_empty());
    }

    #[test]
    fn test_snapshot_checksum_changes_with_updated_at() {
        let a = snapshot_of(vec![node("a", "A", "v1")], vec![]);
        let mut n = node("a", "A", "v1");
        n.updated_at = 99;
        let b = snapshot_of(vec![n], vec![]);
        assert_ne!(a.checksum(), b.checksum(), "updated_at 变更必须影响指纹");
    }

    #[test]
    fn test_snapshot_file_roundtrip() {
        let dir = std::env::temp_dir().join(format!("nt_kb_snap_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("snap.json");
        let snap = snapshot_of(
            vec![node("a", "A", "v1")],
            vec![edge("e1", "a", "a")],
        );
        snapshot_to_file(&snap, &path).unwrap();
        let loaded = snapshot_from_file(&path).unwrap();
        assert_eq!(loaded.nodes.len(), 1);
        assert_eq!(loaded.edges.len(), 1);
        assert_eq!(loaded.checksum(), snap.checksum());
    }

    #[test]
    fn test_snapshot_from_file_rejects_foreign_format() {
        let dir = std::env::temp_dir().join(format!("nt_kb_snap_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("foreign.json");
        std::fs::write(&path, r#"{"format":"other","version":1}"#).unwrap();
        assert!(snapshot_from_file(&path).is_err());
    }
}