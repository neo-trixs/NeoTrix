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
    /// 可验证回放收据签名 — 仅成功写入 (新建) 节点时产出;
    /// 被 Blocked 的吸收已在边界前置返回, 不产生收据 (拒绝即无痕)。
    pub receipt_signature: Option<String>,
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
        recall_weight: 1.0,
        supersedes: None,
        source_episode: None,
        parent_id: None,
        depth: 0,
        cluster_id: None,
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

/// W1.4 摄取时概念编译 (单一分词口径, 写读两侧共用):
/// ASCII 词 ≥4 字符 (小写化 + 停用词过滤) + CJK 连续段 ≥2 字。
/// 语义对齐经验层 `_extract_concepts`。返回按首现序去重的 ≤24 个概念。
pub fn compile_ingest_index(title: &str, summary: &str, content: &str) -> Vec<String> {
    const ASCII_STOPWORDS: &[&str] = &[
        "this", "that", "with", "from", "have", "has", "had", "will", "would", "could",
        "should", "into", "than", "then", "them", "they", "when", "what", "which", "while",
        "about", "after", "also", "been", "before", "being", "were", "your", "their",
        "there", "these", "those", "some", "such", "through", "under", "until", "very",
        "where", "other", "more", "most", "only", "over", "same", "just", "like", "make",
        "made", "many", "much", "need", "each", "both", "between", "because", "used",
        "using", "uses", "onto", "upon", "here", "does", "doing", "done", "from",
    ];
    const MAX_CONCEPTS: usize = 24;
    let mut out: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let text = format!("{}\n{}\n{}", title, summary, content);
    for word in text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || ('\u{4e00}'..='\u{9fff}').contains(&c))) {
        if word.is_empty() {
            continue;
        }
        let is_cjk = word.chars().next().map(|c| ('\u{4e00}'..='\u{9fff}').contains(&c)).unwrap_or(false);
        if is_cjk {
            // CJK 连续段 ≥2 字, 截断超长段 (12 字) 防整段正文混入。
            // 段长 >6 时额外取前/后 6 字窗口 — 提升子串查询命中率
            // ("研究记忆压缩断崖" → 追加 "研究记忆压缩"/"记忆压缩断崖")。
            let n = word.chars().count();
            if n >= 2 {
                let chars: Vec<char> = word.chars().take(12).collect();
                let m = chars.len();
                let full: String = chars.iter().collect();
                if seen.insert(full.clone()) {
                    out.push(full);
                }
                if n > 6 && m >= 6 {
                    for window in [chars[..6].iter().collect::<String>(), chars[m - 6..].iter().collect::<String>()] {
                        if seen.insert(window.clone()) {
                            out.push(window);
                        }
                    }
                }
            }
        } else {
            let lower = word.to_lowercase();
            if lower.len() >= 4 && !ASCII_STOPWORDS.contains(&lower.as_str()) && seen.insert(lower.clone()) {
                out.push(lower);
            }
        }
        if out.len() >= MAX_CONCEPTS {
            break;
        }
    }
    out
}

