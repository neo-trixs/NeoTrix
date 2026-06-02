use std::time::Instant;

use neotrix::neotrix::nt_memory_kb::KnowledgeBase;

/// 多领域探索种子: 按领域分组, 每组高优先级
const EXPLORE_TOPICS: &[&str] = &[
    // 哲学缺口
    "Karma", "Buddhist_philosophy", "Taoism", "Confucianism", "Stoicism", "Existentialism",
    "Phenomenology", "Epistemology", "Metaphysics", "Ethics", "Moral_philosophy",
    "Philosophy_of_mind", "Free_will", "Determinism", "Consciousness",
    // 认知科学
    "Cognitive_science", "Cognitive_psychology", "Neuroscience", "Cognitive_neuroscience",
    "Neuroplasticity", "Brain", "Neural_correlates_of_consciousness",
    // AI 和 ML
    "Artificial_intelligence", "Machine_learning", "Deep_learning", "Neural_network",
    "Reinforcement_learning", "Large_language_model", "Transformer_(machine_learning_model)",
    "Attention_(machine_learning)", "Generative_AI", "Computer_vision",
    // 数学补充
    "Linear_algebra", "Probability_theory", "Statistics", "Information_theory",
    "Complexity_theory", "Graph_theory",
    // 物理
    "Physics", "Quantum_mechanics", "Relativity", "Thermodynamics",
    "Statistical_mechanics", "Particle_physics",
    // 生物
    "Biology", "Evolution", "Genetics", "Cell_biology", "Molecular_biology",
    "Ecology",
    // 系统与工程
    "Systems_theory", "Cybernetics", "Control_theory", "Information_technology",
    "Software_engineering", "Computer_science",
    // 社会与文明
    "Civilization", "History_of_science", "Anthropology", "Sociology",
    "Economics", "Political_science", "Linguistics",
    // VSA / HD Computing
    "Vector_Symbolic_Architecture", "Hyperdimensional_computing",
    "Holographic_Reduced_Representation",
    // 意识理论
    "Integrated_information_theory", "Global_workspace_theory",
    "Predictive_coding", "Free_energy_principle",
];

fn main() {
    println!("╔═══════════════════════════════════════════════════╗");
    println!("║  NeoTrix 自主知识探索 — 多领域 Wikipedia 摄取     ║");
    println!("╚═══════════════════════════════════════════════════╝");

    let kb = KnowledgeBase::open(None).expect("KB open failed");
    let start = Instant::now();

    let before = kb.stats().expect("Stats");
    println!("\n  开始: {} 节点, {} 边\n", before.total_nodes, before.total_edges);

    let mut total_new = 0usize;
    let mut success = 0usize;
    let mut fail = 0usize;

    for (i, topic) in EXPLORE_TOPICS.iter().enumerate() {
        let t_start = Instant::now();
        match kb.ingest_wikipedia(topic) {
            Ok(n) => {
                total_new += n;
                success += 1;
                let domain = topic.split('_').next().unwrap_or(topic);
                println!("  [{:>2}/{}] {:>50} | +{:<3} nodes | {:>6.2}s",
                    i+1, EXPLORE_TOPICS.len(),
                    topic.replace('_', " "),
                    n, t_start.elapsed().as_secs_f64());
            }
            Err(e) => {
                fail += 1;
                println!("  [{:>2}/{}] {:>50} | ⚠ {}",
                    i+1, EXPLORE_TOPICS.len(),
                    topic.replace('_', " "),
                    e);
            }
        }
    }

    let after = kb.stats().expect("Stats final");
    let elapsed = start.elapsed();

    println!("\n━━━ 探索完成 ━━━");
    println!("  耗时:          {:.1}s", elapsed.as_secs_f64());
    println!("  成功/总:       {}/{}", success, EXPLORE_TOPICS.len());
    println!("  新节点:        {} ({} → {})",
        after.total_nodes - before.total_nodes, before.total_nodes, after.total_nodes);
    println!("  新边:          {} ({} → {})",
        after.total_edges - before.total_edges, before.total_edges, after.total_edges);

    // Run crawl cycle to process discovered links
    println!("\n━━━ 爬取队列处理 ━━━");
    let c_start = Instant::now();
    match kb.run_crawl_cycle(20) {
        Ok(report) => {
            println!("  尝试: {} | 完成: {} | 失败: {}",
                report.attempted, report.completed, report.failed);
            println!("  新建节点: {} | 新建边: {}", report.nodes_created, report.edges_created);
            println!("  耗时: {:.2}s", c_start.elapsed().as_secs_f64());
        }
        Err(e) => println!("  ⚠ 爬取循环: {}", e),
    }

    let final_stats = kb.stats().expect("Stats final2");
    println!("\n╔═══════════════════════════════════════════════╗");
    println!("║  探索终态                                     ║");
    println!("╚═══════════════════════════════════════════════╝");
    println!("  总节点:       {}", final_stats.total_nodes);
    println!("  总边:         {}", final_stats.total_edges);
    println!("  域数:         {}", final_stats.by_domain.len());
    println!("  爬取待处理:   {}", final_stats.crawl_pending);

    // Final type distribution
    println!("\n  类型分布:");
    for (t, c) in &final_stats.by_type {
        println!("    {}: {}", t, c);
    }
    println!("\n  域分布 (top 10):");
    for (d, c) in final_stats.by_domain.iter().take(10) {
        println!("    {}: {}", d, c);
    }
}
