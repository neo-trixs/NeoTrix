# Model Reverse Engineering #227 — Code LLM Architecture Survey

**Date**: 2026-09-11
**Models Surveyed**: 10
**NeoTrix Mapping**: Architecture innovations → NT-* domain absorption

---

## 1. Model Overview Matrix

| Model | Params | Active | Architecture | Context | Training | License |
|-------|--------|--------|-------------|---------|----------|---------|
| **Qwen3-Coder-480B-A35B** | 480B | 35B | MoE (160 experts, 8 active) | 256K native, 1M YaRN | 7.5T tokens, 70% code | Apache-2.0 |
| **DeepSeek-Coder-V2** | 236B / 16B | 21B / 2.4B | MoE (DeepSeekMoE) | 128K | 6T additional tokens | MIT + Model |
| **StarCoder2-15B** | 15B | 15B | Dense Transformer | 16K (4K sliding window) | 4T tokens, 600+ languages | OpenRAIL |
| **CodeLlama-3 (Llama 3)** | 70B | 70B | Dense Transformer + GQA | 8K native | Llama 3 continued pretraining | Llama 3 |
| **Granite Code (IBM)** | 2B-8B | 2B-8B | Dense Transformer | 8K-32K | Code-focused pretraining | Apache-2.0 |
| **Codestral-22B** | 22B | 22B | Mistral Transformer | 32K | Multi-lingual code | MNPL-0.1 |
| **Phind-CodeLlama-34B-v2** | 34B | 34B | CodeLlama finetune | 16K | Code-instruction tuned | Llama 2 |
| **WizardCoder-15B** | 15B | 15B | StarCoder finetune | 2K | Evol-Instruct on StarCoder | OpenRAIL-M |
| **Replit Code V-1.5** | 3.3B | 3.3B | MPT-style + Flash Attention | 4K | 1T tokens, 30 languages | Apache-2.0 |
| **SQLCoder-8B** | 8B | 8B | CodeLlama finetune | 4K | SQL-focused instruction tuning | BigCode |

---

## 2. Architectural Innovations by Model

### 2.1 Qwen3-Coder-480B-A35B (Alibaba, 2025)

**Architecture Innovations**:
- **Extreme MoE**: 480B total, 35B active — ratio 13.7:1 (highest open-source MoE sparsity)
- **160 experts, 8 active per token**: Each token routes to only 5% of experts
- **96 attention heads (Q) × 8 KV heads**: GQA with 12:1 Q/KV ratio for KV cache compression
- **Non-thinking mode**: Dedicated agentic coding mode without `<think>` blocks
- **YaRN extrapolation**: 256K native → 1M with YaRN context extension

**Training Innovations**:
- **Code RL at scale**: Execution-driven reinforcement learning on real-world coding tasks
- **Long-horizon Agent RL**: 20,000 parallel environments for multi-turn tool-use training
- **Synthetic data cleaning**: Qwen2.5-Coder used to clean/rewrite noisy training data

**Performance**: SWE-bench Pro 38.7, comparable to Claude Sonnet 4

### 2.2 DeepSeek-Coder-V2 (DeepSeek, 2024)

**Architecture Innovations**:
- **DeepSeekMoE**: Fine-grained expert specialization with 160 experts, 6 active
- **Multi-head Latent Attention (MLA)**: KV cache compression via low-rank projections
- **128K context**: Native long-context for repository-scale understanding
- **Continued pretraining from DeepSeek-V2**: Leveraging general LLM knowledge

**Key Insight**: First open-source MoE code model matching GPT-4-Turbo performance
- 236B total, 21B active → cost-effective deployment
- Supports 338 programming languages (vs. 86 in predecessor)

### 2.3 StarCoder2-15B (BigCode, 2024)

**Architecture Innovations**:
- **Sliding Window Attention (SWA)**: 4K window within 16K context for efficient attention
- **Grouped Query Attention (GQA)**: Efficient KV sharing across attention heads
- **The Stack v2**: 600+ languages, 4T tokens from Software Heritage archive
- **Fill-in-the-Middle (FIM)**: Native code completion with prefix/suffix/middle

