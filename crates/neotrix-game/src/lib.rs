//! neotrix-game 引擎库 — 纯运行时框架（purge-round17 后仅留引擎）.
//!
//! 游戏域（codex/quest/奇门遁甲/背包/ECOLOGY/对话/AI/存档…）已全部删除，
//! 备份见 `_archive/purge-round17/`；游戏体重建于 `games/neotrix-guixu`。
//! WASM 入口亦走此 lib。

#![forbid(unsafe_code)]

pub mod nt_juice;
pub mod nt_clock;
pub mod nt_camera;
pub mod nt_move;
pub mod nt_platform;
pub mod lighting;
pub mod particles;
pub mod audio;
pub mod events;
pub mod state_stack;
pub mod ecs;
pub mod nt_behavior;
pub mod ui;
pub mod input;
pub mod components;
pub mod tween;
pub mod ecs_systems;
// ── 三引擎吸收（2026-09-25）：KAPLAY / OpenWarcraft3 / Pokerogue ──
pub mod nt_object;
pub mod nt_timer;
pub mod nt_tilemap;
pub mod nt_commands;
pub mod nt_grid;
pub mod nt_phase;
pub mod nt_run;
pub mod nt_battle;
// ── 九源吸收（2026-09-26）：kaplay SpriteAnims + bevy Atlas ──
pub mod nt_anim;
// ── 缺陷并行补齐（2026-09-26）：空间分区 + 相机变换 + 发射器 ──
pub mod nt_spatial;
