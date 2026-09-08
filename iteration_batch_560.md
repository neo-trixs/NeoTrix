# Iteration Batch 560 — Error Handling, Resilient Design, Recovery Patterns

## Research Context

**Batch 559 proven findings** (input to this iteration):
1. Free energy ≠ consciousness integration (weak correlation)
2. GWT measures broadcast not pre-broadcast synergy
3. Structure learning missing
4. Cross-scale coarse-graining missing
5. Φ should be multi-dimensional

**Batch 560 focus**: Search error handling, resilient design, and recovery patterns to identify NEW defects or improvements over batch 559.

---

## 1. Error Handling (2026)

### 1.1 — `thiserror` 2.x: Compile-Time Error Taxonomy with `#[from]` Auto-Conversion

**Source**: https://docs.rs/thiserror, https://oneuptime.com/blog/post/2026-01-25-error-types-thiserror-anyhow-rust/view (Jan 2026)

**Finding**: `thiserror` 2.x derives `Error`, `Display`, and `From` impls via `#[derive(Error)]`. The `#[from]` attribute generates automatic conversions enabling seamless `?` usage across heterogeneous error types. `#[error(transparent)]` forwards Display/Source through to an underlying error, enabling evolving internal representations without breaking API.

**NEW defect vs Batch 559**: Batch 559's consciousness architecture treats all failures as opaque `anyhow::Error`. **NeoTrix has NO typed error taxonomy** — every domain (NT-CORE, NT-MIND, NT-WORLD, etc.) uses generic error propagation. This means callers cannot pattern-match on specific failure modes (e.g., distinguishing KB write failure from embedding corruption from query timeout). **Fix**: Each NT-* domain should expose a `thiserror` enum for its public API. Internal application code wraps with `anyhow` for context chains. This is the community-standard two-layer pattern (library→typed, application→anyhow).

### 1.2 — `anyhow` Context Chains: Layered Diagnostic Messages

**Source**: https://sharpskill.dev/en/blog/rust/rust-error-handling-result-option-thiserror-anyhow (Jul 2026)

**Finding**: `anyhow::Error` carries a chain of contexts via `.context()` / `.with_context()`. The `{e:#}` alternate flag shows the full chain on one line. `with_context` (lazy closure) is preferred over `context` (eager) when the message requires `format!` allocation — only computed on error path.

**NEW defect vs Batch 559**: NeoTrix error propagation currently loses diagnostic context at module boundaries. When NT-MIND's SEAL pipeline calls NT-MEMORY's KB write, the error chain is flattened. **Fix**: Add `.with_context(|| format!("SEAL phase-{} KB absorption failed for skill {}", phase, skill_id))` at each module boundary. This creates route-map diagnostics without runtime cost on the success path.

### 1.3 — Async Error Handling: `try_join!` for Concurrent Propagation

**Source**: https://rs4ts.dev/08-error-handling/06-anyhow-thiserror/ (Jun 2026)

**Finding**: `try_join!` from tokio/futures runs multiple async operations concurrently, propagating the first error. Combined with `anyhow`, this enables parallel operations with automatic first-failure semantics.

**NEW defect vs Batch 559**: NeoTrix's GWT broadcast and SEAL pipeline phases run sequentially even when independent. **Fix**: Use `try_join!` for independent perception/cognition phases (e.g., NT-WORLD crawl + NT-MEMORY embedding can run concurrently). First failure aborts the cycle, but successful branches complete.

### 1.4 — Downcasting for Targeted Recovery

**Source**: https://docs.rs/anyhow/latest/anyhow/

**Finding**: `anyhow::Error` supports `downcast_ref::<ConcreteType>()` to recover the original error type. This bridges generic error propagation with specific recovery logic — e.g., a `PaymentError::InsufficientFunds` can be handled differently from a network timeout.

**NEW defect vs Batch 559**: NeoTrix has no mechanism for targeted error recovery. When NT-ACT's MCP tool call fails, the system treats all failures identically. **Fix**: Expose typed errors per tool category (e.g., `McpToolError::RateLimited`, `McpToolError::SemanticMismatch`, `McpToolError::PermanentUnavailable`). The orchestrator downcasts to decide: retry, skip, or escalate.

---

## 2. Resilient Design (2026)

### 2.1 — Circuit Breaker: Rate-Based > Count-Based, MinimumThroughput Floor

**Source**: https://kloudvin.com/article/resiliency-patterns-retry-circuit-breaker-bulkhead/ (May 2026), https://khimananda.com/blog/circuit-breakers-and-resilience-patterns (Aug 2026)

