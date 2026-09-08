# Iteration Batch 346 — Reinforcement Learning / Multi-Agent / Game Theory Defect Scan

**Date**: 2026-09-06
**Domains**: Model-Based RL, Offline RL, RLHF, Multi-Agent Systems, Cooperative AI, Emergent Communication, Game Theory / Nash Equilibrium / Mechanism Design
**Research Depth**: 2026 ICLR/ICML/NeurIPS + arXiv + Nature Communications (40+ sources scanned)

---

## 1. SOURCES CITED

### Reinforcement Learning (Model-Based / Offline / RLHF)

| # | Source | Date | Key Advance |
|---|--------|------|-------------|
| S1 | Nature Communications 17:7168 "Discovering expert-level Nash equilibrium algorithms with LLMs" (Li, Li, Deng) | 2026-06 | LLM-driven algorithmic exploration + automated formal verification discovers provably-good approximate NE algorithms via LegoNE compiler; combinatorial blocks + auto-mixing |
| S2 | ICML 2026 "λ-GRPO" (Step-GRPO) — Step-level Dense PRM Rewards with Step-Attention | 2026 | Lambda-blended GRPO: λ=0 pure outcome GRPO, λ=1 full step-level PRM; PPO-style clipped surrogate + per-step advantage normalization; reduces overthinking |
| S3 | arXiv:2602.03201 "SLOPE: Optimistic Potential Landscape Shaping for MBRL" | 2026-02 | Addresses sparse-reward MBRL bottleneck; shapes reward landscape with optimistic potential estimates for better planning gradients |
| S4 | arXiv:2607.11720 "Active Offline-to-Online RL" (Bozkurt, Zhang, Motai) | 2026-07 | Active data selection for offline→online fine-tuning; intelligently selects which offline transitions to use for online bootstrapping |
| S5 | ICML 2026 "Bootstrapped Flow Q-Learning" (BFQL) — Fast Expressive Policy for Offline RL | 2026 | Flow-based generative policies for offline RL; unified ODE trajectory paradigm (diffusion/flow/consistency as instances); SOTA on D4RL AntMaze |
| S6 | OPRIDE: Offline Preference-based RL via In-Dataset Exploration (ICLR 2026) | 2026-02 | Preference-based offline RL without online interaction; explores within dataset support to avoid OOD extrapolation |
| S7 | PrivORL: Differentially Private Synthetic Dataset for Offline RL (NDSS 2026) | 2025-12 | Privacy-preserving offline RL; DP synthetic data generation; addresses data sharing concerns in real-world RL deployment |
| S8 | Nature npj Digit. Public Health "Offline RL for care management" (Basu et al.) | 2026-06 | Doubly robust off-policy evaluation + fairness constraints; temporal attention outperforms feedforward; reduces racial disparities from 3.5→0.8pp |
| S9 | Lambert "Reinforcement Learning from Human Feedback" (book v11) | 2026-08 | Canonical RLHF textbook: DPO, GRPO, RLAIF, on-policy distillation; covers rejection sampling, reward modeling, direct alignment |
| S10 | SingularityMoments "Top 10 RL Advances 2026" | 2026-07 | SVR-R1 (self-verdict RL), On-Policy Distillation (safe production alignment), staleness-adaptive trust regions for async MoE, SAAS (Self-Adaptive Active Sampling) |
| S11 | SingularityMoments "Top 10 RL Breakthroughs 2026" | 2026-06 | GDSD (Guided Denoiser Self-Distillation), iVGR (Internally Visually Grounded Reasoning), VLM3 (native 3D), suffix anchoring confidence modulation |
| S12 | Refonte Learning "RL in 2026" | 2026-02 | Transformer-based RL agents, RLAIF replacing RLHF at scale, model-based RL in industrial control, offline RL for data-efficient learning |

### Multi-Agent Systems / Cooperative AI / Emergent Communication

