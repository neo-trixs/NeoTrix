# Cryptographic Forgetting Certificate — HyperCube Design

## Motivation

### Why Forgetting Matters for NeoTrix

NeoTrix's HyperCube knowledge representation (`nt_core_hcube`) uses FHRR (Fourier Holographic Reduced Representation) VSA vectors for all knowledge encoding. Every user query, system interaction, and learned pattern accumulates into the HyperCube codebook and `nt_memory_kb` SQLite store. Without a cryptographic forgetting mechanism, three problems compound:

1. **Data Sovereignty (GDPR Right to Deletion)**: Article 17 of GDPR requires provable deletion of personal data upon request. A signed forgetting certificate provides cryptographic proof that a specific HyperCube entry has been permanently removed, satisfying regulatory audit trails.

2. **Memory Hygiene**: The HyperCube currently has no eviction-guarantee mechanism. While `dream_consolidation.rs` and pruning exist, there is no cryptographically anchored proof that a piece of knowledge was *ever* present and then *verifiably* removed. This is distinct from simple deletion — it is a commitment to the act of forgetting.

3. **Privacy-Preserving Audit**: A forgetting certificate reveals only the content hash (SHA-512 of the FHRR phase vector serialization) and the target identifier — never the actual content or the VSA vector phases. This allows third parties to verify deletion without learning what was deleted.

### Relationship to Existing Privacy Layer

The current `privacy.rs` provides `DataSovereigntyProof` — a HMAC-like hash-based proof that requires shared secret knowledge to verify. This design replaces that with a full Ed25519 asymmetric key system, JWS-formatted certificates, and chain-of-evidence via linked certificate chains. The existing `PrivacyEnforcer` modes (Stateless/Encrypted/Sovereign) integrate with the forgetting certificate subsystem as the Sovereign-mode deletion proof carrier.

## Architecture

### ForgettingCertificate Struct

```rust
/// A signed, immutable record proving a data item was forgotten.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgettingCertificate {
    /// Certificate version (currently 1).
    pub version: u8,
    /// Unique certificate identifier (UUID v4).
    pub id: String,
    /// The logical identifier of the forgotten data item
    /// (KB node ID, HyperCube symbol name, or external reference).
    pub target_id: String,
    /// SHA-512 of the canonical serialization of the forgotten FHRR vector.
    /// For content not representable as FHRR, SHA-512 of the JSON-serialized
    /// KnowledgeNode content field.
    pub content_hash: String,
    /// Unix timestamp (seconds since epoch) when forgetting occurred.
    pub timestamp: i64,
    /// The certificate chain nonce: HMAC-SHA256(prev_nonce, target_id || timestamp).
    /// Prevents replay of old certificates.
    pub nonce: String,
    /// Ed25519 signature over the canonical payload.
    pub signature: String,
    /// Ed25519 public key that signed this certificate (base64-encoded).
    pub signer_pubkey: String,
    /// Key ID identifying which rotation epoch's key was used.
    pub key_id: String,
    /// Optional human-readable reason for forgetting
    /// (e.g., "GDPR Article 17 request", "user-initiated delete", "memory hygiene").
    pub reason: Option<String>,
    /// Optional metadata (JSON blob for extensibility).
    pub metadata: Option<serde_json::Value>,
}
```

### Canonical Payload for Signing

The Ed25519 signature is computed over the following UTF-8 string, constructed deterministically:

```
FORGET:v1:{target_id}:{content_hash}:{timestamp}:{nonce}:{key_id}
```

Field order is fixed. This prevents field-swapping attacks.

### JWS (JSON Web Signature) Format

Certificates are serialized to JWS Compact Serialization for interoperability:

```json
{
  "payload": "<base64url-encoded certificate JSON>",
  "protected": {
    "alg": "EdDSA",
    "kid": "<key_id>",
    "typ": "forgetting-certificate+v1",
    "crit": ["exp"],
    "exp": 1893456000
  },
  "signature": "<base64url-encoded Ed25519 signature>"
}
```

The JWS-serialized certificate is the canonical interchange format. Internal Rust operations use the struct directly; SQLite stores the JWS compact serialization as a TEXT column.

### CertificateChain

