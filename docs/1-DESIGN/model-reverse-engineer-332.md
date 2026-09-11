# Model Reverse Engineering — Cycle 332

**Date**: 2026-09-11
**Focus**: Recent papers on efficient inference, attention mechanisms, agent coordination
**Mapping**: Patterns → NeoTrix 7 domains

---

## Model 1: Sheaf-ADMM — Sheaf-Constrained Multi-Agent Coordination

**Paper**: "Learning Multi-Agent Coordination via Sheaf-ADMM" (ICML 2026)
**URL**: https://arxiv.org/abs/2605.31005
**Authors**: Jeffrey Seely, Bartłomiej Cupiał, Llion Jones (Sakana AI)

### Core Mechanism

Sheaf-ADMM decomposes input into overlapping local views processed by agents with convex subproblems. Coordination occurs through ADMM with constraints specified by a **cellular sheaf** — a topological structure that defines which aspects of neighboring solutions must agree while leaving the rest private.

Three-step coordination cycle:
1. **Local Guess (x-update)**: Each agent solves `argmin f_i(x_i) + (ρ/2)||x_i - z_i^k + u_i^k||²` — local optimization plus consensus penalty
2. **Consensus (z-update)**: Sheaf diffusion — agents communicate only boundary interfaces via restriction maps F_ij x_i = F_ji x_j
3. **Dual Accumulation (u-update)**: `u^{k+1} = u^k + x^{k+1} - z^{k+1}` — persistent memory of disagreement forces compromise

### Key Results

| Task | Sheaf-ADMM | MPNN Baseline | Improvement |
|------|-----------|---------------|-------------|
| Multi-Agent Sudoku | 92.6% | 34.7% | +167% |
| MNIST (canvas-size shift) | 86% | 11% | +682% |
| Maze Pathfinding | Matches baseline | Baseline | 8x smaller channel (5 vs 42 dims) |

### NeoTrix Domain Mapping

| Domain | Pattern Mapping | Application |
|--------|----------------|-------------|
| **NT-CORE** | GWT salience routing = sheaf restriction maps | Domains agree on interface contracts (KB edges) while keeping internal state private. Sheaf Laplacian drives consensus across the 7-domain graph. |
| **NT-ACT** | Cross-domain orchestration = ADMM coordination | DomainBridge implements sheaf-like interface contracts. Primal/consensus/dual variables expose coordination dynamics for ConsciousnessTree monitoring. |
| **NT-GOVERNANCE** | Sheaf constraints = policy enforcement | Restriction maps encode which domain behaviors must align (e.g., NT-SHIELD trust tiers constrain NT-ACT tool execution). |
| **NT-MEMORY** | Consensus state = KB namespace alignment | Overlapping local views parallel KB namespace isolation with cross-namespace references. Dual variables track cross-domain knowledge tension. |

### Absorption Signal

**Strength**: 9/10 — Mathematically rigorous foundation for the exact problem NeoTrix faces (cross-domain coordination without centralization). The sheaf provides a principled alternative to ad-hoc message passing.

**Action**: Implement cellular sheaf structure for the 7-domain interface graph. Each domain is a node, restriction maps define which KB fields must be synchronized. Dual variables track coordination health in ConsciousnessTree.

---

## Model 2: Token Sparse Attention — Reversible Interleaved Sparsity

**Paper**: "Token Sparse Attention: Efficient Long-Context Inference with Interleaved Token Selection" (ICML 2026)
**URL**: https://arxiv.org/abs/2602.03216
**Authors**: Dongwon Jo, Beomseok Kang, Jiwon Song, Jae-Joon Kim

### Core Mechanism

Token Sparse Attention performs **per-head token selection** at each layer, then **scatters attention output back** into the original sequence dimension. This "Compress and then Decompress" design is the key innovation:

1. **Dynamic Token Coverage**: Estimates head-specific sparsity budgets at inference time via normalized drift rank
2. **Per-Head Selection**: Each head independently selects top-k tokens based on its own importance scores
3. **Compression**: Q, K, V are compressed to selected subset → standard attention on reduced space
4. **Decompression**: Output is scattered back to full sequence dimension → next layer sees full context

Critical insight: Token importance **shifts substantially across layers**. Permanently evicting tokens based on early-layer selection can eliminate candidates that become relevant later.

### Key Results

| Metric | Value |
|--------|-------|
| Speedup (128K context) | 3.23x |
| Accuracy degradation | <1% |
| Composability | Compatible with Flash Attention, Minference, X-Attention |
| Design point | New intersection of token selection + sparse attention |

### NeoTrix Domain Mapping

