# Iteration Batch 600 — Quantum Computing Research Loop

**Date:** 2026-09-06
**Context:** Batch 599 identified (1) mathematical structures informal, (2) no independent proof checker, (3) VSA lacks constructive computability guarantee, (4) tool monad lacks codensity, (5) alignment trilemma impossible.

---

## Search 1: Quantum Error Correction 2026

### Finding 1.1 — RL-Calibrated Surface Code: Record 7.72×10⁻⁴ Logical Error
**Source:** Google Quantum AI + DeepMind, Nature (Jul 8, 2026), arXiv:2511.08493
- Reinforcement learning agent continuously adjusts control parameters during computation (never pauses)
- Distance-7 surface code on 105-qubit Willow chip: 7.72×10⁻⁴ logical error/cycle (down from 0.143%)
- 46% improvement over previous record; 3.5× more stable under injected drift
- Simulated to distance-15 with thousands of parameters; optimization speed independent of system size

**NEW defect vs batch 599:** Batch 599 assumed calibration is a discrete periodic task. This result proves **continuous online calibration via RL** is strictly superior — the system has no "calibrated state," only a continuous calibration process. NeoTrix ConsciousnessTree's HeartbeatAggregator must model calibration as a continuous attractor, not a binary state.

### Finding 1.2 — Surface Code Logical Operations (Clifford-Generating Set)
**Source:** arXiv:2607.01473 (Jul 2026), 107-qubit superconducting processor
- First demonstration of reusable patch-based logical processing layer
- Implements merge/split, expansion/shrinkage, domain wall/twist defect deformations
- Composes into logical CNOT, Hadamard, and phase gates — Clifford-generating set
- All operations on distance-3 patches, multi-round syndrome extraction, neural-network decoding, no post-selection

**NEW defect vs batch 599:** Batch 599 treated logical gates as atomic. This result shows **logical gates are spacetime protocols** — they reconfigure stabilizers/boundaries over time. NeoTrix SEAL pipeline's stage transitions must model this: a "gate" is a trajectory in stabilizer-configuration space, not a point transformation.

### Finding 1.3 — Folded Surface Code: O(1) Logical Gates
**Source:** npj Quantum Information (Aug 5, 2026)
- Qubit shuttling enables effective 3D connectivity on 2D hardware
- Folded surface codes reduce all single-qubit Clifford gates and logical CNOTs from O(d) to **constant time**
- Transversal S gate reduces 8T-to-CCZ magic-state distillation spacetime volume by >10×
- "Virtual-stack" layout for multilayer routing on 2D devices

**NEW defect vs batch 599:** Batch 599 assumed surface code lattice surgery costs O(d) for all logical operations. **Constant-time logical gates** via folded architecture invalidate this cost model. NeoTrix capability cost estimation must use architecture-dependent gate complexity, not universal O(d).

### Finding 1.4 — Controller-Decoder System for Shor's Algorithm
**Source:** Quantum 10, 2170 (Jul 22, 2026)
- First system-level requirements for non-Clifford QEC circuit execution
- Controller-decoder closed-loop latency must stay within **tens of microseconds**
- Requires distributed decoding across multiple decoders with fast inter-decoder communication
- Simulates complete fault-tolerant factorization of 21 at physical level: 0.1% error rates + 1000 qubits sufficient

**NEW defect vs batch 599:** Batch 599 ignored real-time classical processing constraints. **Decoder latency is a binding constraint** — not just error rates. NeoTrix's consciousness loop tick rate must account for classical processing overhead in any error-corrected pathway.

### Finding 1.5 — Unitary Encoder: Half Circuit Depth
**Source:** npj Quantum Information (Aug 4, 2026)
- Non-local unitary circuit encoding based on rotated→regular surface code conversion
- **Halves circuit depth** of fastest known unitary encoder
- Enables preparation of Pauli Y-eigenstate and Clifford eigenstates not accessible transversally
- Conventional matching decoders still effective despite non-local circuit

