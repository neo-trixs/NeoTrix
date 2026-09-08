# Iteration Batch 695 — Protocol Transport Defects (WebSocket / gRPC / SSE)

## Batch 694 Recap

Batch 694 identified: (1) `serde_json::Value` eliminates compile-time schema enforcement, (2) no incremental type-check cache, (3) no typed fast-path for hot module boundaries (4-25× speedup possible), (4) no formal resource safety for sensor/GPU, (5) no predicate normalization for KB queries.

## New Findings from Protocol Transport Search

### 1. WebSocket: Head-of-Line Blocking + No Unreliable Mode

**Source**: youngju.dev (2026-05-14), FOSDEM 2026 WebTransport talk, WHATWG Living Standard (2026-03-15)

**New Defect D695-1**: NeoTrix NT-IO has no transport abstraction layer — `docs/3-API/events.md:16` exposes raw `ws://localhost:3456/ws` with no protocol negotiation, no multiplexing, and no fallback chain. Every WebSocket message blocks subsequent messages on the same TCP connection (head-of-line blocking). For real-time perception data (L2 NT-WORLD sensor streams) this creates tail-latency spikes that propagate through GWT attention routing.

**New Defect D695-2**: WebSocket offers no unreliable mode. NeoTrix NT-PHYSICAL sensor fusion requires both reliable (commands) and unreliable (telemetry frames) channels on one connection. WebSocket forces all traffic through one reliable ordered pipe — a lost packet stalls the entire stream, even when newer telemetry makes old frames obsolete. WebTransport (RFC 9484, finalized 2024) solves this with reliable streams + unreliable datagrams over QUIC, but NeoTrix has zero WebTransport integration.

**New Defect D695-3**: WebSocket reconnection logic is manual (no built-in `Last-Event-ID` or exponential backoff). NeoTrix `nt_io_session_recovery.rs` exists but has no protocol-level resume — on disconnect, the session state is rebuilt from scratch rather than replaying missed events. SSE's `Last-Event-ID` + `retry` header pattern is not implemented.

**New Defect D695-4**: WebSocket requires sticky sessions for backend pinning. NeoTrix's gateway `gateway/selection.rs` uses `total_calls ascending` rotation without session affinity — a WebSocket client reconnecting after failure may hit a different backend with no session state, losing in-flight attention routing context.

### 2. gRPC Bidirectional: No Proto Schema Enforcement at Transport

**Source**: grpc.io core concepts (2026-05-11), gRPC 1.60 internals (2026-05-05), oneuptime.com (2026-01-24)

