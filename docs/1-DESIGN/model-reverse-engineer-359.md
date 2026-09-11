# Model Reverse Engineering — Cycle 359 (2026-09-12)

## Selection Criteria
Recent papers (2025-2026) on efficient inference, attention mechanisms, agent coordination, and memory routing. Mapped to NeoTrix 7 domains (NT-CORE, NT-MIND, NT-MEMORY, NT-WORLD, NT-ACT, NT-IO, NT-SHIELD). Focus: papers not covered in cycles 318-358.

---

## 1. RaaS — Reasoning-Aware Attention Sparsity with Milestone Tokens

| Field | Detail |
|-------|--------|
| **Paper** | ACL 2025 Findings (aclanthology.org/2025.findings-acl.131) |
| **Authors** | Junhao Hu, Wenrui Huang, Weidong Wang, et al. |
| **Key Innovation** | Identifies "milestone tokens" in reasoning chains — tokens analogous to lemmas in mathematical proofs that emerge, are utilized, and become unimportant. Retains KV vectors only for milestone tokens, achieving O(L) time and O(L) memory (both linear in cache budget). Solves the "impossible trinity" of accuracy, time, and memory. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Milestone Token Detection** | During reasoning decode, certain tokens serve as logical anchors (e.g., "therefore", "assume", key variable names). These are identified dynamically, not by position or frequency. |
| **Lifetime Tracking** | Milestone tokens have a lifecycle: emerge → utilized → become unimportant. KV vectors are retained only during the utilization window, then evicted. Not static importance scores but temporal relevance. |
| **Impossible Trinity Resolution** | Prior methods: Quest achieves O(L) time but O(N) memory. RaaS achieves O(L) time AND O(L) memory by evicting expired milestones. First method to break the trinity. |
| **Reasoning-Aware Design** | The sparsity pattern is specific to reasoning tasks — different from general long-context sparsity. Milestone tokens are a property of logical reasoning, not just attention patterns. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-MEMORY** | Milestone tokens = experience-tree branch importance scoring. Not all experiences are equally important — some are "milestones" (key decisions, breakthroughs) that deserve long-term retention. Others are transient. RaaS validates temporal relevance scoring for KB entries. |
| **NT-CORE** | Milestone lifecycle = ConsciousnessTree growth cycle phases. Each phase (Soil→Roots→Trunk→Branches→Fruits→Core) has milestone transitions. Milestones in one phase become unimportant in the next. GWT should track milestone lifecycle across reasoning steps. |
| **NT-MIND** | Impossible trinity resolution = SEAL pipeline efficiency frontier. SEAL must balance accuracy (thorough distillation), time (cycle speed), and memory (KB storage). RaaS proves this trinity is solvable with temporal eviction. |
| **NT-NEXUS** | Expired milestone eviction = cross-session memory compression. Old session milestones that are no longer relevant get compressed, not retained at full fidelity. |

### Key Takeaway for NeoTrix
**Temporal relevance scoring for memory** — not static importance but lifecycle-aware retention. An experience that was critical in cycle N may be irrelevant by cycle N+5. NeoTrix's KB should track milestone lifecycle: detect when an entry transitions from "utilized" to "expired" and compress accordingly. This is the reasoning-aware version of forgetting curves (validated by ZenBrain in cycle 358).

---

