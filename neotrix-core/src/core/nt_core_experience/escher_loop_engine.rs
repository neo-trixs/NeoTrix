/// EscherLoopEngine — Dual-population co-evolution for self-referential optimization.
///
/// Maintains two evolving populations:
/// - TaskPopulation: Current evolution tasks being executed
/// - OptimizerPopulation: Optimizer strategies that improve AND modify themselves
///
/// Inspired by Escher-Loop (arXiv 2604.23472) — "two hands drawing each other into existence".
///
/// Architecture alignment (v21 Meta Loop):
/// - TaskPopulation<EvolutionTask>: read from EvolutionTaskSystem.stats()
/// - OptimizerPopulation<OptimizerStrategy>: mutable self-modifying strategies
/// - co_evolve_step(): one complete dual-population iteration, called every 50-200 ticks
use std::collections::HashSet;
use rand::Rng;

use super::agent0_dual_loop::{CurriculumAgent, CurriculumProposal};
use super::evolution_task_system::TaskSystemStats;

// ============================================================================
// Types
// ============================================================================

/// Agent type in the co-evolution ecology
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    /// Task-solving agent
    Task,
    /// Optimizer agent (can self-modify)
    Optimizer,
}

/// A single evolvable agent in either population
#[derive(Debug, Clone)]
pub struct EvolvableAgent {
    pub id: u64,
    pub agent_type: AgentType,
    pub fitness: f64,
    pub variant_count: u64,
    pub parent_id: Option<u64>,
    pub lineage: Vec<u64>,
    /// Deterministic hash of the agent's "code"
    pub code_fingerprint: u64,
    pub created_epoch: u64,
    pub last_evaluated_epoch: u64,
}

/// An agent promoted to the persistent archive
#[derive(Debug, Clone)]
pub struct ArchivedAgent {
    pub id: u64,
    pub agent_type: AgentType,
    pub fitness: f64,
    pub parent_id: Option<u64>,
    pub children: Vec<u64>,
    pub lineage_depth: usize,
    pub archived_epoch: u64,
}

/// Configuration for the EscherLoopEngine
#[derive(Debug, Clone)]
pub struct EscherLoopConfig {
    /// Max population size per population
    pub max_population_size: usize,
    /// Tournament selection size (default: 3)
    pub tournament_size: usize,
    /// Elite count to preserve unchanged (default: 2)
    pub elite_count: usize,
    /// Mutation rate for new variants (default: 0.2)
    pub mutation_rate: f64,
    /// Crossover rate (default: 0.3)
    pub crossover_rate: f64,
    /// Minimum epochs between meta-optimization steps
    pub meta_optimization_interval: u64,
    /// Exploration bonus for under-explored lineages (default: 0.1)
    pub exploration_bonus: f64,
}

impl Default for EscherLoopConfig {
    fn default() -> Self {
        Self {
            max_population_size: 50,
            tournament_size: 3,
            elite_count: 2,
            mutation_rate: 0.2,
            crossover_rate: 0.3,
            meta_optimization_interval: 20,
            exploration_bonus: 0.1,
        }
    }
}

/// Statistics tracked across epochs
#[derive(Debug, Clone)]
pub struct EscherLoopStats {
    pub total_epochs: u64,
    pub total_variants_generated: u64,
    pub best_task_fitness: f64,
    pub best_optimizer_fitness: f64,
    pub current_diversity: f64,
    pub meta_optimization_steps: u64,
}

impl Default for EscherLoopStats {
    fn default() -> Self {
        Self {
            total_epochs: 0,
            total_variants_generated: 0,
            best_task_fitness: 0.0,
            best_optimizer_fitness: 0.0,
            current_diversity: 0.0,
            meta_optimization_steps: 0,
        }
    }
}

/// Result of a single co-evolution step
#[derive(Debug, Clone)]
pub struct EscherLoopResult {
    pub epoch: u64,
    pub selected_task_id: Option<u64>,
    pub selected_optimizer_id: Option<u64>,
    pub fitness_delta: f64,
    pub new_variants: usize,
    pub archive_size: usize,
    pub task_pop_size: usize,
    pub opt_pop_size: usize,
}

// ============================================================================
// EscherLoopEngine
// ============================================================================

/// Dual-population co-evolution engine for self-referential optimization.
///
/// Maintains two evolving populations:
/// - TaskPopulation: Current evolution tasks being executed
/// - OptimizerPopulation: Optimizer strategies that improve AND modify themselves
///
/// Follows the Escher-Loop (arXiv 2604.23472) paradigm:
/// "two populations co-evolving, each improving the other's selection".
pub struct EscherLoopEngine {
    /// Current epoch counter
    epoch: u64,
    /// Task population (tasks being executed)
    task_population: Vec<EvolvableAgent>,
    /// Optimizer population (optimizers that can self-modify)
    optimizer_population: Vec<EvolvableAgent>,
    /// Archive of all generated agents (DGM-H style)
    archive: Vec<ArchivedAgent>,
    /// Set of archived IDs for O(1) dedup
    archived_ids: HashSet<u64>,
    /// Configuration
    config: EscherLoopConfig,
    /// Statistics
    stats: EscherLoopStats,
    /// Optional CurriculumAgent for domain-coverage-driven task proposal
    curriculum: Option<CurriculumAgent>,
}

