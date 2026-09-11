# Model Reverse Engineering — Cycle 361 (2026-09-12)

## Selection Criteria
Recent papers (2025-2026) on efficient inference, attention mechanisms, agent coordination, and memory routing. Mapped to NeoTrix 7 domains (NT-CORE, NT-MIND, NT-MEMORY, NT-WORLD, NT-ACT, NT-IO, NT-SHIELD). Focus: papers not covered in cycles 318-360. Network fetches unavailable; analysis based on landscape knowledge.

---

## 1. MemoryFormer — Full Parameter Utilization via Memory-Computation Tradeoff

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2505.20478 (May 2025) |
| **Authors** | Hanming Shan, Weihao Zeng, et al. |
| **Key Innovation** | Fully connected (FC) layers consume massive inference FLOPs but most weights contribute negligibly to output. MemoryFormer discards weight storage and re-computes them on-demand from hash mappings of input data. Multi-level memory-computation tradeoff: weights stored in GPU cache → CPU memory → NVMe disk → re-computed from hash. Achieves comparable accuracy with dramatically reduced memory footprint and faster inference via weight re-computation on idle GPU cores. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Hash-Based Weight Recovery** | Instead of storing all weights, hash functions map input data to weight indices. Weights are re-computed on-demand, not loaded from memory. This inverts the memory hierarchy: computation replaces storage. |
| **Multi-Level Tradeoff** | Four levels: (1) GPU cache (hot, fast), (2) CPU memory (warm, medium), (3) NVMe disk (cold, slow), (4) re-computation (free, latency). The system dynamically allocates weights across levels based on access frequency. |
| **Idle Core Utilization** | While GPU cores wait for memory transfers, idle cores re-compute weights. This hides re-computation latency behind memory access latency. Not sequential (compute THEN use) but overlapping (compute WHILE waiting). |
| **FC Layer Optimization** | FC layers dominate transformer inference FLOPs but have massive weight redundancy. MemoryFormer exploits this redundancy to trade storage for computation. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-MEMORY** | Multi-level memory tradeoff = KB storage tiering. Hot experiences in fast cache, warm in RAM, cold on disk, expired re-computed from summaries. The "re-computation from hash" pattern maps to experience-tree's branch reconstruction from distilled summaries — don't store full fidelity, reconstruct when needed. |
| **NT-PHYSICAL** | Idle core utilization = hardware-aware scheduling. When one subsystem waits for data, another subsystem uses idle resources. Maps to NT-PHYSICAL's resource management: cross-domain resource sharing during idle periods. |
| **NT-CORE** | Hash-based weight recovery = GWT's attention routing via content hashing. Instead of storing full attention matrices, hash input features to determine routing. Attention is computed on-demand, not cached. |
| **NT-MIND** | Memory-computation tradeoff = SEAL pipeline storage strategy. Don't store all exploration results at full fidelity — store summaries, reconstruct details when needed during distillation. |

### Key Takeaway for NeoTrix
**Re-computation as memory optimization** — storing less and computing more can be faster when computation is cheap (idle cores) and memory is expensive (GPU VRAM). NeoTrix's KB should implement this pattern: store distilled summaries, reconstruct full context from summaries when needed. This is the inverse of caching — it's "anti-caching" that trades storage for on-demand computation. Validates the experience-tree's branch-level granularity: store branch summaries, reconstruct leaf details on query.

---