## 2. MoBA — Mixture of Block Attention for Long-Context LLMs

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2502.13189 (Feb 2025, deployed in Kimi) |
| **Authors** | Enzhe Lu, Zhejun Jiang, et al. (Moonshot AI) |
| **Key Innovation** | Applies Mixture of Experts (MoE) principles to the attention mechanism. Divides context into blocks; each block is an "expert" that can be attended to or skipped. Model learns which blocks to attend autonomously — no predefined biases (unlike sink/window attention). Seamlessly transitions between full and sparse attention. Already deployed in Kimi's production. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Block-Level Experts** | Context is divided into fixed-size blocks. Each block is an expert that the attention mechanism can choose to attend to or skip. Not token-level (too fine) or sequence-level (too coarse). |
| **Autonomous Selection** | The model learns which blocks are relevant for each query. No predefined structure — not window, not sink, not random. The selection is content-dependent and trained end-to-end. |
| **Full↔Sparse Transition** | MoBA can seamlessly switch between full attention (all blocks) and sparse attention (selected blocks). No architecture change needed — same model, different inference mode. |
| **Production Deployment** | Already deployed in Kimi (Moonshot AI) for long-context requests. Not theoretical — proven at scale with real users. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|--------|
| **NT-CORE** | Block-level experts = GWT's domain-level attention allocation. Each NT-* domain is a "block" that GWT can attend to or skip. Not per-feature routing (too fine) but per-domain routing. MoBA validates domain-level granularity for attention. |
| **NT-WORLD** | Block selection = NT-WORLD's crawl source selection. Different tasks need different information sources (web, KB, API). MoBA-style routing selects which sources to query. |
| **NT-MEMORY** | Block-level caching = KB namespace routing. Each namespace (ephemeral, session, cross-session) is a block. Query-time routing selects relevant namespaces. |
| **NT-IO** | Full↔Sparse transition = NT-IO's provider fallback. When full attention (expensive provider) is needed, use it. When sparse attention (cheap provider) suffices, switch. Seamless transition without reconfiguration. |

### Key Takeaway for NeoTrix
**Domain-level attention as MoE experts** — each NT-* domain is an expert block that GWT can attend to or skip. This is the architecture for GWT's attention allocation: not per-feature (too fine, creates overhead) but per-domain (hardware-friendly, semantically meaningful). MoBA proves this works at production scale (Kimi deployment). NeoTrix should implement domain-level attention blocks with learned selection.

---

## 3. LISA — Linear-Indexed Sparse Attention with Corrective Reasoning

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2607.19358 (May 2026) |
| **Authors** | (Multiple authors, 2026) |
| **Key Innovation** | Two-branch architecture: linear attention (LA) for global state + sliding-window softmax (SA) for local context. Indexer module (inspired by DeepSeek V3.2) selects important tokens. k-step corrective reasoning partitions generated tokens into segments and corrects state deviations. Training pipeline: Stage 1 (LA initialization + KD), Stage 2 (corrective reasoning). |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Dual-Branch Architecture** | LA branch captures long-range dependencies via global state matrix. SA branch captures local context via sliding window. Gating mechanism fuses both. O(nM) complexity where M ≪ n. |
| **Indexer Module** | Lightweight token selector with only Q/K projections + ReLU dot product. Inspired by DeepSeek V3.2 and GLM-5. Selects important tokens from full context for subsequent generation. |
| **Corrective Reasoning** | Generated tokens partitioned into segments of size k. Dual-update mechanism synchronizes LA state transitions with segment-level updates. Corrects noise-induced deviations from correct reasoning trajectory. |
| **State Transition Framework** | Reasoning modeled as state transitions. LA maintains global reasoning state matrix. Each reasoning step transitions the state. Corrective mechanism ensures state doesn't deviate. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|--------|
| **NT-CORE** | Corrective reasoning = ConsciousnessTree self-correction. When growth cycle state deviates from correct trajectory (e.g., exploration without distillation), corrective mechanism realigns. State transition model = growth cycle phase transitions with correction. |
| **NT-MEMORY** | LA global state = KB cross-session state matrix. Long-range dependencies across sessions are maintained in a compressed state matrix, not raw entries. Indexer selects relevant historical context for current query. |
| **NT-MIND** | Segment-level correction = SEAL pipeline stage correction. Each SEAL stage (exploration→distillation→self-test→absorption) is a segment. If a stage deviates (e.g., over-exploration), corrective mechanism realigns before next stage. |
| **NT-ACT** | k-step corrective = NT-ACT's action plan correction. Action plans partitioned into segments. After each segment, check if execution is on track. Corrective mechanism adjusts before next segment. |

### Key Takeaway for NeoTrix
**Corrective reasoning with state transitions** — reasoning isn't just forward progression; it needs active deviation correction. LISA's k-step corrective mechanism maps directly to SEAL pipeline health monitoring: after each stage, verify the state is on trajectory. If exploration has over-extended (state deviation), correct before moving to distillation. The LA global state matrix is the architecture for cross-session memory compression.

