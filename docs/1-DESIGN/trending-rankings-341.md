# Trending Rankings — Cycle 341

**Date**: 2026-09-11
**Sources**: GitHub Trending, ProductHunt (Sep 8-11 2026), arXiv, AI Agent News

## 10 New Projects (Not in Cycles 318-340)

### 1. OpenHands (OpenHands/OpenHands)
- **URL**: https://github.com/OpenHands/OpenHands
- **Stars**: 80.8K+ | **Language**: Python | **License**: MIT
- **Category**: Autonomous Coding Agent Platform
- **Pattern**: Production-ready 1.0 release with modular SDK architecture (sdk + tools + workspace + agent-server). Docker sandboxing, built-in security policies, resource limits, plugin system. SWE-bench Verified: ~68% autonomous completion. Agent Canvas for visual orchestration. GUI + CLI + SDK entry points. ACP-compatible (works with Claude Code, Codex, Gemini). Self-hosted enterprise deployment with RBAC, audit trails, cost guardrails.
- **NeoTrix Mapping**: NT-ACT + NT-SHIELD — modular SDK = domain trait architecture. Sandbox isolation = NT-SHIELD safety kernel. Agent Canvas = NT-IO developer dashboard. **Absorption candidate**: composable SDK architecture — decouple core agent logic from tools/workspace/execution, enabling swappable backends per deployment context.

### 2. Lightpanda (lightpanda-io/browser)
- **URL**: https://github.com/lightpanda-io/browser
- **Stars**: 34.8K+ | **Language**: Zig | **License**: AGPL-3.0
- **Category**: AI-Native Headless Browser
- **Pattern**: Browser built from scratch in Zig (not Chromium/WebKit). 9x faster execution, 16x less memory than Chrome. CDP-compatible (Puppeteer/Playwright/chromedp). MCP server integration. Lightpanda Agent: LLM-driven browser control with PandaScript (replay without LLM). Native markdown output for token reduction. Instant startup for large-scale automation.
- **NeoTrix Mapping**: NT-WORLD + NT-IO — purpose-built browser for agent web perception = NT-WORLD crawler infrastructure. CDP/MCP dual interface = NT-IO protocol adapter pattern. PandaScript = cached execution template (Skill Factory pattern). **Absorption candidate**: AI-native browser as perception substrate — replace headless Chrome with purpose-built runtime for agent web interaction, reducing resource overhead 10x.

### 3. DeerFlow (bytedance/deer-flow)
- **URL**: https://github.com/bytedance/deer-flow
- **Stars**: 81.8K+ | **Language**: Python/TypeScript | **License**: MIT
- **Category**: SuperAgent Harness for Long-Horizon Tasks
- **Pattern**: LangGraph-based orchestration of sub-agents, memory, sandboxes for multi-hour autonomous tasks. All-in-One sandbox (Browser + Shell + File + MCP + VSCode). Session Goals for task decomposition. Context engineering with manual compaction. Long-term memory with configurable retention. Multi-model routing (Doubao-Seed-2.0-Code, DeepSeek v3.2, Kimi 2.5). IM gateway (Feishu, Slack, Telegram, Discord). Skills loaded progressively — only what's needed.
- **NeoTrix Mapping**: NT-MIND + NT-MEMORY — SuperAgent harness = SEAL pipeline with multi-agent orchestration. Progressive skill loading = lazy branch loading in experience-tree. Session Goals = ConsciousnessTree growth cycle decomposition. **Absorption candidate**: progressive skill loading — skills loaded only when task-matched, not pre-loaded. Reduces context waste for long-horizon agent execution.

### 4. Revolte (revolte.ai)
- **URL**: https://revolte.ai
- **Stars**: ProductHunt #4 (May 2026) | **Category**: AI Software Delivery Lifecycle
- **Pattern**: Full SDLC execution: Jira → Figma → Git → Code → Test → Deploy → Operate. Agent Harness coordinates coding, testing, review, release, operations. Platform-as-Code (PaC): one YAML defines platform requirements. Human-in-the-loop governance at every gate. DORA metrics + flow metrics built-in observability. Enterprise: 12+ multi-cluster deployments, 4.2M agent actions, 85% faster lead time.
- **NeoTrix Mapping**: NT-ACT + NT-GOVERNANCE — Platform-as-Code = capability-as-declaration (YAML-driven agent workflows). Governance gates = rev-officer audit dimensions. DORA metrics = HeartbeatAggregator for delivery health. **Absorption candidate**: Platform-as-Code pattern — declarative agent workflow specification that converts to executable pipelines with governance checkpoints.

