# Phase 36 — World Model 接线阶段（DeepMind AGI→ASI 启示融合版）

> 日期: 2026-06-14 | 状态: 规划中
>
> 核心理念：NeoTrix 同时走在 DeepMind AGI→ASI 报告中的 **路径③（递归自改进）** 和 **路径④（多智能体集体）** 上，这是四条路径中最激进的两条。世界模型接线是解锁路径③加速的关键前置依赖。

## 文献启示

### arXiv:2606.12683 "From AGI to ASI" (DeepMind, Jun 2026)

14 位 DeepMind 研究者（Shane Legg, Marcus Hutter, Laurent Orseau 等）定义了从 AGI→ASI 的框架：

**对 NeoTrix 的 5 个核心启示**:

1. **数字智能的优势随计算规模放大**（Table 1）：I/O 速度、内部处理速度、工作记忆、无损复制、高带宽经验共享。NeoTrix 的 VSA 超立方体 + NTSSEG 存储正是这些优势的实现载体。

2. **递归自改进会产生双曲增长潜力**（路径③）：论文指出"如果 AI 系统能加速 AI 研究，结果可能是超指数增长"。NeoTrix 的 SEAL/DGM-H/SelfEvolution 是目前代码库中实现此路径的基础设施，但**世界模型是递归改进的输入信号**——没有世界模型预测误差，自我编辑就没有梯度方向。

3. **多智能体集体智能**（路径④）：论文提出"专业化/分工可能导致 AI 集体的递归改进"。NeoTrix 的 A2A 总线 + AgentCommunicationBus 已经存在，但智能体之间的世界模型共享（counterfactual scenario 交换、预测分歧检测）未接线。

4. **抽象屏障**（Abstraction Barrier, Section 5.5）：论文讨论人类的高"具身因子"（内部处理/I/O 比）迫使形成深层抽象模型，而 AI 的低具身因子可能不需要。NeoTrix 的 VSA 统一表征（意识体十条 #2）正是对抽象屏障的回应——所有子系统共享 VSA，不需要跨抽象层转换。

5. **研究议程**（Section 7.1）直接映射到 NeoTrix 的待办事项：
   - 4a-f（递归自改进测量）→ 我们已经实现了 RSI 指标（`rsi_metrics.rs`）
   - 5c（专业化→递归改进）→ 需要接线多智能体世界模型共享
   - 5a-f（多智能体缩放律）→ A2A 总线 + AgentCommunicationBus 还未被世界模型利用

### arXiv:2605.23872 "Training-Free Looped Transformers" (May 2026)

将预训练 transformer 的中间块在推理时循环，把 consciousness pipeline 从 flat batch 建模为 ODE 精化循环：
- 每个 `handle_*_tick()` = ODE 的一步
- `attractor_state` = 积分状态
- 阻尼系数 α 随 coherence 动态调整
- 收敛检测提前 break

### Hivemind (activeloopai/hivemind, Jun 2026)

捕捉意识 cycle trace → VSA 聚类 → 注册为新 CapabilitySynthesizer composite：
- 每 50 cycle 挖掘 thought_history + vsa_buffer 中的重复模式
- 跨 cycle 技能传播

## Phase 36 实施计划（6 个子阶段，3 波并行）

### 依赖图

```
波 1 (完全独立):
  P0 ── JEPA 世界模型桥接 ──→ 修改 types.rs + modules.rs + core.rs
  P2 ── 反事实推理接线 ────→ 修改 modules.rs + types.rs
  P3 ── 想象合成→Fusion ───→ 修改 fusion_deliberator.rs + types.rs
  P4 ── 物理+空间推理接线 ──→ 修改 modules.rs + types.rs

波 2 (依赖 P0):
  P1 ── 自修正回路 ────────→ 扩展 P0 (P0 的预测误差驱动)
  P5 ── AcT+EFE+JEPA 规划 ──→ 修改 act_planner + efe_minimizer

波 3 (依赖前面所有):
  P6a ─ Looped Consciousness ──→ 修改 core.rs (收敛循环)
  P6b ─ Trace Mining ──────────→ 修改 capability_synthesizer.rs
```

### 波 1: P0 + P2 + P3 + P4（4 路并行）

#### P0 — JEPA 世界模型桥接（最高优先级）

**文件**: `neotrix-core/src/neotrix/nt_mind_background_loop/consciousness/types.rs`, `core.rs`

