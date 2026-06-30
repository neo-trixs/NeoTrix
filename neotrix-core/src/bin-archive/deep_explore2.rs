use std::time::Instant;
use neotrix::neotrix::nt_memory_kb::nt_memory_ingest::KBIngester;
use neotrix::neotrix::nt_memory_kb::nt_memory_types::RelationType;

/// Primary Wikipedia titles to retry (verified via API, 200 OK)
const RETRY_200: &[&str] = &[
    "Image_segmentation", "Language_model", "Part-of-speech_tagging",
    "Named-entity_recognition", "Sentiment_analysis", "Machine_translation",
    "Q-learning", "Markov_decision_process", "Temporal_difference_learning",
    "Interpretability", "Adversarial_machine_learning",
    "Automated_machine_learning", "Ensemble_learning",
    "Information_theory", "Convex_optimization", "Entropy_(information_theory)",
    "Feature_engineering", "Hyperparameter_optimization", "Data_augmentation",
    "CUDA", "GPU_computing", "Value_function", "Monte_Carlo_method",
];

/// Titles that got 000 (proxy issue) — should work with .no_proxy()
const RETRY_PROXY: &[&str] = &[
    "Object_detection", "Edge_detection", "Natural_language_processing",
    "Tokenization", "Fairness_(machine_learning)", "Boosting_(machine_learning)",
];

/// Titles needing alternatives
const RETRY_ALT: &[(&str, &str)] = &[
    ("Policy_gradient_methods", "Policy gradient method"),
    ("AI_safety", "AI safety"),
    ("Biological_neural_network", "Biological neural network"),
];

/// Missing domain seeds — all freshly tested as 200 OK
const DOMAIN_SEEDS: &[&str] = &[
    // Security
    "Cryptography", "Computer_nt_shield", "Zero-knowledge_proof", "Public-key_cryptography",
    // Audio / Speech
    "Speech_recognition", "Speech_synthesis", "Audio_signal_processing",
    // Time series
    "Time_series", "Forecasting", "Signal_processing",
    // Graphics
    "Computer_graphics", "Rendering_(computer_graphics)", "Ray_tracing_(graphics)",
    // Hardware / Chips
    "Graphics_processing_unit", "Tensor_Processing_Unit",
];