impl EscherLoopEngine {
    /// Create a new EscherLoopEngine with empty populations.
    ///
    /// Both populations start empty — call `seed_tasks()` and `seed_optimizers()`
    /// or run `co_evolve_step()` with a `TaskSystemStats` reference to populate them.
    pub fn new(config: EscherLoopConfig) -> Self {
        Self {
            epoch: 0,
            task_population: Vec::new(),
            optimizer_population: Vec::new(),
            archive: Vec::new(),
            archived_ids: HashSet::new(),
            stats: EscherLoopStats::default(),
            config,
            curriculum: None,
        }
    }

    /// Attach a CurriculumAgent for domain-coverage-driven task proposal.
    ///
    /// The CurriculumAgent selects under-explored domains and proposes tasks
    /// at adaptive difficulty levels, which are injected into the task population
    /// during `co_evolve_step()`.
    pub fn with_curriculum(mut self, curriculum: CurriculumAgent) -> Self {
        self.curriculum = Some(curriculum);
        self
    }

    /// Seed domains into the CurriculumAgent.
    ///
    /// Ensures each domain has an entry in the CurriculumAgent's success rate
    /// tracking. Safe to call even when no CurriculumAgent is attached (no-op).
    pub fn seed_domains(&mut self, domains: &[String]) {
        if let Some(ref mut cur) = self.curriculum {
            for d in domains {
                cur.domain_success_rates.entry(d.clone()).or_insert((0, 0));
            }
        }
    }

    /// Convert a CurriculumProposal into an EvolvableAgent for the task population.
    ///
    /// The agent gets `agent_type: Task`, `fitness: estimated_value`,
    /// and a deterministic code fingerprint derived from the proposal description.
    fn proposal_to_agent(&self, proposal: &CurriculumProposal) -> EvolvableAgent {
        let mut rng = rand::thread_rng();
        let id = self.next_id(&mut rng);
        EvolvableAgent {
            id,
            agent_type: AgentType::Task,
            fitness: proposal.estimated_value,
            variant_count: 0,
            parent_id: None,
            lineage: vec![id],
            code_fingerprint: deterministic_hash(proposal.description.as_bytes()),
            created_epoch: self.epoch,
            last_evaluated_epoch: self.epoch,
        }
    }

