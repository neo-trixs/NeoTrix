# Model Reverse Engineering — Cycle 334

**Date**: 2026-09-11
**Focus**: Efficient inference, attention mechanisms, agent coordination, memory architectures
**Previous cycles**: 318–333 (excluded)

---

## Paper 1: GLIDE — Guided Layerwise Hybrid Attention for Efficient LLM Inference

**URL**: https://arxiv.org/abs/2607.24788
**Authors**: Vimal William, Ravi Tandon, Jyotikrishna Dass
**Date**: 2026-06-26

### Core Idea
Layer-wise heterogeneity in transformers: early layers exhibit high sensitivity to softmax removal (they need full attention), while deeper layers tolerate aggressive replacement by linear alternatives. GLIDE introduces a layer-wise adaptive mechanism where each layer balances an efficient linear recurrence with a variable-sized softmax window. Non-uniformly compresses the softmax footprint across the model.

Key insight: **not all layers are equal** — the optimal attention mechanism varies by layer depth. Uniform hybrid approaches (same FA/SA ratio everywhere) leave performance on the table.

### Key Results
- Superior performance-efficiency tradeoffs vs uniform hybrid approaches
- Reduces end-to-end latency for long-context generation without quality loss
- Layer-wise heterogeneity as a design principle, not just an observation

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-CORE (GWT)** | Layer-wise attention allocation → domain-wise attention allocation; early layers (high sensitivity) map to foundation modules that need full context, deeper layers (tolerant) map to leaf modules that can operate on summaries |
| **NT-MEMORY** | Variable-sized context windows per module → dynamic experience compression based on module sensitivity |
| **NT-PHYSICAL** | Linear recurrence for constrained layers → lightweight processing for resource-constrained subsystems |

### NeoTrix Application
**Domain-wise attention budgeting**: GLIDE's core insight — layers have heterogeneous sensitivity — maps directly to NeoTrix's 7 domains. NT-CORE (E8, GWT) needs full context for reasoning. NT-ACT (tools) can operate on structured summaries. NT-MEMORY (KB) needs indexed access, not raw context. Instead of broadcasting equally to all domains (current GWT), allocate attention budget by domain sensitivity: high budget for NT-CORE, compressed for NT-ACT, indexed for NT-MEMORY.

**Layer-wise KV cache optimization**: GLIDE's variable softmax window per layer informs NeoTrix's KV cache strategy. Not all KV cache entries are equally valuable — early-layer KV pairs need high precision, deep-layer KV pairs can be aggressively compressed. This maps to our hot/cold tiering: critical reasoning KV pairs stay hot, routine processing KV pairs go cold.

---

## Paper 2: SparDA — Sparse Decoupled Attention for Efficient Long-Context LLM Inference

**URL**: https://arxiv.org/abs/2606.04511
**Authors**: Yaosheng Fu, Guangxuan Xiao, Xin Dong, Song Han, Oreste Villa
**Date**: 2026-06-03

### Core Idea
Introduces a fourth per-layer projection called **Forecast**, alongside Query, Key, and Value. Forecast predicts the KV blocks needed by the next layer, enabling **lookahead selection** that overlaps CPU-to-GPU prefetch with current-layer execution. Because Forecast is decoupled from the attention query, GQA implementation uses one Forecast head per GQA group.

Key innovation: **decouple selection from computation**. Current sparse attention selects KV blocks using the same query that computes attention — this creates a bottleneck. SparDA separates these: a lightweight Forecast projection predicts what's needed, while the actual attention runs on pre-fetched data.

Adds <0.5% parameters and trains only Forecast projections by matching original selector's attention distribution.

### Key Results
- Up to 1.25x prefill speedup and 1.7x decode speedup over sparse-attention offload baseline
- Up to 5.3x higher decode throughput via larger feasible batch sizes
- <0.5% parameter overhead
- Matches or slightly improves accuracy

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-CORE (GWT)** | Forecast projection → GWT salience prediction; lookahead selection → pre-compute which modules need attention before broadcast |
| **NT-WORLD** | Prefetch overlap → pre-fetch world data (crawl results, sensor feeds) before GWT decides to attend to it |
| **NT-MEMORY** | KV block prediction → experience branch prediction; load relevant KB branches before query arrives |
| **NT-IO** | Decoupled selection → decouple model routing decision from model execution; route while previous response is still processing |

### NeoTrix Application
**GWT with lookahead**: SparDA's Forecast projection maps to a "GWT Forecast" — predict which modules will be salient in the next cycle, begin pre-loading their context before the cycle starts. Currently GWT reacts to current salience. With lookahead, GWT proactively prepares: if current conversation trends toward code review, pre-load NT-SHIELD rev-officer context before the review trigger fires.

