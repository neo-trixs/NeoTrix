pub mod config;
pub mod decision;
pub mod actions;
pub mod evolution;
pub mod observation;
pub mod persistence;
pub mod renderer;
pub mod ui;

pub use config::*;

use crate::foundation::simulation_bus::{SimulationBus, SimEvent, EventPriority};
use crate::foundation::sim_time::{SimClock, TimeModifiers};
use crate::foundation::math_bridge::{Vec2, SpatialGrid, SimulationRng};
use crate::foundation::tick_schedule::{TickSchedule, TickTier};
use crate::environment::terrain::{Heightmap, HeightmapConfig, BiomeMap, ResourceDistribution};
use crate::environment::structures::StructureManager;
use crate::agents::sim_agent::{SimAgent, AgentAction};
use crate::agents::action_awareness::ActionAwareness;
use crate::agents::memory_stream::MemoryStream;
use crate::agents::graph_memory::GraphMemory;
use crate::agents::spatial_memory::SpatialMemory;
use crate::agents::planning::PlanningStack;
use crate::agents::reflection::ReflectionEngine;
use crate::agents::action_costs::{ActionCostTable, ActionBudget};
use crate::agents::personality_drift::PersonalityDrift;
use crate::consciousness::phi_bridge::PhiBridge;
use crate::consciousness::coherence_tracker::CoherenceTracker;
use crate::consciousness::convergence::ConvergenceDetector;
use crate::consciousness::dual_representation::DualRepresentation;
use crate::evolution::fitness_landscape::{FitnessLandscape, LandscapeConfig};
use crate::evolution::selection_pressure::SelectionPressure;
use crate::evolution::mutation_ops::MutationOps;
use crate::evolution::speciation::Speciation;
use crate::feel::EmotionEngine;
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
use crate::agents::pheromone::PheromoneField;
use crate::safety::{
    CapabilityTracker, SafetyMonitor, EvolutionConstraints, AuditTrail,
};
use std::collections::HashMap;

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
    pub goal_outcome_feedback: GoalOutcomeFeedback,
    pub intention_commitment: IntentionCommitment,
    pub thought_generation: ThoughtGeneration,
    pub social_learning: SocialLearning,
    pub pheromone_field: PheromoneField,
    pub capability_tracker: CapabilityTracker,
    pub safety_monitor: SafetyMonitor,
    pub evolution_constraints: EvolutionConstraints,
    pub audit_trail: AuditTrail,
}

