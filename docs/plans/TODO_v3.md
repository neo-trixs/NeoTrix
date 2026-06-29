# NeoTrix 进化补缺任务清单 v3.1

> 基于四轮全景查漏+深度源分析（CXLVIII.v2） — 3URL抓取+25项目 → 20缺口 G56-G75
> 优先级: P0 > P1 > P2 > P3
> 调度: Wave 1-5 并行约束下的最优顺序

---

## Wave 1 — 立即启动 (5路并行)

### W1-T1: G61 VSA-Only Reasoning 🔴 P0
**来源**: PRISM (Artaeon), Dragonfly VSA, Victor  
**文件**: 10+ files  
**估算**: 3-5 sessions  

子任务:
- [ ] 实现 Blackboard VSA 推理架构（类比 E8 Hexagram 但不依赖 LLM）
- [ ] VSA 类比推理: bind/unbind pattern matching
- [ ] VSA 因果推理: trajectory composition
- [ ] VSA 多跳推理: chain binding
- [ ] VSA 矛盾检测: similarity threshold
- [ ] Zero-parameter 推理基准测试
- [ ] Qualia-as-truth: 1024D compact latent 集成

### W1-T2: G67 Darwinian Identity Evolution 🔴 P0
**来源**: soul.py (menonpg)  
**文件**: 5 files  
**估算**: 2-3 sessions  

子任务:
- [ ] Identity mutation operators（trait drift, value shift, weight perturbation）
- [ ] Selection pressure（session success→reinforce, failure→mutate）
- [ ] Session-based identity evolution cycle
- [ ] Identity versioning + rollback

### W1-T3: G63 Epistemic Gap Queue 🔴 P0
**来源**: Nūr (balfiky/nur)  
**文件**: 3 files  
**估算**: 1 session  

子任务:
- [ ] GapType enum: Contradiction, LowConfidence, DriveGap, KnowledgeMissing
- [ ] EpistemicQueue struct with priority ranking
- [ ] Resolution tracking
- [ ] Curiosity drive integration (current scalar → structured queue)

### W1-T4: G58 Dream Consolidation 🟡 P1
**来源**: mark-improving-agent (yun520-1)  
**文件**: 4 files  
**估算**: 2 sessions  

子任务:
- [ ] Ebbinghaus forgetting curve (30-day half-life)
- [ ] SM-2 spaced repetition scheduler
- [ ] Sleep-mode consolidation cycle
- [ ] Hippocampal trace structure（pattern separation/completion）

### W1-T5: G71 Adversarial Evaluator 🔴 P0
**来源**: PaulDuvall/ai-development-patterns  
**文件**: 4 files  
**估算**: 2 sessions  

子任务:
- [ ] 从 InnerCritic 分离独立 JudgeAgent trait
- [ ] Cross-model divergence 作为评估信号
- [ ] Adversarial pressure testing 框架
- [ ] Judge agent fallback（同模型作为降级）

---

## Wave 2 — W1完成后/W1中期启动 (条件并行)

### W2-T1: G62 Identity Boundary Hooks 🔴 P0
**来源**: Instar (gfrankgva), Ouroboros (joi-lab)  
**文件**: 8 files  
**估算**: 2-3 sessions  

子任务:
- [ ] Constitution trait: pre-action identity verification
- [ ] System boundary guard layer
- [ ] Self vs World tag enforcement
- [ ] Violation tracking

### W2-T2: G60 Formal Error Bounds 🔴 P0
**来源**: Kairos (arXiv 2606.16533)  
**文件**: 4 files  
**估算**: 3-4 sessions  

子任务:
- [ ] Hybrid Linear Temporal Attention（sliding + dilated + gated linear）
- [ ] Error bound derivation for E8WorldModel
- [ ] Long-horizon prediction validation
- [ ] Error accumulation metrics

### W2-T3: G68 Qualia Layer 🟡 P1
**来源**: Dragonfly VSA  
**文件**: 5 files  
**估算**: 2 sessions  

子任务:
- [ ] 1024D compact latent encoding
- [ ] ≥97% fidelity compression/decompression
- [ ] Qualia vs workspace layer separation
- [ ] Transformation functions (qualia ↔ workspace)

### W2-T4: G64 Autonomous Goal Synthesis 🟡 P1
**来源**: TEQUMSA (HF)  
**文件**: 6 files  
**估算**: 2-3 sessions  

子任务:
- [ ] Entropy→goal mapping
- [ ] Intent engine（generate, prioritize, schedule）
- [ ] Curiosity→concrete goal pipeline
- [ ] Goal progress tracking

### W2-T5: G74 Emotional Steering 🟡 P1
**来源**: genesis-agent, KernelBot, mark-improving-agent  
**文件**: 5 files  
**估算**: 2 sessions  

子任务:
- [ ] 5 emotional dimensions (curiosity, satisfaction, frustration, energy, loneliness)
- [ ] Frustration→model escalation
- [ ] Energy→rest suggestion
- [ ] Curiosity→exploration priority
- [ ] Emotional state persistence

