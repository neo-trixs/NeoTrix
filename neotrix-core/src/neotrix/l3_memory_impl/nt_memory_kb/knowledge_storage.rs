//! knowledge_storage — NeoTrix Knowledge Engine v2 高性能增量存储引擎
//!
//! 忠实移植自已退役的 `scripts/knowledge_storage.py`。
//! 参考: LevelDB (append-only log + periodic compact) / SQLite WAL / LMDB。
//! 核心策略:
//!   1. 内存优先 — 所有写入走内存，定期刷盘
//!   2. 增量写入 — 只在新增时 append .jsonl，不重写全文件
//!   3. 懒惰 compact — 仅当碎片率 >30% 或调用 compact()
//!   4. 零拷贝读取 — 读取避免 json 全量解析

use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

fn unix_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 增量存储引擎 — append-only journal + 全量快照 (类 LevelDB compaction)。
pub struct KnowledgeStorage {
    journal_path: PathBuf,
    max_memory: usize,
    entries: HashMap<String, Value>,
    dirty: bool,
    last_compact_size: usize,
    /// 快照文件名后缀 (`.snap`)，加载与 compact 共用。
    snap_path: PathBuf,
    log_counter: AtomicU32,
}

impl KnowledgeStorage {
    /// 打开存储: 优先全量快照 + 回放增量日志。
    /// `path` 为 `compact_path` (全量快照主路径), journal 为同名前缀的 `.jsonl`。
    pub fn open(path: impl AsRef<Path>, max_memory_entries: usize) -> std::io::Result<Self> {
        let compact_path = path.as_ref().to_path_buf();
        let journal_path = compact_path.with_extension("jsonl");
        let snap_path = PathBuf::from(format!("{}.snap", compact_path.display()));
        let mut store = Self {
            journal_path,
            max_memory: max_memory_entries,
            entries: HashMap::new(),
            dirty: false,
            last_compact_size: 0,
            snap_path,
            log_counter: AtomicU32::new(0),
        };
        store.load()?;
        Ok(store)
    }

    fn load(&mut self) -> std::io::Result<()> {
        // 1. 全量快照 (若有)
        if self.snap_path.exists() {
            let raw = fs::read_to_string(&self.snap_path)?;
            if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                if let Some(map) = v.get("entries").and_then(Value::as_object) {
                    for (k, val) in map {
                        self.entries.insert(k.clone(), val.clone());
                    }
                    self.last_compact_size = self.entries.len();
                }
            }
        }

        // 2. 回放增量日志
        if self.journal_path.exists() {
            let file = File::open(&self.journal_path)?;
            for line in BufReader::new(file).lines() {
                let line = line?;
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Ok(entry) = serde_json::from_str::<Value>(line) {
                    if let Some(eid) = entry.get("id").and_then(Value::as_str) {
                        self.entries.insert(eid.to_string(), entry);
                    }
                }
            }
        }

