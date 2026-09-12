# Trending Rankings — Cycle 431

**Date**: 2026-09-12  
**Scope**: New AI/developer tools trending Sep 2026, not in cycles 318–430

---

## Top 10 Trending Projects

### 1. Vercel Eve — Filesystem-First Agent Framework
- **URL**: https://github.com/vercel/eve (4,957 ★)
- **Category**: Agent Framework
- **Key Innovation**: Agents live in conventional filesystem locations — `instructions.md`, `tools/`, `skills/`, `channels/`, `schedules/`. Projects are inspectable, extensible, operable without special tooling.
- **NeoTrix Relevance**: NT-CORE. Filesystem-as-agent-state mirrors NT-MEMORY's persistent node/edge storage. Validate whether NT-CORE could adopt a "conventional location" pattern for agent configurations (e.g., `neotrix.toml` as single truth source like Eve's `agent.ts`).

### 2. CodeSoul Hypha — Agent Core + Production Harness
- **URL**: https://github.com/CodeSoul-co/Hypha (165 ★)
- **Category**: Enterprise Agent Runtime
- **Key Innovation**: Separates **Agent Core** (reasoning/ReAct) from **Production Harness** (FSM execution, checkpoints, recovery, replay, audit). DomainPack compiles product-specific config into shared runtime. Cache & Reuse Plane operates on "reuse without authority" principle.
- **NeoTrix Relevance**: NT-ACT + NT-SHIELD. Hypha's "Cache & Reuse Plane" concept (accelerate without authorizing) maps directly to NeoTrix's cache policy. DomainPack as declared product boundary is analogous to NT-CORE's CapabilityRegistry. Worth studying for SEAL pipeline production-wiring patterns.

### 3. AgentOS (Framers AI) — Cognitive Memory + Runtime Tool Forging
- **URL**: https://github.com/framersai/agentos (619 ★)
- **Category**: Cognitive Agent Framework
- **Key Innovation**: Runtime tool forging (agent writes TS function, LLM judge approves, runs in hardened VM, joins catalog). 8 neuroscience-backed memory mechanisms (Ebbinghaus decay, retrieval-induced forgetting, reconsolidation). HEXACO personality vector biases retrieval/routing.
- **NeoTrix Relevance**: NT-MEMORY + NT-FEEL. Cognitive memory mechanisms directly inform NT-MEMORY's KB recall strategy. Tool forging as capability evolution mirrors SEAL's skill crystallization. HEXACO personality → NT-FEEL EmotionLabel modulation of behavior.

### 4. GitAgent — Git-Native Agent Framework
- **URL**: https://github.com/open-gitagent/gitagent (670 ★)
- **Category**: Version-Controlled Agents
- **Key Innovation**: Agent IS a git repo — `SOUL.md` (identity), `RULES.md` (constraints), `memory/` (git-committed), `tools/` (YAML declarative), `skills/` (composable). Fork an agent = fork personality. `git log` shows memory evolution.
- **NeoTrix Relevance**: NT-MEMORY + NT-NEXUS. Git-as-memory-versioning is a clean alternative to KB versioning for agent-level state. Validate whether NT-NEXUS could use git commit hashes as experience pointers (addressing the pointer conservation rule in AGENTS.md).

### 5. Harden AIF — Security Layer for AI Coding Agents
- **URL**: https://harden.run (Product Hunt #2, Sep 9 — 399 upvotes)
- **Category**: Agent Security
- **Key Innovation**: Post-trained model checks tool calls before they run, using request + session context. Beats frontier models on agent-security benchmarks. Local-first, no repo data leaves machine.
- **NeoTrix Relevance**: NT-SHIELD. Validates the egress privacy guard pattern. Harden's "post-trained model as gatekeeper" is a more principled version of NT-SHIELD's egress policy. Study for `rev-officer` skill integration — could Harden's model serve as a pre-commit security oracle.

### 6. GenericAgent — Self-Evolving Minimal Agent
- **URL**: https://github.com/lsdefine/genericagent (14,114 ★)
- **Category**: Self-Evolving Agent
- **Key Innovation**: ~3K lines core, ~100-line agent loop. 9 atomic tools + self-evolving skill tree. Morphling mode (project-level skill absorption from external repos). Token-efficient (<30K context vs 200K+ peers). Conductor sub-agent orchestration.
- **NeoTrix Relevance**: NT-CORE + NT-MIND. GenericAgent's "Morphling mode" (absorb external repo → decide call/rewrite/discard per component) is nearly identical to NeoTrix's external absorption protocol (R-P42/R-P79). Their <30K context efficiency validates Cost-Aware Routing axiom (A1). Conductor pattern maps to NT-ACT orchestration.

