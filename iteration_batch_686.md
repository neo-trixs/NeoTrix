# Iteration Batch 686 — WebAssembly, WASI, Browser Extensions 2026

**Date**: 2026-09-06
**Context**: Building on Batch 685's serialization defects (serde no schema evolution, FlatBuffers unsafe builder, no unified serialization abstraction, KB format migration no strategy, rmp-serde no max-size validation).

---

## 1. WebAssembly 2026 — Key Findings & New Defects

### 1.1 WasmGC + Component Model = Structured Serialization Without Serde

**Source**: [dev.to — Rust & WASM in 2026](https://dev.to/dataformathub/rust-wasm-in-2026-a-deep-dive-into-high-performance-web-apps-20c6); [zylos.ai — WebAssembly Ecosystem 2026](https://zylos.ai/research/2026-02-05-webassembly-ecosystem-2026); [byteiota — WebAssembly 3.0 Production Checklist](https://byteiota.com/webassembly-3-wasmgc-memory64-production-checklist)

**What's New**: WebAssembly 3.0 (W3C standard Sep 2025) ships WasmGC, Memory64, exception handling, and Relaxed SIMD. WasmGC lets Wasm modules declare typed struct/array heap objects managed by the host engine's GC — **eliminating per-module GC bundling**. Component Model (stable 2025-2026) uses WIT (WebAssembly Interface Types) as a language-neutral IDL with **zero-cost interop via value-type lifting/lowering** — no manual serialization needed between composed components.

**Defect D-686-01**: **No WIT-based schema for KB interop**. NeoTrix uses Serde derive macros for KB node/edge serialization with no type evolution. The Component Model's WIT provides a **typed, versioned, language-neutral interface contract** that inherently solves schema evolution (interfaces can declare backward-compatible additions). NeoTrix has zero WIT definitions — KB data cannot be safely consumed by external WASM toolchains (Python, Go, Java data pipelines).

**Defect D-686-02**: **Memory64 breaks FlatBuffers size assumptions**. WebAssembly 3.0's Memory64 breaks the 4GB linear memory barrier. NeoTrix's rmp-serde max-size validation (batch 685) assumes 32-bit address space limits. If any Wasm-hosted KB component uses Memory64, the OOM validation threshold becomes incorrect. No runtime detection of address space width before applying size limits.

**Defect D-686-03**: **WasmGC typed references bypass unsafe builder pattern**. FlatBuffers builder uses `unsafe` (violating R-P1). WasmGC's typed references (structs, arrays managed by host GC) provide a **safe alternative** to manual memory management in serialization — objects passed by reference with host-managed lifetime, zero FFI overhead. NeoTrix hasn't evaluated WasmGC as a safe replacement for FlatBuffers' unsafe builder.

### 1.2 WASI 0.2/0.3 + Component Model = Portable Capability-Based Security

**Source**: [youngju.dev — Wasm Ecosystem 2026 Complete Guide](https://www.youngju.dev/blog/culture/2026-05-16-webassembly-wasm-ecosystem-2026-wasmtime-wasmedge-wasmer-wasi-component-model-spin-fermyon-bytecode-alliance-deep-dive.en); [javacodegeeks — WebAssembly in 2026](https://www.javacodegeeks.com/2026/04/webassembly-in-2026-where-it-has-landed-what-wasi-0-2-changes-and-why-java-and-kotlin-developers-should-pay-attention-now.html)

**What's New**: WASI Preview 2 (GA Jan 2024) provides capability-based security: resource handles passed explicitly, no ambient authority. WASI 0.3 targets late 2026 with native async I/O. WASI 1.0 planned for 2026 with production stability guarantees. Component registries (WARG) enable signed, trusted distribution of Wasm components.

**Defect D-686-04**: **NT-SHIELD sandbox lacks WASI capability-based model**. NeoTrix's `nt_shield_sandbox` defines egress policies (allow/deny host/port rules) but has no capability-based resource model. WASI's approach — explicitly grant filesystem/network/clock capabilities per-component — is a stronger security primitive than NeoTrix's allow/deny list. NeoTrix components can't be isolated by capability; they inherit full ambient authority.

**Defect D-686-05**: **No WIT interface contracts for domain module composition**. The 7 NeoTrix domains (NT-CORE, NT-MIND, etc.) communicate via Rust trait objects with no formal interface definition. WASI Component Model's WIT provides **formal, versioned, language-neutral interface contracts** — each domain could export/import typed capabilities with backward-compatible evolution. NeoTrix's inter-domain coupling is implicit and fragile.

### 1.3 Wasm Performance + SIMD for Perception/Compute Pipelines

**Source**: [dualmedia — WebAssembly 2026 Browser](https://www.dualmedia.com/webassembly-2026-browser/); [architecturediagram.ai — Wasm Architecture Diagrams](https://architecturediagram.ai/blog/wasm-architecture-diagram)

**What's New**: WASM + SIMD achieves 10-15x speedups over JS for image/video/ML inference workloads. WASM components achieve ~10µs inter-component latency vs ~1ms for HTTP microservices. Startup time: sub-10ms cold starts on edge runtimes vs seconds for containers.

**Defect D-686-06**: **NT-WORLD perception pipeline doesn't leverage WASM SIMD for content parsing**. NeoTrix's UnifiedCrawler parses HTML/markdown in native Rust. WASM SIMD-accelerated parsing could enable browser-hosted content extraction with near-native performance, enabling NT-WORLD to run as a Wasm component in browser extensions or edge workers.

---

## 2. WASI + Component Model 2026 — Key Findings & New Defects

### 2.1 Component Composition Without Serialization Overhead

**Source**: [tech.webnet17 — Mastering Wasm Component Model 2026](https://tech.webnet17.com/mastering-the-webassembly-component-model-in-2026/); [devstarsj — Wasm Components 2026](https://devstarsj.github.io/2026/02/19/wasm-components-2026-server-revolution/)

**What's New**: Component Model eliminates serialization overhead between composed modules. WIT interfaces define typed contracts; value types are lifted/lowered at the boundary with zero-copy semantics. A Rust component exporting `process-transaction(tx: Transaction) -> RiskAssessment` can be consumed by a Python component without JSON/protobuf/message-pack.

**Defect D-686-07**: **NeoTrix inter-domain communication is serialization-heavy**. Domains communicate via KB queries (SQLite rows → Serde → Rust structs → Serde → SQLite rows). The Component Model pattern eliminates this round-trip: domains could be Wasm components sharing typed values directly. NeoTrix's architecture has unnecessary serialization hops in every cross-domain call.

### 2.2 Component Registry + Signing = Trust Chain for Plugin Ecosystem

**Source**: [beyondtmrw — Wasm Component Model 2026 Plugins](https://beyondtmrw.org/article/webassembly-component-model-cross-language-plugins-and-browser-sandboxing-in-2026); [youngju.dev — Wasm Ecosystem Deep Dive](https://www.youngju.dev/blog/culture/2026-05-25-webassembly-wasi-spin-wasmtime-wasmer-wasmedge-component-model-2026-deep-dive.en)

**What's New**: Bytecode Alliance's WARG (WebAssembly Registry) provides OCI-based component distribution with signing and versioning. Browser extensions can load sandboxed Wasm components as plugins. Chrome/Firefox have component prototype support.

**Defect D-686-08**: **No signed plugin/extension mechanism for NeoTrix skill system**. NeoTrix skills (rev-officer, dev-implementer, etc.) are loaded from local markdown files with no integrity verification. WASI's component registry + signing provides a trust chain: skills could be distributed as signed Wasm components with provenance tracking. Current skill loading is a supply chain risk.

### 2.3 Thread Limitations in WASI

**Source**: [zylos.ai — WebAssembly Ecosystem 2026](https://zylos.ai/research/2026-02-05-webassembly-ecosystem-2026); [byteiota — WebAssembly 3.0 Checklist](https://byteiota.com/webassembly-3-wasmgc-memory64-production-checklist)

**What's New**: WASI threading is a separate proposal without a confirmed 2026 date. Cloudflare Workers don't support threading. Shared memory + atomic instructions exist at the Wasm level but WASI doesn't expose multi-threaded system calls portably. This limits Wasm as a general-purpose backend runtime.

**Defect D-686-09**: **WASM-only deployment breaks NT-MIND SEAL pipeline parallelism**. The SEAL pipeline (exploration→distillation→self-test→absorption) relies on multi-threaded parallel task execution. WASI's threading limitation means NeoTrix cannot deploy the SEAL pipeline as a Wasm component without losing parallelism. No fallback strategy for single-threaded WASM execution.

---

## 3. Browser Extensions 2026 — Key Findings & New Defects

### 3.1 Manifest V3 + Service Worker Lifecycle

**Source**: [developer.chrome.com — I/O 2026 Extensions Recap](https://developer.chrome.com/blog/extensions-io-2026); [medium — MV3 in Practice](https://medium.com/@katsuya.ds/chrome-extension-mv3-in-practice-where-individual-developers-actually-get-stuck-service-workers-fa83899f3d88)

**What's New**: MV2 fully removed from Chrome Web Store August 31, 2026. Service workers terminate after ~30 seconds of inactivity; global variables wiped on termination. `chrome.storage.session` provides in-memory persistence across service worker lifecycle. CSP tightened: all logic must be bundled; no remote code execution. `browser.*` namespace now supported cross-browser (Chrome, Firefox, Edge).

**Defect D-686-10**: **NT-IO extension interface assumes persistent background process**. NeoTrix's web server (NT-IO) assumes long-lived server processes for API endpoint handling. If NeoTrix were packaged as a browser extension, the MV3 service worker lifecycle would break stateful API handlers. No architecture for stateless, event-driven extension endpoints.

**Defect D-686-11**: **Service worker 30s timeout breaks KB query transactions**. NeoTrix KB operations (multi-step transactions: read→transform→write) can take >30s for large datasets. If run inside an MV3 service worker, these transactions would be terminated mid-operation. No transactional checkpoint/resume mechanism for long-running KB operations.

### 3.2 Extension Malware + Supply Chain Attacks (2026 Campaign)

**Source**: [itechify — Chrome Extensions Malware 2026](https://itechify.com/2026/08/31/chrome-extensions-malware-2026/); [deepstrike — Chrome Extension Security](https://deepstrike.io/blog/chrome-extensions-security-threats-risk-analysis); [checkyourvibe — MV3 Security Checklist](https://checkyourvibe.dev/blog/checklists/chrome-extension-checklist)

**What's New**: Major 2026 campaign: extensions passed Google review, then activated malicious modules weeks later. Attack modules: CSP stripping (removed security headers), credential harvesting, crypto wallet draining, browsing history exfiltration. Exploited extension update mechanism (updates receive less rigorous review than initial submissions). SquareX (DEF CON 32) demonstrated MV3 bypass techniques.

**Defect D-686-12**: **No runtime integrity verification for NeoTrix skills/extensions**. The 2026 malware campaign exploits the gap between initial review and post-install behavior. NeoTrix skills are loaded from filesystem with no runtime integrity checks (hash verification, behavioral monitoring). A compromised skill could execute arbitrary code during NeoTrix sessions with full system access.

**Defect D-686-13**: **MV3 CSP model conflicts with NeoTrix's dynamic code execution**. NeoTrix's `nt_mind` distillation pipeline generates and executes code dynamically. MV3's CSP (`script-src 'self'`) prohibits remote/eval'd code. If NeoTrix skills were Wasm components in a browser extension, the distillation pipeline's dynamic code generation would be blocked by CSP. No sandboxed execution path for generated code.

### 3.3 Cross-Browser Extension Divergence

**Source**: [youngju.dev — Browser Extension Development 2026](https://www.youngju.dev/blog/culture/2026-05-16-browser-extension-development-2026-manifest-v3-plasmo-wxt-side-panel-api-deep-dive.en); [alphonsolabs — Browser Trends 2026](https://www.alphonsolabs.com/browser-trends-2026/)

**What's New**: Chrome MV3: service workers, declarativeNetRequest, `browser.*` namespace. Firefox MV3: event page model (not service worker), retains blocking webRequest (why uBlock Origin works). Safari: WebKit-based, some APIs unsupported, iOS/iPadOS support (Safari 15+). "Write once, run everywhere" is only half true. `webextension-polyfill` bridges API gaps but doesn't resolve architectural differences.

**Defect D-686-14**: **No cross-browser abstraction layer for NeoTrix IO**. NeoTrix's NT-IO targets single runtime (Tauri desktop or native CLI). If deployed as browser extension, the Chrome/Firefox/Safari divergence in service worker lifecycle, API namespaces, and blocking capabilities would require a per-browser adapter layer. No abstraction exists for this.

---

## 4. Summary: New Defects (Batch 686)

| ID | Domain | Defect | Severity | Source |
|---|---|---|---|---|
| D-686-01 | NT-MEMORY | No WIT-based schema for KB interop (schema evolution gap) | HIGH | Component Model |
| D-686-02 | NT-MEMORY | Memory64 breaks rmp-serde size validation thresholds | MEDIUM | Wasm 3.0 |
| D-686-03 | NT-CORE | WasmGC typed refs provide safe alternative to FlatBuffers unsafe | MEDIUM | WasmGC |
| D-686-04 | NT-SHIELD | No WASI capability-based security model for sandbox | HIGH | WASI 0.2 |
| D-686-05 | NT-CORE | No WIT interface contracts for domain composition | HIGH | Component Model |
| D-686-06 | NT-WORLD | Perception pipeline doesn't leverage WASM SIMD | LOW | SIMD |
| D-686-07 | NT-CORE | Inter-domain communication is serialization-heavy | MEDIUM | Component Model |
| D-686-08 | NT-MEMORY | No signed plugin/skill distribution mechanism | HIGH | WARG Registry |
| D-686-09 | NT-MIND | WASM threading gap breaks SEAL parallelism | HIGH | WASI Threading |
| D-686-10 | NT-IO | Extension interface assumes persistent background process | MEDIUM | MV3 Lifecycle |
| D-686-11 | NT-MEMORY | Service worker timeout breaks KB transactions | HIGH | MV3 30s timeout |
| D-686-12 | NT-SHIELD | No runtime integrity verification for skills | CRITICAL | 2026 malware campaign |
| D-686-13 | NT-MIND | MV3 CSP blocks dynamic code distillation | HIGH | MV3 CSP |
| D-686-14 | NT-IO | No cross-browser extension abstraction layer | MEDIUM | Browser divergence |

---

## 5. What's NEW (Not in Prior Batches)

1. **WIT as schema evolution solution** (D-686-01): Component Model's WIT interfaces provide typed, versioned contracts that inherently solve the "serde has no schema evolution" problem from batch 685. NeoTrix should define WIT schemas for all domain interfaces.

2. **WasmGC as safe FlatBuffers replacement** (D-686-03): Host-managed GC + typed references eliminate the need for unsafe FlatBuffers builders. Directly addresses R-P1 violation.

3. **WASI capability model for NT-SHIELD** (D-686-04): Capability-based security is architecturally superior to allow/deny lists. NeoTrix sandbox should adopt explicit resource grants per component.

4. **WARG signed skill distribution** (D-686-08): WASI component registries provide provenance + integrity. NeoTrix skills are currently unsigned filesystem files — a supply chain attack vector.

5. **MV3 30s timeout × KB transactions** (D-686-11): Long-running KB operations incompatible with service worker lifecycle. Requires transactional checkpoint/resume.

6. **2026 extension malware campaign → NeoTrix skill integrity** (D-686-12): Real-world attack (Chrome Web Store, Aug 2026) validates the risk. Extensions passed review then activated malicious modules. NeoTrix skills have no equivalent protection.

7. **MV3 CSP × dynamic code distillation** (D-686-13): Fundamental conflict between CSP's no-eval policy and NeoTrix's code generation pipeline. Requires Wasm sandbox execution path.

8. **Memory64 × size validation** (D-686-02): 64-bit address space makes 32-bit size assumptions invalid. OOM guards must detect address width.

---

## Sources Cited

1. https://dev.to/dataformathub/rust-wasm-in-2026-a-deep-dive-into-high-performance-web-apps-20c6
2. https://zylos.ai/research/2026-02-05-webassembly-ecosystem-2026
3. https://byteiota.com/webassembly-3-wasmgc-memory64-production-checklist/
4. https://www.dualmedia.com/webassembly-2026-browser/
5. https://www.javacodegeeks.com/2026/04/webassembly-in-2026-where-it-has-landed-what-wasi-0-2-changes-and-why-java-and-kotlin-developers-should-pay-attention-now.html
6. https://www.youngju.dev/blog/culture/2026-05-16-webassembly-wasm-ecosystem-2026-wasmtime-wasmedge-wasmer-wasi-component-model-spin-fermyon-bytecode-alliance-deep-dive.en
7. https://www.youngju.dev/blog/culture/2026-05-25-webassembly-wasi-spin-wasmtime-wasmer-wasmedge-component-model-2026-deep-dive.en
8. https://tech.webnet17.com/mastering-the-webassembly-component-model-in-2026/
9. https://beyondtmrw.org/article/webassembly-component-model-cross-language-plugins-and-browser-sandboxing-in-2026
10. https://devstarsj.github.io/2026/02/19/wasm-components-2026-server-revolution/
11. https://architecturediagram.ai/blog/wasm-architecture-diagram
12. https://developer.chrome.com/blog/extensions-io-2026
13. https://medium.com/@katsuya.ds/chrome-extension-mv3-in-practice-where-individual-developers-actually-get-stuck-service-workers-fa83899f3d88
14. https://itechify.com/2026/08/31/chrome-extensions-malware-2026/
15. https://deepstrike.io/blog/chrome-extensions-security-threats-risk-analysis
16. https://checkyourvibe.dev/blog/checklists/chrome-extension-checklist
17. https://www.alphonsolabs.com/browser-trends-2026/
18. https://www.youngju.dev/blog/culture/2026-05-16-browser-extension-development-2026-manifest-v3-plasmo-wxt-side-panel-api-deep-dive.en
19. https://internet-pros.com/blog/webassembly-wasi-component-model-2026/
20. https://pockit.tools/blog/webassembly-wasi-2-component-model-beyond-browser-guide