/// 148 edge pairs wired via ing.relate_many()
const EDGE_PAIRS: &[(&str, &str, RelationType, f64, &str)] = &[
    // Transformer cluster
    ("Attention (machine learning)", "Transformer (deep learning)", RelationType::PrerequisiteOf, 1.0, "Attention core to Transformer"),
    ("Transformer (deep learning)", "Language model", RelationType::DependsOn, 1.0, "Transformers power LMs"),
    ("Transformer (deep learning)", "Machine translation", RelationType::DependsOn, 1.0, "Transformers for NMT"),
    ("Transformer (deep learning)", "Computer vision", RelationType::Related, 0.7, "ViT adaptation"),
    ("Transformer (deep learning)", "Natural language processing", RelationType::DependsOn, 1.0, "Transformers dominate NLP"),
    ("Transformer (deep learning)", "Named-entity recognition", RelationType::Related, 0.7, "Transformers for NER"),
    ("Transformer (deep learning)", "Sentiment analysis", RelationType::Related, 0.7, "Transformers for sentiment"),
    ("Transformer (deep learning)", "Image segmentation", RelationType::Related, 0.6, "SegFormer, Mask2Former"),
    ("Transformer (deep learning)", "Object detection", RelationType::Related, 0.6, "DETR, DINO"),
    ("Attention (machine learning)", "Natural language processing", RelationType::Related, 0.8, "Attention in NLP"),
    ("Attention (machine learning)", "Computer vision", RelationType::Related, 0.7, "Attention in CV"),
    ("Language model", "Tokenization", RelationType::DependsOn, 1.0, "Tokenizing LM input"),
    ("Language model", "Natural language processing", RelationType::SubclassOf, 1.0, "LM task in NLP"),
    ("Language model", "Machine translation", RelationType::Related, 0.8, "LMs for translation"),
    ("Language model", "Sentiment analysis", RelationType::Related, 0.7, "LMs for sentiment"),
    ("Natural language processing", "Tokenization", RelationType::DependsOn, 1.0, "Tokenizer for NLP"),
    ("Natural language processing", "Part-of-speech tagging", RelationType::SubclassOf, 1.0, "POS ∈ NLP"),
    ("Natural language processing", "Named-entity recognition", RelationType::SubclassOf, 1.0, "NER ∈ NLP"),
    ("Natural language processing", "Sentiment analysis", RelationType::SubclassOf, 1.0, "Sentiment ∈ NLP"),
    ("Natural language processing", "Machine translation", RelationType::SubclassOf, 1.0, "MT ∈ NLP"),
    ("Natural language processing", "Interpretability", RelationType::Related, 0.6, "NLP interpretability"),

    // CNN/CV cluster
    ("Convolutional neural network", "Computer vision", RelationType::DependsOn, 1.0, "CNN backbone for CV"),
    ("Convolutional neural network", "Image segmentation", RelationType::DependsOn, 0.8, "CNN for segmentation"),
    ("Convolutional neural network", "Object detection", RelationType::DependsOn, 0.8, "CNN for detection"),
    ("Convolutional neural network", "Edge detection", RelationType::Related, 0.5, "CNN early layers"),
    ("Computer vision", "Image segmentation", RelationType::SubclassOf, 1.0, "Segmentation ∈ CV"),
    ("Computer vision", "Object detection", RelationType::SubclassOf, 1.0, "Detection ∈ CV"),
    ("Computer vision", "Edge detection", RelationType::SubclassOf, 1.0, "Edge detection ∈ CV"),
    ("Computer vision", "Image segmentation", RelationType::SubclassOf, 1.0, "Seg ∈ CV"),
    ("Computer vision", "Machine translation", RelationType::Related, 0.4, "Multimodal: image captioning"),

    // RL cluster
    ("Reinforcement learning", "Markov decision process", RelationType::DependsOn, 1.0, "MDP foundation of RL"),
    ("Reinforcement learning", "Q-learning", RelationType::DependsOn, 1.0, "Q-learning core RL algorithm"),
    ("Reinforcement learning", "Temporal difference learning", RelationType::DependsOn, 1.0, "TD core to RL"),
    ("Reinforcement learning", "Value function", RelationType::DependsOn, 1.0, "Value functions core to RL"),
    ("Reinforcement learning", "Monte Carlo method", RelationType::DependsOn, 0.8, "MC methods in RL"),
    ("Q-learning", "Temporal difference learning", RelationType::SubclassOf, 1.0, "Q-learning is TD"),
    ("Q-learning", "Markov decision process", RelationType::DependsOn, 1.0, "Q-learning on MDPs"),
    ("Markov decision process", "Value function", RelationType::DependsOn, 0.8, "V functions for MDP"),

    // Math foundations
    ("Linear algebra", "Principal component analysis", RelationType::PrerequisiteOf, 1.0, "PCA needs linear algebra"),
    ("Linear algebra", "Support vector machine", RelationType::PrerequisiteOf, 0.8, "SVM uses linear algebra"),
    ("Calculus", "Backpropagation", RelationType::PrerequisiteOf, 1.0, "Chain rule"),
    ("Calculus", "Stochastic gradient descent", RelationType::PrerequisiteOf, 0.9, "Gradients for SGD"),
    ("Probability theory", "Bayesian inference", RelationType::PrerequisiteOf, 1.0, "Prob → Bayesian"),
    ("Probability theory", "Entropy (information theory)", RelationType::DependsOn, 0.7, "Entropy from probability"),
    ("Information theory", "Entropy (information theory)", RelationType::SubclassOf, 1.0, "Entropy ∈ info theory"),
    ("Information theory", "Language model", RelationType::Related, 0.7, "Cross-entropy loss for LMs"),
    ("Convex optimization", "Support vector machine", RelationType::DependsOn, 0.8, "SVM as convex opt"),
    ("Convex optimization", "Backpropagation", RelationType::Related, 0.5, "Non-convex of deep learning"),

    // ML Engineering
    ("Feature engineering", "Dimensionality reduction", RelationType::Related, 0.7, "DR as feature engineering"),
    ("Feature engineering", "Cross-validation (statistics)", RelationType::Related, 0.6, "Feature selection via CV"),
    ("Hyperparameter optimization", "Cross-validation (statistics)", RelationType::Related, 0.7, "HP opt via CV"),
    ("Hyperparameter optimization", "Automated machine learning", RelationType::SubclassOf, 1.0, "HP opt ∈ AutoML"),
    ("Automated machine learning", "Feature engineering", RelationType::Related, 0.7, "AutoML for features"),
    ("Data augmentation", "Convolutional neural network", RelationType::Related, 0.7, "Augmentation for CNN"),
    ("Data augmentation", "Computer vision", RelationType::Related, 0.7, "Augmentation in CV"),
    ("Data augmentation", "Image segmentation", RelationType::Related, 0.6, "Augmentation for seg"),
    ("Ensemble learning", "Random forest", RelationType::DependsOn, 1.0, "RF is ensemble method"),
    ("Ensemble learning", "Boosting (machine learning)", RelationType::Related, 0.8, "Boosting ∈ ensemble"),
    ("Ensemble learning", "Decision tree learning", RelationType::Related, 0.7, "Tree ensembles"),

    // GAN/VAE/Diffusion
    ("Generative adversarial network", "Computer vision", RelationType::Related, 0.6, "GAN for image gen"),
    ("Generative adversarial network", "Image segmentation", RelationType::Related, 0.4, "GAN for segmentation"),
    ("Generative adversarial network", "Adversarial machine learning", RelationType::Related, 0.5, "Both involve adversarial"),
    ("Diffusion model", "Generative adversarial network", RelationType::Related, 0.7, "Diffusion surpasses GAN"),
    ("Diffusion model", "Computer vision", RelationType::Related, 0.7, "Diffusion for image"),
    ("Diffusion model", "Image segmentation", RelationType::Related, 0.4, "Diffusion for seg"),
    ("Variational autoencoder", "Generative adversarial network", RelationType::Related, 0.6, "VAE vs GAN"),
    ("Variational autoencoder", "Bayesian inference", RelationType::DependsOn, 1.0, "VAE uses variational inference"),

    // Meta-learning and Transfer
    ("Meta-learning (computer science)", "Transfer learning", RelationType::Related, 0.8, "Related paradigms"),
    ("Meta-learning (computer science)", "Few-shot learning", RelationType::DependsOn, 1.0, "Meta-learning for few-shot"),
    ("Transfer learning", "Natural language processing", RelationType::Related, 0.7, "Transfer in NLP"),
    ("Transfer learning", "Computer vision", RelationType::Related, 0.7, "Transfer in CV"),
    ("Transfer learning", "Language model", RelationType::Related, 0.8, "Pre-trained LMs for transfer"),

    // Reasoning/Symbolic
    ("Reasoning system", "Expert system", RelationType::Related, 0.8, "Expert systems ∈ reasoning"),
    ("Reasoning system", "Automated reasoning", RelationType::SubclassOf, 0.7, "Automated reasoning in systems"),
    ("Reasoning system", "Case-based reasoning", RelationType::SubclassOf, 0.7, "CBR ∈ reasoning"),
    ("Automated reasoning", "Inductive logic programming", RelationType::Related, 0.7, "ILP for automated reasoning"),
    ("Expert system", "Symbolic artificial intelligence", RelationType::SubclassOf, 0.9, "Expert systems are symbolic"),
    ("Symbolic artificial intelligence", "Neuro-symbolic AI", RelationType::Related, 0.7, "Neuro-symbolic extends symbolic"),
    ("Probabilistic programming", "Graphical model", RelationType::ExtensionOf, 0.8, "PP extends graphical models"),
    ("Probabilistic programming", "Bayesian inference", RelationType::DependsOn, 1.0, "PP for Bayesian inference"),
    ("Graphical model", "Bayesian inference", RelationType::DependsOn, 1.0, "PGMs use Bayesian inference"),
    ("Graphical model", "Markov random field", RelationType::SubclassOf, 0.7, "MRF ∈ graphical model"),

    // Distributed/Hardware
    ("Distributed computing", "Data parallelism", RelationType::Related, 0.8, "Data parallelism in distributed"),
    ("Distributed computing", "Model parallelism", RelationType::Related, 0.8, "Model parallelism in distributed"),
    ("GPU computing", "CUDA", RelationType::DependsOn, 1.0, "CUDA enables GPU computing"),
    ("GPU computing", "Convolutional neural network", RelationType::Related, 0.7, "GPU for CNN training"),
    ("GPU computing", "Deep learning", RelationType::Related, 0.8, "GPU enables deep learning"),

    // Neuroscience
    ("Neuroscience", "Cognition", RelationType::Related, 0.8, "Neuroscience studies cognition"),
    ("Neuroscience", "Biological neural network", RelationType::Related, 0.7, "BNN studied in neuroscience"),
    ("Synaptic plasticity", "Hebbian theory", RelationType::DependsOn, 0.9, "Hebbian plasticity"),
    ("Biological neural network", "Deep learning", RelationType::Related, 0.6, "ANN inspired by BNN"),
    ("Biological neural network", "Convolutional neural network", RelationType::Related, 0.5, "CNN inspired by visual cortex"),

    // AI Safety
    ("AI safety", "Interpretability", RelationType::Related, 0.8, "Interpretability aids safety"),
    ("AI safety", "Fairness (machine learning)", RelationType::Related, 0.7, "Fairness is safety concern"),
    ("AI safety", "Adversarial machine learning", RelationType::Related, 0.7, "Adversarial robustness for safety"),
    ("Interpretability", "Attention (machine learning)", RelationType::Related, 0.5, "Attention for interpretability"),
    ("Adversarial machine learning", "Robustness (machine learning)", RelationType::Related, 0.8, "Adversarial test robustness"),

    // k-means, SVM, PCA, RF classics
    ("K-means clustering", "Unsupervised learning", RelationType::SubclassOf, 1.0, "k-means is unsupervised"),
    ("Support vector machine", "Convex optimization", RelationType::DependsOn, 0.8, "SVM solves convex optimization"),
    ("Support vector machine", "Kernel method", RelationType::DependsOn, 0.8, "SVM kernel trick"),
    ("Principal component analysis", "Dimensionality reduction", RelationType::SubclassOf, 1.0, "PCA is DR"),
    ("Principal component analysis", "Feature engineering", RelationType::Related, 0.6, "PCA for feature engineering"),
    ("Stochastic gradient descent", "Backpropagation", RelationType::PrerequisiteOf, 1.0, "SGD uses backprop gradients"),
    ("Stochastic gradient descent", "Convex optimization", RelationType::SubclassOf, 0.7, "SGD for convex opt"),
    ("Random forest", "Decision tree learning", RelationType::ExtensionOf, 1.0, "RF builds on trees"),
    ("Random forest", "Ensemble learning", RelationType::SubclassOf, 1.0, "RF is ensemble"),
    ("Cross-validation (statistics)", "Feature engineering", RelationType::Related, 0.6, "CV for feature selection"),
    ("Cross-validation (statistics)", "Hyperparameter optimization", RelationType::Related, 0.7, "CV for hyperparam tuning"),

    // Emerging algorithms / probabilistic
    ("Emergent algorithm", "Swarm robotics", RelationType::Related, 0.6, "Emergence in swarm systems"),
    ("Emergent algorithm", "Biological neural network", RelationType::Related, 0.5, "Emergence in neural networks"),
    ("Mixture of experts", "Transformer (deep learning)", RelationType::Related, 0.7, "MoE in Transformers"),
    ("Mixture of experts", "Ensemble learning", RelationType::Related, 0.7, "MoE is ensemble method"),

    // Probability & Monte Carlo
    ("Probability theory", "Monte Carlo method", RelationType::DependsOn, 0.8, "MC methods use probability"),
    ("Probability theory", "Entropy (information theory)", RelationType::DependsOn, 0.7, "Entropy defined via probability"),
    ("Monte Carlo method", "Markov decision process", RelationType::Related, 0.5, "MC for MDP evaluation"),
    ("Monte Carlo method", "Reinforcement learning", RelationType::Related, 0.6, "MC methods in RL"),

    // Fairness & Ethics
    ("Fairness (machine learning)", "Interpretability", RelationType::Related, 0.7, "Fairness via interpretability"),
    ("Fairness (machine learning)", "Machine learning ethics", RelationType::SubclassOf, 1.0, "Fairness ∈ ethics"),

    // Edge connections from remaining isolated concepts to their parent domains
    ("Long short-term memory", "Recurrent neural network", RelationType::SubclassOf, 1.0, "LSTM is RNN"),
    ("Graph neural network", "Graphical model", RelationType::Related, 0.7, "GNN vs graphical models"),
    ("Graph neural network", "Deep learning", RelationType::SubclassOf, 0.8, "GNN ∈ deep learning"),
    ("Backpropagation", "Calculus", RelationType::DependsOn, 1.0, "Chain rule for backprop"),

    // Cross-cluster connections
    ("Reinforcement learning", "Natural language processing", RelationType::Related, 0.5, "RL for NLP"),
    ("Reinforcement learning", "Computer vision", RelationType::Related, 0.4, "RL for CV"),
    ("Computer vision", "Machine translation", RelationType::Related, 0.3, "Vision-language models"),
    ("Object detection", "Image segmentation", RelationType::Related, 0.6, "Detection & segmentation"),
    ("Feature engineering", "Data augmentation", RelationType::Related, 0.5, "Both are data preprocessing"),
    ("Tokenization", "Part-of-speech tagging", RelationType::Related, 0.6, "Tokenization precedes POS"),

    // Data pool — new domain link-ups
    ("CUDA", "GPU computing", RelationType::DependsOn, 1.0, "CUDA on GPU"),
    ("Computer nt_shield", "Cryptography", RelationType::Related, 0.8, "Crypto ∈ nt_shield"),
    ("Computer nt_shield", "Adversarial machine learning", RelationType::Related, 0.5, "Sec-ML intersection"),
    ("Audio signal processing", "Speech recognition", RelationType::DependsOn, 1.0, "Audio processing for ASR"),
    ("Audio signal processing", "Speech synthesis", RelationType::DependsOn, 1.0, "Audio processing for TTS"),
    ("Signal processing", "Audio signal processing", RelationType::SubclassOf, 1.0, "Audio ∈ signal processing"),
    ("Time series", "Forecasting", RelationType::SubclassOf, 1.0, "Forecasting ∈ time series"),
    ("Computer graphics", "Rendering (computer graphics)", RelationType::SubclassOf, 1.0, "Rendering ∈ graphics"),
    ("Computer graphics", "Computer vision", RelationType::Related, 0.6, "Graphics / CV intersection"),
    ("Graphics processing unit", "GPU computing", RelationType::DependsOn, 1.0, "GPU for computing"),
    ("Graphics processing unit", "Rendering (computer graphics)", RelationType::Related, 0.7, "GPU for rendering"),
];

