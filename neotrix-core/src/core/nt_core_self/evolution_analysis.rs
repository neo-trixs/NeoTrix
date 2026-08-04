//! KB health analysis and evolution todo generation.
//! Port of `scripts/generate-evolution-todo.py` into the self-audit pipeline.

use rusqlite::Connection;

/// A single KB defect found during analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct KbDefect {
    pub priority: String,
    pub severity: f64,
    pub title: String,
    pub area: String,
    pub detail: String,
}

/// Aggregate health report from KB analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct KbHealthReport {
    pub defects: Vec<KbDefect>,
    pub generated_at: i64,
}

impl KbHealthReport {
    pub fn p0_count(&self) -> usize {
        self.defects.iter().filter(|d| d.priority == "P0").count()
    }
    pub fn p1_count(&self) -> usize {
        self.defects.iter().filter(|d| d.priority == "P1").count()
    }
    pub fn p2_count(&self) -> usize {
        self.defects.iter().filter(|d| d.priority == "P2").count()
    }
    pub fn areas(&self) -> Vec<String> {
        let mut areas: Vec<String> = self.defects.iter().map(|d| d.area.clone()).collect();
        areas.sort();
        areas.dedup();
        areas
    }
}

/// Run deep analysis on KB and return a sorted defect list.
pub fn analyze_kb_health(conn: &Connection) -> Vec<KbDefect> {
    let mut defects = Vec::new();

    // Helper: safe fetch single row
    fn count(conn: &Connection, sql: &str, params: &[&dyn rusqlite::types::ToSql]) -> i64 {
        conn.query_row(sql, params, |r| r.get::<_, i64>(0)).unwrap_or(0)
    }

    // Helper: count by node type with optional empty content filter
    let total = count(conn, "SELECT COUNT(*) FROM nodes", &[]);

    // ── P0: Life-threatening ──

    let empty = count(conn, "SELECT COUNT(*) FROM nodes WHERE content IS NULL OR content = ''", &[]);
    if empty > 0 {
        let pct = if total > 0 { empty * 100 / total } else { 0 };
        let insight_empty = count(conn, "SELECT COUNT(*) FROM nodes WHERE node_type='Insight' AND (content IS NULL OR content = '')", &[]);
        let article_empty = count(conn, "SELECT COUNT(*) FROM nodes WHERE node_type='Article' AND (content IS NULL OR content = '')", &[]);
        let concept_empty = count(conn, "SELECT COUNT(*) FROM nodes WHERE node_type='Concept' AND (content IS NULL OR content = '')", &[]);
        let repo_empty = count(conn, "SELECT COUNT(*) FROM nodes WHERE node_type='Repository' AND (content IS NULL OR content = '')", &[]);
        defects.push(KbDefect {
            priority: "P0".into(), severity: 0.9,
            title: format!("{}/{} nodes empty ({}%) — bulk content fill pipeline needed", empty, total, pct),
            area: "kb-content".into(),
            detail: format!("Node types: Insight({}), Article({}), Concept({}), Repository({})",
                insight_empty, article_empty, concept_empty, repo_empty),
        });
    }

    // Broken edges
    let broken_src = count(conn, "SELECT COUNT(*) FROM edges e LEFT JOIN nodes n ON e.source_id = n.id WHERE n.id IS NULL", &[]);
    let broken_tgt = count(conn, "SELECT COUNT(*) FROM edges e LEFT JOIN nodes n ON e.target_id = n.id WHERE n.id IS NULL", &[]);
    if broken_src + broken_tgt > 0 {
        defects.push(KbDefect {
            priority: "P0".into(), severity: 0.85,
            title: format!("{} broken edges ({} source, {} target missing) — integrity crisis",
                broken_src + broken_tgt, broken_src, broken_tgt),
            area: "kb-integrity".into(),
            detail: "Edges reference nodes that no longer exist. Likely from dual-write cleanup without cascade delete.".into(),
        });
    }

    // Zero embeddings
    let emb = count(conn, "SELECT COUNT(*) FROM embeddings", &[]);
    if emb == 0 {
        defects.push(KbDefect {
            priority: "P0".into(), severity: 0.8,
            title: "0 embeddings — semantic search dead, RAG pipeline non-functional".into(),
            area: "kb-embedding".into(),
            detail: "Must run kb-generate-embeddings.py with NEOTRIX_EMBEDDING_API_KEY set".into(),
        });
    }

    // Repository metadata quality
    let repos_no_meta = count(conn, "SELECT COUNT(*) FROM nodes WHERE node_type='Repository' AND (metadata IS NULL OR metadata='{}' OR metadata='')", &[]);
    let total_repos = count(conn, "SELECT COUNT(*) FROM nodes WHERE node_type='Repository'", &[]);
    if repos_no_meta > 0 {
        let repo_pct = if total_repos > 0 { repos_no_meta * 100 / total_repos } else { 0 };
        defects.push(KbDefect {
            priority: "P0".into(), severity: 0.75,
            title: format!("{}/{} repos have no metadata ({}%) — useless stubs", repos_no_meta, total_repos, repo_pct),
            area: "kb-repo-quality".into(),
            detail: "Repository nodes created by scripts without fetching GitHub API for stars/language/topics".into(),
        });
    }

    // ── P1: Structural issues ──

    // Duplicate URLs
    let dup_query = "SELECT url, COUNT(*) as cnt FROM nodes WHERE url != '' GROUP BY url HAVING cnt > 1";
    if let Ok(mut stmt) = conn.prepare(dup_query) {
        if let Ok(rows) = stmt.query_map([], |r| {
            let url: String = r.get(0)?;
            let cnt: i64 = r.get(1)?;
            Ok((url, cnt))
        }) {
            let dup_results: Vec<_> = rows.filter_map(|r| r.ok()).collect();
            let total_dups: i64 = dup_results.iter().map(|(_, c)| c).sum();
            if !dup_results.is_empty() {
                let worst = dup_results.first().map(|(u, _)| u.as_str()).unwrap_or("");
                defects.push(KbDefect {
                    priority: "P1".into(), severity: 0.7,
                    title: format!("{} duplicate URLs ({} extra copies) — waste and search noise",
                        dup_results.len(), total_dups),
                    area: "kb-dedup".into(),
                    detail: format!("Worst: {} ({}x)", worst, dup_results.first().map(|(_, c)| c).unwrap_or(&0)),
                });
            }
        }
    }

    // Case-inconsistent node types
    let case_query = "SELECT node_type, COUNT(*) FROM nodes GROUP BY node_type HAVING node_type != substr(node_type, 1, 1) || substr(lower(node_type), 2)";
    if let Ok(mut stmt) = conn.prepare(case_query) {
        if let Ok(rows) = stmt.query_map([], |r| {
            let t: String = r.get(0)?;
            let c: i64 = r.get(1)?;
            Ok((t, c))
        }) {
            let case_results: Vec<_> = rows.filter_map(|r| r.ok()).collect();
            if !case_results.is_empty() {
                let detail: String = case_results.iter()
                    .map(|(t, c)| format!("{}({})", t, c))
                    .collect::<Vec<_>>()
                    .join(", ");
                defects.push(KbDefect {
                    priority: "P1".into(), severity: 0.65,
                    title: format!("{} lowercase-starting node types — schema violation", case_results.len()),
                    area: "kb-schema".into(),
                    detail,
                });
            }
        }
    }

    // Orphaned nodes
    let orphaned = count(conn, "SELECT COUNT(*) FROM nodes n WHERE NOT EXISTS (SELECT 1 FROM edges e WHERE e.source_id = n.id OR e.target_id = n.id)", &[]);
    if orphaned > 0 {
        defects.push(KbDefect {
            priority: "P1".into(), severity: 0.6,
            title: format!("{} nodes have zero edges — disconnected knowledge islands", orphaned),
            area: "kb-connectivity".into(),
            detail: "Nodes exist in the DB but no relationship connects them to the rest of the graph".into(),
        });
    }

    // Missing domain
    let no_domain = count(conn, "SELECT COUNT(*) FROM nodes WHERE domain IS NULL OR domain = ''", &[]);
    if no_domain > 0 {
        defects.push(KbDefect {
            priority: "P1".into(), severity: 0.55,
            title: format!("{} nodes missing domain field — search and filtering degraded", no_domain),
            area: "kb-metadata".into(),
            detail: "Domain field is NULL or empty, causing domain-based queries to miss these nodes".into(),
        });
    }

    // Crawl queue status
    let pending = count(conn, "SELECT COUNT(*) FROM crawl_queue WHERE status='pending'", &[]);
    let failed = count(conn, "SELECT COUNT(*) FROM crawl_queue WHERE status='failed'", &[]);
    let completed = count(conn, "SELECT COUNT(*) FROM crawl_queue WHERE status='completed'", &[]);
    defects.push(KbDefect {
        priority: "P1".into(), severity: 0.65,
        title: format!("Crawl queue: {} completed, {} failed, {} pending — need new seeds",
            completed, failed, pending),
        area: "kb-crawl".into(),
        detail: "Need to inject new seed URLs to continue external absorption".into(),
    });

    // ── P2: Quality improvements ──

    // Legacy dual-write tables
    let legacy_nodes = count(conn, "SELECT COUNT(*) FROM knowledge_nodes", &[]);
    if legacy_nodes > 0 {
        defects.push(KbDefect {
            priority: "P2".into(), severity: 0.5,
            title: format!("{} knowledge_nodes in legacy table — dual-write not fully migrated", legacy_nodes),
            area: "kb-legacy".into(),
            detail: "Script pipeline writes to both nodes + knowledge_nodes; need to stop writing to legacy".into(),
        });
    }
    let legacy_edges = count(conn, "SELECT COUNT(*) FROM knowledge_edges", &[]);
    if legacy_edges > 0 {
        defects.push(KbDefect {
            priority: "P2".into(), severity: 0.45,
            title: format!("{} knowledge_edges in legacy table — duplicate edge storage", legacy_edges),
            area: "kb-legacy".into(),
            detail: "Same as knowledge_nodes — dual-write to clean up".into(),
        });
    }

    // ArXiv paper quality
    let paper_empty = count(conn, "SELECT COUNT(*) FROM nodes WHERE node_type='Paper' AND (content IS NULL OR content = '')", &[]);
    let paper_total = count(conn, "SELECT COUNT(*) FROM nodes WHERE node_type='Paper'", &[]);
    if paper_empty > 0 {
        let pct = if paper_total > 0 { paper_empty * 100 / paper_total } else { 0 };
        defects.push(KbDefect {
            priority: "P2".into(), severity: 0.5,
            title: format!("{}/{} Paper nodes empty ({}%) — ArXiv fill pipeline", paper_empty, paper_total, pct),
            area: "kb-content-paper".into(),
            detail: "arXiv abstract fetch failing for some papers; need better retry + HTML fallback".into(),
        });
    }

    // Empty Insight nodes
    let insight_empty = count(conn, "SELECT COUNT(*) FROM nodes WHERE node_type='Insight' AND (content IS NULL OR content = '')", &[]);
    if insight_empty > 0 {
        defects.push(KbDefect {
            priority: "P2".into(), severity: 0.4,
            title: format!("{} empty Insight nodes — are these needed?", insight_empty),
            area: "kb-housekeeping".into(),
            detail: "Insight nodes are auto-generated; may not need content, but should be documented".into(),
        });
    }

    // Sort by priority
    let order = |p: &str| -> u8 { match p { "P0" => 0, "P1" => 1, "P2" => 2, _ => 99 } };
    defects.sort_by(|a, b| {
        let pa = order(&a.priority);
        let pb = order(&b.priority);
        pa.cmp(&pb).then_with(|| b.severity.partial_cmp(&a.severity).unwrap_or(std::cmp::Ordering::Equal))
    });

    defects
}

