# Iteration Batch 656 — Decision Theory / MCDM / Game Theory Sweep

**Date**: 2026-09-06
**Prior batch**: 655 (Kanai GMW orthogonality, no aligned-mediation metric, HCI→agent delegation, design tokens lack motion/voice/emotion, no purposeful forgetting)

---

## Sources Cited

| # | Source | Domain |
|---|--------|--------|
| 1 | Stanford Encyclopedia of Philosophy, "Normative Theories of Rational Choice: Expected Utility" (Spr/Sum 2026) | Decision Theory |
| 2 | Stanford Encyclopedia, "Normative Theories of Rational Choice: Rivals to Expected Utility" (Sum 2026) | Decision Theory |
| 3 | UW Decision Theory Course, Spring 2026 Lectures (Conor Walsh) | Decision Theory |
| 4 | ADT 2026 — 9th Intl Conf on Algorithmic Decision Theory, Paris Nov 2026 | Decision Theory |
| 5 | Das (2026), "Boundedly Rational Expected Utility Theory" (MPRA) | Decision Theory |
| 6 | Springer (2026), "The coexistence of loss aversion and regret aversion in decision making under risk" | Regret Theory |
| 7 | SAGE Open (2025), "Trust Under Bounded Rationality: Exploring Human-AI Interaction" | Bounded Rationality + AI |
| 8 | Duperrin (2026), "Bounded rationality in management in the age of AI" | Bounded Rationality + AI |
| 9 | Acadlore (2026), "A Regret-Based Generative AI Framework for Human-Robot Interaction" | Regret + AI |
| 10 | Springer (2026), "A systematic review of MCDM methods applied to personnel selection" | MCDM |
| 11 | ResearchGate (2026), "An Overview of MCDA and Applications of AHP and TOPSIS" | MCDM |
| 12 | MDPI Technologies (2025), "Overview of Existing MCDM Methods in Industrial Environments" | MCDM |
| 13 | INT J FUZZY SYST (2026), "An Extended G-TOPSIS Method Based on Prospect Theory" | MCDM + Prospect Theory |
| 14 | ResearchGate (2026), "OPTBIAS: Ordering preference targeting at bi-ideal average solutions" | MCDM |
| 15 | ScienceDirect (2026), "decideXpert: Collaborative system using AHP-TOPSIS and fuzzy" | MCDM + AI |
| 16 | Nature Communications (2026), "Discovering expert-level Nash equilibrium algorithms with LLMs" | Game Theory + LLM |
| 17 | ScienceDirect (2026), "Safe implementation in mixed Nash equilibrium" | Game Theory |
| 18 | ScienceDirect (2026), "On the existence of a strong Nash equilibrium under EDA mechanism" | Game Theory |
| 19 | Taylor & Francis (2026), "Distributed Nash equilibrium seeking for nonsmooth games" | Game Theory |
| 20 | ResearchGate (2026), "Distributed Nash Equilibrium Seeking with Dynamic Set of Players" | Game Theory |
| 21 | arXiv (2026), "Game-Theoretic Lens on LLM-based Multi-Agent Systems" (Hao et al.) | Game Theory + MAS |
| 22 | arXiv (2026), "Bilevel Coordinated Reflection: A Game-Theoretic Approach to Multi-Agent LLM Systems" (Chen et al.) | Game Theory + MAS |
| 23 | AAAI 2026, "Adaptive Theory of Mind for LLM-based Multi-Agent Coordination" (Mu et al.) | ToM + MAS |
| 24 | ACL 2026, "Towards Effective and Efficient Multi-Agent Language Model Systems" | MAS + LLM |
| 25 | JIMO (2026), "Mechanism design and equilibrium analysis of smart contract-mediated resource allocation" | Mechanism Design |
| 26 | TARA Publications (2026), "A security aware discrete mathematics framework for Nash equilibrium-based incentive design" | Game Theory + Security |

---

## NEW Defects Found

### DEFECT-656.1: NeoTrix Has No Formal Decision-Theoretic Foundation

