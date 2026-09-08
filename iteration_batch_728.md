# Iteration Batch 728 — Secrets, HSM, Key Rotation Research

**Date**: 2026-09-06 | **Research Loop**: 728/10000+

---

## Topic 1: Secrets Management (Vault / SOPS / ESO)

### Sources
1. Latchkey Learn (2026-06-26): Vault vs SOPS comparison — [latchkey.dev](https://latchkey.dev/learn/tool-comparisons/vault-vs-sops)
2. IAN Cloud (2026-04-22): K8s Secret Management 2026 — [iancloud.ai](https://iancloud.ai/blog/kubernetes-secret-management-external-secrets-sops-vault-2026)
3. ZeonEdge (2026-02-09): Secrets Management 2026 — [zeonedge.com](https://zeonedge.com/blog/secrets-management-2026-vault-sops-external-secrets-operator)
4. Unixy.io (2026-02-15): Vault vs Secrets Manager vs SOPS — [unixy.io](https://unixy.io/blog/secrets-management-2026/)
5. Sachith Dassanayake (2026-06-18): Secret Management Monitoring & Observability — [sachith.co.uk](https://www.sachith.co.uk/secret-management-sops-vault-and-kms-monitoring-observability-practical-guide-jun-18-2026/)
6. CORE Systems (2025-12-13, updated 2026-03): Secrets Management Vault/SOPS — [core.cz](https://core.cz/en/blog/2026/secrets-management-vault-sops-2026/)
7. SecureCoding (2026-09-01): Best Secrets Management Tools — [securecoding.com](https://www.securecoding.com/identity/secrets-management-tools/)
8. Pavan Rangani (2026-02-13): Vault vs AWS SM vs SOPS — [pavanrangani.com](https://pavanrangani.com/blog/secrets-management-vault-aws-sops)

### Key Findings
- **Three-tier architecture is 2026 consensus**: SOPS (GitOps bootstrap secrets) + Cloud Secrets Manager (bulk app secrets) + Vault (dynamic credential tier). No single tool covers all needs.
- **SOPS is NOT a vault** — it's an encrypted file envelope. No access control beyond KMS, no audit trail beyond KMS logs, no dynamic secrets, no rotation mechanism.
- **Vault dynamic secrets are the gold standard** — per-request short-lived credentials that auto-revoke. Exposure window drops from days to ≤1 hour.
- **ESO bridges Vault to K8s** — External Secrets Operator syncs from Vault/AWS SM/GCP SM into K8s Secrets automatically.
- **64% of leaked credentials were still valid** in Jan 2026 per GitGuardian (cited in Digital Applied). Governance failure, not detection failure.
- **OpenBao** is the Linux Foundation Vault fork (MPL-2.0) — same API, open license, younger operator memory.

### NEW Defects Found for NeoTrix

| # | Defect | Severity | Detail |
|---|--------|----------|--------|
| 1 | **No secrets management strategy at all** | CRITICAL | NeoTrix has no Vault, no SOPS, no ESO, no secrets manager. KB password, API keys, and any credentials are presumably in plaintext config or env vars. Batch 727 confirmed zero backup; this confirms zero secrets hygiene. |
| 2 | **No dynamic credential tier** | HIGH | All credentials in NeoTrix are static. No per-service short-lived DB credentials. A leaked KB password grants permanent access to all 7 domains. |
| 3 | **No secret rotation policy** | HIGH | No scheduled or event-driven rotation for any credential. No rotation logs, no rotation automation, no rotation ownership. |
| 4 | **No secret scanning** | HIGH | No gitleaks, trufflehog, or detect-secrets in pre-commit or CI. Hardcoded secrets in code/commits go undetected. |
| 5 | **No SOPS bootstrap layer** | MEDIUM | No encrypted-in-Git secrets for infrastructure-as-code (Terraform, K8s manifests). All config secrets presumably in plaintext. |
| 6 | **No OIDC federation** | MEDIUM | No short-lived federated tokens for CI/CD or cloud access. Long-lived API keys in env vars = permanent leak surface. |
| 7 | **No consumer inventory** | MEDIUM | No registry of which services consume which secrets. Rotation cannot be safely executed without knowing all consumers. |
| 8 | **No pre-commit leak prevention** | MEDIUM | No hooks to prevent secrets from being committed. GitGuardian reports 29M secrets pushed to public GitHub in 2025. |

---

## Topic 2: Hardware Security Modules (HSM)

### Sources
1. Google Cloud (2026-02-03): Single-tenant Cloud HSM — [cloud.google.com](https://cloud.google.com/blog/products/identity-security/introducing-single-tenant-cloud-hsm-for-more-data-encryption-control)
2. Thales (2026-08-04): Luna 8 HSM — [data-protection-updates.gemalto.com](https://data-protection-updates.gemalto.com/2026/08/04/introducing-luna-8-our-fastest-hsm-ever-built-for-the-post-quantum-era/)
3. Microsoft Azure (2026-07-07): External Key Management for Managed HSM — [azure.microsoft.com](https://azure.microsoft.com/en-us/blog/external-key-management-for-azure-managed-hsm-is-now-in-public-preview/)
4. Microsoft Azure (2026-04-30): Azure Integrated HSM open-sourced — [azure.microsoft.com](https://azure.microsoft.com/en-us/blog/enforcing-trust-and-transparency-open-sourcing-the-azure-integrated-hsm/)
5. ABI Research (2026-08-04): HSM Market Data 2Q 2026 — [cpl.thalesgroup.com](https://cpl.thalesgroup.com/resources/encryption/hardware-security-modules-market-data-overview-2q-2026-report)
6. Utimaco: u.trust CSe-Series HSM — [utimaco.com](https://utimaco.com/data-protection/gp-hsm/utrust-general-purpose-hsm-cse-series)
7. IBM (2026-08-20): HSM Manager for IBM Z — [ibm.com](https://www.ibm.com/new/announcements/introducing-ibm-hsm-manager)

### Key Findings
- **FIPS 140-3 is the new standard** replacing FIPS 140-2 in 2026. All new HSMs targeting FIPS 140-3 Level 3.
- **Post-quantum cryptography (PQC) readiness** is now a primary HSM differentiator. Thales Luna 8 and Utimaco CSe both advertise PQC-upgradeable firmware.
- **Azure open-sourced Integrated HSM firmware** — hardware roots of trust, measured boot, attestation. Trust is verifiable, not contractual.
- **Google Cloud single-tenant Cloud HSM** — dedicated FIPS 140-2 L3 partitions, quorum-based admin, revocation capability. Customer owns root key.
- **External key management** (Azure) — keep key material physically outside cloud provider datacenters. For sovereignty/regulatory compliance.
- **HSM-backed keys have rotation limitations** — automatic KMS rotation often silently disabled for HSM-backed keys. Manual rotation required (from rotation orchestration research).

### NEW Defects Found for NeoTrix

| # | Defect | Severity | Detail |
|---|--------|----------|--------|
| 9 | **No hardware root of trust** | CRITICAL | NeoTrix KB encryption keys (if any) live in software. No HSM backing. A software compromise extracts all keys. 2026 FIPS 140-3 L3 is the baseline for regulated industries. |
| 10 | **No PQC readiness** | HIGH | NeoTrix has no post-quantum cryptography plan. When quantum computers break RSA/ECC, all current encryption is compromised. HSMs like Luna 8 offer PQC-upgradeable firmware; NeoTrix has nothing. |
| 11 | **No key custody chain** | HIGH | No quorum-based key ceremony. No split knowledge. No dual control. Single-person access to all cryptographic material. |
| 12 | **No attestation/verification** | MEDIUM | No hardware attestation, measured boot, or cryptographic proof that the runtime environment is trusted. Azure Integrated HSM makes trust verifiable; NeoTrix trust is purely contractual (if that). |
| 13 | **No sovereignty controls** | LOW | No mechanism to keep key material in specific geographic/legal jurisdictions. May matter for future compliance. |

---

## Topic 3: Key Rotation

### Sources
1. Digital Applied (2026-06-04): Secrets Management and Key Rotation Reference — [digitalapplied.com](https://www.digitalapplied.com/blog/secrets-management-api-key-rotation-2026-engineering-reference)
2. Reform.app (2026-06-21): Ultimate Key Rotation Policies Guide — [reform.app](https://www.reform.app/blog/ultimate-key-rotation-policies-guide)
3. Systems Hardening (2026-04-27): Secrets Rotation Orchestration — [systemshardening.com](https://www.systemshardening.com/articles/cross-cutting/secrets-rotation-orchestration/)
4. DevSecOps School (2026-02-20): What is Secret Rotation — [devsecopsschool.com](https://devsecopsschool.com/blog/secret-rotation/)
5. Safeguard.sh (2026-01-26): Credential Rotation Guide — [safeguard.sh](https://safeguard.sh/resources/blog/what-is-credential-rotation)
6. Valtik Studios (2026-01-23): Secrets Management Complete Guide — [valtikstudios.com](https://www.valtikstudios.com/blog/secrets-management-complete-guide-2026)
7. Decryption Digest (2026-06-19): Secrets Rotation in Production — [decryptiondigest.com](https://www.decryptiondigest.com/blog/secrets-rotation-live-production-without-outage)

### Key Findings
- **Rotation is solved; governance isn't.** The mechanical act of rotating is trivial. The hard part is: knowing all consumers, executing overlap windows, and having automation that runs without humans.
- **OWASP revoke-rotate-delete-log** is the incident response playbook for leaked secrets. Order matters: revoke first (stop the bleeding), then rotate, then delete from history, then log blast radius.
- **Dual-credential overlap window is mandatory** — both old and new must be valid simultaneously until every consumer has migrated. Most rotation outages are premature revocation.
- **NIST SP 800-63B** removed mandatory human password rotation in 2017 — but machine credentials (API keys, service accounts) still need fixed rotation windows.
- **Recommended intervals**: DB passwords 30-90 days, cloud IAM keys 90 days, API keys 30-180 days, TLS certs 90-398 days, SSH keys 3-6 months.
- **OIDC federation eliminates rotation entirely** for some credentials — short-lived tokens (60-min TTL) mean nothing to rotate.
- **HSM-backed keys silently skip automatic rotation** — KMS auto-rotation is often incompatible with HSM-backed keys. Requires manual rotation with calendar reminders.
- **Vault dynamic secrets have no rotation event** — credentials are ephemeral by design. Strongest pattern where supported.

### NEW Defects Found for NeoTrix

| # | Defect | Severity | Detail |
|---|--------|----------|--------|
| 14 | **No rotation state machine** | CRITICAL | No provision→propagate→verify→revoke sequence. No dual-credential windows. No overlap management. Any credential change is a manual, risky, all-or-nothing operation. |
| 15 | **No rotation orchestration** | HIGH | No controller/script that sequences rotation across DB password + Vault + app config. Cross-system rotations will be out of sync. |
| 16 | **No emergency revocation path** | HIGH | No pre-authorized emergency rotation procedure. If a credential leaks, there's no runbook to revoke it within 24 hours. OWASP says revoke FIRST. |
| 17 | **No rotation audit trail** | HIGH | No logs of who rotated what, when, with what approval. SOC 2 and PCI DSS auditors ask for rotation logs, not rotation policies. |
| 18 | **No canary rotation** | MEDIUM | No subset-first rotation with health checks before full rollout. Every rotation is a global flip — high blast radius if the new credential is bad. |
| 19 | **No rotation SLIs/SLOs** | MEDIUM | No measurable targets: rotation success rate (target 99.9%), time-to-rotate (target <5min for infra), auth error rate during rotation. |
| 20 | **No event-based rotation triggers** | MEDIUM | No automated response to: employee offboarding, vendor incident, suspected compromise, system migration. Only calendar-based (if any). |

---

## Summary: NEW Defects (Batch 728)

| # | Domain | Defect | Severity |
|---|--------|--------|----------|
| 1 | Secrets | No secrets management strategy | CRITICAL |
| 2 | Secrets | No dynamic credential tier | HIGH |
| 3 | Secrets | No rotation policy | HIGH |
| 4 | Secrets | No secret scanning (gitleaks/trufflehog) | HIGH |
| 5 | Secrets | No SOPS bootstrap layer | MEDIUM |
| 6 | Secrets | No OIDC federation | MEDIUM |
| 7 | Secrets | No consumer inventory | MEDIUM |
| 8 | Secrets | No pre-commit leak prevention | MEDIUM |
| 9 | HSM | No hardware root of trust | CRITICAL |
| 10 | HSM | No PQC readiness | HIGH |
| 11 | HSM | No key custody chain (quorum/split knowledge) | HIGH |
| 12 | HSM | No attestation/verification | MEDIUM |
| 13 | HSM | No sovereignty controls | LOW |
| 14 | Rotation | No rotation state machine | CRITICAL |
| 15 | Rotation | No rotation orchestration | HIGH |
| 16 | Rotation | No emergency revocation path | HIGH |
| 17 | Rotation | No rotation audit trail | HIGH |
| 18 | Rotation | No canary rotation | MEDIUM |
| 19 | Rotation | No rotation SLIs/SLOs | MEDIUM |
| 20 | Rotation | No event-based rotation triggers | MEDIUM |

**Critical defects**: 3 (secrets mgmt, HSM, rotation state machine)
**High defects**: 7
**Medium defects**: 8
**Low defects**: 1
**Total new defects**: 19 (plus 1 low)

**Cumulative across batches 727-728**: Zero backup + zero secrets + zero HSM + zero rotation = **the entire security stack is missing**.
