=== 跨域引用 ===
l5_cognition→l1_action:        4
l5_cognition→l2_perception:        1
l5_cognition→l6_meta:        2

=== 分析 ===
实际导入违规 (非注释):
1. l5_cognition/nt_mind/evolution/self_diagnose.rs:10
   use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;

2. l5_cognition/nt_mind/evolution/evolution_loop.rs:67
   pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;

注释中的警告 (非违规):
- l2_perception:16, l6_meta:22, l6_meta:137 都是说明避免导入的注释

结论: L5→L1 下行依赖实际为 2 处，均涉及 ProjectSnapshot 类型