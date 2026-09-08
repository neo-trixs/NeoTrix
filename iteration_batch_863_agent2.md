# Agent 2: WASM Component Model (Batch 863)

## Sources
1. https://component-model.bytecodealliance.org/language-support/building-a-simple-component/rust.html — Official Rust Component Model tutorial
2. https://blog.yoshuawuyts.com/why-webassembly-components — Yosh Wuyts on Rust 2026 Wasm Component goals
3. https://docs.wasmtime.dev/api/wasmtime/component/index.html — Wasmtime Component Model embedding API
4. https://component-model.bytecodealliance.org/design/component-model-concepts.html — Component Model concepts (worlds, WIT, interfaces)
5. https://wasi.dev/releases/wasi-p3 — WASI 0.3 release notes (June 2026)
6. https://github.com/bytecodealliance/component-docs/blob/main/component-model/src/design/migrating-to-p3.md — WASI 0.2→0.3 migration guide
7. https://github.com/bytecodealliance/component-docs/blob/main/component-model/src/design/async.md — Native async with WASI 0.3
8. https://techtalknews.com/posts/wasm-components-microservices — WASM Components production adoption (Akamai 75M req/s)
9. https://maddevs.io/writeups/wasm-component-model-practical-guide-part-two/ — Component Model production trade-offs
10. https://muhammadamal.my.id/blog/wasm-component-model-in-production-practical-walkthrough/ — Production WIT discipline patterns
11. https://thebackenddevelopers.substack.com/p/webassembly-server-side-component — Server-side Wasm: wasmCloud and Spin
12. https://gothar.com/en/insights/wasm-component-model-edge-2026 — Edge computing with Component Model
13. https://github.com/sarmakska/sandboxd — wasmtime sandbox with deny-by-default, fuel+epoch watchdog
14. https://softwarepatternslexicon.com/rust/security-patterns/sandboxing-and-isolation-patterns/ — Rust sandboxing patterns
15. https://deepwiki.com/nearai/ironclaw/8.2-wasm-sandbox-security — IronClaw WASM sandbox security (capability-based, secrets never in WASM memory)

## Defects

**D-WASM-001: wasmtime v42 pinned; WASI 0.3 async/component-model-async unavailable** | `neotrix-core/Cargo.toml:94` | P2-HIGH | Source: WASI 0.3 release notes, Wasmtime 46 changelog. NeoTrix pins `wasmtime = { version = "42" }` (optional, behind `sandbox` feature). Wasmtime 46 (June 2026) enables WASI 0.3 and `component-model-async` by default. Wasmtime 42 has NO WASI 0.3 support and only experimental async via the `0.3.0-rc-2026-01-06` snapshot. This blocks any component-model-async adoption and leaves the sandbox on a deprecated ABI surface.

**D-WASM-002: No WIT interface definitions for cross-domain sandbox isolation** | `nt_shield_sandbox/mod.rs:76-173` | P2-HIGH | Source: Component Model concepts, production walkthrough. NeoTrix defines `EgressPolicy`/`EgressRule` as plain Rust structs with string-based host matching. The WASM Component Model defines typed WIT interfaces with capability-based imports—components only access what the host explicitly grants via WIT imports. NeoTrix has no WIT files defining sandbox boundaries, meaning cross-domain isolation is ad-hoc Rust trait checks rather than formally typed ABI contracts enforced at composition time.

**D-WASM-003: Sandbox capability model lacks deny-by-default with explicit capability grants** | `nt_shield_sandbox/device.rs:68-74` | P1-CRITICAL | Source: sandboxd design decisions, IronClaw WASM security. Best practice (sandboxd, IronClaw) is deny-by-default: the guest gets NO capabilities unless explicitly granted. NeoTrix `DeviceSandbox` uses `SandboxSpec::default()` with broad defaults and `EgressPolicy::permissive()` as a common fallback. The `default_allow: false` flag in `EgressPolicy` is opt-in rather than the enforced default. This violates the principle that WASM sandboxing should start from zero capabilities and add only what is needed.

**D-WASM-004: No fuel/epoch CPU bounding for sandboxed execution** | `nt_shield_sandbox/device.rs:145-212` | P2-HIGH | Source: sandboxd, wasm-sandbox patterns. Production WASM sandboxes require THREE independent fences: fuel metering (deterministic instruction count), epoch interruption (wall-clock watchdog), and memory limits. NeoTrix `DeviceSandbox::evaluate()` performs logical egress checks but has no evidence of wasmtime fuel consumption or epoch interruption configuration. A malicious or infinite-looping guest can exhaust host CPU without any runtime-level termination mechanism.

**D-WASM-005: No Component Model composition for NT-CORE subsystem isolation** | `neotrix-core/src/neotrix/mod.rs:68-90` | P1-CRITICAL | Source: Component Model production patterns, Beyond Containers. The WASM Component Model's killer feature is composing independently compiled, memory-isolated subsystems via typed WIT interfaces within a single process. NeoTrix's 6-layer architecture (L1-L6) runs all domains in a single Rust process with no memory isolation between NT-SHIELD, NT-ACT, NT-CORE, etc. A panic or OOM in one domain can crash the entire consciousness architecture. Component Model composition would isolate each domain as a separate `.wasm` component with typed WIT boundaries.

**D-WASM-006: Cross-component async not addressed; wasi:io poll pattern used instead of native async** | `nt_shield_sandbox/mod.rs:1-4` | P3-MEDIUM | Source: WASI 0.3 release, migrating-to-p3.md. WASI 0.3 (June 2026) deletes `wasi:io` entirely, replacing pollables with native `stream`, `future`, and `async func`. NeoTrix's sandbox uses synchronous `std::net::TcpStream` and `Duration`-based timeouts—no async primitives compatible with WASI 0.3's completion-based concurrency model. For I/O-heavy sandbox workloads (LLM calls, network crawls), this blocks efficient concurrent execution across component boundaries.

