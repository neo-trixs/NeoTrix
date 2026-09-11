# Model Reverse Engineering — Cycle 336 (2026-09-11)

## Summary

5 new AI models/papers reverse-engineered for pattern extraction and NeoTrix domain mapping. Focus: attention self-control, multi-agent topology design, setwise policy optimization, error-bounded attention routing, parallel-then-reason architecture.

---

## Paper 1: Declarative Attention — LMs Control Their Own Attention

**Source**: arXiv:2609.02737 (Sept 2, 2026)
**Authors**: (Not specified in source)

### Core Innovation
Declarative Attention (DA) introduces a protocol where the language model declares where it needs to attend within its chain-of-thought, partitioning generation into three modes:
1. **`<global>`** — full context scan
2. **`<focus>`** — specific region of KV cache
3. **`<local>`** — recent output only

The inference engine parses these declarations like tool calls and skips most KV cache reads. Zero-shot on off-the-shelf models (Gemma-4-31B, Qwen-3.6-27B).

### Key Results
- **52.0% reduction** in total attended tokens (Gemma-4-31B)
- **31.1% reduction** (Qwen-3.6-27B)
- Modest accuracy drops (1.27pp, 2.75pp) that shrink with model scale
- Works without any training or fine-tuning

### Pattern Extraction
| Pattern | Mechanism | NeoTrix Mapping |
|---------|-----------|-----------------|
| Intrinsic attention declaration | Model declares attention regions in CoT | GWT salience broadcasting from specialist modules |
| Three-mode partitioning | global/focus/local as natural language tokens | GWT broadcast/specialist/focused routing levels |
| KV skip via declaration | Inference engine skips KV reads based on declarations | Cost-aware routing at attention level (A1) |
| Zero-shot applicability | Works on off-the-shelf models | Runtime adaptation without training |

### NeoTrix Domain Mapping
- **NT-CORE**: DA's attention declaration is the model-level analog of our GWT's salience broadcasting. The three modes (global/focus/local) map directly to our GWT's broadcast-to-all/specialist/focused routing levels.
- **NT-IO**: Token reduction validates cost-aware routing (A1) — the model itself can tell us which context is worth reading.
- **NT-MEMORY**: Focus mode suggests selective experience retrieval — only load relevant branches from experience-tree.

### Absorption Verdict
**ABSORB (HIGH PRIORITY)** — DA proves that attention routing should be intrinsic, not extrinsic. This is the strongest validation of GWT's design philosophy. The three-mode protocol should become our GWT attention vocabulary. Zero-shot capability means we can adopt this pattern immediately without retraining.

---

## Paper 2: ReActNet — Inference-Time Graph Engineering for Multi-Agent Workflows

**Source**: arXiv:2609.05774 (Sept 4, 2026)
**Authors**: (Not specified in source)

### Core Innovation
ReActNet compiles a query and role-specialized agents into a sequence of directed communication graphs (temporal workflow):
1. **Graph compilation** — query-conditioned; each snapshot = one reasoning stage; each edge = natural-language instruction
2. **Structured message passing** — agents integrate previous states with messages from controller-assigned neighbors
3. **Final aggregator** — synthesizes resulting states into answer
4. **Training-free** — no RL or gradient-based topology optimization

### Key Results
- Consistently improves over fixed-topology and learned-topology baselines
- Competitive inference cost
- Works across knowledge reasoning, math, code generation, and GAIA assistant tasks

### Pattern Extraction
| Pattern | Mechanism | NeoTrix Mapping |
|---------|-----------|-----------------|
| Temporal graph compilation | Separate compilation from execution | SEAL pipeline stage → GWT broadcast |
| Edge-as-instruction | Natural language on each communication edge | EventBus message semantics |
| Task-conditioned topology | Graph structure adapts to query | GWT salience adapts to task type |
| Aggregator synthesis | Final state aggregation into answer | ConsciousnessTree fruit/core stages |

### NeoTrix Domain Mapping
- **NT-CORE**: Temporal graph = GWT attention routing with stage-aware broadcast. "When, why, and how information should flow" is exactly GWT's design brief.
- **NT-ACT**: Graph compilation = SEAL pipeline's planning phase. Edge instructions = tool call specifications.
- **NT-MEMORY**: Message passing = cross-module knowledge transfer via EventBus.

### Absorption Verdict
**ABSORB** — ReActNet formalizes what GWT does intuitively. Their key insight — "effective orchestration depends not only on which agents communicate, but on engineering executable workflow graphs that encode when, why, and how information should flow" — should become a NeoTrix axiom. The compilation/execution separation validates our SEAL pipeline design.

