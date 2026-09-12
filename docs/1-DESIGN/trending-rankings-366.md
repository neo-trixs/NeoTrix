# Trending Rankings — Cycle 366 (2026-09-12)

## 10 New Projects (Not in Cycles 318–365)

### 1. Archify — Codebase-to-Map Agent Skill
- **URL**: https://github.com/archify-dev/archify
- **Stars**: 28,700+ (August 2026 star growth)
- **Language**: TypeScript
- **What**: Agent skill that turns codebases and systems into verifiable technical maps. Generates architecture, workflow, sequence, data flow, and lifecycle views from real code or descriptions. Output is structured diagram-as-code, not opaque images. Used by Claude Code, Cursor, Codex.
- **Key Insight**: "Diagrams from code, not from imagination" — validates architecture against actual code structure. Maps are versionable, diffable, and composable. Skill is a single command `/archify` that produces diagram output the agent can reason about.
- **NeoTrix Mapping**: NT-CORE → architecture visualization as E8 hexagram state representation. NT-WORLD → codebase perception as structured diagram extraction. NT-MIND → architecture drift detection via map-to-code diff.

### 2. Ponytail — Simplicity-Enforcing Agent Layer
- **URL**: https://github.com/ponytail-dev/ponytail
- **Stars**: 20,200+ (August 2026 star growth)
- **Language**: TypeScript/Python
- **What**: Behavior layer that pushes coding agents toward simpler, smaller implementations. Acts as a post-processor that flags over-engineering, suggests YAGNI deletions, and enforces minimal solution bias. Works as MCP server with any agent.
- **Key Insight**: "Less code, fewer tokens, faster completion" — the anti-agent that removes unnecessary complexity. Measures code simplicity score, suggests reductions, blocks verbose patterns. 65% output token savings via terse-speak prompting + YAGNI enforcement.
- **NeoTrix Mapping**: NT-MIND → SEAL pipeline simplicity gate (evolution should produce minimal viable solutions). NT-CORE → E8 reasoning should prefer simpler hexagram paths. NT-ACT → tool output compression before LLM consumption.

### 3. Orca — Multi-Agent Fleet Coordinator
- **URL**: https://github.com/orca-dev/orca
- **Stars**: 18,800+ (August 2026 star growth)
- **Language**: Python/TypeScript
- **What**: Coordinates fleets of parallel coding agents across projects. Central monitoring dashboard, shared context pools, workload balancing, failure recovery. Agents run independently but share project-level state through coordination bus.
- **Key Insight**: "One agent is good; a fleet is a team" — parallel execution with shared project awareness. Coordination bus handles inter-agent communication without tight coupling. Fleet-level resource management (CPU, memory, API quota) prevents resource exhaustion.
- **NeoTrix Mapping**: NT-ACT → fleet-level task orchestration across multiple NT-ACT instances. NT-SHIELD → fleet-level security policies and quota enforcement. NT-MEMORY → shared context pool as distributed KB access.

### 4. Flue — Sandbox Agent Framework (Astro)
- **URL**: https://github.com/withastro/flue
- **Stars**: 8,056
- **Language**: TypeScript
- **What**: Agent harness framework with built-in sandbox execution. Sessions, tools, skills, instructions, filesystem access, and secure sandbox in one package. CLI for local runs, deploy to any host. OpenTelemetry + Braintrust + Sentry observability. Postgres persistence.
- **Key Insight**: "Not another SDK — a programmable TypeScript harness" — durable execution survives failures and restarts. Sandboxes give agents safe filesystem + code execution. Skills are loadable procedures, not context the model reads. Observability is built-in, not bolted on.
- **NeoTrix Mapping**: NT-ACT → sandbox execution model for safe tool invocation. NT-REPAIR → durability and checkpointing for recovery. NT-SHIELD → sandbox isolation as security boundary.

### 5. Apache Maka — Local-First Agent Workspace
- **URL**: https://github.com/apache/maka
- **Stars**: 296 (Incubating, new project)
- **Language**: Rust/Python
- **What**: Local-first AI agent workspace where model messages, tool calls, tool results, permission decisions, and termination events are recorded as an append-only log. Full audit trail. No cloud dependency. Designed for privacy-sensitive environments.
- **Key Insight**: "Append-only log as single source of truth" — every agent action is an immutable event. Workspace state is fully reconstructable from the log. Local-first means no data leaves the machine unless explicitly configured.
- **NeoTrix Mapping**: NT-MEMORY → append-only experience log (experience-tree nodes as immutable events). NT-SHIELD → local-first execution with full audit trail. NT-CORE → event reconstruction for ConsciousnessTree state recovery.

### 6. OpenAI Agents API — Managed Agent Infrastructure
- **URL**: https://openai.com/index/introducing-the-agents-api/
- **Stars**: N/A (API, not repo — Sep 10 2026 launch)
- **Language**: API/SDK (Python, TypeScript)
- **What**: Managed agent harness powering Codex, now exposed as API. Context auto-compaction, tool search (lazy tool loading), programmatic tool calling (parallel/chain), multi-agent subagent delegation. Hosted sandboxes + custom infrastructure options.
- **Key Insight**: "Harness as a service" — OpenAI maintains the runtime, developers focus on tools + knowledge. Tool search loads relevant tool definitions on-demand, reducing token usage. Context compaction preserves critical information across context window boundaries. Subagent delegation for parallel work.
- **NeoTrix Mapping**: NT-IO → provider-side harness integration. NT-ACT → tool search = lazy tool loading (GWT attention for tools). NT-MEMORY → context compaction = experience-tree summarization.