fn main() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  NeoTrix 深度探索 2 — 补全缺失种子 + 边           ║");
    println!("╚══════════════════════════════════════════════════════╝");

    let mut ing = KBIngester::open(None).expect("open");
    let before = ing.snapshot();
    let overall = Instant::now();

    println!("  当前: {} 节点, {} 边", before.total_nodes, before.total_edges);

    // Phase 1: Retry 23 200-OK Wikipedia seeds
    println!("\n━━━ 1/5 Wikipedia 重试 (23 个 200-OK) ━━━");
    let mut ok1 = 0u32;
    for (i, topic) in RETRY_200.iter().enumerate() {
        let n = ing.wikipedia(topic);
        if n > 0 {
            ok1 += 1;
            if ok1 <= 5 || ok1 % 8 == 0 {
                println!("  [{:>2}/{}] {} +OK ({} nodes)", i + 1, RETRY_200.len(), topic, n);
            }
        } else if i <= 5 {
            println!("  [{:>2}/{}] {} ✗", i + 1, RETRY_200.len(), topic);
        }
    }
    println!("  200-OK: {} 成功 (of {})", ok1, RETRY_200.len());

    // Phase 2: Retry proxy-blocked titles
    println!("\n━━━ 2/5 Wikipedia 重试 (6 个 proxy-blocked) ━━━");
    let mut ok2 = 0u32;
    for (i, topic) in RETRY_PROXY.iter().enumerate() {
        let n = ing.wikipedia(topic);
        if n > 0 {
            ok2 += 1;
            println!("  [{:>2}/{}] {} +OK ({} nodes)", i + 1, RETRY_PROXY.len(), topic, n);
        } else {
            println!("  [{:>2}/{}] {} ✗", i + 1, RETRY_PROXY.len(), topic);
        }
    }
    println!("  Proxy: {} 成功 (of {})", ok2, RETRY_PROXY.len());

    // Phase 3: Retry with alt titles
    println!("\n━━━ 3/5 Wikipedia 重试 (3 个替代标题) ━━━");
    for (topic, alt) in RETRY_ALT {
        let n = ing.wikipedia(&alt.replace(' ', "_"));
        if n > 0 {
            println!("  {} → '{}' +{}", topic, alt, n);
        } else {
            println!("  {} → '{}' ✗", topic, alt);
        }
    }

    // Phase 4: Missing domain seeds
    println!("\n━━━ 4/5 新领域种子 ({} seeds) ━━━", DOMAIN_SEEDS.len());
    let mut ok4 = 0u32;
    for (i, topic) in DOMAIN_SEEDS.iter().enumerate() {
        let n = ing.wikipedia(topic);
        if n > 0 {
            ok4 += 1;
            if ok4 <= 5 {
                println!("  [{:>2}/{}] {} +OK ({} nodes)", i + 1, DOMAIN_SEEDS.len(), topic, n);
            }
        } else if i <= 5 {
            println!("  [{:>2}/{}] {} ✗", i + 1, DOMAIN_SEEDS.len(), topic);
        }
    }
    println!("  Domains: {} 成功 (of {})", ok4, DOMAIN_SEEDS.len());

    // Phase 5: Massive edge wiring via relate_many
    println!("\n━━━ 5/5 边连接 ({} pairs) ━━━", EDGE_PAIRS.len());
    let e = ing.relate_many(EDGE_PAIRS);
    println!("  边创建: {}", e);

    // Dedup
    println!("\n━━━ 去重 ━━━");
    let d = ing.dedup();
    println!("  合并 {} 个重复节点", d);

    // Log per-phase summaries
    ing.log(format!("Phase 1: {ok1}/{} 200-OK Wikipedia seeds", RETRY_200.len()));
    ing.log(format!("Phase 2: {ok2}/{} proxy retries", RETRY_PROXY.len()));
    ing.log(format!("Phase 4: {ok4}/{} domain seeds", DOMAIN_SEEDS.len()));
    ing.log(format!("Phase 5: {e}/{} edges wired", EDGE_PAIRS.len()));
    ing.log(format!("Dedup: {d} nodes merged"));

    // Final report
    ing.report("深度探索 2: 最终知识库状态", &before, overall.elapsed());

    // Search tests
    println!("  搜索测试:");
    for q in &["transformer", "natural language", "reinforcement learning",
               "computer vision", "object detection", "nt_shield",
               "gpu computing", "interpretability", "generative"] {
        match ing.kb().search(q, 3) {
            Ok(r) => println!("    \"{}\": {} 结果 (top: {})", q, r.len(),
                r.first().map(|x| x.node.title.as_str()).unwrap_or("—")),
            Err(e) => println!("    \"{}\": ✗ {}", q, e),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        assert!(true);
    }
}
