# Iteration Batch 327 — Research Loop

**Date**: 2026-09-06  
**Research Domains**: Quantum Computing, Neuromorphic Computing, Formal Methods  
**NeoTrix Focus**: Architecture gaps, defect identification, design optimization

---

## 1. Quantum Computing — 2026 Advances

### Sources

1. **RL Control of QEC** — Nature (2026-07-08). Google Willow processor: RL agent manages 1000+ control parameters, 3.5× logical stability improvement, record LER 7.72×10⁻⁴ for surface code. Scalable to 40K params / distance-15 code. ([nature.com/s41586-026-10759-2](https://www.nature.com/articles/s41586-026-10759-2))

2. **VGQEC — Variational Graphical QEC** — Nature Comms Physics (2026-05-15). Learning-based framework embedding tunable parameters into Quon graphs. Adapts to device-specific noise, interpolates between code families. Photonic proof-of-concept demonstrated. ([nature.com/s42005-026-02681-w](https://www.nature.com/articles/s42005-026-02681-w))

3. **Transformer-based QEC Decoder** — arXiv:2606.22194 (2026-06-20). Neural network approximating maximum likelihood decoding, estimating coherent information. Outperforms MWPM decoder. Novel soft post-selection scheme. Thresholds match theoretical limits. ([arxiv.org/abs/2606.22194v1](https://arxiv.org/abs/2606.22194v1))

4. **Scalable QML** — arXiv:2607.24014 (2026-07-27). Unitary brick-wall/butterfly architectures with multi-layer parallel parameter-shift rule. O(k·log n) gradient cost. Provable barren plateau avoidance via particle-number-preserving RBS gates. ([arxiv.org/abs/2607.24014v1](https://arxiv.org/abs/2607.24014v1))

5. **OmniQEC** — arXiv:2607.25865 (2026-07-28). LLM-orchestrated QEC code discovery. Self-evolving reasoning with slow–fast synergistic workflow. Outperforms BB codes at 98/240 qubit budgets. ([arxiv.org/abs/2607.25865](https://arxiv.org/abs/2607.25865))

6. **High-Rank Encoding for AQEC** — arXiv:2609.00778 (2026-09-01). Intrinsic encoding randomness improves entanglement fidelity. Higher-rank encoders needed for near-perfect recovery. ([arxiv.org/abs/2609.00778](https://arxiv.org/abs/2609.00778))

### Defects Found

| # | Defect | Evidence | Suggestion |
|---|--------|----------|------------|
| Q1 | **nt_core_quantum_fusion is metaphorical, not functional.** The module implements "quantum-inspired" signal fusion as deterministic float arithmetic (line 16: "无真量子随机性"). It borrows quantum terminology (superposition, entanglement, collapse) for a classical weighted-average. This is a naming/design mismatch — the 2026 QEC research shows real quantum error correction is now achieving sub-10⁻³ logical error rates with RL agents controlling 1000+ parameters. NeoTrix's "quantum fusion" is a trivial operations research problem dressed in quantum language. | `nt_core_quantum_fusion.rs:16` | **Either (a)** strip quantum naming to `multi_signal_fusion.rs` and embrace it as classical signal fusion, **or (b)** if quantum-inspired computation is desired, integrate actual QEC concepts: surface code syndrome decoding, RL-based parameter tuning, or variational code adaptation (VGQEC pattern). The current implementation adds cognitive overhead without computational value. |
| Q2 | **No quantum-aware error correction layer in SEAL pipeline.** The 2026 RL-QEC result shows that error-correction-as-learning (using error signals as RL reward) is the new paradigm. NeoTrix's SEAL pipeline has no mechanism for using its own self-test error signals as learning signals for continuous parameter adaptation. The pipeline runs discrete cycles (explore→distill→absorb) without continuous drift correction. | `seal_core/` pipeline stages | Add an RL-style **continuous drift correction** phase to SEAL. When SelfTest signals detect degradation (T3 production wiring), the error signals should feed back into parameter adaptation — mirroring how the Willow RL agent uses QEC syndrome events. This closes the gap between NeoTrix's discrete evolution cycles and the 2026 continuous-adaptation paradigm. |
| Q3 | **VSA HyperCube lacks noise-adaptive encoding.** VGQEC (2026) demonstrates that variational parameters embedded in code structure can adapt to device-specific noise. NeoTrix's FHRR VSA encoding (`fhrr_vsa`) uses fixed encoding functions (`encode_scalar`, `bind`, `bundle`) without any noise-adaptive mechanism. The code assumes ideal conditions. | `nt_core_hcube::fhrr_vsa` | Introduce a **VGQEC-inspired adaptive VSA layer**: parameterized encoding circuits that can be tuned per-environment (noisy channels, degraded memory). The binding/bundling operations should accept a learned noise model and adjust their superposition parameters accordingly. This is critical for edge deployment where NeoTrix runs on resource-constrained hardware. |
| Q4 | **Transformer-based decoding not leveraged.** The 2026 transformer-QEC result shows NN decoders outperform MWPM with soft post-selection. NeoTrix's perception layer (NT-WORLD) uses classical information retrieval (BM25, FTS5) but never applies transformer-based soft post-selection to its own classification decisions. | `nt_world_crawl/classifier.rs` | Add **soft post-selection** to NT-WORLD's classifier pipeline. When confidence is below threshold, instead of binary reject/accept, use a transformer decoder to estimate coherent information (signal quality metric) and make graded decisions. This mirrors the QEC soft post-selection approach. |

---

## 2. Neuromorphic Computing — 2026 Advances

### Sources

1. **Intel Loihi 3** — 8M neurons / 64B synapses per chip, 4nm process, 32-bit graded spikes, commercial availability Q4 2026. 1000× power reduction vs GPUs (1.2W peak). On-chip STDP learning. ([machinebrief.com](https://www.machinebrief.com/news/intel-neuromorphic-loihi-3-brain-computing-edge-ai-2026))

2. **Multi-core Neuromorphic for SNN Training** — Nature Comms (2026-03-25). First multi-core architecture enabling BP-based deep SNN training. 1.05 TFLOPS/W @ FP16 @ 28nm. 55-85% DRAM access reduction vs A100. 190-330% performance of Jetson Orin. ([nature.com/s41467-026-70586-x](https://www.nature.com/articles/s41467-026-70586-x))

3. **Neuromorphic Radar Processing** — IOP Neuromorphic Computing (2026-07-24). First real-time radar pipeline on Loihi 2. SNN-based FT, NCI, CFAR detection on real sensor data. Identifies routing/host-transfer bottlenecks. ([iopscience.iop.org](https://iopscience.iop.org/article/10.1088/2634-4386/ae8694))

4. **Hardware Landscape Comparison** — Wagenbach (2026-02-22). Comprehensive comparison of Loihi 2, Loihi 3, SpiNNaker 2, Tianjic, NorthPole, Akida 2.0. Key insight: software usability layer is the bottleneck, not hardware. ([joshwagenbach.com](https://www.joshwagenbach.com/blog/neuromorphic-hardware-landscape-2026))

### Defects Found

| # | Defect | Evidence | Suggestion |
|---|--------|----------|------------|
| N1 | **No event-driven processing model in NT-PHYSICAL.** Loihi 3's power advantage comes from event-driven computation — neurons activate only when inputs change. NeoTrix's `nt_physical` (sensors/motors/safety) uses traditional polling/frame-based processing. The `sensors` and `motors` abstractions have no concept of temporal sparsity. | `CONTEXT.md:30` (NT-PHYSICAL definition), `AGENTS.md:30` | Introduce an **event-driven sensor model** in NT-PHYSICAL. Sensors should emit spikes/events only when readings change beyond a threshold, not at fixed intervals. This aligns with Loihi 3's paradigm and would dramatically reduce power for edge deployment. Define `EventDrivenSensor` trait with `threshold` and `on_change()` callbacks. |
| N2 | **NT-FEEL emotion engine lacks temporal coding.** Loihi 3's graded spikes encode information in timing, not just amplitude. NeoTrix's EmotionLabel (11 variants) and EmotionEngine use static intensity values with no temporal dimension. Emotion transitions are instantaneous state changes, not spike-timing-dependent processes. | `CONTEXT.md:71` (EmotionLabel), `nt_feel/` | Add **temporal emotion coding** to EmotionEngine. Emotion transitions should use spike-timing-dependent patterns: the *rate* of emotion change, not just the current state, carries information. Implement STDP-like learning where repeated emotion sequences (e.g., fear→relief) strengthen associated neural pathways. This makes emotion more biologically realistic and enables predictive emotional modeling. |
| N3 | **No on-chip learning adaptation.** The 2026 multi-core neuromorphic paper demonstrates BP-based on-chip SNN training. NeoTrix's learning is entirely cloud-dependent — all model updates require external LLM calls. For edge/autonomous operation (NT-PHYSICAL domain), this creates a fatal latency and power bottleneck. | Architecture: all reasoning via `nt_io` (LLM providers) | Design an **on-device micro-learner** for NT-PHYSICAL. Implement lightweight STDP-based adaptation for sensor fusion and motor control that operates entirely on-device, without cloud round-trips. Reserve cloud/LLM calls for high-level reasoning only. This mirrors Loihi 3's separation of edge inference (on-chip) from cloud training. |
| N4 | **Hybrid SNN-ANN gap.** Tianjic (2019) proved hybrid SNN-ANN architectures are viable in silicon. NeoTrix has no bridge between its continuous-value computations (VSA embeddings, f64 signals) and discrete spike-based processing. The `nt_core_quantum_fusion` uses f64, but there's no interface to spike-domain processing. | `nt_core_quantum_fusion.rs` (f64 only), no spike types defined | Define a **spike-domain bridge** in NT-CORE: `SpikeEncoder` (continuous→spike train) and `SpikeDecoder` (spike train→continuous). This enables NeoTrix to interface with neuromorphic hardware and benefit from temporal sparsity. The bridge should convert VSA embedding vectors to/from graded spike representations. |

---

## 3. Formal Methods — 2026 Advances

### Sources

1. **LeetProof: Certified Program Synthesis** — arXiv:2604.16584 (2026-04). Multi-modal verifier (PBT + SMT + interactive) on Lean. Staged pipeline: spec validation → synthesis → proof. Defect detection in reference benchmarks. ([arxiv.org/abs/2604.16584](https://arxiv.org/abs/2604.16584))

2. **Schwarz: Solver-Aware Agentic Verification** — arXiv:2608.30803 (2026-08). Obligation-local repair, theory-aware solver policies. 95.2% on 475 benchmarks, 91.5% on SV-COMP 2026 (1427 LOC avg). For C and Rust/Verus. ([arxiv.org/abs/2608.30803](https://arxiv.org/abs/2608.30803))

3. **ProofEvolve: Neuro-Symbolic Evolution** — arXiv:2608.26334 (2026-08). Explicit symbolic proof structures evolved via neural operators. Kernel-verified schema extraction across problems. Highest solve rate on Lean competition benchmarks. ([arxiv.org/abs/2608.26334](https://arxiv.org/abs/2608.26334))

4. **NTP4VC: Neural Theorem Proving for Verification Conditions** — arXiv:2601.18944 (2026). First multi-language VC benchmark from Linux/Contiki-OS. Best model: 2.08% pass@1 — massive gap vs math benchmarks. ([arxiv.org/abs/2601.18944](https://arxiv.org/abs/2601.18944))

5. **Neuro-Formal Verification (NFV)** — arXiv:2608.21516 (2026-08). Language-agnostic formal reasoning via translation agent. 57% coverage at 92% precision on Python problems using Dafny proofs. ([arxiv.org/abs/2608.21516](https://arxiv.org/abs/2608.21516))

6. **OProver: Agentic Theorem Proving** — arXiv:2605.17283 (2026-05). Retrieval + compiler feedback + iterative repair. 93.3% Pass@32 on MiniF2F. 1.77M statements, 6.86M verified proofs in corpus. ([arxiv.org/abs/2605.17283](https://arxiv.org/abs/2605.17283))

7. **Neuro-Symbolic Proof for Systems Verification** — OSDI 2026. Proof step tree search on seL4. 77.6% proof success, 71% reduction in manual proof effort. ([cs.nju.edu.cn](https://cs.nju.edu.cn/yuanyao/static/osdi2026.pdf))

8. **P3: Joint Program-and-Proof Planning** — arXiv:2608.09277 (2026-08). Joint planning of program structure and proof before elaboration. 4.6-11.2pp improvement over sequential pipeline. Lean4Commit0 benchmark. ([arxiv.org/abs/2608.09277](https://arxiv.org/abs/2608.09277))

### Defects Found

| # | Defect | Evidence | Suggestion |
|---|--------|----------|------------|
| F1 | **kani_proofs.rs are tests, not verified proofs.** NeoTrix's "formal proofs" in `kani_proofs.rs` are standard `#[test]` functions that call `assert!`. They use no proof assistant (Lean, Isabelle, Rocq) and no SMT solver. The 2026 landscape shows that verified proofs require machine-checkable certificates (Lean kernel checks, Dafny verification conditions). NeoTrix's current approach provides zero formal guarantees — it's testing, not proving. | `kani_proofs.rs:1-18` (module docs claim "Formal proof harnesses" but implementation is assert-based tests) | **Rename and clarify**: either (a) rename to `exhaustive_tests.rs` to accurately describe what they are, or (b) if formal verification is desired, adopt a staged verification approach: **PBT for specification validation → SMT for automated VCs → interactive Lean proofs for hard obligations** (LeetProof pattern). For NeoTrix's E8 and VSA properties, this is feasible because the property space is well-defined and small. |
| F2 | **No agentic proof repair loop.** The 2026 OProver and Schwarz results show that agentic iterative repair (failed proof → feedback → revised proof) is essential. NeoTrix's SEAL pipeline has no proof repair mechanism — if a SelfTest fails, the response is binary pass/fail with no structured repair trajectory. | `nt_core_self_test_integration.rs`, SEAL pipeline stages | Implement an **agentic proof repair loop** for SelfTest failures. When a SelfTest fails: (1) extract the verification condition (what property was violated), (2) propose repair hypotheses (similar to OProver's retrieved compiler-verified proofs), (3) apply the fix, (4) re-verify. Store successful repair trajectories in KB for future retrieval (OProofs pattern). |
| F3 | **No multi-modal verification strategy.** NeoTrix uses only one verification mode: compile + test. The 2026 LeetProof and Schwarz results demonstrate that multi-modal verification (PBT + SMT + interactive) dramatically outperforms single-mode approaches. NeoTrix's R-P1 (`#![forbid(unsafe_code)]`) is the only static guarantee. | `AGENTS.md` (R-P1 rule), `dev-rules.md` | Layer a **multi-modal verification stack**: (1) PBT for property-based testing of module contracts, (2) Kani/SMT for bounded model checking of core invariants (E8, VSA, GWT), (3) Lean/Isabelle for critical-path proofs (GWT attention correctness, SEAL pipeline termination). Use LeetProof's staged approach: cheapest mode first, escalate only on failure. |
| F4 | **No verification conditions extracted from code.** NTP4VC (2026) extracts VCs from real-world projects (Linux, Contiki) using Why3/Frama-C pipelines. NeoTrix has no VC extraction — properties are hand-written in test functions. This means verification is ad-hoc and coverage is unpredictable. | `kani_proofs.rs` (hand-written properties only) | Build a **VC extraction pipeline** for NeoTrix Rust code. Use existing tools: Kani for bounded verification conditions, or Crux-LLVM for symbolic execution. For each public function, extract postconditions and loop invariants automatically. Feed these into the multi-modal verification stack (F3). |
| F5 | **No cross-problem schema reuse.** ProofEvolve (2026) extracts verified sub-proof schemas and reuses them across problems. NeoTrix's experience absorption (`experience-tree`) stores session summaries but not verified proof schemas. Each SelfTest rewrite starts from scratch. | `experience-tree` KB schema, `nt_core_self_test_integration.rs` | Extend the experience absorption protocol to capture **verified repair schemas** (not just summaries). When a SelfTest repair succeeds, extract the structural pattern (what property was violated, what fix was applied) as a reified schema in KB. Future SelfTest failures can retrieve matching schemas (OProver retrieval pattern). This turns NeoTrix's self-healing from reactive to pattern-guided. |

---

## Cross-Domain Synthesis: Meta-Defects

| # | Meta-Defect | Domains | Impact |
|---|-------------|---------|--------|
| M1 | **Metaphor leakage.** NeoTrix borrows quantum terminology (superposition, entanglement, collapse) for classical algorithms. This creates false analogies that obscure what the system actually does. The 2026 quantum research shows real quantum algorithms are now achieving practical results — the metaphor can no longer serve as a proxy for sophistication. | Q1, N4 | Naming confusion, cognitive overhead, missed integration opportunities |
| M2 | **No continuous adaptation.** SEAL pipeline runs discrete cycles. 2026 QEC shows continuous RL adaptation outperforms discrete recalibration. Neuromorphic shows event-driven continuous processing. Formal methods show iterative agentic repair. NeoTrix lacks a continuous adaptation mechanism across all three domains. | Q2, N3, F2, F5 | System degrades between SEAL cycles, no drift correction, no iterative proof improvement |
| M3 | **Formal verification is performative, not functional.** `kani_proofs.rs` claims formal proofs but implements assert-based tests. The 2026 formal methods landscape shows machine-checkable proofs are now achievable (Lean, Lean4). NeoTrix's current approach provides zero additional safety over testing. | F1, F3, F4 | No formal safety guarantees, misleading documentation |
| M4 | **No hardware-awareness.** NeoTrix is designed for software-only execution but targets edge deployment (NT-PHYSICAL). 2026 neuromorphic shows event-driven processing is essential for edge AI. NeoTrix has no interface to spike-based hardware, no event-driven sensor model, no on-chip learning. | N1, N2, N3, N4 | Cannot deploy to neuromorphic hardware, inefficient on edge devices |

---

## Prioritized Recommendations

| Priority | Action | Effort | Impact |
|----------|--------|--------|--------|
| **P0** | Rename/refactor `kani_proofs.rs` → honest naming OR adopt Lean/Kani for real proofs | Low/Med | Correctness of documentation |
| **P1** | Add event-driven sensor model to NT-PHYSICAL (EventDrivenSensor trait) | Medium | Edge deployment enablement |
| **P1** | Add continuous drift correction to SEAL pipeline (RL-style feedback) | High | Self-healing improvement |
| **P2** | Introduce multi-modal verification stack (PBT → SMT → interactive) | High | Safety guarantees |
| **P2** | Build VC extraction pipeline for NeoTrix Rust code | High | Verification automation |
| **P3** | Add spike-domain bridge (SpikeEncoder/SpikeDecoder) in NT-CORE | Medium | Neuromorphic interop |
| **P3** | Add soft post-selection to NT-WORLD classifier | Medium | Perception quality |
| **P3** | Extend experience absorption with verified repair schemas | Medium | Self-healing pattern library |
| **P4** | Add temporal emotion coding to EmotionEngine | Low | Biological realism |
| **P4** | Add adaptive VSA encoding (VGQEC-inspired noise adaptation) | Medium | Robustness |

---

**Files analyzed**: `CONTEXT.md`, `AGENTS.md`, `nt_core_quantum_fusion.rs`, `kani_proofs.rs`, `nt_core_self_test_integration.rs`, `fhrr_vsa`, `nt_world_crawl/classifier.rs`, SEAL pipeline, NT-PHYSICAL definition  
**External sources**: 14 papers/articles (2026)  
**Defects identified**: 13 specific + 4 meta-defects  
**Next iteration**: Batch 328 — focus on self-healing gap analysis and SEAL pipeline RL integration design
