//! nt_world_novel — 网络小说世界构建知识吸收 (lib)
//!
//! 起点中文网书目注入(port of `scripts/novel-world-absorb.py` 分类+注入层) +
//! novel_queue 队列消费 + 离线重分类。供 daemon `handle_novel_ingest` 周期调用。

use rusqlite::{Connection, params};
use std::time::{SystemTime, UNIX_EPOCH};

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}


// ============================================================================
// 起点中文网 (Qidian) 世界架构吸收 — port of novel-world-absorb.py
// ============================================================================

/// (pattern name, keywords, world tier, power system) — faithful to WORLD_PATTERNS.
pub const WORLD_PATTERNS: &[(&str, &[&str], &str, &str)] = &[
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

pub const REALM_KEYWORDS: &[&str] = &[
    "炼气", "筑基", "金丹", "元婴", "化神", "合体", "渡劫", "大乘", "仙人",
    "斗者", "斗师", "斗王", "斗皇", "斗宗", "斗尊", "斗圣", "斗帝",
    "学徒", "战士", "师级", "王级", "皇级", "帝级", "神级",
    "凡人", "超凡", "圣者", "神话", "半神",
];

/// 从书名/简介/分类/标签分类世界观。genre 命中 +3, 文本命中 +1, 最高胜出,
/// 平局取列表顺序靠前者。Faithful port of `classify_setting`。
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

/// 提取文本中出现的境界关键词。
pub fn extract_realms(text: &str) -> Vec<String> {
    REALM_KEYWORDS.iter().filter(|k| text.contains(**k)).map(|k| k.to_string()).collect()
}

/// 一本被吸收的起点书目(采集侧为外部 qidian-mcp-server)。
#[derive(Debug, Clone)]
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

/// 一次注入结果: 书数 + 边数。
#[derive(Debug, Default, Clone, Copy)]
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

/// 吸收一本书进 KB (Book 节点 + 世界观/力量/境界/作者/分类/榜单 Concept + 边)。
/// port of `ingest_batch`'s per-book body in novel-world-absorb.py。
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
            params![nid, book.title, summary.chars().take(2000).collect::<String>(), content.chars().take(2000).collect::<String>(), book.book_url, ts, meta],
        )
        .unwrap_or(0);
    if stored == 0 {
        return report;
    }
    report.books += 1;

    let sid = qidian_nid(&format!("ws_{}", stype));
    store_qidian_concept(conn, &sid, &format!("世界观: {}", stype), &format!("小说世界观: {}. 层级: {}. 力量: {}", stype, stier, spower), "novel_world_arch", ts);
    if add_qidian_edge(conn, &nid, &sid, "about_topic", 0.85, &format!("世界观类型: {}", stype), ts) {
        report.edges += 1;
    }

    if !spower.contains("Unknown") && !spower.contains("Mixed") {
        let pid = qidian_nid(&format!("ps_{}", spower));
        store_qidian_concept(conn, &pid, &format!("力量体系: {}", spower), &format!("{} 力量体系: {}", book.title, spower), "novel_power", ts);
        if add_qidian_edge(conn, &nid, &pid, "related_to", 0.75, &format!("力量体系: {}", spower), ts) {
            report.edges += 1;
        }
    }

    for r in &realms {
        let rid = qidian_nid(&format!("realm_{}", r));
        store_qidian_concept(conn, &rid, &format!("境界: {}", r), &format!("修炼境界: {}", r), "novel_realm", ts);
        if add_qidian_edge(conn, &nid, &rid, "belongs_to", 0.65, &format!("包含境界: {}", r), ts) {
            report.edges += 1;
        }
    }

    if !book.author.is_empty() {
        let aid = qidian_nid(&format!("author_{}", book.author));
        store_qidian_concept(conn, &aid, &format!("作者: {}", book.author), &format!("起点中文网作者: {}", book.author), "novel_author", ts);
        if add_qidian_edge(conn, &aid, &nid, "developed_by", 0.5, &format!("作者: {}", book.author), ts) {
            report.edges += 1;
        }
    }

    if !book.genre.is_empty() {
        let gid = qidian_nid(&format!("genre_{}", book.genre));
        store_qidian_concept(conn, &gid, &format!("分类: {}", book.genre), &format!("小说品类: {}", book.genre), "novel_genre", ts);
        if add_qidian_edge(conn, &nid, &gid, "categorized", 0.7, &format!("品类: {}", book.genre), ts) {
            report.edges += 1;
        }
    }

    if !book.ranking_name.is_empty() {
        let rid = qidian_nid(&format!("ranking_{}", book.ranking_name));
        store_qidian_concept(conn, &rid, &format!("榜单: {}", book.ranking_name), &format!("起点中文网排行榜: {}", book.ranking_name), "novel_ranking", ts);
        if add_qidian_edge(conn, &nid, &rid, "categorized", 0.4, &format!("来源榜单: {}", book.ranking_name), ts) {
            report.edges += 1;
        }
    }

    report
}

