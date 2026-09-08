# Iteration Batch 584 — Security Audit, Penetration Testing & Supply Chain Vulnerability Landscape (2026)

**Date**: 2026-09-06
**Parent**: Batch 583 (macro system & code generation defects)
**Focus**: 2026 security audit methodology, red team tradecraft, supply chain attacks — identifying NEW defects/improvements over batch 583

---

## 1. Security Audit Findings — 2026

### Source: OSTIF Cortex Audit (2026-04), Datacendia Q1 Audit, ScorpioX Code Audit, OX Security SAST/SCA Report, LeanZero Forge App Audit

**New Defect D584-1: SAST/SCA Isolation Blindness (Runtime Context Gap)**
- Batch 583 covered build.rs caching and macro AST blindness. Neither SAST nor SCA sees **execution context** — which code paths are actually reachable, which dependencies are loaded at runtime, which cloud identities touch which data
- OX Security (2026-01): "SAST sees code flaws; SCA sees package flaws, but neither sees execution. They cannot tell which endpoints call which functions or whether a dependency is ever loaded in a running service."
- ASPM (Application Security Posture Management) is the 2026 answer: correlates SAST + SCA + runtime behavior + cloud identity + CI/CD metadata
- **NeoTrix defect**: `cargo audit` / `cargo deny` flag dependency CVEs but don't tell us if the vulnerable code path is ever invoked in NeoTrix. `clippy` + `cargo vet` don't trace data flow from external input to dangerous sink. We have no runtime reachability analysis.
- **vs batch 583**: D583-6 covered build.rs stale caching (silent incorrect binaries). D584-1 is the *complementary* blind spot: even correct binaries may link to vulnerable dependencies whose code paths are never reached — or conversely, reachable paths may lack SAST coverage.

