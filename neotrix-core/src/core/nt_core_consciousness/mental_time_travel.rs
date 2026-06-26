use std::collections::VecDeque;

use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};

#[derive(Debug, Clone)]
pub struct EpisodicScene {
    pub scene_vsa: Vec<u8>,
    pub context_vsa: Vec<u8>,
    pub emotional_valence: f64,
    pub timestamp: u64,
    pub label: String,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct FutureSimulation {
    pub trajectory: Vec<EpisodicScene>,
    pub outcome_vsa: Vec<u8>,
    pub expected_valence: f64,
    pub uncertainty: f64,
    pub plausibility: f64,
    pub horizon: u64,
}

#[derive(Debug, Clone)]
pub struct MentalTimeTravel {
    pub episodic_memory: VecDeque<EpisodicScene>,
    pub simulations: VecDeque<FutureSimulation>,
    pub max_memory: usize,
    pub max_simulations: usize,
    pub temporal_binding_key: Vec<u8>,
    pub context_binding_key: Vec<u8>,
    pub simulation_depth: usize,
    pub imagination_noise: f64,
    pub cycle_count: u64,
}

impl Default for MentalTimeTravel {
    fn default() -> Self {
        Self::new()
    }
}

impl MentalTimeTravel {
    pub fn new() -> Self {
        Self {
            episodic_memory: VecDeque::with_capacity(200),
            simulations: VecDeque::with_capacity(50),
            max_memory: 200,
            max_simulations: 50,
            temporal_binding_key: QuantizedVSA::seeded_random(42, VSA_DIM),
            context_binding_key: QuantizedVSA::seeded_random(1234, VSA_DIM),
            simulation_depth: 5,
            imagination_noise: 0.2,
            cycle_count: 0,
        }
    }

    pub fn encode_scene(
        &mut self,
        scene_vsa: Vec<u8>,
        context: &[u8],
        valence: f64,
        label: &str,
    ) -> EpisodicScene {
        let time_key = QuantizedVSA::bind(
            &self.temporal_binding_key,
            &QuantizedVSA::seeded_random(self.cycle_count, VSA_DIM),
        );
        let scene_bound = QuantizedVSA::bind(&scene_vsa, &time_key);
        let context_bound = QuantizedVSA::bind(context, &self.context_binding_key);

        let scene = EpisodicScene {
            scene_vsa: scene_bound,
            context_vsa: context_bound,
            emotional_valence: valence,
            timestamp: self.cycle_count,
            label: label.to_string(),
            confidence: 1.0 - self.imagination_noise * 0.1,
        };

        self.episodic_memory.push_back(scene.clone());
        if self.episodic_memory.len() > self.max_memory {
            self.episodic_memory.pop_front();
        }
        self.cycle_count += 1;

        scene
    }

    pub fn simulate_past(&self, cue_vsa: &[u8], n: usize) -> Vec<&EpisodicScene> {
        if self.episodic_memory.is_empty() || n == 0 {
            return Vec::new();
        }

        let mut scored: Vec<(f64, usize)> = self
            .episodic_memory
            .iter()
            .enumerate()
            .map(|(i, s)| (QuantizedVSA::similarity(cue_vsa, &s.scene_vsa), i))
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let count = n.min(scored.len());
        scored
            .iter()
            .take(count)
            .map(|&(_, idx)| &self.episodic_memory[idx])
            .collect()
    }

