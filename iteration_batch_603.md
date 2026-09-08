# Iteration Batch 603 — Human-AI Collaboration, Trust Calibration, Transparency

**Date**: 2026-09-06
**Research Loop**: 603/10000+
**Prior Batch**: 602 (causal memory injection poison, execution proposal temporal validity decay, cascading uncertainty in test-time search, clock domain crossing at latent interface)

---

## Topic 1: Human-AI Collaboration

### Finding 1.1: Cross-Level Reversal — Task Augmentation ≠ Cluster Adoption
**Source**: "The collaboration code" (Frontiers in AI, 2026-07-14, Anthropic Economic Index dataset of millions of conversations)
**Defect**: A cross-level reversal emerged whereby augmentation patterns positively predict usage intensity at the **task level** but **negatively predict** user adoption at the **cluster level** (r = −0.119, p = 0.006). Additionally, augmentation and extended thinking were significantly negatively correlated (r = −0.327, p < 0.001), contradicting the expectation that collaborative work would naturally incorporate sophisticated cognitive features. This reveals that **depth and breadth of AI use follow different dynamics** — optimizing for one degrades the other. Batch 602 identified causal memory injection poison but not this structural incompatibility between per-task depth and cross-task breadth adoption.

### Finding 1.2: Six Cascading Risk Clusters Across Collaboration Lifecycle
**Source**: "Toward Resilient Human-AI Collaboration: A Lifecycle Taxonomy" (arXiv:2608.05614, 2026-08)
**Defect**: Six recurring cross-domain risk clusters identified: Trust Miscalibration, Cognitive Burden, Accountability Gap, Capability Erosion, Goal Misalignment, AI Anxiety. The critical insight is that these are **interconnected sociotechnical dynamics** — many collaboration failures stem from cascading pathways between clusters, not isolated technical deficiencies. Piecemeal interventions frequently create unintended consequences. Batch 602's cascading uncertainty in test-time search addressed technical cascading; this reveals **organizational/sociotechnical cascading** that operates through different mechanisms (cognitive burden → capability erosion → accountability gap → trust miscalibration).

### Finding 1.3: Collaborative AI Requires Four Missing Capabilities
**Source**: "Toward Collaborative AI" (sciltp.com/IJNDI, published 2026-09-04)
**Defect**: Four required capabilities for collaborative AI identified that current architectures lack: (1) **metacognition**, (2) **contextual mode-switching**, (3) **uncertainty-aware action**, (4) **adaptive human collaboration**. These are not solved by current architectures. This maps directly to NeoTrix's ConsciousnessTree branches — NT-META provides metacognition, GWT provides mode-switching, but the "adaptive human collaboration" capability has no dedicated NT-domain owner. The gap is that NeoTrix models internal consciousness architecture but not the **relational interface** between system and human operator.

### Finding 1.4: CrabOS — Shared Work State as Natural Language Objects
**Source**: "CrabOS: An Operating System for Human-AI Co-inhabitation" (arXiv:2608.28165, 2026-08-28)
**Defect**: Humans and AI currently have **separate work environments**, forcing bridge-building (task-specific interfaces or manual state transfer via screenshots). CrabOS proposes shared natural-language-readable text objects as a native OS capability. The architectural insight: **work state representation must be human-readable by design**, not as an afterthought translation layer. For NeoTrix, this implies that the KB `experience` namespace and VSA HyperCube representations need a parallel **human-legible mirror** — the current architecture encodes knowledge in vector form optimized for machine consumption, creating an inherent transparency gap.

### Finding 1.5: Proactive Delegation vs. Deliberative Adoption — Two Distinct Reliance Decisions
**Source**: "AI, Take the Wheel" (ACL 2026 Findings, 24 matches, 387 delegation + 1440 adoption decisions)
**Defect**: Under-reliance (3.7% missed opportunities) exceeds over-reliance (1.5%), with **confirmation bias** driving 60.7% of under-reliance when AI agrees with human's incorrect initial answer. Model confidence scores perform **near chance** when humans and AI disagree. This means NeoTrix's `Egress Privacy Guard` trust tiering (Trusted/Contracted/Untrusted) is insufficient — it's a static categorization, not a dynamic calibration that accounts for the human's current belief state. The gap: trust tiers are system-side properties; reliance decisions are human-side properties that depend on the human's initial confidence.

