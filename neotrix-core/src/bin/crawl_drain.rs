use neotrix::neotrix::nt_memory_kb::KnowledgeBase;
use std::time::{Duration, Instant};

const DEFAULT_BATCH: usize = 500;
const DEFAULT_CYCLES: u64 = 100;
const DEFAULT_WORKERS: usize = 48;

fn main() {
    let batch: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(DEFAULT_BATCH);
    let cycles: u64 = std::env::args()
        .nth(2)
        .and_then(|a| a.parse().ok())
        .unwrap_or(DEFAULT_CYCLES);
    let workers: usize = std::env::args()
        .nth(3)
        .and_then(|a| a.parse().ok())
        .unwrap_or(DEFAULT_WORKERS);
    let mode = std::env::args().nth(4).unwrap_or_else(|| "drain".into());
    let fetch_links = mode == "full";

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  NeoTrix 爬取队列排空 v2 — 多进程并行加速                  ║");
    println!("║                                                             ║");
    println!(
        "║  批次大小: {}                                              ║",
        batch
    );
    println!(
        "║  循环数:   {}                                              ║",
        cycles
    );
    println!(
        "║  并行数:   {} workers                                      ║",
        workers
    );
    println!(
        "║  模式:     {}                                             ║",
        if fetch_links { "full (发现新链接)" } else { "drain (只排空)" }
    );
    println!("╚══════════════════════════════════════════════════════════════╝");

    let kb = KnowledgeBase::open(None).expect("KB open failed");
    let start = Instant::now();

    // Phase 1: Reset stuck "processing" items
    println!("\n━━━ Phase 1: 重置卡住的 processing 条目 ━━━");
    let reset = kb.reset_stuck_items().unwrap_or(0);
    println!("  已重置 {} 个卡住条目", reset);

    // Phase 2: Show queue stats
    let before_pending = {
        let conn = kb.conn.lock().expect("Lock");
        conn.query_row(
            "SELECT COUNT(*) FROM crawl_queue WHERE status = 'pending'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0i64)
    };
    println!("\n━━━ Phase 2: 当前待处理队列 ━━━");
    println!("  待处理: {} URLs", before_pending);

    // Phase 2b: Purge skip domains — skip in full mode
    if !fetch_links {
        println!("\n━━━ Phase 2b: 清除已知无用域名 ━━━");
        let purged = kb.purge_skip_domains().unwrap_or(0);
        println!("  已清除 {} URLs", purged);
        let after_purge = {
            let conn = kb.conn.lock().expect("Lock");
            conn.query_row("SELECT COUNT(*) FROM crawl_queue WHERE status='pending'", [], |r| r.get(0)).unwrap_or(0i64)
        };
        println!("  清除后待处理: {} URLs", after_purge);
    }

    // Phase 2c: Quick connectivity validation — skip in full mode (seed URLs need HTTP)
    if !fetch_links {
        println!("\n━━━ Phase 2c: 连通性验证 (HEAD, 1s timeout) ━━━");
        let v_start = std::time::Instant::now();
        match kb.validate_urls(workers) {
            Ok((alive, dead)) => {
                println!("  ✅ 存活: {}, ❌ 失效: {} | [{:.1}s]",
                    alive, dead, v_start.elapsed().as_secs_f64());
            }
            Err(e) => println!("  ⚠ 验证错误: {}", e),
        }
        let after_validate = {
            let conn = kb.conn.lock().expect("Lock");
            conn.query_row("SELECT COUNT(*) FROM crawl_queue WHERE status='pending'", [], |r| r.get(0)).unwrap_or(0i64)
        };
        println!("  验证后待处理: {} URLs", after_validate);
    }

    // Phase 2d: Bulk-delete all skip-pattern URLs — skip in full mode
    if !fetch_links {
        println!("\n━━━ Phase 2d: 批量清除 skip_url 匹配项 ━━━");
        let s_start = std::time::Instant::now();
        let skipped = kb.purge_all_skip_patterns().unwrap_or(0);
        let after_skip = {
            let conn = kb.conn.lock().expect("Lock");
            conn.query_row("SELECT COUNT(*) FROM crawl_queue WHERE status='pending'", [], |r| r.get(0)).unwrap_or(0i64)
        };
        println!("  清除 {} URLs | [{:.1}s]", skipped, s_start.elapsed().as_secs_f64());
        println!("  清除后待处理: {} URLs", after_skip);
    }

    // Phase 3: Crawl cycles (parallel)
    println!(
        "\n━━━ Phase 3: 并行{} ({} URLs/cycle, {} workers, {} cycles) ━━━",
        if fetch_links { "爬取+发现新链接" } else { "排空" },
        batch, workers, cycles
    );
    let mut total_processed = 0u64;
    let mut total_absorbed = 0u64;
    let mut total_edges = 0u64;
    let mut peak_throughput = 0.0;

    for cycle in 1..=cycles {
        let c_start = Instant::now();

        let remaining = {
            let conn = kb.conn.lock().expect("Lock");
            conn.query_row(
                "SELECT COUNT(*) FROM crawl_queue WHERE status = 'pending'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0i64)
        };
        if remaining == 0 && cycle > 1 {
            println!("\n  队列已空！在循环 #{} 结束", cycle);
            break;
        }

            match kb.run_crawl_cycle_parallel(batch, workers, fetch_links) {
            Ok(report) => {
                let elapsed = c_start.elapsed();
                total_processed += report.completed as u64;
                total_absorbed += report.nodes_created as u64;
                total_edges += report.edges_created as u64;

                let throughput = if elapsed.as_secs_f64() > 0.0 {
                    report.attempted as f64 / elapsed.as_secs_f64()
                } else {
                    0.0
                };
                if throughput > peak_throughput {
                    peak_throughput = throughput;
                }

                let pending_now = {
                    let conn = kb.conn.lock().expect("Lock");
                    conn.query_row(
                        "SELECT COUNT(*) FROM crawl_queue WHERE status = 'pending'",
                        [],
                        |r| r.get(0),
                    )
                    .unwrap_or(0i64)
                };

                if cycle % 3 == 1 || cycle == cycles || report.completed < batch / 4 {
                    let _drain_rate = if cycle > 0 {
                        (before_pending - pending_now) as f64 / cycle as f64
                    } else {
                        0.0
                    };
                    println!("  [{:>3}/{}] 尝试={}, ✅{} ❌{}, 节点={}, 边={} | 待处理={} | {:.1}/s | [{:.1}s]",
                        cycle, cycles,
                        report.attempted, report.completed, report.failed,
                        report.nodes_created, report.edges_created,
                        pending_now, throughput, elapsed.as_secs_f64());

                    if !report.errors.is_empty() {
                        for item in report.errors.iter().take(3) {
                            println!(
                                "    ⚠ {} → {}",
                                item.0,
                                item.1.chars().take(120).collect::<String>()
                            );
                        }
                    }
                }
            }
            Err(e) => {
                if cycle % 5 == 1 {
                    println!("  [{:>3}/{}] ⚠ 循环错误: {}", cycle, cycles, e);
                }
            }
        }

        if cycle % 15 == 0 {
            let elapsed = start.elapsed();
            let pct = cycle as f64 / cycles as f64 * 100.0;
            let eta = if cycle > 0 {
                let per_cycle = elapsed / cycle as u32;
                Duration::from_secs(per_cycle.as_secs() * (cycles - cycle))
            } else {
                Duration::from_secs(0)
            };
            println!(
                "  --- [进度 {:.0}% | 已用 {}s | 预计剩余 {}s | 峰值 {:.1} URLs/s] ---",
                pct,
                elapsed.as_secs(),
                eta.as_secs(),
                peak_throughput
            );
        }

        if cycle < cycles {
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    // Phase 4: Final report
    let elapsed = start.elapsed();
    let after_pending = {
        let conn = kb.conn.lock().expect("Lock");
        conn.query_row(
            "SELECT COUNT(*) FROM crawl_queue WHERE status = 'pending'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0i64)
    };

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  爬取队列排空 — 完成报告                                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!(
        "  总耗时:   {:.0}s ({:.1}m)",
        elapsed.as_secs_f64(),
        elapsed.as_secs_f64() / 60.0
    );
    println!("  处理:     {} URLs", total_processed);
    println!("  吸收:     {} 节点, {} 边", total_absorbed, total_edges);
    println!("  峰值吞吐: {:.1} URLs/s", peak_throughput);
    println!("\n  队列状态:");
    println!("    排空前: {} pending", before_pending);
    println!("    排空后: {} pending", after_pending);
    let drained = before_pending.saturating_sub(after_pending);
    if drained > 0 {
        println!(
            "    净减少: {} URLs (排空率 {:.1}%)",
            drained,
            drained as f64 / before_pending.max(1) as f64 * 100.0
        );
    } else if after_pending > before_pending {
        println!(
            "    净增长: {} URLs (发现新链接 > 处理速度)",
            after_pending - before_pending
        );
    } else {
        println!("    持平: 0 URLs");
    }

    if let Ok(stats) = kb.stats() {
        println!(
            "\n  知识库: {} 节点, {} 边, {} 域",
            stats.total_nodes,
            stats.total_edges,
            stats.by_domain.len()
        );
    }
}