    pub fn simulate_future(
        &mut self,
        current_state: &[u8],
        action_prototypes: &[&[u8]],
        horizon: usize,
    ) -> FutureSimulation {
        let h = horizon.min(self.simulation_depth);
        let mut trajectory = Vec::with_capacity(h);
        let mut step_state = current_state.to_vec();

        for step in 0..h {
            let action = if action_prototypes.is_empty() {
                step_state.clone()
            } else {
                let action_idx = step % action_prototypes.len();
                QuantizedVSA::bind(&step_state, action_prototypes[action_idx])
            };

            let noisy_state = if self.imagination_noise > 0.0 {
                let noise_seed = self.cycle_count.wrapping_add(step as u64);
                let noise_vec = QuantizedVSA::seeded_random(noise_seed, VSA_DIM);
                let threshold = (self.imagination_noise * 255.0) as u8;
                let mut blended = action;
                for (b, n) in blended.iter_mut().zip(noise_vec.iter()) {
                    if *n > threshold {
                        *b ^= 1;
                    }
                }
                blended
            } else {
                action
            };

            let context = QuantizedVSA::bind(&noisy_state, &self.context_binding_key);

            let scene = EpisodicScene {
                scene_vsa: step_state.clone(),
                context_vsa: context,
                emotional_valence: 0.0,
                timestamp: self.cycle_count + step as u64,
                label: format!("future_step_{}", step),
                confidence: 1.0 - self.imagination_noise * (step as f64 + 1.0) / h as f64,
            };
            trajectory.push(scene);
            step_state = noisy_state;
        }

        let outcome_vsa = trajectory
            .last()
            .map(|s| s.scene_vsa.clone())
            .unwrap_or_default();

        let expected_valence = if !self.episodic_memory.is_empty() {
            let sims: Vec<f64> = self
                .episodic_memory
                .iter()
                .map(|s| QuantizedVSA::similarity(&outcome_vsa, &s.scene_vsa))
                .collect();
            let weighted: f64 = self
                .episodic_memory
                .iter()
                .zip(sims.iter())
                .map(|(s, &sim)| s.emotional_valence * sim)
                .sum();
            let total: f64 = sims.iter().sum();
            if total > 0.0 {
                weighted / total
            } else {
                0.0
            }
        } else {
            0.0
        };

        let uncertainty = if action_prototypes.is_empty() {
            0.5
        } else if self.episodic_memory.is_empty() {
            0.8
        } else {
            let total_sim: f64 = action_prototypes
                .iter()
                .flat_map(|a| {
                    self.episodic_memory
                        .iter()
                        .map(|s| QuantizedVSA::similarity(a, &s.scene_vsa))
                })
                .sum();
            let count = (action_prototypes.len() * self.episodic_memory.len()) as f64;
            let avg_sim = total_sim / count;
            1.0 - avg_sim
        };

        let sim = FutureSimulation {
            trajectory,
            outcome_vsa,
            expected_valence,
            uncertainty: uncertainty.clamp(0.0, 1.0),
            plausibility: (1.0 - uncertainty).clamp(0.0, 1.0),
            horizon: h as u64,
        };

        self.simulations.push_back(sim.clone());
        if self.simulations.len() > self.max_simulations {
            self.simulations.pop_front();
        }

        sim
    }

    pub fn reconstruct_narrative(&self) -> Vec<&EpisodicScene> {
        let mut scenes: Vec<&EpisodicScene> = self.episodic_memory.iter().collect();
        scenes.sort_by_key(|s| s.timestamp);
        scenes
    }

    pub fn recent_scenes(&self, n: usize) -> Vec<&EpisodicScene> {
        let count = n.min(self.episodic_memory.len());
        self.episodic_memory.iter().rev().take(count).collect()
    }

    pub fn scene_count(&self) -> usize {
        self.episodic_memory.len()
    }

    pub fn simulation_count(&self) -> usize {
        self.simulations.len()
    }

    pub fn merge_simulations(&self, sims: &[&FutureSimulation]) -> FutureSimulation {
        if sims.is_empty() {
            return FutureSimulation {
                trajectory: Vec::new(),
                outcome_vsa: vec![0; VSA_DIM],
                expected_valence: 0.0,
                uncertainty: 1.0,
                plausibility: 0.0,
                horizon: 0,
            };
        }

        let refs: Vec<&[u8]> = sims.iter().map(|s| s.outcome_vsa.as_slice()).collect();
        let outcome_vsa = QuantizedVSA::bundle(&refs);

        let avg_valence = sims.iter().map(|s| s.expected_valence).sum::<f64>() / sims.len() as f64;
        let max_uncertainty = sims.iter().map(|s| s.uncertainty).fold(0.0_f64, f64::max);
        let max_horizon = sims.iter().map(|s| s.horizon).max().unwrap_or(0);

        let best_traj = sims
            .iter()
            .max_by(|a, b| {
                a.plausibility
                    .partial_cmp(&b.plausibility)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|s| s.trajectory.clone())
            .unwrap_or_default();

        FutureSimulation {
            trajectory: best_traj,
            outcome_vsa,
            expected_valence: avg_valence,
            uncertainty: max_uncertainty.clamp(0.0, 1.0),
            plausibility: (1.0 - max_uncertainty).clamp(0.0, 1.0),
            horizon: max_horizon,
        }
    }

    pub fn set_imagination_noise(&mut self, noise: f64) {
        self.imagination_noise = noise.clamp(0.0, 1.0);
    }

    pub fn reset(&mut self) {
        self.episodic_memory.clear();
        self.simulations.clear();
        self.cycle_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vsa(seed: u64) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed, VSA_DIM)
    }

