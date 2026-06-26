use super::types::*;
use rand::Rng;
use std::collections::HashMap;

pub struct AdversarialArena {
    pub population: Vec<AgentGenotype>,
    pub gödel_population: Vec<GödelAgent>,
    pub generation: u32,
    pub round: u64,
    pub history: Vec<GenerationResult>,
    pub config: ArenaConfig,
    default_trait_names: Vec<String>,
}

impl AdversarialArena {
    pub fn new(config: ArenaConfig) -> Self {
        Self {
            population: Vec::new(),
            gödel_population: Vec::new(),
            generation: 0,
            round: 0,
            history: Vec::new(),
            config,
            default_trait_names: Vec::new(),
        }
    }

    pub fn seed_population(&mut self, trait_names: &[&str], std: f64) {
        self.default_trait_names = trait_names.iter().map(|s| s.to_string()).collect();
        let mut rng = fast_rng(42);
        for i in 0..self.config.population_size {
            let mut traits = HashMap::new();
            for name in trait_names {
                let v = (rng() * std * 2.0) + (1.0 - std);
                traits.insert(name.to_string(), v.clamp(0.0, 1.0));
            }
            let mut agent = AgentGenotype::new(&format!("gen0-{}", i), traits);
            agent.generation = 0;
            self.population.push(agent);
        }
    }

