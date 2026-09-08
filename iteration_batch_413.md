# Iteration Batch #413 — External Research: REST API, GraphQL, gRPC (2026 State-of-the-Art)

**Date**: 2026-09-06
**Research Scope**: REST API design, GraphQL federation, gRPC/Connect-RPC (2026 advances)
**Design Doc Reviewed**: `docs/6-REFERENCE/openapi.yaml`, `nt_io_web/api.rs`, `nt_io_web/server.rs`, CONTEXT.md, AGENTS.md

---

## Sources Cited

### REST API (2026)

| # | Source | URL | Key Finding |
|---|--------|-----|-------------|
| S1 | REST API Design in 2026: Engineering Reference | https://www.digitalapplied.com/blog/rest-api-design-2026-engineering-reference-best-practices | OpenAPI 3.2.0 released Sept 2025 (back-compatible with 3.1); RFC 9457 is standard for API errors since July 2023; cursor pagination mandatory at scale; idempotency keys with Stripe's trap (cache only after execution begins); header-based versioning recommended for internal APIs; HATEOAS earns complexity for workflow APIs |
| S2 | REST API Design 2026 (Precision AI Academy) | https://precisionaiacademy.com/blog/rest-api-design-guide-2026 | OpenAPI 3.1+ with JSON Schema 2020-12 alignment; 40% reduction in integration bugs with API-first design; 10x faster SDK generation; rate-limit at multiple granularities (per-second burst + per-minute sustained + per-day quota) |
| S3 | REST API Design Trends 2026 (SesameDisk) | https://sesamedisk.com/rest-api-design-2026-trends/ | RFC 7807/9457 adoption now mainstream; header-based versioning rising for internal/partner APIs; deprecation headers (Sunset, Deprecation) now required by governance teams; automated contract testing closing doc-implementation gap |
| S4 | REST API Best Practices 2026 (Anakin.ai) | https://anakin.ai/blog/best-api-best-practices-2026/ | HTTP/3 now mainstream; OpenAPI 3.1 JSON Schema compatibility; JSON:API gained traction while GraphQL hype cooled; multi-granularity rate limiting; machine-readable everything |
| S5 | Fern: API Design Best Practices Guide | https://buildwithfern.com/post/api-design-best-practices-guide | OpenAPI-first as single source of truth; automated SDK generation with semantic versioning + CI/CD; diff tools detect breaking changes before deployment |

### GraphQL (2026)

| # | Source | URL | Key Finding |
|---|--------|-----|-------------|
| S6 | Zylos Research: GraphQL in 2026 | https://zylos.ai/research/2026-02-04-graphql-modern-api-development/ | 61%+ enterprises running GraphQL in production; SSE increasingly favored over WebSockets for subscriptions; GraphQL Code Generator 614K weekly downloads; GraphQL Yoga lighter alternative to Apollo Server |
| S7 | GraphQL Federation Complete Guide 2026 (ZAX) | https://www.z-ax.com/en/blog/graphql-federation-complete-guide-2026/ | 67% of GraphQL companies adopting federation; Rust query planner: 73% P99 latency reduction; Apollo Connectors integrate REST without code; GraphQL as natural interface for AI agents; Composite Schemas spec standardizing vendor-neutral federation |
| S8 | 15 API Trends for 2026 (Alphonsolabs) | https://www.alphonsolabs.com/api-trends-2026 | REST powers 80%+ public APIs; GraphQL 50%+ enterprises; GraphQL Federation goes multi-team; Event-Driven APIs replace polling; MCP standardizes AI tool discovery; type-safe API clients replace manual HTTP calls |
| S9 | WunderGraph: GraphQL Subscriptions WebSockets vs SSE | https://wundergraph.com/blog/quirks_of_graphql_subscriptions_sse_websockets_hasura_apollo_federation_supergraph | GraphQL spec defines subscriptions but NOT transport layer; at least 5 different subscription implementations exist; SSE simpler, works over standard HTTP, benefits from HTTP/2+3 multiplexing; federated subscriptions require joining root subscription + queries from other subgraphs |
| S10 | GraphQL Federation (graphql.org, Aug 2026) | https://graphql.org/learn/federation/ | GraphQL Foundation working on Composite Schemas open specification; vendor-neutral federation standard in progress |
| S11 | Apollo GraphQL Federation | https://www.apollographql.com/docs/federation | Apollo Router + GraphOS as production federation platform; schema registry for versioning; Apollo Connectors for REST integration |

