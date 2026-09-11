# Model Architecture Reverse Engineering — 2026-09-11 Batch

10 models analyzed: GPT-4o, Claude 3.5 Sonnet, Gemini 2.5 Pro, Llama 4 Scout, DeepSeek V4.1 Flash, Qwen 3, Mistral Large 3, Phi-4 Reasoning, Yi-Lightning, Grok 3.

## Summary Matrix

| Model | Params (Total/Active) | Architecture | Key Innovation | Context | NeoTrix Mapping |
|-------|----------------------|--------------|----------------|---------|-----------------|
| GPT-4o | Undisclosed | Dense Transformer | End-to-end omni (text+audio+vision) | 128K | PerceptionBridge, NT-IO unified |
| Claude 3.5 Sonnet | Undisclosed | Dense Transformer | Constitutional AI + Agentic coding | 200K | NT-SHIELD governance, NT-ACT tool use |
| Gemini 2.5 Pro | MoE (sparse) | MoE Transformer | Thinking budget + 1M native multimodal | 1M+ | GWT salience routing, SEAL thinking |
| Llama 4 Scout | 17B/109B (16 experts) | MoE + iRoPE | 10M context via interleaved attention | 10M | KVMem paged KV, HyperCube long-range |
| DeepSeek V4.1 Flash | 284B/13B | MoE + CSA/HCA | Hybrid compressed attention + Muon optimizer | 1M | NT-WORLD attention gating, kv_cache_optimizer |
| Qwen 3 | 235B/22B (128 experts) | MoE (no shared experts) | Thinking/non-thinking hybrid mode switch | 128K | Dual Specialization, SEAL mode switching |
| Mistral Large 3 | 675B/41B | Granular MoE | Granular expert segmentation + NVFP4 | 256K | Rune Socketing (specialization), skill nodes |
| Phi-4 Reasoning | 14B | Dense Transformer | SFT + GRPO on curated "teachable" prompts | 32K | SEAL distillation, experience-tree |
| Yi-Lightning | MoE (fine-grained) | MoE + hybrid attention | Partitioned EP load balancing + cross-layer KV reuse | 128K | Heartbeat Aggregator, EventBus grounding |
| Grok 3 | Undisclosed | MoE + RL reasoning | 10x compute training + DeepSearch agent | 1M | NT-WORLD DeepSearch, NT-ACT agent loop |

---

## 1. GPT-4o (OpenAI, May 2024)

### Architecture
- **Type**: Autoregressive omni model — single neural network trained end-to-end across text, vision, and audio
- **Unified Tokenization**: Joint token stream of text tokens (BPE), image patch tokens (ViT-style), and audio tokens (neural codec like Encodec/SoundStream at 50-75 Hz)
- **Single Transformer Stack**: Cross-modal attention via self-attention within the unified stream — no separate cross-modal layers
- **Modality-specific Embedding/Unembedding**: Input embedding tables are modality-specific; output heads emit tokens for the active modality

### Key Innovations
1. **End-to-End Multimodal**: Eliminates the 3-stage pipeline (ASR→LLM→TTS) into a single network. Latency dropped from 2.8s to 232ms median
2. **Joint Modality Training**: Trained on interleaved multimodal data (transcribed conversations, captioned images, videos) so attention learns cross-modal relationships natively
3. **Neural Audio Tokenization**: Uses learned neural audio codecs producing discrete tokens at 50-100 Hz, making audio tractable within transformer context windows

### NeoTrix Mapping
- **PerceptionBridge** (L2→L5): GPT-4o's end-to-end modality fusion is the architectural precedent for PerceptionBridge's attention-gated sensory flow — bridge perception directly to consciousness without staging
- **NT-IO unified interface**: GPT-4o's single-model multimodal I/O validates NT-IO's "界面使徒" pattern — one interface for all modalities
- **GWT cost-aware routing** (Axiom A1): GPT-4o shows that modality routing can be learned jointly; NeoTrix's GWT can apply cost weights per-modality

---

## 2. Claude 3.5 Sonnet (Anthropic, June 2024)

### Architecture
- **Type**: Dense transformer, evolutionary upgrade from Claude 3 family
- **Training**: Unsupervised learning + Constitutional AI (CAI) — self-critique against principled constitution
- **Speed**: 2x faster than Claude 3 Opus, $3/M input tokens
- **Context**: 200K tokens

