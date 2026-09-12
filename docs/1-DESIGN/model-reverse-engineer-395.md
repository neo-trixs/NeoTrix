# Model Reverse Engineering — Cycle 395 (2026-09-12)

## 5 New Models/Papers

---

### 1. CEDAR: Error-Bounded Residual Routing for Efficient Long-Context Attention

**Source**: arXiv:2609.07237 (Sep 7, 2026)

**Core Idea**: Coarse-to-fine sparse attention that preserves global coverage. Each semantic chunk contributes a cheap KV summary to a residual path; chunks with high approximation error are expanded to exact attention. Output-error bound governed by within-chunk key/value dispersion.

**Key Metrics**:
- Recovers most quality lost by hard sparse routing
- ~3x kernel speedup at 128K context
- Residual summaries reduce reconstruction error by 98%+ vs hard dropping

**NeoTrix Mapping**:
- **NT-CORE (GWT)**: Residual summary as lightweight salience scoring before full attention broadcast. Maps to GWT's coarse-then-refine attention routing.
- **NT-MEMORY (KB)**: Error-bounded chunk selection mirrors KB query → candidate → full retrieval pipeline.
- **Axiom Alignment**: A2 (Context as Scarce Resource) — bounded error budget per chunk.

---

### 2. Faster Than Flash Decoding (FFD)

**Source**: arXiv:2609.00097 (Aug 31, 2026, accepted ICML 2026)

**Core Idea**: Hardware-algorithm co-design that fuses selector+computer into one kernel. Replaces external metadata indices with content-aware scanning via low-bit quantization. Top-delta strategy dynamically filters blocks for distribution-adaptive sparsity without global synchronization.

**Key Metrics**:
- 11.6x kernel-level speedup
- Scales to 256K context length
- 2.37x end-to-end throughput improvement
- Training-free, plug-and-play

**NeoTrix Mapping**:
- **NT-CORE (E8)**: Fused kernel = fused reasoning path. No separate index→compute phases.
- **NT-PHYSICAL**: Hardware-algorithm co-design philosophy maps to NT-PHYSICAL's embodied constraints.
- **Axiom Alignment**: A1 (Cost-Aware Routing) — dynamic filtering = cheap models for easy queries.

---

### 3. PIVOT: Proxy Indexing Via One full-prefix Traversal

**Source**: arXiv:2607.24593 (Jul 27, 2026)

**Core Idea**: Training-free drop-in replacement for DeepSeek Sparse Attention indexer. Exploits that nearby queries select overlapping top-k tokens. Groups queries into proxy, performs one shared full-prefix scan, then refines per-query from candidate set.

**Key Metrics**:
- 4x acceleration of dense DSA indexer
- 1.6x reduction in end-to-end latency
- Matches dense indexer accuracy on DeepSeek-V3.2 and GLM-5.1

**NeoTrix Mapping**:
- **NT-CORE (GWT)**: Proxy query = GWT broadcast to salient subset. Query grouping = attention head specialization.
- **NT-MIND**: Training-free = skill crystallization without retraining (SEAL Phase-4).
- **Axiom Alignment**: A2 (Context as Scarce Resource) — one scan amortized across query group.

---

### 4. AgensFlow: Coordination-Policy Substrate for Multi-Agent Systems

**Source**: arXiv:2605.27466 (May 26, 2026)

**Core Idea**: Treats multi-agent coordination as online policy learning under partial observability. Inspectable policy graph over skills, models, and topology actions. Skip:X topology learning lets policy exclude skills per task-class. Reward-signal auditability as first-class design.

**Key Metrics**:
- Warm-started policy graphs reduce exploration cost by ~10-21% tokens
- Quality gains concentrated on coordination-heavy classes (+0.13 to +0.18)
- Cross-domain transfer validated (distributed systems → security advisories)

**NeoTrix Mapping**:
- **NT-CORE (GWT)**: Policy graph = GWT salience routing with learnable topology.
- **NT-MIND (SEAL)**: Online policy learning = SEAL evolution loop. Skip:X = skill pruning.
- **NT-GOVERNANCE**: Reward auditability = D1-D50 review dimensions.
- **Axiom Alignment**: P1 (Model Routing), P2 (Isolation-per-Task), P5 (Skill as Template).

---

### 5. NeuralFSM: Adaptive Multi-Agent Coordination via Finite-State Execution Policy

**Source**: ACL 2026 (acl-long.1543)

**Core Idea**: Formulates multi-agent problem solving as finite-state execution process. Learns state transition distribution and inter-agent communication weights via Temporal Coordination Controller (TGN-based). Dual-defense protection: graph regularization + trust-aware message attenuation.

**Key Metrics**:
- 6.74% - 19.39% average margin over baselines
- Substantially reduced token consumption via sparse routing
- Only 1.82% performance drop under adversarial attack (with protection layer)

**NeoTrix Mapping**:
- **NT-CORE (ConsciousnessTree)**: FSM states = ConsciousnessTree growth stages. Temporal coordination = cycle-based attention routing.
- **NT-SHIELD**: Dual-defense protection = trust-aware message filtering (adversarial robustness).
- **NT-MEMORY**: TGN-based history encoding = cross-session pattern persistence.
- **Axiom Alignment**: P2 (Isolation-per-Task) via FSM state isolation. P3 (Profile-Driven Adaptation) via task-conditioned transitions.

---

## Cross-Paper Synthesis

### Pattern 1: Sparse Attention as Routing (CEDAR + FFD + PIVOT)
All three papers solve the same problem: O(L²) attention is prohibitive. They converge on **coarse-to-fine** or **proxy-based** routing. NeoTrix's GWT already implements attention-as-routing; these papers provide the algorithmic backbone for making GWT sub-quadratic at 128K+ contexts.

### Pattern 2: Coordination as Learnable Policy (AgensFlow + NeuralFSM)
Both papers treat multi-agent coordination as a **learned routing problem** rather than a fixed pipeline. AgensFlow uses UCB1-selected action statistics; NeuralFSM uses temporal graph networks. Maps directly to NeoTrix's SEAL pipeline as a coordination policy learner.

### Pattern 3: Auditability as First-Class (AgensFlow reward audit + NeuralFSM protection layer)
Both papers make coordination decisions **inspectable and auditable**. Aligns with NeoTrix's指针守恒 and D1-D50 review dimensions.

### Pattern 4: Training-Free Methods Dominating (CEDAR + FFD + PIVOT)
All three sparse attention papers are training-free. This validates NeoTrix's SEAL skill crystallization approach: post-hoc adaptation without retraining.

## Integration Priority

| Paper | Priority | Integration Target | Effort |
|-------|----------|-------------------|--------|
| CEDAR | P1 | GWT refinement (error-bounded routing) | Medium |
| FFD | P1 | KV cache optimizer extension | Medium |
| PIVOT | P2 | DSA indexer replacement for NT-IO | Low |
| AgensFlow | P2 | SEAL coordination policy | High |
| NeuralFSM | P3 | ConsciousnessTree FSM formalization | High |
