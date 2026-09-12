# Trending Rankings — Cycle 384

**Date**: 2026-09-11
**Source**: GitHub Trending, Product Hunt, arXiv, CoddyKit
**Focus**: AI agents, LLM tools, reasoning frameworks, memory/attention/routing patterns

---

## New Projects (Not in Cycles 318–383)

| # | Project | Stars | Category | NeoTrix Signal |
|---|---------|-------|----------|----------------|
| 1 | **PrimeAgent** (PrimeIntellect-ai/prime-agent) | 15.9K | Self-improving RLM agent | Prompt-as-a-variable + Continual Harness refinement; `/refine` persists lessons as supplemental prompts. Maps to NT-MIND SEAL + NT-MEMORY experience. |
| 2 | **Graft** (trailhq/Graft) | 4.9K | Context engine for coding agents | Knowledge-graph-based context injection; +46% tool-call reduction, +42% token savings on SWE-bench. Maps to NT-CORE GWT attention + NT-MEMORY KB. |
| 3 | **nanobot** (HKUDS/nanobot) | 47.7K | Ultra-lightweight personal AI agent | MCP + long-term memory (Dream) + model routing + multi-agent delegation + scheduled automation. Maps to NT-IO provider routing + NT-ACT orchestration. |
| 4 | **Ponytail** (DietrichGebert/ponytail) | 91.9K | Anti-over-engineering skill | 54% less code, 27% faster, 20% cheaper; "lazy senior dev" prompt strategy. Maps to NT-CORE SEAL pattern crystallization + NT-ACT tool economy. |
| 5 | **Webwright** (microsoft/Webwright) | 6.0K | SWE-style browser agent | Skill Factory: every solve leaves a script, distilled into reusable code skills (0 tokens, ~40s). Maps to NT-ACT skill crystallization + NT-WORLD perception. |
| 6 | **Eve** (vercel/eve) | 5.0K | Filesystem-first agent framework | Durable agents with conventional filesystem locations for inspection/extension. Maps to NT-CORE architecture transparency + NT-MEMORY persistence. |
| 7 | **9Router** (decolua/9router) | 23.8K | AI router + token saver | RTK compression, auto-fallback Subscription→Cheap→Free, 40+ providers. Maps to NT-IO provider routing + cost-aware GWT salience (Axiom A1). |
| 8 | **Harden AIF** (ProductHunt #2, Sep 9) | 399 upvotes | Security layer for AI coding agents | Post-trained model checks tool calls before execution; beats frontier models on agent-security benchmarks. Maps to NT-SHIELD guardrails. |
| 9 | **GoModel** (ProductHunt, Sep 9) | 126 upvotes | Open-source OpenRouter in Go | Single binary, budgets, caching, guardrails, load balancing, failover. ~20MB Docker image. Maps to NT-IO provider infrastructure. |
| 10 | **Agent Builder by Airtop** (ProductHunt, Sep 11) | New | Self-healing web agents | Agents that heal themselves during execution; automated error recovery. Maps to NT-REPAIR self-healing + NT-WORLD perception. |

---

## Cross-Cutting Patterns Detected

### 1. Continual Harness / Self-Refinement
- **PrimeAgent**: `/refine` persists lessons as supplemental prompts, memories, skills
- **Webwright Skill Factory**: every task leaves a reusable script behind
- **Ponytail**: iterative code reduction with safety guarantees
- **NeoTrix mapping**: SEAL pipeline Phase 5 (absorption) + experience-tree KB integration

### 2. Context Compression as First-Class Concern
- **Graft**: knowledge-graph context injection, +42% token savings
- **9Router**: RTK + Headroom + Caveman multi-layer compression
- **Ponytail**: 54% less code = less context consumed
- **NeoTrix mapping**: KVMem-inspired paged KV + Egress Privacy Guard (compression before egress)

### 3. Security for Agent Tool Calls
- **Harden AIF**: post-trained model validates tool calls pre-execution
- **9Router**: format translation + fallback safety
- **NeoTrix mapping**: NT-SHIELD egress guard + R-P1 zero unsafe

### 4. Filesystem-First / Transparent Architecture
- **Eve**: conventional file locations for agent state
- **9Router**: single binary, self-hosted
- **GoModel**: MIT, bring-your-own-keys
- **NeoTrix mapping**: KB as shared state layer + architecture transparency (AGENTS.md rules)

---

## Star Velocity Leaders (24h)

| Project | Velocity | Signal |
|---------|----------|--------|
| Ponytail | ~5K/week | Anti-over-engineering movement gaining mass |
| 9Router | ~2K/week | Token cost optimization is urgent need |
| nanobot | ~1.7K/week | Personal AI agent demand exploding |
| Graft | ~800/week | Context engineering = next frontier |
| Webwright | ~400/week | Browser agents + skill distillation |

---

## Recommendations for NeoTrix

1. **Skill Crystallization Pipeline** (inspired by Webwright Skill Factory): Every SEAL run should leave a reusable skill artifact, not just an experience entry
2. **Continual Harness Pattern** (inspired by PrimeAgent): `neotrix-experience` should support `/refine`-style incremental updates to harness state
3. **Context Compression Stack**: Integrate Headroom-style compression into NT-IO egress path (pre-LLM call)
4. **Agent Security Audit**: Harden AIF pattern → NT-SHIELD pre-execution tool call validation
5. **Anti-Over-Engineering Gate** (inspired by Ponytail): SEAL pipeline should flag over-engineered implementations
