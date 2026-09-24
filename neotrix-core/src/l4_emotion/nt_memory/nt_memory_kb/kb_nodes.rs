//! kb_nodes — 从 `nt_memory_kb/mod.rs` 拆分 (行为零变更).
//! `impl KnowledgeBase` 按域搬运, 原文逐行, 仅补可见性/导入。


use super::KnowledgeBase;
use super::nt_memory_crawl;
use super::nt_memory_store;
use super::{KnowledgeEdge, KnowledgeNode, NodeType, RelationType, TemporalFact};

impl KnowledgeBase {
    pub fn insert_node(&self, node: &KnowledgeNode) -> Result<(), String> {
        self.lock_before_write()
            .map_err(|e| format!("File lock: {}", e))?;
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let r =
            nt_memory_store::insert_node(&conn, node).map_err(|e| format!("insert_node: {}", e));
        if r.is_ok() {
            self.mark_bm25_dirty();
            let _ = nt_memory_crawl::on_node_inserted(&conn, node);
        }
        drop(conn);
        if r.is_ok() {
            self.record_node_fact(node);
            if let Ok(mut lc) = self.lifecycle.write() {
                let now = lc.tick();
                lc.note_updated(&node.id, now);
            }
        }
        let _ = self.unlock_after_write();
        r
    }

    /// 时序事实记账 (TemporalFactLedger 接线, R-P79): 事实型节点 (有正文) 写入
    /// append-only temporal_facts。记账是 side-channel, 失败仅告警不阻断主库写入。
    pub(crate) fn record_node_fact(&self, node: &KnowledgeNode) {
        let Some(object) = node.content.as_ref().filter(|c| !c.trim().is_empty()) else {
            return;
        };
        let object: String = object.chars().take(2048).collect();
        let predicate = node.node_type.as_str();
        let source = node
            .url
            .as_deref()
            .or(node.domain.as_deref())
            .unwrap_or("kb_ingest");
        let (valid_from, valid_until) = match node.temporal.as_ref() {
            Some(t) => (Some(t.valid_from), t.valid_until),
            None => (Some(node.updated_at), None),
        };
        let res = self
            .temporal_ledger
            .lock()
            .map_err(|e| format!("Lock: {}", e))
            .and_then(|lg| {
                lg.add_node_fact(
                    &node.id,
                    &node.title,
                    predicate,
                    &object,
                    valid_from,
                    valid_until,
                    source,
                )
                .map_err(|e| e.to_string())
            });
        if let Err(e) = res {
            log::debug!("[temporal] skip node fact {}: {}", node.id, e);
        }
    }

    /// 时序事实点时刻查询 (生产接线): 返回 subject (节点标题) 在 ts 时刻有效的事实
    /// 版本。旧版事实在 supersede 时 valid_until 已截断, 不再出现在结果中。
    pub fn query_temporal(&self, key: &str, ts: i64) -> Result<Vec<TemporalFact>, String> {
        let lg = self
            .temporal_ledger
            .lock()
            .map_err(|e| format!("Lock: {}", e))?;
        lg.query_valid_at_subject(key, ts)
    }

