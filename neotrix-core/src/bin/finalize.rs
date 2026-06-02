use std::time::Instant;

use neotrix::neotrix::nt_memory_kb::nt_memory_ingest::KBIngester;

const FINAL_TOPICS: &[&str] = &[
    "Attention_(machine_learning)", "Transformer_(deep_learning_architecture)",
    "Generative_adversarial_network", "Diffusion_model", "Variational_autoencoder",
    "Recurrent_neural_network", "Long_short-term_memory",
    "Graph_neural_network", "Convolutional_neural_network",
    "Meta_learning_(computer_science)", "Transfer_learning",
    "Few-shot_learning_(natural_language_processing)",
    "Mixture_of_experts", "Model_parallelism", "Distributed_computing",
    "Backpropagation", "Stochastic_gradient_descent",
    "Random_forest", "Decision_tree_learning", "Support-vector_machine",
    "Principal_component_analysis", "k-means_clustering",
    "Dimensionality_reduction", "Cross-validation_(statistics)",
    "Reasoning_system", "Expert_system", "Automated_reasoning",
    "Symbolic_artificial_intelligence", "Neuro-symbolic_computing",
    "Probabilistic_programming", "Graphical_model",
    "Inductive_logic_programming", "Case-based_reasoning",
    "Emergent_algorithm",
];

fn main() {
    println!("╔═══════════════════════════════════════════════════╗");
    println!("║  NeoTrix 最终填充 + 状态报告                      ║");
    println!("╚═══════════════════════════════════════════════════╝");

    let ing = KBIngester::open(None).expect("KBIngester open");
    let overall = Instant::now();
    let before = ing.snapshot();

    println!("\n  填充前: {} 节点, {} 边, {} 待处理",
        before.total_nodes, before.total_edges, before.crawl_pending);

    println!("\n━━━ Phase 1: Wikipedia 最终填补 ({} 主题) ━━━", FINAL_TOPICS.len());
    let mut ok = 0usize;
    for (i, topic) in FINAL_TOPICS.iter().enumerate() {
        let t = Instant::now();
        let n = ing.wikipedia(topic);
        if n > 0 {
            ok += 1;
            if ok <= 3 || ok % 5 == 0 {
                println!("  [{:>3}/{}] {} +{} ({:.1}s)",
                    i + 1, FINAL_TOPICS.len(), topic.replace('_', " "), n, t.elapsed().as_secs_f64());
            }
        } else {
            if ok <= 3 || i <= 5 {
                println!("  [{:>3}/{}] {} ⚠ 失败", i + 1, FINAL_TOPICS.len(), topic.replace('_', " "));
            }
        }
    }
    println!("  Wikipedia: {} 成功 (共 {} 主题)", ok, FINAL_TOPICS.len());

    println!("\n━━━ Phase 2: 去重 ━━━");
    let deduped = ing.dedup();
    println!("  合并 {} 个重复节点", deduped);

    let elapsed = overall.elapsed();
    ing.report("最终知识库状态", &before, elapsed);

    let after = ing.stats();

    println!("  域分布 (top 15):");
    let mut domains: Vec<_> = after.by_domain.iter().collect();
    domains.sort_by(|a, b| b.1.cmp(&a.1));
    for (d, c) in domains.iter().take(15) {
        let before_c = before.by_domain.iter().find(|(k, _)| k == d).map(|(_, v)| v).copied().unwrap_or(0);
        let delta = if c > &before_c { format!(" +{}", c - before_c) } else { String::new() };
        println!("    {:>30}: {:>4}{}", d, c, delta);
    }

    println!("\n  搜索测试:");
    for q in &["consciousness", "transformer", "reinforcement learning", "vector symbolic"] {
        match ing.kb().search(q, 3) {
            Ok(r) => println!("    \"{}\": {} 结果 (top: {:?})", q, r.len(),
                r.first().map(|x| &x.node.title)),
            Err(_) => (),
        }
    }
}
