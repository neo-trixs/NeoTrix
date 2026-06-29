use super::intelligence_probe::{IntelligenceProbe, ProbeBox, ProbeFinding, ProbeResult};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct InvestigationPlan {
    pub goal: String,
    pub targets: Vec<String>,
    pub probes: Vec<String>,
    pub parallel_groups: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct InvestigationReport {
    pub goal: String,
    pub completed_at: u64,
    pub total_probes: usize,
    pub successful_probes: usize,
    pub total_findings: usize,
    pub critical_findings: usize,
    pub findings: Vec<ProbeFinding>,
    pub probe_results: Vec<ProbeResult>,
    pub duration_ms: u64,
}

pub struct IntelligenceOrchestrator {
    probes: HashMap<String, ProbeBox>,
}

impl IntelligenceOrchestrator {
    pub fn new() -> Self {
        Self {
            probes: HashMap::new(),
        }
    }

    pub fn register_probe(&mut self, probe: ProbeBox) {
        self.probes.insert(probe.name().to_string(), probe);
    }

    pub fn has_probe(&self, name: &str) -> bool {
        self.probes.contains_key(name)
    }

    pub fn list_probes(&self) -> Vec<&str> {
        self.probes.keys().map(|s| s.as_str()).collect()
    }

    pub fn probe_count(&self) -> usize {
        self.probes.len()
    }

    pub fn plan_investigation(&self, goal: &str, targets: &[String]) -> InvestigationPlan {
        let mut plan = InvestigationPlan {
            goal: goal.to_string(),
            targets: targets.to_vec(),
            probes: Vec::new(),
            parallel_groups: Vec::new(),
        };

        for (name, _probe) in &self.probes {
            plan.probes.push(name.clone());
        }

        let mut remaining: Vec<String> = plan.probes.clone();
        while !remaining.is_empty() {
            let group_size = remaining.len().min(3);
            let group: Vec<String> = remaining.drain(..group_size).collect();
            plan.parallel_groups.push(group);
        }

        plan
    }

    pub fn execute_plan(&self, plan: &InvestigationPlan, timeout_secs: u64) -> InvestigationReport {
        let start = Instant::now();
        let mut all_results: Vec<ProbeResult> = Vec::new();
        let mut all_findings: Vec<ProbeFinding> = Vec::new();

        for group in &plan.parallel_groups {
            let mut group_results: Vec<ProbeResult> = Vec::new();
            for probe_name in group {
                if let Some(probe) = self.probes.get(probe_name) {
                    for target in &plan.targets {
                        let result = probe.probe(target, timeout_secs);
                        group_results.push(result);
                    }
                }
            }
            for result in &group_results {
                all_findings.extend(result.findings.clone());
            }
            all_results.extend(group_results);
        }

        let total = all_results.len();
        let successful = all_results.iter().filter(|r| r.success).count();
        let critical = all_findings
            .iter()
            .filter(|f| {
                matches!(
                    f.severity,
                    super::intelligence_probe::ProbeSeverity::Critical
                )
            })
            .count();

        InvestigationReport {
            goal: plan.goal.clone(),
            completed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            total_probes: total,
            successful_probes: successful,
            total_findings: all_findings.len(),
            critical_findings: critical,
            findings: all_findings,
            probe_results: all_results,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    pub fn investigate(&self, goal: &str, target: &str, timeout_secs: u64) -> InvestigationReport {
        let plan = self.plan_investigation(goal, &[target.to_string()]);
        self.execute_plan(&plan, timeout_secs)
    }

    pub fn run_probe(&self, probe_name: &str, target: &str, timeout: u64) -> Option<ProbeResult> {
        self.probes
            .get(probe_name)
            .map(|probe| probe.probe(target, timeout))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_knowledge::osint::intelligence_probe::{
        IntelligenceProbe, ProbeFinding, ProbeResult,
    };

    struct MockProbe {
        name: String,
    }

    impl IntelligenceProbe for MockProbe {
        fn name(&self) -> &str {
            &self.name
        }
        fn description(&self) -> &str {
            "mock probe for testing"
        }
        fn probe(&self, target: &str, _timeout: u64) -> ProbeResult {
            ProbeResult::new(&self.name, target)
                .with_finding(ProbeFinding::new("mock_key", "mock_value", &self.name))
                .mark_success()
        }
    }

    #[test]
    fn test_register_and_list_probes() {
        let mut orchestrator = IntelligenceOrchestrator::new();
        orchestrator.register_probe(Box::new(MockProbe {
            name: "test_probe".into(),
        }));
        assert!(orchestrator.has_probe("test_probe"));
        assert_eq!(orchestrator.probe_count(), 1);
        assert_eq!(orchestrator.list_probes(), vec!["test_probe"]);
    }

    #[test]
    fn test_investigate_with_no_probes() {
        let orchestrator = IntelligenceOrchestrator::new();
        let report = orchestrator.investigate("test goal", "example.com", 30);
        assert_eq!(report.total_probes, 0);
        assert_eq!(report.total_findings, 0);
    }

    #[test]
    fn test_plan_investigation_creates_groups() {
        let mut orchestrator = IntelligenceOrchestrator::new();
        for i in 0..5 {
            orchestrator.register_probe(Box::new(MockProbe {
                name: format!("probe_{}", i),
            }));
        }
        let plan = orchestrator.plan_investigation("test", &["target".into()]);
        assert_eq!(plan.probes.len(), 5);
        assert!(!plan.parallel_groups.is_empty());
    }

    #[test]
    fn test_run_single_probe() {
        let mut orchestrator = IntelligenceOrchestrator::new();
        orchestrator.register_probe(Box::new(MockProbe {
            name: "mock".into(),
        }));
        let result = orchestrator.run_probe("mock", "test_target", 10);
        assert!(result.is_some());
        assert!(result.unwrap().success);
    }

    #[test]
    fn test_run_unknown_probe() {
        let orchestrator = IntelligenceOrchestrator::new();
        let result = orchestrator.run_probe("nonexistent", "target", 10);
        assert!(result.is_none());
    }

    #[test]
    fn test_execute_plan_with_multiple_targets() {
        let mut orchestrator = IntelligenceOrchestrator::new();
        orchestrator.register_probe(Box::new(MockProbe {
            name: "multi_probe".into(),
        }));
        let plan = orchestrator
            .plan_investigation("multi-target test", &["target_a".into(), "target_b".into()]);
        let report = orchestrator.execute_plan(&plan, 30);
        assert_eq!(report.total_probes, 2, "should run probe on each target");
    }
}
