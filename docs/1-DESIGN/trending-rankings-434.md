# Trending Rankings — Cycle 434

**Date:** 2026-09-12
**Focus:** AI agents, LLM tools, reasoning frameworks, memory, attention, routing
**Exclusion:** Projects covered in cycles 318–433

---

## 1. DeerFlow 2.0 (ByteDance)

- **GitHub:** 74.9K stars (3.3K this week), #1 GitHub Trending Feb 28 2026
- **Category:** SuperAgent Harness
- **What:** Open-source long-horizon super agent harness. Ground-up rewrite from v1. Orchestrates sub-agents, persistent memory, Docker sandboxes, and extensible skills. LangGraph-compatible API. Three sandbox modes (local/Docker/K8s). IM channels (Lark/Slack/Discord). Skills loaded on-demand via Markdown files.
- **NeoTrix mapping:** NT-ACT + NT-WORLD orchestration. Sub-agent delegation with isolated filesystems maps to NT-ACT worker pool. Skill-as-Markdown pattern validates SKILL-SPEC.md contract (Axiom A3). Sandbox isolation → NT-SHIELD execution boundary. Message gateway → EventBus two-layer architecture.
- **Pattern:** SuperAgent Harness — not a framework you wire, but a complete execution environment with sandbox, memory, run state, and message bus. "Batteries included" agent runtime.
- **Novel:** Deferred tool loading for large MCP catalogs — model calls `tool_search(query)` to retrieve tool schemas on-demand, saving context window space. Auto-promotion based on routing hints.

## 2. Omnigent

- **GitHub:** 9K stars (1.2K+ this week)
- **Category:** Meta-Harness for Multi-Agent Orchestration
- **What:** Common orchestration layer over Claude Code, Codex, Cursor, OpenCode, Hermes, Pi, and custom agents. Swap harnesses without rewriting. Cloud sandboxes (Modal/Daytona/E2B/K8s). Polly (multi-agent coding orchestrator), Debby (dual-LLM brainstorming), Deep Research (cited cross-checked reports).
- **NeoTrix mapping:** NT-ACT multi-harness orchestration. Polly's pattern: plan → delegate to coding sub-agents in parallel git worktrees → route diffs to cross-vendor reviewers. Maps to NT-ACT's ParallelTaskManager with vendor-diverse review. Debby's dual-head debate → NT-CORE E8 hexagram dual-perspective reasoning.
- **Pattern:** Harness-as-Plug-in — agents are YAML files with prompt + tools + sub-agents. No code required to define an agent. Agents can build agents.
- **Novel:** Cross-vendor review routing — reviewer agent must be from a different vendor than the writer. Prevents model-specific blind spots in code review.

## 3. Hive (OpenHive)

- **GitHub:** 11K stars
- **Category:** Multi-Agent Production Harness
- **What:** Colony-based agent system: Queen (persistent client-facing lead) + worker clones. One execution primitive: Queen is an agent loop, every worker is a clone. Shared tracker ledger + persistent plan for coordination. Crash-safe park/resume, cost enforcement, out-of-band human-in-the-loop (Sentinel via Slack/Telegram).
- **NeoTrix mapping:** NT-ACT orchestration. Queen/Worker colony → NT-ACT's BatchProductionManager with queen-style coordinator. Sentinel escalation → NT-SHIELD human-in-the-loop. Tracker ledger → EventBus event log with crash recovery. CEO-style routing → GWT salience-based task assignment.
- **Pattern:** Outcome-Driven Colony — describe the outcome, Queen pilots one unit, proves the path, then fans out workers. No graph compilation, no orchestration boilerplate.
- **Novel:** Queen grows the colony at runtime — workers spawned on demand from proven protocols. One loop controls many loops. SQL-based result validation across parallel workers.

## 4. Harden AIF

- **ProductHunt:** #2 Product of the Day (Sep 9, 2026), 405 upvotes
- **Category:** Agent Security Layer
- **What:** Free, local security tool for AI coding agents. Post-trained model checks tool calls before they run, using request and session context. Beat frontier models on agent-security benchmarks. Keeps repo and tool output on your machine — zero data egress.
- **NeoTrix mapping:** NT-SHIELD tool-call guard. Post-trained classifier as pre-execution gate maps to NT-SHIELD's RiskAssessor (R-P82). Local-only execution aligns with Egress Privacy Guard trust tiers. Session-context-aware checking → NT-SHIELD's audit trail integration.
- **Pattern:** Pre-Execution Security Classification — not prompt filtering, not output filtering, but tool-call-level interception with session context. Model specifically trained for agent security, not general-purpose LLM.
- **Novel:** Training a small model specifically for agent security decisions — faster, more accurate, and more private than using frontier models for guardrails. Post-trained, not prompted.

