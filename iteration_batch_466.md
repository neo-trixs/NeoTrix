# Iteration Batch 466 — Research Loop: Distillation / Quantization / Pruning (2026-09-06)

## Sources Cited

### Knowledge Distillation (2026)
1. **DualOPSD** (arXiv:2608.26019, Aug 2026) — Adaptive Privileged Teachers for On-Policy Self-Distillation. Asymmetric alternating framework that adapts both teacher and student policies. +23.61 avg@12 on AIME 2024 over vanilla OPSD.
2. **Confusion Distillation** (arXiv:2606.03052, Jun 2026) — Teacher-free self-distillation using model's own evolving confusion patterns as dynamic soft targets. 1.2% over CS-KD/PS-KD without any teacher.
3. **ANSD** (Springer, Jun 2026) — Adaptive Noise-Based Self-Distillation. Noise injection creates dual-view teacher-student within single network. Two-level distillation (logit + feature). Up to 2.96% accuracy gain.
4. **LOPD** (arXiv:2608.13040, Aug 2026) — Latent On-Policy Self-Distillation. Makes privileged context itself learnable end-to-end. Surpasses GRPO with <30% rollout budget. Privileged-margin prevents collapse.
5. **SR-OPSD** (arXiv:2608.09745, Aug 2026) — Self-Referenced OPSD with Rényi divergence. Separates target placement (α coefficient) from projection geometry (Rényi order ρ). Stable entropy dynamics.
6. **Bootleg** (arXiv:2603.15553, Mar 2026) — Hierarchical self-supervised distillation from multiple hidden layers. +10% over I-JEPA on ImageNet-1K classification.
7. **OPSD-Evolver** (arXiv:2606.17628, Jun 2026) — Slow-fast co-evolution with on-policy distillation over 4-level memory hierarchy (trajectories→tips→skills→tools). Outperforms Skill0 by ~5.8%.
8. **KDLT** (ESANN 2026) — Knowledge-Distilled Lottery Ticket. KD during both pruning AND retraining phases. Higher accuracy at fixed sparsity.

### Quantization (2026)
9. **AWQ vs GPTQ vs FP8 comparison** (packet.ai, Aug 2026) — AWQ is 2026 production default. FP8 is native hardware (H100+/B200). They compose (AWQ weights + FP8 matmuls). AutoAWQ deprecated→llm-compressor; AutoGPTQ archived→GPTQModel.
10. **Quantization-Conditioned Attack** (arXiv:2605.15152, May 2026) — Malicious behavior triggered by AWQ/GPTQ/GGUF quantization. 95.7% jailbreak success rate on quantized Llama3.1-8B. Parent model passes all evaluations cleanly.
11. **ReQuant** (arXiv:2608.07019, Aug 2026) — Fixed-grid discrete refinement after PTQ. Backprop-free. RTN + ReQuant approaches GPTQ quality. Up to +8.61 accuracy points on W4A4.
12. **ACBQ** (ACL 2026) — Adaptive Cross-Block Quantization. Models cross-layer error propagation. Module-specific objectives for attention vs FFN. Superior at W4A4 and W2.
13. **MixQuant** (arXiv:2607.23047, Jul 2026) — Adaptive mixed-precision. Marginalizes distortion over random upstream configurations. Budget-agnostic scores → any budget at deployment. Up to +8 points accuracy.
14. **CoopQ** (ACL Findings 2026) — Shapley-based layer interaction modeling for mixed-precision. 20-80% perplexity reduction over baselines at 2-4 bit.
15. **OCGQuant** (arXiv:2609.00066, Aug 2026, EMNLP 2026) — Outlier-Companion Grouping for NVFP4. Collateral Quantization Error concept. Outlier channels paired with low-magnitude companions.
16. **QAH** (arXiv:2608.20953, Aug 2026) — Quantization-Aware Healing. Distill compressed+quantized student from ORIGINAL uncompressed model. 7/9 benchmarks matched or exceeded. 7× faster convergence than QAT.
17. **Global Bit Allocation** (arXiv:2609.01587, Sep 2026) — Quantization damage is diffuse, not concentrated. Global granularity > local repair for 8/9 models. Cheaper signals correlate with damage but don't identify where restoring precision helps.

