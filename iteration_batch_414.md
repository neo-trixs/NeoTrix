# Iteration Batch 414 — Error Handling, Resilience & Chaos Engineering

**Date**: 2026-09-06
**Focus**: External research → NeoTrix defect identification

---

## Sources Cited

| # | Source | Date | Topic |
|---|--------|------|-------|
| S1 | blog.rajpoot.dev — Rust Error Handling in 2026 | 2026-04-30 | anyhow/thiserror settled patterns |
| S2 | oneuptime.com — How to Design Error Types with thiserror/anyhow | 2026-01-25 | Library vs application error split |
| S3 | pistack.xyz — anyhow vs thiserror vs eyre guide | 2026-06-22 | eyre as third option for CLI tools |
| S4 | sharpskill.dev — Rust Error Handling 2026 | 2026-07-14 | Async error patterns, downcasting |
| S5 | 1xapi.com — Circuit Breaker, Bulkhead & Retry in Node.js 2026 | 2026-03-17 | Per-operation circuit breakers, jitter mandatory |
| S6 | techinterview.org — System Design: Circuit Breaker/Retry/Bulkhead | 2026-04-17 | Bulkhead thread pool isolation |
| S7 | apiscout.dev — API Resilience 2026 (Cockatiel) | 2026-03-09 | Composable policies: retry→CB→bulkhead→timeout |
| S8 | blog.rajpoot.dev — Circuit Breakers, Bulkheads, Backpressure 2026 | 2026-04-30 | Backpressure as 4th pattern |
| S9 | zylos.ai — Chaos Engineering for AI Agent Systems | 2026-04-09 | ReliabilityBench: AI-specific fault injection |
| S10 | astaqc.com — Chaos Engineering for QA Teams 2026 | 2026-06-29 | State-based fault injection, schema drift |
| S11 | susatest.com — Chaos Testing Complete Guide | 2026-05-29 | CI/CD-integrated chaos experiments |
| S12 | internet-pros.com — Chaos Engineering 2026 | 2026-09-01 | Small-team fault injection patterns |
| S13 | codex.danielvaughan.com — Codex CLI for Chaos Engineering | 2026-09-03 | Agent-driven experiment generation |

---

## Defects Found

### DEFECT-EH-001: `thiserror` pinned to v1 — 2026 ecosystem is v2

**Location**: `neotrix-core/Cargo.toml:26`, `nt_core_capability_tree/Cargo.toml:12`
**Current**: `thiserror = "1"` / `thiserror = "1.0"`
**2026 Standard**: `thiserror = "2.0"` (S1, S3, S4 all reference v2 as the settled standard)
**Impact**: Missing v2 improvements: better `#[source]` handling, improved derive macro, `no_std` by default. Minor but signals staleness.
**Fix**: Bump to `thiserror = "2"` across all Cargo.toml files.

---

### DEFECT-EH-002: `eyre` absent — CLI tools lack rich error reporting

**Location**: Project-wide
**Current**: No `eyre` dependency anywhere. `anyhow` is optional.
**2026 Standard**: `eyre` + `color-eyre` is the recommended choice for CLI tools with colorized output, source-code snippets, and structured suggestions (S3).
**Impact**: NeoTrix CLI produces plain text error output. No colored diagnostics, no `.suggestion()` / `.note()` methods, no backtrace integration for user-facing errors.
**Fix**: Add `eyre = "0.6"` and `color-eyre` to CLI crate. Use `eyre::Report` in CLI binary, keep `thiserror` for library crates.

---

### DEFECT-EH-003: `ExponentialBackoffStrategy::can_handle` excludes 5 of 9 error types

**Location**: `nt_core_error_recovery.rs:98-102`
**Current**: Only handles `RateLimit`, `ServerError`, `Timeout`
**Missing**: `InvalidOutput`, `Hallucination`, `InfiniteLoop`, `ContextOverflow`, `BudgetExceeded`
**Impact**: 55% of defined error types bypass the backoff strategy entirely, falling through to `RecoveryOrchestrator` which may abort. A hallucination or context overflow should trigger retry with prompt reformulation, not immediate abort.
**Fix**: Expand `can_handle` to include `InvalidOutput` and `ContextOverflow` (retry with prompt truncation). Add `Hallucination` as retryable with semantic fallback.

---

### DEFECT-EH-004: Jitter implementation is broken — additive instead of multiplicative

