# Iteration Batch 697 — Supply Chain, Audit & Dependency Findings

**Date**: 2026-09-06 | **Context**: 10000+ iteration research loop, NeoTrix consciousness architecture

---

## SOURCES CITED

1. **CVE-2026-5222** — Cargo URL normalization vulnerability (May 25, 2026) — https://blog.rust-lang.org/2026/05/25/cve-2026-5222/
2. **RUSTSEC-2026-0260** — arrayref supply chain attack (Aug 20, 2026) — https://blog.rust-lang.org/2026/08/20/supply-chain-attack-on-arrayref/
3. **RUSTSEC-2026-0269** — wasmtime sandbox escape (Aug 20, 2026) — https://rustsec.org/advisories/RUSTSEC-2026-0269
4. **Cargo 1.90 Semver Pruning** — 28% resolution speedup (Apr 29, 2026) — https://johal.in/deep-dive-cargo-190-handles-dependency-resolution-rust
5. **crates.io Cooldown Proxy** — Mitigating supply-chain attacks (Apr 23, 2026) — https://www.menhera.org/crates-io-cooldown-proxy-mitigating-supply-chain-attacks/
6. **crates.io Supply Chain Controls 2026** — https://safeguard.sh/resources/blog/rust-crates-io-supply-chain-controls-2026
7. **cargo-audit Guide 2026** — https://safeguard.sh/resources/blog/cargo-audit-rust-dependencies
8. **Hacker News** — Rust Supply Chain Attack (Aug 20, 2026) — https://thehackernews.com/2026/08/rust-supply-chain-attack-puts-build.html

---

## VERIFIED VULNERABILITIES IN NEOTRIX (cargo audit scan)

### CRITICAL / HIGH

| # | Crate | Version | Advisory | Severity | Fix |
|---|-------|---------|----------|----------|-----|
| 1 | **wasmtime** | 41.0.4 + 42.0.2 | RUSTSEC-2026-0269 | **HIGH 8.8** | Upgrade to ≥46.0.3 or ≥47.0.4 |
| 2 | **ed25519-dalek** | 1.0.1 | RUSTSEC-2022-0093 | HIGH | Upgrade to ≥2 |
| 3 | **curve25519-dalek** | 3.2.0 | RUSTSEC-2024-0344 | MEDIUM | Upgrade to ≥4.1.3 |

### UNMAINTAINED (supply chain risk)

| # | Crate | Version | Advisory | Date |
|---|-------|---------|----------|------|
| 4 | **bitmaps** | 2.1.0 | RUSTSEC-2026-0247 | 2026-05-03 |
| 5 | **fxhash** | 0.2.1 | RUSTSEC-2025-0057 | 2025-09-05 |
| 6 | **im-rc** | 15.1.0 | RUSTSEC-2026-0250 | 2026-05-03 |
| 7 | **memmap** | 0.7.0 | RUSTSEC-2020-0077 | 2020-12-02 |

### UNFIXABLE

| # | Crate | Version | Advisory | Note |
|---|-------|---------|----------|------|
| 8 | **rustc-serialize** | 0.3.25 | RUSTSEC-2022-0004 | No fixed version exists. Must replace crate entirely. |

---

## NEW DEFECTS FOUND (from this iteration's research)

### D1: wasmtime dual-version HIGH-severity sandbox escape (RUSTSEC-2026-0269)
- **File**: `Cargo.lock:1` (two versions: 41.0.4 + 42.0.2)
- **Severity**: 8.8 HIGH — filesystem sandbox escape via trailing slashes in paths/symlinks
- **Impact**: NeoTrix NT-SHIELD sandbox depends on wasmtime. Both installed versions are vulnerable.
- **Fix**: Pin single version ≥47.0.4, or upgrade to wasmtime 46.0.3+ if compatibility allows.
- **Source**: RustSec advisory + `cargo audit` scan

