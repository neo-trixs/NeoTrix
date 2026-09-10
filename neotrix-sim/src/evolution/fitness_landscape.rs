// FitnessLandscape - Adaptive fitness terrain for consciousness evolution
// Maps agent traits → fitness score using rugged landscape model
// Integrates with E8 hexagram reasoning for multi-dimensional fitness

use serde::{Serialize, Deserialize};
use crate::foundation::math_bridge::SimulationRng;

/// Agent trait vector for fitness evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGenome {
    pub agent_id: String,
    pub traits: Vec<f32>,          // N-dimensional trait vector
    pub fitness: f64,
    pub generation: u64,
    pub parent_id: Option<String>,
}

impl AgentGenome {
    pub fn random(id: &str, dim: usize, rng: &mut SimulationRng) -> Self {
        let traits = (0..dim).map(|_| rng.range_f32(-1.0, 1.0)).collect();
        Self {
            agent_id: id.to_string(),
            traits,
            fitness: 0.0,
            generation: 0,
            parent_id: None,
        }
    }
}

/// Fitness landscape configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandscapeConfig {
    pub dimensions: usize,        // Trait dimensions
    pub ruggedness: f32,          // 0=smooth, 1=very rugged (epistasis)
    pub neutrality: f32,          // 0=single peak, 1=neutral networks
    pub multimodal: usize,        // Number of fitness peaks
    pub epistasis_strength: f32,  // Interaction between traits
}

impl Default for LandscapeConfig {
    fn default() -> Self {
        Self {
            dimensions: 8,
            ruggedness: 0.4,
            neutrality: 0.2,
            multimodal: 3,
            epistasis_strength: 0.3,
        }
    }
}

/// Fitness landscape - maps genome → fitness
pub struct FitnessLandscape {
    config: LandscapeConfig,
    peaks: Vec<Vec<f32>>,         // Local optima positions
    epistasis_matrix: Vec<Vec<f32>>,  // Trait interaction weights
}

impl FitnessLandscape {
    pub fn new(config: LandscapeConfig, seed: u64) -> Self {
        let mut rng = SimulationRng::new(seed);

        // Generate random fitness peaks
        let peaks: Vec<Vec<f32>> = (0..config.multimodal)
            .map(|_| (0..config.dimensions).map(|_| rng.range_f32(-1.0, 1.0)).collect())
            .collect();

        // Generate epistasis matrix (symmetric, zero diagonal)
        let mut epistasis = vec![vec![0.0f32; config.dimensions]; config.dimensions];
        for i in 0..config.dimensions {
            for j in (i + 1)..config.dimensions {
                let w = rng.range_f32(-config.epistasis_strength, config.epistasis_strength);
                epistasis[i][j] = w;
                epistasis[j][i] = w;
            }
        }

        Self { config, peaks, epistasis_matrix: epistasis }
    }

    /// Evaluate fitness of a genome
    pub fn evaluate(&self, genome: &mut AgentGenome) {
        let fitness = self.compute_fitness(&genome.traits);
        genome.fitness = fitness;
    }

    fn compute_fitness(&self, traits: &[f32]) -> f64 {
        // Base fitness: distance to nearest peak (inverted)
        let min_peak_dist = self.peaks.iter().map(|peak| {
            traits.iter().zip(peak.iter())
                .map(|(t, p)| (t - p).powi(2))
                .sum::<f32>()
                .sqrt()
        }).fold(f32::INFINITY, f32::min);

        let base_fitness = 1.0 / (1.0 + min_peak_dist);

        // Epistasis: trait interactions
        let mut epistasis_bonus = 0.0;
        for i in 0..traits.len().min(self.config.dimensions) {
            for j in (i + 1)..traits.len().min(self.config.dimensions) {
                epistasis_bonus += self.epistasis_matrix[i][j] * traits[i] * traits[j];
            }
        }

        // Ruggedness: add controlled noise
        let rugged_hash = traits.iter().enumerate()
            .map(|(i, t)| (t * (i as f32 + 1.0) * 100.0).sin())
            .sum::<f32>() / traits.len() as f32;
        let ruggedness_noise = rugged_hash * self.config.ruggedness * 0.1;

        // Combine
        let fitness = (base_fitness as f64)
            + (epistasis_bonus as f64 * 0.3)
            + (ruggedness_noise as f64);

        fitness.clamp(0.0, 1.0)
    }

    /// Compute selection pressure based on fitness distribution
    pub fn selection_pressure(&self, population: &[AgentGenome]) -> f64 {
        if population.len() < 2 { return 1.0; }
        let fitnesses: Vec<f64> = population.iter().map(|g| g.fitness).collect();
        let mean = fitnesses.iter().sum::<f64>() / fitnesses.len() as f64;
        let variance = fitnesses.iter().map(|f| (f - mean).powi(2)).sum::<f64>() / fitnesses.len() as f64;
        // High variance = high pressure (strong selection)
        1.0 + variance.sqrt() * 5.0
    }

    /// Find nearest peak to a genome
    pub fn nearest_peak(&self, traits: &[f32]) -> (usize, f32) {
        self.peaks.iter().enumerate().map(|(i, peak)| {
            let dist = traits.iter().zip(peak.iter())
                .map(|(t, p)| (t - p).powi(2))
                .sum::<f32>()
                .sqrt();
            (i, dist)
        }).min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap_or((0, f32::MAX))
    }

    pub fn config(&self) -> &LandscapeConfig { &self.config }
    pub fn peaks(&self) -> &[Vec<f32>] { &self.peaks }
}
