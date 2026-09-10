// MutationOps - Genetic operators for consciousness evolution
// Mutation, crossover, and trait injection

use serde::{Serialize, Deserialize};
use crate::foundation::math_bridge::SimulationRng;
use super::fitness_landscape::AgentGenome;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationConfig {
    pub mutation_rate: f32,        // Probability per trait
    pub mutation_strength: f32,   // Max magnitude of change
    pub crossover_rate: f32,      // Probability of crossover
    pub trait_injection_rate: f32, // Rate of new random traits
    pub elitist_traits: usize,    // Top N traits preserved
}

impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            mutation_rate: 0.1,
            mutation_strength: 0.2,
            crossover_rate: 0.7,
            trait_injection_rate: 0.05,
            elitist_traits: 2,
        }
    }
}

pub struct MutationOps {
    config: MutationConfig,
}

impl MutationOps {
    pub fn new(config: MutationConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &MutationConfig { &self.config }

    pub fn set_mutation_rate(&mut self, rate: f32) {
        self.config.mutation_rate = rate;
    }

    /// Apply mutation to a genome
    pub fn mutate(&self, genome: &mut AgentGenome, rng: &mut SimulationRng) {
        for i in 0..genome.traits.len() {
            // Skip elite traits
            if i < self.config.elitist_traits {
                continue;
            }

            if rng.next_f32() < self.config.mutation_rate {
                let delta = rng.range_f32(-self.config.mutation_strength, self.config.mutation_strength);
                genome.traits[i] = (genome.traits[i] + delta).clamp(-1.0, 1.0);
            }

            // Random trait injection
            if rng.next_f32() < self.config.trait_injection_rate {
                genome.traits[i] = rng.range_f32(-1.0, 1.0);
            }
        }
    }

    /// Crossover two parents to produce offspring
    pub fn crossover(&self, parent_a: &AgentGenome, parent_b: &AgentGenome, child_id: &str, rng: &mut SimulationRng) -> AgentGenome {
        let mut child_traits = Vec::with_capacity(parent_a.traits.len());

        for i in 0..parent_a.traits.len() {
            if rng.next_f32() < 0.5 {
                child_traits.push(parent_a.traits[i]);
            } else {
                child_traits.push(parent_b.traits[i]);
            }
        }

        AgentGenome {
            agent_id: child_id.to_string(),
            traits: child_traits,
            fitness: 0.0,
            generation: parent_a.generation.max(parent_b.generation) + 1,
            parent_id: Some(parent_a.agent_id.clone()),
        }
    }

    /// Produce next generation from selected population
    pub fn breed(&self, population: &[AgentGenome], target_size: usize, rng: &mut SimulationRng) -> Vec<AgentGenome> {
        let mut offspring = Vec::new();

        for i in 0..target_size {
            let parent_a = &population[rng.range_usize(0, population.len())];
            let parent_b = &population[rng.range_usize(0, population.len())];

            let child_id = format!("agent_{}_{}", parent_a.agent_id, i);

            let mut child = if rng.next_f32() < self.config.crossover_rate {
                self.crossover(parent_a, parent_b, &child_id, rng)
            } else {
                let parent = if rng.next_f32() < 0.5 { parent_a } else { parent_b };
                AgentGenome {
                    agent_id: child_id,
                    traits: parent.traits.clone(),
                    fitness: 0.0,
                    generation: parent.generation + 1,
                    parent_id: Some(parent.agent_id.clone()),
                }
            };

            self.mutate(&mut child, rng);
            offspring.push(child);
        }

        offspring
    }
}