### 5. AutoHedge (The-Swarm-Corporation/AutoHedge)
- **URL**: https://github.com/The-Swarm-Corporation/AutoHedge
- **Stars**: 5.2K+ | **Language**: Python | **License**: MIT
- **Category**: Autonomous Agent Hedge Fund
- **Pattern**: Multi-agent swarm for autonomous trading. Director Agent (strategy) → Quant Agent (analysis) → Risk Manager (sizing) → Execution Agent (orders). Risk-first architecture: risk assessment before any execution. Structured JSON output for downstream systems. Enterprise logging with audit trail. Solana trading live, Coinbase coming. Built on Swarms framework.
- **NeoTrix Mapping**: NT-ACT + NT-SHIELD — multi-agent pipeline = domain module orchestration (Director→Quant→Risk→Execution maps to NT-CORE→NT-MIND→NT-SHIELD→NT-ACT). Risk-first = NT-SHIELD pre-execution validation. **Absorption candidate**: risk-first agent architecture — mandatory risk assessment gate before any action execution, not just security validation.

### 6. Screenpipe (screenpipe/screenpipe)
- **URL**: https://github.com/screenpipe/screenpipe
- **Stars**: 21.4K+ | **Language**: Rust/TypeScript | **License**: MIT (source-available)
- **Category**: AI Agent Memory from Screen Capture
- **Pattern**: YC S26. Local-first continuous screen + audio capture. Extracts text via accessibility APIs + OCR fallback. Searchable timeline with app context. MCP server for AI integration. "Pipes" — scheduled AI agents as markdown files that query screen data. Privacy: all processing on-device, custom AI model for private info detection. Triggers on meaningful events (app switches, clicks, typing pauses) not every second. 100% local with optional cloud sync.
- **NeoTrix Mapping**: NT-MEMORY + NT-WORLD — screen capture as perception substrate = NT-WORLD sensory input. Pipes as scheduled agents = SEAL pipeline scheduled tasks. Local-first = NT-SHIELD privacy architecture. **Absorption candidate**: event-driven screen capture — capture on meaningful events, not continuous recording. Reduces storage/compute while maintaining context for agent memory.

### 7. gr (HexmosTech/git-lrc)
- **URL**: https://github.com/HexmosTech/git-lrc
- **Stars**: 3.8K+ | **Language**: Python | **License**: MIT
- **Category**: Micro AI Code Reviews on Git Commit
- **Pattern**: Free, lightweight AI code review that runs on `git commit`. Post-commit hook triggers review automatically. Detects security vulnerabilities, bugs, maintainability issues. Minimal integration — no IDE plugin needed. Works with any LLM provider. Review output in terminal, not a separate dashboard.
- **NeoTrix Mapping**: NT-SHIELD + NT-ACT — commit-hook review = SelfTest T3 production wiring (automatic quality gate). Micro approach = zero-dependency tooling philosophy. **Absorption candidate**: commit-triggered code review as automatic quality gate — every commit passes through AI review before push, integrated into existing git workflow.

### 8. OpenClaw (openclaw.com)
- **URL**: https://openclaw.com
- **Stars**: 8.9K+ | **Language**: Python | **License**: MIT
- **Category**: Open-Source AI Agent for Real Work
- **Pattern**: Native agent that does real things on your machine. Cross-harness config dashboard (Claude Code, Codex, Gemini CLI). Auto-accept mode, AI commit generator. 13 language support. Browser backend integration (Lightpanda, Hermes Agent). Focus on practical desktop automation over theoretical capabilities.
- **NeoTrix Mapping**: NT-ACT + NT-IO — cross-harness dashboard = NT-IO provider abstraction. Desktop automation = NT-ACT tool execution layer. **Absorption candidate**: cross-harness configuration — unified config for multiple agent backends, enabling seamless provider switching without workflow disruption.

