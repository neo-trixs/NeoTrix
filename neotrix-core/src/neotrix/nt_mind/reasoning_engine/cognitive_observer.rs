//! Cognitive observer: meta-cognitive blind-spot detection for the reasoning process.
//!
//! This is the "observer watching the observer" — after each reasoning cycle,
//! the CognitiveEye analyzes recent thinking traces, strategy distribution,
//! attention profile, and context pressure to find blind spots in the
//! system's own reasoning. Findings are fed back into capability vectors
//! and attention/strategy regulators.
//!
//! Architecture:
//!   ReasoningEngine.reason() →
//!     cognitive_eye.observe(task, &self.brain.capability) →
//!       generates CognitiveBlindSpot[] →
//!         applies capability deltas +
//!         stores insights for ThinkingBridge consumption

use crate::core::nt_core_self::{AttentionDomain, ReflectionGrade, StrategyKind};
use crate::core::CapabilityVector;
use std::collections::HashMap;

/// What kind of blind spot was found
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlindSpotKind {
    /// >80% of recent reasoning uses the same strategy
    StrategyFixation,
    /// <2 attention heads active — narrow focus
    AttentionStarvation,
    /// Context window >85% full — potential recency bias
    ContextOverload,
    /// Trace grades declining over last 5 traces
    GradeDegradation,
    /// Key domain (e.g. SelfReflection, RiskAssessment) never activated
    DomainNeglect,
    /// No self-reflection traces in recent history
    LowReflection,
    /// Same error pattern repeated across traces
    ErrorRecurrence,
}

impl BlindSpotKind {
    pub fn label(&self) -> &str {
        match self {
            BlindSpotKind::StrategyFixation => "strategy_fixation",
            BlindSpotKind::AttentionStarvation => "attention_starvation",
            BlindSpotKind::ContextOverload => "context_overload",
            BlindSpotKind::GradeDegradation => "grade_degradation",
            BlindSpotKind::DomainNeglect => "domain_neglect",
            BlindSpotKind::LowReflection => "low_reflection",
            BlindSpotKind::ErrorRecurrence => "error_recurrence",
        }
    }

    pub fn severity(&self) -> u8 {
        match self {
            BlindSpotKind::GradeDegradation => 3,
            BlindSpotKind::ErrorRecurrence => 3,
            BlindSpotKind::StrategyFixation => 2,
            BlindSpotKind::AttentionStarvation => 2,
            BlindSpotKind::ContextOverload => 2,
            BlindSpotKind::LowReflection => 1,
            BlindSpotKind::DomainNeglect => 1,
        }
    }
}

/// A concrete blind-spot finding with repair guidance
#[derive(Debug, Clone)]
pub struct CognitiveBlindSpot {
    pub kind: BlindSpotKind,
    pub description: String,
    pub severity: u8,
    pub repair: String,
    /// Dimension → delta to apply to CapabilityVector
    pub capability_deltas: Vec<(String, f64)>,
    /// Attention domain → stimulation amount
    pub attention_stimuli: Vec<(AttentionDomain, f64)>,
    /// Strategy → effectiveness boost
    pub strategy_boosts: Vec<(StrategyKind, f64)>,
}

/// Snapshot of cognitive state at one point in time
#[derive(Debug, Clone)]
pub struct CognitiveSnapshot {
    pub strategy_distribution: HashMap<StrategyKind, usize>,
    pub attention_domains_active: Vec<AttentionDomain>,
    pub context_usage_pct: f64,
    pub recent_trace_grades: Vec<ReflectionGrade>,
    pub recent_errors: Vec<String>,
    pub total_traces: usize,
    pub timestamp: i64,
}

/// The cognitive observer — tracks state over time and detects blind spots
pub struct CognitiveEye {
    pub history: Vec<CognitiveSnapshot>,
    pub findings: Vec<CognitiveBlindSpot>,
    pub total_observations: usize,
    pub enabled: bool,
}

