# AI Agent Frameworks & Multi-Agent Systems — NeoTrix Absorption Analysis

**Date**: 2026-09-15  
**Category**: AI Agent Frameworks & Multi-Agent Systems  
**Source URLs Analyzed**: 44 (deduplicated to 43 unique projects)  
**Maturity Signal Threshold**: Stars > 10K, active maintenance, production adoption

---

## 1. FULL URL CLASSIFICATION TABLE

### 1.1 Agent Orchestration & Frameworks

| # | Name | URL | Category | Core Capability | NeoTrix Mapping | Maturity | Redundancy Risk |
|---|------|-----|----------|----------------|-----------------|----------|-----------------|
| 1 | **crewAI** | `github.com/crewAIInc/crewAI` | Agent Orchestration | Role-based multi-agent orchestration with Crews (autonomous teams) and Flows (event-driven pipelines); 58K+ stars, 2B+ task executions | NT-ACT (Action Layer) — primary; NT-META (orchestration patterns) | **Production** (v1.0, 27M+ downloads, 150+ enterprise customers) | **HIGH** — overlaps with NT-ACT's agent orchestration primitives; crewAI's Crew/Flow duality mirrors NeoTrix's own agent team patterns |
| 2 | **OpenHands** | `github.com/OpenHands/OpenHands` | Code Agent Platform | Open-source platform for software engineering agents with GUI, CLI, SDK, and enterprise control plane; self-hosted Agent Canvas | NT-ACT — primary; NT-IO (agent-server interface) | **Production** (86K+ stars, MLSys 2026 paper, ACP-compatible) | **HIGH** — directly competes with NT-ACT's coding agent capabilities; OpenHands Agent Canvas overlaps with NT-WORLD's desktop automation |
| 3 | **Hermes Agent** | `github.com/NousResearch/hermes-agent` | Self-Evolving Agent | Personal AI agent with persistent memory, skill self-learning, scheduled jobs, and messaging gateway; creator of the Hermes model series | NT-MIND (self-evolution) + NT-MEMORY (persistent memory) | **Mature** (45K+ stars, known for Hermes models, self-evolving capabilities) | **MEDIUM** — NT-MIND's self-evolution closely mirrors hermes-agent's skill self-learning; memory system overlaps NT-MEMORY |
| 4 | **Aider** | `github.com/Aider-AI/aider` | Code Agent | AI-powered pair programming tool for code editing; 45K+ stars, pip-installed, model-agnostic | NT-ACT — coding agent sub-module | **Production** (5.3M+ PyPI downloads, widely adopted) | **MEDIUM** — core coding agent function overlaps NT-ACT's code agent capabilities |
| 5 | **Orchard** | `github.com/microsoft/Orchard` | Agentic Modeling Framework | Kubernetes-native sandbox environment service for scalable agentic modeling; SFT+RL training recipes across coding, GUI, and assistant domains | NT-CORE (infrastructure) + NT-META (modeling framework) | **Research→Production** (MSR paper, arXiv:2605.15040, OpenForge RL released) | **MEDIUM** — sandbox infrastructure concept overlaps NT-WORLD's environment primitives; training pipeline is novel |
| 6 | **LongHorizon-Harness** | `github.com/AMAP-ML/LongHorizon-Harness` | Loop Engineering | Manage-Execute-Audit loop for long-horizon agent execution; externalized task state with independent auditing; 1,470 stars, +28.9% WeaveBench gain | NT-CORE (execution loop) + NT-MEMORY (state management) | **Research** (paper published, actively developed) | **LOW** — MEA loop pattern is novel; task-state management fills gap in NT-CORE |
| 7 | **Autoresearch** | `github.com/karpathy/autoresearch` | Autonomous Research Loop | AI agent autonomously runs ML experiments — modify → train → measure → keep/discard loop; 95K+ stars; spawned Bilevel Autoresearch research | NT-MIND (self-improvement loop) + NT-CORE (experimentation) | **Mature** (95K stars, widely cited, spawned research sub-fields) | **LOW** — ratchet loop pattern is unique; directly applicable to NeoTrix's self-evolution cycle |
| 8 | **Paseo** | `github.com/getpaseo/paseo` | Remote Agent Daemon | Self-hosted daemon for running Claude Code, Codex, Copilot, OpenCode remotely; drive agents from phone/desktop/web; 16.4K stars | NT-IO (remote agent access) + NT-ACT (agent runtime) | **Production** (active, multi-agent support) | **MEDIUM** — remote agent daemon overlaps NT-IO's remote execution primitives |
| 9 | **Eve** | `github.com/vercel/eve` | Agent Framework | Filesystem-first durable agent framework by Vercel; instructions.md + agent.ts + tools/ + subagents + schedules; built-in sandbox, channels, evals | NT-ACT (agent definition) + NT-IO (channels) + NT-META (subagent orchestration) | **Beta→Production** (Vercel production-grade, public preview) | **MEDIUM** — filesystem-first agent definition overlaps NT-ACT's agent composition; subagent delegation overlaps NT-META |
| 10 | **OmniStudio** | `github.com/kunpengtalk/OmniStudio` | AI Studio Platform | Multi-model AI development studio; likely encompasses design, code, and research agents | NT-ACT + NT-WORLD (studio environment) | **Early** (limited data) | **HIGH** — broad AI studio overlaps multiple NT modules |
| 11 | **Octop** | `github.com/TencentCloud/Octop` | Agent Platform | Tencent's agent platform with ACP (Agent Client Protocol) support; cloud-native agent deployment | NT-IO (protocol layer) + NT-ACT | **Early→Growth** | **MEDIUM** — ACP protocol overlap with NT-IO's agent communication |

