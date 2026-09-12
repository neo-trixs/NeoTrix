# Trending Rankings — Cycle 368

Date: 2026-09-12
Scope: New AI/developer tools not in cycles 318-367

## 10 New Trending Projects

### 1. ToolRank
- **Site**: toolrank.dev | Founded 2026
- **Domain**: Agent Tool Optimization / Discovery
- **What**: Platform that scores AI agent tool definitions across four dimensions: findability, clarity, precision, and efficiency. Rule-based scoring + LLM selection tournaments + runtime reliability testing. 97% of MCP tool descriptions have quality defects; optimized tools get selected 3.6x more often. Provides rewrite proposals and an agent framework SDK.
- **Key Pattern**: **Tool-as-Measurable-Asset** — treats tool definitions as code quality problems with quantifiable metrics. Tool optimization becomes a CI concern, not a runtime concern.
- **NeoTrix Mapping**: NT-ACT (CapabilityRegistry tool health) + NT-SHIELD (tool definition audit). ToolRank's scoring dimensions map to NeoTrix's UnifiedCapability trait contract verification. Could enhance `CapabilityRegistry` with tool-readiness scoring as a pre-flight check.
- **Signal**: Tool optimization is becoming its own infrastructure layer. The shift from "tools work" to "tools are well-defined for agents" marks a maturity transition.

### 2. LoopX (huangruiteng/loopx)
- **Repo**: github.com/huangruiteng/loopx (5.6K★)
- **Domain**: Long-Horizon Agent Control Plane
- **What**: Lightweight state kernel running on top of any agent harness (Codex, Claude Code, Cursor). Provides durable objectives, gates, todos, evidence logs, quota-aware auto-wake, verifiable handoffs, and human-agent collaboration. Provider-neutral — governs state, not execution. Local-first, Apache-2.0.
- **Key Pattern**: **Control Plane as Thin Kernel** — separates governance (goals, gates, evidence) from execution (agent harness). State persists across turns, tools, and agents. Quota-aware scheduling prevents token overruns.
- **NeoTrix Mapping**: NT-MIND (SEAL pipeline governance) + NT-NEXUS (cross-session state). LoopX's objective/gate/todo/evidence model is structurally isomorphic to NeoTrix's ConsciousnessTree cycle governance. Quota-aware scheduling maps to GWT attention budget (Axiom A2: Context as Scarce Resource).

### 3. Colibrì (JustVugg/colibri)
- **Repo**: github.com/JustVugg/colibri (26.9K★)
- **Domain**: MoE Inference Engine / Memory Multitiering
- **What**: Pure C engine (single file, zero deps) that runs frontier MoE models (GLM-5.2 744B, Kimi K3 2.8T, Inkling 975B) on consumer hardware by streaming experts from disk. Dense part (~17B) stays resident in RAM (~9.9GB); 19,456 routed experts live on disk, streamed on demand with per-layer LRU cache. Treats VRAM, RAM, and storage as one managed memory hierarchy. MLA attention with compressed KV-cache: 576 floats/token vs 32,768 (57x smaller).
- **Key Pattern**: **Storage-as-Inference-Hierarchy** — disk is not "too slow for inference" when expert routing is predictable. The memory hierarchy (VRAM→RAM→disk) is managed as one pool with LRU + learned hot-store.
- **NeoTrix Mapping**: NT-CORE (E8 inference engine) + NT-MEMORY (tiered storage). Colibrì's memory multitiering validates NeoTrix's 3-layer architecture concept (L5 consciousness→L3 embodiment→L1 capability). The 57x KV-cache compression via MLA is directly relevant to kv_cache_optimizer.rs.