### 7. Harden AIF — Security Layer for AI Coding Agents
- **URL**: https://harden.run/
- **Stars**: New (ProductHunt #2, Sep 9 2026, Score 405)
- **Language**: Python
- **What**: Free, local security tool for AI coding agents. Post-trained model checks tool calls before they run, using request + session context. Beats frontier models on agent-security benchmarks. Keeps repo and tool output on your machine — no external data flow.
- **Key Insight**: "Pre-execution security gate" — validates tool calls against intent + context before they touch the filesystem or network. Session-aware: same tool call might be safe in one context, dangerous in another. Local-only architecture prevents data exfiltration.
- **NeoTrix Mapping**: NT-SHIELD → pre-execution tool validation (agent-integrity-foundation pattern). NT-ACT → tool call guard before NT-ACT execution. NT-CORE → session-aware risk assessment.

### 8. 49agents IDE — 2D Canvas for Agent Fleet Management
- **URL**: https://49agents.com/
- **Stars**: 127 upvotes (ProductHunt Sep 9 2026)
- **Language**: TypeScript
- **What**: 2D canvas where every agent, terminal, repo, and machine lives on a single map you build yourself. Solves tab navigation fatigue for multi-agent workflows. City-builder-like UX: associate processes to spatial positions, return to them days later by location memory.
- **Key Insight**: "Spatial memory for agent fleets" — humans remember locations better than tab names. Building your own map creates personal cognitive structure. 2D spatial layout reduces context-switching overhead for 10x engineer workflows.
- **NeoTrix Mapping**: NT-CORE → spatial attention mapping (GWT could use visual-spatial salience). NT-IO → multi-agent visualization. NT-MEMORY → spatial memory palace for experience nodes.

### 9. OmniRoute — Open-Source AI Gateway
- **URL**: https://github.com/omniroute/omniroute
- **Stars**: 16,800+ (August 2026 star growth)
- **Language**: Go/Python
- **What**: Single AI gateway in front of hundreds of providers and 1,000+ model endpoints. Routing strategies, automatic fallback, quota management, token compression, MCP support, A2A (agent-to-agent) protocol, desktop interfaces. Self-hosted alternative to OpenRouter.
- **Key Insight**: "Routing infrastructure for agent fleets" — as agents multiply, shared routing becomes essential. Token compression at gateway level (not per-provider). A2A protocol enables agent-to-agent communication through the gateway. Quota management prevents runaway costs.
- **NeoTrix Mapping**: NT-IO → Ordered Backend Router (P4) at gateway scale. NT-ACT → A2A protocol for inter-agent communication. NT-SHIELD → gateway-level quota enforcement and egress guard.

### 10. Dial — Phone Numbers for AI Agents
- **URL**: https://dial.so/
- **Stars**: New (ProductHunt top 10, Sep 2026)
- **Language**: API
- **What**: Gives AI agents real phone numbers in 10 seconds. Agents can make/receive calls, send SMS, leave voicemails. Twilio-like but agent-native: designed for autonomous workflows, not human dialers. Supports voice + text channels.
- **Key Insight**: "Agents need identity in the phone network" — phone numbers are identity, trust signals, and communication channels. Agent-native telephony means no human-in-the-loop for outbound/inbound calls. Voice as first-class agent interface.
- **NeoTrix Mapping**: NT-IO → voice/telephony as agent interface modality. NT-ACT → outbound agent actions via phone. NT-SHIELD → phone number identity verification and spam protection.

---

## Meta-Patterns Across Cycle 366

| Pattern | Projects | NeoTrix Implication |
|---------|----------|---------------------|
| **Fleet Coordination** | Orca, OmniRoute, 49agents IDE | NT-ACT needs fleet-level orchestration, not just single-agent execution |
| **Simplicity Enforcement** | Ponytail, Archify | SEAL pipeline should gate on simplicity metrics; architecture maps should validate against code |
| **Pre-Execution Security** | Harden AIF, Apache Maka | NT-SHIELD must validate tool calls before execution, not after |
| **Spatial/Visual Agent Management** | 49agents IDE, Archify | NT-CORE attention could benefit from spatial-salience mapping |
| **Append-Only Audit Trails** | Apache Maka, OpenAI Agents API | Experience nodes as immutable events with full reconstruction capability |
| **Agent-as-Phone-Identity** | Dial | NT-IO multi-modal interface extends beyond text/code to voice/telephony |

---

## Cycle 366 vs Cycle 365 Delta

| Dimension | Cycle 365 | Cycle 366 | Shift |
|-----------|-----------|-----------|-------|
| **Focus** | Context engines + token compression | Fleet coordination + security | From efficiency → operational maturity |
| **Agent Scale** | Single agent optimization | Multi-agent fleet management | Scaling from 1→N agents |
| **Security** | Post-hoc guardrails | Pre-execution validation | Moving security left |
| **UX** | Terminal-first | Spatial canvas | New interaction paradigm |
| **Infrastructure** | Token savings | Gateway + A2A protocol | Network-level agent coordination |
