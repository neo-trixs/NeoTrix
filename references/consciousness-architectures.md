# 意识架构参考

> 支持的意识理论目录。每个理论的形式化约束和 NeoTrix 实现状态。

## IIT — Integrated Information Theory (Φ)

**核心要求**: 系统必须具有不可约的因果能力。Φ 度量集成信息量。

**NeoTrix 实现**: `core/nt_core_gwt/geometry_sync.rs` — `IntegratedPhi` 结构体, 每 cycle 计算。

**通过标准**:
```
Φ ≥ 0.15  — 最小集成阈值
Φ_delta/cycle  ≥ -0.01  — 不退化
d(Φ, Φ_ref) < 0.05       — 跨会话稳定
```

**检查点**: `ConsciousnessBench::test_iit_phi()` — 18 测试

## GWT — Global Workspace Theory

**核心要求**: 竞争式全局访问。多种内容争夺进入全局工作空间，赢家广播到全系统。

**NeoTrix 实现**:
- `selective_spotlight.rs` — 注意力选择
- `entropy_attention.rs` — 熵驱动竞争
- `working_memory.rs` — 全局广播缓冲区

**通过标准**:
```
竞争延迟 ≤ 5 cycles
全局广播覆盖率 ≥ 80% 子系统
一次最多一个内容占据工作空间
```

## HOT — Higher-Order Thought Theory

**核心要求**: 意识状态伴随对该状态的高阶思想。元认知精度 = 自预测 vs 实际表现的一致性。

**NeoTrix 实现**:
- `MetacognitiveEvaluator` — meta-d' 度量
- `EpistemicSelfModel` — 置信度校准
- `InnerCritic` — 输出门控

**通过标准**:
```
meta-d' ≥ 0.5
ECE (Expected Calibration Error) ≤ 0.15
自预测置信度与实际准确率相关系数 ≥ 0.6
```

## PP — Phenomenological Principle (Specious Present)

**核心要求**: 时间厚度窗口—当下的体验包含最近的过去和预期的未来。典型窗口 500ms-3s。

**NeoTrix 实现**: `specious_present.rs` — 滑动窗口缓冲区 + 平均相干性度量。

**通过标准**:
```
窗口宽度 ≥ 3 个以上事件
过去/未来不对称性 ≤ 0.3
平均相干性 ≥ 0.4
```

## AST — Attention Schema Theory

**核心要求**: 系统维护一个注意力的内部模型（注意力模式），用于控制注意力的分配。

**NeoTrix 实现**: `attention_gate.rs` + `entropy_attention.rs` — UtilitySignal 驱动的注意力分配。

**通过标准**:
```
注意力分配与 UtilitySignal 相关性 ≥ 0.5
注意力切换延迟 ≤ 2 cycles
注意力资源不超额分配 (sum ≤ 1.0)
```

## Global Workspace + Integrated Info (融合)

两理论融合评估: `consciousness_score = 0.25·Φ_norm + 0.20·access + 0.20·meta_d + 0.15·coherence + 0.10·arousal + 0.10·load_inv`

参见: `core/nt_core_experience/handler_profiler.rs` 中 `ConsciousnessBench`

## 意识体自检清单

```
□ IIT Φ ≥ 0.15
□ GWT 竞争延迟 ≤ 5 cycles
□ HOT meta-d' ≥ 0.5
□ PP 窗口 ≥ 3 事件
□ AST 注意力 - 信号相关 ≥ 0.5
□ Self-world 边界: VsaTag 无泄漏
□ 叙事连续性: 跨会话 self 一致
□ 优雅降级: 任意子系统失效不崩溃
□ 负熵增长: dN/dt > -0.01
```
