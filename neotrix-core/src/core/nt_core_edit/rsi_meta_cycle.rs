#![allow(dead_code)]

/// G219: RSI Meta-Cycle — Recursive Self-Improvement
///
/// The system observes its own performance, generates improvement proposals,
/// applies them, and measures the result. Each cycle closes the loop:
///   metric_gap → proposal → apply → measure → assess

#[derive(Debug, Clone, PartialEq)]
pub enum ImprovementType {
    Optimization,
    Refactoring,
    NewCapability,
    ArchitectureChange,
    MetaPrompt,
}

#[derive(Debug, Clone)]
pub struct ImprovementProposal {
    pub id: u64,
    pub description: String,
    pub imp_type: ImprovementType,
    pub target_module: String,
    pub expected_impact: f64,
    pub measured_impact: Option<f64>,
    pub applied: bool,
    pub success: Option<bool>,
    pub created_tick: u64,
    pub applied_tick: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct SelfImprovementMetric {
    pub metric_name: String,
    pub value: f64,
    pub target: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct RsiMetaCycle {
    pub proposals: Vec<ImprovementProposal>,
    pub metrics: Vec<SelfImprovementMetric>,
    pub cycle_count: u64,
    pub max_proposals: usize,
    pub success_rate: f64,
    next_id: u64,
}

impl RsiMetaCycle {
    pub fn new() -> Self {
        Self {
            proposals: Vec::new(),
            metrics: Vec::new(),
            cycle_count: 0,
            max_proposals: 1000,
            success_rate: 0.0,
            next_id: 1,
        }
    }

    pub fn propose_improvement(
        &mut self,
        description: &str,
        imp_type: ImprovementType,
        target: &str,
        expected_impact: f64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.proposals.push(ImprovementProposal {
            id,
            description: description.to_string(),
            imp_type,
            target_module: target.to_string(),
            expected_impact: expected_impact.clamp(0.0, 1.0),
            measured_impact: None,
            applied: false,
            success: None,
            created_tick: self.cycle_count,
            applied_tick: None,
        });
        if self.proposals.len() > self.max_proposals {
            self.proposals.remove(0);
        }
        id
    }

    pub fn apply_improvement(&mut self, id: u64, tick: u64) -> bool {
        if let Some(p) = self.proposals.iter_mut().find(|p| p.id == id) {
            if p.applied {
                return false;
            }
            p.applied = true;
            p.applied_tick = Some(tick);
            true
        } else {
            false
        }
    }

    pub fn measure_impact(&mut self, id: u64, measured: f64) -> bool {
        if let Some(p) = self.proposals.iter_mut().find(|p| p.id == id) {
            if p.measured_impact.is_some() {
                return false;
            }
            let clamped = measured.clamp(0.0, 1.0);
            p.measured_impact = Some(clamped);
            p.success = Some(clamped >= p.expected_impact * 0.8);
            self.update_success_rate();
            true
        } else {
            false
        }
    }

    pub fn record_metric(&mut self, name: &str, value: f64, target: f64) {
        self.metrics.push(SelfImprovementMetric {
            metric_name: name.to_string(),
            value,
            target,
            timestamp: self.cycle_count,
        });
        if self.metrics.len() > self.max_proposals {
            self.metrics.remove(0);
        }
    }

    /// Auto-generates proposals based on metric gaps (value < target * 0.8).
    /// Returns IDs of newly created proposals.
    pub fn run_cycle(&mut self, tick: u64) -> Vec<u64> {
        self.cycle_count = tick;
        let mut new_ids = Vec::new();

        let gaps: Vec<(String, f64)> = self
            .metrics
            .iter()
            .filter(|m| m.value < m.target * 0.8)
            .map(|m| {
                let gap = m.target - m.value;
                let impact = (gap / m.target.max(1e-9)).clamp(0.0, 1.0);
                (m.metric_name.clone(), impact)
            })
            .collect();

        for (name, impact) in gaps {
            let id = self.propose_improvement(
                &format!("Improve '{}': gap={:.3}", name, impact),
                ImprovementType::Optimization,
                &name,
                impact,
            );
            new_ids.push(id);
        }

        new_ids
    }

    /// Generates a self-assessment report string.
    pub fn self_assessment(&self) -> String {
        let total = self.proposals.len();
        let applied = self.proposals.iter().filter(|p| p.applied).count();
        let successes = self
            .proposals
            .iter()
            .filter(|p| p.success == Some(true))
            .count();
        let failures = self
            .proposals
            .iter()
            .filter(|p| p.success == Some(false))
            .count();
        let pending_measure = self
            .proposals
            .iter()
            .filter(|p| p.applied && p.measured_impact.is_none())
            .count();

        let avg_expected: f64 = if total > 0 {
            self.proposals
                .iter()
                .map(|p| p.expected_impact)
                .sum::<f64>()
                / total as f64
        } else {
            0.0
        };

        let avg_measured: f64 = {
            let measured: Vec<_> = self
                .proposals
                .iter()
                .filter_map(|p| p.measured_impact)
                .collect();
            if measured.is_empty() {
                0.0
            } else {
                measured.iter().sum::<f64>() / measured.len() as f64
            }
        };

        format!(
            "RSI Meta-Cycle Assessment\n\
             ─────────────────────────────\n\
             Cycle:                  {}\n\
             Proposals:              {} total, {} applied, {} pending measure\n\
             Successes:              {} / {} failures\n\
             Success rate:           {:.1}%\n\
             Avg expected impact:    {:.3}\n\
             Avg measured impact:    {:.3}\n\
             Metrics tracked:        {}\n",
            self.cycle_count,
            total,
            applied,
            pending_measure,
            successes,
            failures,
            self.success_rate * 100.0,
            avg_expected,
            avg_measured,
            self.metrics.len(),
        )
    }

    /// Recent success rate over the last `window` proposals that have been measured.
    pub fn success_trend(&self, window: usize) -> f64 {
        let measured: Vec<_> = self
            .proposals
            .iter()
            .filter(|p| p.success.is_some())
            .collect();
        let len = measured.len();
        if len == 0 {
            return 0.0;
        }
        let start = if len > window { len - window } else { 0 };
        let recent: Vec<_> = measured[start..].iter().collect();
        let successes = recent.iter().filter(|p| p.success == Some(true)).count();
        successes as f64 / recent.len() as f64
    }

    /// Proposals applied per cycle (measured over total cycle count).
    pub fn improvement_velocity(&self) -> f64 {
        let applied = self.proposals.iter().filter(|p| p.applied).count() as f64;
        let cycles = self.cycle_count.max(1) as f64;
        applied / cycles
    }

    pub fn reset_cycle(&mut self) {
        self.proposals.clear();
        self.metrics.clear();
        self.cycle_count = 0;
        self.success_rate = 0.0;
        self.next_id = 1;
    }

    fn update_success_rate(&mut self) {
        let measured: Vec<_> = self
            .proposals
            .iter()
            .filter(|p| p.success.is_some())
            .collect();
        let len = measured.len();
        if len == 0 {
            self.success_rate = 0.0;
        } else {
            let successes = measured.iter().filter(|p| p.success == Some(true)).count();
            self.success_rate = successes as f64 / len as f64;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cycle() {
        let cycle = RsiMetaCycle::new();
        assert_eq!(cycle.cycle_count, 0);
        assert!(cycle.proposals.is_empty());
        assert!(cycle.metrics.is_empty());
        assert!((cycle.success_rate - 0.0).abs() < 1e-9);
        assert_eq!(cycle.next_id, 1);
    }

    #[test]
    fn test_propose_improvement() {
        let mut cycle = RsiMetaCycle::new();
        let id = cycle.propose_improvement(
            "Optimize query planner",
            ImprovementType::Optimization,
            "nt_core_knowledge",
            0.75,
        );
        assert_eq!(id, 1);
        assert_eq!(cycle.proposals.len(), 1);
        assert_eq!(cycle.proposals[0].description, "Optimize query planner");
        assert_eq!(cycle.proposals[0].imp_type, ImprovementType::Optimization);
        assert!((cycle.proposals[0].expected_impact - 0.75).abs() < 1e-9);

        let id2 = cycle.propose_improvement(
            "Refactor consensus layer",
            ImprovementType::Refactoring,
            "nt_core_agent",
            0.5,
        );
        assert_eq!(id2, 2);
        assert_eq!(cycle.proposals.len(), 2);
    }

    #[test]
    fn test_apply_and_measure() {
        let mut cycle = RsiMetaCycle::new();
        let id = cycle.propose_improvement(
            "Add sparse VSA support",
            ImprovementType::NewCapability,
            "nt_core_hcube",
            0.85,
        );

        assert!(cycle.apply_improvement(id, 42));
        assert!(cycle.proposals[0].applied);
        assert_eq!(cycle.proposals[0].applied_tick, Some(42));

        // double-apply should fail
        assert!(!cycle.apply_improvement(id, 43));

        assert!(cycle.measure_impact(id, 0.9));
        assert!((cycle.proposals[0].measured_impact.unwrap() - 0.9).abs() < 1e-9);
        assert_eq!(cycle.proposals[0].success, Some(true));

        // measure if already measured should fail
        assert!(!cycle.measure_impact(id, 0.95));
    }

    #[test]
    fn test_record_metric() {
        let mut cycle = RsiMetaCycle::new();
        cycle.record_metric("accuracy", 0.65, 0.90);
        cycle.record_metric("latency_ms", 120.0, 50.0);
        assert_eq!(cycle.metrics.len(), 2);
        assert!((cycle.metrics[0].value - 0.65).abs() < 1e-9);
        assert!((cycle.metrics[1].target - 50.0).abs() < 1e-9);
    }

    #[test]
    fn test_run_cycle_generates_proposals() {
        let mut cycle = RsiMetaCycle::new();
        cycle.record_metric("accuracy", 0.65, 0.90);
        cycle.record_metric("latency_ms", 120.0, 50.0);
        cycle.record_metric("throughput", 800.0, 1000.0);

        let new_ids = cycle.run_cycle(1);
        assert_eq!(new_ids.len(), 1); // only accuracy: 0.65 < 0.72
        assert_eq!(cycle.cycle_count, 1);
    }

    #[test]
    fn test_self_assessment() {
        let mut cycle = RsiMetaCycle::new();
        let report_empty = cycle.self_assessment();
        assert!(report_empty.contains("0 total"));

        let id = cycle.propose_improvement("Fix perf", ImprovementType::Optimization, "core", 0.8);
        cycle.apply_improvement(id, 1);
        cycle.measure_impact(id, 0.7);

        let report = cycle.self_assessment();
        assert!(report.contains("1 total"));
        assert!(report.contains("1 applied"));
        assert!(report.contains("1 successes"));
        assert!(report.contains("0 failures"));
    }

    #[test]
    fn test_success_trend() {
        let mut cycle = RsiMetaCycle::new();
        // no measured proposals → 0.0
        assert!((cycle.success_trend(5) - 0.0).abs() < 1e-9);

        for i in 0..10 {
            let id = cycle.propose_improvement(
                &format!("proposal {}", i),
                ImprovementType::Optimization,
                "core",
                0.8,
            );
            cycle.apply_improvement(id, i);
            cycle.measure_impact(id, if i < 7 { 0.9 } else { 0.1 });
        }

        // Window of 5: proposals 5-9, successes: 5,6 are true, 7,8,9 false → 2/5
        assert!((cycle.success_trend(5) - 0.4).abs() < 1e-9);

        // Window of 10: 7/10
        assert!((cycle.success_trend(10) - 0.7).abs() < 1e-9);
    }

    #[test]
    fn test_improvement_velocity() {
        let mut cycle = RsiMetaCycle::new();
        // Run cycle to set cycle_count
        cycle.record_metric("accuracy", 0.5, 1.0);
        cycle.run_cycle(10);
        assert!((cycle.improvement_velocity() - 0.1).abs() < 1e-9); // 1 proposal / 10 cycles

        let id = cycle.propose_improvement("test", ImprovementType::Refactoring, "core", 0.5);
        cycle.apply_improvement(id, 10);
        let id2 = cycle.propose_improvement("test2", ImprovementType::Optimization, "core", 0.5);
        cycle.apply_improvement(id2, 10);

        assert!((cycle.improvement_velocity() - 0.3).abs() < 1e-9); // 3 proposals / 10 cycles
    }
}