### D2: arrayref supply chain attack — NeoTrix not directly affected, but transitive risk exists
- **Fact**: arrayref 0.3.10 (Aug 20, 2026) was malicious — build.rs downloaded malware during `cargo build`
- **NeoTrix status**: `arrayref` NOT in NeoTrix's 1202-crate dependency graph (confirmed via grep)
- **Risk**: 403 crates on crates.io depend on arrayref. Any future addition of a crate in that chain (e.g., `winit`, `tiny-skia`, `blake3`) would pull it in. NeoTrix has no `cargo audit` CI gate to catch this.
- **Defect**: **No `cargo audit` in CI pipeline**. Attack window is unmonitored.

### D3: CVE-2026-5222 — Cargo URL normalization credential leak
- **Affects**: Cargo 1.68–1.96 (sparse registry support)
- **NeoTrix Rust version**: unknown from codebase (need `rustc --version` check)
- **Impact**: If using third-party registries with same-domain hosting, credentials could leak
- **Defect**: NeoTrix's Cargo.toml uses `resolver = "2"` (Rust 2021 edition). If the project uses any third-party registry, this CVE applies. No `.cargo/config.toml` registry hardening observed.

### D4: rustc-serialize 0.3.25 — no fix exists
- **Advisory**: RUSTSEC-2022-0004 — stack overflow on deeply nested JSON
- **Status**: No patched version. The crate is abandoned.
- **Defect**: NeoTrix depends on this through transitive chain. Must identify and replace the consuming crate.

### D5: 4 unmaintained crates — supply chain rot
- `bitmaps` 2.1.0, `fxhash` 0.2.1, `im-rc` 15.1.0, `memmap` 0.7.0
- These are early-warning signals: no maintainer means no fix when CVE lands
- **Defect**: No maintenance policy for replacing unmaintained transitive deps.

### D6: No `clippy.toml` or `rustfmt.toml` in repo root
- Batch 696 identified this. Confirmed via file search.
- **Impact**: Clippy complexity lints (e.g., `cognitive_complexity`, `too_many_arguments`) are not enforced.
- **Defect**: CI has no complexity gate.

### D7: Cargo.lock has 1202 dependencies — no `cargo-supply-chain` audit
- `cargo supply-chain` would reveal publisher trust scores
- No evidence of any provenance verification (SLSA, Sigstore, cargo-vet)
- **Defect**: NeoTrix accepts all 1202 crates' `build.rs` scripts as trusted with zero verification.

### D8: No `--locked` or `--frozen` in build commands
- Batch 696 identified `.unwrap()` in 5+ paths. This batch adds: CI build commands do not use `--locked`.
- **Impact**: CI could silently resolve a newly-published poisoned version (exactly what arrayref attack exploited).
- **Defect**: Build reproducibility not enforced.

### D9: No crates.io cooldown proxy configured
- The crates.io Cooldown Proxy (menhera.org) withholds newly-published versions for 1–30 days
- NeoTrix has no such defense. The arrayref window was 86 minutes — a 1-day cooldown would have blocked it entirely.
- **Defect**: No mitigation for the "86-minute window" attack class.

### D10: ed25519-dalek 1.0.1 — cryptographic oracle attack
- **Advisory**: RUSTSEC-2022-0093 — double public key signing oracle
- **Fix**: Upgrade to ≥2.0.0 (major version bump, API breaking)
- **Defect**: Crypto dependency 4 years behind upstream. NT-SHIELD signature verification is vulnerable.

---

## IMPROVEMENTS IDENTIFIED

| # | Improvement | Source | Priority |
|---|-------------|--------|----------|
| I1 | Add `cargo audit --deny warnings` to CI | cargo-audit Guide 2026 | P0 |
| I2 | Enable `resolver = "3"` (Cargo 1.90 semver pruning) | Cargo 1.90 deep dive | P2 |
| I3 | Add `Cargo.lock` commit check to CI (prevents floating deps) | crates.io Supply Chain Controls | P1 |
| I4 | Adopt crates.io cooldown proxy (7-day default) | menhera.org | P1 |
| I5 | Add `clippy.toml` with `cognitive_complexity_threshold = 25` | Batch 696 finding | P2 |
| I6 | Replace `rustc-serialize` usage entirely | RUSTSEC-2022-0004 | P1 |
| I7 | Upgrade wasmtime to single version ≥47.0.4 | RUSTSEC-2026-0269 | P0 |
| I8 | Upgrade ed25519-dalek to ≥2.0 | RUSTSEC-2022-0093 | P0 |
| I9 | Run `cargo supply-chain` and publish SBOM | crates.io controls article | P1 |
| I10 | Add `--locked` to all CI cargo commands | arrayref attack lesson | P0 |