---

## 4. Sheaf-ADMM — Multi-Agent Coordination via Algebraic Topology

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2605.31005 (May 2026, accepted at ICML 2026) |
| **Authors** | Jeffrey Seely, Bartłomiej Cupiał, Llion Jones |
| **Key Innovation** | Differentiable optimization framework for multi-agent coordination using cellular sheaves. Agents decompose input into overlapping local views. Coordination via ADMM with inter-agent constraints specified by a sheaf — the sheaf specifies which aspects of neighboring solutions must agree. Backprop through unrolled optimization jointly trains all components. Exposed primal/consensus/dual variables enable direct analysis and intervention. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Cellular Sheaf Constraints** | A sheaf specifies which aspects of neighboring agents' solutions must agree. Not global consensus (too rigid) but local consistency requirements. Different agent pairs can have different agreement requirements. |
| **ADMM Coordination** | Alternating Direction Method of Multipliers decomposes the global problem into local subproblems. Each agent solves a convex subproblem parameterized by a neural encoder. Coordination happens through consensus variables. |
| **Exposed Coordination Dynamics** | Primal variables (agent solutions), consensus variables (agreement states), and dual variables (constraint enforcement) are all exposed. Not black-box coordination — every aspect is inspectable and intervenable. |
| **Local View Decomposition** | Input decomposed into overlapping local views. Each agent sees a different view. Overlap ensures information propagation. On MNIST, local-view decomposition yields improved robustness to distribution shifts. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|--------|
| **NT-ACT** | Sheaf-ADMM = NT-ACT's multi-agent coordination protocol. Agents (domains) solve local subproblems, coordinate through consensus. Sheaf constraints = domain trait contracts (which outputs must agree). Primal/dual variables = inspectable coordination state. |
| **NT-CORE** | Sheaf constraints = GWT's attention allocation rules. Which domains must agree on attention allocation? The sheaf specifies domain interaction patterns. Not all domains need to agree — only neighboring ones in the reasoning graph. |
| **NT-GOVERNANCE** | Exposed coordination dynamics = policy auditability. Every coordination decision (primal), agreement state (consensus), and enforcement (dual) is inspectable. Maps to NT-GOVERNANCE's compliance verification with full action traceability. |
| **NT-NEXUS** | Overlapping local views = cross-session memory overlap. Sessions share overlapping context (shared KB entries). The sheaf specifies which session states must agree on shared entries. |

### Key Takeaway for NeoTrix
**Sheaf-based coordination with inspectable dynamics** — multi-agent coordination shouldn't be black-box. Sheaf-ADMM exposes primal (what each agent decided), consensus (where they agree), and dual (how disagreements are resolved) variables. NeoTrix's domain coordination should expose these same variables: what each NT-* domain decided, where domains agree on shared state, and how conflicts are resolved. The cellular sheaf is the mathematical framework for specifying domain interaction contracts.

---

## 5. MPAC — Multi-Principal Agent Coordination Protocol

| Field | Detail |
|-------|--------|
| **Paper** | arXiv:2604.09744 (Apr 2026) |
| **Authors** | Kaiyang Qian, Xinmin Fang, Zhengxiong Li |
| **Key Innovation** | Application-layer protocol for multi-principal agent coordination (independent agents from different owners). Five coordination layers: Session, Intent, Operation, Conflict, Governance. 21 message types, three state machines, Lamport-clock causal watermarking, optimistic concurrency control. 95% reduction in coordination overhead, 4.8x speedup vs human-mediated baseline. |

### Architecture Analysis

