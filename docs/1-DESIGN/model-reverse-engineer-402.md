# Model Reverse Engineering — Cycle 402

**Date:** 2026-09-12
**Scope:** 5 new AI models/papers with patterns mapped to NeoTrix 7 domains

---

## 1. Attention-MoA: Inter-Agent Semantic Attention
**Source:** arXiv:2601.16596 (Jan 2026)

### Architecture
- **Inter-agent Semantic Attention:** Agents attend to each other's hidden representations, not just final outputs. Cross-agent attention weights computed via scaled dot-product over semantic embeddings.
- **Inter-layer Residual Module:** Residual connections between MoA layers prevent information degradation. Adaptive early stopping when consensus confidence exceeds threshold.
- **Deep Residual Synthesis:** Final output synthesized from residual streams across all layers, not just the last layer.

### Key Insight
Small open-source models (3-8B) ensembled via semantic attention outperform single massive models (Claude-4.5-Sonnet, GPT-4.1). The coordination mechanism matters more than individual model scale.

### NeoTrix Domain Mapping
| Domain | Integration | Mechanism |
|--------|------------|-----------|
| **NT-CORE** | GWT salience routing | Semantic attention = salience-weighted broadcast across specialist modules |
| **NT-MIND** | SEAL pipeline | Residual synthesis = experience accumulation across evolution layers |
| **NT-ACT** | Tool orchestration | Agent-as-worker with semantic coordination, not just function calling |
| **NT-IO** | Multi-model routing | Ensemble of cheap models > single expensive model (A1: Cost-Aware Routing) |

### Absorption Pattern
```
GWT Refinement: Add inter-module semantic attention to GWT broadcast mechanism.
Current: Salience-weighted broadcast to all modules.
Enhanced: Modules attend to each other's hidden representations, creating
          semantic resonance patterns that strengthen cross-domain coordination.
```

---

## 2. MemDecay: Region-Aware KV Cache Eviction
**Source:** arXiv:2607.10582 (Jul 2026)

### Architecture
- **Semantic Region Classification:** Tokens classified into regions (system instructions, plans, user turns, tool outputs, scratchpad) with region-specific eviction policies.
- **Attention Lifetime Calibration:** System tokens: 148-190 step half-life. Scratchpad tokens: 14-16 steps. Measured from actual attention patterns.
- **Priority Pinning:** Critical regions (system instructions) pinned at full-cache accuracy. No baseline preserves >13 of 24 system facts without pinning.
- **Decay Rate Calibration:** Decay rates calibrated from measured attention lifetimes, not heuristics.

### Key Insight
Semantic prompt structure is a robust signal for KV-cache management. Recency-based retention collapses as context grows, but region-aware retention remains effective.

### NeoTrix Domain Mapping
| Domain | Integration | Mechanism |
|--------|------------|-----------|
| **NT-CORE** | KV cache optimization | Region-aware eviction for KVMem tiered storage |
| **NT-MEMORY** | Knowledge persistence | System knowledge pinned, working memory decays naturally |
| **NT-MIND** | Experience distillation | Scratchpad/experience tokens decay faster, system principles persist |
| **NT-SHIELD** | Security policies | System-level security rules pinned at full fidelity |

### Absorption Pattern
```
KVMem Enhancement: Implement semantic region classification in kv_cache_optimizer.rs.
- System instructions → Pinned (full precision, never evicted)
- Task plans → High priority (slow decay, ~150 step half-life)
- Working memory → Medium priority (fast decay, ~15 step half-life)
- Scratchpad → Low priority (evict first)
- Tool outputs → Context-dependent (pin if referenced recently)
```

---

## 3. Explicit Trait Inference (ETI) for Multi-Agent Coordination
**Source:** arXiv:2604.19278 (ACL 2026)

### Architecture
- **Warmth/Competence Dimensions:** Two psychological dimensions derived from interaction histories. Warmth = trust/reliability. Competence = skill/capability.
- **Trait Profile Construction:** Agents maintain profiles of collaborators updated after each interaction. Profiles predict future behavior.
- **Profile-Guided Decision Making:** Coordination decisions (who to delegate to, how much to trust) informed by trait profiles.
- **Lightweight Mechanism:** No fine-tuning required. Works with any LLM via structured prompting.

### Key Insight
LLM agents can reliably infer others' traits from interaction histories and leverage structured awareness of others' traits for coordination. 45-77% payoff loss reduction in economic games.

### NeoTrix Domain Mapping
| Domain | Integration | Mechanism |
|--------|------------|-----------|
| **NT-MIND** | SelfModel dynamics | Warmth/competence tracking for all interacting agents/providers |
| **NT-ACT** | Provider routing | Route to providers based on measured warmth (reliability) + competence (capability) |
| **NT-CORE** | GWT salience | Trait profiles modulate attention allocation — reliable agents get more salience |
| **NT-SHIELD** | Trust assessment | Warmth dimension = trust score for security-sensitive operations |

### Absorption Pattern
```
SelfModel Extension: Add trait inference module to nt_core_self.
- Each provider/agent gets warmth (0-1, reliability history) and competence (0-1, capability history) scores
- Scores updated after each interaction based on outcome
- GWT routing weights include trait scores: salience × warmth × competence
- Provides data-driven provider selection instead of static configuration
```

