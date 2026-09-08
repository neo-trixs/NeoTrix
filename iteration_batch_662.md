# Iteration Batch 662 — Zero Trust / IAM / Passwordless Research

**Date**: 2026-09-06
**Context**: Building on Batch 661 defect findings (no insight→action pipeline, no NLQ interface, no service dependency graph from traces, prescriptive dashboards not descriptive).

---

## 1. Zero Trust (ZTNA) — 2026 State

### Sources
- NCSC UK: Zero Trust Network Access guidance (May 2026) — https://www.ncsc.gov.uk/sites/default/files/2026-05/NCSC-Zero-Trust-Network-Access-%28ZTNA%29_0.pdf
- Exabeam: Zero Trust in 2026 Principles & Best Practices (Nov 2025) — https://www.exabeam.com/explainers/zero-trust/zero-trust-in-2026-principles-technologies-and-best-practices/
- Decryption Digest: Zero Trust Network Architecture 2026 Implementation Roadmap (Jun 2026) — https://www.decryptiondigest.com/blog/guide-zero-trust-network-architecture
- ZeonEdge: Zero Trust Network Access in 2026 (Mar 2026) — https://zeonedge.com/lb/blog/zero-trust-network-access-2026-architecture-implementation-pitfalls
- TheWeekGeek: Zero Trust Architecture 2026 IT Implementation Guide (Apr 2026) — https://theweekgeek.com/cybersecurity/ai-threats/zero-trust-architecture-2026/

### Key Findings
1. **ZTNA market reached $48.43B in 2026**, growing 16% annually. 81% of orgs actively implementing. 50% lower breach impact costs, 83% faster incident response (TheWeekGeek).
2. **CISA 5-Pillar model** (Identity, Devices, Networks, Applications, Data) with 3 maturity stages each: Traditional → Advanced → Optimal. Identity is the foundational pillar — start with phishing-resistant MFA (Decryption Digest).
3. **NCSC defines 8 ZTNA design requirements**: explicit authorization before connection, multi-signal policy decisions, application-level exposure reduction, network segment isolation, continuous verification, OAuth 2.0/OIDC over SAML for modern architectures (NCSC).
4. **Service-to-service trust is the blind spot** — most "zero trust" implementations authenticate humans but leave microservices freely communicating. SPIFFE/SPIRE workload identity is the fix. Mutual TLS between services required (ZeonEdge).
5. **Full ZTNA implementation: 3-5 years for large enterprises**, but 6-12 months for meaningful risk reduction by focusing on highest-impact controls first (Decryption Digest).

### NEW Defects for NeoTrix

| # | Defect | Evidence | Severity |
|---|--------|----------|----------|
| ZT-1 | **NT-SHIELD lacks policy decision point (PDP) abstraction** — current access control is ad-hoc per-module, not centralized PDP/PEP architecture. ZTNA demands every access request evaluated against policy BEFORE connection. | NCSC §2, ZeonEdge architecture diagram | HIGH |
| ZT-2 | **No continuous verification loop** — NeoTrix authenticates once at session start but never re-evaluates trust signals mid-session. ZTNA requires re-evaluation on risk change. | NCSC requirement #6, Decryption Digest continuous verification section | HIGH |
| ZT-3 | **Service-to-service identity absent** — NT-* domain modules communicate without mutual TLS or workload identity. Compromised module can freely reach any other. | ZeonEdge "Mistake 2", SPIFFE/SPIRE guidance | CRITICAL |
| ZT-4 | **No microsegmentation model** — all modules run in a flat trust domain. No east-west traffic control between NT-CORE, NT-WORLD, NT-ACT etc. | Decryption Digest microsegmentation section, CISA model | HIGH |
| ZT-5 | **Multi-signal policy decisions not implemented** — access decisions don't factor device posture, behavioral risk, or session context. | NCSC "multiple signals" requirement, ZeonEdge policy engine | MEDIUM |

---

## 2. Identity Management (IAM) — 2026 State

