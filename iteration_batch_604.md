# Iteration Batch 604 — Versioning Contracts, API Compatibility, Schema Evolution

**Date**: 2026-09-06
**Research Loop**: 604/10000+
**Prior Batch**: 603 (trust is human-side not system-side, distrust as safety default, VSA needs human-legible mirror, constellation maturity needs action reversibility, EU Article 50 mandatory disclosure)
**Topic Vector**: Semantic versioning, API compatibility, breaking changes / schema evolution

---

## Topic 1: Semantic Versioning as Machine-Enforced Contract

### Finding 1.1: SemVer Is a Promise Format, Not a Verifier
**Source**: "SemVer 2.0.0's 9 Rules Quietly Govern Every Dependency" (flowrust.com, 2026-06-12); SemVer 2.0.0 spec (semver.org)
**Defect**: SemVer does **not check compatibility** — it is a promise format, not a verifier. Nothing about `1.4.3` tells you whether the maintainer obeyed the contract. The contract is enforced socially (changelog, trust, diff review) and operationally (CI runs upgrade before ship). Package managers act on version numbers **unattended** — a caret range in `package.json` delegates upgrade decisions to the publisher's self-assessment. When the promise is broken, dependency resolution fails silently in every consumer's build pipeline at once. **Batch 603 gap**: Batch 603 identified trust as a human-side property (Finding 1.5); this extends the same principle to **version trust** — the version string is a social signal, not a machine guarantee. NeoTrix's Constellation maturity (C0-C6) is a system-internal property; it provides no machine-readable compatibility signal to external consumers. **NEW Defect**: NeoTrix lacks a machine-readable version-compatibility contract that external consumers (other modules, downstream tools) can validate independently of trusting the maintainer.

### Finding 1.2: Version Range Semantics Create Asymmetric Risk
**Source**: PkgPulse SemVer Guide (pkgpulse.com, 2026-03-29); byteledger.vizleo.com (2026)
**Defect**: Under caret semantics, `^0.2.0` only allows `0.2.x` — consumers cannot get `0.3.0` without updating their range. The 0.x version zone has **no stability guarantee**: any MINOR bump can break consumers. Tools like `semantic-release` and Changesets automate version bumps from conventional commits, but **human judgment about breaking vs non-breaking is the failure point** — commit messages can be wrong. The architectural implication: automation improves consistency but doesn't eliminate the semantic ambiguity of "what counts as breaking." **NEW Defect**: NeoTrix modules in pre-1.0 state (which most are) have no formal boundary between "this MINOR bump is safe" and "this MINOR bump breaks consumers." The C0-C6 maturity ladder encodes readiness but not **interface contract stability**.

### Finding 1.3: Automated Versioning from Spec Diff, Not Commit Log
**Source**: Fern SDK versioning playbook (buildwithfern.com, 2026-08-28)
**Defect**: For generated API SDKs, the correct version bump is a **property of the API contract diff**, not of the commit log. `fern diff` computes bump from spec comparison, exits non-zero on major — the whole CI gate in one exit code. The critical distinction: an **API breaking change** alters the wire contract (removing endpoint, adding required field, changing response type); an **SDK breaking change** alters anything a consumer imports (method names, exported types, error class hierarchies). Adding an enum value is additive over HTTP but a compile-time break in any client that deserializes into a closed enum. **NEW Defect**: NeoTrix treats all module interface changes uniformly — there is no distinction between "wire-level breaking" (KB schema change, CLI command removal) and "import-level breaking" (Rust trait signature change, type rename). This missing axis means version bumps cannot be computed automatically.