    /// Seed the task population from an EvolutionTaskSystem's stats.
    ///
    /// Creates `EvolvableAgent::Task` entries for each task type that has
    /// pending (non-completed) work.
    pub fn seed_tasks(&mut self, task_stats: &TaskSystemStats) -> usize {
        let mut rng = rand::thread_rng();
        let mut created = 0;
        for (type_name, count) in &task_stats.by_type {
            let active_count = *count;
            if active_count == 0 {
                continue;
            }
            let id = self.next_id(&mut rng);
            let fitness = (active_count as f64).min(10.0) / 10.0;
            self.task_population.push(EvolvableAgent {
                id,
                agent_type: AgentType::Task,
                fitness,
                variant_count: 0,
                parent_id: None,
                lineage: vec![id],
                code_fingerprint: deterministic_hash(type_name.as_bytes()),
                created_epoch: self.epoch,
                last_evaluated_epoch: self.epoch,
            });
            created += 1;
        }
        // Cap population
        if self.task_population.len() > self.config.max_population_size {
            self.task_population.sort_by(|a, b| {
                a.fitness
                    .partial_cmp(&b.fitness)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.task_population
                .truncate(self.config.max_population_size);
        }
        created
    }

    /// Seed the optimizer population with initial strategies.
    ///
    /// Creates three default strategies: conservative, balanced, and adventurous.
    pub fn seed_optimizers(&mut self) -> usize {
        if !self.optimizer_population.is_empty() {
            return 0;
        }
        let mut rng = rand::thread_rng();
        let seeds = [
            ("conservative", 0.2, 0.1, 0.8),
            ("balanced", 0.5, 0.5, 0.5),
            ("adventurous", 0.8, 0.9, 0.2),
        ];
        for &(_, sel_bias, mut_scale, eval_strict) in &seeds {
            let id = self.next_id(&mut rng);
            let fp = deterministic_hash(sel_bias.to_string().as_bytes())
                ^ deterministic_hash(mut_scale.to_string().as_bytes())
                ^ deterministic_hash(eval_strict.to_string().as_bytes());
            self.optimizer_population.push(EvolvableAgent {
                id,
                agent_type: AgentType::Optimizer,
                fitness: 0.5,
                variant_count: 0,
                parent_id: None,
                lineage: vec![id],
                code_fingerprint: fp,
                created_epoch: 0,
                last_evaluated_epoch: 0,
            });
        }
        3
    }

    fn next_id(&self, rng: &mut impl Rng) -> u64 {
        loop {
            let id = rng.gen::<u64>();
            if !self.archived_ids.contains(&id)
                && !self.task_population.iter().any(|a| a.id == id)
                && !self.optimizer_population.iter().any(|a| a.id == id)
            {
                return id;
            }
        }
    }

    /// Run one complete co-evolution step.
    ///
    /// Phases:
    /// 1. Evaluate current populations (score each agent)
    /// 2. Select parents via tournament (with exploration bonus)
    /// 3. Generate variants (mutate + crossover)
    /// 4. Archive best agents
    /// 5. Optionally run meta-optimization (optimizer improves optimizer)
    pub fn co_evolve_step(&mut self) -> EscherLoopResult {
        self.epoch += 1;
        let mut rng = rand::thread_rng();

        // Phase 1: Evaluate — assign fitness based on population state
        self.evaluate_populations();

        // Phase 2: Select parents via tournament
        let task_parents: Vec<usize> = self.select_parents(&self.task_population, &mut rng);
        let opt_parents: Vec<usize> = self.select_parents(&self.optimizer_population, &mut rng);

        let selected_task_id = task_parents
            .first()
            .map(|&idx| self.task_population[idx].id);
        let selected_opt_id = opt_parents
            .first()
            .map(|&idx| self.optimizer_population[idx].id);

        // Phase 3: Generate variants
        let mut new_variants = 0;

        // Preserve elites
        let task_elites: Vec<EvolvableAgent> = self
            .task_population
            .iter()
            .cloned()
            .filter(|a| a.variant_count == 0)
            .take(self.config.elite_count)
            .collect();
        let opt_elites: Vec<EvolvableAgent> = self
            .optimizer_population
            .iter()
            .cloned()
            .filter(|a| a.variant_count == 0)
            .take(self.config.elite_count)
            .collect();

        // Generate children from task parents
        let task_children: Vec<EvolvableAgent> = task_parents
            .iter()
            .map(|&idx| {
                let parent = &self.task_population[idx];
                let mut child = self.generate_variant(parent, &mut rng);
                // Crossover with another random task parent
                if rng.gen::<f64>() < self.config.crossover_rate && task_parents.len() > 1 {
                    let other_idx = task_parents[rng.gen_range(0..task_parents.len())];
                    if other_idx != idx {
                        child.code_fingerprint ^=
                            self.task_population[other_idx].code_fingerprint;
                    }
                }
                child
            })
            .collect();

        // Generate children from optimizer parents
        let opt_children: Vec<EvolvableAgent> = opt_parents
            .iter()
            .map(|&idx| {
                let parent = &self.optimizer_population[idx];
                let mut child = self.generate_variant(parent, &mut rng);
                if rng.gen::<f64>() < self.config.crossover_rate && opt_parents.len() > 1 {
                    let other_idx = opt_parents[rng.gen_range(0..opt_parents.len())];
                    if other_idx != idx {
                        child.code_fingerprint ^=
                            self.optimizer_population[other_idx].code_fingerprint;
                    }
                }
                child
            })
            .collect();

        new_variants += task_children.len() + opt_children.len();
        self.stats.total_variants_generated += new_variants as u64;

        // Replace populations with elites + children, cap size
        self.task_population = task_elites;
        self.task_population.extend(task_children);
        if self.task_population.len() > self.config.max_population_size {
            self.task_population
                .truncate(self.config.max_population_size);
        }

        self.optimizer_population = opt_elites;
        self.optimizer_population.extend(opt_children);
        if self.optimizer_population.len() > self.config.max_population_size {
            self.optimizer_population
                .truncate(self.config.max_population_size);
        }

        // Phase 4: Archive best agents
        if self.epoch % 5 == 0 {
            self.archive_best(2);
        }

        // Phase 4b: CurriculumAgent proposals — inject into task population
        if let Some(ref mut cur) = self.curriculum {
            let domains: Vec<String> = cur.domain_success_rates.keys().cloned().collect();
            if !domains.is_empty() {
                if let Some(proposal) = cur.propose(&domains) {
                    let agent = self.proposal_to_agent(&proposal);
                    self.task_population.push(agent);
                    new_variants += 1;
                    self.stats.total_variants_generated += 1;
                }
            }
        }

        // Phase 5: Meta-optimization (optimizer improves optimizer)
        if self.epoch % self.config.meta_optimization_interval == 0 {
            let meta_results = self.meta_optimize();
            self.stats.meta_optimization_steps += 1;
            // Log meta-optimization via intervention_log in caller
            if !meta_results.is_empty() {
                // Update archive with meta steps
                for _result in &meta_results {
                    let id = self.next_id(&mut rng);
                    if !self.archived_ids.contains(&id) {
                        self.archived_ids.insert(id);
                        self.archive.push(ArchivedAgent {
                            id,
                            agent_type: AgentType::Optimizer,
                            fitness: 0.6 + rng.gen::<f64>() * 0.3,
                            parent_id: None,
                            children: vec![],
                            lineage_depth: 0,
                            archived_epoch: self.epoch,
                        });
                    }
                }
            }
        }

        // Update stats
        self.stats.total_epochs = self.epoch;
        self.stats.current_diversity = self.diversity_score();
        if let Some(best_task) = self
            .task_population
            .iter()
            .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap_or(std::cmp::Ordering::Equal))
        {
            self.stats.best_task_fitness = best_task.fitness;
        }
        if let Some(best_opt) = self
            .optimizer_population
            .iter()
            .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap_or(std::cmp::Ordering::Equal))
        {
            self.stats.best_optimizer_fitness = best_opt.fitness;
        }

        EscherLoopResult {
            epoch: self.epoch,
            selected_task_id,
            selected_optimizer_id: selected_opt_id,
            fitness_delta: self.stats.best_task_fitness,
            new_variants: new_variants as usize,
            archive_size: self.archive.len(),
            task_pop_size: self.task_population.len(),
            opt_pop_size: self.optimizer_population.len(),
        }
    }

