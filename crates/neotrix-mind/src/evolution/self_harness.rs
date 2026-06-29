use std::collections::VecDeque;

/// 3-order self-model hierarchy (Stack Theory, 2025)
#[derive(Debug, Clone, PartialEq)]
pub enum SelfModelOrder {
    /// First-order: beliefs about the external world
    World,
    /// Second-order: beliefs about oneself (capabilities, limits)
    Self_,
    /// Third-order: beliefs about one's own cognitive processes
    Meta,
}

#[derive(Debug, Clone)]
pub struct SelfModelSnapshot {
    pub order: SelfModelOrder,
    pub content: String,
    pub confidence: f64,
    pub coherence: f64,
    pub timestamp: usize,
}

#[derive(Debug, Clone)]
pub struct WeaknessReport {
    pub weakness: String,
    pub confidence: f64,
    pub affected_module: String,
    pub trace_ref: String,
}

#[derive(Debug, Clone)]
pub struct EvolutionProposal {
    pub proposal: String,
    pub weakness_ref: usize,
    pub estimated_gain: f64,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub proposal_ref: usize,
    pub passed: bool,
    pub score_before: f64,
    pub score_after: f64,
    pub artifacts: Vec<String>,
    pub narrative: String,
}

pub struct SelfHarnessLoop {
    weakness_archive: VecDeque<WeaknessReport>,
    #[allow(dead_code)]
    proposals: Vec<EvolutionProposal>,
    validations: Vec<ValidationResult>,
    max_history: usize,
    iteration: usize,
    /// 3-order self-model hierarchy (Stack Theory)
    self_models: Vec<SelfModelSnapshot>,
}

impl SelfHarnessLoop {
    pub fn new() -> Self {
        Self {
            weakness_archive: VecDeque::with_capacity(128),
            proposals: Vec::with_capacity(64),
            validations: Vec::with_capacity(64),
            max_history: 128,
            iteration: 0,
            self_models: Vec::with_capacity(32),
        }
    }

    pub fn mine_weaknesses(&mut self, traces: Vec<String>) -> Vec<WeaknessReport> {
        let mut reports = Vec::new();
        for trace in traces {
            let report = self.analyze_trace(&trace);
            reports.push(report);
        }
        for r in &reports {
            if self.weakness_archive.len() >= self.max_history {
                self.weakness_archive.pop_front();
            }
            self.weakness_archive.push_back(r.clone());
        }
        self.iteration += 1;
        reports
    }

    pub fn generate_proposals(&self, weaknesses: &[WeaknessReport]) -> Vec<EvolutionProposal> {
        weaknesses
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let gain = w.confidence * (1.0 / (self.iteration as f64 + 1.0).sqrt());
                EvolutionProposal {
                    proposal: format!("Fix: {} in {}", w.weakness, w.affected_module),
                    weakness_ref: i,
                    estimated_gain: gain.clamp(0.0, 1.0),
                }
            })
            .collect()
    }

    pub fn record_validation(&mut self, result: ValidationResult) {
        self.validations.push(result);
    }

    pub fn success_rate(&self) -> f64 {
        if self.validations.is_empty() {
            return 0.0;
        }
        let passed = self.validations.iter().filter(|v| v.passed).count();
        passed as f64 / self.validations.len() as f64
    }

    pub fn average_improvement(&self) -> f64 {
        if self.validations.is_empty() {
            return 0.0;
        }
        let total: f64 = self
            .validations
            .iter()
            .map(|v| v.score_after - v.score_before)
            .sum();
        total / self.validations.len() as f64
    }

    pub fn top_weaknesses(&self, n: usize) -> Vec<&WeaknessReport> {
        let mut sorted: Vec<_> = self.weakness_archive.iter().collect();
        sorted.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.into_iter().take(n).collect()
    }

    pub fn closed_loop(&mut self, traces: Vec<String>) -> Vec<ValidationResult> {
        let weaknesses = self.mine_weaknesses(traces);
        let proposals = self.generate_proposals(&weaknesses);
        let mut results = Vec::new();
        for proposal in proposals {
            let score_before = 0.5;
            let simulated_after = (score_before + proposal.estimated_gain * 0.3).min(1.0);
            let passed = simulated_after > score_before * 1.02;
            let result = ValidationResult {
                proposal_ref: proposal.weakness_ref,
                passed,
                score_before,
                score_after: simulated_after,
                artifacts: vec![proposal.proposal.clone()],
                narrative: format!(
                    "Proposal {}: {:.2} -> {:.2}",
                    proposal.weakness_ref, score_before, simulated_after
                ),
            };
            self.record_validation(result.clone());
            results.push(result);
        }
        results
    }

    pub fn update_self_model(
        &mut self,
        order: SelfModelOrder,
        content: String,
        confidence: f64,
        coherence: f64,
    ) {
        let snapshot = SelfModelSnapshot {
            order,
            content,
            confidence,
            coherence,
            timestamp: self.iteration,
        };
        if self.self_models.len() >= 32 {
            self.self_models.remove(0);
        }
        self.self_models.push(snapshot);
    }

    pub fn get_self_model(&self, order: &SelfModelOrder) -> Option<&SelfModelSnapshot> {
        self.self_models.iter().rev().find(|s| &s.order == order)
    }

    pub fn self_model_coherence(&self) -> f64 {
        let orders = [
            SelfModelOrder::World,
            SelfModelOrder::Self_,
            SelfModelOrder::Meta,
        ];
        let present: Vec<f64> = orders
            .iter()
            .filter_map(|o| self.get_self_model(o))
            .map(|s| s.coherence)
            .collect();
        if present.is_empty() {
            return 0.0;
        }
        present.iter().sum::<f64>() / present.len() as f64
    }

    pub fn cross_order_consistency(&self) -> f64 {
        let world = self.get_self_model(&SelfModelOrder::World);
        let self_ = self.get_self_model(&SelfModelOrder::Self_);
        let meta = self.get_self_model(&SelfModelOrder::Meta);
        let w_s = match (world, self_) {
            (Some(w), Some(s)) => 1.0 - (w.confidence - s.confidence).abs(),
            _ => 0.5,
        };
        let s_m = match (self_, meta) {
            (Some(s), Some(m)) => 1.0 - (s.coherence - m.coherence).abs(),
            _ => 0.5,
        };
        (w_s + s_m) / 2.0
    }

    fn analyze_trace(&self, trace: &str) -> WeaknessReport {
        let words: Vec<&str> = trace.split_whitespace().collect();
        let error_keywords = ["error", "fail", "crash", "bug", "wrong", "incorrect"];
        let keyword_count = words
            .iter()
            .filter(|w| error_keywords.contains(&w.to_lowercase().as_str()))
            .count();
        let confidence = (keyword_count as f64 / words.len().max(1) as f64 * 5.0).clamp(0.1, 0.95);
        WeaknessReport {
            weakness: trace.chars().take(100).collect(),
            confidence,
            affected_module: "unknown".into(),
            trace_ref: format!("trace-{}", self.iteration),
        }
    }
}

