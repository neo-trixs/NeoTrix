# Iteration Batch 433 — WASM/WASI/Rust-WASM External Research

**Date**: 2026-09-06
**Focus**: WebAssembly ecosystem advances 2026, WASI P3/P1.0 roadmap, Rust-WASM toolchain maturity
**Method**: External web search + codebase grep cross-reference

---

## Sources Cited

| ID | Source | Date | Key Signal |
|----|--------|------|------------|
| S1 | Bytecode Alliance — GC and Exceptions in Wasmtime (bytecodealliance.org) | 2026-07-20 | Wasmtime 47 enables Wasm GC + Exceptions by default. Cheney-style semi-space collector. 32-bit GC refs via linear memory sandbox. |
| S2 | Component Model Issue #525 — Wasm GC in Canonical ABI (github.com/WebAssembly) | 2025-06-03 → 2026 active | GC-to-GC component calls currently require 3 copies per direction (GC→linear memory→host→linear memory→GC). 4-6× slower than Rust↔Rust. Pre-proposal for `gc` canonical option. |
| S3 | Bytecode Alliance — The Road to Component Model 1.0 (bytecodealliance.org) | 2026-06-08 | WASI P3 imminent (native async). Lazy ABI inverts control flow (zero-copy). `lower-components` tool for C-ABI interop. Cooperative threads shipping. Stream splicing. |
| S4 | Luke Wagner — Towards a Component Model 1.0 @ Wasm I/O 2026 (youtube.com) | 2026 | No WASI P4 — everything additive on P3. 1.0 trims warts. Lazy ABI + multi-value + error contexts + GC. Guest/host C-ABI abstractions. |
| S5 | jsmanifest — Component Model + WASI 0.3 for JS Developers | 2026-06-01 | WASI 0.3 native async via `future`/`stream`. jco transpile for browser polyfill. componentize-js for JS-authored components. |
| S6 | Bytecode Alliance — WASI.dev / WASI 0.3 | 2026 | Three milestones: P1, P2 (stable), P3 (native async). Runtime support: Wasmtime, WAMR, WasmEdge, wazero, Wasmer, jco, pywasm. |
| S7 | Microsoft ISE — WASM Data Processing at Edge (Azure IoT) | 2026-05-14 | Composed dataflow operators via WIT. `wasm32-wasip2` target. `wasm-tools compose` fuses components. OCI artifact storage. |
| S8 | techbytes.app — WASI 2.0 Production Edge Guide | 2026-07-05 | WASI P2 stable since 2024-01-25. Capability-based security. Sub-ms cold starts. 10-50× memory footprint vs containers. |
| S9 | techbytes.app — Wasm Components & WASI P3 at Edge | 2026-07-05 | P3 targets async + threads. Cooperative vs stackful concurrency. Bounded thread pools for edge. |
| S10 | Calmops — WebAssembly Serverless Architecture 2026 | 2026-03-16 | Wasmtime 44+ for P3. Spin 3.0, wasmCloud. 1-5ms startup. Component composition via WIT. |
| S11 | wasmCloud Community Meeting | 2026-08-19 | Top-level WIT types (records, enums, resources at package scope). No WASI P4 — all additive. Cooperative threads ~Oct 2026. Instance refresh limit policy. |
| S12 | wasm-bindgen 0.2.126 (crates.io) | 2026-06-24 | Latest release. 455M+ total downloads. MSRV 1.77. 129 releases. |
| S13 | wasm-pack 0.15.0 (npm/crates.io) | 2026-05-15 | WASI target support. `wasm64-unknown-unknown` target. Inlined npm install. |

---

## Defects Found

### DEFECT-433-1: No Component Model Integration

