# Iteration Batch 432 — Versioning, Release Management, Crate Publishing Research

**Date:** 2026-09-06
**Research Scope:** 2026 advances in semantic versioning, release management, crate publishing

---

## Sources Cited

| # | Source | Date | Topic |
|---|--------|------|-------|
| S1 | [Semantic Versioning 2026 — ByteLedger](https://byteledger.vizleo.com/blog/semantic-versioning-2026) | 2026-03-03 | SemVer rules, Conventional Commits automation, common mistakes |
| S2 | [Best Automated Changelog Tools 2026 — Notra](https://www.usenotra.com/blog/best-automated-changelog-tools-in-2026) | 2026-05-02 | git-cliff, release-please, Changesets, AI changelog generation |
| S3 | [Release Management 2026 — MTechZilla](https://www.mtechzilla.com/blogs/mastering-release-management-for-product-development) | 2025-07-03 | AI-powered release orchestration, predictive deployment |
| S4 | [Release Management Tools AI-Era — Exceeds AI](https://blog.exceeds.ai/release-management-tools/) | 2026-04-22 | AI-generated code release challenges, 72% production incidents |
| S5 | [cargo-release Releases — GitHub](https://github.com/crate-ci/cargo-release/releases) | 2026-08-11 | v1.1.5 (stable), TOML 1.1, workspace publishing |
| S6 | [cargo-smart-release Releases — GitHub](https://github.com/GitoxideLabs/cargo-smart-release/releases) | 2026-08-18 | v0.21.9, --target passthrough, signoff, emoji |
| S7 | [release-plz — GitHub](https://github.com/release-plz/release-plz) | — | CI-based Release PRs, semver-checks integration, Conventional Commits |
| S8 | [crates.io Trusted Publishing](https://crates.io/docs/trusted-publishing) | 2026 | OIDC publishing, no long-lived tokens, RFC 3691 |
| S9 | [Semantic Versioning Breaking Changes Guide — PkgPulse](https://www.pkgpulse.com/blog/semantic-versioning-guide-breaking-changes-2026) | 2026-03-29 | SemVer contract, 0.x semantics, automation tools |
| S10 | [API Changelog & Versioning Communication — APIScout](https://apiscout.dev/guides/api-changelog-versioning-communication-2026) | 2026-03-29 | Deprecation process, HTTP Deprecation/Sunset headers |
| S11 | [Software Release Management Guide — Harness](https://www.harness.io/blog/software-release-management) | 2026-08-31 | 72% incidents from AI code, developer velocity vs safety |
| S12 | [API Versioning Strategies 2026 — DevWithTools](https://devwithtools.org/blog/api-versioning-best-practices) | 2026-06-28 | URL/header/semver strategies, deprecation patterns |
| S13 | [API Versioning 2026 — DigitalApplied](https://www.digitalapplied.com/blog/api-versioning-strategies-2026-engineering-decision-matrix) | 2026-06-01 | 26% SemVer adoption stat, hybrid versioning |
| S14 | [Changelog Best Practices — Unmarkdown](https://unmarkdown.com/blog/changelog-best-practices) | 2026-02-25 | Keep a Changelog format, automated vs manual |
| S15 | [release-plz Issue #2693](https://github.com/release-plz/release-plz/issues/2693) | 2026-02-27 | git_only + publish=false still attempts crates.io publish |

---

## Defects Found

### DEFECT-432-01: No SemVer Commitment — Stuck at 0.21.0 [CRITICAL]

**Current state:** `Cargo.toml` workspace version = `0.21.0`. SemVer spec 0.x signals "unstable API — anything can break in a minor version" (S1, S9). The project has 11+ domains, a KB system, SEAL pipeline, and a Tauri desktop app — this is not an unstable prototype.

**Gap:** No declared SemVer stability commitment. Consumers cannot distinguish intentional breaking changes from accidental ones. Only 26% of API teams implement SemVer correctly (S13); NeoTrix is in the 74% gap.

**Suggestion:** Bump to 1.0.0 with a deliberate stability commitment. Define the public API surface (CLI interface, KB API, MCP tools). Reserve MAJOR for genuine breaking changes. Adopt Conventional Commits + automated semver bumps.

---

### DEFECT-432-02: Zero CI/CD Release Pipeline [CRITICAL]

**Current state:** No `.github/workflows/` directory exists. No release workflow. `cliff.toml` exists but has no consumer. Release is entirely manual.

**Gap:** 2026 standard is automated release pipelines with AI-powered decision making (S3, S4). Harness reports 72% of organizations hit production incidents from AI-generated code (S11) — automated guardrails are essential, not optional.

**Suggestion:** Implement `release-plz` GitHub Action:
- Conventional Commits → automatic version bumps
- `cargo-semver-checks` integration for breaking change detection
- Release PR workflow: branch → PR → merge → tag → publish
- Git tag → crates.io publish via Trusted Publishing (OIDC)

---

### DEFECT-432-03: No Automated Changelog [HIGH]

**Current state:** `design/CHANGELOG.md` contains only UI preview notes (52 lines, unrelated to releases). Root has no `CHANGELOG.md`. `cliff.toml` is configured but unused.

**Gap:** 2026 best practice: changelog is a trust signal — "no changelog means 'we'll break you without warning'" (S10). A version bump without a changelog entry forces consumers to read the diff (S1, S9). Keep a Changelog format is the standard (S2, S14).

**Suggestion:** Add root `CHANGELOG.md` following Keep a Changelog format. Wire `git-cliff` into the release pipeline. Each release must include:
- Added/Changed/Deprecated/Removed/Fixed/Security sections
- Migration guide for MAJOR bumps
- Links to compare diffs

---

### DEFECT-432-04: No Breaking Change Detection [HIGH]

**Current state:** No `cargo-semver-checks` integration. Workspace crates (`neotrix-types`, `neotrix-sysctl`, `nt-lang`) are interdependent but no automated check validates that version bumps match API changes.

**Gap:** `release-plz` integrates `cargo-semver-checks` for automated API compatibility validation (S7). This catches accidental breaking changes before they ship. Without it, MAJOR/MINOR/PATCH decisions are manual and error-prone.

**Suggestion:** Add `cargo-semver-checks` to CI. Run on every PR that touches public API surfaces. Fail CI on accidental breaking changes in MINOR/PATCH bumps.

---

### DEFECT-432-05: No Trusted Publishing (OIDC) [MEDIUM]

**Current state:** No automated publishing. If publishing were automated, it would likely use static API tokens in secrets.

**Gap:** crates.io Trusted Publishing (RFC 3691) is now production-ready (S8). Uses OIDC cryptographic verification — no long-lived tokens, auto-expiry in 30 minutes, repository/workflow verification prevents unauthorized publishing. GitLab CI/CD support in public beta.

**Suggestion:** When implementing the release pipeline, use Trusted Publishing:
```yaml
permissions:
  id-token: write  # Required for OIDC
steps:
  - uses: rust-lang/crates-io-auth-action@v1
  - run: cargo publish
    env:
      CARGO_REGISTRY_TOKEN: ${{ steps.auth.outputs.token }}
```

---

### DEFECT-432-06: No Conventional Commits Enforcement [MEDIUM]

**Current state:** No commit message standard. `cliff.toml` has `conventional_commits = true` but no enforcement mechanism.

**Gap:** Conventional Commits became the default commit format for automated releases in 2026 (S1). Tools like `semantic-release`, `release-please`, and `release-plz` all parse commit messages to determine version bumps. Without enforcement, the changelog generator produces noise.

**Suggestion:** Add commitlint or `cargo-semver-checks` pre-commit hook. Enforce `feat:/fix:/docs:/chore:/refactor:/test:/ci:` prefixes. Consider `BREAKING CHANGE:` footer requirement for MAJOR bumps.

---

### DEFECT-432-07: Workspace Version Coupling [MEDIUM]

**Current state:** All 6 workspace crates inherit `version = "0.21.0"` from `[workspace.package]`. If one crate has a breaking change, all crates must bump together.

**Gap:** Independent versioning per package is more accurate for multi-crate workspaces (S9, S15). `release-plz` and `cargo-smart-release` support per-package versioning. The current lockstep approach forces unnecessary MAJOR bumps for packages with no breaking changes.

**Suggestion:** Evaluate whether lockstep is intentional. If crates have independent public APIs, switch to independent versioning:
- `neotrix` (CLI) — SemVer per release
- `neotrix-types` — SemVer per breaking change
- `nt-lang` — SemVer per breaking change
- `src-tauri` — follows `neotrix` version

---

### DEFECT-432-08: No Deprecation Process [MEDIUM]

**Current state:** No documented deprecation policy. No `#[deprecated]` macro usage pattern. No HTTP Deprecation/Sunset headers for any exposed API.

**Gap:** Deprecation is a process, not an event: announce → notify → sunset → remove (S10). HTTP `Deprecation` and `Sunset` headers let SDK tooling catch deprecated usage automatically. Proper change communication reduces developer disruption by up to 70% (S10, citing Theneo 2026).

**Suggestion:** Define a deprecation policy:
1. Mark as `#[deprecated(since = "X.Y.Z", note = "use new_fn instead")]`
2. Keep deprecated API for N minor versions
3. Add deprecation entry to CHANGELOG under `Deprecated` section
4. Remove only in next MAJOR version

---

### DEFECT-432-09: No Release Verification Gate [LOW]

**Current state:** No `cargo publish --dry-run` or `cargo package` verification before publish. No `cargo-semver-checks` gate. No MSRV verification in release flow.

**Gap:** `cargo-release` and `release-plz` both verify: right branch, up-to-date with remote, clean tree, dry-run packaging (S5, S7). The `rust-version = "1.81"` field exists but isn't verified against the actual MSRV during release.

**Suggestion:** Add pre-publish verification step:
```bash
cargo publish --dry-run
cargo msrv verify  # or cargo-hack
cargo-semver-checks  # verify no accidental breaking changes
```

---

## Summary

| Severity | Count | Items |
|----------|-------|-------|
| CRITICAL | 2 | No SemVer commitment (0.21.0), Zero CI/CD pipeline |
| HIGH | 2 | No automated changelog, No breaking change detection |
| MEDIUM | 4 | No Trusted Publishing, No Conventional Commits, Workspace coupling, No deprecation process |
| LOW | 1 | No release verification gate |

**Key insight:** NeoTrix has the `cliff.toml` and `rust-version` fields but no pipeline to consume them. The 2026 ecosystem has matured significantly — `release-plz` + `cargo-semver-checks` + Trusted Publishing is now the standard Rust release stack. The project needs to move from "has config files" to "has automated release pipeline."
