# Trending Rankings — Cycle 435

**Date:** 2026-09-12
**Focus:** AI agents, LLM tools, reasoning frameworks, memory, attention, routing
**Exclusion:** Projects covered in cycles 318–434

---

## 1. Kilo Code

- **GitHub:** Open-source (acquired by Anaconda, Sep 2026)
- **Category:** Agentic Engineering Platform
- **What:** All-in-one agentic coding agent across VS Code, JetBrains, and CLI. Ships with specialized agent modes: Code (implement/edit), Architect (plan), Ask (query), Debug (trace), Review (audit). Auto Model routing across 500+ models (GPT-5.5, Claude Opus 4.7, Gemini 3.1 Pro). Cloud Agents for remote execution. Agent Manager for parallel multi-agent sessions. Memory Bank for architectural decisions. MCP marketplace integration. Security Agent for vulnerability analysis with sandbox testing.
- **NeoTrix mapping:** NT-IO + NT-ACT. Multi-mode agent switching → NT-ACT's Ascendancy dual-weapon-set specialization (L5 Cognition routes between Architect/Code/Debug modes). Auto Model routing → Axiom A1 cost-aware routing with live budget optimization. Memory Bank → NT-MEMORY structured decision cache. Security Agent → NT-SHIELD pre-execution vulnerability scanning with sandbox isolation. Agent Manager parallel worktrees → NT-ACT's ParallelTaskManager with Git worktree isolation.
- **Pattern:** Mode-as-Context — each agent mode resets the system prompt, tools, and context window. Switching modes is a context switch, not a model switch. The same model operates differently based on mode. Emergent from OpenCode (CLI) + Cline/Roo Code (IDE) merger.
- **Novel:** Auto Model with three tiers (Efficient/Frontier/Free) — dynamically selects model tier based on task complexity and budget. Live pricing feedback loop. 5% processing fee on credit purchases, zero markup on provider pricing.

## 2. Mastra

- **GitHub:** 15.1K stars (TypeScript, from Gatsby creators)
- **Category:** TypeScript Agent Framework
- **What:** Open-source TypeScript framework for AI agents and applications. Typed agents with instructions, models, tools, and runtime behavior. Graph-based workflow engine (.then(), .branch(), .parallel()) with human-in-the-loop suspension. Memory system with conversation history, data retrieval from APIs/databases/files, and semantic memory. Built-in scorers and observability. 90+ provider support via one standard interface. Deploy to Vercel/Netlify/Cloudflare or standalone Hono server. Workspace management for multi-agent coordination. @mastra/mem0 first-party integration for persistent memory.
- **NeoTrix mapping:** NT-IO + NT-MEMORY. Typed agents → NT-IO's LLM provider abstraction (typed tool signatures like PTC). Graph-based workflows → SEAL pipeline's stage composition with explicit control flow. Memory system → NT-MEMORY's KB embedding with conversation + entity + semantic layers. Observability → NT-CORE HeartbeatAggregator tracing. 90+ provider routing → Axiom A1 cost-aware model routing. Workspace coordination → NT-NEXUS cross-agent shared state.
- **Pattern:** Framework-as-Runtime — not just agent definition, but complete execution lifecycle: define → compose → execute → observe → improve. TypeScript-first with strict typing throughout. Human-in-the-loop as first-class workflow primitive (suspend/resume with persistent state).
- **Novel:** Memory as two tools pattern: `Mem0-memorize` (write) and `Mem0-remember` (read) — memory operations are tool calls, not side effects. Async memory writes to avoid blocking response generation. Observability traces agent calls and token usage natively.

## 3. Hermes Agent (NousResearch)

- **GitHub:** 13.5K+ stars (MIT)
- **Category:** Self-Improving Multi-Platform Agent
- **What:** Self-improving AI agent with built-in learning loop. Creates skills from experience, improves them during use, nudges itself to persist knowledge. FTS5 session search with LLM summarization for cross-session recall. Honcho dialectic user modeling. 12 messaging platform adapters (Telegram, Discord, Slack, WhatsApp, Signal, Matrix, Home Assistant, Email, SMS, IRC, Webhook, REST API). 6 execution backends (local, Docker, SSH, Singularity, Modal, Daytona). Scheduled automations via built-in cron. Subagent spawning for parallel work. Research-ready batch trajectory generation.
- **NeoTrix mapping:** NT-MIND + NT-MEMORY + NT-ACT. Self-improving skills → NT-MIND's SEAL pipeline skill crystallization. FTS5 cross-session search → NT-MEMORY's KB FTS5 search. Honcho dialectic user modeling → NT-CORE's SelfModel dynamic profile. 12 platform adapters → NT-IO's protocol layer (MCP, ACP, LSP). 6 execution backends → NT-SHIELD's sandbox backend abstraction (local/Docker/Modal/Daytona). Scheduled automations → NT-ACT's cron-like scheduled task execution. Subagent spawning → NT-ACT's worker pool.
- **Pattern:** Agent-as-Server — always-on daemon process, not a CLI tool. Maintains state across sessions. Message gateway bridges platforms to a single agent instance. Skills are procedural memory that the agent writes and reuses. Learning loop: execute → extract skill → persist → reuse → improve.
- **Novel:** Transport ABC architecture (AnthropicTransport, ChatCompletionsTransport, ResponsesApiTransport, BedrockTransport) — each provider owns its own format conversion and API shape. Native AWS Bedrock via Converse API. Plugin surface with register_command, dispatch_tool, pre_tool_call blocking, transform_tool_result.

