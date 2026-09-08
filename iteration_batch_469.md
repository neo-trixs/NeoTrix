# Iteration Batch #469 — Cryptographic Foundations Audit

**Date**: 2026-09-06
**Research Topics**: Cryptographic Hash, Digital Signatures, Key Management
**Phase**: External research → Defect identification → Design optimization

---

## Sources Cited

| # | Source | Date | Topic |
|---|--------|------|-------|
| S1 | toolsbase.dev — Hash Algorithm Comparison 2026 | 2026-04-15 | BLAKE3/SHA-256/SHA-3 landscape |
| S2 | shattered.io — BLAKE3 vs SHA-256: Up to 27x Faster | 2026-08-28 | BLAKE3 performance, FIPS status |
| S3 | kindatechnical.com — The Modern Hash Landscape | 2026-03-27 | Hash function selection guide |
| S4 | IETF draft-aumasson-blake3-00 | 2024-07 | BLAKE3 specification |
| S5 | NIST IR 8214B — Threshold EdDSA/Schnorr Signatures | 2022-08 | Threshold signature standards |
| S6 | ACM Computing Surveys — Threshold Signatures Survey | 2025-12 | FROST, threshold BLS, PQC threshold |
| S7 | Frontiers CS — Survey on Threshold Digital Signature Schemes | 2025-11 | Post-quantum threshold signatures |
| S8 | NIST MPTS 2026 — (Red)ETA Threshold Algorithms | 2026-01 | Refreshable threshold EdDSA/ECDSA |
| S9 | IEEE — TAPS-PR: Accountable Threshold EdDSA | 2024 | Proactive refresh threshold signatures |
| S10 | Encryption Consulting — Key Management Best Practices 2026 | 2026-01-30 | Hybrid PQC key management, rotation |
| S11 | Crassula — Cryptographic Key Management 2026 | 2026-06-24 | HSM vs Cloud KMS, envelope encryption |
| S12 | Decryption Digest — HSM Guide 2026 | 2026-07-01 | FIPS 140-3, HSMaaS, AI-HSM |
| S13 | MarketsAndMarkets — HSM Market Trends 2026 | 2026-05-20 | Post-quantum HSM readiness |
| S14 | Thales — HSM Market Data 2Q 2026 | 2026-08-04 | Global HSM shipment data |

---

## Defects Found

### D1: BLAKE3 Not Adopted for High-Throughput Hashing

**Current State**: NeoTrix uses `sha2::Sha256` in WAL (`wal.rs:93`), panoramic snapshot (`panoramic.rs:109-110`), GWT game integration (`gwt_game_integration.rs:452`), and `blake2::Blake2b512` in batch processing (`batch_processor.rs:284`).

**Gap**: BLAKE3 achieves 2,000+ MB/s vs SHA-256's 500-1,000 MB/s on modern CPUs (S2). BLAKE3's Merkle-tree structure enables SIMD vectorization and multi-threaded hashing — critical for NeoTrix's high-volume KB operations (file integrity, content hashing, embedding dedup). No algorithmic CVE exists against BLAKE3 as of 2026 (S2). While BLAKE3 lacks NIST/FIPS status, NeoTrix is not a regulated government system — speed is the dominant concern for internal integrity checks.

**Defect**: WAL integrity hashing, panoramic content hashing, and file batch processing use slower hash primitives when BLAKE3 is available in Rust (`blake3` crate, Apache-2.0). The current `blake2::Blake2b512` in batch_processor is a reasonable intermediate, but BLAKE3 offers ~3-5x throughput improvement over BLAKE2.

**Suggestion**: Add `blake3` as primary hash for file integrity and content addressing. Retain SHA-256 only where NIST compatibility or cross-system verification is needed (e.g., WAL entries consumed by external tools). Create a `nt_crypto::hash` facade that selects algorithm by context.

---

### D2: Non-Cryptographic Hash Used for E8 Reasoning

