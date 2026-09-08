# Iteration 692 — Resilience Pattern Audit

**Date**: 2026-09-06  
**Batch**: 692 (of 10000+)  
**Focus**: Retry, Circuit Breaker, Error Recovery patterns — 2026 state-of-the-art vs NeoTrix gaps  

---

## Sources Consulted

| Source | Date | Key Insight |
|--------|------|-------------|
| [Platformwale: Resilience Patterns at Scale](https://platformwale.blog/2026/04/30/resilience-patterns-at-scale-exponential-backoff-jitter-retry-budgets-and-circuit-breakers-in-practice/) | 2026-04-30 | Retry budgets MUST be service-level, not per-request. Budget >80% sustained = earliest degradation warning. |
| [knowledgelib: Retry with Backoff](https://knowledgelib.io/software/patterns/retry-exponential-backoff/2026) | 2026-02-24 | Full jitter has lowest server load per AWS sim. Single retry point with retry budget prevents amplification. |
| [LavX: Timeout and Retry Patterns](https://news.lavx.hu/article/timeout-and-retry-patterns-in-distributed-systems-a-practical-guide) | 2026-05-20 | Google gRPC supports retry budgets natively. Selective retry must distinguish retriable vs non-retriable at classification time. |
| [youngju: Circuit Breaker + Resilience4j](https://www.youngju.dev/blog/architecture/2026-03-09-circuit-breaker-resilience-patterns-guide.en) | 2026-03-09 | Key principle: "Standalone Retry is prohibited" — always pair with CircuitBreaker. Multi-level fallback: alt→cache→default. |
| [Nemorize: Distributed System Patterns](https://nemorize.com/roadmaps/production-observability-from-signals-to-root-cause-2026/lessons/distributed-system-patterns) | 2026-01-10 | Bulkhead: instrument BEFORE sizing. Circuit state changes are high-signal metrics. Retry amplification: 1×3×3×3=27x. |
| [Zylos: Graceful Degradation in AI Agents](https://zylos.ai/research/2026-02-20-graceful-degradation-ai-agent-systems/) | 2026-02-20 | AI agents fail silently (41-86.7% production failure rate without fault tolerance). Behavioral metrics > infrastructure metrics. |
| [Hermes: Error Recovery Benchmarks](https://hermes-agent.reviews/error-recovery-patterns.html) | 2026-06-05 | Self-Healing Score (0-100): Recovery Rate × Recovery Latency × Strategy. 6 failure categories benchmarked. |
| [Zylos: Agent Self-Healing](https://zylos.ai/research/2026-05-06-agent-self-healing-failure-recovery/) | 2026-05-06 | ALAS framework: fault-tolerant compensator per agent. Supervisor tree (Erlang/OTP) for agent fault isolation. |
| [system-design.space: Fault Tolerance](https://system-design.space/en/chapter/resilience-patterns) | 2026-08-08 | Blast-radius-control model: circuit breaker + bulkhead + retry + fallback as unified operating model. |
| [web-alert: Graceful Degradation](https://web-alert.io/blog/graceful-degradation-designing-resilient-systems) | 2026-08-10 | Degradation hides failures so well it hides them from YOUR TEAM too. Monitor degraded states, not just outages. |

---

## Defects Found

### DEFECT-692.1: Two Circuit Breakers — One Proper, One Dead (CRITICAL)

**Dual implementations found:**

1. **`nt_core_observer_error.rs:127-200`** — PROPER `CircuitBreaker` with full state machine (Closed→Open→HalfOpen), `record_success`/`record_failure`, timeout-based transitions. BUT **only used by the `+1 Observer` subsystem** (single consumer).

2. **`nt_core_error_recovery.rs:142-189`** — BROKEN `CircuitBreakerStrategy`. Has `failure_counts` HashMap but **no methods to mutate it**. `recover()` takes `&self` (immutable). `can_handle()` reads counts that are never incremented. This is the **general-purpose** error recovery strategy used by all domains.

**2026 standard** (source: youngju 2026-03-09, nemorize 2026-01-10):
- Three-state machine: CLOSED → OPEN → HALF-OPEN → CLOSED
- `record_failure()` increments count, transitions CLOSED→OPEN at threshold
- `record_success()` resets count, transitions HALF-OPEN→CLOSED
- Single circuit breaker per dependency, shared across all consumers

**NeoTrix gap**: The proper circuit breaker is siloed in one subsystem. The general error recovery strategy (`CircuitBreakerStrategy`) is dead code — it can never open because `failure_counts` is never mutated.

**Severity**: P0 — general-purpose circuit breaker is theater, not protection.

---

### DEFECT-692.2: Jitter Implementation Is Not Decorrelated (MODERATE)

**File**: `neotrix-core/src/unified/core/nt_core_error_recovery.rs:131-140`

```rust
fn compute_backoff(base: u64, attempt: usize, max_delay: u64, jitter: f64) -> u64 {
    let exp_delay = base.saturating_mul(1 << attempt.min(30));
    let jitter_val = if jitter > 0.0 {
        let j = (jitter * base as f64) as u64;
        j / 2
    } else { 0 };
    exp_delay.saturating_add(jitter_val).min(max_delay)
}
```

This adds a **fixed offset** (jitter_factor × base / 2) to every retry. It does NOT decorrelate retry waves across clients. Per AWS 2026 simulations (knowledgelib 2026-02-24), full jitter `random(0, exp_delay)` has lowest server load.

**2026 standard** (source: platformwale 2026-04-30, kestra 2026-07-21):
- Full jitter: `sleep = random(0, min(max_delay, base × 2^attempt))`
- Equal jitter: `sleep = min_delay/2 + random(0, min_delay/2)`
- Decorrelated jitter: `sleep = min(max_delay, random(base, prev_sleep × 3))`

**NeoTrix gap**: Jitter is additive constant, not multiplicative random. Under load, all clients retry at nearly identical intervals (thundering herd).

**Severity**: P1 — causes retry storms during dependency degradation.

---

### DEFECT-692.3: No Global Retry Budget (CRITICAL)

**Across codebase** — no retry budget exists.

Per-operation `max_retries` (scheduler: 3, visual: 2, self_heal: 3, registry_watcher: 3) but no service-level budget limiting total retry amplification.

**2026 standard** (source: platformwale 2026-04-30, lavx 2026-05-20):
- Retry budget = max percentage of total requests that can be retries (e.g., 5%)
- Token bucket: each retry consumes a token; bucket refills at steady rate
- Budget exhausted → fail fast, no retry
- Alert: sustained >80% budget utilization for 60s = earliest degradation warning

**NeoTrix gap**: N concurrent operations each retrying M times → N×M load on already-failing dependency. The `nt_core_scheduler` runs multiple jobs in parallel; a failing LLM provider causes unbounded retry amplification across all jobs.

**Severity**: P0 — retry amplification cascading is the #1 cause of production outages (platformwale 2026-04-30, citing AWS 2025 Kinesis incident).

---

### DEFECT-692.4: No Bulkhead Isolation (MODERATE)

**Across codebase** — no bulkhead pattern. All LLM calls, tool calls, crawls, and KB operations share thread/connection resources.

**2026 standard** (source: youngju 2026-03-09, nemorize 2026-01-10):
- Separate thread pools per dependency tier (critical vs non-critical)
- Separate connection pools per service
- Per-tenant or per-endpoint rate limits
- Resource quotas prevent one noisy neighbor from starving critical paths

**NeoTrix gap**: A slow crawl operation (NT-WORLD) can starve LLM provider calls (NT-IO), which can starve KB writes (NT-MEMORY). No resource isolation between domains.

**Severity**: P1 — cross-domain resource starvation during partial degradation.

---

### DEFECT-692.5: No Idempotency Keys for Retries (MODERATE)

**File**: `neotrix-core/src/unified/core/nt_core_error_recovery.rs` — `RecoveryAction::Retry` carries only delay and reason, no idempotency key.

**2026 standard** (source: platformwale 2026-04-30, oneuptime 2026-01-30):
- Every retryable operation must have an idempotency key
- Key derived from operation hash (e.g., `sha256(prompt + model + timestamp_bucket)`)
- Server deduplicates by key within TTL window

**NeoTrix gap**: Retrying an LLM call with same prompt may produce duplicate side effects (KB writes, tool invocations, cost accounting double-counts).

**Severity**: P2 — data consistency risk on retries.

---

### DEFECT-692.6: No SLO-Tied Error Budget for Resilience Decisions (MODERATE)

**Across codebase** — no connection between resilience patterns and SLO/error budgets.

**2026 standard** (source: system-design.space 2026-08-08, web-alert 2026-08-10):
- Tie retry budgets to SLO error budget (e.g., 99.9% availability = 0.1% error budget)
- When error budget exhausted → stop retries, shift to graceful degradation
- Circuit breaker threshold derived from error budget, not arbitrary constant

**NeoTrix gap**: `CircuitBreakerStrategy { threshold: 3 }` is hardcoded. No mechanism to derive threshold from actual SLO or error budget consumption.

**Severity**: P2 — resilience thresholds are disconnected from reliability targets.

---

### DEFECT-692.7: No Behavioral Degradation Metrics for AI Agents (MODERATE)

**Across codebase** — only infrastructure metrics (latency, error rate, token count). No behavioral metrics.

**2026 standard** (source: zylos 2026-02-20, hermes 2026-06-05):
- AI agents fail SILENTLY — hallucination, step-skipping, quality degradation without error signals
- Behavioral metrics: task completion rate, output quality score, hallucination rate, step coverage
- Self-Healing Score = Recovery Rate × Recovery Latency × Strategy effectiveness
- 41-86.7% production failure rate without deliberate fault tolerance

**NeoTrix gap**: `CrawlReport` tracks completed/failed but not quality of completed results. No detection of silent degradation (e.g., LLM returning lower-quality output without error).

**Severity**: P1 — silent degradation goes undetected.

---

### DEFECT-692.8: ObserverErrorRecovery RetryConfig Has No Jitter (MODERATE)

**File**: `neotrix-core/src/unified/core/nt_core_observer_error.rs:94-99`

```rust
pub fn delay(&self, attempt: u32) -> u64 {
    let exp = self.base_delay_ms.saturating_mul(2u64.saturating_pow(attempt.saturating_sub(1)));
    exp.min(self.max_delay_ms)
}
```

Pure exponential backoff with zero jitter. When the `+1 Observer` retries, all concurrent observer instances retry at identical intervals (thundering herd).

**2026 standard**: Full jitter, equal jitter, or decorrelated jitter required for any retry in distributed systems (platformwale 2026-04-30, knowledgelib 2026-02-24).

**Severity**: P1 — observer retry storms during dependency degradation.

---

### DEFECT-692.9: No Graceful Degradation Declaration Protocol (NEW)

**2026 standard** (source: zylos 2026-05-30, web-alert 2026-08-10):
- Agent must DECLARE its degraded state explicitly
- Degradation level must be observable and auditable
- Restoration must be explicit (not just "try again next time")

**NeoTrix gap**: `RecoveryAction::FallbackToModel` and `RecoveryAction::Abort` exist but there's no protocol for declaring "I am now operating at degraded level X" that downstream consumers can observe and adapt to.

**Severity**: P2 — degradation is implicit, not auditable.

---

## Summary of Findings

| # | Defect | Severity | Category |
|---|--------|----------|----------|
| 692.1 | Two circuit breakers: one proper (siloed), one dead (general-purpose) | P0 | Circuit Breaker |
| 692.2 | Jitter is additive constant, not decorrelated | P1 | Retry |
| 692.3 | No global retry budget | P0 | Retry |
| 692.4 | No bulkhead isolation | P1 | Fault Isolation |
| 692.5 | No idempotency keys for retries | P2 | Retry |
| 692.6 | No SLO-tied error budget | P2 | Resilience Governance |
| 692.7 | No behavioral degradation metrics | P1 | Observability |
| 692.8 | ObserverErrorRecovery RetryConfig has no jitter | P1 | Retry |
| 692.9 | No graceful degradation declaration protocol | P2 | Degradation |

**P0 defects**: 2 (dead circuit breaker, no retry budget)  
**P1 defects**: 4 (jitter ×2, bulkhead, behavioral metrics)  
**P2 defects**: 3 (idempotency, SLO tie, degradation protocol)  

**Key architectural insight**: NeoTrix has a well-implemented `CircuitBreaker` in `nt_core_observer_error.rs` but it's consumed by only one subsystem. The general-purpose `CircuitBreakerStrategy` in `nt_core_error_recovery.rs` is a dead stub. The fix should unify: promote the observer's `CircuitBreaker` to a shared primitive, delete the broken stub, and wire the error recovery strategy to use it.

**Next priority**: Fix DEFECT-692.1 (unify circuit breakers) and DEFECT-692.3 (retry budget) — these are P0 and directly prevent retry amplification cascading.
