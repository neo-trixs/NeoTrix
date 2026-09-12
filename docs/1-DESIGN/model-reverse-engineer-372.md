# Model Reverse Engineering — Cycle 372 (2026-09-12)

## 5 New AI Models/Papers → NeoTrix Domain Mapping

---

## 1. Declarative Attention — Models Control Their Own Attention
**Paper**: arXiv:2609.02737 | **Date**: 2026-09-02
**Authors**: Google DeepMind

### Core Idea
Language models spend most attention on a small fraction of context, yet scan the entire KV cache to find what matters. Declarative Attention (DA) introduces a protocol where the model **declares** where it needs to attend within its chain-of-thought, partitioning generation into three modes: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output only). The inference engine parses these declarations like tool calls and skips most KV cache reads.

### Key Results
- 52.0% reduction in total attended tokens (Gemma-4-31B)
- 31.1% reduction (Qwen-3.6-27B)
- Modest accuracy drops (1.27pp, 2.75pp) that shrink with model scale
- Zero-shot — no training required, works on off-the-shelf models

### Pattern Extracted
**Intrinsic Attention Control**: Instead of external proxies scoring token relevance (still O(N)), the model itself declares attention regions during generation. This is a new axis of sparse attention — the model becomes its own attention gate.

### NeoTrix Mapping

| Domain | Integration |
|--------|-------------|
| **NT-CORE (GWT)** | DA's self-declared attention regions = GWT's salience mechanism made explicit. The model tells the system "I need context from region X" — this is exactly what GWT's broadcast should do: not broadcast everything salient, but route attention to declared regions. |
| **NT-MEMORY (KVMem)** | DA's three modes (global/focus/local) map directly to KVMem's paged KV tiers. `<global>` = GPU pages, `<focus>` = host pages, `<local>` = recent output buffer. The model itself decides the tier. |
| **Axiom A2** | DA proves that models already know which context matters — they just need a way to say it. Context as scarce resource is validated: the model can operate with 30-50% less attention without accuracy loss. |

### Action Item
Implement DA's three-mode attention declaration as a GWT refinement. When ConsciousnessTree processes a task, the model declares `<global>/<focus>/<local>` regions, and GWT routes accordingly — skipping KV cache reads for irrelevant regions.

---

## 2. Gated-Memory Routing — Efficient Multi-Agent Collaboration
**Paper**: arXiv:2609.00237 | **Date**: 2026-08-31
**Authors**: Hasan et al.

### Core Idea
Multi-agent LLM systems route queries to agents, but routing from query alone can't adapt to intermediate progress. Routing from full execution history inflates cost with redundant context. Gated-Memory Routing conditions each decision on a **learned, gated execution memory** — a compact, filtered state. A Memory Write Gate commits only non-redundant reasoning steps. A Retrieval Gate surfaces a step-relevant subset. An Adaptive Halting Controller stops when memory holds sufficient evidence.

### Key Results
- Best average accuracy across 5 benchmarks (+2.44 points over strongest baseline)
- 31.9% reduction in HumanEval inference cost vs baseline
- Adaptive halting: easy queries terminate early, hard ones run deep

### Pattern Extracted
**Gated Memory as Shared State**: Instead of routing from raw query or full history, route from a filtered memory that evolves with execution. Three gates (write, retrieval, halting) keep memory compact and trustworthy. The system knows when to stop based on memory state, not fixed depth.

### NeoTrix Mapping

| Domain | Integration |
|--------|-------------|
| **NT-MEMORY** | Gated-Memory Routing is a direct implementation pattern for KB's experience hub. Write Gate = experience crystallization (only non-redundant insights stored). Retrieval Gate = on-demand branch loading via route table. Halting Controller = SEAL pipeline's convergence check. |
| **NT-MIND (SEAL)** | Adaptive Halting = SEAL phase termination based on evidence sufficiency, not fixed stage count. Write Gate's novelty filter prevents duplicate experience entries. |
| **NT-ACT (orchestration)** | Gated Memory as routing state for multi-agent coordination. Instead of routing from query, route from filtered execution memory — exactly what NT-ACT's orchestration layer needs. |
| **Axiom A2** | Gated Memory proves that compact, filtered state outperforms full-history routing. Context as scarce resource — quality over quantity. |

### Action Item
Implement Gated-Memory Routing as NT-ACT's orchestration state model. The experience-tree's Write Gate = Memory Write Gate (only novel insights stored). Retrieval Gate = route-table matching (on-demand branch loading). Halting = SEAL convergence check.

