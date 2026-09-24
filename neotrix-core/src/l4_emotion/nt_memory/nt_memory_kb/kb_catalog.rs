//! kb_catalog — 从 `nt_memory_kb/mod.rs` 拆分 (行为零变更).
//! `impl KnowledgeBase` 按域搬运, 原文逐行, 仅补可见性/导入。


use super::KnowledgeBase;
use super::nt_memory_store;
use super::{KnowledgeEdge, KnowledgeNode, KnowledgeStats, NodeType, ProceduralMemoryRecord, RelationType};

impl KnowledgeBase {
    /// Query KB for Repository nodes by domain, with optional min_stars filter
    pub fn find_repositories(
        &self,
        domain: &str,
        min_stars: Option<i64>,
    ) -> Result<Vec<KnowledgeNode>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let mut sql = "SELECT id, node_type, title, summary, content, url, domain, language, confidence, importance, created_at, updated_at, access_count, metadata FROM nodes WHERE node_type = 'repository'".to_string();
        if !domain.is_empty() {
            sql.push_str(" AND domain = ?1");
        }
        if let Some(_min) = min_stars {
            // min_stars 参数化绑定，避免字符串插值 (与 domain 的 ?1 风格一致)
            sql.push_str(" AND CAST(json_extract(metadata, '$.stars') AS INTEGER) >= ?2");
        }
        sql.push_str(" ORDER BY rowid DESC");
        let mut stmt = conn.prepare(&sql).map_err(|e| format!("prepare: {}", e))?;
        let mapper = |row: &rusqlite::Row| {
            Ok(KnowledgeNode {
                recall_weight: 1.0,
                id: row.get(0)?,
                node_type: NodeType::from_str(&row.get::<_, String>(1)?),
                title: row.get(2)?,
                summary: row.get(3)?,
                content: row.get(4)?,
                url: row.get(5)?,
                domain: row.get(6)?,
                language: row.get(7)?,
                confidence: row.get(8)?,
                importance: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
                access_count: row.get(12)?,
                metadata: row
                    .get::<_, Option<String>>(13)?
                    .and_then(|m| serde_json::from_str(&m).ok()),
                temporal: None,
                supersedes: None,
                source_episode: None,
                parent_id: None,
                depth: 0,
                cluster_id: None,
            })
        };
        let mapped_rows = match (domain.is_empty(), min_stars) {
            (true, None) => stmt
                .query_map([], mapper)
                .map_err(|e| format!("query: {}", e))?,
            (false, None) => stmt
                .query_map([domain], mapper)
                .map_err(|e| format!("query: {}", e))?,
            (true, Some(min)) => stmt
                .query_map([min], mapper)
                .map_err(|e| format!("query: {}", e))?,
            (false, Some(min)) => stmt
                .query_map(rusqlite::params![domain, min], mapper)
                .map_err(|e| format!("query: {}", e))?,
        };
        let mut repos = Vec::new();
        for row in mapped_rows {
            repos.push(row.map_err(|e| format!("row: {}", e))?);
        }
        Ok(repos)
    }

    /// Query KB for CodeSnippet nodes linked to a given repository
    pub fn find_code_snippets(&self, repo_node_id: &str) -> Result<Vec<KnowledgeNode>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let edges = nt_memory_store::get_edges_for_node(&conn, repo_node_id)
            .map_err(|e| format!("get_edges: {}", e))?;
        let snippet_ids: Vec<String> = edges
            .iter()
            .filter(|e| e.relation_type == RelationType::PartOf)
            .map(|e| e.target_id.clone())
            .collect();
        let mut snippets = Vec::new();
        for sid in &snippet_ids {
            if let Ok(Some(node)) = nt_memory_store::get_node(&conn, sid) {
                snippets.push(node);
            }
        }
        Ok(snippets)
    }

    // ── procedural memory ──

    pub fn store_procedural_memory(&self, record: &ProceduralMemoryRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_store::store_procedural_memory(&conn, record)
            .map_err(|e| format!("store_procedural_memory: {}", e))
    }

    pub fn list_procedural_memories(
        &self,
        top_k: usize,
    ) -> Result<Vec<ProceduralMemoryRecord>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_store::list_procedural_memories(&conn, top_k)
            .map_err(|e| format!("list_procedural_memories: {}", e))
    }

    // ── stats ──

    pub fn stats(&self) -> Result<KnowledgeStats, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_store::get_stats(&conn).map_err(|e| format!("stats: {}", e))
    }

    /// 向量化密度养料采集 — 非空 embedding 数量。
    /// 供 background_loop 意识树土壤喂料 (data_nourishment_factor 调制果实质量)。
    pub fn embedding_count(&self) -> usize {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return 0,
        };
        conn.query_row(
            "SELECT COUNT(*) FROM embeddings WHERE embedding IS NOT NULL",
            [],
            |row| row.get::<_, usize>(0),
        )
        .unwrap_or(0)
    }

    /// 枚举全部知识节点 — 供超立方体/图等内存结构批量灌入。
    pub fn all_nodes(&self) -> Result<Vec<KnowledgeNode>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_store::get_all_nodes(&conn).map_err(|e| format!("all_nodes: {}", e))
    }

    /// 四态记忆资产查询（吸收 TencentDB-Agent-Memory）: 按 `MemoryAssetKind` 过滤全部节点。
    /// 派生维度, 不落物理列 — 由 `MemoryAssetKind::classify` 实时推断, 避免与 node_type 双源真相漂移 (R-P42)。
    pub fn nodes_by_asset_kind(
        &self,
        kind: crate::l0_substrate::nt_core_memory_asset::MemoryAssetKind,
    ) -> Result<Vec<KnowledgeNode>, String> {
        let all = self.all_nodes()?;
        Ok(all
            .into_iter()
            .filter(|n| {
                crate::l0_substrate::nt_core_memory_asset::MemoryAssetKind::classify(n)
                    == Some(kind)
            })
            .collect())
    }

    /// 枚举全部知识边 — 供快照/图结构批量灌入。
    pub fn all_edges(&self) -> Result<Vec<KnowledgeEdge>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_store::get_all_edges(&conn).map_err(|e| format!("all_edges: {}", e))
    }

    // ── dedup ──

    pub fn dedup_nodes(&self) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_store::dedup_nodes(&conn).map_err(|e| format!("dedup_nodes: {}", e))
    }
}
