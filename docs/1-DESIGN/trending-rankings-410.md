# Trending Rankings — Cycle 410 (2026-09-12)

## 10 New Projects (Not in Cycles 318–409)

---

### 1. GenericAgent — Self-Evolving LLM Agent (~3K LOC)

| Field | Detail |
|-------|--------|
| **Repo** | `lsdefine/GenericAgent` |
| **Stars** | 14.1K |
| **Language** | Python |
| **Core Idea** | Minimal (~3K LOC) self-evolving agent. 9 atomic tools + ~100-line Agent Loop. Auto-crystallizes tasks into Skills forming a personal skill tree. |
| **Key Pattern** | **Contextual Information Density Maximization** — stays under 30K context window (vs 200K–1M for peers). Morphling mode absorbs external repos into reusable SOPs. Goal Hive mode for multi-worker cooperative tasks. |
| **NeoTrix Integration** | NT-MIND: Self-evolving skill tree maps directly to SEAL pipeline crystallization. NT-CORE: Token-efficient context management via information density = GWT salience optimization. NT-ACT: Conductor sub-agent orchestration pattern for Agent Swarms. |

---

### 2. OmniAgent — Full-Dimensional Self-Evolving Agent

| Field | Detail |
|-------|--------|
| **Repo** | `YeQing17-2026/OmniAgent` |
| **Stars** | 2.6K |
| **Language** | Python |
| **Core Idea** | Open-source self-evolving agent with 3D evolution: Skill + Context + BrainModel. Dual-path proactive memory via IM induction. Online RL feedback loop for BrainModel self-evolution. |
| **Key Pattern** | **OmniEvolve** — real-time skill self-evolution during execution (not post-hoc). Hyper Harness provides dynamic multi-agent orchestration (Sentinel for planning, Guardian for safety). Deep Reflexion dual-layer: real-time risk interception + failure-to-insight conversion. |
| **NeoTrix Integration** | NT-MIND: Online RL evolution loop = SEAL pipeline continuous mode. NT-SHIELD: Hyper Harness 4-layer security scanning (LLM review → policy engine → approval → sandbox) mirrors NT-SHIELD defense-in-depth. NT-CORE: Deep Reflexion inner-outer loop = ConsciousnessTree feedback stages. |

---

### 3. Prime Agent — Self-Improving RLM Agent

| Field | Detail |
|-------|--------|
| **Repo** | `PrimeIntellect-ai/prime-agent` |
| **Stars** | 1.5K |
| **Language** | Python |
| **Core Idea** | Recursive Language Model (RLM) treats context as variables and tools as recursive sub-agents inside a persistent REPL. Continual Harness stores durable state (prompts, memories, skills, subagent specs) refined through small evidence-backed updates. |
| **Key Pattern** | **Prompt-as-a-Variable** — context is mutable state, not static. `/refine` reviews trajectory and applies small, reviewable updates to harness state (never rewrites base system prompt). Agent-to-agent direct communication without user routing. Daemon-backed continuity survives terminal disconnect. |
| **NeoTrix Integration** | NT-MEMORY: Continual Harness = KB experience evolution with versioned snapshots. NT-CORE: Mutable context-as-variable = GWT attention routing with runtime-adjustable salience. NT-ACT: RLM sub-agent spawning maps to Agent Swarm delegation. |

---

### 4. GitAgent — Git-Native AI Agent Framework

| Field | Detail |
|-------|--------|
| **Repo** | `open-gitagent/gitagent` |
| **Stars** | 670 |
| **Language** | Python |
| **Core Idea** | Agent IS a git repository. Identity (agent.yaml), rules (RULES.md), personality (SOUL.md), memory/, tools/, skills/ are all version-controlled. Branch an agent, diff its memory, review its evolution via git history. |
| **Key Pattern** | **Agents-as-Repos** — full version control for agent state. MCP client auto-discovers servers. SDK runs in-process (no subprocess/IPC). Multi-model via unified pi-ai interface. |
| **NeoTrix Integration** | NT-MEMORY: Git-versioned agent memory = KB versioned snapshots with git-like provenance tracking. NT-GOVERNANCE: Version-controlled rules/SOUL.md = constitution versioning in AGENTS.md. NT-ACT: MCP client auto-discovery = capability registry dynamic loading. |

---

### 5. Mycelium — Multi-Agent Coordination Layer

