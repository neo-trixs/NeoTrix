use std::time::Instant;

use neotrix::neotrix::nt_memory_kb::nt_memory_ingest::KBIngester;
use neotrix::neotrix::nt_memory_kb::{NodeType, RelationType};

const RETRY_TOPICS: &[(&str, &str)] = &[
    ("Transformer (deep learning)", "Transformer (deep learning)"),
    ("Meta-learning (computer science)", "Meta-learning (computer science)"),
    ("Mixture of experts", "Mixture of experts"),
    ("Support vector machine", "Support vector machine"),
    ("K-means clustering", "K-means clustering"),
    ("Cross-validation (statistics)", "Cross-validation (statistics)"),
    ("Reasoning system", "Reasoning system"),
    ("Automated reasoning", "Automated reasoning"),
    ("Probabilistic programming", "Probabilistic programming"),
    ("Graphical model", "Graphical model"),
    ("Inductive logic programming", "Inductive logic programming"),
    ("Case-based reasoning", "Case-based reasoning"),
    ("Emergent algorithm", "Emergent algorithm"),
    ("Neuro-symbolic AI", "Neuro-symbolic AI"),
];

const MANUAL_CONCEPTS: &[(&str, &str, &str)] = &[
    ("Model parallelism", "concept",
     "Technique for distributing a neural network across multiple devices by splitting model layers. Enables training beyond single-GPU memory limits."),
    ("Few-shot learning", "method",
     "ML paradigm where a model learns from very few labeled examples. Implemented via meta-learning, prompt engineering, or memory augmentation."),
    ("Symbolic artificial intelligence", "theory",
     "AI paradigm based on symbol manipulation, logical rules, and explicit knowledge representation. Contrasts with connectionist/sub-symbolic approaches."),
    ("Data parallelism", "method",
     "Parallelization strategy replicating the same model across devices, each processing different data shards with gradient synchronization."),
];

const NEW_SEEDS: &[&str] = &[
    "Computer vision", "Image segmentation", "Object detection",
    "Edge detection",
    "Natural language processing", "Language model", "Tokenization",
    "Part-of-speech tagging", "Named-entity recognition",
    "Sentiment analysis", "Machine translation",
    "Reinforcement learning", "Q-learning", "Policy gradient methods",
    "Markov decision process", "Temporal difference learning",
    "Neuroscience", "Biological neural network", "Cognition",
    "Synaptic plasticity", "Hebbian theory",
    "AI safety", "Interpretability",
    "Fairness (machine learning)", "Adversarial machine learning",
    "Automated machine learning", "Ensemble learning",
    "Boosting (machine learning)",
    "Linear algebra", "Calculus", "Probability theory",
    "Information theory", "Bayesian inference",
    "Convex optimization", "Entropy (information theory)",
    "Distributed computing",
    "Feature engineering", "Hyperparameter optimization",
    "Data augmentation", "CUDA", "GPU computing",
    "Value function", "Monte Carlo method",
];

const ARXIV_PAPERS: &[&str] = &[
    "1512.03385", "1706.03762", "1509.02971",
    "1606.03476", "1611.07004",
    "1806.07366", "1904.01681", "1904.09128",
    "2101.01169", "2208.11046",
];

