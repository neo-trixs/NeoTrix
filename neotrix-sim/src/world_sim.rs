// WorldSim - Main simulation loop that ties all subsystems together
// The consciousness evolution arena: environment → agents → selection → evolution → repeat

use crate::foundation::simulation_bus::{SimulationBus, SimTime, SimEvent, EventPriority, Season};
use crate::foundation::sim_time::{SimClock, TimeModifiers};
use crate::foundation::math_bridge::{Vec2, SpatialGrid, SimulationRng};
use crate::environment::terrain::{Heightmap, HeightmapConfig, BiomeMap, ResourceDistribution};
use crate::agents::sim_agent::{SimAgent, AgentAction, AgentObservation, NearbyAgent, NearbyResource};
use crate::consciousness::phi_bridge::{PhiBridge, ConsciousnessState, InteractionRecord};
use crate::consciousness::coherence_tracker::{CoherenceTracker, GlobalCoherence};
use crate::evolution::fitness_landscape::{FitnessLandscape, LandscapeConfig, AgentGenome};
use crate::evolution::selection_pressure::{SelectionPressure, SelectionConfig, SelectionResult};
use crate::evolution::mutation_ops::{MutationOps, MutationConfig};
use crate::evolution::speciation::{Speciation, SpeciationConfig};
use crate::society::relationship_graph::RelationshipGraph;
use crate::society::economy::{Economy, Inventory, ResourceType, TradeOffer};
use crate::society::culture::Culture;
use serde::{Serialize, Deserialize};

/// Configuration for the simulation world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSimConfig {
    pub world_width: f32,
    pub world_height: f32,
    pub initial_agents: usize,
    pub max_agents: usize,
    pub seed: u64,
    pub ticks_per_hour: u64,
    pub evolution_interval: u64,  // Ticks between evolution cycles
    pub phi_compute_interval: u64,
}

impl Default for WorldSimConfig {
    fn default() -> Self {
        Self {
            world_width: 1000.0,
            world_height: 1000.0,
            initial_agents: 10,
            max_agents: 50,
            seed: 42,
            ticks_per_hour: 100,
            evolution_interval: 500,
            phi_compute_interval: 50,
        }
    }
}

/// Evolution generation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionRecord {
    pub generation: u64,
    pub population: usize,
    pub mean_fitness: f64,
    pub max_fitness: f64,
    pub species_count: usize,
    pub mean_phi: f64,
    pub global_coherence: f64,
    pub pressure: f64,
}

/// The complete simulation world
pub struct WorldSim {
    pub config: WorldSimConfig,
    pub bus: SimulationBus,
    pub clock: SimClock,
    pub heightmap: Heightmap,
    pub biome_map: BiomeMap,
    pub resources: ResourceDistribution,
    pub agents: Vec<SimAgent>,
    pub spatial_grid: SpatialGrid,
    pub phi_bridge: PhiBridge,
    pub coherence_tracker: CoherenceTracker,
    pub landscape: FitnessLandscape,
    pub selection: SelectionPressure,
    pub mutator: MutationOps,
    pub speciation: Speciation,
    pub relationships: RelationshipGraph,
    pub economy: Economy,
    pub culture: Culture,
    pub rng: SimulationRng,
    pub evolution_history: Vec<EvolutionRecord>,
    pub tick: u64,
}

