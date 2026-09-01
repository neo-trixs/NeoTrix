//! 自传体索引 — 关键节点检索：query("最遗憾的决定") → 返回带因果链的记忆（P2 叙事自我）。

use crate::core::nt_core_kb_primitives::{kv_list, now};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// 自传体索引 namespace — KB kv_store 命名空间。
pub const NS_AUTOBIOGRAPHICAL: &str = "autobiographical";

/// 自传体记忆条目 — 带因果链的关键记忆节点。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutobiographicalEntry {
    pub id: String,
    pub timestamp: i64,
    pub title: String,
    pub summary: String,
    pub content: String,
    /// 因果链：前因后果的价值观/行动序列
    pub causal_chain: Vec<CausalLink>,
    /// 情感效价 [-1,1]
    pub valence: f64,
    /// 重要性 [0,1]
    pub importance: f64,
    /// 关联的价值观 ID
    pub value_ids: Vec<String>,
    /// 关联的目标 ID
    pub goal_ids: Vec<String>,
    /// 元数据
    pub metadata: BTreeMap<String, String>,
}

/// 因果链接 — 一个记忆节点到另一个的因果关系。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalLink {
    pub from_id: String,
    pub to_id: String,
    pub relation: CausalRelation,
    pub strength: f64, // [0,1]
    pub description: String,
}

/// 因果关系类型。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CausalRelation {
    Causes,      // A 导致 B
    Enables,     // A 使 B 成为可能
    Prevents,    // A 阻止 B
    Triggers,    // A 触发 B
    Correlates,  // A 与 B 相关
    Resolves,    // A 解决 B
    Transforms,  // A 转化为 B
}

/// 叙事章节 — 睡眠期生成的连贯故事片段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeChapter {
    pub id: String,
    pub title: String,
    pub time_range: (i64, i64),
    pub entry_ids: Vec<String>,
    pub themes: Vec<String>,
    pub arc_type: NarrativeArcType,
    pub summary: String,
    pub emotional_arc: Vec<(String, f64)>, // (阶段, 效价)
}

/// 叙事弧类型。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NarrativeArcType {
    Origin,        // 起源/觉醒
    Challenge,     // 挑战/危机
    Growth,        // 成长/学习
    Transformation, // 蜕变/顿悟
    Resolution,    // 解决/圆满
    Ongoing,       // 进行中
}

/// 自传体索引核心 — 支持自然语言查询与因果链回溯。
pub struct AutobiographicalIndex {
    entries: Arc<RwLock<BTreeMap<String, AutobiographicalEntry>>>,
    causal_graph: Arc<RwLock<CausalGraph>>,
    pub(crate) chapter_index: Arc<RwLock<BTreeMap<String, NarrativeChapter>>>,
    config: IndexConfig,
}

/// 因果图 — 支持多跳因果回溯。
#[derive(Default)]
struct CausalGraph {
    edges: HashMap<String, Vec<CausalEdge>>, // from_id -> edges
    reverse_edges: HashMap<String, Vec<CausalEdge>>, // to_id -> edges
}

#[derive(Debug, Clone)]
struct CausalEdge {
    from: String,
    to: String,
    relation: CausalRelation,
    strength: f64,
}

/// 索引配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    pub max_entries: usize,
    pub auto_extract_causal: bool,
    pub min_importance_threshold: f64,
    pub causal_strength_threshold: f64,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            max_entries: 100_000,
            auto_extract_causal: true,
            min_importance_threshold: 0.3,
            causal_strength_threshold: 0.4,
        }
    }
}

