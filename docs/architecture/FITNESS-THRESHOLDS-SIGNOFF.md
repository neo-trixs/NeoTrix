# Fitness 阈值签字单（EQ-11；SIM-42 提案）

> **状态**: 待 L5 书面签（数字＋理由逐行确认）。签字后 EQ-11 关。
> 依据：代码实证行号＋SIM-42 §41 P-4。V-3 原则：只许调严。

| 守卫 | 阈值 | 理由 | 签字 |
|------|------|------|------|
| NoCycleFitness | 0 环（任一环即红） | 能力网 DAG 有环＝拓扑契约破，无容忍空间 | ⬜ |
| CapabilityConsistencyFitness | 0 重边 | 重边＝注册表双写源， fail 即修 | ⬜ |
| TreeSingletonFitness | 生产实例化点 ≤1 | ConsciousnessTree 单例语义，多一处即分裂 | ⬜ |
| DeadCodeFitness | 0 警告＋禁 crate 级 `allow(dead_code)` | 死代码＋全局抑制＝腐烂入口 | ⬜ |
| PanicDensityFitness | max_abs 3000／max_per_kloc 12.0 | 既有数（SIM-09 先例），本次追认 | ⬜ |
| ConfidenceLabelFitness (B2) | 干净对新增引用 0（L2→L6/L3→L4/L4→L6）；label 缺失仅 info | 缩减归 P1-02，先卡增量不卡存量 | ⬜ |

签字人：__________ 日期：__________
