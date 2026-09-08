# Iteration Batch 357 — Privacy-Preserving Computation Research

**Date**: 2026-09-06  
**Domain**: NT-SHIELD + NT-CORE (Privacy Architecture)  
**Research Depth**: Deep (24 sources, 3 domains)

---

## Sources Cited

### Differential Privacy (2026)

| # | Title | Venue/Date | Key Advance |
|---|-------|------------|-------------|
| S1 | Escaping Iterative Parameter-Space Noise: DP Learning with Hypernetwork | arXiv 2606.26772, Jun 2026 | DP-DeepSets: inject noise once into low-dimensional dataset embedding via hypernetwork, not per-step into high-dim gradients. LoRA fine-tuning of diffusion models: FID maintained down to ε=1 where DP-SGD collapses at ε≤16. |
| S2 | Spectral Gradient Orthogonalization Improves DP Training at Scale | ECCV 2026, arXiv 2608.17415 | Post-processing via polar decomposition recovers directional signal from noisy gradients at zero privacy cost. +20.9% over DP-SGD on WRN-28-10 (B=4096, ε=4). SNR phase transition: benefits only above spectral gap threshold. |
| S3 | DP-Muon: DP Optimization via Matrix-Orthogonalized Momentum | arXiv 2605.12994 | Muon-style optimizer with per-matrix clipping + Gaussian noise + Newton-Schulz. Bias correction (DP-MuonBC) removes output-level bias post-Newton-Schulz at zero privacy cost. |
| S4 | Revisiting the Provable-Auditable Privacy Gap of DP-SGD | arXiv 2608.28934, Aug 2026 | Privacy auditing framework: defense that improves empirical privacy lower bound at no theoretical privacy cost for DP-SGD. |
| S5 | DP-IVON-Gradsq: DP Squared-Gradient Variational Online Newton | arXiv 2607.23649, Jul 2026 | Bayesian DP learning: squared-gradient curvature estimation reduces interaction between posterior sampling noise and privacy noise. Competitive under weak-to-moderate privacy. |
| S6 | Efficient DP-SGD for LLMs with Randomized Clipping | TPDP 2026 | Hutchinson's estimator reduces per-sample gradient norm computation from O(B·min{T²,d²}) to O(B·k·T + k·p). 98% FLOPs reduction. |
| S7 | Nearly Linear-Time User-Level DP-SCO with Optimal Rates | COLT 2026 (PMLR v336) | Adaptive outlier removal integrates sparse vector technique into SGD loop; bounds sensitivity without privatizing all intermediate steps. |

### Federated Learning (2026)

| # | Title | Venue/Date | Key Advance |
|---|-------|------------|-------------|
| S8 | Deep Latent Variable Model Based VFL with Flexible Alignment | ICLR 2026 | Unified framework for arbitrary alignment/labeling scenarios in VFL. Outperforms baselines in 160/168 configs across 4 datasets. |
| S9 | Equilibrium-Driven VFL with Selective Privacy Protection | AAAI 2026 | NashCoder: joint accuracy-privacy optimization via Nash equilibrium. Adaptive Shapley-value decomposition for distributed optimization. |
| S10 | H-OutFed / V-OutFed: Combined H+V Partitioning | Applied Soft Computing, 2026 | First frameworks for combined horizontal+vertical FL partitioning. V-OutFed matches centralized accuracy. |
| S11 | FedRE: Representation Entanglement for Model-Heterogeneous FL | CVPR 2026 | Entangled representations: normalized random weights blend cross-category info. Mitigates representation inversion attacks. Communication-efficient. |
| S12 | DP-FedAdamW: DP Optimizer for Federated Large Models | CVPR 2026 | First AdamW-based DP-FL optimizer. Stabilizes second-moment variance, removes DP bias, aligns local-global updates. +5.83% over SOTA on Tiny-ImageNet (ε=1). |
| S13 | Split-MoPE: VFL with Mixture of Predefined Experts | arXiv 2602.12708, Feb 2026 | Alignment-agnostic VFL via predefined experts for each data alignment pattern. Single communication round. Robust against malicious parties. |
| S14 | Breaking Structural Identity: Personalized Federated LoRA under Rank Heterogeneity | EMNLP 2026 | FedRoRA: rank-wise personalized LoRA. Decouples shared global directions from personalized magnitudes via learnable diagonal scales. |
| S15 | GA-Based Group Client Selection for FL | Evolutionary Intelligence, Jul 2026 | Multi-criteria fitness (data size, feature/label coverage, class balance, divergence, importance, power, reputation). Works for both HFL and VFL. |

