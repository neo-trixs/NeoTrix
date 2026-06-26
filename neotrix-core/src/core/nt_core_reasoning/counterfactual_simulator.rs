use std::collections::{HashMap, VecDeque};

use rand::Rng;

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

#[derive(Debug, Clone)]
pub struct CounterfactualConfig {
    pub max_simulations: usize,
    pub perturbation_strength: f64,
    pub simulation_depth: usize,
    pub min_divergence: f64,
    pub diversity_penalty: f64,
    pub outcome_confidence_threshold: f64,
}

impl Default for CounterfactualConfig {
    fn default() -> Self {
        Self {
            max_simulations: 5,
            perturbation_strength: 0.15,
            simulation_depth: 3,
            min_divergence: 0.1,
            diversity_penalty: 0.2,
            outcome_confidence_threshold: 0.3,
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum CounterfactualType {
    InputPerturbation,
    ActionAblation,
    StructuralIntervention,
    HypotheticalAddition,
    TemporalShift,
    AlternativePath,
}

#[derive(Debug, Clone)]
pub struct SimulatedOutcome {
    pub predicted_success: f64,
    pub outcome_state: Vec<u8>,
    pub reasoning_steps: Vec<String>,
    pub reward: f64,
    pub uncertainty: f64,
}

#[derive(Debug, Clone)]
pub struct CounterfactualScenario {
    pub id: u64,
    pub cf_type: CounterfactualType,
    pub factual_state: Vec<u8>,
    pub counterfactual_state: Vec<u8>,
    pub perturbation: Vec<u8>,
    pub divergence: f64,
    pub simulated_outcome: Option<SimulatedOutcome>,
    pub confidence: f64,
    pub is_valid: bool,
}

#[derive(Debug, Clone)]
pub struct CounterfactualSimulator {
    pub config: CounterfactualConfig,
    pub scenarios: Vec<CounterfactualScenario>,
    pub next_id: u64,
    pub vsa: Option<QuantizedVSA>,
    pub simulation_history: VecDeque<CounterfactualScenario>,
    pub total_simulations: usize,
    pub successful_simulations: usize,
}

#[derive(Debug, Clone)]
pub struct CounterfactualStats {
    pub total_scenarios: usize,
    pub simulated_count: usize,
    pub avg_divergence: f64,
    pub best_outcome: f64,
    pub worst_outcome: f64,
    pub type_distribution: HashMap<CounterfactualType, usize>,
    pub simulation_success_rate: f64,
}

impl CounterfactualSimulator {
    pub fn new(config: CounterfactualConfig) -> Self {
        Self {
            config,
            scenarios: Vec::new(),
            next_id: 1,
            vsa: None,
            simulation_history: VecDeque::new(),
            total_simulations: 0,
            successful_simulations: 0,
        }
    }

    pub fn generate_scenarios(
        &mut self,
        factual: &[u8],
        cf_type: CounterfactualType,
        n: usize,
    ) -> Vec<u64> {
        let n = n.min(self.config.max_simulations);
        let mut ids = Vec::with_capacity(n);

        for _ in 0..n {
            let counterfactual_state = match cf_type {
                CounterfactualType::InputPerturbation
                | CounterfactualType::ActionAblation
                | CounterfactualType::HypotheticalAddition
                | CounterfactualType::TemporalShift
                | CounterfactualType::AlternativePath => {
                    self.perturb_vsa(factual, self.config.perturbation_strength)
                }
                CounterfactualType::StructuralIntervention => {
                    let mut rng = rand::thread_rng();
                    let target_len = factual.len();
                    let target_bits: Vec<usize> = (0..target_len)
                        .filter(|_| rng.gen_bool(self.config.perturbation_strength))
                        .collect();
                    let values: Vec<u8> = target_bits.iter().map(|_| rng.gen()).collect();
                    self.structural_intervention(factual, &target_bits, &values)
                }
            };

            let perturbation: Vec<u8> = factual
                .iter()
                .zip(counterfactual_state.iter())
                .map(|(a, b)| a ^ b)
                .collect();

            let divergence = self.vsa_distance(factual, &counterfactual_state);

            if divergence < self.config.min_divergence {
                continue;
            }

            let id = self.next_id;
            self.next_id += 1;

            let scenario = CounterfactualScenario {
                id,
                cf_type: cf_type.clone(),
                factual_state: factual.to_vec(),
                counterfactual_state,
                perturbation,
                divergence,
                simulated_outcome: None,
                confidence: 0.0,
                is_valid: false,
            };

            self.scenarios.push(scenario);
            ids.push(id);
        }

        ids
    }

    pub fn perturb_vsa(&self, state: &[u8], strength: f64) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        state
            .iter()
            .map(|&byte| {
                if rng.gen_bool(strength) {
                    let bit_flip_mask: u8 = (0..8)
                        .filter(|_| rng.gen_bool(0.5))
                        .fold(0u8, |acc, bit| acc | (1 << bit));
                    byte ^ bit_flip_mask
                } else {
                    byte
                }
            })
            .collect()
    }

    pub fn structural_intervention(
        &self,
        state: &[u8],
        target_bits: &[usize],
        values: &[u8],
    ) -> Vec<u8> {
        let mut result = state.to_vec();
        for (&idx, &val) in target_bits.iter().zip(values.iter()) {
            if idx < result.len() {
                result[idx] = val;
            }
        }
        result
    }

    pub fn simulate_scenario(&mut self, scenario_id: u64) -> Option<SimulatedOutcome> {
        let idx = self.scenarios.iter().position(|s| s.id == scenario_id)?;
        let scenario = &self.scenarios[idx];

        if !scenario.is_valid && scenario.divergence < self.config.min_divergence {
            return None;
        }

        let mut rng = rand::thread_rng();
        let mut steps = Vec::with_capacity(self.config.simulation_depth);
        let mut current_state = scenario.counterfactual_state.clone();

        for step in 0..self.config.simulation_depth {
            let step_desc = format!(
                "step_{}: cf_type={:?} divergence={:.3}",
                step + 1,
                scenario.cf_type,
                scenario.divergence
            );
            steps.push(step_desc);

            current_state =
                self.perturb_vsa(&current_state, self.config.perturbation_strength * 0.5);
        }

        let base_success = 1.0 - scenario.divergence.min(1.0);
        let noise: f64 = rng.gen_range(-0.1..0.1);
        let predicted_success = (base_success + noise).clamp(0.0, 1.0);

        let outcome_uncertainty = 1.0
            - (scenario
                .confidence
                .max(self.config.outcome_confidence_threshold));
        let uncertainty = outcome_uncertainty.max(0.05);

        let reward = predicted_success * (1.0 - uncertainty * 0.5);

        let outcome = SimulatedOutcome {
            predicted_success,
            outcome_state: current_state,
            reasoning_steps: steps,
            reward,
            uncertainty,
        };

        let scenario = &mut self.scenarios[idx];
        scenario.simulated_outcome = Some(outcome.clone());
        scenario.is_valid = true;

        self.total_simulations += 1;
        if predicted_success > self.config.outcome_confidence_threshold {
            self.successful_simulations += 1;
        }

        self.simulation_history.push_back(scenario.clone());
        if self.simulation_history.len() > 100 {
            self.simulation_history.pop_front();
        }

        Some(outcome)
    }

    pub fn simulate_all(&mut self) -> Vec<(u64, SimulatedOutcome)> {
        let ids: Vec<u64> = self.scenarios.iter().map(|s| s.id).collect();
        ids.iter()
            .filter_map(|&id| self.simulate_scenario(id).map(|o| (id, o)))
            .collect()
    }

    pub fn compare_factual_vs_counterfactual(&self, factual_reward: f64, scenario_id: u64) -> f64 {
        let scenario = match self.scenarios.iter().find(|s| s.id == scenario_id) {
            Some(s) => s,
            None => return 0.0,
        };
        let cf_reward = scenario
            .simulated_outcome
            .as_ref()
            .map(|o| o.reward)
            .unwrap_or(0.0);
        factual_reward - cf_reward
    }

    pub fn best_counterfactual(&self) -> Option<&CounterfactualScenario> {
        self.scenarios
            .iter()
            .filter(|s| s.simulated_outcome.is_some())
            .max_by(|a, b| {
                let a_r = a.simulated_outcome.as_ref().map(|o| o.reward).unwrap_or(0.0);
                let b_r = b.simulated_outcome.as_ref().map(|o| o.reward).unwrap_or(0.0);
                a_r.partial_cmp(&b_r).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    pub fn diverse_counterfactuals(&self, n: usize) -> Vec<&CounterfactualScenario> {
        let mut selected: Vec<&CounterfactualScenario> = Vec::new();
        let mut candidates: Vec<&CounterfactualScenario> =
            self.scenarios.iter().filter(|s| s.is_valid).collect();

        if candidates.is_empty() {
            return selected;
        }

        candidates.sort_by(|a, b| {
            let a_reward = a
                .simulated_outcome
                .as_ref()
                .map(|o| o.reward)
                .unwrap_or(0.0);
            let b_reward = b
                .simulated_outcome
                .as_ref()
                .map(|o| o.reward)
                .unwrap_or(0.0);
            b_reward
                .partial_cmp(&a_reward)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(best) = candidates.first() {
            selected.push(best);
        }

        for &candidate in candidates.iter() {
            if selected.len() >= n {
                break;
            }

            let too_close = selected.iter().any(|&sel| {
                self.vsa_distance(&candidate.counterfactual_state, &sel.counterfactual_state)
                    < self.config.diversity_penalty
            });

            if !too_close {
                selected.push(candidate);
            }
        }

        selected
    }

    pub fn vsa_distance(&self, a: &[u8], b: &[u8]) -> f64 {
        let max_len = a.len().max(b.len());
        let min_len = a.len().min(b.len());
        let mut diffs = 0u64;
        for i in 0..min_len {
            diffs += (a[i] ^ b[i]).count_ones() as u64;
        }
        diffs += (max_len - min_len) as u64 * 8;
        diffs as f64 / (max_len as f64 * 8.0)
    }

    pub fn divergence_matrix(&self) -> Vec<Vec<f64>> {
        let n = self.scenarios.len();
        let mut matrix = Vec::with_capacity(n);
        for i in 0..n {
            let mut row = Vec::with_capacity(n);
            for j in 0..n {
                let dist = self.vsa_distance(
                    &self.scenarios[i].counterfactual_state,
                    &self.scenarios[j].counterfactual_state,
                );
                row.push(dist);
            }
            matrix.push(row);
        }
        matrix
    }

    pub fn filter_similar(&self, threshold: f64) -> Vec<u64> {
        let mut to_remove = Vec::new();
        let n = self.scenarios.len();
        for i in 0..n {
            if to_remove.contains(&self.scenarios[i].id) {
                continue;
            }
            for j in (i + 1)..n {
                if to_remove.contains(&self.scenarios[j].id) {
                    continue;
                }
                let dist = self.vsa_distance(
                    &self.scenarios[i].counterfactual_state,
                    &self.scenarios[j].counterfactual_state,
                );
                if dist < threshold {
                    to_remove.push(self.scenarios[j].id);
                }
            }
        }
        to_remove
    }

    pub fn stats(&self) -> CounterfactualStats {
        let simulated: Vec<&CounterfactualScenario> = self
            .scenarios
            .iter()
            .filter(|s| s.simulated_outcome.is_some())
            .collect();

        let total_scenarios = self.scenarios.len();
        let simulated_count = simulated.len();

        let avg_divergence = if !self.scenarios.is_empty() {
            self.scenarios.iter().map(|s| s.divergence).sum::<f64>() / self.scenarios.len() as f64
        } else {
            0.0
        };

        let rewards: Vec<f64> = simulated
            .iter()
            .filter_map(|s| s.simulated_outcome.as_ref().map(|o| o.reward))
            .collect();

        let best_outcome = rewards.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let worst_outcome = rewards.iter().cloned().fold(f64::INFINITY, f64::min);

        let mut type_distribution: HashMap<CounterfactualType, usize> = HashMap::new();
        for s in &self.scenarios {
            *type_distribution.entry(s.cf_type.clone()).or_insert(0) += 1;
        }

        let simulation_success_rate = if self.total_simulations > 0 {
            self.successful_simulations as f64 / self.total_simulations as f64
        } else {
            0.0
        };

        CounterfactualStats {
            total_scenarios,
            simulated_count,
            avg_divergence,
            best_outcome,
            worst_outcome,
            type_distribution,
            simulation_success_rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_factual() -> Vec<u8> {
        let mut rng = rand::thread_rng();
        (0..64).map(|_| rng.gen()).collect()
    }

    #[test]
    fn test_new_simulator() {
        let config = CounterfactualConfig::default();
        let sim = CounterfactualSimulator::new(config);
        assert_eq!(sim.config.max_simulations, 5);
        assert_eq!(sim.config.perturbation_strength, 0.15);
        assert_eq!(sim.config.simulation_depth, 3);
        assert!(sim.scenarios.is_empty());
        assert_eq!(sim.next_id, 1);
        assert_eq!(sim.total_simulations, 0);
    }

    #[test]
    fn test_generate_input_perturbation() {
        let mut sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let factual = make_factual();
        let ids = sim.generate_scenarios(&factual, CounterfactualType::InputPerturbation, 3);
        assert!(ids.len() <= 3);
        assert_eq!(sim.scenarios.len(), ids.len());
        for id in &ids {
            let s = sim.scenarios.iter().find(|s| s.id == *id).unwrap();
            assert_eq!(s.cf_type, CounterfactualType::InputPerturbation);
            assert!(s.divergence >= 0.0);
            assert_eq!(s.factual_state, factual);
        }
    }

    #[test]
    fn test_generate_action_ablation() {
        let mut sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let factual = make_factual();
        let ids = sim.generate_scenarios(&factual, CounterfactualType::ActionAblation, 2);
        assert!(ids.len() <= 2);
        for id in &ids {
            let s = &sim.scenarios.iter().find(|s| s.id == *id).unwrap().cf_type;
            assert_eq!(*s, CounterfactualType::ActionAblation);
        }
    }

    #[test]
    fn test_generate_structural_intervention() {
        let mut sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let factual = make_factual();
        let ids = sim.generate_scenarios(&factual, CounterfactualType::StructuralIntervention, 2);
        assert!(ids.len() <= 2);
        for id in &ids {
            let s = &sim.scenarios.iter().find(|s| s.id == *id).unwrap().cf_type;
            assert_eq!(*s, CounterfactualType::StructuralIntervention);
        }
    }

    #[test]
    fn test_generate_hypothetical_addition() {
        let mut sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let factual = make_factual();
        let ids = sim.generate_scenarios(&factual, CounterfactualType::HypotheticalAddition, 2);
        assert!(ids.len() <= 2);
        for id in &ids {
            let s = &sim.scenarios.iter().find(|s| s.id == *id).unwrap().cf_type;
            assert_eq!(*s, CounterfactualType::HypotheticalAddition);
        }
    }

    #[test]
    fn test_generate_temporal_shift() {
        let mut sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let factual = make_factual();
        let ids = sim.generate_scenarios(&factual, CounterfactualType::TemporalShift, 2);
        assert!(ids.len() <= 2);
        for id in &ids {
            let s = &sim.scenarios.iter().find(|s| s.id == *id).unwrap().cf_type;
            assert_eq!(*s, CounterfactualType::TemporalShift);
        }
    }

    #[test]
    fn test_generate_alternative_path() {
        let mut sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let factual = make_factual();
        let ids = sim.generate_scenarios(&factual, CounterfactualType::AlternativePath, 2);
        assert!(ids.len() <= 2);
        for id in &ids {
            let s = &sim.scenarios.iter().find(|s| s.id == *id).unwrap().cf_type;
            assert_eq!(*s, CounterfactualType::AlternativePath);
        }
    }

    #[test]
    fn test_vsa_distance_identical() {
        let sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let v = vec![0u8; 16];
        assert_eq!(sim.vsa_distance(&v, &v), 0.0);
    }

    #[test]
    fn test_vsa_distance_maximal() {
        let sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let a = vec![0u8; 8];
        let b = vec![0xFFu8; 8];
        assert!((sim.vsa_distance(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vsa_distance_different_lengths() {
        let sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let a = vec![0u8; 4];
        let b = vec![0u8; 8];
        let dist = sim.vsa_distance(&a, &b);
        assert!(dist > 0.0);
        assert!(dist <= 1.0);
    }

    #[test]
    fn test_perturb_vsa_produces_different_state() {
        let sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let state = vec![0u8; 64];
        let perturbed = sim.perturb_vsa(&state, 1.0);
        assert_ne!(state, perturbed);
    }

    #[test]
    fn test_perturb_vsa_zero_strength() {
        let sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let state = vec![42u8; 32];
        let perturbed = sim.perturb_vsa(&state, 0.0);
        assert_eq!(state, perturbed);
    }

    #[test]
    fn test_structural_intervention() {
        let sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let state = vec![0u8; 10];
        let target_bits = vec![0, 5, 9];
        let values = vec![1, 2, 3];
        let result = sim.structural_intervention(&state, &target_bits, &values);
        assert_eq!(result[0], 1);
        assert_eq!(result[5], 2);
        assert_eq!(result[9], 3);
        assert_eq!(result[1], 0);
    }

    #[test]
    fn test_simulate_scenario() {
        let mut sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let factual = make_factual();
        let ids = sim.generate_scenarios(&factual, CounterfactualType::InputPerturbation, 1);
        if ids.is_empty() {
            return;
        }
        let outcome = sim.simulate_scenario(ids[0]);
        assert!(outcome.is_some());
        let outcome = outcome.unwrap();
        assert!(!outcome.reasoning_steps.is_empty());
        assert!(outcome.predicted_success >= 0.0 && outcome.predicted_success <= 1.0);
        assert!(outcome.uncertainty >= 0.0);
    }

    #[test]
    fn test_simulate_all() {
        let mut sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let factual = make_factual();
        sim.generate_scenarios(&factual, CounterfactualType::InputPerturbation, 3);
        let results = sim.simulate_all();
        assert!(!results.is_empty());
        for (id, outcome) in &results {
            assert!(*id > 0);
            assert!(!outcome.reasoning_steps.is_empty());
        }
    }

    #[test]
    fn test_compare_factual_vs_counterfactual() {
        let mut sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let factual = make_factual();
        let ids = sim.generate_scenarios(&factual, CounterfactualType::InputPerturbation, 1);
        if ids.is_empty() {
            return;
        }
        sim.simulate_scenario(ids[0]);
        let delta = sim.compare_factual_vs_counterfactual(0.8, ids[0]);
        assert!(delta.is_finite());
    }

    #[test]
    fn test_best_counterfactual() {
        let mut sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let factual = make_factual();
        let ids = sim.generate_scenarios(&factual, CounterfactualType::InputPerturbation, 3);
        sim.simulate_all();
        let best = sim.best_counterfactual();
        assert!(best.is_some() || sim.scenarios.is_empty());
    }

    #[test]
    fn test_diverse_counterfactuals() {
        let mut sim = CounterfactualSimulator::new(CounterfactualConfig {
            perturbation_strength: 0.5,
            ..Default::default()
        });
        let factual = make_factual();
        sim.generate_scenarios(&factual, CounterfactualType::InputPerturbation, 5);
        sim.simulate_all();
        let diverse = sim.diverse_counterfactuals(3);
        assert!(!diverse.is_empty() || sim.scenarios.is_empty());
        for s in &diverse {
            assert!(s.is_valid);
        }
    }

    #[test]
    fn test_filter_similar() {
        let mut sim = CounterfactualSimulator::new(CounterfactualConfig {
            perturbation_strength: 0.3,
            ..Default::default()
        });
        let factual = vec![0u8; 64];
        sim.generate_scenarios(&factual, CounterfactualType::InputPerturbation, 5);
        let to_remove = sim.filter_similar(0.3);
        assert!(to_remove.len() <= sim.scenarios.len());
    }

    #[test]
    fn test_divergence_matrix() {
        let mut sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let factual = make_factual();
        sim.generate_scenarios(&factual, CounterfactualType::InputPerturbation, 3);
        let matrix = sim.divergence_matrix();
        assert_eq!(matrix.len(), sim.scenarios.len());
        if !matrix.is_empty() {
            assert_eq!(matrix[0].len(), sim.scenarios.len());
            assert!((matrix[0][0] - 0.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_stats() {
        let mut sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let factual = make_factual();
        sim.generate_scenarios(&factual, CounterfactualType::InputPerturbation, 3);
        sim.simulate_all();
        let stats = sim.stats();
        assert_eq!(stats.total_scenarios, sim.scenarios.len());
        assert!(stats.avg_divergence >= 0.0);
        assert!(stats.best_outcome.is_finite());
        assert!(stats.worst_outcome.is_finite() || stats.simulated_count == 0);
        assert!(stats.simulation_success_rate >= 0.0);
    }

    #[test]
    fn test_counterfactual_config_default() {
        let config = CounterfactualConfig::default();
        assert_eq!(config.max_simulations, 5);
        assert!((config.perturbation_strength - 0.15).abs() < 1e-10);
        assert_eq!(config.simulation_depth, 3);
        assert!((config.min_divergence - 0.1).abs() < 1e-10);
        assert!((config.diversity_penalty - 0.2).abs() < 1e-10);
        assert!((config.outcome_confidence_threshold - 0.3).abs() < 1e-10);
    }

    #[test]
    fn test_perturbation_xor_tracking() {
        let mut sim = CounterfactualSimulator::new(CounterfactualConfig {
            perturbation_strength: 1.0,
            ..Default::default()
        });
        let factual = vec![0xAAu8; 16];
        let ids = sim.generate_scenarios(&factual, CounterfactualType::InputPerturbation, 1);
        if ids.is_empty() {
            return;
        }
        let s = sim.scenarios.iter().find(|s| s.id == ids[0]).unwrap();
        for i in 0..factual.len() {
            assert_eq!(s.perturbation[i], factual[i] ^ s.counterfactual_state[i]);
        }
    }

    #[test]
    fn test_type_distribution_in_stats() {
        let mut sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let factual = make_factual();
        sim.generate_scenarios(&factual, CounterfactualType::InputPerturbation, 1);
        sim.generate_scenarios(&factual, CounterfactualType::AlternativePath, 1);
        sim.generate_scenarios(&factual, CounterfactualType::TemporalShift, 1);
        let stats = sim.stats();
        assert_eq!(
            *stats
                .type_distribution
                .get(&CounterfactualType::InputPerturbation)
                .unwrap_or(&0),
            1
        );
        assert_eq!(
            *stats
                .type_distribution
                .get(&CounterfactualType::AlternativePath)
                .unwrap_or(&0),
            1
        );
        assert_eq!(
            *stats
                .type_distribution
                .get(&CounterfactualType::TemporalShift)
                .unwrap_or(&0),
            1
        );
    }

    #[test]
    fn test_confidence_and_validity_tracking() {
        let mut sim = CounterfactualSimulator::new(CounterfactualConfig::default());
        let factual = make_factual();
        let ids = sim.generate_scenarios(&factual, CounterfactualType::InputPerturbation, 2);
        if ids.is_empty() {
            return;
        }
        for id in &ids {
            let s = sim.scenarios.iter().find(|s| s.id == *id).unwrap();
            assert!((s.confidence - 0.0).abs() < 1e-10);
            assert!(!s.is_valid);
        }
        sim.simulate_scenario(ids[0]);
        let s = sim.scenarios.iter().find(|s| s.id == ids[0]).unwrap();
        assert!(s.is_valid);
        assert!(s.simulated_outcome.is_some());
    }
}
