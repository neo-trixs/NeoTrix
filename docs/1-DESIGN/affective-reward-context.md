# 情感奖励上下文 — affective → SEAL reward 注入设计 (Affective Reward Context)

> 状态: **Draft (P1-P4 全部实现, 待评审/提交)** | 作者: NT-CORE + NT-MIND | 日期: 2026-08-20
> 前置: Q2 延期项 (KB cycle 1203 待办) — 把用户情感检测/信任阶段信号作为 SEAL 进化奖励
> 上下文注入, 使进化受用户情绪反馈引导。本文为纯设计 (不写码), 待 seal_core 无并发
> 编辑窗口实现。

---

## 1. 现状盘点 (全部经源码核实, 非假设)

| 资产 | 内容 | 位置 | 状态 |
|------|------|------|------|
| 奖励融合 | `PerformanceEvaluator::combine_reward(capability_score, feedback, external_weight)` — 外部执行奖励与内部能力自评加权融合 | `seal_core/core/evaluator.rs:107` | ⚠️ **纯测试函数 — 零生产调用点** |
| 执行反馈 | `ExecutionFeedback { verified, latency_ratio, quality }` | `evaluator.rs:15` | ⚠️ 同 combine_reward, 无生产消费 |
| 奖励来源 | `RewardSource { External, Internal }`, External 计权 2x | `nt_core_knowledge/types.rs` | ✅ 生产 (loop_impl reward_source) |
| 奖励消费 | checkpoint 存取 reward (负奖励 rewind), `brain_core.weight_update(reward)` 微调 capability 权重 | `checkpoint.rs` / `brain_core.rs:125` | ✅ 生产 |
| 情感信号 | `EmotionState` (6 维; `valence()=(Joy+Confidence)/2`, `arousal()=(Frustration+Urgency)/2`) | `core/nt_core_self/emotion_state.rs:82` | ✅ 生产 (bg loop 持久化 + 数字人消费) |
| 关系阶段 | `RelationshipStage { Stranger→Acquaintance→Friend→Confidant→Bond }` + interactions 计数 | `affective_interface.rs:238` | ✅ 生产 (C4: ws 数字人消费) |
| 数字人产出 | `process_audio_input` → `{type, content, emotion, animation, trust}` | `nt_io_digital_human.rs:296` | ✅ 生产 (C4: handle_ws_text) |

**核心洞察**: 奖励管线 (`combine_reward`/`ExecutionFeedback`) 是 **CUDA Agent 吸收的纯能力**,
设计为外部执行反馈的奖励源, 但**从未接入生产**。Q2 不是「情感信号挤进已接线的奖励」,
而是**两条都未接线的通道** (外部执行奖励 + 情感信号) 各自接线, 情感作为其中一种
外部信号源。这避免与并发 session 正在重构的 `loop_impl/core.rs` reward 内部字段耦合 —
设计只定义**新类型的生产消费点**, 不动既有字段。

---

## 2. 设计语义

### 2.1 目标
在 SEAL 迭代产生 reward 时, 若当前会话存在用户情感/信任观测, 则将情感状态映射为
情感奖励上下文 `AffectiveFeedback`, 与既有 `ExecutionFeedback` 并列作为外部信号源,
经 `RewardSource::External` 优先通道进入奖励计算。

### 2.2 情感→奖励映射 (防污染纪律, R-P84)

情感信号**只做引导, 不做裁判**: 情感奖励幅度上限 ≤ 外部执行奖励的 30%,
防止「讨好用户」替代「验证通过」成为进化主信号。

| 信号 | 映射 | 说明 |
|------|------|------|
| `valence()` | 正贡献基底 | 用户满意 → 该迭代方向被正向强化 |
| `arousal()` | 负向警戒 | 高唤醒+低愉悦(挫败) 是**故障信号**, 压低奖励而非抬高 |
| `RelationshipStage` | 信任权重 | Bond 时情感信号置信度高 (权重↑), Stranger 时置信低 (权重↓) |
| `interactions` | 冷启动抑制 | interactions < 3 时情感信号不生效 (置信不足) |

### 2.3 新类型 (纯数据结构, 不碰既有字段)

```rust
/// 情感奖励上下文: 用户情感观测 → 奖励引导信号。
/// 语义: 只做引导不做裁判 (幅度 cap 0.3); 冷启动抑制; 高信任加权。
pub struct AffectiveFeedback {
    pub valence: f64,          // 0..1, 用户情绪愉悦度
    pub arousal: f64,          // 0..1, 唤醒度
    pub stage: u8,             // RelationshipStage 序数 (0=Stranger..4=Bond)
    pub interactions: u32,     // 交互计数 (冷启动门限 <3 抑制)
    pub signal_weight: f64,    // 0..1, 情感信号占总奖励比例 (cap 0.3)
}
```

