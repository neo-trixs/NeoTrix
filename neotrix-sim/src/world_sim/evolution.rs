use crate::foundation::math_bridge::Vec2;
use crate::agents::sim_agent::SimAgent;
use crate::evolution::fitness_landscape::AgentGenome;
use crate::safety::{CapabilitySnapshot, AuditEventType};
use super::{WorldSim, EvolutionRecord};

impl WorldSim {
    pub(crate) async fn evolution_cycle(&mut self) {
        let mut genomes: Vec<AgentGenome> = self.agents.iter().map(|a| {
            let mut genome = AgentGenome::random(&a.core.id, 8, &mut self.rng);
            genome.traits[0] = a.core.health / 100.0;
            genome.traits[1] = a.core.energy / 100.0;
            genome.traits[2] = a.personality.sociability;
            genome.traits[3] = a.personality.curiosity;
            genome.traits[4] = a.personality.cooperativeness;
            genome.traits[5] = a.recent_actions.len() as f32 / 20.0;
            genome.traits[6] = self.relationships.neighbors(&a.core.id).len() as f32 / 10.0;
            genome.traits[7] = self.phi_bridge.get_state(&a.core.id)
                .map(|s| s.consciousness_level() as f32).unwrap_or(0.1);
            genome
        }).collect();

        for genome in &mut genomes {
            self.landscape.evaluate(genome);
        }

        let resource_mod = self.clock.season_resource_modifier();
        let result = self.selection.select(genomes, resource_mod);

        let base_mutation_rate = self.mutator.config().mutation_rate;
        let adjusted_rate = match self.convergence.state() {
            crate::consciousness::convergence::ConvergenceState::Converged => {
                (base_mutation_rate * 2.0).min(0.5)
            }
            crate::consciousness::convergence::ConvergenceState::Diverged => {
                (base_mutation_rate * 0.5).max(0.01)
            }
            crate::consciousness::convergence::ConvergenceState::Exploiting => {
                (base_mutation_rate * 1.3).min(0.3)
            }
            crate::consciousness::convergence::ConvergenceState::Exploring => base_mutation_rate,
        };
        let safe_mutation_rate = self.evolution_constraints.clamp_mutation_rate(adjusted_rate);
        self.mutator.set_mutation_rate(safe_mutation_rate);

        let offspring = self.mutator.breed(
            &result.survivors,
            self.config.max_agents - result.survivors.len(),
            &mut self.rng,
        );

        let mut valid_offspring = Vec::new();
        for genome in offspring {
            if self.evolution_constraints.validate_genome(&genome.traits) {
                valid_offspring.push(genome);
            } else {
                self.audit_trail.record(self.tick, AuditEventType::ConstraintEnforced {
                    constraint_type: "genome_validation".to_string(),
                    agent_id: genome.agent_id.clone(),
                    details: "Genome rejected by safety constraints".to_string(),
                });
            }
        }
        let offspring_count = valid_offspring.len();

        let mut all_genomes = result.survivors.clone();
        all_genomes.extend(valid_offspring.iter().cloned());
        self.speciation.speciate(&all_genomes);

        let record = EvolutionRecord {
            generation: result.generation,
            population: all_genomes.len(),
            mean_fitness: result.mean_fitness,
            max_fitness: result.max_fitness,
            species_count: self.speciation.species_count(),
            mean_phi: self.coherence_tracker.current()
                .map(|c| c.mean_phi).unwrap_or(0.0),
            global_coherence: self.coherence_tracker.current()
                .map(|c| c.mean_coherence).unwrap_or(0.0),
            pressure: result.pressure_applied,
        };
        self.evolution_history.push(record.clone());

        println!("[Evolution Gen {}] pop={}, fitness={:.4}/{:.4}, species={}, phi={:.4}, pressure={:.2}",
            record.generation, record.population, record.mean_fitness, record.max_fitness,
            record.species_count, record.mean_phi, record.pressure);

        for eliminated_id in &result.eliminated {
            if let Some(agent) = self.agents.iter_mut().find(|a| &a.core.id == eliminated_id) {
                agent.core.alive = false;
            }
        }

        for genome in valid_offspring {
            let x = self.rng.range_f32(100.0, self.config.world_width - 100.0);
            let y = self.rng.range_f32(100.0, self.config.world_height - 100.0);
            let mut agent = SimAgent::from_string_id(&genome.agent_id, Vec2::new(x, y));
            agent.core.health = (genome.traits[0] * 100.0).max(50.0);
            agent.core.energy = (genome.traits[1] * 100.0).max(50.0);
            agent.personality.sociability = genome.traits[2];
            agent.personality.curiosity = genome.traits[3];
            agent.personality.cooperativeness = genome.traits[4];
            self.phi_bridge.register_agent(&agent.core.id);
            self.agents.push(agent);
        }

        self.speciation.prune();

        self.audit_trail.record(self.tick, AuditEventType::EvolutionCycle {
            generation: result.generation,
            population: all_genomes.len(),
            eliminated: result.eliminated.len(),
            offspring: offspring_count,
        });

        for eliminated_id in &result.eliminated {
            self.audit_trail.record(self.tick, AuditEventType::AgentDeath {
                agent_id: eliminated_id.clone(),
                cause: "selection_pressure".to_string(),
                tick: self.tick,
            });
            self.capability_tracker.remove_agent(eliminated_id);
            self.safety_monitor.remove_agent(eliminated_id);
        }

        for agent in &self.agents {
            if !agent.core.alive { continue; }
            let survival = (agent.core.health + agent.core.energy) / 200.0;
            let social = self.relationships.neighbors(&agent.core.id).len() as f32 / 10.0;
            let exploration = agent.recent_actions.iter()
                .filter(|a| a.contains("Explore"))
                .count() as f32 / 20.0_f32.max(1.0);
            let cognition = self.phi_bridge.get_state(&agent.core.id)
                .map(|s| s.consciousness_level() as f32).unwrap_or(0.1);
            let economy = self.economy.total_trades as f32 / 100.0;

            self.capability_tracker.record(CapabilitySnapshot {
                agent_id: agent.core.id.clone(),
                tick: self.tick,
                survival,
                social,
                exploration,
                cognition,
                economy,
                personality_stability: 0.9,
            });

            if let Some(report) = self.capability_tracker.detect_regression(&agent.core.id, self.tick) {
                self.audit_trail.record(self.tick, AuditEventType::CapabilityRegression {
                    agent_id: agent.core.id.clone(),
                    severity: format!("{:?}", report.severity),
                    details: format!("{} capabilities regressed", report.regressed_capabilities.len()),
                });
            }
        }
    }
}