        let frag = self.fragmentation_ratio();
        log::info!(
            "[store] loaded {} entries (frag={:.1}%)",
            self.entries.len(),
            frag * 100.0
        );
        Ok(())
    }

    pub fn get(&self, eid: &str) -> Option<&Value> {
        self.entries.get(eid)
    }

    /// 增量写入内存 + append journal。返回是否为新条目。
    pub fn put(&mut self, mut entry: Value) -> std::io::Result<bool> {
        let eid = entry
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        entry["id"] = Value::String(eid.clone());
        entry["updated_at"] = json!(unix_ts());

        let is_new = !self.entries.contains_key(&eid);
        self.entries.insert(eid, entry.clone());
        self.dirty = true;

        self.append_journal(&entry)?;

        // 触发 compact
        if (self.last_compact_size > 0
            && self.entries.len() > self.last_compact_size + self.max_memory / 2)
            || self.fragmentation_ratio() > 0.3
        {
            self.compact()?;
        }

        Ok(is_new)
    }

    /// 批量写入 — 单次 append journal。
    pub fn put_batch(&mut self, entries: Vec<Value>) -> std::io::Result<usize> {
        let mut added = 0usize;
        let mut framed: Vec<Value> = Vec::with_capacity(entries.len());
        for entry in entries {
            let eid = entry
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            if !self.entries.contains_key(&eid) {
                added += 1;
            }
            let mut entry = entry;
            entry["id"] = Value::String(eid.clone());
            entry["updated_at"] = json!(unix_ts());
            self.entries.insert(eid, entry.clone());
            framed.push(entry);
        }

        if added > 0 {
            self.dirty = true;
            self.append_journal_lines(&framed)?;
            if self.fragmentation_ratio() > 0.3 {
                self.compact()?;
            }
        }

        Ok(added)
    }

    /// 整理: 全量快照 + 清空 journal (类 LevelDB compaction)。
    pub fn compact(&mut self) -> std::io::Result<()> {
        if !self.dirty {
            return Ok(());
        }

        let snap_tmp = PathBuf::from(format!("{}.tmp", self.snap_path.display()));

        // 序列化 entries 为 Map 保持与 Python 的 dict 一致
        let mut entries_map = Map::new();
        for (k, v) in self.entries.iter() {
            entries_map.insert(k.clone(), v.clone());
        }
        let data = json!({
            "entries": Value::Object(entries_map),
            "compacted_at": unix_ts(),
            "entry_count": self.entries.len(),
        });

        {
            let f = File::create(&snap_tmp)?;
            let mut w = BufWriter::new(f);
            serde_json::to_writer(&mut w, &data)?;
            w.flush()?;
        }
        fs::rename(&snap_tmp, &self.snap_path)?;

        if self.journal_path.exists() {
            fs::remove_file(&self.journal_path)?;
        }

        self.last_compact_size = self.entries.len();
        self.dirty = false;

        log::info!("[store] compacted {} entries", self.entries.len());
        Ok(())
    }

    /// 确保所有数据持久化 (fsync)。
    pub fn flush(&mut self) -> std::io::Result<()> {
        if self.dirty {
            self.compact()?;
        }
        let f = File::open(&self.snap_path)?;
        f.sync_all()?;
        Ok(())
    }

    /// 碎片率: journal 体积 vs 快照条目数估算。
    pub fn fragmentation_ratio(&self) -> f64 {
        let journal_size = match fs::metadata(&self.journal_path) {
            Ok(m) => m.len() as f64,
            Err(_) => 0.0,
        };
        if self.last_compact_size == 0 {
            return 0.0;
        }
        journal_size / (self.last_compact_size as f64 * 2000.0).max(1.0)
    }

    /// 内存全文搜索 (避免每次读盘)。分数: title +5 / tags +3 / summary +1 / body 命中 *0.5。
    pub fn search(&self, keyword: &str, limit: usize) -> Vec<Value> {
        let kw = keyword.to_ascii_lowercase();
        let mut scored: Vec<(f64, &Value)> = Vec::new();

        for entry in self.entries.values() {
            let mut score = 0.0f64;
            let title = entry.get("title").and_then(Value::as_str).unwrap_or("").to_ascii_lowercase();
            let body = entry.get("body").and_then(Value::as_str).unwrap_or("").to_ascii_lowercase();
            let summary = entry.get("summary").and_then(Value::as_str).unwrap_or("").to_ascii_lowercase();
            let tags: String = entry
                .get("tags")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(" ")
                        .to_ascii_lowercase()
                })
                .unwrap_or_default();

            if title.contains(&kw) {
                score += 5.0;
            }
            if tags.contains(&kw) {
                score += 3.0;
            }
            if summary.contains(&kw) {
                score += 1.0;
            }
            score += body.matches(&kw).count() as f64 * 0.5;

            if score > 0.0 {
                scored.push((score, entry));
            }
        }

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(limit).map(|(_, e)| e.clone()).collect()
    }

    pub fn stats(&self) -> Value {
        let memory_estimate = self
            .entries
            .values()
            .map(|v| serde_json::to_string(v).map(|s| s.len() as f64).unwrap_or(0.0))
            .sum::<f64>()
            / 1024.0
            / 1024.0;

        json!({
            "entries": self.entries.len(),
            "memory_estimate_mb": memory_estimate,
            "fragmentation": self.fragmentation_ratio(),
            "journal_exists": self.journal_path.exists(),
            "snapshot_exists": self.snap_path.exists(),
        })
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    fn append_journal(&mut self, entry: &Value) -> std::io::Result<()> {
        self.append_journal_lines(std::slice::from_ref(entry))
    }

    fn append_journal_lines(&mut self, entries: &[Value]) -> std::io::Result<()> {
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.journal_path)?;
        for e in entries {
            let line = serde_json::to_string(e)?;
            writeln!(f, "{line}")?;
        }
        Ok(())
    }

    /// 提供运行时递增序号 (供 CLI 用, 与 Python 主模块解耦)。
    pub fn next_log_seq(&self) -> u32 {
        self.log_counter.fetch_add(1, Ordering::Relaxed)
    }
}