| Field | Detail |
|-------|--------|
| **Repo** | `mycelium-io/mycelium` |
| **Stars** | 116 |
| **Language** | Python |
| **Core Idea** | Coordination for autonomous peer agents (no orchestrator, no hierarchy). SLIM-encrypted channels. Rooms with shared memory. Aligner uses real NEGMAS Stacked Alternating Offers negotiation. |
| **Key Pattern** | **Peer-Negotiated Alignment** — agents negotiate to single shared answer via formal protocol, not parallel outputs. Room memory is single-store on hub (no sync/drift). Work rows = memory per task with hold status. IOC Layer 9 epistemic envelopes for coordination messages. |
| **NeoTrix Integration** | NT-CORE: GWT attention routing could use negotiation-based consensus for multi-domain broadcast decisions. NT-MEMORY: Room memory with no-sync-hub = centralized KB with agent-level access control. NT-ACT: Conductor sub-agent pattern enhanced with formal negotiation (not just delegation). |

---

### 6. CORTEX — Cognitive Memory Architecture with Dream Cycles

| Field | Detail |
|-------|--------|
| **Repo** | `ATERNA-AI/cortex` |
| **Stars** | 17 |
| **Language** | TypeScript |
| **Core Idea** | Neurocomputational memory with hippocampal indexing, CA3 autoassociative recall, reconsolidation, Ebbinghaus decay. 5-phase dream cycle (resonance → prune → consolidate → associate → synthesize). 7-factor hybrid scoring. |
| **Key Pattern** | **Synthetic Sleep** — nightly dream cycles: resonance decay, adaptive percentile-based pruning, cluster consolidation with LLM abstractive summaries, free association, insight synthesis. Reconsolidation: retrieved memories enter labile window for update. Temporal validity with valid_from/valid_until/superseded_by. |
| **NeoTrix Integration** | NT-MEMORY: Dream cycle = experience-tree absorption phases (fast snapshot → distill → classify → persist → feedback). NT-CORE: Reconsolidation with labile window = ConsciousnessTree belief update on new evidence. NT-FEEL: Emotional valence weighting maps to EmotionLabel 11-variant scoring. |

---

### 7. CogniHive — Transactive Memory for Multi-Agent Teams

| Field | Detail |
|-------|--------|
| **Repo** | `vmore2/CogniHive` |
| **Stars** | ~1K |
| **Language** | Python |
| **Core Idea** | Transactive Memory System (TMS) for AI agents — "who knows what" queries. Expertise routing prevents redundant work. Conflict detection + resolution. Integrates with CrewAI, AutoGen, LangChain, OpenAI Assistants, MCP. |
| **Key Pattern** | **Expertise Routing** — `who_knows(topic)` returns ranked experts with scores. `ask(query)` auto-routes to best expert + retrieves relevant memories. Expertise matrix tracks knowledge distribution. Dedup prevents token waste from agents researching the same thing. |
| **NeoTrix Integration** | NT-ACT: Expertise routing = Capability Registry with competence scoring per domain module. NT-CORE: GWT salience enhanced with "who knows" meta-routing across 7 domains. NT-MIND: Skill crystallization aware of cross-domain expertise gaps. |

---

### 8. PMB — Local-First Memory for Coding Agents

| Field | Detail |
|-------|--------|
| **Repo** | `pmb-ai` (PyPI: `pmb-ai`) |
| **Stars** | New (ProductHunt featured) |
| **Language** | Python |
| **Core Idea** | Persistent project memory via MCP for Claude Code/Cursor/Codex/Zed. SQLite workspace on disk. No cloud, no API keys. Hybrid retriever: BM25 + vectors + entity graph, fused. Forgetting-curve decay. Corrections override stale entries. |
| **Key Pattern** | **Typed Memory** — lessons treated as rules, goals as goals, project work as recent activity. Append-only with 4-layer dedup. Recency + forgetting-curve decay so stale context loses weight. Correction overrides contradicting entries. Session diff shows what memory shaped each suggestion. |
| **NeoTrix Integration** | NT-MEMORY: PMB's typed memory + forgetting curve = KB experience decay with typed hubs (lesson/goal/work). NT-MIND: Session diff tracking = experience-tree absorption trace. NT-GOVERNANCE: Append-only + dedup = no-overwrite KB integrity. |

