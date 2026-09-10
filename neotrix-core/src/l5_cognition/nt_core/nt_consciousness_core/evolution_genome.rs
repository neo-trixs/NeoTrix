#![forbid(unsafe_code)]

//! Evolution Genome (进化基因组)
//!
//! Records the full history of mutations as a genome, supporting:
//!   - Complete mutation history with fitness tracking
//!   - Rollback to any previous state
//!   - Fitness score tracking across generations
//!   - Genealogy: parent-child mutation lineage
//!
//! Inspired by genetic algorithms and biological evolution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Evolution Genome — complete mutation history with genealogy
pub struct EvolutionGenome {
    /// All mutation records (generation → mutations)
    pub generations: Vec<Generation>,
    /// Mutation index for fast lookup
    pub mutation_index: HashMap<String, MutationRecord>,
    /// Current generation number
    pub current_generation: u32,
    /// Genome configuration
    pub config: GenomeConfig,
}

/// Genome configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeConfig {
    /// Maximum generations to retain
    pub max_generations: usize,
    /// Maximum mutations per generation
    pub max_mutations_per_generation: usize,
    /// Enable genealogy tracking
    pub track_genealogy: bool,
    /// Fitness decay factor per generation
    pub fitness_decay: f64,
}

impl Default for GenomeConfig {
    fn default() -> Self {
        Self {
            max_generations: 500,
            max_mutations_per_generation: 100,
            track_genealogy: true,
            fitness_decay: 0.99,
        }
    }
}

/// A generation of mutations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Generation {
    /// Generation number
    pub number: u32,
    /// Mutations in this generation
    pub mutations: Vec<MutationRecord>,
    /// Aggregate fitness of the generation
    pub aggregate_fitness: f64,
    /// Generation timestamp
    pub timestamp: String,
}

/// A single mutation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationRecord {
    /// Unique mutation ID
    pub id: String,
    /// Parent mutation ID (None for root mutations)
    pub parent_id: Option<String>,
    /// Child mutation IDs
    pub children: Vec<String>,
    /// Generation number
    pub generation: u32,
    /// Mutation description
    pub description: String,
    /// Target module
    pub target: String,
    /// Mutation type
    pub mutation_type: MutationType,
    /// Fitness score after mutation
    pub fitness: f64,
    /// Delta fitness from parent
    pub fitness_delta: f64,
    /// Whether mutation was successful
    pub success: bool,
    /// Rollback state (serialized state snapshot)
    pub rollback_state: Option<StateSnapshot>,
    /// Timestamp
    pub timestamp: String,
}

/// Types of mutations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MutationType {
    /// Parameter tuning
    ParameterTune,
    /// Adding new capability
    CapabilityAdd,
    /// Removing capability
    CapabilityRemove,
    /// Structural change
    StructuralChange,
    /// Configuration change
    ConfigChange,
    /// Bug fix
    BugFix,
    /// Performance optimization
    PerfOptimization,
}

/// State snapshot for rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Snapshot ID
    pub id: String,
    /// Serialized state
    pub state: HashMap<String, String>,
    /// Checksum for integrity
    pub checksum: u64,
    /// Timestamp
    pub timestamp: String,
}

/// Error type for genome operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenomeError {
    /// Mutation not found
    MutationNotFound(String),
    /// Generation not found
    GenerationNotFound(u32),
    /// Rollback target not found
    RollbackTargetNotFound(String),
    /// No parent to rollback to
    NoParentAvailable(String),
    /// Integrity check failed
    IntegrityCheckFailed(String),
}

impl std::fmt::Display for GenomeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MutationNotFound(id) => write!(f, "Mutation not found: {}", id),
            Self::GenerationNotFound(gen) => write!(f, "Generation not found: {}", gen),
            Self::RollbackTargetNotFound(id) => write!(f, "Rollback target not found: {}", id),
            Self::NoParentAvailable(id) => write!(f, "No parent available for mutation: {}", id),
            Self::IntegrityCheckFailed(msg) => write!(f, "Integrity check failed: {}", msg),
        }
    }
}

impl std::error::Error for GenomeError {}

