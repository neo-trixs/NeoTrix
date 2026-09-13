# Targeted Research #551: Mask/Maskit Pattern Absorption into Egress Privacy Guard

**Date**: 2026-09-13
**Source**: maskaisolutions/mask (Enterprise-grade AI DLP), maskit (config field masking)
**Target**: `nt_core_llm::egress_privacy_guard` + `privacy_guard.rs` (neotrix layer)
**Status**: Implemented

---

## Source Pattern: Mask (maskaisolutions/mask)

Mask is a Just-In-Time Privacy SDK for AI Agents. Core architecture:

1. **Request masking** — PII detected and replaced with Format-Preserving Encryption (FPE) tokens
2. **Model processing** — LLM reasons over ciphertext, never sees raw PII
3. **Response unmasking** — Pre-Tool Decryption Hook restores real values for backend execution
4. **Re-masking** — Post-Tool Encryption Hook catches new PII in tool output

### Key Technical Insights

| Insight | Mask Implementation | NeoTrix Absorption |
|---------|--------------------|--------------------|
| **Deterministic FPE** | HMAC-SHA256 generates same token for same PII within session | `session_consistent_placeholder()` — thread-local HashMap maps same PII → same `[MASKED:P-{Type}-{Seq}]` |
| **Tier 0 Deterministic** | Regex + checksums (Luhn/Mod-97/Mod-11) for structured PII | `PII_RULES` + `luhn_checksum_ok()` — zero NLP dependency |
| **50+ PII Types** | SSN, Credit Cards, IBAN, Email, Phone, Passport, NPI, etc. | `PII_RULES` covers 12 high-frequency types (Financial/Contact/Identity/Healthcare) |
| **Collision Avoidance** | Invalid prefixes (SSN 000-, CC 4000-) prevent false positives | Luhn validation on credit cards prevents normal digit sequences from being redacted |
| **Fail-Shut Default** | Halts on vault failure, never returns plaintext | NeoTrix Untrusted tier blocks on internal fingerprint detection (fail-closed) |
| **Session Consistency** | Same PII → same token preserves LLM reasoning context | Thread-local maps ensure multi-turn conversations see consistent placeholders |

---

## Changes Applied

### 1. Enhanced SECRET_PREFIXES (nt_core_llm.rs:492-551)

**Before**: 21 prefixes covering basic API keys and Bearer tokens.
**After**: 50+ prefixes organized by category:

| Category | Prefixes Added |
|----------|---------------|
| Cloud API Keys | `rk_live`, `rk_test`, `sk_live`, `sk_test`, `SG.`, `EAAI`, `pat-`, `napi_`, `aioa`, `AZURE_CLIENT_SECRET` |
| CI/CD & DevOps | `ghr_`, `glpat-`, `gldt-`, `ATATT`, `bxcb`, `boatu`, `drone.` |
| Container & Infra | `docker.`, `hkr_` |
| Key-Value Leaks | `api-key=`, `app_secret=`, `passwd=`, `pwd=`, `access_token=`, `auth_token=`, `private_key=`, `signing_key=`, `DATABASE_URL=`, `REDIS_URL=`, `MONGO_URI=`, `AWS_SECRET_ACCESS_KEY=`, `TWILIO_AUTH_TOKEN=`, `SLACK_WEBHOOK_URL=`, `WEBHOOK_SECRET=` |
| Crypto Wallets | `0x` (Ethereum), `bc1` (Bitcoin Bech32), `lntb` (Lightning) |

### 2. PII Regex Detection Rules (nt_core_llm.rs:553-591)

New `PII_RULES` static array with 12 regex patterns:

| Rule | Pattern | Type |
|------|---------|------|
| SSN | `\b\d{3}-\d{2}-\d{4}\b` | Financial |
| Credit Card | `\b(?:\d[ -]*?){13,19}\b` | Financial |
| IBAN | `\b[A-Z]{2}\d{2}[A-Z0-9]{4,30}\b` | Financial |
| Routing Number | `\b[1-9]\d{8}\b` | Financial |
| Email | `\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b` | Contact |
| US Phone | `(?:\+1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b` | Contact |
| IPv4 | `\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b` | Contact |
| IPv6 | `\b(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}\b` | Contact |
| MAC | `\b[0-9a-fA-F]{2}[:-]...{5}\b` | Contact |
| Passport | `\b[A-Z]\d{8}\b` | Identity |
| Tax ID | `\b\d{2}-\d{7}\b` | Identity |
| NPI | `\b[12]\d{9}\b` | Healthcare |

### 3. Session-Consistent Placeholder Mechanism (nt_core_llm.rs:613-665)

New functions:

