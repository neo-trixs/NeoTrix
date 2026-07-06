//! neotrix-shanhai-query — 山海世界数据查询与GeoJSON导出
//!
//! Usage:
//!   cargo run -p neotrix --bin neotrix-shanhai-query stats
//!   cargo run -p neotrix --bin neotrix-shanhai-query peaks
//!   cargo run -p neotrix --bin neotrix-shanhai-query mappings
//!   cargo run -p neotrix --bin neotrix-shanhai-query evidence
//!   cargo run -p neotrix --bin neotrix-shanhai-query schools
//!   cargo run -p neotrix --bin neotrix-shanhai-query export-geojson [path]

use neotrix::neotrix::nt_memory_kb::nt_memory_schema;
use rusqlite::Connection;

fn open_kb() -> Connection {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let db_path = format!("{}/.neotrix/knowledge.db", home);
    let conn = Connection::open(&db_path).expect("Failed to open KB");
    nt_memory_schema::initialize(&conn).expect("Failed to init schema");
    conn
}

fn cmd_stats(conn: &Connection) {
    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM nodes WHERE id LIKE 'shanhai-%'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_default();
    let edges: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM edges WHERE id LIKE 'shanhai-edge:%'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_default();

    println!("📊 山海世界 KB 统计");
    println!("  {}", "─".repeat(40));
    println!("  节点总数: {}", total);
    println!("  关系总数: {}", edges);

    let mut stmt = conn
        .prepare("SELECT node_type, COUNT(*) FROM nodes WHERE id LIKE 'shanhai-%' GROUP BY node_type ORDER BY COUNT(*) DESC")
        .expect("stats: node_type SQL prepare");
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
        .expect("stats: node_type query_map");
    println!("\n  按类型:");
    for row in rows {
        let (ty, n) = row.expect("stats: node_type row");
        println!("    {:25} {}", ty, n);
    }

    let mut stmt = conn
        .prepare("SELECT relation_type, COUNT(*) FROM edges WHERE id LIKE 'shanhai-edge:%' GROUP BY relation_type ORDER BY COUNT(*) DESC")
        .expect("cmd_stats: relation_type SQL prepare");
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
        .expect("cmd_stats: relation_type query_map");
    println!("\n  按关系:");
    for row in rows {
        let (ty, n) = row.expect("cmd_stats: relation_type row");
        println!("    {:25} {}", ty, n);
    }
}

fn parse_meta(s: &str, key: &str) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
        v.get(key).and_then(|l| l.as_str()).unwrap_or("—").to_string()
    } else {
        "—".to_string()
    }
}

fn cmd_peaks(conn: &Connection) {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, importance, metadata FROM nodes \
             WHERE id LIKE 'shanhai-peak:%' ORDER BY id",
        )
        .expect("cmd_peaks: SQL prepare");
    let rows: Vec<(String, String, f64, Option<String>)> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, f64>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })
        .expect("cmd_peaks: query_map")
        .filter_map(|r| r.ok())
        .collect();

    println!("🏔️  山海经山系 ({} 座)", rows.len());
    for (_id, title, importance, meta) in &rows {
        let location = meta.as_deref().map(|m| parse_meta(m, "location")).unwrap_or_default();
        println!(
            "  {:35} importance={:.1}  location={}",
            title, importance, location
        );
    }
}

fn parse_meta_value(s: &str) -> Option<serde_json::Value> {
    serde_json::from_str::<serde_json::Value>(s).ok()
}

fn cmd_mappings(conn: &Connection) {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, summary, metadata FROM nodes \
             WHERE id LIKE 'shanhai-map:%' ORDER BY id",
        )
        .expect("cmd_mappings: SQL prepare");
    let rows: Vec<(String, String, Option<String>, Option<String>)> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })
        .expect("cmd_mappings: query_map")
        .filter_map(|r| r.ok())
        .collect();

    println!("🌍  全球对应映射 ({} 个)", rows.len());
    for (id, title, summary, meta_str) in &rows {
        let meta = meta_str.as_deref().and_then(parse_meta_value);
        let location = meta.as_ref().and_then(|v| v.get("modern_location").and_then(|l| l.as_str())).unwrap_or("—");
        let modern_name = meta.as_ref().and_then(|v| v.get("modern_name").and_then(|l| l.as_str())).unwrap_or("—");
        let confidence = meta.as_ref().and_then(|v| v.get("confidence").and_then(|c| c.as_f64())).unwrap_or(0.0);
        let scholar = meta.as_ref()
            .and_then(|v| v.get("attributed_by").and_then(|a| a.as_array()))
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
            .unwrap_or_default();

        println!("  {} ({})", id.replace("shanhai-map:", ""), title);
        println!("    现代: {} @ [{}]  c={:.0}%", modern_name, location, confidence * 100.0);
        if !scholar.is_empty() {
            println!("    归因: {}", scholar);
        }
        if let Some(ref s) = summary {
            if s.len() > 120 {
                println!("    证据: {}...", &s[..120]);
            } else {
                println!("    证据: {}", s);
            }
        }
        println!();
    }
}

