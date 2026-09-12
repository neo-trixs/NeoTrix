# Trending Rankings — Cycle 371 (2026-09-12)

## Methodology
GitHub trending (velocity-weighted star growth) + ProductHunt recent launches + arxiv high-signal repos. Deduplicated against cycles 318-370.

---

## 1. Colibri
- **Repo**: JustVugg/colibri
- **Stars**: 17.8K+ | **Language**: C
- **What**: Run GLM-5.2 (744B MoE) on a 25GB-RAM consumer machine — pure C, zero deps, experts streamed from disk. Tiny engine, immense model.
- **Signal**: 1.7K stars/month. Breakthrough in consumer-hardware LLM inference — making 700B+ models accessible without GPU clusters.
- **NeoTrix Mapping**: NT-PHYSICAL + NT-IO convergence. Colibri's disk-streaming expert activation is a novel memory-tier strategy for NT-MEMORY (KVMem). Validates Axiom A2 (context as scarce resource) at the hardware level — memory capacity is the bottleneck, not compute. **Absorb: disk-streamed expert activation for edge inference**.

## 2. OmniRoute
- **Repo**: diegosouzapw/OmniRoute
- **Stars**: 1.2K+ | **Language**: TypeScript
- **What**: Free MIT AI gateway — one endpoint, 290+ providers (90+ free), 500+ models. Quota-aware auto-fallback, RTK+Caveman compression saves 15-95% tokens, MCP/A2A. Works with Claude Code, Codex, Cursor, OpenCode, Cline & Copilot.
- **Signal**: 190 stars/month. Unified routing layer becoming the de facto standard for multi-provider agent setups.
- **NeoTrix Mapping**: NT-IO Ordered Backend Router. OmniRoute's 290+ provider fallback chain validates P4 (Ordered Backend Fallback) at massive scale. RTK+Caveman compression = pre-routing token optimization. **Absorb: quota-aware provider fallback with token compression**.

## 3. amux
- **Repo**: mixpeek/amux
- **Stars**: emerging | **Language**: Rust
- **What**: Open-source control plane for AI coding agents. Run an AI engineering team: parallel Claude Code, Codex, and Gemini workers with a shared board, atomic tasks, schedules, loops, origin-stamped messaging, model switching, and self-healing recovery. Single Rust binary.
- **Signal**: Trending on agent-orchestration topic. MIT license, Rust-native.
- **NeoTrix Mapping**: NT-ACT orchestration. amux's atomic task board + self-healing recovery maps to SEAL pipeline task scheduling. Origin-stamped messaging = event provenance. **Absorb: atomic task board pattern for multi-agent coordination**.