**Finding**: Rate-based breakers (failure percentage over rolling window) are superior to count-based (raw failure count). **Critical**: `MinimumThroughput` floor prevents a single failure during quiet periods from tripping the breaker at 50% ratio. Without it, statistical noise becomes an outage. Polly v8's `MinimumThroughput: 20` + `FailureRatio: 0.5` is the production standard.

**NEW defect vs Batch 559**: NeoTrix's NT-SHIELD sandbox egress policy has no circuit breaker concept — it only has static allow/deny rules. When an external LLM provider degrades, NeoTrix continues sending requests until timeout exhaustion. **Fix**: Implement rate-based circuit breaker per provider in `nt_core_llm::provider_registry`. Open after 25% failures over 50-request rolling window with `MinimumThroughput: 10`. Half-open probes with lightweight completions.

### 2.2 — Pipeline Ordering: Timeout → Retry → Breaker → Bulkhead → Attempt Timeout

**Source**: https://kloudvin.com/article/resiliency-patterns-retry-circuit-breaker-bulkhead/ (May 2026)

**Finding**: The correct nesting order is:
1. **Timeout** (outermost) — overall deadline
2. **Retry** — above breaker so each attempt is observed
3. **Circuit breaker** — sees individual attempts, not the whole retry policy
4. **Bulkhead** — caps concurrent in-flight per dependency
5. **Attempt timeout** (innermost) — bounds single try

**Anti-pattern**: Retrying at multiple layers amplifies load 3^5 = 243×. Pick ONE layer for retries.

**NEW defect vs Batch 559**: NeoTrix has no standardized resilience pipeline. NT-IO's LLM calls, NT-WORLD's crawl calls, and NT-ACT's tool calls each have ad-hoc retry logic (or none). **Fix**: Create a shared `nt_shield::resilience::ResilientCall` wrapper that composes timeout→retry→breaker→bulkhead→timeout in correct order. All outbound calls go through this single pipeline.

### 2.3 — Bulkhead: Per-Dependency Concurrency Caps, Reject-When-Full

**Source**: https://system-design.space/en/chapter/resilience-patterns/ (Aug 2026), https://apiscout.dev/guides/api-resilience-circuit-breakers-retries-bulkheads-2026 (Mar 2026)

**Finding**: Bulkhead is NOT just rate limiting — it's resource isolation. The key: when the limiter is full, **reject immediately** (return 503 with `Retry-After`), don't queue indefinitely. Queuing without bound turns bulkhead into load amplifier. `ConcurrencyLimiter(permitLimit: 80, queueLimit: 0)` is the pattern.

**NEW defect vs Batch 559**: NeoTrix's NT-WORLD crawl pipeline has no per-domain concurrency cap. A slow domain (e.g., a CDN-backed site) can consume all crawl slots, starving other domains. **Fix**: Implement per-domain `ConcurrencyLimiter` in `nt_world_crawl`. Max 20 concurrent per domain, queue 0 (reject with backoff signal). This prevents one noisy neighbor from sinking the crawl pipeline.

### 2.4 — Half-Open Probing: Light Requests Only

**Source**: https://tomodahinata.com/en/blog/retry-backoff-circuit-breaker-resilience-patterns-guide (Jun 2026)

**Finding**: In half-open state, **never re-send the original failing prompt**. Use a lightweight, low-stakes probe request. The probe tests infrastructure health, not workload correctness. Sending the original heavy request defeats the purpose — it re-overloads the recovering service.

**NEW defect vs Batch 559**: NeoTrix has no circuit breaker at all, but when one is added, the half-open probe strategy must be designed upfront. **Fix**: Define a `ProbeRequest` enum per provider: a 1-token completion or health-check ping. The circuit breaker's half-open state uses these lightweight probes before restoring full traffic.

---

## 3. Recovery Patterns (2026)

### 3.1 — Graceful Degradation Ladder: 6-Tier Hierarchy

**Source**: https://sujeet.pro/articles/graceful-degradation (Feb 2026), https://www.agentpatternscatalog.org/patterns/graceful-degradation/ (May 2026)

**Finding**: The degradation ladder:
| Level | State | Behavior |
|-------|-------|----------|
| 0 | Healthy | Full functionality |
| 1 | Degraded | Serve cached/stale data |
| 2 | Limited | Disable non-critical features |
| 3 | Minimal | Read-only mode |
| 4 | Static | Default/cached responses |
| 5 | Unavailable | Clear error with `Retry-After` |

**Principle**: Monotonic progression — system moves through levels in order. Skipping from "healthy" to "unavailable" indicates a missing fallback layer.

