# Iteration Batch 453 — Text Planning / Surface Realization / Narrative Generation Research

**Date**: 2026-09-06
**Scope**: Text Planning (NLG pipeline, document planning, discourse structure), Surface Realization (controlled generation, text generation architectures), Narrative Generation (story generation, plot generation, interactive storytelling)

---

## 1. Text Planning Research (2026)

### Sources

1. **INLG 2026 Call for Papers** — 19th International Natural Language Generation Conference, Utrecht, Netherlands, Oct 17-21, 2026. Topics include: content and text planning, NLG architectures, cognitive modeling of language production, storytelling and narrative generation, LLMs for NLG, explainability and trustworthiness of NLG systems. [2026.inlgmeeting.org]
2. **INLG 2026 Accepted Papers** — PLeW-NWG (interactive human eval data exploration), TruthSplit (multi-perspective reasoning graphs for contested claims), validated triple set collection for data-to-text. [2026.inlgmeeting.org/accepted-papers.html]
3. **"Which course? Discourse! Teaching Discourse and Generation in the Era of LLMs"** — arXiv 2602.02878, 2026. Comprehensive course design for discourse-aware NLG: RST, PDTB, QUD frameworks; entity-based coherence (Centering Theory, Entity Grid, DiscoScore); discourse structure informing content selection in summarization and planning in simplification. Highlights that LLMs "struggle with maintaining global consistency, logical flow, and discourse-level planning" despite surface fluency. [arXiv]
4. **"Beyond Chunking: Discourse-Aware Hierarchical Retrieval for Long Document QA"** — ACL 2026. RST-based hierarchical framework leveraging discourse structures for long-document comprehension. Discourse relations (elaboration, contrast, causal, temporal) used as retrieval anchors. [ACL Anthology 2026.acl-long.829]
5. **"Fit-for-Purpose Discourse"** — AJSLP 2026. Clinical focus article from ICCDC 2026: discourse elicitation tasks (e.g., Cinderella retell) remain valuable for cross-study comparison despite computational advances. Highlights tension between canonical tasks and modern neural approaches. [pubs.asha.org]
6. **Jurafsky & Martin 3rd Edition** — Speech and Language Processing, online manuscript released January 2026. Updated NLG pipeline coverage including discourse structure, content planning, and lexicalisation for the LLM era. [web.stanford.edu/~jurafsky/slp3/]

### Defects Found in NeoTrix Design

**DEFECT-NLG-1: No Discourse Structure Representation in Text Generation**
NeoTrix's text generation capabilities (NT-IO for LLM output, NT-MIND for distillation) operate without explicit discourse structure models (RST, PDTB, QUD). The 2026 discourse-aware NLG research demonstrates that discourse relations are critical for controlling coherence over long-form output.
- *Gap*: No `DiscourseStructureModel` trait in NT-MIND or NT-IO for guiding multi-paragraph generation.
- *Fix*: Add a `DiscoursePlanner` module in NT-MIND that constructs RST-style discourse trees before realization, encoding rhetorical relations (elaboration, contrast, causal, temporal) as constraints on the generation pipeline.

**DEFECT-NLG-2: No Entity-Based Coherence Tracking in Generation**
Centering Theory research (Grosz et al., Entity Grid, DiscoScore) shows entity-based coherence is a measurable, controllable property of generated text. NeoTrix has no mechanism to track entity focus, salience, or transition patterns across generated paragraphs.
- *Gap*: No `EntityCoherenceTracker` in NT-IO for maintaining entity-based discourse structure.
- *Fix*: Implement entity grid tracking in NT-IO: maintain entity mention lists per discourse unit, enforce Centering Theory transition rules (continue > retain > shift), and use coherence score as a post-generation filter.

**DEFECT-NLG-3: No Multi-Perspective Reasoning in Knowledge Synthesis**
TruthSplit (INLG 2026) demonstrates interactive generation of multi-perspective reasoning graphs for contested claims. NeoTrix's KB stores knowledge as single-fact nodes without explicit perspective tracking or contested-claim resolution.
- *Gap*: NT-MEMORY has no perspective-tagged knowledge representation.
- *Fix*: Add `PerspectiveTag` to KB node schema, enabling multi-viewpoint storage. Implement `PerspectiveResolver` in NT-MIND that synthesizes conflicting perspectives into structured reasoning graphs before presenting to users.

