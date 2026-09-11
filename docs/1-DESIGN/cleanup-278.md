# 跨域引用检查 — 2026-09-11

## 摘要

| 源域 → 目标域 | 引用数 | 状态 |
|---|---|---|
| l5_cognition → l1_action | 2 | ⚠️ 直接引用，绕过 act_facade |
| l5_cognition → l2_perception | 0 | ✅ 已通过 l2_facade 集中化 |
| l5_cognition → l3_embodiment | 0 | ✅ 已通过 l3_facade 集中化 |
| l5_cognition → l6_meta | 0 | ✅ 已通过 l6_facade 集中化 |
| 其余跨域 | 0 | ✅ 无违规 |

## 活跃违规详情

### l5_cognition → l1_action (2 处)

| 文件 | 行号 | 引用内容 |
|---|---|---|
| `l5_cognition/nt_mind/evolution/self_diagnose.rs` | 10 | `use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;` |
| `l5_cognition/nt_mind/evolution/evolution_loop.rs` | 67 | `pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;` |

**问题**: `ProjectSnapshot` 直接从 `l1_action` 导入，绕过已有的 `act_facade` 集中化层。

**修复方案**: 在 `act_facade.rs` 中 re-export `ProjectSnapshot`，两处改为 `use crate::l5_cognition::act_facade::ProjectSnapshot`。

## Facade 覆盖情况

| Facade | 文件 | 覆盖目标 | 状态 |
|---|---|---|---|
| `act_facade.rs` | l5_cognition/ | l1_action (nt_act) | ✅ 存在，2 处未走此路径 |
| `l2_facade.rs` | l5_cognition/ | l2_perception (nt_world) | ✅ 完全覆盖 |
| `l3_facade.rs` | l5_cognition/ | l3_embodiment (nt_physical) | ✅ 完全覆盖 |
| `l6_facade.rs` | l5_cognition/ | l6_meta (nt_meta) | ✅ 完全覆盖 |
| `kb_facade.rs` | l5_cognition/ | nt_memory (KB) | ✅ 完全覆盖 |
| `io_facade.rs` | l5_cognition/ | l1_action (nt_io) | ✅ 完全覆盖 |
| `io_skills_facade.rs` | l5_cognition/ | L3 厂商技能 | ✅ 完全覆盖 |

## 修复计划

```
Step 1: 在 act_facade.rs 添加 re-export
        pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;

Step 2: self_diagnose.rs:10 改为
        use crate::l5_cognition::act_facade::ProjectSnapshot;

Step 3: evolution_loop.rs:67 改为
        pub use crate::l5_cognition::act_facade::ProjectSnapshot;

Step 4: cargo check --lib -p neotrix 验证
```

## 结论

架构分层基本干净。仅 `l5_cognition → l1_action` 存在 2 处直接引用绕过 facade，可一次性修复。其余跨域引用已全部通过 facade 集中化。
