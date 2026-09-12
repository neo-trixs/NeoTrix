# Model Reverse Engineering — Cycle 444 (2026-09-12)

## 5 New Models/Papers

---

### 1. Flux Attention: Context-Aware Hybrid Attention for Efficient LLMs Inference

**Paper**: arXiv:2604.07394 (Apr 2026)
**Authors**: Quantong Qiu, Zhiyi Hong, Yi Yang, et al.
**Venue**: Preprint (ML/CL)

#### Core Mechanism
- **Layer Router**: Lightweight module injected into frozen pretrained LLMs that dynamically routes each layer to either Full Attention (FA) or Sparse Attention (SA) based on input context
- **Layer-wise routing** (not head-wise): preserves contiguous memory access, avoiding hardware-unfriendly load imbalance
- **Training cost**: Only 12 hours on 8×A800 GPUs (parameter-efficient)

#### Key Results
- 2.8× prefill speedup, 2.0× decode speedup
- Maintains accuracy on long-context and math reasoning benchmarks
- Compatible with existing sparse attention kernels (FlashAttention, etc.)

#### NeoTrix Domain Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** | GWT attention routing refinement | Layer-level attention switching as GWT salience mechanism — each layer dynamically decides full-vs-sparse based on context |
| **NT-MEMORY** | KV cache optimization | Layer Router could inform selective KV cache retention: SA layers cache less, FA layers cache more |
| **NT-WORLD** | Perception pipeline efficiency | For long-context perception streams (crawl results, multi-doc ingestion), layer-level routing reduces compute per token |

#### Actionable Pattern
```
GWT.salience(task) → LayerRouter.input_context → {FA, SA} per layer
```
The key insight: **routing at layer granularity** avoids head-level synchronization overhead while still adapting to input. NeoTrix GWT could adopt this: route entire attention blocks rather than individual heads.

---

### 2. Gated-Memory Routing for Multi-Agent LLM Systems

**Paper**: arXiv:2609.00237 (EMNLP 2026)
**Authors**: Rakibul Hasan Rajib, Mengxing Zheng, Qian Lou
**Venue**: EMNLP 2026 Main Conference

#### Core Mechanism
- **Memory Write Gate**: Learned gate that commits only non-redundant reasoning steps to memory, preventing information bloat
- **Retrieval Gate**: Learned gate that selects compact, relevant memory subset per agent, conditioning each decision on clean state
- **Adaptive Halting Controller**: Stops execution when accumulated memory contains sufficient evidence for answering

#### Key Results
- +2.44 accuracy over strongest baseline across 5 benchmarks
- 31.9% inference cost reduction on HumanEval
- Best average accuracy across reasoning and code-generation tasks

#### NeoTrix Domain Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-MEMORY** | Experience absorption gating | Write Gate → `experience-tree` absorption: filter redundant entries before KB commit. Retrieval Gate → query routing for `neotrix-experience query` |
| **NT-CORE** | SEAL pipeline convergence | Halting Controller → SEAL phase transition detection: stop evolution cycle when knowledge is sufficient |
| **NT-MIND** | Skill crystallization | Write Gate prevents skill node duplication during SEAL distillation phase |

#### Actionable Pattern
```
WriteGate(reasoning_step, existing_memory) → {commit, skip}
RetrievalGate(query, full_memory) → compact_context
HaltingController(memory_state) → {continue, stop_and_answer}
```
**Direct implementation target**: NT-MEMORY experience absorption. The Write Gate is the missing filter in `experience-tree` — currently all distilled experiences are committed; gated writes would prevent KB bloat.

---

### 3. AMA: Adaptive Memory via Multi-Agent Collaboration

**Paper**: arXiv:2601.20352 (ACL 2026 Findings)
**Authors**: Weiquan Huang, Zixuan Wang, Hehai Lin, et al.
**Venue**: ACL 2026 Findings

#### Core Mechanism
- **4-agent memory pipeline**:
  - **Constructor**: Multi-granularity memory construction (sentence/paragraph/topic-level)
  - **Retriever**: Adaptive query routing aligned with task complexity
  - **Judge**: Relevance/consistency verification, triggers iterative retrieval or Refresher
  - **Refresher**: Memory consistency enforcement — targeted updates or outdated entry removal
- **Hierarchical memory design**: Dynamically aligns retrieval granularity with task complexity

#### Key Results
- Significantly outperforms SOTA baselines on long-context benchmarks
- ~80% token reduction vs full-context methods
- Maintains retrieval precision and long-term memory consistency

#### NeoTrix Domain Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-MEMORY** | Memory lifecycle management | Constructor/Retriever/Judge/Refresher = 4-phase memory pipeline. Maps directly to `neotrix-experience` lifecycle: ingest→retrieve→validate→consolidate |
| **NT-SHIELD** | Consistency enforcement | Judge agent pattern for cross-session memory consistency checking |
| **NT-META** | Cross-domain memory coordination | Judge detects logical conflicts across domains → triggers Refresher |

#### Actionable Pattern
```
Constructor(raw_data) → multi_granularity_memory
Retriever(query, memory) → {sentence, paragraph, topic}_context
Judge(retrieved, query) → {sufficient, insufficient, conflicting}
Refresher(conflicting_memory) → {update, remove}
```
**Key insight**: Multi-granularity construction. NeoTrix KB currently stores flat experience nodes. AMA shows that **granularity-aware construction** (sentence for facts, paragraph for reasoning, topic for themes) improves retrieval precision. Target: NT-MEMORY `experience-tree` absorption with multi-granularity storage.

---

### 4. Token Sparse Attention: Efficient Long-Context Inference

**Paper**: arXiv:2602.03216 (ICML 2026)
**Authors**: Dongwon Jo, Beomseok Kang, Jiwon Song, Jae-Joon Kim
**Venue**: ICML 2026