### Finding 1.4: RFC 9745/8594 — Machine-Readable Deprecation Timelines
**Source**: Fern playbook (2026-08-28); GitHub REST API versioning (docs.github.com, 2026); HubSpot date-based versioning (2026-03-30)
**Defect**: RFC 9745 defines the `Deprecation` response header (with date); RFC 8594 defines the `Sunset` header (removal date). GitHub API versions are supported for **24 months** after a newer version is released, with both headers emitted proactively. HubSpot switched to `/YYYY-MM/` date-based versioning with **18-month support windows** and twice-yearly breaking releases (March/September). The pattern: machine-readable deprecation signals are now **industry standard** — they give integrators a window that a changelog entry cannot. **NEW Defect**: NeoTrix module lifecycle (C0-C6) has no machine-readable deprecation/sunset metadata. When a module is deprecated, downstream consumers must read documentation manually; there is no `Deprecation` or `Sunset` equivalent in the KB or CLI output.

---

## Topic 2: API Versioning as Consumer-Side Concern

### Finding 2.1: API Version ≠ SDK Version — Decouple the Clocks
**Source**: Fern playbook (2026-08-28), citing Stripe's model since `2024-09-30.acacia`
**Defect**: Stripe ships **monthly dated API versions** (no breaking changes) with **named breaking releases twice a year**. Minor SDK versions track monthly API versions; major SDK versions track twice-yearly breaking releases. Consumers are pinned to a dated API version on first request and upgrade deliberately. The lesson: an **SDK major** communicates a client-side migration; an **API version** communicates a server-side one. Conflating them forces every consumer onto the faster of the two clocks. **NEW Defect**: NeoTrix conflates module maturity (C0-C6, system-side) with interface stability (consumer-side). A module reaching C4 "integrated into pipeline" doesn't tell consumers whether its interface is stable enough to depend on. The architecture needs a **dual-axis versioning** model: one axis for internal maturity, one for external contract stability.

### Finding 2.2: Go Semantic Import Versioning — Major Bump Is a Source-Level Rename
**Source**: Fern playbook (2026-08-28)
**Defect**: Go enforces semantic import versioning: from v2 onward, the module path must carry a matching `/v2` suffix (`example.com/mod` → `example.com/mod/v2`). A major bump in Go is a **source-level rename of every import in every consumer**, not a number in a manifest. Automation that treats it as a manifest edit produces a module that cannot be resolved. **NEW Defect**: NeoTrix's Rust module system doesn't have an equivalent source-level versioning enforcement. When NT-CORE v2 introduces breaking trait changes, downstream crates (nt_mind, nt_memory) must be manually updated — there's no compiler-enforced version boundary. The `#![forbid(unsafe_code)]` rule (R-P1) protects memory safety but not **API contract safety**.

### Finding 2.3: Kubernetes Deprecation Policy — The Gold Standard for Multi-Version Coexistence
**Source**: Kubernetes Deprecation Policy (kubernetes.io, 2024-10-25, still current 2026)
**Defect**: Kubernetes has **9 rules** governing API deprecation: (1) elements can only be removed by incrementing API group version, (2) objects must round-trip between API versions without info loss, (3) GA cannot be deprecated in favor of less stable, (4a) GA deprecated but not removed within a major version, beta deprecated ≤9 months or 3 minor releases, alpha removed without notice, (4b) preferred/storage version cannot advance until next release supports both, (7) deprecated behaviors function ≥1 year after announcement, (8) never deprecate in favor of less stable alternative, (9) feature gates deprecated on lifecycle transitions. **NEW Defect**: NeoTrix has **zero formal deprecation policy**. Modules can be renamed, restructured, or removed without any guaranteed notice period, compatibility window, or round-trip guarantee. The Dark Forest rule ("compile + test + connect or be deleted") is a survival axiom but not a deprecation protocol — it optimizes for system health, not consumer safety.

