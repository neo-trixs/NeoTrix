/// 意识层知识种子 — 补全 NeoTrix 核心理论与外部知识
use std::time::{Duration, Instant};
use neotrix::neotrix::nt_memory_kb::nt_memory_ingest::KBIngester;
use neotrix::neotrix::nt_memory_kb::{NodeType, RelationType, nt_memory_store as store};

/// 手工意识/哲学/系统概念节点
const CONCEPT_NODES: &[(&str, NodeType, &str, &str, &[(&str, RelationType, f64, &str)])] = &[
    ("Qualia", NodeType::Concept, "philosophy",
     "Individual instances of subjective, conscious experience. The 'what it is like' aspect of consciousness. Central to the hard problem of consciousness.",
     &[("Consciousness", RelationType::Related, 1.0, "Qualia are properties of conscious experience"),
       ("Philosophy of mind", RelationType::Related, 0.9, "Qualia studied in philosophy of mind")]),
    ("Self-evolution", NodeType::Theory, "ai_systems",
     "AI paradigm where a system autonomously modifies its own architecture, code, or weights. Core to NeoTrix SEAL self-iteration. Related to recursive self-improvement.",
     &[("Meta-learning (computer science)", RelationType::Related, 0.7, "Self-evolution via meta-learning"),
       ("Automated machine learning", RelationType::Related, 0.6, "Self-evolution as extreme AutoML")]),
    ("Predictive processing", NodeType::Theory, "neuroscience",
     "Neuroscientific theory where the brain continuously generates and updates a predictive model of nt_world_sense input. Minimizes prediction error via perception and action.",
     &[("Free Energy Principle", RelationType::SubclassOf, 1.0, "Predictive processing implements FEP"),
       ("Bayesian inference", RelationType::DependsOn, 0.9, "Predictive processing uses Bayesian inference"),
       ("Active inference", RelationType::Related, 0.9, "Active inference extends predictive processing")]),
    ("Active inference", NodeType::Theory, "neuroscience",
     "Framework in theoretical neuroscience where action and perception jointly minimize variational free energy. Agents act to make sensations match predictions.",
     &[("Free Energy Principle", RelationType::SubclassOf, 1.0, "Active inference derived from FEP"),
       ("Predictive processing", RelationType::Related, 0.9, "Active inference extends predictive processing"),
       ("Reinforcement learning", RelationType::Related, 0.7, "Active inference as alternative to RL")]),
    ("AI alignment", NodeType::Theory, "ai_safety",
     "Research field ensuring AI systems pursue intended goals. Subproblems: value learning, corrigibility, interpretability, robustness. Central to existential safety.",
     &[("AI safety", RelationType::SubclassOf, 1.0, "Alignment is core subproblem of AI safety"),
       ("Interpretability", RelationType::Related, 0.9, "Interpretability aids alignment"),
       ("Reinforcement learning", RelationType::Related, 0.6, "RLHF for alignment")]),
    ("Open-ended evolution", NodeType::Theory, "ai_systems",
     "Systems that generate novel, increasingly complex patterns/behaviors without external direction. Aims to create truly creative AI. Related to novelty search.",
     &[("Emergent algorithm", RelationType::Related, 0.9, "Emergence in open-ended evolution"),
       ("Self-evolution", RelationType::Related, 0.8, "Open-ended evolution enables self-evolution")]),
    ("Recursive self-improvement", NodeType::Theory, "ai_safety",
     "Hypothetical scenario where an AI system improves its own intelligence without human intervention, leading to rapid capability gains. Core to FOOM/alignment debates.",
     &[("AI alignment", RelationType::Related, 0.9, "RSI is key alignment concern"),
       ("Self-evolution", RelationType::Related, 0.8, "RSI is self-evolution with capability gain"),
       ("AI safety", RelationType::Related, 0.8, "RSI raises safety questions")]),
    ("Hyperdimensional computing", NodeType::Method, "vsa",
     "Computing paradigm using high-dimensional random vectors (e.g., 10,000-D) as basic representations. Operations: bundle, bind, permute. Used for VSA. Also called HDC.",
     &[("Vector Symbolic Architecture", RelationType::Related, 1.0, "HDC is foundation for VSA"),
       ("HyperCube (NeoTrix)", RelationType::Related, 0.8, "HyperCube uses HDC principles")]),
    ("Holographic Reduced Representations", NodeType::Method, "vsa",
     "Tony Plate's vector-symbolic architecture using circular convolution for binding and vector addition for bundling. Foundation of modern VSA research.",
     &[("Vector Symbolic Architecture", RelationType::SubclassOf, 1.0, "HRR is a VSA implementation"),
       ("Hyperdimensional computing", RelationType::SubclassOf, 0.9, "HRR ∈ HDC"),
       ("HyperCube (NeoTrix)", RelationType::Related, 0.7, "HRR informs HyperCube design")]),
    ("The Global Neuronal Workspace", NodeType::Theory, "consciousness",
     "Dehaene's neurobiological theory where consciousness arises from global availability of information in a distributed neuronal workspace. Brain-scale version of Baars' GWT.",
     &[("Global Workspace Theory", RelationType::ExtensionOf, 1.0, "GNW extends Baars' psychological GWT"),
       ("Consciousness", RelationType::Related, 1.0, "GNW theory of consciousness"),
       ("Attention (machine learning)", RelationType::Related, 0.6, "Attention mechanisms loosely inspired by GNW")]),
    ("Liquid neural network", NodeType::Method, "neural_networks",
     "Time-continuous neural network using ODE-based dynamics with time-constant parameters. Processes temporal information at continuous timescales. Naturally handles irregularly sampled data.",
     &[("Recurrent neural network", RelationType::ExtensionOf, 0.8, "Liquid networks extend RNN"),
       ("Long short-term memory", RelationType::Related, 0.6, "Alternative to LSTM for temporal data")]),
    ("State space model", NodeType::Method, "deep_learning",
     "Sequence modeling framework based on linear state-space representations. Modern deep SSMs (Mamba, S4) achieve transformer-comparable performance with linear complexity.",
     &[("Mamba State Space Model", RelationType::SubclassOf, 1.0, "Mamba is a deep SSM"),
       ("Transformer (deep learning)", RelationType::Related, 0.7, "SSM as alternative to transformer for sequences"),
       ("Recurrent neural network", RelationType::ExtensionOf, 0.6, "SSM generalizes RNN")]),
    ("World model", NodeType::Method, "reinforcement_learning",
     "Internal predictive model of the environment used by agents to simulate outcomes, plan actions, and learn efficiently. Foundation of model-based RL and imagination-augmented agents.",
     &[("Reinforcement learning", RelationType::SubclassOf, 1.0, "World models used in model-based RL"),
       ("Predictive processing", RelationType::Related, 0.8, "World models paralleling predictive coding"),
       ("Markov decision process", RelationType::DependsOn, 0.8, "World model approximates MDP dynamics")]),
];

