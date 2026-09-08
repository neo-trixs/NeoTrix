# Iteration Batch 670 — Service Mesh / API Gateway / gRPC External Scan

**Date**: 2026-09-06
**Context**: Batch 669 proved (1) zero structured logging, (2) no trace context propagation across SEAL phases, (3) no distributed tracing, (4) no PII scrubbing, (5) no KB audit trail. This batch scans service mesh, API gateway, and gRPC ecosystems for NEW defects.

---

## 1. Service Mesh Findings

### Source: Istio Ambient Mesh (2026)
- **URL**: https://www.buoyant.io/articles/linkerd-vs-istio-ambient-mode-an-operators-architecture-comparison-for-2026
- **Published**: 2026-06-10

**Key 2026 fact**: Istio Ambient uses a two-tier architecture: per-node `ztunnel` (Rust, L4) + per-namespace Envoy waypoints (L7). Linkerd uses a single-tier Rust microproxy per pod. The architectural distinction matters for NeoTrix because:

**DEFECT-670-01**: NeoTrix's `nt_shield` domain has no service mesh awareness. When NeoTrix runs in a Kubernetes deployment with Istio or Linkerd, there is:
- No mTLS peer identity extraction for module-to-module trust
- No W3C `traceparent` header forwarding through mesh proxies
- No waypoint/ztunnel health signal ingestion into `HeartbeatAggregator`
- No mesh-aware routing for NT-ACT tool calls across service boundaries

**DEFECT-670-02**: Linkerd 2.19 (Oct 2025) shipped post-quantum cryptography via ML-KEM-768 for mTLS. NeoTrix's `nt_shield` TLS configuration is static — no PQ algorithm negotiation, no hybrid key exchange support. If NeoTrix runs behind Linkerd 2.19 with PQ mTLS, the Node.js/Tauri client cannot negotiate the hybrid key exchange.

### Source: Cilium Service Mesh (2026)
- **URL**: https://cronfeed.work/oss-service-mesh-ecosystem-map-istio-ambient-linkerd-cilium-2026/
- **Published**: 2026-03-11

**Key 2026 fact**: Cilium converges mesh into eBPF networking layer with Envoy for L7. Supports Gateway API v1.4.1. Provides mutual authentication via SPIFFE/SPIRE identity flow.

**DEFECT-670-03**: NeoTrix has no SPIFFE/SPIRE identity integration. Every module identity is self-declared (Cargo.toml name), not cryptographically verified. In a Cilium-meshed environment, NT-CORE and NT-MIND cannot present verifiable workload identity for inter-module calls.

---

## 2. API Gateway Findings

### Source: APISIX vs Kong (2026)
- **URL**: https://api7.ai/apisix-vs-kong
- **Published**: 2026

**Key 2026 fact**: APISIX propagates config via etcd watch in milliseconds (no restart). Kong 3.10+ dropped free mode — OSS gateway is now effectively paywalled. APISIX supports HTTP/3, gRPC, MQTT, Dubbo natively with 100+ open-source plugins.

**DEFECT-670-04**: NeoTrix's `nt_io` LLM provider routing has no API gateway pattern. When multiple LLM providers exist (Ollama local, OpenAI, Anthropic), there is:
- No rate limiting per provider
- No circuit breaker (provider failure → crash, not graceful degradation)
- No request deduplication
- No health-check-based provider rotation
- No config hot-reload (provider changes require restart)

The `total_calls ascending` sort in CONTEXT.md is a primitive load balancer, not a gateway. A real gateway would track per-provider error rates, latency percentiles, and enforce budgets.

### Source: APISIX AI Gateway (2026)
- **URL**: https://apisix.apache.org/
- **Published**: 2026

**Key 2026 fact**: APISIX ships a built-in AI/LLM proxy plugin in open-source core — handles routing across LLM providers, caching, cost attribution in the same gateway layer.

**DEFECT-670-05**: NeoTrix has no LLM cost attribution. The `ResourceBudgetManager` (from batch 669 absorption) tracks token budgets but has no per-request cost tracking tied to provider billing. APISIX's ai-proxy plugin tracks cost per request per model — NeoTrix cannot answer "how much did this SEAL cycle cost?"

