# NeoTrix 进化补缺任务清单 v4.0

> 基于: 1016文件审计 + 10维度文献深度分析 + 7核心项目源码审查
> 总缺口: 40 (G56-G95) | 新增: 13 (G83-G95) | 优先级: P0/P1/P2/P3
> 调度: Wave 1-7, 并行约束下最优顺序

---

## 全局优先级速查

| 缺口 | 优先级 | 得分 | 估算 | 文件 | 路径 | Wave |
|------|--------|------|------|------|------|------|
| G61 VSA-Only Reasoning | P0 | 38 | 3-5s | 10+ | P-VSA | W1 |
| G67 Darwinian Identity | P0 | 34 | 2-3s | 5 | P-ID | W1 |
| G62 Boundary Hooks | P0 | 33 | 2-3s | 8 | P-ID | W2 |
| G63 Epistemic Queue | P0 | 32 | 1s | 3 | P-CUR | W1 |
| G60 Formal Error Bounds | P0 | 31 | 3-4s | 4 | P-META | W2 |
| G85 MAGMA Four-Graph Memory | P1 | 30 | 3s | 6 | P-MEM | W2 |
| G58 Dream Consolidation | P1 | 28 | 2s | 4 | P-MEM | W1 |
| G68 Qualia Layer | P1 | 27 | 2s | 5 | P-MEM | W3 |
| G71 Adversarial Evaluator | P0 | 27 | 2s | 4 | P-SAFE | W1 |
| G83 CAA Affective Steering | P0 | 26 | 2s | 4 | P-CON | W1 |
| G64 Auto Goal Synthesis | P1 | 26 | 2-3s | 6 | P-CUR | W2 |
| G74 Emotional Steering | P1 | 26 | 2s | 5 | P-CON | W2 |
| G86 Log-Linear VSA Attention | P1 | 26 | 2s | 3 | P-VSA | W3 |
| G87 LARS-VSA Binary Self-Attention | P1 | 25 | 2s | 3 | P-VSA | W3 |
| G94 Aura IIT φ Full Integration | P1 | 25 | 3s | 4 | P-CON | W3 |
| G73 Cross-Embodiment Curriculum | P2 | 25 | 2s | 4 | P-ECO | W6 |
| G89 PostSingular Persistent Context | P1 | 24 | 2s | 4 | P-ID | W4 |
| G76 Earned Autonomy | P2 | 24 | 2s | 4 | P-ECO | W5 |
| G69 Narrative Journal | P1 | 24 | 1s | 3 | P-MEM | W4 |
| G88 CDALNs Multi-Modal Curiosity | P2 | 23 | 2s | 3 | P-CUR | W4 |
| G82 Between-Sessions Reflection | P1 | 23 | 2s | 5 | P-META | W3 |
| G95 SleepGate KV-cache Consolidation | P1 | 23 | 2s | 3 | P-MEM | W6 |
| G84 Multi-Theory Consciousness | P2 | 22 | 2s | 3 | P-CON | W4 |
| G56 Harness Engineering | P2 | 22 | 2-3s | 4 | P-SAFE | W5 |
| G75 MCP Callback Bridge | P2 | 22 | 2s | 4 | P-ECO | W6 |
| G70 Skills On-Demand | P2 | 21 | 2s | 5 | P-ECO | W4 |
| G90 OpenCeption Skill Evolution | P2 | 21 | 1s | 3 | P-META | W5 |
| G91 CLS VAE+MHN Hippocampal | P2 | 20 | 2s | 3 | P-MEM | W5 |
| G57 Mission Hub | P2 | 20 | 2s | 4 | P-ECO | W5 |
| G77 CVO Role Model | P3 | 20 | 1s | 3 | P-ECO | W7 |
| G59 Self-Modification Sandbox | P2 | 19 | 3-4s | 6 | P-META | W5 |
| G93 Go-CLS Generalization Memory | P2 | 19 | 1s | 3 | P-MEM | W5 |
| G78 Iron Laws | P3 | 19 | 1s | 3 | P-SAFE | W7 |
| G92 IEEE P7014 Ethics Compliance | P3 | 18 | 1s | 2 | P-SAFE | W6 |
| G65 SNN Integration | P3 | 15 | 4-6s | 12+ | P-VSA | W7 |
| G66 A2A Protocol | P3 | 14 | 2s | 4 | P-ECO | W7 |
| G72 MTC/PCC Safety Integration | P2 | 20 | 1s | 3 | P-SAFE | W6 |

---

