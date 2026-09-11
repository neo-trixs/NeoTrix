# Trending Rankings — Cycle 344

**Date**: 2026-09-11
**Sources**: GitHub Trending, ProductHunt (Sep 9-11 2026), ossinsight.io, arXiv

## 10 New Projects (Not in Cycles 318-343)

### 1. Kilo Code (Kilo-Org/kilocode)
- **URL**: https://github.com/Kilo-Org/kilocode
- **Stars**: 27.1K+ | **Language**: TypeScript | **License**: MIT
- **Category**: All-in-One Agentic Engineering Platform
- **Pattern**: #1 on OpenRouter. 1.5M+ users, 25T+ tokens processed. Unified agentic coding across VS Code, JetBrains, CLI. Native MCP integration, multi-model routing, context-aware editing. Hot-reload skill system with community marketplace. Built-in cost tracking per task. Full agent harness with file access, terminal, browser.
- **NeoTrix Mapping**: NT-ACT + NT-IO — unified agentic platform = capability-as-platform pattern. Multi-model routing = GWT cost-aware routing (Axiom A1). Community skill marketplace = SEAL pipeline skill registry. **Absorption candidate**: agentic platform as single entry point — decouple agent reasoning from IDE/terminal/browser execution contexts while maintaining unified state.

### 2. Ouroboros (razzant/ouroboros)
- **URL**: https://github.com/razzant/ouroboros
- **Stars**: 1.2K+ | **Language**: Python | **License**: MIT
- **Category**: Self-Evolving Frontier Coding Agent
- **Pattern**: Self-creating agent born Feb 16, 2026. Rewrites its own code, architecture, prompts, tools, and dependencies. Identity persists across restarts. Coordinates live swarm of specialist agents. 161-day Hope deployment with 32 evolution cycles in first 48 hours. Terminal-Bench 2.1 and OSWorld-Verified benchmarks. Reviewed core-evolution system — changes pass human review before application.
- **NeoTrix Mapping**: NT-MIND + NT-REPAIR — self-rewriting = SEAL pipeline with human-reviewed evolution. Identity persistence = NT-NEXUS cross-session memory. Swarm coordination = EventBus multi-domain orchestration. **Absorption candidate**: reviewed core-evolution — agent modifications pass review gates before application, preventing drift. Maps to ConsciousnessTree Fruits→Core phase with governance check.

### 3. Letta (letta-ai/letta)
- **URL**: https://github.com/letta-ai/letta
- **Stars**: 15K+ | **Language**: Python | **License**: Apache 2.0
- **Category**: Memory-as-Operating-System Agent Framework
- **Pattern**: Formerly MemGPT. Treats memory like an OS: main context = RAM, archival memory = disk. Agent decides what to promote, archive, or forget. Model-agnostic, self-hosted. Production-grade memory management with hierarchical tiers. Memory blocks as persistent state across sessions.
- **NeoTrix Mapping**: NT-MEMORY + NT-CORE — memory-as-OS = experience-tree hierarchical memory (L0 instinct → L5 identity). Promotion/archive/forget = ConsciousnessTree consolidation cycle. **Absorption candidate**: OS-style memory hierarchy — agent-controlled memory promotion between volatile (context) and persistent (KB) tiers, with explicit forget operations for memory hygiene.

### 4. Trace MCP (nikolai-vysotskyi/trace-mcp)
- **URL**: https://github.com/nikolai-vysotskyi/trace-mcp
- **Stars**: 170+ | **Language**: TypeScript | **License**: MIT
- **Category**: Framework-Aware Code Intelligence MCP Server
- **Pattern**: 70.5% fewer input tokens to review a pull request. Framework-aware: understands React, Next.js, Vue, Svelte, Django, FastAPI code structures. Semantic code graph for dependency analysis. Smart context injection — only relevant code paths sent to LLM. Works with Claude Code and Codex.
- **NeoTrix Mapping**: NT-WORLD + NT-IO — framework-aware code perception = NT-WORLD domain-specific parsers. Token reduction via semantic filtering = Axiom A2 (Context as Scarce Resource). **Absorption candidate**: framework-aware code intelligence — understanding framework-specific patterns (React hooks, Next.js routing) to inject only semantically relevant code context, not raw file dumps.

### 5. Skydive (skydive.ai)
- **URL**: https://skydive.ai
- **Stars**: ProductHunt #1 (Aug 27) | **Category**: Cloud Agent Infrastructure
- **Pattern**: Build cloud agents that work across your tools. Agents run in persistent cloud environments with filesystem, terminal, browser access. Cross-tool integration (GitHub, Linear, Slack, Notion). Session persistence — agents resume where they left off. Team collaboration with shared agent workspaces.
- **NeoTrix Mapping**: NT-ACT + NT-IO — persistent cloud agent environment = NT-ACT execution substrate. Cross-tool integration = NT-IO protocol adapter pattern. Session persistence = NT-NEXUS cross-session state. **Absorption candidate**: persistent agent environment — agents maintain stateful workspaces across sessions, not stateless request-response cycles.