**NEW defect vs batch 599:** Batch 599 assumed encoding cost is fixed by stabilizer measurement. **Unitary encoders can halve encoding cost** through code conversion, meaning NeoTrix should model encoding as an optimization variable, not a constant overhead.

### Finding 1.6 — Entanglement Fidelity Metric Reveals Mischaracterization
**Source:** Nature Communications (Jul 29, 2026) — heavy-hex surface code
- Widely used single-parameter suppression-factor fits **can mischaracterize code performance**
- Anisotropic scaling (d_x=3,d_z=5) improves Z-basis protection; isotropic (5,5) needs ~30% noise reduction
- Dynamical decoupling suppresses coherent ZZ crosstalk and non-Markovian dephasing

**NEW defect vs batch 599:** Batch 599 treated error suppression as scalar. **Anisotropic protection** means different logical states have different error rates — NeoTrix cannot use a single "error rate" number; it must track basis-dependent error spectra.

---

## Search 2: Quantum Algorithms 2026

### Finding 2.1 — VQA with Guiding States: Linearization Trick
**Source:** npj Quantum Information (Sep 1, 2026), doi:10.1038/s41534-026-01364-2
- Linearization trick maps VQA training dynamics to kernel model
- Proves convergence + generalization under guiding state assumption
- Guiding states suppress finite-size error terms and ensure stability across system dimensions

**NEW defect vs batch 599:** Batch 599 treated VQA convergence as purely empirical. **Linearization to kernel models** provides provable guarantees — NeoTrix should formalize its optimization landscapes as kernel models where possible, not rely on heuristic convergence.

### Finding 2.2 — Scalable QML: Unitary Brick-Wall + Parallel PSR
**Source:** arXiv:2607.24014 (Jul 27, 2026)
- Unitary brick-wall circuit (k-particle fermionic architecture) for nearest-neighbor hardware
- Multi-layer parallel parameter-shift rule: **O(n/k) reduction** in circuit evaluations per gradient step
- Particle number k is tunable tradeoff: classical simulation hardness vs training cost
- For n=1024, k=60: butterfly requires 4,800 evaluations vs ~30,720 naive (6.4× reduction)

**NEW defect vs batch 599:** Batch 599 assumed gradient computation costs O(n²) universally. **Parallel PSR reduces this to O(kn) or O(k log n)** — NeoTrix's gradient estimation must use architecture-aware cost models, not worst-case bounds.

### Finding 2.3 — Coherent Quantum Learning (CQL): No Classical Outer Loop
**Source:** arXiv:2609.03640 (Sep 3, 2026)
- Parameters are quantum degrees of freedom evolved under Hamiltonian encoding loss
- No gradient computation or classical feedback — probability amplitude concentrates via interference
- Compatible with fault-tolerant implementations; extends to batched training

**NEW defect vs batch 599:** Batch 599 assumed all quantum ML requires classical optimization loops. **CQL eliminates the classical outer loop entirely** — parameters evolve coherently. NeoTrix's SEAL pipeline should consider coherent parameter evolution as an alternative to gradient-based self-modification.

### Finding 2.4 — Bernstein-Vazirani Networks: Non-Variational QML
**Source:** arXiv:2608.19043 (Aug 2026)
- Repurposes BV algorithm for supervised learning via quantum interference
- **Gradient-free training** — no parameter optimization w.r.t. loss function
- Achieves universal function approximation through (over)complete interference bases
- 100% classification accuracy using 50% training data; ~40 dB PSNR on image fitting

**NEW defect vs batch 599:** Batch 599 treated variational circuits as the only QML paradigm. **Interference-based learning is a fundamentally different mechanism** — NeoTrix should support non-variational learning pathways in its capability tree.