    #[test]
    fn test_new_mtt_defaults() {
        let mtt = MentalTimeTravel::new();
        assert_eq!(mtt.max_memory, 200);
        assert_eq!(mtt.max_simulations, 50);
        assert_eq!(mtt.simulation_depth, 5);
        assert!((mtt.imagination_noise - 0.2).abs() < 1e-6);
        assert_eq!(mtt.cycle_count, 0);
        assert_eq!(mtt.temporal_binding_key.len(), VSA_DIM);
        assert_eq!(mtt.context_binding_key.len(), VSA_DIM);
        assert!(mtt.episodic_memory.is_empty());
        assert!(mtt.simulations.is_empty());
    }

    #[test]
    fn test_encode_scene() {
        let mut mtt = MentalTimeTravel::new();
        let scene_vsa = make_vsa(100);
        let context = make_vsa(200);
        let scene = mtt.encode_scene(scene_vsa.clone(), &context, 0.75, "test_scene");

        assert_eq!(scene.label, "test_scene");
        assert!((scene.emotional_valence - 0.75).abs() < 1e-6);
        assert!(scene.confidence > 0.0);
        assert_eq!(scene.timestamp, 0);
        assert_eq!(scene.scene_vsa.len(), VSA_DIM);
        assert_eq!(scene.context_vsa.len(), VSA_DIM);
        assert_ne!(scene.scene_vsa, scene_vsa);
        assert_eq!(mtt.scene_count(), 1);
        assert_eq!(mtt.cycle_count, 1);
    }

    #[test]
    fn test_simulate_past_by_similarity() {
        let mut mtt = MentalTimeTravel::new();
        let a = make_vsa(10);
        let b = make_vsa(20);
        let ctx = make_vsa(99);

        mtt.encode_scene(a.clone(), &ctx, 0.5, "scene_a");
        mtt.encode_scene(b.clone(), &ctx, 0.5, "scene_b");

        let results = mtt.simulate_past(&a, 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].label, "scene_a");

        let results_b = mtt.simulate_past(&b, 1);
        assert_eq!(results_b.len(), 1);
        assert_eq!(results_b[0].label, "scene_b");
    }

    #[test]
    fn test_simulate_past_returns_n_results() {
        let mut mtt = MentalTimeTravel::new();
        let ctx = make_vsa(99);
        for i in 0..5 {
            mtt.encode_scene(make_vsa(i as u64), &ctx, 0.5, &format!("s{}", i));
        }

        let results = mtt.simulate_past(&make_vsa(0), 3);
        assert_eq!(results.len(), 3);

        let results_all = mtt.simulate_past(&make_vsa(0), 100);
        assert_eq!(results_all.len(), 5);
    }