**Decoupled routing from execution**: SparDA's key architectural move — decouple the "what to attend to" decision from the "how to attend" computation. NeoTrix can apply this to provider routing: decide which model to use (routing decision) while the previous model is still generating (execution). The routing latency becomes zero because it overlaps with execution.

---

## Paper 3: Language Models Can Control Their Own Attention

**URL**: https://arxiv.org/abs/2609.02737
**Authors**: Tian Jin et al.
**Date**: 2026-09-02

### Core Idea
Models can be prompted to **declare** which KV cache blocks they will attend to for future tokens, using special tokens in their output. The inference engine parses these declarations like tool calls and skips most of the KV cache read. This is **declarative attention** — the model itself decides what to attend to, rather than an external system imposing sparsity.

Zero-shot evaluation across 15 long-context tasks: on off-the-shelf models (Gemma-4-31B, Qwen-3.6-27B), reduces total attended tokens during decoding (52.0%, 31.1%) with modest accuracy drops (1.27pp, 2.75pp) that shrink with model scale.

### Key Results
- 52% reduction in attended tokens on Gemma-4-31B, 31.1% on Qwen-3.6-27B
- Modest accuracy drops that shrink with model scale
- Works on off-the-shelf models — no training required
- New axis of sparse attention: model-declared, not externally imposed

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-CORE (GWT)** | Declarative attention → modules declare what information they need; GWT broadcasts only to self-selected receivers |
| **NT-MIND** | Self-declared attention → SEAL pipeline stages declare their information needs for next stage |
| **NT-ACT** | Tool-call-like declarations → agents declare which tools/capabilities they will use before execution |
| **NT-GOVERNANCE** | Declared attention as accountability — modules must justify what they attend to |

### NeoTrix Application
**Self-selecting GWT receivers**: The paper's core insight — let the model declare what it attends to — maps to letting NeoTrix modules declare their information needs. Currently GWT broadcasts to all modules (broadcast model). With declarative attention, each module outputs "I need X, Y, Z" for the next cycle, and GWT only routes those specific signals. This reduces broadcast overhead and increases signal-to-noise.

**Attention as accountability**: If modules must declare what they attend to, this creates an audit trail. NT-GOVERNANCE can verify: "Module M claimed to need information I, did it actually use I?" This maps to our rev-officer evidence-first principle — every action justified by declared intent.

---

## Paper 4: RAPS — Reputation-Aware Publish-Subscribe for Adaptive, Scalable, and Robust Coordination of LLM Agents

**URL**: https://arxiv.org/abs/2602.08009
**Authors**: Rui Li, Zeyu Zhang, Xiaohe Bo, Quanyu Dai, Chaozhuo Li, Feng Wen, Xu Chen
**Date**: 2026-02-08

### Core Idea
Frames multi-agent coordination as a dynamic ad-hoc networking problem: how to establish adaptive and reliable communication among scalable agentic hosts? RAPS uses Distributed Publish-Subscribe Protocol — agents exchange messages based on **declared intents** rather than predefined topologies.

Two coherent overlays:
1. **Reactive Subscription**: agents dynamically refine their intents as context evolves
2. **Bayesian Reputation**: each agent has a local watchdog to detect and isolate malicious peers

### Key Results
- Reconciles adaptivity, scalability, and robustness in unified framework
- Effective across five benchmarks
- Decentralized — no single point of coordination failure
- Reputation-based trust without central authority

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-CORE (GWT)** | Publish-subscribe → GWT as pub/sub bus; modules subscribe to relevant signal types |
| **NT-SHIELD** | Bayesian reputation → module trust scoring; malicious module detection and isolation |
| **NT-ACT** | Reactive subscription → dynamic capability subscription based on task demands |
| **NT-GOVERNANCE** | Reputation as governance — trust scores determine module influence on global decisions |
| **NT-MEMORY** | Intent-based messaging → experience retrieval based on declared intent, not keyword matching |

### NeoTrix Application
**GWT as pub/sub with reputation**: RAPS's core architecture maps directly to NeoTrix. Currently GWT is a broadcast bus — all modules receive all broadcasts. RAPS shows that a pub/sub model (modules subscribe to relevant signal types) with reputation scoring (trust-weighted message delivery) is more scalable and robust. Low-reputation modules get their signals attenuated, not blocked — a softer governance model than binary allow/deny.

