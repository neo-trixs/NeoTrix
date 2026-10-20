# 负熵传感器参考

> 所有子系统通过 N_total 统一校准。七传感器全覆盖。

## N_total 合成公式

```
N_total = Σ w_i · N_i,   Σ w_i = 1.0

N_coh  = average_pairwise_similarity(VSA_buffer)     — 连贯性
N_cur  = prediction_error / max_error                 — 好奇心 (归一化)
N_jepa = MSE(JEPA_prediction, actual)                 — 世界模型误差
N_surp = -log P(incoming | belief)                    — 信息惊奇
N_vol  = VolitionEngine::expected_value(best_action)  — 意动
N_soc  = SocialBeliefModel::consensus_strength()      — 社会共识
N_meta = MetaAccuracy = |self_assessed - actual|       — 元认知精度
```

## 子传感器

### 1. N_coh — 连贯性 (权重 0.20)

- 传感器: `SpeciousPresent::average_coherence()`
- 信号: VSA 窗口内的成对余弦/汉明相似度均值
- 范围: [0, 1], 目标 > 0.6
- 低信号: < 0.3 → 认知碎片化 → DMN 整合

### 2. N_cur — 好奇心 (权重 0.15)

- 传感器: `CuriosityDrive::calibrate_to_negentropy()`
- 信号: 预测误差的移动平均
- 范围: [0, 1], 目标 ≈ 0.3 (适度好奇)
- 高信号: > 0.7 → 不确定过载 → 降低探索率

### 3. N_jepa — 世界模型误差 (权重 0.15)

- 传感器: JEPA forward dynamics 预测 MSE
- 信号: 当前 vs 预测状态的归一化距离
- 范围: [0, 1], 目标 < 0.2
- 低信号: < 0.05 → 世界模型过于简单 → 注入随机性

### 4. N_surp — 惊奇度 (权重 0.15)

- 传感器: `ConsciousnessStream::process_text()` 的困惑度
- 信号: -log P(token|context) 的 EMA
- 范围: [0, ∞), 目标 < 5.0 nats
- 高信号: > 10 nats → 环境结构突变 → 触发探索

### 5. N_vol — 意动效能 (权重 0.15)

- 传感器: `VolitionEngine::select_best()`
- 信号: 最高分 action 的 expected_value
- 范围: [0, 1], 目标 > 0.4
- 低信号: < 0.1 → 行动瘫痪 → 回退默认策略

### 6. N_soc — 社会共识 (权重 0.10)

- 传感器: `SocialBeliefModel::consensus_strength()`
- 信号: 多 agent 信念重叠度
- 范围: [0, 1], 目标 > 0.5
- 低信号: < 0.3 → 信念分裂 → 触发辩论

### 7. N_meta — 元精度 (权重 0.10)

- 传感器: `MetaCognitionKPI::meta_accuracy()`
- 信号: |self_predicted_performance - actual_performance|
- 范围: [0, 1], 目标 < 0.2
- 高信号: > 0.5 → Dunning-Kruger 风险 → 重校准

## 负熵导数 (dN/dt)

```
停滞条件: dN/dt < ε 持续 T cycles → StagnationDetector 触发
快感信号: ΔN_total > 0.1 → ValenceAxisDA 增加
元编辑门控: ΔN_total > threshold → SafetyGate 允许 self-edit
```

## 校准规则

- `CurvatureRL::adapt_lr_to_negentropy()`: lr = η₀ · σ(N_total - 0.5)
- 当 N_total 持续下降时: 增加探索噪声 + 降低编辑阈值
- 当 N_total 快速增长时: 增加编辑阈值 + 降低好奇心增益