impl EvolutionGenome {
    /// Create a new Evolution Genome
    pub fn new(config: GenomeConfig) -> Self {
        Self {
            generations: Vec::new(),
            mutation_index: HashMap::new(),
            current_generation: 0,
            config,
        }
    }

    /// Record a new mutation
    pub fn record_mutation(
        &mut self,
        description: String,
        target: String,
        mutation_type: MutationType,
        parent_id: Option<String>,
        fitness: f64,
        rollback_state: Option<StateSnapshot>,
    ) -> Result<MutationRecord, GenomeError> {
        let fitness_delta = if let Some(ref pid) = parent_id {
            if let Some(parent) = self.mutation_index.get(pid) {
                fitness - parent.fitness
            } else {
                return Err(GenomeError::MutationNotFound(pid.clone()));
            }
        } else {
            fitness
        };

        let record = MutationRecord {
            id: format!("mut_{}", uuid::Uuid::new_v4()),
            parent_id: parent_id.clone(),
            children: Vec::new(),
            generation: self.current_generation,
            description,
            target,
            mutation_type,
            fitness,
            fitness_delta,
            success: true,
            rollback_state,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Update parent's children list
        if let Some(ref pid) = parent_id {
            if let Some(parent) = self.mutation_index.get_mut(pid) {
                parent.children.push(record.id.clone());
            }
        }

        // Add to current generation
        if let Some(gen) = self
            .generations
            .iter_mut()
            .find(|g| g.number == self.current_generation)
        {
            gen.mutations.push(record.clone());
            let total_fitness: f64 = gen.mutations.iter().map(|m| m.fitness).sum();
            let count = gen.mutations.len() as f64;
            if count > 0.0 {
                gen.aggregate_fitness = total_fitness / count;
            }
        } else {
            let gen = Generation {
                number: self.current_generation,
                mutations: vec![record.clone()],
                aggregate_fitness: fitness,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            self.generations.push(gen);
        }

        self.mutation_index
            .insert(record.id.clone(), record.clone());
        self.trim_generations();

        Ok(record)
    }

    /// Update generation aggregate fitness
    #[allow(dead_code)]
    fn update_generation_fitness(&self, gen: &mut Generation) {
        if !gen.mutations.is_empty() {
            gen.aggregate_fitness =
                gen.mutations.iter().map(|m| m.fitness).sum::<f64>() / gen.mutations.len() as f64;
        }
    }

    /// Advance to next generation
    pub fn next_generation(&mut self) {
        self.current_generation += 1;
    }

    /// Get mutation by ID
    pub fn get_mutation(&self, id: &str) -> Option<&MutationRecord> {
        self.mutation_index.get(id)
    }

    /// Get generation by number
    pub fn get_generation(&self, number: u32) -> Option<&Generation> {
        self.generations.iter().find(|g| g.number == number)
    }

    /// Rollback to a specific mutation
    pub fn rollback(&self, mutation_id: &str) -> Result<&StateSnapshot, GenomeError> {
        let mutation = self
            .mutation_index
            .get(mutation_id)
            .ok_or_else(|| GenomeError::MutationNotFound(mutation_id.to_string()))?;

        mutation
            .rollback_state
            .as_ref()
            .ok_or_else(|| GenomeError::NoParentAvailable(mutation_id.to_string()))
    }

    /// Get lineage (ancestor chain) for a mutation
    pub fn lineage(&self, mutation_id: &str) -> Vec<&MutationRecord> {
        let mut chain = Vec::new();
        let mut current_id = Some(mutation_id.to_string());

        while let Some(id) = current_id {
            if let Some(mutation) = self.mutation_index.get(&id) {
                chain.push(mutation);
                current_id = mutation.parent_id.clone();
            } else {
                break;
            }
        }

        chain.reverse();
        chain
    }

    /// Get children of a mutation
    pub fn children(&self, mutation_id: &str) -> Vec<&MutationRecord> {
        if let Some(mutation) = self.mutation_index.get(mutation_id) {
            mutation
                .children
                .iter()
                .filter_map(|id| self.mutation_index.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get fitness history across generations
    pub fn fitness_history(&self) -> Vec<(u32, f64)> {
        self.generations
            .iter()
            .map(|g| (g.number, g.aggregate_fitness))
            .collect()
    }

    /// Get best mutation by fitness
    pub fn best_mutation(&self) -> Option<&MutationRecord> {
        self.mutation_index.values().max_by(|a, b| {
            a.fitness
                .partial_cmp(&b.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Get mutations in a generation range
    pub fn mutations_in_range(&self, from_gen: u32, to_gen: u32) -> Vec<&MutationRecord> {
        self.mutation_index
            .values()
            .filter(|m| m.generation >= from_gen && m.generation <= to_gen)
            .collect()
    }

    /// Calculate mutation statistics
    pub fn stats(&self) -> GenomeStats {
        let total_mutations = self.mutation_index.len();
        let successful = self.mutation_index.values().filter(|m| m.success).count();

        let avg_fitness = if total_mutations > 0 {
            self.mutation_index.values().map(|m| m.fitness).sum::<f64>() / total_mutations as f64
        } else {
            0.0
        };

        let max_fitness = self
            .mutation_index
            .values()
            .map(|m| m.fitness)
            .fold(0.0_f64, f64::max);

        let avg_children = if total_mutations > 0 {
            self.mutation_index
                .values()
                .map(|m| m.children.len())
                .sum::<usize>() as f64
                / total_mutations as f64
        } else {
            0.0
        };

        GenomeStats {
            total_generations: self.generations.len() as u32,
            total_mutations,
            successful_mutations: successful,
            success_rate: if total_mutations > 0 {
                successful as f64 / total_mutations as f64
            } else {
                0.0
            },
            avg_fitness,
            max_fitness,
            avg_children_per_mutation: avg_children,
        }
    }

    fn trim_generations(&mut self) {
        while self.generations.len() > self.config.max_generations {
            if let Some(old_gen) = self.generations.first() {
                let old_gen_num = old_gen.number;
                // Remove old mutations from index
                for mutation in &old_gen.mutations {
                    self.mutation_index.remove(&mutation.id);
                }
                self.generations.remove(0);
                // Rebuild index from remaining generations
                self.rebuild_index();
                let _ = old_gen_num; // Suppress unused warning
            } else {
                break;
            }
        }
    }

    fn rebuild_index(&mut self) {
        self.mutation_index.clear();
        for gen in &self.generations {
            for mutation in &gen.mutations {
                self.mutation_index
                    .insert(mutation.id.clone(), mutation.clone());
            }
        }
    }
}

/// Genome statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeStats {
    pub total_generations: u32,
    pub total_mutations: usize,
    pub successful_mutations: usize,
    pub success_rate: f64,
    pub avg_fitness: f64,
    pub max_fitness: f64,
    pub avg_children_per_mutation: f64,
}

impl std::fmt::Display for GenomeStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "        Evolution Genome Stats")?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "Generations:      {}", self.total_generations)?;
        writeln!(f, "Total mutations:  {}", self.total_mutations)?;
        writeln!(f, "Successful:       {}", self.successful_mutations)?;
        writeln!(f, "Success rate:     {:.2}%", self.success_rate * 100.0)?;
        writeln!(f, "Avg fitness:      {:.4}", self.avg_fitness)?;
        writeln!(f, "Max fitness:      {:.4}", self.max_fitness)?;
        writeln!(f, "Avg children:     {:.1}", self.avg_children_per_mutation)?;
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genome_creation() {
        let genome = EvolutionGenome::new(GenomeConfig::default());
        assert_eq!(genome.current_generation, 0);
        assert_eq!(genome.generations.len(), 0);
    }

    #[test]
    fn test_record_mutation() {
        let mut genome = EvolutionGenome::new(GenomeConfig::default());
        let result = genome.record_mutation(
            "Test mutation".to_string(),
            "nt_core".to_string(),
            MutationType::ParameterTune,
            None,
            0.8,
            None,
        );

        assert!(result.is_ok());
        let mutation = result.unwrap();
        assert_eq!(mutation.generation, 0);
        assert!(mutation.fitness > 0.0);
    }

    #[test]
    fn test_record_mutation_with_parent() {
        let mut genome = EvolutionGenome::new(GenomeConfig::default());

        let parent = genome
            .record_mutation(
                "Parent".to_string(),
                "nt_core".to_string(),
                MutationType::ParameterTune,
                None,
                0.5,
                None,
            )
            .unwrap();

        let child = genome.record_mutation(
            "Child".to_string(),
            "nt_core".to_string(),
            MutationType::ParameterTune,
            Some(parent.id.clone()),
            0.7,
            None,
        );

        assert!(child.is_ok());
        let child = child.unwrap();
        assert_eq!(child.parent_id, Some(parent.id.clone()));
        assert!((child.fitness_delta - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_lineage() {
        let mut genome = EvolutionGenome::new(GenomeConfig::default());

        let root = genome
            .record_mutation(
                "Root".to_string(),
                "mod".to_string(),
                MutationType::ParameterTune,
                None,
                0.3,
                None,
            )
            .unwrap();

        let mid = genome
            .record_mutation(
                "Mid".to_string(),
                "mod".to_string(),
                MutationType::ParameterTune,
                Some(root.id.clone()),
                0.5,
                None,
            )
            .unwrap();

        let leaf = genome
            .record_mutation(
                "Leaf".to_string(),
                "mod".to_string(),
                MutationType::ParameterTune,
                Some(mid.id.clone()),
                0.8,
                None,
            )
            .unwrap();

        let chain = genome.lineage(&leaf.id);
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].id, root.id);
        assert_eq!(chain[1].id, mid.id);
        assert_eq!(chain[2].id, leaf.id);
    }

    #[test]
    fn test_rollback() {
        let mut genome = EvolutionGenome::new(GenomeConfig::default());

        let state = StateSnapshot {
            id: "snap1".to_string(),
            state: HashMap::from([("key".to_string(), "value".to_string())]),
            checksum: 12345,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let mutation = genome
            .record_mutation(
                "With state".to_string(),
                "mod".to_string(),
                MutationType::ParameterTune,
                None,
                0.5,
                Some(state),
            )
            .unwrap();

        let rollback = genome.rollback(&mutation.id);
        assert!(rollback.is_ok());
        assert_eq!(rollback.unwrap().state.get("key").unwrap(), "value");
    }

    #[test]
    fn test_fitness_history() {
        let mut genome = EvolutionGenome::new(GenomeConfig::default());

        genome.record_mutation(
            "M1".to_string(),
            "mod".to_string(),
            MutationType::ParameterTune,
            None,
            0.5,
            None,
        );
        genome.next_generation();
        genome.record_mutation(
            "M2".to_string(),
            "mod".to_string(),
            MutationType::ParameterTune,
            None,
            0.7,
            None,
        );

        let history = genome.fitness_history();
        assert_eq!(history.len(), 2);
        assert!((history[0].1 - 0.5).abs() < 0.001);
        assert!((history[1].1 - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_best_mutation() {
        let mut genome = EvolutionGenome::new(GenomeConfig::default());

        genome.record_mutation(
            "Low".to_string(),
            "mod".to_string(),
            MutationType::ParameterTune,
            None,
            0.3,
            None,
        );
        genome.record_mutation(
            "High".to_string(),
            "mod".to_string(),
            MutationType::ParameterTune,
            None,
            0.9,
            None,
        );

        let best = genome.best_mutation().unwrap();
        assert_eq!(best.description, "High");
    }

    #[test]
    fn test_stats() {
        let mut genome = EvolutionGenome::new(GenomeConfig::default());
        genome.record_mutation(
            "M1".to_string(),
            "mod".to_string(),
            MutationType::ParameterTune,
            None,
            0.6,
            None,
        );
        genome.record_mutation(
            "M2".to_string(),
            "mod".to_string(),
            MutationType::ParameterTune,
            None,
            0.8,
            None,
        );

        let stats = genome.stats();
        assert_eq!(stats.total_mutations, 2);
        assert!(stats.avg_fitness > 0.0);
    }

    #[test]
    fn test_nonexistent_mutation_rollback() {
        let genome = EvolutionGenome::new(GenomeConfig::default());
        let result = genome.rollback("nonexistent");
        assert!(result.is_err());
    }
}
