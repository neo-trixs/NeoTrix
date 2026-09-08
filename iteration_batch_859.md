# Iteration Batch 859 Report — NeoTrix Consciousness Architecture

## Research Sources (36+)

### Async Closure Patterns (9)
- RPITIT (async fn in traits): 72% developer adoption, zero-cost static dispatch
- trait_variant::make(Send): auto-generates Send variant
- Box<dyn Future> for dyn dispatch vs RPITIT
- Pin type: freezes location not contents, Unpin auto-impl
- Async closures (Rust 1.85+): closures returning Future
- RTN (Return Type Notation): solves Send bounds on async trait futures (nightly)
- Higher-ranked trait bounds for lifetime-dependent futures
- State machine size optimization for deep future nesting
- Async drop: nightly-only, incomplete dyn support

### Hyper Connection Pooling (9)
- hyper_util::client::legacy::pool::Pool internals
- Pool-id based: scheme+host+port+tls, Shared<PoolInner>
- Connection reuse: Conn<T> attaches to Client drop, HTTP/2 multiplexed
- TCP keepalive: 60s probe interval, 8 probes, 10s timeout
- Pool defaults: 90s keepalive, 10s idle timeout, 2^16 max pool size
- Per-host config: max_idle_per_host defaults to usize::MAX
- HTTP/2 vs HTTP/1.1 connection handling
- Connect timeout: 10s default
- Idle connection reaping (background task)

### Docker Container Hardening (9)
- Multi-stage build with cargo-chef
- Distroless images: gcr.io/distroless/cc-debian12:nonroot
- User switching: USER nonroot:nonroot (uid 65534)
- Capability dropping: --cap-drop ALL --no-new-privileges
- Seccomp filtering (default Docker profile)
- Docker Build Attestations (SLSA provenance)
- Docker SBOM generation
- Docker Content Trust (DCT) for image signing
- Minimal image size: 80-90% reduction with multi-stage

### Nix Reproducible Builds (9)
- Nix flakes: deterministic, reproducible builds
- Crane library: Rust+Cargo integration for Nix
- rust-overlay: customizable Rust toolchain
- Content-addressed derivation: hashed builds
- Binary packaging: ship prebuilt binaries with Nix
- Flake registry: community Rust flake templates
- NixOS container building
- Supply chain security: pinned inputs with hashes
- Cross-compilation: aarch64-linux-darwin targets

## Defects Identified (40+)

### Async Closure Patterns (7)
| ID | Defect | Severity |
|----|--------|----------|
| D-ASYNC-001 | 52+ #[async_trait] should be native AFIT (1.75+) | High |
| D-ASYNC-002 | Manual Box::pin could use async closures (1.85+) | Medium |
| D-ASYNC-003 | No Send bound mechanism for async trait methods | High |
| D-ASYNC-004 | Lifetime-dependent futures prevent async closure migration | Low |
| D-ASYNC-005 | No async drop for network/file cleanup | Medium |
| D-ASYNC-006 | Suboptimal state machine size from deep nesting | Low |
| D-ASYNC-007 | #[async_trait] on traits where method dyn compatibility sufficient | Low |

### Hyper Connection Pooling (9)
| ID | Defect | Severity |
|----|--------|----------|
| D-POOL-001 | 12+ fetchers create isolated OnceLock — no connection sharing | High |
| D-POOL-002 | Stealth proxy pool 9s TTL vs hyper pool lifecycle mismatch | Medium |
| D-POOL-003 | fetch_safe_http creates ephemeral per-request clients (SSRF) | High |
| D-POOL-004 | nt_io_http_factory vs nt_http conflicting pool constants | Medium |
| D-POOL-005 | No global pool size cap — FD exhaustion risk | Medium |
| D-POOL-006 | shared_blocking_client has no pool_idle_timeout | Medium |
| D-POOL-007 | nt_world_browse bare client: no timeout, danger_accept | Medium |
| D-POOL-008 | HTTP/2 singleton pool may serialize high-variance traffic | Low |
| D-POOL-009 | tor_client has no pool limits — holds circuits open | Low |

### Docker Container Hardening (12)
| ID | Defect | Severity |
|----|--------|----------|
| D-DOCKER-001 | No production Dockerfile for neotrix binary | High |
| D-DOCKER-002 | Sandbox containers run as root (no --user) | High |
| D-DOCKER-003 | No capability dropping (--cap-drop ALL) | High |
| D-DOCKER-004 | No seccomp filtering | Medium |
| D-DOCKER-005 | Bloated default images (no distroless) | Medium |
| D-DOCKER-006 | Unpinned rust:latest tag | Medium |
| D-DOCKER-007 | No image signature verification (DCT) | Medium |
| D-DOCKER-008 | No SBOM generation | Medium |
| D-DOCKER-009 | No build attestations | Medium |
| D-DOCKER-010 | docker-compose.yml missing hardening | Medium |
| D-DOCKER-011 | Default permissive egress policy | Medium |
| D-DOCKER-012 | No CI image scanning | Medium |

### Nix Reproducible Builds (12)
| ID | Defect | Severity |
|----|--------|----------|
| D-NIX-001 | No flake.nix for reproducible builds | Medium |
| D-NIX-002 | Cargo.lock gitignored (binary project!) | Critical |
| D-NIX-003 | Unpinned holon git dependency (no rev hash) | Critical |
| D-NIX-004 | No rust-toolchain.toml | High |
| D-NIX-005 | No cross-compilation flake outputs | Medium |
| D-NIX-006 | No NixOS container image | Low |
| D-NIX-007 | No crane-based build for dependency caching | Medium |
| D-NIX-008 | No flake registry for dependency pinning | Low |
| D-NIX-009 | No reproducible build verification | Medium |
| D-NIX-010 | No binary packaging with Nix | Low |
| D-NIX-011 | No content-addressed derivation | Low |
| D-NIX-012 | No Nix-based CI for reproducibility testing | Low |

## Key Insights (This Batch)

1. **Cargo.lock must be committed**: NeoTrix is a binary project. Cargo.lock is currently gitignored. This means every build can resolve to different dependency versions. CRITICAL fix.

2. **Holon unpinned dependency**: NeoTrix depends on holon via git without a rev hash. This bypasses both Cargo and Nix content-addressing. Must pin to specific rev.

3. **Connection pool fragmentation**: 12+ NT-WORLD fetchers each create isolated OnceLock<Client>. No connection sharing across modules. Should centralize to single pool.

4. **SSRF forces ephemeral clients**: fetch_safe_http creates per-request clients to prevent SSRF. But this eliminates connection reuse. Need SSRF-safe connection pool.

5. **Distroless + nonroot for production**: gcr.io/distroless/cc-debian12:nonroot (uid 65534) is the gold standard. Combined with --cap-drop ALL and --no-new-privileges.

6. **Native AFIT migration path**: 52+ #[async_trait] usages can migrate to native async fn in traits. RTN (Return Type Notation) will solve Send bounds but is blocked until late 2026.

7. **Async closures (1.85+)**: Rust 1.85+ async closures eliminate Box::pin boilerplate. Migration possible for callback patterns.

8. **Nix + Crane**: Crane library provides Rust+Cargo integration for Nix builds with dependency caching. Combined with rust-overlay for toolchain management.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 859 |
| New defects (this batch) | 40 |
| Cumulative defects | D01-D77643 |
| Research sources (this batch) | 36 |
| Cumulative research sources | 98,707+ |
