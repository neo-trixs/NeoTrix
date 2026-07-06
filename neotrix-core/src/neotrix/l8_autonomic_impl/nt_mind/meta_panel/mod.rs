//! Meta-Panel Reasoning — 多视角推理面板
//!
//! 将 E8 推理状态 + GWT 专家类型 + PerspectiveLens 融合为多视角分析。
//! 灵感: UZI-Skill 的 66 评委/9 流派面板 → NeoTrix 元推理层。
//!
//! 3 级深度:
//!   - Lite:  3 视角, 自检门, 快速结论
//!   - Mid:  12 视角, EWHR 加权融合, 完整报告
//!   - Deep: 24 视角, Bull/Bear 辩论, 深度自检

mod types;
mod fusion;
mod engine;

pub use types::{AnalysisDepth, FusionResult, MetaPanelResult, Viewpoint};
pub use fusion::FusionEngine;
pub use engine::MetaPanelEngine;

// ─── Constructor shortcut ───
pub fn meta_panel(depth: AnalysisDepth) -> MetaPanelEngine {
    MetaPanelEngine::new(depth, true)
}

pub fn meta_panel_lite() -> MetaPanelEngine {
    MetaPanelEngine::new(AnalysisDepth::Lite, false)
}

pub fn meta_panel_mid() -> MetaPanelEngine {
    MetaPanelEngine::new(AnalysisDepth::Mid, true)
}

pub fn meta_panel_deep() -> MetaPanelEngine {
    MetaPanelEngine::new(AnalysisDepth::Deep, true)
}
