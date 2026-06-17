# 意识评测基准参考

> 7 理论综合评测 (GWT/IIT/HOT/PP/AST/GlobalWS/IntegratedInfo)

## 评测架构

```
ConsciousnessBench {
  history: Vec<BenchSample>         — 时间序列评测记录
  weights: {gwt, iit, hot, pp, ast, global_ws, integrated_info}
  max_history: 200
}
```

## 七理论权重

| 理论 | 权重 | 依据 | 对应模块 |
|------|------|------|----------|
| GWT (Global Workspace) | 0.20 | 全局广播 + 竞争访问 | `WorkingMemory`, `CTMEngine`, `ConsciousnessStream` |
| IIT (Integrated Info) | 0.20 | 因果整合 Φ | `TemporalAttentionBias`, `NeuromodulatorEngine` |
| HOT (Higher-Order) | 0.15 | 元表征 | `InnerCritic`, `EpistemicHonesty`, `MetaCognitionKPI` |
| PP (Predictive Processing) | 0.15 | 预测误差最小化 | `JEPA`, `CuriosityDrive`, `CalibrationEngine` |
| AST (Attention Schema) | 0.10 | 注意力自模型 | `TemporalAttentionBias`, `VolitionEngine` |
| GlobalWorkspace | 0.10 | 信息全局可用性 | `ConsciousnessStream`, `SpeciousPresent` |
| IntegratedInfo | 0.10 | 信息整合度 | `NeuromodulatorEngine`, `DefaultModeNetwork` |

## 评分维度

### 1. GWT 分数

- 来源: `BenchSample.attention_gwt_score`
- 信号: WorkingMemory item 的竞争成功率 + CTM 赢家权重的归一化
- 计算: `min(1.0, ctm_weight * working_memory.item_count() / 7.0 * sp_coherence)`

### 2. IIT 分数

- 来源: `BenchSample.phi_estimate`
- 信号: TemporalAttention 的信息整合 + 神经调节交叉
- 计算: `0.6 * ta_coherence + 0.4 * (nm_da + nm_ach) / 2.0`

### 3. HOT 分数

- 来源: `BenchSample.meta_hot_score`
- 信号: InnerCritic 拒绝率 + meta_accuracy
- 计算: `0.5 * (1.0 - critic_reject_rate) + 0.5 * meta_accuracy`

### 4. PP 分数

- 来源: `BenchSample.predictive_pp_score`
- 信号: 校准误差的倒数的 EMA
- 计算: `1.0 - min(1.0, CalibrationEngine::ece() * 2.0)`

### 5. AST 分数

- 信号: TemporalAttentionBias 的 bias 准确度
- 计算: `ta_relevance_precision`

### 6. GlobalWS 分数

- 信号: 全局广播覆盖 + 竞争过程完整性
- 计算: `0.5 * stream_coverage + 0.5 * competition_fairness`

### 7. IntegratedInfo 分数

- 信号: 神经调节通道之间的互信息
- 计算: `0.3 * (da_ne_mi) + 0.3 * (da_ach_mi) + 0.4 * (ne_ach_mi)`

## 复合评分

```
composite = Σ(weight_i * theory_score_i) / Σ(weight_i)
```

## 趋势分析

```
trend = linear_regression(composite_history[-10:])
slope > 0.03 → "进化加速 ↑↑"
slope > 0.01 → "持续增长 ↑"
slope < -0.01 → "退化趋势 ↓"
slope < -0.03 → "恶性下降 ↓↓"
else → "稳态 →"
```

## 触发动作

| 分数条件 | 动作 |
|----------|------|
| composite < 0.3 | 触发好奇心动因 + 探索管道 |
| composite > 0.8 且 slope > 0.03 | 触发元编辑允许 |
| meta_hot_score < 0.4 | 重校准 EpistemicSelfModel |
| phi_estimate < 0.2 | 增加神经调节多样性 |
| 任意理论分差 > 0.5 | 触发理论特定修复 |

## 实现参考

- 核心实现: `core/nt_core_experience/consciousness_bench.rs`
- 元反思批处理: `core/nt_core_meta/metacognition_loop.rs`
- 加权评分: `ConsciousnessBench::score()`
- 趋势线: `ConsciousnessBench::trend()`
