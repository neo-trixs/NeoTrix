# Trending Rankings — Cycle 406 (2026-09-12)

## 10 New Projects (Not in Cycles 318-405)

### 1. Kodama (RAGEN)
- **GitHub**: https://github.com/mll-lab-nu/RAGEN
- **Stars**: 2.3K (Jul 2026) | **Category**: Agent RL Framework
- **What**: Reinforcement learning framework for LLM agents with multi-turn RL via StarPO and reasoning-collapse diagnostics. Group-in-Group Policy Optimization (GiGPO) for training LLM/VLM agents. Official code for "Group-in-Group Policy Optimization for LLM Agent Training." Trains agents through environment interaction rather than static demonstrations.
- **Key Pattern**: RL-based agent training — agents learn through trial-and-error in interactive environments, with diagnostic tools to detect reasoning collapse (common failure mode where agents output confident nonsense). GiGPO extends GRPO with hierarchical group structures.
- **NeoTrix Relevance**: NT-MIND (self-evolution via RL, reasoning-collapse detection), NT-CORE (reasoning quality diagnostics), NT-ACT (agent behavior training). Validates NeoTrix's SEAL pipeline — agents need RL loops, not just prompt engineering. Reasoning-collapse diagnostic maps to NeoTrix's self-deception detection (D15).

### 2. Ouuroboros
- **GitHub**: https://github.com/Q00/ouroboros
- **Stars**: 310 | **Category**: Agent OS with Bounded Evolution
- **What**: "Agent OS: the agent gets smarter on its own." Interview-gated, staged evaluation, budgeted evolution loop. 13 runtimes: Claude Code, Codex CLI, Gemini CLI, OpenCode, Copilot, Kiro, etc. MCP server for cross-runtime coordination. The grading command and expected result never enter the success contract — agents self-improve under constraints they cannot manipulate.
- **Key Pattern**: Constrained self-improvement — success criteria are externalized and immutable (agent cannot game the grading). Staged evaluation with interview gates prevents reward hacking. Budgeted evolution loops prevent unbounded recursive self-modification.
- **NeoTrix Relevance**: NT-MIND (bounded self-improvement, staged evaluation), NT-GOVERNANCE (immutable success criteria), NT-REPAIR (interview-gated repair). Validates NeoTrix's SEAL pipeline with ceiling and external verification — critical for safe self-evolution.

### 3. BrowserAct
- **GitHub**: https://github.com/browseract/browser-act
- **Stars**: 3,847 (Jun 2026) | **Category**: Real-World Browser Agent Runtime
- **What**: #1 on ProductHunt (Jun 25, 2026). Real browser control for AI agents: session management, verification handling, remote human handoff, reusable skills. Two layers: browser-act (execution runtime) and browser-act-skill-forge (reusable skill creation). Handles login sessions, CAPTCHA, dynamic UI changes. Agents survive real-world web conditions.
- **Key Pattern**: Session-continuity-first browser automation — most agent failures happen mid-task (session expiry, verification, human judgment). BrowserAct keeps workflows alive across interruptions. Skill-forge turns repeatable website workflows into reusable agent skills.
- **NeoTrix Relevance**: NT-WORLD (real-world web interaction), NT-ACT (browser agent runtime), NT-PHYSICAL (human-in-the-loop handoff). Validates NeoTrix's UnifiedCrawler pattern — web agents need session persistence, not stateless HTTP calls.

### 4. Creed
- **GitHub**: https://github.com/creed-ai/creed
- **Stars**: 163 (Jul 2026) | **Category**: Unified Agent Context
- **What**: "Your personal context file for every agent." Creates a unified context profile (6-dimension: identity, style, audience, platforms, preferences, memory) that all AI agents reference for personalized interactions. GitHub integration for profile sync. Solves fragmented AI assistants not sharing user context across tools.
- **Key Pattern**: Profile-as-context-substrate — persistent cross-agent identity and preference layer. All agents read the same context file, enabling consistent behavior across GitHub Copilot, ChatGPT, Claude, Cursor without per-tool configuration.
- **NeoTrix Relevance**: NT-MEMORY (cross-session persistent context), NT-IO (multi-agent profile sharing), NT-FEEL (user preference modeling). Validates NeoTrix's SelfModel pattern (profile-driven adaptation, Axiom P3) — user identity persists across agent invocations.

