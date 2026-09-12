# Trending Rankings — Cycle 382 (2026-09-12)

## 10 New Projects (Not in Cycles 318-381)

| # | Project | Stars | Category | NeoTrix Domain | Key Pattern |
|---|---------|-------|----------|----------------|-------------|
| 1 | **PrimeAgent** | 15.9K | Self-improving coding agent | NT-CORE / NT-MIND | Recursive Language Model (RLM) — prompt-as-variable + programmatic sub-agent calling in persistent REPL |
| 2 | **Taifeng** | — | Agent microkernel/scheduler | NT-CORE | Skills as SKILL.md, LLM as scheduler, cache-aware compaction, JSONL persistence |
| 3 | **KISS Sorcar** | — | Multi-model agent framework | NT-IO / NT-ACT | 641-model catalog, 9 providers, daemon-backed, natural-language cron, wake-word voice |
| 4 | **EVE** (Vercel) | 4.9K | Filesystem-first agent framework | NT-CORE / NT-ACT | agents/ directory convention, tools/skills/channels/schedules as files, sandbox isolation |
| 5 | **Hive/Aden** | — | Self-adaptive agent platform | NT-MIND / NT-REPAIR | Goal-driven graph generation, failure-capture → evolve → redeploy loop, SDK-wrapped nodes |
| 6 | **Scrapling** | 59.5K | Adaptive web scraping | NT-WORLD | Anti-detection, adaptive parsing, handles JS-rendered sites, anti-fingerprint |
| 7 | **Open-LLM-VTuber** | 8.6K | Voice-interactive local LLM | NT-IO / NT-FEEL | Hands-free voice, voice interruption, Live2D avatar, fully local |
| 8 | **Hermes WebUI** | 12.8K | Agent dashboard | NT-IO | Web/mobile interface for terminal-only agents, real-time visibility |
| 9 | **Supermemory** | 24.8K | Memory API for AI | NT-MEMORY | Drop-in persistent memory across sessions/users, no custom vector DB needed |
| 10 | **MarkItDown** (Microsoft) | — | Universal document converter | NT-WORLD / NT-MEMORY | PDF/Office/HTML → Markdown normalization for RAG pipelines |

## Deep Dives

### 1. PrimeAgent — Recursive Language Model Agent
- **Source**: PrimeIntellect-ai/prime-agent (GitHub)
- **Core Innovation**: RLM (Recursive Language Model) treats context as variables and tools as recursive sub-agent function calls inside a persistent Python REPL
- **Continual Harness**: Stores supplemental prompts, memories, skill descriptions as durable state; `/refine` applies evidence-backed updates
- **Key Features**: daemon-backed sessions, agent-to-agent direct messaging, persistent goals, heartbeat/schedule, bounded autonomous mode
- **NeoTrix Mapping**: Maps to NT-CORE (recursive reasoning) + NT-MIND (skill crystallization via `/refine`). The Continual Harness pattern is isomorphic to experience-tree KB absorption

### 2. Taifeng — Python Agent Microkernel
- **Source**: pypi.org/project/taifeng (v2026.9.6.0)
- **Core Innovation**: Skills are documented in SKILL.md, LLM is the scheduler. Business-decoupled kernel with explicit concurrency, cancellation, cache safety
- **Key Features**: composite dispatch with depth guards/cycle detection, cache-aware compaction, HITL permissions, JSONL transcripts, MCP integration
- **NeoTrix Mapping**: Maps to NT-CORE (agent kernel) + NT-ACT (skill dispatch). The cache-aware compaction strategy is directly relevant to KVMem integration

### 3. KISS Sorcar — Multi-Model Agent
- **Source**: pypi.org/project/kiss-agent-framework (v2026.9.9)
- **Core Innovation**: 641-model catalog across 9 providers, multiple models in same task, daemon-backed agents, AI discovery via prompt
- **Key Features**: VS Code extension + web/mobile + Python API, scheduled automations, wake word, 32 communication channels (Slack/Gmail/SMS/WhatsApp)
- **NeoTrix Mapping**: Maps to NT-IO (multi-provider routing) + NT-ACT (orchestration). The model catalog pattern supports cost-aware routing (Axiom A1)

