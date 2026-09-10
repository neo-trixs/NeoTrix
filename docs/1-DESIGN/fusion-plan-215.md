# Fusion Plan 215 — 聚焦冗余 + 扁平缺陷 + 跨域错位

> 生成时间: 2026-09-10 | 基于代码静态分析 + CONTEXT.md 架构定义

---

## 一、聚焦冗余清理清单

### 1.1 Registry 泛滥 (77 个 Registry)

核心问题：`core/` 下存在 **77 个独立 Registry struct**，职责高度重叠。

| 冗余 Registry | 所在文件 | 建议 |
|---------------|---------|------|
| `SubAgentRegistry` | `nt_core_subagent.rs:275` | 合并入 `CapabilityRegistry` (nt_core_capability/mod.rs:598) |
| `MethodRegistry` | `nt_core_reasoning.rs:67` | 合并入 `ReasoningStrategyRegistry` (nt_core_self/reasoning_strategy.rs:152) |
| `AgentPatternRegistry` | `nt_core_agent_patterns.rs:52` | 合并入 `CapabilityRegistry` |
| `ModelSkillRegistry` | `nt_core_model_skills.rs:22` | 合并入 `SkillRegistry` (l7_capability/skill_acquire.rs:143) |
| `GameRegistry` | `nt_game/env.rs:206` | 保留（游戏域专用） |
| `GameToolRegistry` | `nt_game/mcp.rs:407` | 保留（游戏域专用） |
| `CrystalRegistry` | `nt_core_self/skill_crystal.rs:52` | 合并入 `SkillRegistry` |
| `ResourceRegistry` | `nt_core_resource_pool/resource_registry.rs:32` | 保留（资源池专用） |

**清理目标**: 77 → ~50 个 Registry（合并 27 个）

### 1.2 Bridge 泛滥 (39 个 Bridge)

| 冗余 Bridge | 所在文件 | 建议 |
|-------------|---------|------|
| `EchoPrmBridge` | `nt_core_echo_terminal.rs:403` | 合并入 `MetaGoalBridge` (nt_core_meta/planner.rs:271) |
| `E8AbductionBridge` | `nt_core_e8/e8_abduction_bridge.rs:9` | 合并入 `E8EwhrBridge` (ewhr_bridge.rs:4) |
| `VisionBridge` | `nt_core_e8/nt_multimodal.rs:178` | 合并入 `E8EwhrBridge` |

**清理目标**: 39 → ~32 个 Bridge（合并 7 个）

### 1.3 `pub fn new()` 热点文件

| 文件 | new() 数量 | 建议 |
|------|-----------|------|
| `seal_core/self_iterating/pipeline.rs` | 35 | 拆分为 `StageFactory` + `PipelineConfig` |
| `nt_world_media_source/infra/deployment/mod.rs` | 13 | 统一为 `DeploymentBuilder` |
| `nt_world_media_source/infra/observability/mod.rs` | 7 | 统一为 `ObservabilityBuilder` |
| `seal_core/self_iterating/goal_contract.rs` | 7 | 拆分为 `GoalFactory` |
| `nt_world_media_source/infra/ai/mod.rs` | 6 | 统一为 `AiProviderFactory` |

**清理目标**: 前 5 热点文件从 68 → ~35 个 new()

### 1.4 重复能力模式

| 重复模式 | 涉及模块 | 建议 |
|---------|---------|------|
| 6 个 EmotionEngine 变体 | `nt_core_self/emotion_state.rs` + `nt_feel/` | 统一为 NT-FEEL 单一事实源 |
| 3 个 SymbolicRegressionEngine | `nt_core_dao_engine.rs` + `nt_core_forecast.rs` | 统一为预测引擎 |
| 2 个 SchedulerEngine | `nt_core_scheduler/engine.rs` + `nt_act/` 调度 | 统一到 NT-ACT |

---

## 二、扁平缺陷修复清单

### 2.1 生产代码中的 `.unwrap()` (关键级)

