//! neotrix-world-novel-ingest — 网络小说世界构建知识吸收
//!
//! 爬取 Royal Road 等小说排行榜，提取优秀小说数据，
//! 分析世界观/剧情/人物设定模式，创建结构化 KB 节点。

use rusqlite::Connection;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn http_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .user_agent("NeoTrix/0.18 (WorldNovelIngest; research)")
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(15))
        .build()
        .expect("Failed to build HTTP client")
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let db_path = format!("{}/.neotrix/knowledge.db", home);
    println!("Opening KB: {}", db_path);
    let conn = Connection::open(&db_path).expect("Failed to open KB");
    let client = http_client();

    let rrc = crawl_royalroad_rankings(&conn, &client);
    let summary = summarize_world_building_patterns(&conn);

    println!("\n📊 世界构建知识吸收结果:");
    println!("   小说吸收: {}", rrc);
    println!("   世界构建模式节点: {}", summary);

    // Final overview
    let total_novel = conn.query_row(
        "SELECT COUNT(*) FROM nodes WHERE domain='royalroad.com' OR domain='world_building'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0);
    println!("   世界知识总节点: {}", total_novel);
}

/// Royal Road 排行榜
const ROYALROAD_RANKINGS: &[(&str, &str)] = &[
    ("best-rated",      "https://www.royalroad.com/fictions/best-rated"),
    ("rising-stars",    "https://www.royalroad.com/fictions/rising-stars"),
    ("trending",        "https://www.royalroad.com/fictions/trending"),
    ("weekly-popular",  "https://www.royalroad.com/fictions/weekly-popular"),
    ("ongoing",         "https://www.royalroad.com/fictions/active-popular"),
    ("completed",       "https://www.royalroad.com/fictions/complete"),
];

const MAX_PER_LIST: usize = 30;

fn crawl_royalroad_rankings(conn: &Connection, client: &reqwest::blocking::Client) -> usize {
    let ts = now();
    let mut total = 0usize;

    for (list_name, url) in ROYALROAD_RANKINGS {
        println!("\n=== 爬取 Royal Road: {} ===", list_name);
        let fetched = fetch_and_parse_list(conn, client, list_name, url, ts);
        println!("  {} 吸收 {} 本小说", list_name, fetched);
        total += fetched;
        std::thread::sleep(Duration::from_secs(2));
    }
    total
}

fn fetch_and_parse_list(
    conn: &Connection,
    client: &reqwest::blocking::Client,
    list_name: &str,
    url: &str,
    ts: i64,
) -> usize {
    let html = match client.get(url).send() {
        Ok(r) if r.status().is_success() => r.text().unwrap_or_default(),
        Ok(r) => { eprintln!("  HTTP {} for {}", r.status(), url); return 0; }
        Err(e) => { eprintln!("  Failed: {}", e); return 0; }
    };

    // Parse fiction list items
    let mut count = 0usize;

    // The actual HTML on Royal Road uses: fiction-list-item row
    for entry in html.split("fiction-list-item row\">").skip(1) {
        if count >= MAX_PER_LIST { break; }

        let title = extract_between(entry, "class=\"font-red-sunglo bold\">", "<")
            .or_else(|| extract_between(entry, "fiction-title\">", "</a>"))
            .map(|s| {
                // Extract from inside the <a> tag
                if let Some(t) = s.rfind('>') {
                    s[t+1..].trim()
                } else { s.trim() }
            })
            .unwrap_or_default();

        // Extract URL from the cover image link
        let href = extract_between(entry, "<a href=\"", "\"")
            .unwrap_or_default();

        // Tags: <a class="label ... fiction-tag" ...>Tag</a>
        let tags: Vec<&str> = entry.split("fiction-tag\"")
            .skip(1)
            .filter_map(|s| {
                // Find the tag text between > and <
                let after_gt = s.find('>').map(|i| i + 1)?;
                let rest = &s[after_gt..];
                let end = rest.find('<')?;
                Some(rest[..end].trim())
            })
            .collect();

        // Rating from star span title attribute
        let rating = extract_between(entry, "title=\"", "\"")
            .or_else(|| {
                // Try aria-label pattern
                extract_between(entry, "aria-label=\"Rating: ", " out of 5\"")
            })
            .unwrap_or_default()
            .trim()
            .to_string();

        // Followers
        let followers = entry.split("Followers")
            .nth(0)
            .and_then(|s| extract_between(s.rsplit('>').next()?, "", ""))
            .unwrap_or("")
            .trim()
            .to_string();

        // Synopsis — in hidden description div
        let synopsis = extract_between(entry, "display: none", "</div>")
            .map(|s| {
                let lines: Vec<&str> = s.lines()
                    .filter(|l| l.contains("<p>") && l.len() > 30)
                    .collect();
                if lines.is_empty() {
                    s.trim().to_string()
                } else {
                    lines[0..lines.len().min(2)].join(" ").trim().to_string()
                }
            })
            .unwrap_or_default();

        if title.is_empty() { continue; }

        let full_url = if href.starts_with('/') {
            format!("https://www.royalroad.com{}", href)
        } else if href.starts_with("http") {
            href.to_string()
        } else {
            format!("https://www.royalroad.com/fiction/{}", href)
        };

        let tag_string = tags.join(", ");
        let followers_clean = followers.replace(',', "").trim().to_string();

        // Create/update KB node
        let node_id = format!("rr-{}-{}", list_name, slugify(title));
        _ = create_or_update_node(conn, &node_id, title, "", &full_url,
            &format!("Royal Road {} novel: {}. Tags: {}. Rating: {}. Followers: {}.",
                list_name, title, tag_string, rating, followers_clean),
            &synopsis,
            list_name, ts);

        // Create world-building analysis
        let tags_owned: Vec<String> = tags.iter().map(|s| s.to_string()).collect();
        create_world_building_analysis(conn, &node_id, title, &tags_owned, list_name, ts);

        count += 1;
    }

    count
}