**Data Innovations**:
- **Software Heritage integration**: Full provenance tracking of training data
- **Multi-source mixing**: Code + Wikipedia + ArXiv + GitHub issues
- **License compliance**: OpenRAIL with granular usage restrictions

### 2.4 CodeLlama-3 / Llama 3 (Meta, 2024-2025)

**Architecture Innovations**:
- **GQA (Grouped Query Attention)**: 8 KV heads for 70B model, ~2x KV cache reduction
- **RoPE (Rotary Position Embeddings)**: Better extrapolation than absolute positions
- **Iterative post-training**: SFT → RLHF → rejection sampling cycles
- **Code-specific continued pretraining**: Llama 3 base → code-focused domain adaptation

**Training Innovations**:
- **Multi-task fine-tuning**: Code generation + fill-in-middle + instruction following
- **Rejection sampling**: Generate multiple candidates, select best via reward model
- **Constitutional AI**: Policy-based safety alignment for code generation

### 2.5 Granite Code (IBM, 2024)

**Architecture Innovations**:
- **Efficient small models**: 2B-8B focused on enterprise deployment
- **Code-specific tokenizer**: Optimized for code tokens (indentation, keywords)
- **K8s-focused variants**: Granite-8B-code-k8s for Kubernetes-specific tasks
- **Apache-2.0 licensing**: Enterprise-friendly open source

**Training Innovations**:
- **Curriculum learning**: Progressive difficulty in code training
- **Multi-task pretraining**: Code + documentation + tests in unified objective

### 2.6 Codestral-22B (Mistral, 2024)

**Architecture Innovations**:
- **Mistral architecture**: Sliding window attention + GQA
- **Fill-in-the-Middle (FIM)**: Native code completion support
- **Multi-lingual code**: 80+ programming languages
- **Mistral Common tokenizer**: Shared tokenizer across Mistral model family

**Deployment Innovations**:
- **Mistral-native inference**: Optimized `mistral_inference` library
- **Instruction + FIM dual mode**: Same model for chat and completion
- **Efficient serving**: SGLang/vLLM integration with FP8 quantization

### 2.7 Phind-CodeLlama-34B-v2 (Phind, 2023)

**Architecture Innovations**:
- **CodeLlama-based finetune**: Leveraging Meta's code architecture
- **16K context**: Extended context for longer code files
- **Developer-focused tuning**: Optimized for programming Q&A and debugging

**Training Innovations**:
- **Stack Exchange data**: High-quality programming Q&A pairs
- **Multi-turn conversation**: Developer assistant dialogue format
- **Code explanation**: Natural language ↔ code bidirectional mapping

### 2.8 WizardCoder-15B (WizardLM, 2023)

**Architecture Innovations**:
- **Evol-Instruct**: Instruction evolution algorithm for code complexity
- **StarCoder-based**: Fine-tuned on StarCoder-15B architecture
- **Domain-specific adaptation**: Code instruction following capability

**Training Innovations**:
- **Progressive instruction evolution**: Simple → complex instructions
- **Code-specific Evol-Instruct**: Adapted from WizardLM's general instruction evolution
- **78K evolved instructions**: Curated training set with increasing complexity

### 2.9 Replit Code V-1.5 (Replit, 2023)

**Architecture Innovations**:
- **MPT-style architecture**: Multi-head attention with Flash Attention support
- **Triton Flash Attention**: Optional Triton-based efficient attention
- **Custom vocabulary**: 32,768 tokens optimized for code compression
- **4K context**: Compact but efficient for code completion

**Training Innovations**:
- **128 H100-80GB GPUs**: Large-scale distributed training
- **LLM Foundry + Composer**: MosaicML's efficient training framework
- **Multi-epoch training**: 5 epochs on code data with linear cooldown
- **Mixed data sources**: Stack Dedup + RedPajama + StackExchange

### 2.10 SQLCoder-8B (Defog, 2023)

**Architecture Innovations**:
- **CodeLlama-based finetune**: SQL-specific domain adaptation
- **Text-to-SQL focus**: Natural language → SQL query generation
- **Schema-aware prompting**: Database schema integration in context

**Training Innovations**:
- **SQL-specific instruction tuning**: Domain-targeted finetuning
- **Schema grounding**: Database metadata as structured context
- **Query optimization awareness**: Training on optimized query patterns