| 文件:行 | 上下文 | 风险等级 | 修复建议 |
|---------|--------|---------|---------|
| `core/nt_core_plan/mod.rs:508` | `plan.steps.first_mut().unwrap()` | **高** | 改为 `ok_or_else(NtError::EmptySteps)?` |
| `core/nt_core_hcube/bayesian_experiment.rs:377-378` | `partial_cmp().unwrap()` + `.unwrap()` | **高** | 改为 `unwrap_or(Ordering::Equal)` + `ok_or_else()?` |

### 2.2 测试代码中的 `.unwrap()` (低风险)

以下为测试代码，可接受，但建议统一使用 `.expect("reason")`:

- `core/nt_core_state.rs:117-147` — 测试 DB 操作
- `core/nt_core_echo_terminal.rs:585-639` — 测试信号量
- `core/nt_core_hcube/qfhrr_vsa.rs` — 测试 VSA 操作
- `core/nt_core_hcube/fhrr_vsa.rs` — 测试 VSA 操作

### 2.3 跨层依赖违规 (致命级)

| 违规位置 | 违规类型 | 修复建议 |
|---------|---------|---------|
| `l2_perception/nt_world/nt_world_urlhaus.rs` 直接引用 `l1_action::nt_memory::nt_memory_kb` | L2 → L1 反向依赖 | 通过 KB trait 接口解耦 |
| `l2_perception/nt_world/nt_world_usgs.rs` 直接引用 `l1_action::nt_memory::nt_memory_kb` | L2 → L1 反向依赖 | 通过 KB trait 接口解耦 |
| `l1_action/nt_act/nt_act_orchestrator/critic.rs` 引用 `l2_perception::nt_world::nt_world_model` | L1 → L2 反向依赖 | 通过 PerceptionBridge 解耦 |
| `l5_cognition/nt_mind_background_loop/` 引用 `l3_embodiment::nt_shield` | L5 → L3 跨层 | 通过 NT-SHIELD trait 解耦 |
| `l5_cognition/nt_mind_background_loop/` 引用 `neotrix::nt_shield_stealth_net` | L5 → neotrix 命名空间 | 通过 NT-SHIELD trait 解耦 |

### 2.4 `nt_core_self` 重复放置 (致命级)

- `l2_perception/nt_core_self/` 存在副本（含 `character_interaction.rs`, `dynamic_params.rs` 等）
- `core/nt_core_self/` 是主副本
- **修复**: 删除 `l2_perception/nt_core_self/` 副本，保留 `core/nt_core_self/` 为单一事实源

---

## 三、跨域错位重构清单

### 3.1 L1-L2 层间错位

| 错位模块 | 当前位置 | 目标位置 | 理由 |
|---------|---------|---------|------|
| `nt_world_model` (TaskType) | `l2_perception/nt_world/` | `l5_cognition/nt_core/` | TaskType 是认知决策类型，非感知 |
| `nt_memory_spatial` | `l1_action/nt_memory/` | `l2_perception/nt_sense/` | 空间记忆属于感知层 |
| `nt_infra_semantic_router` | `l1_action/` | `l2_perception/nt_sense/` | 语义路由属于感知匹配 |

### 3.2 L3-L5 层间错位

| 错位模块 | 当前位置 | 目标位置 | 理由 |
|---------|---------|---------|------|
| `nt_shield::browser_security` | `l3_embodiment/nt_shield/` | `l6_meta/nt_governance/` | 安全审计属于治理层 |
| `nt_shield::check_registry` | `l3_embodiment/nt_shield/` | `l6_meta/nt_governance/` | 检查注册属于治理层 |

### 3.3 neotrix 命名空间错位

| 错位模块 | 当前位置 | 目标位置 | 理由 |
|---------|---------|---------|------|
| `nt_shield_stealth_net` | `neotrix/` | `l3_embodiment/nt_shield/` | 应归入 NT-SHIELD 域 |
| `nt_world_model` | `neotrix/` | `l2_perception/nt_world/` 或 `l5_cognition/nt_core/` | 应归入对应域 |
| `nt_consciousness_core` | `neotrix/` | `l5_cognition/nt_core/` | 意识核心属于认知层 |
| `nt_feel` (guardrail_pipeline) | `neotrix/nt_consciousness_core/` | `l4_emotion/nt_feel/` | 情感守卫属于情感层 |

---

## 四、新能力吸收清单 (从外部研究)

### 4.1 核心公理 (3)