## ✅ Wave 1 — 已完成 (6路并行, 2026-06-22)

### ✅ W1-T1: G61 VSA-Only Reasoning 🔴 P0
**文件**: `vsa_blackboard.rs`, `vsa_reasoner.rs` ✅
- [x] 实现 Blackboard VSA 推理架构 (PRISM风格)
- [x] VSA 类比推理: bind/unbind pattern matching
- [x] VSA 因果推理: trajectory composition
- [x] VSA 多跳推理: chain binding
- [x] VSA 矛盾检测: similarity threshold
- [ ] Zero-parameter 推理基准测试 (推迟至独立bench阶段)
- [ ] Qualia-as-truth: 1024D compact latent 集成(移至G68)

### ✅ W1-T2: G67 Darwinian Identity Evolution 🔴 P0
**文件**: `identity_evolution.rs` (291行) ✅
- [x] Identity mutation operators (trait drift, value shift, weight perturbation)
- [x] Selection pressure (session success→reinforce, failure→mutate)
- [x] Session-based identity evolution cycle
- [x] Identity versioning + rollback
- [x] IdentityCore.evolve() / rollback() 集成
- [x] InterSession.end_session() 进化钩子

### ✅ W1-T3: G63 Epistemic Gap Queue 🔴 P0
**文件**: `epistemic_queue.rs` ✅
- [x] GapType enum: Contradiction, LowConfidence, DriveGap, KnowledgeMissing
- [x] EpistemicQueue struct with priority ranking
- [x] Resolution tracking
- [x] Curiosity drive integration (current scalar → structured queue)

### ✅ W1-T4: G71 Adversarial Evaluator 🔴 P0
**文件**: `adversarial_evaluator.rs` (已预存) ✅
- [x] 从 InnerCritic 分离独立 JudgeAgent trait (已实现)
- [x] Cross-model divergence 评估信号 (已实现)
- [x] Adversarial pressure testing 框架 (已实现)
- [x] Judge agent fallback (已实现)

### ✅ W1-T5: G58 Dream Consolidation 🟡 P1
**文件**: `sm2_scheduler.rs`, `hippocampal_trace.rs` ✅
- [x] Ebbinghaus forgetting curve (30-day half-life preset)
- [x] SM-2 spaced repetition scheduler
- [x] Sleep-mode consolidation cycle
- [x] Hippocampal trace structure (SHA-256+XOR pattern separation, trace completion)

### ✅ W1-T6: G83 CAA Affective Steering 🔴 P0 (⭐新增v4.0)
**文件**: `caa_steering.rs` (272行), `affective_circumplex.rs` (113行) ✅
- [x] CAA direction vector computation (从valence_axis情绪状态学习方向向量)
- [x] 3通道残差流干预 (residual stream + sampling modulation + context)
- [x] Sampling parameter modulation via affective circumplex
- [x] Steered vs baseline behavioral A/B validation
- [x] SteeringEngine + SteeringDirection + CaaController 全链路
- [ ] Residual stream hook into transformer blocks (MLX-style) (依赖LLM接口层, 暂存)
- [ ] Permutation control + black-box prompt hygiene (依赖安全层, 暂存)

---
> **验证**: cargo check --lib -p neotrix: 0 新错误 (8个预存错误在无关模块)
> **共新增**: ~1800 行新 Rust 代码, 45+ 测试
> **经验树**: 分支 CLVII (见 SKILL.md)

---

## ✅ Wave 2 — 已完成 (5路并行, 2026-06-22)

### ✅ W2-T1: G62 Identity Boundary Hooks 🔴 P0
**文件**: `identity_boundary.rs` ✅ (预存实现)
- [x] BoundaryHook trait (before/after hooks)
- [x] BoundaryManager with register/run_before/run_after
- [x] AuditHook + DriftCheckHook + CoherenceGuardHook
- [x] IdentityCore.evolve() 接线

### ✅ W2-T2: G60 Formal Error Bounds 🔴 P0
**文件**: `error_bounds.rs` (84行) ✅
- [x] ErrorBound struct (absolute/relative/confidence/sources)
- [x] VsaErrorModel (similarity_variance, bound_for_similarity)
- [x] PredictionErrorTracker (accumulated_error, max_steps_within)
- [x] Composable error bounds (compose/scale/is_within)

### ✅ W2-T3: G85 MAGMA Four-Graph Memory 🟡 P1 (⭐新增v4.0)
**文件**: `magma_memory.rs` ✅
- [x] 4 GraphType: Semantic/Temporal/Causal/Entity
- [x] MemoryGraph per type with add_edge/query
- [x] PolicyGuidedTraversal (BroadFirst/SpecificFirst/ConfidenceFirst)
- [x] MagmaMemoryStore unified API