fn create_world_building_analysis(
    conn: &Connection,
    novel_id: &str,
    title: &str,
    tags: &[String],
    list_name: &str,
    ts: i64,
) {
    // Extract world-building dimensions from tags and title
    let tag_str = tags.join(" ");
    let lower = tag_str.to_lowercase();

    let world_type = if lower.contains("isekai") || lower.contains("portal") || lower.contains("transmigrat") {
        "异世界/穿越"
    } else if lower.contains("litrpg") || lower.contains("game") || lower.contains("system") {
        "游戏化世界"
    } else if lower.contains("dungeon") || lower.contains("cultivat") || lower.contains("xianxia") {
        "修炼/地下城"
    } else if lower.contains("scifi") || lower.contains("sci-fi") || lower.contains("space") || lower.contains("cyberpunk") {
        "科幻未来"
    } else if lower.contains("fantasy") || lower.contains("magic") || lower.contains("myth") {
        "奇幻世界"
    } else if lower.contains("apocalypse") || lower.contains("post-apocalyptic") || lower.contains("survival") {
        "末世生存"
    } else if lower.contains("romance") || lower.contains("slice of life") || lower.contains("contemporary") {
        "现实/浪漫"
    } else {
        "综合设定"
    };

    let magic_system = if lower.contains("hard magic") || lower.contains("system") || lower.contains("litrpg") {
        "硬性系统(有明确规则/数值)"
    } else if lower.contains("soft magic") || lower.contains("mystery") {
        "软性魔法(神秘/不可预测)"
    } else if lower.contains("magic") || lower.contains("cultivat") {
        "修炼体系"
    } else {
        "无/弱魔法系统"
    };

    let progression = if lower.contains("progression") || lower.contains("litrpg") {
        "等级化成长(数值/阶级)"
    } else if lower.contains("training arc") || lower.contains("growth") {
        "阶段性成长"
    } else {
        "剧情驱动(非量化)"
    };

    let setting_scope = if lower.contains("kingdom") || lower.contains("empire") || lower.contains("continent") {
        "大陆/王国级"
    } else if lower.contains("world") || lower.contains("multiverse") {
        "世界/多元宇宙级"
    } else if lower.contains("city") || lower.contains("academy") || lower.contains("school") {
        "城市/学院级"
    } else {
        "区域级"
    };

    let analysis = format!(
        "世界构建分析: {title}
来源排行榜: {list}
世界观类型: {wt}
魔法/力量体系: {ms}
成长模式: {pg}
设定规模: {ss}
标签: {tags}
---
基于 Royal Road 排行榜数据自动分析。分类依据标签关键词匹配。
",
        title = title,
        list = list_name,
        wt = world_type,
        ms = magic_system,
        pg = progression,
        ss = setting_scope,
        tags = tag_str,
    );

    // Store world-building analysis as a Concept node
    let analysis_id = format!("{}-worldbuild", novel_id);
    _ = conn.execute(
        "INSERT OR REPLACE INTO nodes (id, title, node_type, summary, content, url, domain, created_at, updated_at)
         VALUES (?1, ?2, 'Concept', ?3, ?4, ?5, 'world_building', ?6, ?6)",
        rusqlite::params![
            analysis_id,
            format!("{} 世界构建分析", title),
            format!("{}世界观类型: {}, 魔法体系: {}, 成长模式: {}, 设定规模: {}", title, world_type, magic_system, progression, setting_scope),
            analysis,
            format!("https://www.royalroad.com/fiction/{}", slugify(title)),
            ts,
        ],
    ).unwrap_or(0);

    // Create edge to novel if it exists
    _ = conn.execute(
        "INSERT OR IGNORE INTO edges (source_id, target_id, relation_type, weight, description, created_at)
         VALUES (?1, ?2, 'Analyzes', 1.0, ?3, ?4)",
        rusqlite::params![analysis_id, novel_id, format!("World-building analysis for {}", title), ts],
    ).unwrap_or(0);
}

