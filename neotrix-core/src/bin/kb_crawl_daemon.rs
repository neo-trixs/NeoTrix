#![forbid(unsafe_code)]
use std::time::{Duration, Instant};

use neotrix::neotrix::nt_memory_kb::KnowledgeBase;

fn main() {
    println!("╔═══════════════════════════════════════════════════╗");
    println!("║  NeoTrix KnowledgeBase 自主爬取守护进程            ║");
    println!("║  每6小时: Wikipedia + ArXiv + GitHub 管道         ║");
    println!("╚═══════════════════════════════════════════════════╝");

    let kb = KnowledgeBase::open(None).expect("Failed to open KnowledgeBase");
    println!("  DB: {}", kb.db_path.display());

    let interval = Duration::from_secs(6 * 3600);
    let mut cycle = 0u64;
    // D74/R-P38 停止条件: 连续失败熔断, 防止守护空转
    const MAX_CONSECUTIVE_FAILURES: u32 = 5;
    let mut consecutive_failures = 0u32;

    loop {
        cycle += 1;
        let start = Instant::now();
        println!("\n━━━ Cycle #{} @ {} ━━━", cycle, now_str());

        let before = kb.stats().expect("Stats failed");

        match kb.run_crawl_cycle(10) {
            Ok(report) => {
                consecutive_failures = 0;
                println!("  Crawl: attempted={}, completed={}, failed={}, nodes={}, edges={}",
                    report.attempted, report.completed, report.failed,
                    report.nodes_created, report.edges_created);
                if !report.errors.is_empty() {
                    for (url, err) in &report.errors[..report.errors.len().min(3)] {
                        println!("    ⚠ {}: {}", url, err);
                    }
                }
            },
            Err(e) => {
                consecutive_failures += 1;
                println!("  ⚠ Crawl cycle: {} (连续失败 {}/{})", e, consecutive_failures, MAX_CONSECUTIVE_FAILURES);
                if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    println!("  ✗ 连续 {} 次 crawl 失败 — 熔断退出", MAX_CONSECUTIVE_FAILURES);
                    std::process::exit(1);
                }
            },
        }

        let after = kb.stats().expect("Stats failed");
        let elapsed = start.elapsed();
        println!("  Status: {} nodes, {} edges, {} pending (took {:.2}s)",
            after.total_nodes, after.total_edges, after.crawl_pending, elapsed.as_secs_f64());
        println!("  Δ: {} nodes, {} edges",
            after.total_nodes - before.total_nodes,
            after.total_edges - before.total_edges);

        println!("\n── 休眠 {}h 至 {} ──", interval.as_secs() / 3600, future_str(interval));
        std::thread::sleep(interval);
    }
}

fn now_str() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn future_str(dur: Duration) -> String {
    (chrono::Local::now() + chrono::Duration::from_std(dur).unwrap_or_default())
        .format("%H:%M:%S").to_string()
}
