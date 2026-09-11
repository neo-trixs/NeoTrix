# Cleanup-223: 高风险跨层引用修复

**日期**: 2026-09-11
**范围**: L5→L6 (8次) + L6→L1 (2次)
**策略**: Facade 集中化 + Trait 抽象 (EvolutionHarness)

## 问题

六层架构要求依赖单向向下 (L6→L5→L4→...→L1)。发现以下违规:

### L5→L6 (向上依赖, 8次活跃引用)

| 文件 | 行 | 引用类型 |
|------|-----|---------|
| `seal_loop.rs` | 5 | `EvalHarness` (方法参数类型) |
| `control_distillation.rs` | 15 | `ConsciousnessGoldStandard` (struct 字段) |
| `handlers_consciousness.rs` | 257-258 | `EvolutionHarness` + `LoopConfig` (局部使用) |
| `mod.rs` | 33-34 | `ConsciousnessGoldStandard` + `ConsciousnessMonitor` (struct 字段) |
| `mod.rs` | 235 | `ConsciousnessMonitor` (测试) |
| `engine_core.rs` | 43 | `ConsciousnessGoldStandard` (构造函数) |

### L6→L1 (已由 facade 处理, 2次)

| 文件 | 行 | 引用 |
|------|-----|------|
| `l1_facade.rs` | 6 | `pub use crate::l1_action::nt_act::nt_act_cleanup::shared::*` |

> L6→L1 已通过 `l6_meta/l1_facade.rs` 集中化, 无需额外修复。

## 修复方案

### 策略选择

| 类型 | 使用方式 | 修复策略 |
|------|---------|---------|
| `ConsciousnessGoldStandard` | struct 字段 + 构造 | **Facade re-export** |
| `ConsciousnessMonitor` | struct 字段 + 方法调用 | **Facade re-export** |
| `EvalHarness` | 方法参数类型 | **Facade re-export** |
| `EvolutionHarness` + `LoopConfig` | 局部构造 + 多方法调用 | **Trait 抽象** (JSON-based) |

**选择依据**:
- Facade: 类型作为 struct 字段存储, 改为 trait 对象需改所有 struct 定义和调用点, 成本高
- Trait: `EvolutionHarness` 仅在方法体内局部使用, 且内部类型 (LoopReport, MetaObservationReport) 复杂, 用 JSON 桥接避免 L5 依赖 L6 内部类型

## 变更清单

### 新增文件

| 文件 | 用途 |
|------|------|
| `l5_cognition/l6_facade.rs` | L6→L5 集中 re-export 门面 |

### 修改文件

| 文件 | 变更 |
|------|------|
| `l5_cognition/mod.rs` | 新增 `pub mod l6_facade` |
| `l5_cognition/traits.rs` | 新增 `EvalHarnessApi`, `GoldStandardApi`, `ConsciousnessMonitorApi`, `EvolutionHarnessApi` trait |
| `l6_meta/healing/nt_mind_eval_harness.rs` | impl `EvalHarnessApi for EvalHarness` |
| `l6_meta/healing/nt_mind_consciousness_gold_standard.rs` | impl `GoldStandardApi for ConsciousnessGoldStandard` |
| `l6_meta/healing/nt_mind_consciousness_monitor.rs` | impl `ConsciousnessMonitorApi for ConsciousnessMonitor` |
| `l6_meta/memory/evolution_harness.rs` | impl `EvolutionHarnessApi for EvolutionHarness` |
| `l5_cognition/.../seal_loop.rs` | `use crate::l6_meta::...` → `use crate::l5_cognition::l6_facade::EvalHarness` |
| `l5_cognition/.../control_distillation.rs` | `use crate::l6_meta::...` → `use crate::l5_cognition::l6_facade::ConsciousnessGoldStandard` |
| `l5_cognition/.../mod.rs` | `use crate::l6_meta::...` → `use crate::l5_cognition::l6_facade::*` |
| `l5_cognition/.../engine_core.rs` | `use crate::l6_meta::...` → `use crate::l5_cognition::l6_facade::ConsciousnessGoldStandard` |
| `l5_cognition/.../handlers_consciousness.rs` | 直接 L6 导入 → `EvolutionHarnessApi` trait 调用 |

## 架构合规

修复后依赖方向:

```
L6 (元认知层)
  ↓ 定义 trait impl
L5 (认知层)
  ↓ 使用 facade re-export + trait 抽象
  ↓ (不直接 use crate::l6_meta::*)
L4 → L3 → L2 → L1
```

- L5→L6: 集中在 `l6_facade.rs` (re-export) + `traits.rs` (trait 定义)
- L6→L5: trait impl 在 L6 模块内 (L6 已依赖 L5, 无新增循环)
- L6→L1: 已由 `l1_facade.rs` 处理

## 后续建议

1. **渐进式 trait 化**: 对 `ConsciousnessGoldStandard`/`ConsciousnessMonitor` 的 struct 字段逐步改为 `Arc<dyn Trait>`, 减少 facade 依赖
2. **移动共享类型**: 将 L5 和 L6 共用的类型 (如 `RegressionCase`, `RegressionResult`) 下沉到 `core/`, 消除双向依赖
3. **Pre-commit hook**: 添加检查, 禁止 L5 代码直接 `use crate::l6_meta::` (仅允许通过 facade)