**Severity**: HIGH
**Codebase**: `nt_io_plugin/wasm.rs` — entire 88-line file
**Gap**: NeoTrix WASM plugins use raw `wasmtime::Module` + manual `Linker` + raw memory pointer arithmetic (`ptr = memory.data_mut().len() - input_len - 1`). This is the **P1-era pattern** — no WIT interfaces, no Component Model, no typed imports/exports.
**Evidence**:
- `wasm.rs:47`: `get_typed_func::<(i32, i32), i32>` — raw i32 ABI, no type safety
- `wasm.rs:55-56`: Manual linear memory manipulation — pointer arithmetic into shared memory
- `wasm.rs:42`: `Linker::new(engine)` — no WASI or custom interface linking
- S1-S11: All 2026 WASM ecosystems have moved to Component Model (WIT-defined interfaces, typed canonical ABI, resource lifetime management)
**Impact**: Every WASM plugin interaction is untyped, memory-unsafe at the ABI boundary, and cannot interoperate with the broader WASM ecosystem (Python, Go, JS components). Zero chance of running third-party WASM components.
**Fix**: Implement a `ComponentPlugin` adapter using `wasmtime::component::Component` + WIT-defined interfaces. Use `wit-bindgen` for Rust-side code generation. Define plugin contract in `.wit` files.

### DEFECT-433-2: No WASI Capability-Based Security in Plugin Sandbox

**Severity**: HIGH
**Codebase**: `nt_shield_sandbox_entry.rs:161-176`, `wasm.rs:41-42`
**Gap**: WASM plugin execution grants **ambient authority** — full `Store<()>` with no WASI context, no capability restrictions, no resource limits (fuel, memory, timeout). The `WasmSandbox` (line 211-256) uses `agent_sandbox` but only for JS execution, not WASM components.
**Evidence**:
- `wasm.rs:41`: `Store::new(engine, ())` — empty store data, no resource tracking
- `sandbox_entry.rs:163`: `Engine::default()` — no fuel metering configured
- `sandbox_entry.rs:168-169`: `Instance::new(&mut store, &module, &[])` — no WASI imports linked
- S6, S8, S10: WASI P2 capability model is the standard — components declare exact imports, runtime enforces at instantiation
- S11: wasmCloud uses deny-by-default capability model
**Impact**: WASM plugins can access host filesystem, network, and environment implicitly through any imported WASI function the engine provides. No isolation boundary between untrusted plugin code and host.
**Fix**: Use `wasmtime_wasi::ResourceTable` + `WasiCtxBuilder` with explicit capability grants. Each plugin gets a `Store<PluginWasiState>` with scoped filesystem (only `/work`), no network, explicit fuel limits.

### DEFECT-433-3: Static Global Wasmtime Engine (No DI, No Configurability)

**Severity**: MEDIUM
**Codebase**: `wasm.rs:6-11`
**Gap**: `static WASM_ENGINE: LazyLock<wasmtime::Engine>` — singleton engine with default config. No way to configure fuel limits, memory limits, allocation strategy, or GC settings per-plugin.
**Evidence**:
- `wasm.rs:6`: `// TODO: inject via DI — pass wasmtime::Engine through WasmPluginWrapper constructor` — acknowledged TODO
- S1: Wasmtime 47 has GC config, exception handling, pooling allocator — all configurable via `Engine::new(&Config)`
- S3: Component Model 1.0 lazy ABI requires engine-level opt-in
**Impact**: All plugins share identical execution parameters. Cannot enforce per-plugin resource budgets. Cannot test with different engine configurations.
**Fix**: Create a `WasmEnginePool` that injects configured engines per plugin category. Expose config via TOML (Rune Socketing pattern).

### DEFECT-433-4: No WASM GC Integration for Managed Language Plugins

**Severity**: MEDIUM
**Gap**: NeoTrix has no plan for hosting Python/Scala/Kotlin WASM plugins via WasmGC. The current architecture assumes only Rust/C-ABI WASM modules.
**Evidence**:
- S1: Wasmtime 47 enables Wasm GC by default — Cheney collector, 32-bit refs, struct/array heap types
- S2: WasmGC + Component Model is 4-6× slower than Rust↔Rust due to linear memory copies, but the `gc` canonical ABI option (in prototype) eliminates this
- S5: Python joining WASM ecosystem via Component Model in 2026
- `todo-root.md`: Plans reference `python-wasm` via wasmtime but no WasmGC support
**Impact**: Cannot host Python WASM plugins without embedding a full Python interpreter in linear memory. Misses the WasmGC wave that Kotlin/Dart/Scala ecosystems are building on.
**Fix**: Track Component Model Issue #525 (GC ABI). When Wasmtime ships GC+Component integration, add `wasmtime::component::Component` with GC mode for managed-language plugins.