---

### 9. Understanding Graph — Reasoning-Capture Memory via Stigmergy

| Field | Detail |
|-------|--------|
| **Repo** | `emergent-wisdom/understanding-graph` |
| **Stars** | MCP-based |
| **Language** | TypeScript |
| **Core Idea** | MCP server storing "understanding updates" — tensions, surprises, decisions, evidence, belief evolution. Multi-agent coordination through graph stigmergy (no direct messaging needed). Nodes never deleted, only superseded. Commit messages are coordination layer. |
| **Key Pattern** | **Stigmergic Coordination** — agents leave inspectable traces in shared graph. Triggers classify contributions (tension, question, decision, surprise). Solver system for long-running cross-session async handoff. Supersession preserves epistemic journey. |
| **NeoTrix Integration** | NT-MEMORY: Understanding updates (tensions/surprises/decisions) = KB experience nodes with trigger classification. NT-CORE: Stigmergic coordination = GWT salience modulation by shared graph state. NT-MIND: Superseded nodes = experience evolution with full lineage. |

---

### 10. Plasmod — Agent-Native Database for Cognitive Objects

| Field | Detail |
|-------|--------|
| **Repo** | `CodeSoul-co/Plasmod` |
| **Stars** | 10 |
| **Language** | Go |
| **Core Idea** | Agent-native DB with Event/Memory/State/Artifact/Edge as first-class objects. Append-only WAL with replay. Structured evidence retrieval (provenance + proof traces + graph context). Hot/warm/cold tiered storage. |
| **Key Pattern** | **Event-Driven State Evolution** — state changes flow through WAL, not direct overwrites. Evidence packages: matching memory + provenance + proof traces + 1-hop graph expansion. Workspace/session isolation. Python SDK + LangChain adapter. |
| **NeoTrix Integration** | NT-MEMORY: Plasmod's Event/Memory/State model = KB node types with append-only WAL. NT-ACT: Evidence packages with proof traces = experience-tree absorption with full lineage. NT-WORLD: Tiered storage (hot/warm/cold) for crawl pipeline data management. |

---

## Cross-Cutting Patterns (Cycle 410)

| # | Pattern | Definition | NeoTrix Mapping |
|---|---------|-----------|-----------------|
| P1 | **Self-Evolving Skills** | Skills crystallize automatically from task execution, not manual authoring (GenericAgent, OmniAgent, Prime Agent) | SEAL pipeline crystallization stage automation |
| P2 | **Context-as-Variable** | Context is mutable state refined by evidence, not static prompt (Prime Agent, CORTEX reconsolidation) | GWT attention routing with runtime-adjustable salience |
| P3 | **Agents-as-Repos** | Agent state is version-controlled git repository (GitAgent, PMB append-only) | KB versioned snapshots with git-like provenance |
| P4 | **Peer Negotiation** | Formal negotiation protocol for consensus, not orchestrator routing (Mycelium NEGMAS) | GWT broadcast with negotiation-based salience arbitration |
| P5 | **Dream Consolidation** | Periodic background processing to consolidate, prune, and synthesize memories (CORTEX dream cycle) | experience-tree absorption phases + ConsciousnessTree growth cycle |
| P6 | **Transactive Memory** | "Who knows what" routing prevents redundant work and surfaces expertise gaps (CogniHive) | Capability Registry with competence scoring per domain |
| P7 | **Stigmergic Coordination** | Shared graph as coordination surface — agents leave traces, not messages (Understanding Graph) | KB as shared cognitive workspace across NT-* domains |

---

## Source List

| # | Source | URL |
|---|--------|-----|
| 1 | GitHub Trending | `github.com/trending` (2026-09-12) |
| 2 | OSSInsight AI Repos | `ossinsight.io/trending/ai` |
| 3 | GitHub Trending AI Repos 2026 | `fungies.io/top-github-repositories-ai-agent-frameworks-2026` |
| 4 | Product Hunt Sep 2026 | `producthunt.com/products` |
| 5 | ODSC Agentic Repos 2026 | `odsc.medium.com/top-agentic-ai-github-repos-worth-watching-in-2026` |
| 6 | AskGlitch May 2026 | `askglitch.com/blog/top-5-trending-ai-github-repos-may-2026` |
| 7 | DataAIHub GitHub | `dataaihub.co/github` |