const EDGE_PAIRS: &[(&str, &str, RelationType, f64, &str)] = &[
    ("Attention (machine learning)", "Transformer (deep learning)", RelationType::PrerequisiteOf, 1.0, "Attention is core mechanism of Transformer"),
    ("Transformer (deep learning)", "Language model", RelationType::DependsOn, 1.0, "Transformers power modern LMs"),
    ("Transformer (deep learning)", "Machine translation", RelationType::DependsOn, 1.0, "Transformer for NMT"),
    ("Transformer (deep learning)", "Computer vision", RelationType::Related, 0.7, "ViT adaptation"),
    ("Convolutional neural network", "Convolution", RelationType::DependsOn, 1.0, "CNN uses convolution operation"),
    ("Convolutional neural network", "Computer vision", RelationType::DependsOn, 1.0, "CNN backbone for CV"),
    ("Recurrent neural network", "Backpropagation", RelationType::DependsOn, 1.0, "BPTT"),
    ("Long short-term memory", "Recurrent neural network", RelationType::SubclassOf, 1.0, "LSTM is RNN variant"),
    ("Graph neural network", "Convolutional neural network", RelationType::ExtensionOf, 0.7, "GNN extends CNN to graphs"),
    ("Graph neural network", "Graphical model", RelationType::Related, 0.7, "Both handle relational data"),
    ("Generative adversarial network", "Computer vision", RelationType::Related, 0.6, "GAN for image generation"),
    ("Variational autoencoder", "Bayesian inference", RelationType::DependsOn, 1.0, "VAE uses variational inference"),
    ("Diffusion model", "Generative adversarial network", RelationType::Related, 0.7, "Diffusion surpasses GAN"),
    ("Meta-learning (computer science)", "Transfer learning", RelationType::Related, 0.8, "Related paradigms"),
    ("Transfer learning", "Natural language processing", RelationType::Related, 0.7, "Transfer in NLP"),
    ("Few-shot learning", "Meta-learning (computer science)", RelationType::SubclassOf, 1.0, "Few-shot via meta-learning"),
    ("Backpropagation", "Stochastic gradient descent", RelationType::PrerequisiteOf, 1.0, "SGD uses backprop gradients"),
    ("Stochastic gradient descent", "Convex optimization", RelationType::SubclassOf, 0.7, "SGD variant for convex opt"),
    ("Random forest", "Decision tree learning", RelationType::ExtensionOf, 1.0, "RF = ensemble of trees"),
    ("Random forest", "Ensemble learning", RelationType::SubclassOf, 1.0, "RF is ensemble method"),
    ("Support vector machine", "Convex optimization", RelationType::DependsOn, 0.8, "SVM = convex optimization"),
    ("Principal component analysis", "Dimensionality reduction", RelationType::SubclassOf, 1.0, "PCA is DR technique"),
    ("Principal component analysis", "Linear algebra", RelationType::DependsOn, 1.0, "PCA requires linear algebra"),
    ("k-means clustering", "Unsupervised learning", RelationType::SubclassOf, 1.0, "k-means is unsupervised"),
    ("Reasoning system", "Expert system", RelationType::Related, 0.8, "Expert systems are reasoning systems"),
    ("Automated reasoning", "Reasoning system", RelationType::SubclassOf, 0.8, "Automated reasoning in systems"),
    ("Expert system", "Symbolic artificial intelligence", RelationType::SubclassOf, 0.9, "Expert systems are symbolic AI"),
    ("Inductive logic programming", "Automated reasoning", RelationType::Related, 0.7, "ILP as automated reasoning"),
    ("Probabilistic programming", "Graphical model", RelationType::ExtensionOf, 0.8, "PP extends graphical models"),
    ("Probabilistic programming", "Bayesian inference", RelationType::DependsOn, 1.0, "PP built on Bayesian inference"),
    ("Graphical model", "Bayesian inference", RelationType::DependsOn, 1.0, "PGMs use Bayesian inference"),
    ("Neuro-symbolic AI", "Symbolic artificial intelligence", RelationType::ExtensionOf, 0.8, "Neuro-symbolic = neural + symbolic"),
    ("Reinforcement learning", "Markov decision process", RelationType::DependsOn, 1.0, "MDP is RL foundation"),
    ("Reinforcement learning", "Q-learning", RelationType::DependsOn, 1.0, "Q-learning is core RL algorithm"),
    ("Q-learning", "Temporal difference learning", RelationType::SubclassOf, 1.0, "Q-learning uses TD updates"),
    ("Value function", "Reinforcement learning", RelationType::DependsOn, 0.9, "Value functions core to RL"),
    ("Biological neural network", "Neural network", RelationType::Related, 0.8, "ANN inspired by BNN"),
    ("Neuroscience", "Cognition", RelationType::Related, 0.8, "Neuroscience studies cognition"),
    ("Synaptic plasticity", "Hebbian theory", RelationType::DependsOn, 0.9, "Hebbian plasticity"),
    ("AI safety", "Interpretability", RelationType::Related, 0.8, "Interpretability aids safety"),
    ("Adversarial machine learning", "Robustness (machine learning)", RelationType::Related, 0.8, "Adversarial tests robustness"),
    ("Probability theory", "Bayesian inference", RelationType::PrerequisiteOf, 1.0, "Probability → Bayesian inference"),
    ("Information theory", "Entropy (information theory)", RelationType::Related, 0.9, "Entropy ∈ information theory"),
    ("Calculus", "Backpropagation", RelationType::PrerequisiteOf, 1.0, "Backprop requires calculus"),
    ("Automated machine learning", "Hyperparameter optimization", RelationType::Related, 0.9, "AutoML includes HP optimization"),
    ("Ensemble learning", "Random forest", RelationType::DependsOn, 1.0, "RF is ensemble method"),
    ("Ensemble learning", "Boosting (machine learning)", RelationType::Related, 0.8, "Boosting is ensemble method"),
    ("Natural language processing", "Language model", RelationType::SubclassOf, 1.0, "LM task ∈ NLP"),
    ("Natural language processing", "Machine translation", RelationType::SubclassOf, 1.0, "MT task ∈ NLP"),
    ("Natural language processing", "Sentiment analysis", RelationType::SubclassOf, 1.0, "Sentiment task ∈ NLP"),
    ("Language model", "Transformer (deep learning)", RelationType::Related, 1.0, "Transformer-based LMs"),
    ("Language model", "Tokenization", RelationType::DependsOn, 1.0, "Tokenization preprocesses LM input"),
    ("Computer vision", "Image segmentation", RelationType::SubclassOf, 1.0, "Segmentation ∈ CV"),
    ("Computer vision", "Object detection", RelationType::SubclassOf, 1.0, "Detection ∈ CV"),
    ("Computer vision", "Convolutional neural network", RelationType::DependsOn, 0.9, "CNN backbone for CV"),
    ("Object detection", "Convolutional neural network", RelationType::DependsOn, 0.8, "CNN-based detection"),
    ("Distributed computing", "Data parallelism", RelationType::Related, 0.8, "Data parallelism in distributed systems"),
    ("Distributed computing", "Model parallelism", RelationType::Related, 0.8, "Model parallelism in distributed training"),
    ("GPU computing", "CUDA", RelationType::DependsOn, 1.0, "CUDA enables GPU computing"),
    ("GPU computing", "Convolutional neural network", RelationType::Related, 0.7, "GPU training of CNNs"),
    ("Feature engineering", "Dimensionality reduction", RelationType::Related, 0.7, "DR as feature engineering"),
    ("Data augmentation", "Convolutional neural network", RelationType::Related, 0.7, "Augmentation for CNN training"),
];