### 4. EVE — Filesystem-First Agent Framework
- **Source**: vercel/eve (GitHub)
- **Core Innovation**: Core agent capabilities in conventional filesystem locations (agent/instructions.md, agent/tools/, agent/skills/, agent/channels/, agent/schedules/)
- **Key Features**: OpenAI-compatible model config, typed tool definitions, markdown skills loaded on demand, subagents, human-in-the-loop
- **NeoTrix Mapping**: Maps to NT-CORE (agent structure) + NT-ACT (tool/skill conventions). Filesystem-first aligns with NeoTrix's module-as-directory architecture

### 5. Hive/Aden — Self-Adaptive Agent Platform
- **Source**: semr9/hive (GitHub)
- **Core Innovation**: Goal-driven development — describe outcomes in NL, coding agent generates agent graph + connection code. Failure → capture → evolve → redeploy loop
- **Key Features**: SDK-wrapped nodes (shared memory, monitoring, tools), dynamic node connections, real-time observability, cost/budget control, self-hostable
- **NeoTrix Mapping**: Maps to NT-MIND (self-evolution via failure capture) + NT-REPAIR (auto-repair loop). The evolve-and-redeploy pattern extends SEAL pipeline concepts

### 6. Scrapling — Adaptive Web Scraping
- **Source**: Scrapling on GitHub (59.5K stars)
- **Core Innovation**: Adaptive scraping that handles anti-detection, JS-rendered sites, fingerprint bypass
- **Key Features**: Anti-detection by default, adaptive parsing, handles Cloudflare/anti-bot
- **NeoTrix Mapping**: Maps to NT-WORLD (perception/data acquisition). Complements UnifiedCrawler with production-grade anti-detection

### 7. Open-LLM-VTuber — Voice-Interactive Local LLM
- **Source**: Open-LLM-VTuber (GitHub)
- **Core Innovation**: Hands-free voice interaction with any LLM, voice interruption, Live2D avatar, fully local
- **Key Features**: Wake-word activation, real-time voice, no cloud dependency
- **NeoTrix Mapping**: Maps to NT-IO (voice interface) + NT-FEEL (emotional expression via avatar). Voice interruption is a novel interaction pattern for NT-FEEL regulation

### 8. Hermes WebUI — Agent Dashboard
- **Source**: Hermes WebUI (GitHub, 12.8K stars)
- **Core Innovation**: Web/mobile dashboard for terminal-only coding agents (Claude Code, Codex, etc.)
- **Key Features**: Real-time visibility, session management, multi-agent monitoring
- **NeoTrix Mapping**: Maps to NT-IO (interface layer). Provides the missing "agent observability dashboard" for NT-CORE/NT-MIND operations

### 9. Supermemory — Memory API for AI
- **Source**: Supermemory (GitHub, 24.8K stars)
- **Core Innovation**: Drop-in Memory API for persistent context across sessions, users, conversations — no custom vector DB infrastructure needed
- **Key Features**: Blazingly fast, scalable, session/user/conversation-scoped memory
- **NeoTrix Mapping**: Maps to NT-MEMORY (knowledge persistence). The session-scoped memory pattern aligns with NT-NEXUS cross-session weaving

### 10. MarkItDown — Universal Document Converter
- **Source**: Microsoft MarkItDown
- **Core Innovation**: Converts PDFs, Office docs, images, HTML to clean Markdown as standard RAG preprocessing
- **Key Features**: Consistent input format improves retrieval quality, by Microsoft
- **NeoTrix Mapping**: Maps to NT-WORLD (perception/document parsing) + NT-MEMORY (KB ingestion). Standardizes the document→embedding pipeline

## Cross-Cutting Patterns

| Pattern | Projects | NeoTrix Integration |
|---------|----------|---------------------|
| **Self-Evolution via Failure** | Hive/Aden, PrimeAgent `/refine` | NT-MIND SEAL loop extension |
| **Cache-Aware Compaction** | Taifeng, EVE | KVMem integration pattern |
| **Multi-Model Routing** | KISS Sorcar, 9Router (prev) | Cost-Aware Routing (A1) |
| **Filesystem-as-Interface** | EVE, Taifux | NT module directory conventions |
| **Voice as Agent Interface** | Open-LLM-VTuber | NT-FEEL + NT-IO integration |
| **Memory-as-API** | Supermemory, PrimeAgent Continual Harness | NT-MEMORY + NT-NEXUS |
| **Goal-Driven Graph Generation** | Hive/Aden | SEAL pipeline auto-topology |
| **Adaptive Anti-Detection** | Scrapling | NT-SHIELD + NT-WORLD |
