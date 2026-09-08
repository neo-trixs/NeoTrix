# Iteration Batch 742 — API Versioning, Deprecation & Backward Compatibility Audit

**Date**: 2026-09-07
**Research Loop**: 742/10000+
**Trigger**: Batch 741 proved Trojan Source + i18n gaps. This batch audits API lifecycle governance.

---

## 1. API Versioning — Findings & Defects

### DEFECT V1: No Versioning Strategy Documented in Codebase

**Source**: ASOasis 2026-05-10 ([asoasis.tech](https://asoasis.tech/articles/2026-05-10-0253-api-semantic-versioning-strategy/)), Docsio 2026-04-30 ([docsio.co](https://docsio.co/blog/api-versioning)), Cadence 2026-05-08 ([cadence.withremote.ai](https://cadence.withremote.ai/blog/api-versioning-2026))

**Finding**: NeoTrix exposes CLI commands (`nt` binary), LLM provider APIs (nt_io), and internal cross-module calls, but **no versioning strategy is codified anywhere**. The 2026 consensus is that every API surface — even internal — needs a declared versioning pattern (URI, header, or date-based) with automated breaking-change detection in CI.

**Industry Practice (2026)**:
- URI versioning (`/v1/...`) is the safest default for public APIs — caches cleanly, debuggable from curl
- Date-based versioning (`Stripe-Version: 2026-04-22`) is gold standard at scale (Stripe, GitHub, Shopify)
- Only 26% of teams use semantic versioning despite 60% versioning APIs (Postman 2025 State of API) — SemVer is for libraries, not HTTP contracts
- Mandatory PR classification: every spec change must declare MAJOR/MINOR/PATCH with justification

**Impact**: Without a version strategy, any change to CLI output format, LLM provider response schema, or KB query interface is an untracked breaking change that silently breaks downstream consumers (other agents, SDKs, integrations).

**Fix**: Codify a versioning policy in `CONTEXT.md` or `AGENTS.md`. For CLI: date-based. For LLM providers: URI path. Add `oasdiff` or similar tool to CI.

---

### DEFECT V2: No Semantic Contract for Cross-Module Data Shapes

**Source**: api-portal.io 2026-05-27 ([api-portal.io](https://www.api-portal.io/en/resources/articles/api-versioning-strategy)), Digital Applied 2026-06-01 ([digitalapplied.com](https://www.digitalapplied.com/blog/api-versioning-strategies-2026-engineering-decision-matrix))

**Finding**: NeoTrix modules communicate via structs (e.g., `SelfModel`, `EmotionLabel`, `SystemHealthSnapshot`), but these are **not versioned contracts**. The 2026 consensus says: "Treat the choice as an engineering decision matrix across CDN behavior, tooling support, and client migration cost." For internal Rust crates, this means:
- Each crate's public types are its contract
- Adding fields to shared structs is a minor change; removing or renaming is major
- Field-level deprecation (`#[deprecated]`) should precede removal by at least one release cycle

**Impact**: When `nt_core_self::SelfModel` gains or loses fields, downstream consumers (nt_mind, nt_meta, nt_repair) break silently at compile time with no migration path.

**Fix**: Enforce `#[deprecated(since = "X.Y", note = "use ...")]` on every struct field slated for removal. Add a CHANGELOG per crate tracking struct-level changes.

---

### DEFECT V3: No Automated Breaking-Change Detection in CI

**Source**: api-portal.io 2026-05-27, Docsio 2026-04-30, Cadence 2026-05-08

**Finding**: Postman's 2025 report found only **17% of teams run contract testing** despite 60% versioning APIs. NeoTrix runs `cargo check` and `cargo test`, but neither detects **semantic** breaking changes (field removal, type narrowing, behavioral drift). Tools like `cargo-semver-checks`, `oasdiff` (for OpenAPI), or `cargoublic-api` exist but are not integrated.

**Impact**: A struct field rename compiles cleanly but breaks any external consumer (CLI user scripts, Python bindings, Tauri IPC). No CI job catches this.

**Fix**: Integrate `cargo-semver-checks` into CI. For any public crate, enforce semver compliance on every PR.

---

## 2. Deprecation Policy — Findings & Defects

### DEFECT D1: No Deprecation Lifecycle Defined

**Source**: Zalando Restful API Guidelines ([github.com/zalando](https://github.com/zalando/restful-api-guidelines/blob/main/chapters/deprecation.adoc)), API Commons ([apicommons.org](https://apicommons.org/common/deprecation-policy/)), APIScout 2026-03-29 ([apiscout.dev](https://apiscout.dev/guides/api-changelog-versioning-communication-2026)), beefed.ai 2026-05-02 ([beefed.ai](https://beefed.ai/en/api-deprecation-process-plan-communicate))

**Finding**: NeoTrix has **zero deprecation infrastructure**. No `#[deprecated]` attributes on public items, no Sunset/Deprecation headers on LLM responses, no changelog, no migration guides. The 2026 consensus is a 4-phase lifecycle:

| Phase | Action | Duration |
|-------|--------|----------|
| Announce | Publish changelog + migration guide | Day 0 |
| Deprecate | Add Deprecation/Sunset headers, `#[deprecated]` | Day 0 |
| Monitor | Track usage of deprecated items | 6-12 months |
| Retire | Return 410 Gone, remove code | After window |

**Standards Referenced**:
- RFC 8594 (Sunset header) — now obsoleted by RFC 9745
- RFC 9745 (The Sunset HTTP Response Header Field)
- RFC 9745 §2 (Deprecation header, Standards Track, published March 2025)
- OpenAPI 3.x `deprecated: true` on operations/fields

**Impact**: When NeoTrix removes a CLI subcommand or renames a struct, consumers get a cryptic compile error or runtime failure with no guidance on what replaced the removed item.

**Fix**: Implement a deprecation protocol:
1. Add `#[deprecated]` to any item slated for removal, minimum 1 release before removal
2. For CLI: print deprecation warnings to stderr when deprecated subcommands are used
3. For LLM providers: include `Sunset` and `Deprecation` headers in responses
4. Maintain a `CHANGELOG.md` with Added/Changed/Deprecated/Removed/Fixed/Security categories

---

### DEFECT D2: No Machine-Readable Deprecation Signals

**Source**: Zalando Guidelines, APIScout 2026-03-29, beefed.ai 2026-05-02, api-platform.com ([api-platform.com](https://api-platform.com/docs/v4.2/core/deprecations/))

**Finding**: Modern APIs emit `Deprecation` and `Sunset` HTTP headers (RFC 9745/8594) on every response from deprecated endpoints. NeoTrix's web server (`nt_io`) and LLM provider responses have **no deprecation signaling mechanism**. Machine-readable deprecation enables:
- SDK tooling to warn developers at build time
- Monitoring systems to alert on deprecated traffic
- Automated migration detection

**Impact**: When an LLM provider endpoint or CLI interface is deprecated, no downstream tool or agent can detect it programmatically. Users only discover breakage at runtime.

**Fix**: 
1. Add `Sunset` header support to `nt_io` web server responses
2. For CLI: add `--deprecated` field to JSON output for deprecated features
3. For LLM providers: emit `Deprecation: <date>` and `Sunset: <date>` headers

---

### DEFECT D3: No Sunset Enforcement or Traffic Monitoring

**Source**: APIScout 2026-03-29, beefed.ai 2026-05-02, APIScout "How to Handle API Deprecation" 2026-04-03 ([apiscout.dev](https://apiscout.dev/guides/how-to-handle-api-deprecation-2026))

**Finding**: The 2026 consensus mandates monitoring deprecated endpoint traffic before sunset. Metrics:
- `deprecated_endpoint_traffic` — share of traffic hitting deprecated operations
- `time_to_sunset_days` — days remaining to announced sunset
- `migration_completion_rate` — fraction of clients migrated
- Safe to retire when: traffic < 1% of peak AND no enterprise clients active for 30 consecutive days

NeoTrix has no telemetry on deprecated feature usage, no sunset enforcement mechanism, and no traffic-based retirement decision framework.

**Impact**: Deprecated features accumulate indefinitely, increasing maintenance burden and attack surface (Zalando guideline: "Long-tail use of deprecated endpoints accumulates known vulnerabilities").

**Fix**: 
1. Add a `deprecated_usage` counter to `HeartbeatAggregator` for each deprecated item
2. Implement a sunset gate: block new onboarding to deprecated features
3. Auto-retire features when usage drops below threshold + minimum window elapsed

---

## 3. Backward Compatibility — Findings & Defects

### DEFECT B1: No Backward Compatibility Test Suite

**Source**: Endurance Softwares 2026 ([endurancesoftwares.com](https://www.endurancesoftwares.com/blog/nodejs-api-versioning-backward-compatible-design-2026)), Gruv 2026-04-05 ([gruv.ai](https://gruv.ai/blog/version-payment-api-strategies-backward-compatible-changes)), PkgPulse 2026-03-29 ([pkgpulse.com](https://www.pkgpulse.com/guides/semantic-versioning-guide-breaking-changes-2026))

**Finding**: The 2026 consensus is explicit: "Run unchanged legacy client fixtures against the proposed release. If the same calls and parsing logic still work with compatible behavior, staying on the current version is reasonable." NeoTrix runs `cargo test` but has no:
- Golden file/snapshot tests for CLI output format
- Consumer-driven contract tests for cross-module interfaces
- Backward-compatibility replay of known client requests
- Version-tagged test fixtures

**Impact**: A change to CLI output JSON structure silently breaks downstream scripts. A change to KB query response schema breaks all callers. No test catches this.

**Fix**:
1. Add snapshot tests for CLI output (`insta` crate)
2. Maintain versioned test fixtures for each public API surface
3. Add consumer-driven contract tests using `cargo-test` + golden files

---

### DEFECT B2: No Behavioral Contract Documentation

**Source**: Endurance Softwares 2026, medium.com/@serifcolakel 2026-07-14 ([medium.com](https://medium.com/@serifcolakel/backward-compatibility-a-practitioners-guide-to-evolving-apis-without-breaking-clients-49cd19225cde))

**Finding**: Backward compatibility extends beyond schema to **behavioral contracts**: status codes, default pagination, error message formats, ordering semantics, rate limits. The .NET team's rules (2026-04-10 updated) explicitly state: "With a sufficient number of users of an API, any observable behavior will be depended on by somebody." NeoTrix does not document which behaviors are contractual vs. implementation details.

**Impact**: Changing default sort order of KB query results, changing error message text, or changing CLI exit codes are all breaking changes that no process catches.

**Fix**: 
1. Document behavioral contracts per public interface (exit codes, output format, error messages)
2. Mark unstable/implementation-detail behaviors explicitly
3. Add regression tests for behavioral contracts

---

### DEFECT B3: No Dual-Write/Dual-Read Migration Pattern

**Source**: Cadence 2026-05-08, medium.com/@serifcolakel 2026-07-14

**Finding**: When breaking changes are unavoidable, the 2026 consensus is dual-write/dual-read: support both old and new shapes simultaneously during migration, route traffic gradually, verify parity, then retire old shape. NeoTrix has no such pattern — changes are atomic and immediately breaking.

**Impact**: Any schema migration (e.g., KB embedding format change, SelfModel field restructuring) forces an all-or-nothing upgrade with no gradual rollout path.

**Fix**:
1. Implement versioned response serializers that can emit both old and new shapes
2. Add feature flags for schema migrations (controlled via config or CLI args)
3. Route a percentage of traffic to new shape, monitor, then complete migration

---

### DEFECT B4: SDK/CLI Version Lockstep Not Enforced

**Source**: Cadence 2026-05-08, ASOasis 2026-05-10

**Finding**: NeoTrix ships a CLI binary and has Python/JS SDK consumers. The 2026 consensus: "Ship typed SDKs from your OpenAPI spec, version the SDK alongside the API, and document the API version each SDK release targets." NeoTrix has no mechanism to ensure CLI and SDK versions are compatible, no `Accept` header versioning for SDKs, and no feature gate for preview features.

**Impact**: A Python SDK user installs CLI v2.0 but SDK v1.x, gets silent incompatibilities.

**Fix**:
1. Add version negotiation: SDK sends `Accept: application/vnd.neotrix.v2+json`
2. Server returns capability discovery response with supported features
3. Version SDK and CLI in lockstep; document compatibility matrix

---

## Summary Table

| ID | Category | Defect | Severity | Effort |
|----|----------|--------|----------|--------|
| V1 | Versioning | No versioning strategy documented | HIGH | Medium |
| V2 | Versioning | No semantic contract for cross-module data shapes | MEDIUM | Medium |
| V3 | Versioning | No automated breaking-change detection in CI | HIGH | Low |
| D1 | Deprecation | No deprecation lifecycle defined | HIGH | Medium |
| D2 | Deprecation | No machine-readable deprecation signals | MEDIUM | Low |
| D3 | Deprecation | No sunset enforcement or traffic monitoring | MEDIUM | High |
| B1 | Compat | No backward compatibility test suite | HIGH | Medium |
| B2 | Compat | No behavioral contract documentation | MEDIUM | Low |
| B3 | Compat | No dual-write/dual-read migration pattern | HIGH | High |
| B4 | Compat | SDK/CLI version lockstep not enforced | MEDIUM | Medium |

---

## Sources Cited

1. ASOasis — "A Practical API Semantic Versioning Strategy" (2026-05-10) — https://asoasis.tech/articles/2026-05-10-0253-api-semantic-versioning-strategy/
2. api-portal.io — "API Versioning Strategy: SemVer for REST and GraphQL" (2026-05-27) — https://www.api-portal.io/en/resources/articles/api-versioning-strategy
3. Docsio — "API Versioning: The Complete 2026 Guide" (2026-04-30) — https://docsio.co/blog/api-versioning
4. Cadence — "How to do API versioning correctly in 2026" (2026-05-08) — https://cadence.withremote.ai/blog/api-versioning-2026
5. Digital Applied — "API Versioning Strategies: 2026 Engineering Matrix" (2026-06-01) — https://www.digitalapplied.com/blog/api-versioning-strategies-2026-engineering-decision-matrix
6. Zalando — "Restful API Guidelines: Deprecation" — https://github.com/zalando/restful-api-guidelines/blob/main/chapters/deprecation.adoc
7. API Commons — "Deprecation Policy" — https://apicommons.org/common/deprecation-policy/
8. APIScout — "API Changelog & Versioning Communication 2026" (2026-03-29) — https://apiscout.dev/guides/api-changelog-versioning-communication-2026
9. beefed.ai — "API Deprecation Process: Plan & Communicate" (2026-05-02) — https://beefed.ai/en/api-deprecation-process-plan-communicate
10. api-platform.com — "Deprecating Resources and Properties" — https://api-platform.com/docs/v4.2/core/deprecations/
11. APIScout — "How to Handle API Deprecation Notices 2026" (2026-04-03) — https://apiscout.dev/guides/how-to-handle-api-deprecation-2026
12. APICourse — "API Deprecation: How to Retire Endpoints Without Burning Integrators" (2026-05-02) — https://apicourse.com/api-deprecation-how-to-retire-endpoints-without-burning-integrators/
13. Endurance Softwares — "Node.js API Versioning: Backward-Compatible Design Guide" (2026) — https://www.endurancesoftwares.com/blog/nodejs-api-versioning-backward-compatible-design-2026
14. Gruv — "How to Version Your Payment API" (2026-04-05) — https://gruv.ai/blog/version-payment-api-strategies-backward-compatible-changes
15. PkgPulse — "Semantic Versioning: Breaking Changes Guide 2026" (2026-03-29) — https://www.pkgpulse.com/guides/semantic-versioning-guide-breaking-changes-2026
16. .NET Docs — "Library change rules for compatibility" (2026-04-10) — https://github.com/dotnet/docs/blob/main/docs/core/compatibility/library-change-rules.md
17. medium.com/@serifcolakel — "Backward Compatibility: A Practitioner's Guide" (2026-07-14) — https://medium.com/@serifcolakel/backward-compatibility-a-practitioners-guide-to-evolving-apis-without-breaking-clients-49cd19225cde
18. DEV Community — "API Versioning Strategy Playbook" (2026-05-24) — https://dev.to/beefedai/api-versioning-strategy-playbook-4214

---

**Research Loop Continuation**: Batch 743 should audit: (1) observability/metrics standards, (2) error handling consistency, (3) configuration management patterns.