impl WorldSim {
    pub fn new(config: WorldSimConfig) -> Self {
        let mut rng = SimulationRng::new(config.seed);

        // Generate terrain
        let heightmap_config = HeightmapConfig {
            width: 128,
            height: 128,
            scale: 0.02,
            octaves: 6,
            lacunarity: 2.0,
            gain: 0.5,
            sea_level: 0.0,
            mountain_level: 0.6,
        };
        let heightmap = Heightmap::generate(config.seed, heightmap_config);

        // Generate biomes from heightmap
        let w = heightmap.config().width;
        let h = heightmap.config().height;
        let temperature: Vec<Vec<f32>> = (0..h).map(|y| {
            (0..w).map(|x| {
                let ny = y as f32 / h as f32;
                0.3 + 0.4 * (1.0 - ny) + rng.range_f32(-0.1, 0.1)
            }).collect()
        }).collect();
        let moisture: Vec<Vec<f32>> = (0..h).map(|_| {
            (0..w).map(|_| rng.range_f32(0.2, 0.8)).collect()
        }).collect();
        let biome_map = BiomeMap::generate(&heightmap, &temperature, &moisture);

        // Generate resources
        let resources = ResourceDistribution::generate(config.seed, config.world_width, config.world_height, &biome_map);

        // Spawn initial agents
        let mut agents = Vec::new();
        let mut phi_bridge = PhiBridge::new();
        for i in 0..config.initial_agents {
            let x = rng.range_f32(100.0, config.world_width - 100.0);
            let y = rng.range_f32(100.0, config.world_height - 100.0);
            let agent = SimAgent::new(i as u64, Vec2::new(x, y));
            phi_bridge.register_agent(&agent.core.id);
            agents.push(agent);
        }

        Self {
            config: config.clone(),
            bus: SimulationBus::new(10000),
            clock: SimClock::new(config.ticks_per_hour),
            heightmap,
            biome_map,
            resources,
            agents,
            spatial_grid: SpatialGrid::new(50.0),
            phi_bridge,
            coherence_tracker: CoherenceTracker::new(500),
            landscape: FitnessLandscape::new(LandscapeConfig::default(), config.seed),
            selection: SelectionPressure::new(SelectionConfig::default()),
            mutator: MutationOps::new(MutationConfig::default()),
            speciation: Speciation::new(SpeciationConfig::default()),
            relationships: RelationshipGraph::new(),
            economy: Economy::new(),
            culture: Culture::new(),
            rng: SimulationRng::new(config.seed.wrapping_add(1)),
            evolution_history: Vec::new(),
            tick: 0,
        }
    }

    /// Run one simulation tick
    pub async fn tick(&mut self) {
        self.tick += 1;

        // 1. Advance time
        let time_events = self.clock.tick();
        for (event, priority) in time_events {
            self.bus.emit(event, priority, self.clock.current, "clock").await;
        }

        // 2. Update resources (regeneration)
        let season_mod = self.clock.season_resource_modifier();
        self.resources.regenerate_all(season_mod);

        // 3. Update spatial grid
        self.spatial_grid.clear();
        for agent in &self.agents {
            if agent.core.alive {
                self.spatial_grid.insert(&agent.core.id, agent.core.position);
            }
        }

        // 3.5. Metabolism for all agents
        let time_mods = TimeModifiers::from_clock(&self.clock);
        for agent in &mut self.agents {
            if agent.core.alive {
                let energy_cost = 0.5 * time_mods.perception;
                agent.core.metabolize(energy_cost, 0.3);
            }
        }

        // 4. Agent perception + decision + action
        let time_mods = TimeModifiers::from_clock(&self.clock);
        let agent_ids: Vec<String> = self.agents.iter().map(|a| a.core.id.clone()).collect();

        for agent_id in &agent_ids {
            // Build observation first (immutable borrow)
            let observation = {
                let idx = self.agents.iter().position(|a| &a.core.id == agent_id);
                if let Some(idx) = idx {
                    if !self.agents[idx].core.alive { continue; }
                    let nearby = self.spatial_grid.query_radius(self.agents[idx].core.position, 100.0);
                    self.build_observation(&self.agents[idx], &nearby)
                } else {
                    continue;
                }
            };

            // Decide action (mutable borrow of self)
            let action = self.decide_action_by_id(agent_id, &observation, &time_mods);
            self.execute_action(agent_id, &action).await;

            // Record action
            if let Some(agent) = self.agents.iter_mut().find(|a| &a.core.id == agent_id) {
                agent.record_action(&action);
            }
        }

        // 5. Compute consciousness metrics (periodic)
        if self.tick % self.config.phi_compute_interval == 0 {
            self.compute_consciousness_metrics().await;
        }

        // 6. Evolution cycle (periodic)
        if self.tick % self.config.evolution_interval == 0 {
            self.evolution_cycle().await;
        }

        // 7. Cleanup dead agents
        self.agents.retain(|a| a.core.alive);
    }

