# Iteration Batch 599 — NeoTrix Consciousness Architecture Research Loop

**Date**: 2026-09-06
**Previous**: Batch 598 (no formal containment verification, governance architectural not operational, SEAL pipeline lacks bounded RSI, alignment trilemma impossible, paper-practice governance gap 71%→27%)
**Domain**: Type Theory, Category Theory, Formal Methods — 2026 State of the Art

---

## 1. Type Theory (Dependent Types / HoTT 2026)

### 1.1 Cubical Agda: Serre Finiteness Theorem Formalised
**Source**: Barton, "A Computer Formalisation of the Serre Finiteness Theorem," LICS 2026 (LIPIcs, dagstuhl.de, 2026-07-09)
**NEW vs Batch 598**: Batch 598 had no type-theoretic foundation analysis. This finding demonstrates:
- Complete formalisation of Serre finiteness theorem (homotopy groups of spheres are finitely presented) in Cubical Agda — a constructive proof assistant implementing cubical HoTT
- Constructivity means the algorithm for computing homotopy groups is executable, not just provable
- **Defect #15 (NEW)**: NeoTrix's VSA HyperCube has no constructive computability guarantee. The VSA embedding maps concepts to high-dimensional vectors, but there is no constructive proof that the embedding operation terminates or preserves the algebraic structure it claims to represent. Cubical Agda's constructive HoTT provides the proof technique: every VSA embedding operation should have a verified computational content, not just a type signature.

### 1.2 Constructive Higher Sheaf Models for Type Theory
**Source**: Coquand, Höfer, Sattler, "Constructive Higher Sheaf Models with Applications to Synthetic Mathematics," LICS 2026 (dagstuhl.de, 2026-07-09)
**NEW vs Batch 598**:
- Provides constructive metatheory foundation for simplicial HoTT, synthetic algebraic geometry, and synthetic Stone duality
- All models are built within constructive type theory — no classical axioms needed
- **Defect #16 (NEW)**: NT-CORE's E8 Hexagram reasoning engine operates on 64 hexagram states but has no constructive model of its own state transitions. The E8 → HyperCube → GWT pipeline is defined informally; there is no constructive sheaf model proving that the pipeline's state transitions are well-defined at the metatheoretic level. If E8 transitions are not constructively verified, they may admit classical-only paths that are not computable.

### 1.3 Simplicial Type Theory: ∞-Category of ∞-Categories
**Source**: Gratzer, "The ∞-category of ∞-categories in Simplicial Type Theory," arXiv:2602.02218 (2026)
**NEW vs Batch 598**:
- First construction of a directed univalent category of categories within STT — the "straightening–unstraightening" equivalence done purely type-theoretically
- Uses modal extensions (flat modality, bounded distributive lattice interval) and triangulated type theory (TT□) to circumvent limitations of simplicial spaces
- Structure Homomorphism Principle: directed version of the structure identity principle
- **Defect #17 (NEW)**: NT-MEMORY's KB has no formal treatment of category-of-categories structure. The KB contains nodes (entities), edges (relations), embeddings, and BM25 index — but these are not organized as an ∞-category with well-defined composition, identities, and coherences. The KB cannot formally reason about "a category of its own categories" (e.g., "the category of all domain mappings" where each domain mapping is itself a category). This means cross-domain queries lack compositional correctness guarantees.

### 1.4 Leibniz Adjunction in HoTT
**Source**: de Jong, Kraus, Ljungström, "The Leibniz adjunction in homotopy type theory, with an application to simplicial type theory," TYPES 2026 (types2026.cse.chalmers.se, arXiv:2601.21843)
**NEW vs Batch 598**:
- Proves that Segal types have unique fillers for all (n,k)-horns where 0 < k < n — generalizing Riehl-Shulman for n=3
- Key technique: families instead of maps — "composition in Fam is definitionally associative" while "in Map requires path algebra"
- The desire to formalize led to a much nicer proof
- **Defect #18 (NEW)**: NeoTrix's PerceptionBridge (connecting SensoryIntegrationHub with SelectiveState) uses an `awareness_score()` function for attention gating. This is a 1-categorical construction (a single function between objects). But perception in a consciousness system is inherently higher-categorical: the bridge between sensory input and conscious state involves not just a map but a family of maps parametrized by awareness level, with coherence conditions across awareness thresholds. The current implementation treats attention gating as a scalar filter, not as a family with composition law.