---

## 2. Surface Realization / Controlled Generation Research (2026)

### Sources

1. **"A Comparative Study of Controlled Text Generation Systems Using Level-Playing-Field Evaluation Principles"** — Lorandi & Belz, arXiv:2605.12395, May 2026. LPF evaluation reveals that published CTG performance claims "substantially misrepresent true system capabilities" when re-evaluated with standardized methods. Performance results differ substantially from originally reported, mostly worse. Highlights urgent need for standardized, reproducible evaluation. [arXiv]
2. **C3TG: Conflict-aware, Composite, and Collaborative Controlled Text Generation** — AAAI 2026. Two-phase framework: generation phase uses weighted KL divergence to fuse attribute distributions and adjust token probabilities; optimization phase uses energy function (classifier score + conflict penalty) with Feedback Agent iterative rewriting. 90.4% attribute accuracy across 17 attribute subclasses, significant toxicity reduction. [AAAI 2026 / papernotes.org]
3. **Controlled Generation Encyclopedia** — Agentica, March 2026. Comprehensive overview of CTG principles, history, and applications: prompt-based control, fine-tuning, decoding-time intervention, classifier-guided generation. [agentica.wiki]
4. **Causal Perspective on Controllable Text Generation** — ScienceDirect, 2025. Survey of CTG from causal perspective: existing approaches capture statistical association but lack causality consideration. Proposes causal interventions for controllable generation. [ScienceDirect]
5. **CVPR 2026: Yume1.5** — Text-controlled interactive world generation model combining long-video generation with context compression and linear attention, bidirectional attention distillation for real-time streaming. Demonstrates text-as-control-signal at scale. [CVPR 2026]

### Defects Found in NeoTrix Design

**DEFECT-CTG-1: No Standardized Evaluation Framework for Generated Output**
The Lorandi & Belz LPF study proves that CTG evaluation without standardized protocols produces unreliable results. NeoTrix has no internal evaluation framework for its text generation quality—no standardized metrics, no reproducible evaluation pipeline, no cross-system comparison methodology.
- *Gap*: No `GenerationEvaluationFramework` in NT-MIND for assessing output quality.
- *Fix*: Implement LPF-style evaluation in NT-MIND: standardized tokenization, shared evaluation methods (attribute accuracy, fluency, toxicity), and reproducible benchmarking across generation modes. Store evaluation results in KB for longitudinal tracking.

**DEFECT-CTG-2: No Multi-Attribute Conflict Resolution in Generation**
C3TG demonstrates that multi-attribute control requires explicit conflict resolution between competing constraints (e.g., formal tone vs. emotional expressiveness). NeoTrix's generation pipeline has no mechanism for detecting or resolving attribute conflicts.
- *Gap*: No `AttributeConflictResolver` in NT-IO.
- *Fix*: Add C3TG-inspired conflict detection in NT-IO: when multiple control attributes are specified, compute pairwise conflict scores via classifier ensemble, apply weighted KL divergence fusion with conflict penalty terms, and use iterative rewriting for optimization.

**DEFECT-CTG-3: No Causal Modeling in Generation Constraints**
The causal perspective survey shows that statistical association in training data leads to spurious correlations in controlled generation. NeoTrix's generation control is purely correlational—no causal graph guides constraint satisfaction.
- *Gap*: NT-MIND has no causal model for generation constraints.
- *Fix*: Add `CausalConstraintGraph` in NT-MIND that encodes cause-effect relationships between control attributes and output properties. Use causal interventions (do-calculus style) instead of conditional generation for more robust control.

---

## 3. Narrative Generation Research (2026)

### Sources

