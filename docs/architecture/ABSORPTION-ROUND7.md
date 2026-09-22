# NeoTrix Absorption Round 7: Flags / Signing / Behavioral Scanning

> **Absorption Date**: 2026-09-21 | **SIM**: SIM-37 | **Status**: Landed
> **Rounds**: R1 graphify · R2 fitness/verifier/supply/ADR · R3 routing/OTEL/memory/fuzz
> | R4 interop/eval/release/prompts · R5 adversarial/DORA/arch-tooling · R6 API/Diátaxis/judge/chaos |
> **This round: NTS-G09 (flag discipline) + IOC tripwire + signing bake**.

---

## 1. Source Map

| # | Source | Type | Used For |
|---|--------|------|----------|
| S-39 | OpenFeature spec + OFREP + Flagd (CNCF incubating) | Standard | Vendor-neutral SDK shape, file backend, provider swap |
| S-40 | Bubble/OpenFeature-in-practice + Hassan + Spinella (2026) | Guides | TTL-at-creation, kill-switch testing, progressive 1→100%, hooks, sticky bucketing |
| S-41 | sigstore/cosign (keyless Fulcio+Rekor) + sigstore-rust + sigstore-sign + jdx/sigstore-verification | Tools | Keyless signing flow, Rust-native verify, mise precedent |
| S-42 | Socket.dev Rust GA + Aug-2026 proc-macro1 attack + time/finch campaigns | Threat intel | Build-time loader pattern, typosquat/brandjacking, .env targeting |
| S-43 | Socket MCP (flags hallucinated packages in AI suggestions) | Tool note | Agent-assisted dep additions need a checker (P3) |

Rejected: vendor-locked flag SDKs (OpenFeature instead); managed-key signing before keyless bake;
full Socket suite (enterprise tier — behavior list distilled into IOC script instead).

---

## 2. Pattern → NeoTrix Mapping (with local baselines)

### 2.1 Flag discipline (S-39–S-40)

Local baseline: threshold flips done by code edit (SDB scores, coverage 70→80,
bench gates) — no flag mechanism, every flip is a deploy.

Landing (NTS-G09): release flags carry TTL at creation and die on schedule;
kill switches tested in non-production on a cadence; progressive rollout
1%→100% with sticky bucketing and per-stage metrics; flag evaluation never in
the availability critical path (fail-open default + tight timeout);
hooks for metrics/tracing on every evaluation; permission checks are NOT flags
(authorization stays audited). Start backend: Flagd-style file rules in repo
(zero vendor), graduate only on pain.

### 2.2 Signing bake (S-41)

Local baseline: SBOM via cyclonedx (CI), no signatures, no provenance verification.

Landing (P3 bake): cosign keyless (Fulcio+Rekor) on release artifacts;
sigstore-rust/jdx crates for in-repo verification (mise precedent: replace CLI
calls with the library); identity policy on CI ambient credentials.
No new keys to manage — that is the point of keyless.

### 2.3 Behavioral IOC tripwire (S-42–S-43)

Local baseline: deny/audit cover advisories+policy, blind to behavior.

Landing (this round, executable): `scripts/check-supply-iocs.sh` sweeps Cargo.lock
for the Aug-2026 loader (`proc-macro1` et al.), pinned-malicious exact versions,
typosquat/brandjacking names and .env exfiltrators. Verified both directions
(clean→0, poisoned→1) without touching the real lockfile (parametrized path).
Socket full suite (App/CLI/Firewall/MCP) stays P3; the IOC list is the distilled
zero-cost subset. IOC review date is part of the script header — stale lists rot.

---

## 3. Landings (this round)

| ID | Artifact | Verifies As |
|----|----------|-------------|
| L-R7-1 | scripts/check-supply-iocs.sh (dual-verified, parametrized) | clean 0 / poisoned 1 |
| L-R7-2 | NTS-G09 in NT-STD 1.0.5 | clause present with Verify |
| L-R7-3 | Makefile `supply-iocs` target | `make -n` resolves |
| L-R7-4 | SIM-37 record | row + §36 present |
| L-R7-5 | BLUEPRINT v1.6.2 changelog | line present |

Deferred (recorded): Flagd backend + progressive rollout wiring (P-task, needs first
flag candidate — SDB log-only flip is the natural first use); cosign CI job (P3);
Socket suite (P3).

---

*End of Absorption Round 7*
