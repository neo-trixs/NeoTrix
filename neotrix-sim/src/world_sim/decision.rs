use crate::foundation::math_bridge::Vec2;
use crate::foundation::sim_time::TimeModifiers;
use crate::agents::sim_agent::{SimAgent, AgentAction, AgentObservation};
use crate::agents::pheromone::PheromoneSignal;
use crate::feel::EmotionType;
use super::WorldSim;

impl WorldSim {
    /// Unified decision pipeline: evaluate layers in priority order, first non-None wins.
    pub(crate) fn decide_action(&mut self, agent: &SimAgent, obs: &AgentObservation, time_mods: &TimeModifiers) -> AgentAction {
        let pos = [agent.core.position.x, agent.core.position.y];
        let pheromone_signal = self.pheromone_field.sense(pos, 120.0, self.tick);

        if let Some(action) = self.layer_survival(agent, obs, &pheromone_signal) {
            return action;
        }
        if let Some(action) = self.layer_goals(agent, obs) {
            return action;
        }
        if let Some(action) = self.layer_social(agent, obs, time_mods) {
            return action;
        }
        if let Some(action) = self.layer_stigmergy(agent, obs, &pheromone_signal) {
            return action;
        }
        if let Some(action) = self.layer_personality(agent, obs, time_mods) {
            return action;
        }
        self.layer_default(agent)
    }

    pub(crate) fn decide_action_by_id(&mut self, agent_id: &str, obs: &AgentObservation, time_mods: &TimeModifiers) -> AgentAction {
        let agent = self.agents.iter().find(|a| &a.core.id == agent_id).cloned();
        if let Some(agent) = agent {
            self.decide_action(&agent, obs, time_mods)
        } else {
            AgentAction::Rest
        }
    }

    /// Layer 1 — Hard constraints: survival needs, terrain hazards, economy prices.
    /// Pheromone signal: danger pheromones boost urgency for food-seeking.
    fn layer_survival(&mut self, agent: &SimAgent, obs: &AgentObservation, pheromone: &PheromoneSignal) -> Option<AgentAction> {
        let effective_hunger = agent.core.hunger + pheromone.danger_repel * 5.0;
        if effective_hunger > 60.0 {
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
    fn layer_goals(&mut self, agent: &SimAgent, obs: &AgentObservation) -> Option<AgentAction> {
        let planning = self.planning.get(&agent.core.id)?;
        let action = planning.next_action()?;
        let planned = action.clone();

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
    fn layer_social(&mut self, agent: &SimAgent, obs: &AgentObservation, time_mods: &TimeModifiers) -> Option<AgentAction> {
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

        if !obs.nearby_agents.is_empty() {
            if let Some(tom) = self.theory_of_mind.get(&agent.core.id) {
                let target = &obs.nearby_agents[0];
                let threat = tom.threat_of(&target.id);
                let coop = tom.cooperativeness_of(&target.id);

                if threat > 0.6 {
                    let flee_dir = Vec2::new(
                        agent.core.position.x - target.distance,
                        agent.core.position.y,
                    ).normalize();
                    return Some(AgentAction::Explore { direction: flee_dir });
                }

                if coop > 0.6 && self.rng.next_f32() < time_mods.social_activity * 1.5 {
                    return Some(AgentAction::Talk {
                        target_id: target.id.clone(),
                        message: "hello".to_string(),
                    });
                }
            }

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
    fn layer_stigmergy(&mut self, agent: &SimAgent, _obs: &AgentObservation, pheromone: &PheromoneSignal) -> Option<AgentAction> {
        use crate::agents::pheromone::PheromoneType;

        let pos = [agent.core.position.x, agent.core.position.y];

        if pheromone.danger_repel > 0.5 {
            if let Some(dir) = self.pheromone_field.strongest_direction(
                pos, PheromoneType::Danger, 120.0, self.tick,
            ) {
                let away = Vec2::new(
                    pos[0] - dir[0],
                    pos[1] - dir[1],
                ).normalize();
                return Some(AgentAction::Explore { direction: away });
            }
        }

        if agent.core.hunger > 30.0 && pheromone.food_attract > 0.3 {
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

        if agent.core.energy < 40.0 && pheromone.rest_attract > 0.4 {
            return Some(AgentAction::Rest);
        }

        if pheromone.social_attract > 0.6 {
            if let Some(target) = _obs.nearby_agents.first() {
                return Some(AgentAction::Talk {
                    target_id: target.id.clone(),
                    message: "pheromone_greeting".to_string(),
                });
            }
        }

        None
    }

    /// Layer 4 — Personality + Emotion: trait-driven impulses, emotion modulation.
    /// Pheromone net_valence modulates exploration/exploitation tradeoff.
    fn layer_personality(&mut self, agent: &SimAgent, _obs: &AgentObservation, _time_mods: &TimeModifiers, pheromone: &PheromoneSignal) -> Option<AgentAction> {
        let valence = pheromone.net_valence();
        let explore_boost = if valence > 0.3 { 0.1 } else { 0.0 };

        let roll = self.rng.next_f32();
        if agent.personality.aggression > 0.7 && roll < 0.2 {
            if let Some(target) = _obs.nearby_agents.first() {
                return Some(AgentAction::Attack { target_id: target.id.clone() });
            }
        }
        if agent.personality.curiosity > 0.7 && roll < 0.4 + explore_boost {
            let angle = self.rng.range_f32(0.0, std::f32::consts::TAU);
            return Some(AgentAction::Explore { direction: Vec2::new(angle.cos(), angle.sin()) });
        }
        if agent.personality.cooperativeness > 0.7 && roll < 0.3 {
            if let Some(target) = _obs.nearby_agents.first() {
                return Some(AgentAction::Trade {
                    target_id: target.id.clone(),
                    item: "berries".to_string(),
                    amount: 1,
                });
            }
        }

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

        if let Some(awareness) = self.action_awareness.get(&agent.core.id) {
            if awareness.should_explore() {
                let angle = self.rng.range_f32(0.0, std::f32::consts::TAU);
                return Some(AgentAction::Explore { direction: Vec2::new(angle.cos(), angle.sin()) });
            }
        }

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
}
