//! kb_graph — 从 `nt_memory_kb/mod.rs` 拆分 (行为零变更).
//! `impl KnowledgeBase` 按域搬运, 原文逐行, 仅补可见性/导入。


use super::KnowledgeBase;
use super::nt_memory_graph;
use super::nt_memory_hierarchical;
use super::{KnowledgeEdge, KnowledgeNode};

impl KnowledgeBase {
    pub fn search_hierarchical(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<nt_memory_hierarchical::HierarchicalSearchResult>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let clusters = nt_memory_hierarchical::build_semantic_clusters(&conn, 2, 50)
            .map_err(|e| format!("build_clusters: {}", e))?;
        nt_memory_hierarchical::hierarchical_search(&conn, query, limit, &clusters)
            .map_err(|e| format!("hierarchical_search: {}", e))
    }

    // ── Graph ──

    pub fn subgraph(
        &self,
        center_id: &str,
        depth: usize,
    ) -> Result<(Vec<KnowledgeNode>, Vec<KnowledgeEdge>), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_graph::subgraph(&conn, center_id, depth).map_err(|e| format!("subgraph: {}", e))
    }

    /// 查询节点的邻居 (sqlite-knowledge-graph 模式吸收, P0)
    pub fn neighbors(&self, node_id: &str) -> Result<Vec<String>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let mut stmt = conn
            .prepare(
                "SELECT target_id FROM edges WHERE source_id = ?1 \
             UNION \
             SELECT source_id FROM edges WHERE target_id = ?1",
            )
            .map_err(|e| format!("prepare neighbors: {}", e))?;
        let neighbors = stmt
            .query_map([node_id], |row| row.get(0))
            .map_err(|e| format!("query neighbors: {}", e))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(neighbors)
    }

    /// 查询两节点间最短路径 (BFS, sqlite-knowledge-graph 模式吸收, P0)
    pub fn find_path(&self, from: &str, to: &str, max_depth: usize) -> Result<Vec<String>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((from.to_string(), vec![from.to_string()]));
        visited.insert(from.to_string());

        while let Some((current, path)) = queue.pop_front() {
            if current == to {
                return Ok(path);
            }
            if path.len() > max_depth {
                continue;
            }
            let mut stmt = conn
                .prepare(
                    "SELECT target_id FROM edges WHERE source_id = ?1 \
                 UNION \
                 SELECT source_id FROM edges WHERE target_id = ?1",
                )
                .map_err(|e| format!("prepare find_path: {}", e))?;
            let neighbors: Vec<String> = stmt
                .query_map([&current], |row| row.get(0))
                .map_err(|e| format!("query find_path: {}", e))?
                .filter_map(|r| r.ok())
                .collect();

            for neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor.clone());
                    let mut new_path = path.clone();
                    new_path.push(neighbor.clone());
                    queue.push_back((neighbor, new_path));
                }
            }
        }
        Ok(vec![])
    }

    /// 查询指定关系类型的邻居 (双向, sqlite-knowledge-graph 模式吸收, P0)
    pub fn neighbors_by_relation(
        &self,
        node_id: &str,
        relation_type: &str,
    ) -> Result<Vec<String>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let mut stmt = conn
            .prepare(
                "SELECT target_id FROM edges WHERE source_id = ?1 AND relation_type = ?2 \
             UNION \
             SELECT source_id FROM edges WHERE target_id = ?1 AND relation_type = ?2",
            )
            .map_err(|e| format!("prepare neighbors_by_relation: {}", e))?;
        let neighbors = stmt
            .query_map(rusqlite::params![node_id, relation_type], |row| row.get(0))
            .map_err(|e| format!("query neighbors_by_relation: {}", e))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(neighbors)
    }

    // ── Seed ──
}