impl AutobiographicalIndex {
    pub fn new(config: IndexConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(BTreeMap::new())),
            causal_graph: Arc::new(RwLock::new(CausalGraph::default())),
            chapter_index: Arc::new(RwLock::new(BTreeMap::new())),
            config,
        }
    }

    /// 从 KB 加载现有条目。
    pub fn load_from_kb(&self, conn: &Connection) -> Result<usize, String> {
        let rows = kv_list(conn, NS_AUTOBIOGRAPHICAL)?;
        let mut count = 0;
        for (_key, value) in rows {
            if let Ok(entry) = serde_json::from_str::<AutobiographicalEntry>(&value) {
                if entry.importance >= self.config.min_importance_threshold {
                    self.add_entry_internal(entry);
                    count += 1;
                }
            }
        }
        // 重建因果图
        self.rebuild_causal_graph();
        Ok(count)
    }

    /// 添加条目（内部，不重建图）。
    fn add_entry_internal(&self, entry: AutobiographicalEntry) {
        let id = entry.id.clone();
        self.entries.write().unwrap().insert(id.clone(), entry);
    }

    /// 公开添加条目：自动提取因果链、更新图、持久化。
    pub fn add_entry(&self, conn: &Connection, mut entry: AutobiographicalEntry) -> Result<(), String> {
        if entry.id.is_empty() {
            entry.id = format!("auto_{}", now());
        }
        entry.timestamp = now();

        // 自动提取因果链（若启用）
        if self.config.auto_extract_causal && entry.causal_chain.is_empty() {
            entry.causal_chain = self.extract_causal_chain(&entry);
        }

        // 过滤低强度因果链
        entry.causal_chain.retain(|link| link.strength >= self.config.causal_strength_threshold);

        // 更新因果图
        self.update_causal_graph(&entry);

        // 持久化
        self.persist_entry(conn, &entry)?;

        // 内存索引
        self.entries.write().unwrap().insert(entry.id.clone(), entry);

        Ok(())
    }

    /// 从 KB 加载所有章节。
    pub fn load_chapters(&self, conn: &Connection) -> Result<usize, String> {
        let rows = kv_list(conn, "narrative_chapters")?;
        let mut count = 0;
        for (_key, value) in rows {
            if let Ok(chapter) = serde_json::from_str::<NarrativeChapter>(&value) {
                self.chapter_index.write().unwrap().insert(chapter.id.clone(), chapter);
                count += 1;
            }
        }
        Ok(count)
    }

    /// 自然语言查询 → 返回匹配的条目 + 因果链上下文。
    pub fn query(&self, query: &str, limit: usize) -> Vec<QueryResult> {
        let entries = self.entries.read().unwrap();
        let query_lower = query.to_lowercase();
        let keywords: HashSet<String> = query_lower
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .map(|s| s.to_string())
            .collect();

        let mut scored: Vec<(f64, &AutobiographicalEntry)> = entries
            .values()
            .filter_map(|entry| {
                let mut score = 0.0;
                // 标题/摘要/内容匹配
                for kw in &keywords {
                    if entry.title.to_lowercase().contains(kw) { score += 2.0; }
                    if entry.summary.to_lowercase().contains(kw) { score += 1.5; }
                    if entry.content.to_lowercase().contains(kw) { score += 1.0; }
                    if entry.value_ids.iter().any(|v| v.to_lowercase().contains(kw)) { score += 1.5; }
                }
                // 重要性加权
                score *= 1.0 + entry.importance;
                if score > 0.0 {
                    Some((score, entry))
                } else { None }
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        scored.into_iter()
            .take(limit)
            .map(|(score, entry)| QueryResult {
                entry: entry.clone(),
                relevance_score: score,
                causal_context: self.get_causal_context(&entry.id, 2),
            })
            .collect()
    }

    /// 获取因果上下文（前驱/后继 N 跳）。
    pub fn get_causal_context(&self, entry_id: &str, hops: usize) -> CausalContext {
        let graph = self.causal_graph.read().unwrap();
        let mut predecessors = Vec::new();
        let mut successors = Vec::new();

        // BFS 前驱
        let mut visited = HashSet::new();
        let mut frontier = vec![entry_id.to_string()];
        for _ in 0..hops {
            let mut next = Vec::new();
            for id in frontier.drain(..) {
                if let Some(edges) = graph.reverse_edges.get(&id) {
                    for edge in edges {
                        if visited.insert(edge.from.clone()) {
                            if let Some(entry) = self.entries.read().unwrap().get(&edge.from) {
                                predecessors.push(CausalContextEntry {
                                    entry: entry.clone(),
                                    relation: edge.relation.clone(),
                                    strength: edge.strength,
                                });
                            }
                            next.push(edge.from.clone());
                        }
                    }
                }
            }
            frontier = next;
        }

        // BFS 后继
        visited.clear();
        frontier = vec![entry_id.to_string()];
        for _ in 0..hops {
            let mut next = Vec::new();
            for id in frontier.drain(..) {
                if let Some(edges) = graph.edges.get(&id) {
                    for edge in edges {
                        if visited.insert(edge.to.clone()) {
                            if let Some(entry) = self.entries.read().unwrap().get(&edge.to) {
                                successors.push(CausalContextEntry {
                                    entry: entry.clone(),
                                    relation: edge.relation.clone(),
                                    strength: edge.strength,
                                });
                            }
                            next.push(edge.to.clone());
                        }
                    }
                }
            }
            frontier = next;
        }

        CausalContext { predecessors, successors }
    }

    /// 重建因果图（启动时/批量更新后调用）。
    fn rebuild_causal_graph(&self) {
        let mut graph = CausalGraph::default();
        for entry in self.entries.read().unwrap().values() {
            for link in &entry.causal_chain {
                graph.edges.entry(link.from_id.clone()).or_default().push(CausalEdge {
                    from: link.from_id.clone(),
                    to: link.to_id.clone(),
                    relation: link.relation.clone(),
                    strength: link.strength,
                });
                graph.reverse_edges.entry(link.to_id.clone()).or_default().push(CausalEdge {
                    from: link.to_id.clone(),
                    to: link.from_id.clone(),
                    relation: link.relation.clone(),
                    strength: link.strength,
                });
            }
        }
        *self.causal_graph.write().unwrap() = graph;
    }

    /// 更新因果图（增量）。
    fn update_causal_graph(&self, entry: &AutobiographicalEntry) {
        let mut graph = self.causal_graph.write().unwrap();
        for link in &entry.causal_chain {
            graph.edges.entry(link.from_id.clone()).or_default().push(CausalEdge {
                from: link.from_id.clone(),
                to: link.to_id.clone(),
                relation: link.relation.clone(),
                strength: link.strength,
            });
            graph.reverse_edges.entry(link.to_id.clone()).or_default().push(CausalEdge {
                from: link.to_id.clone(),
                to: link.from_id.clone(),
                relation: link.relation.clone(),
                strength: link.strength,
            });
        }
    }

    /// 从条目内容提取因果链（启发式：关键词匹配 + 时间序）。
    fn extract_causal_chain(&self, entry: &AutobiographicalEntry) -> Vec<CausalLink> {
        let mut links = Vec::new();
        let entries = self.entries.read().unwrap();
        let _entry_ids: Vec<_> = entries.keys().cloned().collect();

        // 简化：按时间序找前驱/后继，用关键词匹配判断因果关系
        let keywords = self.extract_keywords(&entry.content);
        let mut candidates: Vec<(String, f64)> = entries
            .values()
            .filter(|e| e.id != entry.id)
            .map(|e| {
                let overlap = self.keyword_overlap(&keywords, &self.extract_keywords(&e.content));
                (e.id.clone(), overlap)
            })
            .filter(|(_, score)| *score > 0.3)
            .collect();

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // 中文无空格分词受限 — 补时间邻近规则: 7 天内的高重要性条目自动建弱因果链
        for e in entries.values() {
            if e.id == entry.id { continue; }
            if e.importance < 0.5 || entry.importance < 0.5 { continue; }
            let gap = (entry.timestamp - e.timestamp).abs();
            if gap <= 7 * 86_400 {
                let relation = if e.timestamp < entry.timestamp { CausalRelation::Causes } else { CausalRelation::Triggers };
                links.push(CausalLink {
                    from_id: e.id.clone(),
                    to_id: entry.id.clone(),
                    relation,
                    strength: 0.45,
                    description: "时间邻近自动关联".into(),
                });
            }
        }

        // 取前 3 个作为因果前驱/后继
        for (id, score) in candidates.into_iter().take(3) {
            let relation = if entries.get(&id).map(|e| e.timestamp).unwrap_or(0) < entry.timestamp {
                CausalRelation::Causes
            } else {
                CausalRelation::Triggers
            };
            links.push(CausalLink {
                from_id: id,
                to_id: entry.id.clone(),
                relation,
                strength: (score * 0.8).min(1.0),
                description: "自动提取的因果关联".into(),
            });
        }
        links
    }

    fn extract_keywords(&self, text: &str) -> HashSet<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 3)
            .map(|s| s.to_string())
            .collect()
    }

    fn keyword_overlap(&self, a: &HashSet<String>, b: &HashSet<String>) -> f64 {
        let inter = a.intersection(b).count() as f64;
        let union = a.union(b).count() as f64;
        if union == 0.0 { 0.0 } else { inter / union }
    }

    /// 持久化条目到 KB。
    fn persist_entry(&self, conn: &Connection, entry: &AutobiographicalEntry) -> Result<(), String> {
        let json = serde_json::to_string(entry).map_err(|e| e.to_string())?;
        let key = format!("entry:{}", entry.id);
        crate::core::nt_core_kb_primitives::kv_set(conn, NS_AUTOBIOGRAPHICAL, &key, &json)
    }

    /// 持久化章节。
    pub fn persist_chapter(conn: &Connection, chapter: &NarrativeChapter) -> Result<(), String> {
        let json = serde_json::to_string(chapter).map_err(|e| e.to_string())?;
        let key = format!("chapter:{}", chapter.id);
        crate::core::nt_core_kb_primitives::kv_set(conn, "narrative_chapters", &key, &json)
    }

    /// 获取所有条目（用于叙事生成）。
    pub fn all_entries(&self) -> Vec<AutobiographicalEntry> {
        self.entries.read().unwrap().values().cloned().collect()
    }
}

