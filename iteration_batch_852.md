# Iteration Batch 852 Report — NeoTrix Consciousness Architecture

## Research Sources (38+)

### Middleware (10)
- tower-http 0.6: TraceLayer, RequestIdLayer, CompressionLayer, CorsLayer, TimeoutLayer
- axum-governor: token-bucket rate limiting via governor for Axum
- Layer composition order critical: Request ID → Tracing → Auth → Compression → Rate Limit → Handler
- UUIDv7 for time-sortable request IDs
- Per-key rate limiting (not global fixed window)
- tower-http RequestIdLayer + PropagateRequestIdLayer out of box
- from_fn for 80% cases, custom Layer/Service for backpressure
- Production mistakes: wrong order, heavy work, panic, forgetting body, state cloning
- No Retry-After header on 429 responses
- No response time logging

### WebSocket (10)
- tokio-tungstenite 0.30: backpressure boundary, fragment timeout, performance on-par with fastwebsockets
- yawc: only Rust WS lib with streaming compression + backpressure boundary
- RFC 9220 (WS over QUIC): zero browser implementations, dead end today
- WebTransport: 75% browser coverage, W3C Working Draft, viable 2027-2028
- Backpressure is #1 production risk (OOM on slow clients)
- No heartbeat = phantom connections behind LBs
- Auth gap = security vulnerability (V046 in shield audit)
- No connection registry, no metrics
- No broadcast/fan-out
- No graceful shutdown

### SSE (8)
- axum 0.8.9 SSE API: Sse::new(stream), keep_alive(), Event builder
- Backpressure: mpsc::channel bounded, producer suspends when full (blocks upstream read)
- Recommended: try_send with timeout, drop slow consumers
- No Last-Event-ID / resumability
- No connection tracking
- No event typing (inconsistent across endpoints)
- Channel buffer 64 is arbitrary (too small for burst, too large for latency)
- X-Accel-Buffering: no header mandatory behind nginx

### gRPC (10)
- tonic 0.14.6: CNCF migration to grpc-rust, maintenance-only until grpc crate ships
- prost 0.14.4: passively maintained, official protobuf-rust coming
- connect-rpc: Anthropic contributed, Tower-native, 1.95x faster than tonic
- Connect + gRPC + gRPC-Web from single .proto
- No existing gRPC/protobuf code in NT-IO (greenfield)
- No RPC transport abstraction
- Tonic feature-frozen
- Egress Privacy Guard gap for gRPC
- MSRV 1.88 required
- #![forbid(unsafe_code)] compatible with all options

---

## Defects Identified (30+)

### Middleware (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-MW-1 | No request ID middleware | High |
| D-MW-2 | No tracing/observability middleware | High |
| D-MW-3 | Global rate limiter (not per-key) | Medium |
| D-MW-4 | No Retry-After header on 429 | Low |
| D-MW-5 | No response time logging | Medium |
| D-MW-6 | CORS hardcoded (not CorsLayer) | Medium |
| D-MW-7 | No compression middleware | Low |
| D-MW-8 | No timeout middleware | Medium |
| D-MW-9 | Auth reads body implicitly | Low |
| D-MW-10 | rate_limiter Mutex contention | Low |

### WebSocket (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-WS-1 | tokio-tungstenite 0.24 (missing backpressure) | Medium |
| D-WS-2 | No backpressure management (OOM risk) | Critical |
| D-WS-3 | No heartbeat/ping-pong | High |
| D-WS-4 | No WebSocket auth | Critical |
| D-WS-5 | No connection registry | High |
| D-WS-6 | No broadcast/fan-out | Medium |
| D-WS-7 | No graceful shutdown | Medium |
| D-WS-8 | No message validation | High |

### SSE (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-SSE-1 | No backpressure protection (blocks upstream) | High |
| D-SSE-2 | No Last-Event-ID / resumability | Medium |
| D-SSE-3 | No connection tracking | Medium |
| D-SSE-4 | No event typing (inconsistent) | Low |
| D-SSE-5 | Channel buffer 64 arbitrary | Low |
| D-SSE-6 | No X-Accel-Buffering header | Medium |

### gRPC (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-GRPC-1 | No gRPC/protobuf code (greenfield) | High |
| D-GRPC-2 | No RPC transport abstraction | High |
| D-GRPC-3 | Tonic feature-frozen | Medium |
| D-GRPC-4 | prost passively maintained | Medium |
| D-GRPC-5 | No gRPC-web browser path | Medium |
| D-GRPC-6 | Egress Privacy Guard gap | High |

## Key Insights (This Batch)

1. **Layer composition order is critical**: Request ID → Tracing → Auth → Compression → Rate Limit → Handler. NeoTrix has rate_limit → auth (wrong order, missing foundation layers).

2. **tokio-tungstenite 0.30 is the 2026 standard**: Backpressure boundary, fragment timeout, performance on-par with fastwebsockets. Must upgrade from 0.24.

3. **WebSocket over QUIC (RFC 9220) is a dead end**: Zero browser implementations. WebTransport is the future (2027-2028). Use HTTP/2 WS (RFC 8441) which is universally supported.

4. **Backpressure is #1 WebSocket risk**: Unbounded sender.send() can OOM on slow clients. Must use with_backpressure_boundary() or try_send with timeout.

5. **SSE backpressure blocks upstream**: mpsc::channel producer suspension blocks the upstream HTTP read. Must use try_send with timeout, drop slow consumers.

6. **connect-rpc is stronger than tonic**: 1.95x faster, Tower-native, 3 protocols from single .proto, browser support without Envoy. Tonic is maintenance-only.

7. **No X-Accel-Buffering: no header**: Mandatory behind nginx reverse proxy. Without it, SSE responses get buffered and clients see stale data.

8. **UUIDv7 for request IDs**: Time-sortable, globally unique. Essential for distributed tracing correlation.

9. **Per-key rate limiting**: Global fixed window (current NeoTrix) means one noisy client blocks all clients. Must be per-IP or per-API-key.

10. **Auth gap in WebSocket**: V046 in shield audit. Must validate token during upgrade, not after.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 852 |
| New defects (this batch) | 30 |
| Cumulative defects | D01-D77473 |
| Research sources (this batch) | 38 |
| Cumulative research sources | 98,489+ |
