# Trending Rankings — Cycle 372 (2026-09-12)

## Methodology
GitHub trending (velocity-weighted star growth) + ProductHunt recent launches + arxiv high-signal repos. Deduplicated against cycles 318-371.

---

## 1. DeepSeek Harness
- **Repo**: deepseek/harness (GitHub) | **Stars**: +152.1K (August 2026 alone)
- **Language**: Python
- **What**: Plugin-based runtime layer for building extensible AI agents — the harness around models, tools, skills, sessions, interfaces, storage, and execution. Not a foundation model; the infrastructure layer that makes models usable.
- **Signal**: 152.1K stars in a single month — fastest-growing open-source AI project in August 2026. Dominated all GitHub trending charts.
- **NeoTrix Mapping**: NT-ACT runtime substrate. DeepSeek Harness is the "agent OS" that NT-ACT's orchestration layer should align with. Plugin-based architecture validates modular skill composition (P5). **Absorb: plugin-based agent harness as runtime primitive**.

## 2. mattpocock/skills
- **Repo**: mattpocock/skills | **Stars**: +33.9K (August 2026)
- **Language**: Markdown + Shell
- **What**: Portable engineering workflows for coding agents. Skills as small composable files — work across Claude Code, Codex, OpenCode. Each skill is a portable `.md` with embedded instructions.
- **Signal**: 33.9K stars in one month. Proved that agent skills are becoming a distribution format.
- **NeoTrix Mapping**: NT-MIND skill crystallization. mattpocock/skills validates SKILL-SPEC.md contract (<200 lines). Portable skills across agents = cross-platform skill node. **Absorb: portable .md skill files as crystallized agent behavior**.

## 3. Archify
- **Repo**: archify-dev/archify | **Stars**: +28.7K (August 2026)
- **Language**: TypeScript
- **What**: Turns codebases and systems into verifiable technical maps. Architecture, workflow, sequence, data flow, and lifecycle views. Agent skill for system comprehension.
- **Signal**: 28.7K stars in August. Architecture diagrams as agent capability.
- **NeoTrix Mapping**: NT-CORE (Des-观 architect domain). Archify's system-to-diagram capability = automated architecture documentation. Maps to ConsciousnessTree's structural self-model. **Absorb: codebase-to-architecture-map as structural memory**.

## 4. Ponytail
- **Repo**: ponytail-dev/ponytail | **Stars**: +20.2K (August 2026)
- **Language**: Python
- **What**: Pushes coding agents toward simpler, smaller implementations. Acts as a behavioral constraint — when an agent proposes over-engineering, Ponytail intervenes and suggests the minimal solution.
- **Signal**: 20.2K stars. The anti-bloat agent — "less is more" philosophy.
- **NeoTrix Mapping**: NT-MIND + NT-GOVERNANCE. Ponytail's behavioral constraint = Dark Forest axiom enforcement (modules must compile + test + connect or be deleted). Reduces unnecessary output (Ponytail principle). **Absorb: agent-bloat-prevention as behavioral governor**.

## 5. Offsite
- **Repo**: teamoffsite/offsite | **Stars**: emerging (PH Apr 2026, active Sep 2026)
- **Language**: TypeScript/React
- **What**: Shared workspace for hybrid human-agent teams. Org chart with humans and agents as interchangeable nodes. Every node is string-in-string-out (human-readable). Agent coordination via graph edges. Human-in-the-loop by default — no real-world action without approval.
- **Signal**: Novel paradigm — "agent teams as org charts." Supports Claude Code, OpenClaw, MCP-compatible agents.
- **NeoTrix Mapping**: NT-ACT orchestration + NT-IO interface. Offsite's org-chart model = NT-ACT's multi-agent coordination visualized as team topology. Human-in-the-loop = NT-SHIELD safety gate. **Absorb: org-chart agent topology as coordination interface**.

