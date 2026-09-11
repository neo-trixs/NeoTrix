# Model Reverse Engineering — Cycle 343

## 5 Papers

### 1. PARSER: Read in Parallel, Reason in Depth (arxiv:2609.06702)
- **Category**: Long-Context Agent Architecture
- **Key Insight**: Decouple reading from reasoning. Lightweight subagents read chunks in parallel; lead agent reasons through iterative scatter-gather rounds. 4B backbone outperforms sequential baselines by 5.7pts avg, 12pts at 896K tokens. 11x latency reduction.
- **NeoTrix Mapping**: NT-MEMORY parallel retrieval + NT-CORE GWT lead-agent reasoning. Scatter-gather pattern for `experience-tree` branch loading — subagents scan KB namespaces in parallel while ConsciousnessTree synthesizes.

### 2. Language Models Can Control Their Own Attention (arxiv:2609.02737)
- **Category**: Intrinsic Sparse Attention
- **Key Insight**: Declarative Attention (DA) protocol — model declares `<global>`, `<focus>`, or `<local>` regions during chain-of-thought. Engine skips KV cache reads for non-declared regions. 52% KV reduction on Gemma-4-31B with 1.27pp accuracy drop.
- **NeoTrix Mapping**: NT-CORE GWT attention modulation — models self-declare attention scope per growth cycle phase. PerceptionBridge uses DA-style declarations to filter sensory events. Axiom A2 (Context as Scarce Resource) validated.

### 3. Codebook Agent: Amortized Topology Design (arxiv:2609.02264)
- **Category**: Multi-Agent Topology Optimization
- **Key Insight**: Vector-quantized autoencoder compresses successful topologies into 16-entry codebook. Reward-weighted MLP maps query to code distribution. 2.4ms topology generation, 22-33% fewer LLM tokens. Topologies collapse to ~6 distinct graphs regardless of codebook capacity.
- **NeoTrix Mapping**: NT-CORE E8 hexagram as codebook — each hexagram is a pre-compiled topology. GWT routes queries to matching hexagram, avoiding runtime topology search. Skill Tree nodes = learned codebook entries.

### 4. Bilevel Coordinated Reflection (arxiv:2609.02750)
- **Category**: Game-Theoretic Multi-Agent Coordination
- **Key Insight**: Model orchestrator-worker as bilevel coordination game. SRMA (Stochastic Reflective Memory Ascent) accepts memory only after grounded evaluation risk decreases. Environment-grounded gates outperform transcript-only gates (information-theoretic impossibility result). 72.2% SWE-bench resolution.
- **NeoTrix Mapping**: NT-META ConsciousnessTree reflection cycle — SRMA maps to Fruits→Core phase where evolution果实 accepted only if治理 feedback risk decreases. Environment-grounded = KB-persisted evidence, not in-context claims.

### 5. SRPO: Setwise Relative Policy Optimization (arxiv:2609.08452)
- **Category**: Multi-Agent RL Training
- **Key Insight**: Treat active set of outputs consumed by one transition as one multi-agent action. Cardinality-normalized set ratio, single relative advantage, single clip. Unifies division-of-labor and joint co-evolution across fixed/mixed/dynamic workflows.
- **NeoTrix Mapping**: NT-MIND SEAL pipeline — SRPO's setwise approach maps to treating cross-domain module outputs as unified actions. Dual Specialization (Weapon Set I/II) = set-level optimization across CORE+WORLD or CORE+MIND modes.

## Cross-Paper Synthesis

Three convergence patterns:

1. **Decouple reading from reasoning** (PARSER, Declarative Attention) — both achieve massive efficiency gains by separating what to read from what to think. Maps to NeoTrix's L2 Perception → L5 Cognition layer separation.

2. **Pre-compiled topology beats runtime search** (Codebook Agent, SRPO) — compress successful configurations into reusable codebooks rather than optimizing per-query. Validates NeoTrix's E8 hexagram + Skill Tree approach.

3. **Environment-grounded reflection** (Bilevel Coordinated Reflection, SRMA) — memory acceptance requires external evidence, not self-evaluation. Validates NeoTrix's KB-as-truth-source and `converge_check` architecture self-audit.
