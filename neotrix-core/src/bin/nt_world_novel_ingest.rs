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

// ============================================================================
// Qidian (起点中文网) world-architecture absorption
// ============================================================================
// Port of `scripts/novel-world-absorb.py` classification + KB-injection layer.
// The scrape side (playwright-driven qidian-mcp-server) is external; this module
// consumes already-fetched book records via `ingest_qidian_batch` and produces
// the same Book/Concept nodes + edges into the KB.

/// (pattern name, keywords, world tier, power system) — faithful to WORLD_PATTERNS.
const WORLD_PATTERNS: &[(&str, &[&str], &str, &str)] = &[
    ("Xianxia", &["修真", "修仙", "仙侠", "渡劫", "元婴", "金丹", "飞升", "灵根", "法宝", "功法", "道心", "天人", "仙", "凡"], "Immortal Ascension", "Qi Cultivation (修真)"),
    ("Xuanhuan", &["玄幻", "斗气", "魔法", "魔兽", "异界", "大陆", "位面", "神格", "领域", "斗帝", "神", "魔"], "Fantasy World", "Magic / Battle Qi (斗气魔法)"),
    ("Urban Supernatural", &["都市", "异能", "超能力", "现代", "校园", "娱乐圈", "重生", "保镖", "医生", "警察"], "Modern Earth", "Superpower (异能)"),
    ("Science Fiction", &["科幻", "星际", "机甲", "未来", "赛博", "人工智能", "基因", "宇宙", "时空", "星舰", "机器人"], "Interstellar / Futuristic", "Technology (科技)"),
    ("Historical", &["历史", "三国", "穿越", "古代", "王朝", "争霸", "帝王", "架空", "重生", "民国"], "Historical Earth", "Strategy / Martial Arts"),
    ("Game World", &["游戏", "电竞", "虚拟现实", "网游", "副本", "技能", "属性", "面板", "职业", "系统"], "Virtual World", "Game Mechanics (游戏系统)"),
    ("Wuxia", &["武侠", "江湖", "内力", "武功", "武林", "掌门", "宗师", "帮派", "侠客", "剑"], "Martial World", "Internal Energy (真气武学)"),
    ("Light Novel", &["轻小说", "二次元", "动漫", "同人", "综漫", "冒险", "异世界"], "Anime World", "Various / System"),
    ("Mystery / Horror", &["悬疑", "恐怖", "灵异", "惊悚", "盗墓", "鬼怪", "死亡游戏", "推理", "侦探"], "Dark Modern", "Supernatural / Curses"),
    ("Steampunk / Occult", &["蒸汽", "神秘学", "克苏鲁", "魔藥", "符文", "教会", "超凡", "非凡"], "Alternate History / Arcane", "Occult / Alchemy"),
];

const REALM_KEYWORDS: &[&str] = &[
    "炼气", "筑基", "金丹", "元婴", "化神", "合体", "渡劫", "大乘", "仙人",
    "斗者", "斗师", "斗王", "斗皇", "斗宗", "斗尊", "斗圣", "斗帝",
    "学徒", "战士", "师级", "王级", "皇级", "帝级", "神级",
    "凡人", "超凡", "圣者", "神话", "半神",
];

/// Classify a novel's world architecture from title/summary/genre/tags.
/// Faithful port of `classify_setting`: genre hits score 3, text hits score 1,
/// highest total wins, ties keep the earlier pattern (list order).
pub fn classify_setting(title: &str, summary: &str, genre: &str, tags: &[String]) -> (String, String, String) {
    let text = format!("{} {} {} {}", title, summary, genre, tags.join(" ")).to_lowercase();
    let genre_lower = genre.to_lowercase();
    let mut best_score = 0usize;
    let mut best: Option<(&str, &str, &str)> = None;
    for (name, keywords, tier, power) in WORLD_PATTERNS {
        let mut score = 0usize;
        for k in *keywords {
            let kl = k.to_lowercase();
            if genre_lower.contains(&kl) {
                score += 3;
            }
            if text.contains(&kl) {
                score += 1;
            }
        }
        if score > best_score {
            best_score = score;
            best = Some((name, tier, power));
        }
    }
    match best {
        Some((n, t, p)) => (n.to_string(), t.to_string(), p.to_string()),
        None => ("General".to_string(), "Unknown".to_string(), "Mixed".to_string()),
    }
}

