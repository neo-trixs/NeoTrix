# Model Reverse Engineering — Cycle 363 (2026-09-12)

## Scope
5 recent AI papers on efficient inference, attention mechanisms, and agent coordination. Each mapped to NeoTrix 7 domains with actionable integration insights.

---

## 1. GLIDE: Guided Layerwise Hybrid Attention for Efficient LLM Inference
**Paper**: [arXiv:2607.24788](https://arxiv.org/abs/2607.24788) (Jun 26, 2026)

### Core Idea
Layer-wise heterogeneity in transformer attention: early layers are sensitive to softmax removal, deeper layers tolerate aggressive replacement by linear alternatives. GLIDE non-uniformly compresses the softmax footprint across the model, reducing aggregate KV cache I/O while preserving expressive power where most vital. Each layer balances an efficient linear recurrence with a variable-sized softmax window.

### Key Mechanism
1. **Layer-wise sensitivity profiling**: measure per-layer sensitivity to attention mechanism changes
2. **Hybrid routing**: early layers use full softmax; deep layers switch to linear recurrence
3. **Variable window sizing**: softmax window size varies per layer based on sensitivity
4. **Non-uniform compression**: different compression ratios at different depths

### Results
- Superior performance-efficiency tradeoffs vs uniform hybrid approaches
- Reduced end-to-end latency for long-context generation
- Quality preserved where expressivity matters most (early layers)

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|------------|----------|
| **NT-CORE** (E8 reasoning) | GLIDE's layer-wise sensitivity = E8 hexagram's per-hexagram attention weighting. Some reasoning branches need full attention (high-sensitivity); others tolerate compression (low-sensitivity). | P1 |
| **NT-MEMORY** (KB) | Layer-wise compression = tiered KB storage. High-priority nodes get full embedding fidelity; low-priority nodes get compressed summaries. GLIDE validates non-uniform compression across tiers. | P0 |
| **NT-WORLD** (perception) | Early perception layers need full attention (high sensitivity); late interpretation layers can compress. GLIDE's sensitivity profiling maps to PerceptionBridge's awareness_score calibration. | P1 |
| **NT-MIND** (evolution) | SEAL pipeline stages have different sensitivity to compression. Early exploration needs full fidelity; late distillation tolerates compression. GLIDE validates stage-aware compression ratios. | P2 |
| **NT-ACT** (action) | Action planning layers tolerate compression; execution layers need full attention. GLIDE maps to action tier optimization. | P2 |
| **NT-IO** (interface) | GLIDE's layer-wise hybrid = provider-side KV optimization. Request providers to implement non-uniform layer compression for cost reduction. | P1 |
| **NT-SHIELD** (security) | High-sensitivity layers = security-critical decision points that must not be compressed. GLIDE maps to threat-detection attention preservation. | P2 |

### Key Insight for NeoTrix
**Non-uniform compression across architectural layers.** GLIDE proves that not all layers are equal — early layers need full attention, deeper layers tolerate compression. For NeoTrix: NT-MEMORY should apply non-uniform compression to KB embeddings. High-frequency knowledge nodes (frequently accessed) maintain full fidelity; low-frequency nodes use compressed summaries. The compression ratio should be driven by access frequency (sensitivity), not uniform allocation.

---

## 2. Flux Attention: Context-Aware Hybrid Attention for Efficient LLMs Inference
**Paper**: [arXiv:2604.07394](https://arxiv.org/abs/2604.07394) (Apr 8, 2026)

### Core Idea
Dynamic, context-aware hybrid attention at the layer level. A lightweight Layer Router determines per-layer whether to use Full Attention (FA) or Sparse Attention (SA) based on input context. Unlike static allocation, Flux Attention adapts dynamically. Unlike head-level routing (hardware-unfriendly), it routes at the layer level for contiguous memory access. Trained with a dynamic penalty mechanism that controls inference budget.

### Key Mechanism
1. **Lightweight Layer Router**: small network that outputs binary decision per layer
2. **Context-aware routing**: router inspects input context to decide FA vs SA
3. **Dynamic penalty**: training objective includes sparsity constraint to prevent router degeneration
4. **Block-Sparse Attention**: block size 64, chunk size 16,384 for ultra-long sequences
5. **Hard routing**: argmax decision at inference (no soft mixing)

### Results
- 2.8× prefill speedup, 2.0× decode speedup
- Only 12 hours training on 8×A800 GPUs
- Up to 52% reduction in attended tokens
- Works on off-the-shelf frozen pretrained models

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|------------|----------|
| **NT-CORE** (E8/GWT) | Flux's Layer Router = GWT salience router. Each NT-* domain gets a binary attention decision: full processing or abbreviated. Context-aware = task-type-aware routing. | P0 |
| **NT-MEMORY** (KB) | Flux's dynamic routing = KB retrieval scoping. Router decides per-query whether to use full scan or index lookup. Dynamic penalty = cost constraint on retrieval. | P0 |
| **NT-MIND** (evolution) | SEAL pipeline router: each stage gets full or abbreviated processing based on maturity. C0-C1 = full attention; C4-C5 = sparse (compiled skill). | P1 |
| **NT-ACT** (action) | Action routing: simple actions use sparse attention; complex actions use full attention. Router = action complexity classifier. | P1 |
| **NT-IO** (interface) | Provider-side layer routing for cost optimization. Request providers to implement Flux-style dynamic layer compression. | P1 |
| **NT-SHIELD** (security) | Security-critical layers always use full attention. Router hardcodes high-trust layers to FA mode. | P2 |
| **NT-FEEL** (emotion) | Emotional processing layers may need different attention modes. High-intensity emotions = full attention; routine processing = sparse. | P3 |

### Key Insight for NeoTrix
**Dynamic layer routing with budget control.** Flux Attention proves that a lightweight router can make per-layer attention decisions that are both context-aware and hardware-efficient. For NeoTrix: GWT's salience router should implement Flux's dynamic penalty mechanism — the router must be trained to balance attention quality against computational budget. The dynamic penalty prevents the router from degenerating into "always full attention" (trivially optimal but expensive).

---

## 3. Explicit Trait Inference (ETI) for Multi-Agent Coordination
**Paper**: [ACL 2026](https://aclanthology.org/2026.acl-long.77) (Apr 2026)

### Core Idea
LLM agents infer and track partner characteristics along two psychological dimensions — **warmth** (trust) and **competence** (skill) — from interaction histories. These trait profiles guide coordination decisions: low warmth triggers defensive behavior (clarify intentions, discount unreliable input); low competence triggers support behavior (supply information, cover weak spots). No fine-tuning required — prompting-based design with minimal overhead.

### Key Mechanism
1. **Two-dimensional trait space**: warmth (trust/reliability) × competence (skill/effectiveness)
2. **Interaction-based profiling**: agents update trait scores after each interaction
3. **Decision guidance**: trait profiles directly influence coordination strategy
4. **Structured summaries**: task goals, actions, communication, outcomes are summarized for trait inference
5. **No training required**: prompting-based, works on any LLM

### Results
- 45-77% reduction in payoff loss in economic games
- 3-29% improvement in MultiAgentBench depending on scenario
- Trait profiles predict agent actions accurately
- Lightweight: minimal overhead, no fine-tuning

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|------------|----------|
| **NT-SHIELD** (security) | ETI warmth = trust scoring for NT-SHIELD's inter-domain message filtering. Low warmth triggers content verification; high warmth enables direct delegation. | P0 |
| **NT-ACT** (orchestration) | ETI competence = capability assessment for task routing. High-competence domains get complex tasks; low-competence domains get support tasks. | P0 |
| **NT-CORE** (reasoning) | ETI trait inference = E8 hexagram reasoning about partner states. Each hexagram could represent a partner's trait profile. | P1 |
| **NT-META** (meta-cognition) | ETI profiles = meta-cognitive model of partner capabilities. NT-META's ConsciousnessTree monitors not just self but partners. | P1 |
| **NT-MEMORY** (KB) | ETI trait profiles = KB node properties for NT-* domains. Store warmth/competence scores per domain module. | P1 |
| **NT-MIND** (evolution) | ETI profiles evolve over time = domain capability evolution tracking. Skill crystallization increases competence scores. | P2 |
| **NT-FEEL** (emotion) | Warmth dimension = emotional trust in NT-FEEL's social emotion model. Low warmth = distrust; high warmth = trust. | P2 |

### Key Insight for NeoTrix
**Psychologically-grounded trait inference for agent coordination.** ETI proves that simple warmth/competence dimensions — without fine-tuning — improve multi-agent coordination by 3-29%. For NeoTrix: NT-SHIELD should maintain trait profiles (warmth/competence) for each NT-* domain. When NT-ACT routes tasks, it consults these profiles to make trust-aware decisions. Low-trust domains get additional verification; high-trust domains get direct delegation. This is a lightweight coordination improvement that requires no architectural changes — just structured profiling.

---

## 4. KVMem: Virtualizing Million-Token Agent Workspaces on a Consumer GPU
**Paper**: [arXiv:2609.04852](https://arxiv.org/abs/2609.04852) (Sep 4, 2026)

### Core Idea
KV-context virtualization: overflowed workspace history preserved as paged KV state across GPU memory, host memory, and NVMe storage. At each agent step, model-native attention-space indexes (Mean-K vectors at 32-token block granularity) select relevant historical blocks and materialize a query-dependent execution view bounded by the model's native context window. GPU memory remains constant (~35 GiB) regardless of workspace size.

### Key Mechanism
1. **Paged KV virtualization**: KV blocks stored across GPU→Host→NVMe tiers
2. **Attention-space indexes**: Mean-K vectors for model-native relevance scoring
3. **Query-dependent execution view**: at each step, select relevant blocks and materialize in GPU
4. **Step-level scheduling**: update working set once per agent step (inter-step KL 37× higher than intra-step)
5. **Delta reuse**: decompose working set into Retained/Incoming/Outgoing; reuse GPU pages directly

### Results
- 1M tokens on 24GB RTX 5090 (4× model's native 256K window)
- ~50 tokens/s with Qwen3.6-27B NVFP4
- 48.4% success on DeepSWE (vs 43.8% compaction-only)
- GPU memory constant regardless of workspace size

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|------------|----------|
| **NT-MEMORY** (KB) | KVMem IS the NT-MEMORY architecture. Paged KV = KB node storage across memory tiers. Attention-space indexes = BM25/vector hybrid retrieval. Query-dependent view = on-demand context loading. | P0 |
| **NT-NEXUS** (cross-session) | KVMem's tiered storage = cross-session persistence. GPU = hot sessions, Host = warm sessions, NVMe = cold sessions. Step-level scheduling = session boundary detection. | P0 |
| **NT-CORE** (reasoning) | KVMem's execution view = E8's selective attention. At each reasoning step, load only relevant context from the virtual workspace. | P1 |
| **NT-MIND** (evolution) | KVMem's Delta reuse = SEAL pipeline's retained knowledge. Compiled skills (C4+) = GPU-resident; exploration = NVMe-resident. | P1 |
| **NT-ACT** (action) | KVMem's step-level scheduling = action context management. Each action step loads only relevant history, not full session. | P1 |
| **NT-IO** (interface) | KVMem's tiered storage = provider-side context optimization. Request providers to implement virtualized KV for long-context serving. | P2 |
| **NT-SHIELD** (security) | KVMem's query-dependent view = data minimization. Only load context relevant to current task, reducing exposure surface. | P2 |

### Key Insight for NeoTrix
**Virtualized memory with model-native indexing.** KVMem proves that agents can maintain million-token workspaces on consumer hardware by virtualizing KV state across storage tiers. For NeoTrix: NT-MEMORY should implement KVMem's tiered storage architecture. The key innovation is **model-native indexing** — the agent's own attention mechanism scores which blocks to load, rather than external heuristics. This validates NeoTrix's GWT routing: the system's own attention mechanism should drive memory retrieval, not external search.

---

## 5. DELTA: Training-Free Sparse Attention for Long-Context Reasoning
**Paper**: [ACL 2026](https://aclanthology.org/2026.acl-long.77) (referenced in search results)

### Core Idea
Training-free sparse attention that partitions transformer layers into three groups: (1) initial layers using full attention, (2) a small set of Δ-layers that identify salient tokens via aggregated head-level attention scores, (3) subsequent sparse-attention layers that attend only to the selected subset. Full KV cache preserved in GPU memory for accuracy; only computation is sparse.

### Key Mechanism
1. **Three-layer partition**: full → saliency detection → sparse execution
2. **Δ-layers**: dedicated layers that compute attention scores to identify important tokens
3. **Preserved full KV cache**: all tokens remain in GPU memory; only computation is selective
4. **Aggregated head scores**: combine attention scores across heads for robust saliency detection
5. **Training-free**: no additional training required

### Results
- Matches or surpasses full attention on AIME and GPQA-Diamond
- Reduces attended tokens by up to 4.25×
- 1.54× end-to-end speedup
- No accuracy degradation on reasoning tasks

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|------------|----------|
| **NT-CORE** (E8 reasoning) | DELTA's Δ-layers = E8's meta-reasoning layers. Dedicated layers that identify which reasoning paths matter before executing them. Full KV = all paths available; sparse execution = only relevant paths. | P0 |
| **NT-MEMORY** (KB) | DELTA's preserved full cache = KB full index with selective retrieval. All nodes exist; only relevant ones are accessed. Δ-layers = BM25/semantic scoring for retrieval. | P0 |
| **NT-WORLD** (perception) | DELTA's three-partition = perception pipeline: full sensory buffer → saliency detection → focused processing. All data captured; only relevant data processed. | P1 |
| **NT-MIND** (evolution) | DELTA's Δ-layers = SEAL's convergence check (Phase-0). Dedicated meta-layer that identifies which exploration paths matter before committing resources. | P1 |
| **NT-ACT** (action) | DELTA's sparse execution = action selection. All possible actions in buffer; Δ-layers score relevance; only relevant actions executed. | P1 |
| **NT-IO** (interface) | DELTA's training-free = zero-cost integration. Apply DELTA-style sparse attention to any existing provider without retraining. | P1 |
| **NT-SHIELD** (security) | DELTA's preserved cache = complete audit trail. All security-relevant data preserved; only computation is selective. No data loss from sparsification. | P2 |

### Key Insight for NeoTrix
**Preserve everything, compute selectively.** DELTA proves that you can reduce computation by 4× without losing accuracy by keeping all data in memory but only processing what matters. The key is the dedicated Δ-layers that identify saliency before execution. For NeoTrix: NT-CORE should implement DELTA's three-partition pattern. The E8 hexagram's reasoning layers should include dedicated "saliency detection" layers that identify which reasoning branches matter before executing them. Full context is always available (preserved KV); only computation is selective.

---

## Cross-Paper Synthesis: NeoTrix Integration Roadmap

### Immediate (P0 — This Cycle)
1. **KVMem tiered storage** → NT-MEMORY: implement GPU→Host→NVMe KV virtualization with model-native indexing
2. **ETI trait profiles** → NT-SHIELD: maintain warmth/competence profiles for each NT-* domain
3. **Flux Attention dynamic routing** → GWT: implement per-domain attention routing with dynamic budget penalty
4. **DELTA Δ-layers** → NT-CORE: add dedicated saliency detection layers to E8 reasoning

### Short-term (P1 — Next 2 Cycles)
5. **GLIDE non-uniform compression** → NT-MEMORY: tiered compression ratios based on access frequency
6. **Flux's Layer Router** → NT-IO: request providers to implement dynamic layer compression
7. **ETI decision guidance** → NT-ACT: trust-aware task routing based on domain trait profiles
8. **KVMem step-level scheduling** → NT-NEXUS: session boundary detection via inter-step KL divergence

### Medium-term (P2 — Quarter)
9. **DELTA training-free** → NT-IO: zero-cost sparse attention for all provider integrations
10. **GLIDE sensitivity profiling** → NT-MIND: stage-aware compression ratios for SEAL pipeline

### Meta-Pattern
All five papers converge on the same insight: **selective processing with full information retention beats information loss.** KVMem preserves all KV state across tiers. DELTA preserves full cache but computes selectively. GLIDE preserves early layers at full fidelity. Flux routes dynamically but never discards. ETI preserves all interaction history but selectively infers traits.

For NeoTrix: the architecture should always preserve full information (KB never deletes, only archives) and apply selective processing (GWT attention routing, tiered compression, Δ-layer saliency). The competitive advantage is in the selection mechanism, not in information reduction.