### Secure Computation (2026)

| # | Title | Venue/Date | Key Advance |
|---|-------|------------|-------------|
| S16 | Bifrost: Hybrid TEE-FHE Transformer/LLM Serving | arXiv 2606.17421, Jun 2026 | CPU TEE (root of trust) + FHE accelerator delegation. Linear layers on CKKS, non-linear operators in TEE. 9.25× latency reduction on GPT-2. |
| S17 | Google HEIR: Homomorphic Encryption Intermediate Representation | Google Blog, Aug 2026 | Open-source compiler converting pre-trained AI models to operate on encrypted inputs. Partners with Belfort, Niobium, Cornami, Optalysys for hardware acceleration. |
| S18 | SMASH: Scalable Maliciously Secure Hybrid MPC for LLMs | USENIX Security 2026 | DFT-based rotation + lightweight ZKPoK for nonlinear ops. O(n) communication (vs O(n²)). 18.9× runtime, 103× communication reduction over MD-ML. |
| S19 | Euston: Efficient Non-Interactive Secure Transformer Inference | ePrint 2026/046 | SVD-based mask transmission + batched HMM. 90× HMM, 165.7× HNE speedup over NEXUS. 3100× lower user preprocessing. |
| S20 | SENTRA: Privacy-Preserving Training in Outsourced Cloud | ePrint 2026/1443 | Hybrid TEE + secret sharing + MPC. Scalable collective attestation, versioned KVS, DPSS resharing. 8.89 samples/s, 1.29× faster than CrypTen. |
| S21 | PrivDNN: Partial DNN Encryption via MPC | arXiv 2607.21895, Jul 2026 | Encrypt only core neurons (subset essential to performance). 97% reduction in privacy-preserving inference time. |
| S22 | MOSAIC: Masked Outsourcing of Secure AI Computations | arXiv 2607.29221, Jul 2026 | Matrix-multiplication masking with noise relaxation. Random Hadamard rotations bound error growth. Scales to 70B transformer models. |
| S23 | Scalable Honest-majority MPC for ML from Mixed Secret Sharings | ePrint 2026/038 | Shamir + packed Shamir hybrid: PS for non-linear, SS for linear. 3.6-6.1× communication reduction, 1.5-4.3× WAN speedup. |
| S24 | Vertical FL: Structured Literature Review | IJCKG, Feb 2025 (comprehensive survey) | First systematic VFL review. Identifies gaps in privacy, communication, hybrid partitioning. |

---

## Defects Found in NeoTrix Architecture

### DEFECT-1: No Formal Differential Privacy Module (CRITICAL)
**Location**: Missing from NT-SHIELD and NT-CORE  
**Evidence**: The only privacy mechanism is `egress_privacy_guard` in `nt_core_llm.rs:584` which performs redaction/blocking based on trust tiers (Trusted/Contracted/Untrusted). No DP-SGD, no noise calibration, no (ε,δ)-DP accounting exists anywhere.  
**Impact**: NeoTrix cannot provide formal privacy guarantees for any model training or inference pipeline. When using external LLM providers (even Contracted tier), the egress guard redacts source code but has no mathematical bound on what a sophisticated adversary could infer from the model's behavior over multiple queries.  
**Research Gap**: S1 shows hypernetwork-based DP achieves ε=1 utility where DP-SGD fails at ε≤16. S6 shows randomized clipping reduces DP-SGD LLM memory from O(B·min{T²,d²}) to O(B·k·T+k·p) with 98% FLOPs reduction. These make DP practical for NeoTrix's LLM pipelines.  
**Suggestion**: Implement `nt_shield::dp_engine` module with:
- Per-example gradient clipping + Gaussian noise injection (DP-SGD base)
- Hypernetwork-based DP for LoRA fine-tuning (S1 approach)
- Hutchinson's estimator for gradient norm computation (S6)
- RDP accounting for privacy budget tracking

