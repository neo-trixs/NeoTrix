# Iteration 526 — Memory Consolidation, Working Memory, Long-Term Memory

**Date**: 2026-09-06
**Context**: Batch 525 proved (1) hybrid attention abandons broadcast, (2) component collapse in hybrids, (3) reasoning needs dense attention, (4) saliency flow inverted, (5) attention is continuous flow not ignition.

---

## 1. MEMORY CONSOLIDATION — NEW FINDINGS

### 1.1 Theta Tagging Mechanism (PLOS Biology 2026-08-27)
- Theta oscillations (3–8 Hz) during learning **tag** memories for sleep-dependent consolidation
- Theta power at encoding predicts SO-spindle coupling during subsequent sleep
- SO-spindle coupling predicts memory retention at post-sleep test
- **Timeline**: Tagging emerges ~1s after encoding — aligned with early wake replay events
- **Source**: PLOS Biology, journal.pbio.3003938

### 1.2 Single-Neuron Reactivation Driven by Ripples (Kehl et al. 2026-03-31)
- First direct human evidence: ripples drive single-neuron reactivation in MTL
- Sleep ripples elicit **stronger** activation than wake ripples
- Neurons for remembered items fire more strongly during ripples than forgotten items
- Ripple-associated MTL bursts detectable across **widespread cortical activity**
- **Source**: doi.org/10.64898/2026.03.27.714528

### 1.3 Hippocampus-Independent Procedural Replay (Nature Neuroscience 2026-07-23)
- **REVOLUTIONARY**: Striatal replay occurs **independently** of hippocampus
- Complete bilateral hippocampal lesions do not eliminate procedural replay
- Positive/negative outcomes have **opposing effects** on individual replay events
- Replay at both real-world and time-compressed speeds, forward and reverse
- **Source**: Nature Neuroscience, s41593-026-02362-5

### 1.4 SO-Upstate Window for Consolidation (bioRxiv 2026-04-17)
- Inhibiting hippocampus during SO upstates **completely abolishes** memory expression
- Inhibiting outside SOs preserves memory
- Memory impairment mediated by SO upstates nesting spindles
- **Source**: doi.org/10.64898/2026.04.17.719155

### 1.5 Generative Replay in Neocortex (bioRxiv 2026-08-03)
- V1 encodes relationships **never directly experienced**
- Generative replay in V1 predicted by preceding hippocampal activity
- Hippocampus provides teaching signal to reorganize neocortical representations during sleep
- **Source**: doi.org/10.64898/2026.07.24.740539v2

### 1.6 Concept Neuron Reactivation Reflects Contents Not Sequence (bioRxiv 2026-01-10)
- Human concept neurons reactivated during SWS — conjointly, particularly during SWRs
- Time lags suitable for synaptic modification
- **Critical**: Reactivation does NOT reflect sequence of events (unlike rodent place cells)
- Concept neurons are pre-tuned to semantic contents before learning
- **Source**: doi.org/10.64898/2026.01.10.698827

### 1.7 Sleep Strengthens Successor Representations (PLOS Biology 2026-04-07)
- Sequence learning induces successor representations in perceptual task
- Representational shift from low-level visual → higher-level abstract features
- SO-spindle coupling predicts both strength and abstraction shift of successor representations
- **Source**: PLOS Biology, journal.pbio.3003740

### NEW DEFECTS vs Batch 525:

**D-526-1: Replay Architecture Siloed**
Batch 525 treated attention as unified. Neuroscience shows replay operates through **parallel independent mechanisms** — hippocampal (declarative) and striatal (procedural). NT-CORE's attention routing assumes single broadcast channel. Must split into domain-specific replay channels with independent consolidation pathways.

**D-526-2: Missing Theta-Tag for Selective Consolidation**
No mechanism exists in NT-MIND to tag which experiences are worth consolidating during sleep cycles. Theta-tagging (3-8 Hz at encoding) is the biological mechanism for selective consolidation. NT-MIND's SEAL pipeline processes all experiences uniformly — no priority signal at encoding time.

**D-526-3: SO-Upstate Window Not Modeled**
Consolidation requires precise timing within cortical SO upstates. NT-CORE's heartbeat aggregator operates at fixed tick intervals (60s), completely missing the sub-second temporal windows where consolidation actually occurs. The "when" of consolidation matters as much as the "what."

**D-526-4: Concept vs Sequence Encoding Mismatch**
Human concept neurons reflect contents, not temporal sequence. NT-MEMORY's episodic storage encodes sequence-first (timestamp-centric), but consolidation preserves content relationships, not temporal order. The memory system's write path optimizes for the wrong retrieval pattern.