| Domain | Pattern Mapping | Application |
|--------|----------------|-------------|
| **NT-CORE** | GWT attention = reversible token selection | Instead of permanently pruning attention (killing salient signals), maintain full potential but compute only what's currently relevant. Each attention head can specialize for different content types. |
| **NT-MEMORY** | Experience-tree lazy loading = decompression | KB branches are "compressed" (only index loaded), "decompressed" (full content loaded on-demand), and "re-compressed" (evicted back to index). This is exactly Token Sparse Attention's pattern. |
| **NT-IO** | Model routing = per-head selection | Different model heads (providers) select different subsets of the problem. Dynamic token coverage = cost-aware routing that adapts to actual workload. |
| **NT-WORLD** | Perception filtering = dynamic coverage | Sensory events are compressed (selected) per attention head, decompressed for full processing, then re-compressed for storage. Reversible selection preserves perception history. |

### Absorption Signal

**Strength**: 8/10 — The reversible sparsity pattern directly solves a core problem: how to be efficient without losing information that might become relevant later. Composability with existing kernels means zero architectural disruption.

**Action**: Apply interleaved selection pattern to experience-tree branch loading. Instead of permanent branch eviction, maintain full branch index with dynamic loading based on relevance scores that shift across SEAL pipeline stages.

---

## Model 3: Flux Attention — Layer-Level Dynamic FA/SA Routing

**Paper**: "Flux Attention: Context-Aware Hybrid Attention for Efficient LLMs Inference" (Apr 2026)
**URL**: https://arxiv.org/abs/2604.07394
**Authors**: Quantong Qiu, Zhiyi Hong, Yi Yang et al.

### Core Mechanism

Flux Attention addresses the fundamental limitation of hybrid attention: static allocation ratios that can't adapt to variable task demands. It introduces a **lightweight Layer Router** that dynamically routes each layer to Full Attention (FA) or Sparse Attention (SA) based on input context.

Key design decisions:
1. **Layer-level routing** (not head-level): Preserves contiguous memory access for hardware efficiency
2. **Input-dependent**: Router sees actual context, not fixed ratio
3. **Parameter-efficient**: Only 12 hours training on 8xA800 GPUs
4. **FA/SA binary decision**: Simple routing, not complex allocation

### Key Results

| Metric | Value |
|--------|-------|
| Prefill speedup | 2.8x |
| Decode speedup | 2.0x |
| Training cost | 12 hours on 8xA800 |
| Architecture impact | Additive (Layer Router on frozen LLM) |

### NeoTrix Domain Mapping

| Domain | Pattern Mapping | Application |
|--------|----------------|-------------|
| **NT-CORE** | GWT salience = Layer Router | The Layer Router is a direct analog for GWT attention routing: lightweight decision (FA/SA) that determines how much computation to allocate based on actual context demands. |
| **NT-IO** | Model routing (Axiom A1) | Cost-aware routing: cheap model for simple tasks, expensive model for hard tasks. Layer Router's binary decision = which provider to route to. |
| **NT-ACT** | Task allocation | Simple tasks get SA (fast, sparse), complex tasks get FA (full, expensive). The router learns to distinguish task types from context. |
| **NT-MEMORY** | Cache tier routing | Hot data gets full attention (FA), cold data gets sparse attention (SA). Layer Router = automatic tier classification. |

### Absorption Signal

**Strength**: 7/10 — The layer-level routing is a clean, practical solution for dynamic attention allocation. The binary FA/SA decision is simple enough for production deployment. However, it's specific to attention mechanisms rather than a general coordination pattern.

**Action**: Map Layer Router pattern to GWT salience scoring. Instead of binary FA/SA, use multi-level routing (full/partial/sparse) with cost weights per domain. The lightweight router validates that attention routing overhead must be negligible.

---

## Model 4: Flux Attention + Token Sparse Attention — Complementary Sparsity Strategies

**Combined Analysis** of Papers 2 and 3 above.

### Key Insight: Orthogonal Sparsity Dimensions

Token Sparse Attention and Flux Attention operate on **orthogonal dimensions** of the sparsity problem:

| Dimension | Token Sparse Attention | Flux Attention |
|-----------|----------------------|----------------|
| **What to sparsify** | Which tokens to attend to | Which layers need full attention |
| **Granularity** | Per-head, per-layer | Per-layer |
| **Mechanism** | Dynamic token selection + decompression | Binary FA/SA routing |
| **Composability** | Compatible with Flash Attention, Minference | Additive to frozen LLM |
| **Training cost** | Zero (training-free) | 12 hours (parameter-efficient) |

They are **composable**: Flux Attention routes layers to FA/SA, then Token Sparse Attention further sparsifies within FA layers. This creates a two-level sparsity hierarchy:
1. **Layer level**: FA or SA (Flux)
2. **Token level**: Which tokens within FA layers (Token Sparse)

### NeoTrix Integration

This two-level hierarchy maps directly to our attention architecture:

| Level | NeoTrix Component | Analog |
|-------|-------------------|--------|
| Layer-level routing | GWT attention routing | Which domains get full broadcast |
| Token-level selection | Per-domain salience scoring | Which information within a domain gets attention |

The composability validates our architectural principle: each layer handles one aspect of attention optimization, and layers compose without interference.

---

## Model 5: Internet of Agentic AI (IoAI) — Coordination at Scale