### DEFECT-2: No Privacy Budget Accounting (HIGH)
**Location**: Missing from entire codebase  
**Evidence**: No Rényi DP accountant, no numerical privacy accountant, no (ε,δ)-budget tracking. The `DataTrust` enum in `privacy_guard.rs` has Trusted/Contracted/Untrusted tiers but no quantified privacy loss metric.  
**Impact**: Without accounting, NeoTrix cannot reason about cumulative privacy loss across multiple LLM calls, fine-tuning runs, or federated rounds. The system has no mechanism to say "we've used ε=2.3 of our budget" or to stop when budget is exhausted.  
**Research Gap**: S7 shows adaptive outlier removal bounds sensitivity without privatizing all steps. S4 shows privacy auditing can tighten the gap between theoretical and empirical privacy.  
**Suggestion**: Implement `nt_core::privacy_accountant` module:
- RDP accountant with composition theorems (Abadi et al. 2016 + Balle et al. 2018)
- Numerical f-DP accountant (Gopi et al. 2021) for tight tracking
- Per-session and cumulative budget enforcement
- Integration with `egress_privacy_guard` to quantify external call privacy cost

### DEFECT-3: No Federated Learning Infrastructure (CRITICAL)
**Location**: Missing from NT-ACT and NT-CORE  
**Evidence**: Only reference to "federated" is a comment about Mastodon API in `nt_world_osint/social.rs:53`. No FL server, no client aggregation, no VFL/HFL support.  
**Impact**: NeoTrix cannot participate in or orchestrate collaborative model training across distributed nodes. For multi-enterprise deployments (e.g., multiple NeoTrix instances sharing learned capabilities without sharing data), this is a fundamental gap.  
**Research Gap**: S8 (ICLR 2026) solves arbitrary alignment in VFL. S10 introduces combined H+V partitioning. S14 (EMNLP 2026) solves rank-heterogeneous FL for LoRA fine-tuning — directly applicable to NeoTrix's LoRA-based fine-tuning paths.  
**Suggestion**: Implement `nt_act::federated` module with:
- Horizontal FL with FedAvg/FedAdamW (S12) aggregation
- Vertical FL with Split-MoPE (S13) for feature-partitioned scenarios
- Combined H+V partitioning (S10)
- Rank-heterogeneous FedRoRA (S14) for LoRA fine-tuning across nodes
- Client selection with multi-criteria GA (S15)

### DEFECT-4: No Homomorphic Encryption for Inference (HIGH)
**Location**: Missing from NT-SHIELD and NT-IO  
**Evidence**: `key_encryption.rs` uses AES-256-GCM for credential vault encryption. No FHE for computation. All LLM inference sends plaintext to providers (with redaction via egress guard).  
**Impact**: When NeoTrix sends prompts to external LLM providers (OpenAI, Anthropic, etc.), the provider sees the full prompt content. The egress guard redacts NeoTrix's own source code but cannot protect user-provided data in prompts.  
**Research Gap**: S16 (Bifrost) achieves hybrid TEE-FHE serving with 9.25× latency reduction. S17 (Google HEIR) provides a compiler for converting pre-trained models to FHE execution. S19 (Euston) achieves 90× HMM speedup over prior art.  
**Suggestion**: Implement `nt_shield::fhe_engine` module:
- CKKS-based encrypted inference for linear layers
- TEE-offloaded non-linear operators (Softmax, LayerNorm, GELU)
- HEIR compiler integration for model conversion
- Encrypted prompt submission to external providers

