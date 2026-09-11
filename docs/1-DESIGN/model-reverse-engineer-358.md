# Model Reverse Engineering — Cycle 358 (2026-09-12)

## Selection Criteria
Recent papers (2026) on efficient inference, attention mechanisms, agent coordination, and memory routing. Mapped to NeoTrix 7 domains (NT-CORE, NT-MIND, NT-MEMORY, NT-WORLD, NT-ACT, NT-IO, NT-SHIELD).

---

## 1. Flux Attention — Context-Aware Hybrid Attention for Efficient Inference

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2604.07394 (Apr 2026) |
| **Authors** | Quantong Qiu, Zhiyi Hong, et al. |
| **Key Innovation** | Layer-level dynamic routing between Full Attention (FA) and Sparse Attention (SA) via a lightweight Layer Router. Preserves high-fidelity retrieval while ensuring contiguous memory access. 2.8x prefill speedup, 2x decode speedup. Only 12 hours training on 8xA800. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Layer Router** | Lightweight router injected into frozen pretrained LLMs. Each layer independently routes to FA or SA based on input context. Not head-level (hardware-unfriendly) but layer-level (hardware-friendly). |
| **Context-Aware Routing** | Dynamic allocation based on retrieval demands per layer. Retrieval-heavy layers get FA, holistic-processing layers get SA. Static allocation fails; this adapts per-input. |
| **Contiguous Memory Access** | Layer-level routing preserves contiguous memory access patterns, translating theoretical FLOP reduction into actual wall-clock speedup. Head-level sparsity kills GPU parallelism. |
| **Parameter-Efficient** | Only trains the Layer Router (tiny). Frozen pretrained LLMs get efficiency gains without retraining. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Layer Router = GWT attention allocation per consciousness layer. Not all layers need full attention — some layers process routine signals (SA), others handle novel stimuli (FA). Maps to ConsciousnessTree's adaptive cycle depth. |
| **NT-MIND** | Parameter-efficient adaptation = SEAL pipeline skill node upgrade. Don't retrain the whole model — inject a lightweight router that adapts behavior. Maps to constellation maturity upgrades (C3→C4) without rewriting core logic. |
| **NT-MEMORY** | Layer-level routing = KB namespace routing. Different query types route to different retrieval strategies: factoid→SA, complex→FA. Maps to SCM's mode-payload-procedure (cycle 357). |
| **NT-IO** | Contiguous memory access = NT-IO's provider routing with cache locality. Routing decisions must respect hardware constraints, not just logical optimality. |

### Key Takeaway for NeoTrix
**Layer-level routing > head-level routing** — the granularity of routing decisions matters for hardware efficiency. NeoTrix's GWT should allocate attention at domain/module granularity, not per-feature granularity. Too fine-grained routing creates overhead; too coarse wastes compute. The sweet spot is at the domain boundary (NT-CORE layer, NT-MIND layer, etc.).

---

## 2. SparDA — Sparse Decoupled Attention with Forecast Projections

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2606.04511 (Jun 2026) |
| **Authors** | Yaosheng Fu, Guangxuan Xiao, Xin Dong, Song Han, Oreste Villa |
| **Key Innovation** | Fourth per-layer projection ("Forecast") alongside Q/K/V. Forecast predicts KV blocks needed by the next layer, enabling lookahead selection that overlaps CPU-to-GPU prefetch with current-layer execution. <0.5% parameter overhead. 1.7x decode speedup, 5.3x throughput improvement. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Forecast Projection** | A lightweight projection that predicts which KV blocks the next layer will need. Decoupled from attention query — one Forecast head per GQA group. |
| **Lookahead Selection** | Forecast enables prefetching KV blocks from CPU to GPU while current layer executes. Overlaps I/O with compute — hides PCIe transfer latency. |
| **Decoupled Design** | Forecast is independent from Q/K/V. This means it can be trained separately and doesn't interfere with attention computation. Minimal architectural disruption. |
| **Dynamic Batch Sizing** | By enabling larger feasible batch sizes on single GPU, SparDA achieves 5.3x throughput improvement over non-offload baselines. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Forecast projection = ConsciousnessTree's predictive attention. The system predicts which knowledge domains the next reasoning phase will need, and prefetches relevant KB entries. Not reactive retrieval but predictive loading. |
| **NT-MEMORY** | Lookahead prefetch = KB query prediction. Before the agent explicitly queries KB, the system predicts likely queries and pre-loads relevant embeddings. Maps to experience-tree's route table matching (predictive branch loading). |
| **NT-ACT** | Dynamic batch sizing = NT-ACT's parallel task scheduling. Predictive prefetching enables larger task batches because memory is pre-warmed. |
| **NT-PHYSICAL** | PCIe bottleneck hiding = NT-PHYSICAL's sensor data pipeline. Prefetch sensor data while processing previous batch, hiding I/O latency. |