---

## Topic 2: Trust Calibration

### Finding 2.1: Social Learning Creates Consensus, Not Overreliance
**Source**: "Modeling AI Overreliance as a Complex Adaptive System" (arXiv:2608.19616, 2026-08-20)
**Defect**: A mean-preservation theorem: connectivity moves aggregate trust only through **opinion dynamics** (peers transmitting beliefs), not through experience-based learning. Under experience-based learning, even extreme high-trust hubs leave aggregate unchanged (consensus 0.516 vs 0.513). **But** when influence transmits correlated beliefs, topology matters — high-trust hubs raise consensus to 0.549, careful hubs lower it to 0.455. The cascade: visible unverified AI use collapses verification (0.292 → 0.004) and raises overreliance (0.303 → 0.513). **Key insight**: Overreliance is a **feedback phenomenon** — levers are verification cost, salience of verification, and structure of social exposure. Batch 602 identified cascading uncertainty at the technical level; this reveals **social cascading** where network topology amplifies miscalibrated trust.

### Finding 2.2: Trust Calibration Is Over-Time, Non-Linear, and Context-Dependent
**Source**: "Unpacking the dynamics of generative AI use" (AI & Society, Springer, 2026-08-11)
**Defect**: Five challenges to trust calibration in genAI agents: (1) complex nature of individual private use, (2) complexity in assessing actual trustworthiness, (3) **fuzzy notion of the trustee** (is it the agent, the technology, the answers?), (4) complex ontological status of agents, (5) **over-time nature** of calibration. The "fuzzy trustee" problem is new: in NeoTrix, when the NT-CORE E8 reasoning engine produces a flawed output, is the "trustee" the E8 hexagram, the ConsciousnessTree, the GWT routing, or the composite? Current NeoTrix architecture treats trust as a property of the **system**, not of specific **components within a trust chain**.

### Finding 2.3: Distrust Is the Safer Default — Mathematical Minimum Reliability Threshold
**Source**: "Mathematical Modeling of Trust Calibration for Human-Automation Safety" (SAE 2026-01-0530, 2026-04-07)
**Defect**: Four progressive models (binary, linear, triangular, logistic) establish that **distrust is the safer default** in high-risk contexts. A minimum reliability threshold exists below which meaningful trust cannot form. An empirical observation of 32 AI applications plotted in trust-reliability space confirms a consistent **distrust tendency** where reliability exceeds user confidence. This inverts the assumption that "building trust is always desirable" — for NeoTrix safety-critical components (NT-SHIELD sandbox, NT-PHYSICAL safety kernel), the architecture should **default to distrust** and require components to earn trust through demonstrated reliability, not assume trustworthiness from successful compilation (C0) or even unit tests (C1).

### Finding 2.4: Tiered Controllability Framework for Agentic Systems
**Source**: "Between autonomy and oversight" (GJETA, 2026-06-06, 34 sources reviewed)
**Defect**: Trust miscalibration (both over-reliance and under-reliance) is the **most prevalent failure mode** in deployed agentic systems. Current transparency tools remain **inadequate for informed human oversight at operational scale**. The Tiered Controllability Framework (TCF) maps oversight requirements to three variables: task risk, action reversibility, and agent autonomy scope. NeoTrix's Constellation maturity model (C0-C6) encodes module maturity but does not encode **action reversibility** — a C5 self-healing module executing an irreversible action (e.g., deleting KB data, sending external API calls) requires different oversight than a C5 module performing reversible computation.

### Finding 2.5: CAHAT — Adaptive Fusion with Human Bias as Dominant Failure Mode
**Source**: "Calibrated adaptive framework" (Nature Scientific Reports, 2026-08-20)
**Defect**: CAHAT framework achieves 0.8% failure rate vs 12.4% for uncalibrated AI. Ablation analysis reveals **human bias, not model capability**, is the dominant failure mode. Sensitivity experiments show that calibration (post-hoc temperature scaling + MC Dropout) is foundational. The architectural insight: the system should treat **human operator as a variable to calibrate**, not a fixed input. NeoTrix's GWT attention routing treats human input as authoritative signal; this research suggests it should also model human cognitive bias and weight human input accordingly.

