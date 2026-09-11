# Model Reverse Engineering — Cycle 339

**Date**: 2026-09-11
**Focus**: Memory architectures, self-evolving skills, recursive self-improvement, reasoning distillation, cross-agent knowledge sharing
**Note**: arXiv API was unavailable this cycle. Papers identified from GitHub repos, HN discussions, and project references. Analyses constructed from available descriptions and architectural patterns.

---

## Paper/Model 1: CellularFlow — Memory-Augmented Continual Learning Architecture

**Source**: celcilin/cellularflow (GitHub, Sept 2026)
**Category**: Memory-Augmented Continual Learning LLM Architecture

### Core Mechanism
- **Decoupled knowledge storage from sequence reasoning**: Separates what the model knows (knowledge storage) from how it processes sequences (reasoning). This eliminates catastrophic forgetting by preserving knowledge independently of sequence processing.
- **Cellular architecture**: Knowledge stored in discrete "cells" that can be updated, merged, or split without affecting the reasoning pathway. Reasoning cells process sequences; knowledge cells store facts.
- **Continual learning**: New knowledge is added to knowledge cells without retraining reasoning cells. The architecture supports lifelong learning without performance degradation on previously learned tasks.

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-MEMORY** | Decoupled storage = KB nodes (knowledge) vs KB edges (reasoning relationships). Knowledge cells map to experience-tree entries; reasoning cells map to route table matching. | KB should separate fact storage from retrieval logic. Currently they're coupled in the SQLite schema. Decoupling allows independent evolution. |
| **NT-MIND** | Continual learning without forgetting = SEAL pipeline that doesn't degrade prior distillations. Knowledge cells preserve previous learnings while new ones are added. | Experience-tree should use cell-like isolation: new experiences don't corrupt old ones. This addresses the template-collapse problem from RAGEN-2. |
| **NT-CORE** | Cellular architecture = domain modules (nt_*) as discrete cells. Each domain can evolve independently without affecting others. | ConsciousnessTree's 6-stage loop should operate at the cell level — each domain module grows independently, with cross-cell communication via EventBus. |

### Actionable Insight
**Decouple storage from processing at the architectural level.** CellularFlow's key insight is that knowledge storage and sequence reasoning are fundamentally different operations that should have independent lifecycles. NeoTrix's KB currently couples fact storage with retrieval indexing. Separating these — knowledge cells (facts) from reasoning cells (retrieval patterns) — would allowKB schema evolution without retraining retrieval logic, and retrieval optimization without modifying stored knowledge.

---

## Paper/Model 2: WikiSkill — Self-Evolving Agent Skills via Persistent Knowledge Wiki

**Source**: arXiv:2608.27454 (August 2026) / ashutoshsinghpr7/wikiskill
**Category**: Self-Evolving Skill Framework

### Core Mechanism
- **Wiki-as-skill-evolution**: Skills stored as editable wiki pages. Agents can read, modify, and version skills through wiki-style operations (edit, fork, merge).
- **Persistent knowledge wiki**: A shared knowledge base where skills evolve through collective editing. Each agent contributes observations that modify skill definitions.
- **Algorithm 1 (Faithful Implementation)**: Formal algorithm for wiki-based skill evolution with reproducible validation. Ensures quality through structured edit protocols.
- **Reproducible multi-domain validation**: Skills validated across multiple domains to prevent overfitting to specific tasks.

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-MIND** | Wiki-style evolution = experience-tree entries as editable wiki pages. Each experience can be updated, forked, or merged with new observations. | SEAL pipeline distillation should support wiki-style operations: edit (update experience), fork (create variant), merge (combine experiences). Current experience-tree is append-only. |
| **NT-MEMORY** | Persistent knowledge wiki = KB with version history. Each skill modification creates a new version with provenance. | KB entries should track version history — not just current state but how they evolved. This enables rollback and learning-from-mistakes. |
| **NT-CORE** | Multi-domain validation = ConsciousnessTree's cross-domain health check. Skills validated across domains prevent domain-specific overfitting. | ConsciousnessTree should validate experience quality across domains — an experience only graduates to Keystone status if it generalizes. |