---

## Paper 3: SRPO — Setwise Relative Policy Optimization for Multi-Agent LLMs

**Source**: arXiv:2609.08452 (Sept 8, 2026)
**Authors**: (Not specified in source)

### Core Innovation
SRPO treats the "active set" (minimal set of outputs consumed by one state transition) as one multi-agent action:
1. **Set ratio** — member log-ratios combined into cardinality-normalized set ratio
2. **Single relative advantage** — one advantage assigned to the entire set
3. **Single clip** — set-level clipping
4. **Unifies division of labor and joint co-evolution** — actions with different set sizes

### Key Results
- Training interface works for fixed, mixed, and dynamically routed workflows across four model scales
- Strongest macro-average results among reported comparisons
- Stable under different event reductions and set sizes

### Pattern Extraction
| Pattern | Mechanism | NeoTrix Mapping |
|---------|-----------|-----------------|
| Setwise action | Active set as one atomic action | EventBus batch message as atomic unit |
| Cardinality normalization | Normalize by set size | Resource budget per task type |
| Unified co-evolution | Different set sizes in same framework | GWT routing across module types |
| Dynamic routing | Workflow topology changes at runtime | ConsciousnessTree cycle-adaptive routing |

### NeoTrix Domain Mapping
- **NT-CORE**: SRPO's setwise action is our EventBus's batch dispatch. Cardinality normalization maps to our ResourceBudgetManager's per-task allocation.
- **NT-MIND**: Unified co-evolution = SEAL pipeline's cross-domain evolution. Different set sizes = different module complexity levels evolving together.
- **NT-ACT**: Dynamic routing = our task scheduler adapting workflow topology per task.

### Absorption Verdict
**ABSORB** — SRPO's setwise formulation is the mathematically rigorous version of our EventBus batch dispatch. The key insight: when multiple agents jointly cause one state transition, optimize them as one action, not separately. This should inform our GWT's multi-specialist broadcast optimization.

---

## Paper 4: CEDAR — Error-Bounded Residual Routing for Efficient Long-Context Attention

**Source**: arXiv:2609.07237 (Sept 7, 2026)
**Authors**: (Not specified in source)

### Core Innovation
CEDAR is a coarse-to-fine attention routing method:
1. **Residual summaries** — each semantic chunk contributes a cheap key-value summary to a residual attention path
2. **Error-aware expansion** — chunks with high estimated approximation error are expanded to exact token attention
3. **Single softmax normalization** — exact and summarized contributions combined; refinement replaces, not duplicates, coarse evidence
4. **Output-error bound** — governed by within-chunk key/value dispersion

### Key Results
- Residual summaries reduce reconstruction error by **98%** relative to hard dropping
- **3× kernel speedup** at 128K context
- Recovers most quality lost by hard sparse routing

### Pattern Extraction
| Pattern | Mechanism | NeoTrix Mapping |
|---------|-----------|-----------------|
| Residual attention | Cheap summaries + exact expansion | GWT cheap-model broadcast + expensive-model specialist |
| Error-aware budget | Variable refinement per chunk | Cost-aware routing (A1) per task complexity |
| Non-duplicative refinement | Single softmax, not additive | Single-fact-source KB design |
| Dispersion-based error bound | Theoretical guarantee on quality | Module health scoring with formal bounds |

### NeoTrix Domain Mapping
- **NT-CORE**: CEDAR is the attention-level analog of GWT routing. Residual summaries = cheap model broadcasts; exact expansion = expensive model specialist calls. Error bounds provide formal guarantees our GWT lacks.
- **NT-IO**: Error-aware budget = our escalation routing (Switchyard, cycle 335). Dispersion-based error = our module health scoring.
- **NT-MEMORY**: Chunk-level summaries = experience-tree hub index. Exact expansion = lazy branch loading.

### Absorption Verdict
**ABSORB** — CEDAR provides the formal error bounds that GWT needs. Their "refinement replaces, not duplicates" principle validates our single-fact-source design. The dispersion-based error estimation should become our module health scoring mechanism. 3× speedup at 128K is directly relevant to our long-context experience retrieval.

---

## Paper 5: PARSER — Read in Parallel, Reason in Depth for Long-Context LLM Agents

**Source**: arXiv:2609.06702 (Sept 6, 2026)
**Authors**: (Not specified in source)

