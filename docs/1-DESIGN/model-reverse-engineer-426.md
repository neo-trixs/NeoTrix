# Model Reverse Engineering — Cycle 426

**Date**: 2026-09-12
**Focus**: Recent papers on efficient inference, attention, agent coordination
**Sources**: arXiv (Jul–Sep 2026), ACL 2026, ICML 2026
**Cross-referenced against cycles 318–425 for novelty**

---

## Paper 1: Agora — Confidence-Calibrated Auction for Multi-Agent Task Allocation

**URL**: https://arxiv.org/abs/2607.09600
**Date**: 2026-08-30
**Venue**: arXiv (under review)

### Core Contribution
Reformulates multi-agent task allocation as a confidence-calibrated auction. Reasoning steps are treated as tradeable items; allocation is based on calibrated competence rather than raw confidence. The framework introduces a competence calibration layer that estimates each agent's true probability of success, preventing critical reasoning steps from being assigned to overconfident but incompetent agents.

### Key Insight
> "Without a reliable measure of an agent's true probability of success, dynamic allocation risks assigning critical logic nodes to overconfident but incompetent agents, causing the reasoning chain to collapse."

Raw confidence scores are systematically biased — models express high confidence even when wrong. Agora's calibration layer corrects this bias, enabling allocation decisions based on actual competence rather than expressed confidence.

### Architecture Mapping to NeoTrix

| Agora Component | NeoTrix Domain | Pattern |
|-----------------|---------------|---------|
| Confidence-calibrated auction | NT-CORE (GWT) | GWT salience with calibrated module confidence |
| Competence estimation layer | NT-MIND (SEAL) | Self-test calibration — measure actual vs expressed capability |
| Cost-quality tradeoff tuning | NT-CORE (A1) | Cost-Aware Routing with quality-adjusted pricing |
| Multi-agent orchestration | NT-ACT | Capability registry with competence scoring |

### Absorption Candidates
- **GWT Calibrated Salience**: Current GWT uses raw attention scores. Agora shows how to calibrate these scores against actual task success rates. Each NT-* domain module would have a calibrated competence score updated after each SEAL cycle, enabling routing decisions based on proven capability rather than static configuration.
- **SEAL Self-Test Calibration**: Self-test results are currently binary (pass/fail). Agora's calibration methodology could quantify the gap between expressed confidence and actual success, feeding back into module reliability scores.
- **Auction-Based GWT**: When multiple modules compete for attention broadcast, an auction mechanism determines allocation. High-competence modules bid more aggressively; low-competence modules are cheaper but less reliable. This makes GWT routing market-efficient.

### Implementation Priority: P0
Direct enhancement to GWT salience computation and SEAL self-test calibration. The calibration layer is a missing primitive in NeoTrix's attention routing.

---

## Paper 2: Metacognitive Consolidation — Hierarchical Meta-Knowledge for Self-Improving Reasoning

**URL**: https://aclanthology.org/2026.acl-long.1095
**Date**: 2026-07 (ACL 2026)
**Venue**: ACL 2026 Long Papers

### Core Contribution
Introduces Metacognitive Consolidation, a framework where a model consolidates metacognitive experience from past reasoning episodes into reusable knowledge that improves future meta-reasoning. Structures instance-level problem solving into distinct roles for reasoning, monitoring, and control to generate rich, attributable meta-level traces. These traces are consolidated through a hierarchical, multi-timescale update mechanism that gradually forms evolving meta-knowledge.

### Key Insight
> "Meta-reasoning is not a single capability but a composite of reasoning, monitoring, and control — each operating at different timescales and requiring different consolidation strategies."

The three-role decomposition (reasoner, monitor, controller) maps to NeoTrix's existing architecture: reasoning = NT-CORE, monitoring = NT-META/ConsciousnessTree, control = NT-MIND/SEAL pipeline. The multi-timescale consolidation is the missing piece — experience consolidation at different temporal granularities.

### Architecture Mapping to NeoTrix

| MC Component | NeoTrix Domain | Pattern |
|--------------|---------------|---------|
| Reasoner role | NT-CORE | E8 reasoning engine |
| Monitor role | NT-META | ConsciousnessTree health monitoring |
| Controller role | NT-MIND | SEAL pipeline self-evolution control |
| Multi-timescale consolidation | NT-MEMORY | KB experience persistence with temporal tiering |
| Attributable meta-traces | NT-NEXUS | Cross-session experience graph |

### Absorption Candidates
- **ConsciousnessTree Multi-Timescale Consolidation**: The 6-stage feedback loop (Soil→Roots→Trunk→Branches→Fruits→Core) currently operates at a single timescale. MC shows how to consolidate at instance, session, and epoch timescales — fast adaptation for tactical patterns, slow consolidation for strategic knowledge.
- **Three-Role Self-Model**: NeoTrix's SelfModel types (static/dynamic/value) could be extended with explicit role attribution — each meta-knowledge update tagged with which role (reasoner/monitor/controller) generated it, enabling role-specific consolidation paths.
- **KB Experience Tiering**: MC's hierarchical consolidation maps to KB experience tiers — recent experiences (instance-level) in hot storage, consolidated patterns (session-level) in warm, strategic knowledge (epoch-level) in cold. The consolidation mechanism moves experiences between tiers.