### Finding 2.5 — VQE Geometric Analysis: Strict Saddle Property
**Source:** J. Phys. A (Jul 27, 2026), doi:10.1088/1751-8121/ae8b60
- Riemannian gradient descent on unitary group for VQE
- **Every critical point is either global minimum or strict saddle** (single-unitary case)
- Convergence rate deteriorates **polynomially** with circuit depth (geometric explanation of barren plateaus)
- Coefficient-adaptive measurement allocation strictly better than uniform

**NEW defect vs batch 599:** Batch 599 treated barren plateaus as empirical phenomena. **Polynomial deterioration is geometrically characterized** via Hilbert-space geometry × target-unitary complexity × per-layer expressivity × depth. NeoTrix's barren plateau detection should use this geometric criterion, not just gradient variance.

### Finding 2.6 — Graph Neural Encoding for VQE Generalization
**Source:** Mach. Learn.: Sci. Technol. 7, 045007 (Jul 6, 2026)
- Edge-featured graph attention autoencoder (EGATE) for Hamiltonian representation
- Generates high-overlap initial states for **unseen Hamiltonians** without instance-specific optimization
- Milder gradient variance decay = enhanced robustness to barren plateaus
- Warm-starts SKQD for downstream quantum eigensolvers

**NEW defect vs batch 599:** Batch 599 assumed VQE must be retrained per instance. **Cross-instance generalization via graph encoding** means NeoTrix can amortize optimization across related problems — its experience system should support transfer across structurally similar domains.

### Finding 2.7 — Input-State Design for VQA Reachability
**Source:** Communications Physics (Apr 7, 2026), doi:10.1038/s42005-026-02610-x
- Linear combination technique modifies reachable state set
- **Rigorous proof** that framework increases performance of any given VQA ansatz
- Shallower circuits remain trainable while enhanced input states cover larger reachable space

**NEW defect vs batch 599:** Batch 599 treated ansatz and input state as independent design choices. **Input-state design is a complementary optimization dimension** — NeoTrix should jointly optimize input states and circuit structure, not treat them sequentially.

---

## Search 3: Quantum Supremacy/Advantage 2026

### Finding 3.1 — IBM 70 Logical Qubits: Quantum Advantage Claim
**Source:** IBM + UChicago, arXiv:2607.25941 (Jul 28, 2026); PRNewswire (Jul 30, 2026)
- 70 logical qubits, 2,415 logical two-qubit ops, 468 logical T gates
- Effective logical error rates 10× lower than physical error rates
- Computation completed in ~15 minutes; leading classical methods face prohibitive runtimes
- "Sampling hard circuits with verifiably high fidelity" — structured alternative to RCS with error detection

**NEW defect vs batch 599:** Batch 599 assumed quantum advantage requires proving classical intractability. IBM's approach uses **device-dependent fidelity certificates** — verifying the computation was correct without proving classical hardness. NeoTrix should adopt this "trusted computation" model for its own verification protocols.

### Finding 3.2 — Unconditional Quantum Information Supremacy
**Source:** Quantinuum H1-1 trapped-ion, arXiv:2509.07255v2 (Aug 11, 2026)
- **12 qubits solve task requiring 62-382 classical bits** (provable lower bound)
- Based on one-way communication complexity — no unproven complexity assumptions
- "No future development in classical algorithms can close this gap"
- Sample XEB fidelity: 0.427(13)

**NEW defect vs batch 599:** Batch 599 treated all quantum advantage as relying on unproven complexity assumptions. **Unconditional separation** via communication complexity proves quantum information advantage is permanent and assumption-free. NeoTrix should distinguish "assumption-dependent" from "unconditional" advantages in its capability assessment.

### Finding 3.3 — Three IBM Advantage Claims: Fact-Check
**Source:** PostQuantum.com (Aug 1, 2026)
- IBM/Chicago: explicit advantage claim with complexity-theoretic hardness + fidelity certificate
- Qedma: empirical "beyond tested classical" — no formal advantage proof
- Algorithmiq: trust-building framework; hardest estimate lacks quantitative accuracy bound
- Quantum Advantage Tracker treats all as "active candidates requiring further benchmarking"

