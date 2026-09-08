# Iteration Batch 706 — REST API / OpenAPI / API Versioning Research

**Date**: 2026-09-06  
**Domain**: REST API Design, OpenAPI Specification, API Versioning & Deprecation  
**Previous Batch**: 705 (Knowledge Representation / Semantic Web / Knowledge Graphs — 10 defects: no OWL reasoning, no inference, no FAIR modularization, no proof tracing, no triple terms)  

---

## Sources Consulted

### REST API Design
1. **Microsoft Azure Architecture Center** — Best practices for RESTful web API design (2025-05-08)
2. **Cadence / WithRemote** — How to design RESTful API endpoints in 2026 (2026-05-05)
3. **Moesif** — 12 REST API Best Practices That Hold Up in 2026 (2026-05-25)
4. **DevGENT** — Web API Design Standards 2026: Errors, Auth, Versioning (2026-08-11)
5. **Keploy** — API Design Principles, Best Practices & Patterns (2026-05-11)
6. **EncodeDots** — 15 API Design Principles for Scalable Software in 2026 (2026-07-30)
7. **Digital Applied** — REST API Design in 2026: A Full Engineering Reference (2026-06-13)

### OpenAPI Specification
8. **OpenAPI Initiative** — OpenAPI Specification v3.2.0 (2025-09-19, latest published)
9. **Swagger.io** — OpenAPI Specification v3.2.0 canonical rendering
10. **GitHub OAI/OpenAPI-Specification** — OAS 3.2.0 release notes

### API Versioning & Deprecation
11. **Cadence / WithRemote** — How to do API versioning correctly in 2026 (2026-05-08)
12. **AskanTech** — API Versioning Strategies 2026: URL, Header, Date-Based (2026-04-16)
13. **Docsio** — API Versioning: The Complete 2026 Guide (2026-04-30)
14. **FintekCafe** — API Versioning and Deprecation: How Platforms Change Without Breaking (2026-08-12)
15. **Google Cloud** — Versioning | Cloud APIs (AIP-185 / AIP-181)
16. **ASOasis** — A Practical API Semantic Versioning Strategy (2026-05-10)
17. **Pratik Dhanave** — Versioning and Evolution: Changing an API Without Breaking (2026-08-11)

---

## NEW Defects Found in NeoTrix

### DEFECT-706-01: No RFC 9457 problem+json Error Standardization
**Severity**: HIGH  
**Domain**: NT-IO / NT-SHIELD  
**Finding**: RFC 9457 `application/problem+json` is the 2026 standard for HTTP API errors, superseding RFC 7807 since 2023. Top APIs (Stripe, Linear, Cloudflare) return errors with `type`, `title`, `status`, `detail`, and `instance`. NeoTrix's error responses use ad-hoc `{"error": "not_found", "message": "Endpoint not found"}` (server.rs:40-44) and `{"error": "unauthorized", "message": "Invalid or missing API token"}` (server.rs:63-66). No `type` URI, no `instance` ID, no `status` code in body.  
**Impact**: Monitoring tools, retry libraries, and proxy caches all key off HTTP status codes + error body structure. A 200 with `{"error": true}` breaks every one of them (Moesif 2026). NeoTrix's ad-hoc format forces every consumer to write custom error parsers. The 422-vs-400 split is expected in 2026 (400 = malformed JSON, 422 = semantically invalid), but NeoTrix uses 400 uniformly.  
**Fix**: Replace all error responses with RFC 9457 problem+json: `{"type": "https://docs.neotrix.dev/errors/not-found", "title": "Not Found", "status": 404, "detail": "Node abc123 not found", "instance": "/api/kb/nodes/abc123"}`. Add a middleware layer that formats all `axum::response::Json` errors into problem+json. Implement 422 for semantic validation errors.

