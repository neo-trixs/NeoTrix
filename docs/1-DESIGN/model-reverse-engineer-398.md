# Model Reverse Engineering — Cycle 398 (2026-09-12)

## Scope
5 recent papers on efficient inference, memory routing, and agent coordination. Each mapped to NeoTrix 7-domain architecture with actionable integration patterns.

---

## M1: MemRouter — Memory-as-Embedding Routing for Long-Term Conversational Agents

### Paper
- **URL**: https://arxiv.org/abs/2605.00356
- **Date**: May 2026
- **Authors**: Tianyu Hu, Weikai Lin, Weizhi Zhang, Jing Ma, Song Wang
- **Venue**: arXiv cs.CL / cs.AI

### Core Mechanism
Write-side memory router that decouples memory admission from answer generation. Instead of using LLM at every turn to decide "should I store this?", trains a lightweight 12M-parameter classifier:

```
Turn + Recent Context → Frozen LLM Backbone → Embedding → Classification Head → Store/Don't Store
```

**Key results**:
- Outperforms LLM-based memory manager on every question category (F1 52.0 vs 45.6)
- p50 latency: 58ms vs 970ms (16.7x faster)
- Learned admission improves mean F1 by +10.3 over random storage
- Category-specific prompting adds +5.2 over generic prompt

### Architecture Insight
The critical insight: **memory admission is a supervised classification problem, not a generative one**. By freezing the LLM backbone and training only a lightweight head, they get:
1. LLM-quality representations without LLM-quality latency
2. Deterministic, auditable admission decisions
3. Separation of concerns: memory management ≠ answer generation

### NeoTrix Domain Mapping

| Domain | Mapping | Priority |
|--------|---------|----------|
| **NT-MEMORY** | Experience-tree branch admission. Replace heuristic `experience absorb` with learned router. 12M params fits NeoTrix's local-first principle. | P0 |
| **NT-CORE** | GWT salience filtering. The classification head is a saliency scorer — extends GWT attention routing to memory-space. | P1 |
| **NT-MIND** | SEAL pipeline distillation stage. Learn what to distill vs what to keep raw. | P2 |

### Integration Pattern: Learned Admission Gate
```
Current: experience-tree absorb → heuristic rules → KB write
Proposed: experience-tree absorb → MemRouter-style classifier → KB write
```

The 12M-param classifier could be trained on NeoTrix's own absorption history (accepted/rejected experiences as labels). Self-improving memory gate.

### Risk: Low
- Lightweight model, no training infrastructure needed
- Classification task, not generation — deterministic output
- Compatible with existing KB write path

---

## M2: Gated-Memory Routing for Efficient Multi-Agent Collaboration

### Paper
- **URL**: https://arxiv.org/abs/2609.00237
- **Date**: August 2026
- **Authors**: Rakibul Hasan Rajib, Mengxing Zheng, Qian Lou
- **Venue**: EMNLP 2026 (Main Conference)

### Core Mechanism
Multi-agent orchestration conditioned on query + learned execution memory:

```
Query + Execution Memory → Memory Write Gate → Retrieval Gate → Agent Selection → Adaptive Halting
```

Three components:
1. **Memory Write Gate**: Commits only non-redundant reasoning steps (deduplication at write time)
2. **Retrieval Gate**: Supplies each agent a compact, relevant subset of memory
3. **Adaptive Halting Controller**: Stops execution when memory contains sufficient evidence

**Key results**:
- Best average accuracy across 5 benchmarks (exceeds strongest baseline by +2.44 points)
- HumanEval inference cost reduced by 31.9%
- Reduces execution-history overload that inflates cost in later decisions

### Architecture Insight
The problem they solve: in multi-agent systems, later decisions must process every prior step, including redundant ones. Their solution: **learned compression at write time + learned retrieval at read time**. The "sufficient evidence" halting is particularly novel — know when to stop, not just what to do next.

### NeoTrix Domain Mapping

| Domain | Mapping | Priority |
|--------|---------|----------|
| **NT-ACT** | NT-ACT orchestration memory. When multiple agents collaborate, only store non-redundant reasoning. Reduce context bloat across parallel task execution. | P0 |
| **NT-CORE** | GWT attention budget. The halting controller is a GWT "stop broadcasting" signal — conserve attention when sufficient information has been disseminated. | P1 |
| **NT-MIND** | SEAL pipeline phase transitions. Halting controller maps to "sufficient evidence for evolution" detection. | P2 |
| **NT-MEMORY** | KB write deduplication. Memory Write Gate as KB ingestion filter — prevent redundant knowledge entries. | P1 |

### Integration Pattern: Adaptive Execution Budget
```
Current: NT-ACT parallel tasks → fixed iteration count
Proposed: NT-ACT parallel tasks → Gated-Memory → learned halting when evidence sufficient
```

This directly addresses NeoTrix's dual-specialization mode switching. The halting controller could learn when CORE+WORLD (acquisition) mode has gathered enough signal to switch to CORE+MIND (evolution) mode.