/// ArXiv papers for consciousness theory + foundational NeoTrix concepts
const ARXIV_PAPERS: &[(&str, &str)] = &[
    ("0711.0770", "E8 Lie group theory of everything"),
    ("2308.08708", "Consciousness in artificial intelligence"),
    ("0907.2754", "Hyperdimensional computing from Kanerva"),
    ("cs/0309048", "Gödel machine: self-referential universal problem solvers"),
    ("nlin/0310022", "Open-ended evolution definition"),
    ("1401.3972", "Active inference and learning"),
    ("2006.04439", "Liquid time-constant neural networks"),
    ("1803.10122", "World models from Ha and Schmidhuber"),
    ("2312.00752", "Mamba: linear-time sequence modeling"),
    ("2305.13048", "RWKV: Receptance Weighted Key Value"),
];

fn find_node(conn: &rusqlite::Connection, title: &str) -> Option<String> {
    for t in &["concept", "method", "theory", "algorithm", "article"] {
        if let Ok(mut stmt) = conn.prepare("SELECT id FROM nodes WHERE title = ?1 AND node_type = ?2") {
            if let Ok(rows) = stmt.query_map(rusqlite::params![title, t], |row| row.get::<_, String>(0)) {
                for row in rows { if let Ok(id) = row { return Some(id); } }
            }
        }
    }
    None
}