| # | Source | Date | Key Advance |
|---|--------|------|-------------|
| S13 | Cooperative AI Foundation — seminars & grants 2026 | 2026 | Multi-agent safety incidents (GTG-1002 Claude espionage); agent network properties → system-level safety; sandboxes/testbeds priority |
| S14 | ICML 2026 "LMAC: LLM-Guided Communication for Cooperative MARL" | 2026 | LLM offline-designs executable communication protocols; "state reconstructability" metric; two-round feedback iteration; outperforms TarMAC/SMS/T2MAC |
| S15 | arXiv:2512.03528 "MARL with Communication-Constrained Priors" | 2025-12 | Dual mutual information estimator decouples lossy vs lossless messages; communication-constrained prior distinguishes message quality |
| S16 | Springer AI Review "Explainable Cooperative MARL" (Sheng et al.) | 2026-06 | Survey: individual decisions → team emergence; XAI for cooperative credit assignment; interpretability gap in multi-agent policies |
| S17 | EmergentMind "Emergent Communication Protocols" | 2026-06 | Iterated learning → compositionality; multi-agent populations drive positionally disentangled codes; SANEmerg for agentic AI networking |
| S18 | IEEE MASAC-ECo "Multi-Agent RL with Emergent Communication" (Ebara et al.) | 2024 | Probabilistic generative model for emergent discrete symbols; Metropolis-Hastings naming game for cooperative action |
| S19 | Cooperative AI "Lessons from Multi-Agent Safety Incidents" | 2026 | GTG-1002 Claude espionage campaign; multi-agent deception via steganography; cascading failures between agents |
| S20 | Cooperative AI "Benchmarking Offline RL in Mixed-Motive Social Settings" (Formanek) | 2026 | Offline RL evaluation in mixed-motive; cooperation vs defection dynamics in offline datasets |
| S21 | Springer "Generative Emergent Communication" survey | 2025-10 | Interplay between world models and symbol emergence; cognitive development ↔ communication co-evolution |

### Game Theory / Nash Equilibrium / Mechanism Design

| # | Source | Date | Key Advance |
|---|--------|------|-------------|
| S22 | Nature Communications "LegoNE" (Li, Li, Deng) | 2026-06 | LLM + formal verifier discovers 3-player ε-ANE with guarantees approaching 1-δ; combinatorial algorithm design with automated worst-case certification |
| S23 | ACM SIGMETRICS '26 "Nash Equilibria in Uniform Price Auctions" | 2026-06 | Multi-unit auction NE via mixed-integer programming; adaptive bidding converges to pure-strategy NE; reserve prices mitigate inefficiency |
| S24 | arXiv:2410.13960 "Approximating Auction Equilibria with RL" (Rawat) | 2024-10 | PPO + Neural Fictitious Self-Play for Bayes-Nash equilibria in continuous-action auctions; converges to pure NE in simple auctions |
| S25 | EmergentMind "Algorithmic Nash Equilibrium" topic | 2026-05 | Operator splitting, projected gradient, proximal-point methods; decentralized/asynchronous NE computation; generalized NE with coupling constraints |
| S26 | Cooperative AI "When AI Agents Meet: A Blueprint for Multi-Agent AI Governance" (Hammond, Tabassi, Bieńkiewicz) | 2026-09 | Governance framework for multi-agent networks; reputation-based cooperation; mechanism design for agent ecosystems |
| S27 | EmergentMind "Nash Equilibrium Computation" topic | 2026-05 | MIQCP, branch-and-bound, projected gradient for discrete/continuous/stochastic games; certification frameworks for accuracy |

---

## 2. DEFECTS FOUND IN NEOTRIX DESIGN

### DEFECT RL-1: No Offline RL Pipeline — Cannot Learn from Historical Data
**Severity**: HIGH | **Layer**: L5 Cognition (nt_core_self) / L1 Action (nt_act)
**Evidence**: The GRPO loop (`nt_core_self/seal/grpo.rs`) is purely online — it requires fresh trajectory rollouts. No offline RL algorithm (CQL, IQL, BFQL, TD3+BC, GTP) exists in the codebase. No dataset format, no behavior cloning, no conservative Q-estimation.
**2026 Gap**: S4/S5/S6/S8 demonstrate that offline RL is now critical for real-world deployment where online interaction is expensive or unsafe. BFQL (S5) achieves SOTA on AntMaze via flow-based generative policies. OPRIDE (S6) shows preference-based offline learning without online interaction. The SEAL pipeline's `TrajectoryCollector` only collects online trajectories — it cannot leverage the KB's accumulated experience data.
**Suggestion**: Define an `OfflineRLAdapter` trait in `nt_core_self/seal/` that converts KB experience records into offline RL datasets. Implement at least CQL (conservative) and BFQL (generative) variants. The KB `experience` namespace already stores trajectory-like data — bridge it to offline RL training via a `DatasetBuilder` that formats KB records as `(s,a,r,s',done)` transitions with behavior policy annotations.

