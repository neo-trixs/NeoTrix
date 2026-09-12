# Trending Rankings — Cycle 374

**Date**: 2026-09-12  
**Focus**: AI agents, LLM tools, reasoning frameworks, novel patterns for memory/attention/routing  
**Sources**: GitHub Trending, ProductHunt, ossinsight.io, developer blogs

---

## Top 10 New Projects (Not in Cycles 318-373)

### 1. DeerFlow 2.0 — ByteDance SuperAgent Harness
- **URL**: https://github.com/bytedance/deer-flow
- **Stars**: 81K+ (claimed #1 GitHub Trending Feb 2026)
- **Language**: Python + Node.js
- **What**: Long-horizon SuperAgent harness. Orchestrates sub-agents, memory, sandboxes, skills, message gateway. Handles tasks from minutes to hours. Ground-up rewrite of v1.
- **Key Patterns**:
  - Extensible skills system (`.agent/skills/`)
  - Sandbox-aware execution (Docker/K8s)
  - Persistent memory with LangGraph checkpointer
  - Model-agnostic (OpenAI-compatible API)
  - IM integration (Feishu, Slack, Telegram, Discord, DingTalk)
  - Claude Code skill bridge (`claude-to-deerflow`)
- **NeoTrix Mapping**: NT-ACT (orchestration), NT-MEMORY (persistent memory), NT-WORLD (skill discovery)
- **Novel Pattern**: "SuperAgent harness" — not a framework to wire together, but batteries-included with filesystem, memory, skills, sandboxed execution out of the box.

### 2. Lightpanda — AI-Native Headless Browser
- **URL**: https://github.com/lightpanda-io/browser
- **Stars**: 34K+
- **Language**: Zig
- **What**: Headless browser built from scratch for machines, not humans. 9x faster, 16x less memory than Chrome. Not a Chromium fork — a new browser engine.
- **Key Patterns**:
  - Zero graphical rendering overhead
  - CDP compatible (Playwright, Puppeteer)
  - Built-in agent mode (natural language → browser actions)
  - PandaScript: reproducible JS scripts from agent sessions
  - Multi-provider LLM support (Anthropic, OpenAI, Gemini, Ollama, etc.)
- **NeoTrix Mapping**: NT-WORLD (web perception), NT-SHIELD (isolated environments)
- **Novel Pattern**: "Browser as agent substrate" — the browser IS the agent runtime, not a tool the agent calls. Natural language → PandaScript → replayable automation.

### 3. OpenAI Codex Skills Catalog
- **URL**: https://github.com/openai/skills
- **Stars**: 594 (official catalog)
- **What**: Official skill catalog for Codex. Skills are structured directories (SKILL.md + scripts/ + references/ + assets/) with progressive disclosure: name/description at 2% context budget, full instructions on activation.
- **Key Patterns**:
  - Progressive disclosure (2% context for initial list)
  - Explicit vs implicit invocation ($ or @ triggers)
  - Record & Replay skill creation
  - Multi-tier: SYSTEM / CURATED / EXPERIMENTAL
  - Skill installer from any GitHub URL
  - Plugin ecosystem (MCP tools as dependencies)
- **NeoTrix Mapping**: NT-MIND (skill crystallization), NT-IO (skill interface)
- **Novel Pattern**: "Skill as production template" — SKILL.md <200 lines + references/ + scripts/ contract. Progressive disclosure prevents context pollution. Record & Replay lowers skill authoring barrier.

### 4. Kilo Code — Open-Source Agentic Engineering Platform
- **URL**: https://github.com/Kilo-Org/kilocode
- **Stars**: 1.5M+ users (claimed)
- **Language**: TypeScript
- **What**: All-in-one agentic coding platform across VS Code, JetBrains, CLI, Cloud. 500+ models, zero markup. Open source with open pricing.
- **Key Patterns**:
  - Multi-agent control room (parallel agents in isolated worktrees)
  - Cloud agents (browser-based, don't consume local resources)
  - JetBrains native extension (v7, ground-up rebuild)
  - Cross-device session sync
  - Mode switching: Code / Architect / Debug / Auto
  - Gateway: single API for 500+ models
- **NeoTrix Mapping**: NT-ACT (tool execution), NT-IO (multi-IDE interface), NT-CORE (model routing)
- **Novel Pattern**: "Agent command center" — one portal for every agent across IDEs, CLI, and cloud. Worktree-native isolation. Model freedom as first-class feature.

### 5. Flare — Graph-First IDE for Agentic Coding
- **URL**: https://github.com/AlgoNoRhythm/Flare
- **Stars**: ~1.5K (new, launched Aug 2026)
- **Language**: TypeScript (Electron)
- **What**: Desktop IDE where the main surface is a live dependency graph. Files = nodes, imports = edges. Terminal underneath for Claude Code / Codex / OpenCode.
- **Key Patterns**:
  - Live dependency graph (Canvas, Wheel, Districts views)
  - Burst-based review cockpit (change attribution, blast radius)
  - Agent intent recording (MCP tool `record_intent`)
  - Shadow git history (auto-commit per burst, diff/revert)
  - File tiering: read carefully / read / skim (based on blast radius + coverage + complexity)
  - Agent smell detection (shortcut patterns)
- **NeoTrix Mapping**: NT-CORE (E8 reasoning graph), NT-WORLD (codebase perception), NT-SHIELD (safety verification)
- **Novel Pattern**: "Graph-first oversight" — architecture visualization as primary IDE surface, not secondary. Burst grouping + intent recording = auditability for agentic code changes.

### 6. Orra — Plan Engine for AI Agent Workflows
- **URL**: https://github.com/orra-dev/orra
- **Stars**: 246
- **Language**: Go
- **What**: Infrastructure for resilient AI agent workflows. AI-driven plan generation, durable execution, automatic service discovery, failure recovery.
- **Key Patterns**:
  - Progressive planning (base → production with domain grounding)
  - Durable execution with state persistence
  - Pre-validated execution plans
  - Revert state for failure handling
  - Audit logs for traceability
  - Health monitoring + real-time status
  - On-premises deployment
- **NeoTrix Mapping**: NT-ACT (workflow execution), NT-MEMORY (state persistence), NT-REPAIR (failure recovery)
- **Novel Pattern**: "Plan engine ≠ workflow engine" — AI generates plans dynamically, not hardcoded workflows. Durable execution + revert = production resilience.

### 7. HydraFusion — GitHub Copilot Selective Coding
- **URL**: https://github.blog/ai-and-ml (research preview)
- **What**: Selective coding workflows in GitHub Copilot that match or exceed Opus 5 baseline while reducing cost. Research preview announced Sep 2026.
- **Key Patterns**:
  - Selective coding (not all files need full attention)
  - Cost-aware task routing
  - Parallel agent execution in Copilot app
  - Model choice expansion
- **NeoTrix Mapping**: GWT (attention routing), NT-CORE (cost-aware reasoning)
- **Novel Pattern**: "Selective coding" — GWT-style attention for code editing. Route simple tasks to cheap models, complex tasks to frontier models.

### 8. OpenMontage — Agentic Video Production System
- **URL**: https://github.com/calesthio/OpenMontage
- **Stars**: 53K+ (trending)
- **Language**: Python
- **What**: World's first open-source agentic video production system. 12 production pipelines, 100+ tools, 700+ agent skill and production-knowledge files.
- **Key Patterns**:
  - 12 production pipelines (video generation workflows)
  - 700+ agent skills (production-knowledge files)
  - Multi-tool orchestration
  - Production-grade output
- **NeoTrix Mapping**: NT-ACT (production orchestration), NT-MIND (skill library), NT-WORLD (media perception)
- **Novel Pattern**: "Production-grade agent skills" — skills encode not just instructions but production knowledge (quality thresholds, pipeline stages, tool dependencies).

### 9. MagiCrew — Open-Source AI Agent Platform
- **URL**: ProductHunt launch Sep 2026
- **Stars**: 269 upvotes on PH
- **What**: Deploy specialized digital workers that research, analyze, create reports, generate presentations. Multi-agent collaboration with enterprise controls.
- **Key Patterns**:
  - Specialized digital workers (role-based agents)
  - Multi-agent collaboration
  - Enterprise controls
  - Deliverable-ready outputs
- **NeoTrix Mapping**: NT-ACT (role-based execution), NT-GOVERNANCE (enterprise controls)
- **Novel Pattern**: "Digital workforce management" — agents as managed employees with roles, not disposable task runners.

### 10. Monid — Universal API Gateway for AI Agents
- **URL**: ProductHunt launch Sep 2026
- **Stars**: 468 upvotes on PH
- **What**: Connects AI agents to 1,800+ APIs without subscriptions. SEO, lead gen, video/music generation, social media, stocks, on-chain data.
- **Key Patterns**:
  - 1,800+ API integrations
  - Single key access
  - Cross-domain API federation
  - No per-API subscriptions
- **NeoTrix Mapping**: NT-ACT (tool federation), NT-IO (API gateway)
- **Novel Pattern**: "API federation" — single identity for agent across 1,800+ services. Contrast with MCP's per-server model.

---

## Pattern Summary

| Pattern | Projects | NeoTrix Integration |
|---------|----------|-------------------|
| **Progressive Disclosure** | Codex Skills, Flare | GWT attention gating for skill loading |
| **Durable Execution** | Orra, DeerFlow | NT-MEMORY state persistence + NT-REPAIR recovery |
| **Browser-as-Substrate** | Lightpanda | NT-WORLD perception without Chrome overhead |
| **Graph-First Oversight** | Flare | E8 reasoning graph for codebase visualization |
| **Multi-Agent Orchestration** | DeerFlow, MagiCrew, Orra | NT-ACT orchestration + NT-GOVERNANCE controls |
| **Cost-Aware Routing** | HydraFusion, Kilo Code | GWT salience + cost weight (Axiom A1) |
| **API Federation** | Monid | Ordered Backend Router (R-P82) |
| **Production Skills** | OpenMontage, Codex Skills | SKILL-SPEC.md contract (Axiom A3) |

---

## Sources

- GitHub Trending (Sep 2026): deepseek-harness, ponytail, codex, deer-flow, lightpanda
- ProductHunt (Sep 2026): Kilo Code JetBrains, Monid, Flare, MagiCrew, Agent Builder by Airtop
- ossinsight.io: Real-time AI repository rankings
- GitHub Blog (Sep 2026): HydraFusion research preview, Copilot agent sessions
- Developer blogs: DeerFlow 2.0 review, Lightpanda benchmarks, Flare analysis
