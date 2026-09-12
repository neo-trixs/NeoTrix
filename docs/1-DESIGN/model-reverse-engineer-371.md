# Model Reverse Engineering — Cycle 371 (2026-09-12)

## 5 New AI Models/Papers → NeoTrix Domain Mapping

---

## 1. AgentInfer — Co-Design of Inference Architecture and System
**Paper**: arXiv:2512.18337v2 | **Authors**: Weizhe Lin et al. (Huawei)

### Core Idea
Agent inference requires synergistic optimization across reasoning AND inference engine layers, not just model-level improvements. AgentInfer decomposes the problem into 4 components:
- **AgentCollab**: Hierarchical dual-model reasoning — large model for hard subtasks, small model for easy ones, dynamic role assignment
- **AgentSched**: Cache-aware hybrid scheduler that minimizes latency under heterogeneous request patterns
- **AgentSAM**: Suffix-automaton speculative decoding that reuses multi-session semantic memory across agent sessions
- **AgentCompress**: Asynchronous semantic compression — distills and reorganizes memory without disrupting ongoing reasoning

### Key Results
- 50% reduction in ineffective token consumption
- 1.8-2.5x overall speedup with preserved accuracy
- Validated on BrowseComp-zh and DeepDiver benchmarks

### Pattern Extracted
**Full-Stack Agent Optimization**: Optimize the entire agent loop (reasoning + inference + memory + scheduling) as a unified system, not individual components. AgentSAM's cross-session token reuse is the key insight — speculative decoding draft models can share predictions across sessions.

### NeoTrix Mapping

| Domain | Integration |
|--------|-------------|
| **NT-CORE (GWT)** | AgentCollab's dual-model dynamic role assignment maps to GWT's cost-aware routing (Axiom A1). Route hard tasks to large models, easy tasks to small models — dynamically. |
| **NT-IO (provider routing)** | AgentSched's cache-aware scheduling = NT-IO's Ordered Backend Router with workload awareness. Route by request pattern, not just provider availability. |
| **NT-MEMORY (KVMem)** | AgentSAM's cross-session suffix automaton = KVMem's paged KV reuse. Share draft predictions across sessions to reduce redundant compute. |
| **SEAL pipeline** | AgentCompress's asynchronous memory distillation = SEAL knowledge crystallization running as background process, not blocking the main reasoning loop. |
| **Axiom A2** | AgentInfer validates context-as-scarce-resource at the SYSTEM level: the entire agent pipeline, not just the model, must manage context efficiently. |

### Action Item
Design a "Full-Stack Agent Optimizer" in NeoTrix: integrate GWT salience (routing) + KVMem paged KV (memory) + SEAL background distillation (compression) + cross-session suffix automaton (speculative reuse) as a unified optimization pipeline.

---

## 2. PEPO — Perception-Exploration Policy Optimization for Multimodal CoT
**Paper**: arXiv:2603.22847 | **Authors**: Yunheng Li et al.

### Core Idea
Multimodal Chain-of-Thought reasoning requires interleaving perceptual grounding with multi-step inference. Existing RLVR methods optimize reasoning at coarse granularity, treating all CoT tokens uniformly. PEPO performs token-level analysis and discovers that successful reasoning has **structured token dynamics** — some tokens are perception-grounded, others are exploratory inference.

PEPO derives a **perception prior** from hidden state similarity and integrates it with token entropy via smooth gating to produce token-level advantages. Perception-heavy tokens get different reinforcement signals than reasoning-heavy tokens.

### Key Results
- Consistent improvements across geometry reasoning, visual grounding, visual puzzle solving, few-shot classification
- Seamless integration with GRPO and DAPO (no additional supervision needed)
- Stable training dynamics

### Pattern Extracted
**Token-Type-Aware Reinforcement**: Not all tokens in a reasoning chain are equal. Perception tokens (grounded in visual input) need different optimization pressure than exploration tokens (hypothesis generation). Reinforce them separately.

### NeoTrix Mapping

| Domain | Integration |
|--------|-------------|
| **NT-CORE (ConsciousnessTree)** | PEPO's token-type distinction maps to ConsciousnessTree's perception vs. cognition branches. Perception tokens = L2 (Perception), exploration tokens = L5 (Cognition). |
| **NT-FEEL** | Token entropy as "curiosity" signal. High-entropy tokens = exploratory = curiosity emotion. Low-entropy tokens = grounded = confidence. Maps to EmotionLabel variants. |
| **SEAL pipeline** | PEPO's perception-exploration gating maps to SEAL's self-test phases: perception = T1 (existence check), exploration = T2/T3 (registration/wiring). |
| **Axiom A1** | Cost-Aware Routing extended: perception tokens are cheap (pretrained visual encoder), exploration tokens are expensive (LLM reasoning). Route compute budget by token type. |