**New Defect D695-5**: gRPC enforces schema at the wire level via protobuf but NeoTrix NT-IO uses `serde_json::Value` everywhere (batch 694 finding #1). The mismatch is architectural: gRPC's `.proto` files generate typed Rust structs at compile time, eliminating the class of bugs where payload shape drifts from handler expectations. NeoTrix's JSON-over-HTTP gateway (`gateway/registry.rs`, `gateway/selection.rs`) has zero compile-time schema enforcement — every provider response is parsed with `.as_object()?.get("key")?` chains that silently return `None` on schema drift.

**New Defect D695-6**: gRPC 1.60 introduced work-stealing scheduler reducing head-of-line blocking by 73% and per-message memory by 52% via zero-copy protobuf deserialization. NeoTrix has no equivalent — `nt_io_provider/openai.rs`, `anthropic.rs`, `gemini.rs` each allocate and deserialize independently. A unified protobuf-based transport with shared buffer pools would cut allocation overhead significantly for the hot path (LLM token streaming).

**New Defect D695-7**: gRPC's `te: trailers` mandatory header enables late error status after streamed response body. NeoTrix's SSE streaming (`iteration_batch_413` noted SSE for `/api/reason`) has no equivalent — if the server sends 50 tokens then errors, the client sees a truncated stream with no structured error payload. The `grpc-status-details-bin` base64-encoded `google.rpc.Status` pattern (structured error details) is absent from NeoTrix's error handling.

### 3. SSE: Auth Token Leakage + No Binary Support

**Source**: MDN Server-sent events (2026-09-03), server-sent-events.com protocol fundamentals (2026-05-12), WHATWG HTML spec 9.2

**New Defect D695-8**: `EventSource` only issues GET requests and cannot set custom `Authorization` headers. This was noted in batch 585, but the deeper defect is architectural: NeoTrix's NT-IO web layer (`nt_io_web/server.rs`) has no credential injection middleware that maps SSE connections to short-lived tokens via cookie or query parameter. Every SSE endpoint either (a) leaks auth tokens in URL query params (logged by proxies), or (b) uses no auth at all (trusted local network assumption that breaks in multi-tenant deployment).

**New Defect D695-9**: SSE is UTF-8 text only — no binary frame type. NeoTrix NT-WORLD sensor data (binary telemetry, VSA embeddings, HyperCube state vectors) cannot travel over SSE without base64 encoding, adding 33% overhead. The protocol gap: NeoTrix needs a hybrid transport that sends structured JSON control events via SSE but binary payloads via a side channel (WebSocket binary frames or WebTransport datagrams). No such hybrid exists.

**New Defect D695-10**: SSE connection limit is 6 per browser+domain on HTTP/1.1 (Chrome/Firefox "Won't fix"). NeoTrix's desktop Tauri app opens multiple SSE streams for different subsystems (emotion state, perception, attention routing). Under HTTP/1.1, hitting the 6-connection ceiling silently stalls the 7th+ stream. HTTP/2 multiplexes to 100 concurrent streams, but NeoTrix's `nt_io_web/server.rs` does not negotiate or enforce HTTP/2 — it falls through to HTTP/1.1 if the server config doesn't explicitly enable it.

**New Defect D695-11**: SSE heartbeat responsibility is server-side only. The spec defines `:comment` lines as keep-alive, but NeoTrix has no heartbeat generation in `nt_io_web/server.rs`. If a proxy's idle timeout (typically 60s) fires before the next data event, the connection dies silently. The client's `EventSource` auto-reconnects but loses the window of events between drop and reconnect. No idempotent replay buffer exists on the server side.

### 4. Cross-Protocol Architectural Gap

**New Defect D695-12**: No unified transport abstraction exists in NeoTrix. WebSocket, SSE, and gRPC serve different purposes:
- SSE: one-way server→client (LLM token streams, dashboards)
- WebSocket: bidirectional low-latency (chat, collaborative editing)
- gRPC: schema-enforced service-to-service (internal RPC)
- WebTransport: unreliable multi-stream (sensor telemetry, game state)

NeoTrix treats all real-time communication as "HTTP endpoint" without a protocol negotiation layer. The `nt_io_web/mod.rs` file has no `TransportStrategy` enum, no fallback chain (SSE→WS→Long Polling), and no protocol capability advertisement. Clients cannot negotiate the optimal transport for their use case.

**New Defect D695-13**: No backpressure propagation across protocols. gRPC has native HTTP/2 flow control (`WINDOW_UPDATE` frames). WebSocket has manual `bufferedAmount` checking. SSE has no flow control at all — if the client is slow, events pile up in the TCP send buffer until OOM or proxy timeout. NeoTrix's emotion state broadcasting (`nt_feel`) and attention routing (`GWT`) generate events at the rate of system activity with no adaptive throttling or consumer-driven backpressure.

### Summary of New Defects (D695-1 to D695-13)

| ID | Protocol | Defect | Severity |
|---|---|---|---|
| D695-1 | WS | No protocol negotiation, raw `ws://` exposed | P2 |
| D695-2 | WS | No unreliable mode for sensor telemetry | P1 |
| D695-3 | WS | No `Last-Event-ID` resume, full session rebuild on reconnect | P1 |
| D695-4 | WS | No sticky session affinity in gateway rotation | P2 |
| D695-5 | gRPC | No proto schema enforcement, JSON drift undetected at wire | P1 |
| D695-6 | gRPC | No shared buffer pool / zero-copy deserialization | P2 |
| D695-7 | gRPC | No structured late-error status after streaming body | P2 |
| D695-8 | SSE | Auth token leakage, no credential injection middleware | P1 |
| D695-9 | SSE | No binary frame type, 33% base64 overhead for sensor data | P2 |
| D695-10 | SSE | 6-connection HTTP/1.1 ceiling, no HTTP/2 enforcement | P2 |
| D695-11 | SSE | No server heartbeat generation, silent proxy timeout | P2 |
| D695-12 | ALL | No unified transport abstraction / protocol negotiation | P1 |
| D695-13 | ALL | No backpressure propagation across protocols | P1 |

## Sources

1. youngju.dev — "Realtime Web in 2026 — WebSocket vs SSE vs WebTransport vs WebRTC" (2026-05-14)
2. WHATWG — WebSockets Living Standard, last updated 2026-03-15
3. FOSDEM 2026 — Max Inden, "Intro to WebTransport" (2026-03-29, InfoQ)
4. grpc.io — Core concepts, architecture and lifecycle (2026-05-11)
5. gRPC 1.60 internals — johal.in (2026-05-05)
6. oneuptime.com — "How to Handle Bidirectional Streaming in gRPC" (2026-01-24)
7. MDN — Using server-sent events (2026-09-03)
8. server-sent-events.com — Protocol fundamentals & architecture (2026-05-12)
9. WHATWG HTML spec — 9.2 Server-sent events

## Cumulative Defect Count

Batches 1-694: 4,160 defects
Batch 695: +13 defects (D695-1 through D695-13)
**Total: 4,173 defects**