1. **"AI Interactive Storytelling and Narratives: 20 Advances (2026)"** — Yenra, updated March 2026. Comprehensive survey: dynamic story generation tied to world state, procedural content generation with emotional arcs, SCORE for narrative state tracking + context-aware summarization, Narrative Studio for entity-graph branch exploration, Elsewise for possibility-space visualization, PsyMem for character memory + psychological alignment, NVIDIA ACE for multimodal character systems. Key insight: "AI works best when constrained by world state, character memory, authored rules, and human editorial judgment." [yenra.com]
2. **"Best AI Story Generators in 2026"** — Runable, August 2026. Market analysis: $2B (2025) → $16B (2033) at 25% CAGR. Top pain points: plot holes and inconsistencies (45%), writer's block (30%), pacing (15%), character arcs (10%). Key finding: stateless chatbots fail at multi-chapter consistency; stateful systems with persistent memory are essential. [runable.com]
3. **"Story Plot Generator: Build Complete Narratives With AI"** — Jenova, April 2026. Detailed analysis of why plot generators fail: one-line premises without causation, memory that can't sustain plots, feature fragmentation. SCORE system (Yi et al., 2025) as anchor for narrative state tracking. 97% of content marketers plan to use AI for content creation. [jenova.ai]
4. **"Plot Generator AI: Build Story Structures That Actually Work"** — Jenova, May 2026. Deep dive on plot-as-architecture: cause-and-effect chains across distance ("a gun shown in act one must fire in act three"). Context window limitations cause character/conspiracy dropping at 8K tokens. SCORE and Narrative Studio as state-of-the-art. [jenova.ai]
5. **InkOS v1.4.1** — Open-source autonomous novel writing system, 6,200+ GitHub stars. Multi-stage pipeline: planning → composition → drafting → auditing → revision. 9-phase pipeline with 7 state panels (characters, plotlines, timelines, items, locations, POV rules, session memory). Human-in-the-loop confirmation at each phase. [fast.io/OpenClaw skills review]
6. **DeepFiction** — Branching narrative engine maintaining consistency across 23+ possible endings. Over 2M interactive stories created. Specializes in cause-and-effect consistency across divergent paths. [runable.com review]
7. **Elsewise (Wang et al., 2026)** — Authoring AI-based interactive narrative with possibility-space visualization. Treats branching as managed structure, not raw generation. [Yenra 2026 survey]
8. **SCORE (Yi et al., 2025)** — Story Coherence and Retrieval Enhancement: combines dynamic state tracking, context-aware summarization, and retrieval for long-range narrative coherence. [Yenra 2026 survey]

### Defects Found in NeoTrix Design

**DEFECT-NAR-1: No Narrative State Tracking for Multi-Step Workflows**
SCORE demonstrates that complex narratives require explicit state tracking (characters present, relationships changed, plot conditions true). NeoTrix's SEAL pipeline and multi-step workflows have no equivalent narrative state machine—no tracking of which knowledge entities are "present," what relationships have shifted, or which workflow conditions hold.
- *Gap*: NT-MIND lacks a `WorkflowStateTracker` analogous to SCORE's narrative state tracking.
- *Fix*: Add explicit state tracking in NT-MIND SEAL pipeline: maintain a structured state object tracking active knowledge entities, their relationships, and workflow conditions. Enable "story so far" summarization for long-running evolution cycles, analogous to SCORE's context-aware summarization.

**DEFECT-NAR-2: No Causal Chain Enforcement Across Workflow Steps**
Plot research (45% of writers struggle with plot holes = causal disconnections) directly maps to NeoTrix's SEAL pipeline: exploration → distillation → self-test → absorption can produce disconnected stages where distillation doesn't causally follow from exploration.
- *Gap*: SEAL pipeline has no causal consistency verification between stages.
- *Fix*: Add `CausalChainVerifier` in NT-MIND that checks each SEAL stage output has traceable causal links to its input. Flag disconnected stages for human review, analogous to narrative plot-hole detection.

**DEFECT-NAR-3: No Possibility-Space Visualization for Branching Decisions**
Elsewise (2026) and Narrative Studio show that interactive narrative benefits enormously from possibility-space visualization—authors can inspect, compare, and prune branches. NeoTrix's SEAL exploration generates branches but provides no visualization or inspection tooling.
- *Gap*: No `PossibilitySpaceVisualizer` for SEAL pipeline branches.
- *Fix*: Implement branch visualization in NT-IO: render SEAL exploration branches as an inspectable graph, with branch quality scores, dependency tracking, and pruning suggestions. This enables human-in-the-loop decision making analogous to InkOS's 9-phase pipeline with human confirmation gates.