### gRPC / Connect-RPC / Buf (2026)

| # | Source | URL | Key Finding |
|---|--------|-----|-------------|
| S12 | APIScout: gRPC-Web vs REST vs Connect-RPC 2026 | https://apiscout.dev/guides/grpc-web-vs-rest-vs-connect-rpc-frontend-2026 | Connect-RPC resolves gRPC-Web browser compatibility; uses standard HTTP/1.1+HTTP/2 with JSON or binary Protobuf; no translation proxy required; Connect servers simultaneously serve gRPC + gRPC-Web + Connect HTTP; Protobuf 3-5x smaller than JSON |
| S13 | APIScout: gRPC vs Connect-RPC vs tRPC 2026 | https://apiscout.dev/guides/grpc-vs-connect-rpc-vs-trpc-2026 | tRPC v11 works with any HTTP framework; Connect-RPC supports streaming over both gRPC and Connect protocol; buf generate for codegen; curl-able Connect endpoints |
| S14 | Buf Blog: gRPC-Web Failed the Web (Sept 2026) | https://buf.build/blog/connect-a-better-grpc | gRPC-Web made browser a second-class client; Connect protocol is web-native RPC alternative |
| S15 | Buf Blog: connect-rust joins Connect (Aug 2026) | https://buf.build/blog | Anthropic built connect-rust and contributed to Connect project; Rust now has official Connect + gRPC + gRPC-Web implementation |
| S16 | Buf Blog: Protovalidate v1.0 | https://buf.build/blog | Protovalidate semantic validation for Protobuf v1.0; define validation rules once on schemas, enforced everywhere |
| S17 | Buf Blog: Protobuf-ES v2.14 | https://buf.build/blog | Protobuf-ES up to 5x faster serialization, 2x faster parsing; no API changes |
| S18 | Buf Blog: Modern Protobuf Workflow | https://buf.build/blog | Full LSP support for Protobuf; editor integration for go-to-definition, code completion, reference finding |
| S19 | gRPC-Rust Roadmap (grpc.io, June 2026) | https://grpc.io/blog/grpc-rust-roadmap/ | gRPC-Rust client-side preview released; roadmap for full gRPC implementation in Rust |
| S20 | WunderGraph: Federation + gRPC/REST via Cosmo Connect | https://wundergraph.com/graphql-federation | Cosmo Connect federates gRPC and REST services into GraphQL supergraph; Cosmo Streams for event-driven subscriptions |

---

## Defects Found

### DEFECT-413-01: OpenAPI Spec Stale at 3.0.3 — Misses 3.1/3.2 JSON Schema Alignment
**Severity**: HIGH
**Sources**: S1, S2, S5
**NeoTrix Gap**: `docs/6-REFERENCE/openapi.yaml:1` declares `openapi: "3.0.3"`. OpenAPI 3.1 (aligned with JSON Schema 2020-12) has been the standard since 2023; 3.2.0 released Sept 2025 adds structured tag navigation, streaming-friendly media types, and new OAuth flows. The 3.0.3 spec lacks: nullable type handling via `type: ["string", "null"]`, `contentEncoding`/`contentMediaType` for binary data, `examples` array on media types, and `patternProperties` for schema validation.
**Evidence**: `openapi.yaml:1` — `openapi: "3.0.3"`. No JSON Schema 2020-12 keywords used anywhere in the 355-line spec. Error schema (`openapi.yaml:23-27`) uses ad-hoc `{error, message}` instead of RFC 9457 `application/problem+json`.
**Impact**: Code generators targeting 3.1+ features produce incorrect bindings. Clients cannot rely on JSON Schema validation. Spec drift from actual implementation widens.
**Suggestion**: Upgrade to OpenAPI 3.1.0 (or 3.2.0). Align Error schema with RFC 9457 problem+json format (`type`, `title`, `status`, `detail`, `instance`). Use `type: ["string", "null"]` for nullable fields instead of `nullable: true`.