### Key Innovations
1. **Constitutional AI at Scale**: Model critiques itself against a written constitution, enabling alignment without massive human labeling. Iterative self-improvement on safety
2. **Agentic Coding**: 64% success on internal agentic coding eval (vs 38% for Opus) — understanding codebases, implementing PRs with multi-file editing in sandboxed environments
3. **Responsible Scaling Policy (RSP)**: ASL-2 safety classification with quantitative "thresholds of concern" for CBRN/cyber/autonomous risks — safety governance framework
4. **Expert Domain Win Rates**: 82% win rate in Law, 73% in Finance, 73% in Philosophy over Claude 3 Opus

### NeoTrix Mapping
- **NT-SHIELD governance** (rev-officer-agent): Claude's RSP and ASL classification is the direct precedent for NT-SHIELD's risk assessment framework — RiskAssessor scoring with classification gates
- **Constitutional AI → NT-GOVERNANCE**: Claude's constitution-based self-critique maps to NT-GOVERNANCE (Gov-衡) policy enforcement and compliance verification
- **Agentic coding → NT-ACT tool use**: Claude's sandboxed PR implementation pattern validates NT-ACT's orchestration approach — multi-file editing with self-correction loops

---

## 3. Gemini 2.5 Pro (Google DeepMind, March 2025)

### Architecture
- **Type**: Sparse MoE transformer with native multimodal support (text, vision, audio)
- **Training**: TPUv5p pods (8960 chips), synchronous data-parallel across multiple datacenters
- **Innovation**: Slice-granularity elasticity — automatic continuation with fewer slices on failure, 97% throughput during recovery
- **Distillation**: k-sparse distribution approximation for smaller Flash models

### Key Innovations
1. **Thinking Budget Mechanism**: Model self-allocates reasoning tokens; users can set budget caps. Scaling thinking budget → proportional accuracy improvement (Figure 4 in report). Controllable compute-time tradeoff
2. **1M+ Token Context**: Processes entire codebases, 3-hour videos, interleaved text+audio+video. Architecture changes to vision processing for long-form video understanding
3. **Training Stability Breakthrough**: Solved large-scale MoE training instabilities — signal propagation and optimization dynamics improvements yield better out-of-pre-training performance
4. **SDC Detection**: Split-Phase Silent Data Corruption detection via deterministic replay with per-device checksums. 0.25% steps replayed, 6% of replays genuine corruption

### NeoTrix Mapping
- **GWT salience + thinking budget** (Axiom A2): Gemini's thinking budget directly validates NeoTrix's approach of adaptive computation allocation — GWT salience can modulate thinking budget per task
- **SEAL pipeline thinking mode**: Gemini's controllable thinking maps to SEAL's stage-gated reasoning — budget allocation = stage gate thresholds
- **KVMem paged KV** (Axiom A2): Gemini's 1M+ context proves paged KV is viable; NeoTrix's kv_cache_optimizer should adopt similar tiered approach
- **ConsciousnessTree cycle boundary**: Gemini's step-level thinking boundaries align with ConsciousnessTree's growth cycle boundaries

---

## 4. Llama 4 Scout (Meta, April 2025)

### Architecture
- **Type**: Auto-regressive MoE with early fusion for native multimodality
- **Params**: 17B active, 109B total, 16 experts, alternating dense/MoE layers
- **Context**: 10M tokens (industry-leading)
- **Training**: ~40T tokens, 200 languages, fits on single H100 (Int4 quantization)

### Key Innovations
1. **iRoPE Architecture**: Interleaved attention layers WITHOUT positional embeddings, with inference-time temperature scaling for length generalization. "i" = interleaved, targeting "infinite" context. Key innovation: removing RoPE from some layers eliminates positional binding, enabling length extrapolation
2. **Early Fusion Multimodality**: Text and image tokens fused at embedding level from pre-training start — not bolted on as adapter
3. **Alternating Dense/MoE Layers**: MoE layers use routed experts + shared expert. Each token sent to shared expert + one routed expert. Memory-efficient: all params stored, subset activated