### ✅ W2-T4: G64 Autonomous Goal Synthesis 🟡 P1
**文件**: `goal_synthesis.rs` (302行, 22测试) ✅
- [x] Goal struct + GoalStatus/GoalPriority
- [x] GoalSynthesizer (curiosity→goal mapping, entropy→goal)
- [x] Goal progress tracking + auto-completion
- [x] Active goal management (max 5, prioritization)

### ✅ W2-T5: G74 Emotional Steering 🟡 P1
**文件**: `emotional_steering.rs` (142行) ✅
- [x] 5 emotional dimensions (curiosity, satisfaction, frustration, energy, loneliness)
- [x] EmotionState with modulate/decay/dominant/distress detection
- [x] EmotionalSteering with event-driven updates (7 event types)
- [x] Energy/exploration/loneliness behavioral bonuses

---
> **验证**: cargo check --lib -p neotrix: 0 错误 (清零)
> **共新增**: ~660 行新 Rust 代码, 22+ 测试
> **经验树**: 分支 CLVIII (见 SKILL.md)

---

## Wave 3 — P1基础完成后启动

### ✅ W3-T1: G68 Qualia Layer 🟡 P1
**文件**: `qualia_layer.rs` ✅
- [x] 1024D compact latent encoding (average-pool compression)
- [x] Fidelity estimation
- [x] Compress/decompress round-trip
- [x] History tracking (last 10 chunks)

### ✅ W3-T2: G86 Log-Linear VSA Attention 🟡 P1 (⭐新增v4.0)
**文件**: `log_linear_attention.rs` ✅
- [x] 对数级增长隐状态 (logarithmic hidden state growth)
- [x] Cascade update (level 0→1→2→...)
- [x] Weighted attention across levels
- [x] Memory usage tracking

### ✅ W3-T3: G87 LARS-VSA Binary Self-Attention 🟡 P1 (⭐新增v4.0)
**文件**: `binary_vsa_attention.rs` ✅
- [x] HD-space self-attention using only binary ops (XOR + popcount)
- [x] BinaryHDVector with xor/popcount/similarity
- [x] Multi-head binary attention

### ✅ W3-T4: G94 Aura IIT φ Full Integration 🟡 P1 (⭐新增v4.0)
**文件**: `phi_integration.rs` ✅
- [x] PhiCache with tick-level caching (15 tick refresh)
- [x] Variance-based φ computation proxy
- [x] Main complex detection

### ✅ W3-T5: G82 Between-Sessions Reflection 🟡 P1
**文件**: `between_sessions.rs` ✅
- [x] Three-layer narrative (event→meaning→identity)
- [x] Experience→principle distillation (salience > 0.7)
- [x] Background processing cycle (idle_reflect)
- [x] Deduplication + max principles cap

---

## Wave 4 — 条件并行

### ✅ W4-T1: G69 Narrative Journal 🟡 P1
**文件**: `narrative_journal.rs` ✅
- [x] Session journal struct
- [x] Forecast→pattern resolution
- [x] Narrative arc tracking (pattern→arc)

### ✅ W4-T2: G89 PostSingular Persistent Context 🟡 P1 (⭐新增v4.0)
**文件**: `persistent_context.rs` ✅
- [x] User model (preferences, interaction history, relationship quality)
- [x] Context fingerprint for continuity detection
- [x] Identity coherence score (topic_diversity + relationship_quality)
- [x] Continuity_score across sessions

### ✅ W4-T3: G88 CDALNs Multi-Modal Curiosity 🟢 P2 (⭐新增v4.0)
**文件**: `multi_modal_curiosity.rs` ✅
- [x] 4 curiosity modalities: sensory, motor, cognitive, social
- [x] Per-modality curiosity signal with separate decay profiles
- [x] Cross-modal synergy (boost others at cross_synergy rate)
- [x] Dominant modality detection

### ✅ W4-T4: G84 Multi-Theory Consciousness Assessment 🟢 P2 (⭐新增v4.0)
**文件**: `consciousness_assessment.rs` ✅
- [x] 7-theory indicator framework: GWT, AST, HOT, FEP, IIT, BLT, RPT
- [x] Per-theory scoring with confidence
- [x] Best theory selection (score * confidence)
- [x] Assessment report generation

