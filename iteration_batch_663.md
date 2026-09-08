# Iteration Batch 663 — SBOM / Supply Chain / Dependency Management

**Date**: 2026-09-06
**Research Domain**: Supply chain security, SBOM standards, dependency management
**Batch 662 Baseline**: (1) no service-to-service identity, (2) no AI agent identity model, (3) no unified trust evaluation engine, (4) no agent identity lifecycle, (5) 5B passkeys in use

---

## 1. SBOM Findings

### Finding 1.1 — CISA 2026 Minimum Elements Update (v2.1, July 29, 2026)
**Source**: https://media.defense.gov/2026/Jul/29/2003971159/-1/-1/1/CSI_2026_cisa_sbom_minimum_elements_508c.PDF

**What's New**: CISA published the first full update to SBOM minimum elements since 2021. Key additions:
- **SBOM Author Signature** (new field) — requires cryptographic author signature on SBOM documents
- **SBOM Data Format Name/Version** (new fields) — SBOM must self-identify its format and version
- **SBOM Generation Context** (new field) — lifecycle phase where SBOM was captured (design/pre-build/build/post-build/operations/discovery)
- **Component Hash Value + Algorithm** (new fields) — per-component artifact hash (SHA-256/SHA-512)
- **Component License** (new field) — SPDX license expression required per component
- **SBOM Tool Name + Version** (new fields) — tooling provenance for reproducibility

**Defect for NeoTrix**: `nt_shield` and `nt_memory` have NO SBOM generation pipeline. The CISA 2026 update introduces **author signature** requirements — NeoTrix needs a signing pipeline (Sigstore-based) for any SBOM it emits. No component in NeoTrix produces machine-processable SBOMs in CycloneDX or SPDX format. **Severity: CRITICAL** — NeoTrix cannot prove what components it ships.

### Finding 1.2 — CycloneDX 1.7 + ECMA-424 Standardization
**Source**: https://github.com/CycloneDX/specification/blob/master/README.md, https://ecma-tc54.github.io/ECMA-424/

**What's New**: CycloneDX 1.7 released Oct 2025, now ratified as ECMA-424. Extended BOM types:
- **ML-BOM** — Machine Learning model components (mlModel, data types)
- **CBOM** — Cryptography Bill of Materials (algorithms, keys, protocols)
- **HBOM** — Hardware Bill of Materials
- **OBOM** — Operations Bill of Materials
- **CDXA** — CycloneDX Attestations

**Defect for NeoTrix**: NeoTrix uses AI models (LLM providers) but has **zero ML-BOM tracking**. No component records which models are used, their versions, or training data provenance. The CBOM capability is also absent — NeoTrix uses cryptographic operations (signing, hashing) but doesn't track algorithm inventory. **Severity: HIGH** — post-quantum readiness requires CBOM; AI governance requires ML-BOM.

### Finding 1.3 — SPDX 3.0 vs CycloneDX Gap Analysis
**Source**: https://runsafesecurity.com/blog/sbom-minimum-elements-cyclonedx-spdx/, https://getsecureslate.com/blog/spdx-vs-cyclonedx, https://safeguard.sh/resources/blog/cyclonedx-vs-spdx-which-format-for-your-program

**What's New**: SPDX 3.0 (2024) added profiles: Core, Software, Build, AI, Dataset, Security. CycloneDX 1.6 added mlModel and data component types. Key gap analysis:
- CycloneDX stronger on: document signature (native JSF), VEX (inline), format self-identification, build-pipeline tooling
- SPDX stronger on: structured "unknown" handling (NOASSERTION/NONE sentinels), license relationship expressiveness, file-level granularity
- **2026 consensus**: Generate both from same pipeline (cost is near-zero), store with each release

**Defect for NeoTrix**: NeoTrix KB has no SBOM schema. No way to query "which components depend on X" across the codebase. No VEX integration to track known vulnerabilities. **Severity: HIGH** — dependency confusion attacks are possible without inventory.

---

## 2. Supply Chain Security Findings

### Finding 2.1 — SLSA v1.2 Draft + Dependency Track
**Source**: https://slsa.dev/spec/draft/