    pub fn get_node(&self, id: &str) -> Result<Option<KnowledgeNode>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_store::get_node(&conn, id).map_err(|e| format!("get_node: {}", e))
    }

    pub fn insert_edge(&self, edge: &KnowledgeEdge) -> Result<(), String> {
        self.lock_before_write()
            .map_err(|e| format!("File lock: {}", e))?;
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let r =
            nt_memory_store::insert_edge(&conn, edge).map_err(|e| format!("insert_edge: {}", e));
        let _ = self.unlock_after_write();
        r
    }

    pub fn delete_node(&self, id: &str) -> Result<bool, String> {
        self.lock_before_write()
            .map_err(|e| format!("File lock: {}", e))?;
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let r = nt_memory_store::delete_node(&conn, id).map_err(|e| format!("delete_node: {}", e));
        if r.as_ref().ok().copied().unwrap_or(false) {
            self.mark_bm25_dirty();
            if let Ok(mut lc) = self.lifecycle.write() {
                lc.mark_should_forget(id);
            }
        }
        let _ = self.unlock_after_write();
        r
    }

    pub fn delete_edge(&self, id: &str) -> Result<bool, String> {
        self.lock_before_write()
            .map_err(|e| format!("File lock: {}", e))?;
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let r = nt_memory_store::delete_edge(&conn, id).map_err(|e| format!("delete_edge: {}", e));
        let _ = self.unlock_after_write();
        r
    }

    pub fn insert_or_get_node(
        &self,
        title: &str,
        node_type: NodeType,
        summary: Option<&str>,
        url: Option<&str>,
        domain: Option<&str>,
    ) -> Result<String, String> {
        self.lock_before_write()
            .map_err(|e| format!("File lock: {}", e))?;
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let r = nt_memory_store::insert_or_get_node(&conn, title, node_type, summary, url, domain)
            .map_err(|e| format!("insert_or_get_node: {}", e));
        let _ = self.unlock_after_write();
        r
    }

    pub fn upsert_edge(
        &self,
        source_id: &str,
        target_id: &str,
        relation_type: RelationType,
        weight: f64,
        description: Option<&str>,
    ) -> Result<(), String> {
        self.lock_before_write()
            .map_err(|e| format!("File lock: {}", e))?;
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let r = nt_memory_store::upsert_edge(
            &conn,
            source_id,
            target_id,
            relation_type,
            weight,
            description,
        )
        .map_err(|e| format!("upsert_edge: {}", e));
        let _ = self.unlock_after_write();
        r
    }

    /// 边是否已存在 (同 source/target/relation)。供幂等写入计数。
    pub fn edge_exists(
        &self,
        source_id: &str,
        target_id: &str,
        relation_type: RelationType,
    ) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let edges = nt_memory_store::get_edges_for_node(&conn, source_id)
            .map_err(|e| format!("edge_exists: {}", e))?;
        Ok(edges.iter().any(|e| {
            e.source_id == source_id && e.target_id == target_id && e.relation_type == relation_type
        }))
    }

    /// T0.1 类型化边 (metadata 增强版): 承载结构化溯源 (evidence/source/extractor)。
    /// 来自 39 仓库吸收 — codebase-memory-mcp 类型化边 + semantica PROV-O 溯源:
    /// edges 应带 source/confidence/extractor 元数据, 而非只塞进 description。
    pub fn upsert_edge_with_metadata(
        &self,
        source_id: &str,
        target_id: &str,
        relation_type: RelationType,
        weight: f64,
        description: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_store::upsert_edge_full(
            &conn,
            source_id,
            target_id,
            relation_type,
            weight,
            description,
            metadata,
        )
        .map_err(|e| format!("upsert_edge_with_metadata: {}", e))
    }

    pub fn find_node_by_url(&self, url: &str) -> Result<Option<KnowledgeNode>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_store::find_node_by_url(&conn, url)
            .map_err(|e| format!("find_node_by_url: {}", e))
    }

    pub fn update_node(&self, node: &KnowledgeNode) -> Result<(), String> {
        self.lock_before_write()
            .map_err(|e| format!("File lock: {}", e))?;
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let r =
            nt_memory_store::update_node(&conn, node).map_err(|e| format!("update_node: {}", e));
        if r.is_ok() {
            self.mark_bm25_dirty();
            if let Ok(mut lc) = self.lifecycle.write() {
                let now = lc.tick();
                lc.note_updated(&node.id, now);
            }
        }
        let _ = self.unlock_after_write();
        r
    }

    pub fn update_node_content(&self, id: &str, content: &str) -> Result<(), String> {
        self.lock_before_write()
            .map_err(|e| format!("File lock: {}", e))?;
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let mut node = nt_memory_store::get_node(&conn, id)
            .map_err(|e| format!("get_node: {}", e))?
            .ok_or_else(|| format!("Node not found: {}", id))?;
        node.content = Some(content.to_string());
        node.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let r = nt_memory_store::update_node(&conn, &node)
            .map_err(|e| format!("update_node_content: {}", e));
        if r.is_ok() {
            self.mark_bm25_dirty();
        }
        let _ = self.unlock_after_write();
        r
    }

    pub fn update_node_metadata(
        &self,
        id: &str,
        metadata: &serde_json::Value,
    ) -> Result<(), String> {
        self.lock_before_write()
            .map_err(|e| format!("File lock: {}", e))?;
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let r = nt_memory_store::update_node_metadata(&conn, id, metadata)
            .map_err(|e| format!("update_node_metadata: {}", e));
        let _ = self.unlock_after_write();
        r
    }
}