### 9. kodus-ai (kodustech/kodus-ai)
- **URL**: https://github.com/kodustech/kodus-ai
- **Stars**: 2.1K+ | **Language**: Python | **License**: Apache 2.0
- **Category**: AI Code Review with Model Choice Control
- **Pattern**: Full control over model choice and costs for code review. Self-hosted, privacy-first. Custom rules engine for team-specific review criteria. Integrates with GitHub/GitLab PR workflow. Cost transparency: see exactly what each review costs per model. Supports local LLMs for air-gapped environments.
- **NeoTrix Mapping**: NT-SHIELD + NT-IO — model choice control = GWT cost-aware routing (A1 axiom). Custom rules = NT-GOVERNANCE policy engine. **Absorption candidate**: cost-transparent code review — explicit per-review cost tracking with model selection, enabling budget-aware quality gates.

### 10. SkillDock (wanghuan9/skilldock)
- **URL**: https://github.com/wanghuan9/skilldock
- **Stars**: 1.2K+ | **Language**: TypeScript | **License**: MIT
- **Category**: AI Skill Manager Desktop App
- **Pattern**: Install, organize, edit, sync, and update Skills, MCP servers, and plugins with real-directory scanning. Git-aware Diff previews for skill changes. Works across Claude Code, Cursor, Codex, Windsurf, Gemini CLI. Visual skill tree management. Skill version control with rollback.
- **NeoTrix Mapping**: NT-MIND + NT-IO — skill management = SEAL pipeline skill registry. Git-aware diffs = experience-tree version control. **Absorption candidate**: visual skill tree management — graphical interface for organizing, versioning, and diffing agent skills with rollback capability.

## Cross-Cutting Themes (Cycle 341)

| Theme | Projects | NeoTrix Impact |
|-------|----------|----------------|
| **Modular SDK Architecture** | OpenHands, DeerFlow | Decouple core logic from tools/workspace — swappable backends per deployment context |
| **AI-Native Browser Runtime** | Lightpanda, OpenClaw | Purpose-built browsers for agent web perception replacing headless Chrome — 10x resource reduction |
| **Risk-First Execution** | AutoHedge, Harden AIF (340) | Mandatory risk assessment before any action, not just security validation |
| **Declarative Agent Workflows** | Revolte (PaC), Screenpipe (Pipes) | YAML/markdown-defined agent workflows converted to executable pipelines |
| **Event-Driven Capture** | Screenpipe, gr | Capture on meaningful events, not continuous — reduce storage/compute while maintaining context |
| **Cost-Transparent AI** | kodus-ai, 9router (340) | Explicit per-action cost tracking with model selection for budget-aware operation |
| **Skill Lifecycle Management** | SkillDock, DeerFlow | Skills as versioned, diffable, rollbackable artifacts — not just code files |

## NeoTrix Absorption Priority

| Priority | Pattern | Source | Target Domain |
|----------|---------|--------|---------------|
| P0 | Event-driven screen capture for agent memory | Screenpipe | NT-MEMORY (perception) |
| P0 | Progressive skill loading — only when task-matched | DeerFlow | NT-MIND (lazy loading) |
| P1 | AI-native browser as perception substrate | Lightpanda | NT-WORLD (crawler infra) |
| P1 | Risk-first execution architecture | AutoHedge | NT-SHIELD (pre-execution) |
| P1 | Platform-as-Code declarative workflows | Revolte | NT-ACT (workflow engine) |
| P2 | Commit-triggered code review gate | gr | NT-SHIELD (quality gate) |
| P2 | Cross-harness unified configuration | OpenClaw | NT-IO (provider abstraction) |
| P2 | Composable SDK architecture | OpenHands | NT-ACT (modular design) |
| P3 | Cost-transparent code review | kodus-ai | NT-IO (budget tracking) |
| P3 | Visual skill tree management | SkillDock | NT-MIND (skill registry) |