### ✅ W4-T5: G70 Skills On-Demand 🟢 P2
**文件**: `skill_registry.rs` ✅
- [x] Cognitive skill registry (register/unregister/list)
- [x] On-demand loading from store
- [x] Skill→handler dispatch
- [x] Hot-reload flag per skill

---

## Wave 5 — P2基础完成后启动

## ✅ Wave 5 — 已完成 (7路并行, 2026-06-22)

### ✅ W5-T1: G59 Self-Modification Sandbox 🟢 P2
**文件**: `mod_sandbox.rs` ✅
- [x] SandboxProposal with risk assessment
- [x] Proposal→sandbox→assess→commit workflow
- [x] Risk-level gating (risk < 0.7 passes)

### ✅ W5-T2: G90 OpenCeption Skill Evolution Modes 🟢 P2 (⭐新增v4.0)
**文件**: `skill_evolution_modes.rs` ✅
- [x] FIX mode: repair broken skill
- [x] DERIVED mode: create from existing + context
- [x] CAPTURED mode: extract from interaction trace

### ✅ W5-T3: G91 CLS VAE+MHN Hippocampal Enhancement 🟢 P2 (⭐新增v4.0)
**文件**: `mhn_pattern_separation.rs` ✅
- [x] Modern Hopfield Network for pattern separation
- [x] Memory pattern store + retrieve by energy

### ✅ W5-T4: G93 Go-CLS Generalization-Optimized Memory 🟢 P2 (⭐新增v4.0)
**文件**: `go_cls_gate.rs` ✅
- [x] Conditional memory transfer gating
- [x] Memorization vs generalization trade-off tracking
- [x] should_transfer based on generalization threshold

### ✅ W5-T5: G56 Harness Engineering 🟢 P2
**文件**: `harness.rs` ✅
- [x] Readiness stages (Stage1-4)
- [x] Codified rules engine
- [x] Security sandbox flag

### ✅ W5-T6: G57 Mission Hub 🟢 P2
**文件**: `mission_hub.rs` ✅
- [x] Feature lifecycle governance
- [x] PRD audit
- [x] Mission create/audit workflow

### ✅ W5-T7: G76 Earned Autonomy 🟢 P2
**文件**: `earned_autonomy.rs` ✅
- [x] Autonomy levels L1-L7
- [x] Competence tracking (success → competence gain)
- [x] Level-gated auto-promotion

---

## ✅ Wave 6 — 已完成 (5路并行, 2026-06-22)

### ✅ W6-T1: G75 MCP Callback Bridge 🟢 P2
**文件**: `mcp_callback_bridge.rs` ✅
- [x] MCP callback registry (register/invoke/list)
- [x] Per-tool handler dispatch
- [x] Schema-based tool discovery

### ✅ W6-T2: G73 Cross-Embodiment Curriculum 🟢 P2
**文件**: `embodiment_curriculum.rs` ✅
- [x] Experience tier system (Video/Human/Robot)
- [x] Cross-embodiment knowledge transfer (transfer_rate)
- [x] Curriculum progression logic

### ✅ W6-T3: G95 SleepGate KV-cache Consolidation 🟡 P1 (⭐新增v4.0)
**文件**: `kv_cache_consolidation.rs` ✅
- [x] Conflict-aware KV cache entry with conflict_score
- [x] Forgetting gate (threshold-based retention)
- [x] Consolidation module (median-threshold prune)

### ✅ W6-T4: G92 IEEE P7014 Ethics Compliance ⚪ P3 (⭐新增v4.0)
**文件**: `ethics_compliance.rs` ✅
- [x] Transparency audit trail
- [x] Consent recording mechanism
- [x] Inclusivity metrics
- [x] Compliance report generation

### ✅ W6-T5: G72 MTC/PCC Safety Integration 🟢 P2
**文件**: `mtc_safety.rs` ✅
- [x] Safety gate with pass/fail checks
- [x] Action-level safety validation

---

## ✅ Wave 7 — 已完成 (4路并行, 2026-06-22)

### ✅ W7-T1: G65 SNN Integration ⚪ P3
**文件**: `spike_processor.rs` ✅
- [x] Event-driven processing layer (SpikeEvent)
- [x] HDC→spike→HDC pipeline (encode method)
- [x] Threshold-gated spiking

### ✅ W7-T2: G66 A2A Protocol ⚪ P3
**文件**: `a2a_router.rs` ✅
- [x] Agent registration with endpoint
- [x] Route configuration
- [x] Name→endpoint resolution