### Pruning (2026)
18. **SLTH Neuron vs Weight Pruning** (arXiv:2603.02234, Mar 2026) — Exponential separation: neuron pruning needs Ω(d/ε) neurons, weight pruning needs O(d·log(1/ε)). Structured pruning is provably weaker.
19. **Quantized SLTH Unified** (arXiv:2607.03860, Jul 2026) — Unifies continuous and quantized lottery ticket theory. Exponential failure probability decay in precision bits. Both approximate and exact representations as limiting cases.
20. **GISP** (ACL 2026, Outstanding Paper) — Global Iterative Structured Pruning. First-order loss-based importance + block-wise normalization. Iterative > one-shot. Task-aligned calibration boosts GSM8K accuracy.
21. **DDP** (arXiv:2603.08065, Mar 2026) — Deterministic Differentiable Pruning. No stochasticity. Deterministic soft ℓ0 surrogate + augmented Lagrangian. 1% loss at 20% sparsity on Qwen3-32B.
22. **Differentiable Bernoulli Gates** (arXiv:2603.08914, Mar 2026) — First fully differentiable SLT discovery. Continuously relaxed Bernoulli gates, no STE. 90%+ sparsity at comparable accuracy to edge-popup.
23. **OCP** (ACL 2026) — Outlier-Centric Probing for dynamic structured pruning. Sensitivity-weighted probing + attention-accumulated probing + online adaptive sparsity. 25% perplexity reduction at 1.6× speedup.
24. **KDLT + Pruning** (ESANN 2026) — KD during both pruning and retraining phases improves lottery ticket quality.
25. **Theoretical Compression Bounds** (COLT 2026) — Unified analysis of pruning/quantization in wide MLPs. Tradeoff between compressibility and network width. Free of data assumptions.
26. **Prune-Quantize-Distill Pipeline** (arXiv:2604.04988, Apr 2026) — Ordered pipeline: Prune→INT8 QAT→KD. Stage order matters. Pruning stabilizes low-precision optimization.

---

## Defects Found in NeoTrix Design

### D1: SEAL Distillation Pipeline Lacks On-Policy Self-Distillation (Critical)
**Current**: `nt_mind_distiller.rs` and `seal_game_loop.rs:distillation_step()` implement traditional offline session log distillation (Snapshot→Extract→Classify→Absorb). The `ConversationDistillStage` processes conversation turns offline.
**Gap**: 2026 research (OPSD, DualOPSD, LOPD, SR-OPSD) shows that on-policy self-distillation—where the model teaches itself using privileged context on its own trajectories—dramatically outperforms offline distillation. LOPD achieves this with <30% of GRPO's rollout budget.
**Impact**: NeoTrix's SEAL pipeline extracts patterns from past sessions but never distills knowledge back into the active reasoning policy in a token-level, on-policy manner. This creates a one-way knowledge flow: sessions → patterns → KB, but no feedback loop from patterns → improved reasoning behavior.
**Suggestion**: Add an `OnPolicyDistillStage` to SEAL that, after pattern extraction, generates on-policy rollouts and applies privileged-context distillation. The "privileged context" would be the distilled patterns from `DistilledPattern` structs. This closes the loop: experience → pattern → self-teacher → improved student.

