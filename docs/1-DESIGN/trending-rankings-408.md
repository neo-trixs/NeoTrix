# Trending Rankings — Cycle 408 (2026-09-12)

## 10 New Projects (Not in Cycles 318-407)

### 1. Omnigent
- **GitHub**: https://github.com/omnigent-ai/omnigent
- **Stars**: 9,009 (Jun 2026) | **Category**: Multi-Agent Meta-Harness
- **What**: Open-source meta-harness orchestrating Claude Code, Codex, Cursor, OpenCode, Hermes, Pi, and custom agents in a single session. Cloud sandboxes (Modal, Daytona, E2B, K8s). Policy engine for spend caps, tool restrictions, approval gates. Polly agent plans + delegates to coding sub-agents in parallel git worktrees, routes diffs to cross-vendor reviewers. Debby agent does dual-head brainstorming (Claude + GPT) with debate rounds.
- **Key Pattern**: Agent-as-service with governance — harness orchestration layer sits above individual agents, enforcing policies and routing across heterogeneous backends. Cross-vendor review (Claude reviews Codex output) prevents vendor-specific blind spots.
- **NeoTrix Relevance**: NT-ACT (multi-agent orchestration), NT-GOVERNANCE (policy engine, spend caps), NT-SHIELD (sandbox isolation). Validates NeoTrix's ordered backend router pattern — heterogeneous agent routing with governance constraints.

### 2. Vercel Eve
- **GitHub**: https://github.com/vercel/eve
- **Stars**: 4,957 (Jun 2026) | **Category**: Filesystem-First Agent Framework
- **What**: Durable AI agent framework where agent capabilities live in conventional filesystem locations: `agent/instructions.md` (always-on prompt), `agent/tools/` (typed functions), `agent/skills/` (on-demand procedures), `agent/channels/` (message channels), `agent/schedules/` (cron jobs). Agents are inspectable directories, not opaque code. Human-in-the-loop prompts, subagents, and schedules as compositional primitives.
- **Key Pattern**: Agent-as-filesystem — every agent component is a file in a conventional directory. Inspection, extension, and operation follow filesystem conventions (grep, diff, version control). Skills are markdown files loaded on demand, not compiled code.
- **NeoTrix Relevance**: NT-ACT (agent definition as files), NT-MEMORY (skills as composable markdown), NT-IO (channels as filesystem entries). Validates NeoTrix's skill-as-template pattern (Axiom P5) — skills are structured files, not monolithic prompts.

### 3. Microsoft Webwright
- **GitHub**: https://github.com/microsoft/Webwright
- **Stars**: 5,961 (Apr 2026) | **Category**: Code-as-Action Browser Agent
- **What**: Browser agent framework achieving SOTA on long-horizon web tasks (86.7% on Mind2Web, 60.1% on Odysseys). Code-as-action paradigm: agent writes a single Python script that completes the task end-to-end. No multi-agent system, no graph engine, no plugin layer. Skill Factory distills solved tasks into reusable parameterized scripts (~40s, zero tokens). Plugin manifests for Claude Code, Codex, OpenClaw, Hermes.
- **Key Pattern**: Code-as-action over coordinate prediction — browser agents that write executable scripts instead of predicting xy-coordinates. Solved tasks become reusable code skills that run without a model. Progressive reuse: 55% → 70% accuracy via skill recycling.
- **NeoTrix Relevance**: NT-WORLD (browser automation), NT-ACT (code-as-action paradigm), NT-MIND (skill distillation from solved tasks). Validates NeoTrix's skill crystallization — solved problems become executable templates, not just memory entries.

### 4. Nanobot
- **GitHub**: https://github.com/HKUDS/nanobot
- **Stars**: 47,663 (Feb 2026) | **Category**: Ultra-Lightweight Personal Agent
- **What**: Open-source personal AI agent framework (Python). WebUI, terminal, or chat apps (Telegram, Discord, Slack, WeChat, Email, Mattermost). Tools: files, shell, web search, MCP, cron, image generation, subagents. Long-term memory via "Dream" system. OpenAI-compatible API. Model switching per session, parallel search, live config reload. Daemon-backed agents survive terminal disconnect.
- **Key Pattern**: Agent-as-daemon with chat-native reach — persistent background agent that connects to any chat platform. Memory survives across sessions via Dream subsystem. Small readable core with MCP, memory, deployment, and automation built in.
- **NeoTrix Relevance**: NT-IO (multi-channel agent), NT-MEMORY (persistent Dream memory), NT-ACT (daemon-backed continuity). Validates NeoTrix's persistent agent pattern — agents need session persistence and multi-channel reach, not just API endpoints.

