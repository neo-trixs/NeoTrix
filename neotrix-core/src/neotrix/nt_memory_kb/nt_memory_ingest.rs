/// KBIngester — 可复用的 KB 注入工具
/// 封装 7 个 seed binary 的共通模式为简洁 API
use super::nt_memory_store as store;
use super::nt_memory_types::*;
use super::KnowledgeBase;

pub struct KBIngester {
    kb: KnowledgeBase,
    log: Vec<String>,
    errors: Vec<String>,
}

impl KBIngester {
    pub fn open(path: Option<std::path::PathBuf>) -> Result<Self, String> {
        let kb = KnowledgeBase::open(path)?;
        Ok(Self {
            kb,
            log: Vec::new(),
            errors: Vec::new(),
        })
    }

    pub fn close(self) -> Result<(), String> {
        self.kb.close()
    }

    /// Insert a concept node (dedup by title+type).
    pub fn concept(&self, title: &str, summary: &str, domain: &str) -> Result<String, String> {
        let conn = self.kb.conn.lock().unwrap_or_else(|e| e.into_inner());
        match store::insert_or_get_node(
            &conn,
            title,
            NodeType::Concept,
            Some(summary),
            None,
            Some(domain),
        ) {
            Ok(id) => Ok(id),
            Err(e) => {
                self.kb.mark_bm25_dirty();
                Err(format!("concept {}: {}", title, e))
            }
        }
    }

    /// Insert an article node.
    pub fn article(
        &self,
        title: &str,
        summary: &str,
        url: &str,
        domain: &str,
    ) -> Result<String, String> {
        let conn = self.kb.conn.lock().unwrap_or_else(|e| e.into_inner());
        match store::insert_or_get_node(
            &conn,
            title,
            NodeType::Article,
            Some(summary),
            Some(url),
            Some(domain),
        ) {
            Ok(id) => Ok(id),
            Err(e) => {
                self.kb.mark_bm25_dirty();
                Err(format!("article {}: {}", title, e))
            }
        }
    }

    /// Insert a theory node.
    pub fn theory(&self, title: &str, summary: &str, domain: &str) -> Result<String, String> {
        let conn = self.kb.conn.lock().unwrap_or_else(|e| e.into_inner());
        match store::insert_or_get_node(
            &conn,
            title,
            NodeType::Theory,
            Some(summary),
            None,
            Some(domain),
        ) {
            Ok(id) => Ok(id),
            Err(e) => {
                self.kb.mark_bm25_dirty();
                Err(format!("theory {}: {}", title, e))
            }
        }
    }

    /// Fallible insert — returns None on error without panicking.
    pub fn try_concept(&mut self, title: &str, summary: &str, domain: &str) -> Option<String> {
        let conn = self.kb.conn.lock().unwrap_or_else(|e| e.into_inner());
        match store::insert_or_get_node(
            &conn,
            title,
            NodeType::Concept,
            Some(summary),
            None,
            Some(domain),
        ) {
            Ok(id) => {
                self.kb.mark_bm25_dirty();
                Some(id)
            }
            Err(e) => {
                self.errors.push(format!("concept {}: {}", title, e));
                None
            }
        }
    }

    /// Fallible insert with arbitrary NodeType.
    pub fn try_node(
        &mut self,
        title: &str,
        ntype: NodeType,
        summary: &str,
        url: Option<&str>,
        domain: &str,
    ) -> Option<String> {
        let conn = self.kb.conn.lock().unwrap_or_else(|e| e.into_inner());
        match store::insert_or_get_node(&conn, title, ntype, Some(summary), url, Some(domain)) {
            Ok(id) => {
                self.kb.mark_bm25_dirty();
                Some(id)
            }
            Err(e) => {
                self.errors.push(format!("node {}: {}", title, e));
                None
            }
        }
    }

    /// Insert a repo via GitHub API.
    pub fn repo(&self, owner: &str, repo: &str) -> usize {
        self.kb.ingest_github(owner, repo).unwrap_or_else(|e| {
            log::error!("  repo {}/{} failed: {}", owner, repo, e);
            0
        })
    }

    /// Insert a paper via ArXiv.
    pub fn arxiv(&self, id: &str) -> usize {
        self.kb.ingest_arxiv(id).unwrap_or_else(|e| {
            log::error!("  arxiv {} failed: {}", id, e);
            0
        })
    }

    /// Insert a Wikipedia topic.
    pub fn wikipedia(&self, topic: &str) -> usize {
        self.kb.ingest_wikipedia(topic).unwrap_or_else(|e| {
            log::error!("  wiki {} failed: {}", topic, e);
            0
        })
    }

