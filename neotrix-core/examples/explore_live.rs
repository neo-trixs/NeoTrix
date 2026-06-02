/// NeoTrix 探索引擎动态展示
/// cargo run --example explore_live

use std::path::PathBuf;
use neotrix::neotrix::reasoning_brain::exploration_pipeline::{
    ExplorationPipeline, ExploreDomain, seed_urls_by_domain,
};
use neotrix::neotrix::reasoning_brain::self_iterating::SelfIteratingBrain;
use neotrix::neotrix::reasoning_brain::memory::ReasoningBank;
use neotrix::neotrix::knowledge_populator::KnowledgePopulator;

fn print_header(s: &str) {
    println!("\n╔══════════════════════════════════════════════════╗");
    println!("║  {}  ║", s);
    println!("╚══════════════════════════════════════════════════╝");
}

fn print_domain(label: &str, urls: &[String]) {
    println!("  {:20} {} entries", label, urls.len());
    for u in urls.iter().take(3) {
        println!("  {:22} {}", "", u);
    }
    if urls.len() > 3 {
        println!("  {:22} ... ({} more)", "", urls.len() - 3);
    }
}

fn main() {
    print_header("NeoTrix 探索引擎全景 — 知识域与种子库");

    let domains: Vec<(ExploreDomain, &str)> = vec![
        (ExploreDomain::Consciousness,    "Consciousness"),
        (ExploreDomain::MathPhysics,      "Math & Physics"),
        (ExploreDomain::Parapsychology,   "Parapsychology"),
        (ExploreDomain::Theology,         "Theology"),
        (ExploreDomain::EsotericStudies,  "Esoteric Studies"),
        (ExploreDomain::Wiki,             "Wiki / Philosophy"),
        (ExploreDomain::RustML,           "Rust ML"),
        (ExploreDomain::Security,         "Security"),
        (ExploreDomain::Papers,           "Papers (arXiv)"),
        (ExploreDomain::GitHub,           "GitHub Repos"),
        (ExploreDomain::General,          "General"),
    ];

    let mut total_seeds = 0usize;
    for (domain, label) in &domains {
        let urls = seed_urls_by_domain(*domain);
        total_seeds += urls.len();
        print_domain(label, &urls);
    }
    println!("\n  Total seed URLs: {}", total_seeds);

    print_header("KnowledgeSource 播种 — 59 知识源 x 2 记忆 = 118 种子");
    let mut brain = SelfIteratingBrain::new();
    let report = KnowledgePopulator::populate_brain(&mut brain.brain.capability);
    println!("  Sources processed:    {}", report.sources_processed);
    println!("  Dimensions updated:   {}", report.dimensions_updated);
    println!("  Extension keys added: {}", report.extension_keys_added.len());
    println!("  Non-zero dimensions:  {} -> {}", report.non_zero_before, report.non_zero_after);

    print_header("能力缺口低维维度 (< 0.3)");
    let arr = brain.brain.capability.arr();
    let weak: Vec<(usize, f64)> = arr.iter().enumerate()
        .filter(|(_, &v)| v < 0.3).map(|(i, &v)| (i, v)).collect();
    if weak.is_empty() {
        println!("  All 23 dimensions >= 0.3 after 59-KS initialization");
    } else {
        for (idx, val) in &weak {
            println!("  dim[{}] = {:.3}", idx, val);
        }
    }

    print_header("探索管道初始化 — 11 domains ready, 8 background tickers");
    let pipeline = ExplorationPipeline::new(PathBuf::from("."));
    let stats = pipeline.stats();
    println!("  Seed queue ready:              {} to process", stats.queued);
    println!("  KnowledgeEngine entries:       {}", stats.ke_entries);
    println!("  Auto-discovery cache:          {} capacity", 10);

    let mut bank = ReasoningBank::new(200);
    KnowledgePopulator::populate_reasoning_bank(&mut bank, 2);
    let bank_stats = bank.stats();
    println!("  ReasoningBank memories:        {}", bank_stats.total_memories);
    println!("  KnowledgeSource variants:      59");
    println!("  ExploreDomain domains:         11");
    println!("  CapabilityVector dimensions:   23 core + {} extension",
        brain.brain.capability.extension.len());

    print_header("System ready");
    println!("  cargo run  ->  background loop auto-starts with 8 tickers");
    println!("  cargo run --example explore_live  ->  this dashboard");
    println!("  11 domains | 135 seeds | 59 KS | 118 memories | 8 tickers");
}
