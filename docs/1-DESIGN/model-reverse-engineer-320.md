# Model Reverse Engineering — Cycle 320

**Date**: 2026-09-11
**Focus**: Efficient sparse attention, multi-agent topology, self-evolving execution, parallel reasoning

---

## 5 Models/Papers for Reverse Engineering

### 1. RouteRelay: Event-Triggered Cross-Layer Route Reuse for Efficient Dynamic Sparse Attention
- **Paper**: https://arxiv.org/abs/2609.07306 (Sep 2026)
- **Key Innovation**: Dynamic sparse attention that reuses routing metadata across Transformer layers. Anchor layers perform full routing; intermediate layers rescore only the previous top-k route plus a compact sentinel set (near-miss + randomly probed chunks). A query row is rerouted only when a sentinel challenges its weakest selected chunk. Retains 99.99% route recall while rerouting only 25-78% of rows depending on cross-layer drift. Evaluates 38-51% of full-routing score pairs as key-chunk count grows 128→1024.
- **Architecture Pattern**: Two-tier routing: anchor layers (expensive, full) + intermediate layers (cheap, selective). Sentinel-based triggering — reroute only when evidence warrants. Cross-layer stability assumption with probabilistic bounds on missed challengers.
- **NeoTrix Domain Mapping**:
  - **GWT (NT-CORE)**: Anchor/intermediate layer split = GWT broadcast (expensive, full) + salience update (cheap, selective). Sentinel mechanism = attention re-evaluation trigger — don't re-broadcast everything every cycle, only when sentinel detects drift
  - **ConsciousnessTree**: Cross-layer route reuse = branch health propagation — once a branch is assessed healthy at one cycle, subsequent cycles can reuse that assessment unless sentinel detects change
  - **NT-MEMORY**: Route metadata reuse = KB query result caching with sentinel invalidation. Top-k route stability = most KB queries have stable result sets, only invalidate when sentinel detects data drift
  - **Axiom A2 (Context as Scarce)**: Sentinel-gated rerouting preserves context budget — only spend re-routing computation when evidence demands it

### 2. SparseSpec-L: Recallable Sparse-Context Self-Speculative Decoding
- **Paper**: https://arxiv.org/abs/2607.27735 (Jul 2026, updated Aug 2026)
- **Key Innovation**: Training-free self-speculative decoding for long-context inference. Target model serves as both drafter and verifier. During drafting, uses dynamically sparsified KV cache; during verification, retains complete KV cache. Recycles per-head attention statistics from full-context verification as importance signal for next drafting round — no additional forward pass needed. Online entropy-based controller selects speculation length per step based on expected efficiency.
- **Architecture Pattern**: Single-model sparse-to-full pipeline. Three KV positions: sink tokens (global context) + recent tokens (local context) + important historical tokens (attention-ranked). Entropy-guided adaptive speculation avoids efficiency inversion (where extending speculation horizon reduces speedup).
- **NeoTrix Domain Mapping**:
  - **KVMem (NT-MEMORY)**: Three-position KV decomposition maps directly to our paged KV strategy — sink (pinned) + recent (hot) + important (cold-page recall). The entropy-guided adaptation parallels our compaction-vs-paging decision
  - **GWT (NT-CORE)**: Attention-based importance estimation = GWT salience scoring. The "reuse verification statistics" pattern means GWT salience can be updated from previous cycle's attention without full recomputation
  - **Axiom A1 (Cost-Aware)**: Entropy-based speculation length = cost-aware reasoning depth. When confidence (low entropy) is high, speculate more; when uncertain (high entropy), speculate less
  - **Dual Specialization**: Sparse drafting (Weapon Set II, cheap) + dense verification (Weapon Set I, expensive) = our Weapon Set switching pattern at the inference level