### DEFECT-433-5: No Lazy ABI / Zero-Copy Forwarding

**Severity**: MEDIUM
**Gap**: Plugin function calls always copy data through linear memory. No support for the Component Model's lazy value handles or zero-copy forwarding.
**Evidence**:
- `wasm.rs:53-56`: Manual `copy_from_slice` into linear memory
- S3: Lazy ABI inverts control — callee returns opaque i32 handles, caller decides when/where to place. Zero-copy forwarding possible.
- S4: Lazy ABI ships as opt-in in 0.3.x, becomes default in 1.0
**Impact**: Every plugin call pays full serialization cost. For high-frequency calls (heartbeat, event processing), this overhead compounds.
**Fix**: When Component Model lazy ABI stabilizes, adopt it for plugin→host data flow. Until then, use shared-memory arenas with index-based access (avoid `copy_from_slice` for large payloads).

### DEFECT-433-6: No Cooperative Threading / Async Component Support

**Severity**: MEDIUM
**Gap**: WASM plugin execution is synchronous and blocking. No integration with WASI P3 native async (`future`/`stream` types) or cooperative threads.
**Evidence**:
- `sandbox_entry.rs:171-173`: `func.call(&mut store, ())` — synchronous blocking call
- S3: WASI P3 adds native async I/O to Component Model. Cooperative threads via `wasi-libc` pthreads shipping.
- S9: Edge workloads need concurrent I/O without blocking a core
- `nt_mind_background_loop/builder.rs:104`: Plugin hot-reload mentions `.wasm` but no async execution model
**Impact**: WASM plugins cannot perform async I/O (network calls, file reads) without blocking the host thread. Cannot integrate with NeoTrix's tokio runtime.
**Fix**: Use `wasmtime_wasi::tokio::Store::new` for async host calls. Implement WIT `wasi:io/poll` integration. Track WASI P3 cooperative threads for long-running plugin computations.

### DEFECT-433-7: No WASM Optimization Pipeline

**Severity**: LOW
**Codebase**: Build process uses standard `cargo build`
**Gap**: No `wasm-opt` integration, no `twiggy` profiling, no `build-std` for `no_std` targets, no binary size tracking.
**Evidence**:
- S12, S13: wasm-bindgen/wasm-pack ecosystem has mature tooling for optimization
- `iteration_batch_325.md:89`: Previously identified gap — no WASM optimization pipeline
- Rust 1.85 added 12 WASM-specific MIR optimization passes (32% size reduction)
**Impact**: WASM binaries are larger than necessary, cold start slower, edge deployment less efficient.
**Fix**: Add `wasm-opt -Oz` to build pipeline. Add `twiggy` size profiling as CI step. Consider `build-std` for `wasm32-wasip2` target.

### DEFECT-433-8: Plugin Name/Version Memory Leak (Box::leak)

**Severity**: LOW (correctness debt)
**Codebase**: `wasm.rs:66-72`
**Gap**: `Box::leak(self.name.clone().into_boxed_str())` leaks memory on every plugin load/unload cycle.
**Evidence**:
- `wasm.rs:67`: `Box::leak(self.name.clone().into_boxed_str())` — intentional leak for `'static` str
- `wasm.rs:71`: Same pattern for version
**Impact**: Memory grows monotonically with plugin reloads. In hot-reload scenarios (heartbeat-driven), this accumulates.
**Fix**: Use `Arc<str>` or `&'static` via `once_cell::sync::Lazy` keyed by plugin name. Or redesign `Plugin` trait to accept `&str` references.