/// 批量吸收(按 title 去重)。port of `ingest_batch` loop。
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
        let mut b = book.clone();
        b.title = title;
        let r = ingest_qidian_book(conn, &b, ts);
        report.books += r.books;
        report.edges += r.edges;
    }
    report
}

// ============================================================================
// novel_queue — 外部采集器(side 脚本 / qidian-mcp-server)入队, daemon 周期消费
// ============================================================================

/// 入队一本已采集的起点书目(按 book_url 幂等)。
pub fn enqueue_novel_book(conn: &Connection, book: &QidianBook, ts: i64) -> rusqlite::Result<()> {
    let id = qidian_nid(&book.book_url);
    let tags = serde_json::to_string(&book.tags).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT OR IGNORE INTO novel_queue
         (id, title, author, genre, sub_genre, book_url, rank, synopsis, tags, word_count, status, chapter_count, ranking_name, discovered_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'pending', ?11, ?12, ?13)",
        params![
            id, book.title, book.author, book.genre, book.sub_genre, book.book_url, book.rank,
            book.synopsis, tags, book.word_count, book.chapter_count, book.ranking_name, ts,
        ],
    )?;
    Ok(())
}

/// 消费 novel_queue: 按 rank 升序取 pending 注入 KB, 完成后标记 completed。
/// 已存在的书目(INSERT OR IGNORE 返回 0)同样标记完成以去重, 避免死循环。
pub fn drain_novel_queue(conn: &Connection, limit: usize) -> QidianIngestReport {
    let mut report = QidianIngestReport::default();
    let mut stmt = match conn.prepare(
        "SELECT id, title, author, genre, sub_genre, book_url, rank, synopsis, tags, word_count, status, chapter_count, ranking_name
         FROM novel_queue WHERE status='pending' ORDER BY rank ASC LIMIT ?1",
    ) {
        Ok(s) => s,
        Err(_) => return report,
    };
    let items: Vec<(String, QidianBook)> = stmt
        .query_map([&(limit as i64)], |r| {
            let tags_json: String = r.get(8)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            let book = QidianBook {
                title: r.get(1)?,
                author: r.get(2)?,
                genre: r.get(3)?,
                sub_genre: r.get(4)?,
                book_url: r.get(5)?,
                rank: r.get(6)?,
                synopsis: r.get(7)?,
                tags,
                word_count: r.get(9)?,
                status: r.get(10)?,
                chapter_count: r.get(11)?,
                ranking_name: r.get(12)?,
            };
            Ok((r.get::<_, String>(0)?, book))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    for (id, book) in items {
        let r = ingest_qidian_book(conn, &book, now());
        report.books += r.books;
        report.edges += r.edges;
        let _ = conn.execute(
            "UPDATE novel_queue SET status='completed', last_attempt=?1 WHERE id=?2",
            params![now(), id],
        );
    }
    report
}

/// 离线消费者: 为缺少世界构建分析的既有 Book 节点(qidian/royalroad 域)补分类
/// Concept 节点 + edge。无网络依赖, daemon 周期兜底分类。
pub fn classify_unanalyzed_books(conn: &Connection, limit: usize) -> (usize, usize) {
    let ts = now();
    let mut stmt = match conn.prepare(
        "SELECT id, title, COALESCE(summary,''), COALESCE(content,'')
         FROM nodes
         WHERE node_type='Book' AND (domain='qidian.com' OR domain='royalroad.com')
           AND id NOT IN (SELECT source_id FROM edges WHERE relation_type='about_topic')
           AND id NOT IN (SELECT target_id FROM edges WHERE relation_type='Analyzes')
         LIMIT ?1",
    ) {
        Ok(s) => s,
        Err(_) => return (0, 0),
    };
    let rows = stmt
        .query_map([&(limit as i64)], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect::<Vec<_>>())
        .unwrap_or_default();

    let mut classified = 0usize;
    let mut edges = 0usize;
    for (id, title, summary, content) in rows {
        let text = format!("{} {}", summary, content);
        let (stype, stier, spower) = classify_setting(&title, &text, "", &[]);
        let cid = qidian_nid(&format!("ws_{}_{}", id, stype));
        store_qidian_concept(
            conn, &cid,
            &format!("世界观: {}", stype),
            &format!("{} 世界观: {}. 层级: {}. 力量: {}", title, stype, stier, spower),
            "novel_world_arch", ts,
        );
        if add_qidian_edge(conn, &id, &cid, "about_topic", 0.8, &format!("世界观类型: {}", stype), ts) {
            edges += 1;
        }
        classified += 1;
    }
    (classified, edges)
}

// === helpers ===

fn store_qidian_concept(conn: &Connection, nid: &str, title: &str, summary: &str, domain: &str, ts: i64) {
    let summary_owned = summary.chars().take(2000).collect::<String>();
    _ = conn.execute(
        "INSERT OR IGNORE INTO nodes (id,node_type,title,summary,content,domain,language,confidence,importance,created_at,updated_at)
         VALUES (?1,'Concept',?2,?3,?3,?4,'zh',0.9,0.5,?5,?5)",
        params![nid, title, summary_owned, domain, ts],
    );
}

fn add_qidian_edge(conn: &Connection, src: &str, tgt: &str, rel: &str, w: f64, desc: &str, ts: i64) -> bool {
    if src.is_empty() || tgt.is_empty() || src == tgt {
        return false;
    }
    let eid = format!("n_{}_{}_{}", &src[..src.len().min(12)], &tgt[..tgt.len().min(12)], &rel[..rel.len().min(6)]);
    conn.execute(
        "INSERT OR IGNORE INTO edges (id,source_id,target_id,relation_type,weight,created_at,description) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![eid, src, tgt, rel, w, ts, desc],
    )
    .map(|_| true)
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NODES_EDGES: &str = "CREATE TABLE nodes (id TEXT PRIMARY KEY, node_type TEXT, title TEXT, summary TEXT, content TEXT, url TEXT, domain TEXT, language TEXT, confidence REAL, importance REAL, created_at INTEGER, updated_at INTEGER, metadata TEXT);
        CREATE TABLE edges (id TEXT PRIMARY KEY, source_id TEXT, target_id TEXT, relation_type TEXT, weight REAL, created_at INTEGER, description TEXT);";

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
        conn.execute_batch(NODES_EDGES).unwrap();
        let book = sample_book();
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
        conn.execute_batch(NODES_EDGES).unwrap();
        let mut b1 = sample_book();
        let mut b2 = sample_book();
        b1.title = "同名书".to_string();
        b2.title = "同名书".to_string();
        b2.rank = 2;
        let books = vec![b1, b2];
        let report = ingest_qidian_batch(&mut conn, &books, 999);
        let book_count: i64 = conn.query_row("SELECT COUNT(*) FROM nodes WHERE node_type='Book'", [], |r| r.get(0)).unwrap();
        assert_eq!(book_count, 1);
        assert!(report.books >= 1);
    }

    #[test]
    fn test_enqueue_and_drain_novel_queue() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(NODES_EDGES).unwrap();
        conn.execute_batch(
            "CREATE TABLE novel_queue (
                id TEXT PRIMARY KEY, title TEXT NOT NULL, author TEXT, genre TEXT, sub_genre TEXT,
                book_url TEXT NOT NULL UNIQUE, rank INTEGER DEFAULT 0, synopsis TEXT, tags TEXT,
                word_count TEXT, status TEXT DEFAULT 'pending', chapter_count INTEGER DEFAULT 0,
                ranking_name TEXT, discovered_at INTEGER NOT NULL, last_attempt INTEGER,
                retry_count INTEGER DEFAULT 0, error_message TEXT);",
        )
        .unwrap();

        let book = sample_book();
        enqueue_novel_book(&conn, &book, 100).unwrap();
        // idempotent enqueue
        enqueue_novel_book(&conn, &book, 100).unwrap();

        let report = drain_novel_queue(&conn, 10);
        assert!(report.books >= 1);
        assert!(report.edges >= 3);

        let pending: i64 = conn.query_row("SELECT COUNT(*) FROM novel_queue WHERE status='pending'", [], |r| r.get(0)).unwrap();
        assert_eq!(pending, 0);
        let book_count: i64 = conn.query_row("SELECT COUNT(*) FROM nodes WHERE node_type='Book'", [], |r| r.get(0)).unwrap();
        assert_eq!(book_count, 1);
    }

    #[test]
    fn test_classify_unanalyzed_books() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(NODES_EDGES).unwrap();
        conn.execute(
            "INSERT INTO nodes (id,node_type,title,summary,content,domain,language,confidence,importance,created_at,updated_at)
             VALUES ('rr-x','Book','斗破苍穹 星域版','斗气大陆修炼，废柴逆袭。','渡劫飞升。','royalroad.com','zh',0.8,0.6,1,1)",
            [],
        )
        .unwrap();

        let (classified, edges) = classify_unanalyzed_books(&conn, 10);
        assert_eq!(classified, 1);
        assert_eq!(edges, 1);

        let concept_count: i64 = conn.query_row("SELECT COUNT(*) FROM nodes WHERE node_type='Concept'", [], |r| r.get(0)).unwrap();
        assert_eq!(concept_count, 1);

        // Second run is idempotent — already analyzed
        let (classified2, _) = classify_unanalyzed_books(&conn, 10);
        assert_eq!(classified2, 0);
    }

    fn sample_book() -> QidianBook {
        QidianBook {
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
        }
    }
}