## 5. Cotal

- **GitHub:** 248 stars (growing)
- **Category:** Agent Pub/Sub Coordination Standard
- **What:** Open pub/sub standard for AI agents. NATS + JetStream underneath. Three addressing modes: multicast (broadcast to channel), unicast (message one peer, durable delivery), anycast (reach any one of a role). Presence (live state) for every agent. Complements MCP (agent↔tools) and A2A (agent↔agent pairwise). Reuses A2A AgentCard and Message/Part shapes.
- **NeoTrix mapping:** NT-ACT agent-to-agent coordination. Multicast → EventBus broadcast. Unicast → point-to-point agent messaging. Anycast → role-based task routing (any available NT-ACT worker). Presence → HeartbeatAggregator live state. NATS JetStream durability → EventBus persistence layer.
- **Pattern:** Agent Coordination as Pub/Sub — not orchestrator-worker, not pairwise request/response, but many agents in one shared space with presence, channels, and durable delivery. Topology is configuration, not code.
- **Novel:** Anycast addressing for agents — address a service role ("whoever is a reviewer") and exactly one available instance picks it up. Delegation without naming a worker.

## 6. Open Multi-Agent (OMA)

- **GitHub:** 6.8K stars
- **Category:** TypeScript Multi-Agent Orchestration
- **What:** TypeScript framework. Dynamic workflows: coordinator plans task DAG at runtime from a goal description. Deterministic scheduler executes across team. Append-only plan log for replay. Process and ACP backends put Claude Code, Gemini CLI, and Codex on same task DAG. 50+ runnable examples.
- **NeoTrix mapping:** NT-ACT orchestration. Dynamic DAG planning → SEAL pipeline stage decomposition. Plan log → EventBus event溯源. ACP backend integration → NT-IO protocol layer. Cross-vendor execution on same DAG → cost-aware routing (Axiom A1).
- **Pattern:** Goal-to-DAG-at-Runtime — describe the goal, coordinator builds the task graph dynamically. No hand-wired graphs. Deterministic replay from plan log.
- **Novel:** Unified runtime where Claude Code, Gemini CLI, and Codex share task DAG, memory, and budgets. Process backends for agent harness interop.

## 7. Symphony-Coord

- **GitHub:** 916 stars
- **Category:** Decentralized Multi-Agent Coordination
- **What:** Decentralized framework. Agent selection as online multi-armed bandit problem. Three-stage pipeline: Planning (decompose queries) → Execution (LinUCB-based beacon-guided routing) → Voting (CoT voting for robust answers). Edge-optimized: runs on RTX 3060/4090, Jetson, M-series Mac.
- **NeoTrix mapping:** NT-CORE + NT-ACT. LinUCB routing → GWT salience with exploration/exploitation. CoT voting → E8 hexagram multiple-path consensus. Edge-optimized → NT-PHYSICAL constrained device support. Decentralized → no single orchestrator, maps to ConsciousnessTree's distributed health monitoring.
- **Pattern:** Decentralized Coordination via Bandit Learning — roles emerge organically through interaction, not predefined. No central orchestrator. Beacon-based capability matching.
- **Novel:** Multi-armed bandit for agent role selection — learns which agent combination works for which task type, adapting online without retraining.

## 8. Mycelium

- **GitHub:** 116 stars (early, high-signal)
- **Category:** Peer Agent Coordination with Persistent Memory
- **What:** Coordination layer for autonomous agents operating as peers — no predefined workflow, no centralized supervisor, no hierarchy. Rooms with persistent memory, aligner mediator (NEGMAS negotiation protocol), SLIM encrypted group messaging. Agents join rooms and inherit accumulated intelligence. Local embedding index (~384-dim, on-device).
- **NeoTrix mapping:** NT-MEMORY + NT-NEXUS. Room memory → KB experience hub with per-namespace access control. Aligner negotiation → NT-CORE E8 consensus (multiple perspectives converging). SLIM encrypted transport → NT-SHIELD secure agent communication. Inherited intelligence → NT-NEXUS cross-session memory weaving.
- **Pattern:** Peer Coordination Without Orchestrator — alignment (agree on shared position) → work rows (per-task memory). Intelligence compounds instead of resetting. No database, no message broker — markdown files with YAML frontmatter.
- **Novel:** NEGMAS Stacked Alternating Offers negotiation for multi-agent consensus. Every agent has a voice, convergence is measurable. IOC Layer 9 epistemic envelopes for coordination messages.