**Location**: `nt_core_error_recovery.rs:131-140`
**Current**: `compute_backoff` adds `jitter * base / 2` to the exponential delay. Jitter is applied to `base_delay_ms`, not the exponential value.
**2026 Standard**: Jitter should be multiplicative: `actual = exp_delay * (0.5 + random() * 0.5)` (S5, S6, S7 — all specify ±50% random jitter on the full exponential delay).
**Impact**: At high attempt counts (e.g., attempt 5: 3200ms), jitter adds only ~50ms (if base=100), which is negligible. At low attempts (attempt 1: 100ms), jitter adds ~50ms which is 50% — disproportionate. This causes thundering herd at scale.
**Fix**: Replace with: `let jitter = rand::random::<f64>(); let delay = exp_delay as f64 * (0.5 + jitter * 0.5);`

---

### DEFECT-EH-005: `RetryConfig::delay` in observer_error.rs lacks jitter entirely

**Location**: `nt_core_observer_error.rs:94-99`
**Current**: `delay()` computes pure exponential: `base * 2^(attempt-1)` with no jitter.
**Impact**: All retries across all observers are perfectly synchronized. Under load, thousands of concurrent observer retries create thundering herd. This is the exact anti-pattern warned about in S5, S6, S7.
**Fix**: Add jitter factor to `RetryConfig` and apply multiplicative jitter in `delay()`.

---

### DEFECT-RE-001: No bulkhead pattern anywhere in the codebase

**Location**: Project-wide
**Current**: Zero bulkhead implementations. Grep for `bulkhead|Bulkhead|isolation.*pool` returns nothing.
**2026 Standard**: Bulkhead is one of the four essential resilience patterns (S5, S6, S7, S8). Limits concurrency per downstream service to prevent one slow dependency from exhausting all resources.
**Impact**: A slow LLM provider call can block the entire `nt_core` reasoning engine thread pool. No resource isolation between providers (OpenAI vs local Ollama vs Anthropic).
**Fix**: Implement `BulkheadPolicy` struct with `max_concurrent` and `queue_size`. Apply per-provider in `nt_io_llm` gateway.

---

### DEFECT-RE-002: Circuit breaker is synchronous — incompatible with async codebase

**Location**: `nt_core_observer_error.rs:59` — `std::thread::sleep(Duration::from_millis(delay))`
**Current**: The `execute()` method uses blocking `std::thread::sleep` for retry delays.
**Impact**: In async context (Tokio runtime), blocking sleep parks the OS thread. Under concurrent observer calls, this degrades to thread pool exhaustion. The entire retry→CB→fallback pipeline is sync in an async system.
**Fix**: Create `AsyncObserverErrorRecovery` with `tokio::time::sleep` and async `execute()`.

---

### DEFECT-RE-003: No timeout enforcement on circuit breaker or recovery

**Location**: `nt_core_observer_error.rs`, `nt_core_error_recovery.rs`
**Current**: No `TimeoutPolicy` or `TimeLimiter` anywhere.
**2026 Standard**: Timeout is the 4th essential pattern (S6, S7). "A request with no timeout can hang forever" (S7).
**Impact**: If an LLM call hangs (e.g., network partition with half-open TCP connection), the observer blocks indefinitely. No timeout to force cancellation.
**Fix**: Add `tokio::time::timeout()` wrapper around all LLM call sites. Add configurable `timeout_ms` to `ObserverErrorRecovery`.

---

### DEFECT-RE-004: Circuit breaker has no observability — no metrics export

**Location**: `nt_core_observer_error.rs` — `CircuitBreaker` struct
**Current**: Circuit state transitions (Closed→Open→HalfOpen) are not logged, metrics-emitted, or exposed.
**2026 Standard**: "Alerting on circuit breaker state changes is far more actionable than generic error rate alerts" (S7). Prometheus metrics for circuit state are standard (S5, S6).
**Impact**: Silent circuit trips. No way to detect when a provider is failing without reading source code. No Grafana dashboard for resilience health.
**Fix**: Add `on_state_change` callback or emit events via `nt_core_event_bus` on transitions. Export `circuit_breaker_state`, `circuit_breaker_failures_total` as metrics.

---

### DEFECT-RE-005: `RecoveryOrchestrator` strategies are not composable

**Location**: `nt_core_error_recovery.rs:415-488`
**Current**: Orchestrator iterates strategies linearly and returns the first match. No composition of retry + CB + bulkhead as layered policies.
**2026 Standard**: Composable policies: `wrap(retryPolicy, circuitBreaker, bulkheadPolicy)` (S7, Cockatiel pattern).
**Impact**: Each strategy is independent. Can't express "retry with backoff, then circuit breaker, then bulkhead" as a single execution chain. Forces manual composition at every call site.
**Fix**: Implement `PolicyChain` that wraps operations in nested retry→CB→bulkhead→timeout layers.

