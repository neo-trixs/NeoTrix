# Trending Rankings — Cycle 340

**Date**: 2026-09-11
**Sources**: GitHub Trending, ProductHunt (Sep 6-9 2026), arXiv, ossinsight.io

## 10 New Projects (Not in Cycles 318-339)

### 1. nanobot (HKUDS/nanobot)
- **URL**: https://github.com/HKUDS/nanobot
- **Stars**: 47.6K+ | **Language**: Python | **License**: MIT
- **Category**: Ultra-Lightweight Personal AI Agent Framework
- **Pattern**: Single-core readable framework combining tools, long-term memory (Dream), MCP integrations, model routing, multi-agent delegation, scheduled automation, and OpenAI-compatible API. Chat-native reach: WebUI, Telegram, Discord, Slack, WeChat, Email, Mattermost. Model freedom: OpenAI-compatible + local LLMs + image generation + search + fallbacks. Dream memory system for persistent context across sessions.
- **NeoTrix Mapping**: NT-IO + NT-MEMORY — Dream memory = experience-tree persistent context. Model routing = GWT salience-based provider selection. Multi-agent delegation = EventBus task distribution. **Absorption candidate**: Dream memory system for cross-session persistence + chat-native agent gateway pattern.

### 2. Webwright (microsoft/Webwright)
- **URL**: https://github.com/microsoft/Webwright
- **Stars**: 5.9K+ | **Language**: Python | **License**: MIT
- **Category**: Browser Agent Framework with Skill Factory
- **Pattern**: SWE-style browser agent achieving SOTA on long-horizon web tasks. Code-as-action beats coordinate prediction. Skill Factory: every solve leaves a reusable script behind, distilled into parameterized code skills that rerun standalone with ~40s and zero tokens. On WebArena, reuse lifts held-out accuracy 55% → 70% (+15pp). Plugin manifests for Claude Code, Codex, OpenClaw, Hermes Agent.
- **NeoTrix Mapping**: NT-ACT + NT-MIND — Skill Factory = SEAL pipeline skill crystallization from execution traces. Reusable parameterized skills = experience-tree entries as executable templates. Zero-token reuse = cached capability activation. **Absorption candidate**: Skill Factory pattern — solve → extract script → parameterize → cache → reuse without LLM inference.

### 3. Ponytail (DietrichGebert/ponytail)
- **URL**: https://github.com/DietrichGebert/ponytail
- **Stars**: 91.8K+ | **License**: MIT
- **Category**: Anti-Over-Engineering Skill for Coding Agents
- **Pattern**: "Lazy senior dev" prompt that forces minimal YAGNI-first code. Benchmarked on real agent sessions: -54% LOC, -22% tokens, -20% cost, -27% time, 100% safe. Three intensity levels (lite/full ultra). Commands: `/ponytail-review` (diff audit), `/ponytail-audit` (repo-wide), `/ponytail-debt` (deferred shortcuts ledger). Works across Claude Code, Codex, Cursor, Devin CLI, OpenCode, Gemini, Hermes Agent.
- **NeoTrix Mapping**: NT-CORE + NT-SHIELD — anti-over-engineering = Dark Forest axiom enforcement (modules must compile + test + connect or be deleted). Review/audit commands = SelfTest T3 production wiring validation. **Absorption candidate**: over-engineering detection as automated audit dimension — scan agent output for unnecessary complexity, enforce YAGNI at the code generation layer.

### 4. Vercel Eve (vercel/eve)
- **URL**: https://github.com/vercel/eve
- **Stars**: 4.9K+ | **License**: Apache 2.0
- **Category**: Filesystem-First Agent Framework
- **Pattern**: Core agent capabilities live in conventional filesystem locations. Projects are inspectable, extendable, and operable by reading the filesystem structure. Documentation lives in `node_modules/eve/docs` so coding agents can read it locally. Beta status with Vercel infrastructure backing.
- **NeoTrix Mapping**: NT-ACT — filesystem-as-agent-state maps to our directory conventions (nt_* modules). Agent capabilities as inspectable files = domain modules with traits.rs interfaces. **Absorption candidate**: filesystem-first convention for agent state — capabilities discoverable by filesystem structure, not runtime registration.

### 5. 9router (decolua/9router)
- **URL**: https://github.com/decolua/9router
- **Stars**: 23.7K+ | **License**: MIT
- **Category**: Free AI Router & Token Saver
- **Pattern**: RTK token saver compresses tool outputs (git diff, grep, ls, tree) before sending to LLM — saves 20-40% input tokens. Smart 3-tier fallback: Subscription → Cheap → Free. Real-time quota tracking. Format translation across OpenAI/Claude/Gemini/Cursor/Kiro/Vertex. Multi-account support with load balancing. Integrates Caveman (terse output), Ponytail (minimal code), Headroom (context compression).
- **NeoTrix Mapping**: NT-IO + NT-SHIELD — 3-tier fallback = GWT cost-aware routing (A1 axiom). Token compression = context-as-scarce-resource (A2 axiom). Format translation = NT-IO provider abstraction. **Absorption candidate**: RTK-style tool output compression as pre-processing layer before LLM context injection — reduce token cost at the transport layer.