### DEFECT-706-02: No Idempotency-Key Support on Mutating Endpoints
**Severity**: HIGH  
**Domain**: NT-IO / NT-ACT  
**Finding**: Idempotency keys are table stakes in 2026 (Moesif, Digital Applied, DevGENT). The IETF httpapi working group is standardizing the `Idempotency-Key` header (Internet-Draft). Stripe popularized the pattern: client sends `Idempotency-Key: <V4-UUID>` on POST, server caches `(key, request_hash, response_body, status)` for 24h, subsequent calls return the original response. NeoTrix has no idempotency support on any endpoint.  
**Impact**: Agents retry aggressively (Moesif 2026). Without idempotency, retries to `POST /api/kb/nodes` create duplicate nodes. Retries to `POST /api/act/run` create duplicate tool executions. This is the exact failure mode that caused Stripe to adopt idempotency keys — duplicate charges. NeoTrix's MCP-mediated agent consumption pattern makes this critical.  
**Fix**: Accept `Idempotency-Key` header on every mutating POST/PATCH. Store `(key, request_hash, response, status)` in KB with 24h TTL. On cache hit, return the stored response. Skip idempotency on GET/DELETE (already idempotent). Use V4 UUID format, 255-char cap.

### DEFECT-706-03: No Deprecation/Sunset Header Protocol
**Severity**: HIGH  
**Domain**: NT-IO  
**Finding**: RFC 8594 `Sunset` header and RFC 9745 `Deprecation` header are the 2026 machine-readable standards for API retirement. GitHub returns `Deprecation` as a Structured Fields Date (`@1688169599`), `Sunset` as the hard cutoff date, and `Link: <migration-doc>; rel="deprecation"`. Shopify ships `X-Shopify-API-Deprecated-Reason` on deprecated fields. NeoTrix has no deprecation signaling — features are removed without in-band notice.  
**Impact**: When NeoTrix deprecates an API endpoint, consumers (including LLM agents) have no machine-readable way to detect it. Deprecation-by-blog-post is insufficient (DevGENT 2026). Agents hard-code endpoint shapes more than humans do (Cadence 2026), so silent deprecation causes "the AI wrote it and now it broke" tickets. Without `Sunset` headers, consumers cannot programmatically alert on upcoming removals.  
**Fix**: Add `Deprecation: @<unix-timestamp>` and `Sunset: <RFC-1123-date>` headers to every deprecated endpoint response. Add `Link: <https://docs.neotrix.dev/migration>; rel="deprecation"` pointing to migration guide. Maintain a deprecation registry in the KB with per-endpoint sunset dates.

### DEFECT-706-04: No Cursor-Based Pagination
**Severity**: HIGH  
**Domain**: NT-IO / NT-MEMORY  
**Finding**: Offset pagination collapses at scale — `OFFSET 100000` forces the database to scan and discard 100k rows on every request (Digital Applied 2026). Cursor pagination keys off a stable identifier, with opaque base64 tokens. The 2026 default is cursor with `limit=25, max=100`, `has_more` boolean (not `total`), and base64-encoded cursors containing sort key + tiebreaker (Cadence 2026). Switching from offset to cursor later is a breaking change unless both shapes are wrapped from day one.  
**Impact**: NeoTrix's KB list endpoints (nodes, edges, experiences) use offset pagination. As the KB grows beyond 10k records, each page query degrades linearly. The `has_more` vs `total` distinction matters — computing `total` on large tables requires a count query that doubles I/O. The cost on day one is low; the cost on day 400 is a v2 (Cadence 2026).  
**Fix**: Implement cursor-based pagination on all list endpoints from day one. Use opaque base64 cursors containing `(sort_key, tiebreaker_id)`. Return `{data: [...], cursor: "base64...", has_more: true}`. Keep offset pagination as a non-default option for admin tables under 10k rows.

### DEFECT-706-05: No OpenAPI 3.2 Contract as SSOT
**Severity**: HIGH  
**Domain**: NT-IO  
**Finding**: OAS 3.2.0 (released 2025-09-19) is the current published standard. It aligns with JSON Schema draft 2020-12, so the spec is also the validation rule, codegen source, and AI tool definition. FastAPI emits 3.1 by default; Hono ships `@hono/zod-openapi`. In 2026, there is "no excuse for a hand-written Postman collection" (Cadence 2026). NeoTrix's OpenAPI spec is version 3.0 (server.rs:16), not 3.1 or 3.2.  
**Impact**: OAS 3.0 lacks JSON Schema 2020-12 alignment, nullable/const support, and webhook definitions. NeoTrix's spec cannot be used as a validation rule or codegen source without manual translation. Agent-readable OpenAPI fields (`operationId`, `summary`, `description`) are treated as user-facing copy that agents read literally when deciding which endpoint to call (Moesif 2026) — NeoTrix's spec has vague descriptions.  
**Fix**: Upgrade to OAS 3.2.0. Use `operationId` as the agent-facing tool name. Add `summary` and `description` fields to every endpoint. Generate the spec from code (e.g., `utoipa` for Axum) to keep it in sync. Make the spec the single source of truth for validation, codegen, and agent tool definitions.