| 能力 | 来源 | NeoTrix 映射 | 优先级 | 吸收位置 |
|------|------|-------------|--------|---------|
| **Cost-Aware Routing** | Spotify Portal Shunt | GWT salience + token 成本权重 | P0 | `nt_core_gwt/salience.rs` |
| **Context as Scarce Resource** | KVMem | KV paged virtualization | P0 | `kv_cache_optimizer.rs` 扩展 |
| **Skill as Production Template** | Easel 112★ | SKILL-SPEC.md 契约 | P1 | NT-ACT skill nodes |

### 4.2 跨源模式 (5)

| 模式 | 定义 | NeoTrix 映射 | 优先级 |
|------|------|-------------|--------|
| P1: Model Routing / Delegation | 任务路由到最便宜的能力模型 | GWT salience + cost weight | P0 |
| P2: Isolation-per-Task | 每个任务独立上下文/状态 | Worktree isolation + paged memory | P1 |
| P3: Profile-Driven Adaptation | 持久化 profile 跨会话塑造行为 | SelfModel 扩展 | P2 |
| P4: Ordered Backend Fallback | 单接口 + 有序 fallback | Ordered Backend Router | P1 |
| P5: Skill as Reusable Template | 技能是可组合原子 + 严格接口 | SKILL-SPEC.md 契约 | P1 |

### 4.3 Easel 模式 (5 可迁移)

| 模式 | Easel 实现 | NeoTrix 映射 | 优先级 |
|------|-----------|-------------|--------|
| Skill Interface Contract | SKILL.md (<200行) + references/ + scripts/ + tests/ | NT-ACT skill nodes | P0 |
| Manifest-as-Thin-Index | `.easel.json` summary + outputs[] | SEAL pipeline inter-stage | P1 |
| Profile-Driven Continuity | 6 维 profile | SelfModel + NT-MEMORY | P2 |
| Prompt Stack Layering | SOUL→AGENTS→CONTEXT→SKILL | GWT attention routing | P2 |
| Content Guard Taxonomy | BLOCK (fail-closed) vs WARN (soft) | NT-SHIELD egress guard | P1 |

### 4.4 KVMem 关键洞察 (4)

| 概念 | 定义 | NeoTrix 集成 |
|------|------|-------------|
| Attention-Space Index | Block-level Mean-K 向量 (32-token blocks) | GWT refinement: model-native scoring |
| Paged KV Virtualization | GPU→Host→NVMe 分层 KV 存储 | kv_cache_optimizer.rs 扩展 |
| Step-Level Scheduling | 每 agent step 更新 working set | ConsciousnessTree cycle boundary |
| Delta Reuse | Retained/Incoming/Outgoing 分解 | experience-tree lazy branch loading |

---

## 五、执行优先级

| 阶段 | 任务 | 影响范围 | 风险 |
|------|------|---------|------|
| **Phase 0** | 删除 `l2_perception/nt_core_self/` 副本 | 消除致命级重复 | 低 |
| **Phase 0** | 修复 `nt_core_plan/mod.rs:508` unwrap | 消除 panic 风险 | 低 |
| **Phase 1** | 合并 27 个 Registry | 聚焦冗余清理 | 中 |
| **Phase 1** | 合并 7 个 Bridge | 聚焦冗余清理 | 中 |
| **Phase 2** | 修复 5 处跨层依赖违规 | 扁平缺陷 | 中 |
| **Phase 2** | 移动 3 个错位模块 | 跨域错位 | 高 |
| **Phase 3** | 吸收 Cost-Aware Routing | 新能力 | 高 |
| **Phase 3** | 吸收 KVMem paged KV | 新能力 | 高 |
| **Phase 4** | 吸收 Easel Skill Contract | 新能力 | 中 |

---

## 六、度量基线

| 指标 | 当前值 | 目标值 |
|------|--------|--------|
| Registry 总数 | 77 | ≤50 |
| Bridge 总数 | 39 | ≤32 |
| Engine 总数 | 127 | ≤110 |
| 生产 unwrap() | 2 | 0 |
| 跨层依赖违规 | 5 | 0 |
| 重复模块放置 | 2 | 0 |
| l2_perception 副本 | 1 (nt_core_self) | 0 |