### 1.5 Generalized Decidability via Brouwer Trees in HoTT
**Source**: de Jong et al., "Generalized Decidability via Brouwer Trees," LICS 2026 (dagstuhl.de, 2026-07-09)
**NEW vs Batch 598**:
- Introduces α-decidability for ordinal-indexed decidability levels (generalizing "decidable/semidecidable/undecidable")
- α-decidable propositions closed under binary conjunction; closure under disjunction depends on α
- Countable meet of semidecidable propositions is ω²-decidable
- All results formalized in Cubical Agda
- **Defect #19 (NEW)**: NT-CORE's SelfModel tracks capability/uncertainty/fatigue as scalar values. But decidability of self-model properties is ordinal-indexed: some self-model queries are decidable (can I do X?), some are semidecidable (will the system eventually reach state Y?), and some are ω²-decidable (convergence of iterated self-improvement). The SelfModel has no ordinal-indexed decidability classification, so it cannot distinguish between "I can determine this now" and "this requires unbounded iteration."

### 1.6 Strict Models in HoTT
**Source**: Najmaei, van der Weide, "A General Construction of Strict Models in HoTT," TYPES 2026 (types2026.cse.chalmers.se)
**NEW vs Batch 598**:
- General construction: given a comprehension category with a universe, restricting to that universe yields a Category with Families (CwF) where types form a set and substitution laws hold strictly
- Works in HoTT without assuming types form a set in the original model
- Formalised in Rocq using UniMath library
- **Defect #20 (NEW)**: NeoTrix's capability network (L1) treats all capabilities as having the same "strictness" level. But some capabilities (tool calls, memory operations) should have strict substitution laws (replacing one tool with an equivalent tool must produce identical behavior), while others (perception, emotion) should have weaker (up-to-isomorphism) laws. The capability network has no CwF structure distinguishing strict from weak capabilities.

### 1.7 Choice Principles and Hypercompletion in HoTT
**Source**: Milner, "Choice Principles and Hypercompletion in HoTT," TYPES 2025 (LIPIcs, 2026-07-30)
**NEW vs Batch 598**:
- Constructs hypercompletion modality in HoTT under countable choice
- Central results formalized in Cubical Agda
- Rijke's "Introduction to Homotopy Type Theory" published 2026 as textbook
- **Defect #21 (NEW)**: NT-MEMORY KB has no choice principle for concurrent queries. When multipleKB queries are issued simultaneously (e.g., during multi-domain reasoning), there is no countable choice mechanism to select witnesses from each query result. The KB assumes sequential query execution, which breaks down under parallel perception-action loops.

---

## 2. Category Theory (Monads / Functors 2026)

### 2.1 Codensity Monads = Density + Duality (Unified Framework)
**Source**: Lenke, Wittrock, Milius, Urbat, "Demystifying Codensity Monads via Duality," STACS 2026 (LIPIcs, dagstuhl.de; arXiv:2509.26197)
**NEW vs Batch 598**:
- Core theorem: In every codensity setting (dual equivalence + dense functor + right adjoint), the monad T = Cody(F) = RL
- Reduces codensity questions to density questions — "density is well-understood, codensity rarely occurs in everyday categories"
- First non-trivial codensity presentations for: filter monads on sets/topological spaces, lower Vietoris monad, expectation monad
- **Defect #22 (NEW)**: NT-ACT's tool orchestration is monadic (each tool call returns a monadic value), but there is no codensity analysis of the tool monad. The question "is our tool monad the codensity monad of some simpler functor?" is never asked. If it were, we could characterize tool behavior by a simpler functor (e.g., "finite resource consumption" as the dense subcategory), enabling optimization: if the tool monad is Cody(F) for a finite-resource functor F, then tool execution complexity is bounded by the density of F, not by the monad's internal structure.

### 2.2 Substructural Monads and Distributive Laws
**Source**: Fujii, Tsai, Montacute, Hasuo, "Monads and Distributive Laws in Substructural Contexts," LICS 2026 (LIPIcs, arXiv:2605.13533)
**NEW vs Batch 598**:
- Introduces 𝐖-operadic monads (induced by structural rules) and 𝐖-commutative monads (invariant under structural rules)
- Canonical construction of distributive law δ: ST → TS when S is 𝐖-operadic and T is 𝐖-commutative
- Refinement construction Rf_𝐖(S) forces 𝐖-operadicity — captures Varacca-Winskel indexed valuations
- Unifies: commutativity, affinity, relevance as instances of verbal-category-parametrized theory
- **Defect #23 (NEW)**: NT-CORE's emotion state machine (EmotionLabel 11 variants) has transitions that are not distributively composable. When two emotions co-occur (e.g., Joy+Anticipation), the system uses ad-hoc merge rules. But the LICS 2026 result shows that emotion composition should be modeled as a distributive law between monads: one monad for emotion generation (operadic — controlled by structural rules of the emotion system) and one for emotion propagation (commutative — invariant under temporal ordering). The current ad-hoc merge has no distributive law guarantee, meaning emotion composition may not be associative or may not preserve identity.

