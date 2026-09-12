# Model Reverse Engineering — Cycle 406 (2026-09-12)

## 5 New AI Models/Papers → NeoTrix Domain Mapping

---

### 1. HMARS: Hierarchical Multi-Agent Memory System
- **Paper**: arXiv:2606.28349 (Jun 2026)
- **Authors**: Li, Zheng, Zhou, Xu (Tsinghua)
- **Domain**: Long-context reasoning, multi-agent memory

#### Architecture
Three-tier agent hierarchy for managed memory:
- **Sub-agents**: Maintain grounded access to bounded memory regions (each handles a "page" of context)
- **Mid-agents**: Manage regional context, provide query-specific coordination across sub-agents
- **Frontier model**: Performs final reasoning over retrieved evidence pages

Key insight: treats long contexts as **managed memory** rather than flat retrieval corpus. Standard RAG (top-K chunk retrieval) discards evidence before reasoning begins because relevance depends on broader context.

#### Novel Patterns
1. **Hierarchical memory management** — sub-agents own bounded regions, mid-agents coordinate, frontier reasons. Mirrors OS virtual memory: page tables (sub-agents) → TLB (mid-agents) → CPU cache (frontier).
2. **Context-dependent relevance** — relevance is not intrinsic to a chunk; it depends on which other chunks are present. Sub-agents provide grounded access that preserves this dependency.
3. **Evidence breadth diagnostic** — new benchmark measuring whether the system retrieves ALL required supporting evidence, not just the final answer.

#### NeoTrix Domain Mapping

| Domain | Integration Point | Pattern |
|--------|------------------|---------|
| **NT-MEMORY** | KB page management | Sub-agents as "page managers" for KB regions; mid-agents as query-time coordinators |
| **NT-CORE** | GWT attention routing | Hierarchical salience: local (sub-agent) → regional (mid-agent) → global (frontier) |
| **NT-MIND** | SEAL pipeline memory | Evidence breadth diagnostic → memory completeness verification |
| **NT-WORLD** | Document ingestion | Region-bounded crawling with inter-region relevance tracking |

**Absorption priority**: HIGH. HMARS hierarchy maps directly to NeoTrix's 6-layer architecture. Sub-agents ≈ L1 capability nodes, mid-agents ≈ L2 perception coordinators, frontier ≈ L5 cognition. The "evidence breadth" diagnostic should be added to D17-D20 (absorption progress) audit dimensions.

---

### 2. MAGMA: Multi-Graph Agentic Memory Architecture
- **Paper**: ACL 2026 (Jul 2026)
- **Authors**: Jiang, Li, Li, Li
- **Domain**: Agent memory, multi-graph retrieval

#### Architecture
Represents each memory item across **four orthogonal graphs**:
1. **Semantic graph** — concept similarity connections
2. **Temporal graph** — when memories were created/accessed
3. **Causal graph** — cause-effect relationships between memories
4. **Entity graph** — entity co-occurrence and relationships

Retrieval is **policy-guided traversal** over these relational views. Query-adaptive: different queries traverse different graphs. Decouples memory representation from retrieval logic.

#### Novel Patterns
1. **Four-graph memory model** — instead of single vector space, memories live in 4 orthogonal views. Retrieval selects which graph(s) to traverse based on query type.
2. **Policy-guided traversal** — retrieval is not similarity search but learned policy that navigates the graph. Different queries use different traversal strategies.
3. **Transparent reasoning paths** — because traversal is over explicit graphs, every retrieval step is explainable (which graph, which edge, why this path).
4. **Structured context construction** — retrieved evidence is assembled with explicit relational structure, not flat concatenation.

#### NeoTrix Domain Mapping

| Domain | Integration Point | Pattern |
|--------|------------------|---------|
| **NT-MEMORY** | KB graph queries | 4-graph model for KB nodes: semantic + temporal + causal + entity views |
| **NT-CORE** | VSA HyperCube | 4-graph traversal ≈ HyperCube dimension selection (different queries activate different dimensions) |
| **NT-SHIELD** | Audit trails | Transparent reasoning paths enable retrieval audit (D13-D16 consciousness dimensions) |
| **NT-MIND** | Knowledge distillation | Policy-guided traversal as distillation strategy — not all knowledge paths are equal |

**Absorption priority**: HIGH. MAGMA's 4-graph model is isomorphic to NeoTrix's VSA HyperCube dimensions. The policy-guided traversal maps to GWT attention routing — different "tasks" (queries) activate different "dimensions" (graphs). Should be absorbed into KB query engine design.

---

### 3. SAM: State-Adaptive Memory for Long-Horizon Agents
- **Paper**: arXiv:2605.24468 (May 2026)
- **Authors**: Hu, Qian, Wang, Liu, Zhao, Tan, Liu, Dou
- **Domain**: Agent memory, long-horizon reasoning