### 6. Harden AIF (harden.run)
- **URL**: https://harden.run
- **Category**: Security Layer for AI Coding Agents (ProductHunt #2, Sep 9)
- **Pattern**: Free, local security tool for AI coding agents. Post-trained model checks tool calls before they run, using request and session context. Beat frontier models on agent-security benchmarks. Keeps repo and tool output on your machine — no external data leakage. Action-as-gate: every tool call passes through the security model before execution.
- **NeoTrix Mapping**: NT-SHIELD — post-trained security model = egress privacy guard for tool calls (not just outbound requests). Action-as-gate = NT-SHIELD sandbox pre-execution validation. **Absorption candidate**: tool-call-level security gate — validate every agent action against a trained security model before execution, not just network egress.

### 7. Switch (FlintAI/switch)
- **URL**: https://www.flintai.dev/products/switch
- **Category**: Multi-Agent Collaboration Platform (ProductHunt #1, Sep 8)
- **Pattern**: Bring any AI agent into Slack, Teams, Discord. Agents join as named participants sharing context and history with the team. Connect once, use across projects. Each room carries its own context, participants, and rules. Works with Claude Code, OpenAI, Google ADK, LangChain. Open source, self-hostable.
- **NeoTrix Mapping**: NT-IO + NT-ACT — agent-as-team-member = domain module participation in cross-domain workflows. Room-scoped context = per-domain KB namespace isolation. **Absorption candidate**: agent-as-collaboration-participant pattern — agents as named entities in team channels with shared context and room-level governance.

### 8. GenericAgent (lsdefine/genericagent)
- **URL**: https://github.com/lsdefine/genericagent
- **Stars**: 14.1K+ | **Language**: Python | **License**: MIT
- **Category**: Minimal Self-Evolving Autonomous Agent
- **Pattern**: ~3K lines of core code. 9 atomic tools + ~100-line Agent Loop grants system-level control (browser, terminal, filesystem, keyboard/mouse, screen vision, mobile ADB). Self-evolving: crystallizes each task into a Skill, forming a personal skill tree. Token efficient: <30K context window vs 200K-1M others. Morphling mode: project-level skill absorption from external repos. Goal Hive: multi-worker cooperative long-horizon goals via BBS coordination.
- **NeoTrix Mapping**: NT-MIND + NT-ACT — skill crystallization = SEAL pipeline distillation. Goal Hive = production orchestrator with worker coordination. Morphling mode = external-absorption skill. **Absorption candidate**: minimal agent loop pattern — 9 atomic tools + 100-line loop achieving system-level control. Token efficiency via information density maximization.

### 9. Mastra Factory (mastra.ai)
- **URL**: https://mastra.ai
- **Category**: AI Agent Framework from Gatsby Team (ProductHunt #1, Sep 9)
- **Pattern**: From issue to production, run by agents. Framework for building AI-powered apps and agents with workflows, memory, streaming, evals, tracing, and Studio (interactive UI for dev and testing). `npm create mastra@latest` scaffolding. Production-grade with built-in observability.
- **NeoTrix Mapping**: NT-ACT + NT-IO — workflow engine = SEAL pipeline orchestration. Studio UI = NT-IO developer dashboard. Built-in evals/tracing = SelfTest T3 production wiring. **Absorption candidate**: issue-to-production agent pipeline — agent takes an issue, implements, tests, and ships. Studio as interactive development environment for agent workflows.

### 10. OpenMarket (openmarket.m11.ai)
- **URL**: https://openmarket.m11.ai
- **Category**: Multi-Agent Marketplace (ProductHunt #4, Sep 8)
- **Pattern**: Multi-agent marketplace where proof decides who wins. Sellers pitch, competitors challenge claims, independent truth agents verify evidence. Claims must survive scrutiny to earn a buyer's sale. Adversarial verification: not just self-reported quality but challenged and proven.
- **NeoTrix Mapping**: NT-GOVERNANCE + NT-SHIELD — truth agents = rev-officer audit dimensions. Adversarial verification = cross-domain consistency checking. Proof-based selection = evidence-first methodology. **Absorption candidate**: adversarial marketplace pattern — claims must survive independent verification before acceptance. Maps to NT-GOVERNANCE quality gates.

## Cross-Cutting Themes (Cycle 340)

| Theme | Projects | NeoTrix Impact |
|-------|----------|----------------|
| **Token Compression at Transport** | 9router, Ponytail | Pre-processing layer to reduce token cost before LLM context — RTK-style compression for tool outputs |
| **Skill Crystallization from Execution** | Webwright, GenericAgent | Skills emerge from solving tasks, not from manual creation — SEAL pipeline should auto-crystallize from execution traces |
| **Agent-as-Participant** | Switch, OpenMarket | Agents as named entities in team/marketplace contexts with governance — extends NT-ACT beyond solo execution |
| **Security as Pre-Execution Gate** | Harden AIF | Every tool call validated before execution, not just network egress — extends NT-SHIELD to action-level gating |
| **Minimal Agent Loops** | GenericAgent, nanobot | Small core + atomic tools beats large monolithic frameworks — validates NeoTrix's modular domain architecture |
| **Filesystem-as-State** | Eve, GitAgent | Agent capabilities discoverable via filesystem structure — validates our nt_* directory conventions |

## NeoTrix Absorption Priority

| Priority | Pattern | Source | Target Domain |
|----------|---------|--------|---------------|
| P0 | RTK-style tool output compression | 9router | NT-IO (pre-processing) |
| P0 | Skill Factory — solve→extract→parameterize→cache | Webwright | NT-MIND (SEAL pipeline) |
| P1 | Tool-call-level security gate | Harden AIF | NT-SHIELD (pre-execution) |
| P1 | Agent-as-collaboration-participant | Switch | NT-ACT (multi-agent) |
| P2 | Over-engineering detection audit | Ponytail | NT-CORE (Dark Forest) |
| P2 | Filesystem-first agent state | Eve | NT-ACT (conventions) |
| P3 | Adversarial marketplace verification | OpenMarket | NT-GOVERNANCE |
| P3 | Dream memory for cross-session persistence | nanobot | NT-MEMORY |
