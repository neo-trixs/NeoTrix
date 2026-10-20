# HC-09: NeoTrix Architecture Assessment

## 1. Current Architecture Overview

NeoTrix is a Rust agent framework built around a **cognitive architecture** that mimics biological intelligence. At the foundation lies `core/` — a zero-external-dependency layer providing data models, `CapabilityVector` (22-dim learnable trait), `KnowledgeHyperCube` (4096-dim VSA MAP-hypervector storage), and GWT `consciousness/` attention routing. Above it, `reasoning_brain/` implements the SEAL self-iteration loop (generate → edit → absorb → learn) with `GoalLoop` for persistent objective pursuit and a `ReasoningEngine` unifying LLM calls. The `world_model.rs` predicts expert performance from compressed latent states. `agent/` provides LLM provider abstractions (OpenAI, Anthropic, etc.), MCP tool bridge, and `AgentTeam` coordinator for multi-agent debate/hierarchy. `neotrix/` packages security (vault, guardrails, deny lists), crawler (web + classification), agent protocol (TCP+UDP discovery on port 42069), orchestrator (PlannerNode → WorkerNode → CriticNode), and signal processing (Mamba-style select operator). A Tauri desktop shell (`src-tauri/`) wraps the core as a native macOS/Linux/Windows app with a React+Tailwind frontend (~3.8K LOC). Total: ~300 Rust source files, ~77K LOC.

## 2. Module Map

| Module | Files | LOC | Purpose | Dependencies |
|--------|-------|-----|---------|-------------|
| core/ | 37 | 6,536 | Data models, traits, VSA engine, hypercube, consciousness, metacognition, memory tiers | none external |
| reasoning_brain/ | 62 | 20,770 | SEAL loop, goal loop, pipeline, capability vector, absorption, self-iteration, context artifacts | core, world_model |
| agent/ | 22 | 7,836 | LLM providers (OpenAI/Anthropic/Gemini), MCP tools, AgentTeam, coordinator, KVCache | core, reqwest, serde |
| neotrix/security/ | 7 | 2,185 | Guardrails, vault, permissions, deny list, prompt guard | core |
| neotrix/provider/ | 9 | 1,554 | LLM provider abstractions (OpenAI, Anthropic, Gemini, Ollama, DeepSeek) | core, reqwest |
| neotrix/crawler/ | 7 | 2,312 | Web crawling, HTML classification, content extraction | core, reqwest |
| neotrix/orchestrator/ | 7 | 914 | Task decomposition, PlannerNode/WorkerNode/CriticNode pipeline | core, reasoning_brain |
| neotrix/agent_protocol/ | 5 | 385 | TCP+UDP discovery, capability routing, AgentInfo broadcast | tokio |
| neotrix/signal/ | 6 | 661 | Mamba-style select operator, SSM scan, signal core | core |
| world_model.rs | 1 | 732 | Latent state expert prediction, capability forecasting | core, reasoning_brain |
| core/hypercube/ | 5 | 210 | 4096-dim VSA MAP hypervectors, bundle/bind/permute | core |
| core/consciousness/ | 3 | 79 | GWT attention routing, salience competition, broadcast | core, hypercube |
| src-tauri/ (Rust) | 6 | 1,565 | Tauri commands, state management, menu, tray | tauri, serde |
| src-tauri/ (frontend) | 30 | 3,762 | React+Tailwind desktop UI, terminal emulator | React, Tailwind, xterm |
| neotrix/ (other)* | 18 | ~26K | Knowledge populator, HTTP factory, event bus, MCP, sandbox, telemetry, etc. | varied |

*Includes: background_loop, distiller, event_bus, http_factory, knowledge_bridge, knowledge_populator, mcp_tools, project_manager, sandbox, scraper, telemetry, etc.

## 3. Key Architectural Patterns

| Pattern | Description |
|---------|-------------|
| **SEAL Loop** | Self-Editing → Absorption → Learning cycle. `generate_self_edit()` produces capability vector adjustments, `absorb()` persists with RL reward gating, weights apply temporarily then commit only if external validation passes. |
| **VSA / KnowledgeHyperCube** | 4096-dim MAP (Multiply-Add-Permute) hypervectors. `bundle()` = element-wise sum, `bind()` = circular convolution, `permute()` = fixed random permutation. Query via cosine similarity + density region detection. |
| **GWT Consciousness** | Global Workspace Theory: multiple specialist modules compete via salience (urgency × novelty × coherence). Activation above threshold triggers broadcast to all specialists. Decay prevents fixation. |
| **LatentMoE Expert Prediction** | `world_model::predict_expert_performance()` compresses capability vectors into a latent code (via learned projection), then routes tasks to the highest-predicted expert. Effectively a Mixture-of-Experts gated by latent similarity. |
| **CapabilityVector + KnowledgeSource** | 22-dim learnable trait vector with fixed semantic axes. KnowledgeSource enum maps to pre-calibrated vector offsets. `absorb()` uses EMA update: `v' = v + lr × (source_vector - v)`. Normalized to prevent dimension drift. |
| **GoalLoop + RateLimiter** | Persistent goal state machine (Pursuing/Paused/Achieved/Unmet/BudgetLimited). Backed by `~/.neotrix/goals.json`. Circuit breaker trips on stall/failure, rate limiter defaults to 100 calls/hour. Auto-generates goals from task queue every 180s. |
| **Agent Protocol (UDP+TCP)** | Broadcast agent info on UDP :42069, connect via TCP for task delegation. Capability routing matches tasks to agents by advertised `CapabilityVector` similarity. |
| **rkyv Zero-Copy Persistence** | `BrainSnapshot` stores reasoning brain state via zero-copy serialization. `ReasoningBank` memories persist as rkyv archives. Fast mmap-based reload. |
| **Tauri Desktop Shell** | Rust backend exposes commands (invoke hooks) to React+Tailwind frontend via `tauri::command`. Terminal emulator using xterm.js. |

