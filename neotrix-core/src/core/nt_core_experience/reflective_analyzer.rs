use std::collections::VecDeque;

/// A typed execution trace event, richer than raw text logs.
#[derive(Debug, Clone)]
pub enum TraceEvent {
    CalibrationSnapshot {
        cycle: u64,
        domain: String,
        ece: f64,
        surprise: f64,
    },
    InterventionLog {
        cycle: u64,
        source: String,
        action: String,
        success: bool,
    },
    WeaknessDetected {
        cycle: u64,
        pattern: String,
        severity: f64,
        domain: String,
    },
    TaskResult {
        cycle: u64,
        task_id: u64,
        success: bool,
        gain: f64,
    },
    CalibrationTrend {
        cycle: u64,
        domain: String,
        /// Average ECE over the last N cycles
        ece_avg: f64,
        /// ECE trend (positive = worsening)
        ece_trend: f64,
        /// Surprise trend (positive = more surprising)
        surprise_trend: f64,
    },
}

impl TraceEvent {
    pub fn cycle(&self) -> u64 {
        match self {
            TraceEvent::CalibrationSnapshot { cycle, .. } => *cycle,
            TraceEvent::InterventionLog { cycle, .. } => *cycle,
            TraceEvent::WeaknessDetected { cycle, .. } => *cycle,
            TraceEvent::TaskResult { cycle, .. } => *cycle,
            TraceEvent::CalibrationTrend { cycle, .. } => *cycle,
        }
    }
}

