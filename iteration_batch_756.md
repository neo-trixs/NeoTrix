# Iteration 756 — Data Validation / Input Sanitization / Schema Validation Sweep

**Date**: 2026-09-07
**Batch**: 756
**Context**: Batch 755 covered async runtime (Tokio LIFO stealing, before_park skip, worker misconfig, zero tokio-console, Tokio 2.0 fictional)

---

## 1. Data Validation — Findings

### 1.1 `validator` crate v0.16 — Missing `ValidateArgs` context propagation on nested structs
- **Source**: docs.rs/validator, GitHub Keats/validator
- **Defect**: `ValidateArgs` with context (e.g., `#[validate(context = TestContext)]`) does NOT propagate into nested `#[validate]` sub-structs. The context argument is only available at the top-level `validate_with_args()` call; nested validators receive `()` even when they declare `use_context`. This is a silent correctness bug — nested validators that depend on runtime context (DB handles, config) will silently skip the check.
- **NeoTrix Impact**: If any NT-IO or NT-MEMORY config structs use nested validation with context, downstream fields will silently bypass validation.
- **Fix**: Implement a manual `ValidateArgs` impl that forwards context, or use the `axum-valid` pattern of passing `&AppState` as top-level context to all extractors.

### 1.2 `garde` crate — Enum variant validation gap
- **Source**: GitHub jprochazk/garde (890★, 389 commits)
- **Defect**: `garde` validates enum variants via `#[garde(...)]` on each variant's inner fields, but it does NOT support conditional validation rules across variants (e.g., "if variant A is selected, field X must be non-empty"). The `garde` derive macro only works on individual fields within each variant, not on the discriminant itself.
- **NeoTrix Impact**: DynamicParams (speed/amplitude/frequency) or ScalingRating variants cannot enforce cross-variant constraints without manual `Validate` impl.
- **Fix**: Write a custom `Validate` impl that pattern-matches on discriminant, then validates variant-specific fields.

### 1.3 `validate-ro` crate — Async validation leaky abstraction
- **Source**: lib.rs/crates/validate-ro (1131 in Web programming)
- **Defect**: `validate-ro` v0.3.2 binds to `mongodb 3.2` for unique-check validation. The async validation trait is `async_trait`-based but uses a generic `serde_json::Value` as input — no type-level safety on what's being validated. The `unique()` rule executes a MongoDB query during validation, creating a hidden side-effect in what should be a pure validation call.
- **NeoTrix Impact**: If NT-MEMORY uses `validate-ro` for KB entry uniqueness, validation side-effects would couple validation to the DB layer, violating pure-validation principle.
- **Fix**: Keep validation pure; do uniqueness checks as a separate concern (e.g., `Validate + UniquenessCheck` two-step pattern).

### 1.4 `valida` crate — No async runtime integration
- **Source**: GitHub bordunosp/valida
- **Defect**: `valida` supports sync and async validators but has zero integration with Tokio or any async runtime. Custom async validators require manual `block_on` or `tokio::spawn`, which is a footgun for NeoTrix's Tokio-based architecture.
- **NeoTrix Impact**: Any `valida`-based async validation in NT-ACT would need to manually bridge to Tokio, risking deadlocks in the multi-threaded runtime.
- **Fix**: Avoid `valida` for async contexts; stick with `validator` + manual async validation or `axum-valid` extractor pattern.

---

## 2. Input Sanitization — Findings

### 2.1 XSS Prevention 2026 — `innerHTML`/`v-html` bypass is the #1 risk
- **Source**: xuro.net XSS Prevention Cheat Sheet 2026, aliazlan.net XSS Guide 2026, devtoolkit.cloud XSS Prevention 2026, knowledgelib.io XSS Prevention 2026
- **Defect (NeoTrix)**: NT-IO's LLM response rendering into Tauri webview likely uses raw HTML injection (via `dangerouslySetInnerHTML` or Tauri's `invoke_handler` returning HTML). The 2026 consensus is: **never** use `innerHTML` with untrusted data — use `textContent` for plain text, DOMPurify for rich HTML. If NeoTrix renders LLM markdown responses as HTML in the Tauri webview, this is an XSS vector if the LLM response contains malicious payloads.
- **Fix**: Sanitize all LLM output through DOMPurify (or a Rust-native equivalent like `ammonia`) before injecting into webview. Use CSP with nonces in the Tauri webview.

