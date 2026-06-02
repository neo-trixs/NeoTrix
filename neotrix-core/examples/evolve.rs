//! NeoTrix 自主进化守护进程
//!
//! 持续循环: 爬取 → 分类 → 吸收(brain+hypercube) → gap分析 → 种子注入
//! 每50 cycle: 自愈检查 + 注意力路由稀疏域检测
//! Ctrl+C 停止, 打印终态
//!
//! 使用: cargo run --bin evolve --release
use std::time::Instant;

use neotrix::neotrix::crawler::{
    CrawlerConfig, CrawlStrategy, CrawlTopic, SeedEntry, UnifiedCrawler,
};
use neotrix::neotrix::reasoning_brain::attention_router::AttentionRouter;
use neotrix::{ReasoningBrain, ReasoningBank};

fn main() {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║     NeoTrix 自主进化守护进程 v0.1                         ║");
    println!("║     爬取 → 分类 → 吸收(brain+hypercube) → gap → 种子      ║");
    println!("║     Ctrl+C 停止                                          ║");
    println!("╚═══════════════════════════════════════════════════════════╝");

    // ── 初始化推理大脑 + 知识库 ──
    let mut brain = ReasoningBrain::default();
    let mut bank = ReasoningBank::new(10000);
    let mut router = AttentionRouter::new();
    router.seed_knowledge();

    // ── 初始化爬虫 ──
    let config = CrawlerConfig {
        strategy: CrawlStrategy::Polite,
        max_pages_per_domain: 50,
        max_depth: 3,
        self_heal_interval: 50,
        fetch_timeout_secs: 15,
        max_retries: 2,
        ..Default::default()
    };

    let mut crawler = UnifiedCrawler::new(config);
    crawler.add_seeds(civilization_seeds());

    let start = Instant::now();
    let mut total_absorbed: u64 = 0;
    let mut last_status_cycle: u64 = 0;
    let mut empty_count: u32 = 0;
    let mut cycle_count: u64 = 0;

    // ── 主循环 ──
    // 使用 loop + 终态条件; Ctrl+C 由 OS 处理
    loop {
        let result = crawler.run_cycle(&mut brain, &mut bank);
        cycle_count = result.cycle;

        if result.absorbed {
            total_absorbed += 1;

            // gap分析驱动的种子注入: 每20个成功吸收后检测稀疏域
            if total_absorbed > 0 && total_absorbed % 20 == 0 {
                let sparse = router.sparse_topics();
                if !sparse.is_empty() {
                    let seeds = seeds_for_sparse_topics(&sparse);
                    crawler.add_seeds(seeds);
                    println!(
                        "  ↪ 自驱种子注入: {} 个稀疏域 → Wikipedia",
                        sparse.len()
                    );
                    for t in &sparse {
                        println!("    - {}", t.name());
                    }
                }
            }
        } else if !result.fetched && result.message.contains("empty") {
            empty_count += 1;
            // frontier 空 → 用 gap 驱动注入新种子
            if empty_count >= 5 {
                let sparse = router.sparse_topics();
                if !sparse.is_empty() {
                    let seeds = seeds_for_sparse_topics(&sparse);
                    crawler.add_seeds(seeds);
                    println!(
                        "  ↪ frontier 空 → gap 驱动注入 {} 个种子",
                        sparse.len()
                    );
                    empty_count = 0;
                } else {
                    println!("  ⏹ frontier 空, 无稀疏域可探索 → 停止");
                    break;
                }
            }
        }

        // 自愈检查点 (每50 cycle)
        if result.is_heal_check {
            println!("\n━━━ 自愈检查 @ cycle {} ━━━", cycle_count);
            print_current_state(&brain, &crawler, &bank, &router, start);

            if !crawler.heal_history().is_empty() {
                let last = crawler.heal_history().last().expect("heal history is empty");
                if last.applied {
                    println!("  动作: {}", last.action);
                }
            }
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        }

        // 每10 cycle 轻量状态
        if cycle_count > 0
            && cycle_count % 10 == 0
            && cycle_count > last_status_cycle
        {
            last_status_cycle = cycle_count;
            print_compact_status(&brain, &crawler, cycle_count, total_absorbed, start);
        }
    }

    // ── 终态 ──
    let elapsed = start.elapsed().as_secs_f64();
    println!("\n╔═══════════════════════════════════════════════╗");
    println!("║  自主进化结束                                 ║");
    println!("╚═══════════════════════════════════════════════╝");
    println!("  运行时间:     {:.0}s ({:.1}min)", elapsed, elapsed / 60.0);
    println!("  总 cycle:     {}", cycle_count);
    println!("  总吸收:       {} 页", total_absorbed);
    println!("  自愈次数:     {}", crawler.heal_history().len());
    println!("  能力向量和:   {:.4}", brain.capability.arr().iter().sum::<f64>());
    println!("  向量维度激活: {}/{}",
        brain.capability.arr().iter().filter(|&&v| v > 0.01).count(),
        brain.capability.arr().len()
    );
    println!("  超立方体条目: {}", router.bridge.hypercube.cell_count());
    println!("  Brain知识源:  {}", brain.custom_sources.len());
    println!("  Frontier:     {} 剩余", crawler.frontier_stats());

    // 自愈历史
    for (i, heal) in crawler.heal_history().iter().enumerate() {
        println!("  自愈 #{}: {}", i + 1, heal.action);
    }

    // 能力向量明细
    println!("\n能力向量明细 (top 10):");
    let mut dims: Vec<(String, f64)> = dim_names()
        .iter()
        .zip(brain.capability.arr().iter())
        .map(|(n, v)| (n.clone(), *v))
        .filter(|(_, v)| *v > 0.01)
        .collect();
    dims.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    for (name, val) in dims.iter().take(10) {
        println!("  {:<25} {:.4}", name, val);
    }
}