---

## 3. Cross-Model Innovation Taxonomy

### 3.1 Architecture Patterns

| Pattern | Models | NeoTrix Mapping |
|---------|--------|-----------------|
| **MoE (Mixture of Experts)** | Qwen3-Coder, DeepSeek-Coder-V2 | `nt_core_self::AttentionManager` — dual specialization routing |
| **Sliding Window Attention** | StarCoder2, Codestral, Replit | `nt_core::HyperCube` — attention windowing |
| **GQA (Grouped Query Attention)** | StarCoder2, CodeLlama-3, Codestral | `nt_core_llm::kv_cache_optimizer` — KV cache compression |
| **Fill-in-the-Middle** | StarCoder2, Codestral, DeepSeek-Coder-V2 | `nt_act::code_completion` — bidirectional inference |
| **YaRN Context Extension** | Qwen3-Coder | `nt_core::HyperCube` — context scaling |

### 3.2 Training Methodology Patterns

| Pattern | Models | NeoTrix Mapping |
|---------|--------|-----------------|
| **Execution-driven RL** | Qwen3-Coder | `nt_mind::seal_pipeline` — self-improving code quality |
| **Evol-Instruct** | WizardCoder | `nt_mind::skill_engine` — instruction complexity evolution |
| **Rejection Sampling** | CodeLlama-3 | `nt_meta::quality_gate` — candidate filtering |
| **Continued Pretraining** | DeepSeek-Coder-V2, CodeLlama-3 | `nt_world::domain_adaptation` — domain transfer |
| **Synthetic Data Cleaning** | Qwen3-Coder | `nt_memory::data_quality` — training data curation |

### 3.3 Deployment Patterns

| Pattern | Models | NeoTrix Mapping |
|---------|--------|-----------------|
| **FP8 Quantization** | DeepSeek-Coder-V2, Codestral | `nt_io::model_optimizer` — efficient inference |
| **Expert Parallelism** | Qwen3-Coder, DeepSeek-Coder-V2 | `nt_act::parallel_task` — MoE parallel execution |
| **Multi-device Sharding** | All large models | `nt_physical::gpu_manager` — distributed inference |
| **Mixed Precision Training** | All models | `nt_io::quantization_engine` — training efficiency |

---

## 4. NeoTrix Architecture Absorption

### 4.1 NT-CORE Absorptions

**From Qwen3-Coder MoE**:
- Expert routing algorithm for `AttentionManager`
- Cost-aware expert selection (cheap experts for simple tasks, expensive for complex)
- GWT salience weighting with expert utilization tracking

**From DeepSeek MLA**:
- KV cache compression via low-rank projections in `kv_cache_optimizer`
- Memory-efficient attention for long-context reasoning

**From StarCoder2 SWA**:
- Sliding window for HyperCube attention computation
- Local context focus for code-specific reasoning

### 4.2 NT-MIND Absorptions

**From Qwen3-Coder Code RL**:
- Execution-driven self-improvement in SEAL pipeline
- Parallel environment simulation for skill testing
- Hard-to-solve, easy-to-verify task identification

**From WizardCoder Evol-Instruct**:
- Instruction complexity evolution in skill engine
- Progressive difficulty training for code capabilities
- Domain-specific instruction curation pipeline

**From CodeLlama Rejection Sampling**:
- Multi-candidate generation with reward-based selection
- Quality gate for code generation outputs

### 4.3 NT-MEMORY Absorptions

**From StarCoder2 Stack v2**:
- Training data provenance tracking via KB versioning
- License compliance in knowledge base
- Multi-source data mixing with quality weights

**From Qwen3 Synthetic Data**:
- Automated data cleaning pipeline for KB embeddings
- Quality scoring for knowledge acquisition
- Synthetic data generation for underrepresented domains

### 4.4 NT-ACT Absorptions

**From DeepSeek-Coder-V2 338 Languages**:
- Multi-language code execution in tool registry
- Language-specific code generation templates
- Cross-language compilation and testing

**From Replit Flash Attention**:
- Efficient attention for real-time code completion
- Low-latency inference for IDE integration
- Triton-based acceleration for code tasks

### 4.5 NT-WORLD Absorptions

