# Trending Rankings — Cycle 348

**Date**: 2026-09-11
**Sources**: GitHub Trending, ProductHunt (Sep 6-11 2026), arXiv, awesome-ai-agents-2026, DataAIHub

## 10 New Projects (Not in Cycles 318-347)

### 1. OmniAgent (YeQing17-2026/OmniAgent)
- **URL**: https://github.com/YeQing17-2026/OmniAgent
- **Stars**: 2.6K+ | **Language**: Python | **License**: Other
- **Category**: Self-Evolving Agent Framework
- **Pattern**: Full-dimensional self-evolution (OmniEvolve): Skill evolution (real-time during execution, not post-hoc), Context evolution (multi-layer information stack with dual-path alignment), BrainModel evolution (online RL with GRPO+PRM). Hyper-Harness with dynamic multi-agent (Sentinel planner + Guardian safety), progressive context loading, four-layer dynamic security scanning (unbypassable). Deep Reflexion dual-layer reflective architecture for failure-to-insight conversion. Proactive Memory with explicit feedback + implicit LLM induction.
- **NeoTrix Mapping**: NT-MIND + NT-CORE — real-time skill self-evolution = SEAL pipeline live crystallization (not batch). Online RL brain model evolution = SelfModel value function update. Dual-path memory = experience-tree with explicit + implicit signals. Progressive context loading = GWT attention tiering. **Absorption candidate**: full-dimensional self-evolution agent — skill/context/brainmodel evolve simultaneously during execution, not in separate phases. Validates NeoTrix's SEAL pipeline but suggests real-time crystallization may be superior to batch distillation.

### 2. GitAgent (open-gitagent/gitagent)
- **URL**: https://github.com/open-gitagent/gitagent
- **Stars**: 670+ | **Language**: TypeScript | **License**: MIT
- **Category**: Git-Native Agent Framework
- **Pattern**: Agent IS a git repository — identity, rules, memory, tools, skills are all version-controlled files (agent.yaml, SOUL.md, RULES.md, memory/, tools/, skills/, hooks/). SDK mirrors Claude Agent SDK but runs in-process (no subprocesses, no IPC). MCP client auto-discovers tools. Git-committed memory with full history. HTTP-level OpenTelemetry instrumentation (LLM calls, tool executions). Skills as composable modules. Multi-provider support.
- **NeoTrix Mapping**: NT-MEMORY + NT-IO — git-native memory = KB version control with git-like history tracking. SOUL.md/RULES.md = identity-constrained behavior specification (parallel to NeoTrix CONTEXT.md + AGENTS.md). In-process SDK = zero-IPC agent runtime. **Absorption candidate**: agent-as-repo paradigm — identity, memory, tools, skills all version-controlled in a single git repo with full audit history. Validates NeoTrix's file-based configuration but proposes git as the universal persistence layer.

### 3. Baton (ProductHunt)
- **URL**: ProductHunt (launched ~Apr 2026)
- **Stars**: PH Featured | **Category**: Multi-Agent Orchestration Desktop App
- **Pattern**: Desktop app for parallel AI coding agents — each agent in its own git-isolated workspace. Smart notification badges for attention routing. Review diffs, browse files, search codebase. Agents can spawn new agents through built-in MCP server. Works with Claude Code, Codex, OpenCode, any terminal-based agent. Supervisor agent concept — condense information and direct flow instead of managing each agent individually.
- **NeoTrix Mapping**: NT-ACT + NT-IO — parallel agent workspace = NT-ACT multi-agent orchestration with worktree isolation (P2 pattern). Notification badges = GWT attention salience for agent management. Supervisor agent concept = meta-level agent coordination (NT-META). **Absorption candidate**: spatial agent orchestration — git-isolated parallel agent execution with visual attention routing and hierarchical supervision. Maps to NT-ACT orchestration with GWT-modulated attention across agent fleet.

