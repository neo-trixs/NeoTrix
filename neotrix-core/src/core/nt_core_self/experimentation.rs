use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExperimentPhase {
    Observe,
    Analyze,
    Hypothesize,
    Intervene,
    Verify,
    Complete,
}

impl ExperimentPhase {
    pub fn next(&self) -> ExperimentPhase {
        match self {
            ExperimentPhase::Observe => ExperimentPhase::Analyze,
            ExperimentPhase::Analyze => ExperimentPhase::Hypothesize,
            ExperimentPhase::Hypothesize => ExperimentPhase::Intervene,
            ExperimentPhase::Intervene => ExperimentPhase::Verify,
            ExperimentPhase::Verify => ExperimentPhase::Complete,
            ExperimentPhase::Complete => ExperimentPhase::Complete,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ExperimentPhase::Observe => "Observe",
            ExperimentPhase::Analyze => "Analyze",
            ExperimentPhase::Hypothesize => "Hypothesize",
            ExperimentPhase::Intervene => "Intervene",
            ExperimentPhase::Verify => "Verify",
            ExperimentPhase::Complete => "Complete",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Hypothesis {
    pub id: u64,
    pub description: String,
    pub confidence: f64,
    pub predicted_outcome: String,
    pub observable_to_check: String,
    pub status: &'static str,
}

impl Hypothesis {
    fn new(id: u64, a: &str, b: &str, conf: f64) -> Self {
        Hypothesis {
            id,
            description: format!("{} correlates with {}", a, b),
            confidence: conf,
            predicted_outcome: format!("increasing {} will increase {}", a, b),
            observable_to_check: b.to_string(),
            status: "unverified",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Intervention {
    pub target_variable: String,
    pub old_value: f64,
    pub new_value: f64,
    pub rationale: String,
}

impl Intervention {
    pub fn description(&self) -> String {
        format!(
            "Intervention on {}: {:.4} -> {:.4} because {}",
            self.target_variable, self.old_value, self.new_value, self.rationale
        )
    }
}

#[derive(Debug, Clone)]
pub struct Experiment {
    pub id: u64,
    pub name: String,
    pub phase: ExperimentPhase,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub hypotheses: Vec<Hypothesis>,
    pub interventions: Vec<Intervention>,
    pub baseline_metrics: HashMap<String, f64>,
    pub post_intervention_metrics: HashMap<String, f64>,
    pub result: Option<String>,
    pub conclusion: Option<String>,
}

pub struct ExperimentDesigner {
    pub experiments: Vec<Experiment>,
    pub max_hypotheses_per_loop: usize,
    counter: u64,
}

impl ExperimentDesigner {
    pub fn new() -> Self {
        ExperimentDesigner {
            experiments: Vec::new(),
            max_hypotheses_per_loop: 3,
            counter: 0,
        }
    }

    pub fn design_experiment(&mut self, name: &str, observations: &[(&str, f64)]) -> u64 {
        self.counter += 1;
        let id = self.counter;
        let mut baseline = HashMap::new();
        let mut hypotheses = Vec::new();
        let obs: Vec<(String, f64)> = observations
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect();
        for (k, v) in &obs {
            baseline.insert(k.clone(), *v);
        }
        let mut hyp_count = 0;
        for i in 0..obs.len() {
            if hyp_count >= self.max_hypotheses_per_loop {
                break;
            }
            for j in (i + 1)..obs.len() {
                if hyp_count >= self.max_hypotheses_per_loop {
                    break;
                }
                let (ref an, av) = obs[i];
                let (ref bn, bv) = obs[j];
                let diff = (av - bv).abs();
                let conf = if diff < 0.1 {
                    0.85
                } else if diff < 0.5 {
                    0.65
                } else if diff > 2.0 {
                    0.40
                } else {
                    0.50
                };
                hypotheses.push(Hypothesis::new(
                    self.counter * 100 + hyp_count as u64,
                    &an,
                    &bn,
                    conf,
                ));
                hyp_count += 1;
            }
        }
        let experiment = Experiment {
            id,
            name: name.to_string(),
            phase: ExperimentPhase::Observe,
            start_time: 1,
            end_time: None,
            hypotheses,
            interventions: Vec::new(),
            baseline_metrics: baseline,
            post_intervention_metrics: HashMap::new(),
            result: None,
            conclusion: None,
        };
        self.experiments.push(experiment);
        id
    }

    pub fn propose_intervention(&self, experiment_id: u64) -> Option<Intervention> {
        let exp = self.experiments.iter().find(|e| e.id == experiment_id)?;
        if exp.baseline_metrics.is_empty() {
            return None;
        }
        let target = exp.baseline_metrics.keys().next()?;
        let old_val = *exp.baseline_metrics.get(target.as_str())?;
        let new_val = if old_val == 0.0 { 1.0 } else { old_val * 0.5 };
        Some(Intervention {
            target_variable: target.clone(),
            old_value: old_val,
            new_value: new_val,
            rationale: format!("testing effect of reducing {} by half", target),
        })
    }

    pub fn record_post_intervention(&mut self, experiment_id: u64, metrics: &[(&str, f64)]) {
        if let Some(exp) = self.experiments.iter_mut().find(|e| e.id == experiment_id) {
            for (k, v) in metrics {
                exp.post_intervention_metrics.insert(k.to_string(), *v);
            }
        }
    }

    pub fn verify_hypothesis(&mut self, experiment_id: u64, hypothesis_id: u64) -> bool {
        if let Some(exp) = self.experiments.iter_mut().find(|e| e.id == experiment_id) {
            if let Some(hyp) = exp.hypotheses.iter_mut().find(|h| h.id == hypothesis_id) {
                let target = &hyp.observable_to_check;
                let baseline = exp.baseline_metrics.get(target);
                let post = exp.post_intervention_metrics.get(target);
                if let (Some(&b), Some(&p)) = (baseline, post) {
                    let diff = (b - p).abs();
                    let predicted_diff = hyp.confidence;
                    if diff > predicted_diff {
                        hyp.status = "confirmed";
                        return true;
                    } else {
                        hyp.status = "refuted";
                        return false;
                    }
                }
            }
        }
        false
    }

    pub fn advance_phase(&mut self, experiment_id: u64) {
        if let Some(exp) = self.experiments.iter_mut().find(|e| e.id == experiment_id) {
            let next = exp.phase.next();
            if next == ExperimentPhase::Complete {
                exp.end_time = Some(2);
            }
            exp.phase = next;
        }
    }

    pub fn conclude_experiment(&mut self, experiment_id: u64, conclusion: &str) {
        if let Some(exp) = self.experiments.iter_mut().find(|e| e.id == experiment_id) {
            exp.conclusion = Some(conclusion.to_string());
            exp.phase = ExperimentPhase::Complete;
        }
    }

    pub fn active_experiments(&self) -> Vec<&Experiment> {
        self.experiments
            .iter()
            .filter(|e| e.phase != ExperimentPhase::Complete)
            .collect()
    }

    pub fn verified_hypotheses(&self) -> Vec<&Hypothesis> {
        self.experiments
            .iter()
            .flat_map(|e| e.hypotheses.iter())
            .filter(|h| h.status == "confirmed")
            .collect()
    }

    pub fn loop_iteration(&mut self, observations: &[(&str, f64)]) -> Option<u64> {
        let active = self.active_experiments();
        if active.is_empty() {
            let id = self.design_experiment("auto_experiment", observations);
            return Some(id);
        }
        let exp_id = active[0].id;
        match active[0].phase {
            ExperimentPhase::Observe | ExperimentPhase::Analyze | ExperimentPhase::Hypothesize => {
                self.advance_phase(exp_id);
                Some(exp_id)
            }
            ExperimentPhase::Intervene => {
                if let Some(intervention) = self.propose_intervention(exp_id) {
                    if let Some(exp) = self.experiments.iter_mut().find(|e| e.id == exp_id) {
                        exp.interventions.push(intervention);
                    }
                }
                self.advance_phase(exp_id);
                Some(exp_id)
            }
            ExperimentPhase::Verify => {
                let hyp_ids: Vec<u64> = {
                    let exp = self.experiments.iter().find(|e| e.id == exp_id)?;
                    exp.hypotheses.iter().map(|h| h.id).collect()
                };
                for hid in hyp_ids {
                    self.verify_hypothesis(exp_id, hid);
                }
                self.advance_phase(exp_id);
                Some(exp_id)
            }
            ExperimentPhase::Complete => None,
        }
    }
}

pub struct ExperimentationReport {
    pub total_experiments: usize,
    pub verified_count: usize,
    pub refuted_count: usize,
    pub active_phases: Vec<String>,
    pub recent_conclusions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_experiment() {
        let mut ed = ExperimentDesigner::new();
        let id = ed.design_experiment("grokking_test", &[("accuracy", 0.6), ("loss", 0.8)]);
        assert_eq!(id, 1);
        let exp = ed.experiments.iter().find(|e| e.id == id).unwrap();
        assert_eq!(exp.name, "grokking_test");
        assert_eq!(exp.phase, ExperimentPhase::Observe);
        assert_eq!(*exp.baseline_metrics.get("accuracy").unwrap(), 0.6);
        assert!(exp.hypotheses.len() <= 3);
    }

    #[test]
    fn test_propose_intervention() {
        let mut ed = ExperimentDesigner::new();
        let id = ed.design_experiment("test", &[("lr", 1.0), ("batch", 64.0)]);
        let intervention = ed.propose_intervention(id);
        assert!(intervention.is_some());
        let i = intervention.unwrap();
        assert_eq!(i.target_variable, "lr");
        assert_eq!(i.old_value, 1.0);
    }

    #[test]
    fn test_record_post_intervention() {
        let mut ed = ExperimentDesigner::new();
        let id = ed.design_experiment("test", &[("x", 1.0), ("y", 2.0)]);
        ed.record_post_intervention(id, &[("x", 0.5), ("y", 1.5)]);
        let exp = ed.experiments.iter().find(|e| e.id == id).unwrap();
        assert_eq!(*exp.post_intervention_metrics.get("x").unwrap(), 0.5);
        assert_eq!(*exp.post_intervention_metrics.get("y").unwrap(), 1.5);
    }

    #[test]
    fn test_verify_hypothesis_confirmed() {
        let mut ed = ExperimentDesigner::new();
        let id = ed.design_experiment("test", &[("a", 1.0), ("b", 1.0)]);
        ed.record_post_intervention(id, &[("a", 0.1), ("b", 0.1)]);
        let hyp_id = ed
            .experiments
            .iter()
            .find(|e| e.id == id)
            .unwrap()
            .hypotheses[0]
            .id;
        let result = ed.verify_hypothesis(id, hyp_id);
        assert!(result);
    }

    #[test]
    fn test_verify_hypothesis_refuted() {
        let mut ed = ExperimentDesigner::new();
        let id = ed.design_experiment("test", &[("a", 1.0), ("b", 100.0)]);
        ed.record_post_intervention(id, &[("a", 1.0), ("b", 100.0)]);
        let hyp_id = ed
            .experiments
            .iter()
            .find(|e| e.id == id)
            .unwrap()
            .hypotheses[0]
            .id;
        let result = ed.verify_hypothesis(id, hyp_id);
        assert!(!result);
    }

    #[test]
    fn test_advance_phase_all_six() {
        let mut ed = ExperimentDesigner::new();
        let id = ed.design_experiment("six_phases", &[("m", 1.0)]);
        let phases = [
            ExperimentPhase::Observe,
            ExperimentPhase::Analyze,
            ExperimentPhase::Hypothesize,
            ExperimentPhase::Intervene,
            ExperimentPhase::Verify,
            ExperimentPhase::Complete,
        ];
        for p in &phases {
            let exp = ed.experiments.iter().find(|e| e.id == id).unwrap();
            assert_eq!(exp.phase, *p);
            if *p != ExperimentPhase::Complete {
                ed.advance_phase(id);
            }
        }
    }

    #[test]
    fn test_loop_iteration_full_cycle() {
        let mut ed = ExperimentDesigner::new();
        let id = ed
            .loop_iteration(&[("accuracy", 0.5), ("speed", 0.7)])
            .unwrap();
        assert_eq!(id, 1);
        ed.loop_iteration(&[("accuracy", 0.5), ("speed", 0.7)]);
        ed.loop_iteration(&[("accuracy", 0.5), ("speed", 0.7)]);
        ed.loop_iteration(&[("accuracy", 0.5), ("speed", 0.7)]);
        ed.loop_iteration(&[("accuracy", 0.5), ("speed", 0.7)]);
        ed.loop_iteration(&[("accuracy", 0.5), ("speed", 0.7)]);
        let exp = ed.experiments.iter().find(|e| e.id == id).unwrap();
        assert_eq!(exp.phase, ExperimentPhase::Complete);
    }

    #[test]
    fn test_active_experiments_filtering() {
        let mut ed = ExperimentDesigner::new();
        let id1 = ed.design_experiment("active_exp", &[("x", 1.0)]);
        let id2 = ed.design_experiment("done_exp", &[("y", 2.0)]);
        ed.conclude_experiment(id2, "done");
        let active = ed.active_experiments();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, id1);
    }

    #[test]
    fn test_verified_hypotheses_accumulation() {
        let mut ed = ExperimentDesigner::new();
        let id = ed.design_experiment("verify_acc", &[("a", 1.0), ("b", 1.0)]);
        ed.record_post_intervention(id, &[("a", 0.1), ("b", 0.1)]);
        let hyp_id = ed
            .experiments
            .iter()
            .find(|e| e.id == id)
            .unwrap()
            .hypotheses[0]
            .id;
        ed.verify_hypothesis(id, hyp_id);
        let verified = ed.verified_hypotheses();
        assert_eq!(verified.len(), 1);
        assert_eq!(verified[0].status, "confirmed");
    }

    #[test]
    fn test_empty_state_edge() {
        let ed = ExperimentDesigner::new();
        assert!(ed.active_experiments().is_empty());
        assert!(ed.verified_hypotheses().is_empty());
        assert!(ed.propose_intervention(999).is_none());
    }

    #[test]
    fn test_conclude_experiment() {
        let mut ed = ExperimentDesigner::new();
        let id = ed.design_experiment("conclude", &[("p", 1.0)]);
        ed.conclude_experiment(id, "hypothesis confirmed");
        let exp = ed.experiments.iter().find(|e| e.id == id).unwrap();
        assert_eq!(exp.phase, ExperimentPhase::Complete);
        assert_eq!(exp.conclusion.as_deref(), Some("hypothesis confirmed"));
    }

    #[test]
    fn test_intervention_description() {
        let i = Intervention {
            target_variable: "lr".to_string(),
            old_value: 0.1,
            new_value: 0.01,
            rationale: "testing lower lr".to_string(),
        };
        let desc = i.description();
        assert!(desc.contains("lr"));
        assert!(desc.contains("0.1000"));
        assert!(desc.contains("0.0100"));
    }
}