**From Qwen3-Coder Agentic Coding**:
- 20,000 parallel environment simulation
- Multi-turn tool-use interaction training
- Real-world software engineering task environment

**From SQLCoder Schema Grounding**:
- Database schema as structured context
- Schema-aware code generation
- Query optimization integration

### 4.6 NT-SHIELD Absorptions

**From CodeLlama Constitutional AI**:
- Policy-based safety alignment for code generation
- Malicious code detection and prevention
- License compliance enforcement

**From StarCoder2 License Compliance**:
- OpenRAIL license tracking in KB
- Usage restriction enforcement
- Provenance-based access control

### 4.7 NT-IO Absorptions

**From Codestral Dual Mode**:
- Instruction + FIM dual mode in LLM providers
- Mistral Common tokenizer integration
- Multi-model serving with mode switching

**From Replit MPT Architecture**:
- Flash Attention integration in inference engine
- Triton-based attention acceleration
- Mixed precision inference optimization

---

## 5. Key Insights for NeoTrix

### 5.1 MoE is the Future
- Qwen3-Coder's 13.7:1 sparsity ratio shows MoE scales efficiently
- NeoTrix should implement MoE-style routing for `AttentionManager`
- Cost-aware expert selection aligns with NeoTrix's cost-aware routing axiom

### 5.2 Execution-driven RL is Powerful
- Qwen3-Coder's Code RL on real-world tasks outperforms synthetic benchmarks
- NeoTrix SEAL pipeline should incorporate execution-driven self-improvement
- Parallel environment simulation enables safe experimentation

### 5.3 Context Extension is Critical
- YaRN (Qwen3-Coder) and SWA (StarCoder2) show different context strategies
- NeoTrix HyperCube should support both native and extrapolated context
- Adaptive context strategy based on task complexity

### 5.4 Data Quality Trumps Quantity
- Qwen3-Coder's synthetic data cleaning improves quality
- StarCoder2's provenance tracking ensures compliance
- NeoTrix KB should implement quality scoring for all knowledge

### 5.5 Dual-mode Operation is Standard
- Codestral's instruction + FIM, Qwen3-Coder's thinking + non-thinking
- NeoTrix should support multiple inference modes per model
- Mode switching based on task requirements

---

## 6. Implementation Roadmap

### Phase 1: MoE Foundation (P0)
- [ ] Implement MoE routing in `nt_core_self::AttentionManager`
- [ ] Add cost-aware expert selection (cheap/simple, expensive/complex)
- [ ] KV cache compression via MLA-inspired low-rank projections

### Phase 2: Training Pipeline (P1)
- [ ] Execution-driven RL for SEAL pipeline
- [ ] Evol-Instruct for skill engine complexity evolution
- [ ] Rejection sampling in quality gate

### Phase 3: Context Scaling (P2)
- [ ] YaRN-style context extrapolation for HyperCube
- [ ] Sliding window attention for code-specific tasks
- [ ] Adaptive context strategy based on task type

### Phase 4: Multi-mode Inference (P3)
- [ ] Dual-mode operation (instruction + FIM)
- [ ] Thinking + non-thinking mode switching
- [ ] Language-specific code generation templates

---

## 7. Sources

| Model | Primary Source | Key Reference |
|-------|---------------|---------------|
| Qwen3-Coder | arXiv:2505.09388 | Qwen3 Technical Report |
| DeepSeek-Coder-V2 | arXiv:2406.11931 | DeepSeek-Coder-V2 Paper |
| StarCoder2 | arXiv:2402.19173 | StarCoder 2 & The Stack v2 |
| CodeLlama-3 | Meta Llama 3 | Llama 3 Model Card |
| Granite Code | IBM Research | Granite Code Documentation |
| Codestral-22B | Mistral AI Blog | Codestral Announcement |
| Phind-CodeLlama | Phind Blog | Phind v2 Announcement |
| WizardCoder-15B | arXiv:2306.08568 | WizardCoder Paper |
| Replit Code V-1.5 | Replit Blog | Replit Code V-1.5 Announcement |
| SQLCoder-8B | Defog Research | SQLCoder Documentation |

---

*Generated by NeoTrix Model Reverse Engineering Pipeline*
*Absorbed into NT-MIND domain knowledge*
