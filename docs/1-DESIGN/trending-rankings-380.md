# Trending Rankings — Cycle 380

**Date:** 2026-09-12
**Sources:** GitHub Trending (Sep 2026), Product Hunt, arXiv, dev.to, TuringPost

---

## Top 10 New Projects (Not in Cycles 318–379)

| # | Project | Stars | Category | Key Pattern | NeoTrix Domain |
|---|---------|-------|----------|-------------|----------------|
| 1 | **agent-browser** (vercel-labs) | 41.9K | Browser automation CLI for AI agents | Rust-native browser agent — headless Chrome automation with agent-native API, session persistence, human-in-the-loop checkpoints | NT-ACT + NT-WORLD |
| 2 | **OpenViking** (volcengine) | 35.9K | Self-evolving context database for AI agents | Unified agent memory + knowledge RAG + skills. Self-evolving context that adapts across sessions — agent memory as living database | NT-MEMORY |
| 3 | **Dynamo** (ai-dynamo) | 8.0K | Datacenter-scale distributed inference serving | Rust-native inference framework — disaggregated prefill/decode, multi-GPU scheduling, KV cache management for agentic workloads | NT-IO + NT-CORE |
| 4 | **swarm-forge** (unclebob) | 1.8K | Multi-agent coordination tool (Clojure) | Simple tool for coordinating several AI agents — lightweight swarm orchestration with message-passing primitives | NT-ACT |
| 5 | **Weave** (Ataraxy-Labs) | 3K+ | Entity-level git merge driver for multi-agent | Resolves false conflicts when independent agents edit same file — 95% reduction vs line-based merge. Agent-native version control | NT-SHIELD + NT-ACT |
| 6 | **claw-compactor** (open-compress) | 2K+ | 14-stage fusion pipeline for LLM token compression | Reversible compression + AST-aware code analysis + intelligent content routing. Zero LLM inference cost | NT-IO (token optimization) |
| 7 | **Agent-Radar** (self-published) | — | Training-free context management for multi-agent | Temporal + spatial decay mechanism for attention steering — dynamically steers each agent toward relevant context as conversations lengthen | NT-CORE (GWT) + NT-MEMORY |
| 8 | **RIG** (0xPlaygrounds) | 4K+ | Modular Rust AI agent framework | Type-safe agent composition — rig-4 (modular agent primitives) used by Dria decentralized AI network, Nethermind, Neon | NT-ACT + NT-IO |
| 9 | **Konnect** (mixelpixx) | 518 | AI-assisted PCB design for KiCAD 10 | Native KiCAD plugin — 217 tools (schematic, layout, routing, placement, design-review) exposed to Claude via Rust binary | NT-ACT (domain agent) |
| 10 | **ghost-os** (ghostwright) | 2K+ | Full computer-use for AI agents on macOS | Self-learning workflows, native macOS. No screenshots required — direct OS-level agent integration | NT-ACT + NT-PHYSICAL |

---

## Pattern Analysis

### 1. Browser-as-Agent-Workspace (agent-browser, ghost-os, Konnect)
- **agent-browser**: Vercel Labs' Rust CLI treats browser as first-class agent environment — session persistence, human checkpoints, multi-tab orchestration.
- **ghost-os**: Native macOS integration without screenshots — agents interact with OS primitives directly, self-learning workflows.
- **Konnect**: Domain-specific agent that exposes 217 CAD tools to LLMs via single Rust binary — hardware design as agent skill.
- **NeoTrix mapping**: Validates NT-ACT's tool-as-action philosophy. Browser/OS/domain tools as agent primitives, not API wrappers. ghost-os's no-screenshot approach = direct system integration (NT-PHYSICAL alignment).

### 2. Living Memory Systems (OpenViking, Agent-Radar)
- **OpenViking**: Self-evolving context database — memory that adapts across sessions, unifies RAG + skills + agent state. Memory as living organism, not static store.
- **Agent-Radar**: Training-free attention steering with temporal + spatial decay. Relevance scoring degrades with distance, preventing context dilution.
- **NeoTrix mapping**: OpenViking's self-evolving context aligns with NT-MEMORY's experience-tree absorption. Agent-Radar's decay mechanism maps to GWT salience computation — attention should decay with temporal distance.

### 3. Disaggregated Inference Architecture (Dynamo, claw-compactor)
- **Dynamo**: Datacenter-scale serving with disaggregated prefill/decode — hardware-aware scheduling for agentic workloads with different compute/memory profiles.
- **claw-compactor**: 14-stage token compression pipeline. Zero LLM inference cost. Reversible compression means no information loss.
- **NeoTrix mapping**: Dynamo validates NT-IO's ordered backend router (P4) — different backends for different phases. claw-compactor's zero-cost compression = Axiom A2 (context as scarce resource) — compress before discarding.

### 4. Agent-Native Version Control (Weave, swarm-forge)
- **Weave**: Entity-level merge driver that resolves false conflicts from multi-agent editing. 95% conflict reduction. Git as agent collaboration substrate.
- **swarm-forge**: Lightweight Clojure swarm coordination — message-passing primitives for multi-agent workflows.
- **NeoTrix mapping**: Weave solves a real problem for NeoTrix's multi-agent SEAL pipeline — when multiple agents edit same files, line-based merge fails. Entity-level merge = agent-aware collaboration. swarm-forge's simplicity validates NT-ACT's action-as-primitive design.

### 5. Rust as Agent Infrastructure Language (agent-browser, Dynamo, RIG, Konnect)
- 4 of 10 trending projects are Rust-native. Rust provides: memory safety without GC, native performance, FFI for browser/OS integration, single-binary deployment.
- **NeoTrix mapping**: Validates NeoTrix's Rust core. Agent infrastructure benefits from Rust's guarantees — memory-safe KV cache, safe concurrent agent coordination, native browser automation.

---

## Key Signals for NeoTrix

| Signal | Source | Action |
|--------|--------|--------|
| Entity-level merge for multi-agent | Weave | Add to NT-SHIELD — agent-aware file operations |
| Self-evolving context database | OpenViking | Extend KB pipeline with self-adapting schemas |
| Attention decay as first-class mechanism | Agent-Radar | Implement in GWT — temporal decay factor for salience |
| Browser as agent primitive | agent-browser | Strengthen NT-WORLD browser automation |
| Zero-cost reversible compression | claw-compactor | Apply to context window management before eviction |
| Multi-agent coordination primitives | swarm-forge | Evaluate for SEAL pipeline agent coordination |
| Native OS integration without screenshots | ghost-os | Research direct OS API integration for NT-PHYSICAL |