// ==================== 文明种子 ====================

fn civilization_seeds() -> Vec<SeedEntry> {
    vec![
        SeedEntry { url: "https://www.un.org/en/about-us/universal-declaration-of-human-rights".into(), topic: CrawlTopic::LawAndGovernance, depth: 1, enabled: true },
        SeedEntry { url: "https://www.constituteproject.org".into(), topic: CrawlTopic::LawAndGovernance, depth: 1, enabled: true },
        SeedEntry { url: "https://plato.stanford.edu/entries/philosophy-law".into(), topic: CrawlTopic::PhilosophyAndEthics, depth: 1, enabled: true },
        SeedEntry { url: "https://plato.stanford.edu/entries/ethics".into(), topic: CrawlTopic::PhilosophyAndEthics, depth: 1, enabled: true },
        SeedEntry { url: "https://en.wikipedia.org/wiki/Rule_of_law".into(), topic: CrawlTopic::LawAndGovernance, depth: 1, enabled: true },
        SeedEntry { url: "https://en.wikipedia.org/wiki/Science".into(), topic: CrawlTopic::ScienceAndTechnology, depth: 1, enabled: true },
        SeedEntry { url: "https://en.wikipedia.org/wiki/History".into(), topic: CrawlTopic::HistoryAndArcheology, depth: 1, enabled: true },
        SeedEntry { url: "https://en.wikipedia.org/wiki/Economics".into(), topic: CrawlTopic::SocietyAndEconomics, depth: 1, enabled: true },
        SeedEntry { url: "https://en.wikipedia.org/wiki/Art".into(), topic: CrawlTopic::ArtsAndLiterature, depth: 1, enabled: true },
        SeedEntry { url: "https://www.who.int/health-topics".into(), topic: CrawlTopic::HealthAndMedicine, depth: 1, enabled: true },
    ]
}

