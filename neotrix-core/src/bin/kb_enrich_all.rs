//! neotrix-kb-enrich-all — 知识库全局结构化填充
//!
//! Fills empty summaries/contents for the largest deficient node categories:
//! - Organization (1,997 nodes, 100% null content)
//! - Person (1,083 nodes, 100% null content)
//! - Concept (8,020 nodes, 73% empty summaries)
//!
//! Uses domain-aware templated descriptions.

use rusqlite::Connection;
use std::time::{SystemTime, UNIX_EPOCH};

use neotrix::neotrix::nt_memory_kb::nt_memory_schema;
use neotrix::neotrix::nt_memory_kb::nt_memory_store;
use neotrix::neotrix::nt_memory_kb::nt_memory_types::*;

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let db_path = format!("{}/.neotrix/knowledge.db", home);
    println!("Opening KB at: {}", db_path);

    let conn = Connection::open(&db_path).expect("Failed to open KB");
    nt_memory_schema::initialize(&conn).expect("Failed to init schema");

    let before = total_empty(&conn);

    fill_organizations(&conn);
    fill_persons(&conn);
    fill_concepts(&conn);
    fill_repositories(&conn);
    fill_papers(&conn);

    let after = total_empty(&conn);
    let fixed = before - after;
    println!("\n✅ 全局填充完成! 修复空节点: {}", fixed);
    println!("   填充前空节点: {}, 填充后: {}", before, after);
}

fn total_empty(conn: &Connection) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM nodes WHERE (summary IS NULL OR summary = '') OR (content IS NULL OR content = '')",
        [],
        |r| r.get(0),
    ).unwrap_or(0)
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

fn fill_organizations(conn: &Connection) {
    println!("\n=== Filling Organization nodes ===");
    let mut stmt = conn.prepare(
        "SELECT id, title, domain FROM nodes WHERE node_type='Organization' AND content IS NULL"
    ).expect("prepare failed");

    let rows: Vec<(String, String, Option<String>)> = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?))
    }).expect("query failed").filter_map(|r| r.ok()).collect();

    let ts = now();
    let mut count = 0;
    for (id, title, domain) in &rows {
        let domain_str = domain.as_deref().unwrap_or("unknown");
        let summary = match domain_str {
            "github.com" => format!("GitHub organization account: {}. Contains repositories and open-source projects.", title),
            _ => format!("Organization: {}. Domain: {}", title, domain_str),
        };
        let content = match domain_str {
            "github.com" => format!("{} is a GitHub organization with multiple software repositories. For detailed information, visit https://github.com/{}", title, title),
            _ => format!("Organization: {} (domain: {})", title, domain_str),
        };

        if let Err(e) = conn.execute(
            "UPDATE nodes SET summary=?1, content=?2, updated_at=?3 WHERE id=?4",
            rusqlite::params![summary, content, ts, id],
        ) {
            eprintln!("  ⚠️  update failed for {}: {}", title, e);
        } else {
            count += 1;
        }
    }
    println!("  ✅ 填充 {} 个 Organization 节点", count);
}

fn fill_persons(conn: &Connection) {
    println!("\n=== Filling Person nodes ===");
    let mut stmt = conn.prepare(
        "SELECT id, title, domain, url FROM nodes WHERE node_type='Person' AND content IS NULL"
    ).expect("prepare failed");

    let rows: Vec<(String, String, Option<String>, Option<String>)> = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, Option<String>>(3)?,
        ))
    }).expect("query failed").filter_map(|r| r.ok()).collect();

    let ts = now();
    let mut count = 0;
    for (id, title, domain, url) in &rows {
        let domain_str = domain.as_deref().unwrap_or("unknown");
        let summary = match domain_str {
            "github.com" => format!("GitHub user: {}. Open-source contributor with repositories on GitHub.", title),
            "arxiv.org" => format!("Researcher: {}. Author of academic papers on arXiv.", title),
            "wikipedia.org" => format!("Person referenced on Wikipedia: {}.", title),
            _ => format!("Person: {}. Associated with domain: {}", title, domain_str),
        };
        let content = match domain_str {
            "github.com" => format!("GitHub profile: {}. Active contributor to open-source projects.", title),
            _ => format!("Individual: {} (domain: {})", title, domain_str),
        };

        if let Err(e) = conn.execute(
            "UPDATE nodes SET summary=?1, content=?2, updated_at=?3 WHERE id=?4",
            rusqlite::params![summary, content, ts, id],
        ) {
            eprintln!("  ⚠️  update failed for {}: {}", title, e);
        } else {
            count += 1;
        }
    }
    println!("  ✅ 填充 {} 个 Person 节点", count);
}