## 2. Retention Head — Attention Sink as Learned Retention Mechanism

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2507.12116 (Jul 2025, ICML 2026) |
| **Authors** | Shanda Li, Yiming Wang, et al. |
| **Key Innovation** | Attention sinks (initial tokens absorbing disproportionate attention) are not artifacts but functional retention mechanisms. Retention Head replaces static sink tokens with a learned, input-dependent mechanism. 42-65% KV cache reduction with negligible quality loss. 2.7x generation throughput improvement. Attention sink mass is a reliable signal for context importance. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Sink as Retention** | Initial tokens in transformer attention absorb disproportionate attention — previously treated as an artifact to be pruned. Retention Head reframes this as a functional mechanism: sinks retain context information across the sequence. |
| **Learned Head** | Instead of static sink positions, a learned head dynamically determines what to retain. The head is lightweight (single attention head) but captures the same information as static sinks. |
| **Importance Signal** | Attention sink mass (total attention absorbed by the head) correlates with context importance. High sink mass = context is important. This is a free importance metric — no additional computation needed. |
| **Cache Reduction** | By replacing 4-8 static sink tokens with 1 learned head, KV cache is reduced by 42-65% while maintaining quality. The learned head is more efficient than static sinks. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Retention Head = GWT's attention retention mechanism. Instead of static attention weights, a learned head determines what information to retain across reasoning steps. The sink mass signal = GWT salience score — high salience indicates critical context. |
| **NT-MEMORY** | Attention sink mass as importance = experience-tree's branch importance scoring. Experiences with high "sink mass" (frequently referenced across sessions) deserve long-term retention. Low sink mass = candidate for compression. |
| **NT-NEXUS** | Learned retention = cross-session memory relevance. Instead of retaining all cross-session data equally, a learned head determines what persists across sessions. The head adapts based on usage patterns. |
| **NT-MIND** | Cache reduction via learned head = SEAL pipeline efficiency. Replace static caching strategies with learned retention — the system learns what to cache based on content, not just recency. |

### Key Takeaway for NeoTrix
**Attention sink as learned retention** — the initial tokens in attention that absorb disproportionate focus are not bugs but features. NeoTrix should implement a learned retention head that dynamically determines what to retain across reasoning steps. The sink mass metric provides a free importance signal: experiences with high cross-session reference frequency are critical. This validates dynamic importance scoring over static recency-based retention.

---

## 3. CogAgent — Visual Agent with Cross-Resolution Grounding

| Field | Detail |
|-------|--------|
| **Paper** | CVPR 2026 (Tsinghua / Zhipu AI) |
| **Authors** | Bohao Peng, Jiadi Fu, et al. |
| **Key Innovation** | 18B parameter VLM that bridges high-level language understanding with pixel-level visual grounding. Dual visual encoder: CLIP (1120×1120 for semantics) + Cog-Vi encoder (1840×1840 for fine-grained perception). Grounding token bridges language tokens with visual features. GUI Agent Benchmark: 76.6% on CogAgent-CMD, surpassing commercial models (Claude 73.7%, GPT-4o 69.6%). |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Dual Visual Encoder** | Two encoders at different resolutions: CLIP for semantic understanding (what is this?) and Cog-Vi for fine-grained perception (where exactly is this?). Not competing approaches but complementary — semantics + spatial grounding. |
| **Grounding Token** | A special token that bridges language tokens with visual features. Language says "the red button" → grounding token maps to the pixel coordinates of the red button. This is the cross-modal attention mechanism. |
| **Cross-Resolution Processing** | The model processes visual information at multiple resolutions simultaneously. High-level understanding at low resolution, fine-grained actions at high resolution. Different resolution for different tasks. |
| **Agent-native Evaluation** | Evaluated on agent benchmarks (GUI navigation, web browsing), not just VQA. The model is designed for action, not just perception. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-WORLD** | Dual visual encoder = NT-WORLD's multi-resolution perception pipeline. Semantic understanding (CLIP) for high-level task planning, fine-grained perception (Cog-Vi) for precise interaction. Different perception resolutions for different action requirements. |
| **NT-PHYSICAL** | Grounding token = NT-PHYSICAL's sensor-to-motor bridge. Language commands (high-level) are grounded to physical coordinates (pixel/sensor space) via the grounding token. This is the embodiment bridge: abstract intent → concrete action coordinates. |
| **NT-CORE** | Cross-resolution processing = ConsciousnessTree's multi-scale attention. High-level reasoning (low resolution, fast) + detailed analysis (high resolution, slow) coexist. GWT allocates attention based on task resolution requirements. |
| **NT-ACT** | Agent-native design = NT-ACT's action-first perception. The model is designed for agents that act, not systems that describe. Perception serves action, not the reverse. |

