# Trending Rankings — Cycle 339

**Date**: 2026-09-11
**Sources**: GitHub Search API, HN Algolia API, repo metadata (web search / anysearch APIs were unavailable this cycle — data sourced via direct API calls)

## 10 New Projects (Not in Cycles 318-338)

### 1. OKF Agent Memory (okf-memory/okf-agent-memory)
- **URL**: https://github.com/okf-memory/okf-agent-memory
- **Stars**: 565 | **Language**: Go | **Created**: 2026-09-05
- **Description**: Git-native persistent memory for AI coding agents. Implements Google OKF (Open Knowledge Format) v0.2 with sub-300µs in-memory BM25 search, embedded MCP server, and progressive knowledge loading.
- **Key Pattern**: **OKF-as-standard** — knowledge format interoperability across agents. Git-native storage with sub-300µs BM25 search enables instant recall without external infrastructure. MCP server makes memory accessible to any agent. Progressive loading (L0/L1/L2) for context-budget management.
- **NeoTrix Mapping**: NT-MEMORY — OKF format is a standardized experience schema (vs our KB namespace convention). Sub-300µs BM25 validates our FTS5-based search. Progressive loading is structurally identical to our lazy branch loading. **Absorption candidate**: OKF v0.2 format as cross-agent experience interchange standard + progressive loading tiers.

### 2. WikiSkill (ashutoshsinghpr7/wikiskill)
- **URL**: https://github.com/ashutoshsinghpr7/wikiskill
- **Stars**: 147 | **Language**: Python | **Created**: 2026-08-29
- **Description**: WikiSkill (arXiv:2608.27454) for Hermes Agent — self-evolving agent skills via a persistent knowledge wiki. Faithful Algorithm 1 implementation with reproducible validation. Skills evolve through wiki-style knowledge accumulation.
- **Key Pattern**: **Wiki-as-skill-evolution** — skills stored as editable wiki pages that agents can modify. Knowledge accumulates across sessions via wiki edits. Each skill is a living document, not a frozen template. Reproducible validation ensures quality.
- **NeoTrix Mapping**: NT-MIND — wiki-style skill evolution maps to SEAL pipeline distillation. Living documents = experience-tree entries that update in-place. **Absorption candidate**: wiki-as-skill-evolution pattern — skills as editable knowledge pages with version history + provenance tracking.

### 3. NeoHorse (TokenRhythm/NeoHorse)
- **URL**: https://github.com/TokenRhythm/NeoHorse
- **Stars**: 142 | **Language**: N/A | **Created**: 2026-09-04
- **Description**: NeoHorse-1: Towards Recursive Self-Improvement via Agentic Post-Training with Routing Harness. Explores recursive self-improvement (RSI) through agentic post-training — the agent improves its own training data and routing decisions.
- **Key Pattern**: **Recursive self-improvement (RSI)** — agent generates training signals for its own improvement. Routing harness selects which improvement signals to act on. Closed-loop: agent → evaluate → generate improvement data → retrain → repeat.
- **NeoTrix Mapping**: NT-MIND — RSI is the SEAL pipeline's self-evolution taken to its logical conclusion. Routing harness = GWT salience for self-improvement signals. **Absorption candidate**: RSI loop pattern — agent-driven improvement signal generation with routing-based selection of which improvements to act on.

### 4. Penelopa.ai (chigwell/Penelopa.ai)
- **URL**: https://github.com/chigwell/Penelopa.ai
- **Stars**: 119 | **Language**: JavaScript | **Created**: 2026-09-03
- **Description**: Continuous improvement for AI coding agents. Analyzes real Codex and Claude Code sessions, finds repeated workflow patterns, and turns them into reusable skills. Session-to-skill pipeline: observe → extract pattern → codify as skill → recommend.
- **Key Pattern**: **Session-to-skill extraction** — observe actual agent behavior, extract repeated patterns, codify as reusable skills. Not manual skill creation but automatic skill crystallization from real usage. Workflow pattern detection + recommendation engine.
- **NeoTrix Mapping**: NT-MIND — session-to-skill extraction maps to SEAL pipeline's distillation stage. Pattern detection = experience-tree's route table learning. **Absorption candidate**: automatic skill crystallization from session traces — observe → extract → codify → recommend pipeline for experience-tree evolution.