### Actionable Insight
**Skills should be living documents, not frozen templates.** WikiSkill's wiki-as-skill-evolution pattern is fundamentally different from NeoTrix's current append-only experience-tree. Making experiences editable with version history enables: (1) incremental improvement (edit), (2) exploration of alternatives (fork), (3) combining insights (merge). The reproducible validation protocol ensures quality isn't sacrificed for flexibility.

---

## Paper/Model 3: NeoHorse — Recursive Self-Improvement via Agentic Post-Training

**Source**: TokenRhythm/NeoHorse (GitHub, Sept 2026)
**Category**: Recursive Self-Improvement (RSI) Framework

### Core Mechanism
- **Agentic post-training**: The agent generates its own training data through interaction with the environment. Not just learning from human feedback but generating improvement signals autonomously.
- **Routing harness**: A lightweight router that decides which self-improvement signals to act on. Not all signals are equally valuable — the harness filters noise from genuine improvement opportunities.
- **Recursive loop**: Agent → evaluate performance → generate improvement data → retrain → evaluate → repeat. Each cycle produces measurable improvement.
- **RSI convergence**: The system converges to a stable improvement trajectory, avoiding runaway self-modification.

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-MIND** | RSI = SEAL pipeline's self-evolution taken to completion. Currently SEAL is human-initiated; RSI makes it agent-initiated. | SEAL pipeline should include agent-initiated evolution triggers, not just human-triggered. ConsciousnessTree detects improvement opportunities and initiates distillation. |
| **NT-CORE** | Routing harness = GWT salience for self-improvement signals. GWT already routes task-attention; extend to route improvement-attention. | GWT should broadcast improvement-signal salience alongside task-salience. The routing harness decides which improvements to act on based on predicted impact. |
| **NT-ACT** | Recursive loop = production orchestrator with self-improvement feedback. Each execution generates data for the next iteration's improvement. | Production orchestrator should log execution traces as training data for SEAL distillation. Closed-loop: execute → trace → distill → improve → execute. |

### Actionable Insight
**Self-improvement requires routing, not just accumulation.** NeoHorse's key insight is that recursive self-improvement needs a routing harness to filter improvement signals. Without it, the agent drowns in noise. NeoTrix's GWT already routes task-attention; extending it to route improvement-attention would enable targeted self-evolution. The convergence guarantee prevents runaway self-modification — a safety property NeoTrix needs.

---

## Paper/Model 4: Aha-Flow Distillation — Flow Markers in LLM Reasoning

**Source**: Wang-Xiaodong1899/Aha-Flow-Distillation (GitHub, Sept 2026)
**Category**: Reasoning Distillation via Flow Markers

### Core Mechanism
- **Flow markers**: Distinctive tokens or embeddings that mark reasoning transitions (hypothesis → evidence → conclusion, question → exploration → answer). These markers make reasoning structure explicit.
- **Marker-guided distillation**: When distilling from teacher to student, flow markers preserve reasoning structure. Student learns not just the answer but the reasoning pathway.
- **Aha-moment detection**: The system identifies "aha" moments — reasoning transitions where the model's understanding shifts. These moments are high-value distillation targets.
- **Flow-aware generation**: During inference, the model uses flow markers to maintain reasoning coherence across long chains-of-thought.

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-MIND** | Flow markers = experience-tree tags that mark reasoning transitions. Each experience entry should include flow markers (soil→root→trunk→branch→fruit→core). | Experience-tree entries should be tagged with ConsciousnessTree stage markers. This preserves reasoning structure during cross-session experience transfer. |
| **NT-CORE** | Aha-moment detection = ConsciousnessTree's fruit/core stage identification. The "aha" moment is when branch reasoning crystallizes into fruit. | ConsciousnessTree should detect and highlight aha-moments in growth cycles. These become high-priority experience-tree entries. |
| **NT-MEMORY** | Marker-guided distillation = experience compression that preserves reasoning pathways. Not just "what happened" but "how we reasoned about it." | KB experience storage should include reasoning-flow metadata, not just facts. This enables experience replay that reconstructs reasoning, not just outcomes. |

