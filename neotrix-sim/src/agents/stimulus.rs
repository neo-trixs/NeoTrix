use crate::agents::SimAgent;
use crate::agents::sim_agent::AgentAction;
use crate::foundation::math_bridge::Vec2;

#[derive(Debug, Clone, PartialEq)]
pub enum StimulusType {
    Danger,
    Food,
    Agent,
    Territory,
    Resource,
}

#[derive(Debug, Clone)]
pub struct Stimulus {
    pub stimulus_type: StimulusType,
    pub intensity: f32,
    pub position: Vec2,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReflexResponse {
    Fight,
    Flight,
    Freeze,
    Seek,
    Explore,
    None,
}

pub struct StimulusResponseSystem {
    pub danger_threshold: f32,
    pub food_threshold: f32,
    pub social_threshold: f32,
}

impl StimulusResponseSystem {
    pub fn new() -> Self {
        Self {
            danger_threshold: 0.7,
            food_threshold: 0.5,
            social_threshold: 0.3,
        }
    }

    pub fn evaluate_stimuli(&self, stimuli: &[Stimulus], agent: &SimAgent) -> ReflexResponse {
        let danger = stimuli.iter()
            .filter(|s| s.stimulus_type == StimulusType::Danger)
            .map(|s| s.intensity)
            .fold(0.0f32, f32::max);

        let food = stimuli.iter()
            .filter(|s| s.stimulus_type == StimulusType::Food)
            .map(|s| s.intensity)
            .fold(0.0f32, f32::max);

        let social = stimuli.iter()
            .filter(|s| s.stimulus_type == StimulusType::Agent)
            .map(|s| s.intensity)
            .fold(0.0f32, f32::max);

        if danger > self.danger_threshold {
            if agent.core.energy < 30.0 {
                ReflexResponse::Flight
            } else if agent.personality.aggression > 0.6 {
                ReflexResponse::Fight
            } else {
                ReflexResponse::Freeze
            }
        } else if food > self.food_threshold && agent.core.hunger > 50.0 {
            ReflexResponse::Seek
        } else if social > self.social_threshold {
            ReflexResponse::Explore
        } else {
            ReflexResponse::None
        }
    }

    pub fn reflex_to_action(&self, reflex: &ReflexResponse) -> Option<AgentAction> {
        match reflex {
            ReflexResponse::Fight => Some(AgentAction::Explore { direction: Vec2::new(1.0, 0.0) }),
            ReflexResponse::Flight => Some(AgentAction::Explore { direction: Vec2::new(-1.0, 0.0) }),
            ReflexResponse::Freeze => Some(AgentAction::Rest),
            ReflexResponse::Seek => Some(AgentAction::Harvest { resource_id: String::new() }),
            ReflexResponse::Explore => Some(AgentAction::Explore { direction: Vec2::new(1.0, 0.0) }),
            ReflexResponse::None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stimulus_response_danger() {
        let system = StimulusResponseSystem::new();
        let stimuli = vec![Stimulus {
            stimulus_type: StimulusType::Danger,
            intensity: 0.9,
            position: Vec2::new(5.0, 5.0),
        }];
        let agent = SimAgent::new(0, Vec2::new(0.0, 0.0));
        let response = system.evaluate_stimuli(&stimuli, &agent);
        assert_eq!(response, ReflexResponse::Freeze);
    }

    #[test]
    fn test_stimulus_response_food() {
        let system = StimulusResponseSystem::new();
        let stimuli = vec![Stimulus {
            stimulus_type: StimulusType::Food,
            intensity: 0.8,
            position: Vec2::new(5.0, 5.0),
        }];
        let mut agent = SimAgent::new(0, Vec2::new(0.0, 0.0));
        agent.core.hunger = 80.0;
        let response = system.evaluate_stimuli(&stimuli, &agent);
        assert_eq!(response, ReflexResponse::Seek);
    }

    #[test]
    fn test_reflex_to_action() {
        let system = StimulusResponseSystem::new();
        assert!(system.reflex_to_action(&ReflexResponse::Fight).is_some());
        assert!(system.reflex_to_action(&ReflexResponse::None).is_none());
    }
}
