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
        Ok(Self { kb, log: Vec::new(), errors: Vec::new() })
    }

    pub fn close(self) -> Result<(), String> {
        self.kb.close()
    }

    /// Insert a concept node (dedup by title+type).
    pub fn concept(&self, title: &str, summary: &str, domain: &str) -> String {
        let conn = match self.kb.conn.lock() {
            Ok(c) => c,
            Err(_) => { eprintln!("concept {}: lock poisoned", title); return String::new(); }
        };
        match store::insert_or_get_node(&conn, title, NodeType::Concept, Some(summary), None, Some(domain)) {
            Ok(id) => id,
            Err(e) => { self.kb.mark_bm25_dirty(); eprintln!("concept {}: {}", title, e); String::new() }
        }
    }

    /// Insert an article node.
    pub fn article(&self, title: &str, summary: &str, url: &str, domain: &str) -> String {
        let conn = match self.kb.conn.lock() {
            Ok(c) => c,
            Err(_) => { eprintln!("article {}: lock poisoned", title); return String::new(); }
        };
        match store::insert_or_get_node(&conn, title, NodeType::Article, Some(summary), Some(url), Some(domain)) {
            Ok(id) => id,
            Err(e) => { self.kb.mark_bm25_dirty(); eprintln!("article {}: {}", title, e); String::new() }
        }
    }

    /// Insert a theory node.
    pub fn theory(&self, title: &str, summary: &str, domain: &str) -> String {
        let conn = match self.kb.conn.lock() {
            Ok(c) => c,
            Err(_) => { eprintln!("theory {}: lock poisoned", title); return String::new(); }
        };
        match store::insert_or_get_node(&conn, title, NodeType::Theory, Some(summary), None, Some(domain)) {
            Ok(id) => id,
            Err(e) => { self.kb.mark_bm25_dirty(); eprintln!("theory {}: {}", title, e); String::new() }
        }
    }

    /// Fallible insert — returns None on error without panicking.
    pub fn try_concept(&mut self, title: &str, summary: &str, domain: &str) -> Option<String> {
        let conn = self.kb.conn.lock().unwrap_or_else(|e| e.into_inner());
        match store::insert_or_get_node(&conn, title, NodeType::Concept, Some(summary), None, Some(domain)) {
            Ok(id) => { self.kb.mark_bm25_dirty(); Some(id) }
            Err(e) => { self.errors.push(format!("concept {}: {}", title, e)); None }
        }
    }

    /// Fallible insert with arbitrary NodeType.
    pub fn try_node(&mut self, title: &str, ntype: NodeType, summary: &str, url: Option<&str>, domain: &str) -> Option<String> {
        let conn = self.kb.conn.lock().unwrap_or_else(|e| e.into_inner());
        match store::insert_or_get_node(&conn, title, ntype, Some(summary), url, Some(domain)) {
            Ok(id) => { self.kb.mark_bm25_dirty(); Some(id) }
            Err(e) => { self.errors.push(format!("node {}: {}", title, e)); None }
        }
    }

    /// Insert a repo via GitHub API.
    pub fn repo(&self, owner: &str, repo: &str) -> usize {
        self.kb.ingest_github(owner, repo).unwrap_or_else(|e| { eprintln!("  repo {}/{} failed: {}", owner, repo, e); 0 })
    }

    /// Insert a paper via ArXiv.
    pub fn arxiv(&self, id: &str) -> usize {
        self.kb.ingest_arxiv(id).unwrap_or_else(|e| { eprintln!("  arxiv {} failed: {}", id, e); 0 })
    }

    /// Insert a Wikipedia topic.
    pub fn wikipedia(&self, topic: &str) -> usize {
        self.kb.ingest_wikipedia(topic).unwrap_or_else(|e| { eprintln!("  wiki {} failed: {}", topic, e); 0 })
    }

    /// Wire an edge between two nodes (looked up by title).
    pub fn relate(&self, from_title: &str, to_title: &str, rel: RelationType, weight: f64, desc: &str) -> bool {
        let conn = self.kb.conn.lock().unwrap_or_else(|e| e.into_inner());
        let id_a = find_node_by_any_title(&conn, from_title);
        let id_b = find_node_by_any_title(&conn, to_title);
        let (ida, idb) = match (id_a, id_b) {
            (Some(a), Some(b)) => (a, b),
            _ => return false,
        };
        if ida == idb { return false; }
        store::upsert_edge(&conn, &ida, &idb, rel, weight, if desc.is_empty() { None } else { Some(desc) }).is_ok()
    }

    /// Wire an edge using known node IDs directly.
    pub fn relate_ids(&self, from_id: &str, to_id: &str, rel: &RelationType, weight: f64, desc: &str) -> bool {
        let conn = self.kb.conn.lock().unwrap_or_else(|e| e.into_inner());
        store::upsert_edge(&conn, from_id, to_id, rel.clone(), weight, if desc.is_empty() { None } else { Some(desc) }).is_ok()
    }

    /// Wire many edges at once from (from_title, to_title, relation, weight, desc) tuples.
    /// Only succeeds for pairs where both nodes exist.
    pub fn relate_many(&self, pairs: &[(&str, &str, RelationType, f64, &str)]) -> u32 {
        let mut ok = 0u32;
        for (a, b, rel, w, desc) in pairs {
            if self.relate(a, b, rel.clone(), *w, desc) { ok += 1; }
        }
        ok
    }

    /// Log a message for the final report.
    pub fn log(&mut self, msg: impl Into<String>) {
        self.log.push(msg.into());
    }

    /// Get KB stats.
    pub fn stats(&self) -> KnowledgeStats {
        self.kb.stats().unwrap_or_else(|e| {
            eprintln!("[neotrix] WARNING: KBIngester::stats() failed: {}", e);
            KnowledgeStats::default()
        })
    }

    /// Get errors collected during ingestion.
    pub fn errors(&self) -> &[String] { &self.errors }

    /// Deduplicate nodes.
    pub fn dedup(&self) -> usize {
        self.kb.dedup_nodes().unwrap_or(0)
    }

    /// Print a pretty report of before → after stats.
    pub fn report(&self, label: &str, before: &KnowledgeStats, elapsed: std::time::Duration) {
        let after = self.stats();
        println!("\n╔══════════════════════════════════════════════════════╗");
        println!("║  {}  ║", pad_right(label, 46));
        println!("╚══════════════════════════════════════════════════════╝");
        println!("  总耗时: {:.1}s", elapsed.as_secs_f64());
        println!("  节点:   {} → {} (+{})", before.total_nodes, after.total_nodes, after.total_nodes - before.total_nodes);
        println!("  边:     {} → {} (+{})", before.total_edges, after.total_edges, after.total_edges - before.total_edges);
        if !self.log.is_empty() {
            println!("  日志:");
            for l in &self.log { println!("    {}", l); }
        }
        if !self.errors.is_empty() {
            println!("  错误 ({}):", self.errors.len());
            for e in &self.errors { println!("    ✗ {}", e); }
        }
        println!();
    }

    /// Shorthand: stats snapshot for before/after comparison.
    pub fn snapshot(&self) -> KnowledgeStats {
        self.stats()
    }

    /// Pass-through to KB methods
    pub fn kb(&self) -> &KnowledgeBase { &self.kb }
}

