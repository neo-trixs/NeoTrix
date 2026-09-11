# Trending Rankings — Cycle 338

**Date**: 2026-09-11
**Sources**: GitHub Trending, ProductHunt, ossinsight.io, arXiv, awesome-ai-agents-2026

## 10 New Projects (Not in Cycles 318-337)

### 1. OmniAgent (YeQing17-2026)
- **URL**: https://github.com/YeQing17-2026/OmniAgent
- **Stars**: 2.5K+ | **License**: Other
- **Category**: Full-Dimensional Self-Evolving Agent Framework
- **Pattern**: OmniEvolve — real-time self-evolution during execution (Skill + Context + BrainModel). Hyper-Harness with dynamic multi-agent (Sentinel planning + Guardian safety) and concurrent tool execution. Deep Reflexion — inner-outer dual-layer reflective architecture for failure-to-insight conversion. Four-layer security scanning (LLM review → policy engine → interactive approval → sandbox). Progressive Context Loading (L0/L1/L2).
- **NeoTrix Mapping**: NT-MIND + NT-SHIELD — OmniEvolve = SEAL pipeline with real-time adaptation (not just post-execution). Deep Reflexion maps to ConsciousnessTree's feedback loops. Four-layer security = NT-SHIELD graduated trust model. **Absorption candidate**: real-time skill evolution during execution + progressive context loading stages.

### 2. Prime Agent (PrimeIntellect-ai)
- **URL**: https://github.com/PrimeIntellect-ai/prime-agent
- **Stars**: 1.5K+ | **License**: MIT
- **Category**: Self-Improving RLM Agent for Long-Running Tasks
- **Pattern**: Recursive Language Model (RLM) — context as variables, tools as recursive subagents, persistent REPL. Continual Harness stores supplemental prompts, memories, skills, subagent specs as durable state refined via `/refine`. Agent-to-agent direct communication without user routing. Daemon-backed continuity — sessions survive terminal disconnect. Bounded autonomous mode with quality gates.
- **NeoTrix Mapping**: NT-ACT — RLM = persistent execution context (maps to production orchestrator). Continual Harness = AGENTS.md-style persistent config refined at runtime. Agent-to-agent messaging = EventBus direct routing. **Absorption candidate**: evidence-backed harness refinement with rollback support + daemon-backed session continuity.

### 3. GitAgent (open-gitagent)
- **URL**: https://github.com/open-gitagent/gitagent
- **Stars**: 670 | **License**: MIT
- **Category**: Git-Native AI Agent Framework
- **Pattern**: Agent-as-repo — identity (agent.yaml + SOUL.md + RULES.md), memory (git-committed), tools (declarative YAML), skills (composable modules), hooks (lifecycle scripts) all version-controlled. MCP client — auto-discovers any MCP server tools. Skills as importable Python packages.
- **NeoTrix Mapping**: NT-MEMORY + NT-ACT — git-committed memory = version-controlled KB snapshots. Agent-as-repo = domain module convention (nt_* directories). Skills as packages = SKILL.md contract pattern. **Absorption candidate**: declarative tool YAML + lifecycle hooks for agent state management.

### 4. PMB (Local-First Memory for AI Agents)
- **URL**: producthunt.com/products/pmb-local-first-memory-for-ai
- **Category**: Persistent Project Memory via MCP
- **Pattern**: Typed memory — lessons (rules), goals, recent work, keyed facts in one SQLite workspace. BM25 + vector + entity graph hybrid retriever with RRF fusion. Recency + forgetting-curve decay. Correction overrides (high-priority lessons outrank contradictions). Keyed facts latest-wins with old value archived. Dedup merges near-identical entries. Session-tagged provenance tracking.
- **NeoTrix Mapping**: NT-MEMORY — typed memory = experience-tree namespaces. Hybrid retriever = KB search pipeline. Forgetting-curve decay = experience relevance weighting. Correction overrides = KB versioning with supersede semantics. **Absorption candidate**: forgetting-curve decay model for experience relevance + entity graph for cross-experience relationships.

### 5. Timbal AI
- **URL**: producthunt.com/products/timbal-ai
- **Category**: Production AI Platform with Deterministic Runtime
- **Pattern**: ACE (Action Control Engine) — behavioral runtime as proxy providing deterministic layer. Agents/workflows/tools compile to clean Python code (no black box). Native evals, tracing, governance in runtime. Composer generates custom connectors for anything with API. ISO 27001, SOC 2 Type II, NIS2 compliance built in.
- **NeoTrix Mapping**: NT-ACT + NT-GOVERNANCE — ACE = deterministic execution sandbox (maps to nt_shield_sandbox). Code-as-source-of-truth = compile-to-Rust for agent workflows. Built-in governance = NT-GOVERNANCE compliance. **Absorption candidate**: behavioral runtime proxy pattern for deterministic agent execution.

### 6. oqoqo
- **URL**: producthunt.com/products/oqoqo
- **Category**: Agent Evaluation & Benchmarking Platform
- **Pattern**: Run eval experiments at scale in isolated sandboxes. Catalog every agent step including tool calls, retries, discovery loops. Measure token consumption, cost, success/failure. Create custom benchmarks for how agents discover and use products. Dynamic insights to detect product interface frictions or token inefficiencies.
- **NeoTrix Mapping**: NT-MIND — eval platform = SEAL pipeline validation layer. Agent step catalog = experience-tree detailed execution traces. Token efficiency metrics = SelfModel resource tracking. **Absorption candidate**: step-level execution tracing with cost attribution for SEAL phase evaluation.