### 1.2 Code Agents & Developer Tools

| # | Name | URL | Category | Core Capability | NeoTrix Mapping | Maturity | Redundancy Risk |
|---|------|-----|----------|----------------|-----------------|----------|-----------------|
| 12 | **Aider** (cross-ref) | `github.com/Aider-AI/aider` | Code Agent | See above — model-agnostic code editing assistant | NT-ACT | Production | MEDIUM |
| 13 | **OpenHands** (cross-ref) | `github.com/OpenHands/OpenHands` | Code Agent Platform | See above | NT-ACT | Production | HIGH |
| 14 | **alibaba/open-code-review** | `github.com/alibaba/open-code-review` | Code Review Agent | Automated code review agent; Alibaba's open-source code review system | NT-SHIELD (review/guardrails) | **Production** | **MEDIUM** — code review function overlaps NT-SHIELD's quality gates |
| 15 | **Oh My OpenAgent (OmO)** | `github.com/code-yeongyu/oh-my-openagent` | Multi-Agent Harness | 11 specialized agents, Team Mode, background tasks, 60+ lifecycle hooks; 4.12.1; ports patterns to OpenCode ecosystem | NT-META (orchestration) + NT-ACT (agent team) | **Growth** | **HIGH** — 11-agent orchestration directly overlaps NT-META's multi-agent patterns |
| 16 | **Claude Task Master** | `github.com/eyaltoledano/claude-task-master` | Task Management Agent | Claude-based task management and delegation system | NT-ACT (task delegation) | **Early** | **LOW** — simple task delegation; minimal overlap |
| 17 | **Defending Code Reference Harness** | `github.com/anthropics/defending-code-reference-harness` | Code Defense Agent | Anthropic's code reference harness for defending against prompt injection and adversarial code | NT-SHIELD (security) | **Research** | **LOW** — security-specific; fills NT-SHIELD gap |

### 1.3 Browser & Computer-Use Agents

