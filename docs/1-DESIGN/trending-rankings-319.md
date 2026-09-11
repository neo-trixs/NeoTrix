# Trending Rankings — Cycle 319

**Date**: 2026-09-11
**Focus**: AI agents, LLM tools, developer productivity, attention/memory/reasoning patterns

---

## 10 New Projects (Not in Previous Cycles)

### 1. GenericAgent
- **GitHub**: https://github.com/lsdefine/GenericAgent
- **Stars**: ~14,100
- **Description**: Minimal self-evolving autonomous agent framework. ~3K lines of core code. Automatically crystallizes each task into a reusable Skill. 9 atomic tools (code_run, file_read, file_write, file_patch, web_scan, web_execute_js, ask_user, update_working_checkpoint, start_long_term_update). Achieves full system control with <30K context window.
- **Key Pattern**: **Self-evolving skill tree from seed code** — capabilities grow with every use, forming personal skill tree from 3K lines. Token-efficient via contextual information density maximization.
- **NeoTrix Relevance**: Maps directly to NT-MIND (skill crystallization, self-evolution). The skill tree growth pattern mirrors our SEAL pipeline stages. The "Morphling mode" (project-level skill absorption) parallels R-P42/R-P79 absorption rules. The "Goal Hive" multi-worker coordination maps to NT-ACT orchestration.

### 2. AweAgent
- **GitHub**: https://github.com/AweAI-Team/AweAgent
- **Stars**: ~3,000+ (growing fast)
- **Description**: Unified composable framework for building, evaluating, and training agents. Split into scaffold (policy), AgentLoop (rollout engine), AgentContext (shared bus), and interaction layer (LLM/tools/runtime). 5 scaffolds: SearchSWE, DeepSearch, IterResearch, Terminus-2, CalibForge. RL training via Slime bridge.
- **Key Pattern**: **Separation of scaffold (policy) from loop (engine) from context (bus)** — clean agent architecture with protocol-centered extensibility. Task-specific behavior composed through reusable interfaces.
- **NeoTrix Relevance**: The scaffold/loop/context separation mirrors our 6-layer architecture (L1-L6). The AgentContext bus pattern maps to EventBus. The multi-scaffold approach parallels our Dual Specialization (Weapon Set I/II). The RL training integration is relevant to NT-MIND evolution.

### 3. Vercel Eve
- **GitHub**: https://github.com/vercel/eve
- **Stars**: ~4,900
- **Description**: Filesystem-first framework for durable AI agents. Core agent capabilities live in conventional file locations (agent.ts, instructions.md, tools/, skills/, channels/, schedules/). Includes human-in-the-loop prompts, subagents, and cron schedules.
- **Key Pattern**: **Filesystem-as-configuration** — agent state lives in structured files rather than scattered config. Durable by convention.
- **NeoTrix Relevance**: The filesystem-first approach aligns with our KB-backed persistent state. The skills/ directory convention maps to our skill node system. The channels/ pattern is relevant to NT-IO interface design.

### 4. Omnigent
- **GitHub**: https://github.com/omnigent-ai/omnigent
- **Stars**: ~9,000
- **Description**: Meta-harness orchestrating Claude Code, Codex, Cursor, OpenCode, Hermes, Pi, and custom agents. Swap harnesses without rewriting. Policy governance (pause for approval, cap spend, limit tools). Cloud sandboxes (Modal, Daytona, E2B, K8s). Includes "Polly" (multi-agent coding orchestrator), "Debby" (two-headed brainstorming with Claude+GPT), and "Deep Research" (cited cross-checked reports).
- **Key Pattern**: **Agent governance as first-class concern** — policies apply at server/agent/chat level. The "Polly" pattern of routing each diff to a reviewer from a different vendor is novel cross-vendor verification.
- **NeoTrix Relevance**: Maps to NT-SHIELD (governance, sandboxing) and NT-GOVERNANCE (policy enforcement). The cross-vendor review pattern is relevant to our rev-officer review loops. The multi-agent orchestration parallels our ConsciousnessTree coordination.

### 5. Cortex Agent Framework
- **GitHub**: https://github.com/kritird/Cortex-Agent-Framework
- **Stars**: ~1 (very new, but architecturally significant)
- **Description**: YAML-driven agent framework. Single cortex.yaml defines agent behavior. Intent Gate routes chat vs task turns. Wave engine with LLM-generated DAG + parallel execution. Learning Engine observes task patterns and stages delta proposals. Any agent becomes MCP server. Ant Colony orchestrator self-spawns specialist agents at runtime.
- **Key Pattern**: **Intent Gate** (cheap heuristic + LLM cascade routes chat vs task turns) and **Ant Colony** (orchestrator self-spawns specialist agents as MCP servers with health checking). The Learning Engine's delta proposal pattern (surface what it learned, ask for approval) is governance-aware evolution.
- **NeoTrix Relevance**: Intent Gate maps to GWT salience routing. Ant Colony parallels our CapabilityRegistry + CapabilityBridge. The Learning Engine's delta proposal pattern is exactly our SEAL pipeline's absorption protocol (stage → review → apply). The YAML-as-contract pattern is relevant to SKILL-SPEC.md.

### 6. MemMA (Memory Cycle Multi-Agent Coordination)
- **Paper**: https://arxiv.org/abs/2603.18718
- **Code**: https://github.com/ventr1c/memma
- **Description**: Plug-and-play multi-agent framework coordinating memory cycle along forward and backward paths. Forward: Meta-Thinker (strategic reasoning) + Memory Manager (construction) + Query Reasoner (iterative retrieval). Backward: in-situ self-evolving memory — synthesizes probe QA pairs, verifies memory, converts failures into repair actions before memory is committed.
- **Key Pattern**: **In-situ memory self-evolution** — generate probe QA pairs after each session, verify memory against them, repair failures before memory is committed. Separates strategic reasoning (Meta-Thinker) from execution (Memory Manager). This is "verify before commit" for memory systems.
- **NeoTrix Relevance**: Directly maps to NT-MEMORY (KB verification, experience-tree quality gate). The backward path pattern is relevant to our SEAL Phase-0 converge_check. The Meta-Thinker/Worker split parallels our GWT attention routing + specialist execution.

