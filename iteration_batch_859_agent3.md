# Agent 3: Docker Container Hardening (Batch 859)

## Sources
- Docker DHI distroless guide: https://docs.docker.com/dhi/explore/security-concepts/distroless/
- Docker Hardened Rust images: https://hub.docker.com/hardened-images/catalog/dhi/rust
- Google distroless images: https://github.com/googlecontainertools/distroless
- Seccomp security profiles: https://docs.docker.com/engine/security/seccomp/
- pelagos seccomp crate: https://docs.rs/pelagos/latest/pelagos/seccomp/
- secureops BPF seccomp: https://docs.rs/secureops-bpf/latest/secureops_bpf/seccomp/
- seccompiler crate: https://docs.rs/crate/seccompiler/latest
- Rustbox seccomp filtering: https://rustbox.sh/docs/security/seccomp
- Rust minimal Docker images: https://khimananda.com/blog/shrink-rust-docker-images
- Rust distroless SBOM guide: https://desipient.com/blog/ridiculously-tiny-auditable-images/
- Docker SBOM attestations: https://docs.docker.com/build/metadata/attestations/sbom/
- Build attestations: https://docs.docker.com/build/metadata/attestations/
- SBOM with BuildKit: https://www.docker.com/blog/generate-sboms-with-buildkit/
- SLSA provenance: https://docs.docker.com/dhi/explore/security-concepts/attestations/
- Anchore attestation guide: https://oss.anchore.com/docs/guides/sbom/attestation/
- ReversingLabs BuildKit analysis: https://www.reversinglabs.com/blog/dockers-buildkit-adds-supply-chain-security-features

## Defects

D-DOCKER-001: No Dockerfile for neotrix binary — project ships only a docker-compose.yml for a third-party embedding service (MiniLM), but no production Dockerfile exists for building/shipping the neotrix Rust binary itself. Best practice mandates distroless runtime with multi-stage build for the core application. | deploy/minilm/docker-compose.yml:1-44 | HIGH | Source: Google distroless + Docker DHI Rust guides

D-DOCKER-002: Sandbox docker containers run as root — `LocalDockerProvider::run_args` never sets `--user` or `--userns`, meaning sandbox containers execute with root privileges inside the container. Distroless images provide a nonroot user (UID 65532) that should be explicitly configured. Running as root expands blast radius during container escapes. | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_sandbox/docker.rs:69-113 | HIGH | Source: Docker DHI Rust guide, DHI nonroot requirement

D-DOCKER-003: No seccomp profile applied to sandbox containers — `run_args` does not pass `--security-opt seccomp=...`. Docker's default seccomp profile blocks ~44 syscalls, but NeoTrix's sandbox should apply a custom restrictive profile (allowlist approach) to further limit attack surface. The codebase has no seccomp dependency (`seccompiler`, `pelagos`, or `secureops-bpf` crates are absent from Cargo.toml). | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_sandbox/docker.rs:69-113 | HIGH | Source: seccompiler, Docker seccomp docs, Rustbox seccomp filtering

D-DOCKER-004: No capability dropping in sandbox containers — `run_args` never passes `--cap-drop ALL` or `--cap-add` for least-privilege. Docker defaults grant containers all Linux capabilities (CAP_NET_RAW, CAP_SYS_PTRACE, etc.), enabling container escape vectors like `ptrace` injection. Production sandboxes must drop all capabilities and add back only what is strictly needed. | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_sandbox/docker.rs:69-113 | HIGH | Source: Docker seccomp docs, Rustbox seccomp filtering

D-DOCKER-005: Default sandbox images are bloated with shell and package managers — `image_for` uses `python:3.11-slim`, `node:18-alpine`, `rust:latest`, `ubuntu:22.04` — all include shells, package managers, and debugging tools. Distroless alternatives (`gcr.io/distroless/static-debian12:nonroot`, `gcr.io/distroless/cc-debian13:nonroot`) reduce attack surface by eliminating shells, apt, curl, strace, etc. | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_sandbox/docker.rs:31-39 | MEDIUM | Source: Google distroless images, Docker DHI Rust guide