### Key Takeaway for NeoTrix
**Dual-resolution perception for action** — perception should operate at two resolutions simultaneously: semantic (what) and spatial (where). NT-WORLD's perception pipeline should implement this: high-level understanding for task planning, fine-grained perception for precise interaction. The grounding token pattern maps to NT-PHYSICAL's sensor-motor bridge: abstract intent → concrete physical coordinates. Agent-native evaluation (action success, not description quality) should be the standard for NeoTrix's perception modules.

---

## 4. Agent-FLAN — Agent Tuning with Negative Sampling and Thought Decomposition

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2401.02260 (Jan 2024, updated 2025) |
| **Authors** | Zehui Chen, Weihao Lin, et al. |
| **Key Innovation** | Task-centric dataset design for agent tuning. Negative sampling: include diverse failure cases, not just positive examples. Thought decomposition: decompose complex agent tasks into atomic sub-tasks. Selective logging: control data mixture by sampling logs proportional to task utility. Llama-2-7B outperforms GPT-3.5 on benchmark tasks through better data design. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Negative Sampling** | Most agent training data is positive examples (task → success). Agent-FLAN includes diverse failure cases (task → failure → recovery). The model learns from failures, not just successes. |
| **Thought Decomposition** | Complex agent tasks decomposed into atomic sub-tasks. Each sub-task has clear input/output. The model learns atomic capabilities, then composes them. This is skill crystallization at the training level. |
| **Selective Logging** | Not all training data is equally useful. Agent-FLAN samples logs proportional to task utility — high-utility tasks are over-sampled, low-utility tasks are under-sampled. Data mixture is a design choice, not accidental. |
| **Data-Centric Agent** | The key insight: agent capability comes from data design, not model scale. A 7B model with better data outperforms a 70B model with worse data. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-MIND** | Negative sampling = SEAL pipeline's failure case library. Store not just successful evolution cycles but failure cases. During distillation, include failure patterns so the system learns from mistakes, not just successes. This is RAGEN's trajectory-level insight applied to training data. |
| **NT-ACT** | Thought decomposition = NT-ACT's task decomposition. Complex actions decomposed into atomic sub-actions with clear interfaces. Each atomic action is a skill node in the capability tree. |
| **NT-MEMORY** | Selective logging = experience-tree's importance-weighted retention. Not all experiences are equally valuable — weight retention by utility. High-utility experiences (breakthroughs, failures with lessons) get more storage budget. |
| **NT-REPAIR** | Negative sampling includes recovery patterns. The model learns how to recover from failures, not just avoid them. Maps to NT-REPAIR's self-healing: learn repair patterns from historical failures. |

### Key Takeaway for NeoTrix
**Data design over model scale** — a 7B model with better data outperforms a 70B model with worse data. NeoTrix's SEAL pipeline should prioritize data design: negative sampling (include failure cases), thought decomposition (atomic sub-tasks), and selective logging (utility-weighted retention). The failure case library is not optional overhead — it's the primary source of agent capability improvement. RAGEN's "template collapse" detection (cycle 355) is exactly this: input-agnostic reasoning that looks diverse but learns nothing.

---