fn pad_right(s: &str, width: usize) -> String {
    let mut r = s.to_string();
    while r.len() < width { r.push(' '); }
    r
}

fn find_node_by_any_title(conn: &rusqlite::Connection, title: &str) -> Option<String> {
    for t in &["concept", "article", "paper", "repository", "theory", "method", "insight", "organization", "person"] {
        if let Ok(mut stmt) = conn.prepare("SELECT id FROM nodes WHERE title = ?1 AND node_type = ?2") {
            if let Ok(rows) = stmt.query_map(rusqlite::params![title, t], |row| row.get::<_, String>(0)) {
                if let Some(id) = rows.flatten().next() { return Some(id); }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_kb_path() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("nt_kb_ingest_{}_{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&dir).ok();
        dir.join("test_ingest.db")
    }

    #[test]
    fn test_concept_insert_and_dedup() {
        let path = temp_kb_path();
        let ing = KBIngester::open(Some(path.clone())).expect("open");
        let id1 = ing.concept("dedup测试概念", "summary", "test-domain");
        assert!(!id1.is_empty(), "concept should insert");
        // 同 title+type 二次插入 → dedup 返回同 id
        let id2 = ing.concept("dedup测试概念", "summary2", "test-domain");
        assert_eq!(id1, id2, "same title+type should dedup");
        // 不同 title → 不同 id
        let id3 = ing.concept("另一概念", "summary", "test-domain");
        assert_ne!(id1, id3);
        ing.close().expect("close");
    }

    #[test]
    fn test_article_and_theory_insert() {
        let path = temp_kb_path();
        let ing = KBIngester::open(Some(path.clone())).expect("open");
        let a = ing.article("文章标题", "摘要", "https://example.com/x", "test-domain");
        assert!(!a.is_empty());
        let t = ing.theory("理论标题", "摘要", "test-domain");
        assert!(!t.is_empty());
        assert_ne!(a, t);
        // article 写入 url
        let url: String = {
            let kb = ing.kb();
            let conn = kb.conn.lock().unwrap();
            conn.query_row(
                "SELECT url FROM nodes WHERE id = ?1",
                rusqlite::params![a],
                |row| row.get(0),
            ).expect("query url")
        };
        assert_eq!(url, "https://example.com/x");
        ing.close().expect("close");
    }

    #[test]
    fn test_relate_wires_edge_between_existing_nodes() {
        let path = temp_kb_path();
        let mut ing = KBIngester::open(Some(path.clone())).expect("open");
        ing.concept("起点A", "s", "test-domain");
        ing.concept("终点B", "s", "test-domain");
        assert!(ing.relate("起点A", "终点B", RelationType::RelatedTo, 0.8, "desc"));
        // 不存在的节点 → false
        assert!(!ing.relate("起点A", "不存在C", RelationType::RelatedTo, 0.5, ""));
        // 自环 → false
        assert!(!ing.relate("起点A", "起点A", RelationType::RelatedTo, 0.5, ""));
        ing.close().expect("close");
    }

    #[test]
    fn test_relate_many_counts_successes() {
        let path = temp_kb_path();
        let mut ing = KBIngester::open(Some(path.clone())).expect("open");
        ing.concept("X1", "s", "d");
        ing.concept("X2", "s", "d");
        ing.concept("X3", "s", "d");
        let ok = ing.relate_many(&[
            ("X1", "X2", RelationType::RelatedTo, 0.5, "e1"),
            ("X2", "X3", RelationType::RelatedTo, 0.5, "e2"),
            ("X1", "幽灵", RelationType::RelatedTo, 0.5, "e3"), // 失败
        ]);
        assert_eq!(ok, 2);
        ing.close().expect("close");
    }

    #[test]
    fn test_try_node_collects_errors_not_panics() {
        let path = temp_kb_path();
        let mut ing = KBIngester::open(Some(path.clone())).expect("open");
        let id = ing.try_node("try节点", NodeType::Concept, "s", None, "d");
        assert!(id.is_some());
        // 非法 title (空) 等不应 panic
        let _ = ing.try_node("", NodeType::Concept, "s", None, "d");
        ing.close().expect("close");
    }
}
