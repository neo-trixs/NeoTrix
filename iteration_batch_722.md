# Iteration 722 — Container Runtime / OCI / Build Ecosystem Survey

**Date**: 2026-09-06
**Batch**: 722 of 10000+
**Context**: Continuing from Batch 721 (no merge queue, no feature flags, trunk-based dev, AI autonomous merging, AI code review bottleneck)

---

## Sources Cited

1. **Youngju Kim** — "Container Runtime Alternatives 2026 Deep Dive" (2026-05-16) — comprehensive runtime landscape
2. **containerd.io/releases** — containerd v2.3+ (April 2026+), 4-month minor release cadence
3. **EITT Academy** — "Docker vs Podman vs containerd 2026" (2026-03-30) — side-by-side comparison
4. **Lucaberton** — "Podman vs containerd 2026" (2026-04-04) — daemonless vs daemon analysis
5. **Panelica** — "Docker vs Podman: Which Container Runtime in 2026?" (2026-05-05) — security-first angle
6. **PikVue** — "Docker vs Podman vs Containerd 2026" (2026-06-22) — benchmarked head-to-head
7. **Kubernetes.io** — Container Runtimes documentation (2026-05-18)
8. **containers.news** — "OCI Image Spec Update 2026: Security Hooks and SBOMs" (2025-12-30)
9. **opencontainers.org** — OCI Distribution Spec Conformance Redesign (2026-04-06)
10. **opencontainers.org** — OCI Runtime Spec v1.3 (2025-11-04)
11. **buildah.io** — Buildah v1.44.0 Release (2026-05-27)
12. **buildah.io** — Buildah v1.45.0 Release (2026-07-30)
13. **DEV Community / DataFormatHub** — "Why Podman and Buildah are Replacing Docker in 2026" (2026-02-02)
14. **Stack Harbor** — "The OCI image spec — what your container image actually is" (2026-04-22)
15. **Oracle Cloud Blog** — OCI Container Services Newsletter July 2026 (2026-07-27)

---

## A. Container Runtime Landscape (2026 State)

### Key Facts
- **95%+ of K8s clusters** run containerd or CRI-O (May 2026)
- **containerd 2.0** is cluster default; v2.3 shipped April 2026 with 4-month cadence
- **CRI-O 1.31** = lightweight K8s-only runtime; stream isolation, userns auto-mode, Wasm runtime class
- **Podman 5.4** (May 2026): daemonless, rootless-by-default, pod concept, Docker CLI compatible
- **Fedora, RHEL, Rocky Linux** moved entirely to Podman as default container tool
- **OrbStack** eating into Docker Desktop share on macOS; enterprise users fleeing Docker Desktop paid policy
- **gVisor, Kata Containers, Firecracker** = standard isolation primitives in multi-tenant SaaS/serverless
- **WasmEdge/Wasmtime via runwasi shim** = enters K8s properly; faster startup, lighter memory
- **Confidential Containers** expected GA late 2026 (AMD SEV-SNP, Intel TDX mature)
- **Rootless + userns + cgroups v2 + seccomp + eBPF LSMs** = 2026 baseline

### Runtime Layer Architecture (2026)
```
L5: Registry (ECR/GAR/Harbor) + Signing (cosign) + Scanning (Trivy)
L4: High-level runtime (containerd, CRI-O, Podman)
L3: Low-level runtime (runc, crun, youki, kata-runtime, runsc)
L2: Isolation (namespaces+cgroups, microVMs, userspace kernels, Wasm)
L1: OCI specs (image-spec, runtime-spec, distribution-spec)
```

---

## B. OCI Specifications (2026 Updates)

### OCI Image Spec 2026 Update (Breaking)
Three headline features:
1. **Structured SBOM support** inside image manifests with standard query APIs
2. **Runtime attestation hooks** — runtimes publish signed execution evidence
3. **Extensible image signing primitives** — multi-signer workflows for supply chain

### OCI Runtime Spec v1.3.0 (Nov 2025)
- 24 PRs merged since 1.2
- Foundation for 2026 runtime implementations

### OCI Distribution Spec Conformance Redesign (Apr 2026)
Complete rewrite of conformance suite:
- Old: 4-group monolithic go test binary, copy-paste code, first-error-only reporting
- New: Granular API selection, data type selection, Pass/Skip/Disabled/FAIL results
- Tests edge cases: artifacts, different digest algorithms, empty lists, nested indexes
- Supports blob range requests (lazy loading, chunked pulling)
- Results in `results.yaml` with redacted config + granular API/data test status
- Backwards-compatible transition; registry authors will see new issues

### OCI image-spec current state
- image-spec 1.1 + 1.2 RC (May 2026)
- runtime-spec 1.2.31
- distribution-spec redesigned conformance

---

## C. Container Build Tools (2026 Updates)