#### Core Mechanism
- **Dynamic token-level sparsification**: Compresses per-head Q,K,V to reduced token set during attention, then decompresses output back to original sequence
- **Interleaved selection**: Token information reconsidered in subsequent layers (reversible, not permanent eviction)
- **Adaptive sparsity budget**: Determines how many tokens to retain and which ones per attention head at inference time

#### Key Results
- 3.23× attention speedup at 128K context
- Less than 1% accuracy degradation
- Compatible with FlashAttention, composable with existing sparse attention kernels

#### NeoTrix Domain Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** | E8 reasoning efficiency | Token-level sparsification for HyperCube knowledge traversal — skip low-relevance concept nodes |
| **NT-WORLD** | Perception stream filtering | For long crawl outputs, dynamically select salient tokens before embedding |
| **NT-MEMORY** | KB query optimization | Token-level selection as attention mechanism for BM25/vector hybrid retrieval |

#### Actionable Pattern
```
TokenSparsify(head_QKV, sparsity_budget) → compressed_QKV
Attention(compressed) → sparse_output
TokenDecompress(sparse_output, original_seq_len) → full_output
```
**Key insight**: Reversible sparsification. Unlike eviction-based methods, TSA can recover evicted tokens in later layers. Maps to NT-CORE E8 reasoning: prune low-relevance branches temporarily, but allow re-exploration in deeper reasoning layers.

---

### 5. DECENTMEM: Decentralized Self-Evolving Memory for Multi-Agent Systems

**Paper**: arXiv:2605.22721 (May 2026)
**Authors**: (Multi-author)
**Venue**: Preprint (CS.MA)

#### Core Mechanism
- **Dual-pool per agent**:
  - **Exploitation pool (E-pool)**: Consolidated successful trajectories
  - **Exploration pool (X-pool)**: LLM-generated candidate strategies
- **Online routing**: Stochastic bandit problem balancing exploitation vs exploration
- **Graph-structured random walk** with heuristic teleportation for self-evolving search
- **Stage-wise feedback**: External LLM-as-judge reweights E-pool and X-pool after each execution stage

#### Key Results
- Outperforms centralized memory (MetaGPT, ChatDev, G-Memory) on unstructured agent topologies
- Preserves role-complementary specialization (centralized memory homogenizes behavior)
- Framework-agnostic: works with AutoGen, DyLAN, AgentNet

#### NeoTrix Domain Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-MEMORY** | Experience hub dual-pool | E-pool = KB experience nodes (exploit known good patterns). X-pool = SEAL exploration candidates (test new approaches) |
| **NT-MIND** | SEAL pipeline exploration | X-pool feeds SEAL exploration phase; E-pool feeds crystallization phase |
| **NT-CORE** | GWT attention balance | Stochastic bandit routing = GWT salience balancing exploration vs exploitation |
| **NT-GOVERNANCE** | Per-domain specialization | Decentralized pools prevent cross-domain homogenization — each NT-* domain maintains its own experience pool |

#### Actionable Pattern
```
Agent.select_action() → Bandit.route(E_pool, X_pool) → {exploit, explore}
Execute(action) → LLM_judge.stage_feedback() → reweight(pools)
E_pool.add(consolidated_trajectory)
X_pool.add(LLM_generated_candidate)
```
**Key insight**: Decentralized memory preserves specialization. NeoTrix's 7-domain architecture naturally supports this: each domain (NT-CORE, NT-WORLD, etc.) should maintain its own dual-pool, not share a single memory pool. Cross-domain knowledge transfer happens via GWT routing, not memory merging.

---

## Cross-Paper Synthesis

### Dominant Pattern: Gated Memory Operations

Three of five papers converge on **gated memory** as the key mechanism:
1. Gated-Memory Routing: Write Gate + Retrieval Gate
2. AMA: Judge agent as consistency gate
3. DECENTMEM: Bandit router as exploration/exploitation gate

**NeoTrix Implication**: NT-MEMORY needs a unified gate layer across all memory operations:
- **Ingestion Gate**: Filter redundant experiences before KB commit
- **Retrieval Gate**: Compact context selection per query
- **Consistency Gate**: Judge + Refresher for memory integrity
- **Exploration Gate**: Bandit routing between known patterns and new candidates

### Attention Efficiency Convergence

Two papers address attention sparsity:
1. Flux Attention: Layer-level FA/SA routing
2. Token Sparse Attention: Token-level reversible sparsification

**NeoTrix Implication**: Both patterns could enhance NT-CORE E8 reasoning:
- Layer-level routing for HyperCube traversal depth selection
- Token-level sparsification for concept node pruning during reasoning

### Multi-Agent Coordination

Three papers address agent coordination:
1. Agora: Auction-based task allocation
2. Gated-Memory: Adaptive halting per agent
3. DECENTMEM: Decentralized dual-pool memory

**NeoTrix Implication**: NT-ACT orchestration could adopt:
- Auction-based model routing (replace static provider selection)
- Per-domain dual-pool memory (preserve NT-* specialization)
- Adaptive halting for SEAL pipeline convergence

---

## Priority Absorption Targets

| Priority | Paper | Target Domain | Implementation |
|----------|-------|---------------|----------------|
| P0 | Gated-Memory Routing | NT-MEMORY | Add Write Gate to experience-tree absorption |
| P1 | AMA | NT-MEMORY | Multi-granularity memory construction in KB |
| P1 | DECENTMEM | NT-MEMORY + NT-GOVERNANCE | Per-domain dual-pool architecture |
| P2 | Flux Attention | NT-CORE | Layer-level attention routing for GWT |
| P2 | Token Sparse Attention | NT-CORE | Reversible sparsification for E8 reasoning |