fn create_or_update_node(
    conn: &Connection,
    node_id: &str,
    title: &str,
    author: &str,
    url: &str,
    summary: &str,
    content: &str,
    list_name: &str,
    ts: i64,
) -> Result<(), rusqlite::Error> {
    let clean_content = if content.is_empty() { summary } else { content };
    conn.execute(
        "INSERT OR REPLACE INTO nodes (id, title, node_type, summary, content, url, domain, created_at, updated_at)
         VALUES (?1, ?2, 'Book', ?3, ?4, ?5, 'royalroad.com', ?6, ?6)",
        rusqlite::params![node_id, title, summary, clean_content, url, ts],
    )?;

    // Connect to world-building domain
    let wb_id = "world-building-root";
    conn.execute(
        "INSERT OR IGNORE INTO nodes (id, title, node_type, summary, content, url, domain, created_at, updated_at)
         VALUES ('world-building-root', '世界构建知识库', 'Concept', 'Royal Road 网络小说世界构建模式分析集合。包含世界观、魔法体系、人物设定、剧情结构等知识。', 'World-building knowledge base derived from web novel rankings and analysis.', '', 'world_building', ?1, ?1)",
        rusqlite::params![ts],
    )?;

    // Create edges
    if !author.is_empty() {
        let author_id = format!("author-{}", slugify(author));
        conn.execute(
            "INSERT OR IGNORE INTO nodes (id, title, node_type, summary, content, url, domain, created_at, updated_at)
             VALUES (?1, ?2, 'Person', ?3, ?4, '', 'royalroad.com', ?5, ?5)",
            rusqlite::params![author_id, author,
                format!("Royal Road author: {}", author),
                format!("Author of {} and other web novels on Royal Road.", title),
                ts],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO edges (source_id, target_id, relation_type, weight, description, created_at)
             VALUES (?1, ?2, 'CreatedBy', 1.0, ?3, ?4)",
            rusqlite::params![node_id, author_id, format!("{} written by {}", title, author), ts],
        )?;
    }

    conn.execute(
        "INSERT OR IGNORE INTO edges (source_id, target_id, relation_type, weight, description, created_at)
         VALUES (?1, ?2, 'PartOf', 1.0, ?3, ?4)",
        rusqlite::params![node_id, wb_id, format!("{} ranked on {}", title, list_name), ts],
    )?;

    Ok(())
}

fn summarize_world_building_patterns(conn: &Connection) -> usize {
    let ts = now();

    // Count world-building nodes grouped by world type
    let mut stmt = conn.prepare(
        "SELECT content FROM nodes WHERE domain='world_building' AND node_type='Concept' AND summary LIKE '%世界观类型%'"
    ).expect("prepare failed");

    let mut world_types: Vec<String> = Vec::new();
    let rows = stmt.query_map([], |r| r.get::<_, String>(0)).expect("query failed");
    for row in rows.flatten() {
        for line in row.lines() {
            if line.contains("世界观类型:") {
                let wt = line.split(':').nth(1).unwrap_or("").trim().to_string();
                if !wt.is_empty() && !world_types.contains(&wt) {
                    world_types.push(wt);
                }
            }
        }
    }

    if !world_types.is_empty() {
        let pattern_summary = format!(
            "世界构建模式总结: 共吸收 {} 种世界观类型。\n{}",
            world_types.len(),
            world_types.iter().map(|t| format!("- {}", t)).collect::<Vec<_>>().join("\n")
        );
        _ = conn.execute(
            "INSERT OR IGNORE INTO nodes (id, title, node_type, summary, content, url, domain, created_at, updated_at)
             VALUES ('world-building-patterns', '世界构建模式总结', 'Summary', ?1, ?1, '', 'world_building', ?2, ?2)",
            rusqlite::params![pattern_summary, ts],
        ).unwrap_or(0);
    }

    world_types.len()
}

// === Utility functions ===

fn extract_between<'a>(s: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let s1 = s.find(start)?;
    let rest = &s[s1 + start.len()..];
    let end_pos = rest.find(end)?;
    Some(rest[..end_pos].trim())
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
