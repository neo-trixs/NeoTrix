# Model Reverse Engineering — Cycle 382 (2026-09-12)

## 5 New Models/Papers

| # | Paper/Model | Venue | Core Innovation | NeoTrix Domain |
|---|-------------|-------|-----------------|----------------|
| 1 | **FFD (Faster Flash Decoding)** | ICML 2026 | Hardware-algorithm co-design for attention sparsity: fused selector+computer kernel, top-delta dynamic filtering | NT-CORE (GWT attention) |
| 2 | **EvoSparse** | ACL 2026 | Dynamic sparsity via Cross-Step Accumulation + Cross-Layer Propagation, 5.36× attention speedup | NT-CORE (attention routing) |
| 3 | **ReActNet** | arXiv 2609.05774 | Inference-time graph engineering: compile query→temporal communication graphs for multi-agent workflows | NT-CORE / NT-MIND |
| 4 | **DeAR** | arXiv 2608.17282 | Decentralized agentic reasoning: capability grounding + thought map navigation + adaptive topology update | NT-CORE / NT-MIND |
| 5 | **COMPASS** | ACL 2026 | Hierarchical context management: Main Agent + Meta-Thinker + Context Manager for long-horizon reasoning | NT-CORE / NT-MEMORY |

---

## Paper 1: FFD — Faster Flash Decoding

**Source**: arXiv:2609.00097 (ICML 2026 accepted)

### Core Problem
Long-context LLM decoding is bottlenecked by memory bandwidth (quadratic attention). Existing sparse attention trades metadata overhead for compute savings, but neither approach is optimal.

### Key Innovation
**Hardware-algorithm co-design**: FFD fuses the selector (which tokens to attend) and computer (attention computation) into a single GPU kernel. Instead of external metadata indices, it uses content-aware scanning via low-bit quantization.

**Top-Delta Strategy**: Dynamically filters blocks to achieve distribution-adaptive sparsity without global synchronization. Each block is scored and only the top-δ contribute.

### Results
- **11.6× kernel-level speedup** over FlashAttention
- **2.37× end-to-end throughput improvement**
- Scales to **256K context length**
- Training-free, plug-and-play
- Maintains model accuracy on RULER and LongBench

### NeoTrix Mapping
- **NT-CORE (GWT)**: The top-delta strategy is a hardware-optimized version of GWT salience routing — each "block" of tokens is a specialist module, and top-δ selection is attention gating
- **NT-MEMORY (KV cache)**: The fused kernel design eliminates the metadata overhead that plagues software-only sparse attention, directly applicable to kv_cache_optimizer.rs
- **Axiom A2 (Context as Scarce Resource)**: FFD proves that with hardware co-design, 256K context is tractable — validates KVMem's paged KV approach

### Actionable Insight
NeoTrix's GWT attention router could adopt the **fused selector-computer pattern** for its salience computation. Instead of separate "compute importance" → "route to specialist" steps, fuse them into a single operation for sub-step latency.

---

## Paper 2: EvoSparse — Evolving Sparsity

**Source**: ACL 2026 (Han et al.)

### Core Problem
Prior sparse attention methods treat each decoding step independently, ignoring two properties: (1) token importance is temporally consistent, and (2) different attention heads have different retrieval capabilities.

### Key Innovation
**Two lightweight mechanisms**:
1. **Cross-Step Accumulation**: Maintains a global "Heat" vector — exponentially decayed sum of historical sparse attention scores. Captures Long-term Anchors and enforces Short-term Reuse
2. **Cross-Layer Propagation**: Retrieval Heads (few, specialized) broadcast their high-quality indices to Standard Heads (many, general) across layers

### Results
- **5.36× attention latency speedup** (96K context, 2048-token budget)
- **2.33× end-to-end decoding speedup**
- Outperforms Quest, TidalDecode, Loki across PG-19, RULER, LongBench
- Approaches full attention performance in high-budget regimes

### NeoTrix Mapping
- **NT-CORE (GWT)**: Cross-Step Accumulation is isomorphic to GWT's salience persistence across consciousness cycles — "Heat" is a decaying attention memory
- **NT-CORE (AttentionManager)**: Cross-Layer Propagation mirrors NeoTrix's Ascendancy dual specialization — Retrieval Heads are "weapon set" specialists that guide generalist Standard Heads
- **NT-MIND (experience-tree)**: The Heat vector pattern maps to experience-tree's hub index — accumulated historical importance guides current routing

### Actionable Insight
NeoTrix's `AttentionManager` could implement **Cross-Layer Propagation**: let NT-CORE's E8 reasoning heads (Retrieval Heads) broadcast attention indices to NT-MIND's evolution heads (Standard Heads), reducing redundant salience computation across modules.

---

## Paper 3: ReActNet — Inference-Time Graph Engineering

**Source**: arXiv:2609.05774 (Sep 2026)

### Core Problem
Multi-agent LLM systems use graph-structured communication, but existing approaches either use fixed topologies (inflexible) or learned topologies (require training). Neither can adapt at inference time.

### Key Innovation
**Training-free graph compilation**: Given a query and role-specialized agents, ReActNet compiles a **task-conditioned temporal workflow graph**. Each graph snapshot = one reasoning stage; each edge carries a natural-language instruction specifying the message.

**Separation of concerns**: Graph compilation (what topology?) is separated from graph execution (how to pass messages?). The compiled graph is explicit, inspectable, and task-conditioned.

### Results
- Consistently improves over fixed-topology and learned-topology baselines
- Competitive inference cost
- Works across knowledge reasoning, math, code, GAIA-style tasks