    /// Wire an edge between two nodes (looked up by title).
    pub fn relate(
        &self,
        from_title: &str,
        to_title: &str,
        rel: RelationType,
        weight: f64,
        desc: &str,
    ) -> bool {
        let conn = self.kb.conn.lock().unwrap_or_else(|e| e.into_inner());
        let id_a = find_node_by_any_title(&conn, from_title);
        let id_b = find_node_by_any_title(&conn, to_title);
        let (ida, idb) = match (id_a, id_b) {
            (Some(a), Some(b)) => (a, b),
            _ => return false,
        };
        if ida == idb {
            return false;
        }
        store::upsert_edge(
            &conn,
            &ida,
            &idb,
            rel,
            weight,
            if desc.is_empty() { None } else { Some(desc) },
        )
        .is_ok()
    }

    /// Wire an edge using known node IDs directly.
    pub fn relate_ids(
        &self,
        from_id: &str,
        to_id: &str,
        rel: &RelationType,
        weight: f64,
        desc: &str,
    ) -> bool {
        let conn = self.kb.conn.lock().unwrap_or_else(|e| e.into_inner());
        store::upsert_edge(
            &conn,
            from_id,
            to_id,
            rel.clone(),
            weight,
            if desc.is_empty() { None } else { Some(desc) },
        )
        .is_ok()
    }

    /// Wire many edges at once from (from_title, to_title, relation, weight, desc) tuples.
    /// Only succeeds for pairs where both nodes exist.
    pub fn relate_many(&self, pairs: &[(&str, &str, RelationType, f64, &str)]) -> u32 {
        let mut ok = 0u32;
        for (a, b, rel, w, desc) in pairs {
            if self.relate(a, b, rel.clone(), *w, desc) {
                ok += 1;
            }
        }
        ok
    }

    /// Log a message for the final report.
    pub fn log(&mut self, msg: impl Into<String>) {
        self.log.push(msg.into());
    }

    /// Get KB stats.
    pub fn stats(&self) -> KnowledgeStats {
        self.kb.stats().expect("KnowledgeBase::stats() failed — check KB storage integrity and path")
    }

    /// Get errors collected during ingestion.
    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    /// Deduplicate nodes.
    pub fn dedup(&self) -> usize {
        self.kb.dedup_nodes().unwrap_or(0)
    }

    /// Print a pretty report of before → after stats.
    pub fn report(&self, label: &str, before: &KnowledgeStats, elapsed: std::time::Duration) {
        let after = self.stats();
        log::info!("\n╔══════════════════════════════════════════════════════╗");
        log::info!("║  {}  ║", pad_right(label, 46));
        log::info!("╚══════════════════════════════════════════════════════╝");
        log::info!("  总耗时: {:.1}s", elapsed.as_secs_f64());
        log::info!(
            "  节点:   {} → {} (+{})",
            before.total_nodes,
            after.total_nodes,
            after.total_nodes - before.total_nodes
        );
        log::info!(
            "  边:     {} → {} (+{})",
            before.total_edges,
            after.total_edges,
            after.total_edges - before.total_edges
        );
        if !self.log.is_empty() {
            log::info!("  日志:");
            for l in &self.log {
                log::info!("    {}", l);
            }
        }
        if !self.errors.is_empty() {
            log::info!("  错误 ({}):", self.errors.len());
            for e in &self.errors {
                log::info!("    ✗ {}", e);
            }
        }
        log::info!("");
    }

    /// Shorthand: stats snapshot for before/after comparison.
    pub fn snapshot(&self) -> KnowledgeStats {
        self.stats()
    }

    /// Pass-through to KB methods
    pub fn kb(&self) -> &KnowledgeBase {
        &self.kb
    }
}

fn pad_right(s: &str, width: usize) -> String {
    let mut r = s.to_string();
    while r.len() < width {
        r.push(' ');
    }
    r
}

fn find_node_by_any_title(conn: &rusqlite::Connection, title: &str) -> Option<String> {
    for t in &[
        "concept",
        "article",
        "paper",
        "repository",
        "theory",
        "method",
        "insight",
        "organization",
        "person",
    ] {
        if let Ok(mut stmt) =
            conn.prepare("SELECT id FROM nodes WHERE title = ?1 AND node_type = ?2")
        {
            if let Ok(rows) =
                stmt.query_map(rusqlite::params![title, t], |row| row.get::<_, String>(0))
            {
                for row in rows {
                    if let Ok(id) = row {
                        return Some(id);
                    }
                }
            }
        }
    }
    None
}
