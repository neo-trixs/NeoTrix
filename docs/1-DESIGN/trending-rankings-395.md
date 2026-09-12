# Trending Rankings — Cycle 395 (2026-09-12)

## 10 New Projects (Not in Cycles 318-394)

| # | Project | Stars | Category | Key Pattern | NeoTrix Domain Mapping |
|---|---------|-------|----------|-------------|----------------------|
| 1 | **Ponytail** | 91.8K | Coding Agent Skill | Anti-over-engineering: "write only what the task needs" — 54% less code, 20% cheaper, 27% faster on real agent sessions | NT-ACT (R-P42 anti-proliferation), NT-MIND (distillation discipline) |
| 2 | **Graphify** | 107.8K | Knowledge Graph | AST-based deterministic code→knowledge graph with Leiden community detection, no vector store | NT-MEMORY (KB graph edges), NT-CORE (VSA HyperCube graph relations) |
| 3 | **GBrain** | 27.0K | Agent Memory | Hybrid search (vector+BM25+RRF+graph signals) with self-wiring knowledge graph, 43 curated skills | NT-MEMORY (hybrid retrieval), NT-CORE (GWT salience scoring) |
| 4 | **Webwright** | 5.9K | Browser Agent | Terminal-as-interface browser agent; Skill Factory distills solved tasks into reusable code scripts (40s, zero tokens) | NT-ACT (skill crystallization), NT-WORLD (web perception) |
| 5 | **GitAgent** | 670 | Agent-as-Repo | Agent IS a git repo — identity/rules/memory/tools/skills all version-controlled files | NT-MEMORY (versioned knowledge), NT-SHIELD (audit trail) |
| 6 | **Vercel Eve** | 4.9K | Agent Framework | Filesystem-first durable agents; conventional file locations for inspectability and operability | NT-ACT (action layer convention), NT-IO (interface patterns) |
| 7 | **TanStack AI** | 3.0K | AI SDK | Type-safe, provider-agnostic TypeScript SDK with Code Mode (LLM writes+executes TS in sandbox) | NT-IO (provider abstraction), NT-ACT (code execution) |
| 8 | **Mastra** | N/A | Agent Workflow | Gatsby team's TypeScript agent framework with workflows, memory, streaming, evals, tracing, Studio UI | NT-MIND (evaluation), NT-IO (streaming) |
| 9 | **Nanobot** | 47.7K | Personal Agent | Ultra-lightweight self-hosted agent with Dream memory, model routing, multi-agent delegation | NT-CORE (model routing), NT-MEMORY (Dream long-term memory) |
| 10 | **GoModel** | N/A | AI Gateway | Open-source OpenRouter in Go: single binary, budgets, caching, guardrails, load balancing, failover | NT-IO (provider gateway), NT-SHIELD (guardrails) |

## Emerging Patterns

### 1. Agent-as-Repo (GitAgent, Eve)
Agents are becoming **version-controlled artifacts** — identity, rules, memory, and skills live in git. This aligns with NeoTrix's `experience-tree` KB persistence and the指针守恒 principle. The trend validates storing agent state in structured, auditable stores rather than ephemeral context.

### 2. Skill Crystallization (Ponytail, Webwright Skill Factory)
The most impactful projects **distill agent experience into reusable code skills**. Webwright's Skill Factory reduces token cost to zero for solved tasks. Ponytail proves anti-over-engineering as a skill discipline. Maps directly to NeoTrix's SEAL pipeline → skill crystallization flow.

### 3. Hybrid Retrieval Convergence (GBrain, Graphify)
GBrain's P@5 +31.4 lift from graph+vector+BM25 over vector-only RAG confirms NeoTrix's KB design (SQLite + FTS5 + embeddings). Graphify's deterministic AST-based graph extraction validates the "no hallucinated relations" principle.

### 4. Local-First AI (Nanobot, GoModel, Open WebUI)
Self-hosted, privacy-preserving, zero-cost inference is a major trend. Nanobot at 47K stars in 7 months shows demand for lightweight agent frameworks. Aligns with NT-IO local provider support.

### 5. Visual Agent Building (Mastra Studio, Dify, Langflow)
Drag-and-drop agent workflow design is going mainstream. Mastra Studio from the Gatsby team signals TypeScript-first agent development. Maps to NT-IO interface patterns.

## Source Tracking

| Source | Date | Projects Found |
|--------|------|---------------|
| GitHub Trending (OSSInsight) | 2026-09-12 | Ponytail, Graphify, GBrain, Webwright, GitAgent, Eve, TanStack AI |
| ProductHunt Weekly | 2026-09-07 | Mastra, GoModel, Nanobot |
| ByteByteGo Top Repos | 2026-03 | Cross-validated star counts |

## Cycle-over-Cycle Delta

- **New patterns**: Agent-as-Repo, Skill Crystallization (code not prompts), Anti-Over-Engineering
- **Accelerating**: Hybrid retrieval (graph+vector+BM25), Local-first agents
- **Decelerating**: Pure prompt-chain agents (superseded by graph-based coordination)
