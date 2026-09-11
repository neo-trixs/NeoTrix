# Trending Rankings — Cycle 346

> **Date**: 2026-09-11
> **Sources**: GitHub Trending, ProductHunt, OSSInsight, academic preprints
> **Focus**: AI agents, LLM tools, reasoning frameworks, novel patterns

---

## Top 10 New Projects (Not in Cycles 318–345)

### 1. HKUDS/nanobot — 47.6K ⭐
**Category**: Agent Framework (Self-Hosted)
**Link**: https://github.com/HKUDS/nanobot

Ultra-lightweight, self-hosted personal AI agent framework. Combines tools, long-term memory (Dream), MCP integrations, model routing, multi-agent delegation, scheduled automation, and OpenAI-compatible API in a readable core. Runs in WebUI, terminal, or chat apps (Telegram, Discord, Slack, WeChat).

**Key Patterns**:
- **Model routing + fallbacks**: OpenAI-compatible with local LLM support
- **Dream memory**: Persistent long-term memory across sessions
- **Chat-native reach**: Multi-channel agent deployment (Telegram, Discord, Slack, WeChat, Email)
- **Scheduled automation**: Cron-like goal scheduling for long-horizon tasks

**NeoTrix Mapping**:
- NT-IO: Model routing and fallback chain → `Ordered Backend Router`
- NT-MEMORY: Dream-like persistent memory → experience-tree integration
- NT-ACT: Multi-agent delegation → ConsciousnessTree coordination
- NT-SHIELD: Self-hosted privacy-first deployment

---

### 2. omnigent-ai/omnigent — 9K ⭐
**Category**: Meta-Harness / Multi-Agent Orchestration
**Link**: https://github.com/omnigent-ai/omnigent

Open-source meta-harness orchestrating Claude Code, Codex, Cursor, OpenCode, Hermes, Pi, and custom agents. Features "Polly" (multi-agent coding orchestrator) and "Debby" (dual-head brainstorming with Claude + GPT debate). Runs agents in cloud sandboxes (Modal, Daytona, E2B, CoreWeave).

**Key Patterns**:
- **Harness abstraction**: Swap/combine coding harnesses without rewriting
- **Cross-vendor review**: Agent A writes code, Agent B (different vendor) reviews
- **Dual-head debate**: `/debate` mode forces multi-perspective convergence
- **Cloud sandboxes**: Disposable execution environments per session

**NeoTrix Mapping**:
- NT-CORE: GWT attention routing for multi-agent selection
- NT-ACT: Polly orchestration → `ProductionOrchestrator`
- NT-SHIELD: Sandboxed execution → sandbox isolation
- NT-REPAIR: Cross-vendor review as self-healing pattern

---

### 3. lsdefine/GenericAgent — 14.1K ⭐
**Category**: Self-Evolving Autonomous Agent
**Link**: https://github.com/lsdefine/GenericAgent

Minimal (~3K LOC core) self-evolving autonomous agent. 9 atomic tools + ~100-line Agent Loop. Auto-crystallizes tasks into Skills, forming a personal skill tree. Supports Claude/Gemini/Kimi/MiniMax. <30K context window (vs 200K-1M for competitors).

**Key Patterns**:
- **Contextual Information Density Maximization**: <30K context window, <1/10th of competitors
- **Self-evolving skill tree**: Each task crystallizes into a reusable Skill
- **Morphling mode**: Project-level skill absorption — extract goals/tests from external repos
- **Goal Hive**: Multi-worker cooperative Goal mode with BBS coordination
- **Conductor orchestration**: Spawn, supervise, auto-clean parallel sub-agents

**NeoTrix Mapping**:
- NT-MIND: Skill crystallization → `SkillTree` + SEAL pipeline
- NT-CORE: Token-efficient reasoning → attention routing optimization
- NT-ACT: Conductor sub-agent orchestration
- NT-MEMORY: L4 session archive memory

---

### 4. Qwen-AgentWorld — 984 ⭐
**Category**: Language World Model
**Link**: https://github.com/qwenlm/qwen-agentworld

Native language world model (MoE, 35B total / 3B active, 256K context) simulating agentic environments via chain-of-thought across 7 domains: MCP, Search, Terminal, SWE, Android, Web, OS. Trained through 3-stage pipeline: CPT → SFT → RL on 10M+ real interaction trajectories.