---

### DEFECT-CH-001: Zero chaos engineering infrastructure

**Location**: Project-wide
**Current**: No chaos experiments, no fault injection framework, no resilience tests. Grep for `chaos|fault_injection` returns only KB seed data about "Chaos Theory" as a concept.
**2026 Standard**: Chaos engineering is a CI/CD gate (S10, S11, S12). "Chaos engineering is no longer exclusively SRE responsibility" (S10).
**Impact**: Resilience regressions ship silently. No way to verify circuit breakers, retries, or fallbacks work under real fault conditions. Only unit tests exist (which verify happy path or single-error scenarios).
**Fix**: Add `nt_shield_chaos` module with fault injection traits. Wire into CI for staging: inject LLM latency, token budget exhaustion, provider unavailability.

---

### DEFECT-CH-002: No schema drift testing for MCP tool outputs

**Location**: Project-wide
**Current**: Tool outputs are consumed without schema validation or drift detection.
**2026 Standard**: Schema drift injection is a first-class chaos test (S9, S10). "A service can respond promptly and still return incorrect data" (S10).
**Impact**: If an MCP tool changes its output schema (renames fields, drops fields), NeoTrix silently processes garbage. No validation gate, no drift detection.
**Fix**: Add schema versioning to tool output contracts. Add `SchemaDriftInjector` for chaos tests that renames/drops fields.

---

### DEFECT-CH-003: No AI-agent-specific fault injection

**Location**: Project-wide
**Current**: No testing for: silent LLM degradation, hallucination cascades, token budget exhaustion mid-task, partial response handling.
**2026 Standard**: ReliabilityBench (S9) introduced 3D reliability surface: consistency, robustness, fault tolerance. "An LLM API doesn't just go down; it degrades silently" (S9).
**Impact**: The system has `Hallucination` and `BudgetExceeded` error types but no way to test recovery from them. Multi-agent hallucination cascades are undetectable.
**Fix**: Create `AgentChaosInjector` that simulates: partial responses, schema drift, token exhaustion, hallucination patterns. Wire into SEAL pipeline testing.

---

### DEFECT-CH-004: No backpressure mechanism

**Location**: Project-wide
**Current**: No backpressure implementation. When downstream is slow, requests queue unboundedly.
**2026 Standard**: Backpressure is the 4th resilience pattern alongside circuit breaker, bulkhead, retry (S8).
**Impact**: Under load, request queues grow without bound, consuming memory. No signal to producers to slow down.
**Fix**: Implement bounded channels with `tokio::sync::Semaphore` at gateway entry points. Expose saturation metrics.

---

## Summary

| Category | Defects | Critical | High | Medium |
|----------|---------|----------|------|--------|
| Error Handling | 5 | 2 (EH-003, EH-004) | 2 (EH-002, EH-005) | 1 (EH-001) |
| Resilience | 5 | 2 (RE-001, RE-003) | 2 (RE-002, RE-004) | 1 (RE-005) |
| Chaos Engineering | 4 | 3 (CH-001, CH-002, CH-003) | 1 (CH-004) | 0 |
| **Total** | **14** | **7** | **5** | **2** |

## Priority Recommendations

1. **P0 — Fix jitter** (EH-004, EH-005): Broken jitter = thundering herd under load. One-line fix.
2. **P0 — Add bulkhead** (RE-001): No resource isolation = single slow provider kills entire system.
3. **P0 — Add chaos infra** (CH-001): No fault injection = untested resilience claims.
4. **P1 — Async error recovery** (RE-002): Blocking sleep in async context = thread pool exhaustion.
5. **P1 — Expand can_handle** (EH-003): 5 error types bypass recovery = silent aborts.
6. **P1 — Add timeout** (RE-003): No timeout = indefinite hangs on network partitions.
7. **P2 — Add eyre** (EH-002): CLI UX improvement.
8. **P2 — Schema drift testing** (CH-002): Prevents silent garbage processing.
9. **P2 — Circuit breaker observability** (RE-004): Enables operational visibility.
10. **P3 — thiserror v2 bump** (EH-001): Modernization.
11. **P3 — Policy composition** (RE-005): Architectural improvement.
12. **P3 — AI-agent chaos** (CH-003): Advanced testing.
13. **P3 — Backpressure** (CH-004): Load shedding.