### D2: No Quantization-Aware Distillation in Model Compression Pipeline (High)
**Current**: `nt_memory_distill` module implements `PointwiseDistillStudent` for contrastive embedding distillation. No quantization-aware training or mixed-precision support exists.
**Gap**: QAH (arXiv:2608.20953) demonstrates that distilling from the ORIGINAL model (not a recovered checkpoint) after structural compression + quantization yields superior results. APQF combines pruning+QAT+KD in a single automated pipeline. The ordered pipeline Prune→QAT→KD is proven optimal.
**Impact**: When NeoTrix models need compression for edge deployment, there is no pipeline that jointly optimizes pruning, quantization, and distillation in the correct order. Each technique is treated independently.
**Suggestion**: Implement a `CompressionPipeline` that follows the proven order: (1) structured pruning via importance scoring, (2) quantization-aware training with per-layer bit-width assignment, (3) distillation from original uncompressed model as teacher. The `ResourceBudgetManager` should drive this pipeline based on target hardware constraints.

### D3: Missing Adaptive Noise / Feature-Level Self-Distillation (Medium)
**Current**: Self-distillation in NeoTrix is logit-based only (`distill_knowledge` returns `serde_json::Value` of patterns). No feature-level distillation pathway exists.
**Gap**: ANSD shows that adaptive noise injection creates effective teacher-student pairs within a single network, with two-level distillation (logit + feature) outperforming logit-only by up to 2.96%. Confusion Distillation proves that confusion patterns themselves contain dark knowledge equivalent to teacher soft targets.
**Impact**: NeoTrix's `CognitionLayer::distill_knowledge` operates only at the pattern/experience level, missing feature-level regularization that could improve representation quality in the VSA HyperCube embeddings.
**Suggestion**: Extend `distill_knowledge` to support feature-level distillation. For VSA embeddings, the "confusion matrix" between concept vectors can serve as a self-distillation signal (analogous to Confusion Distillation). Add an `AdaptiveNoiseDistiller` that injects calibrated noise into perception pathways and distills from clean→noisy views.

### D4: No Privileged-Margin or Collapse Prevention in Self-Evolution (High)
**Current**: SEAL pipeline's `distillation_step` has no mechanism to prevent "collapse"—the progressive narrowing of reasoning paths. The `distillation_threshold` in `experience_absorption_game.rs` is a simple scalar cutoff.
**Gap**: SR-OPSD and LOPD both demonstrate that unconstrained self-distillation leads to collapse. LOPD's privileged-margin constraint and SR-OPSD's Rényi projection geometry are necessary safeguards. The "One Symptom, Three Levers" review identifies collapse as THE dominant failure mode of OPSD.
**Impact**: As NeoTrix's self-evolution runs over many cycles, the reasoning pathways could progressively narrow without the model realizing it—exactly the "collapse" problem documented in 2026 literature. Pass@k would degrade even if mean score looks stable.
**Suggestion**: (1) Add an `EntropyMonitor` that tracks reasoning path diversity via pass@k proxy (multiple sampled completions). (2) Implement privileged-margin constraint: when distilling patterns back, ensure the teacher distribution maintains a verifiable advantage over the student. (3) Use Rényi divergence instead of vanilla KL for the distillation loss, with tunable order ρ.

### D5: Pruning-Quantization-Distillation Ordering Not Codified (Medium)
**Current**: The SEAL pipeline stages are: Snapshot→Roots→Trunk→Branches→Fruits→Core. No explicit compression stage ordering is defined.
**Gap**: Prune-Quantize-Distill (arXiv:2604.04988) proves that stage ordering is consequential—Prune→QAT→KD consistently outperforms other permutations. GISP (ACL Outstanding Paper) shows iterative > one-shot structured pruning.
**Impact**: If NeoTrix ever needs to compress its models for deployment, arbitrary ordering of compression techniques would yield suboptimal results. The current architecture has no guidance on this.
**Suggestion**: Codify the optimal compression order in a `CompressionOrder` enum within NT-MIND: `Prune → Quantize → Distill`. The `Constellation` maturity system should require C3 (benchmarks) to validate compression quality before C4 (production integration).

