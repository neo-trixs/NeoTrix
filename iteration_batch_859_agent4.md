# Agent 4: Nix Reproducible Builds (Batch 859)

## Sources

1. **rustfaq.org** — "How to Use Nix for Reproducible Rust Builds" (2026-04-17) — Flake setup, rust-overlay, pitfalls
2. **codex.danielvaughan.com** — "Codex CLI on NixOS: Reproducible Agent Environments with Nix" (2026-05-13) — Multi-language Nix flakes, container builds
3. **EffortlessMetrics/hl7v2-rs-swarm** — ADR-0005: Nix for Reproducible Builds (2025-11-19) — Production ADR: flake structure, CI parity, Docker images
4. **safeguard.sh** — "Nix Reproducible Builds: A Supply Chain Case Study" (2026-03-04) — Supply chain trust boundaries, FOD security, bootstrapping
5. **safeguard.sh** — "Rust Supply Chain Security: The 2026 Landscape" (2026-07-15) — crates.io attack surface, cargo-deny, SBOM gaps
6. **fzakaria.com** — "Demystifying Nix's Intensional Model" (2025-03-08) — Content-addressed derivations, early-cutoff, binary reproducibility limits
7. **zemdregon.github.io/nix-docs** — "Supply Chain" — Trust surfaces, FOD security, substituter trust, lockfile attestation gaps
8. **zemdregon.github.io/nix-docs** — "Import From Derivation" — IFD mechanics, sequential evaluation, performance implications
9. **wyattgill9/knowledge-base** — "Bleeding-edge Rust on NixOS: 2025–2026 practitioner's guide" — crane vs naersk vs buildRustPackage, flake-parts, cross-compilation
10. **nixcademy.com** — "Demonstrably Secure Software Supply Chains with Nix" (2025-05-12) — Verifiable closures, rebuild from source, NixOS ISO
11. **wiki.nixos.org** — "Ca-derivations" — Content-addressed derivation opt-in, experimental status
12. **Advanced Nix Breakdown (wyattgill9 gist)** (2026-07-12) — CA derivation stabilization ~65%, fork landscape (CppNix/Lix/Determinate), IFD limitations
13. **nixpkgs/rust.section.md** — Official Rust packaging guide: buildRustPackage, cargoHash, Cargo.lock requirements
14. **NixOS Discourse** — "Nix flakes explained: what they solve, why they matter" (2025-11-19) — Community debate on reproducibility claims, pinning vs attestation
15. **harmonia (nix-community)** — Rust binary cache server, content compression, TLS

## Defects

### D-NIX-001: No Reproducible Build Environment — Zero Nix Infrastructure
- **Defect**: NeoTrix has no `flake.nix`, `flake.lock`, or any Nix configuration. There is no mechanism to produce a hermetic, reproducible build environment. Every developer's local toolchain, system libraries, and environment variables are uncontrolled variables.
- **File**: project root (no `.nix` files exist)
- **Severity**: HIGH
- **Source**: rustfaq.org, hl7v2-rs-swarm ADR-0005

### D-NIX-002: Cargo.lock Gitignored — Binary Project Missing Dependency Pin
- **Defect**: `Cargo.lock` is listed in `.gitignore` (line 2). NeoTrix produces binary targets (`neotrix`, `neotrix-dl`). Per Cargo's own best practices and nixpkgs Rust packaging guidelines, binary projects MUST commit `Cargo.lock` to ensure reproducible dependency resolution. Without it, two builds from the same source may resolve different transitive crate versions, producing non-identical binaries and breaking Nix's content-addressed store assumptions.
- **File**: `.gitignore:2`, `Cargo.toml:18-24` (bin targets)
- **Severity**: CRITICAL
- **Source**: nixpkgs/rust.section.md, safeguard.sh (Rust supply chain)

### D-NIX-003: Unpinned Git Dependency — `holon` Without Revision Hash
- **Defect**: `holon = { git = "https://github.com/watmin/holon-rs", optional = true }` in both `neotrix-core/Cargo.toml:111` and `crates/neotrix-types/Cargo.toml:29` specifies no `rev`, `tag`, or `branch`. This means every `cargo update` (or fresh clone without a lockfile) can pull a different commit. In a Nix context, this is a non-FOD input that cannot be content-addressed — the store path changes whenever the git HEAD moves. Combined with D-NIX-002, this is a compounding supply chain vulnerability.
- **File**: `neotrix-core/Cargo.toml:111`, `crates/neotrix-types/Cargo.toml:29`
- **Severity**: CRITICAL
- **Source**: safeguard.sh (supply chain), zemdregon (trust surfaces)

