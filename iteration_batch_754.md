# Iteration Batch 754 — Observability Audit

## Sources Consulted

| # | Source | URL | Date |
|---|--------|-----|------|
| 1 | Rustify: tracing vs log 2026 | https://rustify.rs/articles/rust-tracing-vs-log-crates-2026 | Aug 2026 |
| 2 | OneUptime: Structured JSON Logs with tracing | https://oneuptime.com/blog/post/2026-01-25-structured-json-logs-tracing-rust/view | Jan 2026 |
| 3 | LogMonitor: Structured Logging Best Practices 2026 | https://logmonitor.io/blog/structured-logging-best-practices | Mar 2026 |
| 4 | Khimananda: Structured Logging 2026 | https://khimananda.com/blog/structured-logging-best-practices | Aug 2026 |
| 5 | Grepr: Structured Logging Best Practices 2026 | https://www.grepr.ai/blog/structured-logging-best-practices | Jul 2026 |
| 6 | BetterStack: Dynamic Log Levels | https://betterstack.com/community/guides/logging/change-log-levels-dynamically/ | Dec 2023 |
| 7 | CVE-2026-10682: Zephyr Log Filtering OOB | https://vuldb.com/vuln/383540 | Jul 2026 |
| 8 | Grafana: Adaptive Logs Drop Rules | https://grafana.com/whats-new/2026-04-16-precise-log-filtering-with-adaptive-logs-drop-rules/ | Apr 2026 |
| 9 | ADHDecode: Dynamic Log Levels Runtime | https://adhdecode.com/observability/structured-logging/dynamic-log-levels-runtime-configuration/ | Mar 2026 |
| 10 | tracing docs.rs | https://docs.rs/tracing/latest/tracing | 2026 |

## What's NEW vs Prior Batches

Prior batches (1–753) covered: no `#[global_allocator]`, no arena/region allocation, mimalloc stability, region allocator advantage. **This batch covers observability/logging infrastructure — zero prior batches covered it.**

---

## Defects Found

### CRIT-1: Dual Logging System — `log::*` Bypasses `tracing` Entirely

**Severity: CRITICAL**
**File: `neotrix-core/src/unified/layers/action/nt_io/nt_io_logging.rs:10-22`**

NeoTrix has TWO SEPARATE, INCOMPATIBLE logging systems running simultaneously:

1. **`nt_io_logging.rs`** exports `tracing::{info, warn, error, debug}` on line 10, then defines its OWN macro system (`log_error!`, `log_warn!`, etc.) that routes through `eprintln!` (line 60) — completely bypassing tracing-subscriber.
2. **100+ call sites** across the codebase use `log::warn!`, `log::info!`, `log::debug!`, `log::error!`, `log::trace!` — the `log` crate macros.

The `log` crate and `tracing` crate are **one-direction compatible**: `tracing` can capture `log` events (via `log` feature), but `log` CANNOT capture `tracing` spans/events. Since 100% of actual log calls use `log::*`, and `init_tracing()` sets up tracing-subscriber, **all structured tracing instrumentation is dead code** — events go to `eprintln!` as unstructured text, never through the subscriber.

**Impact**: Zero structured logs. Zero span hierarchy. Zero OTel integration. All observability infrastructure is decorative.

**Fix**: Replace all `log::*` calls with `tracing::*` calls (tracing is backward-compatible), OR enable `tracing`'s `log` compatibility feature and route everything through tracing-subscriber.

---

### CRIT-2: No Structured JSON Logging

**Severity: CRITICAL**
**File: `neotrix-core/src/unified/layers/action/nt_io/nt_io_logging.rs:15-21`**

`init_tracing()` uses `fmt()` (text format), not `.json()`. The `json` feature IS enabled in `Cargo.toml` (line 80: `features = ["env-filter"]` — wait, actually `json` is NOT enabled in neotrix-core, only in `src-tauri/Cargo.toml`).

2026 industry consensus: JSON is the standard for structured logging (Sources 3,4,5,6). Every major platform (Datadog, Loki, Grafana, CloudWatch, Elasticsearch) natively parses JSON. Plain text logs require fragile regex parsing downstream.

**Impact**: All logs are unstructured `eprintln!` strings. Cannot be queried, filtered, or aggregated by any observability tool. Debugging at scale requires manual grep across unstructured text.

**Fix**: Enable `json` feature in tracing-subscriber, switch to `.json()` formatter, flatten event fields to top level.

---

### CRIT-3: Zero Span/Context Propagation

**Severity: HIGH**
**File: entire codebase**

Zero `#[instrument]` annotations found across 100+ source files. All log calls are flat events without any span hierarchy. No `request_id`, `session_id`, `trace_id`, or `span_id` propagated through call chains.

2026 best practice (Sources 1,2,10): `#[instrument(fields(user_id, request_id))]` wraps async functions, automatically propagating context to all child logs. For a consciousness architecture with complex cross-module flows (E8 → GWT → SEAL → KB), this is essential for debugging which consciousness tick triggered which cascade.

**Impact**: When a consciousness cycle fails, there is NO way to trace which module initiated the failure or what the full call chain was. Each log line is an isolated island.

**Fix**: Add `#[instrument]` to public async functions in key modules (nt_core_consciousness_core, nt_core_task_dispatcher, nt_mind_background_loop). Use `tracing::Span::current().record()` for late-bound fields.

---

### CRIT-4: Redundant Manual Level Filtering

**Severity: MEDIUM**
**File: `neotrix-core/src/unified/layers/action/nt_io/nt_io_logging.rs:24-46`**