### 5. Shared Memory Vault (songs-aaa/shared-memory-vault)
- **URL**: https://github.com/songs-aaa/shared-memory-vault
- **Stars**: 73 | **Language**: N/A | **Created**: 2026-09-05
- **Description**: Memory governance engine for multi-agent cloud knowledge bases — one knowledge base shared by every agent and every device, with consistency guarantees. Solves the multi-agent memory coordination problem.
- **Key Pattern**: **Governed shared memory** — single KB shared across agents with governance (consistency, conflict resolution, access control). Not just shared storage but governed coordination — who can write, who can read, how conflicts resolve.
- **NeoTrix Mapping**: NT-MEMORY + NT-GOVERNANCE — governed KB sharing maps to our KB namespace isolation + cross-domain access patterns. Consistency guarantees = our KB edge versioning. **Absorption candidate**: governance-as-memory-layer — consistency guarantees and conflict resolution for cross-domain KB access.

### 6. miniEvoAgent (TunaaaAaaaa/miniEvoAgent)
- **URL**: https://github.com/TunaaaAaaaa/miniEvoAgent
- **Stars**: 85 | **Language**: Python | **Created**: 2026-08-28
- **Description**: A simple, feasible repository to reproduce and learn self-evolving / recursive self-improvement agent. Minimal reproduction of RSI patterns for education and research.
- **Key Pattern**: **Minimal RSI reproduction** — stripped-down implementation of self-evolution for understanding the core mechanics. Serves as reference architecture for how self-improvement loops actually work.
- **NeoTrix Mapping**: NT-MIND — reference implementation for SEAL pipeline self-evolution mechanics. **Absorption candidate**: use as reference architecture for SEAL pipeline's self-evolution loop design.

### 7. Charter (boundflow/charter)
- **URL**: https://github.com/boundflow/charter
- **Stars**: 10 | **Language**: Python | **Created**: 2026-08-17
- **Description**: Build and operate production-safe agents that run on your own infrastructure. YAML-defined agents with durable execution, human-in-the-loop gates, OpenTelemetry tracing, MCP integration. Self-hosted, no cloud dependency.
- **Key Pattern**: **YAML-as-agent-spec** — agent definition as declarative YAML with governance baked in. Durable execution survives failures. Human-in-the-loop gates at critical decision points. OpenTelemetry for observability. Self-hosted = data never leaves your infra.
- **NeoTrix Mapping**: NT-ACT + NT-SHIELD — YAML agent spec maps to our domain module conventions. Durable execution = our production orchestrator. HITL gates = NT-SHIELD's graduated trust. **Absorption candidate**: YAML-as-agent-spec with built-in governance gates + durable execution model.

### 8. MaruCheck (Kidus-M/MaruCheck)
- **URL**: https://github.com/Kidus-M/MaruCheck
- **Stars**: 6 | **Language**: TypeScript | **Created**: 2026-08-15
- **Description**: Independent QA and verification for AI-generated software. Turns product intent into Quality Contracts, analyzes code changes, remembers regressions. MCP server + GitHub Actions integration. CI/CD-native testing.
- **Key Pattern**: **Quality Contracts** — formalize product intent as verifiable contracts. AI-generated code tested against intent, not just syntax. Regression memory across sessions. Independent of the code-generating agent (meta-verification).
- **NeoTrix Mapping**: NT-SHIELD + NT-GOVERNANCE — Quality Contracts = our SelfTest tiers (T1/T2/T3). Regression memory = experience-tree failure patterns. Independent verification = rev-officer's dual verification principle. **Absorption candidate**: Quality Contract pattern — formalize intent as verifiable contracts for agent output.