### 7. GNAP — Git-Native Agent Protocol (farol-team)
- **URL**: https://github.com/farol-team/gnap
- **Category**: Git-Native Multi-Agent Coordination Protocol
- **Pattern**: Coordinate AI agent teams with 4 JSON files in a git repo. No server, no database. Any agent that can git push can participate. Agent state, task assignments, and coordination via git commits. Minimal infrastructure — coordination emerges from git operations.
- **NeoTrix Mapping**: NT-ACT + NT-MEMORY — git-as-coordination-bus maps to EventBus with git as transport. 4-JSON-file protocol = minimal state representation for agent coordination. **Absorption candidate**: git-native agent coordination for cross-session multi-agent workflows.

### 8. Arkor
- **URL**: producthunt.com/products/arkor
- **Category**: TypeScript Fine-Tuning & Deployment Platform
- **Pattern**: Fine-tune open-weight models in TypeScript. Claude Code/Codex prepare datasets and training projects. Local control surface with remote GPU training. OpenAI-compatible API deployment. Starting with Gemma 4 for classification, extraction, routing, rewriting tasks. No GPU setup, no ML expertise required.
- **NeoTrix Mapping**: NT-IO — model fine-tuning pipeline for NeoTrix-specific tasks. TypeScript-first = aligns with NT-IO web server stack. **Absorption candidate**: agent-driven fine-tuning workflow for domain-specific model adaptation.

### 9. NEXUS MCP Server (Senpai-Sama7)
- **URL**: https://github.com/Senpai-Sama7/nexus-mcp-server
- **Category**: Deep Code Intelligence MCP Server
- **Pattern**: 31 tools across 10 families: semantic symbol/reference/dependency graph, blast radius computation before changes, namespaced persistent memory, context budgeting (repo map, context packs), structured diagnostics parsing, deterministic parallel fan-out + persistent task DAGs, secret scanning + prompt injection guard. Zero native dependencies — pure JS regex parser for 20+ languages.
- **NeoTrix Mapping**: NT-ACT + NT-SHIELD — semantic graph = HyperCube knowledge representation for code. Blast radius = risk assessment before execution. Context budgeting = GWT salience for tool output. Persistent memory = KB execution history. **Absorption candidate**: blast-radius pre-computation before code edits + structured diagnostics from tool output.

### 10. Declarative Attention (arXiv:2609.02737)
- **URL**: https://arxiv.org/abs/2609.02737
- **Stars**: Paper (not a repo, but high-impact pattern)
- **Category**: Intrinsic Attention Control Protocol
- **Pattern**: Model declares where it needs to attend within its chain-of-thought. Three modes: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output only). Inference engine parses declarations like tool calls and skips most KV cache reads. Zero-shot: 52% reduction in attended tokens on Gemma-4-31B with 1.27pp accuracy drop.
- **NeoTrix Mapping**: NT-CORE (GWT) — model-driven attention allocation = consciousness choosing what to attend to. `<global>/<focus>/<local>` = GWT broadcast modes. Declaration-as-tool-call = consciousness output as actionable signal. **Absorption candidate**: three-mode attention protocol (global/focus/local) for GWT routing decisions.

## Cross-Cutting Themes (Cycle 338)

| Theme | Projects | NeoTrix Impact |
|-------|----------|----------------|
| **Self-Evolution During Execution** | OmniAgent, Prime Agent | SEAL pipeline should evolve skills in real-time, not just post-execution |
| **Git-as-Infrastructure** | GitAgent, GNAP | Git commits as memory snapshots and coordination protocol |
| **Typed Memory Systems** | PMB, OmniAgent | Experience-tree needs typed storage (lessons/goals/facts) with provenance |
| **Deterministic Execution** | Timbal, NEXUS | Behavioral runtime proxy for safe agent execution |
| **Agent Evaluation** | oqoqo | Step-level execution tracing for SEAL validation |
| **Declarative Attention** | Declarative Attention | Model-driven attention allocation for GWT routing |
| **Blast Radius Awareness** | NEXUS, OmniAgent | Pre-compute impact before execution changes |

## Prioritization

| Priority | Project | Action |
|----------|---------|--------|
| P0 | OmniAgent | Study real-time skill evolution + progressive context loading |
| P0 | Declarative Attention | Map three-mode protocol to GWT attention routing |
| P1 | PMB | Implement forgetting-curve decay + entity graph for experience KB |
| P1 | NEXUS | Study blast-radius computation + structured diagnostics |
| P1 | Prime Agent | Map Continual Harness refinement to AGENTS.md evolution |
| P2 | Timbal | Study ACE deterministic runtime for sandbox execution |
| P2 | oqoqo | Integrate step-level tracing into SEAL evaluation |
| P2 | GitAgent | Study declarative YAML tools + lifecycle hooks |
| P3 | GNAP | Git-native coordination for cross-session multi-agent |
| P3 | Arkor | Agent-driven fine-tuning workflow reference |