**Severity**: CRITICAL
**Evidence**: Stanford Encyclopedia (Sources 1,2) documents that expected utility theory faces four major challenges (Allais Paradox, Ellsberg Paradox, Buchak's risk-rationality, bounded rationality). ADT 2026 (Source 4) is an entire conference dedicated to algorithmic decision theory. Das (2026) (Source 5) formalizes "Boundedly Rational Expected Utility Theory." NeoTrix's E8 reasoning engine uses utility maximization without formal decision-theoretic foundations — no axiomatic basis, no utility representation theorem, no handling of violations of transitivity/Sure-Thing Principle.

**Impact**: All decisions made by NT-CORE reasoning engine lack theoretical grounding. When the system encounters Allais-type or Ellsberg-type scenarios (ambiguity aversion), it has no principled way to resolve them.

**Fix**: Implement `nt_core_decision` module with: (a) Expected Utility maximizer as default, (b) Cumulative Prospect Theory extension for risk/ambiguity aversion, (c) Bounded Rationality satisficing mode with aspiration-level adaptation, (d) Regret-aware variant. All variants must satisfy axiomatic checks (transitivity, completeness, independence).

---

### DEFECT-656.2: No Regret-Aware Decision Loop

**Severity**: HIGH
**Evidence**: Springer (Source 6) empirically demonstrates that loss aversion and regret aversion coexist as distinct psychological mechanisms — regret is NOT reducible to loss aversion. Acadlore (Source 9) proposes a dual-regret model architecture for generative AI HRI. NeoTrix's NT-FEEL EmotionEngine has EmotionLabel enum but NO regret signal. The emotion engine treats all negative valence uniformly.

**Impact**: After a suboptimal decision, NeoTrix cannot distinguish between "I lost utility" (loss) and "I could have done better" (regret). Regret is a forward-looking signal that drives behavior change; loss is backward-looking. Without regret, the system cannot learn from counterfactual alternatives.

**Fix**: Add `RegretSignal` to NT-FEEL: `pub struct RegretSignal { counterfactual_utility: f64, chosen_utility: f64, information_set_at_decision: InformationState }`. Regret feeds into NT-MIND's learning loop as a distinct signal from loss. Implement the dual-regret architecture from Source 9.

---

### DEFECT-656.3: No Multi-Criteria Decision Making Module

**Severity**: HIGH
**Evidence**: MCDM literature is vast (Sources 10-15). AHP, TOPSIS, PROMETHEE, VIKOR are standard methods. decideXpert (Source 15) integrates AHP-TOPSIS with fuzzy logic. G-TOPSIS (Source 13) extends TOPSIS with prospect theory for group decisions. NeoTrix has NO formal MCDM capability. When selecting between providers, routing paths, or evolution strategies, the system uses ad-hoc weighted sums instead of principled MCDM methods.

**Impact**: NeoTrix's multi-objective decisions (cost vs. quality vs. latency vs. safety) lack theoretical rigor. No Pareto front analysis, no ideal/anti-ideal distance computation, no sensitivity analysis on criteria weights.

**Fix**: Implement `nt_core_mcdm` module: (a) TOPSIS for rapid alternative ranking, (b) AHP for criteria weight elicitation, (c) PROMETHEE for pairwise preference outranking, (d) Fuzzy extensions for uncertainty. This module lives in L5 Cognition layer and is called by NT-CORE decision engine and NT-ACT resource allocation.

---

### DEFECT-656.4: No Game-Theoretic Interaction Model Between Agents

**Severity**: CRITICAL
**Evidence**: Hao et al. (Source 21) provide a comprehensive game-theoretic framework for LLM multi-agent systems organized around players/strategies/payoffs/information. Chen et al. (Source 22) model orchestrator-worker as a bilevel coordination game with convergence analysis. Mu et al. (Source 23) demonstrate Theory of Mind is essential for coordination. NeoTrix has 10+ domains (NT-CORE through NT-FEEL) but NO game-theoretic model of their interactions. Domains compete for resources, attention, and EventBus bandwidth without formal strategic analysis.

**Impact**: Domain conflicts (e.g., NT-SHIELD blocking NT-WORLD crawls, NT-MIND demanding compute vs. NT-ACT needing latency) are resolved ad-hoc. No equilibrium concept, no mechanism design, no convergence guarantee. The system can oscillate between conflicting domain priorities.

**Fix**: Model domain interactions as a structured game: (a) Define payoff functions per domain (utility of successful crawl, cost of security violation, etc.), (b) Compute Nash equilibrium of domain resource allocation, (c) Implement mechanism design (Source 25) for incentive-compatible resource sharing, (d) Add convergence analysis (Source 22 drift bounds) to ensure stability.

---

### DEFECT-656.5: No Theory of Mind for Multi-Domain Coordination

**Severity**: HIGH
**Evidence**: Mu et al. (Source 23, AAAI 2026) show adaptive ToM dramatically improves LLM multi-agent coordination. LLM-Coordination benchmark (referenced in Source 21) reveals LLM agents are strong at Environment Comprehension but weak at Theory of Mind Reasoning (<40% accuracy on Joint Planning). NeoTrix domains operate as if other domains have perfect information — no modeling of what NT-WORLD "knows" vs. what NT-MEMORY "believes" vs. what NT-SHIELD "suspects."

**Impact**: When NT-CORE asks NT-WORLD to crawl, it doesn't model NT-WORLD's information state. When NT-MIND distills, it doesn't consider what NT-MEMORY has already cached. Cross-domain coordination is brittle because no agent reasons about other agents' beliefs.

**Fix**: Add `DomainToM` trait: `fn model_belief(&self, domain: DomainId) -> BeliefState; fn infer_intention(&self, domain: DomainId, observed_action: Action) -> IntentionEstimate`. Each domain maintains a lightweight belief model of sibling domains. Updated via EventBus observations.

---

### DEFECT-656.6: Bounded Rationality Not Modeled — AI Shifts Bottleneck, Doesn't Eliminate It

**Severity**: MEDIUM
**Evidence**: Duperrin (Source 8) and SAGE (Source 7) both argue that AI doesn't remove bounded rationality — it shifts the constraint from data collection to interpretation. "The bottleneck is no longer in data collection but in the system's ability to interpret the correlations produced." NeoTrix's NT-CORE assumes unbounded computation for reasoning tasks. There's no aspiration-level adaptation, no satisficing threshold, no computation budget allocation.

**Impact**: When facing complex multi-domain decisions, NT-CORE may spend excessive compute trying to optimize when satisficing would be sufficient. No graceful degradation under cognitive load.

**Fix**: Implement `BoundedRationalityController` in L6 Meta-Cognition: (a) Track computation budget per decision, (b) Set dynamic aspiration levels based on available resources, (c) Switch from optimization to satisficing when budget exhausted, (d) Log when satisficing was used vs. optimization for learning.

---

### DEFECT-656.7: No Mechanism Design for Incentive Compatibility

**Severity**: HIGH
**Evidence**: JIMO (Source 25) proves existence/uniqueness of contract equilibria for smart-contract-mediated resource allocation with fairness-efficiency tradeoffs. TARA Publications (Source 26) designs Nash equilibrium-based incentive mechanisms against collusion. NeoTrix has no mechanism design — domains don't have incentive-compatible reporting. A domain can exaggerate its resource needs to get more EventBus bandwidth.

**Impact**: Resource allocation between domains is not strategyproof. Domains can "game" the system. No Vickrey-Clarke-Groves mechanism, no auction theory, no dominant-strategy implementation.

**Fix**: Implement `nt_mechanism_design` module: (a) VCG mechanism for domain resource allocation (truthful reporting is dominant strategy), (b) Scoring rule for domain health reporting (proper scoring), (c) Auction-based provider selection for NT-ACT tool calls.

---

### DEFECT-656.8: No Convergence Analysis for Multi-Agent Self-Improvement Cycles

**Severity**: HIGH
**Evidence**: Chen et al. (Source 22) prove convergence rates for bilevel coordinated reflection with drift analysis: `E[R_{t+1}] <= R_t - c*R_t^{1+beta}`. They show "hallucination cascades" as documented failure mode. NeoTrix's SEAL pipeline (evolution loop) and ConsciousnessTree growth cycle have NO convergence proof. The system could oscillate, diverge, or plateau without detection.

**Impact**: NT-MIND's self-evolution could enter pathological states (constant refactoring without improvement, knowledge base bloat, skill tree over-specialization) with no mathematical guarantee of convergence.

**Fix**: Add convergence monitor to NT-META: (a) Define regret metric `R_t = U*(optimal) - U_t(current)` per growth cycle, (b) Track drift `Delta_t = E[R_t - R_{t+1}]`, (c) If drift < epsilon for K cycles, trigger meta-intervention (architecture review), (d) If drift < 0 (divergence), trigger rollback.

---

### DEFECT-656.9: No Prospect-Theory-Weighted Decision Criteria

**Severity**: MEDIUM
**Evidence**: G-TOPSIS (Source 13) extends TOPSIS with prospect theory value function for group decision-making. The value function is concave for gains, convex for losses, steeper for losses (loss aversion λ≈2.25). NeoTrix's E8 reasoning treats all utility differences linearly. No reference-point dependence, no probability weighting.

**Impact**: NeoTrix overweights small probability events (e.g., rare security threats) or underweights high-probability low-impact events in the same way expected utility predicts. Real agents use prospect theory weighting.

**Fix**: Add `ProspectValueFunction` to NT-CORE: `fn v(x: f64, reference: f64) -> f64` with parameters α (gain sensitivity), β (loss sensitivity), λ (loss aversion). Apply to all utility computations before decision selection.

---

### DEFECT-656.10: No Distributed Equilibrium Seeking for Decentralized Domains

**Severity**: MEDIUM
**Evidence**: Liang (2026, Source 19) solves distributed Nash equilibrium seeking for nonsmooth games using small-gain method. Liu et al. (2026, Source 20) handle dynamic player sets. NeoTrix's domains are semi-autonomous but coordinate through centralized EventBus. No distributed protocol for reaching agreement without central coordinator.

**Impact**: EventBus is single point of failure and bottleneck. If EventBus degrades, all cross-domain coordination fails. No graceful degradation to distributed mode.

**Fix**: Implement `DistributedConsensus` trait for domain-to-domain communication: (a) Each domain maintains local strategy, (b) Gossip protocol for approximate agreement, (c) Small-gain convergence guarantee, (d) Fallback from centralized EventBus to distributed consensus under load.

---

## Summary

| # | Defect | Severity | Layer |
|---|--------|----------|-------|
| 656.1 | No formal decision-theoretic foundation | CRITICAL | L5 Cognition |
| 656.2 | No regret-aware decision loop | HIGH | L4 Emotion |
| 656.3 | No multi-criteria decision making module | HIGH | L5 Cognition |
| 656.4 | No game-theoretic interaction model | CRITICAL | L6 Meta-Cognition |
| 656.5 | No Theory of Mind for multi-domain coordination | HIGH | L5 Cognition |
| 656.6 | Bounded rationality not modeled | MEDIUM | L6 Meta-Cognition |
| 656.7 | No mechanism design for incentive compatibility | HIGH | L6 Meta-Cognition |
| 656.8 | No convergence analysis for self-improvement | HIGH | L6 Meta-Cognition |
| 656.9 | No prospect-theory-weighted decision criteria | MEDIUM | L5 Cognition |
| 656.10 | No distributed equilibrium seeking | MEDIUM | L1 Action |

## What's NEW vs. Batch 655

Batch 655 focused on: Kanai GMW orthogonality (perception), aligned-mediation metric gap (evaluation), HCI→agent delegation (interaction), design tokens missing motion/voice/emotion (UI), purposeful forgetting (memory).

Batch 656 shifts to **decision foundations**: NeoTrix lacks formal decision theory, game theory for multi-agent coordination, MCDM for multi-objective optimization, mechanism design for incentive compatibility, and bounded rationality modeling. The critical insight from 2026 literature is that **AI doesn't eliminate bounded rationality — it shifts the bottleneck from computation to interpretation** (Duperrin/SAGE), and **regret is a distinct psychological mechanism from loss aversion** (Springer 2026), requiring separate modeling in NT-FEEL.

**Pattern across batches**: NeoTrix has strong perception (Kanai GMW), strong memory (KB embeddings), strong emotion (EmotionLabel 11-variants), but **weak decision foundations** — no axiomatic basis for choices, no strategic interaction model, no convergence guarantees for self-improvement.