## 6. Switch (FlintAI)
- **Repo**: flintai/switch | **Stars**: emerging (PH #1 day, Sep 2026)
- **Language**: TypeScript
- **What**: Bring any AI agent into Slack, Teams, Discord. Agents join as named participants sharing context/history. Room-centric context — each room carries its own rules, participants, and context. Works with Claude Code, OpenAI ADK, LangChain. Open source, self-hostable.
- **Signal**: #1 ProductHunt day. Solves the "agents in tabs" problem — agents join existing collaboration tools.
- **NeoTrix Mapping**: NT-IO communication layer. Switch's room-centric context = NT-MEMORY per-conversation context isolation. Agent-as-participant = social presence in collaboration tools. **Absorb: room-centric agent context as collaboration primitive**.

## 7. SkillKit
- **Repo**: skillkit-ai/skillkit | **Stars**: 1.5K+ (emerging)
- **Language**: TypeScript
- **What**: Package manager for AI agent skills. Install from 400K+ skills across 31 sources, auto-translate between agent formats, ship the same skill to 46 different agents at once.
- **Signal**: npm install agent skills. Cross-agent skill portability at scale.
- **NeoTrix Mapping**: NT-MIND + NT-IO. SkillKit's cross-agent skill portability = skill-as-commodity (P5). 31 source adapters = ordered backend fallback (P4) for skill sources. **Absorb: skill package manager as skill distribution infrastructure**.

## 8. Nuphos
- **Repo**: nuphos-ai/nuphos | **Stars**: emerging (PH Aug 2026)
- **Language**: Python/TypeScript
- **What**: AI-native DevOps workspace. Shared environment where AI agents learn infrastructure, investigate issues, operate production. Connects AWS/GCP/K8s/observability. Agents read-only by default, approval for write actions. Shared audit trail.
- **Signal**: PH launch, focused on production infrastructure operations.
- **NeoTrix Mapping**: NT-WORLD (perception) + NT-SHIELD (safety). Nuphos's infrastructure learning = NT-WORLD's environment model. Approval gates = NT-SHIELD's risk assessment. Shared audit trail = NT-MEMORY event log. **Absorb: AI-native infrastructure workspace with safety gates**.

## 9. Viberia
- **Repo**: viberia-ai/viberia | **Stars**: emerging (PH May 2026, active Sep 2026)
- **Language**: TypeScript
- **What**: Spatial command center for AI agents. Isometric map view of agent org. Status icons show who's blocked/asking/done. Zoom in to chat with any agent. Agents collaborate, build teams, pick up skills. Docs, terminals, browsers built in.
- **Signal**: Novel "spatial agent management" — Civilization-style command for agents.
- **NeoTrix Mapping**: NT-IO visualization + NT-ACT orchestration. Viberia's spatial agent view = agent fleet status visualization. Team-building agents = self-organizing agent coordination. **Absorb: spatial agent management as fleet visualization**.

## 10. Timbal AI
- **Repo**: timbal-ai/platform | **Stars**: emerging (PH Jun 2026)
- **Language**: Python
- **What**: Build AI agents, workflows, and apps in one stack. ACE (Action Control Engine) provides deterministic behavioral runtime as proxy — consistent outcomes at infrastructure level, not prompt level. Code-first (export to clean Python). Enterprise compliance built in.
- **Signal**: PH launch. ACE is the "killer feature" — deterministic agent behavior layer.
- **NeoTrix Mapping**: NT-ACT + NT-SHIELD. Timbal's ACE = deterministic execution layer for agents. Maps to NT-ACT's tool execution with NT-SHIELD's safety constraints. Code-first export = PTC alignment. **Absorb: action control engine as deterministic agent runtime**.

---

## Emerging Patterns (Cycle 372)

### Pattern C372-1: Agent Harness as Platform
DeepSeek Harness (+152K stars), Amux, and Timbal AI all prove that the **harness** (runtime around the model) is becoming the platform. Models are commoditized; the orchestration, tooling, skills, and session management layers are where value accumulates. This is the "operating system" thesis for AI agents.

### Pattern C372-2: Skills as Distribution Format
mattpocock/skills (+34K), SkillKit (400K+ skills across 31 sources), and Archify all show skills are becoming the **distribution format** for agent behavior — like npm packages but for agent capabilities. Portable, composable, cross-agent.

### Pattern C372-3: Agents Join Existing Workspaces
Switch (Slack/Teams/Discord), Offsite (org chart), Viberia (spatial map), Nuphos (DevOps) — agents are moving **into** existing human tools rather than requiring separate tabs/interfaces. The agent-as-coworker paradigm requires integration, not isolation.

### Pattern C372-4: Behavioral Constraints > Capability Expansion
Ponytail (+20K stars) proves that **reducing** agent output (anti-bloat) is as valuable as expanding it. Combined with Timbal's ACE (deterministic behavior), the trend is toward **constrained agents** — doing less, but doing it reliably. Maps to Dark Forest axiom: modules that don't connect get deleted.

### Pattern C372-5: Human-in-the-Loop as First-Class
Offsite, Nuphos, and Switch all default to human approval for real-world actions. The pattern is: agents propose → humans approve → agents execute → audit trail. This is the production safety pattern replacing "autonomous agent" hype.

---

## NeoTrix Absorption Priorities

| Priority | Pattern | Absorption Target | Domain |
|----------|---------|-------------------|--------|
| P0 | DeepSeek Harness | Plugin-based agent runtime | NT-ACT |
| P0 | mattpocock/skills | Portable skill format | NT-MIND |
| P1 | Ponytail | Anti-bloat behavioral governor | NT-GOVERNANCE |
| P1 | Timbal ACE | Deterministic execution layer | NT-ACT + NT-SHIELD |
| P1 | Offsite | Org-chart agent topology | NT-ACT |
| P2 | SkillKit | Skill package manager | NT-MIND |
| P2 | Switch | Room-centric agent context | NT-IO |
| P2 | Archify | Codebase-to-architecture-map | NT-CORE |
| P3 | Nuphos | AI-native DevOps workspace | NT-WORLD |
| P3 | Viberia | Spatial agent fleet view | NT-IO |
