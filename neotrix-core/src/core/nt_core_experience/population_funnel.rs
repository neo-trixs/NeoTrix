// G398.5 + G412.5: Parallel population funnel + self-stopping governor
// scholar-loop inspired: propose N in parallel, smoke-screen, climb survivors
use crate::core::nt_core_hcube::vsa_vector::{MapVsaBackend, VsaBackend, VsaVector};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelConfig {
    pub initial_population: usize,
    pub top_k_survivors: usize,
    pub max_iterations: usize,
    pub plateau_window: usize,
    pub plateau_threshold: f64,
    pub smoke_screen_gates: Vec<String>,
}

impl Default for FunnelConfig {
    fn default() -> Self {
        Self {
            initial_population: 8,
            top_k_survivors: 3,
            max_iterations: 5,
            plateau_window: 3,
            plateau_threshold: 0.02,
            smoke_screen_gates: vec!["syntax".into(), "semantic".into(), "novelty".into()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalVariant {
    pub id: u64,
    pub description: String,
    pub score: f64,
    pub gate_results: Vec<(String, bool)>,
    pub vsa_signature: VsaVector,
    pub iteration: usize,
    pub parent_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelRound {
    pub round: usize,
    pub variants: Vec<ProposalVariant>,
    pub best_score: f64,
    pub mean_score: f64,
    pub diversity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationFunnel {
    pub config: FunnelConfig,
    pub rounds: Vec<FunnelRound>,
    pub survivors: Vec<ProposalVariant>,
    pub governor: SelfStoppingGovernor,
    pub total_proposals_evaluated: u64,
}

impl PopulationFunnel {
    pub fn new(config: FunnelConfig) -> Self {
        Self {
            governor: SelfStoppingGovernor::new(config.plateau_window, config.plateau_threshold),
            config,
            rounds: Vec::new(),
            survivors: Vec::new(),
            total_proposals_evaluated: 0,
        }
    }

    pub fn seed_population(&mut self, variants: Vec<ProposalVariant>) {
        self.survivors = variants;
    }

    pub fn run_smoke_screen(&mut self, round: usize) -> Vec<ProposalVariant> {
        let mut passed = Vec::new();
        for variant in &self.survivors {
            let gates_passed: Vec<(String, bool)> = self
                .config
                .smoke_screen_gates
                .iter()
                .map(|g| (g.clone(), self.evaluate_gate(g, variant)))
                .collect();
            let all_pass = gates_passed.iter().all(|(_, p)| *p);
            if all_pass {
                let mut v = variant.clone();
                v.gate_results = gates_passed;
                v.iteration = round;
                passed.push(v);
            }
        }
        passed
    }

    fn evaluate_gate(&self, _gate: &str, _variant: &ProposalVariant) -> bool {
        true
    }

    pub fn select_survivors(&mut self, variants: &[ProposalVariant]) -> Vec<ProposalVariant> {
        let mut sorted = variants.to_vec();
        sorted.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.truncate(self.config.top_k_survivors);
        sorted
    }

    pub fn record_round(&mut self, round: usize, variants: Vec<ProposalVariant>) {
        let best_score = variants
            .iter()
            .map(|v| v.score)
            .fold(f64::NEG_INFINITY, f64::max);
        let mean_score = if variants.is_empty() {
            0.0
        } else {
            variants.iter().map(|v| v.score).sum::<f64>() / variants.len() as f64
        };

        let diversity = if variants.len() < 2 {
            1.0
        } else {
            let mut total_sim = 0.0;
            let mut pairs = 0;
            for i in 0..variants.len() {
                for j in (i + 1)..variants.len() {
                    let sim = MapVsaBackend
                        .similarity(&variants[i].vsa_signature, &variants[j].vsa_signature);
                    total_sim += sim;
                    pairs += 1;
                }
            }
            1.0 - total_sim / pairs as f64
        };

        self.rounds.push(FunnelRound {
            round,
            variants: variants.clone(),
            best_score,
            mean_score,
            diversity,
        });

        self.total_proposals_evaluated += variants.len() as u64;
        self.governor.record_score(best_score);
    }

    pub fn should_stop(&self) -> bool {
        self.rounds.len() >= self.config.max_iterations || self.governor.should_stop()
    }

    pub fn best_variant(&self) -> Option<&ProposalVariant> {
        self.survivors.iter().max_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn summary(&self) -> String {
        format!(
            "PopulationFunnel: {} rounds, {} evaluated, {} survivors, best={:.4}, stopped={}",
            self.rounds.len(),
            self.total_proposals_evaluated,
            self.survivors.len(),
            self.best_variant().map(|v| v.score).unwrap_or(0.0),
            self.should_stop()
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfStoppingGovernor {
    pub plateau_window: usize,
    pub plateau_threshold: f64,
    pub scores: VecDeque<f64>,
    pub best_ever: f64,
    pub rounds_without_improvement: usize,
}

impl SelfStoppingGovernor {
    pub fn new(plateau_window: usize, plateau_threshold: f64) -> Self {
        Self {
            plateau_window,
            plateau_threshold,
            scores: VecDeque::with_capacity(plateau_window + 1),
            best_ever: f64::NEG_INFINITY,
            rounds_without_improvement: 0,
        }
    }

    pub fn record_score(&mut self, score: f64) {
        if score > self.best_ever + self.plateau_threshold {
            self.best_ever = score;
            self.rounds_without_improvement = 0;
        } else {
            self.rounds_without_improvement += 1;
        }
        self.scores.push_back(score);
        while self.scores.len() > self.plateau_window + 1 {
            self.scores.pop_front();
        }
    }

    pub fn should_stop(&self) -> bool {
        if self.scores.len() < self.plateau_window {
            return false;
        }
        self.rounds_without_improvement >= self.plateau_window
    }

    pub fn improvement_rate(&self) -> f64 {
        if self.scores.len() < 2 {
            return 0.0;
        }
        let recent: Vec<&f64> = self.scores.iter().rev().take(self.plateau_window).collect();
        if recent.len() < 2 {
            return 0.0;
        }
        (recent[0] - recent[recent.len() - 1]).abs() / recent.len() as f64
    }
}
