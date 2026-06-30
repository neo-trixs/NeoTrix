//! NeoTrix 自主进化守护进程
//!
//! 持续循环: 爬取 → 分类 → 吸收(brain+hypercube) → gap分析 → 种子注入
//! 每50 cycle: 自愈检查 + 注意力路由稀疏域检测 + hypercube同步
//! Ctrl+C 停止, 打印终态
use std::time::Instant;

use neotrix::neotrix::nt_world_crawl::{
    CrawlerConfig, CrawlStrategy, CrawlTopic, SeedEntry, UnifiedCrawler,
};
use neotrix::neotrix::nt_mind::attention_router::AttentionRouter;
use neotrix::neotrix::nt_mind::stagnation::{StagnationDetector, StagnationSignal};
use neotrix::{ReasoningBrain, ReasoningBank};

fn main() {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║     NeoTrix 自主进化守护进程 v0.1                         ║");
    println!("║     爬取 → 分类 → 吸收(brain+hypercube) → gap → 种子      ║");
    println!("║     Ctrl+C 停止                                          ║");
    println!("╚═══════════════════════════════════════════════════════════╝");

    let mut brain = ReasoningBrain::default();
    let mut bank = ReasoningBank::new(10000);
    let mut router = AttentionRouter::new();
    router.seed_knowledge();

    let config = CrawlerConfig {
        strategy: CrawlStrategy::Polite,
        max_pages_per_domain: 50,
        max_depth: 3,
        self_heal_interval: 50,
        fetch_timeout_secs: 15,
        max_retries: 2,
        ..Default::default()
    };

    let mut nt_world_crawl = UnifiedCrawler::new(config);
    nt_world_crawl.add_seeds(civilization_seeds());

    let start = Instant::now();
    let mut total_absorbed: u64 = 0;
    let mut last_status_cycle: u64 = 0;
    let mut empty_count: u32 = 0;
    let mut stagnation = StagnationDetector::new();

    loop {
        let result = nt_world_crawl.run_cycle(&mut brain, &mut bank);
        let cycle_count = result.cycle;

        let have_minor_errors = if let Some(history) = nt_world_crawl.heal_history().last() {
            history.action.contains("minor errors")
        } else {
            false
        };
        let sig = stagnation.observe(
            result.absorbed,
            result.fetched,
            0,
            0.0,
            !result.message.contains("empty"),
            have_minor_errors,
        );
        match sig {
            StagnationSignal::Stop(reason) => {
                println!("  ⏹ 停滞检测触发: {}", reason);
                break;
            }
            StagnationSignal::Pause(secs, reason) => {
                println!("  ⏸ 停滞暂停 {}s: {}", secs, reason);
                std::thread::sleep(std::time::Duration::from_secs(secs));
                stagnation.reset();
            }
            StagnationSignal::Continue => {}
        }

        if result.absorbed {
            total_absorbed += 1;
            if total_absorbed > 0 && total_absorbed % 20 == 0 {
                // sync nt_world_crawl → router bridge so gap analysis has data
                for entry in nt_world_crawl.bridge.hypercube.entries() {
                    router.bridge.hypercube.insert(&entry.coord, &entry.source, &entry.label);
                }
                let sparse = router.sparse_topics();
                if !sparse.is_empty() {
                    let seeds = seeds_for_sparse_topics(&sparse);
                    nt_world_crawl.add_seeds(seeds);
                    println!("  ↪ 自驱种子注入: {} 个稀疏域 → Wikipedia", sparse.len());
                }
            }
        } else if !result.fetched && result.message.contains("empty") {
            empty_count += 1;
            if empty_count >= 5 {
                // sync then check gaps
                for entry in nt_world_crawl.bridge.hypercube.entries() {
                    router.bridge.hypercube.insert(&entry.coord, &entry.source, &entry.label);
                }
                let sparse = router.sparse_topics();
                if !sparse.is_empty() {
                    nt_world_crawl.add_seeds(seeds_for_sparse_topics(&sparse));
                    println!("  ↪ frontier 空 → gap 驱动注入 {} 个种子", sparse.len());
                    empty_count = 0;
                } else {
                    println!("  ⏹ frontier 空, 无稀疏域可探索 → 停止");
                    break;
                }
            }
        }

        if result.is_heal_check {
            // sync nt_world_crawl → router bridge
            for entry in nt_world_crawl.bridge.hypercube.entries() {
                router.bridge.hypercube.insert(&entry.coord, &entry.source, &entry.label);
            }

            println!("\n━━━ 自愈检查 @ cycle {} ━━━", cycle_count);
            let s = nt_world_crawl.summary();
            let gaps = router.bridge.analyze_gaps();
            let sparse_count = gaps.iter().filter(|g| g.sparsity_score > 0.7).count();
            println!(
                "  时间: {}s  吸收: {}  错误率: {:.1}%  frontier: {}",
                start.elapsed().as_secs(), s.absorbed, s.error_rate * 100.0, s.frontier_size
            );
            println!("  能力向量和: {:.4}", brain.capability.arr().iter().sum::<f64>());
            println!("  超立方体:   {} (nt_world_crawl: {})",
                router.bridge.hypercube.cell_count(),
                nt_world_crawl.bridge.hypercube.cell_count()
            );
            println!("  Brain知识源: {}  稀疏维度: {}/8", brain.custom_sources.len(), sparse_count);

            if !nt_world_crawl.heal_history().is_empty() {
                let last = nt_world_crawl.heal_history().last().expect("result");
                if last.applied {
                    println!("  动作: {}", last.action);
                }
            }
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        }

        if cycle_count > 0 && cycle_count % 10 == 0 && cycle_count > last_status_cycle {
            last_status_cycle = cycle_count;
            let s = nt_world_crawl.summary();
            let cap_sum: f64 = brain.capability.arr().iter().sum();
            println!(
                "[{:>3}s] cycle={:<4} absorbed={:<4} frontier={:<3} cap={:.4} err={:.1}%",
                start.elapsed().as_secs(), cycle_count, total_absorbed, s.frontier_size,
                cap_sum, s.error_rate * 100.0,
            );
        }
    }

    let elapsed = start.elapsed().as_secs_f64();
    println!("\n╔═══════════════════════════════════════════════╗");
    println!("║  自主进化结束                                 ║");
    println!("╚═══════════════════════════════════════════════╝");
    println!("  时间:     {:.0}s ({:.1}min)", elapsed, elapsed / 60.0);
    println!("  总吸收:   {} 页", total_absorbed);
    println!("  自愈:     {}", nt_world_crawl.heal_history().len());
    println!("  能力:     {}", brain.capability.arr().iter().sum::<f64>());
    println!("  HC:       {} nt_world_crawl / {} router",
        nt_world_crawl.bridge.hypercube.cell_count(),
        router.bridge.hypercube.cell_count()
    );
    println!("  知识源:   {}", brain.custom_sources.len());

    let mut dims: Vec<(String, f64)> = [
        "inference_depth","creativity","analysis","synthesis",
        "domain_specificity","accessibility","quality_gates","verification",
    ].iter().map(|n| (n.to_string(), 0.0)).collect();
    for (name, val) in dims.iter_mut() {
        if let Some(idx) = dim_name_to_index(name) {
            *val = brain.capability.arr()[idx];
        }
    }
    dims.sort_by(|a, b| b.1.partial_cmp(&a.1).expect("result"));
    println!("\n能力向量 (top):");
    for (name, val) in dims.iter().filter(|(_, v)| *v > 0.01) {
        println!("  {:<25} {:.4}", name, val);
    }
}

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
        SeedEntry { url: url.into(), topic: *t, depth: 1, enabled: true }
    }).collect()
}

fn dim_name_to_index(name: &str) -> Option<usize> {
    let dims = [
        "typography","grid","color","whitespace","data_viz","emotion",
        "minimalism","experimental","inference_depth","creativity",
        "analysis","synthesis","domain_specificity","accessibility",
        "compound_composition","tailwind_proficiency","react_aria_usage",
        "bem_naming","figma_integration","ai_native_states",
        "semantic_layer","quality_gates","verification",
    ];
    dims.iter().position(|d| *d == name)
}