fn fill_concepts(conn: &Connection) {
    println!("\n=== Filling Concept nodes (empty summaries) ===");
    let mut stmt = conn.prepare(
        "SELECT id, title, domain FROM nodes WHERE node_type='Concept' AND (summary IS NULL OR summary = '')"
    ).expect("prepare failed");

    let rows: Vec<(String, String, Option<String>)> = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?))
    }).expect("query failed").filter_map(|r| r.ok()).collect();

    let ts = now();
    let mut count = 0;

    for (id, title, domain) in &rows {
        let domain_str = domain.as_deref().unwrap_or("general");
        let summary = generate_concept_summary(title, domain_str);
        if summary.is_empty() { continue; }

        if let Err(e) = conn.execute(
            "UPDATE nodes SET summary=?1, updated_at=?2 WHERE id=?3",
            rusqlite::params![summary, ts, id],
        ) {
            eprintln!("  ⚠️  update failed for {}: {}", title, e);
        } else {
            count += 1;
        }

        if count % 1000 == 0 {
            println!("  ... 已填充 {} 个概念节点", count);
        }
    }
    println!("  ✅ 填充 {} 个 Concept 节点", count);
}

fn fill_repositories(conn: &Connection) {
    println!("\n=== Filling Repository nodes (content) ===");
    let mut stmt = conn.prepare(
        "SELECT id, title, domain FROM nodes WHERE node_type='Repository' AND content IS NULL"
    ).expect("prepare failed");

    let rows: Vec<(String, String, Option<String>)> = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?))
    }).expect("query failed").filter_map(|r| r.ok()).collect();

    let ts = now();
    let mut count = 0;
    for (id, title, domain) in &rows {
        let domain_str = domain.as_deref().unwrap_or("unknown");
        let summary = format!("Software repository: {}. Source code and project resources.", title);
        let full = format!("{} is a software repository from {}. Contains source code, documentation, and related project resources.", title, domain_str);

        if let Err(e) = conn.execute(
            "UPDATE nodes SET summary=?1, content=?2, updated_at=?3 WHERE id=?4",
            rusqlite::params![summary, full, ts, id],
        ) {
            eprintln!("  ⚠️  update failed for {}: {}", title, e);
        } else {
            count += 1;
        }
    }
    println!("  ✅ 填充 {} 个 Repository 节点", count);
}

fn fill_papers(conn: &Connection) {
    println!("\n=== Filling Paper nodes (content) ===");
    let mut stmt = conn.prepare(
        "SELECT id, title, domain FROM nodes WHERE node_type='Paper' AND content IS NULL AND summary IS NOT NULL AND summary != ''"
    ).expect("prepare failed");

    let rows: Vec<(String, String, Option<String>)> = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?))
    }).expect("query failed").filter_map(|r| r.ok()).collect();

    let ts = now();
    let mut count = 0;
    for (id, title, domain) in &rows {
        let domain_str = domain.as_deref().unwrap_or("unknown");
        let content = format!("Research paper: {}. Published in {}. Academic research publication with methodology, results, and analysis.", title, domain_str);

        if let Err(e) = conn.execute(
            "UPDATE nodes SET content=?1, updated_at=?2 WHERE id=?3",
            rusqlite::params![content, ts, id],
        ) {
            eprintln!("  ⚠️  update failed for {}: {}", title, e);
        } else {
            count += 1;
        }
    }
    println!("  ✅ 填充 {} 个 Paper 节点", count);
}

fn generate_concept_summary(title: &str, domain: &str) -> String {
    let t = title.trim();
    if t.is_empty() { return String::new(); }

    match domain {
        "github.com" => format!("GitHub topic/concept: {}. Used for categorizing open-source repositories.", t),
        "philosophy_science" => format!("Philosophy of science concept: {}. An idea within the philosophy and methodology of science.", t),
        "consciousness" => format!("Consciousness studies concept: {}. A topic within the study of consciousness and cognitive science.", t),
        "wikipedia.org" => format!("Wikipedia concept: {}. A topic referenced in Wikipedia.", t),
        "anna_archive" => format!("Archived concept: {}. A knowledge entry from the Anna's Archive collection.", t),
        "aa_books_catalog" => format!("Book catalog concept: {}. A topic from the book catalog classification.", t),
        "civilization_crawl" => format!("Civilization knowledge: {}. A topic discovered through knowledge base expansion.", t),
        "knowledge_crawl" => format!("Knowledge discovery: {}. A concept identified through automated crawling.", t),
        "scholar.google.com" => format!("Academic concept: {}. A research topic indexed from academic sources.", t),
        _ => format!("Concept: {}. A knowledge entry in the NeoTrix knowledge base.", t),
    }
}
