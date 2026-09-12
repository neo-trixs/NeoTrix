# Trending Rankings — Cycle 370 (2026-09-12)

## Methodology
GitHub trending (velocity-weighted star growth) + ProductHunt recent launches + arxiv high-signal repos. Deduplicated against cycles 318-369.

---

## 1. OpenViking (Volcengine)
- **Repo**: volcengine/OpenViking
- **Stars**: 31K+ | **Language**: Python
- **What**: Self-evolving Context Database for AI Agents. Unifies agent memory, knowledge RAG, and skills into a single persistent store.
- **Signal**: 950 stars/day — fastest-growing context infra project in Sep 2026.
- **NeoTrix Mapping**: Directly maps to NT-MEMORY (KB hub). Confirms Axiom A2 (context as scarce resource). OpenViking's self-evolving schema aligns with SEAL pipeline's knowledge crystallization. **Absorb: context DB as memory tier**.

## 2. caveman
- **Repo**: JuliusBrussee/caveman
- **Stars**: 99K+ | **Language**: Go
- **What**: Claude Code skill that cuts 65% of tokens by compressing prompts into terse "caveman" speak. Zero inference cost — pure prompt rewriting.
- **Signal**: 258 stars/day. Viral among Claude Code power users.
- **NeoTrix Mapping**: Token compression as NT-IO cost optimization. Validates Axiom A1 (cost-aware routing) at the prompt level. **Absorb: prompt compression as pre-routing filter**.

## 3. Munder-difflin
- **Repo**: chaitanyagiri/munder-difflin
- **Stars**: 3.1K+ | **Language**: TypeScript
- **What**: Local multi-agent harness. Runs multiple coding agents in parallel with shared state and inter-agent messaging.
- **Signal**: 507 stars/day. Top trending agent framework.
- **NeoTrix Mapping**: Multi-agent coordination for NT-ACT orchestration. Maps to ETI (Explicit Trait Inference) coordination patterns. **Absorb: parallel agent harness patterns**.

## 4. turbovec
- **Repo**: RyanCodrai/turbovec
- **Stars**: 15.9K+ | **Language**: Rust + Python
- **What**: Vector index built on TurboQuant. Rust core with Python bindings. Quantized vector search for agent memory retrieval.
- **Signal**: 230 stars/day. High-performance vector infra.
- **NeoTrix Mapping**: NT-MEMORY vector search backend. Rust-native aligns with R-P1. Potential KB embedding acceleration layer. **Absorb: quantized vector index for KB queries**.

## 5. Agent Substrate
- **Repo**: agent-substrate/substrate
- **Stars**: 1.4K+ | **Language**: Go
- **What**: Core system for agent execution. Provides isolation, state management, and lifecycle for long-running agents.
- **Signal**: Emerging. Cross-vendor agent runtime standard.
- **NeoTrix Mapping**: NT-ACT execution substrate. Aligns with P2 (Isolation-per-Task). **Absorb: agent runtime isolation patterns**.

## 6. ai-memory (akitaonrails)
- **Repo**: akitaonrails/ai-memory
- **Stars**: 3.6K+ | **Language**: Rust
- **What**: Long-term memory for agent coding CLIs. Shared persistent wiki from sanitized session observations. Cross-vendor handoff.
- **Signal**: 332 stars/day. Solves session-boundary problem.
- **NeoTrix Mapping**: NT-MEMORY cross-session persistence. Maps to NT-NEXUS bridge. Validates experience-tree absorption protocol. **Absorb: session handoff memory format**.

## 7. mainline
- **Repo**: mainline-org/mainline
- **Stars**: emerging | **Language**: likely Rust/Go
- **What**: Git-native memory for coding agents. Repo memory before the diff — tracks agent context at the repository level.
- **Signal**: High velocity in agent-context topic.
- **NeoTrix Mapping**: NT-MEMORY repo-scoped context. Maps to KB's file-system integration. **Absorb: git-native context as memory primitive**.

## 8. vellis
- **Repo**: volantlabs/vellis
- **Stars**: emerging | **Language**: Rust
- **What**: Open-source context graph engine. Typed, local-first memory with explicit schema, validated change, deterministic query, migration, snapshots, replay, audit. MCP-native.
- **Signal**: 2026-09 trending in agent-context.
- **NeoTrix Mapping**: NT-MEMORY graph-native context. Schema-first approach aligns with KB node/edge model. **Absorb: deterministic context graph engine**.

## 9. Mnemovela
- **Repo**: axisrobo/mnemovela-open
- **Stars**: emerging | **Language**: Python & Go
- **What**: Agent Cognition Runtime. Typed memory, Git-like branches, hybrid retrieval, governed forgetting. Embedded-first. Open-source SDKs.
- **Signal**: Novel "governed forgetting" — Ebbinghaus-curve memory decay.
- **NeoTrix Mapping**: NT-MEMORY cognitive memory with forgetting. Maps to NT-FEEL (emotion-driven retention). **Absorb: governed forgetting for KB cleanup**.

## 10. cortex (gambletan)
- **Repo**: gambletan/cortex
- **Stars**: emerging | **Language**: Rust
- **What**: Memory for AI agents that never leaves your device. Local-first, end-to-end encrypted, zero-telemetry. Rust + MCP.
- **Signal**: Privacy-first agent memory.
- **NeoTrix Mapping**: NT-SHIELD + NT-MEMORY convergence. Aligns with Egress Privacy Guard trust tiers. **Absorb: encrypted local memory as trust-tier primitive**.

---

## Emerging Patterns (Cycle 370)

### Pattern C370-1: Context Database as Standalone Layer
OpenViking, vellis, and Mnemovela all position context management as a **first-class infrastructure layer** — not embedded in the agent framework. This validates NeoTrix's KB-centric architecture but signals that the context DB should be more explicitly exposed as a service, not just internal state.

### Pattern C370-2: Token Compression at Prompt Level
caveman's 65% token reduction via prompt rewriting (zero inference cost) suggests NT-IO should integrate lightweight prompt compression as a pre-routing step, before any model call.

### Pattern C370-3: Governed Forgetting
Mnemovela's "governed forgetting" with Ebbinghaus decay curves aligns with NT-FEEL emotion-driven retention. Memory that forgets intentionally is as important as memory that remembers.

### Pattern C370-4: Git-Native Context
mainline and ai-memory both treat the git repo itself as a memory primitive. NT-MEMORY should expose repo-level context as a first-class query target.

### Pattern C370-5: Rust-Native Vector Infra
turbovec (Rust quantized vectors) + vellis (Rust context graph) + cortex (Rust encrypted memory) — the agent memory infra layer is converging on Rust. Validates NeoTrix's Rust-first strategy.
