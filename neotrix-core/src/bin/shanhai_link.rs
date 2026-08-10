//! neotrix-shanhai-link — 跨引用关系推断
//!
//! 薄壳: 推断逻辑已吸收归档到 `nt_shanhai_geo::linking` (R-P42)。
//!
//! Usage: cargo run -p neotrix --bin neotrix-shanhai-link

use neotrix::neotrix::nt_memory_kb::nt_memory_schema;
use neotrix::neotrix::nt_shanhai_geo::{infer_shanhai_links, shanhai_edge_count};
use rusqlite::Connection;

fn open_kb() -> Connection {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let db_path = format!("{}/.neotrix/knowledge.db", home);
    let conn = Connection::open(&db_path).expect("Failed to open KB");
    nt_memory_schema::initialize(&conn).expect("Failed to init schema");
    conn
}

fn main() {
    let conn = open_kb();
    println!("🔗 山海经关系链接器");

    let edges_created = infer_shanhai_links(&conn).unwrap_or_else(|e| {
        eprintln!("link error: {}", e);
        std::process::exit(1);
    });

    println!("\n✅ 关系链接完成!");
    println!("   新增边: {} 条", edges_created);

    let total_edges = shanhai_edge_count(&conn).unwrap_or(0);
    println!("   山海边总数: {}", total_edges);
}
