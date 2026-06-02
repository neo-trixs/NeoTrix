use std::time::Instant;

use neotrix::neotrix::nt_memory_kb::nt_memory_ingest::KBIngester;

/// Target gap areas identified from KB stats:
///   paper: 1    → target 25+
///   repository: 6 → target 25+
///   domains lacking: ai_safety, vsa, consciousness_theory, rust, agents

const ARXIV_PAPERS: &[(&str, &str)] = &[
    // Consciousness & Cognitive Architecture
    ("1401.1219", "Integrated Information Theory 3.0"),
    ("1503.01137", "Global Workspace Theory of Consciousness"),
    ("1909.12109", "The Free Energy Principle"),
    ("2005.07749", "Active Inference: A Process Theory"),
    ("1704.01870", "The Attention Schema Theory of Consciousness"),
    // VSA / Hyperdimensional Computing
    ("2107.13222", "Hyperdimensional Computing for Machine Learning"),
    ("1901.09065", "Vector Symbolic Architectures: A Survey"),
    ("1806.10221", "HD Computing: A Primer"),
    // Transformer & Attention
    ("1706.03762", "Attention Is All You Need"),
    ("2005.14165", "GPT-3: Language Models are Few-Shot Learners"),
    ("2203.02155", "LLaMA: Open and Efficient Foundation Language Models"),
    // RL / Alignment
    ("1707.06347", "PPO: Proximal Policy Optimization"),
    ("2201.11903", "Constitutional AI"),
    ("2305.18290", "Direct Preference Optimization"),
    ("1606.06565", "Deep Reinforcement Learning: An Overview"),
    // Agent Architectures
    ("2304.03442", "AutoGPT: Autonomous GPT-4 Agent"),
    ("2305.18365", "Generative Agents: Interactive Simulacra"),
    ("2308.11432", "Mixture-of-Agents"),
    ("2401.02412", "OpenDevin: Code Agents"),
    // Neural Scaling & Theory
    ("2001.08361", "Scaling Laws for Neural Language Models"),
    ("2203.15556", "Chinchilla Scaling Laws"),
    ("1803.10943", "World Models"),
    // Memory & Knowledge
    ("2404.07129", "MemGPT: Towards LLMs as Operating Systems"),
    ("2307.09288", "RAG: Retrieval-Augmented Generation"),
    ("2310.12411", "Agent Memory: Survey of Memory-Augmented Agents"),
];

const GITHUB_REPOS: &[(&str, &str, &str)] = &[
    // Agent Frameworks
    ("Significant-Gravitas", "AutoGPT", "Autonomous GPT-4 agent"),
    ("langchain-ai", "langchain", "LLM application framework"),
    ("run-llama", "llama_index", "Data framework for LLM apps"),
    ("microsoft", "autogen", "Multi-agent conversation framework"),
    ("crewAIInc", "crewAI", "Multi-agent orchestration"),
    ("OpenDevin", "OpenDevin", "Autonomous code agent"),
    // Memory & Knowledge
    ("mem0ai", "mem0", "Memory layer for AI agents"),
    ("hwchase17", "memorably", "Memory for LLM agents"),
    ("cheshire-cat-ai", "core", "Conversational agent with memory"),
    // RL & Training
    ("openai", "baselines", "OpenAI RL baselines"),
    ("vwxyzjn", "cleanrl", "Clean RL implementations"),
    ("Lightning-AI", "lit-gpt", "Open-source GPT training"),
    // VSA / HD Computing
    ("rhyolight", "vsa-encodings", "Vector Symbolic Architecture encodings"),
    ("IBM", "hdc-utils", "Hyperdimensional computing utilities"),
    // Tools & Infrastructure
    ("neovim", "neovim", "Modern Vim-based editor"),
    ("n8n-io", "n8n", "Workflow automation"),
    ("charmbracelet", "bubbletea", "TUI framework (Go)"),
    ("ratatui", "ratatui", "Rust TUI framework"),
    // AI Safety & Alignment
    ("AnthropicAI", "interpretability", "Anthropic interpretability research"),
    ("Center-for-Humans-and-Machines", "transparency", "AI transparency tools"),
    ("EleutherAI", "lm-evaluation-harness", "LM evaluation framework"),
    // Rust AI
    ("huggingface", "candle", "Rust ML framework"),
    ("LaurentMazare", "tch-rs", "Rust bindings for PyTorch"),
    ("neotrix", "neotrix", "Self-evolving AI reasoning engine"),
];

