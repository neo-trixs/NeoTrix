//! neotrix-shanhai-query — 山海世界数据查询与GeoJSON导出
//!
//! 薄壳: 查询逻辑已吸收归档到 `nt_memory_shanhai::query` (R-P42)。
//!
//! Usage:
//!   cargo run -p neotrix --bin neotrix-shanhai-query stats
//!   cargo run -p neotrix --bin neotrix-shanhai-query peaks
//!   cargo run -p neotrix --bin neotrix-shanhai-query mappings
//!   cargo run -p neotrix --bin neotrix-shanhai-query evidence
//!   cargo run -p neotrix --bin neotrix-shanhai-query schools
//!   cargo run -p neotrix --bin neotrix-shanhai-query export-geojson [path]

#![forbid(unsafe_code)]
use neotrix::l1_action::nt_memory::nt_memory_kb::nt_memory_schema;
use neotrix::l1_action::nt_memory::nt_memory_kb::nt_memory_shanhai::{export_geojson, shanhai_evidence, shanhai_mappings, shanhai_peaks, shanhai_schools, shanhai_stats};
use rusqlite::Connection;

/// 按**字符**截断到 `max` 个字符（不足 `max` 则原样返回）。
///
/// ⛔ 不能写 `&s[..max]`：`max` 是**字节**偏移，`&str` 按非边界字节切会 panic
/// （`byte index … is not a char boundary`），且 `&str[..]` 本身触发
/// `clippy::string_slice`（workspace warn 级 + CI `-D warnings`）。
/// 山海世界的 `summary` 全是中文（《山海经》条目正文），一汉字 3 字节 ⇒
/// 旧代码的 `&m.summary[..120]` / `&summary[..100]` 遇到任何超阈值中文记录就崩。
fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        s.chars().take(max).collect()
    } else {
        s.to_string()
    }
}

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
        if m.summary.chars().count() > 120 {
            println!("    证据: {}...", truncate_chars(&m.summary, 120));
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
        if summary.chars().count() > 100 {
            println!("    {}", truncate_chars(summary, 100));
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

#[cfg(test)]
mod tests {
    use super::truncate_chars;

    /// `n` 个汉字（3 字节/字）——山海世界 `summary` 的真实形态。
    fn cjk(n: usize) -> String {
        "山海经".chars().cycle().take(n).collect()
    }

    #[test]
    fn truncate_chars_cjk_never_splits_a_character() {
        // 300 汉字 = 900 字节 ⇒ 旧代码 `&s[..120]` / `&s[..100]` 必 panic。
        let s = cjk(300);
        let t = truncate_chars(&s, 100);
        assert_eq!(t.chars().count(), 100, "must keep 100 chars");
        assert_eq!(
            t.len(),
            300,
            "100 CJK chars == 300 bytes (byte-cut would be 100)"
        );
        assert_eq!(t, cjk(100));
    }

    #[test]
    fn truncate_chars_ascii_matches_old_byte_semantics() {
        let s = "A".repeat(200);
        assert_eq!(truncate_chars(&s, 120), "A".repeat(120));
        assert_eq!(truncate_chars(&s, 100), "A".repeat(100));
    }

    #[test]
    fn truncate_chars_4byte_emoji_boundary() {
        // 200 个 emoji = 800 字节 ⇒ 任何按字节的定长切分都会落在 4 字节字符中间。
        let s: String = std::iter::repeat('🙂').take(200).collect();
        let t = truncate_chars(&s, 40);
        assert_eq!(t.chars().count(), 40);
        assert_eq!(t.len(), 160);
        let want: String = std::iter::repeat('🙂').take(40).collect();
        assert_eq!(t, want);
    }

    #[test]
    fn truncate_chars_shorter_than_max_is_noop() {
        let s = cjk(30); // 90 字节但只有 30 字符
        assert_eq!(truncate_chars(&s, 100), s);
        assert_eq!(truncate_chars(&s, 30), s, "exactly max ⇒ no cut");
        assert_eq!(truncate_chars(&s, 29), cjk(29));
        assert_eq!(truncate_chars("", 100), "");
    }

    /// 钉死 `cmd_mappings` / `cmd_schools` 的展示语义：
    /// 阈值与截断长度现在都以**字符**计（120 / 100），省略号仍留在调用点。
    #[test]
    fn summary_display_semantics_preserved_on_cjk() {
        // 160 汉字 = 480 字节：旧代码两处都会 panic（120 / 100 皆非边界）。
        let summary = cjk(160);

        // cmd_schools 形态：>100 字符 ⇒ 取前 100 字符，无省略号。
        let printed = if summary.chars().count() > 100 {
            truncate_chars(&summary, 100)
        } else if !summary.is_empty() {
            summary.clone()
        } else {
            String::new()
        };
        assert_eq!(printed.chars().count(), 100);

        // cmd_mappings 形态：>120 字符 ⇒ 取前 120 字符 + "..."。
        let evidence = format!("{}...", truncate_chars(&summary, 120));
        assert!(evidence.ends_with("..."));
        assert_eq!(evidence.chars().count(), 123);

        // 短于阈值时原样输出（空串不输出，见调用点的 else if 分支）。
        let short = cjk(30); // 90 字节 < 100 字节阈值，且只有 30 字符
        assert_eq!(truncate_chars(&short, 100), short);
    }
}