**D-526-5: Generative Replay Gap**
Neocortex generates novel combinations never experienced. NT-MIND's evolution loop only recombines existing patterns — no mechanism for generating truly novel capability combinations during offline consolidation. The generative capacity is architecturally absent.

---

## 2. WORKING MEMORY — NEW FINDINGS

### 2.1 WMC = Attention Control, Not Storage (Lee & Engle 2026)
- WMC predictive power reflects **attention control** (AC), not passive storage
- AC comprises: goal maintenance, interference resolution, disengagement
- When AC variance is controlled, WMC links to higher-order criteria **diminish substantially**
- Latent-variable modeling shows AC fully mediates WMC→gF relationship
- **Source**: Journal of Intelligence 2026, 14, 22

### 2.2 LLMs: Strong Memory, Weak Control (de Langis et al. 2026, EACL)
- LLMs exceed human normative scores on working memory tasks
- BUT increased WM capacity does NOT correlate with executive functioning or problem-solving
- LLMs have deficits in **attentional control and cognitive flexibility**
- Difficulty inhibiting automatic responses and adapting to shifting information
- **Source**: ACL Anthology, 2026.eacl-long.281

### 2.3 Representational Interference as Core LLM Constraint (arXiv 2604.09670)
- LLMs reproduce human interference signatures on N-back
- Performance degrades with load, biased by recency and stimulus statistics
- Multiple memory items encoded in **entangled representations**
- Successful recall depends on **interference control** — suppressing task-irrelevant content
- Progressive suppression of task-irrelevant content across layers
- **Source**: arxiv.org/pdf/2604.09670

### 2.4 Cognitive Load Framework for AI Agents (Wang et al. 2026)
- Shared mechanisms: bounded workspaces and chunking in humans AND AI
- Divergences: human metacognition absent in AI
- "Bounded agent complementarity" model for dynamic load-balancing
- **Source**: doi.org/10.1007/s10462-026-11510-z

### 2.5 WM Constraints as Beneficial Inductive Bias (ACL Findings 2026)
- Fixed-width attention windows improve grammatical accuracy in data-scarce settings
- WM constraints serve as beneficial inductive bias, guiding robust representations
- Constrained models align more strongly with human processing metrics
- Pushes against trends toward longer contexts and weaker inductive biases
- **Source**: ACL Findings 2026, findings-acl.2133

### 2.6 Tool-Use Agent Cognitive Load (arXiv 2601.20412)
- First formal CLT application to AI agent capability boundaries
- Intrinsic Load = solution path complexity; Extraneous Load = ambiguous presentation
- Performance cliffs as cognitive load increases → precise capability boundaries
- **Source**: arxiv.org/html/2601.20412v1

### NEW DEFECTS vs Batch 525:

**D-526-6: WMC-AC Conflation in NT-CORE**
NT-CORE conflates attention control (AC) with working memory capacity (WMC). The GWT broadcast assumes capacity is the bottleneck, but the 2026 evidence shows **interference control** is the operative bottleneck. NT-CORE's SelectiveState measures capacity (how many items) when it should measure control (suppression of irrelevant items). This is the root cause of D-525-4 (inverted saliency flow).

**D-526-7: LLM Memory-Control Asymmetry Unmodeled**
Batch 525 found hybrid attention collapses components. The 2026 EACL paper shows WHY: LLMs have strong memory but weak control. NT-IO's LLM providers exhibit this asymmetry — they can store context but cannot suppress interference from irrelevant context. No proxy mechanism exists in NT-CORE to compensate for this provider-level deficit.

**D-526-8: Interference Control Missing from Architecture**
Representational interference is the core constraint on WM in both biological and artificial systems. NT-CORE has no explicit interference suppression mechanism. The GWT broadcast amplifies salient signals but does not suppress irrelevant ones. Need bidirectional modulation: amplification AND inhibition operating in parallel.

**D-526-9: WM Constraints as Feature Not Bug**
Batch 525 treated WM limitations as problems to overcome. The ACL Findings 2026 paper shows WM constraints **improve** learning efficiency and human alignment. NT-CORE should intentionally impose WM constraints (fixed-width windows, temporal decay) rather than treating context windows as resources to maximize. Less context = better reasoning when data is scarce.

**D-526-10: No Metacognitive Load Monitor**
Human WM operates with metacognition — awareness of own capacity limits. AI agents lack this. NT-META tracks module health but not cognitive load state of the reasoning process itself. Need a load monitor that detects when reasoning is approaching interference threshold and triggers proactive suppression.

---

## 3. LONG-TERM MEMORY — NEW FINDINGS

### 3.1 EverMemOS: Engram-Inspired Memory Lifecycle (ACL 2026)
- Three-phase: Episodic Trace Formation → Semantic Consolidation → Reconstructive Recollection
- MemCells bridge low-level data and high-level semantics
- MemScene consolidation with conflict tracking and user profile evolution
- Reconstructive recollection guided by necessity and sufficiency
- +9.2% on LoCoMo, +6.7% on LongMemEval vs SOTA
- **Source**: ACL 2026, acl-long.2125

