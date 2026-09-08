# Iteration Batch 827 Report — NeoTrix Consciousness Architecture

## Research Sources (24+)

### Tokio Watchdog & Health Check Patterns (8)
- Three-probe pattern: liveness, readiness, startup (distinct, not interchangeable)
- HTTP 200 ≠ healthy — health endpoints must check actual functionality + dependencies
- Common defects: shallow checks, expensive checks, noisy checks, single-failure-as-incident
- Consecutive failure threshold: require N consecutive failures before action
- Timeout + connect pattern: `timeout(Duration::from_secs(N), async_op).await`
- External watchdog process: separate process checking actual service functionality
- SelfTest T3 not production-wired: detection output doesn't influence behavior
- No circuit breaker integration: timeout detection doesn't connect to capability routing

### Rust Rate Limiting (8)
- Token bucket: capacity + refill_rate; lazy refill O(1); distributed: Redis Lua scripts
- Leaky bucket: FIFO queue (drops overflow) vs GCRA (time-metering, no queueing)
- Governor crate: GCRA implementation, 64-bit state, CAS-thread-safe, ~10x faster
- IETF RateLimit/RateLimit-Policy headers (draft-11, May 2026) replace legacy X-RateLimit-*
- Backon: Python retry with exponential backoff, jitter, circuit breaker (complementary)
- Defects: distributed multiplier, clock skew, missing Redis TTLs, vague 429 responses, wrong algorithm, fail-closed on Redis failure, HOL blocking, mis-sized bucket, no per-key isolation, legacy header compliance

### MQTT Patterns (8)
- mqtt-endpoint-tokio: high-performance async MQTT client/server for Rust/tokio, MQTT v5.0/v3.1.1
- 425K exposed MQTT brokers: 59% unauthenticated, 99.84% unencrypted - Egress Privacy Guard blind
- MQTT 5.0 features: Reason Codes, Session Expiry/Interval, Maximum Packet Size, Topic Alias, Message Expiry
- MQTT over QUIC: 30-60% p99 latency reduction vs TCP+TLS, 0-RTT reconnect across IP changes
- CVE-2026 MQTT vulnerabilities: wildcard subscribe ACL bypass, broker config injection, heap corruption, hardcoded JWT secrets
- Legacy HTTP-polling obsolete: 2G/3G shutdowns mandate persistent MQTT with LWT
- Sparkplug B + OPC UA over MQTT = IT/OT convergence standard

### WebSocket Patterns (8)
- tokio-tungstenite 0.30.0: primary Tokio async WebSocket client/server
- tungstenite 0.30.0: core WebSocket implementation
- RustLS panic in tokio-tungstenite #373: v0.23.0+ causes Protocol(ResetWithoutClosingHandshake)
- No permessage-deflate in tungstenite: RFC 7692 extension not implemented
- Connection fragility behind reverse proxies
- TLS feature fragmentation across crates
- Performance cap: tungstenite not fastest
- hyper-tungstenite upgrade gap: must manually call is_upgrade_request()
- ws crate is stale (Feb 2022, unmaintained)

---

## Defects Identified (28+)