/// Extract realm keywords present in text. Faithful port of `extract_realms`.
pub fn extract_realms(text: &str) -> Vec<String> {
    REALM_KEYWORDS.iter().filter(|k| text.contains(**k)).map(|k| k.to_string()).collect()
}

/// A single scraped Qidian book record (subset consumed by the KB injection).
pub struct QidianBook {
    pub title: String,
    pub author: String,
    pub genre: String,
    pub sub_genre: String,
    pub book_url: String,
    pub rank: i64,
    pub synopsis: String,
    pub tags: Vec<String>,
    pub word_count: String,
    pub status: String,
    pub chapter_count: i64,
    pub ranking_name: String,
}

/// One KB-injection result: number of books and edges created.
#[derive(Debug, Default)]
pub struct QidianIngestReport {
    pub books: usize,
    pub edges: usize,
}

/// Hash a string to a deterministic `nt-` node id (md5 first 20 hex).
fn qidian_nid(url: &str) -> String {
    use md5::{Digest, Md5};
    let mut h = Md5::new();
    h.update(url.as_bytes());
    let digest = h.finalize();
    let hex: String = digest.iter().map(|b| format!("{:02x}", b)).collect();
    format!("nt-{}", &hex[..20])
}

/// Absorb one book record into the KB (Book node + Concept nodes + edges).
/// Faithful port of `ingest_batch`'s per-book body in novel-world-absorb.py.
pub fn ingest_qidian_book(
    conn: &Connection,
    book: &QidianBook,
    ts: i64,
) -> QidianIngestReport {
    let mut report = QidianIngestReport::default();

    let full_cat = format!("{} {}", book.genre, book.sub_genre).trim().to_string();
    let (stype, stier, spower) = classify_setting(&book.title, &book.synopsis, &full_cat, &book.tags);
    let realms = extract_realms(&format!("{} {} {}", book.title, book.synopsis, full_cat));

    let nid = qidian_nid(&book.book_url);

    let mut content_parts = vec![
        format!("来源: 起点中文网({}) | 排名: #{}", book.ranking_name, book.rank),
        format!("作者: {}", book.author),
        format!("分类: {}", full_cat),
        format!("字数: {} | 章节: {} | 状态: {}", book.word_count, book.chapter_count, book.status),
        format!("世界观: {} | 层级: {} | 力量: {}", stype, stier, spower),
    ];
    if !realms.is_empty() {
        content_parts.push(format!("境界: {}", realms.join(", ")));
    }
    let content = content_parts.join("\n");
    let summary = if book.synopsis.is_empty() { &content } else { &book.synopsis };

    // Book node
    let meta = serde_json::json!({
        "author": book.author,
        "category": full_cat,
        "source": "qidian",
    })
    .to_string();
    let stored = conn
        .execute(
            "INSERT OR IGNORE INTO nodes (id,node_type,title,summary,content,url,domain,language,confidence,importance,created_at,updated_at,metadata)
             VALUES (?1,'Book',?2,?3,?4,?5,'qidian.com','zh',0.8,0.6,?6,?6,?7)",
            rusqlite::params![nid, book.title, summary.chars().take(2000).collect::<String>(), content.chars().take(2000).collect::<String>(), book.book_url, ts, meta],
        )
        .unwrap_or(0);
    if stored == 0 {
        return report;
    }
    report.books += 1;

    // World setting concept
    let sid = qidian_nid(&format!("ws_{}", stype));
    store_qidian_concept(conn, &sid, &format!("世界观: {}", stype), &format!("小说世界观: {}. 层级: {}. 力量: {}", stype, stier, spower), "novel_world_arch", ts);
    if add_qidian_edge(conn, &nid, &sid, "about_topic", 0.85, &format!("世界观类型: {}", stype), ts) {
        report.edges += 1;
    }

    // Power system
    if !spower.contains("Unknown") && !spower.contains("Mixed") {
        let pid = qidian_nid(&format!("ps_{}", spower));
        store_qidian_concept(conn, &pid, &format!("力量体系: {}", spower), &format!("{} 力量体系: {}", book.title, spower), "novel_power", ts);
        if add_qidian_edge(conn, &nid, &pid, "related_to", 0.75, &format!("力量体系: {}", spower), ts) {
            report.edges += 1;
        }
    }

    // Realms
    for r in &realms {
        let rid = qidian_nid(&format!("realm_{}", r));
        store_qidian_concept(conn, &rid, &format!("境界: {}", r), &format!("修炼境界: {}", r), "novel_realm", ts);
        if add_qidian_edge(conn, &nid, &rid, "belongs_to", 0.65, &format!("包含境界: {}", r), ts) {
            report.edges += 1;
        }
    }

    // Author
    if !book.author.is_empty() {
        let aid = qidian_nid(&format!("author_{}", book.author));
        store_qidian_concept(conn, &aid, &format!("作者: {}", book.author), &format!("起点中文网作者: {}", book.author), "novel_author", ts);
        if add_qidian_edge(conn, &aid, &nid, "developed_by", 0.5, &format!("作者: {}", book.author), ts) {
            report.edges += 1;
        }
    }

    // Genre category
    if !book.genre.is_empty() {
        let gid = qidian_nid(&format!("genre_{}", book.genre));
        store_qidian_concept(conn, &gid, &format!("分类: {}", book.genre), &format!("小说品类: {}", book.genre), "novel_genre", ts);
        if add_qidian_edge(conn, &nid, &gid, "categorized", 0.7, &format!("品类: {}", book.genre), ts) {
            report.edges += 1;
        }
    }

    // Ranking source
    if !book.ranking_name.is_empty() {
        let rid = qidian_nid(&format!("ranking_{}", book.ranking_name));
        store_qidian_concept(conn, &rid, &format!("榜单: {}", book.ranking_name), &format!("起点中文网排行榜: {}", book.ranking_name), "novel_ranking", ts);
        if add_qidian_edge(conn, &nid, &rid, "categorized", 0.4, &format!("来源榜单: {}", book.ranking_name), ts) {
            report.edges += 1;
        }
    }

    report
}

