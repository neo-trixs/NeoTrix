# Iteration Batch 417 — Authentication, Authorization & Identity Management Research (2026-09-06)

## Sources Cited

| # | Source | Year | Topic |
|---|--------|------|-------|
| S1 | FIDO Alliance — *The State of Passkeys 2026: Global Consumer and Workforce Report* (May 2026) | 2026-05 | 5 billion passkeys in active use; 90% consumer awareness; 75% enabled on at least some accounts. 87% of US/UK workforce deploying passkeys (Okta/Corbado analysis). |
| S2 | Clerk — *Authentication Trends in 2026: Passkeys, AI Agents, and Edge* (Jun 2026) | 2026-06 | 15 billion user accounts passkey-enabled; Microsoft defaulting new consumer accounts to passkeys; 41% of passwordless implementations still use magic links; OAuth 2.0 PDF authentication for AI agents. |
| S3 | HID Global — *IAM in 2026: Passwordless, Passkeys & AI-Driven Threats* (Jan 2026) | 2026-01 | Gartner: by 2026, 30% of enterprises will no longer consider biometric verification reliable in isolation (deepfakes). Adaptive IAM with real-time risk scoring, context-aware policies. 90%+ MFA transactions will use FIDO by 2027. |
| S4 | Security Boulevard — *The Complete Guide to Passwordless Authentication in 2026* (Apr 2026) | 2026-04 | Passwordless success rates: Passkeys 99.1%, Biometric 98.9%, Magic Link 98.2%. Passkey login speed 0.9s. Account recovery as first-class feature. NIST SP 800-63-4 no longer recognizes SMS OTP for AAL2. |
| S5 | MojoAuth — *The State of Passwordless Authentication 2026* (Apr 2026) | 2026-04 | 2,847 enterprise deployments across 89 countries. Biometric 0.7s first-time. Mobile users experience 23% faster auth with biometric/passkey; 2.8x higher abandonment with password flows. |
| S6 | Petronella Tech — *ABAC vs RBAC: Policy-as-Code to Secure Enterprise AI* (Jun 2026) | 2026-06 | ABAC + Policy-as-Code (PBAC) replaces rigid RBAC for LLMs, vector DBs, AI agents. Dynamic context-aware decisions for prompts, tool calls, RAG. OPA/Rego dominant. |
| S7 | LoginRadius — *ABAC vs RBAC: Choosing Access Control Model for Zero Trust* (Apr 2026) | 2026-02 | Hybrid RBAC+ABAC: RBAC as coarse baseline, ABAC as fine-grained overlay. Principle of Least Privilege + Just-in-Time Access + Zero Standing Privilege. Policy sprawl is ABAC's main operational risk. |
| S8 | Tech Insider — *Zero Trust Architecture: The 2026 Implementation Guide* (Jun 2026) | 2026-06 | 60% of large enterprises will have measurable zero trust programs by 2026 (Gartner). $31.6B market (2025) → $67.3B by 2028 (16.6% CAGR). 78% of ZT initiatives begin with IAM modernization. 50% reduction in breach impact costs. |
| S9 | Softices — *What is Zero Trust Architecture?* (Jul 2026) | 2026-07 | Policy-as-Code (OPA) centralizes access rules. DevSecOps pipeline integration: automated policy compliance checks halt builds on failures. IaC validation before deployment. |
| S10 | Security Boulevard — *Decentralized Identity and Verifiable Credentials: The Enterprise Playbook 2026* (Mar 2026) | 2026-03 | Decentralized identity market $7.4B (2026). eIDAS 2.0 mandates EU digital identity wallets by year-end. W3C VC 2.0 standard. VC + liveness detection = strongest anti-deepfake architecture. OpenID4VP for credential presentation. |
| S11 | Okta Developer — *How Verifiable Digital Credentials Are Reshaping Trust Architecture* (Jun 2026) | 2026-06 | Mobile driver's licenses live in dozens of US states. Apple/Google Wallet hold gov credentials. Digital Credentials API browser rollout. Redirect-based OpenID4VP as baseline. Auth and verification layers separating permanently. |
| S12 | Indicio Tech — *Decentralized Identity in 2026: Paving The Way For Progress* (Feb 2026) | 2026-02 | AI deepfakes defeating legacy identity systems. VC + biometric liveness = combined assurance stronger than either factor alone. Convergence of decentralized credentials with AI-powered liveness detection. |
| S13 | MajorKey Tech — *Decentralized Identity Explained: A Practical Q&A for 2026* (Mar 2026) | 2026-03 | Self-sovereign identity: individuals own and control identity via secure digital wallets. Hybrid deployments (centralized + decentralized). DID resolvers behind existing IAM. |
| S14 | Gupta Deepak — *Decentralized Identity Playbook: eIDAS 2.0 Guide 2026* (Jun 2026) | 2026-06 | Post-quantum cryptography migration needed for VC crypto (currently ECC-based). Trust registries and issuer vetting. CIAM platform VC roadmap. Graceful fallback to centralized methods. |
| S15 | ContentWave — *ABAC for ZTNA: Practical Zero Trust Deployment Guide 2026* (Aug 2026) | 2026-08 | Phased ABAC deployment for ZTNA in hybrid cloud/on-prem. Attribute hygiene, policy testing, audit discipline. |