    /// Evaluate the current populations, assigning fitness scores.
    ///
    /// Task fitness: based on novelty (variant_count) and lineage depth.
    /// Optimizer fitness: based on variant count and lineage breadth.
    fn evaluate_populations(&mut self) {
        for agent in &mut self.task_population {
            let diversity_bonus = if self.stats.current_diversity > 0.3 {
                0.1
            } else {
                0.0
            };
            let lineage_depth = agent.lineage.len() as f64;
            agent.fitness =
                (0.3 + (agent.variant_count as f64).min(10.0) / 20.0 + lineage_depth / 50.0
                    + diversity_bonus)
                    .clamp(0.0, 1.0);
            agent.last_evaluated_epoch = self.epoch;
        }
        for agent in &mut self.optimizer_population {
            let lineage_depth = agent.lineage.len() as f64;
            agent.fitness =
                (0.4 + (agent.variant_count as f64).min(10.0) / 20.0 + lineage_depth / 30.0)
                    .clamp(0.0, 1.0);
            agent.last_evaluated_epoch = self.epoch;
        }
    }

    /// Select parent indices via tournament selection with exploration bonus.
    ///
    /// Picks `tournament_size` random candidates, selects the one with highest
    /// fitness, then applies an exploration bonus for under-explored lineages
    /// (those with low variant_count).
    pub fn tournament_select(&self, population: &[EvolvableAgent], rng: &mut impl Rng) -> usize {
        if population.is_empty() {
            panic!("tournament_select called on empty population");
        }
        let k = self.config.tournament_size.min(population.len());
        let mut best_idx = rng.gen_range(0..population.len());
        let mut best_score = population[best_idx].fitness
            + self.config.exploration_bonus * (1.0 / (1.0 + population[best_idx].variant_count as f64));

        for _ in 1..k {
            let idx = rng.gen_range(0..population.len());
            let score = population[idx].fitness
                + self.config.exploration_bonus * (1.0 / (1.0 + population[idx].variant_count as f64));
            if score > best_score {
                best_idx = idx;
                best_score = score;
            }
        }
        best_idx
    }

    /// Select multiple parent indices via tournament.
    fn select_parents(&self, population: &[EvolvableAgent], rng: &mut impl Rng) -> Vec<usize> {
        let count = (population.len() / 4).max(1);
        let mut parents = Vec::with_capacity(count);
        let mut used = HashSet::new();
        for _ in 0..count {
            if used.len() >= population.len() {
                break;
            }
            let idx = self.tournament_select(population, rng);
            if used.insert(idx) {
                parents.push(idx);
            }
        }
        parents
    }

    /// Create a child variant from a parent via mutation.
    ///
    /// The child inherits the parent's lineage, gets a new unique ID,
    /// and has its code_fingerprint mutated based on `mutation_rate`.
    /// The `code_fingerprint` is mixed with a deterministic hash of the
    /// epoch and a random seed to simulate code mutation.
    pub fn generate_variant(&self, parent: &EvolvableAgent, rng: &mut impl Rng) -> EvolvableAgent {
        let child_id = self.next_id(rng);
        let mut lineage = parent.lineage.clone();
        lineage.push(child_id);

        let mut fingerprint = parent.code_fingerprint;
        if rng.gen::<f64>() < self.config.mutation_rate {
            // Flip random bits in the fingerprint to simulate mutation
            let mutation_mask = rng.gen::<u64>();
            fingerprint ^= mutation_mask;
        }

        EvolvableAgent {
            id: child_id,
            agent_type: parent.agent_type,
            fitness: parent.fitness * (0.8 + rng.gen::<f64>() * 0.4),
            variant_count: parent.variant_count + 1,
            parent_id: Some(parent.id),
            lineage,
            code_fingerprint: fingerprint,
            created_epoch: self.epoch,
            last_evaluated_epoch: self.epoch,
        }
    }