**New Defect D584-2: AI-Generated Code as Attack Surface for SAST**
- SAST in 2026 must handle code written by Cursor, Claude, Copilot — developers reviewing code they didn't manually author
- OX Security: "Developers often review code they didn't manually author, and SAST becomes the first automated control capable of spotting easy-to-miss security mistakes before they enter a pipeline."
- AI-generated code has distinct vulnerability patterns: over-trusting inputs, missing bounds checks, hallucinated APIs, insecure defaults
- **NeoTrix defect**: NeoTrix itself generates code (build.rs codegen, SEAL pipeline macros, VSA HyperCube compilation). SAST tools don't understand that generated code is *machine-written* and may have systematic blind spots different from human code.
- **vs batch 583**: D583-1 covered derive macro AST blindness. D584-2 is the next layer: the *output* of code generation tools (including NeoTrix's own) is itself a new class of attack surface.

**New Defect D584-3: Decompression Bomb via Expansion Ratio (Cortex V04 + Forge App)**
- Cortex audit (2026-04): V04 — unbounded gzip decompression, small request expands to gigabytes, crashing server. CVSS 4.3. Fixed by wrapping gzip reader with `io.LimitReader`.
- Forge app audit (2026-07): 15MB download cap bounded the download, NOT the expansion. DEFLATE reaches ~1000:1 ratio. Parser fully expands inside 512MB function. Character truncation happened AFTER extraction completed — guarded nothing at parse time.
- **NeoTrix defect**: NeoTrix NT-WORLD handles web content fetching (UnifiedCrawler, fetchers). If any fetcher decompresses responses without limiting expanded size, a 1MB gzipped payload could expand to 1GB+ in memory. The Cortex and Forge patterns confirm this is a real 2026 attack vector, not theoretical.
- **vs batch 583**: No overlap with batch 583 (macro/codegen focus). This is a runtime vulnerability class that neither SAST nor macro analysis catches.

---

## 2. Penetration Testing & Red Team Findings — 2026

### Source: CISA AA26-237A "Two SOCs" (2026-08), Adayptus Red Team Guide (2026-05), Lorikeet Modern Playbook (2026-04), Rapid7 Multi-Agent Red Team (2026-07), Cyberange Airport Red Team (2026-05)

**New Finding D584-4: Alert Noise as Fundamental Security Deficiency (CISA)**
- CISA ran identical red tradecraft against two organizations. Org A: SOC saw nothing — overwhelmed by alert volume from untuned detection. Org B: isolated initial payload within 2-20 minutes.
- Root cause: "Without well-defined baselines and alert filtering, false positives and routine alerts overwhelm defenders, obscuring real threats."
- Both orgs had the same underlying weaknesses. Difference was detection signal quality, not control existence.
- **NeoTrix implication**: NeoTrix's NT-SHIELD (stealth net, proxy pool) generates network-level signals. If monitoring/alerting isn't tuned, security events become noise. NT-MEMORY's event bus must implement alert deduplication and baseline-aware filtering. No point having detection if SOC is drowned.

**New Finding D584-5: Modern Cloud Kill Chain — No Endpoint Compromise Required**
- Lorikeet (2026-04): "Modern cloud kill chains go: phishing → IdP token theft → Graph API access → SharePoint/Drive exfiltration → done. No endpoint compromise, no lateral movement, no EDR signal, no SIEM correlation."
- Red teams that focus exclusively on endpoint-first kill chains miss the actual risk in 2026.
- CISA confirmed: both orgs underestimated cloud risks, granted excessive permissions to cloud apps, lacked mature cloud compromise detection.
- **NeoTrix implication**: NeoTrix's NT-IO (LLM providers, API keys) and NT-ACT (MCP tools) interact with cloud services. Compromised API keys or OAuth tokens could enable cloud-only lateral movement without touching NeoTrix's own infrastructure. Need cloud identity monitoring even for a local-first toolkit.

**New Finding D584-6: AI Agent/MCP as Red Team Attack Surface (Emerging)**
- Lorikeet (2026-04): "The integration of AI agents and MCP servers into enterprise workflows is moving faster than the security discipline around them. Expect this to be a standard red team test category by 2027."
- Rapid7 (2026-07) built production multi-agent red team system using Claude Mythos — validated that AI agents can chain vulnerability analysis and exploit development.
- Attack vector: prompt injection → tool-call hijacking → lateral movement through agent's tool permissions.
- **NeoTrix defect**: NeoTrix is an AI agent with MCP tool access (nt_agent_mcp_gateway). If NeoTrix's tool-call routing doesn't validate instruction provenance, a prompt injection in web-fetched content could escalate to arbitrary tool execution. Batch 583 didn't consider the *agent itself* as an attack surface.
- **vs batch 583**: D583 covered macro/codegen blind spots. D584-6 is a completely new attack surface: the AI agent's own tool-calling is the vulnerability.

**New Finding D584-7: Supply Chain Attack via CI/CD Identity Hijacking (TanStack CVE-2026-45321)**
- TeamPCP compromised 42 TanStack packages by chaining: `pull_request_target` misconfiguration + GitHub Actions cache poisoning + runtime OIDC token extraction
- Published 84 malicious versions with **valid SLSA Build Level 3 attestations** — trust frameworks themselves were weaponized
- Dead man's switch: if developer revokes stolen npm token, wiper executes `rm -rf ~/`
- Spread to Mistral AI, UiPath, OpenSearch, Guardrails AI packages via same OIDC token hijacking
- **NeoTrix defect**: NeoTrix uses GitHub Actions for CI/CD. If any workflow uses `pull_request_target` with mutable version tags or unscoped OIDC trusted publishers, the same attack class applies. The wiper component means incident response must isolate BEFORE revoking tokens.
- **vs batch 583**: D583-6 was build.rs caching. D584-7 is supply chain compromise of the build infrastructure itself — a higher-order threat.

**New Finding D584-8: Rust Crate Supply Chain Attack (CVE-2026-77649 — `internment` 0.8.7)**
- The `internment` crate for Rust (0.8.7) contains a rogue dependency that registers with a C2 server to offer arbitrary code execution
- CVSS 9.8 CRITICAL. Published 2026-08-21. CWE-506: Embedded Malicious Code
- NeoTrix is a Rust project — any `cargo build` pulling `internment` 0.8.7 would execute malicious code at compile time
- **NeoTrix defect**: NeoTrix's `Cargo.lock` must be audited for this specific crate version. `cargo audit` only catches known CVEs — it doesn't detect *new* supply chain attacks in real-time. Need proactive dependency monitoring.
- **vs batch 583**: D583-2 covered proc macro crate isolation tax (build overhead). D584-8 is the *security* dimension of dependency management: even "trusted" crates can be compromised.

---

## 3. Vulnerability Scanning & Supply Chain Findings — 2026

### Source: NVD CVE-2026-33634 (Trivy), CVE-2026-45321 (TanStack), CVE-2026-77649 (Rust internment), CVE-2026-28407/24845 (malcontent), Black Kite 2026 Report, Red Hat RHSB-2026-001

**New Defect D584-9: Non-Atomic Credential Rotation Enables Persistent Compromise**
- Trivy attack (CVE-2026-33634): After initial disclosure on March 1, credential rotation was performed but was NOT atomic (not all credentials revoked simultaneously). Attacker used valid token to exfiltrate newly rotated secrets during rotation window.
- Result: March 19 re-compromise using credentials stolen during the rotation gap.
- **NeoTrix defect**: NeoTrix manages API keys for LLM providers (NT-IO), MCP tokens (NT-ACT), and potentially GitHub tokens. If key rotation is sequential rather than atomic, a compromised key can steal the replacement during the rotation window. Need atomic rotation protocol or temporary dual-revocation.
- **vs batch 583**: No overlap. D583 focused on build-time issues. D584-9 is a runtime credential management defect.

**New Defect D584-10: Supply Chain Scanner Evasion via Nested Archives**
- CVE-2026-28407: `malcontent` (Chainguard's supply-chain scanner) silently deletes nested archives that fail to extract, rather than scanning them. Attackers embed malicious content in deliberately malformed nested archives to bypass detection.
- CVE-2026-24845: `malcontent` leaks Docker registry credentials when scanning a malicious OCI image reference with forged `WWW-Authenticate` redirect.
- **NeoTrix defect**: If NeoTrix uses any supply-chain scanning tool (cargo-audit, cargo-deny, malcontent), the scanner itself may have bypasses. Scanning tools are part of the trust chain — a compromised scanner creates false confidence.
- **vs batch 583**: No overlap. This is meta-vulnerability: the security tools themselves are vulnerable.

**New Defect D584-11: EPSS/CVSS Prioritization Mismatch (Black Kite 2026)**
- 48,000+ CVEs published in 2025 (18% YoY increase). ~800 exploited in the wild. Only 58 were both OSINT-discoverable AND had EPSS > 60% ("Code Red").
- Traditional CVSS scoring treats a critical CVE in dead code the same as a low-severity flaw on a public endpoint.
- EPSS (Exploit Prediction Scoring System) estimates probability of exploitation within 30 days — far more operationally relevant than static CVSS.
- **NeoTrix defect**: NeoTrix's dependency audit workflow (if any) likely uses `cargo audit` which reports by CVSS severity. Should incorporate EPSS scores and CISA KEV data to prioritize. A "critical" CVE in an unused dependency path is lower priority than a "medium" CVE on a reachable code path.
- **vs batch 583**: D583 covered macro recursion limits and build caching. D584-11 is the vulnerability management priority problem.

**New Finding D584-12: SLSA Attestation Weaponized — Trust Framework Subverted**
- TanStack attack produced packages with **valid SLSA Build Level 3 provenance** — the very framework designed to prevent supply chain attacks was turned into a trust signal for malicious packages.
- The attacker didn't forge provenance; they abused legitimate CI/CD identity (OIDC trusted publisher) to generate valid attestations.
- **NeoTrix implication**: If NeoTrix ever publishes packages or artifacts with SLSA provenance, the provenance only proves the CI/CD pipeline ran — not that the pipeline wasn't compromised. SLSA is necessary but insufficient. Need additional verification: commit signing, branch protection, workflow file integrity checks.

---

## 4. NEW Defects vs Batch 583

| ID | Defect | Category | Severity |
|----|--------|----------|----------|
| D584-1 | SAST/SCA isolation blindness — no runtime reachability analysis | Security Tooling | **Critical** |
| D584-2 | AI-generated code as distinct SAST attack surface | Security Tooling | High |
| D584-3 | Decompression bomb via expansion ratio (15MB → 1GB+) | Runtime | High |
| D584-4 | Alert noise kills SOC detection (CISA "Two SOCs") | Detection | High |
| D584-5 | Cloud-only kill chain — no endpoint compromise needed | Architecture | **Critical** |
| D584-6 | AI agent tool-calling as attack surface (prompt injection → tool hijack) | Agent Security | **Critical** |
| D584-7 | Supply chain attack via CI/CD OIDC identity hijacking (CVE-2026-45321) | Supply Chain | **Critical** |
| D584-8 | Rust crate supply chain attack (`internment` 0.8.7, CVE-2026-77649) | Supply Chain | **Critical** |
| D584-9 | Non-atomic credential rotation enables persistent compromise | Credential Mgmt | High |
| D584-10 | Supply chain scanner evasion via nested archives (CVE-2026-28407) | Security Tooling | Medium |
| D584-11 | EPSS/CVSS prioritization mismatch — CVSS alone is insufficient | Vulnerability Mgmt | Medium |
| D584-12 | SLSA attestation weaponized — trust framework subverted | Supply Chain | High |

---

## 5. What's NEW vs Batch 583

| Dimension | Batch 583 | Batch 584 |
|-----------|-----------|-----------|
| **Focus** | Macro system, code generation, build.rs | Security audit, red team, supply chain |
| **Build defects** | Stale caching, host-vs-target trap | CI/CD OIDC hijacking, credential rotation gaps |
| **Dependency risks** | Proc macro crate overhead, syn/quote deps | Rogue crate C2 (`internment` 0.8.7), scanner bypasses |
| **Code analysis** | Derive macro AST blindness | SAST/SCA isolation blindness, no runtime reachability |
| **New attack surface** | Not considered | AI agent tool-calling, cloud-only kill chains |
| **Detection gaps** | Not covered | Alert noise as fundamental deficiency, EPSS vs CVSS mismatch |
| **Trust frameworks** | Not covered | SLSA attestation weaponized, supply chain scanner evasion |
| **Runtime vulns** | Not covered | Decompression bombs, unbounded memory allocation |
| **Threat model scope** | Code/build-time only | Extended to runtime, cloud, agent, supply chain |

---

## 6. Sources

1. OSTIF Cortex Security Audit (2026-04) — https://ostif.org/wp-content/uploads/2026/07/26-04-2696-REP-Cortex-security-audit-V1.1.pdf
2. Datacendia Q1 2026 Audit Report — https://github.com/datacendia/datacendia-core/blob/master/docs/AUDIT-REPORT-2026-Q1.md
3. ScorpioX Code Security Audit (2026-04-28) — https://code.scorpiox.net/security-audit-download?file=security-audit-report.pdf
4. OX Security: SAST vs SCA in 2026 (2026-01-05) — https://www.ox.security/blog/sast-vs-sca-2026/
5. LeanZero: Forge App SAST/SCA (2026-07-17) — https://leanzero.net/tutorials/security-reports-for-forge-apps-the-clean-scan-that-isnt-2026
6. ZeroPath: 7 Best SAST Tools 2026 (2026-03-04) — https://zeropath.com/blog/best-sast-tools
7. CISA AA26-237A: Two SOCs Red Team (2026-08-25) — https://www.cisa.gov/news-events/cybersecurity-advisories/aa26-237a
8. Adayptus: Red Team Attack Simulation Guide (2026-05-03) — https://www.adayptus.com/blog/red-team-attack-simulation-adversary-emulation-guide
9. Lorikeet: Modern Red Team Playbook 2026 (2026-04-18) — https://lorikeetsecurity.com/blog/modern-red-team-playbook-2026
10. Rapid7: Multi-Agent AI Red Team Architecture (2026-07-02) — https://www.rapid7.com/blog/post/so-red-teaming-offensive-methodology-multi-agent-ai-architecture/
11. Cyberange: 90-Day Airport Red Team (2026-05-27) — https://cyberange.io/insights/90-day-red-team-tier-1-indian-airport/
12. NVD: CVE-2026-33634 (Trivy Supply Chain) — https://nvd.nist.gov/vuln/detail/CVE-2026-33634
13. NVD: CVE-2026-45321 (TanStack Worm) — https://advisories.gitlab.com/npm/@tanstack/router-utils/CVE-2026-45321/
14. NVD: CVE-2026-77649 (Rust internment crate) — https://nvd.nist.gov/vuln/detail/cve-2026-77649
15. Armis: CVE-2026-28407 (malcontent nested archive bypass) — https://cve.armis.com/CVE-2026-28407
16. Armis: CVE-2026-24845 (malcontent Docker credential leak) — https://cve.armis.com/CVE-2026-24845
17. Red Hat: RHSB-2026-001 Supply Chain Compromises — https://access.redhat.com/security/vulnerabilities/RHSB-2026-001
18. Black Kite: 2026 Supply Chain Vulnerability Report — https://blackkite.com/reports/2026-supply-chain-vulnerability-report
19. RedEye: Mini Shai-Hulud Worm (SLSA-attested malware) — https://threat-intelligence.redeyesecurity.com/blog/mini-shai-hulud-worm-slsa-attested-supply-chain-attack-2026
20. Kerne Protocol: Independent Security Review June 2026 — https://github.com/kerne-protocol/contracts-public/blob/main/audits/INDEPENDENT_REVIEW_2026-06.md
21. Aardwolf Security: CISA Two-SOC Lessons (2026-08-29) — https://aardwolfsecurity.com/red-team-penetration-test-cisa-lessons/

---

## 7. Recommendations for NeoTrix

1. **Runtime Reachability Analysis**: Integrate ASPM-style analysis — correlate `cargo audit` findings with actual call graph data. Use `cargo llvm-lines` or `cargo bloat` to identify which dependency code is actually linked. Disable or pin dependencies whose vulnerable code paths are unreachable.

2. **Decompression Guard in NT-WORLD**: Add size-limiting wrapper around any decompression in web fetchers. Enforce max expanded size (e.g., 50MB) before content enters NT-MEMORY. Pattern: `io::Read::take(reader).limit(MAX_EXPANDED_SIZE)`.

3. **Atomic Credential Rotation**: Implement atomic key rotation protocol for all API keys (LLM providers, GitHub tokens, MCP tokens). If atomic rotation is impossible, implement dual-revocation: revoke old key BEFORE issuing new key, accepting temporary service disruption.

4. **Agent Tool-Call Provenance Validation**: In `nt_agent_mcp_gateway`, validate that tool-call instructions originate from trusted sources. Implement instruction provenance tracking: user-authored vs system-authored vs web-fetched. Gate dangerous tool executions (file writes, network calls) on provenance level.

5. **Dependency Supply Chain Hardening**:
   - Pin all GitHub Actions to full commit SHA (not mutable version tags)
   - Scope OIDC trusted publishers to specific protected branches + workflow files
   - Run `cargo audit` + `cargo deny` in CI with `--fail-on` severity threshold
   - Monitor for new Rust crate supply chain attacks (CISA KEV + crates.io advisories)
   - Audit `Cargo.lock` for `internment` 0.8.7 specifically

6. **Alert Baseline for NT-SHIELD**: If NT-SHIELD monitoring generates alerts, implement baseline-aware filtering. Tune detection to reduce noise so real threats aren't drowned. Follow CISA's pattern: "Organization B had an established baseline and a fine-tuned alert system."

7. **EPSS-Based Prioritization**: When triaging dependency CVEs, weight by EPSS score (exploitation probability within 30 days) rather than CVSS alone. CISA KEV inclusion is a stronger signal than CVSS critical rating.

8. **SLSA Provenance Verification**: If publishing NeoTrix artifacts with SLSA provenance, add secondary verification: GPG-signed commits, branch protection rules, workflow file integrity checks. SLSA proves pipeline ran — not that pipeline was uncompromised.