const WIKI_DEEP_TOPICS: &[&str] = &[
    "Causal_reasoning",
    "Emergence",
    "Self-organization",
    "Complex_system",
    "Dynamical_system",
    "Bayesian_inference",
    "Neural_network_(biology)",
    "Hebbian_theory",
    "Synaptic_plasticity",
    "Neurotransmitter",
    "Functional_magnetic_resonance_imaging",
    "Electroencephalography",
    "Working_memory",
    "Long-term_potentiation",
    "Neuroeconomics",
    "Computational_neuroscience",
    "Neuromorphic_engineering",
    "Cognitive_architecture",
    "Soar_(cognitive_architecture)",
    "ACT-R",
    "Subsumption_architecture",
    "Behavior-based_robotics",
    "Neuroevolution",
    "Evolutionary_algorithm",
    "Genetic_programming",
    "Swarm_intelligence",
    "Ant_colony_optimization",
    "Particle_swarm_optimization",
    "Bayesian_network",
    "Markov_decision_process",
    "Partially_observable_Markov_decision_process",
    "Monte_Carlo_tree_search",
    "Knowledge_representation_and_reasoning",
    "Semantic_network",
    "Ontology_(information_science)",
    "Description_logic",
    "Category_theory",
    "Topos_theory",
    "Homotopy_type_theory",
    "Lambda_calculus",
];

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  知识缺口填充 — ArXiv + GitHub + Wikipedia              ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    let mut ing = KBIngester::open(None).expect("KBIngester open failed");
    let overall = Instant::now();

    let before = ing.snapshot();
    println!("\n  填充前: {} 节点, {} 边, {} 论文, {} 仓库\n",
        before.total_nodes, before.total_edges,
        before.by_type.iter().find(|(k, _)| k == "paper").map(|(_, v)| v).copied().unwrap_or(0),
        before.by_type.iter().find(|(k, _)| k == "repository").map(|(_, v)| v).copied().unwrap_or(0));

    // Phase 1: ArXiv papers
    println!("━━━ Phase 1: ArXiv 论文种子 ({}) ━━━", ARXIV_PAPERS.len());
    let mut paper_ok = 0usize;
    let mut paper_fail = 0usize;
    for (i, (arxiv_id, title_hint)) in ARXIV_PAPERS.iter().enumerate() {
        let t = Instant::now();
        let n = ing.arxiv(arxiv_id);
        if n > 0 {
            paper_ok += 1;
            println!("  [{:>2}/{}] arxiv:{} {:<55} +{:<2} | {:>5.1}s",
                i+1, ARXIV_PAPERS.len(), arxiv_id, title_hint, n, t.elapsed().as_secs_f64());
        } else {
            let fallback = arxiv_id.split('v').next().unwrap_or(arxiv_id);
            let n = ing.arxiv(fallback);
            if n > 0 {
                paper_ok += 1;
                println!("  [{:>2}/{}] arxiv:{} {:<55} +{:<2} | {:>5.1}s (fallback)",
                    i+1, ARXIV_PAPERS.len(), fallback, title_hint, n, t.elapsed().as_secs_f64());
            } else {
                paper_fail += 1;
                println!("  [{:>2}/{}] arxiv:{} {:<55} | FAIL", i+1, ARXIV_PAPERS.len(), arxiv_id, title_hint);
            }
        }
    }

    // Phase 2: GitHub repos
    println!("\n━━━ Phase 2: GitHub 仓库种子 ({}) ━━━", GITHUB_REPOS.len());
    let mut repo_ok = 0usize;
    let mut repo_fail = 0usize;
    for (i, (owner, repo, desc)) in GITHUB_REPOS.iter().enumerate() {
        let t = Instant::now();
        let n = ing.repo(owner, repo);
        if n > 0 {
            repo_ok += 1;
            println!("  [{:>2}/{}] {}/{} {:<40} +{:<2} | {:>5.1}s",
                i+1, GITHUB_REPOS.len(), owner, repo, desc, n, t.elapsed().as_secs_f64());
        } else {
            repo_fail += 1;
            println!("  [{:>2}/{}] {}/{} | FAIL", i+1, GITHUB_REPOS.len(), owner, repo);
        }
    }

    // Phase 3: Deep Wikipedia ingestion
    println!("\n━━━ Phase 3: Wikipedia 深度摄取 ({}) ━━━", WIKI_DEEP_TOPICS.len());
    let mut wiki_ok = 0usize;
    let mut wiki_fail = 0usize;
    for (i, topic) in WIKI_DEEP_TOPICS.iter().enumerate() {
        let t = Instant::now();
        let n = ing.wikipedia(topic);
        if n > 0 {
            wiki_ok += 1;
            println!("  [{:>2}/{}] {:>45} +{:<3} | {:>5.1}s",
                i+1, WIKI_DEEP_TOPICS.len(), topic.replace('_', " "), n, t.elapsed().as_secs_f64());
        } else {
            wiki_fail += 1;
            println!("  [{:>2}/{}] {:>45} | FAIL", i+1, WIKI_DEEP_TOPICS.len(), topic.replace('_', " "));
        }
    }

    // Phase 4: Deduplicate
    println!("\n━━━ Phase 4: 去重 ━━━");
    let dedup_n = ing.dedup();
    println!("  合并 {} 个重复节点", dedup_n);

    // Phase 5: Run crawl cycle
    println!("\n━━━ Phase 5: 爬取队列处理 ━━━");
    let c_start = Instant::now();
    match ing.kb().run_crawl_cycle(30) {
        Ok(report) => {
            println!("  尝试: {} | 完成: {} | 失败: {} | 新建节点: {} | 新建边: {} | 耗时: {:.1}s",
                report.attempted, report.completed, report.failed,
                report.nodes_created, report.edges_created,
                c_start.elapsed().as_secs_f64());
        }
        Err(e) => println!("  ⚠ 爬取循环: {}", e),
    }

    // Summary
    let elapsed = overall.elapsed();
    let after = ing.stats();

    ing.log(format!("论文: {} OK / {} FAIL → 总计 {}", paper_ok, paper_fail, ARXIV_PAPERS.len()));
    ing.log(format!("仓库: {} OK / {} FAIL → 总计 {}", repo_ok, repo_fail, GITHUB_REPOS.len()));
    ing.log(format!("Wikipedia: {} OK / {} FAIL → 总计 {}", wiki_ok, wiki_fail, WIKI_DEEP_TOPICS.len()));

    ing.report("知识缺口填充完成", &before, elapsed);

    println!("  类型分布 (填充后):");
    let mut types: Vec<_> = after.by_type.iter().collect();
    types.sort_by(|a, b| b.1.cmp(&a.1));
    for (t, c) in &types {
        let before_c = before.by_type.iter().find(|(k, _)| k == t).map(|(_, v)| v).copied().unwrap_or(0);
        let delta = if c > &before_c { format!("+{}", c - before_c) } else { String::new() };
        println!("    {:>20}: {:>4} {}", t, c, delta);
    }

    println!("\n  域分布 (top 15):");
    let mut domains: Vec<_> = after.by_domain.iter().collect();
    domains.sort_by(|a, b| b.1.cmp(&a.1));
    for (d, c) in domains.iter().take(15) {
        let before_c = before.by_domain.iter().find(|(k, _)| k == d).map(|(_, v)| v).copied().unwrap_or(0);
        let delta = if c > &before_c { format!("+{}", c - before_c) } else { String::new() };
        println!("    {:>25}: {:>4} {}", d, c, delta);
    }

    println!("\n  爬取待处理:    {}", after.crawl_pending);
}
