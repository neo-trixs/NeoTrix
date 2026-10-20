# NeoTrix Architecture Evaluation 2026-05

**Date:** 2026-05-24
**Version:** 0.18.0
**Scope:** neotrix-core crate (workspace root), excluded src-tauri

---

## 1. Executive Summary

**Overall Health Score: B+**

NeoTrix has undergone substantial architectural modernization in the current cycle. The HC (HyperCube-Consciousness) series added 8 new core modules across the VSA engine, GWT routing, and hypercube bridge layers. R2 and R3 refactoring series cleaned up module splits, permissions, and dead code. The project is structurally sound at Layers 1-2 (core and reasoning_brain) with well-defined zero-dependency boundaries.

Three areas require attention: (a) the neotrix/ module accounts for 75% of all code, a crate-split candidate; (b) legacy_kernel.rs at 3,868 lines is a maintenance burden with 5 known consumers; (c) tokio::spawn fire-and-forget patterns exist across multiple files without JoinHandle management.

| Metric | Value | Assessment |
|--------|-------|------------|
| Total .rs files | 286 | Moderate |
| Lines of Rust | 75,878 | Large codebase |
| Frontend (TSX/CSS/JS) | 4,832 lines, 27 files | Small |
| Public API exports | 18 | Well-scoped |
| Feature gates | ~35 across 7 files | Manageable |
| Total test functions (#[test]) | 1,269 | Good |
| Ignored tests | 5 files | Acceptable |
| Crate split candidate | neotrix/ (75% LOC) | Planned R4-01 |

---

## 2. Module Architecture

### 2.1 Layer Diagram

```
Layer 1: core/ (29 files, 4,524 LOC)
  ├── hypercube/     (8 files)  4096-dim VSA, 8-axis coordinates, gap analysis
  ├── consciousness/ (6 files)  Global Workspace Theory, module_def, workspace
  ├── memory/        (9 files)  ReasoningBank, tiered L1/offload/pipeline, iteration
  ├── capability.rs             CapabilityVector (23 dim + extension + provenance)
  ├── knowledge.rs              KnowledgeSource (6 built-in + KnowledgeProvider trait)
  ├── hypergraph.rs             HyperGraph, HyperNode, HyperEdge
  └── signal.rs, edit.rs, absorb.rs, iteration.rs, traits.rs, event.rs, rkyv_store.rs

Layer 2: reasoning_brain/ (42 sub-modules, ~12,000+ LOC)
  ├── self_iterating/           SEAL loop (16 stages: brain_impl, loop_impl, pipeline, persist)
  ├── goal_loop/                GoalTracker, loop_impl, types, RateLimiter, CircuitBreaker
  ├── context_artifacts/        store, indexer, types
  ├── core/                     capability, knowledge_source, evaluator, access
  ├── reasoning_engine.rs       (4 reasoning types: Conversation/TaskSolving/ErrorDebugging/KnowledgeQuery)
  ├── cortex_memory.rs          (992 lines, largest brain sub-module)
  ├── hypercube_bridge.rs       AttentionRouter + HyperCubeBridge integration
  ├── attention_router.rs       GWT-based attention routing
  └── 25+ other modules: code_graph, knowledge_engine, pipeline, memory, self_evolver, etc.

Layer 3: agent/ (22 files, 7,836 LOC)
  ├── tools/                    MCP tools (mod.rs: 748 lines, patch.rs)
  ├── sub_agent.rs              Sub-agent dispatch (802 lines)
  ├── team.rs                   AgentTeam + Coordinator (774 lines)
  ├── persona.rs                Agent persona definitions (770 lines)
  ├── skills.rs                 Skill management (815 lines)
  └── workflow.rs, agent_workflow.rs, blackboard.rs, cognitive_memory.rs, etc.

Layer 4: neotrix/ (216 files, 59,337 LOC)
  ├── security/                 guard, vault, keyvault, permissions, audit, policy
  ├── stealth_net/              (28 files, 10,000+ LOC) proxy chain, http_client, tor, etc.
  ├── orchestrator/             DAG scheduler, planner, worker, critic, state_graph
  ├── provider/                 LLM interfaces (openai, anthropic, ollama, gemini)
  ├── crawler/                  unified, fetcher, frontier, classifier
  ├── reasoning_kernel/         (22 files, legacy shim) tests (1,017 lines)
  └── subagent/, browser/, signal/, parallel/, acp_server/, etc.

Infrastructure: cli/ (9 files), server/ (5 files), bin/ (3 files), main.rs (873 lines)
```

### 2.2 Dependency Flow

```
core/  ←  reasoning_brain/  ←  agent/  ←  neotrix/*
  |           |                    |
  |           +-- goal_loop/       +-- MCP tools (rmcp 0.5)
  |           +-- self_iterating/  +-- sub-agent dispatch
  |           +-- memory/          +-- swarm modes (5)
  |
  +-- hypercube/  ──→  hypercube_bridge.rs  ──→  reasoning_brain/
  +-- consciousness/  ──→  attention_router.rs  ──→  reasoning_brain/
```

**Key properties:**
- Layer 1 (core/) has zero external runtime dependencies. This invariant is enforced by design and holds.
- Layer 2 depends only on core/ + tokio. No circular dependencies detected between core/ and reasoning_brain/.
- Layer 3 (agent/) depends on core/ but does not import from reasoning_brain/ directly -- it accesses brains through trait objects (BrainProvider, EngineProvider in core/traits.rs).
- The neotrix/ module is a flat namespace containing all integration code. Its 216 files and 59,337 lines represent the main candidate for cavity decomposition.

### 2.3 Module Dependency Analysis

| From | To | Dependency Type | Risk |
|------|----|-----------------|------|
| core/ | (none) | Zero external deps | None (by design) |
| reasoning_brain/ | core/ | Internal struct/trait usage | None |
| agent/ | core/ | Trait-based via traits.rs | Low |
| neotrix/{orchestrator,provider,...} | agent/, core/ | Direct use | Low |
| stealth_net/ | (none within project) | External deps only | Moderate (28 files, tightly coupled) |
| legacy_kernel.rs | reasoning_engine, reasoning_kernel | Superseded | High (see section 4) |

---

## 3. Feature Gate Analysis

### 3.1 Feature Table

| Feature | Dependencies | Modules Gated | Files Gated | Status |
|---------|-------------|---------------|-------------|--------|
| `default` | (none) | — | — | Empty, intentional |
| `rkyv-storage` | rkyv, memmap2 | core::rkyv_store, core::memory::bank | 2 files | Not default, used by persistence |
| `sandbox` | wasmtime, aes-gcm | neotrix::sandbox, neotrix::security | 2 files | Security-isolated |
| `telemetry` | opentelemetry*, tracing-opentelemetry | neotrix::telemetry | 1 file | Tracing integration |
| `chromiumoxide` | chromiumoxide | (stealth-net dependency) | 0 direct | Enables stealth browser |
| `stealth-net` | chromiumoxide | neotrix::stealth_net, background_loop | 2 files (16 gates) | 50% of all cfg gates |
| `keyring` | keyring | neotrix::security::keyvault | 1 file (7 gates) | OS keychain storage |
| `full` | chromiumoxide, stealth-net, sandbox, telemetry | All of the above | Aggregate | Meta-feature for dev |

### 3.2 Gate Distribution

```
stealth-net:  16 gates  (background_loop.rs)
sandbox:       5 gates  (sandbox.rs + security/mod.rs)
stealth-net:   7 gates  (background_loop.rs additional)
keyring:       7 gates  (security/keyvault.rs)
rkyv-storage:  3 gates  (core/mod.rs + memory/bank.rs)
```

### 3.3 Orthogonality Assessment

- **stealth-net** and **chromiumoxide** overlap intentionally (stealth-net depends on chromiumoxide). Should be consolidated: chromiumoxide is never used independently.
- **sandbox**, **keyring**, **rkyv-storage**, and **telemetry** are fully orthogonal -- they gate entirely different module trees.
- **default** being empty is correct for this type of project (no minimal feature set beyond base code).
- **full** aggregates 4 features but omits keyring. This asymmetry should be documented or resolved.

**Recommendation:** Merge chromiumoxide into stealth-net as an internal dep, and add keyring to the full meta-feature if appropriate for dev workflows.

---

## 4. Top Hotspots

### 4.1 Largest Files

| Rank | File | Lines | Module | Risk |
|------|------|-------|--------|------|
| 1 | `neotrix/legacy_kernel.rs` | 3,868 | neotrix | HIGH -- superseded by reasoning_engine |
| 2 | `neotrix/reasoning_kernel/tests.rs` | 1,017 | neotrix | MEDIUM -- large test file for legacy module |
| 3 | `neotrix/reasoning_brain/cortex_memory.rs` | 992 | reasoning_brain | MEDIUM -- single large file |
| 4 | `core/knowledge.rs` | 964 | core | LOW -- data model, stable |
| 5 | `neotrix/reasoning_brain/self_iterating/loop_impl.rs` | 959 | reasoning_brain | MEDIUM -- SEAL loop core logic |
| 6 | `neotrix/reasoning_brain/reasoning_engine.rs` | 955 | reasoning_brain | LOW -- well-structured |
| 7 | `neotrix/reasoning_brain/knowledge_engine.rs` | 913 | reasoning_brain | LOW |
| 8 | `neotrix/project_manager.rs` | 904 | neotrix | LOW -- declining usage |
| 9 | `main.rs` | 873 | — | LOW -- entry point, stable |
| 10 | `neotrix/reasoning_brain/tests.rs` | 863 | reasoning_brain | MEDIUM -- test concentration |

### 4.2 legacy_kernel.rs Analysis

The file at 3,868 lines is the single largest hotspot. It was superseded by `reasoning_engine.rs` (955 lines) but retained for backward compatibility. The 5 known consumers should be audited:

```
legacy_kernel.rs consumers (from grep):
  neotrix/reasoning_kernel/mod.rs
  neotrix/orchestrator/* (likely trait dispatch)
  agent/ (likely through BrainProvider trait)
  neotrix/reasoning_brain/ (shim layer)
  neotrix/* (integration tests)
```

**Recommended actions:**
1. Verify each consumer against `reasoning_engine.rs` API compatibility.
2. For each consumer, test a swap to `reasoning_engine` in isolation.
3. Once all 5 are migrated, gate `legacy_kernel.rs` behind a `legacy` feature.
4. Remove in Q3 2026.

### 4.3 Code Concentration

- `neotrix/` contains 75% of all Rust code (59,337 / 75,878 LOC). This is the primary candidate for a crate split.
- Within `neotrix/`, three sub-clusters dominate:
  - `stealth_net/`: 28 files, ~10,000+ LOC (proxies, http_client, tor, firewall)
  - `reasoning_brain/`: 42 sub-modules, ~12,000+ LOC
  - `reasoning_kernel/`: 22 files, legacy shim with significant test mass

**Crate-split candidates for R4 (in priority order):**
1. `neotrix-stealth-net` -- self-contained, feature-gated, 28 files. Most isolated.
2. `neotrix-ra` (reasoning_brain) -- largest brain logic. Dependent on core/ only.
3. `neotrix-security` -- vault/guard/keyvault/audit. Already has self-contained sub-module tree.
4. `neotrix-agent` -- 22 files, 7,836 LOC. Clean trait boundaries through core/traits.rs.

---

## 5. Test Coverage

### 5.1 Distribution

| Module | Tests (#[test]) | Assessment |
|--------|-----------------|------------|
| neotrix/legacy_kernel.rs | 101 | High, but for obsolete code |
| neotrix/reasoning_kernel/tests.rs | 92 | Dedicated test file |
| neotrix/reasoning_brain/tests.rs | 49 | Dedicated test file |
| neotrix/stealth_net/http_client/request.rs | 32 | Unit tests inline |
| neotrix/project_manager.rs | 32 | Inline unit tests |
| neotrix/stealth_net/rule_api.rs | 26 | Inline unit tests |
| neotrix/reasoning_brain/context_artifacts/mod.rs | 24 | Inline unit tests |
| agent/skills.rs | 21 | Inline unit tests |
| agent/team.rs | 20 | Inline unit tests |
| neotrix/stealth_net/rules.rs | 19 | Inline unit tests |
| agent/tools/mod.rs | 18 | Inline unit tests |
| neotrix/security/guard.rs | 17 | Security-critical, adequate |
| remaining | ~800+ | Spread across 180+ files |

### 5.2 Slow / Ignored Tests

5 files contain `#[ignore]` tests:

| File | Reason | Impact |
|------|--------|--------|
| `stealth_net/stealth_browser.rs` | Requires chromiumoxide binary | Integration-only |
| `stealth_net/geo_proxy.rs` | External geo-IP service | Integration-only |
| `stealth_net/lan_router.rs` | Requires network setup | Integration-only |
| `reasoning_brain/code_graph.rs` | Large dependency graph resolution | Performance |
| `reasoning_brain/code_graph_executor.rs` | Full SEAL loop execution | Performance |

**Recommendation:** Move ignored tests to a separate `tests/` integration directory with explicit `#[cfg(feature = "integration")]` gating, rather than `#[ignore]`.

### 5.3 Test Gaps

- `core/hypercube/` (8 files) has tests only in `cube.rs` (3 tests), `rkyv_store.rs` (4 tests). The axis, coord, and gap modules have zero tests.
- `core/consciousness/` (6 files) has zero tests -- workspace.rs and module_def.rs are untested.
- `core/memory/` (9 files) has tests only in `bank.rs`. Sub-modules (l1, tier, offload, pipeline, stats) are untested.
- `agent/workflow.rs`, `agent/agent_workflow.rs`, `agent/blackboard.rs` have tests but at low density relative to their complexity.

---

## 6. Security Posture

### 6.1 Security Modules

| Module | File | Purpose |
|--------|------|---------|
| guard.rs | `neotrix/security/` | Interactive prompt guard (17 tests) |
| vault.rs | `neotrix/security/` | Encrypted secret storage (5 tests) |
| keyvault.rs | `neotrix/security/` | OS keyring integration (6 tests, 7 cfg gates) |
| permissions.rs | `neotrix/security/` | Capability-based permission model (15 tests) |
| audit.rs | `neotrix/security/` | Security event audit log (4 tests) |
| policy.rs | `neotrix/security/` | Policy definition engine (0 tests) |
| sandbox.rs | `neotrix/` | Wasm sandbox (7 tests, gated by sandbox feature) |
| prompt_guard.rs | `neotrix/` | Prompt injection guard (10 tests) |
| secure_string.rs | `neotrix/` | Zero-on-drop secure string (2 tests) |

### 6.2 Key Findings

- `#![forbid(unsafe_code)]` is enforced in `lib.rs`. The entire codebase is safe Rust.
- `#[deny(warnings)]` is active. Compiler warnings become errors.
- The vault module uses AES-GCM (via `aes-gcm` crate, gated by `sandbox` feature).
- The keyvault module uses the system OS keyring (macOS Keychain, Linux Secret Service).
- `policy.rs` has zero tests -- this is a gap for a security-critical module.
- The Wasm sandbox (`sandbox.rs`, wasmtime) isolates untrusted code execution.

### 6.3 CSP and Frontend

The frontend (dist/) includes a CSP policy. The SPA uses DOMPurify for XSS prevention. No injection vulnerabilities identified in the Rust HTTP/WS server paths.

---

## 7. Recommendations

### 7.1 Priority Matrix

| ID | Action | Priority | Impact | Effort | Module |
|----|--------|----------|--------|--------|--------|
| R4-01 | Split neotrix/ into sub-crates | P1 | High | Large | neotrix/ |
| R4-02 | Migrate legacy_kernel consumers to reasoning_engine | P1 | High | Medium | neotrix/ |
| R4-03 | Add tests to core/hypercube/ and core/consciousness/ | P1 | Medium | Small | core/ |
| R4-04 | Gate legacy_kernel.rs behind `legacy` feature | P2 | Medium | Small | neotrix/ |
| R4-05 | Replace `let _ = tokio::spawn` with JoinHandle management | P2 | Medium | Medium | Multiple |
| R4-06 | Move ignored tests to `tests/` with `integration` feature | P2 | Low | Small | Multiple |
| R4-07 | Merge chromiumoxide feature into stealth-net | P2 | Low | Small | Cargo.toml |
| R4-08 | Add keyring to `full` meta-feature | P3 | Low | Trivial | Cargo.toml |
| R4-09 | Add tests for core/memory/ sub-modules (l1, tier, offload) | P3 | Medium | Medium | core/ |
| R4-10 | Add tests for security::policy | P3 | Medium | Small | security/ |
| R4-11 | Make rkyv-storage default-enabled (evaluate binary size impact) | P3 | Low | Small | Cargo.toml |

### 7.2 R4-01: Crate Split Strategy

The neotrix/ module at 59,337 lines is due for decomposition. Recommended split boundaries:

```
Split 1: neotrix-stealth-net (28 files, ~10K LOC)
  - Feature: stealth-net
  - Already gated behind cfg(feature = "stealth-net")
  - Cleanest break -- zero internal dependency on other neotrix/ sub-modules

Split 2: neotrix-reasoning (42 files, ~12K LOC)
  - Contains reasoning_brain/ and reasoning_kernel/
  - Depends on core/ only
  - Requires trait re-exports through core/traits.rs

Split 3: neotrix-agent (22 files, ~7.8K LOC)
  - Depends on core/ for trait interfaces
  - Already uses BrainProvider/EngineProvider abstraction

Split 4: neotrix-security (7 files)
  - Smallest split, already self-contained
  - Independent lifecycle (audit/vault/keyring dependency chain)
```

### 7.3 R4-02: Legacy Kernel Migration Plan

1. Audit all 5 consumers of `legacy_kernel.rs` for exact API usage.
2. For each consumer, implement a `reasoning_engine` equivalent function.
3. Run all tests after each swap.
4. After all 5 migrated, gate `legacy_kernel` behind `#[cfg(feature = "legacy")]`.
5. Add deprecation warning to consumer sites.
6. Remove entirely in Q3 2026.

### 7.4 R4-05: Fire-and-Forget Task Management

Currently 12+ locations use `let _ = tokio::spawn(...)` without JoinHandle. These tasks can silently fail or leak resources.

**Affected files:**
- `agent/sub_agent.rs` (lines 260, 353)
- `neotrix/stealth_net/transparent_proxy.rs` (44)
- `neotrix/stealth_net/http_client/mod.rs` (285, 292)
- `neotrix/stealth_net/system_proxy.rs` (115)
- `neotrix/stealth_net/local_proxy.rs` (293)
- `bin/proxy_daemon.rs` (35, 45, 49, 56, 69, 81, 88) -- 7 instances

**Mitigation:** Add a `SpawnHandle` wrapper or use structured concurrency with cancellation tokens. At minimum, log errors on panic via `tokio::spawn(...).unwrap_or_else(...)`.

---

*End of evaluation. Generated from codebase audit on 2026-05-24.*
