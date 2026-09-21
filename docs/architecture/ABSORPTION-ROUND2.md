# NeoTrix Absorption Round 2: Fitness / Verifier / Supply-Chain / ADR

> **Absorption Date**: 2026-09-21 | **SIM**: SIM-10 | **Status**: Landed
> **Round 1**: ABSORPTION-GRAPHIFY.md (knowledge graph patterns, R-P230–R-P240)
> **This round**: 4 mechanism directions, 4 rules (R-P246–R-P249), 4 artifacts.

---

## 1. Source Map

| # | Source | Type | Used For |
|---|--------|------|----------|
| S-1 | Ford/Parsons/Kua "Building Evolutionary Architectures" (book + workshop + ffkatas) | Book | Fitness taxonomy, pipeline staging, temporal fitness, weekly report |
| S-2 | SELF-REFINE (Madaan et al., arXiv 2303.17651) | Paper | Feedback→refine loop, multi-aspect feedback + scores, stop condition |
| S-3 | Reflexion (Shinn et al., NeurIPS 2023) | Paper | Actor/Evaluator/Self-Reflection triad, episodic memory bound 1–3 |
| S-4 | Reflect (Bell et al., arXiv 2601.18730) | Paper | Per-principle Likert scoring + threshold-gated critique/revision, 2–5x token saving |
| S-5 | PSR — Progressive Self-Reflection (arXiv 2510.01270) | Paper | Adaptive reflection depth by input risk, mid-generation checks |
| S-6 | Self-Verifying Reflection (NeurIPS 2025) | Paper | Verifier only needs better-than-random; false-negatives worse than false-positives |
| S-7 | Safeguard Rust Supply-Chain Defence Program (2026) | Industry guide | Build.rs/proc-macro tiered review, feature-flag delta, vet as policy input |
| S-8 | gh-guard (sbom-tool) + rust-supply-chain-attestation skill | GitHub project | Minimal/Standard/Hardened levels, Scorecard, SLSA, SHA-pinning, cargo-vet/crev |
| S-9 | cargo-vet (Mozilla) + cargo-crev + crevette | Tool docs | Audit sharing, delta audits, deferred audits (exceptions ratchet) |
| S-10 | MADR 4.x + madr-lint + log4brains + ADR tooling comparison (WhyChose 2026) | Tool docs | Status enum, ISO dates, supersedes-bidirectional, lint rules, no-numbering-gap |

Rejected alternatives: cargo-udeps for unused deps (needs nightly, CI-hostile — cargo-machete chosen in Round 1);
full SLSA provenance pipeline (deferred to P3, needs release workflow changes);
Log4Brains web UI (no non-git stakeholders yet — hand-rolled template suffices).

---

## 2. Pattern → NeoTrix Mapping

### 2.1 Fitness Function Taxonomy (S-1)

Ford categories every guard must carry: scope (atomic/holistic) × cadence
(triggered/continual/temporal) × result (static/dynamic) × invocation (automated/manual).
NeoTrix current 7 guards are all atomic+triggered+static+automated — the taxonomy
exposes exactly what is missing:

| Missing Category | Ford Precedent | NeoTrix Landing |
|-----------------|---------------|-----------------|
| Temporal (allowlist expiry) | allowlist-with-review-date | R-P246: every allowlist carries expiry; expired = red |
| Holistic+triggered | contract tests at integration | P2 search-unification contract tests |
| Holistic+continual | Chaos Monkey / Conformity Monkey | P3 chaos drills (already planned, now named) |
| Manual stages | security review / audit stages | P3 pentest + audit as pipeline stages |
| Weekly architect report | PenultimateWidgets weekly fitness mail | R-P246: weekly SelfTestRegistry summary artifact |
| Coupling metrics | afferent/efferent, instability/distance, JDepend directionality | check-layer-deps.sh IS the directionality guard; efferent-cap guard queued P2 |

### 2.2 Verifier Scoring Standard (S-2–S-6)

Synthesis for SDB-REGISTRY v0.2 (Verifier column upgrade):

1. **Score, don't just pass/fail** (S-4 Reflect): V emits per-policy Likert 1–5;
   score < 3 triggers critique+revision; commit only on pass. This is the typed
   Reject signal with a number attached.
2. **Adaptive depth by risk tier** (S-5 PSR): Triage Gate already classifies input risk;
   low-risk → fast verifier (schema+policy), high-risk → full critique+revision.
   Cost follows risk, not blanket coverage.
3. **Bias to false positives** (S-6 theorem): under fixed compute, false negatives
   (accept bad) cost more than false positives (reject good) → fail-closed is
   theoretically justified, not just cautious.
4. **Episodic memory bound** (S-3 Reflexion, mem 1–3): experience-tree context
   injection must be bounded and scored — link recorded in SDB-REGISTRY.
5. **Stop condition** (S-2 Self-Refine): max N refine rounds + "no further refinement"
   signal; unbounded propose→reject loops are a BLOCKER (DoS on self).

### 2.3 Supply-Chain Tiers (S-7–S-9)

2026 Rust program shape, NeoTrix-sized:

1. **Tiered review**: no-build.rs crates = standard policy (deny+audit already in CI);
   build.rs crates = surface what the script does (new script does this);
   proc-macro exporters = explicit allowlist + human review on add/bump.
2. **Lockfile as policy**: `--locked` builds (check current CI), hash-pinned installs.
3. **Attestation as input, not verdict**: cargo-vet deferred-audit (exceptions ratchet
   down over time — same shape as our fitness allowlist+expiry).
4. **Levels**: Minimal (have: deny+audit+SECURITY.md+SBOM) → Standard (Scorecard,
   Trusted Publishing, CodeQL) → Hardened (SLSA L3, fuzz, osv-scanner). P3 scope.

### 2.4 ADR Automation Bar (S-10)

madr-lint recommended rule set as the lint bar (no new runtime dep — hand-rolled
template + manual index until the failure modes bite: numbering collision, stale
index, dangling supersedes pointers). ADR immutability: only status changes.
Supersession bidirectional. Our extensions (sim-id, quality-attributes, requirements)
ride in frontmatter/body without breaking Nygard/MADR readers.

---

## 3. Landings (this round)

| ID | Artifact | Verifies As |
|----|----------|-------------|
| L-R2-1 | scripts/check-build-surface.sh (baseline: 2 benign build.rs, 0 proc-macro) | `bash scripts/check-build-surface.sh` green |
| L-R2-2 | docs/adr/0000-template.md + docs/adr/README.md | Template + index exist; first real ADR uses it |
| L-R2-3 | SDB-REGISTRY v0.2 scoring section | Section present; SDB-01..07 scoring filled during P1-03 |
| L-R2-4 | Makefile `build-surface` target | `make -n build-surface` resolves |
| L-R2-5 | R-P246–R-P249 in dev-rules v1.5.0 | Rules present, each with source+implementation |
| L-R2-6 | SIM-10 record in SIM-PROTOCOL.md | Row + §8 present |

Deferred (recorded, not dropped): cargo-vet bake (needs supply-chain init + audit labor → P3);
Scorecard workflow (P3); SLSA provenance (needs release changes → P3);
efferent-coupling guard (P2); weekly fitness report artifact (P2 with coverage gate).

---

*End of Absorption Round 2*
