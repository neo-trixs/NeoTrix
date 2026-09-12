# Trending Rankings — Cycle 422 (2026-09-12)

## Search Window: September 2026

### 1. HydraFusion (GitHub Copilot Research Preview)
- **URL**: https://github.blog/ai-and-mxl
- **Category**: Coding Agent / Selective Workflow
- **Key Pattern**: Selective coding workflows that match or exceed Opus 5 baseline while reducing estimated workflow cost. Routes between coding strategies dynamically based on task difficulty.
- **NeoTrix Relevance**: Directly validates **Cost-Aware Routing (A1)** — different tasks get different model tiers. Aligns with GWT salience + cost weight routing.
- **Stars**: Part of GitHub Copilot ecosystem (100M+ users)
- **Novel Signal**: Hybrid strategy selection — not just model routing, but workflow routing.

### 2. Mozaik
- **URL**: https://github.com/jigjoy-ai/mozaik
- **Category**: Agent Runtime / TypeScript
- **Key Pattern**: TypeScript runtime for interoperable AI agents. Concurrent agent execution with cross-harness compatibility.
- **NeoTrix Relevance**: Maps to **NT-ACT** orchestration. Interoperable agent runtime parallels NT's multi-agent coordination patterns. Validates Isolation-per-Task (P2).
- **Stars**: Growing, featured on ProductHunt LLM tools
- **Novel Signal**: Agent interoperability standard — agents from different frameworks running in same runtime.

### 3. IQ Routing
- **URL**: ProductHunt launch Sep 2026
- **Category**: LLM Routing / Cost Optimization
- **Key Pattern**: Trajectory-aware LLM routing that cuts agent cost. Routes based on execution trajectory, not just current prompt.
- **NeoTrix Relevance**: Extends **GWT** attention routing with trajectory awareness. Validates A1 (Cost-Aware Routing) at the trajectory level — not just per-turn, but per-execution-path.
- **Novel Signal**: Trajectory-level routing (vs turn-level). First product-grade implementation.

### 4. Reflexio
- **URL**: ProductHunt #2 of the day Sep 5, 2026
- **Category**: Agent Learning / Behavioral Adaptation
- **Key Pattern**: Behavioral learning that makes AI agents better over time. Agents learn from interaction patterns and improve coordination.
- **NeoTrix Relevance**: Directly maps to **SEAL pipeline** (self-evolving architecture loop). Behavioral learning = experience crystallization. Validates ConsciousnessTree feedback loop.
- **Novel Signal**: Production behavioral learning — not just prompt improvement, but runtime behavioral adaptation.

### 5. oMLX
- **URL**: ProductHunt launch Sep 2026
- **Category**: Local LLM Inference / Mac
- **Key Pattern**: Mac LLM server that cuts agent wait times from 90s to 5s. Optimized for Apple Silicon with model caching and speculative prefill.
- **NeoTrix Relevance**: Maps to **NT-PHYSICAL** (edge inference) + **NT-IO** (local provider). Validates Context as Scarce Resource (A2) — local inference as context preservation strategy.
- **Novel Signal**: Platform-specific inference optimization that dramatically reduces latency for local agents.

### 6. screenpipe (YC S26)
- **URL**: ProductHunt, 5.0 rating
- **Category**: Agent Memory / Screen Recording
- **Key Pattern**: AI that records your computer work to power agents. Continuous screen + audio capture for agent context.
- **NeoTrix Relevance**: Maps to **NT-MEMORY** (persistent context). Continuous capture as attention stream — connects to GWT's selective attention. Pattern P3 (Profile-Driven Adaptation) via persistent user context.
- **Novel Signal**: Ambient agent memory — not explicit memory commands, but continuous passive capture.

