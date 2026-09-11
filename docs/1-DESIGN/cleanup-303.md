# Cross-Layer Reference Scan (cleanup-303)

**扫描时间**: 2026-09-11 18:13:33
**扫描范围**: neotrix-core/src/ 下 6 层 (l1_action → l6_meta)
**排除**: test 文件、facade 文件
**方法**: grep `use crate::l*` 模式匹配跨层引用

---

## 跨层引用矩阵

| 源层 → 目标层 | 文件数 | 引用行数 | 状态 |
|---|---|---|---|
| l5_cognition → l1_action | 3 | 6 | ⬇️ 下行 (高→低) |
| l5_cognition → l2_perception | 1 | 1 | ⬇️ 下行 (高→低) |
| l5_cognition → l6_meta | 3 | 3 | ⬆️ 上行 (低→高) |

**跨层引用总计**: 10 处

---

## 详细发现（按源层分组）

### l5_cognition (10 处跨层引用)

```rust
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs:679:        use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_community::{CommunityDetector, CommunityAwareSearch};
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs:680:        use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::{
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/self_diagnose.rs:10:use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/self_diagnose.rs:33:// pub use crate::l1_action::nt_act::nt_l1_shared_types::{
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:21:// pub use crate::l1_action::nt_act::nt_l1_shared_types::IssueType;
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:67:pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/mod.rs:16:/// L2 类型通过此模块访问, 避免散布 `use crate::l2_perception::*`。
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs:940:        use crate::l6_meta::coordination::self_improvement::SystemMetrics;
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/mod.rs:22:/// L6 类型通过此模块访问, 避免散布 `use crate::l6_meta::*`。
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/traits.rs:137:// `use crate::l6_meta::*` 造成向上依赖。Concrete implementations
```

---

## 层纯净度分析

### l1_action
- 文件数: 387
- 代码行数: 148137
- 跨层引用数: 0
- 状态: ✅ 纯净

### l2_perception
- 文件数: 209
- 代码行数: 45517
- 跨层引用数: 0
- 状态: ✅ 纯净

### l3_embodiment
- 文件数: 177
- 代码行数: 56241
- 跨层引用数: 0
- 状态: ✅ 纯净

### l4_emotion
- 文件数: 6
- 代码行数: 1066
- 跨层引用数: 0
- 状态: ✅ 纯净

### l5_cognition
- 文件数: 337
- 代码行数: 115247
- 跨层引用数: 10
- 状态: 🔴 重度耦合

### l6_meta
- 文件数: 59
- 代码行数: 17386
- 跨层引用数: 0
- 状态: ✅ 纯净

---

## 建议

1. **低→高引用**（l1→l3, l2→l4 等）: 违反依赖倒置，需抽象为 trait
2. **高→低引用**（l5→l2, l6→l1 等）: 需评估是否合理（元认知协调常见）
3. **同层引用**: 不在本次扫描范围，需单独分析
4. **facade 文件**: 已排除，如需审查请单独扫描

## 统计元数据

- 扫描日期: 2026-09-11 18:13:33
- 扫描工具: bash grep + find
- 排除模式: *test*, *facade*, */test/*