### DEFECT-5: No Secure Multi-Party Computation Layer (MEDIUM)
**Location**: Missing from NT-SHIELD  
**Evidence**: No MPC framework, no secret sharing, no garbled circuits. The vault uses symmetric encryption (AES-256-GCM) which requires a single key holder.  
**Impact**: NeoTrix cannot perform privacy-preserving computation across multiple untrusted parties. Threshold cryptography (requiring t-of-n key holders) is impossible.  
**Research Gap**: S18 (SMASH) achieves O(n) communication for nonlinear ops in MPC with malicious security. S20 (SENTRA) combines TEE + secret sharing for resilient training. S23 provides hybrid Shamir/packed-Shamir for scalable honest-majority MPC.  
**Suggestion**: Implement `nt_shield::mpc_engine` module:
- 2-party and n-party computation for sensitive aggregations
- Secret sharing for threshold key management
- SM-LUT based nonlinear operations (S18 approach)
- Integration with NT-MEMORY for private knowledge aggregation

### DEFECT-6: No TEE Integration (MEDIUM)
**Location**: Missing from NT-SHIELD and NT-PHYSICAL  
**Evidence**: No Intel TDX, AMD SEV, or Apple Secure Enclave integration. The `apple_silicon.rs` in NT-SHIELD exists but focuses on Metal GPU compute, not secure enclave.  
**Impact**: NeoTrix has no hardware-rooted trust boundary. All computation runs in untrusted userspace. For sensitive workloads (medical, financial data processing), this is a compliance blocker.  
**Research Gap**: S16 (Bifrost) uses TEE as root of trust with FHE delegation. S20 (SENTRA) provides collective attestation for TEE enclaves with 8.3% overhead.  
**Suggestion**: Implement `nt_shield::tee_layer` module:
- Apple Secure Enclave attestation (leveraging existing `apple_silicon.rs`)
- Intel TDX/AMD SEV support for cloud deployments
- Collective attestation protocol (SENTRA approach)
- TEE-sealed key management for vault operations

### DEFECT-7: Egress Guard Has No Formal DP Guarantee (HIGH)
**Location**: `nt_core_llm.rs:584` — `egress_privacy_guard()`  
**Evidence**: The guard performs pattern-matching redaction (regex-based) and trust-tier blocking. It has no formal (ε,δ)-DP guarantee. An adversary with access to multiple query-response pairs could infer redacted content via correlation attacks.  
**Impact**: The "Contracted" tier (paid cloud providers) allows data through with redaction, but redaction is not formally private. The system claims privacy protection that cannot be mathematically verified.  
**Research Gap**: S4 shows privacy auditing can tighten the gap between theoretical and empirical privacy. S7 shows adaptive outlier removal bounds sensitivity without full privatization.  
**Suggestion**: Upgrade `egress_privacy_guard` to:
- Add calibrated noise to all outbound requests (not just redaction)
- Implement per-provider privacy budget tracking via RDP accountant
- Apply S7's adaptive outlier removal to detect correlated query patterns
- Provide formal (ε,δ) guarantee statement per trust tier

### DEFECT-8: No Encrypted KV-Cache for LLM Inference (MEDIUM)
**Location**: Missing from NT-IO  
**Evidence**: No encrypted KV-cache management. LLM inference caches (which contain conversation context) are stored in plaintext memory.  
**Impact**: KV-caches can be extracted from memory dumps, leaking conversation history. For multi-tenant deployments, cross-user KV-cache leakage is possible.  
**Research Gap**: S16 (Bifrost) handles KV-cache transitions inside CPU TEE with ciphertext refresh. S19 (Euston) manages encrypted state across autoregressive steps.  
**Suggestion**: Implement encrypted KV-cache:
- TEE-sealed KV-cache for in-memory state
- Encrypted KV-cache persistence for long-context sessions
- Ciphertext refresh between users (Bifrost DtE approach)