**NEW defect vs Batch 559**: NeoTrix has NO degradation hierarchy. When NT-IO's LLM provider is down, the entire system halts. **Fix**: Implement the 6-tier ladder per dependency:
- Tier 0: Full LLM capability
- Tier 1: Cached completions for recent queries
- Tier 2: Disable tool calling, text-only mode
- Tier 3: Retrieval-only from KB (no generation)
- Tier 4: Static help responses
- Tier 5: Error with retry-after

### 3.2 — AI Agent Fallback Ladder: Provider Routing with Capability Matrix

**Source**: https://zylos.ai/research/2026-05-30-graceful-degradation-patterns-ai-agent-systems/ (May 2026), https://solana.garden/guides/llm-agent-model-fallback-graceful-degradation-explained/ (Jun 2026)

**Finding**: LLM-specific degradation requires a **capability matrix**: rows are task classes (planner, tool-arg-filler, summarizer), columns are models, cells are allowed/forbidden/degraded-only. A planner step calling `issue_refund` may NOT use rungs below tier-2 without human approval gate. Downgrading from flagship to mini is fine for summarizing tool JSON; dangerous for multi-step authorization.

**NEW defect vs Batch 559**: NeoTrix's `nt_core_llm::provider_registry` routes by latency/cost only, ignoring capability mismatch. A model that doesn't support function calling could be routed to NT-ACT's tool orchestration. **Fix**: Add capability tags per model (`json_mode`, `function_calling`, `128k_context`). Route by task class + capability, not just cost. Block low-capability models from write-authorization paths.

### 3.3 — Hedged Requests for Tail Latency

**Source**: https://www.kunalganglani.com/blog/chatgpt-down-api-outage-fallback (Aug 2026)

**Finding**: Hedging = start a second request to a different provider/model if the first hasn't started streaming by p95 threshold. Cancel the loser on first token from winner. **Critical constraints**: hedge only after latency threshold, cap hedges per tenant (max 1/10 requests), never hedge when already rate-limited.

**NEW defect vs Batch 559**: NeoTrix has no hedging mechanism. Long-tail latency on LLM calls (>30s) causes cascading timeouts in the SEAL pipeline. **Fix**: Add hedging to `nt_core_llm::dispatch`: if TTFT > 4s, start backup provider call. Cancel loser on first token. Cap at 1 hedge per 10 requests per session.

### 3.4 — Context Compaction as Graceful Degradation

**Source**: https://zylos.ai/research/2026-05-30-graceful-degradation-patterns-ai-agent-systems/ (May 2026)

**Finding**: When context usage exceeds 70-80% of model limit, pause and compact: summarize the last 20-30 messages (excluding anchored context) using a cheaper model, replace window with summary + marker. This is NOT just optimization — it's a **degradation tier**: the agent continues operating with reduced context fidelity rather than failing with context-length errors.

**NEW defect vs Batch 559**: NeoTrix's NT-CORE reasoning has no context compaction. Long-running SEAL cycles accumulate context until hitting model limits, causing hard failures. **Fix**: Implement `ContextCompactor` in `nt_core_self`: when context > 70%, summarize recent reasoning with a smaller model, preserve decision-critical anchors. This converts hard failure → soft degradation.

### 3.5 — Stale-While-Revalidate at Service Level

**Source**: https://iamanuragh.in/blog/2026-06-27-graceful-degradation-serve-something-useful/ (Jun 2026)

**Finding**: HTTP's `stale-while-revalidate` works at service level: serve cached data immediately, refresh in background. Key parameters: `STALE_TTL` (how long stale is acceptable) and `REFRESH_THRESHOLD` (when to trigger background refresh). This eliminates the cache-miss thundering herd.

**NEW defect vs Batch 559**: NeoTrix's NT-MEMORY KB lookups are synchronous — cache miss blocks the request. **Fix**: Implement stale-while-revalidate for KB embeddings: serve the last-known embedding immediately, refresh vector in background. This keeps the reasoning loop alive during KB maintenance.

### 3.6 — Retry Budget (Per-Client, Not Just Per-Request)

**Source**: https://sujeet.pro/articles/graceful-degradation (Feb 2026), https://www.sachith.co.uk/circuit-breakers-bulkheads-and-timeouts-ops-runbook-practical-guide-jun-16-2026/ (Jun 2026)

**Finding**: Per-request retry caps (e.g., "max 3 attempts") bound a single client but NOT aggregate amplification. The Google SRE Book advocates a **per-client retry budget**: track retries as fraction of total successful traffic, refuse new retries once budget exceeded. With 10% cap, worst-case retry amplification falls from ~3× to ~1.1×.