### Actionable Insight
**Reasoning structure is as valuable as reasoning outcomes.** Aha-Flow Distillation shows that preserving flow markers during distillation dramatically improves student performance. NeoTrix's experience-tree currently stores outcomes (what happened) but not reasoning pathways (how we reasoned). Adding flow markers — transition tags at each ConsciousnessTree stage — would make experiences more valuable for cross-session learning. The aha-moment detection maps directly to ConsciousnessTree's fruit/core stage identification.

---

## Paper/Model 5: Shared Memory Vault — Memory Governance for Multi-Agent Knowledge Bases

**Source**: songs-aaa/shared-memory-vault (GitHub, Sept 2026)
**Category**: Governed Multi-Agent Memory System

### Core Mechanism
- **Single KB, multiple agents**: One knowledge base shared by every agent and every device. Not per-agent memory but unified organizational memory.
- **Consistency guarantees**: Write conflicts resolved via governance rules (priority-based, recency-based, or user-defined). Read consistency ensures agents see coherent state.
- **Governance layer**: Access control (who can write what), conflict resolution (how to handle disagreements), provenance tracking (who wrote what, when).
- **Device-agnostic**: Same KB accessible from any device with consistency maintained. Enables seamless multi-device agent workflows.

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-MEMORY** | Governed shared KB = our KB with cross-domain governance. Currently domains have isolated namespaces; this adds governance for cross-domain access. | KB should support governed cross-domain queries — not just namespace isolation but controlled sharing with consistency guarantees. |
| **NT-GOVERNANCE** | Governance layer = our rev-officer + NT-GOVERNANCE policy engine. Access control maps to our domain permission model. | NT-GOVERNANCE should extend to KB access policies — not just code review policies but knowledge access policies. |
| **NT-ACT** | Multi-agent coordination via shared KB = our EventBus with KB as transport. Agents coordinate through shared state, not message passing. | Production orchestrator should support shared-KB coordination mode — agents read/write shared KB entries as coordination mechanism alongside EventBus messages. |

### Actionable Insight
**Shared memory needs governance, not just synchronization.** Shared Memory Vault's key insight is that multi-agent memory requires a governance layer — not just "everyone can read/write" but structured access control with conflict resolution. NeoTrix's KB has domain namespaces but no cross-domain governance. Adding a governance layer — who can write to which namespace, how conflicts resolve, what's the consistency model — would enable safe multi-domain knowledge sharing without corruption.

---

## Synthesis: Cross-Paper Patterns

| Pattern | Papers/Models | NeoTrix Integration |
|---------|---------------|---------------------|
| **Decoupled Storage & Processing** | CellularFlow | Separate KB fact storage from retrieval logic for independent evolution |
| **Living Documents** | WikiSkill | Experience-tree entries as editable wiki pages with version history |
| **Routing Self-Improvement** | NeoHorse | GWT routes improvement-attention alongside task-attention |
| **Flow-Aware Distillation** | Aha-Flow | Flow markers at ConsciousnessTree stages for reasoning preservation |
| **Governed Shared Memory** | Shared Memory Vault | Cross-domain KB governance with consistency guarantees |

## Priority Actions

| # | Action | Source | Domain | Effort |
|---|--------|--------|--------|--------|
| 1 | Add flow markers to experience-tree entries (ConsciousnessTree stage tags) | Aha-Flow | NT-MIND + NT-MEMORY | Medium |
| 2 | Implement wiki-style experience operations (edit/fork/merge) | WikiSkill | NT-MIND | High |
| 3 | Add GWT routing for self-improvement signals | NeoHorse | NT-CORE | High |
| 4 | Separate KB fact storage from retrieval indexing | CellularFlow | NT-MEMORY | High |
| 5 | Add cross-domain KB governance layer | Shared Memory Vault | NT-MEMORY + NT-GOVERNANCE | Medium |