### 4. Paritok (paritok.com)
- **URL**: https://github.com/paritok/paritok
- **Stars**: New (Jul 2026) | **Language**: Python | **License**: Open Source
- **Category**: Context Compression for Coding Agents
- **Pattern**: Compresses tool schemas (29K→8K per turn), file reads/tool output (quarter size), and stale history (summarization when context fills). Works with Claude Code, Codex, Cursor, OpenAI/Anthropic compatible. Two commands, fully local. 85% token reduction in context-saturated sessions. 4B model compression + tool schema filter. Split: tool filter dominates early turns, compression dominates late turns. 62K compressions across users cut 447M input tokens. ~13s local on RTX 4060, ~3s on hosted GPU.
- **NeoTrix Mapping**: NT-CORE + NT-IO — context compression = GWT attention optimization via input reduction. Tool schema filtering = NT-IO structured output minimization. Stale history summarization = experience-tree compaction for long sessions. **Absorption candidate**: inference-time context compression — tool schemas + file output + stale history compressed via small model before reaching context window. Validates NeoTrix's token-efficient reasoning (Axiom A2: Context as Scarce Resource) with concrete 85% reduction numbers.

### 5. Angy (ProductHunt)
- **URL**: https://www.producthunt.com/products/angy
- **Stars**: PH Featured (2026) | **Language**: Python | **License**: Open Source
- **Category**: Adversarial Multi-Agent Pipeline
- **Pattern**: Deterministic multi-phase pipeline: Plan → Build → Test with adversarial Counterpart agent that strictly verifies all code. Git worktree isolation for parallel agents without branch conflicts. AI-driven scheduling runs epics overnight. Counterpart agent is persistent session across epic lifecycle. Vague spec mode: Counterpart iteratively improves plans with adversarial system prompt. Born because Claude Code is cheaper than Cursor but terminal UX is poor.
- **NeoTrix Mapping**: NT-ACT + NT-SHIELD + NT-REPAIR — adversarial Counterpart = NT-REPAIR adversarial self-audit (D1-D51 review dimensions). Deterministic pipeline = SEAL phase ordering. Worktree isolation = P2 pattern. Counterpart persistence = experience-tree cross-session continuity. **Absorption candidate**: adversarial verification pipeline — dedicated Counterpart agent blocks merges until satisfied, converting code review from human bottleneck to agent-native gate. Maps to NT-REPAIR self-healing with adversarial stress-testing.

### 6. Glassbrain (ProductHunt)
- **URL**: https://www.producthunt.com/products/glassbrain
- **Stars**: PH Featured (2026) | **Category**: Visual Trace Replay for AI Apps
- **Pattern**: Captures every step of AI app as interactive visual trace tree. Click any node, swap input, replay instantly without redeploying. Snapshot mode (deterministic replays) + Live mode (hits actual stack). Auto-generated fix suggestions reference exact trace data with one-click copy. Diff view shows changes. Shareable replay links. Works with OpenAI, Anthropic, LangChain, LlamaIndex. Two lines of code to integrate.
- **NeoTrix Mapping**: NT-REPAIR + NT-IO — trace replay = NT-REPAIR execution trace recording for self-diagnosis. Visual trace tree = ConsciousnessTree visualization of decision paths. Replay with input swap = counterfactual testing for self-healing validation. **Absorption candidate**: deterministic trace replay — capture agent execution as replayable traces with input-swapping for debugging, enabling post-hoc analysis and fix suggestion generation. Maps to NT-REPAIR's diagnostic phase with visual trace inspection.

### 7. oqoqo (oqoqo.ai)
- **URL**: https://oqoqo.ai
- **Stars**: PH Featured (Jul 2026) | **Category**: Agent Evals & Benchmarks
- **Pattern**: Build evals and custom benchmarks for real-world tasks. Run eval experiments at scale in realistic environments with isolated sandboxes. Catalog every step agents take (tool calls, retries, discovery loops), token consumption, cost. Measure agent-friendliness of product surfaces against Codex/Claude Code/OpenClaw/Hermes/Pi/OpenCode/Cursor/Copilot. Agent can handle setup for you. Regression test MCP/CLI/skills/SDK.
- **NeoTrix Mapping**: NT-MIND + NT-REPAIR — agent evals = SelfTest T3 production wiring with empirical measurement. Custom benchmarks = SEAL pipeline fitness function calibration. Sandboxed evaluation = NT-SHIELD isolation for safe testing. **Absorption candidate**: agent-native product evaluation — measure how well agents interact with your product surfaces, with automated benchmark generation and regression testing. Maps to SelfTest T3 with external validation metrics.