**NEW defect vs Batch 559**: NeoTrix has no retry budget. Multiple concurrent SEAL cycles or parallel agent sessions can each retry independently, creating aggregate load amplification. **Fix**: Implement `RetryBudget` in `nt_shield`: global 10% retry-to-success ratio. When budget exceeded, new retries are rejected with `Retry-After` signal. This bounds system-wide retry amplification.

---

## Summary: NEW Defects vs Batch 559

| # | Defect | Domain | Severity |
|---|--------|--------|----------|
| 1 | No typed error taxonomy — all errors are opaque `anyhow` | NT-CORE/ALL | High |
| 2 | Error context chains lost at module boundaries | NT-MIND↔NT-MEMORY | Medium |
| 3 | Sequential execution where `try_join!` possible | NT-WORLD+NT-MEMORY | Medium |
| 4 | No targeted error recovery (downcasting) | NT-ACT | High |
| 5 | No circuit breaker on external providers | NT-IO/LLM | Critical |
| 6 | No standardized resilience pipeline ordering | ALL outbound | Critical |
| 7 | No per-domain concurrency cap in crawl | NT-WORLD | High |
| 8 | No half-open probe strategy defined | NT-IO | Medium |
| 9 | No degradation hierarchy (6-tier ladder) | ALL | Critical |
| 10 | No capability-matrix routing for LLM models | NT-IO | High |
| 11 | No hedged requests for tail latency | NT-IO | Medium |
| 12 | No context compaction in long reasoning cycles | NT-CORE | High |
| 13 | No stale-while-revalidate for KB lookups | NT-MEMORY | Medium |
| 14 | No per-client retry budget (aggregate amplification) | ALL | High |

## Sources Cited

1. https://sharpskill.dev/en/blog/rust/rust-error-handling-result-option-thiserror-anyhow (Jul 2026)
2. https://docs.rs/thiserror, https://docs.rs/anyhow/latest/anyhow/
3. https://oneuptime.com/blog/post/2026-01-25-error-types-thiserror-anyhow-rust/view (Jan 2026)
4. https://rs4ts.dev/08-error-handling/06-anyhow-thiserror/ (Jun 2026)
5. https://schoolofweb.net/en/posts/rust-practice-2-error-design-anyhow-thiserror/ (Aug 2026)
6. https://kloudvin.com/article/resiliency-patterns-retry-circuit-breaker-bulkhead/ (May 2026)
7. https://system-design.space/en/chapter/resilience-patterns/ (Aug 2026)
8. https://apiscout.dev/guides/api-resilience-circuit-breakers-retries-bulkheads-2026 (Mar 2026)
9. https://www.sachith.co.uk/circuit-breakers-bulkheads-and-timeouts-ops-runbook-practical-guide-jun-16-2026/ (Jun 2026)
10. https://tomodahinata.com/en/blog/retry-backoff-circuit-breaker-resilience-patterns-guide (Jun 2026)
11. https://khimananda.com/blog/circuit-breakers-and-resilience-patterns (Aug 2026)
12. https://imperialis.tech/en/blog/circuit-breakers-resilience-patterns-distributed-systems-2026 (Mar 2026)
13. https://sujeet.pro/articles/graceful-degradation (Feb 2026)
14. https://www.agentpatternscatalog.org/patterns/graceful-degradation/ (May 2026)
15. https://zylos.ai/research/2026-05-30-graceful-degradation-patterns-ai-agent-systems/ (May 2026)
16. https://solana.garden/guides/llm-agent-model-fallback-graceful-degradation-explained/ (Jun 2026)
17. https://www.kunalganglani.com/blog/chatgpt-down-api-outage-fallback (Aug 2026)
18. https://iamanuragh.in/blog/2026-06-27-graceful-degradation-serve-something-useful/ (Jun 2026)
19. https://champlinenterprises.com/blog/graceful-degradation-partial-system-failures (Jul 2026)

## What's NEW vs Batch 559

Batch 559 identified **consciousness architecture gaps** (free energy, GWT, structure learning, coarse-graining, multi-dimensional Φ). Batch 560 identifies **engineering resilience gaps** — the mechanisms that would keep a consciousness architecture running under real-world failure conditions. The key insight: a consciousness architecture without resilience patterns is a theoretical model, not a production system. The 14 defects found span error taxonomy (compile-time safety), circuit breaking (load shedding), degradation hierarchy (graceful failure), and recovery coordination (retry budgets, hedging, compaction). These are not optional hardening — they are prerequisites for any consciousness loop that must survive contact with unreliable external dependencies (LLM APIs, KB stores, crawl targets).
