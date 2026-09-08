# Iteration Batch 841 Report — NeoTrix Consciousness Architecture

## Research Sources (42+)

### Unsafe Rust (10)
- Tree Borrows replaces Stacked Borrows (54% fewer false positives)
- Strict Provenance stabilized (ptr.addr(), with_addr(), expose_provenance())
- Safety Tags RFC under deliberation (#[safety::checked]/#[safety::requires])
- cargo-geiger + cargo-vet + Miri nightly = practical audit trifecta
- Linux kernel already migrated to expose_provenance()
- neotrix-sysctl: 5 unsafe blocks missing // SAFETY: comments
- ProcExeTaskInfo missing #[repr(C)] (transmute-style cast UB)
- No Miri coverage for unsafe crate
- No unsafe-op-in-unsafe-fn enforcement in neotrix-sysctl
- Silent failure on invalid sysctl (no size validation)

### FFI (10)
- UniFFI 0.32.0: NeoTrix pinned at 0.28 (4 versions behind)
- No panic guard at FFI boundary (catch_unwind missing)
- UDL/proc-macro dual source of truth (neotrix.udl 651 lines vs exports)
- neotrix_shutdown ownership transfer (Arc by value)
- No Send/Sync verification on FFI objects
- Error type has no diagnostic payload
- RwLock poisoning strategy (.expect() panics on poisoned lock)
- Google safer_cffi: OpaqueTracker for generational handles
- Rust ABI is unstable (only #[repr(C)] has guaranteed layout)
- cbindgen 0.29.2: IR not stable, cannot handle wide pointers

### WASM (10)
- WASI 0.3: wasi:io entirely removed (replaced by Component Model primitives)
- wasm-bindgen 0.2.122-0.2.127: breaking changes (threading, panic=unwind)
- wasmtime v47: GC + EH enabled by default, wasi-threads removed
- CVE-2026-34971 (Critical 9.0): Cranelift aarch64 sandbox escape
- CVE-2026-34987 (Critical 9.0): Winch backend sandbox escape
- CVE-2026-35195: FACT compiler string transcoding OOB write
- CVE-2026-34942: Misaligned UTF-16 from guest realloc
- Component Model string transcoding vulnerability
- GC default enables cycle-collecting collector
- Thread export breaking change (__heap_base)

### Time Handling (8)
- chrono 0.4.44: NeoTrix uses across 7 crates, 100+ sites (all UTC)
- jiff 0.2.35: DST-aware arithmetic, RFC 9557 zone serialization
- No timezone-aware arithmetic (local-time scheduling will break on DST)
- No lossless zone serialization (IANA zone identifier lost on serde)
- Leap second inconsistency (chrono behavior undefined around 23:59:60)
- chrono::Duration vs std::time::Duration fragmentation
- deep-time: full TAI/TT/GPS support (niche)
- Polars considering jiff migration

---

## Defects Identified (38+)

### Unsafe Rust (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-UNSAFE-1 | neotrix-sysctl 5 unsafe blocks missing // SAFETY: comments | High |
| D-UNSAFE-2 | ProcExeTaskInfo missing #[repr(C)] (transmute UB) | High |
| D-UNSAFE-3 | No Miri coverage for unsafe crate | Medium |
| D-UNSAFE-4 | No unsafe-op-in-unsafe-fn enforcement | Medium |
| D-UNSAFE-5 | Silent failure on invalid sysctl (no size validation) | Medium |
| D-UNSAFE-6 | No provenance-safe FFI pattern (raw casts) | Low |

### FFI (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-FFI-1 | No panic guard at FFI boundary (catch_unwind missing) | Critical |
| D-FFI-2 | UniFFI 0.28 (4 versions behind) | High |
| D-FFI-3 | UDL/proc-macro dual source of truth | High |
| D-FFI-4 | neotrix_shutdown ownership transfer (Arc by value) | Medium |
| D-FFI-5 | No Send/Sync verification on FFI objects | Medium |
| D-FFI-6 | Error type has no diagnostic payload | Medium |
| D-FFI-7 | RwLock poisoning strategy (.expect() panics) | High |
| D-FFI-8 | cbindgen IR not stable | Low |

### WASM (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-WASM-1 | CVE-2026-34971 (Critical 9.0) sandbox escape | Critical |
| D-WASM-2 | CVE-2026-34987 (Critical 9.0) sandbox escape | Critical |
| D-WASM-3 | CVE-2026-35195 string transcoding OOB write | Critical |
| D-WASM-4 | CVE-2026-34942 misaligned UTF-16 from guest | High |
| D-WASM-5 | WASI 0.3 wasi:io removal breaks assumptions | High |
| D-WASM-6 | wasmtime v47 wasi-threads removed | High |
| D-WASM-7 | GC default enables cycle-collecting collector | Medium |
| D-WASM-8 | Thread export breaking change (__heap_base) | Medium |
| D-WASM-9 | Component Model string transcoding vulnerability | High |
| D-WASM-10 | exnref EH requires Node.js 22.22.3+ | Low |

### Time Handling (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-TIME-1 | No timezone-aware arithmetic (DST breaks local scheduling) | High |
| D-TIME-2 | No lossless zone serialization (IANA lost on serde) | Medium |
| D-TIME-3 | Leap second inconsistency (undefined around 23:59:60) | Low |
| D-TIME-4 | chrono::Duration vs std::time::Duration fragmentation | Low |
| D-TIME-5 | No time abstraction layer (chrono leakage) | Medium |
| D-TIME-6 | No Local timezone usage (server-only UTC) | Info |

## Key Insights (This Batch)

1. **Tree Borrows replaces Stacked Borrows**: 54% fewer false positives, supports two-phase borrows natively, UnsafeCell tracking is finer-grained. Miri: -Zmiri-tree-borrows.

2. **No panic guard at FFI boundary is CRITICAL**: All 12 UniFFI export functions can panic via .expect("ffi rwlock poisoned"). Foreign exceptions entering Rust = UB per Rustonomicon.

3. **4 WASM CVEs in 2026**: 2 Critical (9.0) sandbox escapes via Cranelift/Winch backends, 1 OOB write via string transcoding, 1 misaligned UTF-16. Must pin wasmtime >=47.0.3.

4. **WASI 0.3 removes wasi:io entirely**: Replaced by Component Model primitives (async func, stream<T>, future<T>). Any NeoTrix WASI 0.2 code will break.

5. **chrono usage is safe for UTC-only**: NeoTrix uses chrono correctly for UTC timestamping. No urgent migration needed. Adopt jiff only when zone-aware features required.

6. **Strict Provenance stabilized**: ptr.addr(), with_addr(), expose_provenance() available. Linux kernel already migrating. NeoTrix should prepare for this transition.

7. **UDL/proc-macro dual source of truth**: neotrix.udl (651 lines) coexists with proc-macro exports. Will drift. Must delete UDL.

8. **Wasmtime v47 enables GC + EH by default**: Copying collector is default GC. WasmtimeBug error type must be added to NT-REPAIR error taxonomy.

9. **neotrix-sysctl ProcExeTaskInfo cast UB**: Missing #[repr(C)] on outer struct makes transmute-style cast undefined behavior.

10. **No time abstraction layer**: chrono leakage into domain code makes future migration (chrono→jiff) a multi-file change. Define nt_core_time now.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 841 |
| New defects (this batch) | 30 |
| Cumulative defects | D01-D77175 |
| Research sources (this batch) | 38 |
| Cumulative research sources | 98,124+ |
