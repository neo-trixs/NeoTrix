
#[derive(Debug, Clone)]
pub struct NetworkEvolution {
    phase: EvolutionPhase,
    metrics: EvolutionMetrics,
    heuristics: Vec<NetworkHeuristic>,
    mutation_log: Vec<MutationRecord>,
    cycle: u64,
    auto_evolve_interval: u64,
    previous_heuristics: Vec<NetworkHeuristic>,
    previous_metrics: EvolutionMetrics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvolutionPhase {
    Idle, Monitor, Distill, Propose, Apply, Verify, Rollback, Commit,
}

#[derive(Debug, Clone)]
pub struct EvolutionMetrics {
    pub total_extractions: u64,
    pub successful_extractions: u64,
    pub avg_latency_ms: f64,
    pub tls_success_rate: f64,
    pub selector_success_rate: f64,
    pub proxy_avg_score: f64,
    pub captcha_success_rate: f64,
    pub vsa_context: [u64; 4],
}

impl Default for EvolutionMetrics {
    fn default() -> Self {
        Self {
            total_extractions: 0,
            successful_extractions: 0,
            avg_latency_ms: 0.0,
            tls_success_rate: 0.5,
            selector_success_rate: 0.5,
            proxy_avg_score: 0.5,
            captcha_success_rate: 0.5,
            vsa_context: [0; 4],
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetworkHeuristic {
    pub name: String,
    pub domain: HeuristicDomain,
    pub weight: f64,
    pub success_count: u32,
    pub fail_count: u32,
    pub vsa_encoding: [u64; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeuristicDomain {
    TlsFingerprint, ProxySelection, SelectorStrategy, ExtractionStrategy, CaptchaSolver,
}

#[derive(Debug, Clone)]
pub struct MutationRecord {
    pub cycle: u64,
    pub phase: EvolutionPhase,
    pub heuristic_name: String,
    pub old_weight: f64,
    pub new_weight: f64,
    pub e8_mode: u8,
    pub success: Option<bool>,
}

#[derive(Debug, Clone)]
pub enum EvolutionAction {
    DistillHeuristics,
    ProposeMutations,
    ApplyMutations(Vec<NetworkHeuristic>),
    VerifyChanges,
    RollbackChanges,
    CommitChanges,
    None,
}

impl NetworkEvolution {
    pub fn new(auto_evolve_interval: u64) -> Self {
        Self {
            phase: EvolutionPhase::Monitor,
            metrics: EvolutionMetrics::default(),
            heuristics: Self::default_heuristics(),
            mutation_log: Vec::new(),
            cycle: 0,
            auto_evolve_interval,
            previous_heuristics: Vec::new(),
            previous_metrics: EvolutionMetrics::default(),
        }
    }

    pub fn tick(&mut self) -> EvolutionAction {
        self.cycle += 1;
        match self.phase {
            EvolutionPhase::Monitor | EvolutionPhase::Idle => {
                if self.cycle % self.auto_evolve_interval == 0 {
                    self.phase = EvolutionPhase::Distill;
                    EvolutionAction::DistillHeuristics
                } else {
                    EvolutionAction::None
                }
            }
            EvolutionPhase::Distill => {
                self.phase = EvolutionPhase::Propose;
                EvolutionAction::ProposeMutations
            }
            EvolutionPhase::Propose => {
                let mutations = self.propose_mutations(&self.heuristics);
                if mutations.is_empty() {
                    self.phase = EvolutionPhase::Monitor;
                    return EvolutionAction::None;
                }
                self.phase = EvolutionPhase::Apply;
                EvolutionAction::ApplyMutations(mutations)
            }
            EvolutionPhase::Apply => {
                self.phase = EvolutionPhase::Verify;
                EvolutionAction::VerifyChanges
            }
            EvolutionPhase::Verify => {
                let improved = self.verify(&self.heuristics);
                if improved {
                    self.phase = EvolutionPhase::Commit;
                    EvolutionAction::CommitChanges
                } else {
                    self.phase = EvolutionPhase::Rollback;
                    EvolutionAction::RollbackChanges
                }
            }
            EvolutionPhase::Rollback => {
                self.rollback();
                self.phase = EvolutionPhase::Monitor;
                EvolutionAction::None
            }
            EvolutionPhase::Commit => {
                self.commit();
                self.phase = EvolutionPhase::Monitor;
                EvolutionAction::None
            }
        }
    }

    pub fn record_metric(&mut self, metric: &str, value: f64) {
        match metric {
            "total_extractions" => self.metrics.total_extractions = value as u64,
            "successful_extractions" => self.metrics.successful_extractions = value as u64,
            "avg_latency_ms" => self.metrics.avg_latency_ms = value,
            "tls_success_rate" => self.metrics.tls_success_rate = value,
            "selector_success_rate" => self.metrics.selector_success_rate = value,
            "proxy_avg_score" => self.metrics.proxy_avg_score = value,
            "captcha_success_rate" => self.metrics.captcha_success_rate = value,
            _ => {}
        }
    }

    pub fn record_outcome(&mut self, heuristic: &str, success: bool) {
        if let Some(h) = self.heuristics.iter_mut().find(|h| h.name == heuristic) {
            if success { h.success_count += 1; } else { h.fail_count += 1; }
            h.weight = h.success_count as f64 / (h.success_count + h.fail_count).max(1) as f64;
        }
    }

    pub fn distill_heuristics(&self) -> Vec<NetworkHeuristic> {
        let mut distilled = self.heuristics.clone();
        for h in &mut distilled {
            let total = h.success_count + h.fail_count;
            if total > 10 {
                let rate = h.success_count as f64 / total as f64;
                h.weight = match h.domain {
                    HeuristicDomain::TlsFingerprint => (rate + self.metrics.tls_success_rate) / 2.0,
                    HeuristicDomain::ProxySelection => (rate + self.metrics.proxy_avg_score) / 2.0,
                    HeuristicDomain::SelectorStrategy => (rate + self.metrics.selector_success_rate) / 2.0,
                    HeuristicDomain::CaptchaSolver => (rate + self.metrics.captcha_success_rate) / 2.0,
                    HeuristicDomain::ExtractionStrategy => rate,
                };
            }
        }
        distilled
    }

    pub fn propose_mutations(&self, base: &[NetworkHeuristic]) -> Vec<NetworkHeuristic> {
        let mut mutations = Vec::new();
        for (i, h) in base.iter().enumerate() {
            if h.success_count + h.fail_count < 3 { continue; }
            let mode = (i * 7 + self.cycle as usize) as u8 % 64;
            if mode % 4 == 0 {
                mutations.push(Self::e8_mutate(h, mode));
            }
            if mode % 16 == 0 && i + 1 < base.len() {
                mutations.push(Self::crossover(&base[i], &base[i + 1], mode));
            }
        }
        mutations
    }

    pub fn apply_mutation(&mut self, heuristics: &[NetworkHeuristic]) {
        self.previous_heuristics = self.heuristics.clone();
        self.previous_metrics = self.metrics.clone();
        for mutation in heuristics {
            if let Some(existing) = self.heuristics.iter_mut().find(|h| h.name == mutation.name) {
                self.mutation_log.push(MutationRecord {
                    cycle: self.cycle,
                    phase: self.phase,
                    heuristic_name: mutation.name.clone(),
                    old_weight: existing.weight,
                    new_weight: mutation.weight,
                    e8_mode: 0,
                    success: None,
                });
                *existing = mutation.clone();
            }
        }
    }

    pub fn verify(&self, _heuristics: &[NetworkHeuristic]) -> bool {
        let prev_overall = (self.previous_metrics.selector_success_rate
            + self.previous_metrics.tls_success_rate
            + self.previous_metrics.proxy_avg_score) / 3.0;
        let curr_overall = (self.metrics.selector_success_rate
            + self.metrics.tls_success_rate
            + self.metrics.proxy_avg_score) / 3.0;
        curr_overall >= prev_overall
    }

    pub fn rollback(&mut self) {
        std::mem::swap(&mut self.heuristics, &mut self.previous_heuristics);
        self.metrics = self.previous_metrics.clone();
        if let Some(last) = self.mutation_log.last_mut() {
            last.success = Some(false);
        }
    }

    pub fn commit(&mut self) {
        self.previous_heuristics.clear();
        if let Some(last) = self.mutation_log.last_mut() {
            last.success = Some(true);
        }
    }

    pub fn e8_mutate(heuristic: &NetworkHeuristic, mode: u8) -> NetworkHeuristic {
        let mut mutated = heuristic.clone();
        match mode % 4 {
            0 => mutated.weight = (heuristic.weight * 1.1).min(1.0),
            1 => mutated.weight = (heuristic.weight * 0.9).max(0.0),
            2 => mutated.weight = 1.0 - heuristic.weight,
            3 => mutated.weight = heuristic.weight + (rand::random::<f64>() - 0.5) * 0.2,
            _ => {}
        }
        mutated.weight = mutated.weight.clamp(0.0, 1.0);
        mutated.vsa_encoding = Self::compute_heuristic_vsa(&mutated.name, mutated.weight);
        mutated
    }

    pub fn crossover(a: &NetworkHeuristic, b: &NetworkHeuristic, _mode: u8) -> NetworkHeuristic {
        NetworkHeuristic {
            name: format!("{}_cross_{}", a.name, b.name),
            domain: a.domain,
            weight: (a.weight + b.weight) / 2.0,
            success_count: a.success_count + b.success_count / 2,
            fail_count: a.fail_count + b.fail_count / 2,
            vsa_encoding: [
                a.vsa_encoding[0] ^ b.vsa_encoding[0],
                a.vsa_encoding[1] ^ b.vsa_encoding[1],
                a.vsa_encoding[2].wrapping_add(b.vsa_encoding[2]),
                a.vsa_encoding[3].wrapping_mul(b.vsa_encoding[3]),
            ],
        }
    }

    fn default_heuristics() -> Vec<NetworkHeuristic> {
        let defaults = [
            ("tls_prefer_chrome", HeuristicDomain::TlsFingerprint, 0.8),
            ("tls_rotate_on_failure", HeuristicDomain::TlsFingerprint, 0.7),
            ("proxy_region_priority", HeuristicDomain::ProxySelection, 0.6),
            ("proxy_weighted_random", HeuristicDomain::ProxySelection, 0.7),
            ("selector_css_first", HeuristicDomain::SelectorStrategy, 0.8),
            ("selector_xpath_fallback", HeuristicDomain::SelectorStrategy, 0.6),
            ("extract_required_only", HeuristicDomain::ExtractionStrategy, 0.5),
            ("extract_validate_types", HeuristicDomain::ExtractionStrategy, 0.7),
            ("captcha_2captcha_first", HeuristicDomain::CaptchaSolver, 0.7),
            ("captcha_ocr_fallback", HeuristicDomain::CaptchaSolver, 0.4),
        ];
        defaults.iter().map(|(name, domain, weight)| {
            NetworkHeuristic {
                name: name.to_string(),
                domain: *domain,
                weight: *weight,
                success_count: 0,
                fail_count: 0,
                vsa_encoding: Self::compute_heuristic_vsa(name, *weight),
            }
        }).collect()
    }

    fn compute_heuristic_vsa(name: &str, weight: f64) -> [u64; 4] {
        let w_bytes = weight.to_le_bytes();
        let combined: Vec<u8> = name.bytes().chain(w_bytes.iter().copied()).collect();
        let h1 = combined.iter().enumerate().fold(0u64, |acc, (i, b)| acc.wrapping_mul(31).wrapping_add(*b as u64 ^ (i as u64 * 7)));
        let h2 = combined.iter().rev().enumerate().fold(0u64, |acc, (i, b)| acc.wrapping_mul(37).wrapping_add(*b as u64 ^ (i as u64 * 13)));
        let h3 = combined.iter().step_by(2).fold(0u64, |acc, b| acc.wrapping_mul(41).wrapping_add(*b as u64));
        let h4 = combined.iter().skip(1).step_by(2).fold(0u64, |acc, b| acc.wrapping_mul(43).wrapping_add(*b as u64));
        [h1 ^ h3, h2 ^ h4, h1.wrapping_add(h2), h3.wrapping_add(h4)]
    }

    pub fn overall_health(&self) -> f64 {
        (self.metrics.tls_success_rate
            + self.metrics.selector_success_rate
            + self.metrics.proxy_avg_score
            + self.metrics.captcha_success_rate) / 4.0
    }

    pub fn evolution_report(&self) -> String {
        format!(
            "NetEvo[cycle={} phase={:?} health={:.2} heuristics={} mutations={}]",
            self.cycle, self.phase, self.overall_health(),
            self.heuristics.len(), self.mutation_log.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_has_default_heuristics() {
        let evo = NetworkEvolution::new(10);
        assert_eq!(evo.heuristics.len(), 10);
        assert_eq!(evo.phase, EvolutionPhase::Monitor);
    }

    #[test]
    fn test_tick_monitor_to_distill_at_interval() {
        let mut evo = NetworkEvolution::new(5);
        for _ in 0..4 { evo.tick(); }
        let action = evo.tick();
        assert!(matches!(action, EvolutionAction::DistillHeuristics));
    }

    #[test]
    fn test_tick_idle_no_action() {
        let mut evo = NetworkEvolution::new(100);
        let action = evo.tick();
        assert!(matches!(action, EvolutionAction::None));
    }

    #[test]
    fn test_full_seal_cycle() {
        let mut evo = NetworkEvolution::new(3);
        let actions: Vec<EvolutionAction> = (0..20).map(|_| evo.tick()).collect();
        let distill_count = actions.iter().filter(|a| matches!(a, EvolutionAction::DistillHeuristics)).count();
        assert!(distill_count >= 1);
    }

    #[test]
    fn test_record_metric_updates() {
        let mut evo = NetworkEvolution::new(10);
        evo.record_metric("tls_success_rate", 0.85);
        assert!((evo.metrics.tls_success_rate - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_record_outcome_updates_weight() {
        let mut evo = NetworkEvolution::new(10);
        evo.record_outcome("tls_prefer_chrome", true);
        evo.record_outcome("tls_prefer_chrome", true);
        evo.record_outcome("tls_prefer_chrome", false);
        let h = evo.heuristics.iter().find(|h| h.name == "tls_prefer_chrome").unwrap();
        assert!((h.weight - 2.0/3.0).abs() < 0.01);
    }

    #[test]
    fn test_e8_mutate_modifies_weight() {
        let h = NetworkHeuristic {
            name: "test".into(), domain: HeuristicDomain::TlsFingerprint,
            weight: 0.5, success_count: 10, fail_count: 0,
            vsa_encoding: [0; 4],
        };
        let mutated = NetworkEvolution::e8_mutate(&h, 0);
        assert!((mutated.weight - 0.55).abs() < 0.01);
    }

    #[test]
    fn test_crossover_combines_weights() {
        let a = NetworkHeuristic {
            name: "a".into(), domain: HeuristicDomain::TlsFingerprint,
            weight: 0.8, success_count: 10, fail_count: 2,
            vsa_encoding: [1; 4],
        };
        let b = NetworkHeuristic {
            name: "b".into(), domain: HeuristicDomain::ProxySelection,
            weight: 0.4, success_count: 5, fail_count: 5,
            vsa_encoding: [2; 4],
        };
        let child = NetworkEvolution::crossover(&a, &b, 0);
        assert!((child.weight - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_verify_accepts_improvement() {
        let mut evo = NetworkEvolution::new(10);
        evo.previous_metrics = EvolutionMetrics {
            tls_success_rate: 0.5, selector_success_rate: 0.5, proxy_avg_score: 0.5, ..Default::default()
        };
        evo.metrics = EvolutionMetrics {
            tls_success_rate: 0.8, selector_success_rate: 0.7, proxy_avg_score: 0.6, ..Default::default()
        };
        assert!(evo.verify(&[]));
    }

    #[test]
    fn test_verify_rejects_degradation() {
        let mut evo = NetworkEvolution::new(10);
        evo.previous_metrics = EvolutionMetrics {
            tls_success_rate: 0.8, selector_success_rate: 0.8, proxy_avg_score: 0.8, ..Default::default()
        };
        evo.metrics = EvolutionMetrics {
            tls_success_rate: 0.4, selector_success_rate: 0.5, proxy_avg_score: 0.3, ..Default::default()
        };
        assert!(!evo.verify(&[]));
    }

    #[test]
    fn test_rollback_restores_previous() {
        let mut evo = NetworkEvolution::new(10);
        let orig_weight = evo.heuristics[0].weight;
        evo.previous_heuristics = evo.heuristics.clone();
        evo.heuristics[0].weight = 1.0;
        evo.rollback();
        assert!((evo.heuristics[0].weight - orig_weight).abs() < 0.01);
    }

    #[test]
    fn test_health_composite() {
        let mut evo = NetworkEvolution::new(10);
        evo.metrics = EvolutionMetrics {
            tls_success_rate: 0.9, selector_success_rate: 0.8, proxy_avg_score: 0.7,
            captcha_success_rate: 0.6, ..Default::default()
        };
        assert!((evo.overall_health() - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_evolution_report_format() {
        let evo = NetworkEvolution::new(10);
        let report = evo.evolution_report();
        assert!(report.starts_with("NetEvo["));
        assert!(report.contains("cycle="));
    }

    #[test]
    fn test_apply_mutation_records_log() {
        let mut evo = NetworkEvolution::new(10);
        let mutations = vec![NetworkHeuristic {
            name: "tls_prefer_chrome".into(), domain: HeuristicDomain::TlsFingerprint,
            weight: 0.9, success_count: 0, fail_count: 0, vsa_encoding: [0; 4],
        }];
        evo.apply_mutation(&mutations);
        assert_eq!(evo.mutation_log.len(), 1);
    }

    #[test]
    fn test_vsa_encoding_deterministic() {
        let a = NetworkEvolution::compute_heuristic_vsa("test", 0.5);
        let b = NetworkEvolution::compute_heuristic_vsa("test", 0.5);
        assert_eq!(a, b);
    }

    #[test]
    fn test_vsa_encoding_changes_with_weight() {
        let a = NetworkEvolution::compute_heuristic_vsa("test", 0.5);
        let b = NetworkEvolution::compute_heuristic_vsa("test", 0.6);
        assert_ne!(a, b);
    }
}