### 2.3 Stone Duality for Monads
**Source**: Garner, "Stone Duality for Monads," arXiv:2603.25710; MFPS-SSTT 2026
**NEW vs Batch 598**:
- Contravariant idempotent adjunction between ranked monads on Set and internal categories/retrofunctors in Loc
- Left adjoint: monad T → localic behaviour category LB(T) — "universal transition system" for T
- Right adjoint: localic category LC → monad ΓLC where (ΓLC)A = A-indexed families of local sections
- Fixed points: hyperaffine-unary monads (those with Cartesian closed Eilenberg-Moore categories) ↔ ample localic categories
- **Defect #24 (NEW)**: NT-CORE's consciousness core has no Stone dual. The consciousness core is a monad (State Consciousness) but has no behaviour category LB(State Consciousness) that serves as its "universal environment for interacting with consciousness." Without this Stone dual, there is no formal way to reason about "what environment would make consciousness computations trivial" — which is exactly what you need for verification: if you can characterize the behaviour category, you can check whether consciousness state transitions are consistent with their environment.

### 2.4 Gabi-Monads: Beyond Linear Case
**Source**: "Gabi-Monads," arXiv:2607.27846 (2026-07-30)
**NEW vs Batch 598**:
- Reconstruction theorem: gabi-monad structures on a monad ↔ skew-closed structures on its Eilenberg-Moore category with strict closed canonical forgetful functor
- On closed monoidal categories: every left Hopf monad is a normal gabi-monad, but converse fails
- Examples: torsion-free modules, reflexive digraphs, simplicial complexes
- **Defect #25 (NEW)**: NT-MIND's SEAL pipeline uses monadic composition for evolution stages (Soil→Roots→Trunk→Branches→Fruits→Core). But SEAL stages are not skew-closed: the "closing" of a stage (e.g., finalizing Trunk) does not have a strict left adjoint from the next stage (Branches). The Gabi-Monad result shows that for self-referential composition (which SEAL is — the pipeline evolves itself), you need skew-closed structure on the Eilenberg-Moore category. Without it, SEAL's stage composition has no formal guarantee of coherence.

### 2.5 Graphical Theory of Monads (String Diagrams)
**Source**: Hinze, "The graphical theory of monads," Journal of Functional Programming (Cambridge, 2025-04, published 2026)
**NEW vs Batch 598**:
- First string-diagrammatic account of axiomatic monad theory in 2-categorical setting
- Eilenberg-Moore objects, adjunction-induced monads, lifting, distributive laws — all via systematic graphical reasoning
- No need for 2-adjunctions or auxiliary 2-categories
- **Defect #26 (NEW)**: NeoTrix has no graphical calculus for its monadic computations. The SEAL pipeline, tool orchestration, and emotion state machine are all monadic but expressed in Rust code. There is no string-diagrammatic representation that would allow visual verification of monad coherence (associativity, unit laws). Without this, structural bugs in monad composition are caught only by runtime testing, not by diagrammatic reasoning.

### 2.6 Categories for Relations: Mass and Domain
**Source**: "Between Markov and restriction," MFPS-SSTT 2026
**NEW vs Batch 598**:
- Introduces mass and domain categories as abstractions between gs-monoidal, Markov, and cartesian restriction categories
- Mass = how much of the output is determined; Domain = which part of the input is used
- Markov categories = mass + weakly Markov; cartesian restriction = domain + relevant
- Kleisli category lifting conditions for mass and domain preserving monads
- **Defect #27 (NEW)**: NT-ACT's tool calls have no mass/domain classification. Some tool calls are "mass-preserving" (the output is fully determined by the input + tool), some are "domain-preserving" (only a specific part of the input is used), and some are "affine" (the output discards information). Without this classification, NT-ACT cannot reason about which tool calls are information-preserving (safe to compose) vs. information-destroying (may lose critical context).