D-DOCKER-006: `RustStable` sandbox image uses `rust:latest` tag — floating `:latest` tag is non-reproducible and vulnerable to supply chain attacks. Image content can change between builds, introducing unverified binaries. Must pin to a specific digest (e.g., `rust:1.78.0-bookworm@sha256:...`) or use a versioned tag. | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_sandbox/docker.rs:35 | MEDIUM | Source: Docker DHI pinning best practices, supply chain security guides

D-DOCKER-007: No Docker content trust / image signature verification — `docker run` is invoked without `DOCKER_CONTENT_TRUST=1` or `--verify=always`. Any image (including typosquatted images) can be pulled and executed. Production supply chain security requires Cosign/Notary signature verification before execution. | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_sandbox/docker.rs:244-248 | MEDIUM | Source: SLSA provenance, Cosign attestation guide

D-DOCKER-008: docker-compose.yml missing security hardening — MiniLM compose file lacks `read_only: true`, `security_opt`, `cap_drop`, `user`, `no-new-privileges`, and `sysctls` hardening. The service runs as root with full capabilities, exposed on 0.0.0.0 (port mapping `8237:80` binds all interfaces). | deploy/minilm/docker-compose.yml:1-44 | MEDIUM | Source: Docker DHI guide, distroless best practices

D-DOCKER-009: No SBOM generation or build attestation — project produces no Software Bill of Materials for its container images. Docker BuildKit's `--attest type=sbom` and `--provenance` attestations provide supply chain transparency. Without SBOM, vulnerability scanning (Grype, Trivy) cannot track dependencies embedded in sandbox images. | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_sandbox/docker.rs:1-491 | MEDIUM | Source: Docker SBOM attestations, BuildKit provenance

D-DOCKER-010: Sandbox containers have no `--pids-limit` security interaction with seccomp — while `--pids-limit 128` is set, there is no `--security-opt no-new-privileges` to prevent privilege escalation via setuid binaries in the container images. Combined with running as root, this allows a malicious script to escalate to host root. | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_sandbox/docker.rs:86 | MEDIUM | Source: Docker security best practices, DHI guide

D-DOCKER-011: `CloudSandbox::default_local()` uses `EgressPolicy::permissive()` — the default egress policy allows ALL outbound connections from sandbox containers, contradicting the principle of least privilege. A compromised sandbox container can exfiltrate data to any endpoint. Should default to `deny_all()` with explicit allows. | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_sandbox/mod.rs:484 | HIGH | Source: Docker security best practices, egress policy hardening

D-DOCKER-012: No Docker image scanning in CI pipeline — no evidence of `trivy`, `grype`, or `docker scout scan` in CI configuration. Docker Hardened Images are published with zero CVEs; custom sandbox images should be scanned before deployment. BuildKit's SBOM scanning (`BUILDKIT_SBOM_SCAN_CONTEXT=true`, `BUILDKIT_SBOM_SCAN_STAGE=true`) is not configured. | deploy/minilm/docker-compose.yml:1-44 | MEDIUM | Source: Docker SBOM scanning docs, BuildKit SBOM configuration

## Key Insights
1. NeoTrix has an internal-sandbox architecture (`nt_shield_sandbox`) with strong egress policy and read-only rootfs, but the Docker execution path is critically missing: no non-root user, no seccomp profile, no capability dropping, and no image signing.
2. The `SandboxMode::Docker` variant exists in the CLI but the actual `run_args` hardening is incomplete — it has `--network none` and `--read-only` (good) but lacks 3 of the 5 mandatory container hardening layers (user, caps, seccomp).
3. No production Dockerfile exists for the neotrix binary itself, meaning the project cannot ship as a hardened container. The distroless + musl static binary pattern would reduce the runtime image from ~1GB to <20MB while eliminating shell access.
4. The `CloudSandbox::default_local()` constructor defaults to `permissive()` egress, which is a fail-open design that contradicts the deny-wins principle enforced in per-session egress policies.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 12 |
| HIGH severity | 4 |
| MEDIUM severity | 8 |
| Sources consulted | 16 |