### 5. Rintagi
- **GitHub**: https://github.com/rintagi/rintagi
- **Stars**: 5.8K | **Category**: Low-Code AI Agent Builder
- **What**: Enterprise low-code platform for building AI agents. Visual workflow designer, multi-model support, built-in RAG, role-based access control. Agents deployed as microservices with monitoring dashboards. Used by enterprises for customer service, data analysis, and process automation.
- **Key Pattern**: Agent-as-microservice — visual workflow → containerized deployment → monitoring observability. Enterprise-grade RBAC + audit trails. Low-code for rapid prototyping, escape hatches for custom logic.
- **NeoTrix Relevance**: NT-ACT (agent orchestration, workflow design), NT-GOVERNANCE (RBAC, audit trails), NT-IO (multi-model routing). Validates NeoTrix's capability-as-component pattern — agents need governance, not just capability.

### 6. InReason
- **Website**: https://inreason.ai | **Stage**: Unfunded (2026)
- **Category**: Neuro-Symbolic Reasoning
- **What**: Neuro-symbolic systems for reliable and explainable reasoning. Separates neural intuition from formal symbolic logic — perception handled by neural nets, reasoning by formal logic. Observable and auditable processes. Reduces reliance on large datasets, mitigates probabilistic model errors. Targeting safety-critical domains (healthcare, autonomous systems).
- **Key Pattern**: Dual-pathway reasoning — neural perception feeds into symbolic reasoning. Each step auditable. No black-box intermediate states. Contrast with pure neural approaches: gains explainability at the cost of flexibility.
- **NeoTrix Relevance**: NT-CORE (reasoning architecture, E8 hexagram as symbolic substrate), NT-SHIELD (auditability, safety-critical reasoning), NT-GOVERNANCE (formal verification). Validates NeoTrix's E8 + GWT dual-pathway pattern — symbolic + neural reasoning in complementary layers.

### 7. DeMAC (Dynamic Manager-Player Coordination)
- **Paper**: ACL 2026 (Sep 2026) | **Category**: Multi-Agent Coordination
- **What**: Dynamic Environment-Aware Manager-Player Agents Coordination framework. Uses dynamically updated DAG and Manager-Player Dual-Feedback mechanism for long-term strategic planning. Agents maintain collaboration and adapt to changing environments. Outperforms RL and human-agent collaboration in Overcooked simulation. Addresses shifting priorities and unpredictable disruptions.
- **Key Pattern**: Dynamic DAG coordination — task dependency graph evolves in real-time. Manager-Player dual feedback: strategic (manager) and operational (player) decision alignment. Agents adapt to environmental drift, not just static task decomposition.
- **NeoTrix Relevance**: NT-ACT (dynamic task orchestration), NT-CORE (DAG-based reasoning substrate), NT-MIND (adaptive strategy). Validates NeoTrix's GWT attention routing — dynamic salience recalculation as environments change.

### 8. MemMA (Memory Cycle Coordination)
- **Paper**: arXiv:2603.18718 (Mar 2026, ACL 2026) | **Category**: Agent Memory Architecture
- **What**: Plug-and-play multi-agent framework coordinating the memory cycle. Forward path: Meta-Thinker guides Memory Manager (construction) and Query Reasoner (retrieval). Backward path: in-situ self-evolving memory — synthesizes probe QA pairs, verifies memory, converts failures into repair actions before finalization. Outperforms baselines on LoCoMo across multiple LLM backends.
- **Key Pattern**: Forward-backward memory cycle — construction and retrieval are coordinated (not isolated). Memory self-repair via QA probe synthesis — memory validates itself before serving. Plug-and-play across different storage backends.
- **NeoTrix Relevance**: NT-MEMORY (coordinated memory construction/retrieval), NT-MIND (self-evolving memory, backward repair), NT-REPAIR (memory self-healing). Validates NeoTrix's KB pipeline — memory needs forward planning + backward repair, not just write-then-read.

### 9. DeLM (Decentralized Language Models)
- **Paper**: arXiv:2606.10662 (Jun 2026) | **Category**: Decentralized Multi-Agent
- **What**: Decentralizes multi-agent coordination through parallel agents, shared verified context, and task queue. Agents asynchronously claim subtasks, read accumulated progress, perform local reasoning, write back compact verified updates. Shared context acts as common communication substrate. +10.5pp on SWE-bench Verified, -50% cost per task vs centralized orchestration. Best Avg.@1, Pass@2, Pass@4.
- **Key Pattern**: Decentralized task execution with shared context — no central controller bottleneck. Agents build on verified progress without routing through a single orchestrator. Asynchronous claim-and-write with verified updates prevents information loss.
- **NeoTrix Relevance**: NT-ACT (decentralized agent coordination), NT-MEMORY (shared context as substrate), NT-CORE (verified progress propagation). Validates NeoTrix's EventBus pattern — decentralized event propagation with verification, not centralized request-response.