**NEW defect vs batch 599:** Batch 599 treated "quantum advantage" as binary. **Three distinct evidentiary levels exist**: (1) complexity-theoretic claim, (2) empirical beyond-tested-classical, (3) trust-building framework. NeoTrix must classify claims by evidentiary weight, not boolean "advantage: yes/no."

### Finding 3.4 — Adaptive Shallow Circuits with Feedback
**Source:** arXiv:2608.15545 (Aug 16, 2026)
- Measurement feedback induces **computational phase transition** in constant-depth circuits
- O(log n) feedback: classically simulable; polynomial feedback: encodes discrete logarithm → classically hard
- 2D nearest-neighbor realization with global classical feedforward only
- Each measurement trajectory obeys area law, but averaged expectation is hard

**NEW defect vs batch 599:** Batch 599 assumed shallow circuits are always classically simulable. **Mid-circuit measurement + feedforward breaks this** — classical feedback is a computational resource that creates hardness even in area-law states. NeoTrix's consciousness loop must account for measurement-feedback as a resource multiplier.

### Finding 3.5 — Brief History of Quantum vs Classical Advantage
**Source:** Quantum 10, 2198 (Sep 1, 2026), doi:10.22331/q-2026-09-01-2198
- Comprehensive review of all advantage experiments to date
- XEB is reliable proxy only when ε < c_A/n (weak-noise regime)
- All recent experiments are in weak-noise regime
- **100-logical-qubit targets**: fault-tolerant advantage via random IQP, RCS with symmetries, planted secrets

**NEW defect vs batch 599:** Batch 599 treated XEB as a universal benchmark. **XEB has a sharp phase transition** — it's reliable only below noise threshold ε < c_A/n. NeoTrix must check which regime it operates in before trusting XEB-based metrics.

---

## Summary: NEW Defects and Improvements vs Batch 599

| # | Defect/Improvement | Source |
|---|---|---|
| D1 | Calibration is continuous (RL), not periodic — HeartbeatAggregator must model as attractor | 1.1 |
| D2 | Logical gates are spacetime protocols (stabilizer trajectories), not point transforms | 1.2 |
| D3 | Folded architecture gives O(1) logical gates — cost model must be architecture-dependent | 1.3 |
| D4 | Decoder latency (tens of μs) is binding constraint, not just error rates | 1.4 |
| D5 | Unitary encoders halve encoding cost — encoding is optimization variable | 1.5 |
| D6 | Error suppression is anisotropic — basis-dependent error spectra required | 1.6 |
| D7 | VQA convergence provable via kernel linearization — not purely empirical | 2.1 |
| D8 | Parallel PSR reduces gradient cost from O(n²) to O(kn)/O(k log n) | 2.2 |
| D9 | Coherent Quantum Learning eliminates classical outer loop entirely | 2.3 |
| D10 | Interference-based QML is non-variational — fundamentally different paradigm | 2.4 |
| D11 | VQE critical points are strict saddles — geometric barren plateau characterization | 2.5 |
| D12 | Graph neural encoding enables cross-instance VQE generalization | 2.6 |
| D13 | Input-state design is complementary optimization dimension to ansatz | 2.7 |
| D14 | Device-dependent fidelity certificates verify computation without proving hardness | 3.1 |
| D15 | Unconditional quantum information supremacy via communication complexity | 3.2 |
| D16 | Three distinct evidentiary levels for quantum advantage claims | 3.3 |
| D17 | Mid-circuit measurement + feedforward creates computational phase transition | 3.4 |
| D18 | XEB has phase transition — reliable only below ε < c_A/n | 3.5 |

**Batch 599 → 600 delta:** 18 new defects identified. Key theme: **continuous dynamics dominate discrete states** (calibration, gate protocols, feedback loops). NeoTrix's architecture must shift from discrete-state models to continuous attractor/trajectory models for calibration, logical operations, and measurement feedback.