---

## Defects Found

### DEFECT-1: No Passkey/FIDO2/WebAuthn Authentication Support
**Severity**: CRITICAL  
**Location**: `nt_io_web/server.rs:49-71` (auth_middleware)  
**Evidence**: The auth middleware does simple string comparison of bearer tokens (iteration_batch_370.md:136-137). No FIDO2/WebAuthn registration or authentication flow exists. No passkey credential storage. No WebAuthn challenge-response protocol.  
**2026 Gap**: S1 reports 5 billion passkeys in active use (90% consumer awareness). S2 confirms 15 billion accounts passkey-enabled. S4 shows passkeys achieve 99.1% success rate at 0.9s login time. NIST SP 800-63-4 no longer recognizes SMS OTP for AAL2 (S4). NeoTrix's bearer-token-only auth is two generations behind the industry standard.  
**Suggestion**: Implement `nt_io::passkey_auth` module: (1) WebAuthn registration flow (challenge → credential creation → storage in KB), (2) WebAuthn authentication flow (challenge → signature verification), (3) sync vs device-bound passkey distinction, (4) cross-platform QR code flow for cross-device auth. Integrate with existing `nt_io_web` middleware. Add account recovery as first-class feature (backup passkeys, backup codes, email re-verification per S4).

### DEFECT-2: No Adaptive/Context-Aware Authentication
**Severity**: HIGH  
**Location**: `nt_io_web/server.rs` (auth_middleware), `nt_shield`  
**Evidence**: Auth middleware performs static token check with no context evaluation. No real-time risk scoring. No device reputation tracking. No geolocation-based policy. No behavioral anomaly detection during session.  
**2026 Gap**: S3 reports Gartner predicts 30% of enterprises will distrust biometric verification in isolation by 2026 due to deepfakes. Adaptive IAM with real-time risk scoring is now a "must-have security control" (S3). S8 shows 78% of Zero Trust initiatives begin with IAM modernization. NeoTrix's static auth has no adaptation layer.  
**Suggestion**: Implement `nt_io::adaptive_auth` module: (1) risk scoring engine (device trust + geolocation + time-of-access + behavioral patterns), (2) step-up authentication triggers (low-risk → passkey only, high-risk → passkey + biometric), (3) session-level risk re-evaluation on each request, (4) integration with NT-SHIELD threat detection for real-time signals.