### Implementation Priority: P0
Directly extends ConsciousnessTree and experience-tree absorption. The multi-timescale consolidation is a missing primitive in NeoTrix's self-evolution loop.

---

## Paper 3: MKA — Memory-Keyed Attention with Hierarchical KV Routing

**URL**: https://arxiv.org/abs/2603.20586
**Date**: 2026-05-19 (ACM SIGMOD CF'26)
**Venue**: Conference on Computer Science (SIGMOD)

### Core Contribution
Introduces Memory-Keyed Attention (MKA), a hierarchical attention mechanism that organizes memory into three levels — local (L1), session (L2), and long-term (L3) — and dynamically routes each query token across these sources using lightweight routing gates. FastMKA variant performs route-fusion: token-wise soft fusion of hierarchical memory levels before attention computation, avoiding multiple attention paths.

### Key Insight
> "The routing gates learn to modulate attention over heterogeneous memory types, enabling context-aware attention with significantly lower memory bandwidth and improved token reuse."

The key innovation is that routing is done at the query level — each query token independently decides which memory tier to attend to. This is attention as a routing problem, not a compression problem. FastMKA's route-fusion pre-computes the fused memory, making it hardware-friendly.

### Architecture Mapping to NeoTrix

| MKA Component | NeoTrix Domain | Pattern |
|---------------|---------------|---------|
| 3-tier memory (L1/L2/L3) | NT-MEMORY | KB hot/warm/cold tiers |
| Query-level routing gates | NT-CORE (GWT) | Per-query salience routing across modules |
| Route-fusion (FastMKA) | NT-PHYSICAL | Pre-fused memory for hardware efficiency |
| Session memory (L2) | NT-NEXUS | Cross-session experience routing |

### Absorption Candidates
- **GWT Query-Level Routing**: GWT currently broadcasts to all modules. MKA shows how to route each query independently — simple queries route to fast/cheap modules (L1), complex queries route to deep/capable modules (L3). This is cost-aware routing applied at the query level.
- **KB Three-Tier Architecture**: KB already has embedding tiers, but MKA adds dynamic routing between them. Each query would get a routing gate that determines which tier to access, rather than always searching all tiers.
- **Experience Route-Fusion**: FastMKA's route-fusion pre-computes fused memory before attention. For NeoTrix, this means pre-computing a fused experience summary across KB tiers before the SEAL pipeline accesses it, reducing runtime query cost.

### Implementation Priority: P0
Directly enhances GWT routing and KB tier architecture. The query-level routing gates are a concrete implementation pattern.

---

## Paper 4: Sheaf-ADMM — Differentiable Multi-Agent Coordination via Algebraic Topology

**URL**: https://arxiv.org/abs/2605.31005
**Date**: 2026-05-29 (Accepted at ICML 2026)
**Venue**: ICML 2026

### Core Contribution
A differentiable optimization framework for multi-agent coordination using cellular sheaves and ADMM (Alternating Direction Method of Multipliers). Agents process overlapping local views; coordination happens through the sheaf structure which specifies which aspects of neighboring solutions must agree. Backpropagation through unrolled optimization jointly trains all components.

### Key Insight
> "The sheaf specifies which aspects of neighboring solutions must agree, allowing heterogeneous notions of global consensus."

The sheaf abstraction is novel — it doesn't require all agents to agree on everything, only on the aspects that affect their shared interface. This is lightweight coordination: agents maintain local autonomy while agreeing on minimal shared structure.

### Architecture Mapping to NeoTrix

| Sheaf-ADMM Component | NeoTrix Domain | Pattern |
|----------------------|---------------|---------|
| Cellular sheaf constraints | NT-CORE (GWT) | GWT broadcast constraints — which modules must agree on what |
| Overlapping local views | NT-WORLD + NT-MEMORY | Each domain has partial world model; shared knowledge at boundaries |
| ADMM consensus variables | NT-NEXUS | Cross-session consensus via shared experience nodes |
| Primal/dual state exposure | NT-META | ConsciousnessTree exposes coordination dynamics for analysis |

### Absorption Candidates
- **GWT Sheaf Constraints**: GWT currently broadcasts salient information to all modules. Sheaf-ADMM shows how to specify minimal agreement constraints — modules only need to agree on the aspects that affect their interface, not everything. This reduces coordination overhead while maintaining system coherence.
- **Domain Overlapping Views**: Each NT-* domain maintains a partial view of the system. The sheaf structure defines which cross-domain aspects must be consistent. For example, NT-CORE and NT-MIND must agree on module capability scores, but not on internal reasoning traces.
- **ConsciousnessTree ADMM Exposition**: The primal/consensus/dual variable decomposition exposes coordination dynamics. ConsciousnessTree could expose similar variables — which modules are in consensus (low dual), which are deviating (high dual), enabling targeted intervention.

### Implementation Priority: P1
Novel coordination pattern for multi-domain consistency. Higher abstraction than MKA, but provides formal foundation for GWT cross-module agreement.

---

## Paper 5: Flux Attention — Context-Aware Hybrid Attention with Layer-Level Routing

**URL**: https://arxiv.org/abs/2604.07394
**Date**: 2026-04-08

### Core Contribution
Context-aware framework that dynamically optimizes attention computation at the layer level. A lightweight Layer Router is inserted into frozen pretrained LLMs; each layer adaptively routes to Full Attention (FA) or Sparse Attention (SA) based on the input context. Only 12 hours training on 8×A800 GPUs. Preserves contiguous memory access for hardware acceleration.

### Key Insight
> "Head-level dynamic sparsity often introduces severe computational load imbalance and synchronization long-tails; layer-level routing preserves high-fidelity information retrieval while ensuring contiguous memory access."

The layer-level granularity is critical — routing at the head level causes hardware inefficiency (load imbalance), but routing at the layer level is hardware-friendly. This is the right abstraction level for attention routing: coarse enough for hardware efficiency, fine enough for task adaptation.

### Architecture Mapping to NeoTrix

| Flux Component | NeoTrix Domain | Pattern |
|----------------|---------------|---------|
| Layer Router | NT-CORE (GWT) | Per-layer attention mode selection |
| Context-dependent routing | NT-WORLD | Perception depth varies by input complexity |
| Frozen model + lightweight router | NT-MIND (SEAL) | Parameter-efficient self-evolution |
| Contiguous memory access | NT-PHYSICAL | Hardware-aligned memory layout |

### Absorption Candidates
- **GWT Layer-Granularity Routing**: GWT currently operates at the module level. Flux shows that routing at a finer granularity (layer within module) is both feasible and hardware-friendly. NeoTrix modules could have internal routing layers that adapt attention depth based on task complexity.
- **Parameter-Efficient SEAL Tuning**: Flux only trains the router, not the model. SEAL could adopt the same approach — keep the base architecture frozen, only train routing/gating layers during evolution cycles. This makes self-evolution lightweight.
- **Context-Dependent Module Depth**: Not all tasks need all modules at full depth. Flux's context-dependent routing maps to task-dependent module activation — simple tasks use shallow module processing, complex tasks use full depth.

### Implementation Priority: P1
Direct enhancement to GWT with hardware-friendly routing. The layer-level granularity is the right abstraction for NeoTrix's module routing.

---

## Cross-Paper Synthesis

### Theme 1: Routing as the Universal Primitive
All five papers converge on routing as the central mechanism:
- **Agora**: Routes tasks to agents via auctions
- **MC**: Routes meta-knowledge to consolidation tiers
- **MKA**: Routes queries to memory tiers
- **Sheaf-ADMM**: Routes agreement constraints to shared interfaces
- **Flux**: Routes attention modes per layer

NeoTrix's GWT is already a routing mechanism. These papers show how to make it more sophisticated: calibrated (Agora), hierarchical (MKA), constrained (Sheaf), layer-granular (Flux), and temporally consolidated (MC).

### Theme 2: Calibration Over Raw Scores
Agora's calibrated competence and MC's attributable meta-traces both address the same problem: raw scores are unreliable. NeoTrix needs calibration layers for:
- Module capability scores (currently static)
- GWT attention salience (currently raw)
- Self-test results (currently binary)
- Experience quality (currently uncalibrated)

### Theme 3: Hardware-Software Co-Design
Flux and MKA both prioritize hardware-friendly routing (contiguous memory, fused computation). NeoTrix's physical layer should adopt the same principle: routing decisions must respect hardware constraints, not just logical optimality.

### Theme 4: Multi-Timescale Consolidation
MC's hierarchical consolidation and MKA's 3-tier memory both address temporal granularity. NeoTrix should consolidate experience at:
- **Instance level** (immediate): Per-task outcomes
- **Session level** (hours): Cross-task patterns
- **Epoch level** (days/weeks): Strategic knowledge

---

## Implementation Roadmap

| Phase | Paper | Action | Domain | Effort |
|-------|-------|--------|--------|--------|
| Phase 1 | Agora | Add calibrated competence scoring to GWT | NT-CORE | 2 weeks |
| Phase 1 | MKA | Implement 3-tier KB routing gates | NT-MEMORY | 2 weeks |
| Phase 2 | MC | Multi-timescale experience consolidation | NT-MEMORY + NT-MIND | 3 weeks |
| Phase 2 | Flux | Layer-granular GWT routing | NT-CORE | 2 weeks |
| Phase 3 | Sheaf-ADMM | Formal cross-domain agreement constraints | NT-CORE + NT-NEXUS | 4 weeks |

### Expected Impact
- **GWT routing quality**: +30-40% via calibration + hierarchical routing
- **KB retrieval efficiency**: +50% via tier-aware routing gates
- **Experience consolidation**: 3x faster strategic knowledge formation
- **Hardware efficiency**: +2x inference speedup via layer-level routing
- **Cross-domain consistency**: Formal guarantees via sheaf constraints