**What's New**: SLSA specification advancing to v1.2 with:
- **Dependency Track** — new track (alongside Build, Source) for tracking dependency integrity
- **Build Environment Track** — hardened build platform requirements
- Build Level 3 remains the practical target for most organizations
- Provenance format: in-toto attestation envelope, Sigstore-signed

**Defect for NeoTrix**: NeoTrix build pipeline has NO provenance attestation. `cargo build` produces artifacts with no signed metadata about how they were built. No SLSA level compliance. **Severity: CRITICAL** — cannot prove build integrity against SolarWinds-type attacks.

### Finding 2.2 — Sigstore Keyless Signing (Widespread Adoption 2026)
**Source**: https://www.youngju.dev/blog/culture/2026-05-16-software-supply-chain-security-2026-sigstore-slsa-sbom-cyclonedx-spdx-chainguard-socket-jfrog-xray-snyk-deep-dive.en, https://pavanrangani.com/blog/supply-chain-security-slsa-sigstore-guide

**What's New**: Sigstore adoption has reached mainstream:
- Kubernetes, npm, PyPI all ship with Sigstore signatures as of May 2026
- GitHub Actions Artifact Attestations GA (May 2024) — Sigstore under the hood
- **Keyless signing is now default** — no long-lived keys, OIDC identity-based, ephemeral certificates (5-10min validity)
- Rekor transparency log provides public audit trail
- `gitsign` for signing git commits with OIDC instead of SSH keys

**Defect for NeoTrix**: NeoTrix artifacts (CLI binary, Tauri desktop, crates) have NO signing. No Sigstore integration. No transparency log entries. Anyone can verify a NeoTrix release hasn't been tampered with — except there's nothing to verify against. **Severity: CRITICAL** — supply chain attack surface is completely open.

### Finding 2.3 — in-toto + GUAC Attestation Graph
**Source**: https://www.youngju.dev/blog/culture/2026-05-16-software-supply-chain-security-2026-sigstore-slsa-sbom-cyclonedx-spdx-chainguard-socket-jfrog-xray-snyk-deep-dive.en

**What's New**: in-toto (CNCF graduated) provides chain-of-custody for software:
- Defines expected build steps and authorized parties per step
- Produces signed attestations at each step
- GUAC (Graph Aggregator for Software Supply Chain) aggregates SBOMs, attestations, vulnerabilities, container metadata into a queryable graph

**Defect for NeoTrix**: No in-toto layout for NeoTrix build pipeline. No attestation chain from source → build → test → release. No GUAC-compatible graph for NeoTrix's dependency relationships. **Severity: HIGH** — no verifiable chain of custody.

### Finding 2.4 — Chainguard Images + Wolfi OS
**Source**: https://www.youngju.dev/blog/culture/2026-05-16-software-supply-chain-security-2026-sigstore-slsa-sbom-cyclonedx-spdx-chainguard-socket-jfrog-xray-snyk-deep-dive.en

**What's New**: Chainguard Images — 350+ distroless container images:
- CVE zero as a target (no shell, no package manager, minimal attack surface)
- Sigstore signed, SBOM bundled
- Wolfi OS — custom distroless base

**Defect for NeoTrix**: NeoTrix container/CLI deployment doesn't use distroless or minimal base images. No Chainguard/Wolfi adoption. Attack surface includes full OS package set. **Severity: MEDIUM** — runtime supply chain hardening absent.

---

## 3. Dependency Management Findings

### Finding 3.1 — Renovate vs Dependabot 2026 State
**Source**: https://blog.codercops.com/blog/renovate-vs-dependabot-dependency-updates-2026, https://rafter.so/blog/renovate-vs-dependabot, https://tomodahinata.com/en/blog/dependabot-vs-renovate-comparison-guide, https://appsecsanta.com/renovate

**What's New**:
| Feature | Dependabot | Renovate |
|---------|-----------|----------|
| Platforms | GitHub only | GitHub/GitLab/Bitbucket/Azure DevOps/Gitea |
| Package managers | 30+ | 90+ (including Cargo) |
| Config model | `.github/dependabot.yml` (simple YAML) | `renovate.json` (layered presets) |
| Grouping | Basic (`groups` key) | Granular (`packageRules` with regex) |
| Automerge | Via GitHub Actions workflow | Built-in first-class policy |
| Dependency Dashboard | No | Yes (single Issue per repo) |
| Self-hosted | No | Yes (AGPL-3.0) |
| Merge Confidence | Compatibility scores | Aggregated CI data badges |
| Regex managers | No | Yes (update versions in any file) |
| Cooldown periods | Yes (1-90 days) | Yes (`stabilityDays`) |