| Aspect | Detail |
|--------|--------|
| **Five-Layer Coordination** | Session (establish connection), Intent (declare what you want to do), Operation (execute), Conflict (detect and resolve), Governance (human oversight). Each layer has specific responsibilities and message types. |
| **Multi-Principal Design** | Unlike MCP (single principal) and A2A (single-principal delegation), MPAC handles independent agents from different owners. No shared trust assumption — coordination through protocol, not trust. |
| **Causal Watermarking** | Lamport-clock causal ordering ensures operations are applied in correct causal order. Not just temporal ordering but causal dependency tracking. |
| **Optimistic Concurrency** | Agents proceed optimistically, detecting conflicts post-hoc. Conflict resolution is a first-class structured object, not ad-hoc. Human-in-the-loop through pluggable governance layer. |
| **223 Tests + 7 Demos** | Production-validated protocol with comprehensive test suite. Not theoretical — two interoperable implementations in Python and TypeScript. |

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|--------|
| **NT-ACT** | MPAC = NT-ACT's inter-domain coordination protocol. Five layers map to NeoTrix's domain interaction: Session (domain discovery), Intent (task declaration), Operation (execution), Conflict (resource contention), Governance (human override). |
| **NT-GOVERNANCE** | Governance layer = NT-GOVERNANCE's policy enforcement. Pluggable governance = configurable compliance rules per domain interaction type. Conflict as first-class object = NT-GOVERNANCE's policy violation tracking. |
| **NT-SHIELD** | Multi-principal design = NT-SHIELD's trust boundary enforcement. Independent agents from different trust domains coordinate through protocol, not trust. Lamport-clock ordering = audit trail for security-relevant operations. |
| **NT-NEXUS** | Session layer = NT-NEXUS's cross-session bridging. Sessions from different principals (users, agents) can coordinate through shared protocol. Causal watermarking = cross-session causal dependency tracking. |

### Key Takeaway for NeoTrix
**Five-layer coordination protocol with conflict as first-class object** — MPAC's architecture maps directly to NeoTrix's domain coordination needs. The key insight is that conflict detection and resolution should be a first-class protocol layer, not bolted-on error handling. When NT-ACT and NT-MEMORY contend for resources, the conflict should be a structured object with resolution options, not an exception. The governance layer enables human-in-the-loop for high-stakes decisions.

---

## Cross-Cutting Synthesis (Cycle 359)

| Theme | Papers | NeoTrix Integration |
|-------|--------|-------------------|
| **Temporal Relevance Lifecycle** | RaaS | Memory entries have lifecycle: emerge → utilized → expired. Track and evict expired milestones. |
| **Domain-Level Attention Blocks** | MoBA | Each NT-* domain is an attention expert. GWT selects which domain blocks to attend. |
| **Corrective Reasoning** | LISA | SEAL stages need active deviation correction, not just forward progression. |
| **Inspectable Coordination** | Sheaf-ADMM | Expose primal/consensus/dual variables for every domain interaction. |
| **Conflict as First-Class Object** | MPAC | Domain conflicts should be structured objects with resolution protocols, not exceptions. |
| **Causal Ordering Across Agents** | MPAC, Sheaf-ADMM | Lamport-clock causal tracking for cross-domain and cross-session operations. |

## Novel vs Incremental

| Paper | Novelty | NeoTrix Priority |
|-------|---------|-----------------|
| **Sheaf-ADMM** | High — algebraic topology for multi-agent coordination with inspectable dynamics | P0 — mathematical framework for NT-ACT domain coordination |
| **MPAC** | High — five-layer protocol with conflict as first-class object, production-validated | P0 — protocol architecture for inter-domain coordination |
| **RaaS** | High — temporal lifecycle for attention sparsity, solves impossible trinity | P1 — temporal relevance scoring for KB memory lifecycle |
| **MoBA** | Medium — block-level attention MoE, production-deployed in Kimi | P1 — domain-level attention allocation for GWT |
| **LISA** | Medium — corrective reasoning with dual-branch attention | P2 — SEAL pipeline deviation correction mechanism |

## Implementation Roadmap

| Phase | Action | Paper |
|-------|--------|-------|
| **Immediate** | Design five-layer domain coordination protocol (Session/Intent/Operation/Conflict/Governance) | MPAC |
| **Week 2** | Implement inspectable coordination variables (primal/consensus/dual) for domain interactions | Sheaf-ADMM |
| **Month 1** | Add temporal lifecycle tracking to KB entries (emerge→utilized→expired) | RaaS |
| **Month 2** | Prototype domain-level attention blocks for GWT (MoE-style domain selection) | MoBA |
| **Quarter** | Add corrective reasoning mechanism to SEAL pipeline stages | LISA |