---

## 4. CacheWise: Agent-Aware KVCache Management
**Source:** arXiv:2606.16824 (Jun 2026)

### Architecture
- **Prefix-Aware Request Scheduling:** Requests from same session scheduled together to maximize KV cache prefix reuse.
- **Predictive KVCache Eviction:** Predict future reuse from session metadata before evicting. Coding agents have predictable patterns (edit→test→edit).
- **Session-Aware Management:** KVCache managed at session level, not individual request level. Successive requests share context.
- **Load Balancer Integration:** Session-to-node affinity ensures KV cache locality.

### Key Insight
Existing LLM serving systems manage KVCache at individual request granularity, ignoring cross-request reuse in long-running agent sessions. Session-aware management reduces recomputation by 30-50%.

### NeoTrix Domain Mapping
| Domain | Integration | Mechanism |
|--------|------------|-----------|
| **NT-CORE** | KVMem optimization | Session-aware cache management across reasoning cycles |
| **NT-MEMORY** | Cross-session persistence | Predictive caching based on session patterns |
| **NT-IO** | Provider optimization | Session affinity for provider connections |
| **NT-ACT** | Workflow optimization | Cache-aware scheduling for multi-step tool workflows |

### Absorption Pattern
```
KVMem Session Awareness: Implement session-level KV cache management.
- Tag KV cache entries with session ID
- Predict next-step cache needs based on workflow stage
- Maintain session-to-node affinity for cache locality
- Evict cross-session entries before intra-session entries
```

---

## 5. AgentInfer: Co-Design of Inference Architecture
**Source:** arXiv:2512.18337 (Dec 2025, v2 Feb 2026)

### Architecture
- **AgentCollab:** Hierarchical dual-model framework — large model for complex reasoning, small model for routine operations. Dynamic role assignment based on task complexity.
- **AgentSched:** Cache-aware hybrid scheduler minimizing latency under heterogeneous request patterns. Co-schedules prefill and decode phases.
- **AgentSAM:** Suffix-automaton-based speculative decoding reusing multi-session semantic memory. Draft tokens predicted from accumulated experience, not just local context.
- **AgentCompress:** Asynchronous semantic compression distilling and reorganizing agent memory without disrupting ongoing reasoning. Self-evolution engine.

### Key Insight
Optimizing for agentic task completion (end-to-end agent loops) rather than per-token throughput is the key to building scalable, efficient, self-improving systems. 50%+ token reduction, 1.8-2.5x speedup.

### NeoTrix Domain Mapping
| Domain | Integration | Mechanism |
|--------|------------|-----------|
| **NT-CORE** | Dual model routing | AgentCollab pattern = our dual specialization (Weapon Set I/II) |
| **NT-MIND** | SEAL pipeline | AgentCompress = experience-tree absorption (async distillation) |
| **NT-MEMORY** | Semantic memory | AgentSAM = experience crystallization for speculative execution |
| **NT-ACT** | Task scheduling | AgentSched = cache-aware workflow scheduling |

### Absorption Pattern
```
SEAL + KVMem Integration: Implement speculative execution from experience.
- After each SEAL cycle, extract reusable reasoning patterns
- Store patterns as "draft templates" in experience namespace
- During new tasks, draft initial reasoning from templates (AgentSAM)
- Async compress session memory after task completion (AgentCompress)
- Dual-model routing: cheap model for template matching, expensive for novel reasoning
```

---

## Cross-Paper Synthesis

### Emerging Paradigm: Agent-Native Infrastructure
All 5 papers converge on the same insight: **agent systems require infrastructure designed for agent workloads, not adapted from chatbot serving.**

| Paper | Traditional Approach | Agent-Native Approach |
|-------|---------------------|----------------------|
| Attention-MoA | Single model scaling | Ensemble coordination via semantic attention |
| MemDecay | Uniform token eviction | Semantic region-aware eviction |
| ETI | Static role assignment | Dynamic trait inference from interaction |
| CacheWise | Request-level caching | Session-level predictive caching |
| AgentInfer | Per-token optimization | End-to-end agent loop optimization |

### NeoTrix Architecture Alignment
NeoTrix's 6-layer architecture is already agent-native:
- **L5 Cognition:** Dual-model routing (AgentCollab pattern)
- **L4 Emotion:** Trait inference for provider trust (ETI pattern)
- **L3 Embodiment:** Region-aware memory management (MemDecay pattern)
- **L2 Perception:** Semantic attention across modules (Attention-MoA pattern)
- **L1 Action:** Session-aware scheduling (CacheWise pattern)
- **L6 Meta-Cognition:** Self-evolving experience distillation (AgentInfer pattern)

### Priority Absorptions
1. **MemDecay** → Immediate: extend `kv_cache_optimizer.rs` with region classification
2. **ETI** → Short-term: add trait tracking to SelfModel for provider routing
3. **CacheWise** → Medium-term: implement session-aware KV cache management
4. **Attention-MoA** → Long-term: enhance GWT with inter-module semantic attention
5. **AgentInfer** → Long-term: integrate speculative execution from experience