A linked list of forgetting events for a single data item. Each new certificate references the previous one via the nonce chain:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateChain {
    /// The data item identifier.
    pub target_id: String,
    /// Most recent forgetting certificate.
    pub latest: ForgettingCertificate,
    /// Previous certificates for the same target_id, most recent first.
    pub history: Vec<ForgettingCertificate>,
}
```

The chain grows when a previously forgotten item is re-inserted and later re-forgotten. The nonce chain provides ordering:

- `nonce_0 = HMAC-SHA256("neotrix-genesis-" + target_id, target_id)`
- `nonce_i = HMAC-SHA256(nonce_{i-1}, target_id || timestamp_i)`

Verifying the chain requires checking that each nonce was correctly derived from its predecessor.

### Verification Protocol

```rust
impl ForgettingCertificate {
    /// Verify the Ed25519 signature against the trusted public key.
    pub fn verify(&self, trusted_pubkey: &[u8; 32]) -> Result<(), VerificationError> {
        // 1. Reconstruct canonical payload
        let payload = format!(
            "FORGET:v1:{}:{}:{}:{}:{}",
            self.target_id, self.content_hash, self.timestamp, self.nonce, self.key_id
        );
        // 2. Decode base64 signature
        let sig_bytes = BASE64_STANDARD.decode(&self.signature)?;
        // 3. Ed25519 verify
        let pubkey = ed25519_dalek::VerifyingKey::from_bytes(trusted_pubkey)?;
        pubkey.verify_strict(payload.as_bytes(), &sig_bytes.into())?;
        // 4. Check timestamp within clock skew (±300 seconds)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?.as_secs() as i64;
        if (self.timestamp - now).abs() > 300 {
            return Err(VerificationError::ClockSkew);
        }
        // 5. Verify nonce is well-formed
        //    (HMAC chain check requires previous cert or genesis)
        Ok(())
    }
}
```

A query path that checks for certificates:

```
1. Given a KB node ID or HyperCube symbol name
2. Look up latest certificate in SQLite `forgetting_certificates` table
3. If cert exists AND verifies AND is newer than the stored data's updated_at
   → Redact content: return node with content = None, summary = "[REDACTED]"
   → Do NOT include in FHRR similarity search results
4. If no cert, return data normally
```

## Key Rotation

### Monthly Rotation Schedule

- **Active signing key**: rotated on the 1st of each month at 00:00 UTC.
- **Key generation**: Ed25519 keypair derived from system entropy + optional HSM.
- **Key identifiers**: `k{YYYYMM}` format (e.g., `k202607` for July 2026).

### Key Lifecycle

| Phase | Duration | Behavior |
|-------|----------|----------|
| **Active** | Month 1 | Used for signing all new certificates |
| **Grace** | Month 2 | Accepts signatures from both old and new for verification |
| **Archived** | Month 3+ | Old keys retained in read-only keystore; only verification |

### HSM-Backed Signing (Primary)

```rust
pub enum SigningBackend {
    /// Software fallback — Ed25519 key in encrypted file.
    Software(Ed25519Keypair),
    /// HSM via PKCS#11 interface (YubiKey, NitroKey, TPM).
    Hsm(Pkcs11Session),
}
```

The HSM backend:
- Loads the key via `C_Login` with a PIN from `nt_shield_vault`
- Signs via `C_Sign` with `CKM_EDDSA` mechanism
- Never exposes the private key material to userspace
- Falls back to software if HSM is unavailable (configurable in `~/.config/neotrix/config.toml` under `[forgetting.hsm]`)

### Key Storage in SQLite

```sql
CREATE TABLE IF NOT EXISTS forgetting_keyring (
    key_id TEXT PRIMARY KEY,
    public_key BLOB NOT NULL,       -- 32 bytes Ed25519 public key
    encrypted_private_key BLOB,     -- AES-256-GCM encrypted private key (software only)
    activated_at INTEGER NOT NULL,   -- unix timestamp
    retired_at INTEGER,             -- NULL if still active
    hsm_key_id TEXT,                -- CKA_LABEL in HSM, NULL for software
    metadata TEXT
);
```

### Verification-Only Key Retention

When a key is rotated, the old public key remains in the `forgetting_keyring` table with `retired_at` set. The verification function checks all non-retired keys first; if none match, it falls back to retired keys. Retired keys can be physically destroyed after 3 months via `cleanup_expired_keys()`.

## Integration with nt_memory_kb

### DDL: forgetting_certificates Table

```sql
-- Schema addition to nt_memory_schema.rs