### 5. OmniAgent
- **GitHub**: https://github.com/YeQing17-2026/OmniAgent
- **Stars**: 2,557 (Apr 2026) | **Category**: Self-Evolving Agent with Dynamic Security
- **What**: Full-dimensional self-evolution framework: Skill Self-Evolution (auto-create/inspect/repair skills during interaction), Context Self-Evolution (real-time user feedback + LLM summarization), BrainModel Self-Evolution (online RL feedback loop). Hyper-Harness: dynamic multi-agent (Sentinel + Guardian), progressive context loading (L0/L1/L2), four-layer dynamic security scanning (LLM review → Policy engine → Interactive approval → Execution sandbox). Deep Reflexion: inner-outer dual-layer reflective architecture.
- **Key Pattern**: Triple self-evolution with progressive disclosure — skills, context, and model all evolve during interaction. Progressive context loading prevents token overflow. Security scanning is unbypassable (trust-level classified). Failure prevention via trajectory repetition detection, error action repetition detection, loop pseudo-termination detection.
- **NeoTrix Relevance**: NT-MIND (triple self-evolution), NT-SHIELD (four-layer security scanning), NT-REPAIR (Deep Reflexion failure prevention). Validates NeoTrix's SEAL pipeline — self-evolution must be multi-dimensional (skills + context + model), not just prompt refinement.

### 6. PrimeAgent
- **GitHub**: https://github.com/PrimeIntellect-ai/prime-agent
- **Stars**: 1,456 (May 2026) | **Category**: Self-Improving Coding Agent
- **What**: Recursive Language Model (RLM) treats context as variables and tools as recursive subagents inside a persistent REPL. Continual Harness stores supplemental prompts, memories, skill descriptions, and subagent specs as durable state refined through evidence-backed updates. `/refine` reviews trajectory and applies small updates. Daemon-backed background sessions with reattach. Agent-to-agent direct communication without user routing. Persistent goals, heartbeats, schedules.
- **Key Pattern**: Prompt-as-a-variable with durable harness state — the harness can improve itself through small, evidence-backed updates without rewriting the immutable base system prompt. Subagents are first-class citizens spawned via `rlm(...)`. Sessions survive terminal disconnect.
- **NeoTrix Relevance**: NT-MIND (harness self-improvement with rollback), NT-ACT (programmatic subagent spawning), NT-MEMORY (durable harness state). Validates NeoTrix's SEAL pipeline — self-improvement via evidence-backed incremental updates, not wholesale rewriting.

### 7. GitAgent
- **GitHub**: https://github.com/open-gitagent/gitagent
- **Stars**: 670 (Mar 2026) | **Category**: Git-Native Agent Framework
- **What**: Agent IS a git repository — identity, rules, memory, tools, and skills are all version-controlled files (`agent.yaml`, `SOUL.md`, `RULES.md`, `memory/`, `tools/`, `skills/`, `hooks/`). SDK provides in-process programmatic interface (no subprocesses, no IPC). MCP client auto-discovers any MCP server's tools. Lifecycle hooks for pre/post actions. Memory is git-committed with full history.
- **Key Pattern**: Agent-as-repository — every agent state is a version-controlled file. Full history of memory, tools, skills, and rules via git. Hooks enable lifecycle interventions. MCP client for universal tool discovery.
- **NeoTrix Relevance**: NT-MEMORY (git-versioned memory with history), NT-ACT (agent-as-repository), NT-SHIELD (lifecycle hooks for governance). Validates NeoTrix's persistence model — agent state should be inspectable, diffable, and rollback-capable via version control.

