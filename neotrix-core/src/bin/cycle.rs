use std::time::Instant;

use neotrix::neotrix::nt_memory_kb::KnowledgeBase;

fn main() {
    println!("╔══════════════════════════════════════════════╗");
    println!("║  NeoTrix 知识维护循环 — 去重 + 爬取 + 统计  ║");
    println!("╚══════════════════════════════════════════════╝");

    let kb = KnowledgeBase::open(None).expect("KB open failed");
    let overall = Instant::now();

    let before = kb.stats().expect("Stats before");
    println!("\n  当前: {} 节点, {} 边, {} 爬取待处理",
        before.total_nodes, before.total_edges, before.crawl_pending);

    println!("\n━━━ 1. 去重 ━━━");
    match kb.dedup_nodes() {
        Ok(n) => println!("  合并 {} 个重复节点", n),
        Err(e) => println!("  ⚠ 去重: {}", e),
    }

    println!("\n━━━ 2. 爬取队列处理 (60 条) ━━━");
    let c_start = Instant::now();
    match kb.run_crawl_cycle(60) {
        Ok(report) => {
            println!("  尝试: {} | 完成: {} | 失败: {}",
                report.attempted, report.completed, report.failed);
            println!("  新建节点: {} | 新建边: {}", report.nodes_created, report.edges_created);
            println!("  耗时: {:.1}s", c_start.elapsed().as_secs_f64());
        }
        Err(e) => println!("  ⚠ 爬取循环: {}", e),
    }

    let after = kb.stats().expect("Stats final");

    println!("\n╔══════════════════════════════════════════════╗");
    println!("║  维护完成                                     ║");
    println!("╚══════════════════════════════════════════════╝");
    println!("  耗时:          {:.1}s", overall.elapsed().as_secs_f64());
    println!("  节点:          {} → {} ({} 新增)",
        before.total_nodes, after.total_nodes, after.total_nodes - before.total_nodes);
    println!("  边:            {} → {} ({} 新增)",
        before.total_edges, after.total_edges, after.total_edges - before.total_edges);
    println!("  爬取待处理:    {}", after.crawl_pending);

    println!("\n  类型分布:");
    let mut types: Vec<_> = after.by_type.iter().collect();
    types.sort_by(|a, b| b.1.cmp(&a.1));
    for (t, c) in &types {
        let before_c = before.by_type.iter().find(|(k, _)| k == t).map(|(_, v)| v).copied().unwrap_or(0);
        let delta = if c > &before_c { format!(" +{}", c - before_c) } else { String::new() };
        println!("    {:>15}: {:>4}{}", t, c, delta);
    }
}