---

## Wave 3 — P1 基础完成后启动

### W3-T1: G69 Narrative Journal 🟡 P1
**来源**: LIFE (TeamSafeAI), Nūr  
**文件**: 3 files  
**估算**: 1 session  

子任务:
- [ ] Session journal struct
- [ ] Forecast→pattern resolution
- [ ] Narrative arc tracking

### W3-T2: G70 Skills On-Demand 🟢 P2
**来源**: clowder-ai (zts212653)  
**文件**: 5 files  
**估算**: 2 sessions  

子任务:
- [ ] Cognitive skill registry
- [ ] On-demand loading from store
- [ ] Skill→handler dispatch
- [ ] Hot-reload

### W3-T3: G59 Self-Modification Sandbox 🟢 P2
**来源**: Autogenesis (DVampire), GBase, Ouroboros  
**文件**: 6 files  
**估算**: 3-4 sessions  

子任务:
- [ ] Resource protocol (RSPL-style)
- [ ] Proposal→sandbox→assess→commit
- [ ] Constitution review queue
- [ ] Versioned rollback

### W3-T4: G82 Between-Sessions Reflection 🟡 P1
**来源**: atman, ouroboros, KernelBot  
**文件**: 5 files  
**估算**: 2 sessions  

子任务:
- [ ] Background processing cycle（idle trigger）
- [ ] Experience→principle distillation
- [ ] Self-observation parallel to task execution
- [ ] Three-layer narrative（event→meaning→identity）

### W3-T5: G73 Cross-Embodiment Curriculum 🟢 P2
**来源**: Kairos (arXiv 2606.16533)  
**文件**: 4 files  
**估算**: 2 sessions  

子任务:
- [ ] Experience tier system（video/human/robot）
- [ ] Cross-embodiment knowledge transfer
- [ ] Curriculum progression logic
- [ ] Source diversity metrics

---

## Wave 4 — P2 基础完成后启动

### W4-T1: G56 Harness Engineering 🟢 P2
**来源**: PaulDuvall/ai-development-patterns  
**文件**: 4 files  
**估算**: 2-3 sessions  

子任务:
- [ ] Readiness Assessment stages
- [ ] Codified Rules engine
- [ ] Security Sandbox
- [ ] Feedforward+feedback controls

### W4-T2: G57 Mission Hub 🟢 P2
**来源**: clowder-ai (zts212653)  
**文件**: 4 files  
**估算**: 2 sessions  

子任务:
- [ ] Feature lifecycle governance
- [ ] PRD audit
- [ ] SOP visualization
- [ ] Bulletin board

### W4-T3: G76 Earned Autonomy 🟢 P2
**来源**: GENesis-AGI (WingedGuardian)  
**文件**: 4 files  
**估算**: 2 sessions  

子任务:
- [ ] Autonomy levels L1-L7
- [ ] Competence tracking
- [ ] Level-gated permissions
- [ ] Autonomy audit log

### W4-T4: G75 MCP Callback Bridge 🟢 P2
**来源**: clowder-ai, persistent-agent-runtime  
**文件**: 4 files  
**估算**: 2 sessions  

子任务:
- [ ] MCP callback bridge trait
- [ ] Cross-model tool sharing
- [ ] Capability discovery
- [ ] Unified tool registry

---

## Wave 5 — 后期并行 ⚪

---

### W5-T1: G65 SNN Integration ⚪ P3
**来源**: Hydra (Medium), TEQUMSA  
**文件**: 12+ files  
**估算**: 4-6 sessions  

子任务:
- [ ] Event-driven processing layer
- [ ] HDC→spike→HDC pipeline
- [ ] Low-power mode
- [ ] Benchmark

### W5-T2: G66 A2A Protocol ⚪ P3
**来源**: clowder-ai (zts212653), Google A2A Protocol  
**文件**: 4 files  
**估算**: 2 sessions  

子任务:
- [ ] @mention routing
- [ ] Thread isolation
- [ ] Shared memory access control
- [ ] Agent card discovery

### W5-T3: G77 CVO Role Model ⚪ P3
**来源**: clowder-ai (zts212653)  
**文件**: 3 files  
**估算**: 1 session  

子任务:
- [ ] Human role taxonomy
- [ ] Interaction mode switching
- [ ] Culture shaping feedback loop

### W5-T4: G78 Iron Laws ⚪ P3
**来源**: clowder-ai (zts212653)  
**文件**: 3 files  
**估算**: 1 session  

子任务:
- [ ] Law 1: database preservation
- [ ] Law 2: parent process protection
- [ ] Law 3: read-only runtime config
- [ ] Law 4: port isolation

---

## 优先级速查