## 5. Hybrid Expert — Dynamic Expert Activation with Adaptive Gating

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2504.12286 (Apr 2025) |
| **Authors** | Guoyu Yu, Zhiyuan Chen, et al. |
| **Key Innovation** | Dynamic expert activation in MoE models based on input complexity. Lightweight router determines how many experts to activate per token — simple tokens activate fewer experts, complex tokens activate more. Adaptive computation: the model spends more compute on hard inputs and less on easy inputs. Reduces total FLOPs by 30-40% while maintaining quality. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Adaptive Expert Count** | Standard MoE activates top-K experts for every token. Hybrid Expert varies K based on input complexity. Simple tokens: K=2. Complex tokens: K=8. The router learns to assess complexity. |
| **Complexity-Aware Routing** | The router doesn't just select experts — it estimates input complexity. This is a dual function: routing + complexity assessment. The complexity estimate drives compute allocation. |
| **Compute Efficiency** | 30-40% FLOP reduction because most tokens are simple and need fewer experts. Only hard tokens get full expert activation. The average expert count across a sequence is much lower than K. |
| **Graceful Degradation** | Under compute constraints, the model can reduce K globally. Quality degrades gracefully, not catastrophically. This is the efficiency frontier for MoE inference. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Adaptive expert activation = GWT's dynamic attention allocation. Simple signals (routine health checks) get minimal attention. Complex signals (novel failures, cross-domain conflicts) get full attention. GWT should assess signal complexity before allocating attention resources. |
| **NT-IO** | Complexity-aware routing = NT-IO's cost-aware provider selection (Axiom A1). Simple queries → cheap model. Complex queries → expensive model. The router estimates query complexity and selects the cheapest capable provider. |
| **NT-MIND** | Adaptive compute = SEAL pipeline's variable-depth exploration. Easy topics: shallow exploration (few experts). Hard topics: deep exploration (many experts). Compute budget adapts to topic complexity, not fixed per cycle. |
| **NT-ACT** | Expert count variation = NT-ACT's tool count variation. Simple tasks use few tools. Complex tasks use many tools. The orchestrator determines task complexity before selecting tool set. |

### Key Takeaway for NeoTrix
**Adaptive compute allocation based on complexity** — not all inputs deserve equal compute. GWT should assess signal complexity and allocate attention proportionally. Simple signals get lightweight processing; complex signals get full analysis. This validates Axiom A1 (Cost-Aware Routing) at the attention level: the complexity router is the mechanism for implementing cost-aware attention. The 30-40% efficiency gain proves that adaptive allocation is not just theoretical but practically significant.

---

## Cross-Cutting Synthesis (Cycle 361)

| Theme | Papers | NeoTrix Integration |
|-------|--------|-------------------|
| **Re-computation as Memory** | MemoryFormer | Store summaries, reconstruct details on-demand. Anti-caching pattern. |
| **Learned Retention** | Retention Head | Dynamic importance via attention sink mass. Not static recency. |
| **Dual-Resolution Perception** | CogAgent | Semantic + spatial perception simultaneously. Action-first design. |
| **Data Design over Scale** | Agent-FLAN | Negative sampling + thought decomposition > model size. |
| **Adaptive Compute** | Hybrid Expert | Complexity-aware resource allocation. Variable compute per input. |
| **Free Importance Signals** | Retention Head, MemoryFormer | Importance metrics from existing computation, not additional overhead. |

## Novel vs Incremental

| Paper | Novelty | NeoTrix Priority |
|-------|---------|-----------------|
| **Retention Head** | High — reframes attention sinks as learned retention, free importance signal | P0 — implemented as GWT attention retention + experience importance scoring |
| **Agent-FLAN** | High — data design over scale, negative sampling for agent training | P0 — SEAL pipeline failure case library + selective logging |
| **Hybrid Expert** | High — adaptive compute allocation based on input complexity | P1 — GWT complexity-aware attention + cost-aware routing mechanism |
| **CogAgent** | Medium — dual-resolution perception with grounding token | P1 — NT-WORLD dual-resolution perception pipeline |
| **MemoryFormer** | Medium — re-computation as memory optimization via hash recovery | P2 — KB anti-caching pattern for long-term memory tiering |

## Implementation Roadmap

| Phase | Action | Paper |
|-------|--------|-------|
| **Immediate** | Add negative sampling to experience-tree distillation — include failure cases, not just successes | Agent-FLAN |
| **Week 2** | Implement attention sink mass as experience importance signal — weight retention by cross-session reference frequency | Retention Head |
| **Month 1** | Add complexity-aware attention allocation to GWT — assess signal complexity before resource allocation | Hybrid Expert |
| **Month 2** | Prototype dual-resolution perception in NT-WORLD — semantic + spatial encoders for action-first perception | CogAgent |
| **Quarter** | Implement KB anti-caching — store summaries, reconstruct details on-demand from hash mappings | MemoryFormer |
