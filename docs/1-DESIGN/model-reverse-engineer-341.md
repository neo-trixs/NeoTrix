# Model Reverse Engineering — Cycle 341

**Date**: 2026-09-11
**Focus**: Efficient inference, attention mechanisms, agent coordination

## 5 New AI Models/Papers

---

### 1. State-Transition Framework for Efficient LLM Reasoning
- **Paper**: "A State-Transition Framework for Efficient LLM Reasoning" (ICLR 2026)
- **Authors**: Liang Zhang, Yu Zhao, Longyue Wang et al.
- **URL**: https://arxiv.org/abs/2602.01198
- **Category**: Efficient Reasoning via Linear Attention

**Core Mechanism**:
- Models LLM reasoning as a **state-transition process** — each reasoning step compresses historical information into a **reasoning state matrix** via linear attention
- **Mixed Attention Module (MAM)**: replaces standard attention with linear attention for historical reasoning information, quadratic → linear complexity
- Each token directly retrieves relevant historical reasoning from the state matrix without attending to all previous tokens
- **State-based reasoning strategy**: mitigates over-thinking on noisy reasoning steps

**Key Results**:
- Improved reasoning efficiency AND performance across multiple datasets/model sizes
- Reasoning state matrix acts as compressed working memory for the reasoning process
- No fine-tuning of base LLM required for efficiency gains

**NeoTrix Mapping**:
- **NT-CORE (E8)**: Reasoning state matrix = compressed attention state across reasoning steps. Maps to E8 hexagram state transitions — each step is a hexagram transition that compresses history
- **NT-MEMORY**: Reasoning state matrix = hierarchical memory (local→session→long-term) compressed into a single representation
- **A2 Axiom (Context as Scarce Resource)**: Directly addresses context window limitation by compressing reasoning history into fixed-size state matrix

**Absorption Pattern**: State-transition reasoning — compress historical reasoning into a fixed-size matrix that can be attended to in linear time. Applicable to ConsciousnessTree cycle state management.

---

### 2. DiffAdapt: Difficulty-Adaptive Reasoning for Token-Efficient LLM Inference
- **Paper**: "DiffAdapt: Difficulty-Adaptive Reasoning for Token-Efficient LLM Inference" (ICLR 2026)
- **Authors**: (Multiple authors, ICLR 2026)
- **URL**: https://arxiv.org/pdf/2510.19669
- **Category**: Adaptive Inference Strategy

**Core Mechanism**:
- **U-shaped entropy discovery**: Easy problems → high entropy (overthinking), Medium → low entropy, Hard → high entropy (uncertainty). Counter-intuitive: easy problems have high entropy despite high accuracy
- **Three-stage framework**: Data preparation → Probe training → Adaptive strategy execution
- **Difficulty classifier**: Small probe network classifies LLM's final hidden state into Easy/Normal/Hard difficulty
- **Per-difficulty strategies**: Fixed prompt + temperature + max token length per difficulty tier
  - Easy: succinct prompt, low temperature, short max tokens
  - Hard: careful thinking prompt, higher temperature, longer budget

**Key Results**:
- Minimal latency overhead (small classifier only)
- No fine-tuning of base LLM — only trains a lightweight probe
- Significant token savings on easy/medium problems while maintaining accuracy on hard ones

**NeoTrix Mapping**:
- **GWT (Attention Routing)**: Difficulty classification = salience-based routing to different inference strategies. Maps to GWT broadcasting different attention patterns based on task difficulty
- **A1 Axiom (Cost-Aware Routing)**: Direct implementation — route easy tasks to cheap strategies, hard tasks to expensive ones
- **NT-MIND (Self-Evolution)**: Difficulty classifier learns to predict when the model is overthinking — meta-cognitive awareness of reasoning depth

**Absorption Pattern**: Difficulty-adaptive inference — classify input difficulty at runtime, route to tiered inference strategies (prompt + temperature + token budget). Applicable to NeoTrix's cost-aware model routing.

---

### 3. SABER: Switchable and Balanced Training for Efficient LLM Reasoning
- **Paper**: "SABER: Switchable and Balanced Training for Efficient LLM Reasoning" (AAAI 2026)
- **Authors**: Kai Zhao, Yanjun Zhao et al. (Bilibili Inc.)
- **URL**: https://ojs.aaai.org/index.php/AAAI/article/view/40799
- **Category**: Token-Budgeted Reasoning with RL

**Core Mechanism**:
- **Four discrete inference modes**: NoThink → FastThink → CoreThink → DeepThink (user-controllable)
- **RL framework**: Profiles each training example's base-model thinking token usage, assigns to budget tiers
- **Length-aware rewards**: During fine-tuning, guided by system prompts and rewards to respect assigned budget
- **No-think examples**: Ensures model remains reliable even when explicit reasoning is turned off
- **Budget profiling**: Pre-assigns each training example to a budget tier based on complexity

**Key Results**:
- SABER-FastThink: -65.4% reasoning length, +3.6% accuracy on MATH (vs base model)
- Graceful degradation across budget tiers
- Cross-scale and cross-domain generalization

**NeoTrix Mapping**:
- **A1 + A2 Axioms**: Four-tier inference budget = cost-aware routing with context budget management
- **GWT**: Mode selection (NoThink→DeepThink) = salience-driven attention depth modulation
- **NT-MIND (SEAL)**: Budget profiling during training = experience-tree categorizing tasks by required reasoning depth

**Absorption Pattern**: Four-mode switchable reasoning — NoThink/FastThink/CoreThink/DeepThink with user-controllable budget. Applicable to NeoTrix's dual specialization weapon sets — different modes for different task complexities.

---