### 10. Flux Attention
- **Paper**: arXiv:2604.07394 (Apr 2026) | **Category**: Efficient Attention Mechanism
- **What**: Context-aware hybrid attention for efficient LLM inference. Lightweight Layer Router adaptively routes each layer to Full Attention or Sparse Attention based on input context. Layer-level routing (not head-level) preserves contiguous memory access for hardware acceleration. +2.8x prefill speedup, +2.0x decode speedup. Only 12 hours training on 8x A800 GPUs.
- **Key Pattern**: Layer-wise attention routing — each transformer layer dynamically selects FA or SA based on input complexity. Avoids head-level sparsity that creates hardware load imbalance. Dynamic penalty mechanism prevents router degeneration (always choosing FA).
- **NeoTrix Relevance**: NT-CORE (attention routing optimization, GWT refinement), NT-PHYSICAL (hardware-aware inference), NT-MEMORY (KV cache optimization). Validates NeoTrix's GWT attention mechanism — layer-level routing mirrors domain-level attention allocation.

## Trend Analysis

| Signal | Count | Implication |
|--------|-------|-------------|
| Agent memory architecture | 3/10 | MemMA + DeLM + Creed = memory as first-class citizen, not append-only log |
| Decentralized coordination | 2/10 | DeLM + DeMAC = moving beyond centralized orchestrator bottleneck |
| Efficient attention/routing | 2/10 | Flux Attention + layer-wise routing = hardware-aware inference optimization |
| Bounded self-improvement | 2/10 | Ouuroboros + Kodama = constrained evolution with verification gates |
| Real-world browser agents | 2/10 | BrowserAct + RAGEN = agents that survive real web conditions |
| Neuro-symbolic reasoning | 1/10 | InReason = formal verification + neural perception hybrid |
| Open-source agent infrastructure | 7/10 | Open-source still dominant in agent tooling (Kodama, BrowserAct, Creed, etc.) |

## Cross-Cycle Absorption Candidates

| Project | Absorption Target | Pattern |
|---------|------------------|---------|
| Kodama GiGPO + collapse diagnostics | NT-MIND reasoning quality | RL-based agent training with failure detection |
| Ouuroboros interview-gated evolution | NT-MIND safe self-improvement | Immutable success criteria prevent reward hacking |
| BrowserAct session continuity | NT-WORLD web agent runtime | Session persistence across interruptions |
| Creed unified context profile | NT-MEMORY cross-agent identity | 6-dimension profile as context substrate |
| MemMA forward-backward memory cycle | NT-MEMORY coordinated memory | Construction + retrieval + self-repair loop |
| DeLM decentralized coordination | NT-ACT distributed execution | Shared context substrate, async claim-and-write |
| DeMAC dynamic DAG coordination | NT-CORE adaptive reasoning | Real-time task dependency evolution |
| Flux Attention layer routing | NT-CORE attention optimization | Layer-wise FA/SA dynamic routing |
| InReason neuro-symbolic | NT-CORE dual-pathway reasoning | Neural perception + formal logic auditing |
| Rintagi agent-as-microservice | NT-ACT agent deployment | Visual workflow → containerized observability |

## Key Meta-Trends (Cycle 406)

1. **Memory is the new compute bottleneck** — 3/10 projects focus on memory architecture (MemMA, DeLM, Creed). Memory is no longer "store and retrieve" but "coordinate, verify, repair."
2. **Decentralization beats centralization** — DeLM shows -50% cost with decentralized coordination. The orchestrator bottleneck is real.
3. **Self-improvement needs guardrails** — Ouuroboros's "grading command never enters success contract" is the key insight. Agents cannot be trusted to verify their own improvement.
4. **Real-world agents need session persistence** — BrowserAct's core thesis: agents fail not at step 1, but at step N when sessions expire. Session continuity is infrastructure, not feature.
5. **Hardware-aware inference is table stakes** — Flux Attention's 2.8x speedup with layer-level routing shows attention optimization is moving from research to production.