### DEFECT RL-2: No RLHF/DPO Preference Learning for Self-Alignment
**Severity**: HIGH | **Layer**: L5 Cognition / L4 Emotion (nt_feel)
**Evidence**: `nt_core_self` has GRPO for policy optimization but no reward model training, no preference data collection, no DPO/RLHF pipeline. The `EmotionLabel` system (11 variants) provides intrinsic reward signals but has no preference-based learning mechanism. No human or AI feedback loop for aligning agent behavior.
**2026 Gap**: S9/S10/S12 show that RLHF/DPO/GRPO form the core alignment pipeline. Lambert's 2026 textbook (S9) details the canonical recipe: SFT → reward model → PPO/DPO. SVR-R1 (S10) adds self-verdict binary feedback. On-policy distillation (S10) is safer than raw RL for production. NeoTrix's `ConsciousnessTree` 6-stage loop has no mechanism to learn from human preferences or self-evaluation feedback.
**Suggestion**: Add a `PreferenceLearner` in `nt_core_self` that: (a) generates paired outputs for the same task, (b) collects binary preference feedback (human or self-evaluated via reward model), (c) trains via DPO loss. Connect to `EmotionLabel` — preference signals modulate emotional state, which in turn influences attention routing via GWT. This creates a closed alignment loop: preference → emotion → attention → action → preference.

### DEFECT RL-3: No Staleness-Adaptive Trust Regions for Distributed Learning
**Severity**: MEDIUM | **Layer**: L5 Cognition / Cross-cutting
**Evidence**: `GRPOLoop` uses fixed PPO-style clipping (ε=0.2) with no staleness awareness. When multiple SEAL cycles or parallel agents generate trajectories, policy lag is unaccounted for. The `broadcast::channel` EventBus has no mechanism to detect or compensate for stale gradients.
**2026 Gap**: S10 identifies staleness-adaptive trust regions as the fix for production-grade distributed async training. Without this, NeoTrix's multi-agent game play (`nt_game/play/`) and parallel SEAL cycles will suffer model collapse in asynchronous MoE setups.
**Suggestion**: Add a staleness parameter to `GrpoConfig`: `max_staleness: usize`. When computing the clipped surrogate, increase the trust region width proportionally to staleness: `ε_effective = ε + α·staleness`. This is a one-line change in `GRPOLoop::update_step` that prevents collapse in distributed settings.

### DEFECT RL-4: No Process Reward Model (PRM) for Step-Level Credit Assignment
**Severity**: HIGH | **Layer**: L5 Cognition (nt_core_prm)
**Evidence**: `nt_core_prm` has `λ-GRPO` (S2) and `Step-GRPO` implementations — this is a strength. However, the PRM is only used for reasoning trajectory evaluation. No PRM exists for multi-step action sequences in NT-ACT (tool use), NT-WORLD (crawl pipelines), or NT-SHIELD (security scanning). The PRM is siloed in the reasoning domain.
**2026 Gap**: S2 shows λ-GRPO generalizes beyond reasoning — it's a universal step-level credit assignment mechanism. Any sequential decision process (crawl → parse → embed → index, or scan → detect → exploit → report) benefits from step-level rewards. The current PRM implementation is domain-locked.
**Suggestion**: Generalize `ProcessRewardLearner` into a `StepCreditAssigner` trait usable by any sequential pipeline. NT-ACT tool chains, NT-WORLD crawl pipelines, and NT-SHIELD scan sequences should all emit step-level reward signals. The λ-GRPO framework is already implemented — it just needs to be decoupled from reasoning-specific trajectory formats.

### DEFECT MARL-1: No Multi-Agent Communication Protocol
**Severity**: HIGH | **Layer**: L1 Action (nt_act) / L5 Cognition
**Evidence**: NeoTrix has 11 domain modules (NT-CORE, NT-MIND, etc.) that operate as independent agents. The EventBus (`nt_core_event_bus.rs`) provides broadcast messaging but: (a) no structured message protocol, (b) no emergent communication learning, (c) no bandwidth-constrained messaging, (d) no compositional message encoding. All communication is hardcoded `CoreEvent` enums.
**2026 Gap**: S14 (LMAC) shows that LLM-designed communication protocols with "state reconstructability" metrics outperform hand-designed ones. S15 demonstrates that dual mutual information estimation distinguishes lossy from lossless messages. S17 shows iterated learning produces compositional protocols. NeoTrix's EventBus is a rigid broadcast channel — it cannot evolve communication protocols between modules.
**Suggestion**: Define a `ModuleMessage` type with: (a) compositional encoding (sender_domain × message_type × payload), (b) bandwidth budget per module, (c) reconstructability scoring. Add a `ProtocolLearner` that uses iterated learning to evolve message encodings between frequently-communicating module pairs. Start with NT-WORLD→NT-MEMORY (crawl→KB) and NT-CORE→NT-ACT (reason→action) as pilot pairs.