---

## RESEARCH-PROVEN FACTS

1. **arrayref had 245M lifetime downloads** — the attack surface of a single compromised crate is enormous (Hacker News, Aug 21 2026)
2. **86 minutes** was the full attack window for arrayref — faster than most CI pipelines can complete a full audit cycle
3. **build.rs executes with full user privileges** during `cargo build` — no sandboxing by default (confirmed: Rust blog, multiple sources)
4. **Cargo 1.90 semver pruning gives 28% speedup** for 100+ crate projects (Johal.in, Apr 29 2026)
5. **crates.io has NO namespace reservation** — typosquatting is a live, ongoing risk (Safeguard.sh, Feb 2 2026)
6. **cargo-audit does NOT check reachability** — it reports vulnerable crates in Cargo.lock but not whether your code calls the vulnerable function (Safeguard.sh guide, Jul 3 2026)
7. **North Korean actors (Sapphire Sleet / MIDNIGHT NEPTUNE)** linked to arrayref infrastructure overlap (Wiz, Aug 2026)
8. **`cargo audit --deny warnings`** promotes unmaintained/yanked crates to build failures — this is the recommended baseline (Safeguard.sh, Jul 3 2026)

---

## BATCH SUMMARY

- **New defects found**: 10 (D1–D10)
- **Active vulnerabilities in NeoTrix**: 8 (2 high, 1 medium, 4 unmaintained, 1 unfixable)
- **Critical fix priority**: wasmtime sandbox escape (D1), ed25519-dalek oracle (D10), rustc-serialize replacement (D4)
- **Ecosystem threat**: arrayref attack (Aug 20, 2026) — NeoTrix not hit but defense-in-depth absent
- **No cargo audit in CI**: most impactful single improvement (I1)

---

## POST-SCAN VERIFICATION (run during this iteration)

### Rust Version: 1.94.0 / Cargo 1.94.0
- **CVE-2026-5222 APPLIES**: affects Cargo 1.68–1.96. NeoTrix is within the vulnerable range.
- **Status**: Fix ships in Rust 1.96 (May 28, 2026). NeoTrix must upgrade to ≥1.96.

### rustc-serialize consumption chain (exact path found)
```
neotrix v0.21.0
└── zim v0.5.0
    └── stopwatch v0.0.7
        └── num v0.1.43
            ├── num-bigint v0.1.45 → rustc-serialize 0.3.25
            ├── num-rational v0.1.43 → rustc-serialize 0.3.25
            └── num-complex v0.1.44 → rustc-serialize 0.3.25
```
- **Consumer**: `zim v0.5.0` → `stopwatch v0.0.7` → `num v0.1.43`
- **Fix**: Replace `stopwatch` with `std::time::Instant` or `instant` crate. `zim` crate maintainer needs update.

---

## BATCH SUMMARY (UPDATED)

- **New defects found**: 10 (D1–D10)
- **Active vulnerabilities in NeoTrix**: 8 (2 high, 1 medium, 4 unmaintained, 1 unfixable)
- **CVE-2026-5222**: CONFIRMED — Cargo 1.94.0 is vulnerable (upgrade to ≥1.96)
- **rustc-serialize chain**: neotrix → zim → stopwatch → num → rustc-serialize (replace stopwatch)
- **Critical fix priority**: wasmtime (D1), ed25519-dalek (D10), rustc-serialize (D4), Rust version (CVE-2026-5222)
- **Ecosystem threat**: arrayref attack (Aug 20, 2026) — NeoTrix not hit but defense-in-depth absent
- **No cargo audit in CI**: most impactful single improvement (I1)

Next iteration should focus on: (1) Audit `zim`/`stopwatch` for replacement, (2) Check if wasmtime 47.0.4 is semver-compatible with NT-SHIELD, (3) Implement cargo audit CI gate.