### D-NIX-004: No Pinned Rust Toolchain — Compiler Version Drift
- **Defect**: No `rust-toolchain.toml` exists. The workspace declares `rust-version = "1.81"` (MSRV) but actual builds can use any Rust version ≥1.81. Different Rust patch versions can emit different machine code for identical source, breaking bit-for-bit reproducibility. Nix flakes solve this via `rust-overlay` + `fromRustupToolchainFile`, but NeoTrix has no such mechanism. The `profile.release` settings (`lto = true`, `codegen-units = 1`) are particularly sensitive to compiler version changes.
- **File**: `Cargo.toml:18`, `Cargo.toml:36-40`
- **Severity**: HIGH
- **Source**: rustfaq.org, hl7v2-rs-swarm ADR-0005, wyattgill9 (practitioner's guide)

### D-NIX-005: No Binary Cache Infrastructure — No Dependency Pre-Build
- **Defect**: No Cachix, Harmonia, or Nix binary cache is configured. For a project with 100+ dependencies (many with C/system library dependencies like `rusqlite`, `openssl`, `ring`), every developer must compile the full dependency tree from source on first build. This compounds with D-NIX-001 to make onboarding slow and environments divergent. The Nix ecosystem standard is `cachix/cachix-action` for CI + Cachix for developer caching.
- **File**: project root
- **Severity**: MEDIUM
- **Source**: wyattgill9 (caching strategy), harmonia (nix-community)

### D-NIX-006: No Platform-Specific Build Input Declaration
- **Defect**: NeoTrix uses macOS-specific frameworks (via `rcgen`, `rustls`, `keyring` for macOS Keychain) and Linux-specific system libraries (`rusqlite` bundled SQLite, `openssl`/`rustls`). There is no Nix `mkShell` or `buildRustPackage` with `lib.optionals stdenv.isDarwin` conditionals. Cross-platform builds rely entirely on the host system having the correct libraries installed — the opposite of hermetic builds.
- **File**: `neotrix-core/Cargo.toml:46,106,114,128`
- **Severity**: HIGH
- **Source**: hl7v2-rs-swarm ADR-0005, rustfaq.org

### D-NIX-007: Minimal build.rs — No Reproducible FFI Generation
- **Defect**: `neotrix-core/build.rs` only watches a UDL file for `uniffi-bindgen-swift`. The `uniffi` build dependency (`uniffi = { version = "0.28", features = ["build"] }`) is pinned to `0.28` but its transitive dependencies are not. More critically, the build script does not verify or pin the exact `uniffi-bindgen-swift` binary version used for iOS binding generation. Different uniffi versions produce different Swift bindings, creating non-reproducible FFI surfaces.
- **File**: `neotrix-core/build.rs:5-10`, `neotrix-core/Cargo.toml:143`
- **Severity**: MEDIUM
- **Source**: nixpkgs/rust.section.md (build script reproducibility)

### D-NIX-008: Release Profile May Produce Non-Deterministic Binaries
- **Defect**: `profile.release` sets `codegen-units = 1` and `lto = true`. While these optimize for size/performance, they interact with compiler version (D-NIX-004) and platform to produce potentially non-deterministic output. More importantly, `strip = "symbols"` depends on the host system's `strip` binary, which varies across platforms. A Nix-based build would use a pinned `binutils`, but without Nix, this is an uncontrolled variable.
- **File**: `Cargo.toml:36-40`
- **Severity**: MEDIUM
- **Source**: fzakaria.com (binary reproducibility limits), reproducible-builds.org

### D-NIX-009: No SBOM Generation for Release Artifacts
- **Defect**: No software bill of materials (SBOM) is generated for release binaries. Rust's static linking means a single binary bundles dozens of transitive dependencies with no runtime artifact trail. The Rust supply chain security landscape (safeguard.sh 2026) identifies this as a critical gap: "accurate, automated SBOM generation (rather than manual Cargo.lock inspection) is essential for anyone who needs to answer 'are we affected by this new RustSec advisory' in minutes rather than days." Without SBOM, NeoTrix cannot answer this question at all since Cargo.lock isn't even committed.
- **File**: project-wide
- **Severity**: HIGH
- **Source**: safeguard.sh (Rust supply chain 2026), zemdregon (supply chain)

### D-NIX-010: No CI Reproducibility Gate — No `nix flake check`
- **Defect**: No CI pipeline runs `nix flake check` or equivalent reproducibility verification. The hl7v2-rs-swarm ADR demonstrates that `nix flake check` should mirror CI: build, format, clippy, test — all as Nix derivations ensuring identical tool versions. Without this, CI may pass with different tool versions than developers use locally, creating "works in CI, breaks locally" or vice versa.
- **File**: project CI config (no Nix integration)
- **Severity**: HIGH
- **Source**: hl7v2-rs-swarm ADR-0005, rustfaq.org

### D-NIX-011: No Build Provenance / Supply Chain Attestation
- **Defect**: No SLSA provenance, Sigstore signing, or build attestation exists for release artifacts. Nix's content-addressed store provides integrity verification (hash chain), but not provenance (who built it, from what source, with what tools). The supply chain docs (zemdregon) explicitly state: "Lockfiles pin, they do not attest." Without attestation, consumers of NeoTrix binaries cannot verify that a binary was built from the claimed source code by a trusted builder.
- **File**: project-wide
- **Severity**: MEDIUM
- **Source**: zemdregon (supply chain), nixcademy.com (verifiable closures)

### D-NIX-012: No Content-Addressed Derivation Usage — Missing Early-Cutoff Benefits
- **Defect**: NeoTrix has no Nix infrastructure, and therefore cannot leverage content-addressed (CA) derivations for early-cutoff optimization. CA derivations (RFC 62) allow Nix to skip rebuilding a derivation if its dependencies haven't changed bit-for-bit, even if the derivation hash has changed. For NeoTrix's large dependency tree (~100+ crates including heavy C dependencies like SQLite, zstd, image codecs), this would provide significant CI and developer build time savings. CA derivations are still experimental (~65% stabilization as of March 2026) but the infrastructure gap means NeoTrix cannot benefit when they stabilize.
- **File**: project-wide
- **Severity**: LOW (future capability gap)
- **Source**: fzakaria.com, wiki.nixos.org (ca-derivations), Advanced Nix Breakdown

## Key Insights

1. **The Cargo.lock gap is the single highest-impact defect**: Without committing `Cargo.lock`, NeoTrix cannot guarantee reproducible dependency resolution even within `cargo build`, let alone across environments. This must be fixed before any Nix infrastructure is added.

2. **The unpinned git dependency (`holon`) creates a double vulnerability**: It bypasses both Cargo's checksum verification and any future Nix content-addressing. The `follows` mechanism in Nix flakes cannot pin what Cargo cannot pin.

3. **The Nix ecosystem has converged on a clear Rust stack (2025–2026)**: `rust-overlay` for toolchains, `crane` for builds, `flake-parts` for structure, `nix-direnv` for shell entry. NeoTrix should adopt this stack rather than inventing a custom solution.

4. **Content-addressed derivations are the future but not ready**: CA derivations offer early-cutoff and multi-user trust, but are ~65% stabilized (Milestone #35, last updated March 2026). The ecosystem is split: CppNix pursues stabilization, Lix is deleting the feature, Determinate ignores it. NeoTrix should design for CA compatibility but not depend on it today.

5. **IFD is the hidden performance killer**: Import From Derivation (IFD) pauses evaluation to build store objects, making `nix flake show` and `nix search` slow. Crane and naersk avoid IFD by parsing `Cargo.lock` in pure Nix. NeoTrix should prefer IFD-free build frameworks.

6. **Rust supply chain security is in a critical transition**: crates.io has 170K+ packages with 100B+ downloads. Attacks are following. The tooling (`cargo-deny`, `cargo-vet`, `cargo-audit`) is strong but NeoTrix uses none of it. Nix provides the build environment layer, but the dependency vetting layer is separate and equally important.

7. **The fork landscape matters for long-term Nix investment**: CppNix (reference), Lix (community fork removing CA), Determinate (commercial, parallel eval), Tvix/Snix (Rust reimplementation). NeoTrix choosing Nix must track which fork's features align with its needs — particularly for CA derivations and recursive-nix.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 12 |
| Defects by severity | CRITICAL: 2, HIGH: 5, MEDIUM: 4, LOW: 1 |
| Sources consulted | 15 |
| Key architectural gap | No Nix infrastructure + no Cargo.lock + unpinned git dep |
