# Iteration Batch 726 — IaC, Configuration Management, GitOps (2026)

**Date**: 2026-09-06
**Context**: Batch 725 proved zero backup strategy, no immutable backup, no DR plan, single DB = single corruption, KB stored plaintext.

---

## 1. IaC Findings (Terraform/Pulumi/OpenTofu 2026)

### Sources
- zop.dev: Infrastructure as Code Best Practices 2026 (Apr 2026)
- talkingtech.io: Pulumi vs Terraform: The IaC Showdown (Jun 2026)
- lucaberton.com: Terraform vs Pulumi 2026 (Apr 2026)
- barakahsoft.com: Terraform vs Pulumi for DevOps 2026 (Apr 2026)
- pdpspectra.com: Terraform vs Pulumi 2026 (May 2026)
- cloudtoolstack.com: Terraform vs Pulumi vs Crossplane (Mar 2026)
- envzero.com: Terraform Alternatives in 2026 (2026)
- guptadeepak.com: Top 5 IaC Tools 2026 (Aug 2026)

### Key Findings
1. **State file = plaintext secret graveyard**: All IaC tools (Terraform, OpenTofu, Pulumi) store outputs in state files. Database passwords, API keys, and private certificates are stored in **plaintext** unless explicitly encrypted. "Anyone with state file read access has those secrets."
2. **Pulumi per-value encryption is the only native solution**: Pulumi encrypts each secret value individually in state. Terraform only marks values `sensitive` (hides from CLI output) but state file contains them plaintext unless entire backend is encrypted.
3. **Module size matters**: Average Terraform module grows to 2,400 lines before teams split. At that size, `plan` takes 4-7 minutes, `apply` takes 12-20 minutes. Enforcing 500-line module limit = 3x faster plan/apply.
4. **Drift detection is manual**: 67% of teams using IaC experience drift. Detection requires scheduled plan runs every 4-6 hours.
5. **OpenTofu diverging from Terraform**: Feature sets are now diverging (provider-defined functions, state encryption). Migration window is narrowing.
6. **Terraform license fracture**: BSL 1.1 vs MPL 2.0 (OpenTofu) vs Apache 2.0 (Pulumi). Regulated environments may require open-source stack.

### NEW Defects Found
- **DEFECT-726-01**: NeoTrix KB is a single SQLite DB — if it were managed by IaC, the state file would contain ALL 7 domain schemas, all embeddings, all knowledge in plaintext. No per-value encryption.
- **DEFECT-726-02**: No IaC drift detection for KB schema changes. If someone manually modifies a table, there is no automatic detection or correction.
- **DEFECT-726-03**: NeoTrix has no module-size discipline. If KB schema grew as monolithic IaC, it would hit the 2,400-line threshold with degraded plan/apply performance.

---

## 2. Configuration Management Findings (Ansible/Salt/Puppet/Chef 2026)

### Sources
- computingforgeeks.com: Ansible vs Chef vs Puppet vs Salt (Apr 2026)
- falcao.org: Ansible, Puppet, Chef, SaltStack 2026 (May 2026)
- ansiblebyexample.com: Ansible vs Puppet vs Chef vs Salt (Mar 2026)
- thenewstack.io: Ansible vs Salt (Mar 2026)
- lucaberton.com: SaltStack vs Ansible 2026 (Jun 2026)
- lucaberton.com: Ansible vs Puppet 2026 (Apr 2026)
- releaserun.hashnode.dev: Configuration Management Tools Compared (2026)
- gitnux.org: Best Configuration Management Software 2026 (Jun 2026)

### Key Findings
1. **Ansible dominates new projects**: 31.94% market share for new projects. Agentless, SSH-based, no infrastructure overhead. "If starting fresh, choose Ansible."
2. **Salt is fastest at scale**: ZeroMQ transport — 10,000 nodes in 15 seconds vs Ansible's 60+ minutes. Event-driven automation via reactor system. BUT: Broadcom ownership uncertainty, community shrinking.
3. **Puppet continuous drift enforcement**: Agent runs every 30 minutes, auto-reverts unauthorized changes. "If someone manually changes a config file, the agent reverts it within 30 minutes."
4. **Chef InSpec compliance**: Still best-in-class for compliance-as-code (PCI, HIPAA, SOX). "Chef's InSpec compliance framework has no real equivalent in the Ansible world."
5. **IBM consolidation**: IBM now owns both Red Hat (Ansible) + HashiCorp (Terraform). "Consolidating Ansible + Terraform under one roof."
6. **Puppet license tightening**: Perforce shipping hardened binaries from private repos, tightening commit cadence. Community fork "Muppet" in discussion.
7. **Salt 3008.0rc4**: Next major Salt release in RC testing. Agentless mode (salt-ssh) as fallback.

### NEW Defects Found
- **DEFECT-726-04**: NeoTrix has zero configuration management for its 9 subsystem modules (NT-CORE through NT-FEEL). No idempotent configuration, no drift detection, no continuous enforcement. Each subsystem's configuration is ad-hoc.
- **DEFECT-726-05**: No compliance-as-code. NeoTrix handles KB with plaintext data but has no InSpec-equivalent or policy-as-code framework to enforce encryption standards.
- **DEFECT-726-06**: No event-driven automation for KB corruption. If KB corruption is detected, there is no reactor/beacon to trigger automatic remediation within seconds.

