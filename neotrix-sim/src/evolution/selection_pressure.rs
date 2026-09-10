// SelectionPressure - Dynamic selection engine for consciousness evolution
// Adjusts pressure based on population health, resource scarcity, and time

use serde::{Serialize, Deserialize};
use super::fitness_landscape::AgentGenome;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SelectionMode {
    Natural,     // Fitness-proportional
    Tournament,  // k-tournament selection
    Rank,        // Rank-based selection
    Elitist,     // Top-N survival
}

impl Default for SelectionMode {
    fn default() -> Self { Self::Natural }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionConfig {
    pub mode: SelectionMode,
    pub tournament_size: usize,
    pub elitist_top_n: usize,
    pub mutation_rate: f32,
    pub crossover_rate: f32,
    pub pressure_scale: f32,    // 1.0 = normal
    pub min_population: usize,
    pub max_population: usize,
}

impl Default for SelectionConfig {
    fn default() -> Self {
        Self {
            mode: SelectionMode::Natural,
            tournament_size: 3,
            elitist_top_n: 2,
            mutation_rate: 0.1,
            crossover_rate: 0.7,
            pressure_scale: 1.0,
            min_population: 5,
            max_population: 50,
        }
    }
}

/// Selection result for one generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionResult {
    pub survivors: Vec<AgentGenome>,
    pub eliminated: Vec<String>,
    pub offspring: Vec<AgentGenome>,
    pub generation: u64,
    pub mean_fitness: f64,
    pub max_fitness: f64,
    pub pressure_applied: f64,
}

pub struct SelectionPressure {
    config: SelectionConfig,
    generation: u64,
    history: Vec<f64>,
}

impl SelectionPressure {
    pub fn new(config: SelectionConfig) -> Self {
        Self { config, generation: 0, history: Vec::new() }
    }

    /// Apply selection to a population, return survivors + offspring
    pub fn select(&mut self, mut population: Vec<AgentGenome>, resource_modifier: f32) -> SelectionResult {
        self.generation += 1;

        if population.is_empty() {
            return SelectionResult {
                survivors: vec![], eliminated: vec![], offspring: vec![],
                generation: self.generation, mean_fitness: 0.0, max_fitness: 0.0,
                pressure_applied: 0.0,
            };
        }

        // Sort by fitness (descending)
        population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal));

        let mean_fitness: f64 = population.iter().map(|g| g.fitness).sum::<f64>() / population.len() as f64;
        let max_fitness = population[0].fitness;

        // Dynamic pressure: more pressure when population is large or fitness is high
        let pop_pressure = (population.len() as f64 / self.config.max_population as f64).clamp(0.5, 2.0);
        let pressure = (self.config.pressure_scale as f64) * pop_pressure * (1.0 / (resource_modifier.max(0.3) as f64));

        self.history.push(pressure);
        if self.history.len() > 100 { self.history.remove(0); }

        // Elitist: always keep top N
        let elite_count = self.config.elitist_top_n.min(population.len());
        let survivors: Vec<AgentGenome> = population.iter().take(elite_count).cloned().collect();
        let rest: Vec<AgentGenome> = population.into_iter().skip(elite_count).collect();
        let pop_len = survivors.len() + rest.len();

        // Apply selection mode to the rest
        let mut selected = Vec::new();
        let mut eliminated = Vec::new();

        match self.config.mode {
            SelectionMode::Natural => {
                let total: f64 = rest.iter().map(|g| g.fitness.max(0.001)).sum();
                for genome in rest {
                    let prob = (genome.fitness.max(0.001) / total) * pressure;
                    if prob > 0.3 {
                        selected.push(genome);
                    } else {
                        eliminated.push(genome.agent_id.clone());
                    }
                }
            }
            SelectionMode::Tournament => {
                let target = (rest.len() as f64 / pressure).ceil() as usize;
                for chunk in rest.chunks(self.config.tournament_size) {
                    if let Some(best) = chunk.iter().max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap()) {
                        if selected.len() < target {
                            selected.push(best.clone());
                        }
                    }
                }
                let selected_ids: std::collections::HashSet<_> = selected.iter().map(|g| g.agent_id.clone()).collect();
                for genome in rest {
                    if !selected_ids.contains(&genome.agent_id) {
                        eliminated.push(genome.agent_id);
                    }
                }
            }
            SelectionMode::Rank => {
                let target = (rest.len() as f64 / pressure).ceil() as usize;
                for (i, genome) in rest.into_iter().enumerate() {
                    let rank_prob = 1.0 - (i as f64 / pop_len as f64);
                    if rank_prob > 0.3 || selected.len() < target {
                        selected.push(genome);
                    } else {
                        eliminated.push(genome.agent_id);
                    }
                }
            }
            SelectionMode::Elitist => {
                let target = (rest.len() as f64 / pressure).ceil() as usize;
                for (i, genome) in rest.into_iter().enumerate() {
                    if i < target {
                        selected.push(genome);
                    } else {
                        eliminated.push(genome.agent_id);
                    }
                }
            }
        }

        // Ensure minimum population via reproduction
        let mut offspring = Vec::new();
        let total = survivors.len() + selected.len();
        if total < self.config.min_population && !survivors.is_empty() {
            let needed = self.config.min_population - total;
            for i in 0..needed {
                let parent = &survivors[i % survivors.len()];
                let child = AgentGenome {
                    agent_id: format!("{}_gen{}", parent.agent_id, self.generation),
                    traits: parent.traits.clone(),
                    fitness: 0.0,
                    generation: self.generation,
                    parent_id: Some(parent.agent_id.clone()),
                };
                offspring.push(child);
            }
        }

        let mut all_survivors = survivors;
        all_survivors.extend(selected);

        SelectionResult {
            survivors: all_survivors,
            eliminated,
            offspring,
            generation: self.generation,
            mean_fitness,
            max_fitness,
            pressure_applied: pressure,
        }
    }

    pub fn generation(&self) -> u64 { self.generation }
    pub fn current_pressure(&self) -> f64 {
        self.history.last().copied().unwrap_or(1.0)
    }
}