### Source: Kong Enterprise Pricing Shift (2026)
- **URL**: https://nomadlab.cc/blog/2026/05/kong-vs-apigee-vs-tyk-vs-apisix-vs-krakend-2026
- **Published**: 2026-05-18

**Key 2026 fact**: Kong OSS stopped at 3.9, Enterprise is at 3.15. Konnect charges $200/month per million requests. Apigee is now cheaper at volume ($20/M vs $200/M).

**DEFECT-670-06**: NeoTrix's external API call logging (e.g., LLM provider calls, crawl fetches) has no structured request/response logging with:
- Request ID correlation
- Latency distribution tracking
- Error classification (transient vs permanent)
- Provider billing event capture

Without this, NeoTrix cannot do cost optimization or provider SLA monitoring.

---

## 3. gRPC / Connect Findings

### Source: Connect RPC (2026)
- **URL**: https://connectrpc.com/
- **Published**: 2026

**Key 2026 fact**: Connect is a multi-protocol RPC library (gRPC + gRPC-Web + Connect protocol). Uses Protobuf-ES for type safety. Interoperates with Envoy, grpcurl, gRPC Gateway. The Connect protocol works over HTTP/1.1 (curl-compatible).

**DEFECT-670-07**: NeoTrix has no gRPC/Connect infrastructure. All inter-module communication is Rust function calls within a single process. When NeoTrix scales to multi-process (e.g., NT-WORLD crawler in separate process, NT-MIND reasoning in separate GPU process), there is:
- No protobuf schema definitions for cross-process messages
- No gRPC service definitions for module boundaries
- No HTTP/2 multiplexing for concurrent module calls
- No streaming support for long-running operations (crawl, reasoning)

### Source: OpenTelemetry Rust + Tonic gRPC (2026)
- **URL**: https://oneuptime.com/blog/post/2026-02-06-instrument-rust-tonic-grpc-opentelemetry/view
- **Published**: 2026-02-06

**Key 2026 fact**: The standard pattern for Rust gRPC observability is:
```rust
tracing + tracing-opentelemetry + opentelemetry-otlp + tonic-tracing-opentelemetry
```
This gives: structured logging via `tracing`, distributed traces via OTLP/gRPC, automatic context propagation via gRPC metadata.

**DEFECT-670-08**: NeoTrix's existing `nt_infra_tracing.rs` is a CUSTOM, IN-MEMORY tracing system that is completely isolated from the OpenTelemetry ecosystem:
- No `traceparent` header injection/extraction
- No span parent-child relationships (each span is flat)
- No W3C TraceContext compliance
- No OTLP export capability
- No correlation with `tracing` crate spans
- In-memory Vec with max 10000 spans — no persistence, no query
- Uses `uuid::Uuid::new_v4()` for span IDs (not W3C-compliant 64-bit span IDs)

This is the ROOT CAUSE of batch 669's "no trace context propagation" finding.

### Source: W3C Trace Context Propagation (2026)
- **URL**: https://opentelemetry.io/docs/concepts/context-propagation/
- **Published**: 2026

**Key 2026 fact**: W3C Trace Context defines two headers:
- `traceparent`: version-traceId-parentId-traceFlags (e.g., `00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01`)
- `tracestate`: vendor-specific key-value pairs

Context propagation is the mechanism that moves context between services. Without it, each service generates independent traces — impossible to link a single user request across services.

**DEFECT-670-09**: NeoTrix SEAL pipeline phases (Soil→Roots→Trunk→Branches→Fruits→Core) have no trace context propagation between phases. Each phase starts a new, disconnected trace. There is no `traceparent` injection into phase transitions. This means:
- Cannot trace a full evolution cycle end-to-end
- Cannot identify which SEAL phase is the bottleneck
- Cannot correlate KB writes with the SEAL phase that triggered them

### Source: OpenTelemetry 1.22 LatencyBasedSampler (2026)
- **URL**: https://johal.in/internals-opentelemetry-122-trace-propagation-debug-cross-service-late
- **Published**: 2026-04-26

**Key 2026 fact**: OTel 1.22 introduced `LatencyBasedSampler` — samples 100% of spans with latency above a configurable threshold, lower percentage below. Reduces overhead by 68% while capturing all slow requests. Debug propagation headers (`otel-debug-propagate`) can leak PII if enabled in production.

