use neotrix::neotrix::nt_memory_kb::KnowledgeBase;

/// 深度探索: 将 Wikipedia URL 注入爬取队列并运行多轮爬取
const SEED_URLS: &[(&str, i64, &str)] = &[
    // 哲学 (优先级最高)
    ("https://en.wikipedia.org/wiki/Karma", 3, "philosophy"),
    ("https://en.wikipedia.org/wiki/Philosophy_of_mind", 3, "philosophy"),
    ("https://en.wikipedia.org/wiki/Free_will", 3, "philosophy"),
    ("https://en.wikipedia.org/wiki/Consciousness", 3, "philosophy"),
    ("https://en.wikipedia.org/wiki/Epistemology", 2, "philosophy"),
    ("https://en.wikipedia.org/wiki/Metaphysics", 2, "philosophy"),
    ("https://en.wikipedia.org/wiki/Ethics", 2, "philosophy"),
    // 认知科学
    ("https://en.wikipedia.org/wiki/Cognitive_science", 3, "neuroscience"),
    ("https://en.wikipedia.org/wiki/Neuroscience", 3, "neuroscience"),
    // AI
    ("https://en.wikipedia.org/wiki/Artificial_intelligence", 2, "ai"),
    ("https://en.wikipedia.org/wiki/Machine_learning", 2, "ai"),
    ("https://en.wikipedia.org/wiki/Large_language_model", 2, "ai"),
    ("https://en.wikipedia.org/wiki/Reinforcement_learning", 2, "ai"),
    // 物理
    ("https://en.wikipedia.org/wiki/Quantum_mechanics", 2, "physics"),
    ("https://en.wikipedia.org/wiki/Information_theory", 2, "physics"),
    // 其他科学
    ("https://en.wikipedia.org/wiki/Evolution", 2, "biology"),
    ("https://en.wikipedia.org/wiki/Systems_theory", 2, "engineering"),
    // VSA
    ("https://en.wikipedia.org/wiki/Hyperdimensional_computing", 3, "vsa"),
];

fn main() {
    println!("╔═══════════════════════════════════════════════════╗");
    println!("║  NeoTrix 深度探索 — HTML 爬取循环                ║");
    println!("╚═══════════════════════════════════════════════════╝");

    let kb = KnowledgeBase::open(None).expect("KB open failed");

    let before = kb.stats().expect("Stats");
    println!("\n  起始: {} 节点, {} 边, {} 爬取待处理\n",
        before.total_nodes, before.total_edges, before.crawl_pending);

    // 注入种子 URL
    println!("━━━ 注入 {} 个种子 ━━━", SEED_URLS.len());
    let enqueued = kb.enqueue_seed_urls(SEED_URLS).expect("Enqueue failed");
    println!("  入队: {} 个 URL\n", enqueued);

    // 运行多轮爬取
    for cycle in 1..=5 {
        println!("━━━ 爬取循环 #{} ━━━", cycle);
        match kb.run_crawl_cycle(15) {
            Ok(report) => {
                println!("  尝试: {} | 完成: {} | 失败: {} | 节点: {} | 边: {}",
                    report.attempted, report.completed, report.failed,
                    report.nodes_created, report.edges_created);
                for (url, err) in &report.errors {
                    println!("    ⚠ {}: {}", url, err);
                }
                if report.attempted == 0 {
                    println!("  队列空 → 停止");
                    break;
                }
            }
            Err(e) => println!("  ⚠ 循环错误: {}", e),
        }
    }

    let after = kb.stats().expect("Stats final");
    println!("\n━━━ 深度探索完成 ━━━");
    println!("  节点: {} (+{})", after.total_nodes, after.total_nodes - before.total_nodes);
    println!("  边:   {} (+{})", after.total_edges, after.total_edges - before.total_edges);
    println!("  域数: {}", after.by_domain.len());

    println!("\n  类型分布:");
    for (t, c) in &after.by_type {
        println!("    {}: {}", t, c);
    }
    println!("\n  域分布 (top 10):");
    for (d, c) in after.by_domain.iter().take(10) {
        println!("    {}: {}", d, c);
    }
}
