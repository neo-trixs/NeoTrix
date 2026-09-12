# Model Reverse Engineering — Cycle 428

**Date**: 2026-09-12
**Focus**: Recent papers on efficient inference, attention, agent coordination
**Sources**: arXiv (Jul–Sep 2026), ICML 2026, EMNLP 2026
**Cross-referenced against cycles 318–427 for novelty**

---

## Paper 1: CEDAR — Error-Bounded Residual Routing for Efficient Long-Context Attention

**URL**: https://arxiv.org/abs/2609.07237
**Date**: 2026-09-07
**Venue**: arXiv preprint

### Core Contribution
Coarse-to-fine Error-aware Dynamic Attention Routing. Each semantic chunk contributes a cheap key-value summary to a residual attention path; chunks with high estimated approximation error are expanded to exact token attention. Exact and summarized contributions combined in a single softmax normalization — refinement replaces, rather than duplicates, coarse evidence. Derives an output-error bound governed by within-chunk key/value dispersion to allocate variable refinement budget. Residual summaries reduce reconstruction error by >98% relative to hard dropping at equal exact-chunk budgets. ~3× kernel speedup at 128K context.

### Key Insight
> "Refinement replaces, rather than duplicates, coarse evidence."

The critical insight is the residual connection: coarse summaries and exact attention share a single softmax normalization. When a chunk is expanded to exact attention, its summary contribution is replaced (not added), preventing information duplication. The error bound is derived from within-chunk key/value dispersion — chunks with high internal variance need more exact attention.

### Architecture Mapping to NeoTrix

| CEDAR Component | NeoTrix Domain | Pattern |
|-----------------|---------------|---------|
| Coarse KV summaries per chunk | NT-MEMORY | KB hub index as "summary" of experience branches |
| Error-bound refinement allocation | NT-REPAIR | Self-healing: allocate more computation to degraded modules |
| Residual attention (replace, not duplicate) | NT-CORE (GWT) | GWT broadcasts should replace, not add to, existing attention |
| Within-chunk dispersion metric | NT-MEMORY | KB node entropy: measure heterogeneity within experience clusters |