### Core Innovation
PARSER decouples reading from reasoning:
1. **Subagent bank** — lightweight subagents, each bound to one chunk, read entire document in parallel
2. **Lead agent** — reasons in depth through iterative scatter-gather rounds
3. **Iterative scatter-gather** — lead broadcasts query → subagents return evidence → lead formulates deeper follow-up
4. **RL-optimized lead, frozen subagents** — all learnable behavior in lead agent

### Key Results
- **5.7 points improvement** over strongest sequential baseline (4B backbone)
- **6.3 points improvement** over DeepSeek-V4-Pro (9B backbone)
- Robust to evidence position, order, and distance perturbations
- **11× latency reduction** vs sequential methods

### Pattern Extraction
| Pattern | Mechanism | NeoTrix Mapping |
|---------|-----------|-----------------|
| Parallel reading, sequential reasoning | Subagents read, lead reasons | GWT specialist broadcast + lead aggregation |
| Scatter-gather rounds | Iterative query→evidence→deeper query | ConsciousnessTree cycle iterations |
| Frozen subagents + learnable lead | Specialization of roles | Module specialization (specialist vs coordinator) |
| Position-robustness | Evidence placement doesn't matter | Knowledge storage agnostic to retrieval order |

### NeoTrix Domain Mapping
- **NT-CORE**: PARSER's parallel reading = GWT's broadcast to specialists. Scatter-gather = GWT's salience-based attention cycling. Position robustness validates our KB's content-addressable storage.
- **NT-MEMORY**: Subagent bank = experience-tree's lazy branch loading (each branch reads independently). Lead agent = ConsciousnessTree's coordinator.
- **NT-ACT**: Parallel subagents = our ParallelTaskManager's concurrent execution. Scatter-ground rounds = our task decomposition pattern.

### Absorption Verdict
**ABSORB** — PARSER's decoupling of reading and reasoning is our GWT's decoupling of perception and cognition. Their scatter-ground architecture is the production-grade version of our ConsciousnessTree's cycle iterations. The 11× latency reduction via parallel reading validates our NT-PHYSICAL's concurrent sensor processing model. Position robustness should inform our KB storage design.

---

## Cross-Paper Synthesis

### Emergent Pattern: Attention Is the New Routing

| Paper | Mechanism | NeoTrix Integration |
|-------|-----------|---------------------|
| **Declarative Attention** | Model declares attention regions intrinsically | GWT salience as intrinsic, not extrinsic |
| **CEDAR** | Error-bounded coarse-to-fine attention | GWT escalation with formal quality guarantees |
| **PARSER** | Parallel reading, sequential reasoning | GWT specialist broadcast + lead aggregation |
| **ReActNet** | Temporal graph compilation | SEAL pipeline stage → GWT broadcast |
| **SRPO** | Setwise multi-agent action optimization | EventBus batch dispatch optimization |

### Key Insight: Three Levels of Agent Coordination

| Level | Paper | Mechanism | NeoTrix Mapping |
|-------|-------|-----------|-----------------|
| **Attention** | Declarative Attention + CEDAR | Intrinsic attention declaration + error bounds | GWT salience broadcasting |
| **Communication** | ReActNet + SRPO | Temporal graphs + setwise optimization | EventBus + SEAL pipeline |
| **Reading** | PARSER | Parallel reading, sequential reasoning | Experience-tree lazy loading |

### Absorption Priority Matrix

| Priority | Paper | Domain | Action |
|----------|-------|--------|--------|
| **P0** | Declarative Attention | NT-CORE | Adopt three-mode attention vocabulary for GWT |
| **P0** | CEDAR | NT-CORE + NT-MEMORY | Implement error-bounded attention for KB retrieval |
| **P1** | PARSER | NT-CORE + NT-MEMORY | Parallel scatter-gather for experience-tree loading |
| **P1** | ReActNet | NT-CORE + NT-ACT | Temporal graph compilation for SEAL pipeline |
| **P2** | SRPO | NT-CORE + NT-ACT | Setwise action optimization for EventBus batch dispatch |

### Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| Declarative Attention's zero-shot vs. CEDAR's trained error bounds | DA for runtime inference, CEDAR for design-time guarantees. Different layers. |
| PARSER's frozen subagents vs. NeoTrix's self-evolving modules | Subagents are read-only workers; lead is the self-evolving coordinator. Our modules are all lead-level. |
| ReActNet's training-free vs. SRPO's RL optimization | ReActNet for inference-time flexibility, SRPO for training-time optimization. Both coexist. |
| CEDAR's additive refinement vs. NeoTrix's single-fact-source | CEDAR uses single softmax (non-additive). We adopt the non-additive principle. |
