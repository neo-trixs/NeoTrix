# Iteration Batch 729 — Compliance Framework Research (SOC 2 / ISO 27001 / NIST CSF)

**Date**: 2026-09-06 | **Research Loop**: 729/10000+
**Context**: Builds on batch 728 (secrets management/HSM/key rotation). Maps industry compliance frameworks to identify compliance-gap defects NeoTrix inherits from having zero security infrastructure.

---

## Topic 1: SOC 2 Trust Services Criteria (2026)

### Sources
1. Konfirmity (2026-08-27): SOC 2 Updates 2026 — [konfirmity.com](https://www.konfirmity.com/blog/soc-2-what-changed-in-2026)
2. SOC2Auditors (2026-06-02): SOC 2 Trust Services Criteria — [soc2auditors.org](https://soc2auditors.org/insights/soc-2-trust-services-criteria)
3. CertPro (2026-05-12): SOC 2 Framework 2026 — [certpro.com](https://certpro.com/soc-2-framework-requirements-2026/)
4. Infosec-Conferences (2026): SOC 2 Explained — [infosec-conferences.com](https://infosec-conferences.com/security-domains/grc/frameworks/soc-2)
5. SOC2Auditors (2026-02-10): SOC 2 Compliance Guide 2026 — [soc2auditors.org](https://soc2auditors.org/soc-2-compliance/)
6. Venn (2026-06-08): SOC 2 Compliance in 2026 — [venn.com](https://www.venn.com/learn/soc2-compliance)
7. Strac (2026-08-21): SOC 2 Trust Services Criteria Complete Reference — [strac.io](https://www.strac.io/blog/soc-2-trust-services-criteria)

### Key Findings
- **64+ control points** required for SOC 2 Type II with continuous monitoring across monthly, quarterly, and annual cadences.
- **Privacy criterion (P1-P8) is now critical**: 20 US states have comprehensive consumer privacy laws (Indiana, Kentucky, Rhode Island effective Jan 1, 2026). Privacy scope is expanding.
- **SOC 2 is NOT a certification** — it's an auditor's report. Continuous compliance expected, not point-in-time.
- **Type II requires 3-12 month observation period** — proves operating effectiveness, not just design.
- **Mandatory Security (CC1-CC9)** + optional Availability, Processing Integrity, Confidentiality, Privacy.
- **Integration expected**: SOC 2 should map to ISO 27001, HIPAA, GDPR simultaneously.

### NEW Defects Found for NeoTrix

| # | Defect | Severity | Detail | Batch 728 Link |
|---|--------|----------|--------|----------------|
| 21 | **No control environment** | CRITICAL | SOC 2 CC1 requires documented control environment (tone at top, organizational structure, commitment to competence). NeoTrix has zero formal security governance — no CISO role, no security policy, no code of conduct. Batch 728 #1 (no secrets strategy) is a symptom; this is the root. |
| 22 | **No access control regime (CC6)** | CRITICAL | SOC 2 CC6 requires logical access controls, identity management, authentication mechanisms. NeoTrix has no RBAC, no access reviews, no principle of least privilege. Batch 728 #2 (no dynamic credentials) compounds this — all access is static and uncontrolled. |
| 23 | **No change management controls (CC8)** | HIGH | SOC 2 CC8 requires formal change management — approval workflows, testing, rollback procedures. NeoTrix has no change control gates. Any developer can push any change to any system with no review. |
| 24 | **No incident response plan** | HIGH | SOC 2 CC7 requires detection, monitoring, and incident response procedures. Batch 728 #16 (no emergency revocation) confirms: there is no incident response capability at all. No runbooks, no escalation, no blast radius assessment. |
| 25 | **No vendor management / third-party risk** | HIGH | SOC 2 CC9 requires vendor oversight. NeoTrix uses external LLM providers, cloud services, and npm crates with zero vendor risk assessment. No SOC 2 reports collected from vendors, no data processing agreements. |
| 26 | **No availability monitoring** | MEDIUM | SOC 2 Availability criterion requires uptime monitoring, disaster recovery, capacity planning. NeoTrix has no SLIs, no health dashboards, no DR plan. Batch 728 didn't cover availability at all — this is a new dimension. |
| 27 | **No processing integrity validation** | MEDIUM | SOC 2 Processing Integrity requires accuracy, completeness, timeliness of data processing. NeoTrix KB operations have no data validation, no integrity checks, no reconciliation processes. |
| 28 | **No privacy data mapping (P1-P8)** | MEDIUM | SOC 2 Privacy criterion requires notice, choice/consent, collection, use/retention/disposal, access, disclosure, quality, monitoring. NeoTrix collects user data with no privacy impact assessment, no consent mechanism, no data lifecycle management. 20 US states now enforce. |

---

## Topic 2: ISO 27001 / ISMS (2026)

### Sources
1. Konfirmity (2026-08-31): ISO 27001 What Changed in 2026 — [konfirmity.com](https://www.konfirmity.com/blog/iso-27001-what-changed-in-2026)
2. ISO.org (2026-07-03): ISO/IEC 27000:2026 Published — [iso.org](https://www.iso.org/standard/27000)
3. Continuum GRC (2026-09-02): ISO 27001 Governance & Compliance Audits 2026 — [continuumgrc.com](https://continuumgrc.com/iso-27001-updates-continuum-grc-governance-and-compliance-audits/)
4. EITT Academy (2026-05-11): IT Security Management Framework 2026 — [eitt.academy](https://eitt.academy/knowledge-base/information-technology-security-management-framework-2026/)
5. StrongDM (2025-10-17): ISO 27001 Requirements 2026 — [strongdm.com](https://www.strongdm.com/blog/iso-27001-requirements)
6. ANSI Blog (2026-08-06): ISO/IEC 27000:2026 Overview — [blog.ansi.org](https://blog.ansi.org/ansi/iso-iec-27000-2026-isms-overview/)
7. Drata (2026-08-05): ISO 27001 Security Guide 2026 — [drata.com](https://drata.com/learn/iso-27001/security)
8. ClearedSystems (2026-07-01): ISO 27001 Readiness 2026 — [clearedsystems.com](https://clearedsystems.com/blog/iso-27001-readiness-in-2026-key-updates-to-the-standard-you-need-to-know-before-you-start)

### Key Findings
- **ISO/IEC 27000:2026 published July 2026** — new overview standard (6th edition), no longer terminology-only, now covers concepts, principles, relationships. Horizontal document status.
- **ISO 27001:2022 is current** — no 2026 edition. 2024 climate-action amendment applies.
- **2013 certs expired Oct 31, 2025** — must be on 2022 edition with 93 Annex A controls (down from 114).
- **Cloud services control A.5.23 is new** — must explicitly address cloud platforms and vendor relationships in ISMS scope.
- **Third-party/supplier risk now central** — not peripheral. ISMS scope must cover vendor dependencies.
- **93 Annex A controls** across 4 themes: Organizational, People, Physical, Technological.
- **Must have defined metrics** — vague "periodic reviews" insufficient. Auditors expect measurement methods and documented results.
- **Certification cost**: $15K-60K, timeline 12-18 months.

### NEW Defects Found for NeoTrix

| # | Defect | Severity | Detail | Batch 728 Link |
|---|--------|----------|--------|----------------|
| 29 | **No ISMS — no security management system at all** | CRITICAL | ISO 27001 requires a documented ISMS with clauses 4-10 (context, leadership, planning, support, operation, performance evaluation, improvement). NeoTrix has zero formal security governance structure. This is the meta-defect: all batch 728 defects exist because there is no management system to detect or remediate them. |
| 30 | **No risk assessment methodology** | CRITICAL | ISO 27001 Clause 6.1 requires risk identification, analysis, evaluation, and treatment. NeoTrix has no risk register, no risk scoring, no risk acceptance criteria. Batch 728's entire defect list (secrets/HSM/rotation) should have been caught by a risk assessment. |
| 31 | **No asset inventory (A.5.9)** | HIGH | ISO 27001 Annex A.5.9 requires information assets to be identified and maintained. NeoTrix has no inventory of: what secrets exist, what databases hold data, what APIs are exposed, what third-party services are integrated. Can't protect what you don't know exists. |
| 32 | **No cryptographic controls documentation (A.8.24)** | HIGH | ISO 27001 Annex A.8.24 requires documented cryptographic controls — key management, algorithm selection, certificate management. Batch 728 #11 (no key custody) + this = NeoTrix has no cryptographic governance at all. |
| 33 | **No cloud scope definition (A.5.23)** | HIGH | New 2022 control A.5.23 requires explicit cloud services within ISMS boundary. NeoTrix uses external LLM providers (cloud) with no documented scope, no data processing agreements, no cloud security assessment. |
| 34 | **No metrics or measurement framework** | MEDIUM | ISO 27001 Clause 9.1 requires monitoring, measurement, analysis, evaluation. Auditors expect defined metrics with methods and results. NeoTrix has no security KPIs, no control effectiveness measurement, no trend analysis. |
| 35 | **No continual improvement process** | MEDIUM | ISO 27001 Clause 10 requires nonconformity handling and continual improvement. NeoTrix has no mechanism to learn from security incidents, near-misses, or audit findings. No corrective action process. |
| 36 | **No internal audit program** | MEDIUM | ISO 27001 Clause 9.2 requires internal audits at planned intervals. NeoTrix has no internal security audits, no independent review of controls, no self-assessment against Annex A controls. |

---

## Topic 3: NIST CSF 2.0 / Compliance Automation (2026)

### Sources
1. NIST.gov (2026-08-19): AI for CSF 2.0 Analysis QSG — [nist.gov](https://www.nist.gov/cyberframework)
2. NIST Updates Archive (2026): CSF 2.0 Updates — [nist.gov](https://www.nist.gov/cyberframework/updates-archive)
3. GokensAI (2026-04-02): NIST Compliance Checklist 2026 — [gokensai.com](https://gokensai.com/blog/nist-compliance-checklist-2026-complete-guide/)
4. SepioCyber (2026-02-02): NIST CSF Compliance Guide — [sepiocyber.com](https://sepiocyber.com/wp-content/uploads/2026/02/NIST-Cybersecurity-Framework-Compliance-Guide-1-2-26.pdf)
5. CSRC (2026): NIST 2026 Updates — [csrc.nist.gov](https://csrc.nist.gov/news/2026)
6. Continuum GRC (2026-04-25): NIST CSF Compliance 2026 Platform Comparison — [continuumgrc.com](https://continuumgrc.com/audit-compliance-solutions-csf/)
7. NIST (2026-08-19): AI for CSF 2.0 Analysis Draft QSG — [csrc.nist.gov](https://csrc.nist.gov/News/2026/using-ai-for-csf-2-analysis-reporting-draft-qsg)
8. Strata (2026-01-13): NIST CSF 2026 Overview — [strata.io](https://www.strata.io/glossary/nist-cybersecurity-framework-csf)

### Key Findings
- **NIST CSF 2.0 has 6 functions** (not 5): Govern → Identify → Protect → Detect → Respond → Recover. "Govern" is new — enterprise risk management, supply chain risk, roles/responsibilities.
- **NIST SP 1353**: AI for CSF 2.0 Analysis and Reporting — Quick-Start Guide (Aug 2026, comment period through Oct 15, 2026). AI-assisted compliance analysis is now officially endorsed.
- **Ransomware Risk Management Profile** (IR 8374r1) — CSF 2.0 community profile for ransomware defense.
- **Multi-Cloud Architecture Challenges** (IR 8613) — security/ATO challenges unique to multi-cloud.
- **Firmware-Based Monitoring** (CSWP 52) — low-cost hardware security visibility for bus-based systems.
- **SP 800-53 Rev 5.2.0** alignment — CSF 2.0 maps to SP 800-53 controls bidirectionally.
- **Post-quantum cryptography** — PIV standards updates for PQC credentials (June 2026).
- **SCAP 1.4** — Security Content Automation Protocol for compliance automation.
- **Compliance automation platforms** (Drata, Vanta, Secureframe, Continuum GRC) now offer AI-auditor capabilities, automated evidence collection, continuous monitoring.

### NEW Defects Found for NeoTrix

| # | Defect | Severity | Detail | Batch 728 Link |
|---|--------|----------|--------|----------------|
| 37 | **No Govern function (new in CSF 2.0)** | CRITICAL | NIST CSF 2.0 added "Govern" as the top-level function — enterprise risk management, supply chain risk, roles, policies, legal/regulatory requirements. NeoTrix has zero governance: no security policies, no risk appetite, no role assignments, no legal compliance mapping. This is the structural root cause of ALL batch 728 defects. |
| 38 | **No supply chain risk management** | HIGH | NIST CSF 2.0 Govern function requires supply chain risk management. NeoTrix depends on npm crates, LLM providers, cloud services with zero supply chain security: no SBOM, no vendor security assessments, no software supply chain integrity verification. |
| 39 | **No detect capability** | HIGH | NIST CSF 2.0 Detect function requires continuous monitoring, anomaly detection, adverse event analysis. NeoTrix has no intrusion detection, no anomaly detection, no security event logging. Batch 728 #17 (no rotation audit trail) is a subset — there's no audit trail for ANY security event. |
| 40 | **No respond/recover capability** | HIGH | NIST CSF 2.0 Respond+Recover functions require incident response planning, communications, analysis, mitigation, improvements, and recovery planning. NeoTrix has zero incident response capability. No containment, no eradication, no recovery procedures. |
| 41 | **No compliance automation** | MEDIUM | 2026 standard: AI-auditor platforms (Drata, Vanta, Continuum GRC) automate evidence collection, continuous monitoring, and framework mapping. NeoTrix has no compliance tooling — every compliance check is manual ad-hoc if it happens at all. |
| 42 | **No SBOM / software supply chain integrity** | MEDIUM | NIST SP 800-218 (SSDF) and EO 14028 require SBOMs. NeoTrix has no software bill of materials for any dependency. Cannot identify vulnerable components, cannot respond to CVE disclosures, cannot prove provenance. |
| 43 | **No post-quantum migration plan** | MEDIUM | NIST PQC standards (ML-KEM, ML-DSA, SLH-DSA) finalized. NIST CSF 2.0 maps to PQC readiness. NeoTrix has no PQC migration plan. When quantum-capable systems arrive, all current crypto is compromised. Batch 728 #10 confirmed no PQC readiness; this adds the NIST-specific requirement. |
| 44 | **No NIST CSF Profile defined** | LOW | NIST CSF is voluntary but organizations define Current Profile + Target Profile + Gap Analysis. NeoTrix has no CSF profile — doesn't know where it stands against even voluntary baseline. |

---

## Summary: NEW Defects (Batch 729)

| # | Domain | Defect | Severity |
|---|--------|--------|----------|
| 21 | SOC 2 | No control environment (CC1) | CRITICAL |
| 22 | SOC 2 | No access control regime (CC6) | CRITICAL |
| 23 | SOC 2 | No change management controls (CC8) | HIGH |
| 24 | SOC 2 | No incident response plan (CC7) | HIGH |
| 25 | SOC 2 | No vendor management / third-party risk (CC9) | HIGH |
| 26 | SOC 2 | No availability monitoring | MEDIUM |
| 27 | SOC 2 | No processing integrity validation | MEDIUM |
| 28 | SOC 2 | No privacy data mapping (P1-P8) | MEDIUM |
| 29 | ISO 27001 | No ISMS at all | CRITICAL |
| 30 | ISO 27001 | No risk assessment methodology | CRITICAL |
| 31 | ISO 27001 | No asset inventory (A.5.9) | HIGH |
| 32 | ISO 27001 | No cryptographic controls documentation (A.8.24) | HIGH |
| 33 | ISO 27001 | No cloud scope definition (A.5.23) | HIGH |
| 34 | ISO 27001 | No metrics or measurement framework | MEDIUM |
| 35 | ISO 27001 | No continual improvement process | MEDIUM |
| 36 | ISO 27001 | No internal audit program | MEDIUM |
| 37 | NIST CSF | No Govern function (new in CSF 2.0) | CRITICAL |
| 38 | NIST CSF | No supply chain risk management | HIGH |
| 39 | NIST CSF | No detect capability | HIGH |
| 40 | NIST CSF | No respond/recover capability | HIGH |
| 41 | NIST CSF | No compliance automation | MEDIUM |
| 42 | NIST CSF | No SBOM / software supply chain integrity | MEDIUM |
| 43 | NIST CSF | No post-quantum migration plan | MEDIUM |
| 44 | NIST CSF | No NIST CSF Profile defined | LOW |

**Critical defects**: 6 (new) + 3 (batch 728) = **9 cumulative CRITICAL**
**High defects**: 9 (new) + 7 (batch 728) = **16 cumulative HIGH**
**Medium defects**: 9 (new) + 8 (batch 728) = **17 cumulative MEDIUM**
**Low defects**: 1 (new) + 1 (batch 728) = **2 cumulative LOW**

---

## Cross-Frame Synthesis: What's NEW Beyond Batch 728

Batch 728 proved the **technical** security stack is missing (secrets/HSM/rotation). Batch 729 proves the **governance** stack is also missing — and governance is the root cause.

### Root Cause Chain

```
No ISMS (ISO 29) → No risk assessment (30) → No control environment (21)
→ No Govern function (37) → No access control (22) → No secrets strategy (#1)
→ No rotation state machine (#14) → No incident response (24)
→ No recovery capability (40) → TOTAL SECURITY FAILURE
```

### The Governance Deficit

| Framework | Governance Requirement | NeoTrix Status |
|-----------|----------------------|----------------|
| SOC 2 CC1 | Control environment, tone at top, commitment to competence | ABSENT |
| ISO 27001 Cl.5 | Leadership, organizational roles, responsibilities | ABSENT |
| NIST CSF 2.0 Govern | Enterprise risk management, supply chain, policies, legal | ABSENT |
| ISO 27001 Cl.6 | Risk assessment, risk treatment, risk acceptance | ABSENT |
| NIST SP 800-53 | Security control baseline (CM, AC, AU, IR families) | ABSENT |

### New Dimensions NOT Covered in Batch 728

1. **Privacy compliance** (SOC 2 P1-P8, 20 US state laws) — batch 728 had zero privacy coverage
2. **Vendor/supply chain risk** (SOC 2 CC9, ISO A.5.23, NIST Govern) — batch 728 focused only on internal secrets
3. **Change management** (SOC 2 CC8) — batch 728 didn't address deployment controls
4. **Availability/DR** (SOC 2 Availability, NIST Recover) — batch 728 didn't address uptime
5. **Metrics/measurement** (ISO Cl.9.1) — batch 728 had no measurement framework
6. **Compliance automation** (NIST SP 800-126r4 SCAP, AI for CSF) — batch 728 didn't address tooling

### Key 2026 Regulatory Shifts

| Shift | Impact on NeoTrix |
|-------|-------------------|
| 20 US state privacy laws | Must implement consent, data mapping, DPIA — or face enforcement |
| ISO 27000:2026 (July) | New overview standard emphasizing relationships — NeoTrix must align |
| NIST CSF 2.0 "Govern" function | Mandatory governance layer — NeoTrix has none |
| FIPS 140-3 replacing 140-2 | All new HSM deployments must target 140-3 L3 |
| PQC standards finalized | ML-KEM, ML-DSA, SLH-DSA — migration window is NOW |
| AI-assisted compliance (SP 1353) | Compliance automation is the 2026 baseline — manual is falling behind |

---

**Next iteration (730)**: Should explore **compliance automation tooling** (Drata/Vanta/Secureframe) and **ISMS implementation patterns** to begin closing the governance gap.
