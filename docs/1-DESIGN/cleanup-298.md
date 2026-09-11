# 跨层Import扫描报告

**扫描时间**: $(date '+%Y-%m-%d %H:%M:%S')
**扫描目录**: neotrix-core/src/
**扫描层**: l1_action, l2_perception, l3_embodiment, l4_emotion, l5_cognition, l6_meta
**排除**: test文件, facade文件

## 扫描规则

- 检查每层中的`use`语句是否引用其他层的模块
- 排除测试文件（包含`test`或`tests`的文件）
- 排除facade文件（包含`facade`的文件）


## 跨层Import统计

| 源层 | 目标层 | 引用数量 |
|------|--------|----------|
| l5_cognition | l1_action |        6 |
| l5_cognition | l2_perception |        1 |
| l5_cognition | l6_meta |        2 |

**总跨层引用数**: 9

## 详细引用列表

### l5_cognition → l1_action

```
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs:679:        use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_community::{CommunityDetector, CommunityAwareSearch};
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs:680:        use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::{
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/self_diagnose.rs:10:use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/self_diagnose.rs:33:// pub use crate::l1_action::nt_act::nt_l1_shared_types::{
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:21:// pub use crate::l1_action::nt_act::nt_l1_shared_types::IssueType;
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:67:pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
```

### l5_cognition → l2_perception

```
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/mod.rs:16:/// L2 类型通过此模块访问, 避免散布 `use crate::l2_perception::*`。
```

### l5_cognition → l6_meta

```
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/mod.rs:22:/// L6 类型通过此模块访问, 避免散布 `use crate::l6_meta::*`。
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/traits.rs:137:// `use crate::l6_meta::*` 造成向上依赖。Concrete implementations
```

## 各层被引用统计

| 被引用层 | 被引用次数 |
|----------|------------|
| l1_action | 6 |
| l2_perception | 1 |
| l3_embodiment | 0 |
| l4_emotion | 0 |
| l5_cognition | 0 |
| l6_meta | 2 |

## 建议

- 高跨层引用可能导致循环依赖
- 建议保持单向依赖：L1 → L2 → L3 → L4 → L5 → L6
- 检查双向引用是否为设计需要