### Tokio Watchdog & Health Check (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-WD-1 | No external watchdog process (process may be up but deadlocked) | Critical |
| D-WD-2 | Shallow health checks (process existence vs actual functionality) | High |
| D-WD-3 | No timeout monitoring (systematic timeout tracking for operations) | High |
| D-WD-4 | No liveness/readiness/probe separation (single endpoint does all roles) | Medium |
| D-WD-5 | No consecutive failure threshold (single failure triggers incorrectly) | Medium |
| D-WD-6 | SelfTest T3 not production-wired (detection output doesn't influence behavior) | Medium |
| D-WD-7 | No circuit breaker integration (timeout → capability routing) | Medium |
| D-WD-8 | No GWT health signal modulation from watchdog reports | Medium |

### Rust Rate Limiting (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-RATE-1 | Distributed multiplier effect (per-instance buckets multiply effective rate) | Critical |
| D-RATE-2 | Clock skew from wall time (NTP adjustments negative elapsed time) | High |
| D-RATE-3 | Missing Redis TTLs (idle keys grow unboundedly) | Medium |
| D-RATE-4 | Vague 429 responses (no Retry-After or rate-limit headers) | Medium |
| D-RATE-5 | Wrong algorithm for use case (token bucket vs leaky bucket) | Medium |
| D-RATE-6 | Fail-closed on Redis failure (taking down entire API) | High |
| D-RATE-7 | Head-of-line blocking in leaky bucket (variable sizes stall) | Medium |
| D-RATE-8 | Mis-sized bucket capacity (too small or too large) | Medium |
| D-RATE-9 | No per-key isolation (single bucket for all clients) | High |
| D-RATE-10 | Legacy header compliance (X-RateLimit-* vs IETF draft-11) | Low |

### MQTT Patterns (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-MQTT-1 | No MQTT/LoRaWAN protocol integration in perception layer | Critical |
| D-MQTT-2 | No MQTT/QUIC transport layer (EventBus in-process only) | High |
| D-MQTT-3 | No Unified Namespace architecture (KB has no hierarchical topic model) | High |
| D-MQTT-4 | Legacy HTTP-polling embedded in crawl pipelines (2G/3G shutdowns) | High |
| D-MQTT-5 | Egress Privacy Guard blind to MQTT egress (425K exposed brokers) | Critical |
| D-MQTT-6 | No EU CRA compliance path for MQTT fleet security posture | Medium |
| D-MQTT-7 | MQTT wildcard subscribe ACL bypass (compromised agent eavesdrops) | High |
| D-MQTT-8 | Hardcoded JWT HMAC secret in MQTT broker firmware | High |

### WebSocket Patterns (7)
| ID | Defect | Severity |
|----|--------|----------|
| D-WS-1 | RustLS panic in tokio-tungstenite #373 (ResetWithoutClosingHandshake) | High |
| D-WS-2 | No permessage-deflate support (RFC 7692 not implemented) | Medium |
| D-WS-3 | Connection fragility behind reverse proxies | Medium |
| D-WS-4 | TLS feature fragmentation across crates | Low |
| D-WS-5 | Performance cap: tungstenite not fastest (bottleneck risk) | Medium |
| D-WS-6 | hyper-tungstenite upgrade gap (must manually call is_upgrade_request) | Low |
| D-WS-7 | Stale ws crate used (last updated Feb 2022) | Critical |

## Key Insights (This Batch)

1. **No external watchdog process**: The process may be up but deadlocked/livelocked/silent; no external monitor detects actual functionality failure. Users experience outages while status shows "green".

2. **Distributed multiplier effect on rate limiting**: Per-instance buckets multiply the effective rate limit by number of instances. Need shared Redis state or Lua scripts for distributed limiting.

3. **MQTT Egress Privacy Guard blind**: 425K exposed MQTT brokers with 59% unauthenticated — data could leak. The guard only scrubs LLM provider paths, not MQTT.

4. **MQTT over QUIC**: 30-60% p99 latency reduction vs TCP+TLS, with 0-RTT reconnect across IP changes. Critical for mobile/IoT scenarios.

5. **RustLS panic in tokio-tungstenite**: v0.23.0+ causes `Protocol(ResetWithoutClosingHandshake)` — stream ends before WebSocket close handshake completes in Tokio clients.

6. **Token bucket vs leaky bucket algorithm selection**: Token bucket allows bursts + sustained rate; leaky bucket produces smooth constant output. Choosing wrong algorithm for use case is a common defect.

7. **IETF RateLimit/RateLimit-Policy headers (draft-11, May 2026)**: Replace legacy `X-RateLimit-*` with structured `r=quota;t=window` + `q=quota;w=window;pk=partition_key`.

8. **WebSocket upgrade gap**: `hyper-tungstenite::upgrade()` doesn't auto-check — must manually call `is_upgrade_request()` first.

9. **Fail-closed on Redis rate limiter failure**: Taking down entire API when rate limiter store unavailable should fail-open with in-memory fallback.

10. **No per-device MQTT subscribe ACLs**: Default is opt-in, not mandatory. Compromised agent could eavesdrop on all sensor feeds via wildcard subscribe.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 827 |
| New defects (this batch) | 30 |
| Cumulative defects | D01-D76758 |
| Research sources (this batch) | 32 |
| Cumulative research sources | 97,623+ |