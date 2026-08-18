//! KB 最短路径数据管道 — 服务意识体 (读端) + 吸收知识 (写端)
//!
//! 意识体内部调用无需走 HTTP/CLI，直接调本模块两个入口：
//! - `absorb_core` — 写端：对话/文件 → 蒸馏分类 → 节点 + 域枢纽 BelongsTo 边 + FTS 同步
//! - `serve_core`  — 读端：GWT 意图路由 → 通道检索 → 结果 + 图最短路径溯源
//!
//! 设计原则 (数据所有权分离 v1):
//! - 意识体学到的知识 → 本模块写入 KB (experience/domain_nt_*)
//! - 用户原始数据 → 用户对话目录，不混入 KB 文件节点
//!
//! 关键陷阱 (固化为 rule, cycle 1147):
//! - `insert_node` 不写 FTS (无触发器) → 本模块手动 `nodes_fts` 双写
//! - FTS JOIN 必须 `nodes_fts.rowid = nodes.rowid` (整数), 用 `n.id` (TEXT) 必错
//! - nodes 表 `updated_at/tier/data_tier` NOT NULL 无默认 → 插入必须补全

use super::nt_memory_types::*;
use super::nt_memory_store;
use super::KnowledgeBase;

use crate::core::nt_core_kb_types::{NodeType, RelationType};
use super::nt_memory_gwt_router::RetrievalChannel;

use serde::{Deserialize, Serialize};

/// 写端输入 — 单个知识条目。意识体从对话/文件中蒸馏后构造。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorbEntry {
    pub title: String,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub node_type: String,
    pub domain: Option<String>,
    pub url: Option<String>,
    pub language: Option<String>,
    pub importance: Option<f64>,
    /// 附加关系: (relation, target_id) — 挂图时自动 upsert
    pub relations: Vec<RelationSpec>,
}

/// 附加关系声明 — 目标用 id 或 (title, node_type) 定位
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationSpec {
    pub relation: String,
    pub target_id: Option<String>,
    pub target_title: Option<String>,
    pub target_type: Option<String>,
    pub weight: Option<f64>,
}

/// 写端输出 — 吸收报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorbReport {
    pub node_id: String,
    pub created: bool,
    pub hub_id: Option<String>,
    pub hub_linked: bool,
    pub fts_synced: bool,
    pub edges_added: usize,
}

/// 读端输出 — 一次完整服务结果 (意图 + 检索 + 图溯源)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServeResult {
    pub query: String,
    pub channel: String,
    pub confidence: f64,
    pub resonance: [f64; 5],
    pub results: Vec<SearchResult>,
    pub graph_path: Option<GraphPath>,
}

/// 域 → 枢纽 id 归一化 (单一事实源: kb_domain_hub_nt-*)
fn hub_id_for(domain: &str) -> String {
    let d = domain.trim().to_ascii_lowercase();
    let core = d
        .strip_prefix("nt-")
        .or_else(|| d.strip_prefix("nt_"))
        .unwrap_or(&d);
    format!("kb_domain_hub_nt-{core}")
}