### 4. TencentDB-Agent-Memory
- **Repo**: github.com/TencentCloud/tencentdb-agent-memory (25.9K★)
- **Domain**: Team-Level Agent Memory Hub
- **What**: Memory hub that converts conversations, docs, and code into four reusable memory assets: Chat Memory, Skill, LLM-Wiki, Code-Graph. Framework-agnostic — one proxy serves Claude Code, Codex, Hermes, OpenClaw, etc. Personal Skills private by default, reviewable before team sharing. Cold-start friendly: import existing codebases on day one. 80 days to 15K stars.
- **Key Pattern**: **Memory-as-Asset** — memory is not raw history but curated, versioned, governed assets with ownership, visibility, and review workflows. Memory compounds across team members and agent sessions.
- **NeoTrix Mapping**: NT-MEMORY (KB experience hub) + NT-GOVERNANCE (asset governance). TencentDB's four memory assets (Chat/Skill/Wiki/CodeGraph) map to NeoTrix's experience-tree KB namespaces. The review-before-sharing workflow aligns with NT-GOVERNANCE policy enforcement.

### 5. ADHD Skill (UditAkhourii/adhd)
- **Repo**: github.com/UditAkhourii/adhd
- **Domain**: Cognitive Skill for Coding Agents
- **What**: Tree-of-thought with pruning, built on Claude & Codex Agent SDK. Fans out parallel divergent thoughts under different cognitive frames, scores, prunes traps, deepens survivors. "The no-brainer skill for creative and interdisciplinary work." Skill-as-cognitive-frame rather than skill-as-instruction.
- **Key Pattern**: **Cognitive Frame Divergence** — multiple parallel reasoning paths under different "lenses" (optimist/pessimist/creative/pragmatic), scored and pruned. Replaces single-path thinking with structured exploration.
- **NeoTrix Mapping**: NT-CORE (E8 hexagram reasoning) + NT-MIND (SEAL exploration). ADHD's parallel cognitive frames map to NeoTrix's E8 multi-hexagram reasoning states. Scoring + pruning ≈ GWT salience filtering. The skill framework aligns with SKILL-SPEC.md contract.

### 6. Mirascope
- **Repo**: github.com/Mirascope/mirascope
- **Domain**: LLM Anti-Framework
- **What**: "The LLM Anti-Framework" — opinionated Python library for LLM applications that emphasizes type safety, structured outputs, and provider abstraction without the bloat of LangChain-style orchestration. Provides decorators, response models, and call conventions that make LLM integration feel like regular Python function calls.
- **Key Pattern**: **Framework-as-Constraint** — instead of providing more flexibility, Mirascope constrains the API surface to force good patterns. Type safety and structured outputs as defaults, not options.
- **NeoTrix Mapping**: NT-IO (LLM provider abstraction). Mirascope's provider-agnostic structured output pattern aligns with NeoTrix's UnifiedCapability trait. The anti-framework stance resonates with Dark Forest axiom (must be minimal and necessary).

### 7. codebase-memory-mcp (DeusData)
- **Repo**: github.com/DeusData/codebase-memory-mcp (~32K★)
- **Domain**: Code Knowledge Graph for Agents
- **What**: MCP server that helps AI coding agents understand large codebases without repeatedly scanning files. Builds and maintains a structural knowledge graph of codebase entities (functions, classes, imports, dependencies) that agents query on-demand via MCP. Reduces context fed to AI tools during reviews and large-repo workflows.
- **Key Pattern**: **Code-as-Graph** — pre-indexed structural knowledge graph eliminates repeated file scanning. Agent queries the graph, not the filesystem. MCP as the interface boundary.
- **NeoTrix Mapping**: NT-MEMORY (KB code indexing) + NT-ACT (MCP tool interface). codebase-memory-mcp's graph structure aligns with NeoTrix's KB edge-based knowledge representation. MCP interface maps to NT-ACT's CapabilityRegistry tool routing.