### NeoTrix Mapping
- **iRoPE → KVMem infinite context**: Llama 4's interleaved position-free attention layers are the architectural key to NeoTrix's KVMem "infinite context" goal — remove positional embeddings from some layers to enable length extrapolation
- **HyperCube long-range**: The 10M context capability validates HyperCube's approach to long-range knowledge association — context windows are no longer the bottleneck
- **Alternating dense/MoE → Skill Tree node tiers**: The dense/MoE alternation pattern maps to NeoTrix's 3-tier skill nodes (Small Passive → Notable Passive → Keystone) — different computation patterns per layer

---

## 5. DeepSeek V4.1 Flash (DeepSeek, 2025)

### Architecture
- **Type**: MoE with DeepSeekMoE framework, Multi-Token Prediction (MTP)
- **Params**: 284B total, 13B activated (V4-Flash); V4-Pro: 1.6T total, 49B activated
- **Context**: 1M tokens
- **Training**: 32T+ tokens, Muon optimizer

### Key Innovations
1. **Hybrid CSA+HCA Attention**: Compressed Sparse Attention (CSA) compresses KV cache along sequence dimension then applies sparse attention; Heavily Compressed Attention (HCA) does extreme compression with dense attention. Interleaved configuration enables 1M context with 27% single-token FLOPs and 10% KV cache vs V3
2. **Manifold-Constrained Hyper-Connections (mHC)**: Upgrades residual connections by constraining residual mapping onto a specific manifold — stabilizes signal propagation while preserving expressivity
3. **Muon Optimizer**: Faster convergence and greater training stability than Adam. Applied to majority of modules
4. **Hash Routing**: Initial Transformer blocks use Hash routing (deterministic by token ID) instead of learned routing — reduces routing overhead early in network
5. **Auxiliary-Loss-Free Load Balancing**: Bias-term approach for expert routing without auxiliary loss that degrades performance

### NeoTrix Mapping
- **CSA+HCA → kv_cache_optimizer**: DeepSeek's hybrid compression is the blueprint for NeoTrix's kv_cache_optimizer — tiered compression (moderate CSA for recent context, aggressive HCA for distant context)
- **mHC → ConsciousnessTree signal propagation**: Hyper-Connections address the same problem as ConsciousnessTree's cross-branch health monitoring — stable signal flow across deep stacks
- **Hash routing → Rune Socketing**: Deterministic routing by token ID in early layers maps to Rune Socketing's deterministic configuration — pre-assigned routing for foundational operations
- **Auxiliary-loss-free → GWT salience**: DeepSeek's bias-term balancing without auxiliary loss validates NeoTrix's approach of implicit balancing through salience rather than explicit loss terms

---

## 6. Qwen 3 (Alibaba, April 2025)

### Architecture
- **Type**: Dense (0.6B-32B) + MoE (30B-A3B, 235B-A22B)
- **MoE**: 128 total experts, 8 activated per token, NO shared experts
- **Key**: QK-Norm (removed QKV-bias), global-batch load balancing loss
- **Context**: 32K-128K, 151K vocabulary (BBPE)

### Key Innovations
1. **Thinking/Non-Thinking Hybrid Mode**: Unified framework switching between step-by-step reasoning (thinking mode) and instant responses (non-thinking mode). Dynamic switching based on query complexity or chat template. Eliminates need for separate chat vs reasoning models
2. **Thinking Budget Mechanism**: Users allocate computational budget; smooth, scalable performance improvements correlated with budget. Task-specific budget configuration for cost-quality optimization
3. **Strong-to-Weak Distillation**: Flagship model knowledge distilled to smaller models (0.6B-30B). 4-stage post-training: CoT cold start → reasoning RL → thinking mode fusion → general RL
4. **No Shared Experts**: Unlike Qwen2.5-MoE, removes shared experts entirely. Relying only on global-batch load balancing loss for expert specialization

### NeoTrix Mapping
- **Dual Specialization → thinking/non-thinking**: Qwen3's hybrid mode directly validates NeoTrix's Dual Specialization (Weapon Set I/II) — switch between deep reasoning and fast response based on task type
- **SEAL 4-stage pipeline**: Qwen3's post-training (CoT→RL→fusion→general RL) maps to SEAL's phased pipeline — each stage builds on the previous
- **Thinking budget → GWT attention modulation**: The budget mechanism is GWT's attention modulation made explicit — allocate more "thinking tokens" to high-salience tasks
- **Strong-to-Weak Distillation → NT-MIND**: Qwen3's distillation validates NT-MIND's approach of crystallizing flagship capabilities into smaller skill nodes