---

## 3. GitOps Findings (ArgoCD/Flux 2026)

### Sources
- iancloud.ai: GitOps in 2026 (Apr 2026)
- zakhassan.com: GitOps with Flux and ArgoCD (May 2026)
- k8s.guru: GitOps in 2026: Argo CD vs Flux (Feb 2026)
- askantech.com: GitOps Guide 2026 (Mar 2026)
- devopsness.com: Flux vs Argo CD 2026 (Jun 2026)
- portainer.io: ArgoCD vs Flux (Jul 2026)
- thegoodshell.com: GitOps Kubernetes Guide (May 2026)
- oneuptime.com: Flux CD vs ArgoCD Architecture (Mar 2026)

### Key Findings
1. **GitOps = 64% enterprise adoption** as primary delivery mechanism. Git is single source of truth. Cluster continuously reconciles.
2. **ArgoCD UI-first**: Rich web dashboard, Application CRD, SSO/RBAC built-in. App of Apps pattern. ~60% of Kubernetes clusters.
3. **Flux API-first**: Modular controllers (source, kustomize, helm, notification, image-automation). No built-in UI. Native Kubernetes RBAC via impersonation.
4. **Both CNCF Graduated**: Stable, production-ready. Support OCI registries, Helm, Kustomize.
5. **Progressive delivery is non-negotiable**: Flagger for canary/blue-green. Metrics-driven rollback (Prometheus/Datogad).
6. **Secrets in Git**: Use Sealed Secrets, External Secrets Operator, or SOPS. "Never commit raw secrets."
7. **Drift = compliance feature**: "When GitOps controller reports out-of-sync, you have evidence that actual cluster state doesn't match declared intent."
8. **Signed commits required**: "Anyone with write access to GitOps repo can trigger production deployment. Require signed commits."
9. **Overpermissioned cluster access**: Both ArgoCD and Flux install with cluster-admin by default. Must scope to managed namespaces.
10. **Reconciler failures are silent**: Sync failure may not surface in existing alerting unless instrumented specifically. "Deployment appears to succeed from CI perspective while cluster quietly ignores the change."

### NEW Defects Found
- **DEFECT-726-07**: NeoTrix has no GitOps model for its own infrastructure. There is no single source of truth for NeoTrix's cluster/infrastructure state. Changes are applied ad-hoc with no reconciliation.
- **DEFECT-726-08**: No signed commit verification on NeoTrix configuration repos. Any contributor can trigger changes without cryptographic verification.
- **DEFECT-726-09**: No progressive delivery for NeoTrix deployments. No canary analysis, no metrics-driven rollback. "Sync and pray" model.
- **DEFECT-726-10**: No drift detection alerting. NeoTrix has no way to know when its own infrastructure has drifted from intended state.
- **DEFECT-726-11**: Silent reconciliation failures. If NeoTrix's KB reconciliation fails (e.g., schema migration rejected), there is no alerting pipeline to surface this.

---

## Summary: 11 New Defects (Batch 726)

| ID | Category | Severity | Description |
|----|----------|----------|-------------|
| 726-01 | IaC | CRITICAL | KB state would contain all 7 domain schemas in plaintext; no per-value encryption |
| 726-02 | IaC | HIGH | No IaC drift detection for KB schema changes |
| 726-03 | IaC | MEDIUM | No module-size discipline; monolithic KB schema risk |
| 726-04 | ConfigMgmt | CRITICAL | Zero configuration management for 9 subsystem modules |
| 726-05 | ConfigMgmt | HIGH | No compliance-as-code or policy enforcement for data protection |
| 726-06 | ConfigMgmt | HIGH | No event-driven automation for KB corruption remediation |
| 726-07 | GitOps | CRITICAL | No GitOps model; no single source of truth for infrastructure |
| 726-08 | GitOps | HIGH | No signed commit verification on config repos |
| 726-09 | GitOps | HIGH | No progressive delivery; no canary analysis |
| 726-10 | GitOps | HIGH | No drift detection alerting for infrastructure |
| 726-11 | GitOps | CRITICAL | Silent reconciliation failures; no alerting on KB sync failures |

---

## Cumulative Critical Defects (Batch 725 + 726)

| Batch | Count | Categories |
|-------|-------|------------|
| 725 | 5 | Backup, DR, Encryption, Single Point of Failure, Plaintext |
| 726 | 11 | IaC (3), ConfigMgmt (3), GitOps (5) |
| **Total** | **16** | Infrastructure resilience |

## Next Steps (Batch 727)

1. Design IaC state management with per-value encryption for KB
2. Propose GitOps reconciliation model for NeoTrix subsystems
3. Define configuration management strategy for 9 modules
4. Research progressive delivery (Flagger/Argo Rollouts) integration patterns
5. Design signed commit + policy-as-code enforcement pipeline