CREATE TABLE IF NOT EXISTS forgetting_certificates (
    id TEXT PRIMARY KEY,                          -- UUID v4
    target_id TEXT NOT NULL,                       -- KB node ID or HyperCube symbol
    target_type TEXT NOT NULL DEFAULT 'kb_node',   -- 'kb_node', 'hc_symbol', 'cold_blob'
    content_hash TEXT NOT NULL,                    -- SHA-512 hex
    timestamp INTEGER NOT NULL,                    -- unix seconds
    nonce TEXT NOT NULL,                           -- HMAC chain nonce (hex)
    signature TEXT NOT NULL,                       -- Ed25519 sig (base64)
    signer_pubkey TEXT NOT NULL,                   -- base64
    key_id TEXT NOT NULL,                          -- e.g. 'k202607'
    reason TEXT,
    jws_compact TEXT NOT NULL,                     -- Full JWS compact serialization
    metadata TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_forgetting_cert_target
    ON forgetting_certificates(target_id, timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_forgetting_cert_key
    ON forgetting_certificates(key_id);

CREATE INDEX IF NOT EXISTS idx_forgetting_cert_hash
    ON forgetting_certificates(content_hash);
```

### Schema Version Bump

The `SCHEMA_VERSION` constant in `nt_memory_schema.rs` increments to `2`. Migration:
```sql
INSERT OR REPLACE INTO schema_version (version) VALUES (2);
```

### KnowledgeBase Integration Methods

```rust
impl KnowledgeBase {
    /// Issue a forgetting certificate for a KB node and redact its content.
    pub fn forget_node(
        &self,
        node_id: &str,
        reason: Option<&str>,
    ) -> Result<ForgettingCertificate, String> {
        // 1. Retrieve node and compute content hash
        // 2. Generate nonce from chain (or genesis)
        // 3. Sign with active key (via nt_shield_vault or HSM)
        // 4. Store cert in forgetting_certificates table
        // 5. Redact the node: set content = NULL, summary = "[REDACTED]"
        // 6. Nullify the FHRR vector in the HyperCube codebook
        // 7. If cold store blob exists, delete it
        // 8. Return the JWS certificate
    }

    /// Verify that a node has been forgotten.
    pub fn is_forgotten(&self, node_id: &str) -> Result<bool, String> {
        // Check most recent cert: exists, signature valid, key trusted
    }

    /// Get the forgetting certificate chain for a node.
    pub fn get_forgetting_chain(
        &self, node_id: &str,
    ) -> Result<Option<CertificateChain>, String> {
        // Query all certs for target_id, ordered by timestamp DESC
    }

    /// Export all forgetting certificates for audit.
    pub fn export_forgetting_audit(
        &self, since: Option<i64>,
    ) -> Result<Vec<ForgettingCertificate>, String> {
        // Return all certs (optionally since a timestamp) as JWS array
    }
}
```

### Query Redaction Path

The critical integration point is the `get_node()` / `search()` / `semantic_search()` path:

```
get_node(id):
  1. node = query nodes table
  2. cert = query latest forgetting_certificate for id
  3. if cert exists AND cert.timestamp >= node.updated_at:
     → return KnowledgeNode { content: None, summary: "[REDACTED]", ... }
  4. return node

semantic_search(query):
  1. results = hybrid_rerank_search(query, limit)
  2. filter: remove any result where is_forgotten(id) == true
  3. return filtered results
```

This ensures forgotten data cannot leak through FTS5, BM25, or vector similarity search. The FTS index entry remains (so we don't rebuild the entire index on forget), but the returned content is always `[REDACTED]`.

## Cold Storage Tier

### Architecture

FHRR VSA vectors at D=8192 consume 64 KiB per vector (f64 × 8192). At scale, the HyperCube codebook can exceed available RAM. A cold storage tier provides:

- **Hot**: In-memory `FhrrHyperCube` codebook (HashMap<String, Vec<f64>>)
- **Cold**: NVMe-backed binary files at `~/.neotrix/cold_store/{symbol_name}.fhrr`
- **Freezing**: LRU eviction from hot → cold when codebook exceeds `max_hot_entries`

### Directory Layout

```
~/.neotrix/cold_store/
├── INDEX            # JSON manifest: frozen symbol names + metadata
├── 3f8a...c1.fhrr   # Binary: dimension (u64 LE) + N × f64 (phase angles)
├── a2b1...7e.fhrr   # Each file is AES-256-GCM encrypted
└── ...
```

### LRU Eviction Policy

```rust
pub struct ColdStore {
    /// Directory path.
    path: PathBuf,
    /// Maximum hot codebook entries before eviction triggers.
    max_hot_entries: usize,
    /// LRU tracker: symbol name → last access timestamp.
    lru_tracker: Mutex<LinkedHashMap<String, Instant>>,
    /// AES-256-GCM key derived from nt_shield_vault master key.
    encryption_key: [u8; 32],
}
```

Eviction trigger: when codebook size exceeds `max_hot_entries`, the least recently accessed entry (by `get_symbol()`) is:
1. Serialized to binary (dimension as u64 LE, followed by `dim` f64 phase angles)
2. Encrypted with AES-256-GCM using a random 12-byte nonce
3. Written to `cold_store/{hash}.fhrr`
4. Removed from the in-memory codebook
5. INDEX file updated

Thaw occurs on cache miss: `get_symbol()` checks cold store, decrypts, loads back into hot codebook, and updates LRU.

### Forgetting and Cold Store

When `forget_node()` is called:
1. Unload the symbol from hot codebook (if present)
2. Delete the cold store file (if present): `std::fs::remove_file(...)`
3. Overwrite with zeros before deletion (optionally, for NVMe secure erase):
   ```rust
   use std::io::Write;
   let mut f = std::fs::File::create(&path)?;
   let zeros = vec![0u8; file_size];
   f.write_all(&zeros)?;
   f.sync_all()?;
   std::fs::remove_file(&path)?;
   ```
4. Update INDEX
5. Issue forgetting certificate

The certificate's `content_hash` covers both the hot codebook vector's SHA-512 and the cold blob's SHA-512, ensuring both are provably destroyed.

### Cold Store Config

```toml
[forgetting.cold_store]
enabled = true
path = "~/.neotrix/cold_store"
max_hot_entries = 10000
aes_gcm_key_derivation = "hkdf-sha256"  # derived from vault/master
zero_overwrite_passes = 1               # 0 = skip overwrite
```

## Security Considerations

### Replay Attacks (Nonce Chain)

Each forgetting certificate carries a nonce derived from the previous certificate's nonce via HMAC-SHA256:

```
nonce_i = HMAC-SHA256(key=nonce_{i-1}, data=target_id || timestamp_i)
```

An attacker who obtains an old certificate cannot replay it because:
- The nonce chain would not match the current chain state
- The timestamp (checked within ±5 min) would be stale
- The `target_id || timestamp` binding prevents cross-target nonce reuse

For the first certificate of a target (genesis):
```
nonce_0 = HMAC-SHA256(key="neotrix-forgetting-genesis", data=target_id)
```

### Key Compromise Recovery

If a signing key is compromised:

1. **Revoke**: Insert a `KeyCompromiseRecord` into `forgetting_keyring` with `retired_at = now`.
2. **Rotate**: Generate a new key (new `key_id`).
3. **Re-sign**: Optionally re-issue all certificates under the new key (bulk operation).
4. **Audit**: The chain shows the transition; old certificates signed by the compromised key are still valid for verification but flagged with `compromised_key=true`.

```sql
CREATE TABLE IF NOT EXISTS key_compromise_log (
    key_id TEXT PRIMARY KEY,
    compromised_at INTEGER NOT NULL,
    detection_method TEXT NOT NULL,   -- 'hsm_attestation', 'manual_report', 'automated_scan'
    rotated_to_key_id TEXT NOT NULL,
    affected_cert_count INTEGER DEFAULT 0,
    mitigated_at INTEGER,
    notes TEXT
);
```

### Clock Skew Tolerance

Timestamps within certificates are compared to the verifying system's clock with a ±300-second (5 minute) tolerance. This accommodates NTP drift across distributed deployments. Certificates outside this window return `VerificationError::ClockSkew`.

The tolerance is a single constant:
```rust
pub const FORGETTING_CLOCK_SKEW_TOLERANCE_SECS: i64 = 300;
```

For high-security deployments, this can be tightened to 60 seconds.

### Privacy: Certificate Reveals Only Hash, Not Content

The forgetting certificate contains:
- `target_id`: Logical identifier (KB node ID) — reveals that *something* was forgotten at this address
- `content_hash`: SHA-512 of the FHRR phase vector — computationally infeasible to reverse
- `timestamp`, `nonce`, `signature`: metadata only

The certificate proves:
1. A data item existed at `target_id` (by reference to the hash chain)
2. It was deleted at `timestamp`
3. The deletion was authorized (by Ed25519 signature)

It does **not** reveal:
- The actual FHRR phase angles (the knowledge content)
- The summary, title, or any semantic content
- Any relation edges pointing to the forgotten node (those are independently deleted)

This satisfies "proof of deletion without disclosure" — a core GDPR audit requirement.

## Implementation Plan

### Phase 1: Data Structures + Signing (3 days)

| Day | Task | Files |
|-----|------|-------|
| 1 | `ForgettingCertificate`, `CertificateChain`, `VerificationError` structs | `neotrix-core/src/core/nt_core_hcube/forgetting.rs` |
| 1 | Canonical payload construction + Ed25519 signing/verification (ed25519-dalek) | same |
| 2 | Nonce chain derivation (HMAC-SHA256 via hmac crate) | same |
| 2 | JWS serialization/deserialization (base64url, protected header) | `neotrix-core/src/core/nt_core_hcube/jws.rs` |
| 3 | Unit tests: sign → verify roundtrip, tampered sig, nonce chain, JWS parse | `forgetting.rs` tests |

**Dependencies**: `ed25519-dalek`, `hmac`, `sha2`, `base64` (already in workspace).

### Phase 2: SQLite Integration + Query Redaction (2 days)

| Day | Task | Files |
|-----|------|-------|
| 1 | DDL migration (`forgetting_certificates` + `forgetting_keyring` tables) | `nt_memory_schema.rs` |
| 1 | `KnowledgeBase::forget_node()`, `is_forgotten()`, `get_forgetting_chain()` | `nt_memory_kb/mod.rs` |
| 2 | Query redaction in `get_node()` / `search()` / `semantic_search()` | `nt_memory_kb/mod.rs`, `nt_memory_search.rs` |
| 2 | Key lifecycle: `rotate_keys()`, `cleanup_expired_keys()` | `forgetting.rs`, `nt_memory_kb/mod.rs` |

### Phase 3: Cold Storage Tier (2 days)

| Day | Task | Files |
|-----|------|-------|
| 1 | `ColdStore` struct: LRU tracking, binary serialization/deserialization | `neotrix-core/src/core/nt_core_hcube/cold_store.rs` |
| 1 | AES-256-GCM encryption/decryption at rest | same |
| 2 | Hot↔cold eviction integrated with `FhrrHyperCube::get_symbol()` | `fhrr_vsa.rs` (modify `get_symbol`) |
| 2 | `forget_node()` deletes cold blob + zero-overwrite | `cold_store.rs` |

### Phase 4: Key Rotation + HSM Integration (2 days)

| Day | Task | Files |
|------|------|-------|
| 1 | `SigningBackend` enum (Software / Hsm), PKCS#11 binding | `forgetting.rs`, `nt_shield_vault/mod.rs` |
| 1 | Scheduler: monthly `rotate_keys()` cron check | `nt_mind_sleep.rs` or standalone tick |
| 2 | `key_compromise_log` table, revocation protocol | `forgetting.rs`, `nt_memory_schema.rs` |
| 2 | Integration test: full lifecycle (create → forget → verify → audit export) | `tests/forgetting_e2e.rs` |

### Total: 9 days

### Post-MVP (Future)

- **Distributed verification network**: Other NeoTrix instances can query certificates via MCP tool `forgetting_verify(target_id, certificate)`.
- **Transparency log**: Append-only CT-style log of all forgetting certificates, gossiped via libp2p.
- **Batch forgetting with Merkle aggregation**: Prove N deletions with O(log N) certificate size.

## Appendix: Example Certificate

### JWS Compact Serialization

```
eyJhbGciOiJFZERTQSIsImtpZCI6ImsyMDI2MDciLCJ0eXAiOiJmb3JnZXR0aW5nLWNlcnRpZmljYXRlK3YxIiwiY3JpdCI6WyJleHAiXSwiZXhwIjoxODkzNDU2MDAwfQ.
eyJ2ZXJzaW9uIjoxLCJpZCI6IjZmOTQ2N2IxLWFhM2MtNDc5MS1iZGNiLTk3NzFiN2E1N2RlYyIsInRhcmdldF9pZCI6Im5vZGVfYTNjMjg0ZjktODk3Mi00YTY5LWJjZjgtNjZlMzVlMjkzY2M5IiwiY29udGVudF9oYXNoIjoiZTNiMGE3NjU3OTgxYjA3ZDFjMDRmYzhhZmRkYTJmODRlYjQ0OTkwMmM5NTU0ODc3NmI5NmQ2M2I1NDZhNTQ1Y2RmZjA5Y2YzYzU1NjcwYjJhZjU4NTliMmYzMGQ4MDA3YjIwYzcxMjVlYzEyYWEzYjUzZTUyN2E1Yzg3OTI3Iiwi
dGltZXN0YW1wIjoxNzgyMTU2ODAwLCJub25jZSI6ImE3ZjNjZGI4MjkwN2IxYzU0ZWNmOTQxNGJjYjM5ODA3YzU5MmI3YmI4ZTc3YzY5YTI2Mjk4ZjRiYzcyODdhMzciLCJzaWduYXR1cmUiOiJmMDFhMjNiYzQ1ZDY3ODkwYWFkY2RlZjAxMjM0NTY3ODkwYWJjZGVmMDEyMzQ1Njc4OTBhYmNkZWYwMTIzNDU2Nzg5MGFiY2RlZjAxMjM0NTY3ODkwYWJjZGVmMDEyMzQ1NjciLCJzaWduZXJfcHVia2V5IjoiZkM2NzQ4OTBhYmNkZWYwMTIzNDU2Nzg5MGFiY2RlZjAxMjM0NTY3ODkwYWJjZGVmMDEyMzQ1Njc4OTBhYmNkIiwia2V5X2lkIjoiazIwMjYwNyIsInJlYXNvbiI6IkdkcHIgUmVxdWVzdCAtIFVzZXIgRGF0YSBEZWxldGlvbiJ9
.ZWQ1NTk2ZjM4YzI3YjE0NTc4OTBhYmNkZWYwMTIzNDU2Nzg5MGFiY2RlZjAxMjM0NTY3ODkwYWJjZGVmMDEyMzQ1Njc4OTBhYmNkZWYwMTIzNDU2Nzg5MGFiY2RlZjAxMjM0NTY3ODkwYWJjZGVmMDEyMzQ1Njc4OTA=
```

### Decoded Protected Header

```json
{
  "alg": "EdDSA",
  "kid": "k202607",
  "typ": "forgetting-certificate+v1",
  "crit": ["exp"],
  "exp": 1893456000
}
```

### Decoded Payload

```json
{
  "version": 1,
  "id": "6f9467b1-aa3c-4791-bdcb-9771b7a57dec",
  "target_id": "node_a3c284f9-8972-4a69-bcf8-66e35e293cc9",
  "content_hash": "e3b0a7657981b07d1c04fc8afdda2f84eb449902c95548776b96d63b546a545cdff09cf3c55670b2af5859b2f30d8007b20c7125ec12aa3b53e527a5c87927",
  "timestamp": 1782156800,
  "nonce": "a7f3cdb82907b1c54ecf9414bcb39807c592b7bb8e77c69a26298f4bc7287a37",
  "signature": "f01a23bc45d67890aadcdef01234567890abcdef01234567890abcdef01234567890abcdef01234567890abcdef0123456",
  "signer_pubkey": "fC674890abcdef01234567890abcdef01234567890abcdef01234567890abcd",
  "key_id": "k202607",
  "reason": "Gdpr Request - User Data Deletion"
}
```

### Rust Verification Example

```rust
// Load the JWS string from SQLite
let jws = db.query_row(
    "SELECT jws_compact FROM forgetting_certificates WHERE target_id = ?1",
    [node_id],
    |row| row.get::<_, String>(0),
)?;
let cert: ForgettingCertificate = ForgettingCertificate::from_jws(&jws)?;
let trusted_pubkey = load_active_pubkey(&db)?;
match cert.verify(&trusted_pubkey) {
    Ok(()) => println!("Verified: node {} was forgotten at {}", 
        cert.target_id, cert.timestamp),
    Err(VerificationError::ClockSkew) => eprintln!("Clock skew detected"),
    Err(VerificationError::BadSignature) => eprintln!("Certificate tampered"),
    Err(e) => eprintln!("Verification failed: {:?}", e),
}
```