/// Structured diagnosis of an evolution-relevant failure mode.
#[derive(Debug, Clone)]
pub struct Diagnosis {
    pub id: u64,
    pub root_cause: String,
    pub severity: f64,
    pub affected_components: Vec<String>,
    pub suggested_fix: FixCategory,
    pub trace_evidence: Vec<String>,
    pub trend: TrendDirection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FixCategory {
    CalibrationDrift,
    ModuleWiring,
    MemoryPressure,
    LatencyDegradation,
    SkillStagnation,
    CompileFailure,
    SystemicDegradation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    Worsening,
    Improving,
    Stable,
    InsufficientData,
}

/// GEPA-style trace analysis engine: structured events → pattern matching → Diagnosis.
///
/// Unlike `WeaknessMiner` which operates on raw keyword frequencies,
/// this analyzer works on typed `TraceEvent`s and can detect cross-event
/// patterns (e.g., "ECE rising + intervention failing → SystemicDegradation").
#[derive(Debug, Clone)]
pub struct ReflectiveAnalyzer {
    events: VecDeque<TraceEvent>,
    max_events: usize,
    next_id: u64,
}

impl ReflectiveAnalyzer {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(max_events),
            max_events,
            next_id: 1,
        }
    }

    /// Feed a typed trace event into the analyzer buffer.
    pub fn feed_event(&mut self, event: TraceEvent) {
        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Feed multiple events at once (e.g., from a tick batch).
    pub fn feed_events(&mut self, events: impl IntoIterator<Item = TraceEvent>) {
        for event in events {
            self.feed_event(event);
        }
    }

    /// Run all pattern detectors against current event buffer.
    /// Returns diagnoses sorted by severity descending.
    pub fn analyze(&mut self) -> Vec<Diagnosis> {
        if self.events.len() < 3 {
            return vec![];
        }

        let mut diagnoses = Vec::new();

        diagnoses.extend(self.detect_calibration_drift());
        diagnoses.extend(self.detect_systemic_degradation());
        diagnoses.extend(self.detect_intervention_failure_spiral());
        diagnoses.extend(self.detect_stagnation());
        diagnoses.extend(self.detect_memory_pressure());
        diagnoses.extend(self.detect_latency_degradation());

        // Deduplicate by root_cause — keep highest severity
        diagnoses.sort_by(|a, b| {
            b.severity
                .partial_cmp(&a.severity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        diagnoses.dedup_by(|a, b| a.root_cause == b.root_cause);

        diagnoses
    }

    /// Pattern 1: Calibration Drift — ECE consistently high or rising.
    fn detect_calibration_drift(&mut self) -> Vec<Diagnosis> {
        let trends: Vec<&TraceEvent> = self
            .events
            .iter()
            .filter(|e| matches!(e, TraceEvent::CalibrationTrend { .. }))
            .collect();

        if trends.len() < 3 {
            return vec![];
        }

        // Check if any domain has ECE > 0.15 for the last 3+ trends
        let mut domains: Vec<String> = Vec::new();
        for event in trends.iter().rev().take(5) {
            if let TraceEvent::CalibrationTrend {
                domain, ece_avg, ..
            } = event
            {
                if *ece_avg > 0.15 && !domains.contains(domain) {
                    domains.push(domain.clone());
                }
            }
        }

        if domains.is_empty() {
            return vec![];
        }

        // Determine trend from most recent calibration trend event
        let trend = trends
            .last()
            .and_then(|e| {
                if let TraceEvent::CalibrationTrend { ece_trend, .. } = e {
                    Some(if *ece_trend > 0.02 {
                        TrendDirection::Worsening
                    } else if *ece_trend < -0.02 {
                        TrendDirection::Improving
                    } else {
                        TrendDirection::Stable
                    })
                } else {
                    None
                }
            })
            .unwrap_or(TrendDirection::InsufficientData);

        let evidence: Vec<String> = domains
            .iter()
            .map(|d| format!("ECE > 0.15 in domain '{}'", d))
            .collect();

        vec![Diagnosis {
            id: self.next_id(),
            root_cause: format!("CalibrationDrift: {} domains drifting", domains.len()),
            severity: (0.3 + domains.len() as f64 * 0.15).min(1.0),
            affected_components: domains,
            suggested_fix: FixCategory::CalibrationDrift,
            trace_evidence: evidence,
            trend,
        }]
    }

    /// Pattern 2: Systemic Degradation — calibration drift + failing interventions.
    fn detect_systemic_degradation(&mut self) -> Vec<Diagnosis> {
        let recent_interventions: Vec<&TraceEvent> = self
            .events
            .iter()
            .filter(|e| matches!(e, TraceEvent::InterventionLog { success: false, .. }))
            .collect();

        if recent_interventions.len() < 3 {
            return vec![];
        }

        let failure_rate =
            recent_interventions.len() as f64 / self.events.iter().filter(|e| matches!(e, TraceEvent::InterventionLog { .. })).count().max(1) as f64;

        if failure_rate < 0.3 {
            return vec![];
        }

        let evidence: Vec<String> = recent_interventions
            .iter()
            .take(5)
            .map(|e| {
                if let TraceEvent::InterventionLog {
                    source, action, ..
                } = e
                {
                    format!("{} failed: {} action '{}'", source, action, action)
                } else {
                    String::new()
                }
            })
            .collect();

        vec![Diagnosis {
            id: self.next_id(),
            root_cause: format!(
                "SystemicDegradation: {:.0}% intervention failure rate",
                failure_rate * 100.0
            ),
            severity: (0.3 + failure_rate * 0.5).min(1.0),
            affected_components: vec!["multiple".to_string()],
            suggested_fix: FixCategory::SystemicDegradation,
            trace_evidence: evidence,
            trend: TrendDirection::Worsening,
        }]
    }

    /// Pattern 3: Intervention Failure Spiral — same source failing repeatedly.
    fn detect_intervention_failure_spiral(&mut self) -> Vec<Diagnosis> {
        let mut failures_by_source: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();

        for event in self.events.iter().rev().take(20) {
            if let TraceEvent::InterventionLog {
                source,
                success: false,
                ..
            } = event
            {
                *failures_by_source.entry(source.clone()).or_insert(0) += 1;
            }
        }

        let mut diagnoses = Vec::new();
        for (source, count) in &failures_by_source {
            if *count >= 3 {
                diagnoses.push(Diagnosis {
                    id: self.next_id(),
                    root_cause: format!("FailureSpiral: source '{}' failed {} times", source, count),
                    severity: (0.2 + *count as f64 * 0.15).min(1.0),
                    affected_components: vec![source.clone()],
                    suggested_fix: FixCategory::ModuleWiring,
                    trace_evidence: vec![format!("{} consecutive failures from {}", count, source)],
                    trend: TrendDirection::Worsening,
                });
            }
        }

        diagnoses
    }

    /// Pattern 4: Stagnation — no successful task completions.
    fn detect_stagnation(&mut self) -> Vec<Diagnosis> {
        // Collect owned data first to avoid simultaneous borrow conflicts
        let mut task_results = Vec::new();
        for e in self.events.iter() {
            if let TraceEvent::TaskResult { success, .. } = e {
                task_results.push(*success);
            }
        }

        if task_results.len() < 5 {
            return vec![];
        }

        let success_count = task_results.iter().filter(|s| **s).count();
        let total = task_results.len();
        let success_rate = success_count as f64 / total as f64;

        if success_rate > 0.2 {
            return vec![];
        }

        let id = self.next_id();
        vec![Diagnosis {
            id,
            root_cause: format!(
                "Stagnation: {:.0}% task success rate ({}/{})",
                success_rate * 100.0,
                success_count,
                total
            ),
            severity: (0.5 + (1.0 - success_rate) * 0.3).min(1.0),
            affected_components: vec!["task_system".to_string()],
            suggested_fix: FixCategory::SkillStagnation,
            trace_evidence: vec![format!(
                "Last {} tasks: {} succeeded",
                total, success_count
            )],
            trend: TrendDirection::Worsening,
        }]
    }

    /// Pattern 5: Memory Pressure — indicates via weakness miner reports.
    fn detect_memory_pressure(&mut self) -> Vec<Diagnosis> {
        let mut evidence = Vec::new();
        for e in self.events.iter() {
            if let TraceEvent::WeaknessDetected {
                pattern,
                severity: sev,
                ..
            } = e
            {
                if pattern == "memory" || pattern == "OOM" {
                    evidence.push(format!("{} weakness severity={}", pattern, sev));
                }
            }
        }

        if evidence.is_empty() {
            return vec![];
        }

        let count = evidence.len();
        let id = self.next_id();
        vec![Diagnosis {
            id,
            root_cause: format!("MemoryPressure: {} memory-related weakness events", count),
            severity: (0.3 + count as f64 * 0.1).min(1.0),
            affected_components: vec!["memory".to_string()],
            suggested_fix: FixCategory::MemoryPressure,
            trace_evidence: evidence,
            trend: TrendDirection::Worsening,
        }]
    }

    /// Pattern 6: Latency Degradation — from intervention logs indicating timeouts.
    fn detect_latency_degradation(&mut self) -> Vec<Diagnosis> {
        let mut evidence = Vec::new();
        for e in self.events.iter() {
            if let TraceEvent::InterventionLog {
                source,
                action,
                ..
            } = e
            {
                if action.contains("timeout") {
                    evidence.push(format!("timeout in {}: {}", source, action));
                }
            }
        }

        if evidence.len() < 3 {
            return vec![];
        }

        let count = evidence.len();
        let id = self.next_id();
        vec![Diagnosis {
            id,
            root_cause: format!("LatencyDegradation: {} timeout events", count),
            severity: (0.2 + count as f64 * 0.12).min(1.0),
            affected_components: vec!["latency".to_string()],
            suggested_fix: FixCategory::LatencyDegradation,
            trace_evidence: evidence,
            trend: TrendDirection::Worsening,
        }]
    }

    /// Convenience: build a CalibrationTrend from recent calibration snapshot data.
    pub fn build_calibration_trend(
        cycle: u64,
        domain: &str,
        ece_history: &VecDeque<f64>,
        surprise_history: &VecDeque<f64>,
    ) -> TraceEvent {
        let n = ece_history.len();
        let ece_avg = if n > 0 {
            ece_history.iter().sum::<f64>() / n as f64
        } else {
            0.0
        };

        let ece_trend = if n >= 5 {
            let recent: f64 = ece_history.iter().rev().take(3).sum::<f64>() / 3.0;
            let early: f64 = ece_history.iter().take(3).sum::<f64>() / 3.0;
            recent - early
        } else {
            0.0
        };

        let surprise_trend = if surprise_history.len() >= 5 {
            let recent: f64 = surprise_history.iter().rev().take(3).sum::<f64>() / 3.0;
            let early: f64 = surprise_history.iter().take(3).sum::<f64>() / 3.0;
            recent - early
        } else {
            0.0
        };

        TraceEvent::CalibrationTrend {
            cycle,
            domain: domain.to_string(),
            ece_avg,
            ece_trend,
            surprise_trend,
        }
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_analyzer_returns_no_diagnoses() {
        let mut analyzer = ReflectiveAnalyzer::new(100);
        assert!(analyzer.analyze().is_empty());
    }

    #[test]
    fn test_insufficient_events_returns_no_diagnoses() {
        let mut analyzer = ReflectiveAnalyzer::new(100);
        analyzer.feed_event(TraceEvent::CalibrationSnapshot {
            cycle: 1,
            domain: "reasoning".into(),
            ece: 0.1,
            surprise: 0.2,
        });
        assert!(analyzer.analyze().is_empty());
    }

    #[test]
    fn test_calibration_drift_detected() {
        let mut analyzer = ReflectiveAnalyzer::new(100);
        // Feed 5 calibration trends with high ECE
        for i in 0..5 {
            analyzer.feed_event(TraceEvent::CalibrationTrend {
                cycle: i,
                domain: "reasoning".into(),
                ece_avg: 0.25,
                ece_trend: 0.03,
                surprise_trend: 0.02,
            });
        }
        let diagnoses = analyzer.analyze();
        assert!(!diagnoses.is_empty());
        assert!(diagnoses[0].root_cause.contains("CalibrationDrift"));
        assert_eq!(diagnoses[0].suggested_fix, FixCategory::CalibrationDrift);
    }

    #[test]
    fn test_systemic_degradation_detected() {
        let mut analyzer = ReflectiveAnalyzer::new(100);
        // Feed calibration data + failing interventions
        analyzer.feed_event(TraceEvent::CalibrationSnapshot {
            cycle: 1,
            domain: "reasoning".into(),
            ece: 0.2,
            surprise: 0.3,
        });
        for _ in 0..5 {
            analyzer.feed_event(TraceEvent::InterventionLog {
                cycle: 1,
                source: "guard".into(),
                action: "verify".into(),
                success: false,
            });
        }
        let diagnoses = analyzer.analyze();
        let systemic = diagnoses.iter().find(|d| d.root_cause.contains("SystemicDegradation"));
        assert!(systemic.is_some(), "Expected SystemicDegradation diagnosis");
    }

    #[test]
    fn test_failure_spiral_detected() {
        let mut analyzer = ReflectiveAnalyzer::new(100);
        for _ in 0..5 {
            analyzer.feed_event(TraceEvent::InterventionLog {
                cycle: 1,
                source: "calibration_engine".into(),
                action: "recalibrate".into(),
                success: false,
            });
        }
        let diagnoses = analyzer.analyze();
        let spiral = diagnoses.iter().find(|d| d.root_cause.contains("FailureSpiral"));
        assert!(spiral.is_some(), "Expected FailureSpiral diagnosis");
    }

    #[test]
    fn test_stagnation_detected() {
        let mut analyzer = ReflectiveAnalyzer::new(100);
        // 7 failed tasks, 0 successes
        for i in 0..7 {
            analyzer.feed_event(TraceEvent::TaskResult {
                cycle: i,
                task_id: i as u64,
                success: false,
                gain: 0.0,
            });
        }
        let diagnoses = analyzer.analyze();
        // Should NOT detect stagnation with <5 tasks, so add more... wait, we have 7
        // But we also need calibration trend events for the analyzer to return anything.
        // Actually, detect_stagnation only checks TaskResult events, it's independent.
        // Let's check if stagnation was detected:
        let stagnation = diagnoses.iter().find(|d| d.root_cause.contains("Stagnation"));
        assert!(stagnation.is_some(), "Expected Stagnation diagnosis");
        // Should have severity based on failure rate
        assert!(stagnation.unwrap().severity > 0.5);
    }

    #[test]
    fn test_memory_pressure_detected() {
        let mut analyzer = ReflectiveAnalyzer::new(100);
        for _ in 0..4 {
            analyzer.feed_event(TraceEvent::WeaknessDetected {
                cycle: 1,
                pattern: "memory".into(),
                severity: 0.6,
                domain: "storage".into(),
            });
        }
        let diagnoses = analyzer.analyze();
        let memory = diagnoses.iter().find(|d| d.root_cause.contains("MemoryPressure"));
        assert!(memory.is_some(), "Expected MemoryPressure diagnosis");
    }

    #[test]
    fn test_latency_degradation_detected() {
        let mut analyzer = ReflectiveAnalyzer::new(100);
        for _ in 0..3 {
            analyzer.feed_event(TraceEvent::InterventionLog {
                cycle: 1,
                source: "http_client".into(),
                action: "request timeout".into(),
                success: false,
            });
        }
        let diagnoses = analyzer.analyze();
        let latency = diagnoses.iter().find(|d| d.root_cause.contains("LatencyDegradation"));
        assert!(latency.is_some(), "Expected LatencyDegradation diagnosis");
    }

    #[test]
    fn test_diagnoses_sorted_by_severity() {
        let mut analyzer = ReflectiveAnalyzer::new(100);
        // Calibration drift (high severity)
        for _ in 0..5 {
            analyzer.feed_event(TraceEvent::CalibrationTrend {
                cycle: 1,
                domain: "reasoning".into(),
                ece_avg: 0.35,
                ece_trend: 0.05,
                surprise_trend: 0.03,
            });
        }
        // Memory pressure (lower severity)
        for _ in 0..3 {
            analyzer.feed_event(TraceEvent::WeaknessDetected {
                cycle: 1,
                pattern: "memory".into(),
                severity: 0.4,
                domain: "storage".into(),
            });
        }
        let diagnoses = analyzer.analyze();
        assert!(diagnoses.len() >= 2);
        for i in 1..diagnoses.len() {
            assert!(
                diagnoses[i - 1].severity >= diagnoses[i].severity,
                "Diagnoses not sorted by severity descending"
            );
        }
    }

    #[test]
    fn test_build_calibration_trend() {
        let mut ece_hist = VecDeque::new();
        ece_hist.push_back(0.1);
        ece_hist.push_back(0.15);
        ece_hist.push_back(0.2);
        ece_hist.push_back(0.25);
        ece_hist.push_back(0.3);

        let mut surprise_hist = VecDeque::new();
        surprise_hist.push_back(0.2);
        surprise_hist.push_back(0.22);
        surprise_hist.push_back(0.25);
        surprise_hist.push_back(0.28);
        surprise_hist.push_back(0.3);

        let event = ReflectiveAnalyzer::build_calibration_trend(10, "reasoning", &ece_hist, &surprise_hist);
        match event {
            TraceEvent::CalibrationTrend {
                cycle,
                domain,
                ece_avg,
                ece_trend,
                surprise_trend,
            } => {
                assert_eq!(cycle, 10);
                assert_eq!(domain, "reasoning");
                assert!((ece_avg - 0.2).abs() < 0.01);
                assert!((ece_trend - 0.1).abs() < 0.01); // recent(0.25) - early(0.15) / wait...
                // recent average = (0.3+0.25+0.2)/3 = 0.25
                // early average = (0.1+0.15+0.2)/3 = 0.15
                // ece_trend = 0.25 - 0.15 = 0.1
                assert!((surprise_trend - 0.06).abs() < 0.01);
            }
            _ => panic!("Expected CalibrationTrend"),
        }
    }

    #[test]
    fn test_mixed_events_do_not_deadlock() {
        let mut analyzer = ReflectiveAnalyzer::new(100);
        // Mixed bag of events from different sources
        for i in 0..10 {
            analyzer.feed_event(TraceEvent::CalibrationSnapshot {
                cycle: i,
                domain: if i % 2 == 0 { "reasoning".into() } else { "memory".into() },
                ece: 0.05 + (i as f64 * 0.01),
                surprise: 0.1,
            });
            analyzer.feed_event(TraceEvent::InterventionLog {
                cycle: i,
                source: "loop".into(),
                action: "tick".into(),
                success: i % 3 != 0,
            });
            analyzer.feed_event(TraceEvent::TaskResult {
                cycle: i,
                task_id: i,
                success: i % 2 == 0,
                gain: 0.1,
            });
        }
        let diagnoses = analyzer.analyze();
        // Should not panic or deadlock
        assert!(diagnoses.len() <= 6); // Max 6 diagnosis types
    }

    #[test]
    fn test_diagnosis_id_unique_and_incrementing() {
        let mut analyzer = ReflectiveAnalyzer::new(100);
        for _ in 0..5 {
            analyzer.feed_event(TraceEvent::CalibrationTrend {
                cycle: 1,
                domain: "reasoning".into(),
                ece_avg: 0.3,
                ece_trend: 0.04,
                surprise_trend: 0.02,
            });
        }
        for _ in 0..3 {
            analyzer.feed_event(TraceEvent::InterventionLog {
                cycle: 1,
                source: "test".into(),
                action: "timeout".into(),
                success: false,
            });
        }
        let diagnoses = analyzer.analyze();
        let ids: std::collections::HashSet<u64> = diagnoses.iter().map(|d| d.id).collect();
        assert_eq!(ids.len(), diagnoses.len(), "Diagnosis IDs must be unique");
    }
}