impl CognitiveEye {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            findings: Vec::new(),
            total_observations: 0,
            enabled: true,
        }
    }

    /// Observe a reasoning cycle from available data.
    /// Called after each reasoning_engine.reason() completes, or from ThinkingBridge.
    pub fn observe(
        &mut self,
        strategy_distribution: HashMap<StrategyKind, usize>,
        attention_domains_active: Vec<AttentionDomain>,
        context_usage_pct: f64,
        recent_trace_grades: Vec<ReflectionGrade>,
        recent_errors: Vec<String>,
        capability: &CapabilityVector,
    ) -> Vec<CognitiveBlindSpot> {
        if !self.enabled {
            return Vec::new();
        }

        let snapshot = CognitiveSnapshot {
            strategy_distribution: strategy_distribution.clone(),
            attention_domains_active: attention_domains_active.clone(),
            context_usage_pct,
            recent_trace_grades: recent_trace_grades.clone(),
            recent_errors: recent_errors.clone(),
            total_traces: 0,
            timestamp: chrono::Utc::now().timestamp(),
        };

        self.history.push(snapshot);
        if self.history.len() > 50 {
            self.history.remove(0);
        }

        self.total_observations += 1;
        let mut spots = Vec::new();

        // 1. Strategy fixation detection
        if let Some(spot) = self.detect_strategy_fixation(&strategy_distribution) {
            spots.push(spot);
        }

        // 2. Attention starvation
        if attention_domains_active.len() < 2
            && self.history.len() >= 3
            && self
                .history
                .iter()
                .rev()
                .take(3)
                .all(|s| s.attention_domains_active.len() < 2)
        {
            spots.push(CognitiveBlindSpot {
                kind: BlindSpotKind::AttentionStarvation,
                description: "Consistently <2 active attention heads — reasoning is too narrow"
                    .into(),
                severity: 2,
                repair: "Stimulate underused domains: SelfReflection, RiskAssessment, Creativity"
                    .into(),
                capability_deltas: vec![("analysis".into(), 0.05), ("planning".into(), 0.03)],
                attention_stimuli: vec![
                    (AttentionDomain::SelfReflection, 0.3),
                    (AttentionDomain::RiskAssessment, 0.2),
                    (AttentionDomain::Creativity, 0.15),
                ],
                strategy_boosts: vec![],
            });
        }

        // 3. Context overload — detect at >85% or based on trace count pressure
        if context_usage_pct > 0.85 {
            spots.push(CognitiveBlindSpot {
                kind: BlindSpotKind::ContextOverload,
                description: format!(
                    "Context window {:.0}% full — risk of recency bias",
                    context_usage_pct * 100.0
                ),
                severity: 2,
                repair: "Trigger context window consolidation or reset".into(),
                capability_deltas: vec![("clarity".into(), -0.02)],
                attention_stimuli: vec![],
                strategy_boosts: vec![(StrategyKind::Reflection, 0.1)],
            });
        }

        // 4. Grade degradation
        if recent_trace_grades.len() >= 3 {
            let recent = recent_trace_grades
                .iter()
                .rev()
                .take(3)
                .map(|g| g.score())
                .collect::<Vec<_>>();
            if recent[0] < recent[1] && recent[1] < recent[2] {
                spots.push(CognitiveBlindSpot {
                    kind: BlindSpotKind::GradeDegradation,
                    description: "Last 3 trace grades declining — reasoning quality dropping"
                        .into(),
                    severity: 3,
                    repair: "Boost reflection and error-checking strategies".into(),
                    capability_deltas: vec![("debugging".into(), 0.08), ("analysis".into(), 0.05)],
                    attention_stimuli: vec![(AttentionDomain::SelfReflection, 0.4)],
                    strategy_boosts: vec![
                        (StrategyKind::Reflection, 0.15),
                        (StrategyKind::IterativeRefinement, 0.1),
                    ],
                });
            }
        }

        // 5. Domain neglect: check all 10 domains, flag those rarely active
        let all_domains = AttentionDomain::all();
        let neglected: Vec<&AttentionDomain> = all_domains
            .iter()
            .filter(|d| !attention_domains_active.contains(d))
            .collect();
        if neglected.len() >= 5 && self.history.len() >= 5 {
            let neglected_labels: Vec<String> =
                neglected.iter().map(|d| d.label().to_string()).collect();
            spots.push(CognitiveBlindSpot {
                kind: BlindSpotKind::DomainNeglect,
                description: format!(
                    "{} domains underused: {}",
                    neglected.len(),
                    neglected_labels.join(", ")
                ),
                severity: 1,
                repair: "Rotate attention to underused domains periodically".into(),
                capability_deltas: vec![("creativity".into(), 0.02), ("planning".into(), 0.02)],
                attention_stimuli: vec![
                    (AttentionDomain::Creativity, 0.2),
                    (AttentionDomain::GoalAlignment, 0.15),
                ],
                strategy_boosts: vec![],
            });
        }

        // 6. Low reflection
        let has_reflection = attention_domains_active.contains(&AttentionDomain::SelfReflection);
        if !has_reflection && self.history.len() >= 4 {
            let last_4 = &self.history[self.history.len().saturating_sub(4)..];
            if last_4.iter().all(|s| {
                !s.attention_domains_active
                    .contains(&AttentionDomain::SelfReflection)
            }) {
                spots.push(CognitiveBlindSpot {
                    kind: BlindSpotKind::LowReflection,
                    description: "No SelfReflection domain activation in last 4 cycles".into(),
                    severity: 1,
                    repair: "Force a reflection step after each reasoning cycle".into(),
                    capability_deltas: vec![("self_awareness".into(), 0.05)],
                    attention_stimuli: vec![(AttentionDomain::SelfReflection, 0.5)],
                    strategy_boosts: vec![(StrategyKind::Reflection, 0.1)],
                });
            }
        }

        // 7. Error recurrence
        if !recent_errors.is_empty() && self.history.len() >= 3 {
            let prev_errors: Vec<&str> = self
                .history
                .iter()
                .rev()
                .skip(1)
                .take(2)
                .flat_map(|s| s.recent_errors.iter().map(|e| e.as_str()))
                .collect();
            for error in &recent_errors {
                if prev_errors.contains(&error.as_str()) {
                    spots.push(CognitiveBlindSpot {
                        kind: BlindSpotKind::ErrorRecurrence,
                        description: format!("Recurring error: {}", error),
                        severity: 3,
                        repair: format!("Analyze root cause and add anti-pattern for: {}", error),
                        capability_deltas: vec![
                            ("debugging".into(), 0.1),
                            ("error_handling".into(), 0.08),
                        ],
                        attention_stimuli: vec![(AttentionDomain::RiskAssessment, 0.3)],
                        strategy_boosts: vec![(StrategyKind::IterativeRefinement, 0.1)],
                    });
                }
                break;
            }
        }

        // Apply capability deltas (simulated — caller decides whether to persist)
        for spot in &spots {
            for (dim_name, delta) in &spot.capability_deltas {
                if let Some(idx) = CapabilityVector::index_from_name(dim_name) {
                    let cur = capability.arr()[idx];
                    let _new_val = (cur + delta).clamp(0.0, 1.0);
                }
            }
        }

        self.findings.extend(spots.clone());
        if self.findings.len() > 200 {
            self.findings.drain(0..self.findings.len() - 200);
        }

        spots
    }

    /// Get the top-N most severe recent findings
    pub fn top_findings(&self, n: usize) -> Vec<&CognitiveBlindSpot> {
        let mut sorted: Vec<&CognitiveBlindSpot> = self.findings.iter().collect();
        sorted.sort_by_key(|b| std::cmp::Reverse(b.severity));
        sorted.truncate(n);
        sorted
    }

    pub fn findings_by_kind(&self, kind: BlindSpotKind) -> Vec<&CognitiveBlindSpot> {
        self.findings.iter().filter(|f| f.kind == kind).collect()
    }

    pub fn last_snapshot(&self) -> Option<&CognitiveSnapshot> {
        self.history.last()
    }

    pub fn reset(&mut self) {
        self.history.clear();
        self.findings.clear();
        self.total_observations = 0;
    }

    pub fn summary(&self) -> String {
        let by_kind: HashMap<BlindSpotKind, usize> =
            self.findings.iter().fold(HashMap::new(), |mut acc, f| {
                *acc.entry(f.kind).or_insert(0) += 1;
                acc
            });
        let mut parts: Vec<String> = by_kind
            .into_iter()
            .map(|(k, c)| format!("{}:{}", k.label(), c))
            .collect();
        parts.sort();
        format!(
            "CognitiveEye #{} | {} spots | [{}]",
            self.total_observations,
            self.findings.len(),
            parts.join(" ")
        )
    }

    // ─── private helpers ───

    fn detect_strategy_fixation(
        &self,
        distribution: &HashMap<StrategyKind, usize>,
    ) -> Option<CognitiveBlindSpot> {
        let total: usize = distribution.values().sum();
        if total < 4 {
            return None;
        }
        if total == 0 {
            return None;
        }
        for (kind, count) in distribution {
            let ratio = *count as f64 / total as f64;
            if ratio > 0.8 {
                return Some(CognitiveBlindSpot {
                    kind: BlindSpotKind::StrategyFixation,
                    description: format!("{:?} used in {:.0}% of recent traces — over-reliance", kind, ratio * 100.0),
                    severity: 2,
                    repair: "Boost alternative strategies: Reflection, IterativeRefinement, CompareAndContrast".to_string(),
                    capability_deltas: vec![("adaptability".into(), 0.05)],
                    attention_stimuli: vec![],
                    strategy_boosts: vec![
                        (StrategyKind::Reflection, 0.1),
                        (StrategyKind::CompareAndContrast, 0.08),
                        (StrategyKind::IterativeRefinement, 0.08),
                    ],
                });
            }
        }
        None
    }
}