/// Absorb a batch of scraped books (dedup by title). Port of `ingest_batch` loop.
pub fn ingest_qidian_batch(
    conn: &Connection,
    books: &[QidianBook],
    ts: i64,
) -> QidianIngestReport {
    let mut report = QidianIngestReport::default();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for book in books {
        let title = book.title.trim().to_string();
        if title.is_empty() || seen.contains(&title) {
            continue;
        }
        seen.insert(title.clone());
        let mut b = book.clone_into_qidian();
        b.title = title;
        let r = ingest_qidian_book(conn, &b, ts);
        report.books += r.books;
        report.edges += r.edges;
    }
    report
}

impl QidianBook {
    fn clone_into_qidian(&self) -> QidianBook {
        QidianBook {
            title: self.title.clone(),
            author: self.author.clone(),
            genre: self.genre.clone(),
            sub_genre: self.sub_genre.clone(),
            book_url: self.book_url.clone(),
            rank: self.rank,
            synopsis: self.synopsis.clone(),
            tags: self.tags.clone(),
            word_count: self.word_count.clone(),
            status: self.status.clone(),
            chapter_count: self.chapter_count,
            ranking_name: self.ranking_name.clone(),
        }
    }
}

fn store_qidian_concept(conn: &Connection, nid: &str, title: &str, summary: &str, domain: &str, ts: i64) {
    let summary_owned = summary.chars().take(2000).collect::<String>();
    _ = conn.execute(
        "INSERT OR IGNORE INTO nodes (id,node_type,title,summary,content,domain,language,confidence,importance,created_at,updated_at)
         VALUES (?1,'Concept',?2,?3,?3,?4,'zh',0.9,0.5,?5,?5)",
        rusqlite::params![nid, title, summary_owned, domain, ts],
    );
}

