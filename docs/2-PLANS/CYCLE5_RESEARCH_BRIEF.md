# Cycle 5 — Research Brief: NeoTrix Architecture Evolution

> Date: 2026-07-01
> Status: Research complete — ready for implementation planning

---

## 1. A2A Protocol — Agent-to-Agent Communication Standard

### Status (2026)
Google's **Agent-to-Agent (A2A) Protocol** reached **v1.0 stable on April 9, 2026**, one year after initial launch. Now backed by **150+ organizations** (AWS, Cisco, Google, IBM, Microsoft, Salesforce, SAP, ServiceNow). Production SDKs in Python, JS, Java, Go, .NET.

### Key Technical Details
- **Base protocol**: JSON-RPC 2.0 over HTTP/WebSocket/gRPC
- **Agent Card**: `/.well-known/agent.json` — capability discovery (skills, input/output modes, streaming support)
- **Core objects**: Task (stateful unit of work), Message (user/agent role), Part (Text/File/Data), Artifact (output result)
- **Interaction modes**: Request/Response, Streaming (SSE), Push Notification (webhook)
- **Security**: OAuth2 + mTLS standardized
- **Task states**: submitted → working → input-required → completed → failed

### Relevance to NeoTrix
| Aspect | Current | Target |
|--------|---------|--------|
| Multi-agent comm | ad-hoc `AgentBus` | A2A v1.0 `Task`/`Message` protocol |
| Agent discovery | hardcoded registry | `/.well-known/agent.json` Agent Cards |
| Transport | in-process | HTTP/WS/gRPC multi-transport |
| Security | none | OAuth2 + mTLS |

### Integration Points
- `nt_act_autonomy` — PER agents need external inter-agent protocol
- `nt_act_social` — social agents already cross-boundary
- `nt_agent_mcp` — MCP bridges tools, A2A bridges agents (complementary)
- `nt_core_gwt` — GWT specialist agents could become A2A endpoints