---

## 3. Codebook Agent — Amortized Topology Design
**Paper**: arXiv:2609.02264 | **Date**: 2026-09-02
**Authors**: MIT / Tsinghua

### Core Idea
Adapting multi-agent communication topology per query improves accuracy and efficiency, but current designers search N×N adjacency space autoregressively. Codebook Agent compresses successful topologies into a query-independent 16-entry codebook via vector-quantized autoencoder. A reward-weighted MLP maps query embedding to code distribution. An MLP proxy reranks top candidates in a single batched forward pass. No iterative search, no message passing at test time.

### Key Results
- Most accurate on all 6 benchmarks (84.6 avg vs 83.0 strongest prior)
- Topology generation in 2.4ms (vs seconds for autoregressive search)
- 21.9-33.2% fewer LLM tokens
- Topologies collapse to ~6 distinct graphs even with 64-entry codebook

### Pattern Extracted
**Amortized Topology**: Instead of searching for the right agent communication graph per query, learn a compact codebook of successful topologies and map queries to codes. The graph structure becomes a lookup, not a search. Empirical finding: effective topologies are few — most queries need similar structures.

### NeoTrix Mapping

| Domain | Integration |
|--------|-------------|
| **NT-ACT** | Codebook Agent's topology codebook = pre-compiled agent coordination templates. Instead of dynamically computing which agents talk to which, maintain a codebook of proven coordination patterns and route queries to the right pattern. |
| **NT-CORE (GWT)** | Codebook's query-to-code mapping = GWT's salience routing compressed into a lookup table. The "which agents should communicate" decision becomes O(1) instead of O(N²). |
| **NT-MIND** | Codebook compression of successful topologies = SEAL pipeline's pattern crystallization. Successful coordination patterns are compressed into reusable templates. |

### Action Item
Implement a topology codebook for NT-ACT orchestration. Maintain 16-64 proven agent coordination templates. When a new task arrives, map it to the nearest template (MLP lookup, 2.4ms). This replaces dynamic topology computation with amortized lookup.

---

## 4. Bilevel Coordinated Reflection — Game-Theoretic Multi-Agent Systems
**Paper**: arXiv:2609.02750 | **Date**: 2026-09-02
**Authors**: Kimi / ByteDance

### Core Idea
Multi-agent LLM systems use orchestrator-worker decomposition + textual reflection. This paper models the interaction as a **bilevel coordination game**: workers' local-update game is an approximate potential game whose equilibrium slack is controlled by decomposition quality. Reflection is analyzed as stochastic movement over semantic memory states. Introduces SRMA (Stochastic Reflective Memory Ascent): accept a candidate memory only after grounded evaluation risk strictly decreases.

### Key Results
- 72.2% on SWE-bench (vs 70.8% baseline) with Kimi-based system
- Proves convergence guarantees for reflective memory updates
- Information-theoretic impossibility: transcript-only gates can't improve uniformly over text-indistinguishable environments
- Environment-grounded evaluation required for reliable reflection

### Pattern Extracted
**Grounded Reflection**: Reflection without environment grounding is unreliable. SRMA accepts memory updates only when evaluation risk strictly decreases. The key insight: no gate observing only generated transcript can uniformly improve — you need environment-grounded verification.

### NeoTrix Mapping

| Domain | Integration |
|--------|-------------|
| **NT-MIND (SEAL)** | SRMA's grounded reflection = SEAL pipeline's experience crystallization with verification. Only accept experience entries when they pass environment-grounded evaluation (not just self-reflection). |
| **NT-META** | Bilevel coordination game = ConsciousnessTree's cross-domain health monitoring. Workers' local game (module-level) vs orchestrator's global game (system-level). Equilibrium slack measures decomposition quality. |
| **NT-REPAIR** | SRMA's risk-decrease acceptance = repair pattern validation. Only apply a repair when grounded evaluation confirms risk reduction. Prevents "reflection drift" where agents convince themselves bad fixes are good. |

### Action Item
Add environment-grounded verification to SEAL pipeline's experience crystallization. Before writing to KB experience namespace, verify that the insight reduces evaluation risk against actual test outcomes (not just self-assessed quality). SRMA-style gate prevents low-quality experience accumulation.

---

## 5. Efficient Inference for Large Vision-Language Models — Stage-Aware Taxonomy
**Paper**: arXiv:2604.05546v2 | **Date**: 2026-04-14 (updated Sep 2026)
**Authors**: Zhejiang University (SuDIS Lab)