    pub fn run_round(
        &mut self,
        task: &str,
        score_fn: &mut impl FnMut(&AgentGenotype) -> f64,
    ) -> GenerationResult {
        self.round += 1;

        if self.population.is_empty() {
            return GenerationResult {
                generation: self.generation,
                population_size: 0,
                matches: Vec::new(),
                top_fitness: 0.0,
                avg_fitness: 0.0,
                diversity: 0.0,
            };
        }

        let mut matches = Vec::new();
        let _rng = fast_rng(self.round);

        for agent in self.population.iter_mut() {
            agent.fitness = score_fn(agent);
        }

        let n = self.population.len();
        let tournament_size = self.config.tournament_size.min(n);
        if tournament_size < 2 {
            let top = self.population.iter().max_by(|a, b| {
                a.fitness
                    .partial_cmp(&b.fitness)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let bot = self.population.iter().min_by(|a, b| {
                a.fitness
                    .partial_cmp(&b.fitness)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            if let (Some(winner), Some(loser)) = (top, bot) {
                if winner.id != loser.id {
                    matches.push(MatchResult {
                        round: self.round,
                        task: task.to_string(),
                        agent_id: winner.id.clone(),
                        opponent_id: loser.id.clone(),
                        agent_score: winner.fitness,
                        opponent_score: loser.fitness,
                        agent_won: true,
                    });
                }
            }
        } else {
            let mut indices: Vec<usize> = (0..n).collect();
            for _ in 0..(n * 2) {
                if indices.len() < tournament_size * 2 {
                    break;
                }
                let pick = |pool: &mut Vec<usize>| -> Option<usize> {
                    if pool.is_empty() {
                        return None;
                    }
                    let mut best = pool.remove(rand::thread_rng().gen_range(0..pool.len()));
                    for _ in 0..tournament_size - 1 {
                        if pool.is_empty() {
                            break;
                        }
                        let idx = pool.remove(rand::thread_rng().gen_range(0..pool.len()));
                        if self.population[idx].fitness > self.population[best].fitness {
                            best = idx;
                        }
                    }
                    Some(best)
                };

                if let (Some(a_idx), Some(b_idx)) = (pick(&mut indices), pick(&mut indices)) {
                    let a = &self.population[a_idx];
                    let b = &self.population[b_idx];
                    matches.push(MatchResult {
                        round: self.round,
                        task: task.to_string(),
                        agent_id: a.id.clone(),
                        opponent_id: b.id.clone(),
                        agent_score: a.fitness,
                        opponent_score: b.fitness,
                        agent_won: a.fitness > b.fitness,
                    });
                }
            }
        }

        let scores: Vec<f64> = self.population.iter().map(|a| a.fitness).collect();
        let top_fitness = scores.iter().cloned().fold(0.0_f64, f64::max);
        let avg_fitness = if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };
        let diversity = self.compute_diversity();

        let result = GenerationResult {
            generation: self.generation,
            population_size: self.population.len(),
            matches,
            top_fitness,
            avg_fitness,
            diversity,
        };

        self.history.push(result.clone());
        self.generation += 1;

        result
    }

    pub fn evolve(&mut self, rng_seed: u64) {
        if self.population.is_empty() || self.history.is_empty() {
            return;
        }

        let _last = match self.history.last() {
            Some(l) => l,
            None => return,
        };
        let scores: Vec<f64> = self.population.iter().map(|a| a.fitness).collect();
        let mut indexed: Vec<(usize, f64)> =
            scores.iter().enumerate().map(|(i, v)| (i, *v)).collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut rng = fast_rng(rng_seed ^ self.generation as u64);
        let mut next_gen: Vec<AgentGenotype> = Vec::new();

        for i in 0..self.config.elite_count.min(indexed.len()) {
            let mut elite = self.population[indexed[i].0].clone();
            elite.survived_rounds += 1;
            elite.generation = self.generation;
            next_gen.push(elite);
        }

        while next_gen.len() < self.config.population_size {
            let parent_a = tournament_pick(
                &self.population,
                &scores,
                self.config.tournament_size,
                &mut rng,
            );
            let parent_b = tournament_pick(
                &self.population,
                &scores,
                self.config.tournament_size,
                &mut rng,
            );

            let child = if rng() < self.config.crossover_rate {
                parent_a.crossover(parent_b, &mut rng, 0.5)
            } else {
                parent_a.clone()
            };

            let child = child.mutate(
                &mut rng,
                self.config.mutation_rate,
                self.config.mutation_sigma,
            );
            next_gen.push(child);
        }

        self.population = next_gen;
    }

    pub fn compute_diversity(&self) -> f64 {
        if self.population.len() < 2 {
            return 1.0;
        }
        let n = self.population.len();
        let trait_keys: Vec<&String> = self.population[0].traits.keys().collect();
        if trait_keys.is_empty() {
            return 1.0;
        }
        let mut total_dist = 0.0;
        let mut pairs = 0;
        for i in 0..n {
            for j in (i + 1)..n {
                let mut dist = 0.0;
                for k in &trait_keys {
                    let a = self.population[i].traits.get(*k).copied().unwrap_or(0.0);
                    let b = self.population[j].traits.get(*k).copied().unwrap_or(0.0);
                    dist += (a - b).abs();
                }
                total_dist += dist / trait_keys.len() as f64;
                pairs += 1;
            }
        }
        total_dist / pairs as f64
    }

    pub fn seed_gödel_population(&mut self, base_code_template: &str, count: usize, std: f64) {
        let mut rng = fast_rng(42);
        for i in 0..count {
            let mut traits = HashMap::new();
            traits.insert(
                "speed".to_string(),
                ((rng() * std * 2.0) + (1.0 - std)).clamp(0.0, 1.0),
            );
            traits.insert(
                "accuracy".to_string(),
                ((rng() * std * 2.0) + (1.0 - std)).clamp(0.0, 1.0),
            );
            traits.insert(
                "plasticity".to_string(),
                ((rng() * std * 2.0) + (1.0 - std)).clamp(0.0, 1.0),
            );
            let code = base_code_template
                .replace("{id}", &format!("gödel-{}", i))
                .replace("{seed}", &format!("{}", (rng() * 1_000_000.0) as u64));
            self.gödel_population.push(GödelAgent {
                id: format!("gödel-{}", i),
                parent_ids: Vec::new(),
                generation: 0,
                fitness: 0.0,
                code: code.clone(),
                traits,
                archive_entries: Vec::new(),
                base_strategy: code,
                weight_deltas: HashMap::new(),
                harness_version: 0,
            });
        }
    }

    pub fn run_gödel_round(
        &mut self,
        task: &str,
        score_fn: &mut impl FnMut(&GödelAgent) -> f64,
        mutate_code_fn: &mut dyn FnMut(&str, &mut dyn FnMut() -> f64) -> String,
    ) -> GenerationResult {
        self.round += 1;

        if self.gödel_population.is_empty() {
            return GenerationResult {
                generation: self.generation,
                population_size: 0,
                matches: Vec::new(),
                top_fitness: 0.0,
                avg_fitness: 0.0,
                diversity: 0.0,
            };
        }

        let mut matches = Vec::new();
        let mut rng = fast_rng(self.round);

        for agent in self.gödel_population.iter_mut() {
            agent.fitness = score_fn(agent);
        }

        let n = self.gödel_population.len();
        if n >= 2 {
            let mut indices: Vec<usize> = (0..n).collect();
            let tournament_size = self.config.tournament_size.min(n);
            for _ in 0..(n.min(10)) {
                if indices.len() < tournament_size * 2 {
                    break;
                }
                let pick = |pool: &mut Vec<usize>| -> Option<usize> {
                    if pool.is_empty() {
                        return None;
                    }
                    let mut best = pool.remove(rand::thread_rng().gen_range(0..pool.len()));
                    for _ in 0..tournament_size - 1 {
                        if pool.is_empty() {
                            break;
                        }
                        let idx = pool.remove(rand::thread_rng().gen_range(0..pool.len()));
                        if self.gödel_population[idx].fitness > self.gödel_population[best].fitness
                        {
                            best = idx;
                        }
                    }
                    Some(best)
                };

                if let (Some(a_idx), Some(b_idx)) = (pick(&mut indices), pick(&mut indices)) {
                    let a = &self.gödel_population[a_idx];
                    let b = &self.gödel_population[b_idx];
                    matches.push(MatchResult {
                        round: self.round,
                        task: task.to_string(),
                        agent_id: a.id.clone(),
                        opponent_id: b.id.clone(),
                        agent_score: a.fitness,
                        opponent_score: b.fitness,
                        agent_won: a.fitness > b.fitness,
                    });
                }
            }
        }

        for agent in self.gödel_population.iter_mut() {
            let new_code = mutate_code_fn(&agent.code, &mut rng);
            if new_code != agent.code {
                agent
                    .archive_entries
                    .push(format!("gen{}:{}", self.generation, agent.fitness));
                agent.code = new_code;
                agent.parent_ids.push(agent.id.clone());
                agent.id = format!("{}-g{}", agent.id, self.generation);
                agent.generation += 1;
            }
        }

        let scores: Vec<f64> = self.gödel_population.iter().map(|a| a.fitness).collect();
        let top_fitness = scores.iter().cloned().fold(0.0_f64, f64::max);
        let avg_fitness = if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

        let result = GenerationResult {
            generation: self.generation,
            population_size: self.gödel_population.len(),
            matches,
            top_fitness,
            avg_fitness,
            diversity: 0.0,
        };

        self.history.push(result.clone());
        self.generation += 1;

        result
    }

    pub fn archive_evolution(&self, archive: &mut Vec<ArchiveEntry>, keep_top: usize) {
        let mut scored: Vec<ArchiveEntry> = archive.drain(..).collect();
        scored.sort_by(|a, b| {
            b.fitness
                .partial_cmp(&a.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut seen = std::collections::HashSet::new();
        for entry in scored {
            if seen.len() >= keep_top {
                break;
            }
            if seen.insert(entry.agent_id.clone()) {
                archive.push(entry);
            }
        }
    }

    pub fn clade_metaproductivity(agents: &[GödelAgent]) -> f64 {
        if agents.is_empty() {
            return 0.0;
        }
        let sum: f64 = agents.iter().map(|a| a.fitness).sum();
        sum / agents.len() as f64
    }

    pub fn self_consistency_check(code: &str, invariants: &[&str]) -> Vec<String> {
        let mut violations = Vec::new();
        for inv in invariants {
            if !code.contains(inv) {
                violations.push(format!("Missing required pattern: '{}'", inv));
            }
        }
        let open_parens = code.matches('(').count();
        let close_parens = code.matches(')').count();
        if open_parens != close_parens {
            violations.push(format!(
                "Unbalanced parentheses: {} open vs {} close",
                open_parens, close_parens
            ));
        }
        let open_braces = code.matches('{').count();
        let close_braces = code.matches('}').count();
        if open_braces != close_braces {
            violations.push(format!(
                "Unbalanced braces: {} open vs {} close",
                open_braces, close_braces
            ));
        }
        violations
    }

    pub fn summary(&self) -> String {
        let latest = match self.history.last() {
            Some(l) => l,
            None => return "No generations completed".into(),
        };
        format!(
            "Gen {} | pop={} top_fit={:.4} avg_fit={:.4} diversity={:.4} matches={}",
            self.generation,
            latest.population_size,
            latest.top_fitness,
            latest.avg_fitness,
            latest.diversity,
            latest.matches.len(),
        )
    }

    pub fn full_report(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("=== Adversarial Co-Evolution Report ==="));
        lines.push(format!("Total generations: {}", self.history.len()));
        lines.push(format!("Population size: {}", self.config.population_size));
        lines.push(format!("Tournament size: {}", self.config.tournament_size));
        lines.push(format!(
            "Mutation rate: {:.2}, Crossover rate: {:.2}",
            self.config.mutation_rate, self.config.crossover_rate
        ));
        lines.push(String::new());

        for (i, gen) in self.history.iter().enumerate() {
            let win_rate = if gen.matches.is_empty() {
                0.0
            } else {
                gen.matches.iter().filter(|m| m.agent_won).count() as f64 / gen.matches.len() as f64
            };
            lines.push(format!(
                "Gen {}: pop={} top={:.4} avg={:.4} div={:.4} win_rate={:.2}",
                i, gen.population_size, gen.top_fitness, gen.avg_fitness, gen.diversity, win_rate,
            ));
        }

        if let Some(_last) = self.history.last() {
            lines.push(String::new());
            lines.push("Top agents:".into());
            let mut ranked: Vec<&AgentGenotype> = self.population.iter().collect();
            ranked.sort_by(|a, b| {
                b.fitness
                    .partial_cmp(&a.fitness)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            for (i, agent) in ranked.iter().take(5).enumerate() {
                lines.push(format!(
                    "  {}. {} (fit={:.4}, gen={}, surv={})",
                    i + 1,
                    agent.id,
                    agent.fitness,
                    agent.generation,
                    agent.survived_rounds
                ));
            }
        }

        lines.join("\n")
    }
}

pub fn tournament_pick<'a>(
    population: &'a [AgentGenotype],
    scores: &[f64],
    tournament_size: usize,
    _rng: &mut impl FnMut() -> f64,
) -> &'a AgentGenotype {
    let n = population.len();
    if n == 0 || tournament_size == 0 {
        return &population[0];
    }
    let k = tournament_size.min(n);
    let mut best_idx = rand::thread_rng().gen_range(0..n);
    for _ in 1..k {
        let idx = rand::thread_rng().gen_range(0..n);
        if scores[idx] > scores[best_idx] {
            best_idx = idx;
        }
    }
    &population[best_idx]
}

pub fn fast_rng(seed: u64) -> impl FnMut() -> f64 {
    let mut state = seed;
    move || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let x = (state >> 33) as u32;
        (x as f64) / (u32::MAX as f64)
    }
}
