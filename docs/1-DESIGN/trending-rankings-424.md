# Trending Rankings — Cycle 424 (2026-09-12)

## Methodology
- GitHub trending (AI agents, LLM tools, reasoning frameworks)
- ProductHunt September 2026 launches
- arXiv September 2026 papers
- Cross-referenced against cycles 318–423 to ensure novelty

---

## Top 10 New Projects

### 1. OmniAgent — Full-Dimensional Self-Evolving Agent Framework
| Field | Value |
|-------|-------|
| **Repo** | [YeQing17-2026/OmniAgent](https://github.com/YeQing17-2026/OmniAgent) |
| **Stars** | 2,557 |
| **Lang** | Python |
| **Created** | 2026-04-16 |

**What it does:** Open-source self-evolving Agent inspired by OpenClaw. Implements full-dimensional self-evolution (OmniEvolve): proactive memory (dual-path alignment), skill self-evolution (auto-create/inspect/repair), context self-evolution (real-time interaction feedback), and brain model self-evolution (online RL via GRPO+PRM). Includes Hyper-Harness (progressive context loading, dynamic multi-agent Sentinel/Guardian, four-layer security scanning) and Deep Reflexion (inner-outer dual-layer reflective architecture with RCA and failure-to-insight conversion).

**NeoTrix mapping:**
- NT-MIND: Skill self-evolution mirrors SEAL pipeline skill crystallization
- NT-CORE: Dual-layer reflexion maps to ConsciousnessTree feedback loop
- NT-SHIELD: Four-layer dynamic security scanning → egress privacy guard + sandbox policy
- NT-NEXUS: Proactive memory with dual-path alignment → cross-session knowledge weaving

---

### 2. PrimeAgent — Self-Improving RLM Agent
| Field | Value |
|-------|-------|
| **Repo** | [PrimeIntellect-ai/prime-agent](https://github.com/PrimeIntellect-ai/prime-agent) |
| **Stars** | 1,456 |
| **Lang** | Python |
| **Created** | 2026-05-08 |

**What it does:** Self-improving coding/research agent built on Recursive Language Model (RLM) abstraction — treats context as variables (prompt-as-a-variable) and tools as recursive subagents in a persistent REPL. Continual Harness stores supplemental prompts, memories, skill descriptions as durable state refined through evidence-backed updates. Features: subagents via `rlm(...)`, `/refine` for harness improvement, daemon-backed sessions, heartbeats, persistent goals, bounded autonomous mode.

**NeoTrix mapping:**
- NT-CORE: RLM as prompt-as-variable → VSA HyperCube symbolic embedding
- NT-MIND: Continual Harness refinement → SEAL phase evolution
- NT-ACT: Subagent spawning → capability registry orchestration
- NT-MEMORY: Durable harness state → KB experience persistence

---

### 3. AtomicAgent — Local-First AI Agent
| Field | Value |
|-------|-------|
| **Repo** | [AtomicBot-ai/atomic-agent](https://github.com/AtomicBot-ai/atomic-agent) |
| **Stars** | 2,432 |
| **Lang** | TypeScript |
| **Created** | 2026-04-21 |

**What it does:** Local-first agent with TurboQuant llama.cpp (+30-50% throughput on small local models). Drives browser, edits files, runs approved commands, remembers context across sessions. GAIA-L1 benchmark: 69.8% accuracy with qwen-3.6-35b-a3b, beating Hermes (58.5%). Features: browser automation via playwright-core, web/HTTP with SSRF guards, document extraction, memory (profile facts, notes, reasoning chains, lessons), MCP integration.

**NeoTrix mapping:**
- NT-PHYSICAL: Local-first runtime → embodied agent skeleton
- NT-WORLD: Browser automation → UnifiedCrawler perception
- NT-MEMORY: Hybrid recall memory → KB knowledge graph
- NT-SHIELD: SSRF guards → sandbox egress policy

---

### 4. BudgetMem — Runtime Agent Memory with Budget-Tier Routing
| Field | Value |
|-------|-------|
| **Paper** | [arXiv:2609.02737](https://arxiv.org/abs/2609.02737) — ICML'26 |
| **Repo** | [ViktorAxelsen/BudgetMem](https://github.com/ViktorAxelsen/BudgetMem) |
| **Stars** | 23+ |

**What it does:** Runtime agent memory framework with explicit performance-cost control. Each memory extraction module exposes three budget tiers (Low/Mid/High) along three axes: implementation tiering (heuristic→task-specific→LLM), reasoning tiering (direct→CoT→multi-step), capacity tiering (small→medium→large). A learned budget-tier router selects tiers module-wise via RL under cost-aware objective. Evaluated on LoCoMo, LongMemEval, HotpotQA.

**NeoTrix mapping:**
- NT-MEMORY: Budget-tier memory → KB embedding with cost-aware retrieval
- NT-CORE: Cost-aware routing axiom (A1) — GWT salience + token cost weight
- NT-MIND: Module-level tiering → SEAL pipeline stage budgeting

---

### 5. Declarative Attention — Model-Controlled Sparse Attention
| Field | Value |
|-------|-------|
| **Paper** | [arXiv:2609.02737](https://arxiv.org/abs/2609.02737) |
| **Published** | 2026-09-02 |

**What it does:** Protocol that elicits LLMs to declare where they need to attend within chain-of-thought, partitioning generation into three modes: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output only). Inference engine parses declarations like tool calls and skips most KV cache reads. Zero-shot on Gemma-4-31B: 52% reduction in attended tokens with 1.27pp accuracy drop. On Qwen-3.6-27B: 31.1% reduction with 2.75pp drop.

**NeoTrix mapping:**
- NT-CORE: Self-declared attention routing → GWT salience modulation
- NT-MEMORY: KV cache skip → KVMem-style paged KV virtualization
- NT-MIND: Intrinsic attention control → meta-cognitive self-regulation

---

### 6. ReActNet — Inference-Time Graph Engineering for Multi-Agent Workflows
| Field | Value |
|-------|-------|
| **Paper** | [arXiv:2609.05774](https://arxiv.org/abs/2609.05774) |
| **Published** | 2026-09-04 |

**What it does:** Training-free framework that compiles query + role-specialized agents into a sequence of directed communication graphs. Each snapshot = one reasoning stage, each edge = natural-language instruction for message passing. Separates graph compilation from graph execution. Consistently improves over fixed-topology and learned-topology baselines on knowledge reasoning, math, code, and GAIA tasks.

**NeoTrix mapping:**
- NT-ACT: Temporal graph compilation → capability registry orchestration
- NT-CORE: Reasoning stage snapshots → ConsciousnessTree cycle phases
- NT-NEXUS: Edge-level communication semantics → cross-session pattern propagation

---

### 7. Codebook Agent — Amortized Topology Design for Multi-Agent Systems
| Field | Value |
|-------|-------|
| **Paper** | [arXiv:2609.02264](https://arxiv.org/abs/2609.02264) |
| **Published** | 2026-09-02 |

**What it does:** Vector-quantized autoencoder compresses successful topologies into 16-entry codebook. Reward-weighted MLP maps query embedding to distribution over codes. Topology in 2.4ms, 21.9-33.2% fewer tokens than prior methods. Key insight: topologies collapse to ~6 distinct graphs even at 64-entry capacity; edge count negatively correlates with token cost.

**NeoTrix mapping:**
- NT-CORE: Topology codebook → E8 hexagram state compression
- NT-ACT: Query-conditioned routing → GWT cost-aware dispatch
- NT-MIND: Amortized topology evolution → SEAL pattern crystallization

---

### 8. Speculative Macro Commit — Faster Tool-Using Agents
| Field | Value |
|-------|-------|
| **Paper** | [arXiv:2609.03236](https://arxiv.org/abs/2609.03236) |
| **Published** | 2026-09-03 (MLSP2026) |

**What it does:** Two-tier agent system: large authoritative model + fast speculative drafter. Drafter predicts and executes future action chains on isolated environment snapshot. Mines recurring multi-action skeletons from traces into macro library. When actor's next call matches drafted action, commits pre-executed steps. 18.59% latency reduction over sequential on τ-Bench, 44.9% on AppWorld.

**NeoTrix mapping:**
- NT-ACT: Speculative execution → NT-ACT tool calling optimization
- NT-CORE: Macro library → E8 pattern memory
- NT-MIND: Trace mining → SEAL experience crystallization

---

### 9. GitNexus (Akon Labs) — Knowledge Graph Kernel for Coding Agents
| Field | Value |
|-------|-------|
| **Platform** | ProductHunt Aug 2026 |
| **Stars** | 45,000+ (GitHub) |

**What it does:** Knowledge graph kernel that unifies every codebase in an org into one source of truth for coding agents. Resolves code into deterministic graph — exact callers, imports, impact — instead of embedding guesses. Works across every repo and SCM. 51% cheaper coding agent runs with GitNexus connected. MCP-native.

**NeoTrix mapping:**
- NT-MEMORY: Knowledge graph kernel → KB graph structure
- NT-WORLD: Cross-SCM codebase indexing → UnifiedCrawler perception
- NT-ACT: MCP-native integration → capability registry pattern

---

### 10. Harden AIF — Security Layer for AI Coding Agents
| Field | Value |
|-------|-------|
| **Platform** | ProductHunt Sep 9, 2026 (#2 Product of Day) |
| **Upvotes** | 399 |

**What it does:** Free, local security tool for AI coding agents. Post-trained model checks tool calls before they run, using request and session context. Beat frontier models on agent-security benchmarks. Keeps repo and tool output local. Single logic source for agent tool-call validation.

**NeoTrix mapping:**
- NT-SHIELD: Tool-call pre-validation → egress privacy guard + sandbox policy
- NT-SHIELD: Post-trained security model → fingerprint management + audit
- NT-CORE: Context-aware validation → ConsciousnessTree health monitoring

---

## Cross-Cutting Trends (Cycle 424)

| Trend | Signal Strength | NeoTrix Impact |
|-------|----------------|----------------|
| **Self-evolving agents** | 🔴 Very High | OmniAgent, PrimeAgent — validates SEAL pipeline direction |
| **Budget-aware memory** | 🔴 Very High | BudgetMem — aligns with A1 Cost-Aware Routing axiom |
| **Local-first execution** | 🟠 High | AtomicAgent, LocalAI — validates NT-PHYSICAL embodiment |
| **Graph topology engineering** | 🟠 High | ReActNet, CodebookAgent — maps to E8 hexagram routing |
| **Speculative execution** | 🟡 Medium | SMC — potential for NT-ACT tool optimization |
| **Security-first agents** | 🟠 High | Harden AIF, OmniAgent Hyper-Harness — validates NT-SHIELD |
| **Knowledge graph kernels** | 🟡 Medium | GitNexus — validates KB graph-first architecture |
| **Declarative attention** | 🔴 Very High | DA protocol — intrinsic attention control for GWT |

---

## Absorption Candidates (Priority Order)

| # | Project | Absorption Pattern | Target Domain |
|---|---------|-------------------|---------------|
| 1 | **BudgetMem** | Budget-tier memory routing | NT-MEMORY + NT-CORE (A1) |
| 2 | **Declarative Attention** | Intrinsic attention self-declaration | NT-CORE (GWT) |
| 3 | **Codebook Agent** | Topology compression codebook | NT-CORE (E8) |
| 4 | **ReActNet** | Temporal graph compilation | NT-ACT orchestration |
| 5 | **OmniAgent** | Full-dimensional self-evolution | NT-MIND (SEAL) |
| 6 | **Speculative Macro Commit** | Macro library + drafter | NT-ACT optimization |
| 7 | **Harden AIF** | Tool-call pre-validation | NT-SHIELD |
