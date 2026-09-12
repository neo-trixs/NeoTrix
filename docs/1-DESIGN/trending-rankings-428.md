# Trending Rankings — Cycle 428 (2026-09-12)

## Methodology
- GitHub trending (AI agents, LLM tools, reasoning frameworks)
- ProductHunt September 2026 launches
- arXiv August–September 2026 papers
- Cross-referenced against cycles 318–427 to ensure novelty

---

## Top 10 New Projects

### 1. OmniAgent — Full-Dimensional Self-Evolving Agent Framework
| Field | Value |
|-------|-------|
| **Platform** | GitHub trending 2026 |
| **Stars** | ~2.5K |
| **Repo** | [YeQing17-2026/OmniAgent](https://github.com/YeQing17-2026/OmniAgent) |
| **Lang** | Python |

**What it does:** Self-evolving agent framework with 3 dimensions of continuous evolution: Skill Self-Evolution (auto-create/diagnose/repair skills from high-frequency patterns), Context Self-Evolution (real-time interaction feedback + LLM summarization for persistent user profiles), and BrainModel Self-Evolution (online RL loop with GRPO + PRM). Includes Hyper-Harness (dynamic multi-agent + 4-layer security scanning) and Deep Reflexion (inner-outer dual-layer reflective architecture for failure-to-insight conversion). Claims 90% token savings on skill injection vs user message injection.

**NeoTrix mapping:**
- NT-MIND: Skill Self-Evolution = SEAL pipeline's skill crystallization; pattern extraction from action sequences maps directly to experience-tree distillation stage
- NT-CORE: BrainModel Self-Evolution (online RL) = ConsciousnessTree's self-optimizing phi computation; the GRPO feedback loop parallels E8 hexagram refinement
- NT-REPAIR: Deep Reflexion (RCA + heuristic extraction) = self-healing pattern library; dual-layer (real-time interception + post-failure conversion) mirrors MAPE-K cycle
- NT-SHIELD: 4-layer security scanning (LLM review → Policy engine → Interactive approval → Execution sandbox) maps to NT-SHIELD's trust-tier egress guard pattern

**Novel Signal:** OmniAgent's full-dimensional evolution (Skill + Context + BrainModel) validates NeoTrix's architecture where evolution happens at multiple layers simultaneously. The "Hyper-Harness" concept — a scaffold that adapts to task complexity and risk level — is a production-ready pattern for NT-ACT tool execution. The 90% token savings on skill injection via structured format (vs raw user messages) supports NeoTrix's SKILL.md contract approach.

---

### 2. DeerFlow 2.0 — ByteDance Super-Agent Harness
| Field | Value |
|-------|-------|
| **Platform** | GitHub #1 Trending (Feb 2026) |
| **Stars** | ~81K |
| **Repo** | [bytedance/deer-flow](https://github.com/bytedance/deer-flow) |
| **Lang** | Python (LangGraph/LangChain) |

**What it does:** Open-source super-agent harness built on LangGraph/LangChain. 2.0 is a ground-up rewrite: sandbox-aware execution (Local/Docker/E2B), persistent memory with confidence-scored facts, skill-based workflows loaded on demand via SKILL.md, subagent delegation (max 3 concurrent, 15-min timeout), 9-middleware pipeline (thread isolation, uploads, sandbox, summarization, todo, title, memory, vision, clarification), and Nginx reverse proxy on port 2026. Skills loaded recursively from nested directories, preserving container paths.

**NeoTrix mapping:**
- NT-ACT: Subagent delegation with concurrency limits = NT-ACT's parallel task manager; the 3-concurrent/15-min-timeout pattern is production-ready
- NT-MEMORY: Confidence-scored facts with debounced extraction = KB-style trust scoring; mtime-based cache invalidation parallels experience-tree freshness
- NT-SHIELD: Per-thread sandbox isolation with virtual path translation = capability-based process isolation; ReadBeforeWriteMiddleware (sha256 gate) = safety for file-modifying tools
- NT-IO: 9-middleware pipeline = GWT attention routing chain; each middleware is a specialist module with specific responsibility

**Novel Signal:** DeerFlow 2.0's "harness not framework" philosophy — the runtime provides infrastructure, agents provide logic — aligns with NeoTrix's architecture where the 6-layer system provides the skeleton and domain modules provide the flesh. The recursive SKILL.md loading (preserving nested paths) is a clean pattern for NeoTrix's skill tree traversal. The middleware chain (9 stages) is a direct analog for GWT attention routing stages.

---

### 3. Qwen-AgentWorld — Language World Model for General Agents
| Field | Value |
|-------|-------|
| **Platform** | GitHub trending Jun 2026 |
| **Stars** | ~1K |
| **Repo** | [QwenLM/Qwen-AgentWorld](https://github.com/qwenlm/qwen-agentworld) |
| **Lang** | Python |

**What it does:** Native language world model that simulates agentic environments via long chain-of-thought reasoning across 7 unified domains: MCP, Search, Terminal, SWE, Android, Web, and OS. Trained via 3-stage pipeline (CPT → SFT → RL) on 10M+ real interaction trajectories. MoE architecture (35B total / 3B active, 256K context). Achieves 58.71 overall score, outperforming GPT-5.4 (58.25). Key: environment modeling is the training objective from CPT onward (not post-hoc adaptation). Supports controllable perturbations and fictional-world construction that generalizes to real tasks.

**NeoTrix mapping:**
- NT-WORLD: World model for 7 agent interaction domains = NT-WORLD's perception pipeline; the 7-domain coverage (MCP/Search/Terminal/SWE/Android/Web/OS) mirrors NT-WORLD's UnifiedCrawler multi-source approach
- NT-CORE: Native world model (CPT→SFT→RL) = ConsciousnessTree's environment modeling; the 3-stage training parallels SEAL pipeline stages
- NT-MIND: Fictional-world construction generalizing to real tasks = SEAL experience distillation where synthetic experiences transfer to production
- NT-PHYSICAL: Android/OS domain coverage = physical embodiment's sensor/actuator interfaces

**Novel Signal:** Qwen-AgentWorld proves that world modeling trained as the primary objective (not added later) produces better agent performance. NeoTrix's PerceptionBridge (L2→L5 attention-gated bridge) could adopt this "native world model" approach: train the perception pipeline to predict next-state, not just classify current-state. The controllable perturbation mechanism (injecting targeted weaknesses to expose agent failures) is a powerful SelfTest tool.

---

### 4. BeaconKV — KV Cache Compression via Beacon Queries
| Field | Value |
|-------|-------|
| **Platform** | arXiv Sep 2026 (ICML 2026) |
| **Stars** | N/A (paper) |
| **Repo** | [aiha-lab/BeaconKV](https://github.com/aiha-lab/BeaconKV) |
| **Lang** | Python |

**What it does:** Training-free KV cache compression method for Large Reasoning Models. Discovers "Thought Revisiting Tokens" (TRT) — decoding steps that re-attent to distant previous context (e.g., early task plans). TRT queries cluster into similarity groups in embedding space. Maintains beacon queries (compact representatives for each cluster) to anticipate which KV pairs will be revisited. Achieves 5.8× memory reduction while preserving accuracy, 4.3× throughput improvement.

**NeoTrix mapping:**
- NT-MEMORY: Beacon query clusters = KB hub index pattern (tiny index + on-demand branch loading); the "cluster representative" concept maps to experience-tree hub pointers
- NT-CORE: TRT detection = GWT attention pattern analysis; identifying which attention steps revisit distant context is analogous to GWT's salience recalculation
- NT-REPAIR: Memory pressure response (compress cache under pressure) = self-healing when KV budget is exceeded; the "beacon" concept provides a principled compression strategy vs naive eviction
- NT-MIND: Thought Revisiting = experience recall; beacon queries are like distilled experience pointers that enable efficient cross-session recall

**Novel Signal:** BeaconKV's discovery that reasoning models generate "Thought Revisiting Tokens" that re-attend to distant context has a direct analog in NeoTrix: experience-tree branches that reference distant sessions. The beacon query pattern (cluster representatives for efficient recall) could replace NeoTrix's current route-table matching with a learned cluster-index, reducing experience lookup from O(N) to O(K) where K is the number of beacon clusters.

---

### 5. Declarative Attention — Models Control Their Own Attention
| Field | Value |
|-------|-------|
| **Platform** | arXiv Sep 2026 |
| **Stars** | N/A (paper) |
| **Repo** | N/A |
| **Lang** | N/A |

**What it does:** Introduces Declarative Attention (DA) — a protocol where the model declares WHERE it needs to attend within its chain-of-thought, partitioning generation into three modes: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output only). The inference engine parses these declarations like tool calls and skips most KV cache reads. Zero-shot on off-the-shelf models (Gemma-4-31B, Qwen-3.6-27B): 52% and 31% reduction in total attended tokens with modest accuracy drops (1.27pp, 2.75pp).

**NeoTrix mapping:**
- NT-CORE (GWT): Declarative Attention = GWT attention routing where modules declare their attention scope; the three modes map to GWT's salience tiers (broadcast vs targeted vs local)
- NT-MIND: Attention scope declarations = experience-tree loading strategy; declaring `<focus>` = lazy-load specific branch, `<local>` = use cached context, `<global>` = full KB scan
- NT-IO: Parsing attention declarations as "tool calls" = treating attention routing as a first-class action; the inference engine becomes an attention scheduler
- NT-REPAIR: Skipping KV cache reads = self-healing memory optimization; when attention is declared as `<local>`, the system avoids loading distant context

**Novel Signal:** DA proves models can self-declare attention scope without training. NeoTrix's GWT could adopt this pattern: instead of computing salience for all modules, let modules declare their attention scope (`<global>` for full reasoning, `<focus>` for targeted analysis, `<local>` for I/O operations). This would reduce GWT broadcast overhead by 30-50% while maintaining accuracy.

---

### 6. LISA — Linear-Indexed Sparse Attention for Long-Context Reasoning
| Field | Value |
|-------|-------|
| **Platform** | arXiv Jul 2026 (ICML 2026) |
| **Stars** | N/A (paper) |
| **Repo** | N/A |
| **Lang** | Python |

**What it does:** Plug-and-play attention replacement module: Linear Attention (O(n) long-range memory) + Lightning Indexer (top-M token selector) + Sparse Self-Attention (precise local retrieval). Gating mechanism fuses both branches. Two-stage training: Stage 1 initializes linear attention + sliding-window distillation; Stage 2 introduces indexer with per-head KL divergence loss. 50% inference speedup at 16K context, +5.6% accuracy on reasoning benchmarks.

**NeoTrix mapping:**
- NT-CORE: Linear Attention (global state) + Sparse Attention (precise retrieval) = E8 hexagram global reasoning + GWT targeted attention; the dual-branch architecture mirrors E8+GWT separation
- NT-MEMORY: Lightning Indexer (top-M token selection) = experience-tree route-table matching; the indexer is a learned version of NeoTrix's route-table
- NT-MIND: Per-head KL divergence loss = SEAL distillation where each head learns from different teacher aspects; the multi-head mechanism parallels domain-specific distillation
- NT-REPAIR: Gating mechanism (adaptive branch selection) = self-healing where the system switches between global and local reasoning based on context

**Novel Signal:** LISA's dual-branch architecture (linear attention for long-range + sparse attention for precise retrieval) is a direct analog for NeoTrix's E8 (global reasoning) + GWT (targeted attention) separation. The Lightning Indexer (learned top-M selector) could replace NeoTrix's hand-crafted route table with a trained selector, reducing experience lookup from heuristic matching to learned attention.

---

### 7. ParaTempo — Asynchronous Parallel Reasoning via Temporal Confidence
| Field | Value |
|-------|-------|
| **Platform** | arXiv Aug 2026 |
| **Stars** | N/A (paper) |
| **Repo** | [ScottZhang812/ParaTempo](https://github.com/ScottZhang812/ParaTempo) |
| **Lang** | Python |

**What it does:** Training-free asynchronous parallel reasoning framework. Core signal: temporal confidence — a branch-local measure of answer-space convergence over time. Each branch probed for tentative answer distribution; temporal confidence measures how sharply recent probes concentrate on a dominant answer. Branches pruned when low-confidence, retired when converged, freed computation reallocated by forking. No synchronization needed between branches. 21.8–32.2% latency reduction, 18.1–30.3% token reduction with competitive accuracy.

**NeoTrix mapping:**
- NT-CORE: Temporal confidence = ConsciousnessTree's phi convergence metric; branches = parallel E8 reasoning paths; pruning low-confidence branches = GWT attention pruning
- NT-MIND: Branch retirement when converged = SEAL cycle early termination; freed computation reallocated = dynamic SEAL pipeline scaling
- NT-REPAIR: Adversarial branch probing (from SABER, related work) = SelfTest mutation testing; probing branches with perturbations to test robustness
- NT-ACT: Fork-on-convergence = NT-ACT's parallel task manager allocating freed GPU to new tasks

**Novel Signal:** ParaTempo's temporal confidence is a more stable signal than token-level confidence for deciding when reasoning has converged. NeoTrix's ConsciousnessTree could adopt this: instead of checking phi at fixed intervals, use temporal convergence (phi changes over time) to decide when a growth cycle is complete. The "fork freed computation" pattern is directly applicable to NeoTrix's resource budget management.

---

### 8. Flux Attention — Context-Aware Hybrid Attention Routing
| Field | Value |
|-------|-------|
| **Platform** | arXiv Apr 2026 (ICML 2026) |
| **Stars** | N/A (paper) |
| **Repo** | N/A |
| **Lang** | Python |

**What it does:** Lightweight Layer Router that assigns each transformer layer to Full Attention or Sparse Attention based on input context. Layer-level routing (not head-level) preserves contiguous memory access, avoiding synchronization long-tails. Gumbel-Softmax for differentiable training; hard routing at inference. Only 12 hours training on 8×A800 GPUs. 2.8× prefill speedup, 2.0× decode speedup at 256K context. Router decision cached per layer, reused across all decoding steps.

**NeoTrix mapping:**
- NT-CORE (GWT): Layer Router = GWT attention router; layer-level (not module-level) routing = coarse-grained attention allocation; Gumbel-Softmax = differentiable salience scoring
- NT-IO: Router decision caching = GWT salience precomputation; compute once during prefill, reuse across generation steps
- NT-MIND: Context-aware routing = SEAL pipeline adapts based on task complexity; full attention for hard reasoning, sparse for I/O
- NT-SHIELD: Layer-level isolation = domain-level capability isolation; each layer/domain has its own attention budget

**Novel Signal:** Flux Attention proves layer-level routing (coarse-grained) outperforms head-level routing (fine-grained) due to hardware efficiency. NeoTrix's GWT should route at the domain level (coarse), not the module level (fine). The "router decision cached per layer, reused across steps" pattern maps to GWT computing salience once per ConsciousnessTree cycle, not per sub-task.

---

### 9. GitNexus — Knowledge Graph Kernel for Coding Agents
| Field | Value |
|-------|-------|
| **Platform** | ProductHunt Aug 2026 (YC-backed) |
| **Stars** | ~45K |
| **Repo** | [Akon Labs/GitNexus](https://akonlabs.com) |
| **Lang** | TypeScript |

**What it does:** Knowledge Graph Kernel that unifies every codebase in your org into one source of truth for coding agents. Resolves code into a deterministic graph (not embeddings), so agents get exact callers, imports, and impact. Works over MCP. On public benchmarks: coding agent runs 51% cheaper with GitNexus connected. Handles cross-repo and cross-SCM queries.

**NeoTrix mapping:**
- NT-MEMORY: Deterministic graph (not embeddings) = KB's structural index; GitNexus proves that for code understanding, exact graph traversal beats vector similarity
- NT-CORE: Knowledge graph as "source of truth" = KB's node-edge-embedding model; the deterministic graph approach validates NeoTrix's KB schema over naive vector search
- NT-ACT: 51% cost reduction via graph-guided context = capability routing via structural relationships vs embedding similarity
- NT-WORLD: Cross-repo, cross-SCM = multi-source perception; the graph abstraction unifies heterogeneous data sources

**Novel Signal:** GitNexus's core insight — deterministic graph > embedding similarity for code understanding — challenges the assumption that vector search is always superior. NeoTrix's KB already uses node-edge structure; GitNexus validates this approach and suggests doubling down on structural relationships for code-related queries, while reserving embeddings for semantic search.

---

### 10. PMB — Local-First Persistent Memory for AI Coding Agents
| Field | Value |
|-------|-------|
| **Platform** | ProductHunt Jun 2026 |
| **Stars** | ~1.2K |
| **Repo** | [pmb-ai](https://github.com/pmb-ai/pmb) |
| **Lang** | Python |

**What it does:** Persistent project memory for Claude Code, Cursor, Codex, Zed via MCP. SQLite workspace on disk — no cloud, no API keys, no LLM calls on read path. Typed memory: lessons (rules), goals, recent activity, project facts, code-entity graph. Hybrid retriever: BM25 + vectors + entity graph, fused. Recency + forgetting-curve decay, correction override (high-priority lessons outrank contradictions), keyed facts (latest-wins with old values archived), 4-layer dedup. Append-only store with ULID event IDs. Multi-agent safe via WAL mode + busy-timeout serialization.

**NeoTrix mapping:**
- NT-MEMORY: Direct analog — PMB is a single-user version of NeoTrix's KB; the typed memory (lessons/goals/facts) maps to KB's node types; forgetting-curve decay = experience-tree freshness scoring
- NT-CORE: Correction override (high-priority lessons outrank contradictions) = E8 hexagram conflict resolution; when a new reasoning contradicts old, the new takes precedence
- NT-REPAIR: Stale decision detection (planned: auto-flag conflicting decisions) = self-healing when KB entries become outdated; the "needs-review" surface is a repair trigger
- NT-ACT: Append-only store with event IDs = EventBus event sourcing; WAL mode = concurrent-safe KB writes

**Novel Signal:** PMB's "typed memory with forgetting-curve decay" is the cleanest implementation of persistent agent memory in the wild. The "correction override" pattern (corrected agent behavior stored as high-priority lesson) directly improves experience-tree quality: when an agent makes a mistake and is corrected, that correction should outrank the original behavior in KB recall. The 4-layer dedup (near-identical merge, keyed latest-wins, correction override, session-scoped isolation) is a production-ready dedup strategy for NeoTrix's experience-tree.

---

## Cross-Cutting Trends (Cycle 428)

| Trend | Projects | NeoTrix Implication |
|-------|----------|---------------------|
| **Self-Evolution as First-Class** | OmniAgent (3-dim), DeerFlow (skill loading), PMB (memory decay) | NeoTrix's SEAL + experience-tree + ConsciousnessTree triple evolution is competitive; add online RL evolution path |
| **Layer-Level Routing > Head-Level** | Flux Attention, LISA, BeaconKV | GWT should route at domain level (coarse), not module level (fine); hardware efficiency wins |
| **World Models as Training Objective** | Qwen-AgentWorld | NT-WORLD perception should be trained to predict next-state, not just classify current-state |
| **Temporal Convergence > Token Confidence** | ParaTempo, ASAG | ConsciousnessTree phi should use temporal convergence (phi-over-time) for cycle completion |
| **Deterministic Graph > Embedding Similarity** | GitNexus, PMB | KB structural index for code queries; embeddings for semantic search only |
| **Skill as Deployable Capability** | OmniAgent, SkillSpector, DeerFlow | SKILL.md contract + security scan before activation; NeoTrix's skill tree needs this gate |

---

## Absorption Priority

| Priority | Pattern | Source | Target Domain |
|----------|---------|--------|---------------|
| P0 | Temporal confidence for convergence detection | ParaTempo | NT-CORE (ConsciousnessTree) |
| P0 | Layer-level attention routing (not head-level) | Flux Attention | NT-CORE (GWT) |
| P1 | KV cache compression via beacon queries | BeaconKV | NT-MEMORY (KB hub) |
| P1 | Typed memory with forgetting-curve decay | PMB | NT-MEMORY (experience-tree) |
| P1 | Deterministic graph > embeddings for code | GitNexus | NT-MEMORY (KB structural index) |
| P2 | Full-dimensional self-evolution (skill+context+brain) | OmniAgent | NT-MIND (SEAL) |
| P2 | World model as primary training objective | Qwen-AgentWorld | NT-WORLD (PerceptionBridge) |
| P2 | Declarative attention scope (global/focus/local) | DA | NT-CORE (GWT broadcast) |