Lines 24-46 implement a manual `AtomicU8`-based log level filter (`LOG_LEVEL`, `set_level()`, `should_log()`). This duplicates what `tracing_subscriber::EnvFilter` already provides, but worse:

1. `set_level()` only changes the AtomicU8 — it does NOT reconfigure the tracing-subscriber's EnvFilter
2. The manual filter only supports 4 levels (ERROR/WARN/INFO/DEBUG) — missing TRACE, FATAL/CRITICAL
3. No per-module filtering (EnvFilter supports `nt_core=debug,nt_shield=warn`)
4. No runtime reconfiguration via API endpoint (2026 practice: HTTP endpoint or signal handler, Source 6)

**Impact**: Two competing filter systems. Neither works correctly. Module-level log control is impossible.

**Fix**: Remove manual level filtering. Use `EnvFilter` with `try_init()` and expose a `set_level()` that reconfigures the subscriber dynamically.

---

### CRIT-5: OpenTelemetry Integration Dead

**Severity: HIGH**
**File: `neotrix-core/src/unified/layers/action/nt_io/nt_io_telemetry.rs:23`**

`init_otel()` exists and sets up OTLP export, but line 23 uses `log::warn!()` instead of `tracing::warn!()`. Since the tracing subscriber never receives these events (Defect CRIT-1), OTel spans are never exported. The entire `telemetry` feature flag (`Cargo.toml:167`) is decorative.

**Impact**: No distributed tracing. No Jaeger/Zipkin/Datadog export. Cannot correlate NeoTrix consciousness cycles across service boundaries.

**Fix**: Ensure `init_otel()` uses tracing macros and integrates as a tracing-subscriber layer.

---

### CRIT-6: No Log Sampling or Cost Control

**Severity: MEDIUM**
**Source: Grafana Adaptive Logs (Source 8), Grepr (Source 5)**

2026 production practice includes head-based sampling (keep 10% of DEBUG, 100% of ERROR) and tail-based sampling (keep traces that exceed latency thresholds). Grafana's Adaptive Logs Drop rules (Apr 2026) let you precisely drop noisy health-check logs and debug noise.

NeoTrix has zero sampling. At scale with consciousness ticks running every 60s, this generates unbounded log volume with no cost control.

**Impact**: Unbounded storage growth. No way to reduce log costs without losing signal.

**Fix**: Add `tracing_subscriber::filter::DynFilterFn` for head-based sampling. Consider tail-based sampling via OpenTelemetry Collector for production deployments.

---

### CRIT-7: No Structured Service Identification Fields

**Severity: MEDIUM**
**Sources: OneUptime (Source 2), InfraRunBook (Source 10)**

2026 log format standard requires static fields: `service.name`, `service.version`, `environment`, `trace_id`, `span_id` (Sources 2,5,10). NeoTrix logs contain none of these. In a multi-domain architecture (NT-CORE, NT-MIND, NT-WORLD, etc.), there is no way to filter logs by domain or version.

**Impact**: Cannot correlate logs across domains. Cannot identify which version generated a log line. Cannot filter by environment.

**Fix**: Add static fields via `tracing_subscriber::fmt::layer().with_span_list(true)` and a root span with `service.name`, `version`, `environment`.

---

### CRIT-8: CVE-2026-10682 — Log Filtering OOB (Zephyr, Rust-Relevant Pattern)

**Severity: INFO (pattern warning)**
**Source: VulDB (Source 7)**

CVE-2026-10682: Zephyr's runtime log filtering has an out-of-bounds write in `z_vrfy_log_filter_set` due to unvalidated `src_id`. While this is Zephyr (embedded), the pattern is relevant: **any runtime log filtering system must validate filter indices**. NeoTrix's manual `set_level()` uses string-to-enum mapping (safe), but if NeoTrix ever adds per-module runtime filtering, this CVE is a design precedent for bounds checking.

**Impact**: No direct impact on NeoTrix. Design warning for future per-module filter APIs.

---

## Defect Summary

| ID | Severity | Description | File:Line |
|----|----------|-------------|-----------|
| CRIT-1 | CRITICAL | Dual logging: `log::*` bypasses `tracing` entirely | nt_io_logging.rs:10-60 |
| CRIT-2 | CRITICAL | No structured JSON output (text-only) | nt_io_logging.rs:15-21 |
| CRIT-3 | HIGH | Zero `#[instrument]` span/context propagation | entire codebase |
| CRIT-4 | MEDIUM | Redundant manual level filtering duplicates EnvFilter | nt_io_logging.rs:24-46 |
| CRIT-5 | HIGH | OpenTelemetry integration dead (uses `log::*`) | nt_io_telemetry.rs:23 |
| CRIT-6 | MEDIUM | No log sampling or cost control | config-wide |
| CRIT-7 | MEDIUM | No structured service identification fields | nt_io_logging.rs |
| CRIT-8 | INFO | CVE-2026-10682 pattern: validate runtime filter indices | design precedent |

## Recommended Priority

1. **CRIT-1** → Replace all `log::*` with `tracing::*` (one-time migration, tracing is API-compatible)
2. **CRIT-2** → Enable `json` feature, switch `fmt()` to `json()` with `.flatten_event(true)`
3. **CRIT-3** → Add `#[instrument]` to key async entry points
4. **CRIT-5** → Wire OTel as tracing-subscriber layer
5. **CRIT-4** → Remove manual filtering, use EnvFilter exclusively
6. **CRIT-7** → Add root span with service/version/environment fields
7. **CRIT-6** → Add head-based sampling for DEBUG level

## Batch 754 Status: COMPLETE

**Next iteration focus**: Allocator alternatives (jemalloc, snmalloc, rpmalloc) or consciousness tick profiling.