### DEFECT-413-02: Error Format Non-Compliant with RFC 9457 (Problem Details for HTTP APIs)
**Severity**: HIGH
**Sources**: S1, S3, S4
**NeoTrix Gap**: The error response format in `api.rs:25-29` returns `{"error": msg}` — a flat string. RFC 9457 (superseding RFC 7807 since July 2023) defines `application/problem+json` with five base fields: `type` (URI), `title`, `status` (HTTP code), `detail`, `instance` (URI). This is now the mainstream standard. The `openapi.yaml:23-27` Error schema also uses ad-hoc `{error, message}`.
**Evidence**: `api.rs:25-29` — `json_err` returns `{"error": msg}`. `server.rs:40-44` — not_found returns `{"error": "not_found", "message": ...}`. Neither uses `type` URI, `status`, or `instance`. No `Content-Type: application/problem+json` header.
**Impact**: Clients cannot programmatically branch on error types. No machine-readable error taxonomy. Debugging requires parsing ad-hoc strings. Breaking from industry standard makes NeoTrix harder to integrate.
**Suggestion**: Implement RFC 9457 error envelope: `json_err_with_status(status, type_uri, title, detail, instance)` returning `Content-Type: application/problem+json`. Add error type URI registry (e.g., `urn:neotrix:errors:session-not-found`).

### DEFECT-413-03: No Cursor Pagination — Only Offset-Based Take/Limit
**Severity**: MEDIUM
**Sources**: S1, S2
**NeoTrix Gap**: The session list handler (`api.rs:238-243`) returns all sessions in memory with no pagination. The knowledge search (`api.rs:169-199`) uses `.take(10)` — a hard-coded offset-style limit with no cursor token. At scale, offset pagination forces the database to scan and discard rows. Cursor pagination (keyed off a stable identifier) is the 2026 standard for any list endpoint.
**Evidence**: `api.rs:186` — `.take(10)` with no `next_cursor` in response. No `Link` header with `rel="next"`. No cursor token in request.
**Impact**: As sessions/knowledge grow, list endpoints become O(n) scans. No consistent pagination across API consumers. Breaks when data mutates between pages.
**Suggestion**: Add cursor-based pagination to all list endpoints: request param `?cursor=<opaque>&limit=20`, response includes `next_cursor` (opaque token encoding position) and `has_more`. Use `Link` header per RFC 8288.

### DEFECT-413-04: No Idempotency Key Support on Mutation Endpoints
**Severity**: MEDIUM
**Sources**: S1, S4
**NeoTrix Gap**: NeoTrix mutation endpoints (`/api/brain/absorb`, `/api/sessions/create`, `/api/sessions/{id}/fork`) have no idempotency key mechanism. Stripe's `Idempotency-Key` header pattern (V4 UUID recommended) is the 2026 standard for safe retries. Critical: Stripe's cache behavior — results cached only AFTER execution begins, so failed validation is NOT served from cache — is a common trap.
**Evidence**: No `Idempotency-Key` header handling anywhere in `api.rs` or `server.rs`. POST endpoints re-execute on retry, potentially double-absorbing knowledge or double-creating sessions.
**Impact**: Network retries cause duplicate side effects. Users see duplicate sessions, duplicate knowledge absorption. Unreliable under flaky networks.
**Suggestion**: Add `Idempotency-Key` header support to POST endpoints. Cache response keyed by `(endpoint, key)` after execution begins. Return cached result on subsequent requests with same key. TTL 24h.

### DEFECT-413-05: No API Versioning Strategy Documented or Implemented
**Severity**: MEDIUM
**Sources**: S1, S3, S5
**NeoTrix Gap**: The OpenAPI spec declares `version: "0.18.0"` but there is no versioning strategy. In 2026: header-based versioning for internal APIs (`API-Version: 2`), URL path versioning for public APIs (`/api/v2/...`), or date-based pinning (Stripe style). Deprecation headers (`Sunset`, `Deprecation`) are now required by governance teams.
**Evidence**: `openapi.yaml:4` — version is in the spec metadata only. All routes are unversioned (`/api/brain/stats`, `/api/sessions`). No `Sunset` or `Deprecation` headers in any response. No `API-Version` request header handling.
**Impact**: Breaking changes to endpoints silently break all consumers. No deprecation runway for consumers to migrate. Cannot roll out v2 alongside v1.
**Suggestion**: Adopt header-based versioning (`API-Version` request header, `X-API-Version` response header) for the internal API. Add `Sunset` header with date for deprecated endpoints. Keep URL paths unversioned for internal use.