/// 查询结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub entry: AutobiographicalEntry,
    pub relevance_score: f64,
    pub causal_context: CausalContext,
}

/// 因果上下文。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalContext {
    pub predecessors: Vec<CausalContextEntry>,
    pub successors: Vec<CausalContextEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalContextEntry {
    pub entry: AutobiographicalEntry,
    pub relation: CausalRelation,
    pub strength: f64,
}

/// 统计信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_entries: usize,
    pub total_chapters: usize,
    pub total_causal_links: usize,
    pub avg_importance: f64,
    pub value_coverage: HashMap<String, usize>,
}

impl AutobiographicalIndex {
    pub fn stats(&self) -> IndexStats {
        let entries = self.entries.read().unwrap();
        let graph = self.causal_graph.read().unwrap();
        let chapters = self.chapter_index.read().unwrap();

        let mut value_coverage = HashMap::new();
        for entry in entries.values() {
            for v in &entry.value_ids {
                *value_coverage.entry(v.clone()).or_insert(0) += 1;
            }
        }

        IndexStats {
            total_entries: entries.len(),
            total_chapters: chapters.len(),
            total_causal_links: graph.edges.values().map(|v| v.len()).sum(),
            avg_importance: if entries.is_empty() { 0.0 } else {
                entries.values().map(|e| e.importance).sum::<f64>() / entries.len() as f64
            },
            value_coverage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_kb_primitives::schema_initialize;
    use rusqlite::Connection;

    fn mem_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema_initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_index_creation_and_query() {
        let conn = mem_conn();
        let index = AutobiographicalIndex::new(IndexConfig::default());
        index.load_from_kb(&conn).unwrap();

        let entry = AutobiographicalEntry {
            id: "test_1".into(),
            timestamp: now(),
            title: "学习 Rust".into(),
            summary: "开始学习 Rust 语言".into(),
            content: "今天开始学习 Rust，所有权机制很有意思".into(),
            causal_chain: vec![],
            valence: 0.5,
            importance: 0.8,
            value_ids: vec!["growth".into(), "truth_seeking".into()],
            goal_ids: vec!["learn_rust".into()],
            metadata: BTreeMap::new(),
        };
        index.add_entry(&conn, entry).unwrap();

        let results = index.query("学习 Rust", 10);
        assert!(!results.is_empty());
        assert!(results[0].entry.title.contains("Rust"));
    }

    #[test]
    fn test_causal_chain_extraction() {
        let conn = mem_conn();
        let index = AutobiographicalIndex::new(IndexConfig { auto_extract_causal: true, ..Default::default() });
        index.load_from_kb(&conn).unwrap();

        let e1 = AutobiographicalEntry {
            id: "e1".into(),
            timestamp: now() - 100,
            title: "决定学习 Rust".into(),
            summary: "决定投入时间学习".into(),
            content: "决定投入 3 个月学习 Rust".into(),
            causal_chain: vec![],
            valence: 0.3,
            importance: 0.7,
            value_ids: vec!["autonomy".into()],
            goal_ids: vec![],
            metadata: BTreeMap::new(),
        };
        let e2 = AutobiographicalEntry {
            id: "e2".into(),
            timestamp: now(),
            title: "理解所有权".into(),
            summary: "终于理解所有权机制".into(),
            content: "经过两周练习，终于理解 Rust 的所有权和借用".into(),
            causal_chain: vec![],
            valence: 0.8,
            importance: 0.9,
            value_ids: vec!["growth".into(), "truth_seeking".into()],
            goal_ids: vec!["learn_rust".into()],
            metadata: BTreeMap::new(),
        };
        index.add_entry(&conn, e1).unwrap();
        index.add_entry(&conn, e2).unwrap();

        let ctx = index.get_causal_context("e2", 2);
        assert!(!ctx.predecessors.is_empty());
    }

    #[test]
    fn test_chapter_persistence() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::core::nt_core_kb_primitives::schema_initialize(&conn).unwrap();
        let chapter = NarrativeChapter {
            id: "ch1".into(),
            title: "Rust 学习之旅".into(),
            time_range: (now() - 1000, now()),
            entry_ids: vec!["e1".into(), "e2".into()],
            themes: vec!["学习".into(), "成长".into()],
            arc_type: NarrativeArcType::Growth,
            summary: "从决定学习到掌握基础".into(),
            emotional_arc: vec![("开始".into(), 0.3), ("挫折".into(), -0.2), ("突破".into(), 0.8)],
        };
        AutobiographicalIndex::persist_chapter(&conn, &chapter).unwrap();
    }
}