## 4. Kopai
- **Repo**: usekopai.com (ProductHunt #14, 84 upvotes, Sep 8 2026)
- **Stars**: emerging | **Language**: Python
- **What**: The cloud for AI agents. Build, host, distribute, and monetize agents. Marketplace with per-message billing, one-command benchmark, certification with expiry. Analytics separate cost from revenue.
- **Signal**: ProductHunt #14 of the day. First "agent marketplace" with lifecycle evaluation.
- **NeoTrix Mapping**: NT-ACT + NT-IO. Kopai's agent marketplace = capability marketplace. Certification expiry = quality gate. Per-message billing = cost-aware routing at economic level. **Absorb: agent marketplace as capability exchange pattern**.

## 5. Agent-Radar
- **Paper**: arXiv:2605.30136 | **Code**: emerging
- **Stars**: emerging | **Language**: Python
- **What**: Training-free context management that dynamically steers agent attention via temporal and spatial decay. No compression or pruning — steers attention toward relevant context. Integrates SPA (Selective Prompt Anchoring) as backend.
- **Signal**: Novel "attention steering" — distinct from context compression. Works across 3 base LLMs, 5 benchmarks.
- **NeoTrix Mapping**: NT-CORE (GWT). Agent-Radar's temporal+spatial decay directly extends GWT salience. Instead of broadcasting all salient info, steer attention toward relevant subset. Complements PerceptionBridge (L2→L5). **Absorb: attention steering as GWT refinement**.

## 6. herdr
- **Repo**: herdrdev/herdr
- **Stars**: 10.5K+ | **Language**: Rust
- **What**: Agent multiplexer that lives in your terminal. The runtime your coding agents live on. Manages multiple agent sessions with typed SDK.
- **Signal**: 898 stars/month. Rust-native agent runtime gaining traction.
- **NeoTrix Mapping**: NT-ACT runtime substrate. herdr's terminal-native multiplexer validates P2 (Isolation-per-Task) at UX level. Typed SDK aligns with PTC (Programmatic Tool Calling). **Absorb: terminal-native agent multiplexing**.

## 7. Waste
- **Repo**: sqliteai/waste
- **Stars**: 459+ | **Language**: C
- **What**: Run the full 2.78-trillion-parameter Kimi K3 model beyond available RAM by streaming activated weights directly from NVMe. Dependency-free, embeddable C inference engine.
- **Signal**: 42 stars/month. Extreme memory optimization — running 2.78T params on consumer NVMe.
- **NeoTrix Mapping**: NT-PHYSICAL + NT-MEMORY. Waste's NVMe streaming = hardware-level KVMem. Validates paged KV virtualization (KVMem paper) with production implementation. **Absorb: NVMe weight streaming for model inference**.

## 8. AgentOS
- **Repo**: rivet-dev/agentos
- **Stars**: emerging | **Language**: Rust (WebAssembly)
- **What**: Give agents an operating system as a library. Runs in your existing backend — no sandboxes, VMs, or SaaS. Powered by WebAssembly & V8 isolates.
- **Signal**: Emerging on agent-orchestration topic. Novel WASM-based isolation model.
- **NeoTrix Mapping**: NT-ACT + NT-SHIELD. AgentOS's WASM isolation = lightweight sandbox for tool execution. Maps to NT-SHIELD sandbox patterns. V8 isolates = per-agent state isolation. **Absorb: WASM-based agent isolation primitive**.

## 9. Graphify
- **Repo**: Graphify-Labs/graphify
- **Stars**: 19K+ | **Language**: Python
- **What**: AI coding assistant skill. Turn any folder of code, SQL schemas, R scripts, shell scripts, docs, papers, images, or videos into a queryable knowledge graph. App code + database schema + infrastructure in one graph.
- **Signal**: 1.8K stars/month. Knowledge graph as universal code comprehension layer.
- **NeoTrix Mapping**: NT-MEMORY knowledge graph. Graphify's multi-modal knowledge graph = KB schema evolution. Maps to VSA HyperCube's associative recall. **Absorb: code-to-knowledge-graph as memory crystallization**.

## 10. Orca
- **Repo**: stablyai/orca
- **Stars**: 16.2K+ | **Language**: Rust/TypeScript
- **What**: The ADE (Agent Development Environment) for working with a fleet of parallel agents. Run any coding agent with your own subscription. Available on desktop and mobile.
- **Signal**: 1.4K stars/month. Multi-agent IDE pattern — "Cursor for agent fleets."
- **NeoTrix Mapping**: NT-IO + NT-ACT. Orca's fleet management = NT-ACT orchestration UI. Desktop+mobile = cross-platform agent access. **Absorb: fleet-level agent IDE as orchestration interface**.

---

## Emerging Patterns (Cycle 371)

### Pattern C371-1: Consumer-Grade Massive Inference
Colibri (744B on 25GB RAM) + Waste (2.78T on NVMe) both prove that massive models can run on consumer hardware through disk streaming. This validates Axiom A2 at the hardware frontier: memory tiering is the true bottleneck, not compute.

### Pattern C371-2: Agent Marketplace Economics
Kopai introduces agent certification with expiry, per-message billing, and cost-vs-revenue analytics. This is the first "agent economy" infrastructure — agents as tradable services with quality guarantees.

### Pattern C371-3: Attention Steering > Context Compression
Agent-Radar proves that steering attention toward relevant context outperforms compressing or pruning context. This is a paradigm shift: don't shrink the context, redirect the attention. Maps directly to GWT's salience-based routing.

### Pattern C371-4: WASM as Agent Isolation Primitive
AgentOS uses WebAssembly + V8 isolates for agent sandboxing — no Docker, no VMs, no SaaS. This is the lightweight security primitive NT-SHIELD should adopt for tool execution isolation.

### Pattern C371-5: Code-as-Knowledge-Graph
Graphify's universal code-to-graph conversion + Colibri's MoE activation = agents that understand codebases as graphs, not files. NT-MEMORY should expose code relationships as first-class graph queries.
