/// 终局: 手工创建被 Wikipedia API 限流的节点 + 剩余边
/// 使用 KBIngester 可复用模块
use std::time::Instant;
use neotrix::neotrix::nt_memory_kb::{nt_memory_ingest::KBIngester, NodeType, RelationType};

const MANUAL: &[(&str, NodeType, &str, &str)] = &[
    ("Object detection", NodeType::Concept, "computer_vision",
     "CV task to detect object instances within images. YOLO, DETR, Faster R-CNN."),
    ("Edge detection", NodeType::Concept, "computer_vision",
     "Identify sharp discontinuities in image brightness. Canny, Sobel."),
    ("Natural language processing", NodeType::Concept, "nlp",
     "AI subfield for computer-human language interaction. Understanding, generation, translation."),
    ("Tokenization", NodeType::Concept, "nlp",
     "Splitting text into tokens (words, subwords). BPE, unigram, WordPiece."),
    ("Fairness (machine learning)", NodeType::Concept, "ai_ethics",
     "Bias, equity, justice in ML. Demographic parity, equal opportunity."),
    ("Boosting (machine learning)", NodeType::Method, "ml_engineering",
     "Ensemble combining weak learners. AdaBoost, XGBoost, LightGBM, CatBoost."),
    ("Zero-knowledge proof", NodeType::Concept, "cryptography",
     "Prove knowledge of secret without revealing it. zk-rollups, privacy."),
    ("Speech synthesis", NodeType::Concept, "audio",
     "Artificial speech generation (TTS). Tacotron, WaveNet, VITS."),
    ("Policy gradient method", NodeType::Method, "reinforcement_learning",
     "RL optimizing policy via gradient ascent. REINFORCE, PPO, A2C."),
    ("AI safety", NodeType::Theory, "ai_alignment",
     "Ensuring AI systems are beneficial and aligned with human values."),
    ("Biological neural network", NodeType::Concept, "neuroscience",
     "Biological neurons in nervous system. Inspires artificial neural networks."),
    ("Image segmentation", NodeType::Concept, "computer_vision",
     "Partition image into meaningful segments. U-Net, Mask R-CNN, SAM."),
];

const MORE_EDGES: &[(&str, &str, RelationType, f64, &str)] = &[
    ("Image segmentation", "Convolutional neural network", RelationType::DependsOn, 0.8, "CNN backbone for segmentation"),
    ("Image segmentation", "Object detection", RelationType::Related, 0.7, "Segmentation & detection"),
    ("Image segmentation", "Computer vision", RelationType::SubclassOf, 1.0, "Seg ∈ CV"),
    ("Object detection", "Computer vision", RelationType::SubclassOf, 1.0, "Detection ∈ CV"),
    ("Object detection", "Convolutional neural network", RelationType::DependsOn, 0.8, "CNN for detection"),
    ("Natural language processing", "Tokenization", RelationType::DependsOn, 1.0, "Tokenizer for NLP"),
    ("Natural language processing", "Language model", RelationType::SubclassOf, 1.0, "LM ∈ NLP"),
    ("Natural language processing", "Machine translation", RelationType::SubclassOf, 1.0, "MT ∈ NLP"),
    ("Natural language processing", "Sentiment analysis", RelationType::SubclassOf, 1.0, "Sentiment ∈ NLP"),
    ("Natural language processing", "Named-entity recognition", RelationType::SubclassOf, 1.0, "NER ∈ NLP"),
    ("Natural language processing", "Part-of-speech tagging", RelationType::SubclassOf, 1.0, "POS ∈ NLP"),
    ("Natural language processing", "Interpretability", RelationType::Related, 0.6, "NLP interpretability"),
    ("Natural language processing", "Fairness (machine learning)", RelationType::Related, 0.6, "NLP fairness"),
    ("Tokenization", "Language model", RelationType::DependsOn, 1.0, "Tokenizer for LM"),
    ("Tokenization", "Part-of-speech tagging", RelationType::Related, 0.6, "Tokens for POS"),
    ("Boosting (machine learning)", "Ensemble learning", RelationType::SubclassOf, 1.0, "Boosting ∈ ensemble"),
    ("Boosting (machine learning)", "Decision tree learning", RelationType::DependsOn, 0.8, "Boosted trees"),
    ("Boosting (machine learning)", "Random forest", RelationType::Related, 0.7, "Boosting vs bagging"),
    ("Fairness (machine learning)", "Interpretability", RelationType::Related, 0.7, "Fairness via interpretability"),
    ("Fairness (machine learning)", "AI safety", RelationType::Related, 0.7, "Fairness ∈ AI safety"),
    ("Policy gradient method", "Reinforcement learning", RelationType::SubclassOf, 1.0, "PG ∈ RL"),
    ("Policy gradient method", "Value function", RelationType::Related, 0.7, "Actor-critic uses V-functions"),
    ("AI safety", "Interpretability", RelationType::Related, 0.8, "Interpretability aids safety"),
    ("AI safety", "Adversarial machine learning", RelationType::Related, 0.7, "Adversarial robustness for safety"),
    ("AI safety", "Fairness (machine learning)", RelationType::Related, 0.6, "Fairness ∈ safety"),
    ("Biological neural network", "Deep learning", RelationType::Related, 0.6, "ANN inspired by BNN"),
    ("Biological neural network", "Convolutional neural network", RelationType::Related, 0.5, "CNN inspired by visual cortex"),
    ("Biological neural network", "Neuroscience", RelationType::Related, 0.7, "BNN studied in neuroscience"),
    ("Zero-knowledge proof", "Cryptography", RelationType::SubclassOf, 1.0, "ZKP ∈ cryptography"),
    ("Zero-knowledge proof", "Computer nt_shield", RelationType::Related, 0.7, "ZKP for nt_shield"),
    ("Speech synthesis", "Audio signal processing", RelationType::SubclassOf, 0.8, "TTS uses audio processing"),
    ("Speech synthesis", "Natural language processing", RelationType::Related, 0.6, "TTS & NLP"),
    ("Edge detection", "Computer vision", RelationType::SubclassOf, 1.0, "Edge ∈ CV"),
    ("Edge detection", "Convolutional neural network", RelationType::Related, 0.5, "CNN early layers detect edges"),
    ("Reinforcement learning", "Policy gradient method", RelationType::DependsOn, 1.0, "PG key RL method"),
    ("Interpretability", "Attention (machine learning)", RelationType::Related, 0.5, "Attention for interpretability"),
    ("Computer nt_shield", "Cryptography", RelationType::Related, 0.8, "Crypto ∈ nt_shield"),
    ("GPU computing", "Graphics processing unit", RelationType::DependsOn, 1.0, "GPU for computing"),
    ("GPU computing", "CUDA", RelationType::DependsOn, 1.0, "CUDA on GPU"),
    ("Speech recognition", "Audio signal processing", RelationType::SubclassOf, 1.0, "ASR ∈ audio processing"),
    ("Speech recognition", "Natural language processing", RelationType::Related, 0.7, "ASR feeds NLP"),
    ("Computer graphics", "Computer vision", RelationType::Related, 0.6, "Graphics—CV intersection"),
    ("Computer graphics", "Rendering (computer graphics)", RelationType::SubclassOf, 1.0, "Rendering ∈ graphics"),
    ("Time series", "Signal processing", RelationType::Related, 0.7, "Time series processing"),
    ("Time series", "Forecasting", RelationType::SubclassOf, 1.0, "Forecasting ∈ time series"),
];