---

## 7. Mistral Large 3 (Mistral AI, December 2025)

### Architecture
- **Type**: Granular MoE + 2.5B Vision Encoder
- **Params**: 675B total, 41B active (16:1 ratio)
- **Context**: 256K tokens
- **Training**: 3000 NVIDIA H200 GPUs from scratch

### Key Innovations
1. **Granular MoE**: "Granular" expert segmentation — finer-grained than standard MoE. Expert sub-networks with dynamic routing activating only subset per token. Dramatic capacity-vs-cost ratio
2. **Integrated Vision Encoder**: 2.5B parameter vision encoder fused natively — not adapter-based. Enables OCR, document understanding, structured extraction
3. **Speculative Decoding**: Partnership with NVIDIA for speculative decoding on GB200 NVL72. Prefill/decode disaggregated serving
4. **NVFP4 Deployment**: Full 675B model runs on single 8×H100/A100 node via NVFP4 quantization. Practical deployment without multi-node tensor parallelism

### NeoTrix Mapping
- **Granular MoE → Rune Socketing**: Mistral's granular expert segmentation is Rune Socketing at model scale — finer-grained specialization with dynamic routing to appropriate "rune slots"
- **Skill nodes 3-layer**: The 16:1 active-to-total ratio validates NeoTrix's Small/Notable/Keystone tier system — massive capacity with selective activation
- **Integrated vision → NT-IO unified**: Native vision encoder validates NT-IO's approach of integrated multi-modal capability rather than adapter-based extension
- **NVFP4 → cost-aware routing** (Axiom A1): Running 675B on single node proves that cost-efficient serving requires hardware-aware quantization, validating NeoTrix's cost-aware model routing

---

## 8. Phi-4 Reasoning (Microsoft, April 2025)

### Architecture
- **Type**: 14B dense decoder-only transformer (same as Phi-4 base)
- **Modifications**: Repurposed placeholder tokens as `<think>`/`</think>`; RoPE base frequency doubled; context extended to 32K
- **Training**: 32 H100 GPUs, 2.5 days, 16B tokens, 1.4M prompt-response pairs

### Key Innovations
1. **"Teachable" Prompt Curation**: Prompts selected at the boundary of base model capability — optimal complexity and diversity. Not random sampling; deliberate selection for maximum learning signal
2. **SFT + GRPO Pipeline**: Supervised fine-tuning on o3-mini generated reasoning traces, followed by outcome-based reinforcement learning (Group Relative Policy Optimization). RL generates 1.5x longer responses with more detailed reasoning
3. **Synthetic Teacher Scaling**: o3-mini medium-effort teacher ≈ DeepSeek-R1 in quality but more token-efficient. High-effort teacher produces stronger but longer responses — inference-time compute scaling
4. **Reasoning as Transferable Meta-Skill**: 14B model outperforms 70B+ models. Improvements transfer to domains not targeted in training (algorithmic, planning) — 30-60% gains on TSP, 3SAT, Calendar Planning
5. **Minimal RL Seed**: Only 6,400 math problems needed for significant RL improvement — demonstrates data efficiency over data volume

### NeoTrix Mapping
- **Teachable prompts → experience-tree**: Phi-4's deliberate prompt selection at capability boundaries maps to experience-tree's curation of "teachable moments" — not all experiences are equally valuable for learning
- **SFT→RL pipeline → SEAL pipeline**: Phi-4's 2-stage post-training (SFT on teacher traces → RL on verifiable rewards) validates SEAL's phased approach — distill first, then optimize via environmental feedback
- **Reasoning as meta-skill → Skill Tree**: Phi-4's finding that reasoning transfers to unrelated domains validates Skill Tree's approach of building meta-capabilities that transfer across branches
- **Minimal seed data → KB efficiency**: 6,400 problems for meaningful RL improvement validates NeoTrix's approach of curated KB entries over massive datasets

---

## 9. Yi-Lightning (01.AI, December 2024)

