# cleanup-291: 跨域引用审计

**日期**: 2026-09-11
**范围**: neotrix-core/src/ 六层架构 (l1_action ~ l6_meta)

## 扫描结果

```
跨域引用 (排除 test/facade/注释)
─────────────────────────────────
l5_cognition → l1_action:  2 处 (活跃)
l5_cognition → l2_perception: 0 处 (仅注释/文档)
l5_cognition → l6_meta:   0 处 (仅注释/文档)
```

**总计**: 仅 1 条活跃跨域依赖 (l5→l1), 其余均已通过 facade 门面集中化。

## 活跃引用详情

| # | 源文件 | 行 | 引用 |
|---|--------|-----|------|
| 1 | `nt_mind/evolution/self_diagnose.rs:10` | `use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;` | **违规** — 绕过 facade 直引 |
| 2 | `nt_mind/evolution/evolution_loop.rs:67` | `pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;` | **违规** — 绕过 facade 直引 |

## Facade 机制分析

`l5_cognition/mod.rs` 已建立 5 个 facade 模块:

| Facade | 用途 | 状态 |
|--------|------|------|
| `act_facade` | L5→L1 NT-ACT 类型 re-export | ✅ 已建立 |
| `kb_facade` | L5→L1 NT-MEMORY KB 类型 re-export | ✅ 已建立 |
| `io_facade` | L5→L1 NT-IO 共享类型 re-export | ✅ 已建立 |
| `io_skills_facade` | L5→L1 NT-IO 技能模块 re-export | ✅ 已建立 |
| `l2_facade` | L5→L2 感知层类型 re-export | ✅ 已建立 |
| `l3_facade` | L5→L3 共享类型 re-export | ✅ 已建立 |
| `l6_facade` | L5→L6 元认知层类型 re-export | ✅ 已建立 |

**结论**: facade 架构已完整, 但 `self_diagnose.rs` 和 `evolution_loop.rs` 未使用 `act_facade`。

## 修复方案

将两处直接引用改为通过 `act_facade` 访问:

```rust
// before
use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;

// after (通过 facade)
use crate::l5_cognition::act_facade::ProjectSnapshot;
```

## 其他层引用

| 层 | 跨域活跃引用 |
|----|------------|
| l1_action | 0 |
| l2_perception | 0 |
| l3_embodiment | 0 |
| l4_emotion | 0 |
| l6_meta | 0 |

六层架构隔离良好, 仅 l5_cognition→l1_action 有 2 处绕过 facade 的直接引用。