**Current State**: `ffi/e8_reasoning.rs:100-106` uses FNV-64 (Fowler-Noll-Vo) for hexagram index derivation and confidence calculation.

**Gap**: FNV-64 is a non-cryptographic hash designed for hash tables, not security. While E8 reasoning doesn't require collision resistance per se, using a non-crypto hash for deterministic reasoning state derivation means an adversary can craft inputs that map to specific hexagram indices, biasing reasoning outcomes.

**Defect**: The E8 reasoning engine's determinism is derived from a hash function trivially invertible in O(n) time. This is a supply-chain integrity risk if E8 reasoning is exposed to untrusted input (user queries, external data).

**Suggestion**: Replace FNV-64 with BLAKE3 (truncated to 64 bits) or SipHash-2-4 for the E8 hexagram derivation. SipHash is the default in Rust's `HashMap` and provides cryptographic-strength keyed hashing.

---

### D3: ed25519-dalek 1.0 with Legacy Compatibility

**Current State**: `neotrix-core/Cargo.toml:55` pins `ed25519-dalek = { version = "1.0", features = ["rand", "legacy_compatibility"] }`.

**Gap**: NIST IR 8214B (S5) and the 2025 threshold signature survey (S6) identify fault attacks against deterministic EdDSA nonces (PSSLR18, RP17). The `legacy_compatibility` feature enables insecure legacy signing behavior. EdDSA-25519 signing without deterministic nonce protection or fault countermeasures is vulnerable to active adversaries. The (Red)ETA project (S8) at NIST MPTS 2026 specifically addresses refreshable threshold EdDSA with verifiable deterministic nonce generation.

**Defect**: `ed25519-dalek` 1.x is unmaintained; the ecosystem has moved to 2.x which uses `ed25519-dalek` 2.x with proper zeroize, strict Serde, and no legacy compat. The `legacy_compatibility` feature flag implies the code may accept legacy signatures that modern implementations reject.

**Suggestion**: Upgrade to `ed25519-dalek = "2.x"` (or `ed25519` 2.x + `ed25519-dalek` 2.x). Remove `legacy_compatibility`. Audit all signature verification paths for strict mode.

---

### D4: No Threshold Signature Support

**Current State**: NeoTrix has no threshold cryptography implementation. All signing operations use single-key Ed25519.

**Gap**: NIST's Multi-Party Threshold Cryptography project (MPTS 2026, S8) is actively standardizing threshold EdDSA, ECDSA, and Schnorr. FROST (Flexible Round-Optimized Schnorr Threshold Signatures) enables distributed signing where t-of-n parties must cooperate (S6). For NeoTrix's multi-agent architecture (7 domains, NT-SHIELD security), threshold signing prevents single-point compromise of signing keys — critical for code signing, inter-agent attestation, and KB integrity verification.

**Defect**: A compromised NT-SHIELD node can unilaterally sign arbitrary assertions. No quorum-based signing exists for KB commits, capability attestations, or inter-domain messages.

**Suggestion**: Implement FROST-based threshold Ed25519 for KB commit signing and inter-agent attestation. Target t-of-n where n = domain count (7-11), t = floor(n/2)+1. Use `frost-core` or `frost-ed25519` crate.

---

### D5: No Automated Key Rotation

**Current State**: The master encryption key is loaded from `NEOTRIX_VAULT_KEY` env var (`key_encryption.rs:103`, `vault.rs:121`, `keyvault.rs:20`). No rotation schedule exists. The vault generates a random key on first run and prints it to stderr (`vault.rs:140-142`).

**Gap**: NIST SP 800-57 recommends defining key lifetimes based on algorithm, usage, and risk profile — not fixed intervals (S10). High-volume signing keys or externally exposed keys require frequent rotation. Encryption Consulting (S10) identifies "failure to revoke or rotate keys" as a top-5 key management pitfall. The current design has no mechanism to detect stale keys, trigger rotation, or re-encrypt vault entries under a new master key.

