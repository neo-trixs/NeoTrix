# Cleanup-225: L5→L2 跨域引用修复

**日期**: 2026-09-11
**类型**: 架构清理 (跨层依赖集中化)
**影响范围**: L5 Cognition → L2 Perception

## 问题

L5 认知层直接引用 L2 感知层具体类型，违反六层架构约束。散布的 `use crate::l2_perception::*` 导致跨层依赖不可审计。

## 原始引用 (5 处)

| # | 文件 | 行 | 引用 |
|---|------|-----|------|
| 1 | `l5_cognition/nt_mind/nt_mind_background_loop/mod.rs` | 15 | `l2_perception::nt_world::nt_world_model_v2::WorldModelV2` |
| 2 | `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 642 | `l2_perception::nt_world::nt_world_novel::{drain_novel_queue, classify_unanalyzed_books}` |
| 3 | `l5_cognition/nt_mind/nt_mind/consciousness/panorama_pipeline.rs` | 9 | `l2_perception::nt_world::nt_world_model_v2::WorldModelV2` |
| 4 | `l5_cognition/nt_mind/nt_mind/consciousness/panorama_pipeline.rs` | 298 | `l2_perception::nt_world::nt_world_model_v2::WorldModelV2` (test) |
| 5 | `l5_cognition/nt_mind/nt_mind/evolution/agent_capability/mod.rs` | 21 | `l2_perception::nt_world::nt_world_search::{SearchResult, UnifiedSearch}` |

额外修复: `run.rs:1198` 通过 `crate::neotrix::nt_world_model_v2` 的间接引用也一并收编。

## 修复方案

创建 `l5_cognition/l2_facade.rs` — L5 对 L2 感知层类型的集中 re-export 门面。

遵循既有模式:
- `kb_facade.rs` — L5 → L1 NT-MEMORY
- `l3_facade.rs` — L5 → L3 具身层
- `l6_facade.rs` — L5 → L6 元认知层
- **`l2_facade.rs`** — L5 → L2 感知层 (本次新增)

## 变更文件

| 文件 | 变更 |
|------|------|
| `l5_cognition/l2_facade.rs` | **新建** — re-export `WorldModelV2`, `SearchResult`, `UnifiedSearch`, `drain_novel_queue`, `classify_unanalyzed_books`, `QidianIngestReport` |
| `l5_cognition/mod.rs` | 注册 `pub mod l2_facade` |
| `l5_cognition/nt_mind/nt_mind_background_loop/mod.rs` | `use crate::l2_perception::*` → `use crate::l5_cognition::l2_facade::*` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 同上 |
| `l5_cognition/nt_mind/nt_mind_background_loop/run.rs` | `use crate::neotrix::nt_world_model_v2::*` → `use crate::l5_cognition::l2_facade::*` (test) |
| `l5_cognition/nt_mind/nt_mind/consciousness/panorama_pipeline.rs` | 含测试共 2 处 |
| `l5_cognition/nt_mind/nt_mind/evolution/agent_capability/mod.rs` | `SearchResult`, `UnifiedSearch` |

## Facade 内容

```rust
// l5_cognition/l2_facade.rs
pub use crate::l2_perception::nt_world::nt_world_model_v2::WorldModelV2;
pub use crate::l2_perception::nt_world::nt_world_search::{SearchResult, UnifiedSearch};
pub use crate::l2_perception::nt_world::nt_world_novel::{
    drain_novel_queue, classify_unanalyzed_books, QidianIngestReport,
};
```

## 验证

- `cargo check --lib -p neotrix` 通过 (无新增错误)
- `grep -rn "use crate::l2_perception" l5_cognition/ --include="*.rs" | grep -v test | grep -v facade` 返回 0 条
- L2 引用仅存在于 `l2_facade.rs` (单一事实源)

## 后续

- L2→L5 trait 抽象可进一步解耦 (如 `WorldModelApi` trait)，但需权衡重构成本
- 当前 facade 方案与既有 `kb_facade`/`l3_facade`/`l6_facade` 保持一致