fn cmd_evidence(conn: &Connection) {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, importance, metadata FROM nodes \
             WHERE id LIKE 'shanhai-evidence:%' ORDER BY importance DESC",
        )
        .expect("cmd_evidence: SQL prepare");
    let rows: Vec<(String, String, f64, Option<String>)> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, f64>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })
        .expect("cmd_evidence: query_map")
        .filter_map(|r| r.ok())
        .collect();

    println!("🔬  证据节点 ({} 个)", rows.len());
    for (_id, title, importance, meta_str) in &rows {
        let meta = meta_str.as_deref().and_then(parse_meta_value);
        let ev_type = meta.as_ref().and_then(|v| v.get("type").and_then(|t| t.as_str())).unwrap_or("unknown");
        let scholar = meta.as_ref()
            .and_then(|v| v.get("scholar").or_else(|| v.get("archaeologist")).and_then(|s| s.as_str()))
            .unwrap_or("—");

        println!("  [{:12}] {} [imp={:.1}]", ev_type, title, importance);
        println!("           → {}", scholar);

        let key_findings = meta.as_ref()
            .and_then(|v| v.get("key_insight").or_else(|| v.get("conclusion")))
            .and_then(|k| k.as_str());
        if let Some(kf) = key_findings {
            println!("           🔑 {}", kf);
        }
        println!();
    }
}

fn cmd_schools(conn: &Connection) {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, summary, importance, metadata FROM nodes \
             WHERE id LIKE 'shanhai-school:%' ORDER BY importance DESC",
        )
        .expect("cmd_schools: SQL prepare");
    let rows: Vec<(String, String, Option<String>, f64, Option<String>)> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, f64>(3)?,
                r.get::<_, Option<String>>(4)?,
            ))
        })
        .expect("cmd_schools: query_map")
        .filter_map(|r| r.ok())
        .collect();

    println!("🏫  学术流派 ({} 个)", rows.len());
    for (_id, title, summary, importance, meta_str) in &rows {
        let meta = meta_str.as_deref().and_then(parse_meta_value);
        let tags = meta.as_ref()
            .and_then(|v| v.get("tags").and_then(|t| t.as_array()))
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
            .unwrap_or_default();
        println!(
            "  {} [imp={:.1}] tags: {}",
            title,
            importance,
            if tags.is_empty() { "—" } else { &tags }
        );
        if let Some(ref s) = summary {
            if s.len() > 100 {
                println!("    {}", &s[..100]);
            } else {
                println!("    {}", s);
            }
        }
        println!();
    }
}

fn cmd_export_geojson(conn: &Connection, path: Option<&str>) {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, summary, metadata FROM nodes \
             WHERE id LIKE 'shanhai-map:%'",
        )
        .expect("cmd_export_geojson: SQL prepare");
    let rows: Vec<_> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })
        .expect("cmd_export_geojson: query_map")
        .filter_map(|r| r.ok())
        .collect();

    let mut features = Vec::new();
    for (id, title, summary, meta_str) in &rows {
        let meta = meta_str
            .as_ref()
            .and_then(|m| serde_json::from_str::<serde_json::Value>(m).ok());
        let loc_str = meta
            .as_ref()
            .and_then(|v| v.get("modern_location"))
            .and_then(|l| l.as_str())
            .unwrap_or("");
        let modern_name = meta
            .as_ref()
            .and_then(|v| v.get("modern_name"))
            .and_then(|l| l.as_str())
            .unwrap_or("");
        let confidence = meta
            .as_ref()
            .and_then(|v| v.get("confidence"))
            .and_then(|c| c.as_f64())
            .unwrap_or(0.0);
        let scholar = meta
            .as_ref()
            .and_then(|v| v.get("attributed_by"))
            .and_then(|a| a.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            })
            .unwrap_or_default();

        if loc_str.is_empty() {
            continue;
        }
        let parts: Vec<&str> = loc_str.split(',').collect();
        if parts.len() != 2 {
            continue;
        }
        let lat: f64 = parts[0].trim().parse().unwrap_or(0.0);
        let lng: f64 = parts[1].trim().parse().unwrap_or(0.0);
        if lat == 0.0 && lng == 0.0 {
            continue;
        }

        let props = serde_json::json!({
            "title": title,
            "modern_name": modern_name,
            "confidence": confidence,
            "scholar": scholar,
            "summary": summary.as_deref().unwrap_or(""),
            "id": id,
        });

        features.push(serde_json::json!({
            "type": "Feature",
            "geometry": {
                "type": "Point",
                "coordinates": [lng, lat]
            },
            "properties": props
        }));
    }

    let geojson = serde_json::json!({
        "type": "FeatureCollection",
        "features": features,
        "metadata": {
            "name": "Shanhai Jing Global Mappings",
            "description": "《山海经》全球地理对应映射 — generated by neotrix-shanhai-query",
            "total_mappings": features.len()
        }
    });

    let output = serde_json::to_string_pretty(&geojson).expect("GeoJSON: to_string_pretty");
    match path {
        Some(p) => {
            std::fs::write(p, &output).expect("Failed to write GeoJSON");
            println!("✅ GeoJSON exported to: {}", p);
            println!("   Features: {}", features.len());
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


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
