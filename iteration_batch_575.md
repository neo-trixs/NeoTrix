# Iteration Batch 575 — TLS / Authentication / Key Management Research

**Date:** 2026-09-06
**Predecessor:** Batch 574 (static fusion, no probabilistic calibration, static mixing weights, no domain-aware strategy selection, adaptive-feedback-loop gap)
**Domains:** TLS, Authentication, Key Management

---

## TLS — Findings & New Defects

### 1. Post-Quantum Certificate Authentication Gap (NEW vs 574)

**Source:** tlsradar.com/blog/post-quantum-tls-state-2026 (Jul 2026)

Key exchange is solved (X25519MLKEM768 ships by default in Chrome/Firefox/Safari + Cloudflare/AWS/Fastly). **Authentication (server identity via certificates) remains classical RSA/ECDSA.** ML-DSA (FIPS 204) and SLH-DSA (FIPS 205) signatures are 2-10 KB each; a full PQ certificate chain balloons the handshake from ~3 KB to tens of KB — extra RTT on lossy links.

**Merkle Tree Certificates (MTC)** — IETF draft-davidben-tls-merkle-tree-certs — offer compact inclusion proofs instead of full signature chains. Designed for the 47-day certificate lifetime regime. **Not yet deployed.**

**Defect D575-TLS-1: No PQ certificate strategy exists in NeoTrix.** The NT-IO LLM/provider TLS stack uses classical certificates. No plan for PQ certificate transition when CA/Browser Forum mandates hybrid or PQ signatures. This is a **time-bombed defect** — dormant until the standards move.

**Defect D575-TLS-2: No adaptive TLS configuration based on client capability.** Current TLS negotiation is static (library default). No runtime selection of hybrid vs classical based on client PQ support, network latency, or security tier. The batch 574 "static mixing weights" finding applies here: X25519+MLKEM-768 is always preferred regardless of context.

**Defect D575-TLS-3: QUIC hybrid TLS lacks adaptive 0-RTT security tradeoff.** QUIC's 0-RTT early data is replay-vulnerable. With hybrid handshakes (1-2 KB larger), the tradeoff between 0-RTT performance and PQ security is unmodeled. No NeoTrix component adjusts 0-RTT policy based on threat model.

### 2. Harvest-Now-Decrypt-Later (HNDL) Threat Model Gap (NEW vs 574)

**Source:** quantumoutpost.com/tutorials/51-hybrid-tls-pqc (Apr 2026)

30-50% of browser TLS handshakes are now hybrid. But NT-WORLD crawl pipelines and NT-SHIELD network components do not enforce PQ-safe session establishment for long-lived data-at-rest derived from captured sessions.

**Defect D575-TLS-4: No HNDL risk scoring for archived crawl data.** Captured web data in NT-MEMORY KB may have been negotiated over classical-only TLS at crawl time. No audit trail tracks what percentage of archived data is PQ-protected at rest.

---

## Authentication — Findings & New Defects

### 1. AI Agents as First-Class Authentication Principals (NEW vs 574)

**Source:** clerk.com/articles/authentication-trends-in-2026-passkeys-ai-agents-and-edge (2026)

MCP (Model Context Protocol) specification (2025-11-25) **mandates OAuth 2.1 with PKCE** for AI agent authentication. NIST's AI Agent Standards Initiative (Feb 2026) treats agents as first-class identity tier. Gartner: 30% of enterprises consider identity verification unreliable in isolation by 2026 due to deepfakes.

**Defect D575-AUTH-1: No OAuth 2.1 + PKCE agent identity protocol for NT-ACT orchestration.** NT-ACT calls external LLM providers and APIs without a standardized agent identity layer. Current approach: API keys as static secrets. No token-exchange flow for delegated agent access. No dynamic client registration. This is the **agent identity gap** — batch 574 had no concept of non-human principals.

**Defect D575-AUTH-2: No deepfake-aware multi-factor escalation.** Authentication layer alone is no longer verification layer (Gartner prediction). NeoTrix has no mechanism to escalate authentication requirements when AI-generated content is detected in the input stream.

### 2. Passkey Recovery Attack Surface (NEW vs 574)

**Source:** clerk.com/articles (2026), proofpoint/squarex/prove research cited

15B+ accounts are passkey-enabled. But account recovery is the **hardest unsolved problem** — fallback methods (magic links, backup codes, phone verification) reintroduce attack surface. Emerging attacks:
- **Synced-passkey downgrade attacks** (Proofpoint): browser spoofing forces weaker fallback
- **Passkey hijacking via WebAuthn API interception** (SquareX, DEF CON 2025)
- **SIM-swap-enabled passkey-sync recovery fraud** (Prove): attacks recovery path, not the passkey itself

**Defect D575-AUTH-3: No passkey recovery security model.** NeoTrix credential storage (if passkey-based auth is adopted) has no defined recovery path security policy. The "recovery reintroduces attack surface" problem is unmodeled.

### 3. Edge JWT Verification Limitations (NEW vs 574)

**Source:** clerk.com (2026), nextjs.org/blog/next-16