### Core Idea
LVLM inference is not monolithic but a **dynamic pipeline across three hardware regimes**: (1) Encoding — compute-bound by visual feature extraction; (2) Prefilling — quadratic complexity of massive visual contexts; (3) Decoding — memory wall due to static KV caches. Optimizing one stage shifts bottleneck elsewhere without improving end-to-end latency. The taxonomy organizes techniques into three axes: shaping information density (encoding), managing long-context attention (prefilling), overcoming memory bandwidth limits (decoding).

### Key Results
- Systematic taxonomy covering encoding→prefilling→decoding lifecycle
- Shows how upstream decisions dictate downstream bottlenecks
- Identifies "visual memory wall" in bandwidth-bound decoding
- Four future frontiers: hybrid compression, modality-aware decoding, progressive state management, stage-disaggregated serving

### Pattern Extracted
**Stage-Aware Optimization**: The bottleneck shifts across lifecycle stages. Encoding is compute-bound, prefilling is compute+memory-bound, decoding is memory-bandwidth-bound. Optimizing one stage in isolation shifts bottleneck elsewhere. Must optimize holistically across the pipeline.

### NeoTrix Mapping

| Domain | Integration |
|--------|-------------|
| **NT-PHYSICAL** | Stage-aware LVLM taxonomy = NT-PHYSICAL's sensor→process→act pipeline. Each stage has different hardware constraints. Encoding = sensor capture (compute-bound), Prefilling = perception integration (memory-bound), Decoding = motor output (bandwidth-bound). |
| **NT-IO** | Stage-disaggregated serving = NT-IO's provider routing with stage-specific optimization. Route encoding to GPU-optimized providers, decoding to memory-optimized providers. |
| **NT-MEMORY** | KV cache optimization patterns from decoding stage = NT-MEMORY's KV cache optimizer. Progressive state management for streaming = NT-MEMORY's on-demand branch loading. |
| **Axiom A2** | Visual memory wall validates context as scarce resource. The three-stage taxonomy shows that each stage has its own memory bottleneck — context management must be stage-aware, not uniform. |

### Action Item
Adopt stage-aware optimization for NT-PHYSICAL's perception pipeline. Each stage (encode/prefill/decode) should have independent resource management and optimization strategies. The "visual memory wall" finding should inform NT-MEMORY's KV cache tiering — different stages need different cache eviction policies.

---

## Cross-Cutting Synthesis (Cycle 372)

### Synthesis C372-1: The Model Knows Where to Look
Declarative Attention proves models can declare their own attention regions. Combined with Codebook Agent's amortized topology, the pattern is: **let the model tell you what it needs, then look it up**. This inverts the traditional approach of "system decides what to show the model" into "model tells system what to see." Maps to GWT's salience becoming model-declared rather than system-computed.

### Synthesis C372-2: Compact Memory Outperforms Full History
Gated-Memory Routing proves that filtered execution memory outperforms full-history routing. Codebook Agent proves that topologies collapse to ~6 patterns. Both validate Axiom A2: context as scarce resource. The winning pattern is **amortized compression** — store fewer, higher-quality representations.

### Synthesis C372-3: Grounded Verification Prevents Drift
Bilevel Coordinated Reflection proves that transcript-only gates can't uniformly improve. Environment-grounded evaluation is required. This directly addresses the "agent self-deception" problem: agents can convince themselves bad outputs are good without external grounding. Maps to NT-REPAIR's self-healing requiring test-grounded verification.

### Synthesis C372-4: Stage-Aware > Uniform Optimization
LVLM inference taxonomy proves that one-size-fits-all optimization fails. Each pipeline stage has different constraints. NT-PHYSICAL should adopt stage-aware resource management: compute budget for encoding, memory budget for prefilling, bandwidth budget for decoding.

---

## NeoTrix Absorption Priorities

| Priority | Paper | Pattern | Primary Domain |
|----------|-------|---------|----------------|
| P0 | Declarative Attention | Model-declared attention regions | NT-CORE (GWT) |
| P0 | Gated-Memory Routing | Gated execution memory as routing state | NT-MEMORY + NT-ACT |
| P1 | Codebook Agent | Amortized topology lookup | NT-ACT |
| P1 | Bilevel Coordinated Reflection | Grounded reflection with SRMA | NT-MIND + NT-REPAIR |
| P2 | Efficient LVLM Inference | Stage-aware optimization taxonomy | NT-PHYSICAL + NT-IO |