### 7. Cortex by SKYNETLAB
- **URL**: ProductHunt launch Sep 2026
- **Category**: Agent Memory / Selective Recall
- **Key Pattern**: The memory layer that decides what's worth remembering. MCP integration, selective memory retention.
- **NeoTrix Relevance**: Maps to **NT-MEMORY** + **GWT** attention. Selective memory = attention-gated retention. Validates A2 (Context as Scarce Resource) — memory as scarce resource requiring intelligent selection.
- **Novel Signal**: Explicit "worth remembering" decision — not just storage, but salience-filtered memory.

### 8. Zero (Vercel)
- **URL**: ProductHunt launch Aug 2026
- **Category**: Agent-Native Language
- **Key Pattern**: Programming language built for AI agents. Designed from scratch for agent workflows, not retrofitted.
- **NeoTrix Relevance**: Validates **Skill as Production Template (A3)** at language level. Agent-native language = structured skill representation. Maps to NT-ACT tool calling patterns.
- **Novel Signal**: First mainstream programming language designed specifically for agent consumption.

### 9. Agnost AI
- **URL**: ProductHunt #3 of the day Aug 25, 2026
- **Category**: Agent Observability / Failure Detection
- **Key Pattern**: Catch agent failures your evals miss. Runtime failure detection beyond static evaluation.
- **NeoTrix Relevance**: Maps to **NT-SHIELD** (safety/audit) + **NT-META** (self-monitoring). Runtime failure detection = ConsciousnessTree health monitoring. Validates D26-D30 production readiness dimensions.
- **Novel Signal**: Gap between eval-time and runtime failure modes — catches what benchmarks miss.

### 10. HarnessRouter Community Edition
- **URL**: ProductHunt Aug 16, 2026
- **Category**: Agent Harness / Unified Interface
- **Key Pattern**: Open-source unified interface for agent harnesses. Single interface across Claude Code, Codex, Cursor, etc.
- **NeoTrix Relevance**: Maps to **NT-IO** (unified interface) + **Ordered Backend Router** (P4). Cross-harness routing validates Pattern P4 fallback chain at the harness level.
- **Novel Signal**: Harness-level abstraction — treating different coding agents as interchangeable backends.

---

## Trending GitHub Repos (Sep 2026)

| # | Repo | Stars/day | Pattern |
|---|------|-----------|---------|
| 1 | tt-a1i/archify | 4562 | Agent skill for verifiable architecture diagrams |
| 2 | K-Dense-AI/scientific-agent-skills | 720 | Turn any agent into an AI Scientist |
| 3 | calesthio/OpenMontage | 1144 | Agentic video production system, 700+ agent skills |
| 4 | JetBrains/go-modern-guidelines | 574 | Help AI coding agents write modern Go |
| 5 | rohitg00/ai-engineering-from-scratch | 703 | Learn it. Build it. Ship it. |

## Cross-Cycle Pattern Summary (422)

| Pattern | Count | Trend |
|---------|-------|-------|
| Cost-Aware Routing (A1) | 4/10 | ↑ Trajectory-level routing emerging |
| Context as Scarce Resource (A2) | 3/10 | ↑ Ambient memory + selective recall |
| Skill as Template (A3) | 2/10 | → Agent-native languages appearing |
| Isolation-per-Task (P2) | 2/10 | → Concurrent agent runtimes maturing |
| Ordered Backend Fallback (P4) | 2/10 | ↑ Harness-level abstraction new |
| Self-Evolution (SEAL) | 2/10 | ↑ Behavioral learning going production |

## Key Insight: The Agent Harness Abstraction Layer

Cycle 422 reveals a new abstraction layer emerging: the **agent harness**. Tools like HarnessRouter, Mozaik, and Zero treat different AI coding agents (Claude Code, Codex, Cursor) as interchangeable backends behind a unified interface. This parallels the Ordered Backend Router pattern (P4) but at the agent level rather than the search/backend level.

**NeoTrix Implication**: NT-IO should consider a harness abstraction layer that routes tasks across different agent harnesses, not just LLM providers. This would enable NT-ACT to orchestrate across Claude Code, Codex, Cursor, and future agents behind a single interface.
