//! NT-GAME 的游戏级实体管理 — **不是** L0 `nt_ecs` 的重复实现。
//!
//! 2026-09-28 裁决（曾被登记为"重复债务"，经取证前提有误，在此划清作用域）：
//! - L0 `l0_substrate/nt_ecs.rs`（1,362 行）= **存储基座**：`ArchetypeId`、
//!   `UniversalEntity`(id+generation+archetype)、`ComponentStorage`
//!   (Dense/Sparse/**SoA**)，服务全系统，无游戏语义。
//! - 本模块（291 行）= **游戏层实体**：带 `alive` 生死位、`tags` 标签系统、
//!   带 `priority` 的 `SystemScheduler` 优先级调度，以及供 `render/scene.rs`
//!   使用的场景图父子结构。以上 L0 **均无**。
//! 唯一重叠是 `EntityId` 这个**名字**，且类型不同：L0 是 newtype
//! `EntityId(pub u64)`（可与 generation 组合），此处是别名 `= u64`。
//! 故**不合并、不删除**；若日后 L0 补上 tag 与优先级调度，再评估收敛。
//!
//! ⚠️ 另见 `crates/neotrix-game/src/ecs.rs`(455 行 component-map)：那个才是
//! 真冗余（被 L0 `nt_ecs.rs` 完整取代），随该 crate 归档时一并处理。

pub mod component;
pub mod entity;
pub mod registry;
pub mod system;

pub use component::{Component, ComponentStore};
pub use entity::{EntityId, EntityManager};
pub use registry::World;
pub use system::{System, SystemScheduler};
