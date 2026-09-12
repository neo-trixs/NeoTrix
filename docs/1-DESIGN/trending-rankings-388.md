# Trending Rankings — Cycle 388 (2026-09-12)

## Cycle Metadata
- **Cycle**: 388
- **Date**: 2026-09-12
- **Previous cycles checked**: 318-387
- **Sources**: GitHub Trending, ProductHunt, arXiv, ToolCenter.ai

---

## 10 New Projects (Not in Cycles 318-387)

### 1. DeerFlow 2.0 (ByteDance)
- **URL**: https://github.com/bytedance/deer-flow
- **Stars**: 82.3K (trending #1 GitHub Sep 7)
- **What**: Open-source SuperAgent harness — orchestrates sub-agents, memory, sandboxes, and extensible skills for long-horizon tasks (minutes to hours). Ground-up rewrite from v1 (Deep Research).
- **Key Patterns**: Sub-agent dispatch, session goals, manual context compaction, long-term memory, sandbox execution, MCP server integration, multi-provider routing.
- **NeoTrix Mapping**:
  - NT-CORE: Sub-agent orchestration ↔ GWT attention routing
  - NT-MIND: Self-evolution via skill extensions ↔ SEAL pipeline
  - NT-MEMORY: Long-term memory with checkpoint system ↔ KB persistence
  - NT-ACT: Sandbox execution ↔ safe action dispatch

### 2. Superpowers (obra)
- **URL**: https://github.com/obra/superpowers
- **Stars**: 285K (Sep 10 trending)
- **What**: Agentic SDLC methodology — brainstorm → signed-off design → TDD plan → subagent execution. 13 composable skills across 13+ harnesses (Claude, Codex, Cursor, Gemini, etc).
- **Key Patterns**: Mandatory skill routing (not suggestions), two-stage subagent review (spec compliance + code quality), red-green TDD enforcement, worktree isolation.
- **NeoTrix Mapping**:
  - NT-ACT: Subagent-driven development ↔ NT-ACT orchestration
  - NT-SHIELD: Two-stage review ↔ rev-officer dual verification
  - NT-CORE: Mandatory workflow routing ↔ GWT salience-gated dispatch

### 3. Ruflo (ruvnet)
- **URL**: https://github.com/ruvnet/ruflo
- **Stars**: 72.1K (trending Sep 7)
- **What**: Agent meta-harness — 100+ specialized agents, swarm coordination, self-learning memory (SONA), federated cross-machine collaboration, 35 plugins, GOAP A* planner.
- **Key Patterns**: Decentralized agent federation (mTLS + ed25519), dual-pool vector memory (AgentDB + HNSW), behavioral trust scoring, plugin marketplace, goal-oriented action planning.
- **NeoTrix Mapping**:
  - NT-CORE: GOAP planner ↔ E8 reasoning engine
  - NT-MEMORY: AgentDB + HNSW ↔ KB vector store
  - NT-SHIELD: Federation trust model ↔ egress privacy guard
  - NT-MIND: SONA self-learning ↔ SEAL distillation

### 4. Screenpipe (YC S26)
- **URL**: https://github.com/screenpipe/screenpipe
- **Stars**: Trending ProductHunt Aug 2026
- **What**: AI agent memory via continuous screen/audio capture. Local-first, MCP server, event-driven capture (not constant recording), PII redaction on-device. Pipes = scheduled AI agents triggered by work activity.
- **Key Patterns**: Event-driven capture (app switches, clicks, typing pauses), local-first privacy, MCP server for agent queries, pipe agents (markdown-defined scheduled workflows), cross-device sync.
- **NeoTrix Mapping**:
  - NT-WORLD: Event-driven perception ↔ SensoryIntegrationHub
  - NT-MEMORY: Persistent screen history ↔ KB temporal storage
  - NT-SHIELD: On-device PII redaction ↔ egress privacy guard
  - NT-PHYSICAL: Pipe agents ↔ action dispatch

### 5. Reflexio
- **URL**: https://github.com/ReflexioAI/reflexio
- **Stars**: 363 (new Apr 2026)
- **What**: AI agent self-improvement harness — user corrections → persisted behavioral improvements. Playbook extraction from corrections, cross-user aggregation, rollback support. -50% planning steps, -57% tokens on GDPVal.
- **Key Patterns**: Correction-to-playbook pipeline, user-scoped learning, cross-user playbook aggregation, success evaluation, automatic rollback on regressions.
- **NeoTrix Mapping**:
  - NT-MIND: Playbook extraction ↔ SEAL distillation
  - NT-MEMORY: User-scoped memory ↔ KB namespaced storage
  - NT-REPAIR: Automatic rollback ↔ self-healing loop
  - NT-CORE: Success evaluation ↔ ConsciousnessTree feedback

### 6. Monid
- **URL**: https://monid.ai
- **Stars**: ProductHunt #1 Day (Sep 2, 2026)
- **What**: OpenRouter for agent tools — 1,800+ APIs accessible without subscriptions. Agent discovers, runs, and pays for tools at runtime. SEO, lead gen, video gen, social, stocks, on-chain data.
- **Key Patterns**: Runtime tool discovery, pay-per-use API routing, unified tool interface, zero-subscription access.
- **NeoTrix Mapping**:
  - NT-ACT: Runtime tool discovery ↔ CapabilityRegistry
  - NT-IO: Unified API routing ↔ ordered backend router
  - NT-CORE: Cost-aware routing ↔ Axiom A1 (cost-aware GWT)

### 7. Flux Attention (Paper)
- **URL**: https://arxiv.org/abs/2604.07394
- **What**: Context-aware hybrid attention — layer-level routing between Full Attention and Sparse Attention via lightweight Layer Router. 2.8x prefill speedup, 2.0x decode speedup. 12 hours training on 8xA800.
- **Key Patterns**: Layer-wise attention routing (not head-level), frozen pretrained LLM + lightweight router, contiguous memory access for hardware acceleration.
- **NeoTrix Mapping**:
  - NT-CORE: Layer-wise attention routing ↔ GWT salience modulation
  - NT-MIND: Parameter-efficient adaptation ↔ SEAL lightweight evolution
  - NT-PHYSICAL: Hardware-aware optimization ↔ physical embodiment constraints

### 8. Gated-Memory Routing (EMNLP 2026)
- **URL**: https://arxiv.org/abs/2609.00237
- **What**: Multi-agent orchestration via learned memory gates — Memory Write Gate commits only non-redundant steps, Retrieval Gate supplies compact relevant subset, Adaptive Halting Controller stops when sufficient evidence accumulated. +2.44 accuracy, -31.9% cost.
- **Key Patterns**: Gated memory write (non-redundancy filtering), compact retrieval (not full history), adaptive halting (evidence-based stopping), execution-history overload prevention.
- **NeoTrix Mapping**:
  - NT-MEMORY: Gated write/retrieval ↔ KB admission control
  - NT-CORE: Adaptive halting ↔ GWT attention budget
  - NT-MIND: Non-redundancy filtering ↔ SEAL distillation
  - NT-ACT: Multi-agent orchestration ↔ swarm coordination

### 9. DecentMem — Decentralized Memory for MAS
- **URL**: https://arxiv.org/abs/2605.22721
- **What**: Decentralized dual-pool memory — each agent has private exploitation pool (consolidated trajectories) + exploration pool (LLM-generated candidates). Online routing via stochastic bandit. O(log T) regret. +23.8% over centralized baselines, -49% tokens.
- **Key Patterns**: Decentralized per-agent memory (no shared repository), dual-pool (exploit vs explore), online routing via bandit, preserves role-specialization.
- **NeoTrix Mapping**:
  - NT-MEMORY: Decentralized memory ↔ KB namespace per domain
  - NT-CORE: Bandit-based routing ↔ GWT cost-aware attention
  - NT-MIND: Exploration pool ↔ SEAL evolution experiments
  - NT-SHIELD: Agent privacy ↔ per-agent isolation

### 10. CoSA — Proxy-Kernel Co-Designed Sparse Attention
- **URL**: https://arxiv.org/abs/2607.25291
- **What**: Two-stage training-free sparse attention — Kernel-Aware Proxy selects blocks + Ordered-Skipping Kernel skips under budget. 4.93x attention speedup, 2.53x TTFT reduction at 128K context.
- **Key Patterns**: Proxy-kernel co-design, ordered skipping with online-softmax, training-free deployment, budget-aware block selection.
- **NeoTrix Mapping**:
  - NT-CORE: Budget-aware attention ↔ cost-aware GWT (Axiom A1)
  - NT-PHYSICAL: Training-free deployment ↔ zero-overhead embodiment
  - NT-MEMORY: Block-level KV cache optimization ↔ KVMem integration

---

## Trend Synthesis

### Dominant Patterns This Cycle

| Pattern | Count | Projects |
|---------|-------|----------|
| **Decentralized/Memory-per-Agent** | 4 | Ruflo, DecentMem, Screenpipe, Gated-Memory |
| **Self-Improvement/Learning** | 3 | Reflexio, Ruflo (SONA), DeerFlow (skills) |
| **Layer-wise/Hierarchical Routing** | 3 | Flux Attention, Gated-Memory, CoSA |
| **Event-Driven Perception** | 2 | Screenpipe, DeerFlow |
| **Multi-Agent Orchestration** | 4 | DeerFlow, Ruflo, DecentMem, Gated-Memory |

### Key Insight for NeoTrix

The trend has firmly shifted from centralized shared-memory agent systems to **decentralized per-agent memory with online routing**. Three independent papers (DecentMem, Gated-Memory, SEDM) converge on the same conclusion: centralized memory erodes agent specialization and prevents self-evolution. This validates NeoTrix's 7-domain architecture (each domain with private KB namespace) and the `experience-tree` absorption protocol.

**Actionable**: The Gated-Memory Routing pattern (Write Gate + Retrieval Gate + Adaptive Halting) maps directly to NT-MEMORY admission control — consider implementing a lightweight gate for KB writes to prevent redundancy accumulation.

---

## References

1. DeerFlow 2.0 — https://github.com/bytedance/deer-flow (82.3K★)
2. Superpowers — https://github.com/obra/superpowers (285K★)
3. Ruflo — https://github.com/ruvnet/ruflo (72.1K★)
4. Screenpipe — https://github.com/screenpipe/screenpipe (YC S26)
5. Reflexio — https://github.com/ReflexioAI/reflexio (363★)
6. Monid — https://monid.ai (PH #1 Sep 2026)
7. Flux Attention — arXiv:2604.07394 (Apr 2026)
8. Gated-Memory Routing — arXiv:2609.00237 (EMNLP 2026)
9. DecentMem — arXiv:2605.22721 (May 2026)
10. CoSA — arXiv:2607.25291 (Jul 2026)
