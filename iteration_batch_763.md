# Iteration Batch 763 — Documentation Engineering & Doc-as-Code Gaps

**Date**: 2026-09-07
**Predecessor**: Batch 762 (10 defects: LM-CC miscalibration, async complexity invisible, Bumpy Road undetectable, architectural debt unmeasured, coupling×complexity compound risk)
**Context**: NeoTrix consciousness architecture — 10000+ iteration research loop

---

## Research Queries Executed

| # | Query | Sources Found |
|---|-------|---------------|
| 1 | `rustdoc 2026, doc comment 2026, documentation 2026` | Noah Lev (Aug 2026, 33% faster rustdoc), Rust FAQ (Apr 2026), Andrew Odendaal (Sep 2025), Rust Blog project goals (May 2026), docs.rs/doc-comment, blimto.com, Rust Reference |
| 2 | `code documentation 2026, doc lint 2026, intra-doc link 2026` | Rust rustdoc lints (rustdoc 1.98.0, Jul 2026), Fern docs linting guide (Jan 2026), Docsie doc-linting (2026), Dualite code doc best practices (Jun 2026), godoc-lint (Go), Unmarkdown docs-as-code (Feb 2026), pg-doc-quality-lint, Merlin-ai-code-review CHANGELOG |
| 3 | `API doc 2026, OpenAPI doc 2026, doc generation 2026` | ByteLedger (Jun 2026), Alphonsolabs 15 API trends (Jun 2026), APIScout OpenAPI vs AsyncAPI (Mar 2026), DigitalAPI doc generator (Jun 2026), OpenAPI Initiative newsletter (Jun 2026), Docsio OpenAPI tutorial (Apr 2026), FluidTopics trends (Apr 2026) |

---

## Findings

### F1: Filter-First Optimization Pattern — 33% Doc Generation Speedup via Lazy Evaluation

**Source**: Noah Lev Bartell-Mangel, "How I made Rustdoc 33% faster in one week" (Aug 2026, noahlev.org)

**What's NEW**: Rustdoc's `build_extern_trait_impls` pass iterated through every trait impl in every dependency, calling expensive `build_impl` on all of them BEFORE filtering. Noah moved filtering FIRST, skipping `build_impl` for impls that never appear in final documentation. Result: **33% wall time reduction**. Two additional PRs achieved 12% (primitive impls skip) and 6% (synthetic impls skip) further gains.

**Defect D763-1**: NeoTrix's SelfTest registry scanning (`nt_core_self::SelfTest`) follows the SAME pre-optimization pattern as old Rustdoc: it iterates ALL registered detection modules and calls their `evaluate()`/`check()` functions before filtering by relevance. The `run.rs` + `pipeline.rs` SelfTest registries process T1/T2/T3 modules unconditionally. With 36+ L3 vendor skills and growing domain modules, this is the `build_extern_trait_impls` anti-pattern applied to consciousness architecture. **Doc generation (if ever implemented) and SelfTest scanning share the same lazy-evaluation gap**.

**Action**: Implement filter-first for SelfTest registry: add a module-relevance gate (based on recent change set from git diff) BEFORE calling expensive `evaluate()` functions. Benchmark before/after. Store scan duration in KB `experience` namespace under `selftest_perf` key.

---

### F2: rustdoc::lint Module — 11 Doc Quality Lints NeoTrix Ignores Entirely

**Source**: Rust rustdoc lint documentation, rustdoc 1.98.0 (Jul 2026), doc.rust-lang.org/stable/nightly-rustc/rustdoc/lint/

**What's NEW**: Rustdoc ships 11 built-in documentation quality lints: `BARE_URLS`, `BROKEN_INTRA_DOC_LINKS`, `INVALID_CODEBLOCK_ATTRIBUTES`, `INVALID_HTML_TAGS`, `INVALID_RUST_CODEBLOCKS`, `MISSING_CRATE_LEVEL_DOCS`, `MISSING_DOC_CODE_EXAMPLES`, `PRIVATE_DOC_TESTS`, `PRIVATE_INTRA_DOC_LINKS`, `REDUNDANT_EXPLICIT_LINKS`, `UNESCAPED_BACKTICKS`. These run automatically during `cargo doc` and `cargo test --doc`.

**Defect D763-2**: NeoTrix's CI pipeline has ZERO documentation lint integration. The `cargo check --all-targets` and `cargo test --lib` commands do NOT invoke `cargo doc` or `cargo test --doc`. Broken intra-doc links (if any exist in neotrix-core) would silently produce broken HTML on docs.rs. The Merlin-ai-code-review project (Feb 2026) explicitly fixed "all broken intra-doc links, private-item links, and bare URLs" as a release blocker — proving these are real defects in Rust projects.