### Absorption Candidates
- **KB Residual Summaries**: Experience-tree hub should maintain "coarse summaries" of branches (like CEDAR's chunk summaries). When a branch is accessed, its summary is replaced by the full content — not duplicated. This prevents information inflation during experience recall.
- **Error-Bound Self-Healing**: NT-REPAIR should estimate "approximation error" for each module (how much the module's output diverges from expected behavior). Modules with high error get more repair computation; low-error modules stay on lightweight monitoring.
- **GWT Residual Broadcast**: When GWT broadcasts a salient signal, it should REPLACE existing attention state for that module, not ADD to it. This prevents attention accumulation where old signals persist alongside new ones.

### Implementation Priority: P1
Enhancement to KB indexing and GWT broadcast mechanism.

---

## Paper 2: FFD — Faster Than Flash: Attention Sparsity for Long-Context Decoding

**URL**: https://arxiv.org/abs/2609.00097
**Date**: 2026-08-31
**Venue**: ICML 2026

### Core Contribution
Faster Flash Decoding (FFD): hardware-algorithm co-design framework that breaks the memory wall in long-context decoding. Integrates selector and computer into a fully fused kernel, replacing external metadata indices with content-aware scanning via low-bit quantization. Top-delta strategy dynamically filters blocks to achieve distribution-adaptive sparsity without global synchronization. Training-free and plug-and-play. Reuses scanning results for computation. Up to 11.6× kernel-level speedup, scaling to 256K context, 2.37× end-to-end throughput improvement.

### Key Insight
> "Distribution-adaptive sparsity without global synchronization."

The key innovation is the top-delta strategy: instead of a fixed sparsity ratio, FFD dynamically filters blocks based on the actual distribution of attention scores. This avoids the global synchronization bottleneck that plagues other sparse attention methods — each block can be filtered independently. Low-bit quantization for content-aware scanning replaces expensive metadata indices.

### Architecture Mapping to NeoTrix

| FFD Component | NeoTrix Domain | Pattern |
|---------------|---------------|---------|
| Top-delta dynamic filtering | NT-CORE (GWT) | GWT salience adapts per-domain, not fixed threshold |
| Fully fused kernel (selector + computer) | NT-ACT | Tool execution combines routing + execution in single pass |
| Low-bit quantization for scanning | NT-MEMORY | KB index compression: low-bit embeddings for fast recall |
| No global synchronization | NT-ACT | Parallel task execution without global coordination barrier |

### Absorption Candidates
- **GWT Top-Delta Routing**: GWT should use distribution-adaptive salience thresholds, not fixed cutoffs. When attention scores are highly concentrated (few modules matter), use aggressive pruning; when distributed (many modules matter), use wider attention. This is the top-delta strategy applied to attention routing.
- **KB Low-Bit Index**: KB hub index should use low-bit quantized embeddings for fast initial scan, then expand to full-precision for the selected candidates. This reduces the O(N) scan cost while maintaining recall quality.
- **NT-ACT Fused Execution**: Tool execution should combine routing (which tool?) and execution (run tool?) in a single kernel pass, avoiding the overhead of separate routing and execution steps.

### Implementation Priority: P0
Direct enhancement to GWT routing efficiency and KB recall performance.

---

## Paper 3: Codebook Agent — Amortized Topology Design for Multi-Agent Systems

**URL**: https://arxiv.org/abs/2609.02264
**Date**: 2026-09-02
**Venue**: arXiv preprint

### Core Contribution
Argues that conditional graph generation for multi-agent topology is misaligned with the problem. Three empirical findings: (1) topologies collapse to ~6 distinct graphs even with codebook capacity 8-64; (2) edge count is negatively correlated with token consumption (r ≈ -0.4) — sparsifying makes inference MORE expensive; (3) message-passing scorer is adjacency-invariant when agents share profiles — cannot rank candidates. Solution: Codebook Agent — vector-quantized autoencoder compresses successful topologies into 16-entry codebook; reward-weighted MLP maps query to code distribution; MLP proxy reranks top candidates in single batched forward pass. No iterative search, no message passing at test time. 84.6 average accuracy (vs 83.0 strongest prior), 2.4ms topology generation, 21.9–33.2% fewer tokens.

### Key Insight
> "Edge count is negatively correlated with token consumption — sparsifying the graph makes inference more expensive."

Counter-intuitive finding: denser topologies (more agent connections) are actually cheaper because they reduce the number of rounds needed to reach consensus. The "optimal" sparse topology wastes tokens on repeated rounds. The real optimization target is not graph sparsity but topology quality — which specific agents communicate, not how many edges exist.

### Architecture Mapping to NeoTrix

| Codebook Agent Component | NeoTrix Domain | Pattern |
|--------------------------|---------------|---------|
| 16-entry topology codebook | NT-CORE | E8 hexagram as topology codebook (64 entries → reusable patterns) |
| Query-to-code distribution mapping | NT-CORE (GWT) | GWT salience as query-to-module routing distribution |
| MLP proxy reranking | NT-MIND (SEAL) | SEAL pipeline: lightweight re-ranker for experience candidates |
| Denser ≠ more expensive | NT-ACT | NT-ACT tool chains: more connections can reduce total cost |

### Absorption Candidates
- **E8 as Topology Codebook**: E8's 64 hexagrams already function as a topology codebook — reusable reasoning patterns for different task types. Codebook Agent's insight validates this: instead of generating new topologies per task, map tasks to existing proven patterns. The 16-entry codebook suggests NeoTrix could compress 64 hexagrams to ~16 "master patterns" without losing coverage.
- **GWT Distribution Mapping**: GWT should map queries to a DISTRIBUTION over modules (like Codebook Agent's query→code distribution), not a single highest-salience module. This enables soft routing where multiple modules contribute proportional to their relevance.
- **Density-Optimized Tool Chains**: NT-ACT tool chains should prefer denser connectivity (more tool-to-tool references) over sparse chains. The "more edges = fewer rounds" insight applies to SEAL pipeline stage transitions.

### Implementation Priority: P1
Enhancement to E8 reasoning patterns and GWT routing.

---

## Paper 4: ProgRouter — Online Progress-Guided Orchestration for Multi-Agent Workflows

**URL**: https://arxiv.org/abs/2608.25992
**Date**: 2026-08-26
**Venue**: EMNLP 2026 (Findings)

### Core Contribution
Online progress-guided routing framework that adaptively selects LLM agents across workflow steps. Multi-view task progress scorer combines coarse workflow outcome regimes with fine-grained signals on subtask completion, progress trends, and workflow state quality. Dual-path task progress predictor estimates progress gain for each candidate routed LLM. Adaptive meta-gating mechanism balances progress gain, task time budgets, and long-term cost efficiency. Makes online step-wise routing decisions — not one-shot query-level decisions like cascade routing.

### Key Insight
> "The right LLM at each step depends on evolving task progress, remaining task difficulty, and cost-efficiency requirements."

Static routing (one model per task type) fails because task difficulty evolves during execution. A step that starts easy may become hard mid-execution. ProgRouter's multi-view scorer tracks progress across dimensions (completion %, trend, quality) to make dynamic routing decisions at each step.

### Architecture Mapping to NeoTrix

| ProgRouter Component | NeoTrix Domain | Pattern |
|----------------------|---------------|---------|
| Multi-view progress scorer | NT-CORE (ConsciousnessTree) | CT health is multi-dimensional (phi, coherence, fatigue) |
| Dual-path progress predictor | NT-MIND (SEAL) | SEAL pipeline predicts next-stage progress from current state |
| Step-wise online routing | NT-CORE (GWT) | GWT re-routes attention per sub-task, not per cycle |
| Cost-efficiency meta-gating | NT-ACT | Resource budget management: cost-aware tool selection |

### Absorption Candidates
- **ConsciousnessTree Multi-View Health**: CT health scoring should use multi-view signals (phi trend, coherence delta, fatigue trajectory) — not just current-state snapshots. The "progress trend" signal catches degrading health before it reaches critical thresholds.
- **GWT Step-Wise Re-Routing**: GWT should re-route attention at each sub-task step within a ConsciousnessTree cycle, not just at cycle boundaries. ProgRouter proves that step-level routing outperforms task-level routing.
- **SEAL Cost-Aware Stage Selection**: SEAL pipeline should consider cost-efficiency when selecting which stage to run next. If a stage has low expected progress gain (high cost, low benefit), skip it. The meta-gating mechanism provides the decision framework.

### Implementation Priority: P0
Direct enhancement to GWT routing granularity and SEAL pipeline efficiency.

---

## Paper 5: ORCH — Organizational Principles for Collective Intelligence in Embodied AI

**URL**: https://arxiv.org/abs/2609.11737
**Date**: 2026-09-10
**Venue**: arXiv preprint

### Core Contribution
Applies human organization theory to multi-agent systems. ORCH (Organizing Roles and Coordination Hierarchies) constructs task-specific hierarchical organizations by combining pooled interdependence (concurrent work) with sequential interdependence (prerequisite-governed work). Tested on 25 wildfire-response missions with up to 50 heterogeneous agents using 8 LLMs. Human-designed ORCH organizations: +63.97% final score, +74.29% execution efficiency. LLM-generated organizations: +43.63% score, +52.53% efficiency. Key: collective performance is NOT monotonically determined by model scale — organization matters more than individual capability.

### Key Insight
> "Collective performance was not monotonically determined by model scale."

Organization structure (how agents are arranged) dominates individual capability (how smart each agent is). A well-organized team of weaker models outperforms a poorly-organized team of stronger models. This is the "orchestration topology dominates model selection" thesis from AdaptOrch, validated in embodied AI.

### Architecture Mapping to NeoTrix

| ORCH Component | NeoTrix Domain | Pattern |
|----------------|---------------|---------|
| Pooled interdependence (concurrent) | NT-ACT | Parallel task execution across independent domains |
| Sequential interdependence (prerequisite) | NT-CORE | ConsciousnessTree growth cycle stages (sequential) |
| Hierarchical organization | NT-CORE | 6-layer architecture (L1→L6 hierarchy) |
| Non-monotonic scale performance | NT-MIND (SEAL) | SEAL optimization: better topology > bigger model |

### Absorption Candidates
- **Domain Organization Topology**: NeoTrix's 7 domains (NT-CORE through NT-FEEL) should be organized based on task-dependent interdependence, not fixed hierarchy. Some tasks need pooled interdependence (all domains contribute in parallel); others need sequential (L1→L2→...→L6). ORCH provides the framework for dynamic topology selection.
- **Model Scale ≠ Performance**: ORCH validates NeoTrix's "small model routing" axiom (A1: Cost-Aware Routing). If organization dominates capability, then using cheaper models with better organization can outperform expensive models with poor organization.
- **Embodied Task Decomposition**: For NT-PHYSICAL tasks, ORCH's pooled+sequential decomposition provides a concrete pattern: concurrent sensor reading (pooled) followed by sequential motor control (sequential).

### Implementation Priority: P1
Framework for dynamic domain organization based on task type.

---

## Cross-Paper Synthesis (Cycle 428)

| Theme | Papers | Unified Insight |
|-------|--------|-----------------|
| **Coarse > Fine** | Flux Attention, Codebook Agent, FFD | Layer-level routing beats head-level; denser topologies beat sparser; block-level filtering beats token-level. NeoTrix should route at domain level, not module level. |
| **Replace > Add** | CEDAR, Declarative Attention | Refinement should replace coarse evidence, not add to it. GWT broadcast should replace attention state, not accumulate it. |
| **Progress > Snapshot** | ProgRouter, ParaTempo | Dynamic routing based on evolving progress beats static routing based on initial assessment. GWT should re-route per sub-task, not per cycle. |
| **Organization > Capability** | ORCH, Codebook Agent | How agents are organized matters more than how capable each agent is. NeoTrix's 6-layer architecture should be dynamically reorganized per task. |
| **Distribution > Point Estimate** | Codebook Agent, ParaTempo | Route to distributions over modules (soft routing), not single highest-salience modules (hard routing). Enables graceful degradation. |

---

## Absorption Priority Matrix

| Priority | Pattern | Source Papers | Target Domain | Expected Impact |
|----------|---------|---------------|---------------|-----------------|
| P0 | Step-wise online routing (not one-shot) | ProgRouter | NT-CORE (GWT) | +15-20% routing accuracy |
| P0 | Top-delta distribution-adaptive sparsity | FFD | NT-CORE (GWT) | 2-3× GWT routing speedup |
| P1 | Error-bound residual summaries | CEDAR | NT-MEMORY (KB) | -50% KB index memory |
| P1 | Topology codebook (pattern reuse) | Codebook Agent | NT-CORE (E8) | 80% faster E8 pattern selection |
| P1 | Pooled + sequential interdependence | ORCH | NT-CORE + NT-ACT | Dynamic domain organization |
| P2 | Replace-not-add attention mechanism | CEDAR, DA | NT-CORE (GWT) | Cleaner attention state |
| P2 | Progress-based early termination | ParaTempo, ProgRouter | NT-MIND (SEAL) | -20% SEAL cycle time |
| P2 | Non-monotonic scale-performance | ORCH | NT-MIND (SEAL) | Cost-aware model routing validation |