### NeoTrix Mapping
- **NT-CORE (GWT)**: ReActNet's temporal graph snapshots are isomorphic to GWT's broadcast cycles — each cycle produces a directed communication graph among specialists
- **NT-MIND (SEAL)**: The compile-then-execute pattern maps to SEAL's distillation phase — compile learned patterns into executable skill graphs
- **NT-ACT (orchestration)**: The edge-level natural-language instructions are "skill protocols" — exactly what NeoTrix's capability registry needs for inter-domain communication

### Actionable Insight
NeoTrix's GWT attention router should support **inference-time graph compilation**: given a task, dynamically compile which NT-* domains communicate, with what message semantics, and for how many cycles. This replaces static module routing with task-adaptive topology.

---

## Paper 4: DeAR — Decentralized Agentic Reasoning

**Source**: arXiv:2608.17282 (Aug 2026)

### Core Problem
Centralized multi-agent coordination has routing bottlenecks and static role allocations that fail on complex multimodal queries. Central coordinators hallucinate capabilities in open-ended scenarios.

### Key Innovation
**Three mechanisms replacing central control**:
1. **Decentralized Capability Grounding**: Agents assess their own suitability for query fragments based on verifiable linguistic benchmarks (not static role labels)
2. **Thought Map Navigation**: Shared topological map of reasoning space; agents perform local graph traversal to select optimal downstream peers
3. **Topology Update**: Progressive backtracking — if a path fails, prune the edge and re-navigate

### Results
- Outperforms centralized baselines across 9 multimodal/text QA benchmarks
- No central coordinator bottleneck
- Adaptive error correction via topology update

### NeoTrix Mapping
- **NT-CORE (E8)**: DeAR's capability grounding is E8's hexagram state assessment — each module evaluates its own fitness for the current reasoning stage
- **NT-MIND (self-evolution)**: Topology Update is SEAL's convergence check — failed reasoning paths trigger structural adaptation
- **NT-GOVERNANCE**: The progressive backtracking (expand failure set step-by-step) is risk-graded cleanup — don't over-prune, expand cautiously

### Actionable Insight
NeoTrix's module coordination should support **decentralized capability grounding**: instead of a central router deciding which domain handles a task, each domain self-assesses fitness and the most capable domain volunteers. This eliminates the routing bottleneck in large domain networks.

---

## Paper 5: COMPASS — Context-Organized Multi-Agent Planning

**Source**: ACL 2026

### Core Problem
Long-horizon agent tasks fail because context management is the bottleneck — extended histories cause agents to overlook evidence or become distracted. Small errors compound across steps.

### Key Innovation
**Three specialized components**:
1. **Main Agent**: Performs reasoning and tool use (tactical)
2. **Meta-Thinker**: Monitors progress, detects anomalies (looping, tool misuse), issues strategic interventions
3. **Context Manager**: Maintains concise, relevant progress briefs; selectively draws from persistent notes, current trajectory, and strategic signals

**Key distinction**: Tactical reasoning (accurate tool use per turn) vs. Strategic reasoning (higher-level oversight that prevents error cascade)

### Results
- **+20% relative accuracy** over single- and multi-agent baselines on GAIA, BrowseComp, Humanity's Last Exam
- Matches established DeepResearch agents when scaled with sampling
- Prevents two failure modes: context overload (premature conclusions) and contextual pollution (repeated errors)

### NeoTrix Mapping
- **NT-CORE (ConsciousnessTree)**: Meta-Thinker is isomorphic to ConsciousnessTree's Branches stage — monitors cross-domain health and issues strategic interventions
- **NT-MEMORY (KB)**: Context Manager is the KB's relevance scoring — selectively retrieves from persistent knowledge based on current reasoning needs
- **NT-REPAIR**: Meta-Thinker's anomaly detection (looping, tool misuse) maps to NT-REPAIR's degradation pattern detection
- **Axiom A2 (Context as Scarce Resource)**: COMPASS proves that **externalizing context management** to a dedicated component is superior to in-context management

### Actionable Insight
NeoTrix's ConsciousnessTree should explicitly separate **tactical execution** (NT-ACT tool calls) from **strategic oversight** (Meta-Thinker pattern) from **context curation** (KB retrieval). The Meta-Thinker should monitor for:
- Looping patterns (repeated failed tool calls)
- Context pollution (irrelevant information accumulating in working memory)
- Error cascade (small mistakes compounding across cycles)

---

## Cross-Paper Synthesis

### Pattern 1: Hardware-Algorithm Co-Design (FFD)
The selector-computer fusion pattern applies beyond attention — NeoTrix could fuse GWT salience computation with module routing into a single operation.

### Pattern 2: Temporal Consistency (EvoSparse)
Token importance persists across steps; module importance persists across consciousness cycles. Heat vector = experience-tree hub index with decay.

### Pattern 3: Inference-Time Topology (ReActNet, DeAR)
Static module routing → dynamic task-adaptive graphs. Compile topology from query, execute via message passing.

### Pattern 4: Decentralized Grounding (DeAR)
Self-assessed capability > central router assignment. Each NT-* domain evaluates its own fitness.

### Pattern 5: Externalized Context Management (COMPASS)
Meta-Thinker + Context Manager as architectural primitives, not emergent behaviors. ConsciousnessTree should formalize this separation.

## NeoTrix Integration Priority

| Priority | Paper | Action | Target Module |
|----------|-------|--------|---------------|
| P0 | COMPASS | Add Meta-Thinker + Context Manager to ConsciousnessTree | nt_core_self |
| P1 | EvoSparse | Implement Heat vector for GWT attention persistence | nt_core_gwt |
| P1 | DeAR | Add decentralized capability grounding to module routing | nt_core_routing |
| P2 | ReActNet | Support inference-time graph compilation in SEAL | nt_mind_seal |
| P2 | FFD | Explore fused selector-computer for GWT salience | nt_core_gwt |