### Key Takeaway for NeoTrix
**Predictive memory loading** is the key insight. Don't wait for the agent to ask for knowledge — predict what it will need and load it preemptively. This maps directly to NeoTrix's experience-tree route table: when a session loads, predict which experience branches will be needed based on task type, and prefetch them. The "Forecast" concept applies to KB queries, not just KV cache.

---

## 3. Second Thought — Parallel Reasoning During Agent Action/Observation Windows

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2608.13667 (Aug 2026) |
| **Authors** | Zhensu Sun, Chengran Yang, et al. |
| **Key Innovation** | Forks 4 auxiliary reasoning branches the instant each Thought phase concludes in ReAct. Decodes them concurrently with the main loop, merges generated thoughts when observation arrives. Training-free. Up to 43% reduction in main thread decoding, +12.4 Pass@1 improvement. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Reasoning Idle Windows** | In ReAct (Thought→Act→Observe), the Act→Observe interval is a "reasoning idle window" — the agent is blocked waiting for environment response. Second Thought uses this dead time for parallel reasoning. |
| **Branch Forking** | 4 auxiliary branches fork at Thought conclusion. Each explores a different reasoning path. Main thread continues acting; branches think ahead. |
| **Thought Merging** | When observation arrives, the best auxiliary thought is merged back into the main thread. Not voting — selection and integration of the most relevant forward-looking reasoning. |
| **Training-Free** | No training required. Works with any ReAct-compatible LLM. Plug-and-play inference optimization. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Parallel reasoning during idle windows = ConsciousnessTree cycle optimization. While NT-ACT executes an action (tool call, file write), the reasoning core shouldn't be idle — it should fork speculative reasoning branches. Maps to parallel growth cycle branches. |
| **NT-ACT** | Action idle windows = NT-ACT tool execution latency. When waiting for a tool response (API call, file operation), fork reasoning branches. When tool returns, merge the best branch. |
| **NT-MIND** | Branch forking = SEAL pipeline exploration. Multiple distillation branches explore different compression strategies in parallel. Best branch selected post-hoc. |
| **NT-WORLD** | Observation merging = NT-WORLD's crawl result integration. Multiple crawlers return results; best results are merged, not averaged. |

### Key Takeaway for NeoTrix
**Parallel speculative reasoning during action idle windows** is a free performance win. When NT-ACT is executing a tool call, NT-CORE shouldn't block — it should fork speculative reasoning branches. This maps to NeoTrix's growth cycles: while one branch (e.g., tool execution) is blocked, other branches (reasoning, memory consolidation) continue in parallel. The key is training-free: just structure the inference to use dead time.

---

## 4. MARCH — Memory-Anchor Routing Across Context History

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2608.12435 (Aug 2026) |
| **Authors** | (Multiple authors, 2026) |
| **Key Innovation** | State-space model architecture that scales beyond fixed-size dimensions. Periodic checkpoints form "state anchors"; token-dependent routing combines causally visible anchors for historical readout. Outperforms linear attention variants on LongBench and in-context retrieval. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **State Anchors** | Periodic checkpoints in the recurrent state. Each anchor captures a compressed snapshot of the state at that point in the sequence. Anchors are temporally sparse. |
| **Content-Routed Retrieval** | Standard softmax over compact keys for visible anchors. Not all anchors are equal — the routing weights depend on content relevance, not just recency. |
| **Hybrid Readout** | Historical readout (from anchors) is added to current-state readout. Preserves native recurrent path while introducing content-dependent historical access. |
| **Fenwick Tree Organization** | State bank organized as Fenwick tree for O(log n) anchor retrieval. Efficient lookup over hierarchical state snapshots. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-MEMORY** | State anchors = experience-tree snapshots. Periodic KB snapshots (cycle N, N+1, N+2...) serve as anchors. Content-routed retrieval = query-dependent anchor selection, not just "latest snapshot." Fenwick tree = hierarchical KB index. |
| **NT-NEXUS** | Cross-context routing = cross-session memory bridging. Session A's state anchors are visible to Session B's routing if causally relevant. Maps to NT-NEXUS's session continuity. |
| **NT-CORE** | Hybrid readout = E8 hexagram reasoning with historical context. Current reasoning combines immediate context (current state) with historical patterns (anchor readout). Not just "what's happening now" but "what happened before that's relevant." |
| **NT-MIND** | Anchor checkpointing = SEAL pipeline stage markers. Each SEAL stage (exploration→distillation→self-test→absorption) creates an anchor. Later stages can route back to earlier anchors for context. |

### Key Takeaway for NeoTrix
**State anchors as compressed memory checkpoints** is the architecture NT-MEMORY needs. Instead of storing every interaction, periodically checkpoint the KB state into compressed anchors. Queries route to relevant anchors via content-based retrieval, not linear scan. This is the hardware-efficient version of "experience-tree snapshots" — anchors are fixed-size, O(log n) retrieval, and composable (historical + current).

