// WorldSim - Main simulation loop that ties all subsystems together
// The consciousness evolution arena: environment → agents → selection → evolution → repeat

use crate::foundation::simulation_bus::{SimulationBus, SimEvent, EventPriority};
use crate::foundation::sim_time::{SimClock, TimeModifiers};
use crate::foundation::math_bridge::{Vec2, SpatialGrid, SimulationRng};
use crate::foundation::tick_schedule::{TickSchedule, TickTier};
use crate::environment::terrain::{Heightmap, HeightmapConfig, BiomeMap, ResourceDistribution};
use crate::environment::structures::{StructureManager, StructureType};
use crate::agents::sim_agent::{SimAgent, AgentAction, AgentObservation, NearbyAgent, NearbyResource, Threat};
use crate::agents::action_awareness::ActionAwareness;
use crate::agents::memory_stream::MemoryStream;
use crate::agents::graph_memory::GraphMemory;
use crate::agents::spatial_memory::SpatialMemory;
use crate::agents::planning::PlanningStack;
use crate::agents::reflection::ReflectionEngine;
use crate::agents::action_costs::{ActionCostTable, ActionBudget};
use crate::agents::personality_drift::PersonalityDrift;
use std::collections::HashMap;
use crate::consciousness::phi_bridge::{PhiBridge, InteractionRecord};
use crate::consciousness::coherence_tracker::CoherenceTracker;
use crate::consciousness::convergence::ConvergenceDetector;
use crate::consciousness::dual_representation::DualRepresentation;
use crate::evolution::fitness_landscape::{FitnessLandscape, LandscapeConfig, AgentGenome};
use crate::evolution::selection_pressure::{SelectionPressure, SelectionConfig};
use crate::evolution::mutation_ops::{MutationOps, MutationConfig};
use crate::evolution::speciation::{Speciation, SpeciationConfig};
use crate::feel::{EmotionEngine, SystemEvent, EmotionType};
use crate::society::relationship_graph::RelationshipGraph;
use crate::society::economy::Economy;
use crate::society::culture::Culture;
use crate::society::theory_of_mind::TheoryOfMind;
use crate::society::constitutional::ConstitutionalFeedback;
use crate::agents::event_reactive::EventReactiveSystem;
use crate::agents::goal_outcome_feedback::GoalOutcomeFeedback;
use crate::agents::intention_commitment::IntentionCommitment;
use crate::agents::thought_generation::ThoughtGeneration;
use crate::agents::social_learning::SocialLearning;
use crate::agents::pheromone::{PheromoneField, PheromoneType, PheromoneSignal};
use crate::safety::{
    CapabilityTracker, CapabilitySnapshot, CapabilityConfig,
    SafetyMonitor, SafetyMonitorConfig,
    EvolutionConstraints, EvolutionConstraintConfig,
    AuditTrail, AuditEventType,
};
use serde::{Serialize, Deserialize};