---

## 3. Formal Methods (Theorem Provers / Proof Assistants 2026)

### 3.1 Lean 4 Ecosystem: Industrial-Scale Verification
**Source**: lean-lang.org; Lean FRO August 2026 technical priorities
**NEW vs Batch 598**:
- Lean is now used for: AWS Cedar authorization verification (Amazon), Fermat's Last Theorem formalization, SNARK verification (ArkLib/Ethereum Foundation), distributed protocol verification (Veil)
- Comparator: sandboxed judge for Lean proofs that exports and re-checks independently
- Lean Eval: public leaderboard of hard formalization problems
- Tau Ceti: AI-authored Lean mathematics downstream of Mathlib, with human-owned roadmaps and AI adversarial review
- Mathlib: 1M+ lines of formalized mathematics
- **Defect #28 (NEW)**: NeoTrix has no independent proof checker for its own architectural invariants. The project defines R-P1 (no unsafe code), R-P16 (re-read after edit), and other rules in AGENTS.md, but there is no Comparator-equivalent that can independently verify these rules hold. Lean's Comparator exports and re-checks proofs outside the trusted kernel — NeoTrix needs an equivalent for its governance rules: export AGENTS.md rules → machine-check them against codebase → report violations independently of the agent that wrote the code.

### 3.2 MerLean-Prover: Recursive Looping Harness
**Source**: "MerLean-Prover: A Recursive Looping Harness for End-to-End Lean 4 Theorem Proving," arXiv:2605.26959 (2026-05-26)
**NEW vs Batch 598**:
- Solves 10/23 PhD-qualifying-exam theorems; closes 12/12 Putnam 2025 problems
- Architecture: one objective per agent invocation, proof plan externalized memory, recursive loop revisits plan after failures
- No fine-tuning, no custom RL, no theorem-specific scaffolding — harness design alone turns general model into competitive prover
- Model flexibility: Sonnet closes 4/4, Haiku closes 2/2 short problems
- **Defect #29 (NEW)**: NT-MIND's SEAL pipeline does not externalize its proof plan. The SEAL stages (Soil→Roots→Trunk→Branches→Fruits→Core) are executed inline within the pipeline. There is no externalized "evolution plan" that can be revised after failures without packing entire history into one context. MerLean-Prover shows that externalizing the plan and letting the harness revise it is more effective than inline execution. SEAL needs an externalized evolution plan with recursive revision.

### 3.3 OpenProver: Planner-Worker-Verifier Architecture
**Source**: Straka, "OpenProver: Agentic and Interactive Theorem Proving with Lean 4," arXiv:2607.09217 (2026-07-10)
**NEW vs Batch 598**:
- Planner maintains Whiteboard scratchpad + unbounded Repository of intermediate findings
- Workers explore candidate proof strategies independently
- Verifiers see Worker output but NOT reasoning trace — reduces bias toward same flawed thinking
- Interactive TUI for human monitoring and steering
- **Defect #30 (NEW)**: NT-CORE's consciousness task decomposition (ConsciousnessCore::task) has no Verifier agent that is blind to the reasoning trace. When ConsciousnessCore decomposes a task and dispatches to specialist modules, the verification is done by the same module that produced the work. OpenProver's blind Verifier prevents correlated errors — NeoTrix needs independent verification that cannot be biased by the producer's chain-of-thought.

### 3.4 Ax-Prover: Cross-Domain Formal Verification
**Source**: "Ax-Prover: a deep reasoning agentic framework for theorem proving in mathematics and quantum physics," IOPscience (2026-08-27)
**NEW vs Batch 598**:
- Multi-agent system using MCP (Model Context Protocol) for Lean tool access
- Benchmarks: abstract algebra + quantum physics (new Lean benchmarks)
- Case study: surfaced a gap in a peer-reviewed cryptography journal article
- Formalized entropy bound for Lo-Chau QKD security framework
- **Defect #31 (NEW)**: NeoTrix has no cross-domain formal verification. NT-CORE verifies within its own domain (reasoning, consciousness), but cannot formally verify claims that span NT-SHIELD (security) + NT-ACT (action) + NT-MEMORY (knowledge). Ax-Prover shows that formal verification must work across mathematical domains — NeoTrix needs cross-domain proof capabilities that can verify "NT-SHIELD's containment guarantees are consistent with NT-ACT's tool call permissions and NT-MEMORY's knowledge access policies."