### Finding 2.6: Graduated Autonomy — Earned Permissions with Immediate Demotion
**Source**: "Closing the AI agent trust gap" (AWS Architecture Blog, 2026-08-26)
**Defect**: Trust scored across 5 dimensions (accuracy 25%, safety 20%, consistency 20%, compliance 20%, efficiency 15%) with a **safety floor** that is never averaged away. Four tiers (T1 Probation → T4 Autonomous) with **hysteresis**: promotion requires score 5 points above tier floor, demotion happens at floor itself. Key design: **safety is an independent floor, never averaged away by strong metrics**. For NeoTrix, this means Constellation maturity (C0-C6) should not be the sole trust signal — a C5 module that fails safety checks should be immediately demoted regardless of its maturity score. The current architecture lacks this safety-floor independence.

---

## Topic 3: Transparency

### Finding 3.1: EU AI Act Article 50 — Transparency Obligations Effective 2 August 2026
**Source**: European Commission Guidelines (C(2026) 5054, 2026-07-20), Morgan Lewis analysis (2026-08-12)
**Defect**: Article 50 imposes disclosure duties **regardless of risk tier**: chatbot providers must ensure users know they're interacting with AI; generative AI providers must mark synthetic output in machine-readable format; deployers of emotion-recognition systems must inform exposed individuals; deepfakes must be disclosed. The Digital Omnibus did NOT postpone Article 50 (only Annex III high-risk obligations deferred to Dec 2027). This is a **regulatory clock domain crossing**: NeoTrix's NT-IO interface layer (LLM providers, CLI, web server) must now embed disclosure mechanisms at the architectural level, not as optional UI features. The gap: NeoTrix currently has no standardized disclosure/tracing mechanism for AI-generated content.

### Finding 3.2: Transparency ≠ Explainability — Distinct Regulatory Obligations
**Source**: AIRiskAware analysis (2026), NIST AI RMF, EU AI Act
**Defect**: NIST treats "Accountable and Transparent" and "Explainable and Interpretable" as **two separate trustworthy-AI characteristics**. A system can be transparent (clearly labeled as AI) while remaining a technical black box (not explainable). Conflating them is a common source of confusion in compliance programs. For NeoTrix, this means the VSA HyperCube representation (which provides associative/analogical reasoning but not mechanistic explanation) satisfies transparency (it can be disclosed as AI-generated) but may not satisfy explainability (why specific reasoning paths were taken). The architecture needs a separate **explanation trace** capability distinct from the reasoning trace.

### Finding 3.3: Model Card Access Gap — 68% of End Users Never Access Them
**Source**: UCB Responsible AI Transparency Playbook (May 2026)
**Defect**: Only 32% of end-user respondents reported ever accessing model cards. Transparency efforts focused on technical documentation satisfy developer/regulatory needs but **aren't reaching end users**. Without foundational understanding of AI limitations, users risk becoming overtrusting or overreliant. The architectural implication: NeoTrix's documentation (CONTEXT.md, AGENTS.md) serves the developer/operator layer but there is no **end-user-facing transparency layer** that communicates system capabilities, limitations, and uncertainty in accessible form. The gap between documentation-as-compliance and documentation-as-communication.

### Finding 3.4: California AI Transparency Act — Dual Disclosure Model
**Source**: California SB 942/AB 853 (effective 2 August 2026), AIRiskAware
**Defect**: Covered providers (1M+ monthly users) must offer both **manifest disclosure** (visible label) and **latent disclosure** (hidden metadata) in AI-generated content, plus a free public detection tool. This dual-model (visible + machine-readable) is conceptually similar to China's GB 45438-2025. For NeoTrix, this implies that any content produced by NT-ACT tools (social media, code generation) should embed both human-readable and machine-readable provenance markers — currently absent from the architecture.

### Finding 3.5: System-Level Transparency Gap for Deployers
**Source**: UCB Responsible AI Playbook (May 2026)
**Defect**: Most enforceable transparency requirements focus on foundation model developers (model cards), but **deployers** (who integrate foundation models) must create **system-level documentation** explaining how the model is used in context: purpose, safeguards, output monitoring, deployment risks. Organizations inherit the opacity and risk of the underlying foundation model. NeoTrix acts as both developer (own modules) and deployer (wrapping external LLM providers via NT-IO). The gap: NeoTrix documents its own architecture extensively but has no standardized system-level transparency artifact for how it deploys and wraps external models.