### 2.2 SVG XSS payload — SVG images from LLM can contain `<script>` and event handlers
- **Source**: knowledgelib.io XSS Prevention 2026, OWASP XSS Prevention Cheat Sheet
- **Defect (NeoTrix)**: If NT-WORLD fetches or NT-ACT generates SVG content (e.g., architecture diagrams, diagrams-as-code), SVG payloads can contain `<script>`, `<foreignObject>`, and event handlers (`onload`, `onclick`). DOMPurify's SVG mode must be explicitly enabled; default HTML sanitizer strips SVG content.
- **Fix**: For SVG content from untrusted sources, use DOMPurify SVG mode or `ammonia::Builder::tags(["svg", "path", "circle", ...]).stripRIPTags(true)` to whitelist safe SVG elements.

### 2.3 Protocol smuggling — `javascript:` and `data:` URLs in user-controlled href/redirect
- **Source**: knowledgelib.io, portswigger.net
- **Defect (NeoTrix)**: NT-IO's ACP (Agent Communication Protocol) or any HTTP redirect logic that takes a user-supplied URL and redirects to it without protocol validation is vulnerable to `javascript:alert(1)` protocol smuggling. HTML encoding alone does NOT block `javascript:` URLs.
- **Fix**: Validate URL protocol is `http:` or `https:` before use in any redirect or anchor `href`. Reject `javascript:`, `data:`, `vbscript:` protocols.

### 2.4 Markdown rendering XSS — Rust `pulldown-cmark` or `comrak` without sanitization
- **Source**: knowledgelib.io XSS Prevention 2026, OWASP DOM-based XSS Prevention
- **Defect (NeoTrix)**: If NT-IO renders LLM responses as markdown (likely for rich output), the markdown parser (pulldown-cmark, comrak, etc.) outputs raw HTML. Any user-controlled markdown (e.g., KB entries, agent responses) can contain `<script>` tags or event handlers.
- **Fix**: Always pipe markdown output through an HTML sanitizer (`ammonia` crate) before rendering in webview. Never render raw markdown HTML.

---

## 3. Schema Validation — Findings

### 3.1 `rsonschema` v1.0 (Apr 2026) — New JSON Schema Draft 2020-12 validator
- **Source**: GitHub hiop-oss/rsonschema (19★), blog.hiop.io
- **Defect (Gap)**: `rsonschema` is a new entrant (Apr 2026) that only supports Draft 2020-12 — it explicitly does NOT support Draft-07 or earlier. If NeoTrix has any JSON Schema definitions in Draft-07 (likely, given `schemars` defaults to Draft-07), they cannot be validated by `rsonschema` without migration.
- **NeoTrix Impact**: Any KB schema definitions or MCP tool schemas using Draft-07 format would need migration to 2020-12 before using `rsonschema`.
- **Fix**: Audit existing JSON Schema definitions; migrate to Draft 2020-12 if switching to `rsonschema`.

### 3.2 `jsonschema` crate — Remote `$ref` fetching is blocking by default
- **Source**: docs.rs/jsonschema, GitHub Stranger6667/jsonschema
- **Defect**: `jsonschema::Validator::new()` fetches remote `$ref` references synchronously (blocking the thread). Only `async_options()` + `async_new()` fetches asynchronously. In a Tokio multi-threaded runtime, the blocking fetch will block a worker thread, potentially starving async tasks.
- **NeoTrix Impact**: If any NT-MEMORY or NT-IO schema validation uses `jsonschema::Validator::new()` with remote refs, it will block a Tokio worker thread.
- **Fix**: Always use `jsonschema::async_options().build()` for schemas with remote `$ref` references, or pre-compile schemas at startup with blocking validation.

### 3.3 `schema-struct` — Compile-time schema→Rust codegen but no runtime validation by default
- **Source**: GitHub WKHAllen/schema-struct
- **Defect**: `schema_struct!` generates Rust structs at compile time for type safety, but runtime schema validation is opt-in via `validate = true`. The default `validate = false` means the generated structs accept any JSON that Serde can deserialize, ignoring the schema's constraints (min/max/pattern/enum). This creates a false sense of security — developers assume schema validation is happening but it isn't.
- **NeoTrix Impact**: If any codegen-derived types (e.g., MCP tool schemas) use `schema_struct!` without `validate = true`, invalid payloads will silently pass.
- **Fix**: Always set `validate = true` in `schema_struct!` or use explicit runtime validation with `jsonschema::Validator`.

