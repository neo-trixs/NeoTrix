/// 前沿知识种子 — E8/VSA/GWT/IIT/SSM/GödelMachine/MetaCognition 深度补全
use std::time::Instant;
use neotrix::neotrix::nt_memory_kb::nt_memory_ingest::KBIngester;
use neotrix::neotrix::nt_memory_kb::nt_memory_types::RelationType;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  NeoTrix 前沿知识种子 — E8/VSA/GWT/IIT/SSM/元认知/自修改   ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    let mut ing = KBIngester::open(None).expect("KB open");
    let overall = Instant::now();
    let before = ing.snapshot();

    // ====================================================================
    // CLUSTER 1: E8 Lie Theory & Exceptional Lie Algebras
    // ====================================================================
    println!("\n━━━ 1/8 E8 Lie 理论 & 例外李代数 ━━━");
    let e8: &[(&str, RelationType, f64, &str)] = &[];
    ing.try_concept("Exceptional Lie algebras", "Five exceptional simple Lie groups/lie algebras beyond the infinite families: G2, F4, E6, E7, E8. Classified by Killing (1887) and Cartan (1890). E8 is the largest (248-dim).", "mathematics");
    ing.try_concept("E8 Lie algebra", "248-dimensional exceptional simple Lie algebra, the largest in the Cartan-Killing classification. Root system of rank 8 with 240 roots; Weyl group order 696,729,600. Unique in that its smallest nontrivial representation is the adjoint.", "mathematics");
    ing.try_concept("Cartan-Killing classification", "Complete classification of finite-dimensional simple Lie algebras over complex numbers: four infinite families (An,Bn,Cn,Dn) plus five exceptional cases (G2,F4,E6,E7,E8). Discovered by Wilhelm Killing 1887, formalized by Élie Cartan 1890.", "mathematics");
    ing.try_concept("E8 lattice", "Even unimodular lattice in 8 dimensions formed by the E8 root system. Densest sphere packing in 8 dimensions (proven by Maryna Viazovska 2016, Fields Medal 2022). The Leech lattice Λ24 is the 24-dimensional analogue.", "mathematics");
    ing.try_concept("Octonion algebra", "8-dimensional non-associative division algebra. The octonions are central to constructions of exceptional Lie groups via triality. E8 can be described using oct-octonions (Barton-Sudbery construction, Kollross 2025).", "mathematics");
    ing.try_concept("String theory and E8", "E8 appears in heterotic string theory (E8×E8 symmetry group). The classification of E8 bundles is central to string phenomenology and M-theory. E8 is also connected to the theory of everything proposals.", "physics");
    e8.iter().for_each(|_| {});

    // ====================================================================
    // CLUSTER 2: VSA Advanced (beyond basic HDC/HRR)
    // ====================================================================
    println!("\n━━━ 2/8 VSA 高级架构 (MAP/FHRR/BSC/LARS) ━━━");
    ing.try_concept("Multiply-Add-Permute (MAP) architecture", "Ross Gayler's VSA using Hadamard product for binding (self-inverse with bipolar vectors), vector addition for bundling, and permutation for role-filler representation. Computationally simple: binding is elementwise multiply on {−1,+1} vectors.", "vsa");
    ing.try_concept("Fourier Holographic Reduced Representation (FHRR)", "VSA using complex-valued vectors (unit-magnitude complex numbers). Binding via elementwise complex multiplication (angle addition). Unbinding via complex conjugate. Continuous-valued, good similarity structure.", "vsa");
    ing.try_concept("Binary Spatter Code (BSC)", "Kanerva's VSA using binary {0,1} vectors with XOR as binding and majority-sum as bundling. Extremely efficient for hardware implementation. Each vector is self-inverse for binding.", "vsa");
    ing.try_concept("VSA binding taxonomy", "Schlegel et al (2021) taxonomy of VSA binding: (1) multiplicative (MAP, FHRR), (2) convolutional (HRR, BSC), (3) tensor-product, (4) permutation-based. Each has tradeoffs in capacity, exactness of unbinding, and noise robustness.", "vsa");
    ing.try_concept("LARS-VSA", "Learning Abstract Rules via VSA (Mejri et al 2024). Uses HD computing for relational bottleneck — separates object features from abstract rules. High-dimensional attention mechanism leveraging VSA binding operations. Resists interference from similar object features.", "vsa");

    // ====================================================================
    // CLUSTER 3: GWT/GNW Deep
    // ====================================================================
    println!("\n━━━ 3/8 GWT/GNW 意识理论前沿 ━━━");
    ing.try_concept("Baars' theater of consciousness", "Bernard Baars' central metaphor for GWT (1988, 1997): conscious contents = bright spot on stage (working memory), selected by spotlight of attention. Unconscious background processors = audience. Global broadcast enables flexible coordination.", "consciousness");
    ing.try_concept("Dehaene's Global Neuronal Workspace (GNW)", "Stanislas Dehaene and Lionel Naccache's neurobiological extension of Baars' GWT (2001). Conscious access triggers 'neuronal ignition' — widespread synchronized frontoparietal activation. Unconscious stimuli activate only local cortical regions.", "neuroscience");
    ing.try_concept("Contrastive analysis method", "Baars' methodology: compare closely matched conscious vs unconscious events to identify the neural correlates of consciousness. Key finding: conscious perception triggers frontoparietal broadcast; unconscious does not.", "consciousness");
    ing.try_concept("Cogitate Consortium", "Large-scale adversarial collaboration testing IIT vs GNWT predictions (led by Lucia Melloni). Designed experimental protocols where the two theories make divergent predictions about conscious perception. Represents a new paradigm in consciousness science.", "consciousness");
    ing.try_concept("GWT blackboard architecture origin", "GWT was inspired by the blackboard architecture from AI (Newell, 1960s): multiple specialized 'knowledge sources' communicate through a shared global blackboard (workspace). Baars adapted this as a cognitive architecture for consciousness.", "ai_systems");
    ing.try_concept("AI implementations of Global Workspace", "Multiple computational models of GWT: LIDA (Franklin), IDA, Global Workspace (Shanahan), sigma-GWT. Key design: multiple specialist modules compete for global broadcast; workspace holds current conscious contents; broadcast enables widespread cognitive access.", "ai_systems");

    // ====================================================================
    // CLUSTER 4: IIT & Consciousness Metrics
    // ====================================================================
    println!("\n━━━ 4/8 IIT & 意识可量化指标 ━━━");
    ing.try_concept("Integrated Information Theory (IIT)", "Giulio Tononi's theory (2004-present): consciousness = integrated information (Φ). IIT 3.0: causal analysis of a physical system via its state-transition graph. IIT 4.0: simplified with directed partitions and system-irreducibility analysis. Panpsychist implications.", "consciousness");
    ing.try_concept("Φ (phi) — integrated information", "Mathematical quantity measuring irreducible cause-effect power of a system. High Φ → high consciousness. Computed by partitioning system into two parts and measuring information loss. Criticized by Scott Aaronson for counterintuitive implications (e.g., photodiode has nonzero Φ).", "consciousness");
    ing.try_concept("Perturbational Complexity Index (PCI)", "Clinical measure of consciousness level: stimulate brain with TMS (transcranial magnetic stimulation) and measure complexity of evoked EEG response. High complexity = conscious. Used for detecting consciousness in coma/vegetative state patients (Casali et al 2013).", "neuroscience");
    ing.try_concept("AI consciousness indicators (Butlin et al 2025)", "Comprehensive framework evaluating AI systems for consciousness indicators derived from neuroscientific theories (IIT, GNWT, Higher-Order Thought, Predictive Processing). Applied to current LLMs: concludes they lack recurrent causal integration needed for consciousness.", "ai_systems");
    ing.try_concept("IIT vs feedforward architectures", "Mathematical result under IIT 3.0/4.0: feedforward architectures (including standard transformers with causal attention) necessarily have Φ=0. Only recurrent architectures with bidirectional causal dependencies can have Φ>0 (75% of RNNs tested).", "consciousness");

    // ====================================================================
    // CLUSTER 5: Self-Modifying Systems (Gödel Machine & successors)
    // ====================================================================
    println!("\n━━━ 5/8 自修改系统 — Gödel Machine 及后继 ━━━");
    ing.try_concept("Gödel Machine", "Jürgen Schmidhuber's (2003) self-referential universal problem solver: rewrites its own code only after proving the rewrite provably improves expected utility. Draws on Gödel's self-referential formulas (1931). First rigorous framework for recursive self-improvement with optimality guarantee.", "ai_systems");
    ing.try_concept("Darwin Gödel Machine (Sakana AI)", "Sakana AI's (2025) implementation of self-improving coding agents using population-based evolution. Maintains lineage of agents; each generation rewrites own source code; validated through SWE-bench benchmarks. Improved from 20% to 50% resolve rate.", "ai_systems");
    ing.try_concept("Huxley-Gödel Machine (KAUST)", "Wenyi Wang, Piotr Piękos, Jürgen Schmidhuber (2025): AI agent that evolves by rewriting its own code. Introduces Clade Metaproductivity (CMP) metric — evaluates collective output of all descendants rather than individual agent performance.", "ai_systems");
    ing.try_concept("Reflexion framework", "Princeton (Shinn et al 2023): language agents improve through verbal self-reflection without weight updates. After task attempt, agent generates natural-language critique stored in episodic memory. Subsequent attempts use past reflections for better decisions.", "ai_systems");
    ing.try_concept("Clade Metaproductivity (CMP)", "HGM metric measuring collective productiveness of an agent's entire descendant lineage. Addresses 'Metaproductivity–Performance Mismatch': short-term high performance does not predict long-term evolutionary potential. Low-performing agents may produce more evolvable descendants.", "ai_systems");

    // ====================================================================
    // CLUSTER 6: SSM Frontiers (Mamba series)
    // ====================================================================
    println!("\n━━━ 6/8 SSM 前沿 — Mamba 系列 ━━━");
    ing.try_concept("State Space Duality (SSD)", "Tri Dao and Albert Gu's (2024) framework proving that state space models and attention mechanisms are mathematically dual: both compute through structured semiseparable matrices. SSM = linear attention with state-space parameterization. Unifies the two dominant sequence model families.", "deep_learning");
    ing.try_concept("Selective State Space Model", "Mamba's core innovation (Gu, Dao 2023): SSM parameters (Δ, B, C) become input-dependent via linear projections. Enables content-aware filtering — model learns what to remember/forget based on token content. Overcomes LTI limitation in S4. O(n) vs Transformer O(n²).", "deep_learning");
    ing.try_concept("Mamba-2", "Second-gen Mamba (Dao, Gu 2024): simplified architecture using SSD framework. 2-8× faster training than Mamba-1 by leveraging matrix multiplication (tensor cores) instead of parallel scan. Supports larger state dimension. Matches Transformers at small-medium scale.", "deep_learning");
    ing.try_concept("Mamba-3", "Third-gen Mamba (Lahoti, Li, et al 2026): inference-optimized SSM. Complex-valued state tracking, MIMO variant for accuracy without decoding slowdown. Outperforms Mamba-2, Gated DeltaNet, Llama-3.2-1B on prefill+decode latency at 1.5B scale.", "deep_learning");
    ing.try_concept("Hybrid Mamba-Transformer architecture", "Production models combining SSM and attention layers: IBM Granite 4.0 (9:1 Mamba-Transformer ratio), NVIDIA Nemotron 3 (million-token context). SSM handles long-range context efficiently; attention layers provide precise token-token retrieval.", "deep_learning");

    // ====================================================================
    // CLUSTER 7: Meta-Cognition in AI
    // ====================================================================
    println!("\n━━━ 7/8 AI 元认知 — 自我监控与自适应控制 ━━━");
    ing.try_concept("Nelson-Narens metacognition framework", "Foundational cognitive science model (Nelson & Narens 1990): two-level architecture — object-level (task execution) and meta-level (monitoring + control). Meta-level reads from object-level (monitoring) and writes to it (control). Maps directly to agent architectures.", "cognitive_science");
    ing.try_concept("MAPE-K loop", "IBM's autonomic computing pattern (2003): Monitor → Analyze → Plan → Execute over a Knowledge base. Most battle-tested adaptive control cycle. Translates directly into agent metacognition: monitor agent behavior, analyze performance, plan adjustments, execute changes.", "ai_systems");
    ing.try_concept("TRAP framework for metacognitive AI", "Wei et al (2024): Transparency (systems understand own operations), Reasoning (about internal processes), Adaptation (adjust strategies), Perception (of own state). Advocates neurosymbolic approach combining symbolic reasoning with neural perception.", "ai_systems");
    ing.try_concept("System 1 / System 2 (Kahneman)", "Daniel Kahneman's dual-process theory (2011): System 1 — fast, intuitive, automatic (pattern matching); System 2 — slow, deliberate, analytical (logical reasoning). Applied to AI: LLMs as System 1; chain-of-thought, deliberation as System 2 emulation.", "cognitive_science");
    ing.try_concept("Dual observation in metacognitive agents", "Architecture pattern (2026): MetaCognition module independently monitors both (1) the Governor's decision quality and (2) Session interaction quality. Consensus mechanism aggregates signals from independent observers for reliable control recommendations.", "ai_systems");
    ing.try_concept("Metacognitive AI (position paper)", "arXiv:2605.15567 (2026): argues for metacognition as general design principle for accurate, secure, efficient AI. Demonstrates via Federated Learning case study: metacognitive resource allocation improves learning efficiency and nt_shield. Open-source framework for metacognition-enabled AI.", "ai_systems");

    // ====================================================================
    // CLUSTER 8: Edge wiring
    // ====================================================================
    println!("\n━━━ 8/8 知识关联 — 跨集群边连接 ━━━");

    let cross_edges: &[(&str, &str, RelationType, f64, &str)] = &[
        // E8 → existing concepts
        ("E8 Lie algebra", "E8 Lie Group", RelationType::Related, 1.0, "E8 Lie algebra generates E8 Lie group"),
        ("E8 lattice", "E8 Lie algebra", RelationType::Related, 0.8, "E8 root system = E8 lattice"),
        ("Exceptional Lie algebras", "E8 Lie algebra", RelationType::Related, 1.0, "E8 ∈ exceptional Lie algebras"),
        ("Octonion algebra", "E8 Lie algebra", RelationType::Related, 0.8, "E8 bracket via oct-octonions"),
        ("Cartan-Killing classification", "Exceptional Lie algebras", RelationType::DependsOn, 1.0, "Classification defines the exceptions"),

        // VSA → existing
        ("Multiply-Add-Permute (MAP) architecture", "Hyperdimensional computing", RelationType::SubclassOf, 1.0, "MAP is an HDC/VSA implementation"),
        ("Fourier Holographic Reduced Representation (FHRR)", "Vector Symbolic Architecture", RelationType::SubclassOf, 1.0, "FHRR ∈ VSA family"),
        ("Binary Spatter Code (BSC)", "Hyperdimensional computing", RelationType::SubclassOf, 1.0, "BSC is Kanerva's HDC-VSA"),
        ("LARS-VSA", "Vector Symbolic Architecture", RelationType::ExtensionOf, 0.9, "LARS extends VSA with relational bottleneck"),

        // GWT/GNW → existing
        ("Baars' theater of consciousness", "Global Workspace Theory", RelationType::Related, 1.0, "Theater metaphor for GWT"),
        ("Dehaene's Global Neuronal Workspace (GNW)", "The Global Neuronal Workspace", RelationType::Related, 1.0, "GNW is the neurobiological GWT"),
        ("Contrastive analysis method", "Global Workspace Theory", RelationType::DependsOn, 1.0, "Methodology for testing GWT"),
        ("GWT blackboard architecture origin", "Global Workspace Theory", RelationType::DependsOn, 0.9, "Blackboard AI → GWT cognitive architecture"),
        ("AI implementations of Global Workspace", "Global Workspace Theory", RelationType::ExtensionOf, 0.9, "Computational GWT implementations"),

        // IIT → existing
        ("Integrated Information Theory (IIT)", "Integrated Information Theory", RelationType::Related, 1.0, "IIT full theory"),
        ("Φ (phi) — integrated information", "Integrated Information Theory", RelationType::DependsOn, 1.0, "Φ is IIT's central quantity"),
        ("Perturbational Complexity Index (PCI)", "Consciousness", RelationType::Related, 0.8, "PCI measures consciousness clinically"),

        // Gödel Machine → existing
        ("Gödel Machine", "Recursive self-improvement", RelationType::Related, 1.0, "Gödel Machine formalizes RSI"),
        ("Gödel Machine", "Self-evolution", RelationType::Related, 0.9, "Gödel Machine = self-evolution with proof"),
        ("Darwin Gödel Machine (Sakana AI)", "Gödel Machine", RelationType::ExtensionOf, 0.9, "DGM implements Gödel Machine principles"),
        ("Reflexion framework", "Meta-learning (computer science)", RelationType::Related, 0.7, "Reflexion as meta-cognitive learning"),

        // SSM → existing
        ("Selective State Space Model", "Mamba State Space Model", RelationType::Related, 1.0, "Mamba is a selective SSM"),
        ("State Space Duality (SSD)", "Mamba State Space Model", RelationType::DependsOn, 0.9, "SSD framework powers Mamba-2"),
        ("Mamba-2", "Mamba State Space Model", RelationType::ExtensionOf, 1.0, "Mamba-2 extends Mamba"),
        ("Mamba-3", "Mamba State Space Model", RelationType::ExtensionOf, 1.0, "Mamba-3 extends Mamba-2"),

        // Metacognition → existing
        ("Nelson-Narens metacognition framework", "Self-evolution", RelationType::Related, 0.6, "Meta-level/object-level informs SEAL architecture"),
        ("System 1 / System 2 (Kahneman)", "Predictive processing", RelationType::Related, 0.5, "Dual process theory relates to predictive coding"),
    ];

    let e = ing.relate_many(cross_edges);
    ing.log(format!("Cross-cluster edges: {}", e));

    // Dedup
    let deduped = ing.dedup();
    if deduped > 0 {
        ing.log(format!("Dedup removed {} duplicate nodes", deduped));
    }

    ing.report("前沿知识种子 — 完成", &before, overall.elapsed());

    // Concept coverage summary
    let after = ing.stats();
    println!("\n  新增概念覆盖:");
    for (t, c) in &after.by_type {
        let b4 = before.by_type.iter().find(|(k, _)| k == t).map(|(_, v)| v).copied().unwrap_or(0i64);
        let d = if *c > b4 { format!(" +{}", c - b4) } else { String::new() };
        println!("    {:>28}: {:>4}{}", t, c, d);
    }
}