### Sources
- Forrester: Top Trends Shaping IAM in 2026 (Jun 2026) — https://www.forrester.com/report/the-top-trends-shaping-identity-and-access-management-in-2026/RES196378
- ISG: Buyers Guide IAM Platforms 2026 (Aug 2026) — https://research.isg-one.com/buyers-guide/information-technology/cybersecurity/identity-and-access-management-platforms/2026
- TechVision Research: Future of Identity Management 2026-2029 (May 2026) — https://techvisionresearch.com/knowledge-base/the-future-of-identity-management-2026-2029/
- CNCF: IAM Whitepaper (Jun 2026) — https://www.cncf.io/blog/2026/06/04/identity-and-access-management-whitepaper/
- askmeidentity: Complete Guide to IAM 2026 (May 2026) — https://askmeidentity.com/resources/complete-guide-to-iam/

### Key Findings
1. **AI agent identities are the #1 IAM priority in 2026** (Forrester). AI agents need their own identity primitives: short-lived, scoped, delegated, auditable. No vendor provides complete identity control plane including AI governance yet (TechVision).
2. **Non-human identity explosion** — API keys, service accounts, workload identities now outnumber human identities. Unified governance across human + non-human is critical (Forrester, TechVision Priority #2).
3. **Identity is now the central control plane** — no longer a supporting function. Organizations govern users, AI systems, APIs, machines, processes, and data simultaneously through identity (TechVision).
4. **Authorization externalization** — 2026 pattern is policy-as-code (OPA/Rego, AWS Cedar, SpiceDB/Zanzibar). Application asks PDP "is this allowed?" — policy lives outside app (askmeidentity).
5. **Identity Data Fabric** — real-time unified layer aggregating identity data from multiple sources, maintaining relationships, providing contextual information for decision-making (TechVision).
6. **SPIFFE for workload identity** — cloud-native standard for service-to-service authentication using short-lived SPIFFE identities (CNCF whitepaper).

### NEW Defects for NeoTrix

| # | Defect | Evidence | Severity |
|---|--------|----------|----------|
| IAM-1 | **No AI agent identity model** — NT-MIND's SEAL pipeline agents, NT-ACT's orchestration agents, NT-IO's LLM interactions have no identity primitives (scoped permissions, delegation chains, audit trails). | Forrester #1, TechVision Priority #1 & #3 | CRITICAL |
| IAM-2 | **Non-human identity unmanaged** — MCP tool credentials, LLM API keys, KB access tokens, EventBus subscriptions lack lifecycle management (rotation, expiry, revocation). | Forrester NHI trend, TechVision Priority #2 | HIGH |
| IAM-3 | **No externalized authorization** — access control logic is embedded in each module's code, not in a policy engine. Violates PDP/PEP separation. | askmeidentity PDP pattern, CNCF whitepaper | HIGH |
| IAM-4 | **No identity data fabric** — identity signals (user role, module access, capability level, trust tier) scattered across config files, KB entries, and hardcoded checks. No unified real-time identity context layer. | TechVision Priority #9 | MEDIUM |
| IAM-5 | **No joiner/mover/leaver lifecycle for modules** — when modules gain/lose capabilities or cross maturity levels (C0→C5), access permissions aren't automatically recalculated. | askmeidentity IGA lifecycle, TechVision IGA reinvention | MEDIUM |

---

## 3. Passwordless / Passkeys / FIDO2 — 2026 State

### Sources
- FIDO Alliance: State of Passkeys 2026 (May 2026) — https://fidoalliance.org/wp-content/uploads/2026/05/The-State-of-Passkeys-Global-Consumer-and-Workforce-Report-1.pdf
- FIDO Alliance: Five Billion Passkeys announcement (May 2026) — https://fidoalliance.org/fido-alliance-reports-accelerating-global-passkey-adoption-on-world-passkey-day-2026/
- Microsoft Security Blog: World Passkey Day 2026 (May 2026) — https://www.microsoft.com/en-us/security/blog/2026/05/07/world-passkey-day-advancing-passwordless-authentication/
- MojoAuth: State of Passwordless Authentication 2026 (Jan 2026) — https://mojoauth.com/data-and-research-reports/state-of-passwordless-2026/
- FIDO Alliance: Passkeys specification (Apr 2026) — https://fidoalliance.org/passkeys-2/

### Key Findings
1. **5 billion passkeys now in active use globally**. 90% consumer awareness, 75% have enabled on at least one account, 49% use regularly. 68% of organizations deploying for workforce (FIDO Alliance).
2. **Deploying passkeys ≠ eliminating passwords** — 57% of organizations that deployed passkeys still rely on phishable auth for primary day-to-day sign-in. The gap between deployment and operational adoption is real (FIDO Alliance).
3. **Passkeys hit 412% YoY growth** in 2025, fastest-growing auth method. Now 15.7% of all authentications. Predicted 28.4% by end 2026. Magic links still dominate at 41% due to universal compatibility (MojoAuth).
4. **Microsoft eliminated 99.6% of legacy auth internally**, now 100% phishing-resistant. Entra passkeys on Windows GA late May 2026. Removing security questions as reset option Jan 2027 (Microsoft).
5. **Recovery flows are the backdoor** — 89% of orgs confident in passkey recovery, but account recovery remains the #1 attack surface. Microsoft adding government-ID + biometric face verification for recovery (Microsoft).
6. **Device-bound vs. synced passkeys** — 50% of deploying orgs use mixed strategy. Europe splits evenly (26% synced-only, 26% device-bound-only). Compliance environments require device-bound (FIDO Alliance).

### NEW Defects for NeoTrix

| # | Defect | Evidence | Severity |
|---|--------|----------|----------|
| PW-1 | **No passkey/FIDO2 auth path** — NeoTrix CLI and Tauri desktop use password or token auth. No WebAuthn/passkey integration for human users. | FIDO Alliance 5B milestone, MojoAuth 412% growth | HIGH |
| PW-2 | **No device-bound credential model** — for high-assurance contexts (KB write, SEAL pipeline execution), no device-bound passkey support. Only synced tokens. | FIDO Alliance device-bound vs. synced split | MEDIUM |
| PW-3 | **Recovery flow not hardened** — no government-ID + biometric verification for account recovery. No multi-party recovery ceremony. | Microsoft recovery flow strengthening, FIDO Alliance recovery confidence data | HIGH |
| PW-4 | **Session re-authentication absent** — once authenticated, never re-verified. ZTNA + passwordless both demand step-up auth for high-sensitivity actions (KB writes, SEAL execution, cross-domain trust changes). | Microsoft step-up auth pattern, ZTNA continuous verification | HIGH |
| PW-5 | **Non-human identity auth weaker than human auth** — MCP tools, API keys, service accounts use static secrets while human auth moves to FIDO2. Security asymmetry. | Forrester NHI gap, TechVision Priority #8 | MEDIUM |

---

## Cross-Domain Synthesis: NEW Architectural Defects

| # | Defect | Combines | Severity |
|---|--------|----------|----------|
| XD-1 | **No unified trust evaluation engine** — ZT-1 + IAM-3 + PW-4 converge on: NeoTrix needs a single PDP that evaluates identity + device + session + behavioral signals for every access decision. | ZT-1, IAM-3, PW-4 | CRITICAL |
| XD-2 | **No agent identity lifecycle** — IAM-1 + ZT-3 + PW-5 converge on: SEAL pipeline agents, MCP tools, and cross-module services all need scoped, short-lived, auditable identities with automatic expiry. | IAM-1, ZT-3, PW-5 | CRITICAL |
| XD-3 | **No continuous trust re-evaluation** — ZT-2 + PW-4 + IAM-5 converge on: trust must be re-evaluated on risk change, not just at session start. Module capability changes should trigger access recalculation. | ZT-2, PW-4, IAM-5 | HIGH |
| XD-4 | **No recovery ceremony** — PW-3 + IAM-5 + ZT-1 converge on: module/module identity loss, KB key rotation, session invalidation all need a hardened multi-party recovery protocol. | PW-3, IAM-5, ZT-1 | HIGH |

---

## Summary

**Sources cited**: 13 unique sources across 5 web searches.
**New defects found**: 19 total (5 ZT + 5 IAM + 5 PW + 4 cross-domain).
**CRITICAL**: 3 (ZT-3 service-to-service, IAM-1 agent identity, XD-1 trust engine, XD-2 agent lifecycle).
**HIGH**: 8.
**MEDIUM**: 6.
**Remaining from prior batches**: Batch 661's 4 defects (no insight→action, no NLQ, no service dependency graph, prescriptive dashboards) remain unresolved.

**Next iteration priority**: Design the Trust Evaluation Engine (XD-1) as the architectural keystone — it resolves ZT-1, IAM-3, PW-4, and enables ZT-2 continuous verification. Consider SPIFFE/SPIRE for workload identity (XD-2).