/// 幂等创建域枢纽节点 (nodes 全字段, 含 FTS 双写)
fn ensure_hub(conn: &rusqlite::Connection, domain: &str) -> Result<String, String> {
    let hid = hub_id_for(domain);
    if nt_memory_store::get_node(conn, &hid).map_err(|e| e.to_string())?.is_some() {
        return Ok(hid);
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let hub = KnowledgeNode {
        id: hid.clone(),
        node_type: NodeType::Concept,
        title: format!("KB-{}", domain.to_ascii_uppercase()),
        summary: Some(format!("知识大脑域总纲枢纽: {domain}")),
        content: None,
        url: None,
        domain: Some(domain.to_string()),
        language: "en".into(),
        confidence: 1.0,
        importance: 0.9,
        created_at: ts,
        updated_at: ts,
        access_count: 0,
        metadata: Some(serde_json::json!({"tier": "hub"})),
        temporal: None,
        supersedes: None,
        source_episode: None,
    };
    let tx = conn.unchecked_transaction().map_err(|e| format!("ensure_hub tx: {}", e))?;
    // insert_node_rows 内部已含 nodes + nodes_fts 双写 (last_insert_rowid), 勿重复写 FTS
    nt_memory_store::insert_node_rows(&tx, &hub).map_err(|e| format!("ensure_hub insert_node_rows: {}", e))?;
    tx.commit().map_err(|e| format!("ensure_hub commit: {}", e))?;
    log::info!("[pipeline] hub 创建: {hid}");
    Ok(hid)
}

/// 手动补 nodes_fts 双写 — `insert_node` 不写 FTS (无触发器)。
fn sync_fts(conn: &rusqlite::Connection, node: &KnowledgeNode) -> Result<(), String> {
    // nodes_fts.rowid == nodes 整数 rowid
    let rowid: i64 = conn
        .query_row("SELECT rowid FROM nodes WHERE id=?1", rusqlite::params![node.id], |r| r.get(0))
        .map_err(|e| format!("resolve rowid for {}: {}", node.id, e))?;
    conn.execute(
        "INSERT OR REPLACE INTO nodes_fts(rowid, title, summary, content, domain) VALUES(?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![rowid, node.title, node.summary, node.content, node.domain],
    )
    .map_err(|e| format!("fts sync {}: {}", node.id, e))?;
    Ok(())
}

/// 从查询提取实体 token 用于图通道实体检索:
/// 大写 ASCII 连续段 (E8/GWT/SEAL) + CJK 连续段 (注意力路由), 过滤 1 字符噪声。
fn extract_entities(query: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur: Vec<char> = Vec::new();
    let mut mode = 0u8; // 0=idle, 1=upper-ascii, 2=cjk
    for ch in query.chars() {
        if ch.is_ascii_uppercase() || ch.is_ascii_digit() {
            if mode == 2 {
                if cur.len() >= 2 {
                    tokens.push(cur.iter().collect::<String>());
                }
                cur.clear();
            }
            mode = 1;
            cur.push(ch);
        } else if ('\u{4e00}'..='\u{9fff}').contains(&ch) {
            if mode == 1 {
                if cur.len() >= 2 {
                    tokens.push(cur.iter().collect::<String>());
                }
                cur.clear();
            }
            mode = 2;
            cur.push(ch);
        } else {
            if cur.len() >= 2 {
                tokens.push(cur.iter().collect::<String>());
            }
            cur.clear();
            mode = 0;
        }
    }
    if cur.len() >= 2 {
        tokens.push(cur.iter().collect::<String>());
    }
    // 去重, 过滤纯数字/过短
    let mut seen = std::collections::HashSet::new();
    tokens.into_iter().filter(|t| t.len() >= 2 && seen.insert(t.clone())).collect()
}

impl KnowledgeBase {
    /// 写端管道 — 吸收一个知识条目到 KB 最短路径。
    /// 幂等 (url 或 title+node_type 已存在则跳过), 自动挂域枢纽边 + FTS 同步。
    pub fn absorb_core(&self, entry: &AbsorbEntry) -> Result<AbsorbReport, String> {
        let node_type = NodeType::from_str(&entry.node_type);
        let conn = self.conn.lock().map_err(|e| format!("KB lock: {}", e))?;

        // 1. 幂等判定
        let existing = if let Some(url) = &entry.url {
            nt_memory_store::find_node_by_url(&conn, url).map_err(|e| e.to_string())?
        } else {
            nt_memory_store::find_node_by_title_and_type(&conn, &entry.title, &node_type)
                .map_err(|e| e.to_string())?
        };

        let (node_id, created, _hub_id) = if let Some(existing) = existing {
            (existing.id, false, None::<String>)
        } else {
            // 2. 构造完整节点 (含 content, 非 summary 占位)
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            let node = KnowledgeNode {
                id: uuid::Uuid::new_v4().to_string(),
                node_type,
                title: entry.title.clone(),
                summary: entry.summary.clone(),
                content: entry.content.clone(),
                url: entry.url.clone(),
                domain: entry.domain.clone(),
                language: entry.language.clone().unwrap_or_else(|| "en".into()),
                confidence: 1.0,
                importance: entry.importance.unwrap_or(0.5),
                created_at: ts,
                updated_at: ts,
                access_count: 0,
                metadata: None,
                temporal: None,
                supersedes: None,
                source_episode: None,
            };
            // 3. 写入 nodes + FTS (事务)
            let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
            nt_memory_store::insert_node_rows(&tx, &node).map_err(|e| e.to_string())?;
            sync_fts(&tx, &node).map_err(|e| e.to_string())?;
            tx.commit().map_err(|e| e.to_string())?;
            (node.id.clone(), true, None)
        };

        // 4. 域枢纽 BelongsTo 边 (幂等 upsert)
        let mut hub_id_res: Option<String> = None;
        let mut hub_linked = false;
        if let Some(domain) = &entry.domain {
            let hid = ensure_hub(&conn, domain)?;
            hub_id_res = Some(hid.clone());
            // 直接走已持锁的 conn, 避免 self.upsert_edge 重入锁死 (Mutex 非重入)
            let linked = nt_memory_store::upsert_edge(
                &conn, &node_id, &hid, RelationType::BelongsTo, 1.0,
                Some(&format!("{} → {}", entry.title, domain)),
            );
            if linked.is_ok() {
                hub_linked = true;
            }
        }

        // 5. 附加关系边
        let mut edges_added = 0usize;
        for spec in &entry.relations {
            let relation = RelationType::from_str(&spec.relation);
            let target_id = if let Some(tid) = &spec.target_id {
                tid.clone()
            } else if let Some(ttitle) = &spec.target_title {
                let ttype = spec
                    .target_type
                    .as_ref()
                    .map(|s| NodeType::from_str(s))
                    .unwrap_or(NodeType::Concept);
                nt_memory_store::find_node_by_title_and_type(&conn, ttitle, &ttype)
                    .map_err(|e| e.to_string())?
                    .map(|n| n.id)
                    .unwrap_or_else(|| {
                        let _ = spec.target_title;
                        return String::new();
                    })
            } else {
                String::new()
            };
            if target_id.is_empty() {
                continue;
            }
            let w = spec.weight.unwrap_or(1.0);
            let desc = Some(format!("{} {}", entry.title, spec.relation));
            if nt_memory_store::upsert_edge(&conn, &node_id, &target_id, relation, w, desc.as_deref()).is_ok() {
                edges_added += 1;
            }
        }

        drop(conn);
        self.mark_bm25_dirty();
        Ok(AbsorbReport {
            node_id: node_id.to_string(),
            created,
            hub_id: hub_id_res,
            hub_linked,
            fts_synced: true,
            edges_added,
        })
    }

    /// 读端管道 — 服务意识体最短路径取知识。
    /// GWT 意图路由 → 按通道检索 → 结果 + 域枢纽图最短路径溯源。
    pub fn serve_core(&self, query: &str, limit: usize) -> Result<ServeResult, String> {
        let intent = self
            .gwt_router
            .read()
            .map_err(|e| format!("gwt_router lock: {}", e))?
            .route(query);

        let limit = limit.min(50).max(1);
        let results = match intent.channel {
            RetrievalChannel::Graph => {
                // 图通道: 混合检索优先; FTS 空结果时按查询中的实体 token 走图实体检索
                let mut r = self.hybrid_rerank_search(query, limit)?;
                if r.is_empty() {
                    r = self.graph_entity_search(query, limit)?;
                }
                r
            }
            _ => self.hybrid_rerank_search(query, limit)?,
        };

        // 图溯源: 取命中节点的域 → 域枢纽 → shortest_path
        let mut graph_path: Option<GraphPath> = None;
        if let Some(top) = results.first() {
            if let Some(domain) = &top.node.domain {
                let conn = self.conn.lock().map_err(|e| format!("KB lock: {}", e))?;
                let hid = hub_id_for(domain);
                if nt_memory_store::get_node(&conn, &hid)
                    .map_err(|e| e.to_string())?
                    .is_some()
                {
                    let path = super::nt_memory_graph::shortest_path(&conn, &top.node.id, &hid, 3)
                        .map_err(|e| e.to_string())?;
                    if let Some(p) = path {
                        if p.nodes.len() > 1 {
                            graph_path = Some(p);
                        }
                    }
                }
            }
        }

        Ok(ServeResult {
            query: query.to_string(),
            channel: intent.channel.as_str().to_string(),
            confidence: intent.confidence,
            resonance: intent.resonance,
            results,
            graph_path,
        })
    }

    /// 图通道实体检索: FTS 空结果时, 从查询提取实体 token (大写序列/中文段),
    /// 按 title 模糊匹配节点, 取深度 1 子图作为图通道结果 (GraphRelation 标记)。
    fn graph_entity_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
        let mut results = Vec::new();
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let conn = self.conn.lock().map_err(|e| format!("KB lock: {}", e))?;
        for token in extract_entities(query) {
            let matched = conn
                .prepare("SELECT id FROM nodes WHERE title LIKE ?1 ESCAPE '\\' LIMIT 5")
                .map_err(|e| format!("graph_entity_search prepare: {}", e))?
                .query_map(rusqlite::params![format!("%{}%", token)], |r| r.get::<_, String>(0))
                .map_err(|e| format!("graph_entity_search query: {}", e))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("graph_entity_search collect: {}", e))?;
            for nid in matched {
                if seen.insert(nid.clone()) {
                    let (nodes, _edges) = super::nt_memory_graph::subgraph(&conn, &nid, 1)
                        .map_err(|e| format!("subgraph {}: {}", nid, e))?;
                    for node in nodes {
                        if seen.insert(node.id.clone()) && results.len() < limit {
                            results.push(SearchResult {
                                node,
                                score: 1.0,
                                matched_on: vec![SearchMatchType::GraphRelation],
                                signals: None,
                            });
                        }
                    }
                }
            }
            if results.len() >= limit {
                break;
            }
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_kb() -> (KnowledgeBase, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("nt_kb_pipe_{}", std::process::id()));
        std::fs::create_dir_all(&dir).ok();
        let db_path = dir.join(format!("test_pipe_{}.db", std::thread::current().name().unwrap_or("t")));
        (KnowledgeBase::open(Some(db_path.clone())).expect("open kb"), db_path)
    }

    #[test]
    fn test_absorb_core_creates_node_and_hub() {
        let (kb, _) = temp_kb();
        let entry = AbsorbEntry {
            title: "最短路径管道测试".into(),
            summary: Some("测试节点".into()),
            content: Some("这是管道写端测试正文".into()),
            node_type: "concept".into(),
            domain: Some("NT-CORE".into()),
            url: None,
            language: Some("zh".into()),
            importance: Some(0.8),
            relations: vec![],
        };
        let report = kb.absorb_core(&entry).expect("absorb");
        assert!(report.created, "应新建节点");
        assert!(report.hub_linked, "应挂域枢纽边");
        assert_eq!(report.edges_added, 0);
        // FTS 检索闭环验证
        let served = kb.serve_core("最短路径管道", 5).expect("serve");
        assert!(!served.results.is_empty(), "FTS 应检索到新节点");
        assert!(served.channel == "fast" || served.channel == "vector", "channel={}", served.channel);
    }

    #[test]
    fn test_absorb_core_idempotent() {
        let (kb, _) = temp_kb();
        let entry = AbsorbEntry {
            title: "幂等测试".into(),
            summary: None,
            content: None,
            node_type: "concept".into(),
            domain: None,
            url: Some("file:///test/idempotent.md".into()),
            language: None,
            importance: None,
            relations: vec![],
        };
        let r1 = kb.absorb_core(&entry).expect("first");
        let r2 = kb.absorb_core(&entry).expect("second");
        assert!(r1.created, "首次应创建");
        assert!(!r2.created, "重复 url 应跳过");
        assert_eq!(r1.node_id, r2.node_id, "幂等应返回同 id");
    }

    #[test]
    fn test_ensure_hub_idempotent() {
        let (kb, _) = temp_kb();
        let conn = kb.conn.lock().expect("lock");
        let h1 = ensure_hub(&conn, "NT-MEMORY").expect("hub1");
        let h2 = ensure_hub(&conn, "NT-MEMORY").expect("hub2");
        assert_eq!(h1, h2, "hub 幂等应同 id");
        assert_eq!(h1, "kb_domain_hub_nt-memory");
        // FTS 存在
        let n = conn
            .query_row("SELECT COUNT(*) FROM nodes_fts WHERE title=?1", rusqlite::params![format!("KB-NT-MEMORY")], |r| r.get::<_, i64>(0))
            .expect("count");
        assert_eq!(n, 1, "hub 应有 FTS 行");
    }

    #[test]
    fn test_serve_core_graph_channel() {
        let (kb, _) = temp_kb();
        // 造两节点 + 边, 验证 graph 意图路由能检索
        let a = AbsorbEntry {
            title: "E8".into(),
            summary: Some("推理引擎".into()),
            content: Some("E8 影响 GWT".into()),
            node_type: "concept".into(),
            domain: Some("NT-CORE".into()),
            url: None,
            language: Some("en".into()),
            importance: Some(1.0),
            relations: vec![],
        };
        let b = AbsorbEntry {
            title: "GWT".into(),
            summary: Some("注意力路由".into()),
            content: Some("GWT 受 E8 影响".into()),
            node_type: "concept".into(),
            domain: Some("NT-CORE".into()),
            url: None,
            language: Some("en".into()),
            importance: Some(1.0),
            relations: vec![],
        };
        kb.absorb_core(&a).expect("a");
        let rb = kb.absorb_core(&b).expect("b");
        let conn = kb.conn.lock().expect("lock");
        let gid = nt_memory_store::find_node_by_title_and_type(&conn, "GWT", &NodeType::Concept)
            .expect("find").expect("gwt").id;
        drop(conn);
        let _ = kb.upsert_edge(&gid, &rb.node_id, RelationType::RelatedTo, 1.0, None);
        // 关系查询路由 Graph 通道
        let served = kb.serve_core("E8 如何影响 GWT 注意力路由的关系", 5).expect("serve");
        assert_eq!(served.channel, "graph", "channel={}", served.channel);
        assert!(!served.results.is_empty(), "graph 通道应检索到结果");
    }

    #[test]
    fn test_absorb_relation_edges() {
        let (kb, _) = temp_kb();
        let base = AbsorbEntry {
            title: "Base".into(),
            summary: None,
            content: None,
            node_type: "concept".into(),
            domain: None,
            url: None,
            language: None,
            importance: None,
            relations: vec![],
        };
        kb.absorb_core(&base).expect("base");
        let conn = kb.conn.lock().expect("lock");
        let base_id = nt_memory_store::find_node_by_title_and_type(&conn, "Base", &NodeType::Concept)
            .expect("find").expect("id").id;
        drop(conn);
        let entry = AbsorbEntry {
            title: "Related".into(),
            summary: None,
            content: None,
            node_type: "concept".into(),
            domain: None,
            url: None,
            language: None,
            importance: None,
            relations: vec![RelationSpec {
                relation: "related_to".into(),
                target_id: Some(base_id.clone()),
                target_title: None,
                target_type: None,
                weight: None,
            }],
        };
        let report = kb.absorb_core(&entry).expect("absorb");
        assert_eq!(report.edges_added, 1, "应添加 1 条关系边");
    }
}