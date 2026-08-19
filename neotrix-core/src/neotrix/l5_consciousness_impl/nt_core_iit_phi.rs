//! IIT Φ 度量 — D3 兼容层
//!
//! 计算器本体 (纯逻辑, 无外部依赖) 已下沉至 core `nt_core_iit_phi`;
//! 本模块 re-export 保持 `neotrix::nt_core_iit_phi::*` 调用方路径不变。

pub use crate::core::nt_core_iit_phi::{
    IITPhiCalculator, PhiReport, PHI_HISTORY_WINDOW, PHI_MIN_THRESHOLD, PHI_RESONANCE_SIGMA,
};