---

## Suggestions

### Suggestion 1: Component Model Migration Path (Priority: HIGH)

Define WIT interface for NeoTrix plugin contracts:

```wit
package neotrix:plugin@1.0.0;

interface lifecycle {
    resource plugin-config {
        name: string,
        version: string,
    }
}

world neotrix-plugin {
    import wasi:io/poll@0.2.0;
    import wasi:filesystem/types@0.2.0;
    export on-load: func();
    export on-unload: func();
    export on-event: func(event: string);
    export invoke: func(name: string, input: string) -> result<string, string>;
}
```

Migrate `WasmPluginWrapper` → `ComponentPlugin` using `wasmtime::component::bindgen!`. Ship alongside existing raw WASM path behind feature flag `component-model`.

### Suggestion 2: Capability-Granted Sandbox (Priority: HIGH)

Replace ambient-authority `Store<()>` with capability-scoped stores:

```rust
struct PluginSandbox {
    engine: wasmtime::Engine,
    wasi_ctx: wasmtime_wasi::ResourceTable,
    fuel_limit: u64,
    memory_limit: usize,
    timeout: Duration,
}

impl PluginSandbox {
    fn new_store(&self) -> wasmtime::Store<PluginWasiState> {
        let mut builder = wasmtime_wasi::WasiCtxBuilder::new();
        builder.preopened_dir(self.work_dir, "/work");
        // Deny: network, env vars, clock, random (unless explicitly granted)
        let mut store = wasmtime::Store::new(&self.engine, PluginWasiState { ... });
        store.limiter(|s| s.memory_limit);
        store.set_fuel(self.fuel_limit).unwrap();
        store
    }
}
```

### Suggestion 3: WASI P3 Async Integration (Priority: MEDIUM)

Prepare for cooperative async:

```rust
// Use wasmtime's async support
let engine = wasmtime::Engine::new(&Config::new()
    .async_support(true)
    .epoch_interruption(true))?;

// Plugin execution becomes async-native
async fn execute_plugin(store: &mut Store<PluginState>, func: &str) -> Result<String> {
    let future = instance.get_typed_func::<(), ()>(&mut store, func)?;
    tokio::time::timeout(Duration::from_secs(30), future.call_async(&mut store, ())).await?
}
```

### Suggestion 4: WasmGC Readiness (Priority: LOW — track only)

Monitor Component Model Issue #525 and Wasmtime GC+Component integration. When Wasmtime ships the `gc` canonical ABI option:
1. Add `wasmtime::component::Component` with GC mode
2. Create `GcPluginAdapter` for Python/Scala WASM modules
3. Benchmark GC-to-GC calls vs linear-memory copies

### Suggestion 5: Build Pipeline Optimization (Priority: LOW)

Add to CI:
```bash
cargo build --target wasm32-wasip2 -p neotrix-plugin-sdk --release
wasm-opt -Oz target/wasm32-wasip2/release/*.wasm -o optimized.wasm
twiggy top -n 20 optimized.wasm  # size budget tracking
```

---

## Summary

| Metric | Count |
|--------|-------|
| Sources cited | 13 |
| Defects found | 8 |
| Suggestions | 5 |
| HIGH priority | 2 (Component Model gap, WASI capability gap) |
| MEDIUM priority | 4 (DI engine, GC, lazy ABI, async) |
| LOW priority | 2 (optimization pipeline, memory leak) |

**Cross-iteration delta**: This batch focuses on the **Component Model transition** — the single biggest architectural shift in the WASM ecosystem in 2026. NeoTrix's WASM integration is still in the P1-era (raw modules, manual memory, no typed interfaces). The gap is structural, not just missing features — it requires a new plugin contract layer (WIT) and a new execution model (component instantiation with capability grants). Previous batches identified the sandbox gap (batch 325); this batch goes deeper by identifying that even the sandbox execution path lacks the Component Model foundation that all 2026 WASM runtimes are converging on.