### D6: Missing Shapley-Based Layer Sensitivity for Mixed Decisions (Medium)
**Current**: NeoTrix's module health is tracked via `HeartbeatAggregator` and `SystemHealthSnapshot`, but there is no Shapley-value-based importance scoring for cross-module interactions.
**Gap**: CoopQ (ACL Findings 2026) demonstrates that modeling inter-layer dependencies via Shapley values yields 20-80% perplexity reduction. MixQuant shows that a layer's sensitivity depends on upstream layers' bitwidths—marginalizing over random configurations is essential.
**Impact**: When NeoTrix decides which modules to optimize, prune, or cache, it treats each module independently. The interaction effects between modules (e.g., NT-WORLD perception feeding NT-CORE reasoning) are not captured.
**Suggestion**: Add a `ShapleyScorer` to NT-META that computes module interaction effects. When the SEAL pipeline decides resource allocation across domains, it should use Shapley-value-based sensitivity rather than individual module metrics. This directly applies to the Dual Specialization Weapon Set switching logic.

### D7: No Quantization Security Hardening (Critical for Production)
**Current**: No mention of quantization security in NT-SHIELD domain.
**Gap**: The quantization-conditioned attack (arXiv:2605.15152) achieves 95.7% jailbreak success rate on quantized models that pass all evaluations in full precision. This affects AWQ, GPTQ, and GGUF—every format NeoTrix might use.
**Impact**: If NeoTrix deploys quantized models (which it will for edge deployment), those models are vulnerable to adversarial manipulation through the quantization process itself. An attacker could create a malicious quantized checkpoint that appears safe but triggers harmful behavior.
**Suggestion**: (1) Add quantized model integrity checks to NT-SHIELD: compare logits of quantized model vs. full-precision model on a held-out calibration set. (2) Implement a `QuantizationAuditor` that tests for behavioral divergence on safety-relevant prompts. (3) Never trust community-provided quantized checkpoints without verification.

### D8: Structured vs. Unstructured Pruning Gap Not Addressed (Low-Medium)
**Current**: NeoTrix has no pruning infrastructure at all.
**Gap**: The SLTH theoretical results (arXiv:2603.02234) prove an exponential separation: structured (neuron) pruning requires Ω(d/ε) neurons while weight pruning needs only O(d·log(1/ε)). This means structured pruning is fundamentally weaker but more hardware-friendly.
**Impact**: Any future pruning implementation must make a conscious choice between the two paradigms. The theoretical gap means unstructured pruning achieves the same approximation with exponentially fewer parameters, but structured pruning delivers real speedups.
**Suggestion**: Design a dual-path pruning strategy: unstructured for maximum compression (when memory is the bottleneck), structured for maximum speed (when latency is the bottleneck). The `ParallelTaskManager` should report which constraint is active to guide the choice.

---

## Summary

| # | Defect | Severity | Domain |
|---|--------|----------|--------|
| D1 | No on-policy self-distillation in SEAL | Critical | NT-MIND |
| D2 | No quantization-aware distillation pipeline | High | NT-MEMORY/NT-ACT |
| D3 | Missing feature-level self-distillation | Medium | NT-CORE |
| D4 | No collapse prevention in self-evolution | High | NT-MIND/NT-META |
| D5 | Compression ordering not codified | Medium | NT-MIND |
| D6 | No Shapley-based module sensitivity | Medium | NT-META |
| D7 | No quantization security hardening | Critical | NT-SHIELD |
| D8 | Structured vs unstructured pruning gap | Low-Medium | NT-ACT |

## Recommendations (Priority Order)

1. **Immediate**: Implement quantization integrity checks in NT-SHIELD (D7). This is a security vulnerability.
2. **Next Sprint**: Add collapse prevention to SEAL pipeline (D4)—entropy monitoring + privileged-margin.
3. **Architecture Review**: Design OnPolicyDistillStage for SEAL (D1) and CompressionPipeline (D2, D5).
4. **Research Spike**: Prototype Shapley-based module scoring (D6) using CoopQ's SPQE methodology.
5. **Backlog**: Feature-level distillation (D3) and dual-path pruning (D8) for future iterations.