fn add_qidian_edge(conn: &Connection, src: &str, tgt: &str, rel: &str, w: f64, desc: &str, ts: i64) -> bool {
    if src.is_empty() || tgt.is_empty() || src == tgt {
        return false;
    }
    let eid = format!("n_{}_{}_{}", &src[..src.len().min(12)], &tgt[..tgt.len().min(12)], &rel[..rel.len().min(6)]);
    conn.execute(
        "INSERT OR IGNORE INTO edges (id,source_id,target_id,relation_type,weight,created_at,description) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        rusqlite::params![eid, src, tgt, rel, w, ts, desc],
    )
    .map(|_| true)
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_setting_xuanhuan_by_tag() {
        let (name, tier, power) = classify_setting("斗破", "", "玄幻", &["斗气".to_string()]);
        assert_eq!(name, "Xuanhuan");
        assert_eq!(tier, "Fantasy World");
        assert!(power.contains("斗气魔法"));
    }

    #[test]
    fn test_classify_setting_default_general() {
        let (name, tier, power) = classify_setting("偶然随笔", "平淡无物的一个故事", "随笔", &[]);
        assert_eq!(name, "General");
        assert_eq!(tier, "Unknown");
        assert_eq!(power, "Mixed");
    }

    #[test]
    fn test_extract_realms_subset() {
        let realms = extract_realms("主角修炼金丹境与元婴，最后渡劫飞升");
        assert!(realms.contains(&"金丹".to_string()));
        assert!(realms.contains(&"元婴".to_string()));
        assert!(realms.contains(&"渡劫".to_string()));
        assert!(!realms.contains(&"斗帝".to_string()));
    }

    #[test]
    fn test_ingest_qidian_book_nodes_and_edges() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE nodes (id TEXT PRIMARY KEY, node_type TEXT, title TEXT, summary TEXT, content TEXT, url TEXT, domain TEXT, language TEXT, confidence REAL, importance REAL, created_at INTEGER, updated_at INTEGER, metadata TEXT);
             CREATE TABLE edges (id TEXT PRIMARY KEY, source_id TEXT, target_id TEXT, relation_type TEXT, weight REAL, created_at INTEGER, description TEXT);",
        )
        .unwrap();
        let book = QidianBook {
            title: "斗破苍穹".to_string(),
            author: "天蚕土豆".to_string(),
            genre: "玄幻".to_string(),
            sub_genre: "热血".to_string(),
            book_url: "https://www.qidian.com/book/1209977/".to_string(),
            rank: 1,
            synopsis: "废柴少年逆天改命，斗气大陆修炼斗帝，渡劫化神。".to_string(),
            tags: vec!["斗气".to_string(), "修炼".to_string()],
            word_count: "500万".to_string(),
            status: "连载".to_string(),
            chapter_count: 2000,
            ranking_name: "月票榜".to_string(),
        };
        let report = ingest_qidian_book(&mut conn, &book, 999);
        assert!(report.books >= 1);
        assert!(report.edges >= 3, "expected at least world/about + power/book edges, got {}", report.edges);

        let book_count: i64 = conn.query_row("SELECT COUNT(*) FROM nodes WHERE node_type='Book'", [], |r| r.get(0)).unwrap();
        assert_eq!(book_count, 1);
        let concept_count: i64 = conn.query_row("SELECT COUNT(*) FROM nodes WHERE node_type='Concept'", [], |r| r.get(0)).unwrap();
        assert!(concept_count >= 3, "concepts: {}", concept_count);
        let edge_count: i64 = conn.query_row("SELECT COUNT(*) FROM edges", [], |r| r.get(0)).unwrap();
        assert_eq!(edge_count as usize, report.edges);
    }

    #[test]
    fn test_ingest_qidian_batch_dedup() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE nodes (id TEXT PRIMARY KEY, node_type TEXT, title TEXT, summary TEXT, content TEXT, url TEXT, domain TEXT, language TEXT, confidence REAL, importance REAL, created_at INTEGER, updated_at INTEGER, metadata TEXT);
             CREATE TABLE edges (id TEXT PRIMARY KEY, source_id TEXT, target_id TEXT, relation_type TEXT, weight REAL, created_at INTEGER, description TEXT);",
        )
        .unwrap();
        let books = vec![
            QidianBook { title: "同名书".to_string(), author: "a".to_string(), genre: "玄幻".into(), sub_genre: "".into(), book_url: "https://a".into(), rank: 1, synopsis: "".into(), tags: vec![], word_count: "".into(), status: "".into(), chapter_count: 0, ranking_name: "月票榜".into() },
            QidianBook { title: "同名书".to_string(), author: "a".to_string(), genre: "玄幻".into(), sub_genre: "".into(), book_url: "https://a".into(), rank: 2, synopsis: "".into(), tags: vec![], word_count: "".into(), status: "".into(), chapter_count: 0, ranking_name: "月票榜".into() },
        ];
        let report = ingest_qidian_batch(&mut conn, &books, 999);
        let book_count: i64 = conn.query_row("SELECT COUNT(*) FROM nodes WHERE node_type='Book'", [], |r| r.get(0)).unwrap();
        assert_eq!(book_count, 1);
        assert!(report.books >= 1);
    }
}