### 2.4 融合函数 (新纯函数, 与 combine_reward 并列)

```rust
impl PerformanceEvaluator {
    /// combine_reward 的情感扩展: external 通道内部再细分
    /// 执行验证信号 (verified/latency/quality) 与情感引导信号 (AffectiveFeedback)。
    /// combined = execution_reward * (1 - signal_weight) + affective_guide * signal_weight
    /// affective_guide ∈ [0, 1], 由 valence/stage/interactions 合成, arousal 修正。
    pub fn combine_reward_with_affective(
        capability_score: f64,
        feedback: ExecutionFeedback,
        affective: Option<AffectiveFeedback>,
        external_weight: f64,
    ) -> f64
}
```

合成规则 (确定性, 防 LLM 打分 flat-band):
- `affective_guide = valence * trust_k * engagement_k`
  - `trust_k = 0.6 + 0.1 * stage` (Stranger 0.6 → Bond 1.0)
  - `engagement_k = if interactions >= 3 { 1.0 } else { 0.0 }` (冷启动抑制)
  - `arousal` 修正: `if valence < 0.5 && arousal > 0.5 { guide *= 0.5 }` (挫败警戒)
- `signal_weight = min(affective.signal_weight, 0.3)` — 情感幅度上限 30%
- 最终 `combined.clamp(0.0, 1.0)`

---

## 3. 接线点 (生产消费)

| 步 | 接线 | 位置 | 依赖 |
|----|------|------|------|
| 1 | 数字人消费处产出情感观测快照: `process_audio_input` 已产出 emotion/trust → 存为待消费的 `AffectiveFeedback` 候选 | `nt_io_digital_human.rs:296` 旁路事件 | 无 (独立于 loop_impl) |
| 2 | SEAL 迭代计算 reward 处调用 `combine_reward_with_affective`, 传入候选情感快照 | loop_impl reward 计算点 (经 RewardSource::External 通道) | **需并发窗口 — loop_impl/core.rs 正在重构** |
| 3 | 情感信号来源置 `RewardSource::External` (计权 2x 由既有枚举承担, 不新增枚举值) | `nt_core_knowledge/types.rs` | 无改动 |

**依赖原则**: 步骤 1 零依赖可先做; 步骤 2 是唯一触碰并发活跃区 (loop_impl) 的点,
必须等其稳定窗口。步骤 3 不改代码。

---

## 4. 测试计划

1. `test_combine_reward_with_affective_boost` — 高 valence + Bond 情感信号抬高奖励
2. `test_combine_reward_with_affective_cold_start` — interactions<3 时情感信号无效应
3. `test_combine_reward_with_affective_frustration` — 低 valence + 高 arousal 压低奖励
4. `test_combine_reward_with_affective_cap_30` — signal_weight=1.0 时仍被 cap 0.3
5. `test_combine_reward_with_affective_none` — None 时退化为 combine_reward 行为 (回归)
6. 生产接线验证: 数字人 ws 交互后, 下一 SEAL 迭代 reward_source == External 且
   reward 被情感引导修正 (T3 生产接线检查)

---

## 5. 实现顺序 (窗口就绪时)

| 阶段 | 内容 | 风险 |
|------|------|------|
| P1 | `AffectiveFeedback` 类型 + `combine_reward_with_affective` 纯函数 + 测试 1-5 (evaluator.rs) | ✅ **已实现** (2026-08-20, 33 测试过; 冷启动时 sw 亦归 0 保持信号完全惰性) |
| P2 | 数字人消费处产出情感快照事件 (步骤 1) | ✅ **已实现** (AffectiveFeedback 迁共享层 nt_core_knowledge; last_affective_feedback 字段 + process_audio_input 旁路快照, 13 测试过) |
| P3 | loop_impl 计算点接线 (步骤 2) | ✅ **已实现** (RewardCalculationStage 融合; 负外部奖励原样保留防 rewind 回归; 共享观测槽 publish/take 跨域桥) |
| P4 | 生产接线验证 (测试 6) + 能力树晋升证据 | ✅ **已实现** (3 集成测试 = T3 验证; 全量 444 测试过) |

**验收**: `cargo check --lib` 0 error + 上述 6 测试全过 + P3 接线后 reward_source
生产路径可观测情感引导。能力树: `nt_mind_seal_core::affective_reward_context` 新节点
bud → C4 (wiring 证据: 数字人事件 + loop_impl 消费点)。

---

## 6. 相关引用
- KB: cycle 1203 Q2 待办条目 (branch 含证据 pointer)
- 既有设计: CUDA Agent 吸收 (cycle 1188) — `combine_reward` 奖励信号接线
- 约束: R-P84 (预检), R-P79 (同 session 接线禁止死代码), R-P42 (强化既有节点非平行适配器)