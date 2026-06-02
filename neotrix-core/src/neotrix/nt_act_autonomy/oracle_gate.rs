use crate::neotrix::nt_act_autonomy::awareness_monitor::{AwarenessReport, GapSeverity};

#[derive(Debug, Clone, PartialEq)]
pub enum OracleReason {
    CriticalCapabilityGap { dimension: String, gap: f64 },
    UnknownTask { task_description: String },
    RepeatedFailure { attempt_count: u32, dimension: String },
    ArchitectureDecision { description: String },
}

#[derive(Debug, Clone)]
pub struct OracleRequest {
    pub id: String,
    pub reason: OracleReason,
    pub context: String,
    pub urgency: OracleUrgency,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OracleUrgency {
    Immediate,
    High,
    Normal,
    Low,
}

#[derive(Debug, Clone)]
pub struct OracleDecision {
    pub needs_oracle: bool,
    pub reason: Option<OracleReason>,
    pub request: Option<OracleRequest>,
    pub suggested_action: String,
}

#[derive(Debug, Clone)]
pub struct OracleGate {
    max_critical_gaps_before_oracle: u32,
    max_attempts_before_oracle: u32,
    oracle_call_count: u64,
}

impl OracleGate {
    pub fn new() -> Self {
        Self {
            max_critical_gaps_before_oracle: 1,
            max_attempts_before_oracle: 3,
            oracle_call_count: 0,
        }
    }

    pub fn evaluate(&mut self, report: &AwarenessReport, _task_description: &str) -> OracleDecision {
        if report.critical_count > self.max_critical_gaps_before_oracle {
            let gap = report.gaps.iter().find(|g| g.severity == GapSeverity::Critical);
            let reason = gap.map(|g| OracleReason::CriticalCapabilityGap {
                dimension: g.dimension.clone(),
                gap: g.gap,
            });
            return OracleDecision {
                needs_oracle: true,
                reason: reason.clone(),
                request: reason.map(|r| OracleRequest {
                    id: format!("oracle-{}", self.oracle_call_count + 1),
                    reason: r,
                    context: format!("{} critical gaps detected", report.critical_count),
                    urgency: OracleUrgency::Immediate,
                    created_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                }),
                suggested_action: "Block until resolved".into(),
            };
        }

        if report.critical_count > 0 {
            let gap = report.gaps.iter().find(|g| g.severity == GapSeverity::Critical);
            let reason = gap.map(|g| OracleReason::CriticalCapabilityGap {
                dimension: g.dimension.clone(),
                gap: g.gap,
            });
            return OracleDecision {
                needs_oracle: true,
                reason: reason.clone(),
                request: reason.map(|r| OracleRequest {
                    id: format!("oracle-{}", self.oracle_call_count + 1),
                    reason: r,
                    context: format!("{} critical gaps, within threshold", report.critical_count),
                    urgency: OracleUrgency::High,
                    created_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                }),
                suggested_action: "Continue but prioritize gap resolution".into(),
            };
        }

        let complex_keywords = ["refactor", "redesign", "architecture", "new feature"];
        if complex_keywords.iter().any(|kw| _task_description.to_lowercase().contains(kw)) {
            return OracleDecision {
                needs_oracle: true,
                reason: Some(OracleReason::UnknownTask {
                    task_description: _task_description.to_string(),
                }),
                request: Some(OracleRequest {
                    id: format!("oracle-{}", self.oracle_call_count + 1),
                    reason: OracleReason::UnknownTask {
                        task_description: _task_description.to_string(),
                    },
                    context: "Complex task identified".into(),
                    urgency: OracleUrgency::Normal,
                    created_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                }),
                suggested_action: "Try internal auto-fix first, then escalate".into(),
            };
        }

        OracleDecision {
            needs_oracle: false,
            reason: None,
            request: None,
            suggested_action: "Proceed autonomously".into(),
        }
    }

    pub fn evaluate_failure(&mut self, attempt_count: u32, dimension: &str) -> OracleDecision {
        if attempt_count >= self.max_attempts_before_oracle {
            let reason = OracleReason::RepeatedFailure {
                attempt_count,
                dimension: dimension.to_string(),
            };
            return OracleDecision {
                needs_oracle: true,
                reason: Some(reason.clone()),
                request: Some(OracleRequest {
                    id: format!("oracle-{}", self.oracle_call_count + 1),
                    reason,
                    context: format!("Failed {} times on {}", attempt_count, dimension),
                    urgency: OracleUrgency::High,
                    created_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                }),
                suggested_action: "Escalate to oracle with failure context".into(),
            };
        }

        OracleDecision {
            needs_oracle: false,
            reason: None,
            request: None,
            suggested_action: format!("Retry autonomously (attempt {}/{})", attempt_count, self.max_attempts_before_oracle),
        }
    }

    pub fn evaluate_architecture(&mut self, description: &str) -> OracleDecision {
        let reason = OracleReason::ArchitectureDecision {
            description: description.to_string(),
        };
        OracleDecision {
            needs_oracle: true,
            reason: Some(reason.clone()),
            request: Some(OracleRequest {
                id: format!("oracle-{}", self.oracle_call_count + 1),
                reason,
                context: "Architecture decision requires human input".into(),
                urgency: OracleUrgency::Immediate,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            }),
            suggested_action: "Present options to human for decision".into(),
        }
    }