/// Configuration for the simulation world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSimConfig {
    pub world_width: f32,
    pub world_height: f32,
    pub initial_agents: usize,
    pub min_agents: usize,
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
            min_agents: 5,
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
    pub schedule: TickSchedule,
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
    pub emotion: EmotionEngine,
    pub evolution_history: Vec<EvolutionRecord>,
    pub action_awareness: HashMap<String, ActionAwareness>,
    pub structures: StructureManager,
    pub tick: u64,
    // Cross-domain integration: new subsystems
    pub memory_streams: HashMap<String, MemoryStream>,
    pub graph_memories: HashMap<String, GraphMemory>,
    pub spatial_memories: HashMap<String, SpatialMemory>,
    pub planning: HashMap<String, PlanningStack>,
    pub reflections: HashMap<String, ReflectionEngine>,
    pub action_costs: ActionCostTable,
    pub action_budgets: HashMap<String, ActionBudget>,
    pub personality_drift: HashMap<String, PersonalityDrift>,
    pub theory_of_mind: HashMap<String, TheoryOfMind>,
    pub constitutional: ConstitutionalFeedback,
    pub convergence: ConvergenceDetector,
    pub dual_repr: DualRepresentation,
    pub event_reactive: EventReactiveSystem,
    // Fusion Adapters
    pub goal_outcome_feedback: GoalOutcomeFeedback,
    pub intention_commitment: IntentionCommitment,
    pub thought_generation: ThoughtGeneration,
    pub social_learning: SocialLearning,
    // Stigmergy: shared pheromone field for indirect coordination
    pub pheromone_field: PheromoneField,
    // Safety Guardrails (NT-SHIELD)
    pub capability_tracker: CapabilityTracker,
    pub safety_monitor: SafetyMonitor,
    pub evolution_constraints: EvolutionConstraints,
    pub audit_trail: AuditTrail,
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
            (0..w).map(|_x| {
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
            schedule: TickSchedule::new(),
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
            emotion: EmotionEngine::new(),
            evolution_history: Vec::new(),
            action_awareness: HashMap::new(),
            structures: StructureManager::new(50.0),
            tick: 0,
            memory_streams: HashMap::new(),
            graph_memories: HashMap::new(),
            spatial_memories: HashMap::new(),
            planning: HashMap::new(),
            reflections: HashMap::new(),
            action_costs: ActionCostTable::new(),
            action_budgets: HashMap::new(),
            personality_drift: HashMap::new(),
            theory_of_mind: HashMap::new(),
            constitutional: ConstitutionalFeedback::new(),
            convergence: ConvergenceDetector::default_new(),
            dual_repr: DualRepresentation::new(16),
            event_reactive: EventReactiveSystem::new(),
            goal_outcome_feedback: GoalOutcomeFeedback::new(),
            intention_commitment: IntentionCommitment::new(),
            thought_generation: ThoughtGeneration::new(),
            social_learning: SocialLearning::new(),
            pheromone_field: PheromoneField::new(50.0, 5000),
            capability_tracker: CapabilityTracker::new(CapabilityConfig::default()),
            safety_monitor: SafetyMonitor::new(SafetyMonitorConfig::default()),
            evolution_constraints: EvolutionConstraints::new(EvolutionConstraintConfig::default()),
            audit_trail: AuditTrail::new(10000),
        }
    }

    /// Run one simulation tick using multi-timescale schedule
    pub async fn tick(&mut self) {
        self.schedule.advance();
        let tick = self.schedule.current_tick();
        self.tick = tick;

        // Reflex tier — every tick
        if self.schedule.should_run(TickTier::Reflex) {
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

            // 4. Metabolism for all agents
            let time_mods = TimeModifiers::from_clock(&self.clock);
            for agent in &mut self.agents {
                if agent.core.alive {
                    let energy_cost = 0.5 * time_mods.perception;
                    agent.core.metabolize(energy_cost, 0.3);
                }
            }
        }

        // Fast tier — every 5 ticks
        if self.schedule.should_run(TickTier::Fast) {
            // 5. Agent perception + decision + action
            let time_mods = TimeModifiers::from_clock(&self.clock);
            let agent_ids: Vec<String> = self.agents.iter().map(|a| a.core.id.clone()).collect();

            for agent_id in &agent_ids {
                // Initialize per-agent subsystems on first encounter
                let pos = self.agents.iter().find(|a| &a.core.id == agent_id).map(|a| [a.core.position.x, a.core.position.y]).unwrap_or([0.0, 0.0]);
                self.memory_streams.entry(agent_id.clone()).or_insert_with(|| MemoryStream::new(200));
                self.graph_memories.entry(agent_id.clone()).or_insert_with(|| GraphMemory::new(500));
                self.spatial_memories.entry(agent_id.clone()).or_insert_with(|| SpatialMemory::new(pos));
                self.planning.entry(agent_id.clone()).or_insert_with(PlanningStack::new);
                self.reflections.entry(agent_id.clone()).or_insert_with(ReflectionEngine::new);
                self.action_budgets.entry(agent_id.clone()).or_insert_with(ActionBudget::new);
                self.personality_drift.entry(agent_id.clone()).or_insert_with(|| PersonalityDrift::new(Default::default()));
                self.theory_of_mind.entry(agent_id.clone()).or_insert_with(|| TheoryOfMind::new(50));

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

                // Constitutional check (M10: type-safe)
                let (compliance, _violated) = self.constitutional.evaluate_action(&action, tick);

                // Action cost check
                let can_afford = {
                    let agent = self.agents.iter().find(|a| &a.core.id == agent_id);
                    agent.map(|a| self.action_costs.can_afford(&action, a.core.energy, a.core.health))
                        .unwrap_or(false)
                };

                if !can_afford || compliance < 0.3 {
                    // Can't afford or unconstitutional — rest instead
                    self.execute_action(agent_id, &AgentAction::Rest).await;
                    continue;
                }

                // Action Awareness: predict before execution
                {
                    let agent = self.agents.iter().find(|a| &a.core.id == agent_id).cloned();
                    if let Some(agent) = agent {
                        self.action_awareness
                            .entry(agent_id.clone())
                            .or_insert_with(ActionAwareness::new)
                            .predict(&action, &agent);
                    }
                }

                // Record budget
                if let Some(budget) = self.action_budgets.get_mut(agent_id) {
                    budget.record_action(&action, &self.action_costs);
                }

                self.execute_action(agent_id, &action).await;

                // Safety Monitor: record action for anomaly detection
                {
                    let action_str = format!("{:?}", action);
                    self.safety_monitor.record_action(agent_id, &action_str, tick);
                    // Run safety checks
                    let alerts = self.safety_monitor.check_all(agent_id, tick);
                    for alert in &alerts {
                        self.audit_trail.record(tick, AuditEventType::SafetyViolation {
                            agent_id: agent_id.clone(),
                            violation_type: format!("{:?}", alert.violation),
                            severity: alert.severity,
                        });
                    }
                    // Personality drift check
                    if let Some(agent) = self.agents.iter().find(|a| &a.core.id == agent_id) {
                        if let Some(alert) = self.safety_monitor.check_personality_drift(
                            agent_id,
                            agent.personality.openness,
                            agent.personality.sociability,
                            agent.personality.aggression,
                            agent.personality.cooperativeness,
                            agent.personality.curiosity,
                            tick,
                        ) {
                            self.audit_trail.record(tick, AuditEventType::SafetyViolation {
                                agent_id: agent_id.clone(),
                                violation_type: format!("{:?}", alert.violation),
                                severity: alert.severity,
                            });
                        }
                        // Update personality snapshot for next drift check
                        self.safety_monitor.record_personality(
                            agent_id,
                            agent.personality.openness,
                            agent.personality.sociability,
                            agent.personality.aggression,
                            agent.personality.cooperativeness,
                            agent.personality.curiosity,
                            tick,
                        );
                    }
                }

                // Action Awareness: verify after execution
                if let Some(agent) = self.agents.iter().find(|a| &a.core.id == agent_id) {
                    if let Some(awareness) = self.action_awareness.get_mut(agent_id) {
                        awareness.verify(agent, tick);
                    }
                }

                // Record action in memory streams
                if let Some(ms) = self.memory_streams.get_mut(agent_id) {
                    ms.add(crate::agents::memory_stream::MemoryNode {
                        id: 0,
                        kind: crate::agents::memory_stream::MemoryKind::Observation,
                        agent_id: agent_id.clone(),
                        created_tick: tick,
                        last_accessed_tick: tick,
                        description: format!("{:?}", action),
                        importance: compliance as f32,
                        keywords: vec![],
                        citations: vec![],
                        embedding: None,
                    });
                }

                // M5: GraphMemory — record action as node + temporal edge
                if let Some(gm) = self.graph_memories.get_mut(agent_id) {
                    let node_id = gm.add_node(
                        crate::agents::graph_memory::NodeKind::Event,
                        &format!("{:?}", action),
                        tick,
                        compliance as f32,
                    );
                    // Connect to previous action if exists
                    if let Some(prev_id) = gm.last_node_id() {
                        if prev_id != node_id {
                            gm.add_edge(prev_id, node_id, crate::agents::graph_memory::EdgeKind::Temporal, 0.8, tick);
                        }
                    }
                    // Social edges for Talk/Trade
                    match action {
                        AgentAction::Talk { ref target_id, .. } | AgentAction::Trade { ref target_id, .. } => {
                            let person_id = gm.add_node(
                                crate::agents::graph_memory::NodeKind::Person,
                                target_id,
                                tick,
                                0.5,
                            );
                            gm.add_edge(node_id, person_id, crate::agents::graph_memory::EdgeKind::Social, 0.7, tick);
                        }
                        _ => {}
                    }
                }

                // M6: SpatialMemory — record visit at current position
                if let Some(sm) = self.spatial_memories.get_mut(agent_id) {
                    if let Some(agent) = self.agents.iter().find(|a| &a.core.id == agent_id) {
                        let biome = "plain".to_string();
                        let danger = if compliance < 0.3 { 0.5 } else { 0.1 };
                        sm.visit(
                            [agent.core.position.x, agent.core.position.y],
                            &biome,
                            vec![],
                            danger,
                            tick,
                        );
                    }
                }

                // Record action
                if let Some(agent) = self.agents.iter_mut().find(|a| &a.core.id == agent_id) {
                    agent.record_action(&action);
                }

                // Theory of Mind: observe other agents
                if let Some(tom) = self.theory_of_mind.get_mut(agent_id) {
                    for obs_agent in &observation.nearby_agents {
                        let positive = obs_agent.relationship > 0.0;
                        tom.observe_interaction(&obs_agent.id, true, positive, tick);
                    }
                }

                // Personality drift: record experience
                if let Some(pd) = self.personality_drift.get_mut(agent_id) {
                    let signal = crate::agents::personality_drift::ExperienceSignal {
                        social_success: observation.nearby_agents.len() as f32 * 0.1,
                        exploration_reward: if matches!(action, AgentAction::Explore { .. }) { 0.2 } else { 0.0 },
                        survival_stress: if compliance < 0.5 { 0.3 } else { 0.0 },
                        achievement: if compliance > 0.8 { 0.1 } else { 0.0 },
                        novelty_exposure: 0.1,
                    };
                    pd.record(signal);
                }
            }
        }

        // Reflex tier — every tick: process reactive events
        {
            let records = {
                let hist = self.bus.history();
                let hist = hist.read().await;
                hist.recent(100).to_vec()
            };
            let events: Vec<SimEvent> = records.iter().map(|r| r.event.clone()).collect();
            let responses = self.event_reactive.process_events(&events, tick);
            for resp in responses {
                if let Some(agent) = self.agents.iter_mut().find(|a| a.core.id == resp.agent_id) {
                    // Execute reactive action directly
                    match &resp.action {
                        AgentAction::Explore { direction } => {
                            let speed = 2.0;
                            agent.core.position.x += direction.x * speed;
                            agent.core.position.y += direction.y * speed;
                        }
                        AgentAction::Rest => {
                            agent.core.rest(5.0);
                        }
                        AgentAction::Talk { target_id, message: _ } => {
                            self.relationships.update_interaction(&resp.agent_id, target_id, 0.1, tick);
                        }
                        _ => {}
                    }
                }
            }
        }

        // Medium tier — every 20 ticks
        if self.schedule.should_run(TickTier::Medium) {
            // 6. Consciousness metrics (phi, coherence)
            self.compute_consciousness_metrics().await;
        }

        // Slow tier — every 100 ticks
        if self.schedule.should_run(TickTier::Slow) {
            // 7. Emotion update from world events
            {
                let events = self.collect_world_events();
                self.emotion.process_events(&events);
            }

            // 8. Action Awareness: learn periodically
            for awareness in self.action_awareness.values_mut() {
                awareness.learn();
            }

            // 9. Planning: regenerate goals from current state
            let agent_ids: Vec<String> = self.agents.iter().map(|a| a.core.id.clone()).collect();
            for agent_id in &agent_ids {
                if let Some(agent) = self.agents.iter().find(|a| &a.core.id == agent_id) {
                    let nearby_count = self.spatial_grid.query_radius(agent.core.position, 100.0).len();
                    let has_rels = self.relationships.neighbors(agent_id).len() > 0;
                    if let Some(planning) = self.planning.get_mut(agent_id) {
                        planning.generate_survival_goals(agent.core.hunger, agent.core.energy, agent.core.health, self.tick);
                        planning.generate_social_goals(nearby_count, has_rels, self.tick);
                        planning.generate_exploration_goals(self.tick);
                        planning.consolidate();
                    }
                }
            }

            // 10. Reflection: trigger when importance accumulates
            for agent_id in &agent_ids {
                let recent_importance = self.memory_streams.get(agent_id)
                    .map(|ms| ms.recent_importance_sum(20))
                    .unwrap_or(0.0);
                if let Some(reflection) = self.reflections.get_mut(agent_id) {
                    if reflection.on_new_memory(recent_importance) {
                        let ms = self.memory_streams.get(agent_id).unwrap();
                        let insights = reflection.reflect(ms, agent_id, self.tick);
                        if let Some(ms) = self.memory_streams.get_mut(agent_id) {
                            for insight in insights {
                                ms.add(insight);
                            }
                        }
                    }
                }
            }

            // 11. Personality drift: apply accumulated drift
            for agent in &mut self.agents {
                if let Some(pd) = self.personality_drift.get_mut(&agent.core.id) {
                    let new_p = pd.drift(&agent.personality, agent.core.age);
                    agent.personality = new_p;
                }
            }

            // 12. Constitutional decay
            self.constitutional.decay();

            // 13. Convergence detection
            let alive_count = self.agents.iter().filter(|a| a.core.alive).count();
            let mean_fitness = self.agents.iter().filter(|a| a.core.alive).map(|a| a.core.health as f64).sum::<f64>()
                / alive_count.max(1) as f64;
            self.convergence.record(mean_fitness, tick);

            // 14. Stigmergy: decay pheromones, prune excess, emit deposit events
            self.pheromone_field.decay(tick);
            self.pheromone_field.prune(tick);
            let deposits = self.pheromone_field.pending_deposits_drain();
            for p in deposits {
                self.bus.emit(
                    SimEvent::PheromoneDeposited {
                        agent_id: p.deposited_by,
                        ptype: format!("{:?}", p.ptype),
                        position: (p.position[0], p.position[1]),
                        strength: p.strength as f64,
                    },
                    EventPriority::Low,
                    self.clock.current,
                    "pheromone_field",
                ).await;
            }
        }

        // Background tier — every 500 ticks
        if self.schedule.should_run(TickTier::Background) {
            // 9. Evolution cycle
            self.evolution_cycle().await;
        }

        // Cleanup — every tick
        self.agents.retain(|a| a.core.alive);
    }

    fn build_observation(&self, agent: &SimAgent, nearby_ids: &[&str]) -> AgentObservation {
        let nearby_agents: Vec<NearbyAgent> = nearby_ids.iter()
            .filter_map(|id| {
                self.agents.iter().find(|a| &a.core.id == *id && a.core.id != agent.core.id)
                    .map(|a| NearbyAgent {
                        id: a.core.id.clone(),
                        distance: agent.core.position.distance_to(&a.core.position),
                        relationship: self.relationships.sentiment_between(&agent.core.id, &a.core.id),
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

        // Query biome_map for actual terrain type at agent position
        let biome = self.biome_map.biome_at(
            agent.core.position.x,
            agent.core.position.y,
            self.config.world_width,
            self.config.world_height,
        );
        let terrain_type = format!("{:?}", biome);

        // Query heightmap and emit mountain threat if elevated
        let height = self.heightmap.height_at(agent.core.position.x, agent.core.position.y);
        let mut threats: Vec<Threat> = vec![];
        if self.heightmap.is_mountain(agent.core.position.x, agent.core.position.y) {
            threats.push(Threat {
                threat_type: "mountain".to_string(),
                position: agent.core.position,
                severity: (height - self.heightmap.config().mountain_level) / (1.0 - self.heightmap.config().mountain_level).max(0.01),
            });
        }
        if self.heightmap.is_water(agent.core.position.x, agent.core.position.y) {
            threats.push(Threat {
                threat_type: "water".to_string(),
                position: agent.core.position,
                severity: 0.3,
            });
        }

        AgentObservation {
            position: agent.core.position,
            nearby_agents,
            nearby_resources,
            terrain_type,
            time_of_day: format!("{}", self.clock.day_phase()),
            season: format!("{:?}", self.clock.current.season),
            threats,
        }
    }

    /// Unified decision pipeline: evaluate layers in priority order, first non-None wins.
    fn decide_action(&mut self, agent: &SimAgent, obs: &AgentObservation, time_mods: &TimeModifiers) -> AgentAction {
        if let Some(action) = self.layer_survival(agent, obs) {
            return action;
        }
        if let Some(action) = self.layer_goals(agent, obs) {
            return action;
        }
        if let Some(action) = self.layer_social(agent, obs, time_mods) {
            return action;
        }
        if let Some(action) = self.layer_stigmergy(agent, obs) {
            return action;
        }
        if let Some(action) = self.layer_personality(agent, obs, time_mods) {
            return action;
        }
        self.layer_default(agent)
    }

    /// Layer 1 — Hard constraints: survival needs, terrain hazards, economy prices.
    /// These override all other considerations — an starving or freezing agent cannot pursue goals.
    fn layer_survival(&mut self, agent: &SimAgent, obs: &AgentObservation) -> Option<AgentAction> {
        // Hunger: eat nearby food or explore searching for it
        if agent.core.hunger > 60.0 {
            if let Some(res) = obs.nearby_resources.iter()
                .filter(|r| r.resource_type.contains("Food") || r.resource_type.contains("Berries"))
                .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
            {
                return Some(AgentAction::Eat { resource_id: res.id.clone() });
            }
            return Some(AgentAction::Explore { direction: Vec2::new(
                self.rng.range_f32(-1.0, 1.0),
                self.rng.range_f32(-1.0, 1.0),
            )});
        }

        // Low energy: prefer flat terrain when resting
        if agent.core.energy < 30.0 {
            let height = self.heightmap.height_at(agent.core.position.x, agent.core.position.y);
            let candidate_positions = [
                Vec2::new(agent.core.position.x + 20.0, agent.core.position.y),
                Vec2::new(agent.core.position.x - 20.0, agent.core.position.y),
                Vec2::new(agent.core.position.x, agent.core.position.y + 20.0),
                Vec2::new(agent.core.position.x, agent.core.position.y - 20.0),
            ];
            let flat_pos = candidate_positions.iter()
                .min_by(|a, b| {
                    let ha = self.heightmap.height_at(a.x, a.y).abs();
                    let hb = self.heightmap.height_at(b.x, b.y).abs();
                    ha.partial_cmp(&hb).unwrap()
                })
                .copied()
                .unwrap_or(agent.core.position);
            if (height - self.heightmap.height_at(flat_pos.x, flat_pos.y)).abs() > 0.01 {
                return Some(AgentAction::Move { target: flat_pos });
            }
            return Some(AgentAction::Rest);
        }

        // Mountain hazard: flee to lower terrain when low on energy
        let is_mountain = self.heightmap.is_mountain(agent.core.position.x, agent.core.position.y);
        if is_mountain && agent.core.energy < 60.0 {
            let candidates = [
                Vec2::new(agent.core.position.x + 30.0, agent.core.position.y),
                Vec2::new(agent.core.position.x - 30.0, agent.core.position.y),
                Vec2::new(agent.core.position.x, agent.core.position.y + 30.0),
                Vec2::new(agent.core.position.x, agent.core.position.y - 30.0),
            ];
            if let Some(safe_pos) = candidates.iter().min_by(|a, b| {
                let ha = self.heightmap.height_at(a.x, a.y);
                let hb = self.heightmap.height_at(b.x, b.y);
                ha.partial_cmp(&hb).unwrap()
            }) {
                if self.heightmap.height_at(safe_pos.x, safe_pos.y) < self.heightmap.config().mountain_level {
                    return Some(AgentAction::Move { target: *safe_pos });
                }
            }
        }

        // Economy: buy food when cheap and very hungry
        if agent.core.hunger > 70.0 {
            if let Some(&food_price) = self.economy.market_prices.get(&crate::society::economy::ResourceType::Food) {
                if food_price < 2.0 {
                    if let Some(res) = obs.nearby_resources.iter()
                        .filter(|r| r.resource_type.contains("Food") || r.resource_type.contains("Berries"))
                        .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
                    {
                        return Some(AgentAction::Eat { resource_id: res.id.clone() });
                    }
                }
            }
        }

        None
    }

    /// Layer 2 — Goal-driven: PlanningStack active goals.
    /// Validates that planned actions are still feasible given current observation.
    fn layer_goals(&mut self, agent: &SimAgent, obs: &AgentObservation) -> Option<AgentAction> {
        let planning = self.planning.get(&agent.core.id)?;
        let action = planning.next_action()?;
        let planned = action.clone();

        // Validate the planned action is still sensible
        match &planned {
            AgentAction::Eat { resource_id } => {
                if obs.nearby_resources.iter().any(|r| &r.id == resource_id) {
                    return Some(planned);
                }
            }
            AgentAction::Rest => {
                if agent.core.energy < 50.0 { return Some(planned); }
            }
            AgentAction::Talk { target_id, .. } => {
                if obs.nearby_agents.iter().any(|a| &a.id == target_id) {
                    return Some(planned);
                }
            }
            _ => {}
        }

        None
    }

    /// Layer 3 — Social: TheoryOfMind threat assessment + Culture norms + sociability.
    /// Social decisions are only made when survival and goals don't dominate.
    fn layer_social(&mut self, agent: &SimAgent, obs: &AgentObservation, time_mods: &TimeModifiers) -> Option<AgentAction> {
        // Culture: cooperativeness norm may prompt greeting
        let cooperativeness_norm = self.culture.compliance_with("cooperativeness");
        if agent.personality.cooperativeness > cooperativeness_norm && agent.core.hunger < 40.0 {
            if !obs.nearby_agents.is_empty() && self.rng.next_f32() < time_mods.social_activity {
                let target = &obs.nearby_agents[0];
                return Some(AgentAction::Talk {
                    target_id: target.id.clone(),
                    message: "greeting".to_string(),
                });
            }
        }

        // TheoryOfMind: threat assessment and cooperativeness
        if !obs.nearby_agents.is_empty() {
            if let Some(tom) = self.theory_of_mind.get(&agent.core.id) {
                let target = &obs.nearby_agents[0];
                let threat = tom.threat_of(&target.id);
                let coop = tom.cooperativeness_of(&target.id);

                // High threat → flee
                if threat > 0.6 {
                    let flee_dir = Vec2::new(
                        agent.core.position.x - target.distance,
                        agent.core.position.y,
                    ).normalize();
                    return Some(AgentAction::Explore { direction: flee_dir });
                }

                // High cooperativeness → prefer trade/talk
                if coop > 0.6 && self.rng.next_f32() < time_mods.social_activity * 1.5 {
                    return Some(AgentAction::Talk {
                        target_id: target.id.clone(),
                        message: "hello".to_string(),
                    });
                }
            }

            // Default social: talk if personality favors it
            if agent.personality.sociability > 0.6 && self.rng.next_f32() < time_mods.social_activity {
                let target = &obs.nearby_agents[0];
                return Some(AgentAction::Talk {
                    target_id: target.id.clone(),
                    message: "hello".to_string(),
                });
            }
        }

        None
    }

    /// Layer 3.5 — Stigmergy: sense shared pheromone field, react to indirect signals.
    /// Pheromone signals override personality impulses when strong enough.
    fn layer_stigmergy(&mut self, agent: &SimAgent, obs: &AgentObservation) -> Option<AgentAction> {
        let pos = [agent.core.position.x, agent.core.position.y];
        let signal = self.pheromone_field.sense(pos, 120.0, self.tick);

        // Danger pheromone: flee immediately if strong enough
        if signal.danger_repel > 0.5 {
            if let Some(dir) = self.pheromone_field.strongest_direction(
                pos, PheromoneType::Danger, 120.0, self.tick,
            ) {
                // Move away from danger
                let away = Vec2::new(
                    pos[0] - dir[0],
                    pos[1] - dir[1],
                ).normalize();
                return Some(AgentAction::Explore { direction: away });
            }
        }

        // Food pheromone: navigate toward food when hungry
        if agent.core.hunger > 30.0 && signal.food_attract > 0.3 {
            if let Some(dir) = self.pheromone_field.strongest_direction(
                pos, PheromoneType::Food, 120.0, self.tick,
            ) {
                let toward = Vec2::new(
                    dir[0] - pos[0],
                    dir[1] - pos[1],
                ).normalize();
                return Some(AgentAction::Explore { direction: toward });
            }
        }

        // Rest pheromone: rest when tired
        if agent.core.energy < 40.0 && signal.rest_attract > 0.4 {
            return Some(AgentAction::Rest);
        }

        // Social pheromone: seek conversation when social pheromones are strong
        if signal.social_attract > 0.6 {
            if let Some(target) = obs.nearby_agents.first() {
                return Some(AgentAction::Talk {
                    target_id: target.id.clone(),
                    message: "pheromone_greeting".to_string(),
                });
            }
        }

        None
    }

    /// Layer 4 — Personality + Emotion: trait-driven impulses, emotion modulation,
    /// ActionAwareness signals, and DualRepresentation cycle-breaking.
    fn layer_personality(&mut self, agent: &SimAgent, obs: &AgentObservation, _time_mods: &TimeModifiers) -> Option<AgentAction> {
        // Personality bias: aggression, curiosity, cooperativeness
        let roll = self.rng.next_f32();
        if agent.personality.aggression > 0.7 && roll < 0.2 {
            if let Some(target) = obs.nearby_agents.first() {
                return Some(AgentAction::Attack { target_id: target.id.clone() });
            }
        }
        if agent.personality.curiosity > 0.7 && roll < 0.4 {
            let angle = self.rng.range_f32(0.0, std::f32::consts::TAU);
            return Some(AgentAction::Explore { direction: Vec2::new(angle.cos(), angle.sin()) });
        }
        if agent.personality.cooperativeness > 0.7 && roll < 0.3 {
            if let Some(target) = obs.nearby_agents.first() {
                return Some(AgentAction::Trade {
                    target_id: target.id.clone(),
                    item: "berries".to_string(),
                    amount: 1,
                });
            }
        }

        // Emotion modulation
        let dominant = self.emotion.dominant_emotion();
        match dominant {
            Some(EmotionType::Anxiety) | Some(EmotionType::Fatigue) => {
                if self.rng.next_f32() < 0.5 {
                    return Some(AgentAction::Rest);
                }
            }
            Some(EmotionType::Curiosity) | Some(EmotionType::Joy) | Some(EmotionType::Wonder) => {
                let angle = self.rng.range_f32(0.0, std::f32::consts::TAU);
                return Some(AgentAction::Explore { direction: Vec2::new(angle.cos(), angle.sin()) });
            }
            Some(EmotionType::Frustration) => {
                if self.rng.next_f32() < 0.3 {
                    let angle = self.rng.range_f32(0.0, std::f32::consts::TAU);
                    return Some(AgentAction::Explore { direction: Vec2::new(angle.cos(), angle.sin()) });
                }
            }
            _ => {}
        }

        // ActionAwareness: periodic exploration signal
        if let Some(awareness) = self.action_awareness.get(&agent.core.id) {
            if awareness.should_explore() {
                let angle = self.rng.range_f32(0.0, std::f32::consts::TAU);
                return Some(AgentAction::Explore { direction: Vec2::new(angle.cos(), angle.sin()) });
            }
        }

        // DualRepresentation: break repeated action cycles
        let prev_action_label = format!("{:?}_{}", agent.core.id, self.tick.saturating_sub(1));
        let similar = self.dual_repr.similar_to(&prev_action_label, 3);
        if let Some((_, sim_score)) = similar.first() {
            if *sim_score > 0.9 && self.rng.next_f32() < 0.4 {
                let angle = self.rng.range_f32(0.0, std::f32::consts::TAU);
                return Some(AgentAction::Explore { direction: Vec2::new(angle.cos(), angle.sin()) });
            }
        }

        None
    }

    /// Layer 5 — Default: random exploration when no higher-priority layer fires.
    fn layer_default(&mut self, _agent: &SimAgent) -> AgentAction {
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
            AgentAction::Talk { target_id, message: _message } => {
                let agent_id = agent_id.to_string();
                let target_id = target_id.clone();
                self.relationships.update_interaction(&agent_id, &target_id, 0.1, self.tick);
                self.phi_bridge.record_interaction(InteractionRecord {
                    agent_a: agent_id.clone(),
                    agent_b: target_id.clone(),
                    interaction_type: "talk".to_string(),
                    timestamp: self.tick,
                    success: true,
                });
                // Culture: spread meme from conversation
                let meme = self.culture.create_meme(&_message, &agent_id, self.tick);
                let meme_id = meme.id.clone();
                self.culture.spread_meme(&meme_id, &target_id, 0.5);
            }
            AgentAction::Explore { direction } => {
                let agent = &mut self.agents[idx];
                let dir = direction.normalize() * 10.0;
                agent.core.position = agent.core.position + dir;
                agent.core.position.x = agent.core.position.x.clamp(0.0, self.config.world_width);
                agent.core.position.y = agent.core.position.y.clamp(0.0, self.config.world_height);
            }
            AgentAction::Trade { target_id, item, amount } => {
                let agent_id_owned = agent_id.to_string();
                let target_id_owned = target_id.clone();
                // Build dynamic inventories from nearby resources for both agents
                let agent_inv = {
                    let agent = &self.agents[idx];
                    let mut inv = crate::society::economy::Inventory::new();
                    let nearby: Vec<&crate::environment::terrain::resources::ResourceNode> = self.resources.nodes.iter()
                        .filter(|r| !r.depleted)
                        .filter(|r| {
                            let dx = r.position.0 - agent.core.position.x;
                            let dy = r.position.1 - agent.core.position.y;
                            (dx * dx + dy * dy).sqrt() < 150.0
                        })
                        .collect();
                    for r in &nearby {
                        match r.resource_type {
                            crate::environment::terrain::resources::ResourceType::Berries
                            | crate::environment::terrain::resources::ResourceType::Fish
                            | crate::environment::terrain::resources::ResourceType::Meat => {
                                inv.add(crate::society::economy::ResourceType::Food, r.amount * 0.3);
                            }
                            crate::environment::terrain::resources::ResourceType::Wood => {
                                inv.add(crate::society::economy::ResourceType::Wood, r.amount * 0.3);
                            }
                            crate::environment::terrain::resources::ResourceType::Stone
                            | crate::environment::terrain::resources::ResourceType::Ore => {
                                inv.add(crate::society::economy::ResourceType::Stone, r.amount * 0.2);
                            }
                            crate::environment::terrain::resources::ResourceType::Water => {
                                inv.add(crate::society::economy::ResourceType::Water, r.amount * 0.3);
                            }
                            _ => {}
                        }
                    }
                    inv
                };
                let target_inv = {
                    if let Some(target_agent) = self.agents.iter().find(|a| &a.core.id == &target_id_owned) {
                        let mut inv = crate::society::economy::Inventory::new();
                        let nearby: Vec<&crate::environment::terrain::resources::ResourceNode> = self.resources.nodes.iter()
                            .filter(|r| !r.depleted)
                            .filter(|r| {
                                let dx = r.position.0 - target_agent.core.position.x;
                                let dy = r.position.1 - target_agent.core.position.y;
                                (dx * dx + dy * dy).sqrt() < 150.0
                            })
                            .collect();
                        for r in &nearby {
                            match r.resource_type {
                                crate::environment::terrain::resources::ResourceType::Berries
                                | crate::environment::terrain::resources::ResourceType::Fish
                                | crate::environment::terrain::resources::ResourceType::Meat => {
                                    inv.add(crate::society::economy::ResourceType::Food, r.amount * 0.3);
                                }
                                crate::environment::terrain::resources::ResourceType::Wood => {
                                    inv.add(crate::society::economy::ResourceType::Wood, r.amount * 0.3);
                                }
                                crate::environment::terrain::resources::ResourceType::Stone
                                | crate::environment::terrain::resources::ResourceType::Ore => {
                                    inv.add(crate::society::economy::ResourceType::Stone, r.amount * 0.2);
                                }
                                crate::environment::terrain::resources::ResourceType::Water => {
                                    inv.add(crate::society::economy::ResourceType::Water, r.amount * 0.3);
                                }
                                _ => {}
                            }
                        }
                        inv
                    } else {
                        crate::society::economy::Inventory::new()
                    }
                };
                let offer = crate::society::economy::TradeOffer {
                    from: agent_id_owned.clone(),
                    to: target_id_owned.clone(),
                    offer: crate::society::economy::ResourceType::Food,
                    offer_amount: *amount as f32,
                    want: crate::society::economy::ResourceType::Wood,
                    want_amount: *amount as f32 * 0.5,
                };
                let mut buyer_inv = agent_inv;
                let mut seller_inv = target_inv;
                let trade_ok = self.economy.execute_trade(offer, &mut buyer_inv, &mut seller_inv);
                self.relationships.update_interaction(&agent_id_owned, &target_id_owned, 0.15, self.tick);
                if trade_ok {
                    self.bus.emit(
                        SimEvent::EconomyTransaction {
                            buyer: agent_id_owned,
                            seller: target_id_owned,
                            item: item.clone(),
                            amount: *amount,
                        },
                        EventPriority::Normal,
                        self.clock.current,
                        "world_sim",
                    ).await;
                }
            }
            AgentAction::Build { position, structure_type } => {
                let struct_type = match structure_type.as_str() {
                    "shelter" => StructureType::Shelter,
                    "farm" => StructureType::Farm,
                    "workshop" => StructureType::Workshop,
                    "watchtower" => StructureType::Watchtower,
                    "market" => StructureType::Market,
                    "wall" => StructureType::Wall,
                    "road" => StructureType::Road,
                    _ => StructureType::Shelter,
                };
                let _ = self.structures.build(
                    struct_type,
                    [position.x, position.y],
                    agent_id,
                    self.tick,
                );
            }
            AgentAction::Harvest { resource_id } => {
                if let Some(res) = self.resources.nodes.iter_mut().find(|r| &r.id == resource_id) {
                    let harvested = res.harvest(30.0);
                    if harvested > 0.0 {
                        let agent = &mut self.agents[idx];
                        let nutrition = res.resource_type.nutrition_value();
                        let energy_gain = res.resource_type.energy_value();
                        if nutrition > 0.0 {
                            agent.core.eat(harvested * nutrition / 30.0);
                        } else {
                            agent.core.energy = (agent.core.energy + energy_gain * harvested / 30.0).min(100.0);
                        }
                        self.bus.emit(
                            SimEvent::AgentActed {
                                agent_id: agent_id.to_string(),
                                action: "harvest".to_string(),
                                result: format!("harvested {:.1} from {}", harvested, resource_id),
                            },
                            EventPriority::Normal,
                            self.clock.current,
                            "world_sim",
                        ).await;
                        if res.depleted {
                            self.bus.emit(
                                SimEvent::ResourceDepleted {
                                    resource_id: res.id.clone(),
                                    position: res.position,
                                },
                                EventPriority::Normal,
                                self.clock.current,
                                "world_sim",
                            ).await;
                        }
                    }
                }
            }
            _ => {}
        }

        // DualRepresentation: encode every executed action as a dual node
        {
            let action_label = format!("{:?}_{}", agent_id, self.tick);
            let mut attrs = std::collections::HashMap::new();
            attrs.insert("action".to_string(), format!("{:?}", action));
            attrs.insert("agent".to_string(), agent_id.to_string());
            self.dual_repr.add(&action_label, "event", attrs, self.tick);
        }

        // Stigmergy: deposit pheromones based on action outcome
        self.deposit_pheromones_for_action(agent_id, action);
    }

    /// Deposit pheromones into the shared field based on what the agent just did.
    /// This is the "indirect communication" channel — other agents will sense these.
    fn deposit_pheromones_for_action(&mut self, agent_id: &str, action: &AgentAction) {
        let pos = if let Some(agent) = self.agents.iter().find(|a| &a.core.id == agent_id) {
            [agent.core.position.x, agent.core.position.y]
        } else {
            return;
        };

        match action {
            AgentAction::Eat { resource_id } => {
                // Found food → deposit Food pheromone at source location
                if let Some(res) = self.resources.nodes.iter().find(|r| &r.id == resource_id) {
                    self.pheromone_field.deposit(
                        PheromoneType::Food,
                        [res.position.0, res.position.1],
                        agent_id,
                        self.tick,
                    );
                }
            }
            AgentAction::Harvest { resource_id } => {
                // Harvested resource → deposit Food pheromone
                if let Some(res) = self.resources.nodes.iter().find(|r| &r.id == resource_id) {
                    self.pheromone_field.deposit(
                        PheromoneType::Food,
                        [res.position.0, res.position.1],
                        agent_id,
                        self.tick,
                    );
                }
            }
            AgentAction::Rest => {
                // Resting → mark as good rest spot
                self.pheromone_field.deposit(
                    PheromoneType::Rest,
                    pos,
                    agent_id,
                    self.tick,
                );
            }
            AgentAction::Talk { .. } => {
                // Social interaction → deposit Social pheromone
                self.pheromone_field.deposit(
                    PheromoneType::Social,
                    pos,
                    agent_id,
                    self.tick,
                );
            }
            AgentAction::Attack { .. } => {
                // Violence → deposit Danger pheromone
                self.pheromone_field.deposit(
                    PheromoneType::Danger,
                    pos,
                    agent_id,
                    self.tick,
                );
            }
            AgentAction::Explore { .. } => {
                // Exploration → deposit Explore trail
                self.pheromone_field.deposit(
                    PheromoneType::Explore,
                    pos,
                    agent_id,
                    self.tick,
                );
            }
            AgentAction::Build { position, .. } => {
                // Building → deposit Territory marker
                self.pheromone_field.deposit(
                    PheromoneType::Territory,
                    [position.x, position.y],
                    agent_id,
                    self.tick,
                );
            }
            AgentAction::Move { .. } | AgentAction::Trade { .. } | AgentAction::Think => {
                // No pheromone for these actions
            }
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
            genome.traits[5] = a.recent_actions.len() as f32 / 20.0;
            genome.traits[6] = self.relationships.neighbors(&a.core.id).len() as f32 / 10.0;
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

        // M9: ConvergenceDetector → mutation rate modulation
        let base_mutation_rate = self.mutator.config().mutation_rate;
        let adjusted_rate = match self.convergence.state() {
            crate::consciousness::convergence::ConvergenceState::Converged => {
                // Converged: increase mutation to escape local optima
                (base_mutation_rate * 2.0).min(0.5)
            }
            crate::consciousness::convergence::ConvergenceState::Diverged => {
                // Diverged: decrease mutation to stabilize
                (base_mutation_rate * 0.5).max(0.01)
            }
            crate::consciousness::convergence::ConvergenceState::Exploiting => {
                // Exploiting: slight increase
                (base_mutation_rate * 1.3).min(0.3)
            }
            crate::consciousness::convergence::ConvergenceState::Exploring => base_mutation_rate,
        };
        // Safety: clamp mutation rate to safe bounds (CPE constraint)
        let safe_mutation_rate = self.evolution_constraints.clamp_mutation_rate(adjusted_rate);
        self.mutator.set_mutation_rate(safe_mutation_rate);

        // 4. Mutation + Breeding
        let offspring = self.mutator.breed(
            &result.survivors,
            self.config.max_agents - result.survivors.len(),
            &mut self.rng,
        );

        // Safety: validate all genomes before entering population
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

        // 5. Speciation
        let mut all_genomes = result.survivors.clone();
        all_genomes.extend(valid_offspring.clone());
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

        // Safety: record evolution cycle in audit trail
        self.audit_trail.record(self.tick, AuditEventType::EvolutionCycle {
            generation: result.generation,
            population: all_genomes.len(),
            eliminated: result.eliminated.len(),
            offspring: offspring_count,
        });

        // Safety: record agent deaths in audit trail
        for eliminated_id in &result.eliminated {
            self.audit_trail.record(self.tick, AuditEventType::AgentDeath {
                agent_id: eliminated_id.clone(),
                cause: "selection_pressure".to_string(),
                tick: self.tick,
            });
            self.capability_tracker.remove_agent(eliminated_id);
            self.safety_monitor.remove_agent(eliminated_id);
        }

        // Safety: track capabilities for all alive agents
        for agent in &self.agents {
            if !agent.core.alive { continue; }
            let survival = (agent.core.health + agent.core.energy) / 200.0;
            let social = self.relationships.neighbors(&agent.core.id).len() as f32 / 10.0;
            let exploration = agent.recent_actions.iter()
                .filter(|a| a.contains("Explore"))
                .count() as f32 / (20.0_f32).max(1.0);
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

            // Check for capability regression
            if let Some(report) = self.capability_tracker.detect_regression(&agent.core.id, self.tick) {
                self.audit_trail.record(self.tick, AuditEventType::CapabilityRegression {
                    agent_id: agent.core.id.clone(),
                    severity: format!("{:?}", report.severity),
                    details: format!("{} capabilities regressed", report.regressed_capabilities.len()),
                });
            }
        }
    }

    /// Collect system-level events from current world state for emotion processing
    fn collect_world_events(&self) -> Vec<SystemEvent> {
        let mut events = Vec::new();

        // Population pressure: agent deaths signal resource scarcity
        let alive = self.agents.iter().filter(|a| a.core.alive).count();
        let total = self.agents.len().max(1);
        let mortality = 1.0 - (alive as f32 / total as f32);
        if mortality > 0.1 {
            events.push(SystemEvent::GoalBlocked { attempts: (mortality * 10.0) as u32 });
        }

        // Resource depletion → anxiety
        let depleted_ratio = if self.resources.total_nodes() > 0 {
            self.resources.depleted_nodes() as f32 / self.resources.total_nodes() as f32
        } else {
            0.0
        };
        if depleted_ratio > 0.3 {
            events.push(SystemEvent::ErrorRate { rate: depleted_ratio });
        }

        // Mean phi → curiosity/interest
        if let Some(global) = self.coherence_tracker.current() {
            if global.mean_phi > 0.5 {
                events.push(SystemEvent::NoveltyDetected { score: global.mean_phi as f32 });
            }
            // Consensus → resonance
            if global.mean_coherence > 0.6 {
                events.push(SystemEvent::ConsensusReached { agreement: global.mean_coherence as f32 });
            }
        }

        // Low average health → fatigue
        let mean_health: f32 = self.agents.iter()
            .filter(|a| a.core.alive)
            .map(|a| a.core.health)
            .sum::<f32>()
            .max(0.01)
            / alive.max(1) as f32;
        if mean_health < 40.0 {
            events.push(SystemEvent::BatteryLow { level: mean_health / 100.0 });
        }

        // Social interactions happened this tick (from phi_bridge)
        let social_count = self.relationships.total_relationships();
        if social_count > 5 {
            events.push(SystemEvent::PeerConnected { id: "community".into() });
        }

        events
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
            emotion_dominant: self.emotion.dominant_emotion()
                .map(|e| format!("{:?}", e)).unwrap_or_else(|| "Neutral".into()),
            emotion_valence: self.emotion.pad.valence,
            emotion_arousal: self.emotion.pad.arousal,
            emotion_dominance: self.emotion.pad.dominance,
            safety_alerts: self.safety_monitor.all_alerts().len(),
            audit_entries: self.audit_trail.len(),
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
    pub emotion_dominant: String,
    pub emotion_valence: f32,
    pub emotion_arousal: f32,
    pub emotion_dominance: f32,
    pub safety_alerts: usize,
    pub audit_entries: usize,
}