// ====== Context Graph (P14) ======
// 吸收 semantica Context Graph + KG + 因果推理 + decision provenance。
// 轻量图建模: 节点/边上下文切片 + 溯源链, 作为 KnowledgeStorage 之上的一层 (R-P42)。

/// 图节点 — 上下文切片 (knowledge entry 的轻量投影)。
#[derive(Debug, Clone, PartialEq)]
pub struct GraphNode {
    pub id: String,
    pub kind: String,
    pub content: String,
}

/// 图边 — 有向关系 + 权重 (权重钳制在 [0,1], R-P6)。
#[derive(Debug, Clone, PartialEq)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub weight: f64,
}

/// 溯源步 — decision provenance 链的节点。
#[derive(Debug, Clone, PartialEq)]
pub struct ProvenanceStep {
    pub node_id: String,
    pub action: String,
    pub ts: u64,
}

/// 上下文图 — 节点/边/溯源链, 支撑因果推理与决策溯源。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ContextGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub provenance: Vec<ProvenanceStep>,
}

impl ContextGraph {
    /// 添加节点; id 重复则替换旧节点。
    pub fn add_node(&mut self, node: GraphNode) {
        if let Some(existing) = self.nodes.iter_mut().find(|n| n.id == node.id) {
            *existing = node;
        } else {
            self.nodes.push(node);
        }
    }

    /// 连接两端节点 (必须均已存在), 否则 Err("missing node: X")。
    /// 权重钳制到 [0,1] (R-P6)。
    pub fn connect(&mut self, from: &str, to: &str, relation: &str, weight: f64) -> Result<(), String> {
        let has = |id: &str| self.nodes.iter().any(|n| n.id == id);
        if !has(from) {
            return Err(format!("missing node: {from}"));
        }
        if !has(to) {
            return Err(format!("missing node: {to}"));
        }
        self.edges.push(GraphEdge {
            from: from.to_string(),
            to: to.to_string(),
            relation: relation.to_string(),
            weight: weight.max(0.0).min(1.0),
        });
        Ok(())
    }

    /// 广度优先收集 depth 层内的邻居节点 (不包含 focus 自身)。
    pub fn neighborhood(&self, id: &str, depth: usize) -> Vec<&GraphNode> {
        if !self.nodes.iter().any(|n| n.id == id) || depth == 0 {
            return Vec::new();
        }
        let mut visited = std::collections::HashSet::new();
        visited.insert(id.to_string());
        let mut frontier: Vec<String> = vec![id.to_string()];
        let mut result: Vec<&GraphNode> = Vec::new();

        for _ in 0..depth {
            let mut next: Vec<String> = Vec::new();
            for current in &frontier {
                for edge in &self.edges {
                    let neighbor = if &edge.from == current {
                        Some(&edge.to)
                    } else if &edge.to == current {
                        Some(&edge.from)
                    } else {
                        None
                    };
                    if let Some(neighbor) = neighbor {
                        if visited.insert(neighbor.clone()) {
                            if let Some(node) = self.nodes.iter().find(|n| &n.id == neighbor) {
                                result.push(node);
                            }
                            next.push(neighbor.clone());
                        }
                    }
                }
            }
            frontier = next;
            if frontier.is_empty() {
                break;
            }
        }
        result
    }