### 8. SEM (Ataraxy-Labs/sem)
- **Repo**: github.com/Ataraxy-Labs/sem
- **Domain**: Semantic Version Control
- **What**: Entity-level diffs, blame, and impact analysis on top of git. 28 languages via tree-sitter. Built specifically for coding agents — resolves false conflicts when independent agents edit the same file (~95% reduction vs line-based merge). Entity-level merge, not line-level.
- **Key Pattern**: **Entity-as-Version-Unit** — version control at the semantic level (function/class/module) rather than text lines. Agents editing the same file don't conflict if they touch different entities. Git reimagined for multi-agent workflows.
- **NeoTrix Mapping**: NT-ACT (multi-agent coordination) + NT-SHIELD (conflict resolution). SEM's entity-level merge directly addresses the problem of parallel coding agents stepping on each other. The 95% false-conflict reduction validates NeoTrix's worktree isolation pattern.

### 9. AG-UI Protocol (CopilotKit)
- **Repo**: github.com/CopilotKit/CopilotKit (AG-UI protocol)
- **Domain**: Agent-UI Protocol
- **What**: Open protocol for agents to stream structured events to frontend UIs in real-time. Agents push state updates, tool calls, and text deltas to React/Angular/mobile/Slack frontends. Decouples agent runtime from presentation layer. CopilotKit provides the reference implementation.
- **Key Pattern**: **Agent-UI as Protocol** — standardizes the agent→UI data plane. Not a framework, not a library — a wire protocol that any agent can emit to any frontend. Real-time streaming by default.
- **NeoTrix Mapping**: NT-IO (interface layer) + NT-WORLD (perception output). AG-UI's event streaming maps to NeoTrix's EventBus grounding pattern. The protocol-level decoupling aligns with NeoTrix's 6-layer architecture separation.

### 10. Reef (Human-Agent-Society/reef)
- **Repo**: github.com/Human-Agent-Society/reef (~470★, born 2026-08-31)
- **Domain**: Continual Learning Infrastructure
- **What**: Continual learning infrastructure for self-improving agents — trains from interaction history without catastrophic forgetting. Agents improve from experience while retaining prior capabilities. Focused on the stability-plasticity dilemma in agent learning.
- **Key Pattern**: **Experience-as-Training-Data** — interaction history becomes training signal for continual self-improvement. Addresses the core problem: how do agents get better over time without forgetting what already works?
- **NeoTrix Mapping**: NT-MIND (SEAL pipeline self-evolution) + NT-MEMORY (experience retention). Reef's continual learning is the training-side analog of NeoTrix's experience-tree absorption. Stability-plasticity dilemma maps to SEAL's distillation→crystallization cycle.

## Emerging Meta-Patterns

| Pattern | Projects | NeoTrix Implication |
|---------|----------|---------------------|
| **Tool-as-Quality-Problem** | ToolRank, codebase-memory-mcp | Tool definitions need CI/CD quality gates, not just "does it work" |
| **Control Plane Separation** | LoopX, AG-UI Protocol | Governance/runtime decoupling as architectural principle |
| **Memory-as-Governed-Asset** | TencentDB-Agent-Memory, Reef | Memory with ownership, versioning, review — not just storage |
| **Entity-Level Abstraction** | SEM, codebase-memory-mcp | Version control and code understanding at semantic, not text, level |
| **Cognitive Divergence** | ADHD Skill | Parallel reasoning under multiple frames as first-class agent pattern |
| **Storage-as-Inference** | Colibrì | Memory hierarchy management blurs the line between storage and compute |

## Priority Absorption Candidates

| Priority | Project | Why |
|----------|---------|-----|
| P0 | **LoopX** control plane | Direct SEAL governance alignment — objective/gate/evidence model |
| P0 | **TencentDB-Agent-Memory** | Memory-as-asset with governance — validates KB experience hub pattern |
| P1 | **ToolRank** scoring | Tool quality CI gate for CapabilityRegistry |
| P1 | **Colibrì** memory multitiering | KV-cache compression and storage hierarchy for inference |
| P2 | **SEM** entity merge | Multi-agent conflict resolution at semantic level |
| P2 | **Reef** continual learning | Stability-plasticity for SEAL pipeline |