    pub fn record_oracle_call(&mut self) {
        self.oracle_call_count += 1;
    }

    pub fn reset(&mut self) {
        self.oracle_call_count = 0;
    }

    pub fn summary(&self) -> String {
        format!(
            "OracleGate{{ oracle_calls={}, max_critical={}, max_attempts={} }}",
            self.oracle_call_count,
            self.max_critical_gaps_before_oracle,
            self.max_attempts_before_oracle,
        )
    }
}

impl Default for OracleGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_act_autonomy::awareness_monitor::{AwarenessReport, CapabilityGap, GapSeverity};

    fn make_report(critical_count: u32) -> AwarenessReport {
        let gaps = if critical_count > 0 {
            (0..critical_count)
                .map(|i| CapabilityGap {
                    dimension: format!("dim_{}", i),
                    current: 0.1,
                    required: 0.9,
                    gap: 0.8,
                    severity: GapSeverity::Critical,
                })
                .collect()
        } else {
            vec![CapabilityGap {
                dimension: "dim_0".into(),
                current: 0.8,
                required: 0.85,
                gap: 0.05,
                severity: GapSeverity::Negligible,
            }]
        };
        AwarenessReport {
            gaps,
            total_gap: critical_count as f64 * 0.8,
            critical_count,
            significant_count: 0,
            recommended_focus: vec![],
            overall_health: 0.9,
        }
    }

    #[test]
    fn test_critical_gap_triggers_oracle() {
        let mut gate = OracleGate::new();
        let report = make_report(2);
        let decision = gate.evaluate(&report, "simple task");
        assert!(decision.needs_oracle);
        assert!(matches!(decision.reason, Some(OracleReason::CriticalCapabilityGap { .. })));
        assert_eq!(decision.request.as_ref().unwrap().urgency, OracleUrgency::Immediate);
    }

    #[test]
    fn test_critical_gap_under_threshold_no_oracle() {
        let mut gate = OracleGate::new();
        gate.max_critical_gaps_before_oracle = 3;
        let report = make_report(2);
        let decision = gate.evaluate(&report, "simple task");
        assert!(decision.needs_oracle);
        assert_eq!(decision.request.as_ref().unwrap().urgency, OracleUrgency::High);
    }

    #[test]
    fn test_no_gaps_no_oracle() {
        let mut gate = OracleGate::new();
        let report = make_report(0);
        let decision = gate.evaluate(&report, "simple task");
        assert!(!decision.needs_oracle);
        assert!(decision.reason.is_none());
    }

    #[test]
    fn test_complex_task_triggers_oracle() {
        let mut gate = OracleGate::new();
        let report = make_report(0);
        let decision = gate.evaluate(&report, "refactor the core module");
        assert!(decision.needs_oracle);
        assert!(matches!(decision.reason, Some(OracleReason::UnknownTask { .. })));
        assert_eq!(decision.request.as_ref().unwrap().urgency, OracleUrgency::Normal);
    }

    #[test]
    fn test_repeated_failure_triggers_oracle() {
        let mut gate = OracleGate::new();
        let decision = gate.evaluate_failure(3, "reasoning_quality");
        assert!(decision.needs_oracle);
        assert!(matches!(decision.reason, Some(OracleReason::RepeatedFailure { attempt_count: 3, .. })));
    }

    #[test]
    fn test_repeated_failure_below_threshold() {
        let mut gate = OracleGate::new();
        let decision = gate.evaluate_failure(2, "reasoning_quality");
        assert!(!decision.needs_oracle);
        assert!(decision.suggested_action.contains("Retry"));
    }

    #[test]
    fn test_architecture_always_triggers() {
        let mut gate = OracleGate::new();
        let decision = gate.evaluate_architecture("Should we migrate to event-driven?");
        assert!(decision.needs_oracle);
        assert!(matches!(decision.reason, Some(OracleReason::ArchitectureDecision { .. })));
        assert_eq!(decision.request.as_ref().unwrap().urgency, OracleUrgency::Immediate);
    }

    #[test]
    fn test_record_oracle_call_increments_counter() {
        let mut gate = OracleGate::new();
        assert_eq!(gate.oracle_call_count, 0);
        gate.record_oracle_call();
        assert_eq!(gate.oracle_call_count, 1);
        gate.record_oracle_call();
        assert_eq!(gate.oracle_call_count, 2);
    }

    #[test]
    fn test_summary_format() {
        let gate = OracleGate::new();
        let s = gate.summary();
        assert!(s.contains("OracleGate{"));
        assert!(s.contains("oracle_calls="));
        assert!(s.contains("max_critical="));
        assert!(s.contains("max_attempts="));
    }

    #[test]
    fn test_reset_clears_counter() {
        let mut gate = OracleGate::new();
        gate.record_oracle_call();
        gate.record_oracle_call();
        gate.record_oracle_call();
        assert_eq!(gate.oracle_call_count, 3);
        gate.reset();
        assert_eq!(gate.oracle_call_count, 0);
    }

    #[test]
    fn test_default_construction() {
        let gate = OracleGate::default();
        assert_eq!(gate.max_critical_gaps_before_oracle, 1);
        assert_eq!(gate.max_attempts_before_oracle, 3);
        assert_eq!(gate.oracle_call_count, 0);
    }
}