### Buildah v1.44.0 (May 2026) — MAJOR
- **Removed**: slirp4netns, cgroups v1, CNI support (breaks legacy setups)
- **Added**: `--metadata-file` for build/commit, `--mount` for all RUN commands, `--save-stages`, `--stage-labels`, `--source-policy-file` (BuildKit-compatible), `--valid-exit-codes`, `--compression-format`/`--cache-compression-format`
- **Fixed**: Race condition during cache lookup (#6688), broken pipe on tar archives (#6678), stale images with bind mount (#6845)
- **New**: FROM `--after<stage>` for explicit stage dependencies, secret-to-env-var mounting, Windows cross-build from Linux
- **Ships on**: Fedora 43/44, included in Podman v6.0
- **Buildah farm build** = distributed builds (competing with BuildKit)

### Buildah v1.45.0 (Jul 2026)
- **Removed**: CirrusCI (closed June 2026), migrated CI
- **Added**: `--no-follow-symlinks` for `buildah add`, `--timestamp 0` for manifest push
- **Fixed**: RUN --mount cache invalidation bug (implicit bind mount not checksummed → stale cache reuse #6957)
- **Fixed**: Symbolic chmod notation for add/copy, stage resolution uses last-match not first-match
- **Fixed**: Empty layer elimination when `--layers=false` + metadata-only changes
- **Improved**: Reject explicit seccomp profiles when not built with seccomp support

### BuildKit / Docker Build
- BuildKit v0.30.0 vendored by Buildah (cross-pollination)
- Docker Engine v28.5.2 / Moby v2.0.0-beta.8 vendored by Buildah
- BuildKit remains dominant for Docker ecosystem

### Kaniko
- Comparison with Buildah: Buildah wins on rootless builds, scripted workflows; Kaniko wins on K8s-native (no privileged needed)

### Buildpacks / ko
- No major 2026 updates surfaced in search; Buildah + BuildKit dominate

---

## D. Defects Found

### DEFECT 1: Buildah v1.44.0 Race Condition in Cache Lookup
- **File**: buildah #6688
- **Issue**: Race condition when storage layer removed in parallel during cache lookup
- **Impact**: Build failure on concurrent builds with layer cleanup
- **Fix**: Merged in v1.44.0 by @Luap99
- **NeoTrix relevance**: If using Buildah for container builds, concurrent CI jobs could fail unpredictably

### DEFECT 2: Buildah v1.45.0 RUN --mount Cache Staleness
- **File**: buildah #6957
- **Issue**: `RUN --mount` flags omitting `type=` (implicit bind mount) were NOT checksummed for cache invalidation
- **Impact**: Builds silently reuse stale cached layer when only mounted file content changed
- **Fix**: Merged in v1.45.0 by @MayukhSobo
- **NeoTrix relevance**: **CRITICAL** — silent stale cache = non-deterministic builds, security risk if mounted configs change but cache isn't invalidated

### DEFECT 3: OCI Conformance Suite Was Non-Granular
- **File**: opencontainers.org (distribution-spec conformance redesign)
- **Issue**: Old conformance tests were monolithic go test binary, first-error-only, no data type selection
- **Impact**: Registries with partial API support couldn't get granular pass/fail; bugs masked
- **Fix**: Complete rewrite (April 2026) with Pass/Skip/Disabled/FAIL granularity
- **NeoTrix relevance**: If NeoTrix ships a registry or OCI tool, must use new conformance suite for validation

### DEFECT 4: OCI Image Spec SBOM Support Creates Migration Risk
- **File**: containers.news (OCI Image Spec Update 2026)
- **Issue**: New SBOM schemas may break toolchains that haven't adopted them
- **Impact**: Registries, scanners, admission controllers need coordinated upgrade
- **Mitigation**: Phased rollout, mocking for policy enforcement testing
- **NeoTrix relevance**: Any NeoTrix module touching OCI images must handle both old and new SBOM formats during transition

### DEFECT 5: CRI-O 1.31 Stream Isolation Gap
- **File**: youngju.dev (Container Runtime Alternatives 2026)
- **Issue**: CRI-O stream isolation runs container log/exec streams in separate process to prevent OOM cascade
- **Impact**: If a container OOMs, its log/exec streams survive — but cross-process log correlation breaks
- **NeoTrix relevance**: Monitoring/observability must handle detached stream processes

### DEFECT 6: Podman Rootless Permission Gotcha
- **File**: DataFormatHub (2026-02-02)
- **Issue**: Userns remapping fundamentally changes file permissions/volume mounts; requires `--userns=keep-id` or `--userns=auto` + SELinux `:Z`/`:z` labels
- **Impact**: Permission denied errors when mounting host directories; learning curve steeper than advertised
- **NeoTrix relevance**: NT-SHIELD (sandbox/egress) must account for userns remapping in rootless mode

### DEFECT 7: Buildah Removed CNI + Slirp4netns (Breaking)
- **File**: buildah v1.44.0 release notes
- **Issue**: CNI support removed, slirp4netns removed — legacy networking paths gone
- **Impact**: Any CI/CD using Buildah with CNI networking breaks on upgrade to v1.44.0+
- **NeoTrix relevance**: Build toolchain must migrate to pasta/netavark before upgrading Buildah

### DEFECT 8: containerd 4-Month Release Cadence Creates Upgrade Pressure
- **File**: containerd.io/releases
- **Issue**: containerd 2.3+ ships minor releases every 4 months (April/August/December)
- **Impact**: Cluster operators must keep pace or fall behind on security patches
- **NeoTrix relevance**: NT-PHYSICAL (power/resource management) must track containerd version for compatibility

### DEFECT 9: Wasm Runtime Interoperability Fragmentation
- **File**: youngju.dev (Container Runtimes 2026 Deep Dive)
- **Issue**: WasmEdge, Wasmtime, Wasmer, Spin, WasmCloud all plug into K8s via runwasi shim — but no standard interop between them
- **Impact**: Choosing wrong Wasm runtime = vendor lock-in within the "open" ecosystem
- **NeoTrix relevance**: If NT-CORE uses Wasm for modular components, must abstract runtime behind runwasi shim

### DEFECT 10: Buildah --mount Stale Image Bug
- **File**: buildah #6845
- **Issue**: Using a previous build stage as bind mount source could reference stale images
- **Impact**: Multi-stage builds mount outdated filesystem snapshots
- **Fix**: Merged in v1.44.0 by @ekedaigle
- **NeoTrix relevance**: Multi-stage builds (common in production) could silently use wrong artifacts

---

## E. New Improvements Identified

### IMPROVEMENT 1: OCI SBOM Attestation Runtime Enforcement
- Runtime attestation hooks + admission controller validation = shift-right security
- Registries become policy hubs; runtime evidence enables forensic root cause analysis
- **Action for NeoTrix**: Integrate OCI attestation hooks into NT-SHIELD audit pipeline

### IMPROVEMENT 2: Buildah --source-policy-file (BuildKit-compatible)
- Source policies allow controlling where images are fetched from
- Prevents supply chain attacks via registry redirection
- **Action for NeoTrix**: Implement source policy for NT-ACT build toolchain

### IMPROVEMENT 3: Buildah --save-stages + --stage-labels
- Preserve and label intermediate stage images in multi-stage builds
- Enables debugging, caching, and reproducibility of build internals
- **Action for NeoTrix**: Use in CI for build artifact inspection

### IMPROVEMENT 4: CRI-O Stream Isolation
- Separate process for log/exec streams = OOM resilience
- Container crash doesn't kill monitoring/logging
- **Action for NeoTrix**: Adopt CRI-O for production K8s deployments where OOM resilience matters

### IMPROVEMENT 5: OCI Distribution Conformance Granularity
- Pass/Skip/Disabled/FAIL per API + per data type
- Enables partial compliance certification
- **Action for NeoTrix**: Run new conformance suite against any OCI registry integration

### IMPROVEMENT 6: Podman kube generate
- Export running Pod as Kubernetes YAML directly
- Enables dev-to-prod parity without Helm chart authoring
- **Action for NeoTrix**: Use in NT-ACT for K8s deployment generation from local pods

### IMPROVEMENT 7: Buildah --valid-exit-codes
- Accept specific non-zero exit codes as success in `buildah run`
- Enables custom build step success criteria
- **Action for NeoTrix**: Use in SEAL pipeline build steps for expected-failure handling

### IMPROVEMENT 8: Confidential Containers GA (Late 2026)
- AMD SEV-SNP and Intel TDX mature enough for production
- Enclave-based isolation for sensitive workloads
- **Action for NeoTrix**: Evaluate for NT-SHIELD high-security sandbox isolation

---

## F. Cross-Domain Defect Analysis (NeoTrix Architecture)

| Defect | NT-SHIELD | NT-ACT | NT-MIND | NT-PHYSICAL |
|--------|-----------|--------|---------|-------------|
| #2 Cache staleness | | Build pipeline | | |
| #4 SBOM migration | Audit pipeline | | | |
| #6 Rootless perms | Egress guard | | | |
| #7 CNI removal | Sandbox network | | | |
| #9 Wasm fragmentation | | Tool selection | | |
| #10 Stale stage mounts | | Build pipeline | | |

**Critical path**: Build toolchain (#2, #7, #10) → affects SEAL pipeline reliability → blocks NT-MIND evolution cycles

---

## G. Recommendations for Next Iteration

1. **Container Build Standardization**: Lock Buildah version in NT-ACT CI; pin to v1.45.0+ with cgroups v2 only
2. **OCI SBOM Pipeline**: Implement SBOM generation in build toolchain, validation at admission time
3. **Wasm Runtime Abstraction**: If adopting Wasm for NT-CORE modules, abstract behind runwasi shim interface
4. **Conformance Testing**: Run OCI distribution-spec conformance suite against any custom registry integration
5. **Rootless Security Baseline**: Enforce rootless + userns across all NT-SHIELD sandbox configurations

---

*Next: Batch 723 — Research merge queue solutions (Mergify, bors, GitHub native), feature flag systems (LaunchDarkly, Unleash, Flagsmith), and trunk-based dev tooling*