### 3. AGAO: Adaptive Goal-aware Attention Orchestration
- **Paper**: https://arxiv.org/abs/2607.23678 (Jul 2026)
- **Key Innovation**: Extends attention from token-level to workflow-level agent coordination. Three complementary mechanisms: (1) Goal-aware attention — semantic relevance between user goals and agent capabilities; (2) Topology-aware attention — structural dependencies within agent graphs; (3) Resource-aware attention — adaptive computational budgets among heterogeneous agents. Transforms static agent graphs into adaptive execution systems. Adaptive Graph Routing dynamically updates attention distributions based on execution feedback.
- **Architecture Pattern**: Hierarchical attention over agent graphs. Goal relevance scores agent importance, topology scores structural importance, resource allocation translates scores into execution policies. Agents are dynamically selectable computational units, not fixed workflow operators.
- **NeoTrix Domain Mapping**:
  - **GWT (NT-CORE)**: Goal-aware attention = GWT salience scoring (user intent → attention allocation). Topology-aware attention = ConsciousnessTree branch dependency modeling. Resource-aware attention = Axiom A1 (Cost-Aware Routing). AGAO's three mechanisms validate our GWT-first architecture
  - **ConsciousnessTree**: The "adaptive graph routing" where attention distributions update based on execution feedback = our 6-stage feedback loop (Soil→Roots→Trunk→Branches→Fruits→Core). Each stage adjusts attention based on prior stage outputs
  - **NT-ACT**: Resource-aware attention = dynamic task allocation to NT-* domains. Cheap domains (NT-IO) get sparse attention, expensive domains (NT-CORE) get full attention
  - **Attention Engineering**: AGAO validates the "Attention Engineering" paradigm — attention mechanisms aren't just for neural networks, they're for multi-agent coordination. Our GWT is an Attention Engineering system at the architecture level

### 4. Procedural Graphs: Self-Evolving Execution Structures for LLM Agents
- **Paper**: https://arxiv.org/abs/2609.09153 (Sep 2026)
- **Key Innovation**: Organizes procedural knowledge into (procedure, relation, procedure) triplets — a knowledge graph for what-to-do questions. At each step, a guidance model translates the surrounding subgraph into step-level situational guidance that biases (not dictates) the solver's next action. Self-evolving: an LLM refiner contrasts failed trajectories with successful ones, edits graph topology and attributes, commits edits that preserve held-out validation performance, retains rejected edits to discourage repetition.
- **Architecture Pattern**: Procedural knowledge as graph structure (not linear chains). Guidance model as soft control (bias, not dictate). Self-evolution via failure-success contrast with validation gating. Starts from minimal skeleton, builds graphs that match or surpass hand-designed ones.
- **NeoTrix Domain Mapping**:
  - **NT-MIND (SEAL)**: Procedural graph = skill node graph. The (procedure, relation, procedure) triplet maps to our skill node edges. Self-evolution via failure-success contrast = SEAL pipeline feedback loop. Validation gating before commit = our experience-tree quality gate
  - **GWT (NT-CORE)**: "Bias without dictate" = GWT salience influences attention but doesn't hard-route. The guidance model is a GWT-like mechanism at the procedural level
  - **ConsciousnessTree**: The procedural graph structure maps to our 6-stage loop — each stage is a procedure with conditional transitions. Self-evolution edits the graph topology = ConsciousnessTree learns optimal stage transitions from execution traces
  - **NT-REPAIR**: Failed trajectory analysis + graph repair = self-healing via structural modification. Retained rejected edits prevent repetition = failure memory

### 5. PARSER: Parallel Access Scatter-gather for Efficient Retrieval
- **Paper**: https://arxiv.org/abs/2609.06702 (Sep 2026)
- **Key Innovation**: Decouples reading from reasoning for long-context agents. Bank of lightweight subagents (frozen, off-the-shelf) each bound to a single chunk read the entire document in parallel. Lead agent (RL-optimized) reasons in depth through iterative scatter-gather rounds: broadcasts query to all subagents, aggregates returned evidence, formulates deeper follow-up queries conditioned on accumulated findings. Concentrates all learnable behavior in lead agent. 4B backbone outperforms strongest sequential baseline by 5.7 points avg (12.0 at 896K tokens). 11x latency reduction.
- **Architecture Pattern**: Parallel read + serial deep reason. Frozen subagents + trained lead agent (clean separation of concerns). Iterative scatter-gather rounds enable progressive deepening — each round asks a deeper question. Evidence accumulation across rounds improves answer quality.
- **NeoTrix Domain Mapping**:
  - **NT-WORLD (Perception)**: Parallel subagents = distributed perception workers for large document/crawl processing. Scatter-gather = GWT broadcast → specialist response → aggregation. The chunk-bound subagent pattern maps to our UnifiedCrawler parallel fetch
  - **GWT (NT-CORE)**: Scatter-gather rounds = GWT attention cycles. Lead agent broadcasting queries = salience broadcast. Aggregating evidence = attention integration. Progressive deepening = iterative attention refinement
  - **Dual Specialization**: Frozen subagents (Weapon Set II, cheap workers) + trained lead (Weapon Set I, expensive reasoning). The separation is clean — workers don't need to learn, lead doesn't need to read everything
  - **Axiom A2 (Context as Scarce)**: Parallel reading distributes context across subagents, keeping each subagent's context compact. Lead agent only sees aggregated evidence, not full document. This is context-efficient long-context processing