### Finding 2.4: OpenAI Mass Deprecation — Model Lifecycle as Versioning Crisis
**Source**: OpenAI Deprecations page (developers.openai.com, 2026-09-06 snapshot)
**Defect**: OpenAI deprecated GPT-5, o3, Assistants API, Evals platform, Agent Builder, reusable prompts, DALL-E 2/3, and Sora 2 in 2026 alone. Shutdown dates cluster in Oct-Dec 2026. The `gpt-5-2025-08-07` → `gpt-5.6-sol` migration path requires consumers to update model references, adjust reasoning mode parameters, and handle behavioral differences. The Assistants API → Responses API migration (Aug 2026) forces a complete architectural rewrite. **NEW Defect**: NeoTrix's NT-IO LLM provider layer wraps external model APIs but has **no deprecation resilience mechanism**. When OpenAI shuts down `gpt-5-2025-08-07`, every NeoTrix session using that model snapshot will fail. The architecture needs: (1) model version pinning with expiry, (2) automatic fallback chains, (3) deprecation header monitoring (RFC 9745), (4) migration path registry.

---

## Topic 3: Schema Evolution as Silent Breaking Change

### Finding 3.1: `additionalProperties: false` — The #1 Accidental Breaking Change
**Source**: "JSON Schema Migration: Versioning & Backward Compatibility" (jsonic.io, 2026-02-07)
**Defect**: `additionalProperties: false` is the most common source of accidental breaking changes in API schemas. When a producer adds a new field (additive, non-breaking), any consumer validator with `additionalProperties: false` **rejects the document** — an additive change becomes a coordinated breaking change requiring simultaneous deployment. The fix: replace with `unevaluatedProperties: false` (JSON Schema draft 2019-09) which is composition-aware. **NEW Defect**: NeoTrix KB schema (SQLite-backed, FTS5) likely uses strict schema validation on writes. If any KB write path enforces strict field validation (equivalent to `additionalProperties: false`), additive schema evolution becomes a breaking change requiring coordinated deployment of all producers. The KB schema evolution policy needs an explicit "additive changes are always safe" invariant.

### Finding 3.2: Expand-Contract Pattern — Three-Phase Safe Migration
**Source**: "How to Handle Schema Evolution in MySQL for Microservices" (oneuptime.com, 2026-03-31)
**Defect**: The expand-contract pattern splits breaking schema changes into three phases: (1) **Expand** — add new column/table, old code ignores it, new code writes both, (2) **Migrate** — backfill data from old to new structure, (3) **Contract** — remove old column/table after all instances use new code. Key: adding `NOT NULL` without a default breaks `INSERT` from old code — must add as nullable first, backfill, then constrain. **NEW Defect**: NeoTrix KB migrations (managed by nt_memory) have no formal expand-contract protocol. Schema changes to the `kv_store` `experience` namespace, `domain_nt_*` namespaces, or BM25 index could silently break existing readers. The experience-tree absorption protocol writes to KB but doesn't follow expand-contract for schema changes.

### Finding 3.3: Schema Compatibility Matrix — Formal Classification of Every Change Type
**Source**: "Schema Evolution | Telemetry" (telemetry.sh, 2026-07-30); StackPractices Message Schema Evolution Policy (2026-07-05)
**Defect**: Every proposed schema change must be classified against a compatibility matrix:
| Change | Ingestion | Query | Action |
|---|---|---|---|
| Add optional field | Compatible | Old rows return null | Add, measure adoption, update consumers |
| Rename field | Creates different field | Old consumers keep old name | Dual-write, migrate, retire |
| Change number to string | Incompatible | Calculations break | New field |
| Change units without renaming | Type matches | Results silently wrong | Add `_ms` suffix field |
| Change event grain | Rows still ingest | Counts/joins invalid | New event name or major version |
**NEW Defect**: NeoTrix KB lacks this formal compatibility matrix. When the experience-tree writes evolve (e.g., adding `cycle_pointer` field to experience entries), there is no formal classification of whether this is additive (safe), type-changing (breaking), or grain-changing (requires new namespace). The absence means schema evolution is ad-hoc and relies on developer judgment — exactly the failure mode identified in Finding 1.1.