**Dynamic intent refinement**: RAPS's Reactive Subscription — agents refine their intents as context evolves — maps to NeoTrix's dual specialization switching. When the task shifts from acquisition (CORE+WORLD) to evolution (CORE+MIND), modules should dynamically update their subscription patterns. This is more fluid than our current binary Weapon Set switching.

---

## Paper 5: U-Mem — Towards Autonomous Memory Agents

**URL**: https://arxiv.org/abs/2602.22406
**Authors**: Xinle Wu, Rui Zhang, Mustafa Anis Hussain, Yao Lu
**Date**: 2026-02-25

### Core Idea
Existing memory agents are passive and reactive — memory growth bounded by information that happens to be available. U-Mem proposes **autonomous memory agents** that actively acquire, validate, and curate knowledge at minimum cost.

Two key innovations:
1. **Cost-aware knowledge-extraction cascade**: escalates from cheap self/teacher signals → tool-verified research → expert feedback (only when needed). Most knowledge extracted cheaply; expensive verification reserved for high-value claims.
2. **Semantic-aware Thompson sampling**: balances exploration (acquiring new knowledge) and exploitation (using known knowledge) over memories, mitigating cold-start bias.

### Key Results
- Improves HotpotQA (Qwen2.5-7B) by 14.6 points
- Improves AIME25 (Gemini-2.5-flash) by 7.33 points
- Consistently beats prior memory baselines
- Can surpass RL-based optimization

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-MEMORY** | Cost-aware cascade → experience-tree absorption cost tiers; cheap self-signal for routine, expensive verification for novel claims |
| **NT-MIND** | Thompson sampling → SEAL pipeline exploration/exploitation balance; try new evolution strategies vs reuse proven ones |
| **NT-WORLD** | Active knowledge acquisition → NT-WORLD doesn't just crawl, it actively seeks knowledge to fill gaps |
| **NT-CORE** | Cold-start mitigation → new modules get exploration budget, not just exploitation of existing patterns |

### NeoTrix Application
**Cost-aware experience absorption**: U-Mem's cascade directly maps to experience-tree. Most experience can be absorbed cheaply (self-signal: "this worked" / "this failed"). High-value experience (novel patterns, contradicting prior knowledge) triggers expensive verification (tool-verified research, cross-domain consistency checks). This prevents the experience-tree from spending equal compute on trivial and important absorptions.

**Exploration/exploitation in evolution**: Thompson sampling for SEAL pipeline — should we try a new evolution strategy (exploration) or apply a proven one (exploitation)? U-Mem's semantic-aware approach means exploration is guided by knowledge gaps, not random. If NT-MEMORY detects "we have no experience with X," it triggers exploration. If "we have strong evidence for Y," it exploits.

---

## Cross-Paper Patterns

| Pattern | Papers | NeoTrix Mapping |
|---------|--------|-----------------|
| **Layer/module-wise heterogeneity** | GLIDE, SparDA | Not all domains/modules need equal attention; allocate by sensitivity |
| **Decouple selection from computation** | SparDA, Declarative Attention | Decide what to attend to separately from how to attend |
| **Self-declared intent** | Declarative Attention, RAPS | Modules declare their information needs; GWT routes by declaration |
| **Cost-aware resource allocation** | U-Mem, SparDA | Cheap operations for routine, expensive for novel; cascade by value |
| **Reputation-based trust** | RAPS | Module trust scores weight their influence on global decisions |
| **Active knowledge acquisition** | U-Mem, Declarative Attention | Don't wait for information — seek it proactively based on gaps |

---

## Action Items for NeoTrix

| Priority | Action | Source Paper | Target Domain |
|----------|--------|-------------|---------------|
| P0 | Domain-wise attention budgeting (heterogeneous GWT broadcast) | GLIDE | NT-CORE |
| P0 | Cost-aware experience absorption cascade | U-Mem | NT-MEMORY |
| P1 | GWT Forecast (lookahead module salience prediction) | SparDA | NT-CORE |
| P1 | Self-selecting GWT receivers (module-declared intent) | Declarative Attention | NT-CORE |
| P1 | GWT as pub/sub with reputation scoring | RAPS | NT-CORE + NT-SHIELD |
| P2 | Decouple routing decision from execution (zero-latency routing) | SparDA | NT-IO |
| P2 | Thompson sampling for SEAL exploration/exploitation | U-Mem | NT-MIND |
| P2 | Reactive subscription for dynamic capability switching | RAPS | NT-ACT |
| P3 | Attention as accountability (declared intent audit trail) | Declarative Attention | NT-GOVERNANCE |
| P3 | Bayesian reputation for module trust without central authority | RAPS | NT-SHIELD |
