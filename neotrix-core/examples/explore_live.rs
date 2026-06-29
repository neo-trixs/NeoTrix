use neotrix::neotrix::nt_memory_knowledge_populator::KnowledgePopulator;
use neotrix::neotrix::nt_mind::exploration_pipeline::{
    seed_urls_by_domain, ExplorationPipeline, ExploreDomain,
};
use neotrix::neotrix::nt_mind::memory::ReasoningBank;
use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
/// NeoTrix 探索引擎动态展示
/// cargo run --example explore_live
use std::path::PathBuf;

fn print_header(s: &str) {
    log::info!("\n╔══════════════════════════════════════════════════╗");
    log::info!("║  {}  ║", s);
    log::info!("╚══════════════════════════════════════════════════╝");
}

fn print_domain(label: &str, urls: &[String]) {
    log::info!("  {:20} {} entries", label, urls.len());
    for u in urls.iter().take(3) {
        log::info!("  {:22} {}", "", u);
    }
    if urls.len() > 3 {
        log::info!("  {:22} ... ({} more)", "", urls.len() - 3);
    }
}

fn main() {
    print_header("NeoTrix 探索引擎全景 — 知识域与种子库");

    let domains: Vec<(ExploreDomain, &str)> = vec![
        (ExploreDomain::Consciousness, "Consciousness"),
        (ExploreDomain::MathPhysics, "Math & Physics"),
        (ExploreDomain::Parapsychology, "Parapsychology"),
        (ExploreDomain::Theology, "Theology"),
        (ExploreDomain::EsotericStudies, "Esoteric Studies"),
        (ExploreDomain::Wiki, "Wiki / Philosophy"),
        (ExploreDomain::RustML, "Rust ML"),
        (ExploreDomain::Security, "Security"),
        (ExploreDomain::Papers, "Papers (arXiv)"),
        (ExploreDomain::GitHub, "GitHub Repos"),
        (ExploreDomain::General, "General"),
    ];

    let mut total_seeds = 0usize;
    for (domain, label) in &domains {
        let urls = seed_urls_by_domain(*domain);
        total_seeds += urls.len();
        print_domain(label, &urls);
    }
    log::info!("\n  Total seed URLs: {}", total_seeds);

    print_header("KnowledgeSource 播种 — 59 知识源 x 2 记忆 = 118 种子");
    let mut brain = SelfIteratingBrain::new();
    let report = KnowledgePopulator::populate_brain(&mut brain.brain.capability);
    log::info!("  Sources processed:    {}", report.sources_processed);
    log::info!("  Dimensions updated:   {}", report.dimensions_updated);
    log::info!(
        "  Extension keys added: {}",
        report.extension_keys_added.len()
    );
    log::info!(
        "  Non-zero dimensions:  {} -> {}",
        report.non_zero_before,
        report.non_zero_after
    );

    print_header("能力缺口低维维度 (< 0.3)");
    let arr = brain.brain.capability.arr();
    let weak: Vec<(usize, f64)> = arr
        .iter()
        .enumerate()
        .filter(|(_, &v)| v < 0.3)
        .map(|(i, &v)| (i, v))
        .collect();
    if weak.is_empty() {
        log::info!("  All 23 dimensions >= 0.3 after 59-KS initialization");
    } else {
        for (idx, val) in &weak {
            log::info!("  dim[{}] = {:.3}", idx, val);
        }
    }

    print_header("探索管道初始化 — 11 domains ready, 8 background tickers");
    let pipeline = ExplorationPipeline::new(PathBuf::from("."));
    let stats = pipeline.stats();
    log::info!(
        "  Seed queue ready:              {} to process",
        stats.queued
    );
    log::info!("  KnowledgeEngine entries:       {}", stats.ke_entries);
    log::info!("  Auto-discovery cache:          {} capacity", 10);

    let mut bank = ReasoningBank::new(200);
    KnowledgePopulator::populate_reasoning_bank(&mut bank, 2);
    let bank_stats = bank.stats();
    log::info!(
        "  ReasoningBank memories:        {}",
        bank_stats.total_memories
    );
    log::info!("  KnowledgeSource variants:      59");
    log::info!("  ExploreDomain domains:         11");
    log::info!(
        "  CapabilityVector dimensions:   23 core + {} extension",
        brain.brain.capability.extension.len()
    );

    print_header("System ready");
    log::info!("  cargo run  ->  background loop auto-starts with 8 tickers");
    log::info!("  cargo run --example explore_live  ->  this dashboard");
    log::info!("  11 domains | 135 seeds | 59 KS | 118 memories | 8 tickers");
}