**Key Patterns**:
- **Native world model**: Environment modeling as training objective from CPT stage
- **7 unified domains**: First LWM covering all major agent interaction domains
- **Controllable simulation**: Inject perturbations to expose agent weaknesses
- **Fictional-world construction**: Train in invented worlds, generalize to real tasks
- **Agent foundation model**: LWM RL warm-up transfers to multi-turn agentic tasks

**NeoTrix Mapping**:
- NT-WORLD: World simulation for agent training → `UnifiedCrawler` pattern
- NT-CORE: E8 reasoning engine for multi-domain coordination
- NT-MIND: Fictional-world training → SEAL exploration pipeline
- NT-SHIELD: Perturbation injection for adversarial robustness testing

---

### 5. Tencent/UI-Mate — 64 ⭐
**Category**: Foundation GUI Agent
**Link**: https://github.com/Tencent/UI-Mate

Foundation GUI agent for long-horizon work across applications and OS. Observes live screen, reasons over visible state, acts through keyboard/mouse on native desktop. Key innovation: in-context demonstrations distill reusable workflows. OSWorkerBench: office-centric benchmark for realistic workflows.

**Key Patterns**:
- **Demonstration-guided execution**: One human demo → reusable workflow, +18pp strict success
- **OSWorkerBench**: Office-centric benchmark (100 tasks, long-horizon)
- **Native desktop control**: Real keyboard/mouse, not screenshot-based
- **Open-weight models**: 9B and 27B checkpoints, model-agnostic desktop app

**NeoTrix Mapping**:
- NT-WORLD: Live screen perception → `PerceptionBridge` integration
- NT-ACT: Keyboard/mouse control → action execution primitives
- NT-PHYSICAL: Physical device interaction → sensors/motors layer
- NT-MIND: Workflow distillation from demonstrations

---

### 6. open-gitagent/gitagent — 670 ⭐
**Category**: Git-Native Agent Framework
**Link**: https://github.com/open-gitagent/gitagent

Universal git-native multimodal agent. Agent IS a git repository — identity (SOUL.md), rules (RULES.md), memory (memory/), tools (tools/), skills (skills/) are all version-controlled files. MCP client for automatic tool discovery. "Agents as repos" paradigm.

**Key Patterns**:
- **Agent-as-Repo**: Identity, rules, memory, tools all version-controlled in git
- **SOUL.md / RULES.md**: Declarative agent personality and behavioral constraints
- **MCP client**: Automatic discovery of MCP server tools
- **Git-committed memory**: Full history of agent memory evolution

**NeoTrix Mapping**:
- NT-MEMORY: Version-controlled KB → git-backed knowledge persistence
- NT-CORE: SOUL.md = personality/identity → SelfModel extension
- NT-GOVERNANCE: RULES.md = behavioral constraints → policy enforcement
- NT-MIND: Git history of skill evolution → SEAL crystallization audit

---

### 7. PrimeIntellect-ai/prime-agent — 1.5K ⭐
**Category**: Self-Improving RLM Agent
**Link**: https://github.com/PrimeIntellect-ai/prime-agent

Self-improving Recursive Language Model (RLM) agent. Context as variables (prompt-as-a-variable), tools as recursive subagents. Continual Harness stores supplemental prompts, memories, skill descriptions. `/refine` applies small, evidence-backed updates to harness state with rollback support.

**Key Patterns**:
- **Prompt-as-a-variable**: Context treated as mutable variables in persistent REPL
- **Continual Harness**: Durable state that agent can refine through small updates
- **Evidence-backed refinement**: `/refine` records snapshots, supports rollback
- **Direct agent-to-agent communication**: Running agents exchange messages without user routing
- **Daemon-backed continuity**: Sessions persist across terminal disconnects

**NeoTrix Mapping**:
- NT-MIND: Continual Harness refinement → `SEAL` self-evolution loop
- NT-MEMORY: Durable harness state → experience-tree with rollback
- NT-ACT: Agent-to-agent communication → EventBus direct messaging
- NT-REPAIR: Rollback support for refinement → self-healing patterns

---

### 8. Mastra Factory — #1 ProductHunt (Sep 9, 2026)
**Category**: AI Agent Framework (Production)
**Link**: https://mastra.ai

From the Gatsby team. Framework for building AI-powered apps and agents with workflows, memory, streaming, evals, tracing, and Studio (interactive UI for dev and testing). "From issue to production, run by agents."