    #[test]
    fn test_simulate_past_empty_memory() {
        let mtt = MentalTimeTravel::new();
        let cue = make_vsa(42);
        let results = mtt.simulate_past(&cue, 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_simulate_future_creates_trajectory() {
        let mut mtt = MentalTimeTravel::new();
        let state = make_vsa(1);
        let action = make_vsa(2);
        let actions = [action.as_slice()];
        let sim = mtt.simulate_future(&state, &actions, 3);

        assert_eq!(sim.trajectory.len(), 3);
        assert_eq!(sim.horizon, 3);
    }

    #[test]
    fn test_simulate_future_computes_expected_valence() {
        let mut mtt = MentalTimeTravel::new();
        let ctx = make_vsa(99);
        let happy = make_vsa(10);
        let sad = make_vsa(20);
        mtt.encode_scene(happy.clone(), &ctx, 0.9, "happy");
        mtt.encode_scene(sad.clone(), &ctx, -0.5, "sad");

        let state = make_vsa(1);
        let action = make_vsa(2);
        let actions = [action.as_slice()];
        let sim = mtt.simulate_future(&state, &actions, 1);

        assert!(sim.expected_valence != 0.0);
    }

    #[test]
    fn test_simulate_future_uncertainty() {
        let mut mtt = MentalTimeTravel::new();
        let state = make_vsa(1);
        let action = make_vsa(2);
        let actions = [action.as_slice()];
        let sim = mtt.simulate_future(&state, &actions, 1);

        assert!(sim.uncertainty >= 0.0);
        assert!(sim.uncertainty <= 1.0);
    }

    #[test]
    fn test_simulate_future_with_no_actions() {
        let mut mtt = MentalTimeTravel::new();
        let state = make_vsa(1);
        let sim = mtt.simulate_future(&state, &[], 2);

        assert_eq!(sim.trajectory.len(), 2);
        assert!((sim.uncertainty - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reconstruct_narrative_ordered_by_time() {
        let mut mtt = MentalTimeTravel::new();
        let ctx = make_vsa(99);
        mtt.encode_scene(make_vsa(1), &ctx, 0.1, "first");
        mtt.encode_scene(make_vsa(2), &ctx, 0.2, "second");
        mtt.encode_scene(make_vsa(3), &ctx, 0.3, "third");

        let narrative = mtt.reconstruct_narrative();
        assert_eq!(narrative.len(), 3);
        assert_eq!(narrative[0].label, "first");
        assert_eq!(narrative[1].label, "second");
        assert_eq!(narrative[2].label, "third");
    }

    #[test]
    fn test_recent_scenes() {
        let mut mtt = MentalTimeTravel::new();
        let ctx = make_vsa(99);
        mtt.encode_scene(make_vsa(1), &ctx, 0.1, "a");
        mtt.encode_scene(make_vsa(2), &ctx, 0.2, "b");
        mtt.encode_scene(make_vsa(3), &ctx, 0.3, "c");

        let recent = mtt.recent_scenes(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].label, "c");
        assert_eq!(recent[1].label, "b");
    }

    #[test]
    fn test_scene_count() {
        let mut mtt = MentalTimeTravel::new();
        assert_eq!(mtt.scene_count(), 0);

        let ctx = make_vsa(99);
        mtt.encode_scene(make_vsa(1), &ctx, 0.0, "a");
        assert_eq!(mtt.scene_count(), 1);

        mtt.encode_scene(make_vsa(2), &ctx, 0.0, "b");
        assert_eq!(mtt.scene_count(), 2);
    }

    #[test]
    fn test_simulation_count() {
        let mut mtt = MentalTimeTravel::new();
        assert_eq!(mtt.simulation_count(), 0);

        let state = make_vsa(1);
        let action = make_vsa(2);
        let actions = [action.as_slice()];
        mtt.simulate_future(&state, &actions, 1);
        assert_eq!(mtt.simulation_count(), 1);
    }

    #[test]
    fn test_merge_simulations() {
        let mut mtt = MentalTimeTravel::new();
        let state = make_vsa(1);
        let a1 = make_vsa(10);
        let a2 = make_vsa(20);
        let sim1 = mtt.simulate_future(&state, &[a1.as_slice()], 2);
        let sim2 = mtt.simulate_future(&state, &[a2.as_slice()], 3);

        let merged = mtt.merge_simulations(&[&sim1, &sim2]);
        assert_eq!(merged.horizon, 3);
        assert!(merged.uncertainty >= 0.0);
        assert!(merged.plausibility >= 0.0);
        assert_eq!(merged.outcome_vsa.len(), VSA_DIM);
    }

    #[test]
    fn test_set_imagination_noise() {
        let mut mtt = MentalTimeTravel::new();
        mtt.set_imagination_noise(0.8);
        assert!((mtt.imagination_noise - 0.8).abs() < 1e-6);

        mtt.set_imagination_noise(1.5);
        assert!((mtt.imagination_noise - 1.0).abs() < 1e-6);

        mtt.set_imagination_noise(-0.5);
        assert!((mtt.imagination_noise).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut mtt = MentalTimeTravel::new();
        let ctx = make_vsa(99);
        mtt.encode_scene(make_vsa(1), &ctx, 0.5, "a");
        let state = make_vsa(1);
        let action = make_vsa(2);
        mtt.simulate_future(&state, &[action.as_slice()], 1);

        assert!(mtt.scene_count() > 0);
        assert!(mtt.simulation_count() > 0);
        assert!(mtt.cycle_count > 0);

        mtt.reset();
        assert_eq!(mtt.scene_count(), 0);
        assert_eq!(mtt.simulation_count(), 0);
        assert_eq!(mtt.cycle_count, 0);
    }

    #[test]
    fn test_temporal_binding_produces_different_scenes() {
        let mut mtt = MentalTimeTravel::new();
        let ctx = make_vsa(99);
        let content = make_vsa(42);

        let s1 = mtt.encode_scene(content.clone(), &ctx, 0.5, "same");
        let s2 = mtt.encode_scene(content.clone(), &ctx, 0.5, "same");

        assert_ne!(s1.scene_vsa, s2.scene_vsa);
        assert_ne!(s1.timestamp, s2.timestamp);
    }

    #[test]
    fn test_max_memory_enforced() {
        let mut mtt = MentalTimeTravel::new();
        mtt.max_memory = 3;
        let ctx = make_vsa(99);

        for i in 0..5 {
            mtt.encode_scene(make_vsa(i as u64), &ctx, i as f64 * 0.1, &format!("s{}", i));
        }

        assert_eq!(mtt.scene_count(), 3);
        assert_eq!(mtt.episodic_memory.front().unwrap().label, "s2");
        assert_eq!(mtt.episodic_memory.back().unwrap().label, "s4");
    }
}
