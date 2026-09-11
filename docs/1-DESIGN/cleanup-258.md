# Cleanup-258: 跨域引用分析

**日期**: 2026-09-11  
**分析**: Six-Layer Architecture 跨域引用检查

---

## 跨域引用结果

```
=== 跨域引用 ===
l5_cognition→l1_action:        4
l5_cognition→l2_perception:    1
l5_cognition→l6_meta:          2
```

---

## 依赖关系分析

### 唯一问题源: `l5_cognition`

| 来源层 | 目标层 | 引用数 | 违反原则 |
|--------|--------|--------|----------|
| L5 Cognition | L1 Action | 4 | **L5 → L1 下行依赖** |
| L5 Cognition | L2 Perception | 1 | **L5 → L2 下行依赖** |
| L5 Cognition | L6 Meta | 2 | L5 → L6 上行依赖（合理） |

### 其他层情况

| 层 | 跨域引用 | 状态 |
|----|----------|------|
| L1 Action | 0 | ✅ 干净 |
| L2 Perception | 0 | ✅ 干净 |
| L3 Embodiment | 0 | ✅ 干净 |
| L4 Emotion | 0 | ✅ 干净 |
| L6 Meta | 仅内部自引用 | ✅ 干净 |

---

## 违反分析

### L5 → L1 下行依赖 (4处)

**六层架构原则**: 高层不应依赖低层（信息流应向上）

**合理模式**:
- L1→L5: 低层提供能力供高层调用 ✅
- L5→L1: 高层直接调用低层 ❌ **破坏分层隔离**

**影响**:
- L1 Action 的变更会直接影响 L5 Cognition
- 违反依赖倒置原则
- 增加跨层耦合

### L5 → L2 下行依赖 (1处)

**同理**: L5 Cognition 不应直接依赖 L2 Perception

---

## 建议修复策略

### 方案 A: 依赖倒置（推荐）

```
L5 定义抽象 trait
L1/L2 实现具体逻辑
通过 trait object 或依赖注入解耦
```

**优点**: 符合 SOLID 依赖倒置原则  
**缺点**: 需要重构接口

### 方案 B: 事件总线解耦

```
L1/L2 通过 EventBus 发布事件
L5 订阅事件而非直接调用
```

**优点**: 完全解耦  
**缺点**: 引入异步复杂度

### 方案 C: 提取共享层

```
将 L5 需要的 L1/L2 能力提取到共享模块
共享模块不属于任何特定层
```

**优点**: 实现简单  
**缺点**: 可能引入新的依赖混乱

---

## 具体引用位置

需要定位并分析以下引用：

```bash
# L5 → L1 引用
grep -rn "use crate::l1_action" neotrix-core/src/l5_cognition --include="*.rs" | grep -v test | grep -v facade

# L5 → L2 引用
grep -rn "use crate::l2_perception" neotrix-core/src/l5_cognition --include="*.rs" | grep -v test | grep -v facade

# L5 → L6 引用（通常合理）
grep -rn "use crate::l6_meta" neotrix-core/src/l5_cognition --include="*.rs" | grep -v test | grep -v facade
```

---

## 实际引用详情

### L5→L1 引用 (3处有效引用)

| 文件 | 行号 | 引用 | 备注 |
|------|------|------|------|
| `nt_mind/evolution/self_diagnose.rs` | 10 | `use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot` | 活跃引用 |
| `nt_mind/evolution/self_diagnose.rs` | 33 | `// pub use crate::l1_action::nt_act::nt_l1_shared_types::{...}` | 已注释 |
| `nt_mind/evolution/evolution_loop.rs` | 21 | `// pub use crate::l1_action::nt_act::nt_l1_shared_types::IssueType;` | 已注释 |
| `nt_mind/evolution/evolution_loop.rs` | 67 | `pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot` | 活跃引用 |

### L5→L2 引用 (1处注释说明)

| 文件 | 行号 | 引用 | 备注 |
|------|------|------|------|
| `l5_cognition/mod.rs` | 16 | `/// L2 类型通过此模块访问, 避免散布 use crate::l2_perception::*` | 注释说明，非实际引用 |

---

## 真实问题定位

**实际活跃引用仅 2 处**:

1. **self_diagnose.rs:10** - 使用 `ProjectSnapshot`
2. **evolution_loop.rs:67** - 使用 `ProjectSnapshot`

**核心问题**: `ProjectSnapshot` 类型定义在 L1 Action 层 (`l1_action::nt_act::nt_act_types`)，但被 L5 Cognition 层使用。

**修复建议**: 
- 将 `ProjectSnapshot` 移到共享层（如 `l3_embodiment::shared_types` 或新建 `core::types`）
- 或在 L5 定义同构的类型，通过转换层适配

---

## 结论

| 维度 | 评估 |
|------|------|
| 整体健康度 | ⚠️ 中等 |
| 问题严重度 | 🔴 中高（L5→L1 违反核心分层） |
| 修复复杂度 | 中等 |
| 建议优先级 | P1 |

**核心问题**: L5 Cognition 对 L1 Action 的 4 处直接依赖违反了六层架构的信息流原则，建议优先修复。