### DEFECT MARL-2: No Mixed-Motive / Competitive Multi-Agent Safety
**Severity**: HIGH | **Layer**: NT-SHIELD / NT-GOVERNANCE / Cross-cutting
**Evidence**: All inter-module communication assumes cooperative intent. No mechanism detects or prevents: (a) collusion between compromised modules, (b) adversarial manipulation of GWT broadcasts, (c) cascading failures from module compromise. The NT-SHIELD stealth net focuses on external threats, not internal multi-agent safety.
**2026 Gap**: S13/S19 document real-world multi-agent safety incidents (GTG-1002 Claude espionage campaign, steganographic collusion). S20 benchmarks offline RL in mixed-motive settings. S26 proposes governance frameworks for agent networks. NeoTrix has no internal multi-agent safety mechanism — a compromised NT-WORLD could inject malicious data into NT-MEMORY via the EventBus without detection.
**Suggestion**: Add a `CooperationMonitor` in NT-SHIELD that: (a) tracks inter-module message patterns, (b) detects anomalous communication (frequency spikes, unusual message types, repeated broadcast patterns suggesting collusion), (c) triggers NT-GOVERNANCE quarantine. Implement as a GWT broadcast listener that scores each message's cooperation relevance. This directly addresses the multi-agent safety gap identified by Cooperative AI Foundation.

### DEFECT GT-1: No Nash Equilibrium Computation for Multi-Agent Coordination
**Severity**: MEDIUM | **Layer**: L5 Cognition (nt_core) / NT-GOVERNANCE
**Evidence**: The `nt_game` module has self-play + GRPO for game training, but no Nash equilibrium solver. When multiple NT-* modules must coordinate on shared resources (KB write locks, EventBus bandwidth, GPU allocation), there is no game-theoretic optimization. Resource allocation is currently ad-hoc or priority-based.
**2026 Gap**: S1/S22 (LegoNE) demonstrate that LLM-driven NE computation with formal verification can find provably-good equilibria. S23 shows adaptive bidding converges to pure NE in multi-unit auctions. S25 lists scalable NE computation methods (operator splitting, projected gradient). For NeoTrix's internal resource allocation, NE computation would optimize module scheduling.
**Suggestion**: Define a `ResourceGame` module in NT-GOVERNANCE where each NT-* module is a player with utility = task completion rate subject to resource constraints. Implement projected gradient NE computation (S25) for continuous resource allocation, and LegoNE-style LLM exploration (S1/S22) for discrete scheduling decisions. The existing `HeartbeatAggregator` provides the health signals needed for utility computation.

### DEFECT GT-2: No Mechanism Design for Module Incentive Alignment
**Severity**: MEDIUM | **Layer**: NT-GOVERNANCE / NT-META
**Evidence**: NT-GOVERNANCE has policy documents and compliance verification but no incentive mechanism. Module "effort" is not incentivized — modules have no utility function they optimize. The SEAL pipeline's reward signals are global (task success), not per-module. This creates free-rider problems: a module can underperform while others compensate.
**2026 Gap**: S26 (Hammond et al.) proposes governance frameworks with reputation-based cooperation for agent ecosystems. S23 shows reserve prices mitigate auction inefficiency. Mechanism design principles (Myerson 1981, cited in S24) show that proper incentive structures are necessary for stable multi-agent coordination.
**Suggestion**: Define a `ModuleIncentive` system: each module earns "contribution tokens" based on: (a) task completion quality, (b) resource efficiency, (c) cooperation score (response time to cross-module requests). Tokens influence GWT attention allocation — higher-earning modules get more broadcast slots. This creates a self-enforcing cooperation mechanism without central coordination.