    /// focus 节点 1 跳邻接的内容切片 (content 截断 60 字符) — decision context。
    pub fn decision_context(&self, focus: &str) -> Vec<&str> {
        if !self.nodes.iter().any(|n| n.id == focus) {
            return Vec::new();
        }
        let mut seen = std::collections::HashSet::new();
        seen.insert(focus.to_string());
        let mut result: Vec<&str> = Vec::new();
        for edge in &self.edges {
            let neighbor = if edge.from == focus && edge.to != focus {
                Some(&edge.to)
            } else if edge.to == focus && edge.from != focus {
                Some(&edge.from)
            } else {
                None
            };
            if let Some(neighbor) = neighbor {
                if !seen.insert(neighbor.clone()) {
                    continue;
                }
                if let Some(node) = self.nodes.iter().find(|n| &n.id == neighbor) {
                    let truncated: &str = if node.content.char_indices().count() > 60 {
                        let mut end = 0;
                        for (i, (idx, _)) in node.content.char_indices().enumerate() {
                            if i == 60 {
                                break;
                            }
                            end = idx + 1;
                        }
                        &node.content[..end.min(node.content.len())]
                    } else {
                        &node.content
                    };
                    result.push(truncated);
                }
            }
        }
        result
    }

    /// 过滤与 node_id 相关的溯源步。
    pub fn provenance_chain(&self, node_id: &str) -> Vec<&ProvenanceStep> {
        self.provenance
            .iter()
            .filter(|s| s.node_id == node_id)
            .collect()
    }

    /// 追加溯源步 (decision provenance)。
    pub fn trace(&mut self, node_id: &str, action: &str, ts: u64) {
        self.provenance.push(ProvenanceStep {
            node_id: node_id.to_string(),
            action: action.to_string(),
            ts,
        });
    }