**D-WASM-007: EgressPolicy string-based matching lacks typed WIT interface validation** | `nt_shield_sandbox/mod.rs:76-131` | P2-HIGH | Source: production WIT discipline, component-model-concepts. Production WASM systems encode network policy as WIT import constraints—e.g., `import wasi:http/outgoing-handler` with host-side policy enforcement. NeoTrix `EgressRule` uses raw string matching (`host: "api.github.com"`, `port: "443"`) with no typed contract. This means policy violations are runtime string mismatches rather than compile-time WIT type errors. Adding a new intel source requires manual rule creation rather than WIT package versioning.

**D-WASM-008: No WASM component registry or versioned WIT packages for domain capabilities** | `nt_shield_sandbox/mod.rs:262-379` | P2-HIGH | Source: production walkthrough (wkg, OCI registries), muhammadamal patterns. Production WASM systems publish WIT packages to OCI registries with semver versioning—each domain's capabilities are versioned and composable. NeoTrix hardcodes ~15 `*_egress_policy()` functions with hardcoded host strings. There is no registry, no versioning, and no WIT package structure. Adding or changing an intel source requires code changes rather than WIT package updates.

**D-WASM-009: No multi-tenant sandbox isolation between consciousness domains** | `nt_shield_sandbox/device.rs:68-74` | P1-CRITICAL | Source: WASM production patterns, server-side Wasm guide. WASM sandboxing provides per-instance linear memory isolation—each plugin/component runs in its own memory space. NeoTrix's architecture runs NT-CORE, NT-MIND, NT-WORLD, NT-ACT, NT-SHIELD in a shared address space. A memory corruption in NT-WORLD crawlers could theoretically corrupt NT-CORE consciousness state. The Component Model's memory isolation would prevent this.

**D-WASM-010: Missing stream/future resource lifecycle management for cross-component data flow** | `nt_shield_sandbox/mod.rs:403-493` | P3-MEDIUM | Source: WASI 0.3 stream/future primitives, Component Model async.md. WASI 0.3 defines `stream<T>` and `future<T>` as Canonical ABI values that cross component boundaries without ownership transfer. NeoTrix's `EgressContext` uses owned `HashMap` and `Arc`-based state sharing—no stream/future abstractions for cross-domain data pipelines. This blocks efficient streaming data flow between NT-WORLD (perception) and NT-ACT (action) without materializing entire payloads in memory.

**D-WASM-011: Sandbox feature is opt-in, not default; production deployments may run without isolation** | `neotrix-core/Cargo.toml:180` | P2-HIGH | Source: sandboxd, wasm-sandbox patterns. The `sandbox` feature flag (`sandbox = ["dep:wasmtime", "dep:agent-sandbox"]`) makes WASM isolation optional. Production best practice (sandboxd, IronClaw) is that sandboxing is the DEFAULT, not opt-in. A NeoTrix deployment compiled without the `sandbox` feature has zero WASM isolation—all tool execution runs in the host process with full capabilities.

**D-WASM-012: No WIT-defined resource handles for cross-domain memory lifecycle** | `nt_shield_sandbox/device.rs:145-212` | P2-HIGH | Source: Component Model resource lifecycle, production walkthrough. The Component Model defines `resource` types that cross component boundaries with explicit ownership semantics—the host manages resource lifetime, preventing use-after-free. NeoTrix's `SandboxSession` and `DeviceTool` use Rust ownership but have no WIT resource definitions for cross-domain handles. If a domain exposes a resource (e.g., a database connection) to another domain, there is no typed lifecycle contract.

## Key Insights

1. **WASI 0.3 is production-ready (shipped June 2026)**: Wasmtime 46 enables `component-model-async` by default. The `wasi:io` package is deleted; native `stream`/`future`/`async func` are the standard. NeoTrix's wasmtime v42 is 4 major versions behind and cannot use any of these features.

2. **Component Model composition is the missing architecture layer**: NeoTrix's 6-layer consciousness architecture maps naturally to WASM components—each layer (L1-L6) or domain (NT-CORE, NT-SHIELD, etc.) could be a separate `.wasm` component with typed WIT interfaces. This would provide memory isolation, typed cross-domain communication, and language-agnostic composition.

3. **Capability-based security requires deny-by-default**: Production WASM sandboxes (sandboxd, IronClaw, wasm-sandbox) all enforce deny-by-default with explicit capability grants. NeoTrix's current approach of starting with broad defaults and opting into restrictions is the opposite pattern.

4. **Three independent CPU fences are required**: Fuel (deterministic instruction count) + epoch interruption (wall-clock watchdog) + memory limits are the minimum production security triad. NeoTrix has none of these at the WASM runtime level.

5. **WIT interfaces replace string-based policy**: Type-checked WIT imports catch policy violations at composition time rather than runtime. NeoTrix's string-based `EgressRule` matching should be replaced with WIT-defined capability interfaces.

6. **Akamai/Fermyon scale validates the approach**: 75M requests/second on Akamai's edge with Spin Wasm components proves the Component Model handles production-scale workloads. The density advantage (200MB for 1,000 idle components vs 20-100GB for containers) is directly relevant to NeoTrix's multi-domain architecture.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 12 |
| Sources consulted | 15 |
| P1-CRITICAL defects | 3 |
| P2-HIGH defects | 7 |
| P3-MEDIUM defects | 2 |