fn main() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  NeoTrix 深度探索 — 补全缺失知识                    ║");
    println!("╚══════════════════════════════════════════════════════╝");

    let mut ing = KBIngester::open(None).expect("open");
    let overall = Instant::now();
    let before = ing.snapshot();

    println!("\n  当前: {} 节点, {} 边", before.total_nodes, before.total_edges);

    // Phase 1: Retry Wikipedia
    println!("\n━━━ 1/6 Wikipedia 重试 ({} topics) ━━━", RETRY_TOPICS.len());
    let mut wp_ok = 0u32;
    for (i, &(title, correct_title)) in RETRY_TOPICS.iter().enumerate() {
        let t = Instant::now();
        let n = ing.wikipedia(correct_title);
        if n > 0 {
            println!("  [{:>2}/{}] {:>45} +OK ({:.1}s)",
                i + 1, RETRY_TOPICS.len(), title, t.elapsed().as_secs_f64());
            wp_ok += 1;
        } else {
            println!("  [{:>2}/{}] {:>45} ✗ (skip) ({:.1}s)",
                i + 1, RETRY_TOPICS.len(), title, t.elapsed().as_secs_f64());
        }
    }
    println!("  Wikipedia: {} OK / {} FAIL", wp_ok, RETRY_TOPICS.len() as u32 - wp_ok);

    // Phase 2: Manual concept nodes
    println!("\n━━━ 2/6 手动概念节点 ({} topics) ━━━", MANUAL_CONCEPTS.len());
    for (i, &(title, ntype_s, summary)) in MANUAL_CONCEPTS.iter().enumerate() {
        let ntype = NodeType::from_str(ntype_s);
        match ing.try_node(title, ntype, summary, None, "deep_learning") {
            Some(id) => println!("  [{:>2}/{}] {:>40} + ({})", i + 1, MANUAL_CONCEPTS.len(), title, &id[..8]),
            None => println!("  [{:>2}/{}] {:>40} ✗ ", i + 1, MANUAL_CONCEPTS.len(), title),
        }
    }

    // Phase 3: New Wikipedia seeds
    println!("\n━━━ 3/6 新 Wikipedia 种子 ({} topics) ━━━", NEW_SEEDS.len());
    let mut seed_ok = 0u32;
    for (i, topic) in NEW_SEEDS.iter().enumerate() {
        let n = ing.wikipedia(topic);
        if n > 0 {
            seed_ok += 1;
            if seed_ok <= 3 || i % 15 == 0 {
                println!("  [{:>2}/{}] {:>35} +{}", i + 1, NEW_SEEDS.len(), topic, n);
            }
        }
    }
    println!("  新种子: {} OK (of {})", seed_ok, NEW_SEEDS.len());

    // Phase 4: Edge wiring — single batched call
    println!("\n━━━ 4/6 边连接 ━━━");
    let edges_created = ing.relate_many(EDGE_PAIRS);
    println!("  边创建: {}", edges_created);

    // Phase 5: ArXiv papers
    println!("\n━━━ 5/6 ArXiv 种子 ({} papers) ━━━", ARXIV_PAPERS.len());
    let mut arxiv_ok = 0u32;
    for (i, arxiv_id) in ARXIV_PAPERS.iter().enumerate() {
        let n = ing.arxiv(arxiv_id);
        if n > 0 {
            println!("  [{:>2}/{}] {} +{}", i + 1, ARXIV_PAPERS.len(), arxiv_id, n);
            arxiv_ok += 1;
        }
    }
    println!("  ArXiv: {} OK", arxiv_ok);

    // Phase 6: Dedup
    println!("\n━━━ 6/6 去重 ━━━");
    let n = ing.dedup();
    println!("  合并 {} 个重复节点", n);

    // Final report
    ing.report("深度探索", &before, overall.elapsed());

    // Search tests
    println!("\n  搜索测试:");
    for q in &["transformer", "computer vision", "reinforcement learning",
               "probabilistic", "attention", "few shot", "deep learning"] {
        match ing.kb().search(q, 3) {
            Ok(r) => println!("    \"{}\": {} 结果 (top: {})", q, r.len(),
                r.first().map(|x| &x.node.title).map(|s| s.as_str()).unwrap_or("—")),
            Err(e) => println!("    \"{}\": ✗ {}", q, e),
        }
    }
}