**Paper**: "The Internet of Agentic AI: Communication, Coordination, and Collective Intelligence at Scale" (Jun 2026)
**URL**: https://arxiv.org/abs/2606.12835
**Author**: Quanyan Zhu

### Core Mechanism

IoAI synthesizes foundations from single-agent agentic AI, multi-agent systems, distributed computing, communication networks, game theory, and security engineering to characterize architectures for scalable agent ecosystems.

Key research challenges identified:
1. **Controlled emergence** — steering agent collectives without centralizing
2. **Semantic interoperability** — agents discovering and communicating across heterogeneous systems
3. **Secure identity** — verifiable agent identities in open ecosystems
4. **Incentive-compatible coordination** — agents cooperating without trust assumptions
5. **Resource-aware orchestration** — efficient allocation across cloud/edge/device
6. **Governance for large-scale networks** — auditability at ecosystem level

### NeoTrix Domain Mapping

| Domain | IoAI Challenge | NeoTrix Solution |
|--------|---------------|------------------|
| **NT-CORE** | Controlled emergence | E8 hexagram provides structured emergence — 64 states as controlled vocabulary for reasoning patterns |
| **NT-ACT** | Semantic interoperability | CapabilityInput/CapabilityOutput type contracts + UCN namespace unification |
| **NT-SHIELD** | Secure identity | Egress Privacy Guard trust tiers (Trusted/Contracted/Untrusted) |
| **NT-GOVERNANCE** | Incentive-compatible coordination | ConsciousnessTree health monitoring + Dark Forest survival axiom |
| **NT-MEMORY** | Resource-aware orchestration | KB namespace isolation + experience-tree lazy loading |
| **NT-IO** | Governance for networks | MCP/A2A/UTCP protocol stack with audit trails |

### Absorption Signal

**Strength**: 6/10 — Provides a comprehensive taxonomy of coordination challenges but is more survey than mechanism. The challenge list is useful for validating NeoTrix's architectural coverage.

**Action**: Use IoAI's 6 challenges as a self-audit checklist against NeoTrix architecture. Identify any gaps in our current implementation. The "controlled emergence" challenge is particularly relevant to ConsciousnessTree's role in steering evolution without centralizing control.

---

## Cross-Paper Synthesis: 5 Meta-Patterns

### Meta-Pattern 1: Sparsity Must Be Reversible
- Token Sparse Attention: decompress after compression
- TCA-Attention: restore full sequence after selection
- Flux Attention: FA layers available even when SA is chosen
- **NeoTrix implication**: Experience-tree branches must be evictable but restorable. KB nodes must support lazy loading with full restoration.

### Meta-Pattern 2: Coordination Without Centralization
- Sheaf-ADMM: agents agree only on boundary interfaces
- IoAI: controlled emergence via structured interaction protocols
- **NeoTrix implication**: Domains coordinate via KB edges (sheaf restriction maps), not central control. ConsciousnessTree monitors without dictating.

### Meta-Pattern 3: Lightweight Routers, Heavyweight Execution
- Flux Attention: Layer Router adds <1% overhead
- Token Sparse Attention: training-free calibration
- TCA-Attention: single forward pass for budgets
- **NeoTrix implication**: GWT salience scoring must be negligible overhead. Model routing decisions should use lightweight heuristics, not expensive inference.

### Meta-Pattern 4: Deterministic + AI Hybrid
- Sheaf-ADMM: convex optimization (deterministic) + neural encoders (learned)
- Flux Attention: frozen LLM + lightweight router
- Token Sparse Attention: deterministic selection + standard attention
- **NeoTrix implication**: SEAL pipeline stages should be deterministic where possible, reserving AI judgment for genuinely hard decisions. The "deterministic backbone, AI overlay" pattern is production-proven.

### Meta-Pattern 5: Composability Over Monolith
- Token Sparse Attention composable with Flash Attention, Minference
- Flux Attention additive to frozen LLM
- Sheaf-ADMM adaptable sheaf structure
- **NeoTrix implication**: Each NeoTrix component should be independently usable. Trait-based architecture (CognitionLayer, PerceptionLayer, etc.) already supports this. Maintain composability as a hard constraint.

---

## Absorption Priority Matrix

| Priority | Paper | Pattern | Domain | Effort | Impact |
|----------|-------|---------|--------|--------|--------|
| P0 | Sheaf-ADMM | Cellular sheaf coordination | NT-CORE | High | Foundation for cross-domain coordination |
| P0 | Token Sparse Attention | Reversible interleaved sparsity | NT-MEMORY | Medium | Experience-tree branch loading optimization |
| P1 | Flux Attention | Layer-level dynamic routing | NT-CORE | Low | GWT salience routing refinement |
| P1 | IoAI | Coordination taxonomy | All | Low | Self-audit checklist for architectural coverage |
| P2 | TCA-Attention | Training-free calibration | NT-CORE | Low | Drop-in inference acceleration option |