### 9. Covenant Framework (asalsali/covenant-framework-community)
- **URL**: https://github.com/asalsali/covenant-framework-community
- **Stars**: 5 | **Language**: Python | **Created**: 2026-07-04
- **Description**: A governance framework for multi-agent AI systems. Structure, lifecycle, and quality controls for AI agent orchestration. Defines agent roles, communication protocols, and governance boundaries.
- **Key Pattern**: **Agent lifecycle governance** — not just runtime safety but full lifecycle: creation → deployment → monitoring → retirement. Structured role definitions and communication protocols. Quality controls at each lifecycle stage.
- **NeoTrix Mapping**: NT-GOVERNANCE — lifecycle governance maps to our constellation maturity model (C0-C6). Role definitions = our domain module conventions. **Absorption candidate**: lifecycle-as-governance — formal lifecycle stages with quality gates at each transition.

### 10. Agentic Stack Desktop (codejunkie99/agentic-stack-desktop)
- **URL**: https://github.com/codejunkie99/agentic-stack-desktop
- **Stars**: 55 | **Language**: Python | **Created**: 2026-09-08
- **Description**: Native macOS workspace with one local knowledge graph across Claude Code, Codex, OpenCode, and Cursor. Unified graph view of all agent activities. Local-first, no cloud dependency.
- **Key Pattern**: **Cross-agent knowledge graph** — single knowledge graph shared across multiple coding agents. Agents contribute to and query from a shared graph. Visual graph interface for human oversight. Local-first privacy.
- **NeoTrix Mapping**: NT-MEMORY + NT-IO — cross-agent graph = our KB with cross-domain edges. Visual interface = potential NT-IO dashboard. **Absorption candidate**: cross-agent knowledge graph as shared context layer — multiple agents contributing to one graph with visual oversight.

## Cross-Cutting Themes (Cycle 339)

| Theme | Projects | NeoTrix Impact |
|-------|----------|----------------|
| **Self-Evolution at Scale** | WikiSkill, NeoHorse, miniEvoAgent, Penelopa | SEAL pipeline should support multiple evolution patterns: wiki-edit, RSI-loop, session-extraction, minimal-repro |
| **Git-Native Memory** | OKF Agent Memory, Charter | Git as transport layer for experience persistence (validated across cycles) |
| **Governed Shared Memory** | Shared Memory Vault, Covenant | Multi-agent KB needs governance layer, not just shared storage |
| **Session-to-Skill Crystallization** | Penelopa, WikiSkill | Automatic skill creation from observed behavior patterns |
| **Quality Contracts** | MaruCheck, Charter | Formalize intent as verifiable contracts for agent output |
| **Cross-Agent Knowledge** | Agentic Stack Desktop, OKF Agent Memory | Shared knowledge graphs across agent boundaries |
| **Minimal RSI Reference** | miniEvoAgent | Reference architecture for understanding self-improvement loops |

## Prioritization

| Priority | Project | Action |
|----------|---------|--------|
| P0 | Penelopa.ai | Study session-to-skill extraction pipeline for SEAL distillation |
| P0 | OKF Agent Memory | Evaluate OKF v0.2 as cross-agent experience interchange format |
| P1 | WikiSkill | Study wiki-as-skill-evolution for experience-tree entries |
| P1 | NeoHorse | Analyze RSI loop pattern for ConsciousnessTree self-evolution |
| P1 | Shared Memory Vault | Study governance-as-memory-layer for multi-domain KB access |
| P2 | MaruCheck | Quality Contract pattern for SelfTest tier formalization |
| P2 | Charter | YAML-as-agent-spec with built-in governance gates |
| P2 | Agentic Stack Desktop | Cross-agent knowledge graph for NT-MEMORY shared context |
| P3 | miniEvoAgent | Reference architecture for SEAL pipeline mechanics |
| P3 | Covenant Framework | Lifecycle-as-governance pattern for constellation maturity |
