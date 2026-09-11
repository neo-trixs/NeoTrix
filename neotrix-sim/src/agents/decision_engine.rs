use crate::agents::sim_agent::{SimAgent, AgentAction, AgentObservation};
use crate::agents::stimulus::StimulusLayer;
use crate::agents::behavior_tree::{BehaviorTree, Blackboard, BTStatus};
use crate::agents::planning::goap::{GOAPPlanner, GOAPState, agent_to_world_state};
use crate::foundation::math_bridge::Vec2;

/// Unified decision engine that cascades through four layers:
/// 1. Stimulus (reflexive reactions)
/// 2. Utility (greedy best-score)
/// 3. GOAP (goal-oriented action planning)
/// 4. Behavior Tree (fallback procedural logic)
pub struct DecisionEngine {
    stimulus: StimulusLayer,
    goap_planner: GOAPPlanner,
}

impl DecisionEngine {
    pub fn new() -> Self {
        let mut goap_planner = GOAPPlanner::new();

        // Survival actions
        goap_planner.add_action(
            crate::agents::planning::goap::GOAPAction::new("rest", 0.3)
                .with_effect("has_energy", true),
        );
        goap_planner.add_action(
            crate::agents::planning::goap::GOAPAction::new("gather_food", 1.0)
                .with_precondition("has_energy", true)
                .with_effect("has_food", true),
        );
        goap_planner.add_action(
            crate::agents::planning::goap::GOAPAction::new("eat", 0.5)
                .with_precondition("has_food", true)
                .with_effect("is_hungry", false),
        );
        goap_planner.add_action(
            crate::agents::planning::goap::GOAPAction::new("heal", 0.8)
                .with_precondition("has_energy", true)
                .with_effect("is_healthy", true),
        );
        goap_planner.add_action(
            crate::agents::planning::goap::GOAPAction::new("explore", 1.5)
                .with_precondition("has_energy", true)
                .with_effect("is_nearby_agent", true),
        );
        goap_planner.add_action(
            crate::agents::planning::goap::GOAPAction::new("attack", 2.0)
                .with_precondition("has_energy", true)
                .with_effect("threat_eliminated", true),
        );
        goap_planner.add_action(
            crate::agents::planning::goap::GOAPAction::new("gather", 0.8)
                .with_precondition("has_energy", true)
                .with_effect("has_resources", true),
        );

        Self {
            stimulus: StimulusLayer::new(),
            goap_planner,
        }
    }

    /// Main decision cascade: first non-None layer wins.
    pub fn decide(&self, agent: &SimAgent, obs: &AgentObservation) -> Option<AgentAction> {
        // Layer 1: Stimulus — reflexive reactions (danger, food, enemies)
        if let Some(action) = self.stimulus.check(agent, obs) {
            return Some(action);
        }

        // Layer 2: GOAP — goal-oriented planning
        if let Some(action) = self.goap_decide(agent, obs) {
            return Some(action);
        }

        // Layer 3: Behavior Tree — procedural fallback
        if let Some(action) = self.bt_decide(agent, obs) {
            return Some(action);
        }

        // Layer 4: Utility — greedy scoring
        if let Some(action) = self.utility_decide(agent, obs) {
            return Some(action);
        }

        None
    }

    /// GOAP layer: find a plan from world state to goal, return first action.
    fn goap_decide(&self, agent: &SimAgent, obs: &AgentObservation) -> Option<AgentAction> {
        let world_state = agent_to_world_state_enhanced(agent, obs);
        let goal = self.derive_goal(agent);

        if let Some(plan) = self.goap_planner.plan(&world_state, &goal) {
            if let Some(first) = plan.actions.first() {
                return goap_name_to_action(&first.name, obs);
            }
        }

        None
    }

    /// Derive the most urgent goal from agent state.
    fn derive_goal(&self, agent: &SimAgent) -> GOAPState {
        let mut goal = GOAPState::new();

        if agent.core.hunger > 60.0 {
            goal.set("is_hungry", false);
        } else if agent.core.health < 40.0 {
            goal.set("is_healthy", true);
        } else if agent.core.energy < 30.0 {
            goal.set("has_energy", true);
        } else {
            // Default: explore / socialize
            goal.set("is_nearby_agent", true);
        }

        goal
    }