fn seeds_for_sparse_topics(topics: &[CrawlTopic]) -> Vec<SeedEntry> {
    topics.iter().map(|t| {
        let url = match t {
            CrawlTopic::LawAndGovernance => "https://en.wikipedia.org/wiki/Law",
            CrawlTopic::PolicyAndRegulation => "https://en.wikipedia.org/wiki/Public_policy",
            CrawlTopic::PhilosophyAndEthics => "https://plato.stanford.edu/entries/moral-philosophy",
            CrawlTopic::ScienceAndTechnology => "https://en.wikipedia.org/wiki/Technology",
            CrawlTopic::HistoryAndArcheology => "https://en.wikipedia.org/wiki/World_history",
            CrawlTopic::SocietyAndEconomics => "https://en.wikipedia.org/wiki/Society",
            CrawlTopic::HealthAndMedicine => "https://en.wikipedia.org/wiki/Medicine",
            CrawlTopic::EducationAndAcademia => "https://en.wikipedia.org/wiki/Education",
            CrawlTopic::HumanitiesAndCulture => "https://en.wikipedia.org/wiki/Culture",
            CrawlTopic::ArtsAndLiterature => "https://en.wikipedia.org/wiki/The_arts",
            CrawlTopic::NewsAndMedia => "https://en.wikipedia.org/wiki/News",
            CrawlTopic::General => "https://en.wikipedia.org/wiki/Knowledge",
        };
        SeedEntry {
            url: url.into(),
            topic: *t,
            depth: 1,
            enabled: true,
        }
    }).collect()
}

// ==================== 状态输出 ====================

fn print_current_state(
    brain: &ReasoningBrain,
    crawler: &UnifiedCrawler,
    _bank: &ReasoningBank,
    router: &AttentionRouter,
    start: Instant,
) {
    let s = crawler.summary();
    let elapsed = start.elapsed().as_secs();
    let cap_sum: f64 = brain.capability.arr().iter().sum();

    println!("  时间:        {}s", elapsed);
    println!("  吸收:        {} 页", s.absorbed);
    println!("  Frontier:    {} 未处理", s.frontier_size);
    println!("  错误率:      {:.1}%", s.error_rate * 100.0);
    println!("  能力向量和:  {:.4}", cap_sum);
    println!("  超立方体:    {} 条目", router.bridge.hypercube.cell_count());
    println!("  Brain知识源: {}", brain.custom_sources.len());

    // Gap 报告
    let gaps = router.bridge.analyze_gaps();
    let sparse = gaps.iter().filter(|g| g.sparsity_score > 0.7).count();
    println!("  稀疏维度:    {}/8 (threshold >0.7)", sparse);
    for g in gaps.iter().filter(|g| g.sparsity_score > 0.7) {
        println!("    dim{}: sparsity={:.2}", g.dim_index, g.sparsity_score);
    }
}

fn print_compact_status(
    brain: &ReasoningBrain,
    crawler: &UnifiedCrawler,
    cycle: u64,
    absorbed: u64,
    start: Instant,
) {
    let s = crawler.summary();
    let cap_sum: f64 = brain.capability.arr().iter().sum();
    let elapsed = start.elapsed().as_secs();
    println!(
        "[{:>3}s] cycle={:<4} absorbed={:<4} frontier={:<3} cap={:.4} err={:.1}%",
        elapsed, cycle, absorbed, s.frontier_size, cap_sum, s.error_rate * 100.0,
    );
}

// ==================== 工具函数 ====================

fn dim_names() -> Vec<String> {
    vec![
        "typography", "grid", "color", "whitespace",
        "data_viz", "emotion", "minimalism", "experimental",
        "inference_depth", "creativity", "analysis", "synthesis",
        "domain_specificity", "accessibility", "compound_composition",
        "tailwind_proficiency", "react_aria_usage", "bem_naming",
        "figma_integration", "ai_native_states", "semantic_layer",
        "quality_gates", "verification",
    ].iter().map(|s| s.to_string()).collect()
}