### Action Item
Add token-type awareness to NeoTrix's reasoning path: classify reasoning tokens as perception-grounded vs. exploratory, apply different processing strategies (perception: cache hit, exploration: full inference).

---

## 3. AsymSpec — Context-Asymmetric Speculative Decoding for Agentic LLMs
**Paper**: arXiv:2608.26004 | **Venue**: EMNLP 2026
**Authors**: Sheng Liang et al.

### Core Idea
Agentic pipelines accumulate massive context across retrieval, tool use, and multi-turn interactions. Standard practice compresses inputs to fit context windows, but this degrades accuracy. Standard speculative decoding assumes drafter and verifier share identical context, so it can't help with the compression-accuracy tradeoff.

AsymSpec breaks this symmetry: a **lightweight drafter reads the full input** while the **large verifier operates on the compressed view**. The drafter steers the verifier via contrastive δ-fusion of logits, with a divergence-aware acceptance gate.

### Key Results
- ~90% of full-context accuracy on average (vs. much lower with pure compression)
- 1.3-1.7x throughput speedup at 0.2-0.3x compute cost
- Works across 4 agentic capabilities + 2 end-to-end agent benchmarks

### Pattern Extracted
**Asymmetric Context Access**: Give the cheap model the full context, give the expensive model the compressed context. The cheap model's role is to compensate for what the compression lost. This is the inverse of standard distillation — the student knows MORE than the teacher.

### NeoTrix Mapping

| Domain | Integration |
|--------|-------------|
| **NT-IO (provider routing)** | AsymSpec's asymmetric context = NT-IO's provider routing with tiered context. Cheap provider gets full context, expensive provider gets compressed. |
| **NT-MEMORY (KVMem)** | KVMem's hot/cold KV tiering is the software analog. AsymSpec proves the drafter (cold tier) can use MORE context than the verifier (hot tier) if the drafter is cheap enough. |
| **Axiom A1 + A2** | AsymSpec synthesizes both axioms: cheap model routes with full context (A1: cost-aware), expensive model operates on compressed context (A2: context is scarce). |
| **Skill Tree** | "Asymmetric Inference" as a Keystone node under NT-IO. This is a fundamentally new inference paradigm, not just an optimization. |

### Action Item
Design an asymmetric inference path in NT-IO: for agentic workloads, route full context to a cheap drafter model, route compressed context to the expensive verifier. The drafter compensates for compression artifacts.

---

## 4. Agent-Radar — Attention Steering with Context Relevance
**Paper**: arXiv:2605.30136 | **Authors**: Agent-Radar Team

### Core Idea
As conversations lengthen, relevant information is diluted by irrelevant context. Existing solutions (compression, pruning, retrieval) discard information. Agent-Radar takes a different approach: **steer attention** toward relevant context without modifying the context itself.

Key innovations:
- **Temporal decay**: Recent context gets higher relevance
- **Spatial decay**: Context closer to current position gets higher relevance
- **Sentence-level semantic relevance scoring**: Each sentence scored for relevance
- Uses SPA (Selective Prompt Anchoring) as backend — adjusts logit distribution to emphasize specified context

### Key Results
- Works across 3 base LLMs, 5 benchmarks
- No training required
- Preserves full transcript and topology (no information loss)
- Outperforms compression and pruning methods on long-context tasks

### Pattern Extracted
**Attention Steering > Context Compression**: Don't shrink the context — redirect the attention. The context is complete, but the model's attention needs guidance. This is a paradigm shift from "manage what the model sees" to "manage where the model looks."

### NeoTrix Mapping

| Domain | Integration |
|--------|-------------|
| **NT-CORE (GWT)** | Agent-Radar is GWT at the attention level. GWT broadcasts salient info across modules; Agent-Radar steers attention within a single module. Same principle, different granularity. |
| **PerceptionBridge** | Agent-Radar's temporal+spatial decay extends PerceptionBridge (L2→L5). PerceptionBridge uses `awareness_score()` to filter; Agent-Radar adds temporal and spatial dimensions to that scoring. |
| **NT-MEMORY** | Agent-Radar's sentence-level relevance scoring = KB query scoring. Both score information pieces by relevance to current context. |
| **Axiom A2** | Context-as-scarce-resource reframed: the context isn't scarce because it's too long — it's scarce because attention is misdirected. Agent-Radar fixes the attention, not the context. |