---

## Suggestions Summary

| Priority | Module | Action | Effort | Research Ref |
|----------|--------|--------|--------|-------------|
| CRITICAL | `nt_shield::dp_engine` | Implement DP-SGD + hypernetwork DP for training/fine-tuning | Large | S1, S2, S3, S6 |
| CRITICAL | `nt_act::federated` | FL server/client with HFL+VFL+hybrid support | Large | S8, S10, S12, S13, S14 |
| HIGH | `nt_core::privacy_accountant` | RDP/f-DP budget tracking and enforcement | Medium | S4, S7, S6 |
| HIGH | `nt_shield::fhe_engine` | CKKS-based encrypted inference for prompts | Large | S16, S17, S19 |
| HIGH | `egress_privacy_guard` upgrade | Add formal DP guarantee, not just redaction | Medium | S4, S7 |
| MEDIUM | `nt_shield::mpc_engine` | Secret sharing + SM-LUT nonlinear computation | Large | S18, S20, S23 |
| MEDIUM | `nt_shield::tee_layer` | TEE attestation and sealed computation | Large | S16, S20 |
| MEDIUM | NT-IO encrypted KV-cache | TEE-sealed encrypted cache for LLM state | Medium | S16, S19 |

---

## Cross-Cutting Analysis

### Alignment with NeoTrix Architecture

| NeoTrix Layer | Current Privacy | Gap Severity | Recommended Layer |
|---------------|-----------------|--------------|-------------------|
| L1 Action | Egress redaction only | CRITICAL | Add DP to `nt_act` outbound |
| L2 Perception | No privacy on crawled data | HIGH | Add DP to crawler pipelines |
| L3 Embodiment (SHIELD) | AES vault, no MPC/TEE/FHE | CRITICAL | Add `dp_engine`, `fhe_engine`, `mpc_engine`, `tee_layer` |
| L4 Emotion | No privacy concern | — | — |
| L5 Cognition | No DP on model training | CRITICAL | Add DP-SGD to `nt_core` training |
| L6 Meta | No privacy accounting | HIGH | Add `privacy_accountant` |

### 2026 State-of-the-Art vs NeoTrix

| Capability | SOTA 2026 | NeoTrix | Gap |
|------------|-----------|---------|-----|
| DP for LLM fine-tuning | Hypernetwork DP (ε=1 utility) | Redaction only | **18 months behind** |
| DP accounting | Numerical f-DP + RDP | None | **No formal guarantee** |
| Federated learning | Combined H+V + rank-hetero FL | None | **No infrastructure** |
| FHE for inference | TEE-FHE hybrid, 9× speedup | None | **No encrypted inference** |
| MPC for nonlinear | O(n) malicious-secure MPC | None | **No MPC layer** |
| TEE integration | Collective attestation, 8.3% overhead | None | **No hardware trust** |

### Immediate Quick Wins (1-2 weeks)

1. **Add RDP accountant** to `egress_privacy_guard` — track ε consumed per external LLM call
2. **Implement DP-SGD** for LoRA fine-tuning (S6 randomized clipping — 98% FLOPs reduction makes it practical)
3. **Add encrypted KV-cache** using Apple Secure Enclave (leverage existing `apple_silicon.rs`)

### Strategic Investments (1-3 months)

1. **FHE inference pipeline** using Google HEIR compiler for encrypted prompt submission
2. **Federated learning module** starting with HFL (FedAvg) then extending to VFL
3. **TEE layer** with Apple Secure Enclave (local) + Intel TDX (cloud)

---

*Iteration 357 complete. 8 defects identified across 3 research domains. 24 sources cited. Recommendations span quick wins to strategic 3-month investments.*