### 7. Gated-Memory Routing
- **Paper**: https://arxiv.org/abs/2609.00237
- **Code**: https://github.com/rajibrhasan/gated-memory-routing
- **Description**: Multi-agent coordination via learned execution memory. Memory Write Gate commits only non-redundant reasoning steps. Retrieval Gate supplies each agent a compact, relevant subset. Adaptive Halting Controller stops when memory contains sufficient evidence. End-to-end trained with reward trading answer quality against cost.
- **Key Pattern**: **Gated execution memory as shared control signal** — write and retrieval decisions trained jointly with role allocation, backbone routing, and halting. Memory determines who acts next, what they read, and when collaboration stops. Reduces HumanEval inference cost by 31.9% vs baselines.
- **NeoTrix Relevance**: Maps to NT-MEMORY (KB write gating, retrieval optimization) and GWT (adaptive halting = attention modulation). The joint training of memory+routing+halting parallels our ConsciousnessTree feedback loop. The cost-aware halting is relevant to Axiom A1 (Cost-Aware Routing).

### 8. CAMA (Correlation-Aware Memory Arbitration)
- **Paper**: https://arxiv.org/abs/2608.19701
- **Description**: Addresses "Memory Correlation Bias" — correlated memories from different agents creating false majorities. Models memories as latent evidence slots, estimates effective independent evidence sources using Hill diversity measure. Learns sequential recovery policy to actively retrieve alternative evidence before decision.
- **Key Pattern**: **Evidence decoupling** — prevents correlated memories from being counted multiple times. Sequential recovery policy expands retrieval space only when current evidence is insufficient. Evaluates at latent evidence factor level, not individual memory entries.
- **NeoTrix Relevance**: Directly relevant to NT-MEMORY (KB deduplication, provenance tracking). The correlation bias problem is real for our multi-agent experience absorption. The latent evidence slot modeling could improve our experience-tree retrieval quality.

### 9. Declarative Attention (Language Models Control Their Own Attention)
- **Paper**: https://arxiv.org/abs/2609.02737
- **Description**: Protocol that elicits the model to declare where it needs to attend within its chain-of-thought. Partitions generation into three modes: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output only). Inference engine parses these declarations like tool calls and skips most KV cache reads. Reduces total attended tokens by 31-52% with modest accuracy drops that shrink with model scale.
- **Key Pattern**: **Intrinsic attention routing** — the model itself declares attention regions, eliminating the need for external proxy scoring. Three-mode partition (global/focus/local) is a clean abstraction for sparse attention.
- **NeoTrix Relevance**: This is a direct analog to our GWT salience mechanism. The global/focus/local modes map to our attention routing tiers. The "model declares its own attention" pattern could inform our ConsciousnessTree — letting the reasoning engine itself declare what context is salient rather than relying on external heuristics.

### 10. Flux Attention
- **Paper**: https://arxiv.org/abs/2604.07394
- **Description**: Context-aware hybrid attention framework. Lightweight Layer Router evaluates semantic context and adaptively routes each transformer layer to Full Attention or Sparse Attention. Only the router is trained (12 hours on 8x A800). Layer-level routing preserves contiguous memory access, enabling GPU to bypass KV tensor loading for sparse layers. Up to 2.8x prefill speedup, 2.0x decode speedup at 256K context.
- **Key Pattern**: **Layer-wise adaptive routing** — different layers get different attention strategies based on input context. Coarse granularity (layer-level, not head-level) preserves hardware efficiency. Router sees task demands and assigns computation budget accordingly.
- **NeoTrix Relevance**: The layer-wise routing pattern maps to our Dual Specialization (Weapon Set switching per context). The "router sees task demands" pattern parallels our GWT salience + cost weight (Axiom A1). The parameter-efficient training (frozen backbone + lightweight router) is relevant to our SEAL pipeline evolution approach.

---

## Cross-Cutting Patterns

| Pattern | Projects | NeoTrix Mapping |
|---------|----------|-----------------|
| **Self-evolving skill trees** | GenericAgent, Cortex | NT-MIND SEAL pipeline, skill crystallization |
| **Gated memory with write-time filtering** | MemMA, Gated-Memory, MMP | NT-MEMORY KB verification, experience-tree |
| **Intrinsic attention routing** | Declarative Attention, Flux Attention | GWT salience, ConsciousnessTree |
| **Agent governance as first-class** | Omnigent, Cortex | NT-SHIELD policies, NT-GOVERNANCE |
| **Multi-agent coordination via shared state** | Gated-Memory, CAMA, Omnigent | EventBus, CapabilityBridge |
| **Layer/scaffold separation** | AweAgent, Eve | 6-layer architecture, trait contracts |
| **Intent-aware routing** | Cortex Intent Gate, GWT | Cost-Aware Routing (Axiom A1) |
| **Verify-before-commit memory** | MemMA, CAMA | SEAL Phase-0 converge_check |

---

## Priority Absorption Candidates

1. **MemMA** — In-situ memory self-evolution (backward path verification)
2. **Gated-Memory Routing** — Joint memory+routing+halting training
3. **Declarative Attention** — Model-declared attention regions (intrinsic GWT analog)
4. **Cortex Intent Gate** — Heuristic+LLM cascade routing (cost-aware)
5. **GenericAgent** — Self-evolving skill tree from minimal seed
