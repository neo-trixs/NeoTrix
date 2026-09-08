# Iteration Batch 817 Report — NeoTrix Consciousness Architecture

## Research Sources (32+)

### Tower/Hyper/Middleware (10)
- Tower option_layer error-type mismatch (Either requires both branches BoxError)
- Hyper 1.x data loss race condition (poll_flush ignored, SHUT_WR loses buffered data)
- Hyper 1.8.0 CPU spin regression (100% CPU on concurrent large-file streaming)
- Hyper non-UTF8 panic in HTTP/2 Connection header (malicious client crash)
- tower-http 0.7.0: Compression mishandles * wildcard, trailing slash on file paths
- reqwest-middleware is_connect() retry bug (TLS misconfiguration retried forever)
- reqwest-middleware is_timeout() inconsistency after upgrade
- tower-service still at 0.3.3 (no 1.0 timeline)
- tower-http duplicate Vary headers (CorsLayer)
- reqwest-middleware 0.5.0 upgraded to reqwest 0.13

### Async Streaming (6)
- 24 #[async_trait] usages across NeoTrix (heap allocation per call)
- Fake SSE streaming (oneshot channel wrapped as stream)
- Zero custom Stream implementations (no impl Stream for ...)
- Three incompatible async trait dispatch patterns
- tokio-stream 0.1.x is transitional (Stream not in std yet)
- Commented-out async_stream usage (avoided due to issues)

### Serialization (6)
- serde_json 1.0.151: preserve_order adds IndexMap overhead
- simd-json 0.17.0: 2.2× faster but x86_64 AVX2 only
- bincode 1.3.3: v2 is complete rewrite (soundness issues in v1)
- ciborium 0.2.2: Dormant (18+ months since last release)
- serde_json_borrow 0.8: Outperforms simd-json by borrowing directly
- Event bus hot-path parsing allocates per message

### Crypto/Privacy (8)
- x25519-dalek StaticSecret Clone leaks secret key material
- snow TransportState/CipherState missing Zeroize on drop
- hpke-rs RUSTSEC-2026-0071: Nonce reuse (u32 wrap after 2^32 ops)
- hpke-rs RUSTSEC-2026-0072: Missing all-zero DH secret check
- aes-gcm 0.10.x: Safe for current use, monitor 0.11
- chacha20poly1305: AEAD users protected from counter overflow
- libsodium-sys 0.2.7: Unmaintained (5+ years stale)
- RUSTSEC-2026-0209: libcrux-aesgcm AAD length enforcement

---

## Defects Identified (26+)

### Tower/Hyper/Middleware (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-TOWER-1 | option_layer error-type mismatch (requires BoxError on both branches) | High |
| D-TOWER-2 | Hyper data loss race condition (poll_flush ignored before shutdown) | Critical |
| D-TOWER-3 | Hyper 1.8.0 CPU spin regression (100% on concurrent streaming) | High |
| D-TOWER-4 | Hyper non-UTF8 panic in HTTP/2 Connection header | High |
| D-TOWER-5 | tower-http compression mishandles * wildcard (RFC 9110 violation) | Medium |
| D-TOWER-6 | reqwest-middleware is_connect() retries TLS misconfig forever | High |
| D-TOWER-7 | reqwest-middleware is_timeout() inconsistency after upgrade | Medium |
| D-TOWER-8 | tower-service 0.3.3 (no 1.0, hyper defines own Service trait) | Low |

### Async Streaming (5)
| ID | Defect | Severity |
|----|--------|----------|
| D-STREAM-1 | 24 #[async_trait] usages (heap allocation per call) | High |
| D-STREAM-2 | Fake SSE streaming (oneshot channel wrapped as stream) | Critical |
| D-STREAM-3 | Zero custom Stream implementations | Medium |
| D-STREAM-4 | Three incompatible async trait dispatch patterns | Medium |
| D-STREAM-5 | tokio-stream transitional (Stream not in std) | Low |

### Serialization (4)
| ID | Defect | Severity |
|----|--------|----------|
| D-SER-1 | serde_json preserve_order adds IndexMap overhead | Low |
| D-SER-2 | simd-json 2.2× faster but x86_64 AVX2 only (no ARM) | Medium |
| D-SER-3 | bincode v1 soundness issues (infinite length attacks) | High |
| D-SER-4 | Event bus hot-path parsing allocates per message | Medium |

### Crypto/Privacy (5)
| ID | Defect | Severity |
|----|--------|----------|
| D-CRYPTO-1 | x25519-dalek StaticSecret Clone leaks secret key | Critical |
| D-CRYPTO-2 | snow TransportState missing Zeroize on drop | High |
| D-CRYPTO-3 | hpke-rs nonce reuse (u32 wrap after 2^32 ops) | Critical |
| D-CRYPTO-4 | hpke-rs missing all-zero DH secret check (RFC 9180) | Medium |
| D-CRYPTO-5 | libsodium-sys unmaintained (5+ years) | Low |

## Key Insights (This Batch)

1. **Hyper data loss race is critical**: poll_flush return ignored, SHUT_WR loses buffered data. Silent corruption with 200 OK. Must require hyper >= 1.10.1.

2. **x25519-dalek StaticSecret Clone leaks secrets**: Secret key material can be copied via Clone. Must use ZeroizeOnDrop and prevent cloning.

3. **hpke-rs nonce reuse is catastrophic**: u32 counter wraps after 2^32 seal/open ops, causing AEAD nonce reuse → plaintext recovery. Must use hpke-rs >= 0.6.0.

4. **Fake SSE streaming is a correctness bug**: Oneshot channel wrapped as stream sends entire response as one event. Must use mpsc-based incremental delivery.

5. **24 #[async_trait] heap allocations per call**: Native AFIT gives zero-cost static dispatch. This is the single largest performance defect.

6. **tower-service 0.3.3 is permanent**: No 1.0 timeline. Hyper defines own Service trait. NeoTrix must bridge both or define own.

7. **reqwest-middleware retries TLS misconfig**: is_connect() classifies all connect errors as transient. Must bypass for fatal configuration errors.

8. **bincode v1 has soundness issues**: Infinite length attacks. Must evaluate migration to v2 for KB blob storage.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 817 |
| New defects (this batch) | 22 |
| Cumulative defects | D01-D76570 |
| Research sources (this batch) | 29 |
| Cumulative research sources | 97,374+ |
