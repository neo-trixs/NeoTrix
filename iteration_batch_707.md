# Iteration Batch 707 — Documentation & DX Defect Analysis

**Date:** 2026-09-06
**Source:** Batch 706 (API hygiene) → 707 (Documentation, DX, API Doc generation)
**Method:** Web research across 3 domains, 24+ sources

---

## 1. Documentation Defects

### D-707-1: No `#![deny(missing_docs)]` on library crates
**Source:** [Rustdoc book](https://doc.rust-lang.org/stable/rustdoc/how-to-write-documentation.html), [Docsio guide](https://docsio.co/blog/rust-documentation)
**Defect:** neotrix-core and sub-crates lack `#![deny(missing_docs)]` at crate root. Public items can ship undocumented without CI failure.
**Impact:** API surface drift — types/functions added without docs accumulate silently.
**Fix:** Add `#![deny(missing_docs)]` to `lib.rs` of each library crate; `#![warn(missing_docs)]` on binary crates.

### D-707-2: No `#![deny(rustdoc::broken_intra_doc_links)]`
**Source:** [Rustdoc book](https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html), [Pragmatic Rust Guidelines (Microsoft)](https://microsoft.github.io/rust-guidelines/guidelines/docs/)
**Defect:** Intra-doc links (e.g., `[SelfModel](nt_core_self::SelfModel)`) can silently rot when types are renamed/moved.
**Impact:** docs.rs output contains dead links that confuse users.
**Fix:** Add `#![deny(rustdoc::broken_intra_doc_links)]` to crate roots. Enables build-time link resolution validation.

### D-707-3: No `#![doc = include_str!("../README.md")]` pattern
**Source:** [Docsio guide](https://docsio.co/blog/rust-documentation)
**Defect:** Crate-level docs in `lib.rs` and `README.md` are likely duplicated (or one is stale). No single-source-of-truth mechanism.
**Impact:** README promises features that doc comments don't cover, or vice versa.
**Fix:** Use `#![doc = include_str!("../README.md")]` in `lib.rs` crate-level docs.

### D-707-4: No `[package.metadata.docs.rs]` section in Cargo.toml
**Source:** [Calmops Rust docs guide](https://calmops.com/programming/rust/documentation-and-api-documentation-in-rust/), [Docsio guide](https://docsio.co/blog/rust-documentation)
**Defect:** Feature-gated items (e.g., `#[cfg(feature = "full")]`) are invisible on docs.rs by default.
**Impact:** Users on docs.rs see incomplete API surface; feature-gated types/functions are undiscoverable.
**Fix:** Add `[package.metadata.docs.rs]` with `all-features = true` and `rustdoc-args = ["--cfg", "docsrs"]`.

### D-707-5: No module-level `//!` documentation
**Source:** [Pragmatic Rust Guidelines (Microsoft)](https://microsoft.github.io/rust-guidelines/guidelines/docs/), [Rust FAQ](https://www.rustfaq.org/en/how-to-write-good-documentation-for-rust-crates/)
**Defect:** Domain modules (`nt_core`, `nt_mind`, `nt_world`, etc.) lack `//!` module-level doc blocks explaining purpose, contents, and when to use each type.
**Impact:** New contributors land on module pages from search and have no overview before picking a type.
**Fix:** Add `//!` blocks to each `mod.rs` / `lib.rs` covering: what the module contains, when to use it, examples, and cross-references.

### D-707-6: Doc comments use `# Arguments` / `# Returns` sections
**Source:** [Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/guidelines/docs/), [Rustdoc book](https://doc.rust-lang.org/stable/rustdoc/how-to-write-documentation.html)
**Defect:** If any public functions use `# Arguments` or `# Returns` sections, these are redundant — names and types already convey this. Community convention skips them.
**Impact:** Verbose docs that slow readers. Rustdoc auto-links types in signatures, making argument tables redundant.
**Fix:** Audit existing docs; replace `# Arguments` / `# Returns` with prose that explains intent and constraints instead.

---

## 2. Developer Experience Defects

### D-707-7: No time-to-first-build (TTFB) metric
**Source:** [Cycloid onboarding guide](https://www.cycloid.io/blogs/developer-onboarding-process/), [Skene onboarding guide](https://www.skene.ai/resources/blog/developer-onboarding-guide)
**Defect:** No measurement of how long a new contributor takes from `git clone` to first successful `cargo build`. Industry best practice: under 5 minutes.
**Impact:** Untracked friction silently repels contributors. If TTFB > 30 min, evaluators close the tab (Skene: "developers who succeed in first session are 3-5x more likely to convert").
**Fix:** Add a CI job that measures `cargo build` time on clean checkout. Document TTFB target in CONTRIBUTING.md.

### D-707-8: No devcontainer / reproducible environment
**Source:** [Engineering Onboarding Guide 2026](https://arcdev.in/engineering-onboarding-programs-get-the-complete-guide-for-2026/), [New Stack](https://thenewstack.io/tackling-developer-onboarding-complexity/)
**Defect:** No `.devcontainer/devcontainer.json` or equivalent. New contributors must manually install Rust toolchain, wasm-pack, cargo-watch, etc.
**Impact:** "It works on my machine" friction. Environment drift between contributors. Onboarding takes days instead of minutes.
**Fix:** Create `.devcontainer/devcontainer.json` with Rust base image, pre-installed tools, and VS Code extensions.

### D-707-9: No CONTRIBUTING.md or contribution quickstart
**Source:** [Kompassify onboarding guide](https://kompassify.com/blog/developer-onboarding-guide)
**Defect:** No single entry point that tells a contributor: how to set up, how to run tests, how to submit a PR, code style rules.
**Impact:** Each new contributor must reverse-engineer the workflow from CI configs and existing PRs.
**Fix:** Create `CONTRIBUTING.md` covering: prerequisites, build, test, lint, PR template, code style, and architecture overview link.

### D-707-10: No "golden path" for common tasks
**Source:** [New Stack](https://thenewstack.io/tackling-developer-onboarding-complexity/), [SensioLabs DX 2026](https://sensiolabs.com/blog/2026/the-developer-experience-revolution-2026)
**Defect:** No documented "paved path" for adding a new domain module, new CLI command, or new SelfTest. Contributors must study existing patterns ad-hoc.
**Impact:** Inconsistent module structure across contributors. Knowledge concentrated in a few maintainers.
**Fix:** Add `docs/golden-paths/` with templates for: new domain module, new CLI subcommand, new SelfTest, new SEAL stage.

### D-707-11: No `cargo doc --open` verification in CI
**Source:** [Rustdoc book](https://doc.rust-lang.org/stable/rustdoc/how-to-write-documentation.html)
**Defect:** CI runs `cargo check` and `cargo test` but never builds docs. Doc build errors (broken links, missing items) reach docs.rs unchecked.
**Impact:** Published docs contain errors invisible to maintainers.
**Fix:** Add `cargo doc --no-deps` to CI pipeline; fail on warnings (which include broken intra-doc links with deny enabled).

---

## 3. API Documentation Defects

### D-707-12: No OpenAPI spec generation from source
**Source:** [OOPS paper (arxiv 2601.12735)](https://arxiv.org/pdf/2601.12735), [DaloyJS OpenAPI](https://daloyjs.dev/docs/openapi), [zod-openapi](https://github.com/StructIQ/zod-openapi)
**Defect:** NeoTrix CLI/server endpoints have no machine-readable API description (OpenAPI/Swagger spec). API docs are manually maintained or absent.
**Impact:** Cannot auto-generate client SDKs, contract tests, or interactive API playgrounds. API drift between code and docs.
**Fix:** Derive OpenAPI spec from source using `utoipa` (Rust) or equivalent. Add `cargo doc --openapi` to build pipeline.

### D-707-13: No RFC 9457 Problem Details on error endpoints
**Source:** Batch 706 finding (confirmed), [DaloyJS OpenAPI](https://daloyjs.dev/docs/openapi) — ships reusable `components.schemas.Problem`
**Defect:** Error responses are ad-hoc JSON objects, not RFC 9457 `application/problem+json`. (Carried from 706, now with API doc angle.)
**Impact:** Clients cannot parse errors programmatically; API docs cannot describe a consistent error schema.
**Fix:** Define a shared `Problem` schema in OpenAPI components; implement `#[derive(Serialize)]` Problem type in Rust.

### D-707-14: No interactive API playground
**Source:** [Skene onboarding guide](https://www.skene.ai/resources/blog/developer-onboarding-guide), [Kompassify](https://kompassify.com/blog/developer-onboarding-guide)
**Defect:** No Scalar/Swagger UI/Redoc endpoint for developers to try API calls interactively.
**Impact:** Evaluators must write code before seeing the API work. "The single most important metric is time to first API call" — Skene.
**Fix:** Serve `/docs` with Scalar or Swagger UI from the web server, reading from the OpenAPI spec (D-707-12).

### D-707-15: No personalized docs with injected API keys
**Source:** [Skene onboarding guide](https://www.skene.ai/resources/blog/developer-onboarding-guide)
**Defect:** Documentation shows placeholder keys (`YOUR_API_KEY`). Stripe's pattern: inject actual test key into code samples for logged-in users.
**Impact:** Extra friction — developer must copy key, find paste location, risk typo.
**Fix:** If web dashboard exists, inject test key into code samples via template engine. For CLI, `nt configure` could set a default profile.

### D-707-16: No error message documentation page
**Source:** [Skene onboarding guide](https://www.skene.ai/resources/blog/developer-onboarding-guide)
**Defect:** No page listing common error codes, their meanings, and remediation steps. Error messages in code are terse.
**Impact:** Developers hit errors, don't understand them, close the tab. "A 401 with `invalid_request` and nothing else = closed tab."
**Fix:** Add `docs/errors.md` with table of error codes, causes, and fixes. Link to it from error responses.

---

## Summary: 16 New Defects (Batch 707)

| ID | Domain | Severity | Category |
|----|--------|----------|----------|
| D-707-1 | Documentation | HIGH | Missing `deny(missing_docs)` |
| D-707-2 | Documentation | HIGH | Broken doc links undetected |
| D-707-3 | Documentation | MED | README/doc duplication |
| D-707-4 | Documentation | MED | Feature-gated docs invisible |
| D-707-5 | Documentation | HIGH | No module-level docs |
| D-707-6 | Documentation | LOW | Redundant Arguments/Returns sections |
| D-707-7 | DX | HIGH | No TTFB metric |
| D-707-8 | DX | HIGH | No devcontainer |
| D-707-9 | DX | HIGH | No CONTRIBUTING.md |
| D-707-10 | DX | MED | No golden path docs |
| D-707-11 | DX | MED | Doc build not in CI |
| D-707-12 | API Docs | HIGH | No OpenAPI spec from source |
| D-707-13 | API Docs | HIGH | No RFC 9457 Problem schema (carry) |
| D-707-14 | API Docs | MED | No interactive playground |
| D-707-15 | API Docs | LOW | No personalized docs |
| D-707-16 | API Docs | MED | No error code reference |

---

## Sources Cited

1. Docsio — "Rust Documentation: The Complete Guide for 2026" (2026-05-29)
2. Calmops — "Documentation and API Documentation in Rust" (2026-02-17)
3. Rustdoc book — doc.rust-lang.org (official)
4. Pragmatic Rust Guidelines — Microsoft (microsoft.github.io)
5. Cycloid — "Developer Onboarding Process: The Complete Guide" (2026-04-23)
6. SensioLabs — "The Developer Experience Revolution 2026" (2026-01-30)
7. Kompassify — "Developer Onboarding: Getting to First API Call" (2026-08-20)
8. Skene — "Developer Onboarding Guide" (2026-03-01)
9. ArcDev — "Engineering Onboarding Programs" (2026-07-11)
10. New Stack — "Tackling Developer Onboarding Complexity" (2026-03-18)
11. Detail.cloud — "Developer Onboarding 2026" (2026-01-08)
12. OOPS paper — arxiv 2601.12735 (LLM-based OpenAPI generation)
13. DaloyJS — OpenAPI generation docs
14. zod-openapi — StructIQ GitHub
15. next-openapi-gen — rbong GitHub
16. Swagger.io — official site