## 9. GlassBrain

- **ProductHunt:** Sep 2026 launch
- **Category:** AI App Debugging / Visual Trace Replay
- **What:** Captures every step of AI app as interactive visual trace tree. Click any node, swap input, replay instantly without redeploying. Snapshot mode (deterministic replays) + Live mode (hits actual stack). Auto-generated fix suggestions reference exact trace data. Diff view shows exactly what changed. Shareable replay links.
- **NeoTrix mapping:** NT-REPAIR + NT-META debugging. Visual trace tree → ConsciousnessTree growth visualization. Node-level replay → SEAL stage-level rerun capability. Fix suggestions → NT-REPAIR automated repair workflows. Diff view → NT-META self-audit comparison.
- **Pattern:** Trace-as-Debugging-Primitive — every AI pipeline run produces an interactive trace tree, not logs. Replay with modified inputs to isolate failures. Shareable replays for team debugging.
- **Novel:** Two-line integration for OpenAI/Anthropic apps. Snapshot mode stores deterministic replays without hitting API. Free tier: 1K traces/month.

## 10. HyperProbe

- **ProductHunt:** #4 Product of the Day (Sep 5, 2026), 208 upvotes
- **Category:** Production Agent Debugging
- **What:** Drop read-only probes into running production services via Claude Code/Codex/Cursor. Captures variable state that was never recorded. Agent debugs like it has a local repro, closing bugs in one sitting. No redeploy required.
- **NeoTrix mapping:** NT-REPAIR production debugging. Read-only probes → NT-SHIELD sandbox-safe inspection. Agent-driven debugging → NT-REPAIR self-healing loop. Variable state capture → NT-MEMORY runtime state snapshot for post-mortem.
- **Pattern:** Agent-Dropped Production Probes — not adding log lines and waiting for deploy, but letting agents instrument production services on-demand with read-only probes.
- **Novel:** Agents that can debug production by modifying their own observation capability. Read-only safety guarantee means probes cannot alter service behavior.

---

## Cross-Cutting Patterns (Cycle 434)

| Pattern | Prevalence | NeoTrix Mapping |
|---------|-----------|-----------------|
| **SuperAgent Harness** (complete execution env) | DeerFlow, Hive, Omnigent | NT-ACT as production harness |
| **Pub/Sub Agent Coordination** (not orchestrator-worker) | Cotal, Mycelium, Symphony-Coord | EventBus as agent coordination layer |
| **Pre-Execution Security** (tool-call interception) | Harden AIF | NT-SHIELD RiskAssessor |
| **Agent-Dropped Probes** (on-demand instrumentation) | HyperProbe | NT-REPAIR self-healing |
| **Trace-as-Debugging** (interactive replay) | GlassBrain | NT-META self-audit visualization |
| **Outcome-Driven Orchestration** (goal → DAG at runtime) | DeerFlow, OMA, Hive | SEAL pipeline decomposition |
| **Negotiation-Based Consensus** (not voting) | Mycelium, Symphony-Coord | NT-CORE E8 multi-perspective |
| **Cross-Vendor Review** (prevent blind spots) | Omnigent | NT-SHIELD audit diversity |
| **Edge-Optimized Agents** (consumer GPU) | Symphony-Coord | NT-PHYSICAL constrained devices |
| **Deferred Tool Loading** (on-demand schema) | DeerFlow 2.0 | NT-IO context-efficient tooling |

## Priority Absorption Targets

1. **DeerFlow 2.0 Deferred Tool Loading** — Context-efficient tool discovery for NT-IO
2. **Cotal Pub/Sub Pattern** — Agent coordination standard for NT-ACT EventBus
3. **Harden AIF Pre-Execution Gate** — Tool-call security for NT-SHIELD
4. **Hive Colony Pattern** — Queen/Worker production orchestration for NT-ACT
5. **Mycelium Room Memory** — Peer-coordinated persistent memory for NT-MEMORY

---

*Generated by NeoTrix iteration loop, cycle 434*