### 7. Microsoft Agent Framework (MAF) — Production Multi-Agent Orchestration
- **URL**: https://github.com/microsoft/agent-framework (13,306 ★)
- **Category**: Enterprise Multi-Agent Platform
- **Key Innovation**: Graph-based workflow patterns (sequential, concurrent, handoff, group collaboration). Foundry Hosted Agents (2 LOC deploy). AF Labs for experimental features. Time-travel debugging for agent workflows.
- **NeoTrix Relevance**: NT-ACT. MAF's graph-based orchestration patterns are directly applicable to SEAL pipeline stages. Time-travel debugging could inform NT-REPAIR's self-healing diagnostics. Foundry hosted deploy pattern validates NT-IO's deployment abstractions.

### 8. OmniRoute — Open-Source AI Gateway
- **URL**: https://github.com/diegosouzapw/OmniRoute (17,900 ★)
- **Category**: AI Gateway / Router
- **Key Innovation**: Single endpoint routing across 231+ providers (50+ free). Token compression, smart fallback, multimodal API. Self-hosted alternative to OpenRouter/LiteLLM.
- **NeoTrix Relevance**: NT-IO. OmniRoute's ordered fallback + provider load balancing validates NeoTrix's "total_calls ascending" routing rule. Token compression layer could reduce GWT salience computation cost. Study for NT-IO provider abstraction.

### 9. Colibri — Pure-C Inference Engine
- **URL**: https://github.com/JustVugg/colibri (14,700 ★)
- **Category**: Edge/Local Inference
- **Key Innovation**: Pure C, zero dependencies. Runs GLM-5.2 (136B MoE) on 5GB RAM via streaming experts from disk. Minimal memory footprint.
- **NeoTrix Relevance**: NT-PHYSICAL + NT-IO. Validates Cost-Aware Routing axiom (A1) — edge inference enables local model deployment. Colibri's expert-streaming pattern could optimize NT-PHYSICAL's resource budget management for constrained devices.

### 10. Kilo Code — Open-Source Agentic Engineering Platform
- **URL**: Product Hunt #1, Sep 2026 — #1 Product of Month
- **Category**: Agentic IDE
- **Key Innovation**: Agentic engineering platform — agents as first-class citizens in development workflow. Open-source, multi-model support.
- **NeoTrix Relevance**: NT-IO. Kilo Code's positioning as "agentic engineering" (not just "coding assistant") validates NeoTrix's developer-toolkit identity. Monitor for integration patterns — could Kilo Code + NeoTrix form a complementary stack.

---

## Cross-Cutting Patterns (Sep 2026)

| Pattern | Projects | NeoTrix Mapping |
|---------|----------|-----------------|
| **Filesystem-as-State** | Eve, GitAgent, Hypha | NT-MEMORY hub → validate node-as-file pattern |
| **Cache Without Authority** | Hypha, Harden | NT-SHIELD cache policy, GWT salience |
| **Self-Evolving Skills** | GenericAgent, AgentOS | SEAL pipeline skill crystallization (C4→C5) |
| **Runtime Tool Forging** | AgentOS | NT-ACT capability evolution |
| **Git-as-Versioning** | GitAgent | NT-NEXUS experience pointer versioning |
| **Multi-Agent Graph** | MAF, Hypha | SEAL pipeline stage orchestration |
| **Ordered Backend Fallback** | OmniRoute | NT-IO provider routing |
| **Edge Inference** | Colibri | Cost-Aware Routing (A1), NT-PHYSICAL |
| **Security Gatekeeper** | Harden AIF | NT-SHIELD egress guard |
| **Cognitive Memory** | AgentOS, GenericAgent | NT-MEMORY recall strategy |

---

## Star Growth Leaders (28-day, Sep 2026)

| Project | Stars | Growth | Signal |
|---------|-------|--------|--------|
| GenericAgent | 14,114 | Fast | Self-evolving minimal agents dominate |
| OmniRoute | 17,900 | Fast | AI gateway as essential infra |
| Colibri | 14,700 | Fast | Edge inference demand surging |
| Nanobot | 47,663 | Steady | Lightweight personal agents |
| Firecrawl | 170,000+ | Steady | Web context APIs maturing |
| Harden AIF | New | PH #2 | Agent security becoming critical |

---

*Generated by NeoTrix iteration loop, cycle 431*
