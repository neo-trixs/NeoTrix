//! Ring Outer - 外环：输出过滤 + 行为监控
//!
//! 输出过滤 + 行为监控 + 共振检测 + 护栏哨兵

pub mod output_filter;
pub mod behavior_monitor;
pub mod resonance_detector;
pub mod guardrail_sentinel;

pub use output_filter::OutputFilter;
pub use behavior_monitor::BehaviorMonitor;
pub use resonance_detector::ResonanceDetector;
pub use guardrail_sentinel::GuardrailSentinel;

/// 外环验证结果
#[derive(Debug, Clone)]
pub struct OuterVerification {
    pub filtered_output: String,
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
    pub behavior_safe: bool,
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
    pub resonance_detected: bool,
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
    pub guardrail_intact: bool,
    pub signals: Vec<String>,
}