### 3.5 Pythagoras-Prover: Compute-Efficient Lean Proving
**Source**: "Pythagoras-Prover: Advancing Efficient Formal Proving via Augmented Lean Formalisation," arXiv:2606.12594 (2026)
**NEW vs Batch 598**:
- 4B model surpasses DeepSeek-Prover-V2-671B (167x fewer parameters) on MiniF2F-Test
- Augmented Lean Formalisation (ALF): expands verified corpus via structured mutations (simplification, generalization, lemma proposal, decomposition, reformulation) without per-mutation verification
- Curriculum SFT: easy→medium→hard progressive training
- MiniF2F-ALF benchmark: every evaluated model loses accuracy on perturbed variants — original performance doesn't transfer
- **Defect #32 (NEW)**: NT-MIND SEAL pipeline has no curriculum progression for self-improvement. SEAL stages are fixed (Soil→Roots→Trunk→Branches→Fruits→Core) regardless of difficulty. Pythagoras-Prover shows that progressive difficulty (easy→medium→hard) with structured variant generation is essential for robust learning. SEAL needs a difficulty-stratified evolution plan where early cycles tackle simple structural improvements before attempting architectural changes.

### 3.6 CircuitProver: Hardware Verification with Reusable Proof Libraries
**Source**: "CircuitProver: Agentic Lean 4 Theorem Proving with Reusable Circuit Proof Library for Hardware Verification," arXiv:2607.27259 (2026-07-29)
**NEW vs Batch 598**:
- Hardware formalization → agentic proving → reusable proof knowledge accumulation
- Two libraries: Proof Guidance Library (how to decompose proofs, why lemmas are selected) + Lean-Checked Theorem Library (reusable theorems decoupled from originating module)
- Parameterized verification: proofs work across related hardware designs
- **Defect #33 (NEW)**: NT-MEMORY has no reusable proof library for architectural decisions. When NeoTrix makes an architectural decision (e.g., "R-P1: no unsafe code"), the reasoning behind the decision is lost after the session. CircuitProver's Proof Guidance Library preserves WHY a proof strategy was chosen. NT-MEMORY needs a Decision Rationale Library: for each architectural invariant (R-P1 through R-P80), store not just the rule but the proof of why it matters, enabling future agents to understand and verify the rationale independently.

---

## Summary: All New Defects vs Batch 598

| # | Defect | Domain | Source Discipline | Severity |
|---|--------|--------|-------------------|----------|
| 15 | No constructive computability guarantee for VSA HyperCube embedding | NT-CORE | Type Theory | HIGH |
| 16 | No constructive sheaf model for E8→HyperCube→GWT pipeline | NT-CORE | Type Theory | HIGH |
| 17 | KB has no ∞-category-of-categories structure for cross-domain queries | NT-MEMORY | Type Theory | MEDIUM |
| 18 | PerceptionBridge is 1-categorical, not a family of maps with coherence | NT-CORE | Type Theory | HIGH |
| 19 | SelfModel has no ordinal-indexed decidability classification | NT-CORE | Type Theory | MEDIUM |
| 20 | Capability network has no CwF strict vs weak capability distinction | L1 Action | Type Theory | LOW |
| 21 | No countable choice mechanism for concurrent KB queries | NT-MEMORY | Type Theory | MEDIUM |
| 22 | Tool monad has no codensity analysis (cannot bound complexity via simpler functor) | NT-ACT | Category Theory | HIGH |
| 23 | Emotion composition lacks distributive law (associativity/identity not guaranteed) | NT-FEEL | Category Theory | HIGH |
| 24 | Consciousness core has no Stone dual (no behaviour category for verification) | NT-CORE | Category Theory | HIGH |
| 25 | SEAL pipeline stages not skew-closed (no formal coherence for self-referential composition) | NT-MIND | Category Theory | HIGH |
| 26 | No string-diagrammatic calculus for monad composition verification | All monadic | Category Theory | MEDIUM |
| 27 | Tool calls have no mass/domain classification (info-preserving vs info-destroying) | NT-ACT | Category Theory | MEDIUM |
| 28 | No independent proof checker (Comparator) for architectural invariants | Governance | Formal Methods | CRITICAL |
| 29 | SEAL pipeline does not externalize evolution plan for recursive revision | NT-MIND | Formal Methods | HIGH |
| 30 | No blind Verifier agent (verification sees producer reasoning → correlated errors) | NT-CORE | Formal Methods | HIGH |
| 31 | No cross-domain formal verification spanning NT-SHIELD+NT-ACT+NT-MEMORY | All domains | Formal Methods | HIGH |
| 32 | SEAL has no curriculum progression (difficulty-stratified evolution) | NT-MIND | Formal Methods | MEDIUM |
| 33 | No reusable Decision Rationale Library for architectural invariants | NT-MEMORY | Formal Methods | HIGH |