### 3.2 SEEM: Structured Episodic Event Memory (ACL 2026)
- Dual-layer: Episodic Memory Layer (narrative) + Graph Memory Layer (facts)
- Episodic Event Frames with 6 core slots (Participants, Action, Time, Location, Causality, Manner)
- Reverse Provenance Expansion (RPE) — uses fragments as entry points to activate fused narrative
- Provenance pointers maintain link to raw source passages
- **Source**: ACL 2026, acl-long.277

### 3.3 ENGRAM: Lightweight Memory Orchestration (ICLR 2026)
- Three canonical memory types: episodic, semantic, procedural
- Single router and retriever — challenges trend toward architectural complexity
- Exceeds full-context baseline by **15 absolute points** on LongMemEval using **~1% tokens**
- **Source**: ICLR 2026 Workshop on Memory for LLM-Based Agentic Systems

### 3.4 SYNAPSE: Spreading Activation Memory (ACL Findings 2026)
- Memory as dynamic graph with relevance emerging from **spreading activation**
- Lateral inhibition + temporal decay filter interference
- Triple Hybrid Retrieval: geometric embeddings + activation-based graph traversal
- +23% on complex multi-hop reasoning, -95% token consumption vs full-context
- **Source**: ACL Findings 2026, findings-acl.1108

### 3.5 RippleMem: Adaptive Associative Recollection (arXiv 2026)
- Replaces one-shot retrieval with adaptive associative recollection
- Memories serve as **cues** for completing missing evidence
- Event-centric memory graph with semantic + structural associations
- +3.95% on LoCoMo, +11.87% on LongMemEval-S, -30x graph construction cost
- **Source**: arxiv.org/html/2608.13334

### 3.6 REMem: Time-Aware Episodic Memory (arXiv 2026)
- Hybrid memory graph: time-aware gists + facts
- Agentic inference with iterative retrieval over memory graph
- Mental time travel — filters memory entries by temporal conditions
- +3.4% on episodic recollection, +13.4% on episodic reasoning vs Mem0/HippoRAG2
- **Source**: arxiv.org/pdf/2602.13530

### 3.7 ECHO: Auditable Memory Plane (arXiv 2026)
- Bitemporal ledger with provenance closure
- Separates discovery from authority — similarity proposes, ledger decides
- Append-only revision (reconsolidation) — never erase prior trace
- Executive boundary separates internal derivation from visible answer
- **Source**: arxiv.org/html/2608.21755

### NEW DEFECTS vs Batch 525:

**D-526-11: No Memory Lifecycle in NT-MEMORY**
Batch 525 treated memory as static storage. All 2026 systems implement explicit **lifecycles**: formation → consolidation → retrieval → reconsolidation. NT-MEMORY has write/read but no consolidation phase that transforms episodic traces into semantic structures. This is why memory retrieval is fragmented.

**D-526-12: Missing Provenance Chain**
SEEM and ECHO maintain provenance pointers from abstract memory units to raw source. NT-MEMORY stores embeddings and text but lacks bidirectional traceability. When a retrieved memory is stale or conflicted, there is no mechanism to trace back to the authoritative source or detect the conflict.

**D-526-13: Retrieval Without Evidence Completion**
RippleMem's key insight: initial recall serves as **cue** for further evidence recovery. NT-MEMORY does single-shot vector retrieval. For multi-hop reasoning, the first retrieved chunk is rarely sufficient — need associative expansion from initial anchors to recover distributed evidence.

**D-526-14: No Interference Suppression in Memory Retrieval**
SYNAPSE implements lateral inhibition during spreading activation — suppresses irrelevant distractors while amplifying relevant sub-graphs. NT-MEMORY's retrieval has no inhibition mechanism. All retrieved chunks compete equally for attention, creating interference in the reasoning context.

**D-526-15: Temporal Authority Not Separated from Semantic Similarity**
ECHO's critical architectural choice: **similarity proposes, but bitemporal ledger decides currentness**. NT-MEMORY conflates relevance (embedding distance) with recency (timestamp). A semantically close but outdated memory can outrank a current one. Need explicit temporal authority layer.

**D-526-16: No Reconsolidation Mechanism**
ECHO's append-only revision (reconsolidation) preserves prior traces while appending updates. NT-MEMORY overwrites or appends without distinguishing revision from original. This prevents detection of belief evolution and makes conflict resolution impossible.

**D-526-17: ENGRAM's Complexity Challenge**
ENGRAM achieves SOTA with simple dense retrieval + memory typing. This challenges NT-MEMORY's trajectory toward increasingly complex retrieval pipelines. May be over-engineering retrieval when memory organization is the actual bottleneck.