### 4. MKA: Memory-Keyed Attention for Efficient Long-Context Reasoning
- **Paper**: "MKA: Memory-Keyed Attention for Efficient Long-Context Reasoning" (ACM Computing Frontiers 2026, Oral)
- **Authors**: Dong Liu, Yanxuan Yu, Ben Lengerich, Ying Nian Wu
- **URL**: https://arxiv.org/abs/2603.20586
- **Category**: Hierarchical Attention with Multi-Level Memory

**Core Mechanism**:
- **Three-tier memory hierarchy**: Local (L1), Session (L2), Long-term (L3)
- **Dynamic routing gates**: Lightweight routing mechanism learns to route each query token across memory levels
- **Route-Fused MKA (FastMKA)**: Fuses local/session/long-term memory via learned routing weights before a single key-value projection
- **Fused KV caching**: Caches routed keys/values instead of raw token KV — reduces memory bandwidth
- Single attention computation over fused memory (vs multiple attention paths)

**Key Results**:
- Comparable perplexity to Multi-Latent Attention (MLA)
- 5x faster training throughput than MLA
- 1.8x lower evaluation latency than MLA
- Hardware-friendly implementation

**NeoTrix Mapping**:
- **NT-MEMORY**: Three-tier memory = KB local cache / session context / persistent knowledge base. Direct mapping to experience-tree hierarchy (session snapshot → distilled experience → KB persistent)
- **GWT**: Routing gates = attention routing across memory tiers. Maps to GWT's salience-based broadcasting to different memory subsystems
- **A2 Axiom**: Memory hierarchy addresses context scarcity by tiering hot/cold/warm memory

**Absorption Pattern**: Hierarchical memory-keyed attention — three-tier memory (local/session/long-term) with learned routing. Fuse before attention to reduce computation. Applicable to NeoTrix's memory architecture — route queries across KB tiers.

---

### 5. ATLAS: Verifier-Guided Adaptive Latent Activation Steering
- **Paper**: "ATLAS: Verifier-Guided Adaptive Latent Activation Steering for Efficient LLM Reasoning" (arXiv 2026, v4)
- **Authors**: Tuc Nguyen, Thai Le (Indiana University)
- **URL**: https://arxiv.org/abs/2601.03093
- **Category**: Latent Space Reasoning Control

**Core Mechanism**:
- **Offline phase**: Segments reasoning traces into thought units, extracts hidden states at boundaries, constructs contrastive steering vectors for execution/reflection/transition modes
- **Lightweight verifier**: Distills Process Reward Model (PRM) step-quality scores into a compact neural network operating on hidden states
- **Per-example per-step adaptation**: Verifier predicts quality of ongoing reasoning from intermediate hidden states, dynamically selects steering action
- **Three steering modes**: Execution (push forward), Reflection (step back), Transition (change direction)
- **No additional LLM decoding**: Control happens in latent space, not text space

**Key Results**:
- Higher accuracy than vanilla decoding AND fixed steering baselines
- Substantially reduced test-time token usage
- Lightweight verifier adds minimal overhead
- Works across mathematical reasoning and code-generation benchmarks

**NeoTrix Mapping**:
- **NT-CORE (E8)**: Steering vectors = hexagram state transitions (execution/reflection/transition maps to specific E8 transitions). Latent verification = internal state quality assessment
- **ConsciousnessTree**: Three steering modes = So→Roots→Trunk→Branches→Fruits→Core cycle phases. Verifier = meta-cognitive quality check at phase boundaries
- **NT-MIND (SEAL)**: PRM distillation = experience-tree distillation of process quality into compact verifier

**Absorption Pattern**: Verifier-guided latent steering — lightweight verifier over hidden states selects steering direction (execute/reflect/transition) per reasoning step. Applicable to ConsciousnessTree meta-cognition — verify reasoning quality at cycle boundaries and steer next phase.

---

## Cross-Cutting Themes (Cycle 341)

| Theme | Papers | NeoTrix Impact |
|-------|--------|----------------|
| **Reasoning as State Machine** | State-Transition, SABER | Model reasoning as state transitions with compressible history — maps to E8 hexagram state machine |
| **Difficulty-Aware Routing** | DiffAdapt, SABER | Classify input difficulty, route to tiered strategies — validates A1 cost-aware routing axiom |
| **Hierarchical Memory Attention** | MKA | Three-tier memory with learned routing — maps to experience-tree hot/cold/warm tiers |
| **Latent Space Control** | ATLAS, State-Transition | Control reasoning in hidden state space, not text space — reduces overhead 10x |
| **Lightweight Verifiers** | ATLAS, DiffAdapt | Small classifiers predict reasoning quality from hidden states — meta-cognitive monitoring |

## NeoTrix Integration Opportunities

| Priority | Pattern | Source | Integration Target |
|----------|---------|--------|-------------------|
| P0 | Three-tier memory with routing gates | MKA | NT-MEMORY — hierarchical memory architecture with learned routing across local/session/persistent tiers |
| P0 | Difficulty-adaptive inference routing | DiffAdapt | GWT — classify task difficulty, route to cost-appropriate inference strategy |
| P1 | Reasoning state compression matrix | State-Transition | NT-CORE — compress ConsciousnessTree cycle history into fixed-size state matrix |
| P1 | Four-mode switchable reasoning | SABER | GWT — NoThink/FastThink/CoreThink/DeepThink modes for different task complexities |
| P1 | Verifier-guided latent steering | ATLAS | ConsciousnessTree — lightweight verifier at phase boundaries steers meta-cognition |
| P2 | Budget-tier profiling during training | SABER | NT-MIND — profile tasks by required reasoning depth for experience-tree categorization |
| P2 | Contrastive steering vectors | ATLAS | NT-CORE — execution/reflection/transition vectors for E8 state transitions |