### Architecture
- **Type**: Enhanced MoE with fine-grained expert segmentation
- **KV Cache**: Hybrid attention blocks (3 sliding window + 1 full attention), cross-layer KV cache reuse
- **Performance**: #6 on Chatbot Arena, #2-4 in Chinese/Math/Coding/Hard Prompts

### Key Innovations
1. **Partitioned EP Load Balancing (PEP)**: Splits experts within EP groups into smaller partitions for balanced token distribution during All-to-All communication. Three-tier balancing: per-expert (ST) → EP-group (EP) → partitioned (PEP)
2. **Cross-Layer KV Cache Reuse**: Shares KV cache states between consecutive full attention layers. Combined with hybrid attention (sliding window + full), achieves 82.8% memory reduction
3. **Hardware-Aware FP8 Architecture**: Model architecture precisely aligned with GPU hardware specs for FP8 quantization. Custom MoE operator achieving 1200 TFLOPS/card at FP8 on Hopper
4. **RAISE Safety Engine**: 4-component framework (RAISE-1/2/3/4) covering pre-training, post-training, input processing, and output control. Comprehensive safety across all phases

### NeoTrix Mapping
- **PEP → EventBus load balancing**: Yi-Lightning's three-tier balancing (expert→group→partition) maps to NeoTrix's EventBus event routing — tiered load balancing across specialist modules
- **Cross-layer KV reuse → kv_cache_optimizer**: The 82.8% memory reduction pattern directly informs NeoTrix's kv_cache_optimizer design — share attention states across similar computation layers
- **RAISE → NT-SHIELD multi-phase safety**: RAISE's 4-phase safety (pre→post→input→output) validates NT-SHIELD's defense-in-depth approach across all system phases
- **Hardware-aware design → cost-aware routing** (Axiom A1): Yi-Lightning's architecture-hardware co-design proves that model architecture must be aware of serving hardware

---

## 10. Grok 3 (xAI, February 2025)

### Architecture
- **Type**: Transformer-based LLM with MoE, trained on Colossus supercomputer
- **Compute**: ~200K H100 GPUs, 10x compute of previous SOTA
- **Modes**: Think (CoT reasoning) + DeepSearch (agentic web research)
- **Context**: 1M tokens (8x previous Grok)

### Key Innovations
1. **Scale-First Reasoning**: 10x compute training with large-scale RL to refine chain-of-thought. Self-correcting reasoning that explores alternatives, verifies solutions. Think time: seconds to minutes
2. **DeepSearch Agent**: Agent that queries web and X in real-time, synthesizes information, reasons about conflicting facts. Not just search — multi-source synthesis with reasoning
3. **Cost-Efficient Reasoning (Grok 3 mini)**: Parallel model optimized for reasoning cost-efficiency. Demonstrates that reasoning capability can be delivered at different price points
4. **Hybrid Dense/MoE**: Dense layers for core reasoning, MoE layers for specialized knowledge. Cross-expert attention gates for knowledge sharing without catastrophic interference

### NeoTrix Mapping
- **DeepSearch → NT-WORLD crawler**: Grok 3's DeepSearch is the architectural precedent for NT-WORLD's "虚空探索者" — multi-source synthesis with reasoning, not just retrieval
- **Think mode → GWT attention routing**: Grok 3's Think/DeepSearch mode switching maps to GWT's attention routing between specialist modules — route to reasoning modules for hard tasks, to fast modules for simple tasks
- **Scale-first approach → Axiom A1 tradeoff**: Grok 3's 10x compute approach validates the inverse of Axiom A1 — some tasks DO need massive compute, but the routing decision is what matters
- **Mini model → Skill Tree tiers**: Grok 3 + Grok 3 mini demonstrate the same tiered approach as NeoTrix's Skill Tree — flagship capability with efficient derivatives

---

## Cross-Model Synthesis: Top 10 Patterns

### Pattern 1: MoE is Universal
All 10 models except GPT-4o and Phi-4 use MoE or sparse activation. The industry has converged: **total params >> active params**. NeoTrix's Skill Tree node tiers should implement this — massive knowledge capacity with selective activation.

### Pattern 2: Thinking Budgets are Standard
Gemini 2.5, Qwen 3, and Grok 3 all implement controllable thinking budgets. **Compute allocation is now a first-class API parameter**. NeoTrix's GWT salience should modulate thinking budget per task.