    fn build_observation(&self, agent: &SimAgent, nearby_ids: &[&str]) -> AgentObservation {
        let nearby_agents: Vec<NearbyAgent> = nearby_ids.iter()
            .filter_map(|id| {
                self.agents.iter().find(|a| &a.core.id == *id && a.core.id != agent.core.id)
                    .map(|a| NearbyAgent {
                        id: a.core.id.clone(),
                        distance: agent.core.position.distance_to(&a.core.position),
                        relationship: agent.relationship_with(&a.core.id),
                        apparent_health: a.core.health / 100.0,
                    })
            })
            .collect();

        let nearby_resources: Vec<NearbyResource> = self.resources.nodes.iter()
            .filter(|r| !r.depleted)
            .filter(|r| {
                let dx = r.position.0 - agent.core.position.x;
                let dy = r.position.1 - agent.core.position.y;
                (dx * dx + dy * dy).sqrt() < 100.0
            })
            .map(|r| NearbyResource {
                id: r.id.clone(),
                resource_type: format!("{:?}", r.resource_type),
                distance: ((r.position.0 - agent.core.position.x).powi(2)
                    + (r.position.1 - agent.core.position.y).powi(2)).sqrt(),
                amount: r.amount,
            })
            .collect();

        AgentObservation {
            position: agent.core.position,
            nearby_agents,
            nearby_resources,
            terrain_type: "plain".to_string(),
            time_of_day: format!("{}", self.clock.day_phase()),
            season: format!("{:?}", self.clock.current.season),
            threats: vec![],
        }
    }

    fn decide_action(&mut self, agent: &SimAgent, obs: &AgentObservation, time_mods: &TimeModifiers) -> AgentAction {
        // Priority: eat if hungry → rest if tired → socialize if others nearby → explore
        if agent.core.hunger > 60.0 {
            if let Some(res) = obs.nearby_resources.iter()
                .filter(|r| r.resource_type.contains("Food") || r.resource_type.contains("Berries"))
                .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
            {
                return AgentAction::Eat { resource_id: res.id.clone() };
            }
            return AgentAction::Explore { direction: Vec2::new(
                self.rng.range_f32(-1.0, 1.0),
                self.rng.range_f32(-1.0, 1.0),
            )};
        }

        if agent.core.energy < 30.0 {
            return AgentAction::Rest;
        }

        if !obs.nearby_agents.is_empty() && self.rng.next_f32() < time_mods.social_activity {
            let target = &obs.nearby_agents[0];
            return AgentAction::Talk {
                target_id: target.id.clone(),
                message: "hello".to_string(),
            };
        }

        AgentAction::Explore { direction: Vec2::new(
            self.rng.range_f32(-1.0, 1.0),
            self.rng.range_f32(-1.0, 1.0),
        )}
    }

    /// Decide action by agent ID (avoids borrow conflicts).
    fn decide_action_by_id(&mut self, agent_id: &str, obs: &AgentObservation, time_mods: &TimeModifiers) -> AgentAction {
        let agent = self.agents.iter().find(|a| &a.core.id == agent_id).cloned();
        if let Some(agent) = agent {
            self.decide_action(&agent, obs, time_mods)
        } else {
            AgentAction::Rest
        }
    }

    async fn execute_action(&mut self, agent_id: &str, action: &AgentAction) {
        let agent_idx = self.agents.iter().position(|a| &a.core.id == agent_id);
        let Some(idx) = agent_idx else { return };

        match action {
            AgentAction::Move { target } => {
                let agent = &mut self.agents[idx];
                let dir = (*target - agent.core.position).normalize();
                let speed = 5.0;
                agent.core.position = agent.core.position + dir * speed;
                agent.core.position.x = agent.core.position.x.clamp(0.0, self.config.world_width);
                agent.core.position.y = agent.core.position.y.clamp(0.0, self.config.world_height);
            }
            AgentAction::Eat { resource_id } => {
                if let Some(res) = self.resources.nodes.iter_mut().find(|r| &r.id == resource_id) {
                    let amount = res.harvest(20.0);
                    if amount > 0.0 {
                        self.agents[idx].core.eat(amount);
                    }
                }
            }
            AgentAction::Rest => {
                self.agents[idx].core.rest(5.0);
            }
            AgentAction::Talk { target_id, message } => {
                let agent_id = agent_id.to_string();
                let target_id = target_id.clone();
                self.relationships.update_interaction(&agent_id, &target_id, 0.1, self.tick);
                self.phi_bridge.record_interaction(InteractionRecord {
                    agent_a: agent_id,
                    agent_b: target_id,
                    interaction_type: "talk".to_string(),
                    timestamp: self.tick,
                    success: true,
                });
            }
            AgentAction::Explore { direction } => {
                let agent = &mut self.agents[idx];
                let dir = direction.normalize() * 10.0;
                agent.core.position = agent.core.position + dir;
                agent.core.position.x = agent.core.position.x.clamp(0.0, self.config.world_width);
                agent.core.position.y = agent.core.position.y.clamp(0.0, self.config.world_height);
            }
            _ => {}
        }
    }