Edge verification (CDN PoP) cuts auth latency 3-10×. But: any decision more complex than "is this JWT signed and unexpired" still belongs at origin. Symmetric signing is a non-starter at 300+ edge locations (breach blast radius).

**Defect D575-AUTH-4: No tiered verification strategy (edge vs origin).** All NT-IO LLM API authentication is origin-level. No concept of progressive verification: fast edge check → full origin authorization. This is a **static strategy** problem (batch 574 pattern).

### 4. FIDO2 Attestation and AAGUID Policy Gap (NEW vs 574)

**Source:** clerk.com (2026), NIST SP 800-63-4 (Jul 2025)

NIST SP 800-63-4 formally qualifies syncable passkeys at AAL2. AAL3 still requires device-bound authenticators. Enterprise programs use AAGUID allowlists to restrict authenticator models. No distinction in NeoTrix between synced vs device-bound credential trust levels.

**Defect D575-AUTH-5: No credential assurance level differentiation.** All credentials treated uniformly. No policy engine that distinguishes AAL2 (synced passkey) from AAL3 (hardware key) based on operation sensitivity.

---

## Key Management — Findings & New Defects

### 1. Secrets Sprawl as Systemic Defect (NEW vs 574)

**Source:** jumpserver.com/blog/secret-management-best-practices-2026 (Apr 2026), gitguardian 2025 report

12.8M secrets exposed in public GitHub repos in 2025 (28% YoY increase). Uber (2022), CircleCI (2023), Vercel (2026 supply chain attack) — all share common thread: insufficient secrets governance enabling lateral movement.

**Defect D575-KM-1: No secret sprawl detection across NeoTrix crates.** No pre-commit hooks, CI scanning (GitGuardian/Trufflehog/Gitleaks), or automated discovery of hardcoded credentials across neotrix-core, crates/, and src-tauri/. The 12.8M exposed secrets stat directly applies.

**Defect D575-KM-2: No automated secret rotation lifecycle.** Static long-lived secrets (API keys, TLS certs) persist without rotation schedules. The "dual-credential pattern" for zero-downtime rotation is not implemented. This extends batch 574's "static mixing weights" to key management — weights (rotations) are never adjusted.

### 2. Post-Quantum HSM Migration Gap (NEW vs 574)

**Source:** pistack.xyz (May 2026), encryptionconsulting.com (Jan 2026), securedapp.io (May 2026)

FIPS 140-3 certification for PQC algorithms is in progress. HSM vendors (Thales, Entrust, AWS CloudHSM) are adding ML-KEM/ML-DSA support but **no production HSM in 2026 handles PQ key generation natively**. Self-hosted HSM options (SoftHSM, netHSM) support Ed25519/RSA/ECDSA but not lattice-based algorithms.

**Defect D575-KM-3: No PQ migration plan for HSM-backed keys.** NeoTrix KB encryption keys, LLM provider credentials, and TLS private keys may be HSM-backed. No assessment of which keys require PQ migration and timeline. The enterprise key management market ($4.19B in 2026, 18.7% CAGR) is moving to PQC — NeoTrix is not tracking.

### 3. MPC vs Multisig vs HSM Decision Gap (NEW vs 574)

**Source:** interexy.com/institutional-key-management (Aug 2026)

Institutional custody in 2026 uses a **layered model**: HSM for hardware-grade root keys, MPC for threshold signing without single point of failure, multisig for on-chain distributed governance. The "Custody 3.0" pattern is: HSM root → MPC operational → multisig policy.

**Defect D575-KM-4: No threshold signing or key splitting.** All NeoTrix cryptographic operations use single-key signing. No MPC (Multi-Party Computation) for distributed trust. No key splitting across multiple holders. Single point of compromise for all KB and communication keys.

### 4. CI/CD Secret Injection Anti-Pattern (NEW vs 574)

**Source:** jumpserver.com (2026)

Secrets passed as plaintext environment variables in CI/CD is explicitly called out as anti-pattern. Vercel 2026 supply chain attack leveraged compromised CI/CD credentials.

**Defect D575-KM-5: No CI/CD secrets vault integration.** NeoTrix build pipeline (GitHub Actions / Cargo) does not integrate with a secrets vault (HashiCorp Vault, AWS Secrets Manager). Build secrets (signing keys, API tokens) likely passed as environment variables or workflow secrets without dynamic injection.

---

## Cross-Domain Defects (NEW vs 574)

### 1. Unified Cryptographic Agility Layer Missing (NEW vs 574)

**Source:** IETF draft-ietf-uta-pqc-app-01 (Feb 2026), beyondtmrw.org (Jun 2026)

Enterprise playbooks emphasize **crypto agility**: the ability to swap algorithms without application changes. NIST FIPS 203/204/205 are the baseline; the next migration (PQ signatures in certificates) will require algorithm-level swappability.

**Defect D575-XD-1: No crypto agility abstraction.** NeoTrix hardcodes algorithm choices (TLS library defaults, signing algorithms in KB, credential types). No pluggable crypto provider interface that allows algorithm substitution without code changes. This is the **meta-defect** underlying D575-TLS-1, D575-KM-3.

