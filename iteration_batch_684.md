# Iteration 684 — Web Framework / HTTP Server / Request Handling Research

**Date**: 2026-09-06
**Context**: Batch 683 found no agentic governance layer, no FinOps-governance convergence, no active metadata management, 60% AI projects abandoned due to insufficient metadata, no quantum-safe encryption.

---

## 1. Web Framework Findings

### Sources
- https://abrarqasim.com/blog/rust-web-frameworks-2026-axum-actix-rocket/
- https://johal.in/rust-web-frameworks-actix-web-50-vs-axum-08
- https://ajmani.dev/best-rust-web-framework-2026/
- https://reintech.io/blog/axum-vs-actix-web-vs-rocket-rust-framework-comparison-2026
- https://www.youngju.dev/blog/culture/2026-05-16-rust-web-backend-frameworks-2026-axum-actix-web-rocket-poem-loco-pavex-salvo-warp-deep-dive.en
- https://developers-heaven.net/blog/choosing-your-web-framework-axum-vs-actix-web-vs-rocket/

### NEW Findings
1. **Axum 0.8 is now de facto standard** — 68% of new Rust backend projects choose between Axum/Actix/Rocket. Tower ecosystem composability is the decisive factor (72% boilerplate reduction for middleware-heavy apps).
2. **Actix-Web 5.0 released** — HTTP/3 support, ARM Graviton optimization, 142k req/s throughput on 16-core ARM (18% faster than Axum). Runtime mismatch remains a footgun (current-thread runtime vs multi-threaded tokio).
3. **Rocket 0.6 scheduled Q3 2026** — adds native WASM target support. Slower release cadence than Axum/Actix.
4. **New entrants**: Poem 3.x (OpenAPI-first), Pavex (compile-time DI), Loco.rs (Rails-style full stack), Salvo (rising in Asia).
5. **Hyper 1.x is the bedrock** — every framework (Axum, Actix, Poem, Salvo, Loco) uses Hyper internally. HTTP/3 support is in progress.
6. **Warp and Tide declining** — Warp's filter pattern losing to Tower composition; Tide fading with async-std.

### DEFECTS Identified
1. **DEFECT-684-01 (CRITICAL)**: NeoTrix `nt_io` module has no unified web framework abstraction. The codebase uses Axum-like patterns but lacks a formal `NtHttpServer` trait that could abstract over Axum/Actix/Tower. If NeoTrix ever needs to expose HTTP endpoints for agent communication, it will need this.
2. **DEFECT-684-02**: No framework benchmark integration in NeoTrix CI. The 42% throughput gap between frameworks under 10k concurrent connections means framework choice matters — we need a benchmark gate in the SEAL pipeline.
3. **DEFECT-684-03**: Actix runtime isolation (own tokio variant) is a deployment risk for NeoTrix if we mix Actix web handlers with tokio-based NT-CORE/NT-MIND services. No runtime compatibility check exists.

---

## 2. HTTP Server Findings

### Sources
- https://docs.rs/hyper-server/latest/hyper_server/
- https://hyper.rs/
- https://hyper.rs/guides/1/server/middleware/
- https://crates.io/crates/hyper-server
- https://docs.rs/hyperlite/latest/hyperlite/
- https://tower-rs.github.io/tower/tower/index.html
- https://docs.rs/tower-http/latest/tower_http/

### NEW Findings
1. **hyper-server 0.6.0** — `#![forbid(unsafe_code)]`, proxy protocol support for L4 load balancers, TLS via rustls/openssl. Forked from axum-server (unmaintained).
2. **Hyperlite** — lightweight framework on hyper/tower with matchit-based path routing, Tower middleware compatibility, graceful shutdown. Zero-macro, type-safe extractors.
3. **TowerToHyperService adapter** — converts Tower Service to hyper Service, enabling Tower middleware in raw hyper servers without axum. Key bridge for custom servers.
4. **Tower HTTP middleware ecosystem** — compression, tracing, CORS, request ID, rate limiting, body limiting, timeout, etc. All feature-gated, composable via ServiceBuilder.
5. **hyper 1.x split** — hyper no longer depends on tower for Service trait. Two bridge patterns: (a) `hyper::service::service_fn` + manual tower wrapping, (b) `TowerToHyperService` adapter.
6. **MSRV**: Tower 1.64.0, Hyper 1.x stable.

### DEFECTS Identified
4. **DEFECT-684-04 (HIGH)**: NeoTrix has no HTTP server abstraction at all. The codebase has `nt_io` for CLI/ACP/LSP but no embedded HTTP server for agent-to-agent communication or dashboard. The hyper-server crate with `#![forbid(unsafe_code)]` aligns with R-P1 — should be adopted.
5. **DEFECT-684-05**: No TLS termination strategy defined. hyper-server supports both rustls and openssl, but NeoTrix has no certificate management, no mTLS for agent mesh, no Let's Encrypt automation.
6. **DEFECT-684-06**: Proxy protocol support (PROXY protocol v1/v2) exists in hyper-server but NeoTrix has no L4 load balancer integration. For Kubernetes/cloud deployments behind ALB/NLB, this means client IP is lost.

---

## 3. Request Handling / Middleware / Rate Limiting Findings

### Sources
- https://crates.io/crates/tower-rate-limiter
- https://docs.rs/crate/rok-rate-limit/latest
- https://docs.rs/crate/tokio-rate-limit/0.8.0
- https://github.com/sadco-io/tokio-rate-limit
- https://crates.io/crates/barnacle-rs
- https://github.com/bzp2010/skp-ratelimit
- https://github.com/JhonaCodes/rate-limiter
- https://tutorialedge.net/projects/building-api-gateway-in-rust/part-4-rate-limiting/