### Finding 3.4: Dual-Write Migration — The Safe Renaming Protocol
**Source**: StackPractices (2026-07-05); Hazelcast Schema Evolution docs (docs.hazelcast.com, 2026)
**Defect**: When renaming a field, the safe protocol is: (1) producer dual-writes to both old and new field, (2) consumers migrate one at a time, (3) monitor v1 for remaining consumers, (4) stop writing old field, (5) 30-day grace period, (6) delete old topic. Hazelcast adds: **versioned maps** (`Order_v1`, `Order_v2`) for incompatible changes, with Jet pipelines for data migration. **NEW Defect**: NeoTrix KB namespace migrations (e.g., `experience` → `experience_v2`) have no dual-write protocol. The experience-tree's `close --cycle NNN` writes once to the KB; if the schema changes between sessions, old-format entries are never migrated to new format — they become **orphan data** that new readers silently skip or misparse.

### Finding 3.5: Consumer-Driven Contracts — Flip the Compatibility Model
**Source**: "Schema Evolution: The Silent Breaking Change" (automatic.co, 2026-08-24)
**Defect**: Traditional model: producers define schema, consumers adapt. The inverted model: **consumers define the parts of the schema they rely on**, then producers test against those expectations. When a producer wants to change something, the pipeline reveals instantly which consumers break. This turns compatibility from a guessing game into a measurable gate. **NEW Defect**: NeoTrix module interfaces (Rust traits in `traits.rs` files) are producer-defined. There is no consumer-side contract testing — nt_mind doesn't declare "I depend on these specific fields from nt_memory's KB interface." If nt_memory changes its KB query API, nt_mind discovers the break at compile time, not at contract-test time. The absence of consumer-driven contracts means breaking changes propagate as compile errors rather than pre-merge detection.

### Finding 3.6: GraphQL Deprecation — Track Real Usage Before Removing
**Source**: "GraphQL Schema Evolution" (networkspy.app, 2026-05-30)
**Defect**: GraphQL supports deprecation metadata, but **deprecation is a communication tool, not a cleanup button**. Mobile apps may remain active for months after a new version ships. The safe removal process: (1) mark old field deprecated, (2) publish replacement, (3) identify active operations using old field, (4) migrate owned clients, (5) notify external consumers, (6) wait for long-lived clients, (7) run breaking-change checks, (8) remove. Key gotcha: **do not remove deprecated fields based only on source-code search** — old mobile apps, scripts, and external consumers may still use them. **NEW Defect**: NeoTrix's Dark Forest rule ("compile + test + connect or be deleted") treats unused modules as candidates for deletion based on **source-level evidence** (no callers in codebase). This misses external consumers (other repos, downstream tools, user scripts) that depend on module interfaces. Deletion based on internal callers alone is a form of the "source-code search fallacy."

---

## Summary: NEW Defects vs Batch 603