### Pattern 3: Context Windows are Exploding
10M (Llama 4), 1M (Gemini, DeepSeek V4, Grok 3). The bottleneck is shifting from context size to context utilization efficiency. **NeoTrix's HyperCube must optimize for long-range retrieval, not just long-range storage**.

### Pattern 4: Hybrid Attention is the Answer
Yi-Lightning's sliding window + full attention, DeepSeek's CSA+HCA, Llama 4's iRoPE — everyone is mixing attention patterns. **One attention mechanism doesn't fit all positions in the sequence**.

### Pattern 5: Distillation is the Path to Efficiency
Gemini Flash (k-sparse distillation), Qwen 3 (strong-to-weak), Phi-4 (teacher-student) — small models get big-model quality through distillation. **NT-MIND's SEAL pipeline should emphasize distillation as a core stage**.

### Pattern 6: Hardware-Software Co-Design
Yi-Lightning (FP8-architecture alignment), DeepSeek (FP8 training), Mistral (NVFP4 deployment) — architecture must be aware of serving hardware. **NeoTrix's cost-aware routing (Axiom A1) must consider hardware capabilities**.

### Pattern 7: RL over SFT for Final Polish
Phi-4 (GRPO on 6K problems), Qwen 3 (reasoning RL), Grok 3 (large-scale RL) — RL consistently provides the final capability jump. **SEAL pipeline should end with RL, not SFT**.

### Pattern 8: Auxiliary-Loss-Free is Better
DeepSeek V4 (bias-term balancing), Yi-Lightning (EP partitioning) — explicit auxiliary losses degrade model quality. **Implicit balancing through architectural design is superior**.

### Pattern 9: Safety Must Be Multi-Phase
Claude (RSP/ASL), Yi-Lightning (RAISE 4-phase), Mistral (governance) — safety cannot be bolted on. **NT-SHIELD must implement defense-in-depth across all system phases**.

### Pattern 10: Agent Loops are Converging
Claude (agentic coding), Grok 3 (DeepSearch), Gemini (tool use) — the agent loop (observe→reason→act→verify) is the standard interaction pattern. **NT-ACT's orchestration must implement this loop as first-class architecture**.

---

## NeoTrix Priority Absorption Queue

| Priority | Innovation | Source Model | NeoTrix Target | Action |
|----------|-----------|--------------|----------------|--------|
| P0 | Thinking Budget Mechanism | Gemini 2.5, Qwen 3 | GWT salience | Implement adaptive computation allocation |
| P0 | Hybrid Attention Patterns | Yi-Lightning, DeepSeek V4 | kv_cache_optimizer | Tiered compression (CSA/HCA + sliding window) |
| P1 | iRoPE Infinite Context | Llama 4 Scout | KVMem | Interleaved position-free attention layers |
| P1 | Teachable Prompt Curation | Phi-4 Reasoning | experience-tree | Capability-boundary selection for learning |
| P1 | Auxiliary-Loss-Free Balancing | DeepSeek V4 | GWT routing | Bias-term implicit balancing |
| P2 | Granular MoE Segmentation | Mistral Large 3 | Rune Socketing | Fine-grained expert partitioning |
| P2 | Multi-Phase Safety | Yi-Lightning RAISE | NT-SHIELD | Defense-in-depth across all phases |
| P2 | DeepSearch Agent Pattern | Grok 3 | NT-WORLD | Multi-source synthesis with reasoning |
| P3 | Hash Routing (Deterministic) | DeepSeek V4 | Rune Socketing | Pre-assigned routing for foundational ops |
| P3 | Cross-Layer KV Reuse | Yi-Lightning | kv_cache_optimizer | Share attention states across similar layers |

---

*Generated: 2026-09-11 | Sources: GPT-4o System Card (arXiv:2410.21276), Claude 3 Model Card + Addendum, Gemini 2.5 Technical Report (arXiv:2507.06261), Llama 4 Model Card + Blog, DeepSeek V3/V4 Technical Reports (arXiv:2412.19437, arXiv:2606.19348), Qwen3 Technical Report (arXiv:2505.09388), Mistral Large 3 Documentation, Phi-4-Reasoning Technical Report (arXiv:2504.21318), Yi-Lightning Technical Report (arXiv:2412.01253), Grok 3 Announcement + Analysis Reports*