### Key References
- [A2A Protocol v1.0 Specification](https://google.github.io/A2A/)
- [A2A Inspector](https://github.com/google-a2a/a2a-inspector) (debug tool)
- GitHub: `google-a2a/a2a-sdk-*`

---

## 2. Neuromorphic Computing — Brain-Inspired Hardware

### Hardware Landscape (2026)

| Chip | Neurons | Key Feature | Power |
|------|---------|-------------|-------|
| Intel Loihi 2 | 1M/chip | 10x faster, SDNN support, Lava framework | ~1W |
| Intel Hala Point | 1.15B | Largest neuromorphic system, 10x neuron capacity | ~10W |
| IBM TrueNorth | 1M | 256M synapses, 28nm CMOS, 70mW | ~70mW |
| NeuroMem NM500 | parallel chain | Zero-code learning, bidirectional neuron bus | low |
| Neuronspike Moore | LLM chip | 650 tok/s, 3x energy efficiency | low |

### Key Technical Developments
- **SDNN (Sigma-Delta Neural Networks)** on Loihi 2: graded activations, event-driven sparse communication, 10x efficiency over rate-coded SNNs
- **Hybrid SNN-ANN systems**: SNN for temporal feature extraction (Loihi) + ANN for classification (Jetson), accumulator bridge
- **NeuEdge framework**: adaptive threshold mechanism, 847 GOp/s/W, 2.3ms inference latency, 67% energy reduction with 96.2% accuracy
- **Automatic SNN generation**: random forest predictor for hardware-specific model generation (FPGA/Loihi)
- **Hala Point**: 1.15B neurons, 12x faster than first-gen, sustainable AI path

### Relevance to NeoTrix
| NeoTrix Module | Neuromorphic Opportunity |
|----------------|------------------------|
| `nt_core_deploy` | Edge deployment target (ANE/NPU/Loihi) |
| `nt_core_e8` | E8 state space as SNN encoding |
| `nt_core_hcube` | FHRR VSA on event-driven hardware |
| `nt_core_reson` | Kuramoto oscillator → spiking resonator |
| `nt_core_ssm` | Mamba-2 SSD on neuromorphic architecture |

### Key References
- Intel: [Loihi 2 brief](https://www.intel.la/content/dam/www/central-libraries/us/en/documents/neuromorphic-computing-loihi-2-brief.pdf)
- [NeuEdge: Comprehensive Framework](https://arxiv.org/html/2602.02439v1)
- [Hybrid SNNs on Loihi + Jetson](https://arxiv.org/abs/2407.08704)

---

## 3. Formal Verification — Proving Rust Code Correctness

### Tool Landscape (2026)

| Tool | Type | Approach | Maturity |
|------|------|----------|----------|
| **Kani** | Model checker | Bit-precise CBMC, `#[kani::proof]` harness | Production (AWS) |
| **Creusot** | Deductive verifier | Rust → Why3 → SMT (Z3), contracts | Active dev (INRIA) |
| **Prusti** | Deductive verifier | Annotation-based, Viper backend | Research |
| **Verus** | SMT-based | Linear ghost resources, `requires`/`ensures` | Active dev (MSR) |

### Kani (Recommended for NeoTrix)
- `cargo install kani-verifier` + `cargo kani setup`
- Checks: memory safety, panics, overflows, user assertions, unsafe blocks
- Nondeterministic inputs via `kani::any()` with `kani::assume()`
- **Function contracts** (experimental): `#[kani::requires]` / `#[kani::ensures]`
- GitHub Action: `model-checking/kani-github-action`
- **Best for**: `unsafe` code blocks, `#![forbid(unsafe_code)]` verification, critical path proofs

### Creusot (For Deep Correctness)
- Translates Rust to WhyML, uses Z3/Alt-Ergo solvers
- Handles `Vec`, `Box`, `Option` natively
- Linear ghost resources for pointer-manipulating `unsafe` code
- **CreuSAT**: verified SAT solver built with Creusot
- Requires: contract annotations longer than code for complex functions
- **Best for**: E8 engine step correctness, HyperCube VSA operations

### Relevance to NeoTrix

| Priority | Target | Tool | Why |
|----------|--------|------|-----|
| P0 | `nt_core_e8` (E8 engine) | Kani | State transitions, 64-state determinism proof |
| P0 | `nt_core_hcube` FHRR ops | Kani/Creusot | VSA bind/bundle algebraic properties |
| P1 | `nt_core_gwt` competition gate | Kani | WTA selection correctness |
| P1 | `nt_shield` sandbox | Kani | Unsafe boundary enforcement |
| P2 | `nt_memory_store` CRUD | Kani | KB operations, invariant enforcement |

### Key References
- [Kani Rust Verifier](https://github.com/model-checking/kani)
- [Creusot](https://github.com/creusot-rs/creusot) — [Guide](https://guide.creusot.rs)
- [Rust Formal Methods Group](https://rust-formal-methods.github.io/) (monthly talks)
- POPL 2026: [Creusot Tutorial](https://popl26.sigplan.org/details/POPL-2026-tutorials/6/Creusot-Formal-verification-of-Rust-programs)

---

## 4. Differential Privacy — Privacy-Preserving Learning

### Core Mechanisms (2026 state of the art)

| Mechanism | Description | Key Property |
|-----------|-------------|-------------|
| **DP-SGD** | Per-sample gradient clip + Gaussian noise | Standard (ε,δ)-DP, RDP accounting |
| **PATE** | Teacher ensemble → student via noisy aggregation | Tight ε for small datasets |
| **Rényi DP** | Moments accountant, tighter composition | Best for deep learning |
| **GG Mechanism** | Generalized Gaussian (β=1→2) | Gauss (β=2) optimal for DP-SGD |
| **Subsampling** | Poisson vs fixed-size without replacement | FSwoR: constant memory, lower variance |
| **PLD/FFT** | Privacy Loss Distribution accountant | Most precise ε tracking |

### DP-SGD Algorithm
1. Compute per-example gradients for minibatch
2. Clip each to ℓ₂-norm ≤ C
3. Aggregate + add Gaussian noise N(0, σ²C²I)
4. Update parameters with noisy gradient
5. Track cumulative ε via RDP accountant (σ, q, T)

### PATE Algorithm
1. Partition data into K disjoint sets
2. Train K teacher models independently
3. Aggregate teacher votes with Gaussian/Laplace noise
4. Train student model on noisy-aggregated labels
5. Tight ε: each teacher sees 1/K of data

### Relevance to NeoTrix

| NeoTrix Module | DP Application |
|----------------|----------------|
| `nt_mind_seal` (SEAL pipeline) | DP-SGD for reward model training, privacy-preserving evolution |
| `nt_memory_kb` / privacy.rs | PATE for query anonymization, DP guarantees on KB access |
| `nt_shield` | DP budget tracking, ε composition auditing |
| `nt_act_autonomy` | Local DP for agent learning across organizations |
| `nt_core_policy` | DP-SGD policy updates, gradient sanitization |

### Rust DP Libraries
- **No mature Rust DP library exists** — would need to implement from spec
- Existing references: `opacus` (PyTorch), `tensorflow-privacy` (TF)
- Implementation scope: per-example gradient, RDP accountant, noise mechanism

### Key References
- [Comprehensive DP Guide (2025)](https://arxiv.org/abs/2509.03294)
- [DP-SGD with Fixed-Size Minibatches](https://dl.acm.org/doi/10.5555/3737916.3738270) (NeurIPS 2024)
- [GG Mechanism Beyond Laplace/Gaussian](https://arxiv.org/pdf/2506.12553)
- [Sequential DP Auditing](https://arxiv.org/html/2509.07055v1)
- [PATE-GAN](https://openreview.net/forum?id=S1zk9iRqF7)

---

## 5. Integration Roadmap — Cycle 5 Implementation Priorities

| Priority | Area | Module | Effort | Dependencies |
|----------|------|--------|--------|-------------|
| **P0** | Formal Verification | `nt_core_e8` — Kani proof harness | 2d | `cargo kani` setup |
| **P0** | Formal Verification | `nt_core_hcube` — FHRR identity proof | 3d | Kani contracts |
| **P1** | A2A Protocol | `nt_act_autonomy` — A2A agent card + task protocol | 5d | A2A SDK Rust port |
| **P1** | Differential Privacy | `nt_mind_seal` — DP-SGD reward model | 5d | Rust DP primitives |
| **P1** | Neuromorphic | `nt_core_deploy` — Loihi/ANE target config | 3d | SNN conversion layer |
| **P2** | A2A Protocol | `nt_agent_mcp` — MCP↔A2A bridge | 4d | A2A task delegation |
| **P2** | Differential Privacy | `nt_memory_kb` — PATE query anonymization | 3d | PATE implementation |
| **P2** | Formal Verification | `nt_core_gwt` — WTA gate Kani proof | 2d | Competition gate spec |
| **P3** | Neuromorphic | `nt_core_e8` → SNN encoding | 5d | SDNN / Lava integration |

### Estimated Total: 32d (6.4 weeks) for full Cycle 5

---

## Next Step

Review this brief and select implementation priority. Recommend starting with **P0 formal verification (E8 + HyperCube)** — these are foundational proofs that protect NeoTrix core correctness with minimal external dependencies, and `cargo kani setup` is installable today.
