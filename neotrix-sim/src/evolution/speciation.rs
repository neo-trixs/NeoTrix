// Speciation - Species differentiation based on trait distance
// Agents with similar traits form species; species compete for resources

use serde::{Serialize, Deserialize};
use super::fitness_landscape::AgentGenome;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Species {
    pub id: String,
    pub name: String,
    pub member_ids: Vec<String>,
    pub centroid: Vec<f32>,
    pub mean_fitness: f64,
    pub best_fitness: f64,
    pub age: u64,
    pub innovation_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciationConfig {
    pub compatibility_threshold: f32,  // Max distance to be same species
    pub niche_threshold: f32,          // Resource sharing threshold
    pub innovation_bonus: f32,         // Bonus for novel species
    pub aging_penalty: f32,            // Penalty for old species
}

impl Default for SpeciationConfig {
    fn default() -> Self {
        Self {
            compatibility_threshold: 0.5,
            niche_threshold: 0.3,
            innovation_bonus: 0.2,
            aging_penalty: 0.05,
        }
    }
}

pub struct Speciation {
    config: SpeciationConfig,
    species: Vec<Species>,
    next_species_id: usize,
}

impl Speciation {
    pub fn new(config: SpeciationConfig) -> Self {
        Self { config, species: Vec::new(), next_species_id: 0 }
    }

    /// Assign genomes to species
    pub fn speciate(&mut self, population: &[AgentGenome]) {
        // Reset membership
        for sp in &mut self.species {
            sp.member_ids.clear();
        }

        for genome in population {
            let mut placed = false;
            for sp in &mut self.species {
                let dist = Self::trait_distance(&genome.traits, &sp.centroid);
                if dist <= self.config.compatibility_threshold {
                    sp.member_ids.push(genome.agent_id.clone());
                    placed = true;
                    break;
                }
            }
            if !placed {
                self.next_species_id += 1;
                self.species.push(Species {
                    id: format!("species_{}", self.next_species_id),
                    name: Self::generate_species_name(),
                    member_ids: vec![genome.agent_id.clone()],
                    centroid: genome.traits.clone(),
                    mean_fitness: genome.fitness,
                    best_fitness: genome.fitness,
                    age: 0,
                    innovation_count: 1,
                });
            }
        }

        // Update species stats
        for sp in &mut self.species {
            sp.age += 1;
            if sp.member_ids.is_empty() {
                continue;
            }
            // Update centroid (average of members)
            if !sp.member_ids.is_empty() {
                let n = sp.member_ids.len() as f32;
                let member_genomes: Vec<&AgentGenome> = population.iter()
                    .filter(|g| sp.member_ids.contains(&g.agent_id))
                    .collect();
                if !member_genomes.is_empty() {
                    let dim = member_genomes[0].traits.len();
                    sp.centroid = (0..dim).map(|i| {
                        member_genomes.iter().map(|g| g.traits[i]).sum::<f32>() / member_genomes.len() as f32
                    }).collect();
                }
            }
            sp.mean_fitness = sp.member_ids.iter()
                .filter_map(|id| population.iter().find(|g| &g.agent_id == id))
                .map(|g| g.fitness)
                .sum::<f64>() / sp.member_ids.len() as f64;
            sp.best_fitness = sp.member_ids.iter()
                .filter_map(|id| population.iter().find(|g| &g.agent_id == id))
                .map(|g| g.fitness)
                .fold(0.0_f64, f64::max);
        }
    }

    /// Get niche-adjusted fitness for each species
    pub fn niche_fitness(&self) -> Vec<(&Species, f64)> {
        self.species.iter().map(|sp| {
            let niche_adj = if sp.member_ids.is_empty() {
                0.0
            } else {
                sp.mean_fitness / sp.member_ids.len() as f64
                    + (self.config.innovation_bonus as f64) * (1.0 / (sp.age as f64 + 1.0))
                    - (self.config.aging_penalty as f64) * sp.age as f64
            };
            (sp, niche_adj.max(0.0))
        }).collect()
    }

    /// Remove empty/extinct species
    pub fn prune(&mut self) {
        self.species.retain(|sp| !sp.member_ids.is_empty() && sp.age < 50);
    }

    fn trait_distance(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum::<f32>().sqrt()
    }

    fn generate_species_name() -> String {
        let names = ["Phoenix", "Drift", "Core", "Veil", "Spark", "Echo", "Shard", "Flux", "Rift", "Bloom"];
        let idx = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos() as usize) % names.len();
        names[idx].to_string()
    }

    pub fn species(&self) -> &[Species] { &self.species }
    pub fn species_count(&self) -> usize { self.species.len() }
}
