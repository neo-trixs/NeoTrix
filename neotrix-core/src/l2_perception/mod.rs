pub mod nt_world;
pub use crate::l1_action::nt_core_llm;

/// L2 → L0 error conversions (moved from l0_substrate to respect L0 ← L2 direction)
///
/// ⚠️ 本文件此前**从未被编译**：`error_conversions.rs` 躺在
/// `l2_perception/` 下却没有任何 `mod.rs` 声明它（L1/L3/L5/L6 四层都有
/// 各自的 `error_conversions` 且都已声明，唯独 L2 漏了）。
/// ⇒ 三个 `From<…> for NeoTrixError` impl 全部未生效：
/// `asset_map::query::ParseError`、`social_access::traits::SocialAccessError`、
/// `source::offline_download::OfflineError`。
/// ⇒ 任何想用 `?` 自动转换这三类错误的代码都编译不过，只能显式 map。
pub mod error_conversions;

// 从 core/ 迁移的 L2 模块
pub mod nt_core_e8;
pub mod nt_core_e8_predictor;
pub mod nt_core_e8_vsa;
pub mod nt_core_hcube;
pub mod nt_core_sense;
pub mod nt_core_knowledge;
pub mod nt_core_vector_store;
pub mod nt_core_code_search;
/// EVO-05 有界符号调用图（symbols/calls/impact/explain，全有界 50）
pub mod nt_code_graph;
/// EVO-12 情报profile管线＋配对收益（rubric/去重/配额/PairedGap）
pub mod nt_intel_digest;
pub mod nt_routing;
pub mod nt_web_perception;
/// 对称阴影投射 FOV + 连续空间几何 LOS（2026-09-28 自 crates/neotrix-abilities 萃取；该 crate 已归档至
/// ~/Downloads/Neo/neotrix-archive/crates/neotrix-abilities/）。
/// 感知层能力：主代码 nt_game 无 FOV 实现，此前为真空；纯函数、零渲染、零 IO。
pub mod nt_fov;