### NEW Findings
1. **tokio-rate-limit v0.8.0** — 17.5M ops/sec deterministic, lock-free via flurry (Java ConcurrentHashMap port). Pluggable algorithms (token bucket, leaky bucket). Axum + Tonic gRPC middleware. Cost-based limiting. IETF RateLimit headers (draft-11). NEW: probabilistic mode (v0.7.0).
2. **tower-rate-limiter 0.1.0-alpha** — keyed fixed-window rate limiting. Tower-first, Axum/Redis optional. IETF draft-11 `RateLimit` and `RateLimit-Policy` headers. LimitProvider for dynamic quotas. Released Aug 2026.
3. **rok-rate-limit 0.3.0** — part of Rok Framework (Axum 0.8 + SQLx 0.8). Sliding window algorithm. JWT sub claim extraction for per-user limits. Redis backend with Lua atomic scripts.
4. **barnacle-rs 0.3.1** — Axum rate limiting + API key validation in one crate. Redis backend. Route-aware Redis keys (separates limits by path+method). Reset-on-success feature.
5. **skp-ratelimit** — 7 algorithms (GCRA, Token Bucket, Leaky Bucket, Sliding Log, Sliding Window, Fixed Window, Concurrent). Per-route quotas. Penalty system (errors increase cost). Credit system (cached responses give credit).
6. **IETF RateLimit headers becoming standard** — `RateLimit-Policy`, `RateLimit`, `Retry-After`. All new crates support draft-11. This is now a MUST for production APIs.

### DEFECTS Identified
7. **DEFECT-684-07 (CRITICAL)**: NeoTrix has no rate limiting middleware. NT-ACT exposes tools via MCP gateway but has zero request throttling. A misbehaving agent can flood the system. tokio-rate-limit's 17.5M ops/sec with lock-free architecture is ideal for integration.
8. **DEFECT-684-08**: No cost-based / weighted rate limiting. NeoTrix tool calls have vastly different costs (simple lookup vs full SEALE pipeline run). Weighted rate limiting (skp-ratelimit's penalty system or tokio-rate-limit's cost-based) is needed to prevent cheap-call abuse.
9. **DEFECT-684-09**: No per-tenant rate limiting for multi-agent scenarios. All agents share a single rate limit bucket. Need keyed rate limiting (per-agent-key or per-tenant) to isolate noisy neighbors.
10. **DEFECT-684-10**: No IETF-compliant rate limit headers in any NeoTrix response. Clients have no visibility into remaining quota or retry-after timing.
11. **DEFECT-684-11**: No dynamic quota adjustment (LimitProvider pattern). Rate limits are static. Need policy-driven quotas that adapt based on system health, time-of-day, or agent priority.
12. **DEFECT-684-12**: No distributed rate limiting across NeoTrix instances. In-memory stores don't work across multiple agent pods. Need Redis-backed stores for production multi-instance deployments.

---

## Summary: 12 New Defects Found

| ID | Severity | Domain | Description |
|----|----------|--------|-------------|
| 684-01 | CRITICAL | NT-IO | No unified HTTP framework abstraction (NtHttpServer trait) |
| 684-02 | MEDIUM | NT-IO | No framework benchmark integration in CI/SEAL pipeline |
| 684-03 | MEDIUM | NT-IO | Actix runtime isolation risk for mixed tokio services |
| 684-04 | HIGH | NT-IO | No embedded HTTP server for agent-to-agent/dashboard |
| 684-05 | HIGH | NT-SHIELD | No TLS termination / mTLS strategy for agent mesh |
| 684-06 | MEDIUM | NT-SHIELD | No proxy protocol support for L4 load balancer deployments |
| 684-07 | CRITICAL | NT-ACT | No rate limiting middleware on tool endpoints |
| 684-08 | HIGH | NT-ACT | No cost-based/weighted rate limiting for variable-cost tools |
| 684-09 | HIGH | NT-ACT | No per-tenant rate limiting for multi-agent isolation |
| 684-10 | MEDIUM | NT-ACT | No IETF-compliant rate limit headers in responses |
| 684-11 | MEDIUM | NT-ACT | No dynamic quota adjustment (LimitProvider pattern) |
| 684-12 | HIGH | NT-ACT | No distributed rate limiting across instances |

---

## Cross-Batch Gap Analysis (vs Batch 683)

| Batch 683 Gap | Batch 684 Connection |
|---------------|---------------------|
| No agentic governance layer | DEFECT-684-07: No rate limiting = no request-level governance |
| No FinOps-governance convergence | DEFECT-684-08: No cost-based rate limiting = no cost-aware governance |
| No active metadata management | DEFECT-684-10: No rate limit headers = no request metadata propagation |
| 60% AI projects abandoned | Rate limiting prevents cascading failures that kill projects |
| No quantum-safe encryption | No TLS strategy at all (DEFECT-684-05), let alone post-quantum |

---

## Recommended Immediate Actions

1. **Integrate tokio-rate-limit** into NT-ACT MCP gateway — lock-free, 17.5M ops/sec, IETF-compliant
2. **Adopt hyper-server** as HTTP server base — `#![forbid(unsafe_code)]` aligns with R-P1
3. **Define NtHttpServer trait** in nt_io — abstract over Tower/Axum for framework independence
4. **Add IETF RateLimit headers** to all NT-ACT responses — draft-11 compliance
5. **Implement cost-weighted rate limiting** — different tool calls cost different amounts
6. **Plan Redis-backed distributed rate limiting** — for multi-instance deployments