| 缺口 | 优先级 | Score | 估算 | 文件 | 并行性 |
|------|--------|-------|------|------|--------|
| G61 VSA-Only Reasoning | P0 | 38/42 | 3-5s | 10+ | W1, 独立 |
| G67 Darwinian Identity | P0 | 34/42 | 2-3s | 5 | W1, 独立 |
| G62 Boundary Hooks | P0 | 33/42 | 2-3s | 8 | W2, 独立 |
| G63 Epistemic Queue | P0 | 32/42 | 1s | 3 | W1, 独立 |
| G60 Formal Error Bounds | P0 | 31/42 | 3-4s | 4 | W2, 需G61部分 |
| G71 Adversarial Evaluator | P0 | 27/42 | 2s | 4 | W1, 独立 |
| G58 Dream Consolidation | P1 | 28/42 | 2s | 4 | W1, 独立 |
| G68 Qualia Layer | P1 | 27/42 | 2s | 5 | W3, 独立 |
| G64 Auto Goal Synthesis | P1 | 26/42 | 2-3s | 6 | W2, 需G63部分 |
| G74 Emotional Steering | P1 | 26/42 | 2s | 5 | W2, 独立 |
| G73 Cross-Embodiment Curriculum | P2 | 25/42 | 2s | 4 | W3, 独立 |
| G76 Earned Autonomy | P2 | 24/42 | 2s | 4 | W4, 需G59 |
| G69 Narrative Journal | P1 | 24/42 | 1s | 3 | W3, 需G58部分 |
| G82 Between-Sessions Reflection | P1 | 23/42 | 2s | 5 | W3, 需G71部分 |
| G75 MCP Callback Bridge | P2 | 22/42 | 2s | 4 | W4, 独立 |
| G56 Harness Engineering | P2 | 22/42 | 2-3s | 4 | W4, 独立 |
| G70 Skills On-Demand | P2 | 21/42 | 2s | 5 | W3, 独立 |
| G57 Mission Hub | P2 | 20/42 | 2s | 4 | W4, 独立 |
| G59 Mod Sandbox | P2 | 19/42 | 3-4s | 6 | W3, 需G62+G56 |
| G77 CVO Role Model | P3 | 20/42 | 1s | 3 | W5, 独立 |
| G78 Iron Laws | P3 | 19/42 | 1s | 3 | W5, 独立 |
| G65 SNN Integration | P3 | 15/42 | 4-6s | 12+ | W5, 需G61 |
| G66 A2A Protocol | P3 | 14/42 | 2s | 4 | W5, 独立 |

---

## 文件映射（新文件/修改文件）

| 缺口 | 新建文件 | 修改文件 |
|------|---------|---------|
| G61 | `vsa_blackboard.rs`, `vsa_reasoner.rs` | `self_reasoner.rs`, `hypercube.rs` |
| G67 | `identity_evolution.rs` | `identity_core.rs`, `inter_session.rs` |
| G62 | `boundary_guard.rs` | `vsa_tag.rs`, `consciousness_core.rs` |
| G63 | `epistemic_queue.rs` | `intrinsic_drive.rs`, `curiosity_drive.rs` |
| G60 | `temporal_attention.rs`, `error_bounds.rs` | `e8_world_model.rs` |
| G58 | `dream_consolidation.rs`, `forgetting_curve.rs` | `experience_pool.rs` |
| G68 | `qualia_layer.rs` | `vsa_hrr.rs`, `hypercube.rs` |
| G64 | `intent_engine.rs`, `goal_synthesizer.rs` | `curiosity_drive.rs`, `volition.rs` |
| G69 | `narrative_journal.rs` | `first_person_ref.rs`, `specious_present.rs` |
| G70 | `skill_registry.rs`, `skill_store.rs` | `handler_tier.rs` |
| G56 | `harness.rs`, `rules_engine.rs` | `seal_pipeline.rs` |
| G57 | `mission_hub.rs`, `feature_tracker.rs` | `governance.rs` |
| G59 | `mod_sandbox.rs`, `mod_protocol.rs` | `seal_closed_loop.rs`, `dgmh.rs` |
| G65 | `spike_processor.rs`, `snn_bind.rs` | `vsa_hrr.rs`, `e8.rs` |
| G66 | `a2a_router.rs`, `agent_messaging.rs` | `fep.rs`, `topology_router.rs` |
| G71 | `adversarial_evaluator.rs` | `inner_critic.rs`, `dgmh.rs` |
| G73 | `embodiment_curriculum.rs` | `experience_pool.rs`, `seal_closed_loop.rs` |
| G74 | `emotional_state.rs`, `affect_steering.rs` | `intrinsic_drive.rs`, `valence_axis.rs` |
| G75 | `mcp_callback_bridge.rs` | `coprocessor_bridge.rs`, `tool_registry.rs` |
| G76 | `earned_autonomy.rs` | `consciousness_core.rs`, `types.rs` |
| G77 | `cvo_role.rs` | `first_person_ref.rs`, `governance.rs` |
| G78 | `iron_laws.rs` | `consciousness_core.rs`, `types.rs` |
| G82 | `between_sessions.rs` | `inter_session.rs`, `specious_present.rs` |