**DEFECT-670-10**: NeoTrix has no sampling strategy. The `nt_infra_tracing` captures ALL capability calls (up to 10000) with no sampling. In production with high-throughput crawling (NT-WORLD), this creates:
- Unbounded memory growth (10000 spans × ~200 bytes = 2MB, but real payloads are larger)
- No distinction between routine calls and anomalous calls
- No latency-based sampling to capture slow reasoning cycles
- No error-biased sampling to capture failures

### Source: OpenTelemetry PII in Trace Headers (2026)
- **URL**: https://opentelemetry.io/docs/concepts/context-propagation/
- **Published**: 2026

**Key 2026 fact**: "Be mindful of what you propagate to external services. Internal trace IDs, span IDs, or baggage items might reveal sensitive information about your internal architecture or business logic. Avoid putting sensitive information (like user credentials, API keys, or PII) in baggage."

**DEFECT-670-11**: NeoTrix has no PII scrubbing in any logging path. The `tracing::info!` calls throughout the codebase (52 locations found) freely log:
- File paths (`path.display()`)
- Session IDs
- Provider names and model names
- Error messages that may contain user input
- Token counts (indirectly revealing conversation content)

No log redaction layer exists. If NeoTrix logs are shipped to an external observability backend, PII leaks.

---

## 4. NEW Defects Summary

| ID | Severity | Domain | Description |
|----|----------|--------|-------------|
| DEFECT-670-01 | HIGH | NT-SHIELD | No service mesh awareness — no mTLS peer identity, no trace forwarding, no mesh health ingestion |
| DEFECT-670-02 | MEDIUM | NT-SHIELD | No post-quantum TLS negotiation — incompatible with Linkerd 2.19 PQ mTLS |
| DEFECT-670-03 | MEDIUM | NT-SHIELD | No SPIFFE/SPIRE identity — modules self-declare identity, no cryptographic verification |
| DEFECT-670-04 | HIGH | NT-ACT | No API gateway pattern for LLM providers — no rate limiting, circuit breaker, dedup, health rotation |
| DEFECT-670-05 | MEDIUM | NT-ACT | No LLM cost attribution per request — cannot answer "how much did this SEAL cycle cost?" |
| DEFECT-670-06 | MEDIUM | NT-IO | No structured external API call logging — no request correlation, latency tracking, error classification |
| DEFECT-670-07 | HIGH | NT-CORE | No gRPC/Connect infrastructure — blocks multi-process scaling of NT-WORLD/NT-MIND |
| DEFECT-670-08 | CRITICAL | NT-CORE | Custom `nt_infra_tracing` is isolated from OpenTelemetry — ROOT CAUSE of trace propagation failure |
| DEFECT-670-09 | HIGH | NT-MIND | SEAL phases have no trace context propagation — cannot trace end-to-end evolution cycles |
| DEFECT-670-10 | MEDIUM | NT-MEMORY | No sampling strategy — unbounded span capture, no latency/error-biased sampling |
| DEFECT-670-11 | HIGH | NT-SHIELD | No PII scrubbing in 52 logging locations — file paths, session IDs, provider details leaked |

---

## 5. Sources Cited

1. Buoyant.io — Linkerd vs Istio Ambient 2026 Architecture Comparison (2026-06-10)
2. cronfeed.work — Open-source service mesh ecosystem map 2026 (2026-03-11)
3. api7.ai — APISIX vs Kong comparison (2026)
4. nomadLab — Kong vs Apigee vs Tyk vs APISIX vs KrakenD 2026 (2026-05-18)
5. apisix.apache.org — APISIX official (2026)
6. connectrpc.com — Connect RPC (2026)
7. oneuptime.com — Instrument Rust Tonic gRPC with OpenTelemetry (2026-02-06)
8. opentelemetry.io — Context Propagation concepts (2026)
9. johal.in — OpenTelemetry 1.22 Trace Propagation (2026-04-26)
10. opentelemetry.io — Rust SDK documentation (2026-01-27)
11. matthewpalma.dev — Distributed tracing with W3C Trace Context (2026-04-03)
12. neotrix-core/src/unified/layers/action/nt_infra_tracing.rs — existing custom tracing (codebase)
13. neotrix-core/Cargo.toml — dependencies with optional telemetry feature (codebase)