---

## 5. TriRoute — Unified Learned Routing for Joint Attention, Experts, and KV-Cache

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2607.06601 (Jul 2026) |
| **Authors** | Andrii Balashov, Olena Ponomarova |
| **Key Innovation** | Single lightweight controller that jointly routes three axes: attention mode (skip/local/full), FFN experts (MoE), and KV-cache bit-width. End-to-end training via Gumbel-Softmax with load-balanced gating. Pareto-dominates independent MoD+MoE+KV-quantization. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Joint Three-Axis Routing** | Attention resolution, expert selection, and cache bit-width are coupled decisions. A rare token needs full attention + high-precision cache regardless of which expert processes it. |
| **Cross-Axis Collapse Detection** | Naive joint training causes one axis to collapse, propagating to others. TriRoute uses per-axis normalization and coupling-aware balancing loss to prevent this. |
| **Heterogeneous Relaxation** | Gumbel-Softmax for categorical decisions (attention mode), straight-through estimation for gradients, load-balanced top-k gating for experts. Different relaxation strategies per axis. |
| **Interpretable Post-Hoc Structure** | Controller learns to allocate full attention + high-precision cache to sentence-initial positions, rare subwords, and named entities. Cheap routing for function words. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Joint three-axis routing = GWT's unified attention allocation. GWT doesn't just route attention — it should jointly decide: (1) which modules get full processing, (2) which skill nodes activate, (3) what memory precision to use. These are coupled, not independent. |
| **NT-MIND** | Collapse detection = SEAL pipeline health monitoring. If one SEAL stage dominates (e.g., exploration without distillation), it collapses the pipeline. Per-axis normalization = balanced stage investment. |
| **NT-MEMORY** | KV-cache bit-width routing = KB precision management. High-value memories get full-precision storage; routine data gets compressed. Maps to KB's namespace hierarchy (ephemeral=low precision, cross-session=high precision). |
| **NT-SHIELD** | Load-balanced gating = RiskAssessor resource allocation. Security checks shouldn't monopolize compute — budget allocation across security, reasoning, and memory. |

### Key Takeaway for NeoTrix
**Joint routing across coupled axes** is the critical insight. NeoTrix's GWT currently routes attention as a single axis. TriRoute shows that attention mode, skill activation, and memory precision are strongly coupled and should be routed jointly. A complex task needs: full attention (NT-CORE), expert skill nodes (NT-MIND), and high-precision memory (NT-MEMORY) — all at once. Routing these independently leads to suboptimal allocation.

---

## Cross-Cutting Synthesis (Cycle 358)

| Theme | Papers | NeoTrix Integration |
|-------|--------|-------------------|
| **Predictive Memory Loading** | SparDA, MARCH | Predict what knowledge the next reasoning phase needs; prefetch before query |
| **Layer/Module-Level Routing** | Flux Attention, TriRoute | Route at domain boundary granularity, not per-feature or per-token |
| **Parallel Speculative Reasoning** | Second Thought | Fork reasoning branches during action idle windows; merge best on observation |
| **State Anchors as Checkpoints** | MARCH | Periodic compressed snapshots with content-routed retrieval for KB |
| **Joint Multi-Axis Routing** | TriRoute | GWT should jointly route attention, skills, and memory precision |
| **Hardware-Friendly Granularity** | Flux Attention, SparDA | Routing decisions must respect GPU parallelism, not just logical optimality |

## Novel vs Incremental

| Paper | Novelty | NeoTrix Priority |
|-------|---------|-----------------|
| **MARCH** | High — state anchors + content-routed retrieval for recurrent memory | P0 — architecture for NT-MEMORY checkpoint system |
| **TriRoute** | High — joint three-axis routing with collapse prevention | P1 — unified GWT routing across attention/skills/memory |
| **Second Thought** | High — training-free parallel reasoning during idle windows | P1 — free performance for NT-ACT + NT-CORE parallelism |
| **Flux Attention** | Medium — layer-level dynamic attention routing | P2 — validate layer-level vs head-level routing for GWT |
| **SparDA** | Medium — predictive KV prefetching | P2 — predictive KB loading for experience-tree |

## Implementation Roadmap

| Phase | Action | Paper |
|-------|--------|-------|
| **Immediate** | Design state-anchor checkpoint system for KB snapshots | MARCH |
| **Week 2** | Prototype parallel reasoning branches during NT-ACT tool execution | Second Thought |
| **Month 1** | Add joint routing controller to GWT (attention + skills + memory precision) | TriRoute |
| **Month 2** | Implement layer-level dynamic attention for domain-specific routing | Flux Attention |
| **Quarter** | Predictive KB prefetching based on task-type inference | SparDA |