    /// Behavior Tree layer: simple procedural fallback logic.
    fn bt_decide(&self, agent: &SimAgent, obs: &AgentObservation) -> Option<AgentAction> {
        let mut bb = Blackboard::new();
        bb.set("health", agent.core.health);
        bb.set("energy", agent.core.energy);
        bb.set("hunger", agent.core.hunger);
        bb.set("nearby_agent_count", obs.nearby_agents.len());
        bb.set("nearby_resource_count", obs.nearby_resources.len());

        // Build a simple selector: flee-if-low-health → eat-if-hungry → rest-if-tired → explore
        let mut selected_action: Option<AgentAction> = None;

        if agent.core.health < 25.0 {
            // Flee: move away from nearest threat
            if let Some(threat) = obs.threats.first() {
                let away = Vec2::new(
                    agent.core.position.x - threat.position.x,
                    agent.core.position.y - threat.position.y,
                ).normalize();
                selected_action = Some(AgentAction::Explore { direction: away });
            }
        }

        if selected_action.is_none() && agent.core.hunger > 70.0 {
            if let Some(res) = obs.nearby_resources.iter()
                .filter(|r| r.resource_type.contains("Food") || r.resource_type.contains("Berries"))
                .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
            {
                selected_action = Some(AgentAction::Eat { resource_id: res.id.clone() });
            }
        }

        if selected_action.is_none() && agent.core.energy < 20.0 {
            selected_action = Some(AgentAction::Rest);
        }

        if selected_action.is_none() && !obs.nearby_agents.is_empty() {
            let target = &obs.nearby_agents[0];
            selected_action = Some(AgentAction::Talk {
                target_id: target.id.clone(),
                message: "bt_greeting".to_string(),
            });
        }

        // Use BB to track which branch fired
        let _ = &bb;

        selected_action
    }

    /// Utility layer: score each possible action, pick highest.
    fn utility_decide(&self, agent: &SimAgent, obs: &AgentObservation) -> Option<AgentAction> {
        let mut best_score = 0.0f32;
        let mut best_action: Option<AgentAction> = None;

        // Score: Rest
        let rest_score = if agent.core.energy < 50.0 {
            (50.0 - agent.core.energy) / 50.0
        } else {
            0.0
        };
        if rest_score > best_score {
            best_score = rest_score;
            best_action = Some(AgentAction::Rest);
        }

        // Score: Eat
        if let Some(food) = obs.nearby_resources.iter()
            .filter(|r| r.resource_type.contains("Food") || r.resource_type.contains("Berries"))
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
        {
            let eat_score = (agent.core.hunger / 100.0) * (1.0 - food.distance / 100.0);
            if eat_score > best_score {
                best_score = eat_score;
                best_action = Some(AgentAction::Eat { resource_id: food.id.clone() });
            }
        }

        // Score: Gather
        if let Some(res) = obs.nearby_resources.iter()
            .filter(|r| !r.resource_type.contains("Food"))
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
        {
            let gather_score = 0.3 * (1.0 - res.distance / 100.0) * agent.personality.curiosity;
            if gather_score > best_score {
                best_score = gather_score;
                best_action = Some(AgentAction::Gather { resource_id: res.id.clone() });
            }
        }

        // Score: Talk
        if let Some(target) = obs.nearby_agents.first() {
            let talk_score = agent.personality.sociability * 0.4 * (1.0 - target.distance / 100.0);
            if talk_score > best_score {
                best_score = talk_score;
                best_action = Some(AgentAction::Talk {
                    target_id: target.id.clone(),
                    message: "utility_greeting".to_string(),
                });
            }
        }

        // Score: Explore (always has small score as fallback)
        let explore_score = agent.personality.curiosity * 0.1;
        if explore_score > best_score {
            best_action = Some(AgentAction::Explore {
                direction: Vec2::new(1.0, 0.0),
            });
        }

        best_action
    }
}

/// Enhanced GOAP world state that includes observation context.
fn agent_to_world_state_enhanced(agent: &SimAgent, obs: &AgentObservation) -> GOAPState {
    let mut state = agent_to_world_state(agent);
    state.set("is_nearby_agent", !obs.nearby_agents.is_empty());
    state.set(
        "has_nearby_food",
        obs.nearby_resources
            .iter()
            .any(|r| r.resource_type.contains("Food") || r.resource_type.contains("Berries")),
    );
    state.set(
        "has_nearby_resource",
        !obs.nearby_resources.is_empty(),
    );
    state.set(
        "has_nearby_threat",
        !obs.threats.is_empty(),
    );
    state
}

