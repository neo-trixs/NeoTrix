//! neotrix-shanhai-query — 山海世界数据查询与GeoJSON导出
//!
//! 薄壳: 查询逻辑已吸收归档到 `nt_shanhai_geo::query` (R-P42)。
//!
//! Usage:
//!   cargo run -p neotrix --bin neotrix-shanhai-query stats
//!   cargo run -p neotrix --bin neotrix-shanhai-query peaks
//!   cargo run -p neotrix --bin neotrix-shanhai-query mappings
//!   cargo run -p neotrix --bin neotrix-shanhai-query evidence
//!   cargo run -p neotrix --bin neotrix-shanhai-query schools
//!   cargo run -p neotrix --bin neotrix-shanhai-query export-geojson [path]

#![forbid(unsafe_code)]
use neotrix::neotrix::nt_memory_kb::nt_memory_schema;
use neotrix::neotrix::nt_shanhai_geo::{export_geojson, shanhai_evidence, shanhai_mappings, shanhai_peaks, shanhai_schools, shanhai_stats};
use rusqlite::Connection;

fn open_kb() -> Connection {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let db_path = format!("{}/.neotrix/knowledge.db", home);
    let conn = Connection::open(&db_path).expect("Failed to open KB");
    nt_memory_schema::initialize(&conn).expect("Failed to init schema");
    conn
}

fn cmd_stats(conn: &Connection) {
    let (nodes, edges, node_types, edge_types) = shanhai_stats(conn).unwrap_or_default();
    println!("📊 山海世界 KB 统计");
    println!("  {}", "─".repeat(40));
    println!("  节点总数: {}", nodes);
    println!("  关系总数: {}", edges);
    if !node_types.is_empty() {
        println!("\n  按类型:");
        for (ty, n) in &node_types {
            println!("    {:25} {}", ty, n);
        }
    }
    if !edge_types.is_empty() {
        println!("\n  按关系:");
        for (ty, n) in &edge_types {
            println!("    {:25} {}", ty, n);
        }
    }
}

fn cmd_peaks(conn: &Connection) {
    let rows = shanhai_peaks(conn).unwrap_or_default();
    println!("🏔️  山海经山系 ({} 座)", rows.len());
    for (_id, title, importance, location) in &rows {
        println!("  {:35} importance={:.1}  location={}", title, importance, location);
    }
}

fn cmd_mappings(conn: &Connection) {
    let rows = shanhai_mappings(conn).unwrap_or_default();
    println!("🌍  全球对应映射 ({} 个)", rows.len());
    for m in &rows {
        println!("  {} ({})", m.id.replace("shanhai-map:", ""), m.title);
        println!(
            "    现代: {} @ [{}]  c={:.0}%",
            m.modern_name,
            m.location,
            m.confidence * 100.0
        );
        if !m.scholars.is_empty() {
            println!("    归因: {}", m.scholars.join(", "));
        }
        if m.summary.len() > 120 {
            println!("    证据: {}...", &m.summary[..120]);
        } else if !m.summary.is_empty() {
            println!("    证据: {}", m.summary);
        }
        println!();
    }
}

fn cmd_evidence(conn: &Connection) {
    let rows = shanhai_evidence(conn).unwrap_or_default();
    println!("🔬  证据节点 ({} 个)", rows.len());
    for (_id, title, importance, ev_type, scholar, key) in &rows {
        println!("  [{:12}] {} [imp={:.1}]", ev_type, title, importance);
        println!("           → {}", scholar);
        if !key.is_empty() {
            println!("           🔑 {}", key);
        }
        println!();
    }
}

fn cmd_schools(conn: &Connection) {
    let rows = shanhai_schools(conn).unwrap_or_default();
    println!("🏫  学术流派 ({} 个)", rows.len());
    for (_id, title, summary, importance, tags) in &rows {
        println!(
            "  {} [imp={:.1}] tags: {}",
            title,
            importance,
            if tags.is_empty() { "—".to_string() } else { tags.to_string() }
        );
        if summary.len() > 100 {
            println!("    {}", &summary[..100]);
        } else if !summary.is_empty() {
            println!("    {}", summary);
        }
        println!();
    }
}

fn cmd_export_geojson(conn: &Connection, path: Option<&str>) {
    let output = export_geojson(conn).unwrap_or_else(|e| {
        eprintln!("GeoJSON export error: {}", e);
        std::process::exit(1);
    });
    match path {
        Some(p) => {
            std::fs::write(p, &output).expect("Failed to write GeoJSON");
            let features: serde_json::Value = serde_json::from_str(&output).unwrap_or_default();
            let n = features["features"].as_array().map(|a| a.len()).unwrap_or(0);
            println!("✅ GeoJSON exported to: {}", p);
            println!("   Features: {}", n);
        }
        None => {
            println!("{}", output);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: neotrix-shanhai-query <command> [args]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  stats                    — KB statistics");
    eprintln!("  peaks                    — list mountains");
    eprintln!("  mappings                 — list geo mappings");
    eprintln!("  evidence                 — list evidence nodes");
    eprintln!("  schools                  — list academic schools");
    eprintln!("  export-geojson [path]    — export mappings as GeoJSON (stdout if no path)");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let conn = open_kb();
    match args[1].as_str() {
        "stats" => cmd_stats(&conn),
        "peaks" => cmd_peaks(&conn),
        "mappings" => cmd_mappings(&conn),
        "evidence" => cmd_evidence(&conn),
        "schools" => cmd_schools(&conn),
        "export-geojson" => cmd_export_geojson(&conn, args.get(2).map(|s| s.as_str())),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            std::process::exit(1);
        }
    }
}