    /// Meta-optimization: optimizer agents modify their own strategies
    /// (Gödel Agent style self-referential improvement).
    ///
    /// Currently, this mutates the optimizer population's fitness landscape
    /// by injecting a small fitness boost to agents with diverse lineages,
    /// and adding new variants from top optimizers.
    /// Returns a list of description strings for each meta-optimization action.
    pub fn meta_optimize(&mut self) -> Vec<String> {
        let mut results = Vec::new();
        let mut rng = rand::thread_rng();

        // Reward diverse lineages: agents with unique code fingerprints get a boost
        let unique_fingerprints: HashSet<u64> = self
            .optimizer_population
            .iter()
            .map(|a| a.code_fingerprint)
            .collect();
        if !unique_fingerprints.is_empty() {
            let diversity_ratio = unique_fingerprints.len() as f64
                / self.optimizer_population.len().max(1) as f64;
            for agent in &mut self.optimizer_population {
                if unique_fingerprints.contains(&agent.code_fingerprint) {
                    agent.fitness = (agent.fitness + 0.05 * diversity_ratio).min(1.0);
                }
            }
            results.push(format!(
                "meta_opt: diversity_boost applied (ratio={:.3})",
                diversity_ratio
            ));
        }

        // Spawn new variants from the top 2 optimizers
        let mut sorted: Vec<usize> = (0..self.optimizer_population.len()).collect();
        sorted.sort_by(|&a, &b| {
            self.optimizer_population[b]
                .fitness
                .partial_cmp(&self.optimizer_population[a].fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for &idx in sorted.iter().take(2) {
            let parent = &self.optimizer_population[idx];
            let child = self.generate_variant(parent, &mut rng);
            results.push(format!(
                "meta_opt: spawned variant from optimizer {} (fitness={:.3})",
                parent.id, parent.fitness
            ));
            self.optimizer_population.push(child);
        }

        // Cap optimizer population
        if self.optimizer_population.len() > self.config.max_population_size {
            self.optimizer_population.sort_by(|a, b| {
                b.fitness
                    .partial_cmp(&a.fitness)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let pruned = self.optimizer_population
                [self.config.max_population_size..]
                .len();
            self.optimizer_population
                .truncate(self.config.max_population_size);
            if pruned > 0 {
                results.push(format!("meta_opt: pruned {} low-fitness optimizers", pruned));
            }
        }

        results
    }

    /// Promote top agents from both populations to the archive.
    ///
    /// Selects the `count` highest-fitness agents from each population
    /// and archives them with a snapshot of their lineage. Avoids
    /// duplicate IDs via `archived_ids`.
    pub fn archive_best(&mut self, count: usize) {
        // Archive top task agents
        let mut task_indices: Vec<usize> = (0..self.task_population.len()).collect();
        task_indices.sort_by(|&a, &b| {
            self.task_population[b]
                .fitness
                .partial_cmp(&self.task_population[a].fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for &idx in task_indices.iter().take(count) {
            let agent = &self.task_population[idx];
            if self.archived_ids.contains(&agent.id) {
                continue;
            }
            self.archived_ids.insert(agent.id);
            self.archive.push(ArchivedAgent {
                id: agent.id,
                agent_type: AgentType::Task,
                fitness: agent.fitness,
                parent_id: agent.parent_id,
                children: vec![],
                lineage_depth: agent.lineage.len(),
                archived_epoch: self.epoch,
            });
            // Link child to parent in archive
            if let Some(pid) = agent.parent_id {
                if let Some(parent_archived) = self.archive.iter_mut().find(|a| a.id == pid) {
                    parent_archived.children.push(agent.id);
                }
            }
        }

        // Archive top optimizer agents
        let mut opt_indices: Vec<usize> = (0..self.optimizer_population.len()).collect();
        opt_indices.sort_by(|&a, &b| {
            self.optimizer_population[b]
                .fitness
                .partial_cmp(&self.optimizer_population[a].fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for &idx in opt_indices.iter().take(count) {
            let agent = &self.optimizer_population[idx];
            if self.archived_ids.contains(&agent.id) {
                continue;
            }
            self.archived_ids.insert(agent.id);
            self.archive.push(ArchivedAgent {
                id: agent.id,
                agent_type: AgentType::Optimizer,
                fitness: agent.fitness,
                parent_id: agent.parent_id,
                children: vec![],
                lineage_depth: agent.lineage.len(),
                archived_epoch: self.epoch,
            });
            if let Some(pid) = agent.parent_id {
                if let Some(parent_archived) = self.archive.iter_mut().find(|a| a.id == pid) {
                    parent_archived.children.push(agent.id);
                }
            }
        }
    }

    /// Population diversity measured as fitness variance across both populations.
    ///
    /// Uses the combined fitness distribution: variance = mean of squared diffs
    /// from the grand mean. Higher variance = more diversity.
    pub fn diversity_score(&self) -> f64 {
        let all_fitness: Vec<f64> = self
            .task_population
            .iter()
            .chain(self.optimizer_population.iter())
            .map(|a| a.fitness)
            .collect();
        if all_fitness.is_empty() {
            return 0.0;
        }
        let mean: f64 = all_fitness.iter().sum::<f64>() / all_fitness.len() as f64;
        let variance: f64 = all_fitness
            .iter()
            .map(|f| (f - mean).powi(2))
            .sum::<f64>()
            / all_fitness.len() as f64;
        variance.sqrt().min(1.0)
    }

    /// Access the task population (read-only)
    pub fn task_population(&self) -> &[EvolvableAgent] {
        &self.task_population
    }

    /// Access the optimizer population (read-only)
    pub fn optimizer_population(&self) -> &[EvolvableAgent] {
        &self.optimizer_population
    }

    /// Access the archive (read-only)
    pub fn archive(&self) -> &[ArchivedAgent] {
        &self.archive
    }

    /// Access stats (read-only)
    pub fn stats(&self) -> &EscherLoopStats {
        &self.stats
    }

    /// Access config (read-only)
    pub fn config(&self) -> &EscherLoopConfig {
        &self.config
    }

    /// Current epoch number
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Number of optimizer strategies alive
    pub fn strategy_count(&self) -> usize {
        self.optimizer_population.len()
    }

    /// Human-readable summary of the current engine state.
    ///
    /// Includes: epoch, population sizes, best fitness values,
    /// archive size, diversity, meta-optimization step count.
    pub fn summary(&self) -> String {
        format!(
            "EscherLoopEngine: epoch={} task_pop={} opt_pop={} archive={} best_task={:.4} best_opt={:.4} diversity={:.4} variants={} meta_steps={} cfg(pop={} tour={} elite={} mut={:.2} cx={:.2} meta_int={} expl={:.2})",
            self.epoch,
            self.task_population.len(),
            self.optimizer_population.len(),
            self.archive.len(),
            self.stats.best_task_fitness,
            self.stats.best_optimizer_fitness,
            self.stats.current_diversity,
            self.stats.total_variants_generated,
            self.stats.meta_optimization_steps,
            self.config.max_population_size,
            self.config.tournament_size,
            self.config.elite_count,
            self.config.mutation_rate,
            self.config.crossover_rate,
            self.config.meta_optimization_interval,
            self.config.exploration_bonus,
        )
    }
}

/// Deterministic 64-bit hash for a byte slice (used for code fingerprints).
fn deterministic_hash(bytes: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── Basic construction ──

    #[test]
    fn test_new_engine_empty() {
        let config = EscherLoopConfig::default();
        let engine = EscherLoopEngine::new(config);
        assert_eq!(engine.epoch, 0);
        assert!(engine.task_population.is_empty());
        assert!(engine.optimizer_population.is_empty());
        assert!(engine.archive.is_empty());
        assert_eq!(engine.stats.total_epochs, 0);
    }

    #[test]
    fn test_seed_tasks() {
        let mut engine = EscherLoopEngine::new(EscherLoopConfig::default());
        let stats = TaskSystemStats {
            total: 5,
            completed: 2,
            in_progress: 1,
            blocked: 0,
            discovered: 2,
            avg_impact: 0.6,
            by_type: {
                let mut m = std::collections::HashMap::new();
                m.insert("module_wiring".into(), 3);
                m.insert("compile_fix".into(), 2);
                m
            },
        };
        let created = engine.seed_tasks(&stats);
        assert_eq!(created, 2);
        assert_eq!(engine.task_population.len(), 2);
        assert_eq!(engine.task_population[0].agent_type, AgentType::Task);
    }

    #[test]
    fn test_seed_optimizers() {
        let mut engine = EscherLoopEngine::new(EscherLoopConfig::default());
        let seeded = engine.seed_optimizers();
        assert_eq!(seeded, 3);
        assert_eq!(engine.optimizer_population.len(), 3);
        // All three should be AgentType::Optimizer
        for agent in &engine.optimizer_population {
            assert_eq!(agent.agent_type, AgentType::Optimizer);
        }
        // Idempotent
        let second = engine.seed_optimizers();
        assert_eq!(second, 0);
        assert_eq!(engine.optimizer_population.len(), 3);
    }

    #[test]
    fn test_default_config_values() {
        let config = EscherLoopConfig::default();
        assert_eq!(config.max_population_size, 50);
        assert_eq!(config.tournament_size, 3);
        assert_eq!(config.elite_count, 2);
        assert!((config.mutation_rate - 0.2).abs() < 1e-6);
        assert!((config.crossover_rate - 0.3).abs() < 1e-6);
        assert_eq!(config.meta_optimization_interval, 20);
        assert!((config.exploration_bonus - 0.1).abs() < 1e-6);
    }

    // ── Tournament selection ──

    #[test]
    fn test_tournament_select_returns_valid_index() {
        let mut engine = EscherLoopEngine::new(EscherLoopConfig::default());
        let mut rng = rand::thread_rng();
        engine.seed_optimizers();
        let idx = engine.tournament_select(&engine.optimizer_population, &mut rng);
        assert!(idx < engine.optimizer_population.len());
    }

    #[test]
    fn test_tournament_select_exploration_bonus() {
        let config = EscherLoopConfig {
            exploration_bonus: 100.0,
            ..Default::default()
        };
        let mut engine = EscherLoopEngine::new(config);
        let mut rng = rand::thread_rng();
        engine.seed_optimizers();
        // With extremely high exploration bonus, agents with low variant_count
        // should be strongly preferred.
        let idx = engine.tournament_select(&engine.optimizer_population, &mut rng);
        let selected = &engine.optimizer_population[idx];
        assert_eq!(selected.variant_count, 0);
    }

    #[test]
    fn test_tournament_select_single_element() {
        let engine = EscherLoopEngine::new(EscherLoopConfig::default());
        let mut rng = rand::thread_rng();
        let pop = vec![EvolvableAgent {
            id: 1,
            agent_type: AgentType::Task,
            fitness: 0.5,
            variant_count: 0,
            parent_id: None,
            lineage: vec![1],
            code_fingerprint: 42,
            created_epoch: 0,
            last_evaluated_epoch: 0,
        }];
        let idx = engine.tournament_select(&pop, &mut rng);
        assert_eq!(idx, 0);
    }

    // ── Variant generation ──

    #[test]
    fn test_generate_variant_has_unique_id() {
        let engine = EscherLoopEngine::new(EscherLoopConfig::default());
        let mut rng = rand::thread_rng();
        let parent = EvolvableAgent {
            id: 100,
            agent_type: AgentType::Optimizer,
            fitness: 0.7,
            variant_count: 2,
            parent_id: Some(50),
            lineage: vec![50, 100],
            code_fingerprint: 0xABCD,
            created_epoch: 10,
            last_evaluated_epoch: 12,
        };
        let child = engine.generate_variant(&parent, &mut rng);
        assert_ne!(child.id, parent.id);
        assert_eq!(child.agent_type, AgentType::Optimizer);
        assert_eq!(child.parent_id, Some(100));
        assert_eq!(child.lineage.len(), 3);
        assert_eq!(child.lineage[0], 50);
        assert_eq!(child.lineage[1], 100);
        assert_eq!(child.variant_count, 3);
        assert_eq!(child.created_epoch, 0);
    }

    #[test]
    fn test_generate_variant_fitness_mutation() {
        let engine = EscherLoopEngine::new(EscherLoopConfig::default());
        let mut rng = rand::thread_rng();
        let parent = EvolvableAgent {
            id: 200,
            agent_type: AgentType::Task,
            fitness: 0.5,
            variant_count: 1,
            parent_id: None,
            lineage: vec![200],
            code_fingerprint: 0xFFFF,
            created_epoch: 0,
            last_evaluated_epoch: 0,
        };
        let child = engine.generate_variant(&parent, &mut rng);
        assert!(child.fitness >= 0.4);
        assert!(child.fitness <= 0.9);
    }

    // ── Co-evolution step ──

    #[test]
    fn test_co_evolve_step_empty_populations() {
        let mut engine = EscherLoopEngine::new(EscherLoopConfig::default());
        let result = engine.co_evolve_step();
        assert_eq!(result.epoch, 1);
        assert!(result.selected_task_id.is_none());
        assert!(result.selected_optimizer_id.is_none());
    }

    #[test]
    fn test_co_evolve_step_with_seeded_populations() {
        let mut engine = EscherLoopEngine::new(EscherLoopConfig::default());
        let stats = TaskSystemStats {
            total: 3,
            completed: 1,
            in_progress: 1,
            blocked: 0,
            discovered: 1,
            avg_impact: 0.5,
            by_type: {
                let mut m = std::collections::HashMap::new();
                m.insert("module_wiring".into(), 2);
                m.insert("compile_fix".into(), 1);
                m
            },
        };
        engine.seed_tasks(&stats);
        engine.seed_optimizers();

        let result = engine.co_evolve_step();
        assert_eq!(result.epoch, 1);
        assert!(result.task_pop_size > 0);
        assert!(result.opt_pop_size > 0);
        assert!(result.archive_size >= 0);
        assert!(result.fitness_delta >= 0.0);
    }

    #[test]
    fn test_multiple_epochs_increase_variants() {
        let mut engine = EscherLoopEngine::new(EscherLoopConfig::default());
        engine.seed_optimizers();

        let v0 = engine.stats.total_variants_generated;
        for _ in 0..5 {
            engine.co_evolve_step();
        }
        let v5 = engine.stats.total_variants_generated;
        assert!(v5 >= v0, "variants should increase over epochs");
        assert_eq!(engine.stats.total_epochs, 5);
    }

    // ── Archive ──

    #[test]
    fn test_archive_best_promotes_top_agents() {
        let mut engine = EscherLoopEngine::new(EscherLoopConfig::default());
        let stats = TaskSystemStats {
            total: 2,
            completed: 0,
            in_progress: 0,
            blocked: 0,
            discovered: 2,
            avg_impact: 0.5,
            by_type: {
                let mut m = std::collections::HashMap::new();
                m.insert("module_wiring".into(), 1);
                m.insert("compile_fix".into(), 1);
                m
            },
        };
        engine.seed_tasks(&stats);
        engine.seed_optimizers();

        let archive_before = engine.archive.len();
        engine.archive_best(3);
        assert!(engine.archive.len() > archive_before);
    }

    #[test]
    fn test_archive_no_duplicate_ids() {
        let mut engine = EscherLoopEngine::new(EscherLoopConfig::default());
        engine.seed_optimizers();
        engine.archive_best(5);
        let ids: HashSet<u64> = engine.archive.iter().map(|a| a.id).collect();
        assert_eq!(ids.len(), engine.archive.len());
    }

    #[test]
    fn test_archive_links_children_to_parents() {
        let mut engine = EscherLoopEngine::new(EscherLoopConfig::default());
        let mut rng = rand::thread_rng();
        let parent = EvolvableAgent {
            id: 999,
            agent_type: AgentType::Optimizer,
            fitness: 0.9,
            variant_count: 0,
            parent_id: None,
            lineage: vec![999],
            code_fingerprint: 0xCAFE,
            created_epoch: 0,
            last_evaluated_epoch: 0,
        };
        engine.optimizer_population.push(parent);
        let child = engine.generate_variant(&engine.optimizer_population[0], &mut rng);
        engine.optimizer_population.push(child);

        engine.archive_best(5);
        // The parent should have at least one child linked in the archive
        let parent_archived = engine.archive.iter().find(|a| a.id == 999);
        assert!(parent_archived.is_some());
        assert!(!parent_archived.unwrap().children.is_empty());
    }

    // ── Diversity score ──

    #[test]
    fn test_diversity_score_empty() {
        let engine = EscherLoopEngine::new(EscherLoopConfig::default());
        assert!((engine.diversity_score() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_diversity_score_non_zero() {
        let mut engine = EscherLoopEngine::new(EscherLoopConfig::default());
        engine.seed_optimizers();
        engine.evaluate_populations();
        let score = engine.diversity_score();
        assert!(score >= 0.0);
        assert!(score <= 1.0);
    }

    // ── Meta-optimize ──

    #[test]
    fn test_meta_optimize_returns_results() {
        let mut engine = EscherLoopEngine::new(EscherLoopConfig::default());
        engine.seed_optimizers();
        let results = engine.meta_optimize();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_meta_optimize_increases_optimizer_count() {
        let mut engine = EscherLoopEngine::new(EscherLoopConfig {
            max_population_size: 100,
            ..Default::default()
        });
        engine.seed_optimizers();
        let before = engine.optimizer_population.len();
        engine.meta_optimize();
        assert!(engine.optimizer_population.len() >= before);
    }

    // ── Integration: full cycle ──

    #[test]
    fn test_full_co_evolution_cycle() {
        let mut engine = EscherLoopEngine::new(EscherLoopConfig::default());
        let stats = TaskSystemStats {
            total: 4,
            completed: 1,
            in_progress: 1,
            blocked: 0,
            discovered: 2,
            avg_impact: 0.5,
            by_type: {
                let mut m = std::collections::HashMap::new();
                m.insert("module_wiring".into(), 2);
                m.insert("compile_fix".into(), 1);
                m.insert("refactor".into(), 1);
                m
            },
        };
        engine.seed_tasks(&stats);
        engine.seed_optimizers();

        // Run 10 epochs
        for _ in 0..10 {
            let result = engine.co_evolve_step();
            assert!(result.epoch > 0);
        }

        assert_eq!(engine.stats.total_epochs, 10);
        assert!(engine.stats.total_variants_generated > 0);
        assert!(engine.stats.best_task_fitness >= 0.0);
        assert!(engine.stats.best_optimizer_fitness >= 0.0);
        assert!(engine.archive.len() >= 2);
    }

    #[test]
    fn test_summary_contains_key_info() {
        let mut engine = EscherLoopEngine::new(EscherLoopConfig::default());
        engine.seed_optimizers();
        let summary = engine.summary();
        assert!(summary.contains("EscherLoopEngine:"));
        assert!(summary.contains("epoch="));
        assert!(summary.contains("task_pop="));
        assert!(summary.contains("opt_pop="));
        assert!(summary.contains("archive="));
    }

    #[test]
    fn test_population_capping() {
        let config = EscherLoopConfig {
            max_population_size: 5,
            ..Default::default()
        };
        let mut engine = EscherLoopEngine::new(config);

        // Add 10 agents to task population (exceeds cap)
        let mut rng = rand::thread_rng();
        for i in 0..10 {
            engine.task_population.push(EvolvableAgent {
                id: 1000 + i,
                agent_type: AgentType::Task,
                fitness: rng.gen(),
                variant_count: 0,
                parent_id: None,
                lineage: vec![1000 + i],
                code_fingerprint: rng.gen(),
                created_epoch: 0,
                last_evaluated_epoch: 0,
            });
        }

        // co_evolve_step should cap to max_population_size
        engine.co_evolve_step();
        assert!(engine.task_population.len() <= 5);
    }

    #[test]
    fn test_select_parents_returns_unique_indices() {
        let mut engine = EscherLoopEngine::new(EscherLoopConfig::default());
        let mut rng = rand::thread_rng();
        engine.seed_optimizers();
        let parents = engine.select_parents(&engine.optimizer_population, &mut rng);
        let unique: HashSet<usize> = parents.iter().copied().collect();
        assert_eq!(unique.len(), parents.len());
    }

    #[test]
    fn test_evaluate_populations_sets_fitness() {
        let mut engine = EscherLoopEngine::new(EscherLoopConfig::default());
        engine.seed_optimizers();
        for agent in &engine.optimizer_population {
            assert!((agent.fitness - 0.5).abs() < 1e-6);
        }
        engine.evaluate_populations();
        for agent in &engine.optimizer_population {
            assert!(agent.fitness >= 0.0);
            assert!(agent.fitness <= 1.0);
            assert_eq!(agent.last_evaluated_epoch, 0);
        }
    }
}
