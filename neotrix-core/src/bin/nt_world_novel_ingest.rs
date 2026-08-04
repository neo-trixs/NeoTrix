//! neotrix-world-novel-ingest — 网络小说世界构建知识吸收 CLI
//!
//! 薄包装: 逻辑在 `neotrix::l2_world_impl::nt_world_novel`(lib)。
//! 用法:
//!   neotrix-world-novel-ingest                爬 Royal Road 排行榜 + 世界构建模式总结
//!   neotrix-world-novel-ingest --drain N      消费 novel_queue 前 N 本起点书目
//!   neotrix-world-novel-ingest --classify N   为既有 Book 节点离线补世界观分类
//!   neotrix-world-novel-ingest --seed         将数据目录下的 seed 书目入队(novel_queue)

use rusqlite::Connection;
use std::time::{SystemTime, UNIX_EPOCH};

use neotrix::neotrix::l2_world_impl::nt_world_novel::{
    crawl_royalroad_rankings, summarize_world_building_patterns,
    drain_novel_queue, classify_unanalyzed_books, enqueue_novel_book, QidianBook,
};

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

fn open_kb() -> Connection {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let db_path = format!("{}/.neotrix/knowledge.db", home);
    println!("Opening KB: {}", db_path);
    Connection::open(&db_path).expect("Failed to open KB")
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let conn = open_kb();

    let drain_n: usize = args.iter().position(|a| a == "--drain")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let classify_n: usize = args.iter().position(|a| a == "--classify")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if args.iter().any(|a| a == "--seed") {
        let ts = now();
        let seed: Vec<QidianBook> = vec![
            QidianBook {
                title: "斗破苍穹".into(), author: "天蚕土豆".into(), genre: "玄幻".into(),
                sub_genre: "热血".into(), book_url: "https://www.qidian.com/book/1209977/".into(),
                rank: 1, synopsis: "废柴少年逆天改命，斗气大陆修炼斗帝。".into(),
                tags: vec!["斗气".into(), "修炼".into()], word_count: "500万".into(),
                status: "连载".into(), chapter_count: 2000, ranking_name: "月票榜".into(),
            },
            QidianBook {
                title: "凡人修仙传".into(), author: "忘语".into(), genre: "仙侠".into(),
                sub_genre: "凡人流".into(), book_url: "https://www.qidian.com/book/1150838/".into(),
                rank: 2, synopsis: "一介凡人体质，机缘巧合踏上修仙之路。".into(),
                tags: vec!["修仙".into(), "筑基".into()], word_count: "760万".into(),
                status: "连载".into(), chapter_count: 2600, ranking_name: "月票榜".into(),
            },
        ];
        let mut queued = 0;
        for b in &seed {
            if enqueue_novel_book(&conn, b, ts).is_ok() { queued += 1; }
        }
        println!("Seeded {} novels into novel_queue", queued);
    }

    if drain_n > 0 {
        let report = drain_novel_queue(&conn, drain_n);
        println!("Drained novel_queue: {} books, {} edges", report.books, report.edges);
        return;
    }

    if classify_n > 0 {
        let (classified, edges) = classify_unanalyzed_books(&conn, classify_n);
        println!("Classified {} books, {} edges", classified, edges);
        return;
    }

    // Default: Royal Road crawl + summary (original CLI behavior)
    let client = neotrix::neotrix::l2_world_impl::nt_world_novel::http_client();
    let rrc = crawl_royalroad_rankings(&conn, &client);
    let summary = summarize_world_building_patterns(&conn);

    println!("\n📊 世界构建知识吸收结果:");
    println!("   小说吸收: {}", rrc);
    println!("   世界构建模式节点: {}", summary);

    let total_novel = conn.query_row(
        "SELECT COUNT(*) FROM nodes WHERE domain='royalroad.com' OR domain='world_building'",
        [], |r| r.get::<_, i64>(0),
    ).unwrap_or(0);
    println!("   世界知识总节点: {}", total_novel);
}