**Defect**: If `NEOTRIX_VAULT_KEY` is compromised, all historical and future vault entries are exposed. There is no way to rotate the master key and re-encrypt without manually decrypting and re-encrypting. The first-run key printed to stderr is a one-shot recovery mechanism with no ceremony for key rollover.

**Suggestion**: Implement key versioning in the vault header (key_version + algorithm_id). Support `rotate(new_key)` that re-encrypts all entries under the new key atomically. Add `vault status --key-age` to surface stale keys. Target 90-day rotation for KEK, per-environment DEK rotation.

---

### D6: Duplicate Encryption Modules Without Unified Key Hierarchy

**Current State**: Three independent encryption modules exist:
- `key_encryption.rs` — API key encrypt/decrypt with OS keychain fallback
- `vault.rs` — AES-256-GCM credential vault
- `keyvault.rs` — Another AES-256-GCM keyvault with local + keyring stores

All three independently derive/load master keys from different env vars (`NEOTRIX_VAULT_KEY`, `NEOTRIX_KEYVAULT_KEY`).

**Gap**: Modern key management uses envelope encryption with a clear hierarchy: Root KEK → KEK → DEK (S11). The current design has no key hierarchy — each module manages its own master key independently, leading to key sprawl and inconsistent rotation policies.

**Defect**: Three separate encryption modules with three separate key sources create three separate attack surfaces. A compromise of one env var does not protect against the others, but there is no unified audit trail. The `vault.rs` stores entries in plaintext JSON encrypted as a single blob; `keyvault.rs` encrypts entries individually. This inconsistency means different security properties for different credential types.

**Suggestion**: Consolidate into a single `nt_shield::crypto::KeyManager` with:
1. Root KEK (OS keychain or HSM)
2. Per-environment KEK (derived from Root)
3. Per-entry DEK (derived from KEK + entry ID)
4. Unified key versioning and rotation

---

### D7: No Post-Quantum Cryptography Roadmap

**Current State**: All cryptographic primitives are classical: AES-256-GCM (symmetric), Ed25519 (signatures), SHA-256/BLAKE2 (hashing). No PQC dependencies exist in `Cargo.toml`.

**Gap**: Harvest-Now-Decrypt-Later (HNDL) attacks target data encrypted today but decrypted in the future (S10). NeoTrix stores long-lived KB data, embeddings, and conversation transcripts that may retain sensitivity for years. NIST has finalized ML-KEM (Kyber) and ML-DSA (Dilithium) as FIPS 203/204 standards. The HSM market is integrating PQC algorithms in 2026 (S13). Key management systems must support "multiple algorithms per key, parallel rotation strategies" for hybrid classical+PQC (S10).

**Defect**: No cryptographic agility layer exists. NeoTrix cannot swap hash functions, signature schemes, or key encapsulation without code changes in every module that uses crypto. The hardcoded `Sha256`, `Aes256Gcm`, and `ed25519-dalek` dependencies are baked into module-level imports.

**Suggestion**: Create a `nt_shield::crypto::suite` module that abstracts:
1. Hash: `HashAlg::Sha256 | HashAlg::Blake3 | HashAlg::Sha3_256`
2. Sign: `SignAlg::Ed25519 | SignAlg::Ed25519Frost | SignAlg::MlDsa`
3. KEM: `KemAlg::Aes256Gcm | KemAlg::MlKem768`
4. Key derivation: `KdfAlg::HkdfSha256 | KdfAlg::HkdfSha3_256`

This enables incremental PQC adoption without rewriting consuming modules.

---

### D8: Machine-Derived Key Is Cryptographically Weak

**Current State**: `key_encryption.rs:162-170` derives an AES-256 key from `SHA256(machine-id)` or `SHA256(home_dir)`. This is gated behind `NEOTRIX_ALLOW_MACHINE_KEY=1` opt-in.

