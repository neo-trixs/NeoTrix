# Agent 4: HTTP/2 gRPC Patterns (Batch 863)

## Sources

1. **ADHDecode** — "Rust gRPC with Tonic: Production Patterns (2026)" — adhdecode.com (2026-04-18)
2. **OneUptime** — "How to Instrument Rust Tonic gRPC Services with OpenTelemetry" — oneuptime.com (2026-02-06)
3. **Markaicode** — "Building Microservices with Rust's Tonic gRPC: Production Best Practices" — markaicode.com (2025-05-18)
4. **Developers Heaven** — "Advanced gRPC/Tonic: Streaming, Interceptors, and Load Balancing" — developers-heaven.net (2026-06-25)
5. **OneUptime** — "How to Build Bidirectional gRPC Streaming with tonic in Rust" — oneuptime.com (2026-01-25)
6. **OneUptime** — "How to Build gRPC Services in Rust" — oneuptime.com (2026-01-08)
7. **Atharva Pandey** — "Lesson 2: gRPC Microservices with tonic — Production-grade RPC" — atharvapandey.com (2025)
8. **DEV Community** — "gRPC Streaming in Rust: The Four Patterns You Need to Know (with Tonic)" — dev.to (2026-03-27)
9. **gRPC.io Blog** — "gRPC-Rust Client API Evolution (pt. 1/2)" — grpc.io (2026-05-29)
10. **hyperium/hyper Issue #4049** — "HTTP/2 CONNECT Upgraded stream bypasses H2 flow-control backpressure, causing OOM" — github.com (2026-04)
11. **hyperium/hyper PR #4050** — "fix(h2): return Poll::Pending when poll_capacity is not ready in UpgradedSendStreamTask" — github.com (2026-04-09)
12. **TrueLayer** — "gRPC load balancing in Rust" — truelayer.com (2021-05-13)
13. **ecliptical/tonic-lb-k8s** — "Tonic client load balancing for Kubernetes" — github.com
14. **hyperium/tonic Issue #3652** — "Issues with client termination of H2 CONNECT streams" — github.com (2024-04-29)
15. **hyper h2 crate docs** — docs.rs/h2 — flow control, SendStream, FlowControl documentation
16. **hyper BDP implementation** — feat(http2): add adaptive window size support using BDP (#2138) — github.com

## Defects

### D-GRPC-001: HTTP/2 flow-control backpressure bypass in CONNECT tunnel path — identical pattern exists in NeoTrix SSE relay
**File:** `nt_io_provider/gateway/mod.rs:546` (mpsc::channel(1) used as stream relay bridge)
**Severity:** P2 — potential unbounded memory growth under asymmetric load
**Source:** hyperium/hyper#4049 (2026-04), hyper h2 BDP docs

NeoTrix gateway provider streaming uses `mpsc::channel(1)` as a bridge between upstream LLM response and downstream SSE relay (line 546). The hyper#4049 defect demonstrates that when an `mpsc::channel(1)` is used as a bridge between a fast producer and a slow consumer, and the consumer side does not properly propagate `Poll::Pending` when the downstream connection cannot accept more data, backpressure breaks and memory grows unbounded. NeoTrix's `stream_complete_raw` implementations across all providers (openai:232, gemini:153, anthropic:255, ollama:133, free_providers:151/340/509/676) all use `mpsc::channel(64)` or `mpsc::channel(1)` without explicit backpressure propagation from the SSE sink. If the SSE connection stalls (client slow read, network jitter), the mpsc channel fills, the spawned task blocks on `tx.send().await`, but the upstream HTTP response body reader may already be buffering in memory. The gap: NeoTrix lacks a `poll_capacity()` equivalent that would signal the upstream to stop reading when the downstream channel is full.

### D-GRPC-002: Missing HTTP/2 `INITIAL_WINDOW_SIZE` tuning for NT-WORLD crawl connections
**File:** `nt_shield_traffic/fingerprint.rs:14` (http2_settings defined but never applied to connection config)
**Severity:** P3 — suboptimal crawl throughput for large page fetches
**Source:** hyper server::conn::http2 Builder docs, h2 crate flow control docs

NeoTrix's `TlsFingerprint` struct defines `http2_settings` including `INITIAL_WINDOW_SIZE` values (Chrome=6MB, Firefox=131KB, Safari=4MB, Edge=6MB) at `fingerprint.rs:30-73`, but these values are only used for TLS fingerprint spoofing — they are never applied as actual HTTP/2 connection configuration when NeoTrix creates outbound connections for NT-WORLD crawling. The h2 crate defaults `INITIAL_WINDOW_SIZE` to 65535 bytes, which is far below what browsers negotiate. For large page fetches (HTML+assets), this means NeoTrix's crawl connections operate with 1/100th the flow control window of a real browser, causing unnecessary round-trips for window updates and reducing throughput.

### D-GRPC-003: No gRPC health check protocol implementation — inter-domain communication unobservable
**File:** `nt_io_provider/gateway/mod.rs` (entire gateway has no health check endpoint)
**Severity:** P3 — cannot detect degraded LLM provider health proactively
**Source:** Atharva Pandey (2025) — "Every gRPC service needs to implement the standard health checking protocol"; tonic-lb-k8s production hardening docs

The gRPC Health Checking Protocol (grpc.health.v1.Health/Check) is the standard mechanism for service meshes, load balancers, and orchestrators to detect service health. NeoTrix's gateway has its own circuit breaker and `success_ema` tracking, but no standardized health check endpoint. This means: (1) external orchestrators cannot query NeoTrix's LLM gateway health, (2) when NeoTrix is deployed as a sidecar or embedded service, health probes must use ad-hoc HTTP endpoints rather than the standard gRPC health protocol, and (3) the gateway cannot propagate structured health status (SERVING/NOT_SERVING/UNKNOWN) to upstream consumers. The absence is particularly notable given NeoTrix already has `HeartbeatAggregator` for internal health — the gap is the external-facing standardized protocol.

### D-GRPC-004: No deadline/timeout propagation across NT-IO provider chain — zombie connection risk
**File:** `nt_io_provider/gateway/mod.rs:315` (test_select_best_load_balance shows no deadline on requests)
**Severity:** P2 — hung downstream provider can hold connections indefinitely
**Source:** Developers Heaven (2026-06): "Deadlines: Always set explicit timeouts on your calls to prevent hanging connections"; Atharva Pandey: "Set deadlines on everything. If you don't set a timeout, a hung downstream service will hold connections forever."

NeoTrix's `LlmRequest` does not carry a deadline field. When the gateway dispatches to a provider (openai, gemini, anthropic, free_providers), there is no mechanism to propagate a deadline from the original user request through the provider chain. In gRPC, deadline propagation is fundamental — each hop decrements the remaining time, and if a downstream service is slow, the caller detects the deadline exceeded error rather than hanging. NeoTrix's `stream_complete_raw` implementations use `mpsc::channel(64)` with spawned tasks, but if the LLM provider hangs (e.g., upstream API stuck), the spawned task and channel remain alive indefinitely. The only timeout is whatever the HTTP client library applies, which is typically a connect timeout, not a per-request deadline.

### D-GRPC-005: No idempotency key for LLM provider retries — duplicate side effects risk
**File:** `nt_io_provider/gateway/mod.rs:546` (stream retry path re-sends same request without idempotency)
**Severity:** P2 — duplicate LLM calls waste tokens and may cause non-deterministic behavior
**Source:** Atharva Pandey (2025): "Notice the idempotency key. This is critical for retries — if a request succeeds on the server but the response is lost due to a network blip, the retry will return the same order instead of creating a duplicate."

NeoTrix's gateway performs aggressive retry on transient streaming failures (test at line 577-588 shows retry behavior). However, `LlmRequest` has no idempotency key field. When the gateway retries a failed stream request to a different provider (or the same provider), the LLM API may process it as a new request, consuming tokens and potentially generating different outputs. For stateful operations like tool-calling chains or structured output generation, this can cause inconsistent state. Production gRPC patterns use idempotency keys explicitly to ensure retries are safe. NeoTrix's `LlmRequest::new()` takes `system` and `user` strings but no request-level unique identifier for deduplication.

### D-GRPC-006: Adaptive flow control (BDP) missing from NT-WORLD crawl pipeline
**File:** `nt_world_crawl/resilient.rs:272` (backoff logic exists but no BDP-equivalent adaptive tuning)
**Severity:** C2 — crawl throughput limited to static connection defaults
**Source:** hyper BDP implementation (#2138), h2 adaptive window docs, hyper::proto::h2::ping module

NeoTrix's crawl pipeline (`resilient.rs`) implements exponential backoff for retry, but the underlying HTTP/2 connections use default (non-adaptive) flow control windows. The BDP (Bandwidth-Delay Product) algorithm, implemented in hyper's `proto::h2::ping` module, dynamically adjusts the HTTP/2 window size based on measured RTT and throughput. For NT-WORLD's diverse crawl targets (varying latency from 50ms to 5000ms+), static window sizes mean: low-latency targets are under-utilized (window too small), high-latency targets stall (window update round-trips dominate). Enabling `adaptive_window(true)` on the HTTP/2 client builder would allow the connection to self-tune, potentially improving crawl throughput by 2-5x for high-bandwidth targets.

### D-GRPC-007: No HTTP/2 `MAX_CONCURRENT_STREAMS` awareness — connection saturation risk
**File:** `nt_io_provider/gateway/mod.rs` (no concurrent stream limit per provider connection)
**Severity:** C3 — can overwhelm a single provider with unbounded concurrent requests
**Source:** hyper client::conn::http2 Builder docs: "The maximum concurrent streams setting only controls the maximum number of streams that can be initiated by the remote peer"; tonic_lb_k8s docs

NeoTrix's gateway allows unbounded concurrent `stream_complete_raw` calls to a single provider. HTTP/2 servers typically set `MAX_CONCURRENT_STREAMS` (e.g., 100-256) to prevent resource exhaustion. When NeoTrix opens many concurrent streams to a single LLM provider, the provider's HTTP/2 server will reset streams exceeding its limit, causing `REFUSED_STREAM` errors. NeoTrix should track the provider's advertised `MAX_CONCURRENT_STREAMS` and implement client-side concurrency limiting (via Tower's `ConcurrencyLimit` layer or equivalent) to stay within the server's capacity. The `tonic::transport::Server` builder's `concurrency_limit_per_connection(32)` pattern (from markaicode.com) is the production standard.

### D-GRPC-008: SSE relay lacks GOAWAY-aware reconnection — stale endpoint after provider restart
**File:** `nt_io_provider/gateway/mod.rs:546-563` (spawned task has no connection lifecycle awareness)
**Severity:** C3 — silent failure after provider pod restart in K8s
**Source:** tonic-lb-k8s production hardening docs: "Long-lived idle HTTP/2 connections to a removed pod can stay open until the next request, at which point the client gets a connection error"; ecliptical/tonic-lb-k8s

NeoTrix's streaming relay spawns a task (line 547) that reads from the LLM response and sends to the mpsc channel. When an LLM provider restarts (e.g., Ollama model reload, cloud provider maintenance), the existing HTTP/2 connection may receive a GOAWAY frame or the connection may silently die. The spawned task will only discover this when it tries to read the next chunk, potentially hanging indefinitely if the connection is in a half-open state. Production gRPC clients implement GOAWAY detection via `h2::RecvStream::poll_reset()` and proactive reconnection. NeoTrix's providers lack this — the circuit breaker only trips on explicit errors, not on silent connection death.

## Key Insights

1. **NeoTrix has no gRPC** — It uses HTTP/SSE for all LLM communication. The gRPC research reveals that NeoTrix's streaming architecture (mpsc channels + spawned relay tasks) is structurally similar to tonic's internal patterns but lacks the production hardening that tonic/hyper provide (deadline propagation, flow control awareness, health checks, idempotency).

2. **Backpressure is the critical gap** — The hyper#4049 OOM bug (165MB→8GB in 2 minutes) demonstrates that mpsc channels used as bridges between fast producers and slow consumers are a well-known failure mode. NeoTrix's gateway streaming pattern is identical to the bug's reproduction path: upstream HTTP response → mpsc channel → downstream SSE sink. Without explicit backpressure propagation, a slow SSE client can cause unbounded memory growth.

3. **HTTP/2 fingerprint spoofing without flow control tuning** — NeoTrix spoofs browser TLS fingerprints (JA3/JA4) including HTTP/2 settings like INITIAL_WINDOW_SIZE, but never applies these values to actual connection configuration. This is a wasted opportunity: the fingerprint library could double as a performance tuning layer.

4. **Load balancing is application-level, not transport-level** — NeoTrix's gateway `select_best()` performs provider selection at the application layer. Production gRPC systems use transport-level load balancing (Tower `balance_channel`, `tonic_lb_k8s`) that operates at the HTTP/2 connection level, distributing streams across backends within a single connection. NeoTrix's approach works for different providers (OpenAI vs Gemini) but not for scaling a single provider across replicas.

5. **Missing production gRPC primitives** — The research reveals 5 standard gRPC production patterns absent from NeoTrix: (a) deadline/timeout propagation, (b) health check protocol, (c) idempotency keys for retries, (d) GOAWAY-aware reconnection, (e) adaptive flow control (BDP). These are not gRPC-specific — they apply to any HTTP/2-based streaming system.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 8 |
| Sources analyzed | 16 |
| P2 defects | 3 |
| P3 defects | 2 |
| C2 defects | 1 |
| C3 defects | 2 |