**Key Patterns**:
- **Studio**: Interactive UI for agent development and testing
- **Workflow-first**: Structured workflow execution, not ad-hoc agent loops
- **Built-in evals + tracing**: Production observability from day one
- **Streaming-native**: Real-time agent output streaming

**NeoTrix Mapping**:
- NT-IO: Studio UI → `nt_io` interface patterns
- NT-ACT: Workflow execution → `ProductionOrchestrator`
- NT-MEMORY: Built-in memory layer
- NT-GOVERNANCE: Evals + tracing → audit/compliance instrumentation

---

### 9. Harden AIF — #2 ProductHunt (Sep 9, 2026)
**Category**: AI Agent Security
**Link**: https://harden.run

Post-trained security model that checks tool calls before they run. Uses request + session context to evaluate tool safety. Beat frontier models on agent-security benchmarks. Free, local, keeps repo and tool output on your machine.

**Key Patterns**:
- **Pre-execution tool call validation**: Security model intercepts before tool execution
- **Session-context-aware**: Uses full conversation context, not just the tool call
- **Post-trained model**: Dedicated small model for security decisions
- **Local-first**: All processing on-device, no data leaves machine

**NeoTrix Mapping**:
- NT-SHIELD: Pre-execution guard → `nt_shield_sandbox` egress policy
- NT-CORE: Context-aware security decisions → GWT attention for threat detection
- NT-MEMORY: Session context for security → `PerceptionBridge` pattern
- NT-REPAIR: Security as self-healing → protection layer integration

---

### 10. Switch (FlintAI) — #1 ProductHunt (Sep 8, 2026)
**Category**: Multi-Agent Collaboration Platform
**Link**: https://www.flintai.dev/products/switch

Brings AI agents into Slack, Teams, Discord, Telegram as named participants. Agents share context and history with the team. Each room carries its own context, participants, and rules. Works with Claude Code, OpenAI, Google ADK, LangChain. Open source, self-hostable.

**Key Patterns**:
- **Room-scoped context**: Each channel has isolated context, participants, rules
- **Named agent participants**: Agents join as team members, not external tools
- **Cross-platform**: Single agent deployment across Slack/Teams/Discord/Telegram
- **Rule-based governance**: Per-room behavioral rules

**NeoTrix Mapping**:
- NT-IO: Multi-platform integration → `PlatformGateway` pattern
- NT-GOVERNANCE: Room-scoped rules → policy enforcement per context
- NT-MEMORY: Room-isolated memory → namespace isolation in KB
- NT-ACT: Agent as team member → multi-agent collaboration model

---

## Cross-Cutting Patterns

| Pattern | Projects | NeoTrix Integration |
|---------|----------|-------------------|
| **Agent-as-Repo** | gitagent, GenericAgent | Version-controlled KB + SOUL.md identity |
| **Continual Self-Evolution** | GenericAgent, Prime Agent, nanobot | SEAL pipeline + experience-tree |
| **Token Efficiency** | GenericAgent (<30K), ALAR (43-84% reduction) | GWT salience + cost-aware routing |
| **Multi-Agent Orchestration** | omnigent, Polly, Switch, Mastra | ConsciousnessTree + EventBus |
| **Pre-execution Security** | Harden AIF, NeuralFSM protection | NT-SHIELD sandbox + egress guard |
| **World Models** | Qwen-AgentWorld | NT-WORLD simulation + training |
| **In-Context Demonstrations** | UI-Mate | Workflow distillation → skill crystallization |
| **Git-Native Identity** | gitagent, Prime Agent | SelfModel + behavioral constraints |

---

## Trend Summary

1. **Self-evolving skill trees** are becoming the default: GenericAgent, nanobot, Prime Agent all auto-crystallize tasks into reusable skills
2. **Context efficiency** is the new battleground: GenericAgent's <30K window vs 200K-1M competitors, ALAR's 43-84% token reduction
3. **Agent-as-repo** paradigm emerging: gitagent, SOUL.md/RULES.md as first-class abstractions
4. **Pre-execution security** gaining traction: Harden AIF validates tool calls before execution
5. **Room-scoped agent collaboration**: Switch, omnigent enable agents as named team participants
6. **Language world models** for agent training: Qwen-AgentWorld simulates 7 agent domains natively
7. **Demonstration-guided execution**: UI-Mate's in-context demos boost performance by 18pp