- `session_consistent_placeholder(plaintext, pii_type)` — Maps same PII → same `[MASKED:P-{Type}-{Seq}]` within session
- `reset_session_placeholders()` — Clears maps at session start
- `resolve_placeholder(masked)` — JIT reverse lookup for local tool execution (Tool Pre-Hook)

Thread-local design:
- `SESSION_PLACEHOLDER_MAP: RefCell<HashMap<String, String>>` — plaintext → placeholder
- `SESSION_REVERSE_MAP: RefCell<HashMap<String, String>>` — placeholder → plaintext (local only)
- `SESSION_COUNTER: RefCell<u32>` — monotonic sequence for unique labels

### 4. Outbound Pipeline Integration (nt_core_llm.rs:733-737)

```rust
fn redact_outbound_str(s: &str) -> String {
    let s = redact_internals(s);
    let s = redact_secrets_str(&s);
    let s = redact_pii_session_consistent(&s);  // NEW: Mask Tier 0
    redact_paths_str(&s)
}
```

### 5. Ingress Guard Enhancement (nt_core_llm.rs:742-748)

Added `redact_pii_session_consistent` to `ingress_privacy_guard` — defense-in-depth catches model-echoed PII.

### 6. Neotrix-Layer INTERNAL_TOKENS Sync (privacy_guard.rs:28-66)

Updated to match core layer's expanded token list (added `nt_scout_`, `SEAL Pipeline`, `LlmProviderType`, `ProviderCategory`, `GatewayProvider`, `Redactor`, `neotrix-experience`, `neotrix-tauri`, `experience.db`).

### 7. Streaming Support Documentation (privacy_guard.rs:128-148)

Documented streaming strategy:
- Request-side: each chunk through `egress_privacy_guard`
- Response-side: each chunk through `ingress_privacy_guard`
- Session consistency: shared `SESSION_PLACEHOLDER_MAP` across all chunks
- Chunk boundary risk: 256-byte overlap buffer recommended for patterns spanning chunk boundaries

---

## Architecture Mapping

```
Mask Architecture                    NeoTrix Mapping
─────────────────                    ───────────────
Masking (FPE tokens)           →     redact_pii_session_consistent()
Pre-Tool Decryption Hook       →     resolve_placeholder() + TOOL PRE-HOOK
Post-Tool Encryption Hook      →     redact_pii_session_consistent() in tool output
Session Vault (MemoryVault)    →     SESSION_PLACEHOLDER_MAP (thread-local)
Tier 0 (Deterministic)         →     PII_RULES + luhn_checksum_ok()
Tier 1 (Neural NER)            →     [Not absorbed — NeoTrix is Rust, no spaCy]
Fail-Shut                      →     Untrusted + block_untrusted_leak
Format-Preserving Tokens       →     [MASKED:P-{Type}-{Seq}] format
```

---

## What Was NOT Absorbed (and Why)

| Mask Feature | Reason |
|-------------|--------|
| Tier 1 NLP (spaCy/Transformers) | NeoTrix is Rust-native; no Python runtime. Future: integrate `ort` crate for local NER. |
| Format-Preserving Encryption (FPE) | Requires HMAC-PE key management. Simplified to counter-based placeholders for MVP. Future: add FPE with session key. |
| Pluggable Vaults (Redis/DynamoDB) | Thread-local HashMap is sufficient for single-process agent. Future: `Arc<RwLock>` for multi-agent. |
| Async API (aencode/adecode) | Rust async already native; no wrapper needed. |
| Framework Integrations (LangChain/ADK) | NeoTrix has its own agent loop, not LangChain. |

---

## Testing Strategy

Tests to add (not in this PR):
1. `test_pii_ssn_redacted` — Verify SSN `\d{3}-\d{2}-\d{4}` → `[MASKED:P-SSN-001]`
2. `test_pii_credit_card_luhn` — Valid CC (4539148803436467) redacted; invalid digits NOT redacted
3. `test_pii_email_redacted` — Email → `[MASKED:P-Email-001]`
4. `test_session_consistency` — Same email twice → same placeholder
5. `test_reset_session_placeholders` — Counter resets, old placeholders gone
6. `test_resolve_placeholder` — Local tool gets real value back
7. `test_new_secret_prefixes` — `ghr_`, `glpat-`, `DATABASE_URL=` etc. detected

---

## Risk Assessment

| Risk | Mitigation |
|------|-----------|
| False positives on RoutingNumber (9-digit sequences) | Pattern `\b[1-9]\d{8}\b` is broad; consider adding keyword proximity check |
| Regex compile cost per call | Patterns are `static`, compiled once via `lazy_static` or `once_cell` (future optimization) |
| Thread-local leak in long-running agents | `reset_session_placeholders()` should be called per conversation turn |
| IPv4 false positives (version numbers, timestamps) | Consider adding keyword proximity or context-aware detection |