**Action**: Add `cargo doc --no-deps --document-private-items` and `cargo test --doc` to CI pipeline. Enable `#![warn(rustdoc::broken_intra_doc_links)]` at crate root for neotrix-core. Map each lint to a SelfTest T2 registration.

---

### F3: Docs-as-Code Requires CI-Level Prose Linting — Vale + markdownlint as Baseline

**Source**: Fern docs linting guide (Jan 2026), Unmarkdown docs-as-code guide (Feb 2026), Docsie doc-linting (2026)

**What's NEW**: The 2026 consensus for docs-as-code quality is a **two-tool linting stack**: Vale (prose linter checking terminology consistency, passive voice, jargon, readability against style guides) + markdownlint (structural linter checking heading levels, list indentation, trailing whitespace, line length). These run in CI on every PR, same as code linters. Fern's guide emphasizes: "documentation rarely fails in dramatic ways — it erodes trust slowly through broken links, inconsistent terminology, and subtle style drift."

**Defect D763-3**: NeoTrix has extensive Markdown documentation (AGENTS.md, CONTEXT.md, dev-rules.md, CONTEXT.md, skill files) but ZERO prose linting. Terminology drift is already a known issue (CONTEXT.md's "Flagged Ambiguities" section exists precisely because terms drift). The `experience-tree` SKILL.md, `rev-officer-agent.md`, and all skill files are unlinted. **Terminology drift between docs and code is invisible** — the same problem Vale was designed to solve.

**Action**: Add Vale with custom NeoTrix style rules (enforce `nt_*` prefix, domain names, constellation C0-C6 notation) + markdownlint to CI. Run on AGENTS.md, CONTEXT.md, and all `skills/` markdown files. Flag violations as warnings, not errors (docs drift is gradual, not catastrophic).

---

### F4: AI-Generated Code Documentation Problem — Trust but Verify

**Source**: Dualite, "Code Documentation Best Practices in 2026: AI Tools, JSDoc, and Auto-Generated Docs" (Jun 2026)

**What's NEW**: AI tools (Claude, Copilot, Mintlify Autopilot) can generate first-draft documentation from code signatures and tests. But the 2026 finding is: **AI-generated docs are a starting point, not an endpoint**. The critical gap: AI docs can hallucinate behavior descriptions, omit edge cases, and miss panic conditions. The recommended pattern is: AI generates → human reviews → doctest validates. The `missing_doc_code_examples` rustdoc lint catches items without examples, which is the validation layer.

**Defect D763-4**: NeoTrix's SEAL pipeline distillation phase may produce module documentation via AI. But there is no validation layer that checks: (a) doctests exist for public APIs, (b) doctests actually pass, (c) doc descriptions match actual behavior. The `missing_doc_code_examples` lint would flag every public item in neotrix-core without examples. **AI-generated documentation without doctest validation is a trust hazard** — the doc says one thing, the code does another, and no automated check catches the discrepancy.

**Action**: Enable `#![warn(rustdoc::missing_doc_code_examples)]` for public items in neotrix-core. Add a CI gate that fails if any public function/struct/trait lacks a `# Examples` section with a passing doctest. This is the "documentation as test" pattern that Rust uniquely enables.

---

### F5: OpenAPI 3.1 + MCP-Readiness — AI Agents Are Now a Documentation Audience

**Source**: ByteLedger (Jun 2026), APIScout OpenAPI vs AsyncAPI (Mar 2026), DigitalAPI doc generator (Jun 2026), OpenAPI Initiative newsletter (Jun 2026)

**What's NEW**: Two 2026 shifts in API documentation: (1) **OpenAPI 3.1 closed the gap with JSON Schema** — full `oneOf`/`anyOf` support, nullable types now use `type: ["string", "null"]`, making specs more expressive. (2) **AI agents are a first-class documentation audience**. DigitalAPI states: "In 2026, generated docs must serve AI agents, not just human developers: MCP-readiness and structured metadata are now baseline requirements." APIs need structured metadata, semantic markup, and machine-parseable descriptions for LLM consumption.

**Defect D763-5**: NeoTrix exposes MCP tools via `nt_agent_mcp_gateway` and has an LLM provider layer in NT-IO. But there is NO OpenAPI specification for any NeoTrix API surface. The MCP tool schemas exist in code but are not machine-readable as a standalone spec. When external AI agents or tools want to integrate with NeoTrix's capabilities, they have no machine-parseable contract. **The consciousness architecture is opaque to the AI agent ecosystem it's designed to serve**.

**Action**: Generate OpenAPI 3.1 spec from MCP tool schemas in `nt_agent_mcp_gateway`. Store as `api/openapi/main.yaml` in repo root. Add Spectral linting to CI. This also enables SDK generation for multi-language NeoTrix clients.

---

### F6: Interactive Documentation as Baseline — Static Docs Are a Dealbreaker

**Source**: Alphonsolabs, "15 API Trends for 2026" (Jun 2026), APIScout (Mar 2026)

**What's NEW**: The 2026 consensus: "If your API docs do not have a working 'Try It' button, developers will switch to a competitor that does." Tools like Stoplight, Redocly, and ReadMe generate interactive try-it-out consoles directly from OpenAPI specs. Developers test endpoints inside the docs without writing code. Static HTML output alone no longer meets developer expectations.

**Defect D763-6**: NeoTrix has no developer-facing documentation portal. The CLI (`cargo build -p neotrix`) is the primary interface, but there is no interactive docs site, no API explorer, no try-it console. New users must read AGENTS.md (a 200+ line agent guidance document, not user docs) or parse Rust source. **The documentation experience is engineer-facing, not developer-facing** — a critical gap for adoption.

**Action**: Add a docs site (Astro Starlight or VitePress) as a phase-1 deliverable. Start with CLI command reference auto-generated from clap derive macros. Interactive MCP tool explorer can be phase-2.

---

## Defect Summary

| ID | Defect | Severity | Layer |
|----|--------|----------|-------|
| D763-1 | SelfTest registry lacks filter-first optimization (33% perf gap proven in rustdoc) | HIGH | L5 Cognition |
| D763-2 | Zero doc lint integration (11 rustdoc lints available, none enabled) | HIGH | CI/CD |
| D763-3 | No prose linting for docs-as-code (Vale/markdownlint absent, terminology drift invisible) | MEDIUM | Documentation |
| D763-4 | AI-generated docs lack doctest validation layer (trust hazard) | HIGH | SEAL Pipeline |
| D763-5 | No OpenAPI 3.1 spec for MCP tool surfaces (AI agents can't discover NeoTrix) | CRITICAL | NT-IO |
| D763-6 | No interactive documentation portal (static docs are a dealbreaker in 2026) | MEDIUM | NT-IO |

---

## Sources Cited

1. Bartell-Mangel, N.L. (2026). "How I made Rustdoc 33% faster in one week." noahlev.org/blog
2. Rust rustdoc lint docs (2026). rustdoc 1.98.0. doc.rust-lang.org/stable/nightly-rustc/rustdoc/lint/
3. Fern (2026). "Docs Linting Guide." buildwithfern.com, Jan 2026
4. Unmarkdown (2026). "Docs-as-Code in 2026: The Complete Guide." Feb 2026
5. Dualite / Agarwal, S. (2026). "Code Documentation Best Practices in 2026." Jun 2026
6. ByteLedger (2026). "How to Document an API in 2026: OpenAPI and Developer Docs." Jun 2026
7. Alphonsolabs (2026). "15 API Trends for 2026." Jun 2026
8. APIScout (2026). "API Documentation: OpenAPI vs AsyncAPI 2026." Mar 2026
9. DigitalAPI / Subramanian, D. (2026). "API Documentation Generator: Auto-Generate Docs in 2026." Jun 2026
10. OpenAPI Initiative (2026). Newsletter, June 2026. openapis.org
11. Docsio (2026). "OpenAPI Documentation: Complete Setup Tutorial." Apr 2026
12. Merlin-ai-code-review (2026). CHANGELOG v0.1.6 — rustdoc fixes. github.com/Arunachalamkalimuthu
13. Rust FAQ (2026). "How to Write Documentation Comments in Rust." Apr 2026
14. Odendaal, A. (2025). "Rust Documentation Practices." andrewodendaal.com
15. FluidTopics (2026). "6 Must-Know Technical Documentation Trends Shaping 2026." Apr 2026

---

## Cumulative Defect Count (Batch 761 + 762 + 763)

| Batch | Defects | Critical | High | Medium |
|-------|---------|----------|------|--------|
| 761 | 5 | 0 | 3 | 2 |
| 762 | 10 | 2 | 6 | 2 |
| 763 | 6 | 1 | 3 | 2 |
| **Total** | **21** | **3** | **12** | **6** |
