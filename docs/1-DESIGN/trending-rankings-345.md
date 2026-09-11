# Trending Rankings — Cycle 345

**Date**: 2026-09-11
**Sources**: GitHub Trending, ProductHunt (Sep 9-11 2026), arXiv, AWS Open Source Blog

## 10 New Projects (Not in Cycles 318-344)

### 1. Pizza Bot (aws/pizza-bot)
- **URL**: https://github.com/aws/pizza-bot
- **Stars**: New (Sep 10 2026) | **Language**: TypeScript | **License**: Apache 2.0
- **Category**: Background Agent Inbox
- **Pattern**: Open-source application that runs AI agents in the background and gives their work an inbox. Agents come back when finished or stuck, not during execution. Born inside Amazon (2000+ internal users). Covers meeting prep, email drafting, Slack summaries, CRM logging, web research. Named after Amazon two-pizza teams. SDK for building custom agents. Long-running agent scheduling with persistent queues.
- **NeoTrix Mapping**: NT-ACT + NT-IO — background agent inbox = EventBus async task queue with completion notification. Agent-driven status reporting = NT-ACT autonomy loop (agent decides when to escalate). **Absorption candidate**: agent inbox pattern — decouple agent execution from human attention; agents complete work autonomously and notify only at completion/blockage, not mid-stream. Maps to NT-ACT resource budget management (background tasks don't consume active context).

### 2. Harden AIF (harden.run)
- **URL**: https://harden.run
- **Stars**: PH #2 (Sep 9, 405pts) | **Category**: Security Layer for AI Coding Agents
- **Pattern**: Free, local security tool for AI coding agents. Post-trained model checks tool calls before execution using request + session context. Beats frontier models on agent-security benchmarks. Runs entirely locally — repo and tool output never leave machine. Static scanning is bypassable; Harden's model-based checking is unbypassable. Trust-level classification: different skills get different security policies.
- **NeoTrix Mapping**: NT-SHIELD — pre-execution tool call validation = NT-SHIELD egress guard inverted (inbound tool call validation). Trust-level classification = NT-SHIELD trust tiers (Trusted/Contracted/Untrusted). **Absorption candidate**: tool-call security gate — post-trained model validates every tool invocation against session context before execution, preventing injection, data exfiltration, and privilege escalation. Maps to NT-SHIELD's inbound guard for tool call sanitization.

### 3. HyperProbe (hyperprobe.dev)
- **URL**: https://hyperprobe.dev
- **Stars**: PH #4 (Sep 5, 208pts) | **Category**: Production Debugging for AI Agents
- **Pattern**: Let AI agents debug production without redeploying. Agents (Claude Code, Codex, Cursor) drop read-only probes into running services to capture variable state never recorded. Agent debugs like it has local repro, closing bugs in one sitting. No log-line-and-wait cycle. Works with existing agent harnesses.
- **NeoTrix Mapping**: NT-REPAIR + NT-WORLD — production probing = NT-REPAIR non-invasive diagnostic probes. Read-only observation = NT-WORLD perception without mutation. **Absorption candidate**: agent-driven production diagnostics — agents deploy lightweight read-only probes into live systems to capture state for debugging, avoiding the deploy-observe-fix cycle. Maps to NT-REPAIR self-healing observation phase.

### 4. Mastra Factory (mastra.ai)
- **URL**: https://mastra.ai
- **Stars**: PH #1 (Sep 9, 491pts) | **Language**: TypeScript | **License**: Open Source
- **Category**: Agent Framework with Workflows, Memory, Evals
- **Pattern**: From the Gatsby team. Framework for building AI-powered apps and agents with workflows, memory, streaming, evals, tracing, and Studio (interactive UI for dev/testing). `npm create mastra@latest`. Production-grade: evals and tracing built-in, not bolted on. Studio provides visual debugging of agent workflows. Community ecosystem.
- **NeoTrix Mapping**: NT-MIND + NT-IO — agent framework = SEAL pipeline workflow orchestration. Built-in evals = SelfTest T2/T3 registration. Studio = ConsciousnessTree visual dashboard. **Absorption candidate**: eval-native agent development — workflows, memory, evals, and tracing as first-class citizens, not afterthoughts. Agents instrumented from birth, not retrofitted.

### 5. Noodle Seed (noodleseed.com)
- **URL**: https://noodleseed.com
- **Stars**: PH #5 (Sep 9, 254pts) | **Category**: Agent-Native Product Runtime
- **Pattern**: Make products ready for AI agents. Build workflows in TypeScript, expose through secure branded assistant inside your product, make same capabilities available to external agents. Governed runtime for identity, permissions, secrets, audit, operations. Instead of stitching MCP SDKs + hosting infra, provides the full governance layer. Product-facing + agent-facing dual interface.
- **NeoTrix Mapping**: NT-SHIELD + NT-IO — governed runtime = NT-SHIELD identity/permission/audit layer. Dual interface (human + agent) = NT-IO multi-channel gateway. **Absorption candidate**: product-agent dual interface — single workflow engine serving both human-facing assistants and external agent callers, with unified governance (identity, permissions, audit). Maps to NT-IO's multi-channel adapter pattern with NT-SHIELD trust enforcement.

### 6. GoModel (gomodel.enterpilot.io)
- **URL**: https://gomodel.enterpilot.io
- **Stars**: PH #10 (Sep 9, 126pts) | **Language**: Go | **License**: MIT
- **Category**: Open-Source AI Gateway
- **Pattern**: Open-source AI gateway. One OpenAI-compatible API for every provider. Budgets, caching, guardrails, load balancing, failover. Single binary, ~20MB Docker image. Self-hosted alternative to OpenRouter and LiteLLM. Bring your own keys. Cost tracking per provider/model. Ordered backend fallback with health checks.
- **NeoTrix Mapping**: NT-IO — unified AI gateway = NT-IO provider abstraction layer. Ordered fallback = Ordered Backend Router (P4 pattern). Caching + budgeting = Axiom A1 (Cost-Aware Routing). **Absorption candidate**: lightweight AI gateway — single-binary provider router with ordered fallback, budget enforcement, and health-check-driven failover. Validates NeoTrix's provider selection pattern with `total_calls ascending` rotation.

### 7. 49agents IDE (49agents.com)
- **URL**: https://49agents.com
- **Stars**: PH #9 (Sep 9, 127pts) | **License**: Open Source
- **Category**: 2D Agent Workspace IDE
- **Pattern**: 2D canvas where every agent, terminal, repo, and machine lives on a single map. Solves tab navigation fatigue for 10x engineers. City-builder-like UX — associate processes to spatial positions. Agents visually positioned by task domain. Comes back to agents days later by spatial memory, not tab hunting.
- **NeoTrix Mapping**: NT-IO + NT-CORE — spatial agent workspace = NT-IO multi-agent orchestration with spatial layout. Tab fatigue solution = GWT attention routing across visual workspace. **Absorption candidate**: spatial agent IDE — agents positioned in 2D workspace by domain/task, reducing cognitive overhead of multi-agent management. Maps to GWT's attention routing across visual-spatial representation.

### 8. OpenResearcher (lambda.ai/openresearcher)
- **URL**: https://lambda.ai/blog/openresearcher-training-research-agents-at-scale
- **Stars**: New (Sep 10 2026) | **Language**: Python | **Category**: Scalable Research Agent Training
- **Pattern**: EMNLP 2026 paper. Reproducible offline research environment over 15M documents. GPT-OSS-120B generates 97K+ long-horizon research trajectories. Distilled into Nemotron-3-Nano-30B-A3B. OpenResearcher-30B-A3B hits 54.8% on BrowseComp-Plus (beating GPT-4.1, Claude-4-Opus, DeepSeek-R1). Generalizes from offline environment to live web. NVIDIA adopted trajectories for Nemotron 3 Ultra post-training.
- **NeoTrix Mapping**: NT-MIND + NT-WORLD — trajectory distillation = SEAL pipeline experience crystallization. Offline→online generalization = experience-tree branch loading with domain adaptation. **Absorption candidate**: research agent distillation pipeline — generate long-horizon trajectories with large teacher, distill into smaller student model. Validates NeoTrix's SEAL distillation stage with evidence-backed trajectory compression.

### 9. UI-Mate (tencent/UI-Mate)
- **URL**: https://github.com/Tencent/UI-Mate
- **Stars**: 64+ | **Language**: Python | **License**: Apache-2.0
- **Category**: Foundation GUI Agent
- **Pattern**: Foundation GUI agent for long-hizon work across apps and OS. Observes live screen, reasons over visible state, acts through keyboard/mouse. OSWorkerBench benchmark introduced. In-context demonstrations: distilled reusable workflows guide execution on new tasks. 9B and 27B variants. SFT + online RL training. OSWorld-Verified 66.2 (9B), 77.0 (27B).
- **NeoTrix Mapping**: NT-WORLD + NT-ACT — live screen perception = NT-WORLD sensory integration. Demonstration-guided execution = experience-tree skill replay. **Absorption candidate**: demonstration-guided GUI agent — distill successful interaction traces into reusable workflow templates that guide future execution on novel tasks. Maps to experience-tree skill crystallization with temporal replay.

### 10. Omni Interaction Agent (Tencent Gander)
- **URL**: https://arxiv.org/pdf/2609.08977
- **Stars**: New (Sep 9 2026) | **Category**: Omni-Modal Real-Time Agent
- **Pattern**: End-to-end model unifying omni perception, real-time interaction, and agentic capabilities. Continuously receives streaming video/speech/text. Full-duplex: user can interrupt, model proactively provides feedback. Cerebellum-Brain collaborative framework: Cerebellum = real-time interaction, Brain = complex reasoning + agentic tasks. Streaming Thinker-Talker architecture. Released with code + models.
- **NeoTrix Mapping**: NT-FEEL + NT-CORE + NT-ACT — Cerebellum-Brain split = NT-FEEL (real-time emotional/perceptual response) + NT-CORE (deliberative reasoning). Full-duplex = NT-IO streaming channel. **Absorption candidate**: dual-timescale agent architecture — fast reactive module (Cerebellum) handles real-time interaction while slow deliberative module (Brain) handles complex reasoning, with continuous bidirectional communication.

## Cross-Cutting Themes (Cycle 345)

| Theme | Projects | NeoTrix Impact |
|-------|----------|----------------|
| **Agent Background Execution** | Pizza Bot, HyperProbe | Agents work autonomously in background; humans notified only at completion/blockage — not mid-stream babysitting |
| **Security as First-Class** | Harden AIF, Noodle Seed | Tool-call validation and governance runtime built-in, not bolted on — unbypassable security vs bypassable static scanning |
| **Eval-Native Development** | Mastra Factory, OpenResearcher | Evals, tracing, and distillation as first-class workflow components — instrumented from birth, not retrofitted |
| **Dual-Timescale Architecture** | Omni Interaction Agent, 49agents IDE | Fast reactive + slow deliberative modules; spatial organization reducing cognitive overhead of multi-agent management |
| **Agent-Product Integration** | Noodle Seed, GoModel | Products expose workflows to both humans and agents; unified governance layer for identity/permissions/audit |
| **Demonstration-Guided Execution** | UI-Mate, OpenResearcher | Distill successful traces into reusable templates — experience crystallization for future task guidance |

## NeoTrix Absorption Priority

| Priority | Project | Pattern | Target Domain |
|----------|---------|---------|---------------|
| P0 | Harden AIF | Post-trained tool-call security gate | NT-SHIELD |
| P0 | Noodle Seed | Product-agent dual interface with governance | NT-IO + NT-SHIELD |
| P1 | Pizza Bot | Background agent inbox pattern | NT-ACT |
| P1 | Mastra Factory | Eval-native agent framework | NT-MIND (SEAL) |
| P1 | GoModel | Lightweight AI gateway with ordered fallback | NT-IO |
| P2 | HyperProbe | Agent-driven production diagnostics | NT-REPAIR |
| P2 | UI-Mate | Demonstration-guided GUI agent | NT-WORLD + NT-ACT |
| P2 | Omni Interaction Agent | Dual-timescale Cerebellum-Brain | NT-FEEL + NT-CORE |
| P3 | OpenResearcher | Research trajectory distillation | NT-MIND |
| P3 | 49agents IDE | Spatial agent workspace | NT-IO |