impl Default for CognitiveEye {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observer_creates_snapshot() {
        let mut eye = CognitiveEye::new();
        let cap = CapabilityVector::default();
        let spots = eye.observe(
            HashMap::new(),
            vec![AttentionDomain::Code],
            0.5,
            vec![],
            vec![],
            &cap,
        );
        assert_eq!(eye.total_observations, 1);
        assert!(eye.last_snapshot().is_some());
        assert!(spots.is_empty());
    }

    #[test]
    fn test_strategy_fixation_detected() {
        let mut eye = CognitiveEye::new();
        let mut dist = HashMap::new();
        dist.insert(StrategyKind::Direct, 9);
        dist.insert(StrategyKind::Reflection, 1);
        let cap = CapabilityVector::default();
        let spots = eye.observe(dist, vec![AttentionDomain::Code], 0.5, vec![], vec![], &cap);
        assert!(
            spots
                .iter()
                .any(|s| matches!(s.kind, BlindSpotKind::StrategyFixation)),
            "expected strategy fixation, got: {:?}",
            spots.iter().map(|s| s.kind).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_attention_starvation_detected() {
        let mut eye = CognitiveEye::new();
        let cap = CapabilityVector::default();
        let dist: HashMap<StrategyKind, usize> = HashMap::new();
        eye.observe(
            dist.clone(),
            vec![AttentionDomain::Code],
            0.5,
            vec![],
            vec![],
            &cap,
        );
        eye.observe(
            dist.clone(),
            vec![AttentionDomain::Code],
            0.5,
            vec![],
            vec![],
            &cap,
        );
        let spots = eye.observe(dist, vec![AttentionDomain::Code], 0.5, vec![], vec![], &cap);
        assert!(spots
            .iter()
            .any(|s| matches!(s.kind, BlindSpotKind::AttentionStarvation)));
    }

    #[test]
    fn test_context_overload_detected() {
        let mut eye = CognitiveEye::new();
        let cap = CapabilityVector::default();
        let spots = eye.observe(
            HashMap::new(),
            vec![AttentionDomain::Code],
            0.9,
            vec![],
            vec![],
            &cap,
        );
        assert!(spots
            .iter()
            .any(|s| matches!(s.kind, BlindSpotKind::ContextOverload)));
    }

    #[test]
    fn test_grade_degradation_detected() {
        let mut eye = CognitiveEye::new();
        let cap = CapabilityVector::default();
        let grades = vec![
            ReflectionGrade::Excellent,
            ReflectionGrade::Good,
            ReflectionGrade::Adequate,
        ];
        let spots = eye.observe(
            HashMap::new(),
            vec![AttentionDomain::Code],
            0.5,
            grades,
            vec![],
            &cap,
        );
        assert!(spots
            .iter()
            .any(|s| matches!(s.kind, BlindSpotKind::GradeDegradation)));
    }

    #[test]
    fn test_error_recurrence_detected() {
        let mut eye = CognitiveEye::new();
        let cap = CapabilityVector::default();
        let dist: HashMap<StrategyKind, usize> = HashMap::new();
        eye.observe(
            dist.clone(),
            vec![AttentionDomain::Code],
            0.5,
            vec![],
            vec!["null pointer".to_string()],
            &cap,
        );
        eye.observe(
            dist.clone(),
            vec![AttentionDomain::Code],
            0.5,
            vec![],
            vec!["other error".to_string()],
            &cap,
        );
        let spots = eye.observe(
            dist,
            vec![AttentionDomain::Code],
            0.5,
            vec![],
            vec!["null pointer".to_string()],
            &cap,
        );
        assert!(spots
            .iter()
            .any(|s| matches!(s.kind, BlindSpotKind::ErrorRecurrence)));
    }

    #[test]
    fn test_low_reflection_detected() {
        let mut eye = CognitiveEye::new();
        let cap = CapabilityVector::default();
        let dist: HashMap<StrategyKind, usize> = HashMap::new();
        let grades = vec![ReflectionGrade::Good];
        for _ in 0..4 {
            eye.observe(
                dist.clone(),
                vec![AttentionDomain::Code, AttentionDomain::Planning],
                0.5,
                grades.clone(),
                vec![],
                &cap,
            );
        }
        let spots = eye.observe(
            dist,
            vec![AttentionDomain::Code, AttentionDomain::Planning],
            0.5,
            grades,
            vec![],
            &cap,
        );
        assert!(spots
            .iter()
            .any(|s| matches!(s.kind, BlindSpotKind::LowReflection)));
    }

    #[test]
    fn test_domain_neglect_detected() {
        let mut eye = CognitiveEye::new();
        let cap = CapabilityVector::default();
        let dist: HashMap<StrategyKind, usize> = HashMap::new();
        for _ in 0..5 {
            eye.observe(
                dist.clone(),
                vec![AttentionDomain::Code, AttentionDomain::Planning],
                0.5,
                vec![],
                vec![],
                &cap,
            );
        }
        let spots = eye.observe(
            dist,
            vec![AttentionDomain::Code, AttentionDomain::Planning],
            0.5,
            vec![],
            vec![],
            &cap,
        );
        assert!(spots
            .iter()
            .any(|s| matches!(s.kind, BlindSpotKind::DomainNeglect)));
    }

    #[test]
    fn test_disabled_observer_produces_no_spots() {
        let mut eye = CognitiveEye::new();
        eye.enabled = false;
        let cap = CapabilityVector::default();
        let spots = eye.observe(HashMap::new(), vec![], 0.9, vec![], vec![], &cap);
        assert!(spots.is_empty());
    }

    #[test]
    fn test_summary_format() {
        let eye = CognitiveEye::new();
        let summary = eye.summary();
        assert!(summary.contains("CognitiveEye"));
        assert!(summary.contains("spots"));
    }

    #[test]
    fn test_top_findings_ordered_by_severity() {
        let mut eye = CognitiveEye::new();
        let cap = CapabilityVector::default();
        let mut dist = HashMap::new();
        dist.insert(StrategyKind::Direct, 9);
        dist.insert(StrategyKind::Reflection, 1);
        let grades = vec![
            ReflectionGrade::Excellent,
            ReflectionGrade::Good,
            ReflectionGrade::Adequate,
        ];
        eye.observe(dist, vec![AttentionDomain::Code], 0.9, grades, vec![], &cap);
        let top = eye.top_findings(5);
        if top.len() >= 2 {
            assert!(top[0].severity >= top[1].severity);
        }
    }
}