| # | Defect | Domain | Severity | Batch 603 Relation |
|---|--------|--------|----------|-------------------|
| D1 | SemVer is promise format, not verifier; version strings lack machine-validatable contract | Versioning | HIGH | Extends — batch 603 Finding 1.5 (trust as human-side); this is version-trust as social signal |
| D2 | 0.x modules lack formal boundary between safe MINOR and breaking MINOR | Versioning | MEDIUM | New — constellation maturity ≠ interface stability |
| D3 | Version bumps computed from commit log, not API contract diff | Versioning | HIGH | New — automation improves consistency but not semantic accuracy |
| D4 | No machine-readable deprecation/sunset metadata (RFC 9745/8594 gap) | Versioning | HIGH | New — industry standard not implemented |
| D5 | API version conflated with SDK version — single clock forces all consumers | API Compat | HIGH | New — dual-axis versioning model needed |
| D6 | No source-level version boundary enforcement (Rust trait changes propagate unchecked) | API Compat | MEDIUM | New — compiler doesn't enforce API contract safety |
| D7 | No formal deprecation policy (Kubernetes gold standard: 1yr GA, 9mo beta, 3 releases alpha) | API Compat | HIGH | New — Dark Forest is survival axiom, not deprecation protocol |
| D8 | NT-IO LLM provider layer has no deprecation resilience (model snapshot shutdowns) | API Compat | HIGH | New — external dependency deprecation vulnerability |
| D9 | KB schema strict validation breaks additive evolution (additionalProperties: false equivalent) | Schema | HIGH | New — silent breaking change in KB writes |
| D10 | No expand-contract pattern for KB migrations (experience-tree writes once, no migration) | Schema | MEDIUM | New — orphan data from schema drift |
| D11 | No formal compatibility matrix for KB schema changes (additive/type/grain classification) | Schema | HIGH | New — ad-hoc schema evolution |
| D12 | No dual-write migration protocol for KB namespace renames | Schema | MEDIUM | New — old-format entries become orphan data |
| D13 | Producer-defined interfaces only; no consumer-driven contract testing | Schema | HIGH | New — breaking changes propagate as compile errors, not pre-merge |
| D14 | Dark Forest deletion based on source-code search misses external consumers | Schema | MEDIUM | New — source-code search fallacy |

---

## Sources Cited

1. SemVer 2.0.0 Specification — semver.org
2. "SemVer 2.0.0's 9 Rules Quietly Govern Every Dependency" — flowrust.com, 2026-06-12
3. PkgPulse Semantic Versioning Guide — pkgpulse.com, 2026-03-29
4. "Semantic versioning in 2026" — byteledger.vizleo.com, 2026
5. Fern SDK Versioning Playbook — buildwithfern.com, 2026-08-28
6. GitHub REST API Versioning — docs.github.com, 2026-03-10 version
7. HubSpot Date-Based API Versioning — developers.hubspot.com, 2026-03-30
8. OpenAI API Deprecations — developers.openai.com, 2026-09-06 snapshot
9. Kubernetes Deprecation Policy — kubernetes.io, 2024-10-25 (current 2026)
10. IBM Verify API Compatibility & Deprecation Policy — ibm.com/docs
11. OpenTelemetry Deprecating OpenTracing Compatibility — opentelemetry.io, 2026-04-23
12. IntelliJ Platform Incompatible Changes 2026 — plugins.jetbrains.com, 2026
13. "JSON Schema Migration: Versioning & Backward Compatibility" — jsonic.io, 2026-02-07
14. "Schema Evolution | Telemetry" — telemetry.sh, 2026-07-30
15. StackPractices Message Schema Evolution Policy — stackpractices.com, 2026-07-05
16. Hazelcast Schema Evolution Documentation — docs.hazelcast.com, 6.0-snapshot
17. "Schema Evolution: The Silent Breaking Change" — automatic.co, 2026-08-24
18. "GraphQL Schema Evolution" — networkspy.app, 2026-05-30
19. "How to Handle Schema Evolution in MySQL" — oneuptime.com, 2026-03-31
20. DevTools Semantic Versioning Guide — devtools.tools, 2026-07-27

---

## Cross-Batch Synthesis: Batch 603 → 604

| Batch 603 Principle | Batch 604 Extension |
|---|---|
| Trust is human-side, not system-side | Version trust is social signal, not machine guarantee (D1) |
| Distrust as safer default | Schema additive-only default: anything that could break consumers must be classified as breaking (D9, D11) |
| VSA needs human-legible mirror | KB schema needs machine-readable compatibility metadata (D4) |
| Constellation maturity needs action reversibility | Constellation maturity ≠ interface stability; need dual-axis model (D2, D5) |
| EU Article 50 mandatory disclosure | RFC 9745/8594 machine-readable deprecation as architectural requirement (D4) |
| Safety floor independent of maturity | Deprecation policy independent of module maturity — deprecated modules must still function for guaranteed window (D7) |