### 6. HarnessRouter Community Edition
- **URL**: ProductHunt (Aug 16)
- **Stars**: PH Featured | **Category**: Unified Agent Harness Interface
- **Pattern**: Open-source unified interface for agent harnesses. Route between Claude Code, Codex, Gemini CLI, OpenCode from single interface. Switch mid-conversation without losing context. Cost comparison dashboard. Skill portability across harnesses.
- **NeoTrix Mapping**: NT-IO — unified harness interface = NT-IO provider abstraction layer. Context preservation across switches = session-level state management. **Absorption candidate**: harness-agnostic agent execution — single interface routing to multiple agent backends with context portability.

### 7. inferock Bench
- **URL**: ProductHunt (Aug 15)
- **Stars**: PH Featured | **Category**: Independent LLM API Verification
- **Pattern**: Independent receipt for every LLM API call. Verify model actually ran, tokens consumed, latency accurate. Anti-fraud: detect model substitution ( cheap model pretending as expensive). Cost audit trail for enterprise compliance.
- **NeoTrix Mapping**: NT-SHIELD + NT-IO — API call verification = NT-SHIELD audit trail. Model substitution detection = provider trust verification. **Absorption candidate**: independent LLM call verification — cryptographically verifiable receipts for model API calls, preventing silent model downgrade.

### 8. Sushidata (sushidata.com)
- **URL**: ProductHunt | **Stars**: PH Featured | **Category**: Agent Token Compounding API
- **Pattern**: APIs for swarms of agents that compound data and save tokens. Agent output feeds into next agent's context with automatic deduplication. Shared memory pool across agent swarms. Token amortization — repeated patterns cached and reused across agents.
- **NeoTrix Mapping**: NT-MEMORY + NT-ACT — token compounding = EventBus data flow optimization. Shared memory pool = KB namespace with cross-agent access. Deduplication = experience-tree entry deduplication. **Absorption candidate**: token compounding API — automatic deduplication and caching of repeated patterns across agent swarm interactions.

### 9. session-indexer
- **URL**: ProductHunt (Aug 23) | **Stars**: 91+ | **Category**: Semantic Search Over Agent Session History
- **Pattern**: Semantic search over your own Claude Code session history. Index all past sessions, query by intent not keyword. Find prior solutions to similar problems. Context retrieval for long-running projects.
- **NeoTrix Mapping**: NT-MEMORY — session history indexing = experience-tree hub index with semantic retrieval. Intent-based query = KB keyword search with embedding support. **Absorption candidate**: semantic session indexing — automatic indexing of agent session transcripts for retrieval by intent, enabling cross-session learning.

### 10. Buddy Visual Tests
- **URL**: ProductHunt (Aug 23) | **Stars**: PH Featured | **Category**: UI Change Review Before Merge
- **Pattern**: Every UI change, reviewed before merge. Automated visual regression detection. AI-powered screenshot comparison with semantic understanding (not just pixel diff). PR-integrated — visual review as part of code review workflow.
- **NeoTrix Mapping**: NT-SHIELD + NT-WORLD — visual regression = SelfTest T3 production wiring for UI. Semantic comparison = NT-WORLD perception beyond pixel-level. **Absorption candidate**: semantic visual regression — AI-understanding of UI changes, not just pixel diffs, integrated into agent code review workflows.

## Cross-Cutting Themes (Cycle 344)

| Theme | Projects | NeoTrix Impact |
|-------|----------|----------------|
| **Memory as OS** | Letta, Sushidata | Agent-controlled memory hierarchy with promotion/archive/forget — not flat context windows |
| **Self-Evolution with Review Gates** | Ouroboros | Agent modifications pass human review before application — prevents drift while enabling growth |
| **Harness-Agnostic Execution** | HarnessRouter, Kilo Code | Single interface routing to multiple agent backends — context portability across providers |
| **Token Cost Optimization at Every Layer** | Trace MCP, Sushidata, inferock | Compression (70% reduction), compounding (dedup), and verification (anti-fraud) — cost awareness throughout |
| **Persistent Agent Environments** | Skydive, session-indexer | Stateful workspaces across sessions + semantic retrieval of past work — continuity over statelessness |
| **Framework-Aware Intelligence** | Trace MCP, Buddy Visual Tests | Understanding framework-specific patterns for smarter context injection — domain knowledge in perception |

## NeoTrix Absorption Priority

| Priority | Pattern | Source | Target Domain |
|----------|---------|--------|---------------|
| P0 | OS-style memory hierarchy with agent-controlled promotion | Letta | NT-MEMORY (memory tiers) |
| P0 | Reviewed core-evolution — changes pass review before application | Ouroboros | NT-MIND (SEAL pipeline) |
| P1 | Framework-aware code intelligence for token reduction | Trace MCP | NT-WORLD (code perception) |
| P1 | Token compounding — dedup and cache across agent interactions | Sushidata | NT-MEMORY ( EventBus optimization) |
| P1 | Harness-agnostic execution with context portability | HarnessRouter | NT-IO (provider abstraction) |
| P2 | Semantic session indexing for cross-session learning | session-indexer | NT-MEMORY (experience retrieval) |
| P2 | Independent LLM call verification receipts | inferock | NT-SHIELD (audit trail) |
| P2 | Persistent agent workspaces across sessions | Skydive | NT-ACT (execution substrate) |
| P3 | Semantic visual regression for UI review | Buddy Visual Tests | NT-SHIELD (quality gate) |