Key insight: **"Use Dependabot first on GitHub. Add Renovate only when Dependabot's grouping/scheduling isn't enough."** Both are free. Running both on same manifests causes duplicate PRs — use Dependabot for security updates only + Renovate for version bumps.

### Finding 3.2 — Monorepo Dependency Update Challenge
**Source**: https://tenthirtyam.org/dispatches/2026/05/13/dependabot-vs-renovate-dependency-management-on-github/

**What's New**: For large monorepos:
- Dependabot requires explicit entry per ecosystem/directory (recent `directories` glob field helps)
- Renovate auto-detects Yarn/npm/pnpm workspaces, Lerna, Nx layouts
- Renovate's shared presets via `extends` roll out org-wide policies
- `regex managers` update versions in non-standard files (Dockerfiles, CI configs, Makefiles)

**Defect for NeoTrix**: NeoTrix is a Cargo workspace with 20+ crates. No dependency update automation configured. Renovate (which supports Cargo natively) is the right tool but not set up. Cargo.lock freshness is unknown. **Severity: MEDIUM** — manual dependency management risks stale dependencies.

### Finding 3.3 — Auto-merge Best Practices 2026
**Source**: https://blog.codercops.com/blog/renovate-vs-dependabot-dependency-updates-2026

**What's New**: Consensus on auto-merge strategy:
- Auto-merge patches to devDependencies where CI is comprehensive
- Auto-merge minor updates with passing CI (highest-leverage config change)
- Never auto-merge major version bumps
- `minimumReleaseAge` (3 days) protects against immediate unpublished packages
- Conservative policy: auto-merge 70% of updates, queue rest for review

**Defect for NeoTrix**: No auto-merge policy. All dependency updates require manual review. For a project with 20+ crates and hundreds of transitive dependencies, this creates review fatigue and stale dependencies. **Severity: MEDIUM** — operational overhead without security benefit.

---

## 4. Meta-Analysis: New Defects for NeoTrix

### Defect 4.1 — CRITICAL: No SBOM Pipeline
NeoTrix produces zero SBOMs. CISA 2026 now requires author signatures, format identification, generation context, component hashes, and license declarations. NeoTrix has none of this. Cannot demonstrate compliance with EU Cyber Resilience Act or US Executive Order 14028.

**Remediation**: Integrate `cargo-cyclonedx` or `syft` into CI to generate CycloneDX SBOMs. Add Sigstore signing. Store SBOMs with each release.

### Defect 4.2 — CRITICAL: No Build Provenance (SLSA)
No SLSA level compliance. No provenance attestation. No in-toto layout. Supply chain attacks (SolarWinds/Codecov/XZ-style) cannot be detected.

**Remediation**: Add `slsa-github-generator` to CI. Reach SLSA Build Level 2 minimum. Add Sigstore signing to release artifacts.

### Defect 4.3 — CRITICAL: No Artifact Signing
NeoTrix CLI binary, Tauri desktop app, and crate publications are unsigned. No Sigstore cosign, no GPG signing, no transparency log entries.

**Remediation**: Add Sigstore keyless signing to release workflow. Sign container images. Log to Rekor.

### Defect 4.4 — HIGH: No ML-BOM / CBOM Tracking
Uses AI models (LLM providers) without tracking which models, versions, or training data. Uses cryptographic operations without algorithm inventory. Post-quantum readiness is blind.

**Remediation**: Adopt CycloneDX ML-BOM for model tracking. Add CBOM for cryptographic asset inventory.

### Defect 4.5 — HIGH: No in-toto Chain of Custody
Build pipeline steps are not formally defined or attested. No guarantee that the artifact running is the one that was built from the expected source.

**Remediation**: Define in-toto layout for NeoTrix build pipeline. Sign each step's attestation.