### 8. Timbal AI (timbal.ai)
- **URL**: https://www.producthunt.com/products/timbal-ai
- **Stars**: PH Featured (Jun 2026) | **Category**: Agent Production Platform
- **Pattern**: One platform for building, deploying, monitoring, evaluating, and governing AI agents. ACE (Action Control Engine) — behavioral runtime as proxy providing deterministic layer. Agents/workflows/tools compile to clean readable Python (no black-box). Natural language creation via Composer. ISO 27001/SOC 2 Type II/NIS2 compliance. Proprietary infrastructure on AWS with one-click deployment. Eval/tracing/ACE built into runtime, not separate tools.
- **NeoTrix Mapping**: NT-MIND + NT-SHIELD — ACE behavioral runtime = NT-SHIELD policy enforcement as runtime proxy (not prompt-level). Compiled-to-code = SEAL pipeline intermediate representation. Compliance built-in = NT-SHIELD governance layer. **Absorption candidate**: governance-as-runtime — deterministic behavioral layer intercepts every agent action for consistency enforcement, with full compliance (ISO/SOC2/NIS2) baked into the infrastructure. Maps to NT-SHIELD's trust-tier enforcement.

### 9. Nuphos (nuphos.ai)
- **URL**: https://www.producthunt.com/products/nuphos
- **Stars**: PH Featured (Aug 2026) | **Category**: AI-Native DevOps Workspace
- **Pattern**: Shared environment where AI agents learn infrastructure, investigate issues, operate production. Connects AWS, GCP, Kubernetes, observability stack. Agents read-only by default, approval required for write actions. Shared context and audit trail with team. Reduces operational learning curve when expanding to unfamiliar platforms. Team-centric: agents understand context from existing infrastructure.
- **NeoTrix Mapping**: NT-WORLD + NT-SHIELD — infrastructure perception = NT-WORLD sensory integration for real-world systems. Read-only default = NT-SHIELD least-privilege principle. Shared audit trail = experience-tree cross-agent context sharing. **Absorption candidate**: agent-native infrastructure operations — agents explore production environments with read-only observation, propose actions requiring approval, maintain shared audit context. Maps to NT-WORLD perception with NT-SHIELD safety boundaries.

### 10. PrimeAgent (PrimeIntellect-ai/prime-agent)
- **URL**: https://github.com/PrimeIntellect-ai/prime-agent
- **Stars**: 1.5K+ | **Language**: Python | **License**: MIT
- **Category**: Self-Improving Coding Agent
- **Pattern**: Recursive Language Model (RLM) treats context as variables and tools as recursive subagent calls inside persistent REPL. Continual Harness stores supplemental prompts, memories, skill descriptions as durable state refined through evidence-backed updates. `/refine` reviews trajectory and applies small, reviewable updates to harness state. Never rewrites immutable base system prompt. Skills are executable Python packages. Daemon-backed sessions persist across disconnects. Agent-to-agent direct communication. Heartbeats and schedules.
- **NeoTrix Mapping**: NT-MIND + NT-ACT — Continual Harness = experience-tree with refinement loop (evidence-backed updates). RLM persistent REPL = NT-ACT execution environment with session continuity. `/refine` = SEAL distillation phase with rollback support. Agent-to-agent communication = EventBus direct messaging. **Absorption candidate**: harness-as-durable-state — supplemental prompts, memories, and skills refined through evidence-backed updates to a Continual Harness that persists across sessions, with immutable base + mutable supplements separation. Maps to experience-tree's pointer-based KB with live refinement.

## Cross-Cutting Themes (Cycle 348)

| Theme | Projects | NeoTrix Impact |
|-------|----------|----------------|
| **Real-Time Self-Evolution** | OmniAgent, PrimeAgent | Evolution during execution, not in separate batch phases — skill/context/model evolve simultaneously |
| **Agent-as-Infrastructure** | GitAgent, Baton, Nuphos | Agents treated as first-class infrastructure with version control, isolation, and shared audit |
| **Adversarial Verification** | Angy, oqoqo | Dedicated verification agents and eval-native benchmarks as first-class pipeline components |
| **Context Compression** | Paritok, Glassbrain | 85% token reduction via inference-time compression; trace replay for post-hoc debugging |
| **Governance-as-Runtime** | Timbal AI, Nuphos | Deterministic behavioral layers and compliance baked into infrastructure, not bolted on |
| **Deterministic Replay** | Glassbrain, oqoqo | Reproducible agent execution traces with input-swapping for counterfactual testing |