## 4. Firecrawl Web Agent

- **GitHub:** 1.2K stars (MIT, from Firecrawl)
- **Category:** Web Data Agent
- **What:** Open-source web data agent optimized for structured web research. Built on Deep Agents (LangChain) harness with plan-act-observe loop. Skills, Subagents, structured output. Firecrawl tools: Search, Scrape, Interact (browser automation), Bash. Browser Sandbox: remote Playwright + agent-browser (60+ CLI commands). Persistent or temporary sessions. CDP access for local control. Interactive Live View for debugging. Spark 2 model (cheaper/faster than Spark 1). Effort levels (low/medium/high). Webhook lifecycle events. Execution traces with snapshots.
- **NeoTrix mapping:** NT-WORLD + NT-SHIELD. Browser Sandbox → NT-WORLD's UnifiedCrawler headless browser pool with session isolation. Skills-as-playbooks → NT-WORLD's crawl pipeline skill-based extraction. Subagents for parallel crawling → NT-ACT's ParallelTaskManager. Structured output → NT-MEMORY's KB embedding pipeline (raw → structured → stored). Persistent browser sessions → NT-SHIELD's sandbox session management. CDP access → NT-PHYSICAL's sensor abstraction (browser as sensory input).
- **Pattern:** Scrape-then-Interact — two-phase web data: first static scrape (cheap, fast), then interactive browser session for dynamic content (forms, pagination, auth). Agent-browser as bash-first CLI instead of Playwright code. 60+ commands replace complex scripting.
- **Novel:** agent-browser CLI as agent-native interface — agents write simple bash commands (`agent-browser snapshot`, `agent-browser click "button"`) instead of Playwright scripts. Auto-prefixed `--cdp` flag connects to active session automatically. Persistent sessions maintain auth state across runs.

## 5. Mem0

- **GitHub:** 52K+ stars
- **Category:** Universal Memory Layer for AI Agents
- **What:** Drop-in memory infrastructure. Single-pass ADD-only extraction: memories accumulate, nothing overwritten. Agent-generated facts as first-class. Entity linking with own embedding (not shared with fact embedding). Multi-signal retrieval: semantic + BM25 keyword + entity matching scored in parallel and fused. Temporal reasoning for time-aware queries. Four-scope memory model: user_id, agent_id, run_id/session_id, app_id/org_id. 21 framework integrations. OpenMemory MCP-compatible local memory server. Benchmark: 92.5 LoCoMo, 94.4 LongMemEval, 64.1 BEAM (1M tokens).
- **NeoTrix mapping:** NT-MEMORY. Four-scope memory → KB's namespace isolation (domain_nt_* namespaces per domain). Multi-signal retrieval → NT-MEMORY's hybrid search (vector + BM25 + entity). Entity linking → KB's node-edge graph with entity embeddings. Temporal reasoning → experience-tree's cycle-aware experience versioning. ADD-only extraction → KB append-only event log pattern. Agent-generated facts → SEAL pipeline's experience capture. OpenMemory MCP server → NT-IO's MCP protocol integration.
- **Pattern:** Memory-as-Service — pluggable layer that bolt onto any agent framework. No runtime commitment. API boundary: add(), search(), update(), delete(). Memory extraction is automatic, not agent-decided. Trade: predictability vs intelligence (Mem0 extracts passively; Letta agents self-edit).
- **Novel:** Token-efficient new algorithm (Apr 2026): hierarchical distillation + multi-signal retrieval. +29.6 points on temporal queries, +23.1 on multi-hop reasoning. Single-pass ADD-only (no UPDATE/DELETE) simplifies deduplication. 97% memory footprint reduction reported for Claude Code integration.

## 6. GitAgent