impl WorldSim {
    pub fn new(config: WorldSimConfig) -> Self {
        let mut rng = SimulationRng::new(config.seed);

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

        let resources = ResourceDistribution::generate(config.seed, config.world_width, config.world_height, &biome_map);

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
            selection: SelectionPressure::new(Default::default()),
            mutator: MutationOps::new(Default::default()),
            speciation: Speciation::new(Default::default()),
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
            capability_tracker: CapabilityTracker::new(Default::default()),
            safety_monitor: SafetyMonitor::new(Default::default()),
            evolution_constraints: EvolutionConstraints::new(Default::default()),
            audit_trail: AuditTrail::new(10000),
        }
    }

    /// Run one simulation tick using multi-timescale schedule
    pub async fn tick(&mut self) {
        self.schedule.advance();
        let tick = self.schedule.current_tick();
        self.tick = tick;

        if self.schedule.should_run(TickTier::Reflex) {
            let time_events = self.clock.tick();
            for (event, priority) in time_events {
                self.bus.emit(event, priority, self.clock.current, "clock").await;
            }
            let season_mod = self.clock.season_resource_modifier();
            self.resources.regenerate_all(season_mod);
            self.spatial_grid.clear();
            for agent in &self.agents {
                if agent.core.alive {
                    self.spatial_grid.insert(&agent.core.id, agent.core.position);
                }
            }
            let time_mods = TimeModifiers::from_clock(&self.clock);
            for agent in &mut self.agents {
                if agent.core.alive {
                    let energy_cost = 0.5 * time_mods.perception;
                    agent.core.metabolize(energy_cost, 0.3);
                }
            }
        }

        if self.schedule.should_run(TickTier::Fast) {
            self.tick_agent_decisions().await;
        }

        if self.schedule.should_run(TickTier::Reflex) {
            self.tick_reactive_events().await;
        }

        if self.schedule.should_run(TickTier::Medium) {
            self.compute_consciousness_metrics().await;
        }

        if self.schedule.should_run(TickTier::Slow) {
            self.tick_slow_systems().await;
        }

        if self.schedule.should_run(TickTier::Background) {
            self.evolution_cycle().await;
        }

        self.agents.retain(|a| a.core.alive);
    }

    async fn tick_agent_decisions(&mut self) {
        let time_mods = TimeModifiers::from_clock(&self.clock);
        let agent_ids: Vec<String> = self.agents.iter().map(|a| a.core.id.clone()).collect();

        for agent_id in &agent_ids {
            self.ensure_agent_subsystems(agent_id);

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

            let action = self.decide_action_by_id(agent_id, &observation, &time_mods);
            let (compliance, _violated) = self.constitutional.evaluate_action(&action, self.tick);

            let can_afford = {
                let agent = self.agents.iter().find(|a| &a.core.id == agent_id);
                agent.map(|a| self.action_costs.can_afford(&action, a.core.energy, a.core.health))
                    .unwrap_or(false)
            };

            if !can_afford || compliance < 0.3 {
                self.execute_action(agent_id, &AgentAction::Rest).await;
                continue;
            }

            {
                let agent = self.agents.iter().find(|a| &a.core.id == agent_id).cloned();
                if let Some(agent) = agent {
                    self.action_awareness
                        .entry(agent_id.clone())
                        .or_insert_with(ActionAwareness::new)
                        .predict(&action, &agent);
                }
            }

            if let Some(budget) = self.action_budgets.get_mut(agent_id) {
                budget.record_action(&action, &self.action_costs);
            }

            self.execute_action(agent_id, &action).await;

            self.record_agent_action(agent_id, &action, &observation, compliance);
        }
    }

    fn ensure_agent_subsystems(&mut self, agent_id: &str) {
        let pos = self.agents.iter().find(|a| &a.core.id == agent_id)
            .map(|a| [a.core.position.x, a.core.position.y])
            .unwrap_or([0.0, 0.0]);
        self.memory_streams.entry(agent_id.to_string()).or_insert_with(|| MemoryStream::new(200));
        self.graph_memories.entry(agent_id.to_string()).or_insert_with(|| GraphMemory::new(500));
        self.spatial_memories.entry(agent_id.to_string()).or_insert_with(|| SpatialMemory::new(pos));
        self.planning.entry(agent_id.to_string()).or_insert_with(PlanningStack::new);
        self.reflections.entry(agent_id.to_string()).or_insert_with(ReflectionEngine::new);
        self.action_budgets.entry(agent_id.to_string()).or_insert_with(ActionBudget::new);
        self.personality_drift.entry(agent_id.to_string()).or_insert_with(|| PersonalityDrift::new(Default::default()));
        self.theory_of_mind.entry(agent_id.to_string()).or_insert_with(|| TheoryOfMind::new(50));
    }

    fn record_agent_action(
        &mut self,
        agent_id: &str,
        action: &AgentAction,
        observation: &crate::agents::sim_agent::AgentObservation,
        compliance: f32,
    ) {
        let tick = self.tick;

        {
            let action_str = format!("{:?}", action);
            self.safety_monitor.record_action(agent_id, &action_str, tick);
            let alerts = self.safety_monitor.check_all(agent_id, tick);
            for alert in &alerts {
                self.audit_trail.record(tick, crate::safety::AuditEventType::SafetyViolation {
                    agent_id: agent_id.to_string(),
                    violation_type: format!("{:?}", alert.violation),
                    severity: alert.severity,
                });
            }
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
                    self.audit_trail.record(tick, crate::safety::AuditEventType::SafetyViolation {
                        agent_id: agent_id.to_string(),
                        violation_type: format!("{:?}", alert.violation),
                        severity: alert.severity,
                    });
                }
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

        if let Some(agent) = self.agents.iter().find(|a| &a.core.id == agent_id) {
            if let Some(awareness) = self.action_awareness.get_mut(agent_id) {
                awareness.verify(agent, tick);
            }
        }

        if let Some(ms) = self.memory_streams.get_mut(agent_id) {
            ms.add(crate::agents::memory_stream::MemoryNode {
                id: 0,
                kind: crate::agents::memory_stream::MemoryKind::Observation,
                agent_id: agent_id.to_string(),
                created_tick: tick,
                last_accessed_tick: tick,
                description: format!("{:?}", action),
                importance: compliance as f32,
                keywords: vec![],
                citations: vec![],
                embedding: None,
            });
        }

        if let Some(gm) = self.graph_memories.get_mut(agent_id) {
            let node_id = gm.add_node(
                crate::agents::graph_memory::NodeKind::Event,
                &format!("{:?}", action),
                tick,
                compliance as f32,
            );
            if let Some(prev_id) = gm.last_node_id() {
                if prev_id != node_id {
                    gm.add_edge(prev_id, node_id, crate::agents::graph_memory::EdgeKind::Temporal, 0.8, tick);
                }
            }
            match action {
                crate::agents::sim_agent::AgentAction::Talk { ref target_id, .. }
                | crate::agents::sim_agent::AgentAction::Trade { ref target_id, .. } => {
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

        if let Some(sm) = self.spatial_memories.get_mut(agent_id) {
            if let Some(agent) = self.agents.iter().find(|a| &a.core.id == agent_id) {
                let danger = if compliance < 0.3 { 0.5 } else { 0.1 };
                sm.visit(
                    [agent.core.position.x, agent.core.position.y],
                    "plain",
                    vec![],
                    danger,
                    tick,
                );
            }
        }

        if let Some(agent) = self.agents.iter_mut().find(|a| &a.core.id == agent_id) {
            agent.record_action(action);
        }

        if let Some(tom) = self.theory_of_mind.get_mut(agent_id) {
            for obs_agent in &observation.nearby_agents {
                let positive = obs_agent.relationship > 0.0;
                tom.observe_interaction(&obs_agent.id, true, positive, tick);
            }
        }

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

    async fn tick_reactive_events(&mut self) {
        let records = {
            let hist = self.bus.history();
            let hist = hist.read().await;
            hist.recent(100).to_vec()
        };
        let events: Vec<SimEvent> = records.iter().map(|r| r.event.clone()).collect();
        let responses = self.event_reactive.process_events(&events, self.tick);
        for resp in responses {
            if let Some(agent) = self.agents.iter_mut().find(|a| a.core.id == resp.agent_id) {
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
                        self.relationships.update_interaction(&resp.agent_id, target_id, 0.1, self.tick);
                    }
                    _ => {}
                }
            }
        }
    }

    async fn tick_slow_systems(&mut self) {
        {
            let events = self.collect_world_events();
            self.emotion.process_events(&events);
        }

        for awareness in self.action_awareness.values_mut() {
            awareness.learn();
        }

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

        for agent in &mut self.agents {
            if let Some(pd) = self.personality_drift.get_mut(&agent.core.id) {
                let new_p = pd.drift(&agent.personality, agent.core.age);
                agent.personality = new_p;
            }
        }

        self.constitutional.decay();

        let alive_count = self.agents.iter().filter(|a| a.core.alive).count();
        let mean_fitness = self.agents.iter().filter(|a| a.core.alive).map(|a| a.core.health as f64).sum::<f64>()
            / alive_count.max(1) as f64;
        self.convergence.record(mean_fitness, self.tick);

        self.pheromone_field.decay(self.tick);
        self.pheromone_field.prune(self.tick);
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