### Action Item
Integrate Agent-Radar's temporal+spatial decay into GWT salience calculation. Add sentence-level relevance scoring to PerceptionBridge's `awareness_score()`. This makes attention routing context-aware, not just salience-aware.

---

## 5. Trace as State — Reasoning Traces as Conditional States for Long-Context Transformers
**Paper**: alphaXiv:2609.02702 | **Authors**: Ok & Lee

### Core Idea
Long-context transformers suffer from **prompt-order sensitivity** — the same information placed at different positions in the context produces different answers. This is because causal attention creates asymmetric information access: tokens at the beginning of context are attended to by all later tokens, but tokens at the end have limited reach.

Trace as State proposes: after an initial reasoning pass, extract the **discovered task state** (the reasoning trace) and move it to the FRONT of the context for a second pass. This ensures the most critical information has maximum attention reach.

### Key Results
- Significant accuracy gains for complex reasoning tasks
- Two-pass inference: first pass discovers state, second pass uses it as front-loaded context
- Addresses prompt-order sensitivity fundamentally

### Pattern Extracted
**State Extraction + Front-Loading**: Reasoning produces a "state" (trace) that is more important than the raw context. Extract the state, place it where attention reaches it best (front of context), then re-reason with state-as-context.

### NeoTrix Mapping

| Domain | Integration |
|--------|-------------|
| **NT-CORE (GWT)** | Trace as State = GWT broadcast refinement. GWT broadcasts salient info; Trace as State ensures the most salient info (the trace) gets maximum attention (front-loaded). |
| **NT-MEMORY (KVMem)** | KVMem's paged KV + Trace as State's front-loading = attention-optimized memory layout. Hot pages = front of context, cold pages = back. |
| **SEAL pipeline** | Trace extraction maps to SEAL distillation phase: raw reasoning → distilled state. The state is the "fruit" of the SEAL cycle. |
| **Axiom A2** | Context-as-scarce-resource at the ATTENTION level: not all positions in context are equal. Front positions have more attention reach. Optimize position, not just content. |

### Action Item
Implement "State Front-Loading" in NT-CORE: after initial reasoning pass, extract the key state, prepend it to context for a follow-up pass. Track accuracy improvement vs. double inference cost.

---

## Cross-Paper Synthesis

### Meta-Pattern M1: Full-Stack Agent Optimization
AgentInfer (4-component system optimization) + AsymSpec (asymmetric context) + Agent-Radar (attention steering) all prove that **agent efficiency requires optimizing the entire pipeline**, not individual components. The agent loop (reason → retrieve → act → compress → repeat) must be optimized holistically.

### Meta-Pattern M2: Attention Is the Bottleneck, Not Context
Agent-Radar (attention steering) + Trace as State (state front-loading) both demonstrate that the problem isn't context length — it's attention misdirection. NeoTrix's GWT should evolve from "broadcast salient info" to "steer attention toward relevant info."

### Meta-Pattern M3: Token-Type Awareness
PEPO (perception vs. exploration tokens) + AgentInfer (dual-model routing) both recognize that **not all tokens are equal**. Perception tokens are cheap, exploration tokens are expensive. Route compute budget by token type.

### Meta-Pattern M4: Asymmetric Knowledge Access
AsymSpec (cheap model gets full context, expensive model gets compressed) + AgentSAM (cross-session token reuse) both exploit the principle that **the cheap model can know more than the expensive model** if structured correctly.

---

## Priority Absorption Queue

| Priority | Item | Domain | Effort |
|----------|------|--------|--------|
| P0 | Agent-Radar temporal+spatial decay → GWT salience | NT-CORE | Medium |
| P0 | AsymSpec asymmetric context → NT-IO provider routing | NT-IO | Medium |
| P1 | AgentInfer full-stack optimizer → unified pipeline | NT-CORE + NT-IO + NT-MEMORY | High |
| P1 | PEPO token-type awareness → reasoning path | NT-CORE + NT-FEEL | Medium |
| P2 | Trace as State front-loading → KVMem attention optimization | NT-MEMORY | Medium |