fn main() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  NeoTrix 意识层种子 — 核心理论 + 外部知识补全      ║");
    println!("╚══════════════════════════════════════════════════════╝");
    let mut ing = KBIngester::open(None).expect("KB open");
    let overall = Instant::now();
    let before = ing.snapshot();

    // Phase 1: Manual consciousness concept nodes
    println!("\n━━━ 1/4 手工理论概念节点 ({} nodes) ━━━", CONCEPT_NODES.len());
    let mut nodes_ok = 0u32;
    for (i, &(title, ref ntype, domain, summary, _edges)) in CONCEPT_NODES.iter().enumerate() {
        if ing.try_node(title, ntype.clone(), summary, None, domain).is_some() {
            nodes_ok += 1;
            println!("  [{:>2}/{}] {:>35} ✓", i + 1, CONCEPT_NODES.len(), title);
        } else {
            let err = ing.errors().last().map(|s| s.as_str()).unwrap_or("unknown error");
            println!("  [{:>2}/{}] {:>35} ✗ {}", i + 1, CONCEPT_NODES.len(), title, err);
        }
    }
    println!("  手工节点: {} OK", nodes_ok);

    // Phase 2: Edge wiring — consciousness theory relationships
    println!("\n━━━ 2/4 边连接 — 意识理论关系网 ━━━");
    let mut edges_ok = 0u32;
    for item in CONCEPT_NODES.iter() {
        let title = item.0;
        for &(target, ref rt, w, desc) in item.4 {
            if ing.relate(title, target, rt.clone(), w, desc) {
                edges_ok += 1;
            }
        }
    }
    // Cross-domain edges connecting consciousness to ML clusters
    let cross_edges: &[(&str, &str, RelationType, f64, &str)] = &[
        ("Consciousness", "Global Workspace Theory", RelationType::Related, 1.0,
         "GWT explains consciousness"),
        ("Consciousness", "Integrated Information Theory", RelationType::Related, 1.0,
         "IIT explains consciousness"),
        ("Consciousness", "Free Energy Principle", RelationType::Related, 0.7,
         "FEP relates to consciousness"),
        ("Consciousness", "Attention (machine learning)", RelationType::Related, 0.5,
         "Consciousness ↔ attention"),
        ("Global Workspace Theory", "Attention (machine learning)", RelationType::Related, 0.5,
         "GWT-inspired attention mechanisms"),
        ("Global Workspace Theory", "Transformer (deep learning)", RelationType::Related, 0.4,
         "GWT broadcast inspired transformer architecture"),
        ("Integrated Information Theory", "Vector Symbolic Architecture", RelationType::Related, 0.5,
         "Both represent holistic information structure"),
        ("Free Energy Principle", "Bayesian inference", RelationType::DependsOn, 1.0,
         "FEP relies on Bayesian inference"),
        ("Free Energy Principle", "Variational autoencoder", RelationType::Related, 0.7,
         "VAE implements free energy minimization"),
        ("World model", "Predictive processing", RelationType::Related, 0.8,
         "World models parallel predictive coding"),
        ("E8 Lie Group", "HyperCube (NeoTrix)", RelationType::Related, 0.6,
         "E8 structure informs HyperCube dimensions"),
        ("Vector Symbolic Architecture", "HyperCube (NeoTrix)", RelationType::Related, 0.9,
         "HyperCube implements VSA concepts"),
        ("Hyperdimensional computing", "HyperCube (NeoTrix)", RelationType::Related, 0.8,
         "HyperCube is HDC-based"),
        ("Active inference", "Reinforcement learning", RelationType::Related, 0.7,
         "Active inference as alternative to RL"),
        ("AI alignment", "AI safety", RelationType::SubclassOf, 1.0,
         "Alignment is core to AI safety"),
        ("AI alignment", "Reinforcement learning", RelationType::Related, 0.6,
         "RLHF for alignment"),
        ("State space model", "Recurrent neural network", RelationType::ExtensionOf, 0.6,
         "SSM generalizes RNN"),
        ("State space model", "Transformer (deep learning)", RelationType::Related, 0.7,
         "SSM as efficient transformer alternative"),
    ];
    edges_ok += ing.relate_many(cross_edges);
    println!("  边创建: {}", edges_ok);

    // Phase 3: ArXiv papers
    println!("\n━━━ 3/4 ArXiv 论文种子 ({} papers) ━━━", ARXIV_PAPERS.len());
    let mut arxiv_ok = 0u32;
    for (i, &(arxiv_id, title)) in ARXIV_PAPERS.iter().enumerate() {
        let n = ing.arxiv(arxiv_id);
        if n > 0 {
            arxiv_ok += 1;
            println!("  [{:>2}/{}] {:>35} +{} — {}", i + 1, ARXIV_PAPERS.len(), arxiv_id, n, title);
        } else {
            println!("  [{:>2}/{}] {:>35} ✗ failed — {}", i + 1, ARXIV_PAPERS.len(), arxiv_id, title);
        }
    }
    println!("  ArXiv: {} OK", arxiv_ok);

    // Phase 4: Wikipedia with rate-limited delays
    println!("\n━━━ 4/4 Wikipedia 限流重试 (带延迟) ━━━");
    let delay_seeds: &[&str] = &[
        "Attention", "Object_detection", "Edge_detection",
        "Natural_language_processing", "Fairness_(machine_learning)",
        "Boosting_(machine_learning)", "Zero-knowledge_proof",
        "Speech_synthesis", "AI_safety",
    ];
    let mut wp_ok = 0u32;
    for (i, topic) in delay_seeds.iter().enumerate() {
        std::thread::sleep(Duration::from_millis(750));
        let n = ing.wikipedia(topic);
        if n > 0 {
            wp_ok += 1;
            println!("  [{:>2}/{}] {:>35} +{}", i + 1, delay_seeds.len(), topic, n);
        } else {
            println!("  [{:>2}/{}] {:>35} ✗ failed", i + 1, delay_seeds.len(), topic);
        }
    }
    println!("  Wikipedia: {} OK (of {})", wp_ok, delay_seeds.len());

    // Dedup
    let deduped = ing.dedup();
    if deduped > 0 {
        ing.log(format!("Dedup removed {} duplicate nodes", deduped));
    }

    // Final report
    ing.report("意识层种子: 最终知识库状态", &before, overall.elapsed());

    // Type distribution
    let after = ing.stats();
    println!("  类型分布:");
    for (t, c) in &after.by_type {
        let b4 = before.by_type.iter().find(|(k, _)| k == t).map(|(_, v)| v).copied().unwrap_or(0i64);
        let d = if *c > b4 { format!(" +{}", c - b4) } else { String::new() };
        println!("    {:>20}: {:>4}{}", t, c, d);
    }

    // Consciousness concept coverage
    println!("\n  意识相关概念覆盖度:");
    let conn = ing.kb().conn.lock().expect("Lock");
    for kw in &["Consciousness", "Qualia", "Global Workspace", "Active inference", "Predictive processing",
                "Integrated Information", "AI alignment", "Self-evolution", "E8 Lie Group",
                "HyperCube", "Vector Symbolic Architecture", "Recursive self-improvement"] {
        let node_id = find_node(&conn, kw);
        if let Some(nid) = node_id {
            let ec = store::get_edges_for_node(&conn, &nid)
                .map(|v| v.len()).unwrap_or(0);
            println!("    {:>30}: ✅ ({} edges)", kw, ec);
        } else {
            let c = store::find_node_by_title_and_type(&conn, kw, &NodeType::Concept)
                .ok().flatten().or_else(|| {
                    store::find_node_by_title_and_type(&conn, kw, &NodeType::Theory).ok().flatten()
                });
            match c {
                Some(n) => println!("    {:>30}: ≈ {}", kw, n.title),
                None => println!("    {:>30}: ✗ (missing)", kw),
            }
        }
    }
    drop(conn);
}