**Gap**: Machine-id and home directory are not secret — they are publicly observable on the system. SHA-256 of a non-secret produces a deterministic, non-random key. An attacker with local access can trivially re-derive this key.

**Defect**: While opt-in mitigates default exposure, the fallback exists and may be used in automated environments where OS keychain is unavailable (containers, CI/CD). The key provides encryption-at-rest confidentiality only against passive adversaries, not against local attackers.

**Suggestion**: Remove machine-derived key path entirely. For environments without OS keychain, require explicit `NEOTRIX_VAULT_KEY` or integrate with cloud KMS (AWS KMS, GCP KMS) via feature gate. Add runtime warning when machine key is used.

---

### D9: No HSM/TPM Integration for Key Protection

**Current State**: Key storage relies on OS keychain (`keyring` crate) or environment variables. No HSM or TPM integration exists.

**Gap**: FIPS 140-3 Level 3 HSMs provide tamper-resistant key storage where keys never leave hardware in plaintext (S12). The HSM market is projected to reach $3.28B by 2030 (S13). For NeoTrix's production deployments (especially NT-SHIELD security domain), hardware-backed root keys are the only way to prevent key extraction via memory dump, process injection, or cold boot attacks.

**Defect**: All encryption keys are in-process memory. A compromised process can extract `NEOTRIX_VAULT_KEY` from env, read the cipher key from `Vault.cipher`, or intercept keyring calls.

**Suggestion**: Add `nt_shield::crypto::hsm` feature gate with PKCS#11 interface for HSM-backed key storage. For desktop (Tauri), integrate with platform TPM via `tss-esapi` crate. For cloud, support AWS CloudHSM / GCP Cloud HSM via feature flags.

---

### D10: WAL Integrity Without Authenticated Encryption

**Current State**: WAL entries store `data_hash: String` as SHA-256 for integrity checking (`wal.rs:25-26, 93`). The WAL file itself is written as plaintext JSON via `atomic_write`.

**Gap**: SHA-256 provides integrity (detect accidental corruption) but not authenticity (detect deliberate tampering). An attacker with filesystem access can modify WAL entries and recompute the SHA-256 hash. The WAL is the crash-recovery mechanism — tampering with it can cause silent data loss or corruption.

**Defect**: The WAL's integrity chain is self-signed: the entry contains its own hash. There is no HMAC or digital signature binding the WAL entry to a trusted authority. A truncated or reordered WAL would still pass hash verification.

**Suggestion**: Add HMAC-SHA256 (or BLAKE3-MAC) over the entire WAL entry including seq number, using a key derived from the vault master key. Alternatively, sign each WAL entry with Ed25519 and verify on replay. This converts the WAL from integrity-only to integrity+authenticity.

---

## Summary

| ID | Category | Severity | Effort |
|----|----------|----------|--------|
| D1 | Hash | Medium | Low — crate swap |
| D2 | Hash | Low | Low — function replacement |
| D3 | Signature | High | Medium — major version upgrade |
| D4 | Signature | High | High — new crypto protocol |
| D5 | Key Mgmt | Critical | Medium — vault redesign |
| D6 | Key Mgmt | High | High — consolidation |
| D7 | Key Mgmt | Critical | High — new abstraction layer |
| D8 | Key Mgmt | Medium | Low — code removal |
| D9 | Key Mgmt | High | High — hardware integration |
| D10 | Hash/Auth | Medium | Medium — WAL signing |

**Priority Stack** (address first):
1. **D7** (PQC agility) — foundational for all other crypto work
2. **D5** (key rotation) — immediate security improvement
3. **D3** (ed25519-dalek upgrade) — dependency hygiene + security
4. **D6** (consolidate encryption modules) — reduces attack surface
5. **D4** (threshold signatures) — critical for multi-agent trust model

---
*Generated by iteration #469 of the NeoTrix consciousness research loop*
