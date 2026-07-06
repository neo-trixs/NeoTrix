use std::io::{self, Write};
use std::time::{Duration, Instant};

use neotrix::neotrix::l3_memory_impl::nt_memory_kb::{
    DiscoveryCycleConfig, DiscoveryPipelineConfig, KnowledgeBase,
};

const CYCLE_INTERVAL: Duration = Duration::from_secs(5);

fn main() {
    p("╔══════════════════════════════════════════════════════════╗");
    p("║  NeoTrix 多源数据发现 — 持续运行                        ║");
    p("║  GitHub/ArXiv 每 5 cycle; OL/Wiki 每 cycle,              ║");
    p("║  Anna's Archive 待 headless browser 集成                ║");
    p("╚══════════════════════════════════════════════════════════╝");

    let kb = KnowledgeBase::open(None).expect("Failed to open KnowledgeBase");
    p(&format!("  DB: {}", kb.db_path.display()));

    let mut cycle = 0u64;
    let start = Instant::now();

    let ol_sets: [[&str; 2]; 4] = [
        ["machine learning", "quantum computing"],
        ["deep learning", "cryptography"],
        ["neural networks", "blockchain"],
        ["computer vision", "natural language"],
    ];
    let wiki_sets: [[&str; 2]; 4] = [
        ["Machine learning", "Ancient history"],
        ["Quantum computing", "Philosophy"],
        ["Artificial intelligence", "Renaissance"],
        ["Computer science", "Cognitive science"],
    ];
    loop {
        cycle += 1;
        p(&format!(
            "\n━ Cycle #{} @ {} (运行 {:.1}m) ━",
            cycle,
            now_str(),
            start.elapsed().as_secs_f64() / 60.0
        ));

        let cyc_start = Instant::now();
        let is_heavy = cycle % 5 == 0;
        let set_idx = ((cycle - 1) / 5 % 4) as usize;

        if is_heavy {
            let gh_cfg = DiscoveryPipelineConfig::default();
            match kb.run_github_topics_discovery(&gh_cfg) {
                Ok(s) => p(&format!(
                    "  GitHub: {} topics, {} repos ({} skipped, {} err)",
                    s.topics_found, s.repos_ingested, s.repos_skipped_existing, s.errors.len()
                )),
                Err(e) => p(&format!("  GitHub: ⚠ {}", e)),
            }
        }

        let mut cfg = DiscoveryCycleConfig::default();
        cfg.run_github_topics = false;
        cfg.run_openlibrary = true;
        cfg.openlibrary_queries = ol_sets[set_idx].iter().map(|s| s.to_string()).collect();
        cfg.run_arxiv = is_heavy;
        cfg.arxiv_queries = vec!["attention mechanism".into()];
        cfg.run_wikipedia = true;
        cfg.wikipedia_topics = wiki_sets[set_idx].iter().map(|s| s.to_string()).collect();
        // Anna's Archive: .org domain blocked, .gl uses JS rendering — needs headless browser
        cfg.run_annas_archive = false;
        cfg.annas_archive_queries = vec![];
        cfg.max_resources_per_source = 5;

        let report = kb.run_discovery_cycle(&cfg);
        let ol = report.openlibrary.as_ref().map(|s| s.resources_ingested).unwrap_or(0);
        let arx = report.arxiv.as_ref().map(|s| s.resources_ingested).unwrap_or(0);
        let wiki = report.wikipedia.as_ref().map(|s| s.resources_ingested).unwrap_or(0);
        let aa = report.annas_archive.as_ref().map(|s| s.resources_ingested).unwrap_or(0);
        p(&format!("  OL={} ArXiv={} Wiki={} AA={}", ol, arx, wiki, aa));
        for (src, err) in &report.errors[..report.errors.len().min(2)] {
            p(&format!("    ⚠ {}: {}", src, err));
        }
        if let Ok(stats) = kb.stats() {
            p(&format!("  KB: {} nodes, {} edges", stats.total_nodes, stats.total_edges));
        }

        let dur = cyc_start.elapsed();
        p(&format!("  Took {:.1}s", dur.as_secs_f64()));

        std::thread::sleep(CYCLE_INTERVAL);
    }
}

fn p(msg: &str) {
    println!("{}", msg);
    let _ = io::stdout().flush();
}

fn now_str() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