---

## Synthesis: Cross-Paper Patterns

### Pattern 1: Anchor-Selective Routing (RouteRelay, SparseSpec-L)
Both papers use an expensive anchor phase followed by cheap selective updates. RouteRelay: anchor layers route fully, intermediate layers rescore selectively. SparseSpec-L: full verification recycles stats for sparse drafting.

**NeoTrix Mapping**: GWT should implement anchor-selective cycles — full salience broadcast every N cycles (anchor), selective updates between anchors (when sentinel detects drift). Reduces GWT overhead without losing accuracy.

### Pattern 2: Guidance Without Dictation (AGAO, Procedural Graphs)
Both papers introduce control mechanisms that bias behavior without hard-routing. AGAO's resource-aware attention allocates budgets but doesn't force execution. Procedural Graphs' guidance model biases next action without dictating it.

**NeoTrix Mapping**: GWT salience should be advisory, not imperative. NT-* domains receive attention scores but retain autonomy to override. The ConsciousnessTree provides situational guidance, not command-and-control.

### Pattern 3: Self-Evolution via Failure-Success Contrast (Procedural Graphs, TROVE)
Both papers learn from trajectory analysis. Procedural Graphs contrasts failed vs successful trajectories to edit graph topology. TROVE distills workflow traces into skills with outcome-conditioned transitions.

**NeoTrix Mapping**: SEAL pipeline should explicitly contrast failure and success trajectories to evolve skill graphs. Experience-tree entries should include outcome labels (success/failure) for differential learning. The "retain rejected edits to discourage repetition" pattern is valuable for our failure memory.

### Pattern 4: Parallel Read + Serial Reason (PARSER, AGAO)
Both papers separate parallel data intake from sequential reasoning. PARSER: frozen subagents read in parallel, lead agent reasons serially. AGAO: agents process in parallel where topology allows, reason serially where dependencies require.

**NeoTrix Mapping**: NT-WORLD perception should parallelize data acquisition (crawling, parsing) while NT-CORE reasoning remains serial. The ConsciousnessTree can broadcast to multiple NT-* domains in parallel, then aggregate findings sequentially.

### Pattern 5: Attention Reuse Across Cycles (RouteRelay, SparseSpec-L)
Both papers reuse attention statistics from previous computations. RouteRelay: reuse route metadata across layers. SparseSpec-L: recycle verification statistics for next drafting round.

**NeoTrix Mapping**: GWT salience scores should be cached and reused across ConsciousnessTree cycles. Only recompute when sentinel detects significant change. KB query results should carry attention metadata for cross-session reuse. This reduces the overhead of our attention mechanisms.

---

## Implementation Candidates

| Priority | Paper | Action | NeoTrix Component |
|----------|-------|--------|-------------------|
| P0 | AGAO | Implement triple attention (Goal/Topology/Resource) in GWT salience | nt_core (GWT) |
| P0 | Procedural Graphs | Add procedural knowledge graph to skill node system | nt_mind (skill nodes) |
| P1 | SparseSpec-L | Add entropy-guided adaptive speculation to model switching | nt_io (provider routing) |
| P1 | PARSER | Implement parallel read + serial reason for NT-WORLD perception | nt_world |
| P2 | RouteRelay | Add sentinel-gated attention re-evaluation to GWT cycles | nt_core (GWT) |
| P2 | TROVE | Add selective suffix replacement to NT-REPAIR workflows | nt_repair |