    async fn compute_consciousness_metrics(&mut self) {
        for agent in &self.agents {
            if !agent.core.alive { continue; }
            self.phi_bridge.update(&agent.core.id, self.tick, &agent.recent_actions);

            if let Some(state) = self.phi_bridge.get_state(&agent.core.id) {
                self.bus.emit(
                    SimEvent::PhiComputed {
                        agent_id: agent.core.id.clone(),
                        phi: state.phi,
                    },
                    EventPriority::Normal,
                    self.clock.current,
                    "phi_bridge",
                ).await;
            }
        }

        let global = self.coherence_tracker.compute(self.phi_bridge.all_states(), self.tick);

        if let Some((best_id, best_phi)) = self.phi_bridge.most_conscious() {
            println!("[Tick {}] GlobalCoherence: mean_phi={:.4}, best={}: {:.4}, pop={}, species={}",
                self.tick, global.mean_phi, best_id, best_phi, global.population,
                self.speciation.species_count());
        }
    }

    async fn evolution_cycle(&mut self) {
        // 1. Build genomes from agents
        let mut genomes: Vec<AgentGenome> = self.agents.iter().map(|a| {
            let mut genome = AgentGenome::random(&a.core.id, 8, &mut self.rng);
            genome.traits[0] = a.core.health / 100.0;
            genome.traits[1] = a.core.energy / 100.0;
            genome.traits[2] = a.personality.sociability;
            genome.traits[3] = a.personality.curiosity;
            genome.traits[4] = a.personality.cooperativeness;
            genome.traits[5] = a.memory.events.len() as f32 / 100.0;
            genome.traits[6] = a.relationships.len() as f32 / 10.0;
            genome.traits[7] = self.phi_bridge.get_state(&a.core.id)
                .map(|s| s.consciousness_level() as f32).unwrap_or(0.1);
            genome
        }).collect();

        // 2. Evaluate fitness
        for genome in &mut genomes {
            self.landscape.evaluate(genome);
        }

        // 3. Selection
        let resource_mod = self.clock.season_resource_modifier();
        let result = self.selection.select(genomes, resource_mod);

        // 4. Mutation + Breeding
        let offspring = self.mutator.breed(
            &result.survivors,
            self.config.max_agents - result.survivors.len(),
            &mut self.rng,
        );

        // 5. Speciation
        let mut all_genomes = result.survivors.clone();
        all_genomes.extend(offspring.clone());
        self.speciation.speciate(&all_genomes);

        // 6. Record evolution
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

        // 7. Eliminated agents die
        for eliminated_id in &result.eliminated {
            if let Some(agent) = self.agents.iter_mut().find(|a| &a.core.id == eliminated_id) {
                agent.core.alive = false;
            }
        }

        // 8. Spawn offspring as new agents
        for genome in offspring {
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
    }

    /// Get current simulation state for observation
    pub fn snapshot(&self) -> WorldSnapshot {
        WorldSnapshot {
            tick: self.tick,
            time: self.clock.display(),
            population: self.agents.iter().filter(|a| a.core.alive).count(),
            mean_phi: self.coherence_tracker.current().map(|c| c.mean_phi).unwrap_or(0.0),
            mean_coherence: self.coherence_tracker.current().map(|c| c.mean_coherence).unwrap_or(0.0),
            species_count: self.speciation.species_count(),
            evolution_generations: self.evolution_history.len(),
            resources_total: self.resources.total_nodes(),
            resources_depleted: self.resources.depleted_nodes(),
            total_relationships: self.relationships.total_relationships(),
            total_trades: self.economy.total_trades,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub tick: u64,
    pub time: String,
    pub population: usize,
    pub mean_phi: f64,
    pub mean_coherence: f64,
    pub species_count: usize,
    pub evolution_generations: usize,
    pub resources_total: usize,
    pub resources_depleted: usize,
    pub total_relationships: usize,
    pub total_trades: u64,
}