#### Architecture
Two-component memory system:
1. **Memory cues** — compact summaries of ongoing interaction (lightweight handles)
2. **Raw trajectory pages** — full interaction history preserved for recall

Key insight: memory cues are **not replacements** for history; they are **handles** that allow reconstructing temporally distant information on-demand. Agent's evolving state determines which historical information becomes relevant.

Memory optimization via:
- **Expert-guided supervision** — human/expert signals train what to cue
- **Reinforcement learning** — aligns cues with trajectory-level utility

#### Novel Patterns
1. **Cues as handles, not summaries** — cues don't compress information; they provide pointers that can reconstruct relevant history on demand. Distinction from RAG: retrieval finds chunks, cues reconstruct context.
2. **State-adaptive recall** — the same historical information may be irrelevant at step 5 but critical at step 20. Cue utility is state-dependent, not static.
3. **Trajectory-level optimization** — cues are optimized for the entire trajectory (not per-turn accuracy), ensuring long-horizon coherence.

#### NeoTrix Domain Mapping

| Domain | Integration Point | Pattern |
|--------|------------------|---------|
| **NT-MEMORY** | Experience-tree lazy loading | Cues as branch pointers in experience hub; on-demand loading via route table |
| **NT-CORE** | ConsciousnessTree cycle boundary | State-adaptive recall: same experience irrelevant at cycle N, critical at cycle M |
| **NT-MIND** | SEAL pipeline distillation | Cue optimization ≈ distillation quality optimization |
| **NT-REPAIR** | Self-healing memory | Cue reconstruction for memory repair — rebuild from handles when raw data corrupted |

**Absorption priority**: HIGH. SAM's "cues as handles" pattern is exactly NeoTrix's experience-tree lazy branch loading. The state-adaptive recall maps to ConsciousnessTree's cycle-aware attention — the system's "state" (phi, coherence, cycle number) determines which memories are relevant. Should formalize the cue/handle distinction in CONTEXT.md.

---

### 4. Flux Attention: Context-Aware Hybrid Attention
- **Paper**: arXiv:2604.07394 (Apr 2026)
- **Authors**: Qiu, Hong, Yang, Wang, Liu, Dang, Li, Zhang
- **Domain**: Efficient LLM inference, attention optimization

#### Architecture
Layer-wise hybrid attention with dynamic routing:
- **Layer Router**: Lightweight classifier (added to frozen pretrained LLM) that routes each layer to Full Attention (FA) or Sparse Attention (SA)
- **Hard routing**: Binary decision per layer via argmax (not soft mixture)
- **Sparsity constraint**: Dynamic penalty prevents router degeneration (always choosing FA)
- **Hardware-aware**: Layer-level routing preserves contiguous memory access for GPU acceleration

Key insight: early layers are sensitive to attention changes (need FA), deeper layers are redundant (tolerate SA). Layer-level routing avoids head-level imbalance that kills hardware acceleration.

Results: +2.8x prefill speedup, +2.0x decode speedup with minimal quality loss. Only 12 hours training on 8x A800 GPUs.

#### Novel Patterns
1. **Layer-level routing > head-level routing** — head-level sparsity creates GPU load imbalance and synchronization long-tails. Layer-level routing maintains contiguous memory access.
2. **Dynamic penalty for sparsity control** — router naturally degenerates to "always FA" without intervention. Dynamic penalty enforces sparsity budget during training.
3. **Frozen pretrained + lightweight router** — only trains the router (tiny), not the full model. Parameter-efficient adaptation.
4. **Context-aware, not static** — different inputs get different FA/SA allocations. Complex reasoning → more FA; simple completion → more SA.

#### NeoTrix Domain Mapping

| Domain | Integration Point | Pattern |
|--------|------------------|---------|
| **NT-CORE** | GWT attention routing | Layer-level FA/SA ≈ domain-level attention allocation (some domains get full attention, others sparse) |
| **NT-PHYSICAL** | Hardware-aware inference | Contiguous memory access pattern for GPU efficiency |
| **NT-MEMORY** | KV cache optimization | SA layers reduce KV cache pressure; FA layers preserve quality |
| **NT-IO** | Provider cost routing | Context-aware routing mirrors cost-aware routing (Axiom A1): complex → expensive model, simple → cheap |

**Absorption priority**: MEDIUM. Flux Attention's layer-level routing is a microcosm of NeoTrix's GWT domain-level routing. The dynamic penalty mechanism (preventing degeneration) is relevant to NT-MIND's self-evolution ceiling. Less directly applicable than memory papers but informs attention architecture design.

---

### 5. Adaptive Coopetition (AdCo): Multi-Agent Reasoning via UCB
- **Paper**: ACL Anthology 2026
- **Authors**: Miin et al.
- **Domain**: Multi-agent coordination, inference-time computation

