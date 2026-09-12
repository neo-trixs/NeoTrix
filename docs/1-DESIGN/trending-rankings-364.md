# Trending Rankings — Cycle 364 (2026-09-12)

## 10 New Projects (Not in Cycles 318–363)

### 1. Harden AIF — Pre-Execution Security Firewall for Coding Agents
- **URL**: https://github.com/hardenrun/aif
- **Stars**: ~249 (ProductHunt launch Sep 9 2026)
- **Language**: Rust/Python
- **What**: Local-first AI firewall that intercepts every agent tool call before execution. Proprietary 8B cybersecurity LLM runs on-device. Checks identity, intent, data destination, and impact. Can allow, ask, rewrite, block, or record.
- **Key Insight**: Session-context-aware security — not naive regex. Beats GPT-5.5 on AgentHazard (83.7% vs 81.4% first-harm catch rate). 2.33s median decision on M5.
- **NeoTrix Mapping**: NT-SHIELD → `nt_shield::sandbox` (egress policy extension). Aligns with Egress Privacy Guard pattern but for inbound agent actions. Could inform NT-SHIELD's runtime safety kernel.
- **Absorb Pattern**: P4 (Ordered Backend Fallback) applied to security decision chain — rule-based fast path → model-based contextual judgment fallback.

### 2. Mastra — TypeScript Agent Framework with Workflow Engine
- **URL**: https://github.com/mastra-ai/mastra
- **Stars**: ~15k+
- **Language**: TypeScript
- **What**: Full agent framework from Gatsby team. Agents + graph-based workflows (.then/.branch/.parallel), persistent memory (working + semantic + observational), human-in-the-loop suspend/resume, built-in evals and observability. Dual license (Apache 2.0 core + Enterprise).
- **Key Insight**: Workflow-as-Graph with typed schemas (Zod/Valibot). Time-travel debugging for workflows. Observational Memory updates working memory from conversation patterns.
- **NeoTrix Mapping**: NT-ACT → workflow orchestration patterns. NT-MEMORY → Observational Memory concept maps to experience-tree's lazy branch loading. `createWorkflow` → SEAL pipeline step composition.

### 3. DeerFlow 2.0 — ByteDance SuperAgent Harness
- **URL**: https://github.com/bytedance/deer-flow
- **Stars**: 74,960 (3,329 new this week)
- **Language**: Python
- **What**: Ground-up rewrite. Sub-agents, persistent memory, Docker/K8s sandbox, extensible skill system, MCP server, IM channels (Lark/Slack/Discord), LangGraph-compatible API. 180 merged PRs since 2.0 milestone.
- **Key Insight**: "SuperAgent harness" not framework — owns sandbox, memory, run state, message bus. Skill progressive discovery + activation policy. Goal continuations across turns. Memory consolidation with staleness review.
- **NeoTrix Mapping**: NT-ACT → sub-agent orchestration pattern. NT-MEMORY → memory consolidation with `expected_valid_days` per-fact. NT-CORE → goal continuation across consciousness cycles.

### 4. Prime Agent — Self-Improving RLM for Coding Workflows
- **URL**: https://github.com/PrimeIntellect-ai/prime-agent
- **Stars**: 1,456
- **Language**: Python
- **What**: Recursive Language Model (RLM) agent. Persistent IPython as built-in tool. Continual Harness stores supplemental prompts/memories as durable state. `/refine` applies evidence-backed updates. Subagents as recursive function calls.
- **Key Insight**: "Prompt-as-a-variable" — context is programmable, not fixed. Harness refinement never rewrites immutable base prompt. Session-level snapshots support rollback. Agent-to-agent direct communication without routing through user.
- **NeoTrix Mapping**: NT-MIND → SEAL pipeline refinement pattern. NT-CORE → prompt-as-variable maps to VSA HyperCube's symbolic embedding. NT-NEXUS → agent-to-agent direct messaging.

### 5. OmniAgent — Full-Dimensional Self-Evolution Framework
- **URL**: https://github.com/YeQing17-2026/OmniAgent
- **Stars**: 2,557
- **Language**: Python
- **What**: Three-axis self-evolution: Skill (real-time during execution), Context (multi-layer information stack with LLM summarization), BrainModel (online RL). Hyper-Harness with dynamic multi-agent + concurrent tool execution. Deep Reflexion dual-layer architecture.
- **Key Insight**: Progressive Context Loading (L0/L1/L2) inspired by Anthropic's Progressive Disclosure. Trust-level security policies per Skill/Tool — "unbypassable" four-layer scanning. Real-time skill evolution vs periodic post-execution.
- **NeoTrix Mapping**: NT-MIND → real-time skill evolution during SEAL execution. NT-SHIELD → four-layer security scanning maps to NT-SHIELD's risk-assessment pipeline. NT-CORE → Deep Reflexion maps to ConsciousnessTree's 6-stage feedback.

