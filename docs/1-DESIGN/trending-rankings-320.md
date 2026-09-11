# Trending Rankings — Cycle 320

**Date**: 2026-09-11
**Focus**: AI agents, multi-agent orchestration, security, efficient inference, self-evolution

---

## 10 New Projects (Not in Cycles 318-319)

### 1. Harden AIF (Agent Integrity Foundation)
- **GitHub**: https://github.com/hardenrun/aif
- **Stars**: ~5,000+ (PH #2 Product of the Day, Sep 9 2026)
- **Description**: Local-first AI firewall for coding agents. Post-trained 8B cybersecurity LLM runs on-device, checks every tool call before execution against developer intent and session context. Beats frontier models (GPT-5.5) on AgentHazard (83.7% vs 81.4%) and LinuxArena (29% vs 34% undetected sabotage). Supports Claude Code, Codex, Cursor, Gemini CLI, Kiro, OpenClaw, Hermes. Four outcomes: allow, ask, rewrite-to-safe, block. Zero-friction single-command install.
- **Key Pattern**: **Pre-execution interception with contextual judgment** — not regex rules but intent-aware security model that reasons across code, dependencies, and session history. Local-first architecture keeps source code on-device.
- **NeoTrix Relevance**: Directly maps to NT-SHIELD (agent security, tool-call interception). The "session + intent aware" pattern is relevant to Egress Privacy Guard — our outbound filter should evaluate tool calls against user intent, not just static rules. The 8B local model approach aligns with our self-hosted architecture. Could be the security layer for NT-ACT tool execution.

### 2. Mastra
- **GitHub**: https://github.com/mastra-ai/mastra
- **Stars**: ~30,000+ (PH #1 Product of the Day, Sep 9 2026)
- **Description**: TypeScript AI agent framework from the Gatsby team. Agents, graph-based workflows, memory, human-in-the-loop, MCP servers, observability. Workflow engine with `.then()`, `.branch()`, `.parallel()` composition. Mastra Studio for dev/test. Deploys to Vercel/Netlify/Cloudflare or standalone Hono server. TypeScript-native with full type inference.
- **Key Pattern**: **Typed workflow composition with suspend/resume** — workflows are first-class citizens with explicit control flow, not agent loops. Human-in-the-loop via storage-backed suspend/resume. Observability built-in, not bolted on.
- **NeoTrix Relevance**: The workflow composition pattern maps to SEAL pipeline stage orchestration. The suspend/resume via persistent state parallels our KB-backed state management. The TypeScript-first approach is relevant to NT-IO (web/API layer). Mastra Studio UI pattern could inform our tooling.

### 3. OmniAgent
- **GitHub**: https://github.com/YeQing17-2026/OmniAgent
- **Stars**: ~2,500+ (growing fast)
- **Description**: Self-evolving agent framework implementing full-dimensional self-evolution (OmniEvolve): Skill (real-time creation/repair), Context (multi-layer information stack), BrainModel (online RL via GRPO+PRM). Hyper-Harness with dynamic multi-agent (Sentinel planner + Guardian safety) and four-layer dynamic security scanning (LLM review → Policy engine → Interactive approval → Execution sandbox). Deep Reflexion dual-layer: inner failure prevention + outer failure-to-insight conversion.
- **Key Pattern**: **Full-dimensional self-evolution (Skill + Context + BrainModel)** — not just skill trees but parallel evolution across three dimensions simultaneously. The "Hyper-Harness" pattern of dynamically spawning Sentinel/Guardian agents based on task complexity is novel.
- **NeoTrix Relevance**: OmniEvolve is a direct analog to our SEAL pipeline + experience-tree. The three-dimensional evolution (Skill/Context/BrainModel) maps to our skill crystallization (NT-MIND), KB evolution (NT-MEMORY), and SelfModel tuning (NT-CORE). The dynamic Sentinel/Guardian agents parallel our ConsciousnessTree branch activation. The "progressive context loading" (L0/L1/L2) mirrors our GWT attention tiers.

### 4. Qwen-AgentWorld
- **GitHub**: https://github.com/qwenlm/qwen-agentworld
- **Stars**: ~984 (recently released)
- **Description**: Native language world model for simulating agentic environments. 35B total / 3B active (MoE), 256K context. Covers 7 unified domains: MCP, Search, Terminal, SWE, Android, Web, OS. Three-stage training: CPT injects environment knowledge, SFT activates next-state-prediction reasoning, RL sharpens simulation fidelity. Trained on 10M+ real-world interaction trajectories. Qwen-AgentWorld-397B-A17B outperforms GPT-5.4 (58.71 vs 58.25 overall).
- **Key Pattern**: **Native world model as agent foundation** — environment modeling is the training objective from CPT onward, not a post-hoc add-on. Controllable perturbations and fictional-world construction generalize better than real-environment training. Zero-shot OOD generalization to unseen environments.
- **NeoTrix Relevance**: Maps to NT-WORLD (perception, environment modeling). The "native world model" concept aligns with our HyperCube knowledge representation — the world model should be intrinsic to the agent, not bolted on. The 7-domain coverage (MCP/Search/Terminal/SWE/Android/Web/OS) maps to our domain architecture. The controllable simulation pattern is relevant to our SEAL pipeline test environments.

### 5. nanobot
- **GitHub**: https://github.com/HKUDS/nanobot
- **Stars**: ~47,600+ (trending on trendshift)
- **Description**: Ultra-lightweight personal AI agent framework in Python. WebUI, terminal, or chat apps. Tools, long-term memory (Dream), MCP integrations, model routing, multi-agent delegation, scheduled automation, OpenAI-compatible API. v0.3.0 "Agency Release" adds inline subagents, model switching per session, parallel search. Connects to Telegram, Discord, Slack, WeChat, Email, Mattermost.
- **Key Pattern**: **Small readable core with progressive disclosure** — starts minimal, adds complexity on demand. "Dream" memory system for long-term persistence. Model switching per session. The "Agency Release" pattern of turning a workbench into an agent runtime is a clean evolution path.
- **NeoTrix Relevance**: The lightweight architecture pattern (small core + progressive complexity) aligns with our modular design philosophy. The "Dream" memory system parallels our KB experience persistence. The multi-platform chat integration maps to NT-IO interface layer. The model-switching-per-session is relevant to Axiom A1 (Cost-Aware Routing).

### 6. AGAO (Adaptive Goal-aware Attention Orchestration)
- **GitHub**: https://github.com/MingzhouFan97/AGAO
- **Stars**: ~200+ (academic, recent)
- **Description**: Framework that extends attention from token-level to workflow-level agent coordination. Three complementary mechanisms: (1) Goal-aware attention — semantic relevance between user goals and agent capabilities; (2) Topology-aware attention — structural dependencies within agent graphs; (3) Resource-aware attention — adaptive computational budgets among heterogeneous agents. Transforms static agent graphs into adaptive execution systems.
- **Key Pattern**: **Attention as execution-level control mechanism** — attention isn't just for token representation learning, it's for workflow-level coordination. Goal/topology/resource triple attention mechanism. Agents as dynamically selectable computational units.
- **NeoTrix Relevance**: This is a direct analog to our GWT attention routing. Goal-aware attention maps to GWT salience (user intent → attention allocation). Topology-aware attention maps to ConsciousnessTree branch dependencies. Resource-aware attention maps to Axiom A1 (Cost-Aware Routing). The "Attention Engineering" paradigm validates our GWT-first architecture.

### 7. ReActNet
- **Paper**: https://arxiv.org/abs/2609.05774 (Sep 2026)
- **Description**: Training-free framework for multi-agent LLM workflows. Compiles query + role-specialized agents into a sequence of directed communication graphs. Each graph snapshot = one reasoning stage, each edge = natural-language instruction for message content. Separates graph compilation from execution. Agents update reasoning states via structured message passing. No RL or gradient-based topology optimization needed.
- **Key Pattern**: **Temporal graph compilation + execution separation** — the workflow graph is synthesized per-query, not fixed. Edges carry natural-language instructions, making coordination explicit and inspectable. Separating compilation from execution enables reasoning-time optimization.
- **NeoTrix Relevance**: The temporal graph pattern maps to our SEAL pipeline — different stages (compilation → execution) with explicit state passing. The "natural-language instruction edges" pattern is relevant to our skill node composition — skills communicate via structured messages. The training-free approach aligns with our self-evolution philosophy (learn from structure, not gradient updates).

### 8. Procedural Graphs
- **Paper**: https://arxiv.org/abs/2609.09153 (Sep 2026)
- **Description**: Self-evolving execution structures for LLM agents. Organizes procedural knowledge into (procedure, relation, procedure) triplets — what-to-do knowledge graph. At each step, a guidance model translates the surrounding subgraph into step-level situational guidance that biases (not dictates) the solver's next action. Graph self-evolves: an LLM refiner contrasts failed vs successful trajectories and edits topology/attributes, committing edits that preserve validation performance.
- **Key Pattern**: **Procedural knowledge graphs as execution scaffolding** — procedural knowledge (what-to-do) organized like factual knowledge (what-is). Self-evolving via failure-success contrast. Guidance model biases action without dictating — preserving agent autonomy.
- **NeoTrix Relevance**: Directly maps to NT-MIND skill crystallization — procedural knowledge should be graph-structured, not linear. The self-evolving topology via failure-success contrast is exactly our SEAL pipeline feedback loop. The "bias without dictating" pattern is relevant to GWT — salience influences attention but doesn't hard-route. The (procedure, relation, procedure) triplet maps to our skill node edges.

### 9. PARSER (Parallel Access Scatter-gather for Efficient Retrieval)
- **Paper**: https://arxiv.org/abs/2609.06702 (Sep 2026)
- **Description**: Decouples reading from reasoning for long-context agents. A bank of lightweight subagents each bound to a single chunk read the entire document in parallel. Lead agent reasons in depth through iterative scatter-gather rounds: broadcasts query to all subagents, aggregates evidence, formulates deeper follow-up queries. Subagents remain frozen off-the-shelf models. Lead agent optimized with RL. 4B backbone outperforms strongest sequential baseline by 5.7 points avg (12.0 at 896K tokens). 9B backbone surpasses DeepSeek-V4-Pro by 6.3 points. 11x latency reduction.
- **Key Pattern**: **Parallel read + serial deep reason** — reading is embarrassingly parallel, reasoning is sequential. Frozen subagents + trained lead agent separates concerns cleanly. Scatter-gather rounds enable iterative deepening — each round asks a deeper question based on accumulated evidence.
- **NeoTrix Relevance**: Maps to NT-WORLD perception layer — parallel chunk reading for large documents. The scatter-gather pattern parallels our GWT broadcast → specialist response → aggregation cycle. The "frozen subagents + trained lead" pattern is relevant to our Dual Specialization — cheap frozen workers + expensive reasoning lead. The 896K token handling is relevant to KVMem paged KV strategy.

### 10. TROVE (Trace-grounded Route Orchestration via Validation and Editing)
- **Paper**: https://arxiv.org/abs/2609.05019 (Sep 2026)
- **Description**: Adaptive agent skill orchestration that revises only what runtime evidence invalidates. Offline: distills evaluated workflow-search traces into atomic/composite skills and outcome-conditioned transition graph. Online: treats planned route as provisional — after committing one skill, controller retains valid continuation, inserts trace-supported local response, or replaces only the invalid suffix. Partial execution preserved, not discarded.
- **Key Pattern**: **Selective suffix replacement** — when evidence invalidates a plan, don't restart from scratch. Replace only the invalid suffix while preserving unaffected progress. Outcome-conditioned transition graph enables conditional branching based on runtime results.
- **NeoTrix Relevance**: Directly relevant to NT-REPAIR (self-healing) — when a plan fails, preserve valid progress and repair only the broken part. The outcome-conditioned transition graph maps to our SEAL pipeline stage transitions (learned from execution traces). The "provisional routes" pattern is relevant to GWT — attention plans are provisional, adjusted based on runtime evidence. The composite skills pattern maps to our skill node composition.

---

## Cross-Cutting Patterns

| Pattern | Projects | NeoTrix Mapping |
|---------|----------|-----------------|
| **Self-evolving structures** | OmniAgent, Procedural Graphs, TROVE | NT-MIND SEAL pipeline, skill crystallization |
| **Attention as coordination** | AGAO, ReActNet, Declarative Attention (319) | GWT salience, ConsciousnessTree |
| **Pre-execution security** | Harden AIF | NT-SHIELD tool-call interception, Egress Privacy Guard |
| **Parallel read + serial reason** | PARSER, nanobot parallel search | NT-WORLD perception, GWT broadcast |
| **Typed workflow composition** | Mastra, ReActNet | SEAL pipeline, skill node orchestration |
| **Selective repair over restart** | TROVE, REVISE (related work) | NT-REPAIR self-healing, experience-tree |
| **Native world models** | Qwen-AgentWorld | NT-WORLD perception, HyperCube |
| **Local-first architecture** | Harden AIF, nanobot, OmniAgent | Self-hosted philosophy, privacy |

---

## Priority Absorption Candidates

1. **Harden AIF** — Pre-execution intent-aware security (NT-SHIELD integration)
2. **AGAO** — Triple attention mechanism (Goal/Topology/Resource) validates GWT
3. **Procedural Graphs** — Self-evolving procedural knowledge graphs (NT-MIND)
4. **TROVE** — Selective suffix replacement for agent self-healing (NT-REPAIR)
5. **PARSER** — Parallel read + serial deep reason pattern (NT-WORLD)