impl KnowledgeBase {
    /// 写端管道 — 吸收一个知识条目到 KB 最短路径。
    /// 幂等 (url 或 title+node_type 已存在则跳过), 自动挂域枢纽边 + FTS 同步。
    ///
    /// W1.4 (batch3 2026-08-26, 源: arxiv 2608.20845 *RAG Deserves an Index*):
    /// 新节点入库时同步编译 `metadata.ingest_index.concepts` (摄取时编译优于
    /// 查询时解释)。读侧 `nt_memory_search::search_fts` 对概念命中节点加分,
    /// 形成写↔读闭环 (R-P79 行为接地)。
    pub fn absorb_core(&self, entry: &AbsorbEntry) -> Result<AbsorbReport, String> {
        let node_type = NodeType::from_str(&entry.node_type);

        // 自毒化防火墙 (EVOMAL): 吸收边界前置扫描 — 拒绝"检索内容被固化为含 payload 的 skill 模板"。
        // fail-closed: 命中 Blocked 直接拒绝写入, 不产生可验证收据 (拒绝即无痕)。
        // 使用 trait 抽象 (消除了 L1→L3 直接依赖)。
        let verdict = {
            let scanner_guard = self.absorb_scanner.read().map_err(|e| format!("KB lock: {}", e))?;
            if let Some(ref scanner) = *scanner_guard {
                scanner.scan(&entry.title, &entry.summary, &entry.content)
            } else {
                // 默认: 无扫描器时不阻断 (兼容未注入 L3 实现的场景)
                crate::core::nt_core_traits::AbsorbVerdict { blocked: false, reasons: Vec::new() }
            }
        };
        if verdict.is_blocked() {
            return Err(format!(
                "self_poison firewall: absorb_core blocked entry '{}' (signals: {}). \
                 Refusing to instantiate retrieved content as a skill/memory template.",
                entry.title,
                verdict.reasons.join(", ")
            ));
        }

        let conn = self.conn.lock().map_err(|e| format!("KB lock: {}", e))?;

        // 1. 幂等判定
        let existing = if let Some(url) = &entry.url {
            nt_memory_store::find_node_by_url(&conn, url).map_err(|e| e.to_string())?
        } else {
            nt_memory_store::find_node_by_title_and_type(&conn, &entry.title, &node_type, false)
                .map_err(|e| e.to_string())?
        };

        let (node_id, created, receipt_sig) = if let Some(existing) = existing {
            (existing.id, false, None::<String>)
        } else {
            // 2. 构造完整节点 (含 content, 非 summary 占位)
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            // W1.4: 摄取时编译概念索引 → metadata.ingest_index
            let concepts = compile_ingest_index(
                &entry.title,
                entry.summary.as_deref().unwrap_or(""),
                entry.content.as_deref().unwrap_or(""),
            );
            let metadata = if concepts.is_empty() {
                None
            } else {
                Some(serde_json::json!({
                    "ingest_index": {
                        "concepts": concepts,
                        "compiled_at": ts,
                    }
                }))
            };
            let node = KnowledgeNode {
                id: uuid::Uuid::new_v4().to_string(),
                node_type,
                title: entry.title.clone(),
                summary: entry.summary.clone(),
                content: entry.content.clone(),
                url: entry.url.clone(),
                domain: entry.domain.clone(),
                language: entry.language.clone().unwrap_or_else(|| "en".to_string()),
                recall_weight: 1.0,
                confidence: 1.0,
                importance: entry.importance.unwrap_or(0.5),
                created_at: ts,
                updated_at: ts,
                access_count: 0,
                metadata,
                temporal: None,
                supersedes: None,
                source_episode: None,
                parent_id: None,
                depth: 0,
                cluster_id: None,
            };
            // 3. 写入 nodes + FTS (事务)
            let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
            nt_memory_store::insert_node_rows(&tx, &node).map_err(|e| e.to_string())?;
            sync_fts(&tx, &node).map_err(|e| e.to_string())?;
            tx.commit().map_err(|e| e.to_string())?;
            // 4. 可验证回放收据: 成功写节点后, 以节点 id 为 run_id,
            //    正文为 input, 摘要为 output 签发票 (事后可回放校验不可篡改)。
            // 使用 trait 抽象 (消除了 L1→L3 直接依赖)。
            let receipt_sig = {
                let emitter_guard = self.receipt_emitter.read().map_err(|e| format!("KB lock: {}", e))?;
                if let Some(ref emitter) = *emitter_guard {
                    Some(emitter.emit_receipt(
                        &node.id,
                        node.content.as_deref().unwrap_or(""),
                        node.summary.as_deref().unwrap_or(""),
                    ))
                } else {
                    None
                }
            };
            (node.id.clone(), true, receipt_sig)
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
                nt_memory_store::find_node_by_title_and_type(&conn, ttitle, &ttype, false)
                    .map_err(|e| e.to_string())?
                    .map(|n| n.id)
                    .unwrap_or_default()
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
            receipt_signature: receipt_sig,
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

        // 图溯源: 取命中节点的域 → 域枢纽 → weighted_shortest_path
        let mut graph_path: Option<GraphPath> = None;
        if let Some(top) = results.first() {
            if let Some(domain) = &top.node.domain {
                let conn = self.conn.lock().map_err(|e| format!("KB lock: {}", e))?;
                let hid = hub_id_for(domain);
                if nt_memory_store::get_node(&conn, &hid)
                    .map_err(|e| e.to_string())?
                    .is_some()
                {
                    // 使用加权最短路径替代 BFS
                    let cache = self.graph_cache.read().map_err(|e| format!("Cache lock: {}", e))?;
                    if let Some((node_ids, edges, cost)) = super::nt_memory_graph_cache::weighted_shortest_path(&cache, &top.node.id, &hid) {
                        if node_ids.len() > 1 {
                            // 转换为 GraphPath 格式
                            let mut nodes = Vec::new();
                            for nid in &node_ids {
                                if let Ok(Some(node)) = nt_memory_store::get_node(&conn, nid) {
                                    nodes.push(node);
                                }
                            }
                            graph_path = Some(GraphPath {
                                nodes,
                                edges,
                                total_distance: cost,
                            });
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
        let gid = nt_memory_store::find_node_by_title_and_type(&conn, "GWT", &NodeType::Concept, false)
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
        let base_id = nt_memory_store::find_node_by_title_and_type(&conn, "Base", &NodeType::Concept, false)
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
/// W1.4 (batch3 2026-08-26) 验收测试。
#[cfg(test)]
mod ingest_index_tests {
    use super::*;

    #[test]
    fn test_compile_ingest_index_mixed_text() {
        let concepts = compile_ingest_index(
            "Compaction Cliff in Agent Memory",
            "研究记忆压缩断崖",
            "The compaction cliff collapses LLM task success when context is compressed. 记忆压缩是长会话核心问题。",
        );
        // ASCII 停用词被滤除, 词被小写化
        assert!(concepts.contains(&"compaction".to_string()), "{concepts:?}");
        assert!(concepts.contains(&"cliff".to_string()));
        assert!(concepts.contains(&"agent".to_string()) || concepts.contains(&"memory".to_string()));
        assert!(!concepts.contains(&"the".to_string()), "停用词泄漏");
        assert!(!concepts.iter().any(|c| {
            c.chars().next().map(|ch: char| ('\u{4e00}'..='\u{9fff}').contains(&ch)).unwrap_or(false)
                && c.chars().count() < 2
        }), "短 CJK 段泄漏");
        // CJK 连续段保留 (≥2 字)
        assert!(concepts.contains(&"记忆压缩断崖".to_string()), "{concepts:?}");
        // 上限 24
        let flood = compile_ingest_index(
            &"word ".repeat(200),
            "",
            &("alpha beta gamma delta ".repeat(40)),
        );
        assert!(flood.len() <= 24);
    }

    #[test]
    fn test_absorb_core_writes_ingest_index() {
        let db = std::env::temp_dir().join(format!(
            "nt_w14_{}_{}.db", std::process::id(), std::thread::current().name().unwrap_or("t").len()
        ));
        let _ = std::fs::remove_file(&db);
        let kb = KnowledgeBase::open(Some(db.clone())).expect("kb");
        let entry = AbsorbEntry {
            title: "Compaction Study".into(),
            summary: Some("量化长会话记忆压缩断崖".into()),
            content: Some("Measures retention ratio collapse across compaction events.".into()),
            node_type: "article".into(),
            domain: Some("NT-MEMORY".into()),
            url: Some(format!("https://example.test/compaction-{}", std::process::id())),
            language: Some("en".into()),
            importance: Some(0.6),
            relations: vec![],
        };
        let report = kb.absorb_core(&entry).expect("absorb");
        assert!(report.created);
        let conn = kb.conn.lock().unwrap();
        let meta_json: String = conn
            .query_row(
                "SELECT metadata FROM nodes WHERE url=?1",
                rusqlite::params![entry.url.as_deref().unwrap()],
                |r| r.get(0),
            )
            .expect("node exists");
        let meta: serde_json::Value = serde_json::from_str(&meta_json).unwrap();
        let idx = &meta["ingest_index"];
        assert!(idx.is_object(), "ingest_index missing: {meta_json}");
        let concepts: Vec<String> = idx["concepts"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|c| c.as_str().map(String::from))
            .collect();
        assert!(concepts.contains(&"compaction".to_string()), "{concepts:?}");
        assert!(concepts.contains(&"记忆压缩断崖".to_string()), "{concepts:?}");
    }

    #[test]
    fn test_search_fts_boosts_concept_matches() {
        let db = std::env::temp_dir().join(format!(
            "nt_w14s_{}_{}.db", std::process::id(), std::thread::current().name().unwrap_or("t").len()
        ));
        let _ = std::fs::remove_file(&db);
        let kb = KnowledgeBase::open(Some(db.clone())).expect("kb");
        let entry = AbsorbEntry {
            title: "Quantum Error Survey".into(),
            summary: None,
            content: Some("Deep dive into quantum correction codes and thresholds.".into()),
            node_type: "article".into(),
            domain: None,
            url: Some(format!("https://example.test/quantum-{}", std::process::id())),
            language: Some("en".into()),
            importance: Some(0.5),
            relations: vec![],
        };
        kb.absorb_core(&entry).expect("absorb");
        let conn = kb.conn.lock().unwrap();
        let results = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_search::search_fts(
            &conn, "quantum", 10,
        )
        .expect("search");
        assert!(!results.is_empty());
        let top = &results[0];
        assert!(
            top.matched_on
                .iter()
                .any(|m| matches!(m, SearchMatchType::IngestConcept)),
            "expected IngestConcept marker, got {:?}",
            top.matched_on
        );
    }
}
