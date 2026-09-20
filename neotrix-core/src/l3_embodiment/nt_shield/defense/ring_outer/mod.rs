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
    pub behavior_safe: bool,
    pub resonance_detected: bool,
    pub guardrail_intact: bool,
    pub signals: Vec<String>,
}