    /// 边数。
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

/// SelfTest (T1): ContextGraph 能力自检。
pub struct ContextGraphSelfTest;

impl crate::core::nt_core_self_test::SelfTest for ContextGraphSelfTest {
    fn name(&self) -> &str {
        "nt_memory_kb_context_graph"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut graph = ContextGraph::default();
        graph.add_node(GraphNode {
            id: "n1".into(),
            kind: "fact".into(),
            content: "alpha".into(),
        });
        graph.add_node(GraphNode {
            id: "n2".into(),
            kind: "fact".into(),
            content: "beta".into(),
        });
        if let Err(e) = graph.connect("n1", "n2", "derives", 0.9) {
            return Err(vec![format!("connect should succeed: {e}")]);
        }
        if graph.edge_count() != 1 {
            return Err(vec!["edge_count should be 1".into()]);
        }
        let ctx = graph.decision_context("n1");
        if ctx.len() != 1 || ctx[0] != "beta" {
            return Err(vec!["decision_context should expose neighbor content".into()]);
        }
        graph.trace("n1", "decided", 1);
        if graph.provenance_chain("n1").len() != 1 {
            return Err(vec!["provenance_chain should filter by node".into()]);
        }
        Ok(())
    }
}

// ====== 迁移工具 ======

/// 从旧格式 (含 `entries` map 的 JSON) 迁移到新存储引擎。
/// 返回迁移条数。
pub fn migrate_from_json(source: &Path, target: &Path, max_memory: usize) -> std::io::Result<usize> {
    let raw = fs::read_to_string(source)?;
    let data: Value = serde_json::from_str(&raw)?;
    let entries = data
        .get("entries")
        .and_then(Value::as_object)
        .map(|m| m.values().cloned().collect())
        .unwrap_or_default();

    let mut store = KnowledgeStorage::open(target, max_memory)?;
    let added = store.put_batch(entries)?;
    store.compact()?;
    Ok(added)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn tmp_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!("nt_ks_test_{}_{}", std::process::id(), n))
    }

    #[test]
    fn test_put_get_roundtrip() {
        let dir = tmp_dir();
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("store.json");

        let mut store = KnowledgeStorage::open(&path, 500).unwrap();
        let entry = json!({"id": "e1", "title": "Hello", "body": "world content here"});
        let is_new = store.put(entry.clone()).unwrap();
        assert!(is_new, "首次写入应为新条目");

        let got = store.get("e1").unwrap();
        assert_eq!(got["title"], "Hello");
        assert!(got["updated_at"].is_i64(), "应有 updated_at 时间戳");
        drop(store);

        // 重开应能回放
        let store2 = KnowledgeStorage::open(&path, 500).unwrap();
        assert_eq!(store2.get("e1").unwrap()["title"], "Hello");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_put_idempotent() {
        let dir = tmp_dir();
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("store.json");
        let mut store = KnowledgeStorage::open(&path, 500).unwrap();
        let entry = json!({"id": "dup", "title": "A"});
        store.put(entry.clone()).unwrap();
        let is_new = store.put(entry).unwrap();
        assert!(!is_new, "重复 id 不应算新增");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_put_batch_counts_only_new() {
        let dir = tmp_dir();
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("store.json");
        let mut store = KnowledgeStorage::open(&path, 500).unwrap();
        let batch = vec![json!({"id": "a", "title": "A"}), json!({"id": "b", "title": "B"})];
        assert_eq!(store.put_batch(batch).unwrap(), 2);
        let again = vec![json!({"id": "a", "title": "A2"}), json!({"id": "c", "title": "C"})];
        assert_eq!(store.put_batch(again).unwrap(), 1, "仅 c 是新增");
        assert_eq!(store.entry_count(), 3);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_compact_writes_snapshot_clears_journal() {
        let dir = tmp_dir();
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("store.json");
        let mut store = KnowledgeStorage::open(&path, 500).unwrap();
        store
            .put_batch((0..10).map(|i| json!({"id": format!("e{i}"), "title": format!("T{i}")})).collect())
            .unwrap();
        assert!(store.journal_path.exists());
        store.compact().unwrap();
        assert!(store.snap_path.exists(), "快照应生成");
        assert!(!store.journal_path.exists(), "journal 应清空");
        drop(store);

        let reloaded = KnowledgeStorage::open(&path, 500).unwrap();
        assert_eq!(reloaded.entry_count(), 10);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_search_ranking() {
        let dir = tmp_dir();
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("store.json");
        let mut store = KnowledgeStorage::open(&path, 500).unwrap();
        store
            .put_batch(vec![
                json!({"id": "t1", "title": "Physics 101", "body": "intro", "summary": ""}),
                json!({"id": "t2", "title": "Math", "body": "physics physics physics", "summary": ""}),
                json!({"id": "t3", "title": "Nothing", "body": "zzz", "summary": ""}),
            ])
            .unwrap();
        let results = store.search("physics", 10);
        assert_eq!(results.len(), 2);
        // title 命中 (t1) 应排在 body 多次命中 (t2) 之前
        assert_eq!(results[0]["id"], "t1");
        assert_eq!(results[1]["id"], "t2");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_flush_and_reload() {
        let dir = tmp_dir();
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("store.json");
        let mut store = KnowledgeStorage::open(&path, 500).unwrap();
        store.put(json!({"id": "x", "title": "X"})).unwrap();
        store.flush().unwrap();
        assert!(store.snap_path.exists());
        drop(store);
        let reloaded = KnowledgeStorage::open(&path, 500).unwrap();
        assert_eq!(reloaded.entry_count(), 1);
        fs::remove_dir_all(&dir).unwrap();
    }

    // ── P14 ContextGraph ──

    fn sample_graph() -> ContextGraph {
        let mut g = ContextGraph::default();
        g.add_node(GraphNode { id: "a".into(), kind: "fact".into(), content: "alpha".into() });
        g.add_node(GraphNode { id: "b".into(), kind: "fact".into(), content: "beta".into() });
        g.add_node(GraphNode { id: "c".into(), kind: "fact".into(), content: "gamma".into() });
        g.add_node(GraphNode { id: "d".into(), kind: "fact".into(), content: "delta".into() });
        g.connect("a", "b", "relates", 0.8).unwrap();
        g.connect("b", "c", "relates", 0.7).unwrap();
        g.connect("c", "d", "relates", 0.6).unwrap();
        g
    }

    #[test]
    fn test_graph_add_node_replaces_duplicate_id() {
        let mut g = ContextGraph::default();
        g.add_node(GraphNode { id: "a".into(), kind: "fact".into(), content: "old".into() });
        g.add_node(GraphNode { id: "a".into(), kind: "fact".into(), content: "new".into() });
        assert_eq!(g.nodes.len(), 1, "重复 id 应替换而非追加");
        assert_eq!(g.nodes[0].content, "new");
    }

    #[test]
    fn test_graph_connect_missing_node_errors() {
        let mut g = sample_graph();
        let err = g.connect("a", "ghost", "relates", 0.5).unwrap_err();
        assert_eq!(err, "missing node: ghost");
        let err2 = g.connect("ghost", "a", "relates", 0.5).unwrap_err();
        assert_eq!(err2, "missing node: ghost");
        assert_eq!(g.edge_count(), 3, "失败连接不应产生边");
    }

    #[test]
    fn test_graph_connect_clamps_weight() {
        let mut g = sample_graph();
        g.connect("a", "d", "chain", 1.7).unwrap();
        assert!(g.edges.last().unwrap().weight <= 1.0);
        g.connect("d", "a", "back", -0.5).unwrap();
        assert!(g.edges.last().unwrap().weight >= 0.0);
    }

    #[test]
    fn test_graph_neighborhood_depth() {
        let g = sample_graph();
        let d1 = g.neighborhood("a", 1);
        assert_eq!(d1.len(), 1, "a 的 1 跳邻居应为 b");
        assert_eq!(d1[0].id, "b");
        let d2 = g.neighborhood("a", 2);
        assert_eq!(d2.len(), 2, "a 的 2 跳邻居应为 b,c");
        let d3 = g.neighborhood("a", 3);
        assert_eq!(d3.len(), 3, "a 的 3 跳邻居应为 b,c,d");
    }

    #[test]
    fn test_graph_decision_context_truncates_content() {
        let mut g = ContextGraph::default();
        g.add_node(GraphNode { id: "focus".into(), kind: "fact".into(), content: "focus body".into() });
        let long = "x".repeat(120);
        g.add_node(GraphNode { id: "nbr".into(), kind: "fact".into(), content: long.clone() });
        g.connect("focus", "nbr", "relates", 0.5).unwrap();
        let ctx = g.decision_context("focus");
        assert_eq!(ctx.len(), 1, "应返回 1 跳邻接内容切片");
        assert_eq!(ctx[0].chars().count(), 60, "内容应截断到 60 字符");
        let empty = g.decision_context("ghost");
        assert!(empty.is_empty(), "缺失 focus 应返回空");
    }

    #[test]
    fn test_graph_provenance_chain_filters() {
        let mut g = ContextGraph::default();
        g.add_node(GraphNode { id: "a".into(), kind: "fact".into(), content: "alpha".into() });
        g.add_node(GraphNode { id: "b".into(), kind: "fact".into(), content: "beta".into() });
        g.trace("a", "created", 1);
        g.trace("a", "decided", 2);
        g.trace("b", "created", 3);
        let chain = g.provenance_chain("a");
        assert_eq!(chain.len(), 2, "应仅过滤出 a 相关 steps");
        assert_eq!(chain[0].action, "created");
        assert_eq!(chain[1].action, "decided");
        assert_eq!(g.provenance_chain("ghost").len(), 0);
    }

    #[test]
    fn test_graph_selftest_runs() {
        use crate::core::nt_core_self_test::SelfTest;
        let t = ContextGraphSelfTest;
        assert!(t.self_test().is_ok());
    }
}
