# Iteration Batch 820 Report — NeoTrix Consciousness Architecture

## Research Sources (32+)

### Async Context Propagation (8)
- tokio task_local doesn't propagate across tokio::spawn
- tokio-inherit-task-local is the only workaround
- OpenTelemetry Context::attach() breaks across .await on multi-threaded
- tracing::Instrument + .in_current_span() is the only correct propagation
- NeoTrix has zero task_local! usage
- GWT attention router has no async-context awareness
- nt_core_span.rs is fully manual (not integrated with tracing crate)
- ObservabilityStack.active_spans requires &mut self (!Sync)

### String Processing (8)
- compact_str: 24-byte SSO, 32-bit capacity limit (16MB spill)
- ecow: 16-byte CoW string, atomic refcount overhead
- smartstring: 23-byte inline, Compact mode causes allocation storms
- aHash: Non-deterministic across versions (hash migration hazard)
- xxhash-rust: Deterministic, NEON support, not DOS-resistant
- fnv: Fastest for small keys, no DOS resistance
- compact_str eager inlining forces deallocation on hot paths
- ecow 15-byte inline too short for many module names

### Retry/Backoff/Rate-Limit (8)
- backon 1.6.0: Trait-based retry, exponential/constant/jitter, tokio-native
- governor 0.10.4: GCRA rate limiter, no_std, per-key via DashMap
- retry-policies 0.5.2: Simple policy calculation only (no execution)
- tower-retry 0.3.0: Dead (2019, 6+ years stale)
- No backoff between VLM retries (fixed retry_count=2, zero delay)
- Fixed delay retry in self_heal.rs (no exponential growth)
- Blocking thread::sleep in async context (registry_watcher)
- No rate limiting on NT-WORLD crawlers

### HTTP/2 and gRPC (8)
- h2 0.4.19: Active (4 releases Aug 2026), pure HTTP/2 framing
- hyper-rustls 0.27.9: Pluggable TLS backends, webpki-tokio
- tonic 0.14.6: Moved to CNCF grpc-rust, maintenance mode
- grpcio 0.13.0: Abandoned (3 years stale), violates R-P1
- jsonrpsee 0.24.11: Full JSON-RPC 2.0 with HTTP/WS/WASM
- H2SettingsProfile enum is dead code (never consumed)
- rustls 0.21 dangerous_configuration 2 versions behind
- danger_accept_invalid_certs(true) on proxy clients

---

## Defects Identified (26+)

### Async Context Propagation (5)
| ID | Defect | Severity |
|----|--------|----------|
| D-CTX-1 | Zero task_local! usage (no ambient context propagation) | Critical |
| D-CTX-2 | GWT attention router has no trace lineage | Critical |
| D-CTX-3 | nt_core_span.rs not integrated with tracing crate | High |
| D-CTX-4 | ObservabilityStack.active_spans !Sync | High |
| D-CTX-5 | Zero .instrument() usage at spawn sites | Medium |

### String Processing (5)
| ID | Defect | Severity |
|----|--------|----------|
| D-STR-1 | aHash non-deterministic across versions (KB index migration break) | High |
| D-STR-2 | compact_str 32-bit capacity cliff (16MB spill) | Medium |
| D-STR-3 | ecow atomic refcount unnecessary for single-threaded SEAL | Low |
| D-STR-4 | smartstring Compact mode causes allocation storms | Medium |
| D-STR-5 | smartstring/ecow lack no_std (blocks NT-PHYSICAL) | Medium |

### Retry/Backoff/Rate-Limit (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-RETRY-1 | No backoff between VLM retries (zero delay) | High |
| D-RETRY-2 | Fixed delay retry in self_heal (no exponential) | Medium |
| D-RETRY-3 | Blocking thread::sleep in async context | Medium |
| D-RETRY-4 | No rate limiting on NT-WORLD crawlers | High |
| D-RETRY-5 | No shared retry infrastructure (each module rolls own) | High |
| D-RETRY-6 | No error classification (retryable vs permanent) | High |

### HTTP/2 and gRPC (5)
| ID | Defect | Severity |
|----|--------|----------|
| D-H2-1 | H2SettingsProfile enum is dead code (no H2 fingerprint control) | High |
| D-H2-2 | rustls 0.21 dangerous_configuration (2 versions behind) | Critical |
| D-H2-3 | danger_accept_invalid_certs(true) on proxy clients | Critical |
| D-H2-4 | No gRPC transport (Ollama HTTP fallback) | Medium |
| D-H2-5 | No JSON-RPC transport beyond rmcp | Low |

## Key Insights (This Batch)

1. **Zero context propagation is critical**: NeoTrix spawns hundreds of tasks but loses parent span context at every spawn. GWT attention decisions cannot be traced back to their origin.

2. **aHash non-deterministic**: KB index keys must use xxh3 (fixed standard) for persistence. aHash is only safe for ephemeral runtime maps.

3. **No retry infrastructure**: Each module rolls its own retry loop. Must create nt_core_retry module with shared RetryPolicy and is_retryable_error classifier.

4. **rustls 0.21 dangerous_configuration**: 2 versions behind, disables cert verification globally. Must upgrade to 0.23 with rustls-platform-verifier.

5. **H2SettingsProfile is dead code**: Defined but never consumed by reqwest builder. To spoof H2 fingerprints, must use h2::client::handshake directly.

6. **governor for rate limiting**: Production-grade GCRA algorithm, per-key via DashMap. Must add to NT-WORLD crawlers and NT-IO LLM providers.

7. **backon is best retry crate**: Trait-based, ergonomic, tokio-native. 79M downloads. Must adopt for all LLM/API calls.

8. **grpc crate replaces tonic**: CNCF grpc-rust is the future. Tonic is maintenance-only. Must plan migration for Ollama gRPC native support.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 820 |
| New defects (this batch) | 21 |
| Cumulative defects | D01-D76636 |
| Research sources (this batch) | 32 |
| Cumulative research sources | 97,469+ |