**修改**:
1. 添加字段到 `ConsciousnessIntegration`:
   ```rust
   pub world_model_predictive_cortex: Option<PredictiveCortex>,
   ```
   初始化: `world_model_predictive_cortex: None,`（懒加载）

2. 在 `core.rs` 中添加 `handle_world_model_tick()`:
   - 如果 `world_model_predictive_cortex` 为 None 且 VSA buffer 非空，懒初始化
   - `attractor_state` → `[f64]` 转换为 PredictiveCortex 的 latent space
   - 调用 `predict_horizon()` → 更新 `attractor_state`
   - 检测 anomaly → 注入 `pending_curiosity_gain`

3. 每 cycle 调用（在 context_gather 后）

#### P2 — 反事实推理接线

**文件**: `neotrix-core/src/neotrix/nt_mind_background_loop/consciousness/types.rs`, `modules.rs`

**修改**:
1. 添加字段:
   ```rust
   pub counterfactual_engine: CounterfactualFuturesEngine,
   ```

2. 替换 `handle_counterfactual_futures_tick()` stub:
   - 从 `attractor_state` 注册反事实问题
   - `generate_counterfactuals()` → 高 divergence 场景注入 FusionDeliberator

3. 每 7 cycle 调用

#### P3 — 想象合成 → Fusion 审议

**文件**: `core/nt_core_experience/fusion_deliberator.rs`

**修改**:
1. 添加 `ImaginationEngine` 字段到 `FusionDeliberator`
2. `deliberate_hierarchical()` 中增加 `PanelSource::Imagination` 变体
3. 合成场景作为额外推理面板进入 Judge

#### P4 — 物理 + 空间推理接线

**文件**: `neotrix-core/src/neotrix/nt_mind_background_loop/consciousness/types.rs`, `modules.rs`

**修改**:
1. 添加字段:
   ```rust
   pub physics_commonsense: PhysicsCommonsense,
   pub spatial_scene: SpatialSceneEngine,
   ```

2. 替换 `handle_physics_reasoning()` 和 `handle_spatial_scene()` stubs

3. 每 5 cycle 调用

### 波 2: P1 + P5（2 路并行）

#### P1 — 自修正回路（扩展 P0）

在 `handle_world_model_tick()` 中检测预测误差趋势:
- EMA prediction error > threshold → 调整 inner_critic 阈值 + cognitive_load arousal

#### P5 — 模型基规划（AcT + EFE + JEPA）

- 创建 `JepaTransitionModel` 包装 `JepaWorldModel.predict()`
- 替换 AcTPlanner 的 `sin(t)` toy transition
- 替换 EFEMinimizer 的 `SimpleTransitionModel`

### 波 3: P6a + P6b（2 路并行）

#### P6a — Looped Consciousness

在 `handle_consciousness_batch()` 中:
```
let mut state = initial;
while delta(state, prev) > ε && step < max_steps:
    state = damp(state, context_gather(state))
    state = damp(state, world_model_tick(state))
    ...
```

#### P6b — Trace Mining

每 50 cycle 挖掘 `thought_history` → VSA clustering → 注册 composite capability

## 量化目标

| 指标 | 接线前 | 接线后 | 衡量方式 |
|------|-------|-------|---------|
| 世界模型调用 | 0/cycle | 1/cycle | `handle_world_model_tick()` 返回值 |
| 反事实场景 | 0 | ≥3/7cycles | `CounterfactualFuturesEngine` scenario 计数 |
| 物理推理 | stub→"ok" | 真实 VSA 物理 | `handle_physics_tick()` 返回值 |
| 模型基规划 | sin(t) toy | JEPA 真实转移 | AcT planner 使用 JepaTransitionModel |
| 自修正触发 | 0 | 检测退化+调整参数 | 预测误差 EMA 跟踪 |
| 收敛检测 | flat batch | 阻尼精化 | delta(state) < ε 提前 break |
| Trace mining | 0 | ≥1/50cycles | 新注册 composite 数 |

## 风险

| 风险 | 概率 | 缓解 |
|------|------|------|
| PredictiveCortex 需要 latent_dim 参数 | 中 | 懒初始化, VSA_DIM=4096 作为 latent_dim |
| CounterfactualFuturesEngine 需要 VSA dim | 低 | Default 使用 DEFAULT_VSA_DIM |
| JepaTransitionModel 需要 JEPA 模型访问 | 中 | 先接线 PredictiveCortex, JEPA 作为增强 |
| 物理/空间触发语义噪声 | 中 | attention_gate + task_type 门控 |
