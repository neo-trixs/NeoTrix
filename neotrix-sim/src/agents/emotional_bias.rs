use crate::agents::SimAgent;
use crate::agents::sim_agent::AgentAction;
use crate::feel::{EmotionEngine, EmotionType};
use crate::foundation::math_bridge::Vec2;

pub struct EmotionalBiasSystem {
    pub frustration_bias: f32,
    pub anxiety_bias: f32,
    pub joy_bias: f32,
    pub fatigue_bias: f32,
}

impl EmotionalBiasSystem {
    pub fn new() -> Self {
        Self {
            frustration_bias: 0.3,
            anxiety_bias: 0.4,
            joy_bias: 0.2,
            fatigue_bias: 0.1,
        }
    }

    pub fn bias_action(&self, action: AgentAction, agent: &SimAgent, emotion: &EmotionEngine) -> AgentAction {
        let dominant = emotion.dominant_emotion();

        match dominant {
            Some(EmotionType::Frustration) => self.apply_frustration_bias(action, agent),
            Some(EmotionType::Anxiety) => self.apply_anxiety_bias(action, agent),
            Some(EmotionType::Joy) => self.apply_joy_bias(action, agent),
            Some(EmotionType::Fatigue) => self.apply_fatigue_bias(action, agent),
            _ => action,
        }
    }

    fn apply_frustration_bias(&self, action: AgentAction, agent: &SimAgent) -> AgentAction {
        match action {
            AgentAction::Explore { .. } => {
                if agent.personality.aggression > 0.5 {
                    let dir = Vec2::new(
                        agent.core.position.y,
                        agent.core.position.x,
                    );
                    AgentAction::Explore { direction: dir }
                } else {
                    action
                }
            }
            _ => action,
        }
    }

    fn apply_anxiety_bias(&self, action: AgentAction, _agent: &SimAgent) -> AgentAction {
        match action {
            AgentAction::Explore { direction } => {
                AgentAction::Explore { direction: -direction }
            }
            _ => action,
        }
    }

    fn apply_joy_bias(&self, action: AgentAction, _agent: &SimAgent) -> AgentAction {
        action
    }

    fn apply_fatigue_bias(&self, action: AgentAction, _agent: &SimAgent) -> AgentAction {
        match action {
            AgentAction::Explore { .. } => AgentAction::Rest,
            _ => action,
        }
    }

    pub fn modulate_energy_cost(&self, base_cost: f32, emotion: &EmotionEngine) -> f32 {
        let dominant = emotion.dominant_emotion();

        match dominant {
            Some(EmotionType::Frustration) => base_cost * (1.0 + self.frustration_bias * 0.2),
            Some(EmotionType::Anxiety) => base_cost * (1.0 - self.anxiety_bias * 0.1),
            Some(EmotionType::Joy) => base_cost * (1.0 - self.joy_bias * 0.1),
            Some(EmotionType::Fatigue) => base_cost * (1.0 + self.fatigue_bias * 0.3),
            _ => base_cost,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotional_bias_creates() {
        let system = EmotionalBiasSystem::new();
        assert_eq!(system.frustration_bias, 0.3);
    }

    #[test]
    fn test_energy_modulation() {
        let system = EmotionalBiasSystem::new();
        let engine = EmotionEngine::new();
        let cost = system.modulate_energy_cost(10.0, &engine);
        assert!(cost > 0.0);
    }
}