## 4. Strengths

1. **Zero-dependency core layer** — `core/` has no external dependencies, making it auditable, portable, and trivially testable. The VSA engine, capability vectors, and memory tiers are pure Rust with no tokio/serde baggage.

2. **Coherent cognitive metaphor** — VSA hypervectors ↔ associative memory, GWT ↔ attention, SEAL ↔ metacognitive self-improvement, CapabilityVector ↔ learned skills. The abstraction stack is internally consistent and extensible.

3. **External RL gating** — `absorb()` requires external validation (compile check, test pass, user feedback, Playwright verification) before committing weight updates. This prevents self-delusion loops that plague self-rated systems.

4. **Production-ready memory management** — 3-tier memory (L1 hot, L2 warm, L3 archive) with LRU eviction, rkyv zero-copy snapshots, and SHA256 cache invalidation. Designed for long-running agents with months of context.

5. **Multi-agent protocol baked in** — UDP service discovery + TCP task delegation with capability routing. The architecture intrinsically supports distributed swarms without retrofitting.

## 5. Weaknesses / Risks

1. **Massive `reasoning_brain/` module** — 62 files, 20,770 LOC (27% of codebase). This module is a monolith in all but file count: the SEAL loop, goal loop, pipeline, self-iteration, context artifacts, and engine are tightly coupled through shared `SelfIteratingBrain` state. No clear module boundary between "loop orchestration" and "iteration logic".

2. **GWT consciousness layer is vestigial** — `core/consciousness/` is 3 files, 79 LOC total. The salience competition loop is implemented but never wired into the main SEAL loop or reasoning pipeline. It's a placeholder architecture that doesn't route attention in practice.

3. **No integration tests between modules** — The module map shows clear dependency chains (core → reasoning_brain → world_model → agent), but there are no cross-module integration tests. Tests are per-module unit tests only. The 76K LOC codebase has no end-to-end test that exercises a full SEAL cycle through LLM provider → absorption → persistence.

4. **Tauri desktop shell is underutilized** — `src-tauri/` frontend has 30 files / 3.8K LOC but `src-tauri/src` only has 6 Rust files / 1.5K LOC. The frontend is a terminal emulator; most of the Rust agent's capabilities are inaccessible from the desktop UI. No visualization of capability vectors, knowledge graph, or goal state.

5. **Orphaned `signal/` select operator** — The Mamba-style SSM select operator (`neotrix/signal/`, 6 files, 661 LOC) was implemented per AGENTS.md spec but is not actually invoked in the SEAL loop. `SelfIteratingBrain` holds an `Option<SelectableOperator>` that is never constructed outside test code.

## 6. Improvement Recommendations (Priority Order)

### P0: Reasoning brain modular extraction
Extract the SEAL loop orchestrator from `SelfIteratingBrain` into a standalone `SealLoop` struct. Split `reasoning_brain/` into three independent subcrates:
- `reasoning-brain-core` — `CapabilityVector`, `KnowledgeSource`, `Memory`
- `reasoning-brain-loop` — `SealLoop` orchestrator, `GoalLoop`
- `reasoning-brain-absorption` — `AbsorptionStrategy`, RL reward handlers

### P1: Wire GWT consciousness into the reasoning pipeline
Integrate `core/consciousness/` as an attention filter in `ReasoningEngine::reason()`. The specialist modules should be the `KnowledgeSource` variants; salience should determine which knowledge sources contribute to the current reasoning trace. Remove the placeholder status — either wire it or delete it.

### P2: End-to-end integration test suite
Write 3-5 integration tests tracing a full cycle: `define capability → run SEAL loop → call LLM provider (mock) → absorb → persist → reload → verify vector delta`. These should live in `neotrix-core/tests/` and use mock providers to avoid real API calls.

### P3: Desktop UI expansion
Extend `src-tauri/src` to expose real-time capability vector visualization (radar chart), knowledge graph browser (hypercube query results), and goal loop dashboard. Bridge `SelfIteratingBrain::stats()` and `ReasoningBank::recall_similar()` to the frontend via Tauri commands.

### P4: Connect signal/select operator to SEAL loop
Init the `SelectableOperator` in `SelfIteratingBrain::new()` with the configured dimension, and apply selective state updates in `run_seal_loop()` between the generate and absorb phases. Measure whether Mamba-style selectivity improves convergence on complex tasks.

### P5: Benchmark and baseline
Add `criterion` benchmarks for: VSA bundle/bind/permute throughput (target: <1µs per op), SEAL loop iteration latency (target: <100ms without LLM), and `ReasoningBank` recall at 10K memory items (target: <5ms via rkyv+mmap).