### Defect 4.6 — MEDIUM: No Dependency Update Automation
No Renovate or Dependabot configured. Cargo workspace with 20+ crates has manual dependency management. Stale dependencies accumulate silently.

**Remediation**: Enable Renovate with Cargo support. Configure grouping for Rust ecosystem packages. Set automerge for patches.

### Defect 4.7 — MEDIUM: No Distroless / Minimal Base Image
Container/CLI deployment includes full OS package set. Attack surface is unnecessarily large.

**Remediation**: Evaluate Chainguard Images or Wolfi OS for container base.

---

## 5. Sources Cited

1. CISA 2026 SBOM Minimum Elements v2.1 — https://media.defense.gov/2026/Jul/29/2003971159/-1/-1/1/CSI_2026_cisa_sbom_minimum_elements_508c.PDF
2. CycloneDX ECMA-424 Specification — https://github.com/CycloneDX/specification/blob/master/README.md
3. ECMA-424 Standard — https://ecma-tc54.github.io/ECMA-424/
4. CISA SBOM to CycloneDX/SPDX Mapping — https://runsafesecurity.com/blog/sbom-minimum-elements-cyclonedx-spdx/
5. SPDX vs CycloneDX 2026 — https://getsecureslate.com/blog/spdx-vs-cyclonedx
6. CycloneDX vs SPDX Format Comparison — https://safeguard.sh/resources/blog/cyclonedx-vs-spdx-which-format-for-your-program
7. SLSA Specification Draft v1.2 — https://slsa.dev/spec/draft/
8. Supply Chain Security 2026 Deep Dive — https://www.youngju.dev/blog/culture/2026-05-16-software-supply-chain-security-2026-sigstore-slsa-sbom-cyclonedx-spdx-chainguard-socket-jfrog-xray-snyk-deep-dive.en
9. SLSA + Sigstore Guide 2026 — https://pavanrangani.com/blog/supply-chain-security-slsa-sigstore-guide
10. Software Supply Chain Security 2026 (Internet Pros) — https://internet-pros.com/blog/software-supply-chain-security-sbom-slsa-sigstore-2026/
11. SBOM + Sigstore + SLSA Guide (DevOpsBoys) — https://devopsboys.com/blog/software-supply-chain-security-sbom-slsa-guide-2026
12. Supply Chain Security 2026 (ZeonEdge) — https://zeonedge.com/pa/blog/software-supply-chain-security-2026-sbom-sigstore
13. Renovate vs Dependabot 2026 (CODERCOPS) — https://blog.codercops.com/blog/renovate-vs-dependabot-dependency-updates-2026
14. Renovate vs Dependabot (Rafter) — https://rafter.so/blog/renovate-vs-dependabot
15. Dependabot vs Renovate Guide (Tomoda Hinata) — https://tomodahinata.com/en/blog/dependabot-vs-renovate-comparison-guide
16. Dependabot vs Renovate on GitHub (TenThirtyAM) — https://tenthirtyam.org/dispatches/2026/05/13/dependabot-vs-renovate-dependency-management-on-github/
17. Renovate Review 2026 (AppSecSanta) — https://appsecsanta.com/renovate
18. OWASP CycloneDX Authoritative Guide — https://www.cyclonedx.org/guides/OWASP_CycloneDX-Authoritative-Guide-to-SBOM-en.pdf
19. Software Supply Chain Security Plain Terms (Nikhil Jathar) — https://www.nikhilj.com/software-supply-chain-security-slsa-sigstore-in-toto/

---

## 6. Summary

| Category | New Defects Found | Severity |
|----------|------------------|----------|
| SBOM Pipeline | 3 (no SBOM gen, no ML-BOM, no CBOM) | CRITICAL |
| Build Provenance | 2 (no SLSA, no in-toto) | CRITICAL |
| Artifact Signing | 1 (no Sigstore) | CRITICAL |
| Dependency Management | 3 (no automation, no automerge, no distroless) | MEDIUM |
| **Total** | **9 new defects** | |

**Batch 663 Delta**: Previous batch identified 5 identity-related gaps. This batch adds 9 supply-chain gaps. Combined attack surface: NeoTrix cannot prove what it ships, how it was built, or who signed it — and dependencies update manually with no provenance chain.