impl Default for SelfHarnessLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_harness_initial() {
        let h = SelfHarnessLoop::new();
        assert!((h.success_rate() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_self_harness_full_loop() {
        let mut h = SelfHarnessLoop::new();
        let traces = vec![
            "error: null pointer in module_x".into(),
            "fail: assertion failed in module_y".into(),
        ];
        let results = h.closed_loop(traces);
        assert!(!results.is_empty());
        assert!(h.iteration > 0);
        assert!(h.average_improvement() >= 0.0);
    }

    #[test]
    fn test_top_weaknesses_sorted() {
        let mut h = SelfHarnessLoop::new();
        h.mine_weaknesses(vec!["error: critical crash".into(), "minor issue".into()]);
        let top = h.top_weaknesses(1);
        assert_eq!(top.len(), 1);
    }

    #[test]
    fn test_self_model_update_retrieval() {
        let mut h = SelfHarnessLoop::new();
        h.update_self_model(SelfModelOrder::World, "world is stable".into(), 0.8, 0.9);
        h.update_self_model(SelfModelOrder::Meta, "meta insight".into(), 0.7, 0.6);
        let world = h.get_self_model(&SelfModelOrder::World);
        assert!(world.is_some());
        assert!((world.unwrap().confidence - 0.8).abs() < 1e-6);
        let meta = h.get_self_model(&SelfModelOrder::Meta);
        assert!(meta.is_some());
        assert_eq!(meta.unwrap().content, "meta insight");
    }

    #[test]
    fn test_self_model_coherence() {
        let mut h = SelfHarnessLoop::new();
        assert!((h.self_model_coherence() - 0.0).abs() < 1e-6);
        h.update_self_model(SelfModelOrder::World, "".into(), 0.5, 0.8);
        h.update_self_model(SelfModelOrder::Self_, "".into(), 0.5, 0.6);
        let coh = h.self_model_coherence();
        assert!((coh - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_cross_order_consistency() {
        let mut h = SelfHarnessLoop::new();
        let c0 = h.cross_order_consistency();
        assert!((c0 - 0.5).abs() < 1e-6);
        h.update_self_model(SelfModelOrder::World, "".into(), 0.9, 0.9);
        h.update_self_model(SelfModelOrder::Self_, "".into(), 0.7, 0.7);
        h.update_self_model(SelfModelOrder::Meta, "".into(), 0.5, 0.5);
        let c = h.cross_order_consistency();
        assert!(c > 0.5);
    }
}