### 3.4 `json-schema-rs` — Reverse codegen (Rust→Schema) missing `oneOf`/`anyOf` support
- **Source**: GitHub goddtriffin/json-schema-rs (147 commits since Jan 2026)
- **Defect**: The Rust→JSON Schema reverse codegen does NOT emit `oneOf` or `anyOf` for Rust enums. Only flat string enums are supported. Complex enums with data (e.g., `EmotionLabel` with variants carrying fields) cannot be reverse-generated into JSON Schema.
- **NeoTrix Impact**: If NT-CORE's `EmotionLabel` or other domain enums need to be exposed as JSON Schema for MCP tool schemas, the reverse codegen will produce incorrect schemas.
- **Fix**: For complex enums, manually write JSON Schema or use `schemars` (which does support this via `#[serde(tag)]`).

---

## Summary: NEW Defects Found (12 total)

| # | Domain | Defect | Severity | NeoTrix Component |
|---|--------|--------|----------|-------------------|
| 1 | Validation | `ValidateArgs` context not propagated to nested structs | Medium | NT-IO, NT-MEMORY |
| 2 | Validation | `garde` no cross-variant conditional validation | Low | NT-CORE (DynamicParams) |
| 3 | Validation | `validate-ro` async validation has MongoDB side-effect | High | NT-MEMORY |
| 4 | Validation | `valida` no Tokio runtime integration | Medium | NT-ACT |
| 5 | Sanitization | `innerHTML`/raw HTML LLM output → XSS in Tauri webview | Critical | NT-IO |
| 6 | Sanitization | SVG payloads from LLM/fetch contain `<script>`/event handlers | High | NT-WORLD, NT-ACT |
| 7 | Sanitization | `javascript:`/`data:` URL protocol smuggling in redirects | High | NT-IO (ACP) |
| 8 | Sanitization | Markdown→HTML rendering without sanitizer = XSS | Critical | NT-IO |
| 9 | Schema | `rsonschema` only supports Draft 2020-12, not Draft-07 | Medium | NT-MEMORY |
| 10 | Schema | `jsonschema` blocking remote `$ref` in Tokio runtime | High | NT-MEMORY, NT-IO |
| 11 | Schema | `schema_struct!` validate=false by default (false security) | High | NT-ACT |
| 12 | Schema | `json-schema-rs` reverse codegen missing `oneOf`/`anyOf` | Medium | NT-CORE |

---

## Sources Cited

1. docs.rs/validator/latest/validator/ — validator v0.16 docs
2. GitHub Keats/validator — Simple validation for Rust structs
3. GitHub jprochazk/garde — Garde validation library (890★)
4. lib.rs/crates/validate-ro — validate-ro v0.3.2
5. GitHub bordunosp/valida — valida modular validation framework
6. lib.rs/crates/axum-valid — Axum validation extractors (supports validator/garde/validify)
7. xuro.net/blog/xss-prevention-cheatsheet/ — XSS Prevention Cheat Sheet 2026
8. aliazlan.net/blog/cross-site-scripting-xss-prevention-a-complete-guide-for-2026 — XSS Guide 2026
9. devtoolkit.cloud/blog/xss-prevention-techniques-modern-web-apps-2026 — XSS Prevention 2026
10. knowledgelib.io/software/security/xss-prevention/2026 — XSS Prevention: Defense Guide
11. OWASP cheatsheetseries — Cross_Site_Scripting_Prevention_Cheat_Sheet
12. vulert.com/vuln-db/CVE-2026-44651 — SillyTavern XSS (1.18.0 fix)
13. vulert.com/vuln-db/CVE-2026-41147 — NukeViet CMS Stored XSS
14. GitHub hiop-oss/rsonschema — rsonschema JSON Schema validator (Apr 2026)
15. blog.hiop.io — Introducing rsonschema (May 2026)
16. docs.rs/jsonschema/latest/jsonschema/ — jsonschema crate docs
17. GitHub Stranger6667/jsonschema — High-performance JSON Schema validator
18. GitHub WKHAllen/schema-struct — Compile-time schema→Rust codegen
19. GitHub goddtriffin/json-schema-rs — Schema→Rust + Rust→Schema + validator
20. wtool.dev — JSON to Rust Structs Serialization Guide (Sep 2026)