### 2. Adaptive Feedback Loop Confirmation (EXTENDS 574)

Batch 574 identified adaptive-feedback-loop gap across simulation/embodiment/search/ranking. Batch 575 confirms the gap extends to:
- **TLS:** No feedback from handshake success/failure rates to negotiate PQ vs classical
- **Auth:** No feedback from passkey adoption metrics to adjust authentication strategy
- **KM:** No feedback from secret age/rotation compliance to trigger emergency rotation

**Defect D575-XD-2: Adaptive feedback gap confirmed in TLS/Auth/KM.** Three additional domains now show the same pattern batch 574 identified. The gap is **architectural**, not per-domain.

---

## Summary: What's NEW vs Batch 574

| # | Defect | Domain | Severity | Novelty |
|---|--------|--------|----------|---------|
| D575-TLS-1 | No PQ certificate strategy | TLS | HIGH (time-bombed) | NEW |
| D575-TLS-2 | Static hybrid TLS config | TLS | MEDIUM | Extends 574 static weights |
| D575-TLS-3 | QUIC 0-RTT security tradeoff unmodeled | TLS | MEDIUM | NEW |
| D575-TLS-4 | No HNDL risk scoring for archived data | TLS | MEDIUM | NEW |
| D575-AUTH-1 | No OAuth 2.1 agent identity protocol | Auth | HIGH | NEW (agent identity is 2026 concept) |
| D575-AUTH-2 | No deepfake-aware MFA escalation | Auth | HIGH | NEW |
| D575-AUTH-3 | No passkey recovery security model | Auth | HIGH | NEW |
| D575-AUTH-4 | No tiered edge/origin verification | Auth | MEDIUM | Extends 574 static strategy |
| D575-AUTH-5 | No credential assurance level differentiation | Auth | MEDIUM | NEW |
| D575-KM-1 | No secret sprawl detection | KM | HIGH | NEW |
| D575-KM-2 | No automated secret rotation | KM | HIGH | Extends 574 static weights |
| D575-KM-3 | No PQ migration plan for HSM keys | KM | HIGH (time-bombed) | NEW |
| D575-KM-4 | No threshold signing / key splitting | KM | MEDIUM | NEW |
| D575-KM-5 | No CI/CD secrets vault integration | KM | HIGH | NEW |
| D575-XD-1 | No crypto agility abstraction | Cross | HIGH | NEW (meta-defect) |
| D575-XD-2 | Adaptive feedback gap in TLS/Auth/KM | Cross | HIGH | Extends 574 |

**Novel defects vs 574:** 12 entirely new findings
**Extended defects from 574:** 4 findings that generalize 574 patterns to new domains

---

## Sources Cited

1. tlsradar.com — Post-Quantum TLS in 2026: What's Deployed, and the Certificate Problem (Jul 2026)
2. beyondtmrw.org — Post-Quantum TLS Migration: Browser Deadlines and Enterprise Playbooks (Jun 2026)
3. quantumoutpost.com — Hybrid TLS with Post-Quantum KEMs: How the Internet Is Migrating (Apr 2026)
4. mobileproxy.space — Post-Quantum Cryptography 2026: ML-KEM, Hybrid TLS, Impact on Proxies (May 2026)
5. techbytes.app — Post-Quantum TLS 2026 Practical Developer Handbook
6. IETF draft-ietf-uta-pqc-app-01 — PQC Recommendations for TLS-based Applications (Feb 2026)
7. clerk.com — Authentication Trends in 2026: Passkeys, AI Agents, and Edge (2026)
8. webhani.com — Authentication in 2026: Passkeys, WebAuthn, and the OAuth3 (Aug 2026)
9. rajpoot.dev — Authentication in 2026: Passkeys, OAuth 2.1, OIDC (May 2026)
10. mojoauth.com — 2026 Industry Report: Accelerated FIDO2 Passkey Adoption (Jun 2026)
11. fidoalliance.org — 15B+ accounts with passkey access; Passkey Index 2025
12. NIST SP 800-63-4 (Jul 2025) — formally qualifies syncable passkeys at AAL2
13. MCP specification 2025-11-25 — mandates OAuth 2.1 with PKCE for AI agents
14. NIST AI Agent Standards Initiative (Feb 2026) — agents as first-class identity tier
15. jumpserver.com — Secrets Management Best Practices for Enterprise Security 2026 (Apr 2026)
16. encryptionconsulting.com — Best Practices for Key Management in 2026 (Jan 2026)
17. pistack.xyz — Self-Hosted HSM and Key Management: SoftHSM vs netHSM (May 2026)
18. interexy.com — Institutional Key Management: MPC vs Multisig vs HSM (Aug 2026)
19. securedapp.io — Top 7 Enterprise HSM Key Management Companies in 2026 (May 2026)
20. ResearchAndMarkets — Enterprise Key Management Market Report 2026 ($4.19B, 18.7% CAGR)