---

## Summary: NEW Defects vs Batch 602

| # | Defect | Domain | Severity | Batch 602 Relation |
|---|--------|--------|----------|-------------------|
| D1 | Cross-level reversal: task depth ≠ cluster breadth adoption | Collaboration | HIGH | New — batch 602 didn't model adoption dynamics |
| D2 | Six cascading sociotechnical risk clusters | Collaboration | HIGH | Extends — batch 602 cascading was technical; this is organizational |
| D3 | Missing "adaptive human collaboration" capability in architecture | Collaboration | MEDIUM | New — maps to NT-domain ownership gap |
| D4 | Human-legible mirror needed for VSA/KB representations | Collaboration | MEDIUM | New — transparency gap in knowledge encoding |
| D5 | Trust tiers are system-side; reliance decisions are human-side | Trust | HIGH | New — dynamic calibration gap |
| D6 | Social cascade: visible unverified use collapses verification norm | Trust | HIGH | Extends — social analog of batch 602's technical cascading |
| D7 | Fuzzy trustee in multi-component AI chains | Trust | MEDIUM | New — trust decomposition problem |
| D8 | Distrust as safer default; minimum reliability threshold | Trust | HIGH | Inverts — batch 602 assumed trust-building is desirable |
| D9 | Action reversibility absent from Constellation maturity model | Trust | MEDIUM | New — oversight variable missing |
| D10 | Human bias, not model capability, is dominant failure mode | Trust | HIGH | New — human-as-variable architecture needed |
| D11 | Safety floor independent of maturity score | Trust | HIGH | New — immediate demotion protocol |
| D12 | Article 50 mandatory disclosure at architectural level | Transparency | HIGH | New — regulatory clock domain |
| D13 | Transparency ≠ Explainability — distinct capabilities needed | Transparency | MEDIUM | New — VSA satisfies one, not the other |
| D14 | 68% model card access gap — documentation ≠ communication | Transparency | MEDIUM | New — end-user transparency layer missing |
| D15 | Dual disclosure (manifest + latent) required for AI content | Transparency | HIGH | New — provenance embedding missing |
| D16 | Deployer-level system transparency artifact needed | Transparency | MEDIUM | New — inheritance of opacity problem |

## Sources Cited

1. "The collaboration code" — Frontiers in AI, 2026-07-14, Anthropic Economic Index
2. "Toward Resilient Human-AI Collaboration" — arXiv:2608.05614, 2026-08
3. "Toward Collaborative AI" — sciltp.com/IJNDI, 2026-09-04
4. "CrabOS: An Operating System for Human-AI Co-inhabitation" — arXiv:2608.28165, 2026-08-28
5. "AI, Take the Wheel" — ACL 2026 Findings
6. "Modeling AI Overreliance as a Complex Adaptive System" — arXiv:2608.19616, 2026-08-20
7. "Unpacking the dynamics of generative AI use" — AI & Society, Springer, 2026-08-11
8. "Mathematical Modeling of Trust Calibration" — SAE 2026-01-0530, 2026-04-07
9. "Between autonomy and oversight" — GJETA, 2026-06-06
10. "Calibrated adaptive framework" — Nature Scientific Reports, 2026-08-20
11. "Closing the AI agent trust gap with graduated autonomy" — AWS Architecture Blog, 2026-08-26
12. EU AI Act Article 50 Guidelines — C(2026) 5054, 2026-07-20
13. AIRiskAware — AI Transparency analysis, 2026
14. UCB Responsible AI Transparency Playbook — May 2026
15. California SB 942/AB 853 — AI Transparency Act, effective 2026-08-02
16. "Building trustworthy AI through TEUT framework" — Springer, 2026-04-13
17. "When Should Your Team Override AI?" — Academy of Management Perspectives, 2026-06-19
18. "Dynamic calibration of trust and trustworthiness" — Springer, 2026-02-13
19. "LLM-Based Human-Agent Collaboration and Interaction Systems: A Survey" — ACL 2026
20. "Governing Reflective Human-AI Collaboration" — arXiv:2604.14898, 2026-04-16