### 8. Mastra
- **Website**: https://mastra.ai | **ProductHunt**: #1 (Sep 9, 2026)
- **Category**: TypeScript AI Agent Framework
- **What**: From the Gatsby team. Framework for building AI-powered apps and agents with workflows, memory, streaming, evals, tracing, and Studio (interactive UI for dev and testing). `npm create mastra@latest` scaffolding. TypeScript-native with modern stack patterns.
- **Key Pattern**: Full-lifecycle agent development — not just runtime, but dev-time tooling (Studio UI for testing), observability (tracing, evals), and deployment (streaming). Framework-level quality controls built in.
- **NeoTrix Relevance**: NT-IO (agent dev-time tooling), NT-GOVERNANCE (evals, tracing), NT-MIND (quality controls during development). Validates NeoTrix's observability-first pattern — agents need dev-time introspection, not just runtime monitoring.

### 9. Harden AIF
- **Website**: https://harden.run | **ProductHunt**: #2 (Sep 9, 2026)
- **Category**: Agent Security Layer
- **What**: Free, local security tool for AI coding agents. Post-trained model checks tool calls before they run, using request and session context. Beat frontier models on agent-security benchmarks. Keeps repo and tool output on your machine (no external calls). Intercepts tool calls at the harness level — not post-hoc scanning but pre-execution validation.
- **Key Pattern**: Pre-execution tool call validation — security model trained specifically for agent tool call patterns, runs locally, validates before execution not after. Trust boundary at the tool call level, not the network level.
- **NeoTrix Relevance**: NT-SHIELD (pre-execution validation), NT-GOVERNANCE (tool call security), NT-ACT (safe tool invocation). Validates NeoTrix's risk assessment pattern (R-P82) — tool calls need pre-execution validation, not just post-hoc scanning.

### 10. HyperProbe
- **Website**: https://hyperprobe.dev | **ProductHunt**: #4 (Sep 5, 2026)
- **Category**: Production Debugging for AI Agents
- **What**: Backend teams debug production issues without redeploying. Claude Code, Codex, or Cursor drop read-only probes into running services and capture variable state that was never recorded. Agent debugs like it has a local repro, closing bugs in one sitting. No new log lines, no redeploy cycles — runtime introspection.
- **Key Pattern**: Read-only runtime probes — agents inject lightweight observers into production services to capture state at execution time. Zero deployment overhead. Agent gets local-repro-quality debugging from live production data.
- **NeoTrix Relevance**: NT-REPAIR (production debugging without redeploy), NT-PHYSICAL (runtime introspection), NT-SHIELD (read-only safety). Validates NeoTrix's self-healing pattern — repair agents need runtime observability, not just log analysis.

## Trend Analysis

| Signal | Count | Implication |
|--------|-------|-------------|
| Agent-as-file/repository | 3/10 | Eve + GitAgent + Nanobot = agent components as inspectable, versionable files |
| Self-evolution frameworks | 2/10 | OmniAgent + PrimeAgent = multi-dimensional self-improvement with safety constraints |
| Pre-execution security | 2/10 | Harden + OmniAgent = tool call validation before execution, not after |
| Code-as-action | 1/10 | Webwright = executable scripts replace coordinate prediction for browser agents |
| Production debugging | 1/10 | HyperProbe = runtime introspection for agent debugging without redeploy |
| TypeScript agent frameworks | 1/10 | Mastra = modern stack agent development with dev-time tooling |

## Meta-Signals

1. **Agent-as-filesystem is consolidating**: Eve (Vercel), GitAgent, and Nanobot all treat agent components as conventional files. This enables git-versioning, diffing, and rollback — solving the "agent state is opaque" problem.

2. **Self-evolution must be bounded and verifiable**: OmniAgent (triple evolution + four-layer security) and PrimeAgent (evidence-backed incremental updates with rollback) both constrain self-modification. Unbounded self-evolution is now recognized as a security risk.

3. **Code-as-action > coordinate prediction**: Webwright demonstrates that browser agents writing executable scripts dramatically outperform those predicting xy-coordinates. This pattern extends beyond browsers — agents should produce code, not just actions.

4. **Pre-execution > post-hoc security**: Harden AIF validates tool calls before execution using a post-trained security model. This is the agent security equivalent of "shift left" in software security — validate early, not after damage.