/// Store health report to KB kv_store.
pub fn store_report_to_kb(conn: &Connection, report: &KbHealthReport) -> rusqlite::Result<()> {
    let now = report.generated_at;

    // Create kv_store if not exists
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS kv_store (namespace TEXT, key TEXT, value TEXT, updated_at INTEGER,
         PRIMARY KEY (namespace, key))"
    )?;

    // Store individual defects
    for (i, d) in report.defects.iter().enumerate() {
        let key = format!("ev-{:x}-{:04x}", now, i);
        let value = serde_json::json!({
            "title": d.title,
            "priority": d.priority,
            "severity": d.severity,
            "area": d.area,
            "detail": d.detail,
            "ts": now,
        }).to_string();
        conn.execute(
            "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["evolution_todo", key, value, now],
        )?;
    }

    // Store aggregate
    let agg = serde_json::json!({
        "total": report.defects.len(),
        "p0_count": report.p0_count(),
        "p1_count": report.p1_count(),
        "p2_count": report.p2_count(),
        "areas": report.areas(),
        "generated_at": now,
    });
    conn.execute(
        "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params!["evolution_todo", "latest_aggregate", agg.to_string(), now],
    )?;

    Ok(())
}

/// Record a runtime operational defect to the KB `kv_store` `meta_cognition`
/// namespace. Generalized port of `scripts/novel-world-absorb.py:record_defect`
/// (and the equivalent pattern in other absorb daemons): a thin, retry-silent
/// append of a structured defect event keyed by a hex timestamp + sequence.
///
/// Returns the generated key on success, `None` if the write failed (callers treat
/// defect recording as best-effort, never fatal).
pub fn record_meta_cognition_defect(
    conn: &Connection,
    defect_type: &str,
    source: &str,
    description: &str,
    severity: f64,
    cycle: i64,
) -> Option<String> {
    let now = unix_now();
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS kv_store (namespace TEXT, key TEXT, value TEXT, updated_at INTEGER,
         PRIMARY KEY (namespace, key))"
    )
    .ok()?;
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let seq = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let key = format!("novel_{cycle:04}_{:x}_{:04x}", now, seq);
    let value = serde_json::json!({
        "defect_type": defect_type,
        "source": source,
        "description": description,
        "severity": severity,
        "ts": now,
        "cycle": cycle,
    })
    .to_string();
    conn.execute(
        "INSERT OR IGNORE INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params!["meta_cognition", key, value, now],
    )
    .ok()?;
    Some(key)
}