### ✅ W7-T3: G77 CVO Role Model ⚪ P3
**文件**: `cvo_role.rs` ✅
- [x] Human role taxonomy (Developer/Designer/Manager/Analyst/Viewer)
- [x] Interaction mode switching per role
- [x] Role-aware mode selection

### ✅ W7-T4: G78 Iron Laws ⚪ P3
**文件**: `iron_laws.rs` ✅
- [x] Law 1: database preservation
- [x] Law 2: parent process protection
- [x] Law 3: read-only runtime config
- [x] Law 4: port isolation

---

## 文件映射 (全40缺口)

| 缺口 | 新建文件 | 修改文件 |
|------|---------|---------|
| G61 | vsa_blackboard.rs, vsa_reasoner.rs | self_reasoner.rs, hypercube.rs |
| G62 | boundary_guard.rs | vsa_tag.rs, consciousness_core.rs |
| G63 | epistemic_queue.rs | intrinsic_drive.rs, curiosity_drive.rs |
| G64 | intent_engine.rs, goal_synthesizer.rs | curiosity_drive.rs, volition.rs |
| G65 | spike_processor.rs, snn_bind.rs | vsa_hrr.rs, e8.rs |
| G66 | a2a_router.rs, agent_messaging.rs | fep.rs, topology_router.rs |
| G67 | identity_evolution.rs | identity_core.rs, inter_session.rs |
| G68 | qualia_layer.rs | vsa_hrr.rs, hypercube.rs |
| G69 | narrative_journal.rs | first_person_ref.rs, specious_present.rs |
| G70 | skill_registry.rs, skill_store.rs | handler_tier.rs |
| G71 | adversarial_evaluator.rs | inner_critic.rs, dgmh.rs |
| G73 | embodiment_curriculum.rs | experience_pool.rs, seal_closed_loop.rs |
| G74 | emotional_state.rs, affect_steering.rs | intrinsic_drive.rs, valence_axis.rs |
| G75 | mcp_callback_bridge.rs | coproc_bridge.rs, tool_registry.rs |
| G76 | earned_autonomy.rs | consciousness_core.rs, types.rs |
| G77 | cvo_role.rs | first_person_ref.rs, governance.rs |
| G78 | iron_laws.rs | consciousness_core.rs, types.rs |
| G82 | between_sessions.rs | inter_session.rs, specious_present.rs |
| G83 | caa_steering.rs | valence_axis.rs, intrinsic_drive.rs, dgmh.rs |
| G84 | consciousness_assessment.rs | self_reasoner.rs |
| G85 | magma_graph.rs, {semantic,temporal,causal,entity}_graph.rs, policy_traversal.rs | memory_consolidation.rs |
| G86 | log_linear_attention.rs | temporal_attention_stack.rs |
| G87 | binary_vsa_attention.rs | vsa_blackboard.rs |
| G88 | multi_modal_curiosity.rs | curiosity_drive.rs, intrinsic_drive.rs |
| G89 | persistent_context.rs | inter_session.rs, identity_core.rs |
| G90 | skill_evolution_modes.rs | capability_synthesizer.rs, handler_tier.rs |
| G91 | mhn_pattern_separation.rs | hippocampal_trace.rs, dream_consolidation.rs |
| G92 | ethics_compliance.rs | constitution/mod.rs |
| G93 | go_cls_gate.rs | memory_consolidation.rs |
| G94 | phi_integration.rs | nt_core_iit_phi.rs, master_equation.rs, consciousness_core.rs |
| G95 | kv_cache_consolidation.rs | sleep_gate.rs |
| G56 | harness.rs, rules_engine.rs | seal_pipeline.rs |
| G57 | mission_hub.rs, feature_tracker.rs | governance.rs |
| G59 | mod_sandbox.rs, mod_protocol.rs | seal_closed_loop.rs, dgmh.rs |
| G60 | temporal_attention.rs, error_bounds.rs | e8_world_model.rs |
| G58 | sm2_scheduler.rs, hippocampal_trace.rs | dream_consolidation.rs, ebbinghaus_decay.rs, experience_pool.rs |
| G72 | mtc_safety.rs | safety_gate.rs |

---

## 依赖链