**DEFECT-NAR-4: No Persistent Cross-Session Memory for Narrative Consistency**
PsyMem and SCORE both demonstrate that character/entity memory must be explicit, structured, and selectively retrieved—not dumped wholesale into every prompt. NeoTrix's NT-NEXUS provides cross-session memory but lacks selective retrieval based on narrative relevance.
- *Gap*: NT-NEXUS retrieval is keyword/embedding-based, not narrative-relevance-based.
- *Fix*: Add `NarrativeRelevanceRanker` in NT-NEXUS that scores retrieved memories by narrative relevance (causal importance, recency, emotional weight) rather than just semantic similarity. This prevents the "forgetting hero's eye color by chapter three" problem identified in 2026 story generator evaluations.

**DEFECT-NAR-5: No Human-in-the-Loop Confirmation Gates**
InkOS's 9-phase pipeline with human confirmation at each phase, and Elsewise's authoring support, demonstrate that production-quality narrative generation requires editorial gates. NeoTrix's SEAL pipeline runs autonomously without human confirmation checkpoints.
- *Gap*: SEAL pipeline has no configurable human-in-the-loop confirmation gates.
- *Fix*: Add `ConfirmationGate` trait in NT-MIND: configurable checkpoints where SEAL pipeline pauses for human review before proceeding to next phase. Gate triggers based on: risk level, novelty score, or phase type (e.g., always gate before absorption phase).

**DEFECT-NAR-6: No Genre/Domain-Aware Structural Adaptation**
DeepFiction maintains consistency across branching paths; story generators show genre-specific beat patterns (romance ≠ mystery ≠ thriller). NeoTrix's SEAL pipeline uses uniform structure regardless of domain context (coding task vs. research synthesis vs. architecture design).
- *Gap*: SEAL pipeline has no domain-aware structural templates.
- *Fix*: Add `DomainStructuralTemplate` registry in NT-MIND: domain-specific SEAL pipeline configurations that adjust stage ordering, branching depth, verification strictness, and human gate frequency based on task domain (software engineering, research, creative writing, etc.).

---

## 4. Cross-Domain Synthesis: Meta-Patterns

### Pattern: Stateful Memory as Prerequisite for Coherence
All three domains converge on the same insight: **coherence requires explicit, structured, stateful memory**. Discourse structure tracks entity focus; controlled generation tracks attribute constraints; narrative generation tracks plot state. Stateless systems fail at all three.
- **NeoTrix Deficit**: While NT-MEMORY provides KB storage, and NT-NEXUS provides cross-session memory, there is no unified **CoherenceState** that tracks discourse-level, attribute-level, and narrative-level state simultaneously. Each domain manages state independently.
- **Fix**: Define `CoherenceState` trait as a unified state container in NT-CORE that aggregates discourse state (entity focus, rhetorical relations), constraint state (active control attributes, conflict scores), and narrative state (active entities, causal chains, workflow conditions). All generation modules read from and write to this shared state.

### Pattern: Human-in-the-Loop as Production Requirement
InkOS, Elsewise, and the WGA 2026 guidance all converge: production-quality generation requires human editorial gates. Autonomous generation produces artifacts; human-gated generation produces products.
- **NeoTrix Deficit**: SEAL pipeline is fully autonomous. NT-ACT has `ResourceBudgetManager` but no editorial review budget or confirmation gate mechanism.
- **Fix**: Add `EditorialReviewBudget` to ResourceBudgetManager: allocate tokens/time for human review phases, with automatic escalation when review budget is exhausted but quality thresholds aren't met.

### Pattern: Evaluation Standardization as Trust Foundation
The Lorandi & Belz LPF study proves that without standardized evaluation, performance claims are unreliable. All three NLG sub-fields face this crisis.
- **NeoTrix Deficit**: No standardized evaluation framework for any text generation output. No reproducible benchmarks, no cross-system comparison methodology.
- **Fix**: Build `GenerationEvaluationSuite` in NT-MIND: standardized evaluation pipelines for each generation type (text planning, realization, narrative) with shared metrics, reproducible evaluation, and longitudinal tracking in KB.

---

## 5. Summary Table

