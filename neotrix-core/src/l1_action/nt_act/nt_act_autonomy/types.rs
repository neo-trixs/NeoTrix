/// L1-local type equivalents for oracle gate evaluation.
///
/// These mirror the L5 `awareness_monitor` types to preserve the dependency
/// direction: L1 must NOT depend on L5. When L5 evolves, these stay stable
/// as the interface contract for the oracle gate.

/// Severity of a detected capability gap.
#[derive(Debug, Clone, PartialEq)]
pub enum GapSeverity {
    Critical,
    Significant,
    Moderate,
    Negligible,
}

/// A single capability gap entry.
#[derive(Debug, Clone)]
pub struct CapabilityGap {
    pub dimension: String,
    pub current: f64,
    pub required: f64,
    pub gap: f64,
    pub severity: GapSeverity,
}

/// Summary report of current capability awareness state.
#[derive(Debug, Clone)]
pub struct AwarenessReport {
    pub gaps: Vec<CapabilityGap>,
    pub total_gap: f64,
    pub critical_count: u32,
    pub significant_count: u32,
    pub recommended_focus: Vec<String>,
    pub overall_health: f64,
}