### 6. BudgetMem — Query-Aware Budget-Tier Memory Routing
- **URL**: https://github.com/ViktorAxelsen/BudgetMem
- **Stars**: 23 (ICML 2026 accepted)
- **Language**: Python
- **What**: Runtime agent memory with explicit performance-cost control. Module-level budget tiers (Low/Mid/High) across three axes: implementation tiering, reasoning tiering, capacity tiering. RL-trained budget-tier router.
- **Key Insight**: Memory extraction is budget-aware and query-adaptive. Not all queries need full memory depth. The router learns when to use cheap heuristics vs expensive LLM-based extraction.
- **NeoTrix Mapping**: NT-MEMORY → budget-aware memory retrieval. Directly maps to Cost-Aware Routing axiom (A1) — extend GWT salience to memory retrieval cost. NT-CORE → AttentionManager could adopt tier-based retrieval.

### 7. M-flow — Bio-Inspired Cognitive Memory Engine (GraphRAG++)
- **URL**: https://github.com/FlowElement-xinliuyuansu/m_flow
- **Stars**: 4,505
- **Language**: Python
- **What**: Four-layer Cone Graph (Episode→Facet→FacetPoint→Entity). Graph-routed retrieval: anchor at matching granularity, then graph propagation scoring evidence paths. Path-cost optimization over flat ranking. Face-aware memory partitioning.
- **Key Insight**: "RAG matches chunks. GraphRAG structures context. M-flow scores evidence paths." Multi-granularity retrieval — query lands on the layer matching its granularity. Association as controlled graph propagation, not one-shot similarity.
- **NeoTrix Mapping**: NT-MEMORY → graph-based evidence path scoring for KB retrieval. NT-CORE → granularity-aware attention routing. VSA HyperCube could adopt cone-graph hierarchy for multi-scale concept representation.

### 8. GitAgent — Git-Native Agent Framework
- **URL**: https://github.com/open-gitagent/gitagent
- **Stars**: 670
- **Language**: Python
- **What**: Agent IS a git repo. Identity (agent.yaml), rules (RULES.md), memory (git-committed), tools (YAML), skills (composable), hooks (lifecycle) — all version-controlled files. MCP client. Always-learning via git history.
- **Key Insight**: "Agents as repos" paradigm — full audit trail, branching for experiments, merge for knowledge consolidation. Memory is git-committed with full history. Skills are composable modules with hooks.
- **NeoTrix Mapping**: NT-NEXUS → version-controlled experience graph. NT-MEMORY → git-native memory with history. NT-ACT → skill composition via git-style branching.

### 9. Atomic Agent — Local-First AI Agent with TurboQuant
- **URL**: https://github.com/AtomicBot-ai/atomic-agent
- **Stars**: 2,432
- **Language**: TypeScript
- **What**: Local-first agent with browser automation (Playwright), file editing, shell commands, document parsing, memory (hybrid recall + voting + reflection), durable tasks, MCP integration. Custom TurboQuant llama.cpp (+30-50% throughput).
- **Key Insight**: GAIA L1 benchmark: 69.8% accuracy with local qwen-3.6-35b-a3b, beating Hermes (58.5%) with same model. Memory uses hybrid recall with voting mechanism. Tauri sidecar for desktop embedding.
- **NeoTrix Mapping**: NT-PHYSICAL → TurboQuant optimization pattern for local inference. NT-MEMORY → voting-based hybrid recall. NT-IO → Tauri integration pattern for desktop embedding.

### 10. GlassBrain — Visual Trace Replay for AI Apps
- **URL**: https://www.producthunt.com/products/glassbrain
- **Stars**: New (launched 2026)
- **Language**: TypeScript
- **What**: Captures every AI app step as interactive visual trace tree. Click any node, swap input, replay instantly. Snapshot mode for deterministic replays. Auto-generated fix suggestions with one-click copy. Diff view. Shareable replay links.
- **Key Insight**: Trace-as-first-class-artifact — not logs, but replayable execution trees. "One full pipeline run = one trace" (1K free tier). Replays don't count against quota. Multi-agent chain support.
- **NeoTrix Mapping**: NT-REPAIR → trace-based debugging for SEAL pipeline failures. NT-IO → visual observability for consciousness cycles. Could replace current EventBus event logging with replayable trace trees.

---

## Trend Signals

| Signal | Projects | NeoTrix Implication |
|--------|----------|---------------------|
| **Agent Security as First-Class Concern** | Harden AIF, OmniAgent | NT-SHIELD needs pre-execution interception, not just egress filtering |
| **Budget-Aware Everything** | BudgetMem, Mastra | Cost-Aware Routing (A1) should extend to memory retrieval, not just model selection |
| **Git-Native Agent State** | GitAgent, DeerFlow | NT-NEXUS could use git-style versioned experience graphs |
| **Trace Replay for Debugging** | GlassBrain, Prime Agent | NT-REPAIR should capture replayable execution traces |
| **Real-Time Skill Evolution** | OmniAgent, Prime Agent | SEAL pipeline should support mid-execution skill refinement |
| **Bio-Inspired Memory Architecture** | M-flow, BudgetMem | NT-MEMORY could adopt cone-graph hierarchy for multi-granularity retrieval |
| **Observational Memory** | Mastra, Atomic Agent | Agents learn from conversation patterns, not just explicit storage |

---

## Sources
- GitHub Trending (Sep 2026): nanobot, OmniAgent, Atomic Agent, Prime Agent, GitAgent, M-flow, BudgetMem, DeerFlow
- ProductHunt (Sep 9 2026): Harden AIF, Mastra Factory, GlassBrain
- awesome-ai-agents-2026 curated list
- Top Agentic AI GitHub Repos (ODSC Jul 2026)