#### Architecture
Two-phase inference-time framework:
1. **Coopetition mechanism** — agents choose to collaborate OR compete based on UCB (Upper Confidence Bound) signal
2. **Adaptive switching** — at each round, agents leverage coarse verifier signals to decide: collaborate (share reasoning) or compete (independently solve)
3. **Peer feedback refinement** — agents iteratively refine reasoning based on peer output

Key insight: without high-performance verifiers (expensive to train), adaptive coopetition achieves 20% relative improvement via uncertainty-driven exploration. Agents with comparable capabilities benefit most from competitive pressure.

#### Novel Patterns
1. **Coopetition > pure cooperation or competition** — dynamic switching between modes based on confidence signals. Low confidence → collaborate; high confidence → compete.
2. **UCB-based mode selection** — uses multi-armed bandit framework (UCB) to balance explore (compete) vs exploit (collaborate).
3. **Coarse verifier signals** — doesn't need perfect verification; even rough correctness signals enable effective mode switching.
4. **Uncertainty-driven exploration** — when participants have comparable capabilities, uncertainty drives competitive exploration that improves collective performance.

#### NeoTrix Domain Mapping

| Domain | Integration Point | Pattern |
|--------|------------------|---------|
| **NT-CORE** | GWT attention competition | Coopetition ≈ attention domain competition: domains compete for salience, collaborate when salience is shared |
| **NT-MIND** | SEAL pipeline exploration | UCB-based explore/exploit for skill crystallization — try new approaches vs refine existing |
| **NT-ACT** | Multi-agent task routing | Dynamic collaboration/competition for parallel task execution |
| **NT-FEEL** | Social emotion dynamics | Coopetition models social dynamics between agents (trust/competition/coordination) |

**Absorption priority**: MEDIUM. AdCo's coopetition mechanism is relevant to NeoTrix's GWT attention routing — domains should compete for attention (salience) but collaborate when tasks require cross-domain synthesis. The UCB-based switching maps to NT-MIND's exploration strategy during SEAL cycles.

---

## Synthesis: 5-Paper Cross-Pollination Matrix

| Paper | NT-CORE | NT-MEMORY | NT-MIND | NT-ACT | NT-SHIELD | NT-IO | NT-PHYSICAL | NT-FEEL |
|-------|---------|-----------|---------|--------|-----------|-------|-------------|---------|
| HMARS | GWT hierarchy | Page management | Evidence verification | — | — | — | — | — |
| MAGMA | VSA dimensions | 4-graph KB | Distillation paths | — | Audit trails | — | — | — |
| SAM | Cycle-aware recall | Cue/handle loading | Cue optimization | — | Memory repair | — | — | — |
| Flux Attention | Layer-level routing | KV cache | Sparsity penalty | — | — | Cost routing | Hardware-aware | — |
| AdCo | Attention competition | — | Exploration strategy | Task routing | — | — | — | Social dynamics |

## Key Meta-Patterns Across 5 Papers

1. **Hierarchy as architecture** — HMARS (sub/mid/frontier), Flux (layer routing), MAGMA (graph hierarchy). NeoTrix's 6-layer architecture is validated: hierarchy enables both efficiency and reasoning quality.

2. **Memory ≠ storage** — MAGMA (4 graphs), SAM (cues as handles), HMARS (managed regions). Memory is active infrastructure with structure, not passive storage. Aligns with NeoTrix's KB as shared state layer.

3. **Routing > static allocation** — Flux (FA/SA routing), AdCo (coopetition routing), HMARS (query-adaptive hierarchy). Dynamic routing beats static allocation in every paper. Validates NeoTrix's GWT + cost-aware routing (Axiom A1).

4. **Verification prevents degeneration** — Flux (dynamic penalty), Ouuroboros (immutable success), AdCo (coarse verifier). Self-improving systems need external verification to prevent reward hacking. Validates NeoTrix's CI-as-gate pattern.

5. **Coarse signals suffice** — AdCo (rough verifier), SAM (expert-guided supervision). Perfect verification is not required; rough signals enable effective adaptation. Reduces infrastructure cost for self-evolution.

## Absorption Action Items

| Action | Source Paper | Target | Priority |
|--------|-------------|--------|----------|
| Add 4-graph KB query model | MAGMA | NT-MEMORY KB design | P1 |
| Formalize cue/handle distinction | SAM | experience-tree SKILL.md | P1 |
| Add evidence breadth diagnostic | HMARS | D17-D20 audit dimensions | P2 |
| Implement layer-level GWT routing | Flux Attention | NT-CORE attention module | P2 |
| Add coopetition to GWT salience | AdCo | NT-CORE competition mechanism | P3 |
| Dynamic penalty for self-evolution | Flux Attention | NT-MIND SEAL ceiling | P3 |
| State-adaptive memory recall | SAM | NT-MEMORY cycle-aware loading | P1 |
| Transparent retrieval audit | MAGMA | NT-SHIELD audit trail | P2 |