### Risk: Medium
- Requires training on execution traces
- Adaptive halting adds complexity to orchestration
- Must validate that stopping early doesn't lose critical information

---

## M3: Flux Attention — Context-Aware Hybrid Attention for Efficient LLM Inference

### Paper
- **URL**: https://arxiv.org/abs/2604.07394
- **Date**: April 2026
- **Authors**: Quantong Qiu, Zhiyi Hong, Yi Yang, Haitian Wang, Kebin Liu, Qingqing Dang, Juntao Li, Min Zhang
- **Venue**: arXiv cs.LG / cs.CL

### Core Mechanism
Layer-level dynamic attention routing: a lightweight Layer Router predicts whether each layer needs Full Attention (FA) or Sparse Attention (SA) based on input context.

```
Input → Layer Router (per-layer decision) → FA or SA per layer → Output
```

**Key results**:
- 2.8x prefill speedup, 2.0x decode speedup
- Only 12 hours training on 8x A800 GPUs
- Preserves contiguous memory access (hardware-friendly)
- Outperforms static FA/SA ratio allocation

### Architecture Insight
Previous hybrid attention methods use static ratios (e.g., first 25% of layers use FA, rest use SA). Flux Attention's insight: **different inputs need different attention patterns per layer**. A lightweight router learns to predict this. The layer-level granularity (not head-level) avoids computational load imbalance and synchronization overhead.

### NeoTrix Domain Mapping

| Domain | Mapping | Priority |
|--------|---------|----------|
| **NT-CORE** | GWT attention allocation. Layer Router pattern = GWT salience scoring per attention head. "Which heads need full broadcast vs sparse attention?" | P1 |
| **NT-MEMORY** | KV cache optimization. Layer-level FA/SA routing reduces KV cache pressure on long-context sessions. Aligns with Axiom A2. | P1 |
| **NT-PHYSICAL** | Hardware-aware scheduling. Contiguous memory access pattern maps to NeoTrix's physical layer resource management. | P2 |

### Integration Pattern: GWT Layer Router
```
Current: GWT broadcasts salient info across all modules uniformly
Proposed: GWT Layer Router → per-module attention budget → FA for critical paths, SA for routine
```

This is a direct architectural isomorphism: GWT's attention routing problem is mathematically similar to Flux Attention's layer routing problem. The lightweight router approach could replace GWT's current salience scoring.

### Risk: Low
- 12-hour training is feasible
- Layer-level granularity is hardware-friendly
- Drop-in replacement for current attention patterns

---

## M4: ODAR — Principled Adaptive Routing for LLM Reasoning via Active Inference

### Paper
- **URL**: https://arxiv.org/abs/2602.23681
- **Date**: February 2026
- **Authors**: Siyuan Ma, Bo Gao, Xiaojun Jia, Simeng Qin, Tianlin Li, Ke Ma, Xiaoshuang Jia, Wenqi Ren, Yang Liu
- **Venue**: arXiv cs.AI

### Core Mechanism
Adaptive routing between Fast Agent (heuristic) and Slow Agent (deliberative) using amortized active inference:

```
Query → Difficulty Estimator (Active Inference) → Route to Fast or Slow → Free-Energy Fusion → Answer
```

Two key innovations:
1. **Difficulty Estimator**: Uses amortized active inference to predict query difficulty
2. **Free-Energy Fusion**: Minimizes variational free energy, balancing log-likelihood with epistemic uncertainty (varentropy) instead of ad-hoc voting

