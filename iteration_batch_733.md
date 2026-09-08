# Iteration Batch 733 — NeoTrix Consciousness Architecture Research

**Date**: 2026-09-07
**Predecessor**: Batch 732 (governance maturity, agent-first registry, cross-protocol gap, semantic layer prerequisite, no open knowledge format)
**Domain**: Sandbox Isolation + Runtime Security + Container Isolation

---

## 1. SANDBOXING — Findings

### 1.1 gVisor vs Firecracker 2026 Production Maturity
**Source**: [Safeguard.sh — gVisor vs Firecracker 2026](https://safeguard.sh/resources/blog/gvisor-vs-firecracker-2026)

**Key data points**:
- Firecracker TCB: ~50,000 lines of Rust (microVM VMM)
- gVisor TCB: ~250,000 lines of Go (userspace kernel)
- Firecracker CVE history: handful of low-severity issues
- gVisor CVE history: more CVEs, mostly in userspace kernel, not isolation layer
- gVisor overhead: 2-5% CPU-bound, 15-30% filesystem-heavy, 30-50% network (UDP)
- Firecracker overhead: 3-8% steady-state, ~125ms boot, ~5MB per VM

**NEW DEFECT FOUND — NT-SHIELD-0733-A: No Tiered Isolation Policy Engine**
NeoTrix has no mechanism to classify workloads into isolation tiers (trusted → gVisor → Firecracker) based on provenance/TPRM score. The 2026 pattern is:
- Tier 0: Standard containers (trusted code)
- Tier 1: gVisor (medium trust, syscall interception)
- Tier 2: Firecracker microVMs (untrusted/multi-tenant)
- Admission controller maps workload labels → RuntimeClass per tier

**Implication**: NT-SHIELD's sandbox module currently uses a flat isolation model. Needs a TrustTier classifier that accepts SBOM provenance, supplier TPRM scores, and reachability analysis to route workloads to appropriate isolation.

### 1.2 Multi-Agent gVisor Isolation (MAGI) — Agent Sandboxing Architecture
**Source**: [gVisor Blog — MAGI (April 2026)](https://gvisor.dev/blog/2026/04/15/magi-multi-agent-gvisor-isolation/)

**Key architectural insight**: Google's MAGI pattern demonstrates:
- Each agent component sandboxed separately (inference, tools, browser, code execution)
- Policy engine lives OUTSIDE core sandbox (agent cannot modify its own policies)
- Credentials live outside core sandbox (API keys, crypto wallets)
- Destructive changes can be rolled back via checkpoint/restore
- gVisor's checkpoint/restore enables fast replay of slow-to-initialize actions

**NEW DEFECT FOUND — NT-SHIELD-0733-B: No Agent-Internal Sandbox Decomposition**
NeoTrix's sandbox architecture treats the entire agent as a single sandbox boundary. MAGI proves that agents need INTERNAL decomposition:
- Core daemon sandbox (agent brain — can self-modify within boundary)
- Tool execution sandboxes (each tool call separately sandboxed)
- Subsystem sandboxes (long-running daemons like signal-cli, browser)
- Policy engine OUTSIDE all sandboxes (immutable)
- Credential store OUTSIDE all sandboxes

**Implication**: NT-SHIELD needs a recursive sandbox model — sandboxes within sandboxes — not a single perimeter.

### 1.3 Ray Sandboxing on GKE — Distributed Agent Isolation
**Source**: [Google Cloud Blog — gVisor Sandboxes for Ray Clusters (August 2026)](https://cloud.google.com/blog/products/containers-kubernetes/gvisor-sandboxes-for-ray-clusters-on-gke)

**Key insight**: Google and Anyscale shipped native gVisor sandboxing into Ray for agentic RL workloads:
- Each sandbox represented as a Ray Actor (resource placement, lifecycle, scaling)
- Sub-second sandbox startup
- OCI-compatible images, no Docker daemon exposure
- Sandbox API: create/set-limits/exec/read-write/terminate

**NEW DEFECT FOUND — NT-ACT-0733-C: No Distributed Sandbox Lifecycle for Agent Fleets**
NeoTrix lacks a mechanism to manage sandbox lifecycles across distributed agent fleets. The Ray pattern shows sandboxes should be:
- Represented as first-class resource objects (like Ray Actors)
- Schedulable across cluster nodes with CPU/memory reservation
- Recoverable from failures via checkpoint/restore
- Scalable with the surrounding workload

### 1.4 Hybrid Isolation Pattern — The Emerging Standard
**Source**: [TURION.AI — Agent Sandboxing Production Isolation (May 2026)](https://turion.ai/blog/agent-sandboxing-firecracker-gvisor-microvm-architecture/)

**Key framework**:
- Layer 1: Network boundary (isolated namespace, allowlist-only egress)
- Layer 2: Filesystem boundary (tmpfs/ephemeral, quarantine writes)
- Layer 3: Execution boundary (Firecracker/gVisor)
- Layer 4: Tool access scoping (scoped short-lived tokens, OAuth2)
- Layer 5: Audit logging (every syscall crossing boundary)

**NEW DEFECT FOUND — NT-SHIELD-0733-D: Missing Layers 4 and 5 in NT-SHIELD**
NT-SHIELD's sandbox implementation covers layers 1-3 but is missing:
- **Layer 4 (Tool Access Scoping)**: No mechanism to issue scoped, short-lived tokens per tool invocation. NT-ACT tools currently inherit broad credentials from the agent's environment.
- **Layer 5 (Audit Logging)**: No syscall-level audit trail crossing the isolation boundary. When an agent does something malicious, the chain of events cannot be reconstructed.

---

## 2. RUNTIME SECURITY — Findings

### 2.1 Seccomp + AppArmor: Still Criminally Underused in 2026
**Source**: [Anuragh KP — Seccomp and AppArmor (June 2026)](https://iamanuragh.in/blog/2026-06-29-seccomp-apparmor-container-runtime-security/)

**Key data points**:
- Docker default seccomp blocks ~44 dangerous syscalls
- Custom profiles should allow only 50-60 syscalls for typical apps
- Kubernetes `RuntimeDefault` seccomp is GA since 1.19
- AppArmor as first-class API field is GA since Kubernetes 1.30
- `--seccomp-default` kubelet flag applies RuntimeDefault cluster-wide

**NEW DEFECT FOUND — NT-SHIELD-0733-E: No Seccomp Profile Generation for NT-ACT Tools**
NeoTrix tools (MCP, social media, code execution) run without application-specific seccomp profiles. The 2026 best practice is:
1. Start in `audit` mode → capture all syscalls
2. Generate profile from observed behavior
3. Switch to `enforce` mode
4. Missing syscall = crash, not bypass

**Implication**: NT-SHIELD needs a syscall profiling pipeline that:
- Instruments each NT-ACT tool to capture its syscall footprint
- Auto-generates minimal seccomp profiles
- Validates profiles in staging before production enforcement

### 2.2 Defense-in-Depth Stack: Falco + Seccomp + AppArmor
**Source**: [ShieldOps AI — Container Runtime Security Guide (June 2026)](https://shieldops-ai.dev/blog/container-runtime-security-falco-seccomp-apparmor)

**Three-layer runtime defense**:
- Falco: Behavioral monitoring (eBPF, <2% CPU overhead)
- Seccomp: System call filtering (kernel boundary firewall)
- AppArmor: Mandatory access control (resource access guard)

**NEW DEFECT FOUND — NT-SHIELD-0733-F: No Runtime Behavioral Monitoring**
NT-SHIELD has no equivalent to Falco — no eBPF-based syscall stream monitoring for anomaly detection. This means:
- Suspicious process spawns inside agent sandboxes go undetected
- File system tampering is not caught in real-time
- Privilege escalation attempts are invisible until post-mortem

### 2.3 Non-Root Is Not Sufficient
**Source**: [Wasil Zafar — Runtime Security & Hardening (May 2026)](https://www.wasilzafar.com/pages/series/containers-docker/containers-docker-part16-runtime-security.html)

**Critical insight**: "Running as non-root" is necessary but insufficient:
- Non-root processes can still call `setns()`, `perf_event_open()`, `ptrace()`
- `--privileged` is the nuclear option — grants ALL capabilities, disables seccomp, disables AppArmor
- `no-new-privileges` flag prevents privilege escalation via setuid/setgid
- Read-only root filesystem + noexec tmpfs = no post-exploitation persistence

**NEW DEFECT FOUND — NT-SHIELD-0733-G: NT-SHIELD Sandboxes Lack no-new-privileges + Read-Only FS Hardening**
NT-SHIELD sandbox containers do not enforce:
- `no_new_privs` bit (PR_SET_NO_NEW_PRIVS)
- Read-only root filesystem with tmpfs for writable paths
- `noexec` on all tmpfs mounts
- `cap_drop: ALL` with selective `cap_add`

---

## 3. CONTAINER ISOLATION — Findings

### 3.1 Confidential Containers: Hardware-Attested Isolation
**Source**: [Red Hat — Confidential Containers on OpenShift (June 2026)](https://developers.redhat.com/articles/2026/06/04/overview-confidential-containers-openshift-bare-metal)

**Architecture**:
- Each pod → dedicated Kata VM → CVM (Confidential VM) on TEE hardware
- Remote attestation via Trustee KBS (Key Broker Service)
- Three policy layers: Kata Agent Policy (inside TEE), KBS Resource Policy (at KBS), Attestation Service Policy
- TLS must terminate INSIDE the TEE
- Signed container images are non-negotiable

**NEW DEFECT FOUND — NT-SHIELD-0733-H: No TEE/Confidential Computing Integration Path**
NeoTrix has zero integration with confidential computing primitives:
- No support for AMD SEV-SNP or Intel TDX
- No remote attestation flow for model weights / credentials
- No encrypted container image support
- No TEE-bound secret release mechanism

**Implication**: For AI agent workloads carrying proprietary model weights or financial credentials, NT-SHIELD cannot guarantee confidentiality against a compromised host operator.

### 3.2 Fasco: Lightweight Confidential Containers (ARM CCA)
**Source**: [arXiv — Fasco: Lightweight Confidential Container Runtime (2026)](https://arxiv.org/html/2605.26018v1)

**Breakthrough**: Instead of container-in-VM (Kata/CoCo), Fasco proposes container-as-isolated-domain:
- Each container = independent ARM CCA Realm (hardware-enforced)
- System Realm provides shared services
- Eliminates full VM software stack overhead
- Dramatically reduces startup latency vs Kata (1-2s → sub-second)
- Lower resource consumption while maintaining hardware-level confidentiality

**NEW DEFECT FOUND — NT-SHIELD-0733-I: Container-in-VM Overhead Unsuitable for Agent Tool Sandboxing**
The Kata/CoCo pattern (container-in-VM) adds 10-30s startup latency + 5-15% CPU overhead. For NT-ACT tool sandboxes that need sub-second creation/destruction:
- 125ms Firecracker boot is borderline acceptable
- Kata's 1-2s is too slow for fine-grained tool isolation
- Fasco's Realm approach is ideal but ARM CCA hardware is emerging (2025-2026)

**Implication**: NT-SHIELD needs a hardware-aware isolation selector that chooses the lightest viable boundary based on tool trust requirements and available hardware.

### 3.3 Azure AKS Confidential Containers
**Source**: [Microsoft — Confidential Containers on AKS (May 2026)](https://learn.microsoft.com/en-us/azure/aks/confidential-containers-overview)

**Key production data**:
- Memory overhead: 2GB base for UVM + workload limits
- Memory overhead for standard Kata: 256MB base + workload limits
- Starting Nov 30, 2025: Azure Linux 2.0 no longer supported
- Conf Containers on AKS currently Azure Linux only

**NEW DEFECT FOUND — NT-SHIELD-0733-J: No Multi-Cloud Confidential Computing Abstraction**
NeoTrix's NT-SHIELD has no unified interface across:
- Azure Confidential Containers (AKS)
- GKE Confidential Nodes
- AWS Nitro Enclaves
- Self-hosted Kata + CoCo on bare metal

Each provider has different attestation flows, policy formats, and hardware requirements. NT-SHIELD needs a ConfidentialComputing trait that abstracts these differences.

### 3.4 Agent Sandbox Decision Framework
**Source**: [Pi Stack — gVisor vs Kata vs Firecracker Guide (April 2026)](https://www.pistack.xyz/posts/2026-04-20-gvisor-vs-kata-containers-vs-firecracker-container-sandboxing-guide-2026/)

**Comparison matrix**:
| Technology | Isolation | Boot Time | Memory | Best For |
|-----------|-----------|-----------|--------|----------|
| gVisor | Syscall-level (userspace kernel) | ~ms | ~100-200MB | Multi-tenant PaaS, shared hosting |
| Kata | Lightweight VM (KVM) | 1-2s | ~128-256MB | Enterprise, compliance |
| Firecracker | MicroVM (KVM VMM) | ~125ms | ~5-50MB | Serverless, FaaS, CI/CD |
| Standard container | Namespaces/cgroups | ~ms | ~0 | Trusted code only |

**Key insight**: GPU passthrough is only supported by Kata — not gVisor or Firecracker. For NT-ACT AI inference workloads requiring GPU, Kata is the only viable sandbox runtime.

**NEW DEFECT FOUND — NT-SHIELD-0733-K: No GPU-Aware Sandbox Routing**
NeoTrix does not consider GPU requirements when selecting sandbox runtime. For AI agent tool execution:
- CPU-only tools → gVisor or Firecracker
- GPU-required tools → Kata (only option with GPU passthrough)
- Mixed workloads → heterogeneous sandbox fleet

---

## 4. SYNTHESIS: Cross-Cutting Defects

### 4.1 NT-SHIELD Architecture Gaps (Priority Order)

| ID | Defect | Severity | Effort |
|----|--------|----------|--------|
| NT-SHIELD-0733-A | No tiered isolation policy engine | HIGH | L |
| NT-SHIELD-0733-B | No agent-internal sandbox decomposition | HIGH | XL |
| NT-SHIELD-0733-D | Missing tool scoping (Layer 4) + audit logging (Layer 5) | HIGH | L |
| NT-SHIELD-0733-E | No seccomp profile generation pipeline | MEDIUM | M |
| NT-SHIELD-0733-F | No runtime behavioral monitoring (Falco equiv) | MEDIUM | L |
| NT-SHIELD-0733-G | Missing no-new-privs + read-only FS hardening | LOW | S |
| NT-SHIELD-0733-H | No TEE/confidential computing integration | MEDIUM | XL |
| NT-SHIELD-0733-I | Container-in-VM overhead for tool sandboxes | MEDIUM | L |
| NT-SHIELD-0733-J | No multi-cloud confidential computing abstraction | LOW | L |
| NT-SHIELD-0733-K | No GPU-aware sandbox routing | MEDIUM | M |
| NT-ACT-0733-C | No distributed sandbox lifecycle for agent fleets | MEDIUM | L |

### 4.2 Pattern: The 5-Layer Agent Isolation Stack (New)

The 2026 consensus across all sources is a **5-layer isolation stack** for AI agents:

```
Layer 5: Audit Trail        — Every syscall, tool call, credential use logged with chain-of-events
Layer 4: Tool Access Scoping — Short-lived scoped tokens per tool invocation (OAuth2)
Layer 3: Execution Boundary  — Firecracker/gVisor/Kata (hardware or syscall-level)
Layer 2: Filesystem Boundary — Ephemeral volumes, quarantine writes, noexec tmpfs
Layer 1: Network Boundary    — Isolated namespace, allowlist-only egress
```

**NeoTrix covers**: Layers 1-3 (partial)
**NeoTrix missing**: Layer 4 (tool scoping), Layer 5 (audit), and proper Layer 3 implementation (no tiered selection, no GPU awareness)

### 4.3 Connection to Batch 732 Findings

| Batch 732 Finding | Connection to 733 |
|---|---|
| 82% orgs lack governance maturity | Sandbox isolation governance is part of this — no audit trail = no governance evidence |
| Agent-first data product registry | Registry must include sandbox requirements per tool/data product |
| Cross-protocol governance gap | Isolation tier selection is a governance decision — needs policy-as-code |
| Semantic layer = AI readiness prerequisite | Seccomp/AppArmor profiles are semantic constraints on syscall vocabulary |
| No open knowledge format | Sandbox policy definitions need a portable, vendor-neutral format |

---

## 5. SOURCES CITED

1. Safeguard.sh — "gVisor vs Firecracker in 2026" (April 2026) — https://safeguard.sh/resources/blog/gvisor-vs-firecracker-2026
2. Google gVisor Blog — "Multi-Agent gVisor Isolation (MAGI)" (April 2026) — https://gvisor.dev/blog/2026/04/15/magi-multi-agent-gvisor-isolation/
3. Google Cloud Blog — "gVisor Sandboxes for Ray Clusters on GKE" (August 2026) — https://cloud.google.com/blog/products/containers-kubernetes/gvisor-sandboxes-for-ray-clusters-on-gke
4. TURION.AI — "Agent Sandboxing: Firecracker, gVisor & Production Isolation" (May 2026) — https://turion.ai/blog/agent-sandboxing-firecracker-gvisor-microvm-architecture/
5. Aleksei Aleinikov — "Firecracker vs gVisor: Which Sandbox in 2026?" (June 2026) — https://www.alekseialeinikov.com/en/blog/topics/devops/microvms-firecracker-vs-gvisor-secure-workloads-2026
6. Northflank — "Firecracker vs gVisor" (January 2026) — https://northflank.com/blog/firecracker-vs-gvisor
7. Pi Stack — "gVisor vs Kata Containers vs Firecracker Guide 2026" (April 2026) — https://www.pistack.xyz/posts/2026-04-20-gvisor-vs-kata-containers-vs-firecracker-container-sandboxing-guide-2026/
8. ShieldOps AI — "Container Runtime Security: Falco, Seccomp, AppArmor" (June 2026) — https://shieldops-ai.dev/blog/container-runtime-security-falco-seccomp-apparmor
9. Anuragh KP — "Seccomp and AppArmor: Kernel-Level Bodyguards" (June 2026) — https://iamanuragh.in/blog/2026-06-29-seccomp-apparmor-container-runtime-security/
10. Md Sanwar Hossain — "Container Runtime Security Hardening 2026" (April 2026) — https://mdsanwarhossain.me/blog-container-runtime-security-hardening.html
11. Wasil Zafar — "Runtime Security & Hardening" (May 2026) — https://www.wasilzafar.com/pages/series/containers-docker/containers-docker-part16-runtime-security.html
12. Kubernetes Docs — "Restrict Syscalls with seccomp" (January 2026) — https://kubernetes.io/docs/tutorials/security/seccomp/
13. Coding Protocols — "Kubernetes Seccomp & AppArmor 2026" (April 2026) — https://codingprotocols.com/tutorials/kubernetes-seccomp-apparmor
14. Red Hat — "Confidential Containers on OpenShift Bare Metal" (June 2026) — https://developers.redhat.com/articles/2026/06/04/overview-confidential-containers-openshift-bare-metal
15. Confidential Containers Docs — "Securing Your Workload" (May 2026) — https://confidentialcontainers.org/docs/getting-started/securing-workloads/
16. Kata Containers — "Quick Start Guide" (2026) — https://kata-containers.github.io/kata-containers/quick-start-guide/
17. Microsoft Azure — "Confidential Containers on AKS" (May 2026) — https://learn.microsoft.com/en-us/azure/aks/confidential-containers-overview
18. Systems Hardening — "Confidential Containers on Kubernetes" (April 2026) — https://www.systemshardening.com/articles/kubernetes/confidential-containers/
19. Confidential Containers Docs — "Confidential Image Storage" (July 2026) — https://confidentialcontainers.org/docs/features/protected-storage/confidential-image-storage/
20. arXiv — "Fasco: Lightweight Confidential Container Runtime" (2026) — https://arxiv.org/html/2605.26018v1