---

## 4. CROSS-DOMAIN SYNTHESIS — CONSOLIDATED DEFECTS

### Architecture-Level Defects:

| ID | Defect | Severity | Domain |
|----|--------|----------|--------|
| D-526-1 | Replay architecture siloed — no parallel independent channels | HIGH | NT-CORE + NT-MIND |
| D-526-2 | Missing theta-tag for selective consolidation | HIGH | NT-MIND |
| D-526-3 | SO-upstate window not modeled — wrong temporal resolution | MEDIUM | NT-CORE |
| D-526-4 | Concept vs sequence encoding mismatch | HIGH | NT-MEMORY |
| D-526-5 | Generative replay gap — no novel combination generation | HIGH | NT-MIND |
| D-526-6 | WMC-AC conflation in GWT broadcast | CRITICAL | NT-CORE |
| D-526-7 | LLM memory-control asymmetry unmodeled | HIGH | NT-IO |
| D-526-8 | Interference control missing from architecture | CRITICAL | NT-CORE |
| D-526-9 | WM constraints as feature not bug | MEDIUM | NT-CORE |
| D-526-10 | No metacognitive load monitor | HIGH | NT-META |
| D-526-11 | No memory lifecycle — missing consolidation phase | CRITICAL | NT-MEMORY |
| D-526-12 | Missing provenance chain | HIGH | NT-MEMORY |
| D-526-13 | Retrieval without evidence completion | HIGH | NT-MEMORY |
| D-526-14 | No interference suppression in retrieval | HIGH | NT-MEMORY |
| D-526-15 | Temporal authority conflated with semantic similarity | HIGH | NT-MEMORY |
| D-526-16 | No reconsolidation mechanism | MEDIUM | NT-MEMORY |
| D-526-17 | ENGRAM's complexity challenge — over-engineering | MEDIUM | NT-MEMORY |

### CRITICAL Defects (3):
1. **D-526-6**: GWT measures capacity when it should measure interference control
2. **D-526-8**: No bidirectional modulation (amplification AND inhibition)
3. **D-526-11**: NT-MEMORY lacks consolidation lifecycle

### What's NEW vs Batch 525:
1. **Selective consolidation** — not all experiences deserve equal processing
2. **Parallel replay channels** — declarative and procedural consolidate independently
3. **Interference control as bottleneck** — NOT capacity
4. **Memory as lifecycle** — not storage
5. **Provenance over similarity** — temporal authority must be separate
6. **WM constraints improve learning** — less can be more
7. **Generative replay** — novel combinations during offline processing
8. **Evidence completion** — recall as cue, not answer

---

## 5. SOURCES

1. Kehl et al. (2026). Sleep ripples drive single-neuron reactivation. doi:10.64898/2026.03.27.714528
2. Takigawa et al. (2026). Interplay of sleep neural oscillations. doi:10.64898/2026.06.12.731367
3. PLOS Biology (2026). Theta oscillations tag episodic memories. journal.pbio.3003938
4. Nature Neuroscience (2026). Replay of procedural memory independent of hippocampus. s41593-026-02362-5
5. bioRxiv (2026). Hippocampus consolidates memory in SO upstates. doi:10.64898/2026.04.17.719155
6. bioRxiv (2026). Generative replay across hippocampal-neocortical circuits. doi:10.64898/2026.07.24.740539v2
7. bioRxiv (2026). Episodic memory consolidation by concept neurons. doi:10.64898/2026.01.10.698827
8. PLOS Biology (2026). Sleep strengthens successor representations. journal.pbio.3003740
9. Lee & Engle (2026). Beyond WMC: Attention Control. J. Intelligence 14(22)
10. de Langis et al. (2026). Strong Memory, Weak Control. EACL 2026, acl-long.281
11. arXiv (2026). Representational interference as core LLM constraint. arxiv.org/pdf/2604.09670
12. Wang et al. (2026). Overloaded minds and machines. doi:10.1007/s10462-026-11510-z
13. ACL Findings (2026). WM Constraints Scaffold Learning. findings-acl.2133
14. arXiv (2026). Cognitive Load Framework for Tool-use Agents. arxiv.org/html/2601.20412v1
15. EverMemOS (2026). ACL 2026, acl-long.2125
16. SEEM (2026). ACL 2026, acl-long.277
17. ENGRAM (2026). ICLR 2026 Workshop
18. SYNAPSE (2026). ACL Findings 2026, findings-acl.1108
19. RippleMem (2026). arxiv.org/html/2608.13334
20. REMem (2026). arxiv.org/pdf/2602.13530
21. ECHO (2026). arxiv.org/html/2608.21755