| # | Domain | Defect ID | Severity | Description |
|---|--------|-----------|----------|-------------|
| 1 | Text Planning | DEFECT-NLG-1 | HIGH | No discourse structure representation (RST/PDTB/QUD) in text generation |
| 2 | Text Planning | DEFECT-NLG-2 | HIGH | No entity-based coherence tracking across generated paragraphs |
| 3 | Text Planning | DEFECT-NLG-3 | MEDIUM | No multi-perspective reasoning in knowledge synthesis |
| 4 | Controlled Gen | DEFECT-CTG-1 | HIGH | No standardized evaluation framework for generation quality |
| 5 | Controlled Gen | DEFECT-CTG-2 | MEDIUM | No multi-attribute conflict resolution in generation |
| 6 | Controlled Gen | DEFECT-CTG-3 | MEDIUM | No causal modeling in generation constraints |
| 7 | Narrative | DEFECT-NAR-1 | HIGH | No narrative state tracking for multi-step workflows |
| 8 | Narrative | DEFECT-NAR-2 | HIGH | No causal chain enforcement across SEAL pipeline stages |
| 9 | Narrative | DEFECT-NAR-3 | MEDIUM | No possibility-space visualization for branching decisions |
| 10 | Narrative | DEFECT-NAR-4 | MEDIUM | No narrative-relevance-based retrieval in cross-session memory |
| 11 | Narrative | DEFECT-NAR-5 | MEDIUM | No human-in-the-loop confirmation gates in SEAL pipeline |
| 12 | Narrative | DEFECT-NAR-6 | LOW | No domain-aware structural adaptation in SEAL pipeline |
| 13 | Cross | META-COHERENCE | HIGH | No unified CoherenceState across discourse/constraint/narrative levels |
| 14 | Cross | META-HUMAN-GATE | MEDIUM | No editorial review budget in resource management |
| 15 | Cross | META-EVAL | MEDIUM | No standardized generation evaluation framework |

**Total defects: 15** (5 HIGH, 7 MEDIUM, 1 LOW, 2 cross-domain meta-patterns)

---

## 6. Sources Cited (Full Bibliography)

1. INLG 2026. "19th International Natural Language Generation Conference." Utrecht, Netherlands, Oct 17-21, 2026. 2026.inlgmeeting.org.
2. INLG 2026 Accepted Papers. 2026.inlgmeeting.org/accepted-papers.html.
3. arXiv:2602.02878. "Which course? Discourse! Teaching Discourse and Generation in the Era of LLMs." 2026.
4. ACL Anthology 2026.acl-long.829. "Beyond Chunking: Discourse-Aware Hierarchical Retrieval for Long Document Question Answering." ACL 2026.
5. AJSLP 2026. "Fit-for-Purpose Discourse." ICCDC 2026. pubs.asha.org.
6. Jurafsky, D. & Martin, J.H. Speech and Language Processing, 3rd ed. January 2026. web.stanford.edu/~jurafsky/slp3/.
7. Lorandi, M. & Belz, A. "A Comparative Study of Controlled Text Generation Systems Using Level-Playing-Field Evaluation Principles." arXiv:2605.12395, May 2026.
8. C3TG. "Conflict-aware, Composite, and Collaborative Controlled Text Generation." AAAI 2026. papernotes.org.
9. Agentica. "Controlled Generation Encyclopedia." March 2026. agentica.wiki.
10. ScienceDirect. "A recent survey on controllable text generation: A causal perspective." 2025.
11. CVPR 2026. "Yume1.5: A Text-Controlled Interactive World Generation Model." openaccess.thecvf.com.
12. Yenra. "AI Interactive Storytelling and Narratives: 20 Advances (2026)." January 2026, updated March 2026.
13. Runable. "Best AI Story Generators in 2026: 10 Tools Tested." August 2026.
14. Jenova. "Story Plot Generator: Build Complete Narratives With AI." April 2026.
15. Jenova. "Plot Generator AI: Build Story Structures That Actually Work." May 2026.
16. InkOS v1.4.1. Open-source autonomous novel writing system. 6,200+ GitHub stars. fast.io/OpenClaw skills review.
17. DeepFiction. Branching narrative engine. 2M+ interactive stories. runable.com review.
18. Wang et al. "Elsewise: Authoring AI-Based Interactive Narrative with Possibility Space Visualization." 2026.
19. Yi et al. "SCORE: Story Coherence and Retrieval Enhancement for AI Narratives." 2025.
20. Cheng et al. "PsyMem: Fine-grained psychological alignment and Explicit Memory Control for Advanced Role-Playing LLMs." 2025.