### DEFECT-3: No Continuous Authentication / Behavioral Biometrics
**Severity**: HIGH  
**Location**: `nt_io_web/session.rs` (session management)  
**Evidence**: Sessions are established once and persist until expiry or explicit logout. No continuous identity verification after initial login. No typing cadence, mouse movement, or navigation pattern monitoring.  
**2026 Gap**: S3/S5 report behavioral biometrics as a 2026 mainstream trend: "continuous authentication monitors user behavior to validate identity throughout a session, detecting account takeovers even after initial login." S4 shows 14.2% of password users abandon after 1 failed attempt — behavioral biometrics catch compromised sessions that static auth misses.  
**Suggestion**: Implement `nt_io::continuous_auth` module: (1) establish per-user behavioral baseline (typing cadence, navigation patterns, command sequences), (2) real-time deviation detection during session, (3) progressive risk escalation (anomaly → step-up auth → session termination), (4) integration with NT-FEEL for interaction pattern analysis.

### DEFECT-4: No Policy-as-Code Authorization Engine (RBAC/ABAC/ReBAC)
**Severity**: HIGH  
**Location**: `nt_io_web/server.rs` (auth_middleware), `nt_shield`  
**Evidence**: Authorization is hardcoded in Rust functions. No externalized, versioned, testable policy definitions. No RBAC role hierarchy. No ABAC attribute evaluation. No ReBAC relationship graph. No OPA/Rego or equivalent policy engine.  
**2026 Gap**: S6/S7/S9 confirm Policy-as-Code (OPA/Rego) is the dominant 2026 authorization pattern. S7: "Most mature 2026 architectures run RBAC as coarse baseline with ABAC layered on top." S8: 78% of ZT initiatives start with IAM modernization. S9: DevSecOps pipelines halt on policy failures. NeoTrix has zero policy externalization.  
**Suggestion**: Implement `nt_io::policy_engine` module: (1) OPA/Rego integration for externalized policy definitions, (2) RBAC role hierarchy (role → permission mapping), (3) ABAC attribute evaluation (subject + resource + action + environment), (4) Policy versioning and testing in CI, (5) Just-in-Time access with zero standing privilege (S7 pattern). Wire into auth middleware for per-request policy evaluation.

### DEFECT-5: No Verifiable Credential / Decentralized Identity Support
**Severity**: MEDIUM  
**Location**: `nt_io` (no DID/VC infrastructure)  
**Evidence**: No Decentralized Identifier (DID) support. No Verifiable Credential issuance, storage, or verification. No identity wallet integration. No OpenID4VP protocol support. No selective disclosure capability.  
**2026 Gap**: S10 reports decentralized identity market at $7.4B (2026) with eIDAS 2.0 mandating EU digital identity wallets. S11: mobile driver's licenses live in dozens of US states; Apple/Google Wallet hold gov credentials. S12: VC + liveness detection = strongest anti-deepfake architecture. S14: post-quantum crypto migration needed for VC (currently ECC-based). NeoTrix cannot participate in the emerging credential ecosystem.  
**Suggestion**: Implement `nt_io::verifiable_credential` module: (1) DID resolver integration (W3C DID Core), (2) VC verification (check issuer signature, expiration, revocation), (3) OpenID4VP redirect-based flow for credential presentation, (4) selective disclosure (present only required attributes), (5) graceful fallback to centralized auth when VC unavailable. Start with verification-only (accept VCs from external issuers) before adding issuance.

### DEFECT-6: No Post-Quantum Cryptography Migration Path for Auth Credentials
**Severity**: MEDIUM  
**Location**: `nt_memory/key_encryption.rs`, `nt_io_web` (token signing)  
**Evidence**: `key_encryption.rs` uses AES-256-GCM for credential vault encryption (iteration_batch_357.md:90). Bearer tokens likely use RSA/ECDSA signing. No post-quantum algorithm migration plan.  
**2026 Gap**: S14 explicitly warns: "verifiable credential cryptography (currently ECC-based) needs a post-quantum upgrade path." NIST PQC standards (ML-KEM, ML-DSA) are finalized. S3: preemptive cybersecurity solutions will make up 50% of IT security spending by 2030. NeoTrix's auth crypto has no PQC migration path.  
**Suggestion**: (1) Audit all cryptographic primitives in auth pipeline (token signing, credential encryption, passkey signatures), (2) add `nt_shield::crypto_agility` module with algorithm negotiation, (3) implement hybrid mode (classical + PQC) for transition period, (4) plan migration timeline aligned with NIST PQC deadlines. Prioritize token signing (most exposed to "harvest now, decrypt later" attacks).