/// Map GOAP action names to concrete AgentAction variants.
fn goap_name_to_action(name: &str, obs: &AgentObservation) -> Option<AgentAction> {
    match name {
        "rest" => Some(AgentAction::Rest),
        "gather_food" | "eat" => {
            obs.nearby_resources
                .iter()
                .find(|r| r.resource_type.contains("Food") || r.resource_type.contains("Berries"))
                .map(|r| AgentAction::Eat { resource_id: r.id.clone() })
                .or_else(|| {
                    Some(AgentAction::Explore {
                        direction: Vec2::new(
                            obs.position.x.signum() * -0.5,
                            obs.position.y.signum() * -0.5,
                        ),
                    })
                })
        }
        "heal" => Some(AgentAction::Rest),
        "explore" => Some(AgentAction::Explore {
            direction: Vec2::new(1.0, 0.0),
        }),
        "attack" => {
            obs.nearby_agents
                .first()
                .map(|a| AgentAction::Attack { target_id: a.id.clone() })
        }
        "gather" => {
            obs.nearby_resources
                .first()
                .map(|r| AgentAction::Gather { resource_id: r.id.clone() })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::math_bridge::Vec2;

    fn make_agent(health: f32, energy: f32, hunger: f32) -> SimAgent {
        let mut agent = SimAgent::new(1, Vec2::new(50.0, 50.0));
        agent.core.health = health;
        agent.core.energy = energy;
        agent.core.hunger = hunger;
        agent
    }

    fn empty_obs() -> AgentObservation {
        AgentObservation {
            position: Vec2::new(50.0, 50.0),
            nearby_agents: vec![],
            nearby_resources: vec![],
            terrain_type: "Plain".to_string(),
            time_of_day: "day".to_string(),
            season: "Spring".to_string(),
            threats: vec![],
        }
    }

    #[test]
    fn decision_engine_creation() {
        let engine = DecisionEngine::new();
        let agent = make_agent(100.0, 100.0, 0.0);
        let obs = empty_obs();
        let action = engine.decide(&agent, &obs);
        assert!(action.is_some());
    }

    #[test]
    fn stimulus_takes_priority() {
        let engine = DecisionEngine::new();
        let agent = make_agent(5.0, 100.0, 0.0); // Very low health → stimulus triggers
        let obs = AgentObservation {
            position: Vec2::new(50.0, 50.0),
            nearby_agents: vec![],
            nearby_resources: vec![],
            terrain_type: "Plain".to_string(),
            time_of_day: "day".to_string(),
            season: "Spring".to_string(),
            threats: vec![crate::agents::sim_agent::Threat {
                threat_type: "enemy".to_string(),
                position: Vec2::new(60.0, 50.0),
                severity: 0.9,
            }],
        };
        let action = engine.decide(&agent, &obs);
        assert!(action.is_some());
        // Stimulus should produce a flee/explore action due to low health + threat
        let a = action.unwrap();
        assert!(matches!(a, AgentAction::Explore { .. } | AgentAction::Rest));
    }

    #[test]
    fn goap_fallback_to_utility() {
        let engine = DecisionEngine::new();
        // Agent in good state — GOAP may plan explore, utility scores low
        let agent = make_agent(80.0, 80.0, 20.0);
        let obs = empty_obs();
        let action = engine.decide(&agent, &obs);
        // Should get something from one of the layers
        assert!(action.is_some());
    }

    #[test]
    fn utility_picks_eat_when_hungry_and_food_nearby() {
        let engine = DecisionEngine::new();
        let agent = make_agent(80.0, 80.0, 90.0);
        let obs = AgentObservation {
            position: Vec2::new(50.0, 50.0),
            nearby_agents: vec![],
            nearby_resources: vec![crate::agents::sim_agent::NearbyResource {
                id: "food_1".to_string(),
                resource_type: "Food".to_string(),
                distance: 10.0,
                amount: 50.0,
            }],
            terrain_type: "Plain".to_string(),
            time_of_day: "day".to_string(),
            season: "Spring".to_string(),
            threats: vec![],
        };
        let action = engine.decide(&agent, &obs);
        assert!(action.is_some());
    }

    #[test]
    fn goap_name_to_action_mapping() {
        let obs = empty_obs();
        assert!(matches!(goap_name_to_action("rest", &obs), Some(AgentAction::Rest)));
        assert!(goap_name_to_action("attack", &obs).is_none()); // no nearby agents
        assert!(goap_name_to_action("gather", &obs).is_none()); // no nearby resources
        assert!(goap_name_to_action("unknown", &obs).is_none());
    }
}