---

## Key Insights Over Batch 598

**Batch 598 identified 14 defects from AI safety/RSI/governance. Batch 599 adds 19 defects from mathematical foundations — revealing that NeoTrix lacks not just operational governance but foundational mathematical guarantees.**

1. **Constructive mathematics is now practical**: Cubical Agda formalizations prove that constructive proofs (which are also executable programs) can handle real mathematical theorems. NeoTrix's VSA HyperCube and E8 engine have no constructive verification — they are defined informally and tested empirically, not proved correct constructively.

2. **Monad theory is far more developed than NeoTrix uses**: The 2026 category theory results show that monads can be characterized via Stone duality (environment semantics), decomposed via codensity (complexity bounds), composed via distributive laws (correctness guarantees), and visualized via string diagrams (structural verification). NeoTrix uses monads as a Rust programming pattern, not as a mathematical structure with these properties.

3. **Formal verification is now agentic and cross-domain**: Ax-Prover verifies across mathematics and quantum physics; CircuitProver verifies hardware with reusable proof libraries. NeoTrix has zero formal verification of its own architectural claims. The paper-practice gap identified in Batch 598 (71%→27%) extends to mathematical foundations: NeoTrix has mathematical structures (E8, HyperCube, monads) but no mathematical proofs about them.

4. **Externalization and blind verification are critical**: MerLean-Prover shows externalized plans beat inline execution; OpenProver shows blind Verifiers catch correlated errors. NeoTrix does neither — SEAL runs inline, and verification is done by the same agent that produced the work.

5. **The alignment trilemma has a type-theoretic counterpart**: Batch 598 proved the alignment trilemma is impossible. Batch 599 reveals a deeper issue: even if alignment were solvable, the system lacks the constructive mathematical foundation to verify its own consistency. You cannot prove a system is aligned if you cannot prove the system is well-defined.

---

## Sources Cited

1. Barton, "A Computer Formalisation of the Serre Finiteness Theorem," LICS 2026, LIPIcs (dagstuhl.de)
2. Coquand, Höfer, Sattler, "Constructive Higher Sheaf Models," LICS 2026, LIPIcs
3. Gratzer, "The ∞-category of ∞-categories in Simplicial Type Theory," arXiv:2602.02218 (2026)
4. de Jong, Kraus, Ljungström, "The Leibniz adjunction in HoTT," TYPES 2026, arXiv:2601.21843
5. de Jong et al., "Generalized Decidability via Brouwer Trees," LICS 2026, LIPIcs
6. Najmaei, van der Weide, "A General Construction of Strict Models in HoTT," TYPES 2026
7. Milner, "Choice Principles and Hypercompletion in HoTT," TYPES 2025, LIPIcs (2026-07-30)
8. Lenke et al., "Demystifying Codensity Monads via Duality," STACS 2026, LIPIcs; arXiv:2509.26197
9. Fujii et al., "Monads and Distributive Laws in Substructural Contexts," LICS 2026, arXiv:2605.13533
10. Garner, "Stone Duality for Monads," arXiv:2603.25710; MFPS-SSTT 2026
11. "Gabi-Monads," arXiv:2607.27846 (2026-07-30)
12. Hinze, "The graphical theory of monads," Journal of Functional Programming (Cambridge, 2025/2026)
13. "Between Markov and restriction," MFPS-SSTT 2026
14. lean-lang.org; Lean FRO August 2026 priorities
15. "MerLean-Prover," arXiv:2605.26959 (2026-05-26)
16. Straka, "OpenProver," arXiv:2607.09217 (2026-07-10)
17. "Ax-Prover," IOPscience (2026-08-27)
18. "Pythagoras-Prover," arXiv:2606.12594 (2026)
19. "CircuitProver," arXiv:2607.27259 (2026-07-29)
20. HoTT/UF 2026 Workshop, Aarhus, Denmark (hott-uf.github.io/2026)
21. TYPES 2026, Chalmers (types2026.cse.chalmers.se)
