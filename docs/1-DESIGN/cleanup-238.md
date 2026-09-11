# cleanup-238 — 跨域引用分析

**日期**: 2026-09-11
**范围**: 6 层架构跨域 `use crate::*` 引用审计

## 原始数据

```
l5_cognition→l1_action: 2
l5_cognition→l2_perception: 1
l5_cognition→l6_meta: 2
```

## 逐行审查

### L5→L1 (认知→行动): 2 处

| 文件 | 行 | 内容 | 状态 |
|------|-----|------|------|
| `nt_mind/evolution/self_diagnose.rs` | 32 | `// pub use crate::l1_action::nt_act::nt_l1_shared_types::{...}` | **已注释** |
| `nt_mind/evolution/evolution_loop.rs` | 21 | `// pub use crate::l1_action::nt_act::nt_l1_shared_types::IssueType;` | **已注释** |

### L5→L2 (认知→感知): 1 处

| 文件 | 行 | 内容 | 状态 |
|------|-----|------|------|
| `l5_cognition/mod.rs` | 16 | `/// L2 类型通过此模块访问, 避免散布 use crate::l2_perception::*。` | **文档注释** |

### L5→L6 (认知→元认知): 2 处

| 文件 | 行 | 内容 | 状态 |
|------|-----|------|------|
| `l5_cognition/mod.rs` | 22 | `/// L6 类型通过此模块访问, 避免散布 use crate::l6_meta::*。` | **文档注释** |
| `l5_cognition/traits.rs` | 137 | `// use crate::l6_meta::* 造成向上依赖。Concrete implementations` | **已注释** |

## 结论

**0 处活跃跨域引用** — 所有 5 处匹配均为注释/文档，非活跃代码。

### Facade 模式运行良好

跨域访问通过 `l*_facade.rs` 受控转发：

| Facade 文件 | 服务层 | 引用目标 |
|-------------|--------|----------|
| `l2_perception/nt_world/l1_facade.rs` | L2→L1 | nt_memory_kb, nt_io_http_factory |
| `l3_embodiment/l1_facade.rs` | L3→L1 | nt_io_provider, nt_memory_kb, nt_act_cleanup |
| `l5_cognition/kb_facade.rs` | L5→L1 | nt_memory_kb (知识层) |
| `l5_cognition/io_facade.rs` | L5→L1 | nt_io_provider, nt_io_standalone |
| `l5_cognition/io_skills_facade.rs` | L5→L1 | nt_io_* 技能模块 |
| `l5_cognition/act_facade.rs` | L5→L1 | nt_act_* 行动模块 |
| `l5_cognition/l2_facade.rs` | L5→L2 | nt_world_* 感知模块 |
| `l5_cognition/l3_facade.rs` | L5→L3 | nt_shield_* 安全审计 |
| `l5_cognition/l6_facade.rs` | L5→L6 | nt_repair, memory 模块 |
| `l6_meta/l1_facade.rs` | L6→L1 | nt_act_cleanup 共享类型 |

## 判定

| 维度 | 状态 |
|------|------|
| 跨域引用违规 | **无** (0 活跃违规) |
| Facade 模式一致性 | **良好** — 所有跨层访问经 facade 转发 |
| 注释代码清理 | 可选 — 2 处注释掉的 `pub use` 可删除 |