- **GitHub:** 670 stars (MIT, early)
- **Category:** Git-Native AI Agent Framework
- **What:** Agent IS a git repository. agent.yaml (model/tools/runtime), SOUL.md (personality), RULES.md (behavioral constraints), memory/ (git-committed memory with full history), tools/ (declarative YAML), skills/ (composable modules), hooks/ (lifecycle hooks). SDK mirrors Claude Agent SDK but runs in-process — no subprocesses, no IPC. MCP client for automatic tool discovery. Multi-model support via pi-ai. OpenTelemetry instrumentation (gen_ai.system spans, tool.execute spans, cost_usd metrics). Git history IS memory version control.
- **NeoTrix mapping:** NT-MEMORY + NT-CORE. Git-as-memory-versioning → KB's node versioning with git-like commit history. SOUL.md personality → NT-CORE's SelfModel identity (EmotionLabel + personality traits). RULES.md constraints → NT-SHIELD's governance rules (gov-steward). Declarative YAML tools → NT-ACT's capability definition (Skill node system). Hooks lifecycle → ConsciousnessTree's 6-stage feedback loop (Soil→Roots→Trunk→Branches→Fruits→Core). In-process SDK → NT-IO's in-process tool invocation (no subprocess overhead).
- **Pattern:** Agent-as-Repo — the entire agent (identity, rules, memory, tools, skills) is a version-controlled repository. Git history provides natural memory versioning, rollback, and audit trail. Changes to the agent are commits. Team collaboration on agents is git collaboration.
- **Novel:** Memory commits — every memory write is a git commit with full diff history. Browse memory evolution over time. Rollback to any previous memory state. Memory conflicts resolved via merge. SOUL.md/RULES.md are living documents that evolve with the agent.

## 7. Daytona

- **Category:** Secure Sandbox Infrastructure for AI Code Execution
- **What:** Secure and elastic infrastructure for AI-generated code execution. Isolated sandboxes with own filesystem, network stack, resources. Programmatic controls via SDKs, APIs, and CLI. Serverless persistence (hibernate when idle, wake on demand). Used by Hermes Agent, DeerFlow, and others as execution backend. Ephemeral or persistent sessions. Multi-language support (Python, Node, Bash).
- **NeoTrix mapping:** NT-SHIELD + NT-PHYSICAL. Sandbox isolation → NT-SHIELD's Egress Policy per-sandbox trust boundary. Serverless persistence → NT-PHYSICAL's power management (idle hibernation, on-demand wake). Filesystem isolation → NT-SHIELD's PathValidator boundary enforcement. Network stack isolation → NT-SHIELD's egress privacy guard network tiers. Programmatic SDK → NT-IO's tool abstraction layer.
- **Pattern:** Execution-as-a-Service — sandboxes as cloud primitives. No local Chromium, no driver management. Each sandbox is disposable or persistent. Cost model: pay for active compute, free during hibernation. Abstracts infrastructure complexity behind simple API.
- **Novel:** Serverless agent execution — sandbox hibernates when idle (no compute cost), wakes on demand with full state preserved. Enables always-on agents on minimal budget. Complementary to Modal for GPU workloads vs CPU workloads.

## 8. Oqoqo

- **Category:** Agent Evals and Custom Benchmarks Platform
- **What:** Build evals and custom benchmarks for real-world tasks. Spins up isolated sandboxes, executes tasks against chosen agents (Codex, Claude Code, OpenClaw, Hermes, Pi, OpenCode, Cursor, GitHub Copilot). Catalogs every step including tool calls, retries, discovery loops. Documents token consumption and cost. Evaluates success/failure based on custom criteria. Dynamic insights detect frictions in product interfaces or token inefficiencies. Regression tests MCP, CLI, skills, SDK, agent-facing interfaces. Custom benchmarks for agent product discovery.
- **NeoTrix mapping:** NT-MIND + NT-CORE. Agent benchmarking → SEAL pipeline's SelfTest T1/T2/T3 tier validation. Sandbox execution → NT-SHIELD isolated test environments. Token consumption tracking → Axiom A1 cost-aware routing calibration. Dynamic insights → ConsciousnessTree's cross-domain health scoring. Regression testing → NT-MIND's skill crystallization quality gates. Custom criteria → NT-CORE's E8 hexagram evaluation (multiple criteria converge).
- **Pattern:** Evals-as-Infrastructure — benchmarks are first-class products, not research artifacts. Agent-agnostic: test any agent against same criteria. Sandboxed execution prevents test contamination. Cost and token tracking embedded in every evaluation run.
- **Novel:** Agent product discovery benchmarks — not just "can the agent solve the task?" but "can the agent discover and use your product's features?" Tests MCP endpoints, CLI tools, SDK interfaces, and skills. Dynamic insights detect where agents get stuck.

## 9. NanoBot (HKUDS)