### DEFECT-706-06: No URI Versioning Strategy
**Severity**: HIGH  
**Domain**: NT-IO  
**Finding**: URI versioning (`/v1/orders`) is the 2026 consensus for most teams — debuggable from a single curl line, every CDN caches it correctly, every client supports it without custom config (Cadence 2026, Moesif 2026, DevGENT 2026). Header versioning "sounds clean and creates support tickets" (Cadence 2026). Date-based versioning (Stripe model) is the gold standard at scale but carries real engineering tax. NeoTrix has no version prefix on any endpoint.  
**Impact**: Any breaking change to NeoTrix's API (renaming a field, changing a response shape, altering validation rules) silently breaks all consumers simultaneously. There is no migration path — v1 and v2 cannot coexist. For a system consumed by LLM agents that hard-code endpoint shapes, this is catastrophic. LLM-generated client code hard-codes endpoint shapes more than human-written code (Cadence 2026).  
**Fix**: Add `/v1/` prefix to all API routes. Plan for additive-only changes within v1. When v2 is needed, mount a new router under `/v2/` alongside v1. Document the versioning strategy publicly. Consider date-based versioning only if NeoTrix becomes a high-volume public API.

### DEFECT-706-07: No X-Request-Id Distributed Tracing
**Severity**: MEDIUM  
**Domain**: NT-IO / NT-SHIELD  
**Finding**: Every 2026 API reference (Moesif, Keploy, EncodeDots) recommends surfacing request IDs in every response via `X-Request-Id` header. When a customer reports an issue, the request ID is what the support team uses to find the call in logs. Without it, every support ticket starts with "can you reproduce it?" (Moesif 2026). NeoTrix has no request ID propagation.  
**Impact**: When an agent or user reports a failed API call, there is no way to trace it through NeoTrix's internal subsystems (NT-IO → NT-ACT → NT-MEMORY). Debugging requires correlating timestamps across logs, which is unreliable in concurrent systems.  
**Fix**: Generate a UUID v4 request ID on every inbound request (or accept the client's `X-Request-Id` for trace continuity). Propagate it through all internal calls. Include it in every response header and in problem+json `instance` field.

### DEFECT-706-08: No Agent-Readiness Layer
**Severity**: HIGH  
**Domain**: NT-IO / NT-ACT  
**Finding**: The 2026 addition to REST best practices (Moesif 2026) is agent-readiness: (1) idempotency keys on writes, (2) agent-readable OpenAPI fields, (3) MCP exposure for agent consumers. WSO2 AI Gateway auto-generates MCP servers from OpenAPI specs. NeoTrix's MCP gateway (`nt_io_mcp`) exists but the REST surface has no agent-specific considerations.  
**Impact**: LLM agents consuming NeoTrix's REST API will: (a) retry without idempotency → duplicate operations, (b) misselect endpoints due to vague OpenAPI descriptions, (c) not receive MCP-formatted tool definitions. The gap between "REST API" and "agent-consumable API" is exactly where the 2026 industry is converging — NeoTrix is not there.  
**Fix**: Add agent-specific headers: `X-Agent-Id`, `X-Agent-Session` for agent call tracking. Ensure OpenAPI `operationId` matches MCP tool names. Add `Idempotency-Key` support (DEFECT-706-02). Document retry semantics: `4xx` (except `408`, `429`) = not retryable; `429` and `5xx` = retryable with exponential backoff.

### DEFECT-706-09: No HATEOAS / Hypermedia Links
**Severity**: MEDIUM  
**Domain**: NT-IO  
**Finding**: Microsoft Azure (2025) and the Richardson Maturity Model still recommend HATEOAS — each GET response includes hypermedia links to related resources, making the API navigable without prior URI knowledge. In practice, most 2026 APIs (Stripe, GitHub, Linear) skip full HATEOAS but use `Link` headers for pagination and `Rel` links for related resources. NeoTrix returns flat JSON with no links.  
**Impact**: API consumers must hard-code all URI paths from documentation. When resource relationships change (e.g., `/api/kb/nodes/{id}/edges` moves to `/api/kb/edges?node_id={id}`), every consumer breaks. Hypermedia links decouple consumers from URI structure.  
**Fix**: At minimum, add `Link` headers for pagination (`<url?cursor=abc>; rel="next"`). For resource responses, add a `_links` object with `self`, `related`, and `collection` URIs. Full HATEOAS is optional but `Link` headers for pagination are table stakes.

### DEFECT-706-10: No CORS Configuration for Browser Consumers
**Severity**: MEDIUM  
**Domain**: NT-IO  
**Finding**: Moesif 2026: "If any of your consumers will call the API from a browser, the Access-Control-Allow-Origin configuration is part of the API contract, not an afterthought." NeoTrix's web server has no CORS middleware visible in server.rs.  
**Impact**: Browser-based tools (docs UIs, testing dashboards, agent web interfaces) cannot call NeoTrix's API from a different origin. Preflight requests fail or are not handled. This blocks integration with web-based agent runtimes.  
**Fix**: Add CORS middleware with explicit allowlist, `Access-Control-Allow-Methods`, `Access-Control-Allow-Headers`, and `Access-Control-Max-Age` for preflight caching. Make the allowlist configurable via environment variable.

---

## Summary

| Metric | Count |
|--------|-------|
| Sources consulted | 17 |
| NEW defects found | 10 |
| HIGH severity | 6 (DEFECT-706-01, 706-02, 706-03, 706-04, 706-05, 706-06, 706-08) |
| MEDIUM severity | 4 (DEFECT-706-07, 706-09, 706-10) |

### What's NEW vs Previous Batches
- **RFC 9457 problem+json**: The error standard since 2023, now universally adopted — NeoTrix still uses ad-hoc error JSON
- **Idempotency-Key**: Table stakes in 2026, IETF standardization in progress — NeoTrix has zero idempotency support
- **RFC 8594/9745 Deprecation/Sunset**: Machine-readable retirement signals — NeoTrix deprecates silently
- **Cursor pagination**: Offset pagination is a legacy pattern that collapses at scale — NeoTrix uses offset
- **OAS 3.2.0**: Released Sep 2025, the de facto contract SSOT — NeoTrix is on OAS 3.0
- **URI versioning consensus**: `/v1/` is the 2026 default for debuggability — NeoTrix has no version prefix
- **Agent-readiness**: New 2026 dimension for AI-consumed APIs — NeoTrix has no agent-specific API considerations
- **X-Request-Id tracing**: Universal in 2026 for debugging — NeoTrix has no request ID propagation
- **HATEOAS-lite (Link headers)**: Pagination links are table stakes — NeoTrix returns flat JSON
- **CORS**: Browser consumers need explicit CORS config — NeoTrix has no CORS middleware

### Cross-Iteration Insight
Batch 705 revealed a **knowledge-level defect cluster** (no inference, no modularization, no proof tracing). Batch 706 reveals a complementary **interface-level defect cluster**: NeoTrix's API surface violates every 2026 convention for machine-readable errors, idempotency, versioning, pagination, and deprecation signaling. The two clusters are coupled: Batch 705's "no proof tracing" defect means NeoTrix cannot verify its own reasoning chains, while Batch 706's "no problem+json" defect means it cannot communicate errors to external consumers. The highest-priority fixes are DEFECT-706-01 (problem+json) and DEFECT-706-02 (idempotency), as they are prerequisites for any public API consumption. The cross-iteration pattern across 705→706 is: NeoTrix has **strong internal architecture** (6-layer consciousness, SEAL pipeline, VSA HyperCube) but **weak external interface contracts** — the system can think but cannot speak the language its consumers expect.

### Cumulative Defect Count (Batches 700–706)
| Batch | Domain | Defects |
|-------|--------|---------|
| 700 | AoS/SoA, cache locality | 8 |
| 701 | SIMD, vectorization | 7 |
| 702 | PGO/LTO, build optimization | 9 |
| 703 | False sharing, concurrency | 10 |
| 704 | Matrix SDK, ecosystem | 8 |
| 705 | Knowledge Representation | 10 |
| 706 | REST API / OpenAPI / Versioning | 10 |
| **Total** | | **62** |