### DEFECT-7: No Digital Identity Wallet / mDL Integration
**Severity**: MEDIUM  
**Location**: `nt_io` (no wallet infrastructure)  
**Evidence**: No mobile driver's license (mDL) support. No EUDI Wallet integration. No ISO 18013-5 (mDL) implementation. No Digital Credentials API awareness.  
**2026 Gap**: S10/S11 report eIDAS 2.0 mandates EU digital identity wallets by year-end 2026. S11: "The broader trends driving VDC adoption are not speculative. Government credential infrastructure is becoming a dependency layer." S11: build on redirect-based OpenID4VP as baseline; layer Digital Credentials API as progressive enhancement. NeoTrix cannot verify or accept government-issued digital credentials.  
**Suggestion**: (1) Add OpenID4VP client to `nt_io` for credential presentation requests, (2) implement ISO 18013-5 mDL verification, (3) build Digital Credentials API integration as progressive enhancement (not hard dependency per S11), (4) plan EUDI Wallet compliance path for EU deployments.

### DEFECT-8: No Trust Registry / Issuer Verification Infrastructure
**Severity**: LOW  
**Location**: `nt_io` (no trust framework)  
**Evidence**: No trust registry for verifying credential issuers. No issuer reputation/vetting system. No mechanism to determine if a presented VC comes from a trusted issuer.  
**2026 Gap**: S14: "Trust registries and issuer vetting needs work." S10: enterprise playbook requires Phase 2 "Build trust framework integration as a first-class architectural requirement." S12: combined VC + liveness detection requires issuer trust verification as foundation. Without trust registries, NeoTrix cannot determine if a VC issuer is legitimate.  
**Suggestion**: (1) Define `TrustRegistry` trait with issuer metadata (DID, name, trust level, revocation status), (2) implement local trust registry with configurable trust anchors, (3) support external trust registry queries (e.g., EBSI Trusted Issuer Registry), (4) integrate into VC verification pipeline as mandatory step.

---

## Summary

| Metric | Count |
|--------|-------|
| Sources cited | 15 |
| Defects found | 8 |
| CRITICAL severity | 1 |
| HIGH severity | 3 |
| MEDIUM severity | 3 |
| LOW severity | 1 |
| Domains affected | NT-IO (auth, session, wallet), NT-SHIELD (crypto, trust), NT-MEMORY (key encryption) |

## Recommended Priority Order

1. **DEFECT-1** (Passkey/FIDO2) — CRITICAL; 15B accounts passkey-enabled, NIST deprecates SMS OTP, NeoTrix bearer-only auth is obsolete
2. **DEFECT-4** (Policy-as-Code) — HIGH; OPA/Rego is 2026 standard, enables RBAC+ABAC hybrid, required for Zero Trust
3. **DEFECT-2** (Adaptive Auth) — HIGH; context-aware auth is mandatory for Zero Trust, prevents deepfake-enabled account takeover
4. **DEFECT-3** (Continuous Auth) — HIGH; behavioral biometrics catch post-login compromise, mainstream 2026 trend
5. **DEFECT-5** (Verifiable Credentials) — MEDIUM; $7.4B market, eIDAS 2.0 mandate, anti-deepfake architecture
6. **DEFECT-7** (mDL/Wallet) — MEDIUM; EU mandate timeline imminent, government credential dependency
7. **DEFECT-6** (PQC Migration) — MEDIUM; harvest-now-decrypt-later threat active, plan needed now
8. **DEFECT-8** (Trust Registry) — LOW; foundational but can be added incrementally