### DEFECT-413-06: No Rate Limiting Granularity — Fixed Window Only
**Severity**: LOW
**Sources**: S2, S4
**NeoTrix Gap**: `server.rs:73-79` implements a single fixed-window rate limiter at 60 req/min. The 2026 standard is multi-granularity: per-second burst protection + per-minute sustained limits + per-day quotas. Rate limit headers (`X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`) are expected. A `429` response should include `Retry-After`.
**Evidence**: `server.rs:74` comment says "固定窗口限流" (fixed window rate limiting). No burst protection. No per-day quota. No `Retry-After` header in 429 response.
**Impact**: Burst traffic passes through to backend. No guidance to clients on when to retry. No daily cost control.
**Suggestion**: Add token-bucket or sliding-window rate limiter with burst protection. Include `X-RateLimit-*` headers on all API responses. Add `Retry-After` header on 429.

### DEFECT-413-07: No HATEOAS / Hypermedia Links for API Discoverability
**Severity**: LOW
**Sources**: S1
**NeoTrix Gap**: All NeoTrix endpoints return flat JSON bodies with no discoverable links. HATEOAS (Hypermedia as the Engine of Application State) — via HAL or JSON:API link relations — reduces client-server coupling. In 2026 it earns its complexity for workflow APIs (like NeoTrix's multi-step evolution pipeline) and state-machine APIs.
**Evidence**: `api.rs` handlers return `json_ok(...)` with no `_links` or `links` field. Session endpoints require clients to hardcode URL patterns. No `rel` relationships.
**Impact**: Clients tightly coupled to URL structure. Adding/restructuring endpoints breaks all consumers. No discoverability of available actions on a resource.
**Suggestion**: Add `_links` to list endpoints: `{ items: [...], _links: { self: "/api/sessions", next: "/api/sessions?cursor=abc" } }`. Lower priority than other defects but improves long-term API evolution.

### DEFECT-413-08: No OpenAPI Contract Testing / Spec-First Workflow
**Severity**: MEDIUM
**Sources**: S3, S5
**NeoTrix Gap**: The OpenAPI spec is included via `include_str!` at compile time (`server.rs:18`) but there is no evidence of spec-first design or contract testing. The 2026 standard is: write OpenAPI first → generate server stubs + client SDKs + tests from spec → CI/CD detects drift. Tools like Prism (mock server), Speakeasy/Stainless (SDK generation), and Spectral (linting) enforce contract correctness.
**Evidence**: `server.rs:16-18` — spec is embedded but hand-written alongside implementation. No CI step that validates spec matches code. No `spectral lint` or `prism mock` in Makefile/scripts.
**Impact**: Spec drifts from implementation silently. Generated clients use stale schemas. Integration bugs discovered late.
**Suggestion**: Add contract testing to CI: (1) Spectral lint for spec quality, (2) Prism mock server for consumer-driven testing, (3) Schemathesis for fuzz-testing endpoints against spec. Consider spec-first workflow with `openapi-generator` for server stubs.

### DEFECT-413-09: No GraphQL Endpoint — Missing Federation-Ready Interface for AI Agents
**Severity**: MEDIUM
**Sources**: S6, S7, S8, S11
**NeoTrix Gap**: NeoTrix has only REST (Axum) and WebSocket (chat). The 2026 ecosystem shows GraphQL as a natural interface for AI agents (S7: introspection enables dynamic API discovery, strong typing reduces interpretation errors). 67% of GraphQL companies are adopting federation (S7). Apollo Connectors can integrate existing REST endpoints into a GraphQL supergraph without rewriting.
**Evidence**: No `.proto` files in the repo. No GraphQL schema. No `async-graphql` or `juniper` dependency. NT-IO has REST + LSP but no flexible-query interface.
**Impact**: AI agents consuming NeoTrix APIs must use rigid REST calls. No introspection capability. Cannot federate NeoTrix endpoints into a larger agent ecosystem graph. NeoTrix remains a silo.
**Suggestion**: Add a thin GraphQL layer using Apollo Connectors pattern: wrap existing REST endpoints as GraphQL subgraph. Use `async-graphql` (Rust) for schema definition. Enable introspection for AI agent tool discovery. Low effort for high interoperability gain.

### DEFECT-413-10: No gRPC/Connect-RPC Interface for Internal Service Communication
**Severity**: MEDIUM
**Sources**: S12, S13, S15, S19
**NeoTrix Gap**: NeoTrix has no `.proto` definitions and no gRPC server. The 2026 standard for internal service-to-service communication is gRPC (HTTP/2 + Protobuf) or Connect-RPC (same `.proto`, but browser-compatible with JSON support). Anthropic contributed `connect-rust` to the Connect project (S15), giving Rust an official implementation. Protovalidate v1.0 (S16) provides schema-level validation.
**Evidence**: Zero `.proto` files in the repo. All inter-service communication (NT-ACT ↔ NT-IO, NT-MEMORY ↔ NT-CORE) is in-process Rust function calls. No wire protocol definition for when NeoTrix splits into microservices or needs cross-language interop.
**Impact**: Cannot split NeoTrix into independently deployable services. Cannot expose gRPC endpoints for high-performance clients. No schema-validated wire format. Rust-native Connect-RPC ecosystem is ready but unused.
**Suggestion**: Define `.proto` schemas for core NT domain interfaces (NT-CORE reasoning, NT-MEMORY KB queries, NT-ACT tool execution). Use `buf generate` for Rust + TypeScript bindings. Connect-RPC for browser compatibility. Protovalidate for input validation at the schema level.

### DEFECT-413-11: No Streaming Response Pattern for Long-Running Operations
**Severity**: MEDIUM
**Sources**: S1, S4, S9
**NeoTrix Gap**: Long-running operations (LLM reasoning via `/api/reason`, knowledge absorption) return only final results. The 2026 pattern is SSE (Server-Sent Events) for server→client streaming, or WebSocket for bidirectional. NeoTrix already uses SSE for agent status (`server.rs` mentions SSE) but the `/api/reason` endpoint (`api.rs:208-234`) blocks until complete with no progress events.
**Evidence**: `api.rs:208-234` — `reason_handler` calls `provider.complete(&request).await` synchronously. No streaming response. No progress events. No cancellation support. Client receives nothing until completion or timeout.
**Impact**: Clients see no progress during 10-30s LLM calls. Timeouts on slow models. No way to cancel in-flight requests.
**Suggestion**: Implement SSE streaming for `/api/reason`: return `text/event-stream` with progress events (`thinking`, `generating`, `complete`). Add `X-Request-ID` header for idempotent tracking. Support client disconnect as cancellation signal.

### DEFECT-413-12: No Machine-Readable Deprecation Protocol
**Severity**: LOW
**Sources**: S3, S5
**NeoTrix Gap**: No `Sunset` or `Deprecation` headers in any API response. The 2026 standard (RFC 8594 Sunset Header, draft-ietf-httpapi-deprecation-header) requires machine-readable deprecation notices so tooling can auto-generate migration warnings. Zalando guideline: 6-12 month deprecation window with headers.
**Evidence**: No deprecation headers anywhere in `api.rs` or `server.rs`. No sunset date tracking for any endpoint.
**Impact**: Consumers are not warned before breaking changes. No automated deprecation monitoring.
**Suggestion**: Add `Deprecation: true` and `Sunset: Sat, 01 Jan 2027 00:00:00 GMT` headers to endpoints approaching removal. Track deprecation in OpenAPI spec with `deprecated: true` and `x-sunset` extension.

---

## Summary

| Metric | Count |
|--------|-------|
| Sources cited | 20 |
| Defects identified | 12 |
| HIGH severity | 2 |
| MEDIUM severity | 7 |
| LOW severity | 3 |

### Top 3 Actionable Recommendations

1. **Upgrade to OpenAPI 3.1 + RFC 9457 error format** (DEFECT-413-01, 02) — Aligns NeoTrix with 2026 standard. Machine-readable errors (`application/problem+json`) are a cheap, high-leverage DX upgrade. JSON Schema 2020-12 enables better code generation and validation.

2. **Add cursor pagination + idempotency keys** (DEFECT-413-03, 04) — Critical for reliability at scale. Cursor pagination prevents O(n) scans; idempotency keys prevent duplicate mutations on retry. Both are battle-tested at Stripe/GitHub.

3. **Define .proto schemas and add Connect-RPC interface** (DEFECT-413-10) — Rust has an official Connect implementation (Anthropic contributed `connect-rust`). Protovalidate v1.0 provides schema-level validation. This prepares NeoTrix for service decomposition and cross-language interop without rewriting.