fn main() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  NeoTrix 深度探索 3 — 手工补全 + 终局              ║");
    println!("╚══════════════════════════════════════════════════════╝");

    let mut ing = KBIngester::open(None).expect("KBIngester open");
    let overall = Instant::now();
    let before = ing.snapshot();

    // Phase 1: Manual concept nodes
    println!("\n━━━ 1/3 手工概念节点 ({} nodes) ━━━", MANUAL.len());
    for (i, &(title, ref ntype, domain, summary)) in MANUAL.iter().enumerate() {
        let id = ing.try_node(title, ntype.clone(), summary, None, domain);
        match id {
            Some(_) => println!("  [{:>2}/{}] {:>40} +", i + 1, MANUAL.len(), title),
            None => println!("  [{:>2}/{}] {:>40} ✓ exists", i + 1, MANUAL.len(), title),
        }
    }
    ing.log(format!("Manual nodes attempted: {}", MANUAL.len()));

    // Phase 2: Edge wiring
    println!("\n━━━ 2/3 边连接 ({} pairs) ━━━", MORE_EDGES.len());
    let edges_ok = ing.relate_many(&MORE_EDGES);
    println!("  边创建: {} / {}", edges_ok, MORE_EDGES.len());
    ing.log(format!("Edges: {}/{} created", edges_ok, MORE_EDGES.len()));

    // Dedup + crawl
    let deduped = ing.dedup();
    let _ = ing.kb().run_crawl_cycle(0);
    ing.log(format!("Duplicates merged: {}", deduped));

    // Phase 3: Search tests
    println!("\n━━━ 3/3 搜索测试 ━━━");
    for q in &["transformer", "natural language", "reinforcement learning", "computer vision",
               "object detection", "nt_shield", "interpretability", "tokenization"] {
        match ing.kb().search(q, 3) {
            Ok(r) => println!("    \"{}\": {} 结果 (top: {})", q, r.len(),
                r.first().map(|x| x.node.title.as_str()).unwrap_or("—")),
            Err(e) => println!("    \"{}\": ✗ {}", q, e),
        }
    }

    ing.report("深度探索 3: 最终 KB 状态", &before, overall.elapsed());
}