**Key results**:
- 98.2% on MATH, 54.8% on HLE (Humanity's Last Exam)
- 82% cost reduction vs homogeneous sampling
- Surpasses all fixed best-of-N and self-consistency baselines
- Fully reproducible on open-source stack (Llama 4 + DeepSeek)

### Architecture Insight
The paradigm shift: **thinking-optimal scaling requires adaptive resource allocation, not brute-force compute**. Their free-energy fusion is principled — it's not just "pick the majority vote" but "pick the answer that minimizes expected surprise across all candidates." The varentropy term penalizes high-variance candidate sets.

### NeoTrix Domain Mapping

| Domain | Mapping | Priority |
|--------|---------|----------|
| **NT-CORE** | GWT Fast/Slow Agent routing. Directly maps to dual-specialization Weapon Set switching. Fast Agent = routine tasks, Slow Agent = deep reasoning. | P0 |
| **NT-IO** | Provider cost optimization. Difficulty estimation enables A1 (Cost-Aware Routing) — route hard queries to expensive models, easy to cheap. | P0 |
| **NT-MIND** | SEAL pipeline effort allocation. When to run full evolution cycle vs quick distillation. | P1 |
| **NT-FEEL** | Free-energy minimization aligns with emotional regulation (surprise minimization = anxiety reduction in predictive processing theory). | P3 |

### Integration Pattern: Dual-Pathway with Free-Energy Scoring
```
Current: NT-CORE → single processing path → GWT broadcast
Proposed: NT-CORE → Difficulty Estimator → Fast Path (heuristic) OR Slow Path (deliberative) → Free-Energy Fusion
```

The varentropy-based fusion could replace GWT's current resonance scoring. Instead of "how resonant is this signal?", ask "how much does this answer reduce my uncertainty?"

### Risk: Medium
- Active inference framework adds mathematical complexity
- Requires calibration of difficulty estimator on NeoTrix's task distribution
- Free-energy fusion needs careful tuning of the uncertainty weight

---

## M5: MKA — Memory-Keyed Attention for Efficient Long-Context Reasoning

### Paper
- **URL**: https://arxiv.org/abs/2603.20586
- **Date**: March 2026 (revised March 2026)
- **Authors**: Dong Liu, Yanxuan Yu, Ben Lengerich, Ying Nian Wu
- **Venue**: ACM Computing Frontiers 2026 (Oral) + ICML 2025 Long Context Workshop

### Core Mechanism
Hierarchical attention with multi-level KV caches (local, session, long-term) and learned routing:

```
Query → Route across local/session/long-term KV caches → Attend → Output
```

**FastMKA variant**: Broadcast-routed, fuses memory sources before attention computation.

**Key results**:
- Comparable perplexity to MLA (Multi-Latent Attention)
- 5x faster training throughput than MLA
- 1.8x lower evaluation latency than MLA
- Extensible framework — easy to add new memory tiers

### Architecture Insight
The key contribution is the **hierarchical KV cache with learned routing**. Instead of one massive KV cache, split into tiers:
- **Local** (current layer's working set)
- **Session** (recent conversation context)
- **Long-term** (compressed historical context)

The router learns which tier to attend for each query, enabling O(L) complexity per tier while maintaining full-context awareness.

### NeoTrix Domain Mapping

| Domain | Mapping | Priority |
|--------|---------|----------|
| **NT-MEMORY** | KB tiered storage. Local (active working set), Session (current task context), Long-term (compressed historical). Maps to KB namespace layering. | P0 |
| **NT-CORE** | GWT tiered attention. Route attention across consciousness layers: L1 (action), L3 (embodiment), L5 (cognition), L6 (meta). | P1 |
| **NT-IO** | Provider context window optimization. Route requests to providers based on context tier requirements. | P1 |
| **NT-NEXUS** | Cross-session memory. Long-term KV cache = NT-NEXUS session bridge. | P2 |

### Integration Pattern: Tiered KB with Learned Routing
```
Current: NT-MEMORY → flat KB queries
Proposed: NT-MEMORY → 3-tier cache (local/session/long-term) → learned router → tiered attention
```

This directly addresses the Axiom A2 (Context as Scarce Resource). The hierarchical approach means NeoTrix never loads the full KB into context — only the relevant tier.

### Risk: Low
- Hierarchical caching is well-understood engineering
- Learned routing can start as heuristic and be upgraded
- Compatible with existing KB architecture

---

## Cross-Paper Synthesis

### Convergent Pattern: Adaptive Routing Everywhere
All 5 papers share one meta-pattern: **replace static allocation with learned, context-aware routing**.

| Paper | What's Routed | How |
|-------|---------------|-----|
| MemRouter | Memory admission | Lightweight classifier |
| Gated-Memory | Agent selection + halting | Write/Retrieval gates |
| Flux Attention | Full vs Sparse attention | Layer Router |
| ODAR | Fast vs Slow reasoning | Active Inference difficulty estimator |
| MKA | KV cache tier access | Learned hierarchical router |

### NeoTrix Integration Priority Matrix

| Integration | Papers | Target Domain | Effort | Impact |
|-------------|--------|---------------|--------|--------|
| Learned Memory Admission | M1, M2 | NT-MEMORY | Low | High |
| Dual-Pathway Reasoning | M4 | NT-CORE | Medium | High |
| Tiered KB Routing | M5 | NT-MEMORY | Low | High |
| GWT Layer Router | M3 | NT-CORE | Medium | Medium |
| Adaptive Execution Halting | M2 | NT-ACT | Medium | Medium |

### Axiom Reinforcement
- **A1 (Cost-Aware Routing)**: M4 (ODAR) achieves 82% cost reduction via difficulty-based routing. M3 (Flux Attention) achieves 2.8x speedup via layer routing.
- **A2 (Context as Scarce Resource)**: M5 (MKA) hierarchical KV cache directly solves context bottleneck. M1 (MemRouter) reduces memory management latency 16.7x.
- **A3 (Skill as Production Template)**: All 5 papers treat their routing mechanism as a reusable component, not a one-off solution.

### Actionable Next Steps
1. **Immediate** (cycle 399): Prototype learned memory admission gate using MemRouter pattern on experience-tree
2. **Short-term** (cycle 400): Implement tiered KB routing (local/session/long-term) following MKA
3. **Medium-term** (cycle 401-405): Build dual-pathway reasoning (ODAR Fast/Slow) into NT-CORE consciousness loop

---

*Generated: 2026-09-12 | Cycle: 398 | Agent: opencode/mimo-v2.5-free*
