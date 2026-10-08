//! Ring Boundary - 边界环：隔离 + 熔断
//!
//! 遏制 + 熔断 + 逃逸检测

pub mod containment;
pub mod circuit_breaker;
pub mod escape_detector;

pub use containment::Containment;
pub use circuit_breaker::CircuitBreaker;
pub use escape_detector::EscapeDetector;

/// 边界环验证结果
#[derive(Debug, Clone)]
pub struct BoundaryVerification {
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
    pub contained: bool,
    pub circuit_state: CircuitState,
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
    pub escape_detected: bool,
    pub signals: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}