pub fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Pretty-print the health report.
pub fn print_report(report: &KbHealthReport) {
    println!();
    println!("┌────────────────────────────────────────────────────────────────────────────────┐");
    println!("│  NeoTrix Evolution Todo List — Full KB Deep Analysis                          │");
    println!("└────────────────────────────────────────────────────────────────────────────────┘");
    println!();

    for (i, d) in report.defects.iter().enumerate() {
        let icon = match d.priority.as_str() {
            "P0" => "🔴",
            "P1" => "🟡",
            "P2" => "🟢",
            _ => "⚪",
        };
        println!("  {}  [{}] #{:2}: {}", icon, d.priority, i + 1, d.title);
        println!("       Area: {} | Severity: {:.2}", d.area, d.severity);
        if !d.detail.is_empty() {
            println!("       {}", d.detail);
        }
        println!();
    }

    println!("┌────────────────────────────────────────────────────────────────────────────────┐");
    println!("│  Total: {} items  |  P0: {}  |  P1: {}  |  P2: {}  │",
        report.defects.len(), report.p0_count(), report.p1_count(), report.p2_count());
    println!("└────────────────────────────────────────────────────────────────────────────────┘");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE nodes (id INTEGER PRIMARY KEY, url TEXT, content TEXT, node_type TEXT,
                domain TEXT, metadata TEXT);
             CREATE TABLE edges (source_id INTEGER, target_id INTEGER);
             CREATE TABLE embeddings (id INTEGER PRIMARY KEY);
             CREATE TABLE crawl_queue (status TEXT);
             PRAGMA busy_timeout=30000;"
        ).unwrap();
        conn
    }

    #[test]
    fn test_analyze_kb_health_empty_db() {
        let conn = create_test_db();
        let defects = analyze_kb_health(&conn);
        assert!(!defects.is_empty(), "Should find defects even in empty DB");
        let p0: Vec<_> = defects.iter().filter(|d| d.priority == "P0").collect();
        assert!(!p0.is_empty(), "Should find P0 defects (empty content, 0 embeddings, etc.)");
    }

    #[test]
    fn test_analyze_kb_health_with_data() {
        let conn = create_test_db();
        conn.execute("INSERT INTO nodes (url, content, node_type, domain) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["http://example.com", "some content", "Article", "example.com"]).unwrap();
        conn.execute("INSERT INTO nodes (url, content, node_type, domain) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["http://example.org", "more content", "Repository", "example.org"]).unwrap();
        conn.execute("INSERT INTO edges (source_id, target_id) VALUES (1, 2)", []).unwrap();

        let defects = analyze_kb_health(&conn);
        let empty_nodes: Vec<_> = defects.iter().filter(|d| d.area == "kb-content").collect();
        // 0 empty nodes since both have content
        assert!(empty_nodes.is_empty() || empty_nodes[0].title.starts_with("0/"),
            "Should report 0 empty nodes when all have content");
    }

    #[test]
    fn test_store_report_to_kb() {
        let conn = create_test_db();
        let defects = vec![
            KbDefect {
                priority: "P0".into(), severity: 0.9,
                title: "Test defect".into(), area: "kb-test".into(),
                detail: "Testing storage".into(),
            },
        ];
        let report = KbHealthReport { defects, generated_at: 1000 };
        store_report_to_kb(&conn, &report).unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM kv_store WHERE namespace='evolution_todo'",
            [], |r| r.get(0)
        ).unwrap();
        assert_eq!(count, 2, "Should store 1 defect + 1 aggregate");
    }

    #[test]
    fn test_print_report_does_not_crash() {
        let defects = vec![
            KbDefect {
                priority: "P0".into(), severity: 0.9,
                title: "Test P0".into(), area: "test".into(),
                detail: "Detail".into(),
            },
        ];
        let report = KbHealthReport { defects, generated_at: 1000 };
        print_report(&report);
        // Should not panic
    }
}