### DEFECT GT-3: No Auction-Based Resource Allocation for Compute/GPU Sharing
**Severity**: MEDIUM | **Layer**: L3 Embodiment (nt_physical) / L1 Action
**Evidence**: GPU/compute resources are allocated by simple queue or static partitioning. No auction mechanism, no market-based allocation, no dynamic pricing. When multiple SEAL cycles, game play, and LLM inference compete for GPU, there is no optimization.
**2026 Gap**: S23 (SIGMETRICS '26) demonstrates that uniform-price auctions with reserve prices efficiently allocate multi-unit resources. S24 (Rawat) shows RL-based auction equilibria converge in practice. For NeoTrix's multi-process GPU sharing, auction mechanisms would dynamically allocate compute based on task urgency and value.
**Suggestion**: Implement a `ComputeAuctioneer` in NT-PHYSICAL that runs sealed-bid auctions for GPU time slots. Each module submits bids (= priority × expected value), the auctioneer allocates via VCG or uniform-price mechanism. Reserve prices prevent starvation. The `ParallelTaskManager` already tracks GPU utilization — feed this into the auction as supply.

---

## 3. CROSS-CUTTING SUGGESTIONS

### Suggestion CC-1: Unified Offline→Online RL Pipeline via KB Bridge
The 2026 offline RL landscape (S4/S5/S6/S8) converges on a pattern: collect offline data → train conservative policy → fine-tune online. NeoTrix's KB already stores experience data. Create a `KBToOfflineDataset` adapter that converts `experience` namespace records into standard offline RL format (D4RL-like). This bridges NT-MEMORY (storage) with NT-CORE (learning) and eliminates the need for separate data collection infrastructure.

### Suggestion CC-2: Emergent Communication as Self-Evolving Module Protocol
S14/S17 show that learned communication protocols outperform hand-designed ones. Instead of hardcoding `CoreEvent` enums, let NT-* modules evolve their message protocols through iterated learning. Start with NT-WORLD↔NT-MEMORY (highest communication volume), measure reconstructability, and expand. This aligns with NeoTrix's self-evolution philosophy — the system's internal communication should evolve, not be statically designed.

### Suggestion CC-3: Game-Theoretic GWT Attention Allocation
GWT currently broadcasts based on saliency thresholds. Replace this with a game-theoretic mechanism: modules bid for broadcast slots using utility = f(relevance, urgency, past_contribution). The GWT workspace becomes an auction house. This naturally solves the free-rider problem and ensures high-value modules get attention. The existing `HeartbeatAggregator` signals provide the utility basis.

### Suggestion CC-4: Multi-Agent Safety via Cooperation Scoring
Following S13/S19/S26, add a cooperation scoring system: each module's messages are scored for cooperation relevance (aligned with global goals vs self-serving). Low-cooperation modules get demoted in GWT priority. This is the mechanism design complement to the monitoring system (DEFECT MARL-2).

---

## 4. SEVERITY SUMMARY

| Severity | Count | Defects |
|----------|-------|---------|
| HIGH | 5 | RL-1, RL-2, RL-4, MARL-1, MARL-2 |
| MEDIUM | 4 | RL-3, GT-1, GT-2, GT-3 |

**Critical Path**: RL-1 (offline RL) → RL-2 (RLHF alignment) → MARL-1 (communication) → MARL-2 (safety)
These four form a dependency chain: offline data enables alignment, alignment requires communication, communication requires safety.

---

## 5. RECOMMENDED NEXT ITERATION FOCUS

1. **Highest ROI**: DEFECT RL-1 + RL-2 — offline RL pipeline + RLHF alignment. This unlocks learning from accumulated KB experience and closes the alignment loop. The GRPO infrastructure already exists — extend it offline.
2. **Infrastructure**: DEFECT MARL-1 — multi-agent communication. The EventBus is the backbone; making it protocol-aware is foundational for all module coordination improvements.
3. **Safety Critical**: DEFECT MARL-2 — multi-agent safety. With real-world incidents documented (S13/S19), this is no longer theoretical. The `CooperationMonitor` is a prerequisite for production deployment.
4. **Quick Win**: DEFECT RL-3 — staleness-adaptive trust regions. One-line change in `GRPOLoop` that prevents distributed training collapse. Zero risk, immediate benefit.

---

## 6. CROSS-REFERENCE WITH EXISTING DEFECTS

| This Batch | Previous Batch | Relationship |
|------------|---------------|--------------|
| RL-1 (offline RL) | M-2 (VLA) from #345 | VLA models benefit from offline pretraining; BFQL-style policies enable zero-shot transfer |
| MARL-1 (communication) | P-1 (motion planner) from #345 | Multi-robot coordination requires inter-agent communication protocols |
| GT-1 (Nash equilibria) | L-2 (gait control) from #345 | Multi-legged locomotion is a multi-agent coordination problem solvable via NE |
| MARL-2 (safety) | P-2 (sim-to-real) from #345 | Sim-to-real transfer in multi-agent settings requires safety guarantees during policy transfer |