- **GitHub:** 47.6K stars (MIT)
- **Category:** Ultra-Lightweight Personal AI Agent
- **What:** Ultra-lightweight self-hosted personal AI agent in Python. WebUI, terminal, or chat apps. Tools: files, shell, web search, web fetch, MCP, cron, image generation, subagents. Session history + long-term memory via Dream. Long-horizon goals and scheduled automations. Python SDK + OpenAI-compatible API. Deploy as local or server-side agent gateway. Chat apps: Telegram, Discord, Slack, WeChat, Email, Mattermost, QQ, Feishu. Model routing with fallbacks. Small readable core.
- **NeoTrix mapping:** NT-IO + NT-MEMORY. Ultra-lightweight → NT-IO's minimal protocol implementation (no framework overhead). Dream memory system → NT-MEMORY's KB embedding with tiered retrieval (recent + semantic + long-term). OpenAI-compatible API → NT-IO's provider abstraction. Multi-platform chat → NT-IO's protocol layer (MCP/ACP). Cron scheduling → NT-ACT's scheduled task execution. Subagent spawning → NT-ACT's worker pool delegation. Small readable core → NeoTrix's C0 compilation target (minimal viable system).
- **Pattern:** Agent-as-Gateway — single process bridges multiple chat platforms to one agent instance. OpenAI-compatible API enables integration with any ecosystem. "Small core, extended by plugins" philosophy. Memory via Dream (file-based, self-contained).
- **Novel:** Dream memory — hierarchical memory system that processes session history into long-term memory autonomously. Not just vector search, but structured memory consolidation. v0.3.0 "Agency Release" added subagents, model switching per session, and autonomous work completion.

## 10. Glassbrain

- **ProductHunt:** Launched 2026
- **Category:** Visual Trace Replay for AI Apps
- **What:** Captures every step of AI app as interactive visual trace tree. Click any node, swap input, replay instantly without redeploying. Snapshot mode (deterministic replays) + Live mode (hits actual stack). Auto-generated fix suggestions reference exact trace data with one-click copy. Diff view shows what changed. Shareable replay links for team debugging. Works with OpenAI and Anthropic. Two lines of code to integrate. Free tier: 1K traces/month.
- **NeoTrix mapping:** NT-CORE + NT-REPAIR. Visual trace tree → ConsciousnessTree's growth visualization (6-stage feedback loop rendered as tree). Trace replay → NT-REPAIR's self-healing loop (replay failure, identify root cause, apply fix). Snapshot vs Live mode → NT-CORE's Phi computation (deterministic state vs dynamic computation). Diff view → NT-MIND's evolution tracking (what changed between iterations). Team debugging → NT-NEXUS cross-session knowledge sharing.
- **Pattern:** Trace-as-UI — not logging, not monitoring, but interactive trace visualization that doubles as a debugging interface. Every LLM call, tool call, and retrieval becomes a node in a tree you can click, modify, and replay.
- **Novel:** Two-line integration SDK wraps Anthropic/OpenAI client — every call from every step captured automatically. Single completion or 15-node agent pipeline with retrieval, tool calls, and nested LLM calls — each node shows inputs, outputs, latency, and tokens. Replay without redeployment.

---

## Cross-Cutting Patterns

| Pattern | Projects | NeoTrix Mapping |
|---------|----------|-----------------|
| **Agent-as-Server** | Hermes, NanoBot, GitAgent | NT-IO always-on daemon with message gateway |
| **Memory-as-Service** | Mem0, GitAgent, NanoBot | NT-MEMORY pluggable retrieval layer |
| **Mode-as-Context** | Kilo Code, Hermes | NT-ACT dual-weapon-set mode switching |
| **Execution-as-a-Service** | Daytona, Firecrawl, Oqoqo | NT-SHIELD sandbox isolation primitives |
| **Trace-as-UI** | Glassbrain, Oqoqo | NT-CORE ConsciousnessTree visualization |
| **Agent-as-Repo** | GitAgent, NanoBot | NT-MEMORY git-versioned memory state |
| **Evals-as-Infrastructure** | Oqoqo, Kilo Code | NT-MIND SelfTest quality gate integration |

## Key Insight

The agent ecosystem is bifurcating into two tiers:
1. **Harness tier** (DeerFlow, Kilo Code, Hermes, NanoBot) — complete execution environments with sandbox, memory, tools, and message bus. "Batteries included."
2. **Infrastructure tier** (Mem0, Daytona, Firecrawl, Glassbrain, Oqoqo) — pluggable services that harnesses consume. "Composable building blocks."

NeoTrix's architecture (6-layer with NT-IO as protocol layer + NT-SHIELD as security layer) naturally fits the infrastructure tier. The trend confirms that harnesses are becoming commodity — the value is in the infrastructure primitives (memory, sandbox, eval, trace) that harnesses compose.
