# 跨域引用检查

## 发现的跨域引用

| 来源域 | 目标域 | 引用数 |
|--------|--------|--------|
| l5_cognition | l1_action | 4 |
| l5_cognition | l2_perception | 1 |
| l5_cognition | l6_meta | 2 |

## 详细引用

### l5_cognition → l1_action (4处)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/self_diagnose.rs:10:use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/self_diagnose.rs:33:// pub use crate::l1_action::nt_act::nt_l1_shared_types::{
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:21:// pub use crate::l1_action::nt_act::nt_l1_shared_types::IssueType;
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:67:pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;

### l5_cognition → l2_perception (1处)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/mod.rs:16:/// L2 类型通过此模块访问, 避免散布 `use crate::l2_perception::*`。

### l5_cognition → l6_meta (2处)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/mod.rs:22:/// L6 类型通过此模块访问, 避免散布 `use crate::l6_meta::*`。
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/traits.rs:137:// `use crate::l6_meta::*` 造成向上依赖。Concrete implementations