| # | Name | URL | Category | Core Capability | NeoTrix Mapping | Maturity | Redundancy Risk |
|---|------|-----|----------|----------------|-----------------|----------|-----------------|
| 18 | **Browser Use** | `github.com/browser-use/browser-harness` | Browser Agent | Self-healing browser harness enabling LLMs to complete any web task; 109K+ stars on main repo; CDP-based, model-agnostic | NT-WORLD (browser automation) | **Production** (89.1% WebVoyager, #4 leaderboard) | **HIGH** — browser automation directly overlaps NT-WORLD's browser/computer-use capabilities |
| 19 | **Browser Use** (web-ui) | `github.com/browser-use/web-ui` | Browser UI | Run AI agents in the browser; web-based agent interface | NT-IO (browser interface) + NT-WORLD | **Production** | **MEDIUM** — browser UI layer overlaps NT-IO |
| 20 | **LongHorizon-Harness** (cross-ref) | `github.com/AMAP-ML/LongHorizon-Harness` | Loop Engineering | See above — also supports browser/desktop automation | NT-CORE + NT-WORLD | Research | LOW |
| 21 | **Claude Video** | `github.com/bradautomates/claude-video` | Video Agent | Claude-based video generation and manipulation agent | NT-ACT (creative agent) | **Early** | **LOW** — niche capability; minimal overlap |
| 22 | **Facebook/Astryx** | `github.com/facebook/astryx` | Design System | ⚠️ **NOT an agent framework** — design system for internal tools; exclude from this category | N/A | N/A | N/A |
| 23 | **Paseo** (cross-ref) | `github.com/getpaseo/paseo` | Remote Agent Daemon | Supports browser-based agent management | NT-IO | Production | MEDIUM |

### 1.4 Memory & Knowledge Agents

| # | Name | URL | Category | Core Capability | NeoTrix Mapping | Maturity | Redundancy Risk |
|---|------|-----|----------|----------------|-----------------|----------|-----------------|
| 24 | **Claude-Mem** | `github.com/thedotmack/claude-mem` | Agent Memory | Persistent context across sessions; AI-compressed observations injected into future sessions; 4,182+ observations in live demo | NT-MEMORY (persistent memory) | **Production** (Apache 2.0, CMEM Cloud commercial) | **HIGH** — directly overlaps NT-MEMORY's core function; cmem.ai live sync is a near-identical concept |
| 25 | **Memmy Agent** | `github.com/MemTensor/memmy-agent` | Cross-Agent Memory | Shared memory hub for all AI agents; supports Claude Code, Codex, OpenClaw, Hermes; 1,280 stars | NT-MEMORY | **Growth** | **HIGH** — cross-agent memory is a direct NT-MEMORY competitor |
| 26 | **Agent Memory Atlas** | `github.com/neoneye/agent-memory-atlas` | Memory Framework | Persistent memory server with iii engine, confidence scoring, knowledge graphs, hybrid search; 53 MCP tools | NT-MEMORY | **Growth** | **HIGH** — memory architecture directly overlaps NT-MEMORY |
| 27 | **Oh My Hermes** (rlaope) | `github.com/rlaope/oh-my-hermes` | Multi-Agent Orchestration | Workflow layer for Hermes Agent; 7 agents covering product lifecycle; CTO loop | NT-META (orchestration) + NT-MEMORY | **Early** (858 stars) | **MEDIUM** — orchestration patterns overlap NT-META |
| 28 | **Oh My Hermes** (witt3rd) | `github.com/witt3rd/oh-my-hermes` | Multi-Agent Orchestration | Composable skills for consensus planning, requirements interviewing, verified execution; OMH pattern | NT-META + NT-ACT | **Growth** (315 stars) | **MEDIUM** — consensus planning pattern is novel |
| 29 | **Claude-Mem** (cross-ref) | `github.com/thedotmack/claude-mem` | Agent Memory | See above | NT-MEMORY | Production | HIGH |

### 1.5 Research & Scientific Agents

| # | Name | URL | Category | Core Capability | NeoTrix Mapping | Maturity | Redundancy Risk |
|---|------|-----|----------|----------------|-----------------|----------|-----------------|
| 30 | **Autoresearch** (cross-ref) | `github.com/karpathy/autoresearch` | Autonomous Research Loop | See above | NT-MIND | Mature | LOW |
| 31 | **HolmesGPT** | `github.com/HolmesGPT/holmesgpt` | Investigation Agent | AI-powered investigation and research agent with Langfuse tracing | NT-CORE (reasoning) + NT-MIND | **Early** | **LOW** — investigation focus is niche |
| 32 | **Storm** | `github.com/stanford-oval/storm` | Research Agent | Stanford's research agent framework for deep research with web search, document processing, MCP protocol | NT-CORE (research) + NT-META | **Research** | **MEDIUM** — deep research pattern overlaps NT-MIND |
| 33 | **Fermats Last Theorem** | `github.com/anthropics/fermats-last-theorem` | Formalization Agent | Anthropic's multi-agent formalization of Fermat's Last Theorem in Lean; 13M lines, 29,500 theorems | NT-CORE (formal reasoning) | **Milestone** (published proof) | **LOW** — formal math is niche; proves multi-agent collaboration at scale |
| 34 | **BountyForge** | `github.com/gabson0x/bountyforge` | Bounty Agent System | Agent-based bounty/task system | NT-ACT (task delegation) | **Early** | **LOW** |
| 35 | **HybridAIOne/HybridClaw** | `github.com/HybridAIOne/hybridclaw` | Hybrid Agent | Hybrid AI agent combining multiple model capabilities | NT-MIND + NT-ACT | **Early** | **LOW** |

### 1.6 Specialized & Utility Agents

| # | Name | URL | Category | Core Capability | NeoTrix Mapping | Maturity | Redundancy Risk |
|---|------|-----|----------|----------------|-----------------|----------|-----------------|
| 36 | **Agency Agents** | `github.com/msitarzewski/agency-agents` | Multi-Agent System | Multi-agent collaboration framework | NT-META | **Early** | **MEDIUM** |
| 37 | **Design Studio AI** | `github.com/bestagentkits/design-studio-ai` | Design Agent | AI-powered design studio | NT-WORLD (creative) | **Early** | **LOW** |
| 38 | **Oryx** | `github.com/wmahfoudh/oryx` | Agent Framework | AI agent framework (limited data) | NT-ACT | **Unknown** | **MEDIUM** |
| 39 | **Open-Code-Review** (cross-ref) | `github.com/alibaba/open-code-review` | Code Review Agent | See above | NT-SHIELD | Production | MEDIUM |
| 40 | **Colibri** | `github.com/JustVugg/colibri` | Lightweight Agent | Lightweight AI agent (limited data) | NT-ACT | **Early** | **LOW** |
| 41 | **Kelos** | `github.com/kelos-dev/kelos` | Agent Framework | Agent framework (limited data) | NT-ACT | **Early** | **MEDIUM** |
| 42 | **Asset Studio** | `github.com/zorrobyte/asset-studio` | Asset Generation | AI asset generation studio | NT-ACT (creative) | **Early** | **LOW** |
| 43 | **FlareMo** | `github.com/realchendahuang/FlareMo` | Blockchain Agent | Flare blockchain agent (specialized) | NT-ACT | **Early** | **LOW** |
| 44 | **Oh My Hermes** (shuangxipop1) | `github.com/shuangxipop1/oh-my-hermes` | Multi-Agent Orchestration | Port of OmO patterns to Hermes AI; /autopilot, /ralph, /team, /plan commands | NT-META | **Early** (2 stars) | **MEDIUM** |

> **Note**: Duplicate URL `rlaope/oh-my-hermes` appears twice in the list. `facebook/astryx` is a design system, NOT an agent framework — excluded from agent categorization.

---

## 2. TOP 5 ARCHITECTURAL PATTERNS FOR NeoTrix ABSORPTION

### Pattern 1: **Self-Evolving Experiment Ratchet Loop** (Autoresearch Pattern)

**Source**: karpathy/autoresearch (95K stars), Bilevel Autoresearch

**Description**: An agent proposes a change → executes → measures → keeps only improvements. The "ratchet" ensures monotonic progress. Extended by Bilevel Autoresearch where an outer loop meta-optimizes the inner search loop.

**Why NeoTrix Needs It**: Directly maps to NT-MIND's self-evolution cycle. The ratchet loop (modify → measure → keep/discard) is the fundamental pattern for agent self-improvement. Bilevel extension (outer loop optimizing search mechanisms) fills a critical gap in NeoTrix's meta-cognitive architecture.

**Implementation Target**: `neotrix-core/src/l5_cognition/nt_mind/` — add ratchet loop primitive with configurable evaluation harnesses.

**Maturity**: High — proven at 700+ experiments over 2 days (Karpathy's original run).

**Redundancy**: LOW — no equivalent in NeoTrix current codebase.

---

### Pattern 2: **Manage-Execute-Audit (MEA) Loop** (LongHorizon-Harness Pattern)

**Source**: AMAP-ML/LongHorizon-Harness (1,470 stars, WeaveBench 80.7%)

**Description**: Separates long-horizon execution into three roles: Manager (defines subtasks from task state), Executor (performs in fresh context), Auditor (independently verifies environment). Task state maintained externally and updated only with verified facts.

**Why NeoTrix Needs It**: Fills a critical gap in NT-CORE's execution primitives. Current NeoTrix lacks formalized state-externalization and independent auditing for long-running tasks. The MEA loop's "fresh context execution" + "independent verification" prevents compounding errors — a known weakness in current agent systems.

**Implementation Target**: `neotrix-core/src/l1_action/nt_act/` — add MEA loop orchestrator with AgentAdapter pattern.

**Maturity**: High — published paper (arXiv:2608.01964), +28.9% benchmark improvement.

**Redundancy**: LOW — novel pattern not present in NeoTrix.

---

### Pattern 3: **Persistent Cross-Agent Memory with Sync** (Claude-Mem / Memmy Pattern)

**Source**: thedotmack/claude-mem (cmem.ai), MemTensor/memmy-agent (1,280 stars)

**Description**: A shared memory layer that all AI agents write to and read from. Observations are AI-compressed, stored with temporal indexing, and injected into future sessions. Cross-agent sync via MCP links or cloud gateways. Supports offline-first with conflict resolution.

**Why NeoTrix Needs It**: NT-MEMORY already has a memory architecture, but lacks the cross-agent sync and compression patterns proven by Claude-Mem. The MCP-based shared memory link is a production-proven pattern. The "observations → compressed → injected" pipeline directly improves agent continuity.

**Implementation Target**: `neotrix-core/src/l1_action/nt_memory/` — add cross-agent sync, AI-compression pipeline, and MCP memory endpoint.

**Maturity**: High — claude-mem is Apache 2.0 with live production deployment (4,182+ observations).

**Redundancy**: **MEDIUM-HIGH** — NT-MEMORY exists but lacks cross-agent sync and compression. Absorption is additive, not duplicative.

---

### Pattern 4: **Agent Canvas / Unified Control Plane** (OpenHands Pattern)

**Source**: OpenHands/OpenHands (86K stars, MLSys 2026)

**Description**: A self-hosted developer control center that manages multiple coding agents across backends (Claude Code, Codex, Gemini, OpenHands). Features: Agent Canvas UI, automation scheduling, model-agnostic routing, sandboxed execution, and lifecycle control. ACP (Agent Client Protocol) compatibility.

**Why NeoTrix Needs It**: NT-ACT currently lacks a unified control plane for managing multiple agent instances. OpenHands' Agent Canvas model — where agents are always-on, schedulable, and backend-switchable — is a production pattern NeoTrix should adopt. The ACP protocol compatibility is particularly valuable for NeoTrix's multi-backend strategy.

**Implementation Target**: `neotrix-core/src/l1_action/nt_act/` — add Agent Canvas control plane with ACP protocol support.

**Maturity**: Very High — MLSys 2026 paper, 86K stars, production deployments.

**Redundancy**: **MEDIUM** — NT-ACT has agent management but lacks the canvas/scheduling/backend-switching abstraction.

---

### Pattern 5: **Role-Based Multi-Agent Delegation with Consensus** (CrewAI / Oh-My Patterns)

**Source**: crewAIInc/crewAI (58K stars), oh-my-claudecode variants, witt3rd/oh-my-hermes

**Description**: Agents are assigned roles (Researcher, Writer, Editor, Architect, Critic, Executor) with defined goals and backstories. Tasks are delegated through hierarchical or sequential workflows. Consensus planning (Planner → Architect → Critic debate) ensures quality before execution. Team Mode enables parallel agent coordination.

**Why NeoTrix Needs It**: This is the most widely adopted multi-agent pattern and directly maps to NT-META's orchestration layer. CrewAI's Crew/Flow duality and the consensus planning pattern from oh-my variants fill gaps in NeoTrix's current agent delegation model. The "Planner → Architect → Critic" three-role consensus is particularly valuable for quality-critical tasks.

**Implementation Target**: `neotrix-core/src/l6_meta/nt_meta/` — add role-based delegation, consensus planning, and team orchestration primitives.

**Maturity**: Very High — crewAI has 2B+ task executions, 150+ enterprise customers.

**Redundancy**: **HIGH** — NT-META already has some orchestration patterns. Absorption requires careful deduplication rather than new implementation.

---

## 3. NEOTRIX MAPPING SUMMARY

### By NT Domain

| NT Domain | Projects Mapped | Count | Primary Overlap |
|-----------|----------------|-------|-----------------|
| **NT-ACT** | crewAI, OpenHands, Aider, Eve, OmO, Oh-My variants, Claude-Task-Master, browser-use | 8 | Agent execution, coding, task delegation |
| **NT-CORE** | LongHorizon-Harness, Orchard, Fermats-LT, Storm, HolmesGPT, BountyForge | 6 | Execution loops, modeling, formal reasoning |
| **NT-MIND** | Hermes-Agent, Autoresearch, HybridClaw | 3 | Self-evolution, autonomous research |
| **NT-WORLD** | Browser-Use, Claude-Video, Design-Studio-AI | 3 | Browser automation, creative, physical |
| **NT-MEMORY** | Claude-Mem, Memmy, Agent-Memory-Atlas | 3 | Persistent memory, cross-agent sync |
| **NT-IO** | Paseo, Octop, Eve (channels), Browser-Use (web-ui) | 4 | Remote access, protocols, interfaces |
| **NT-SHIELD** | Open-Code-Review, Defending-Code-Harness, Oh-My-OpenAgent (hooks) | 3 | Code review, security, guardrails |
| **NT-META** | Orchard (modeling), CrewAI (orchestration), Oh-My variants (orchestration), Agency-Agents | 4 | Meta-cognition, orchestration patterns |

### Maturity Distribution

| Maturity Level | Count | Projects |
|----------------|-------|----------|
| **Production (10K+ stars, enterprise adoption)** | 8 | crewAI, OpenHands, Aider, Autoresearch, Browser-Use, Claude-Mem, Hermes-Agent, Eve |
| **Growth (1K-10K stars, active development)** | 10 | LongHorizon-Harness, Orchard, Paseo, Memmy, Oh-My variants, Claude-Mem alternatives |
| **Research/Proof (papers, milestones)** | 5 | Fermats-LT, LongHorizon-Harness, Storm, Orchard, Bilevel Autoresearch |
| **Early/Niche (<1K stars)** | 20 | Various specialized agents, utility tools, forks |

---

## 4. REDUNDANCY RISK ASSESSMENT

### HIGH Risk (Duplicate Capability — Must Differentiate or Integrate)
1. **NT-ACT ↔ crewAI/OpenHands**: Both provide agent orchestration and coding agent capabilities. NeoTrix must differentiate through its 6-layer architecture and Dark Forest security model.
2. **NT-MEMORY ↔ Claude-Mem/Memmy**: Cross-agent persistent memory is a near-duplicate. Absorption should add NeoTrix's VSA HyperCube memory encoding as a differentiator.
3. **NT-META ↔ Oh-My variants**: Role-based multi-agent delegation overlaps heavily. NeoTrix should absorb the consensus planning pattern but differentiate through its own orchestration primitives.
4. **NT-WORLD ↔ Browser-Use**: Browser automation directly overlaps. NeoTrix's browser capabilities should integrate the self-healing harness pattern.

### MEDIUM Risk (Adjacent Capability — Absorb Selectively)
5. **NT-IO ↔ Paseo/Octop**: Remote agent access and ACP protocol. NeoTrix should adopt the daemon pattern but keep its own agent communication protocol.
6. **NT-SHIELD ↔ Open-Code-Review**: Code review guardrails. Absorb the automated review pattern into NT-SHIELD.
7. **NT-CORE ↔ LongHorizon-Harness/Orchard**: Execution loop infrastructure. Absorb MEA loop and sandbox patterns.

### LOW Risk (Unique Capability — Direct Absorption Recommended)
8. **Autoresearch ratchet loop** — No equivalent in NeoTrix.
9. **MEA loop** — No equivalent in NeoTrix.
10. **Bilevel autoresearch** — Novel meta-optimization pattern.
11. **Formal verification agents** (Fermats-LT) — Niche but proves multi-agent at scale.

---

## 5. RECOMMENDED ABSORPTION PRIORITY

### Priority 1: Immediate (This Cycle)
| Pattern | Source | NT Module | Rationale |
|---------|--------|-----------|-----------|
| Self-Evolving Ratchet Loop | Autoresearch | NT-MIND | Core self-improvement, no existing equivalent |
| MEA Loop | LongHorizon-Harness | NT-CORE | Critical execution primitive, no equivalent |
| Cross-Agent Memory Sync | Claude-Mem | NT-MEMORY | Additive to existing NT-MEMORY |

### Priority 2: Near-Term (Next Cycle)
| Pattern | Source | NT Module | Rationale |
|---------|--------|-----------|-----------|
| Agent Canvas Control Plane | OpenHands | NT-ACT | Unified agent management |
| Consensus Planning (Planner-Architect-Critic) | Oh-My-Hermes | NT-META | Quality gate for agent delegation |
| Subagent Delegation | Eve/CrewAI | NT-META | Hierarchical task breakdown |

### Priority 3: Strategic (Future Cycles)
| Pattern | Source | NT Module | Rationale |
|---------|--------|-----------|-----------|
| Sandbox Environment Service | Orchard | NT-CORE | Kubernetes-native sandbox abstraction |
| Bilevel Meta-Optimization | Bilevel Autoresearch | NT-META | Outer loop optimization of search |
| Formal Verification Agents | Fermats-LT | NT-SHIELD | Multi-agent formal reasoning |

---

## 6. KEY OBSERVATIONS

1. **The agent ecosystem is consolidating around 5 paradigms**: role-based orchestration (crewAI), autonomous coding (OpenHands/Aider), self-evolving loops (Autoresearch/Hermes), browser automation (Browser-Use), and persistent memory (Claude-Mem).

2. **NeoTrix's 6-layer architecture already covers most of these**, but specific implementation patterns (ratchet loop, MEA loop, cross-agent memory sync) are missing and should be absorbed directly.

3. **The biggest redundancy risk is NT-ACT** which overlaps with crewAI, OpenHands, and Oh-My variants simultaneously. NeoTrix must differentiate through its unique VSA HyperCube knowledge representation and Dark Forest security model.

4. **The Autoresearch ratchet loop is the single most valuable pattern** for NeoTrix absorption — it directly enables NT-MIND's self-evolution capability with a proven, production-tested mechanism.

5. **Memory is the battleground**: Claude-Mem, Memmy, and Agent-Memory-Atlas all target the same space. NeoTrix's NT-MEMORY should absorb the cross-agent sync and AI-compression patterns but differentiate through its VSA-based memory encoding.

---

*Report generated for NeoTrix project absorption analysis. All data sourced from GitHub repositories, academic papers, and community sources as of September 2026.*