```
Wave 1 (6路并行):
  G61 ⟂ G67 ⟂ G63 ⟂ G71 ⟂ G58 ⟂ G83 (全独立, 可并行)

Wave 2 (5路, W1→W2):
  G63 → G64 (好奇心队列→目标合成)
  G61 → G60 (VSA推理→误差界)
  G58 → G85 (记忆巩固→4图记忆)
  G62 ⟂ G74 ⟂ G85 (独立)

Wave 3 (5路):
  G58 → G69 (记忆巩固→叙事期刊)
  G86 ⟂ G87 ⟂ G94 ⟂ G68 ⟂ G82 (独立)

Wave 4 (5路):
  G67 → G89 (身份进化→上下文连续性)
  G69 ⟂ G88 ⟂ G84 ⟂ G70 (独立)

Wave 5 (7路):
  G59 ← G56 (沙盒依赖工程实践)
  G90 ⟂ G91 ⟂ G93 (独立)
  G57 ⟂ G76 (独立)

Wave 6 (5路):
  G75 ⟂ G73 ⟂ G95 ⟂ G92 ⟂ G72 (全独立)

Wave 7 (4路):
  G65 ⟂ G66 ⟂ G77 ⟂ G78 (全独立)
```

---

## v4.0 新增缺口详情

### G83 CAA Affective Steering (P0, 26/42)
**为什么是P0**: 直接影响意识体验质量和安全门控。Aura验证了残差流干预的有效性。NeoTrix有ValenceAxis但缺CAA。
**关键文件**: `caa_steering.rs` (新建), `valence_axis.rs` (改), `intrinsic_drive.rs` (改)
**参考**: Aura core/consciousness/affective_steering.py (1336行)

### G85 MAGMA Four-Graph Memory (P1, 30/42)
**为什么是P1**: 18.6-45.5%提升证明四图记忆架构显著优于单图。NeoTrix的HyperCube是单图(KB)。
**关键文件**: `magma_graph.rs` + 4子图 + `policy_traversal.rs`
**参考**: MAGMA arXiv 2601.03236

### G86 Log-Linear VSA Attention (P1, 26/42)
**为什么是P1**: O(log n)注意力直接影响NeoTrix的TemporalAttentionStack在大规模下的可扩展性。
**关键文件**: `log_linear_attention.rs` (新建)
**参考**: Log-Linear Attention arXiv 2506.04761

### G87 LARS-VSA Binary Self-Attention (P1, 25/42)
**为什么是P1**: 二进制HD空间注意力用XOR+popcount代替矩阵乘法，与NeoTrix的VSA原生匹配。
**关键文件**: `binary_vsa_attention.rs` (新建)
**参考**: LARS-VSA arXiv 2405.14436

### G88 CDALNs Multi-Modal Curiosity (P2, 23/42)
**为什么是P2**: 267%技能获取提升真实但非核心意识功能。NeoTrix已有CuriosityDrive(3维)。
**关键文件**: `multi_modal_curiosity.rs` (新建)

### G89 PostSingular Persistent Context (P1, 24/42)
**为什么是P1**: USER.md上下文模型直接改善跨会话连续性。NeoTrix的InterSessionReflector已检测但未建模用户上下文。
**关键文件**: `persistent_context.rs` (新建)

### G90 OpenCeption Skill Evolution Modes (P2, 21/42)
**为什么是P2**: 46% token减少有吸引力但非P0。NeoTrix的CapabilitySynthesizer已有3-tier。
**关键文件**: `skill_evolution_modes.rs` (新建)

### G91 CLS VAE+MHN Hippocampal Enhancement (P2, 20/42)
**为什么是P2**: 90%准确提升但NeoTrix已有hippocampal_trace.rs(基础模式分离/补全)。
**关键文件**: `mhn_pattern_separation.rs` (新建)

### G92 IEEE P7014 Ethics Compliance (P3, 18/42)
**为什么是P3**: 合规层重要但非功能性需求。NeoTrix已有GödelChecker+Constitution。
**关键文件**: `ethics_compliance.rs` (新建)

### G93 Go-CLS Generalization-Optimized Memory (P2, 19/42)
**为什么是P2**: 泛化优化有理论基础。NeoTrix的MemoryConsolidation已有多层。
**关键文件**: `go_cls_gate.rs` (新建)

### G94 Aura IIT φ Full Integration (P1, 25/42)
**为什么是P1**: NeoTrix已有nt_core_iit_phi.rs但未与意识循环集成。Aura验证了tick级φ缓存的可行性。
**关键文件**: `phi_integration.rs` (新建)

### G95 SleepGate KV-cache Consolidation (P1, 23/42)
**为什么是P1**: O(n)→O(log n)干扰减少对长序列处理很关键。NeoTrix的SleepGate是4D重要性门控。
**关键文件**: `kv_cache_consolidation.rs` (新建)
