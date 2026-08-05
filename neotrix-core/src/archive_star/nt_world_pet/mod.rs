pub mod state;
pub mod traits;
pub mod miner;
pub mod evolution;

pub use state::*;
pub use traits::*;
pub use miner::*;
pub use evolution::*;

#[derive(Debug, Clone)]
pub struct PetEngine {
    pub state: PetState,
    decay_rate: f64,
}

impl Default for PetEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PetEngine {
    pub fn new() -> Self {
        Self {
            state: PetState::default(),
            decay_rate: 0.02,
        }
    }

    pub fn with_kb_snapshot(kb_nodes: u64) -> Self {
        Self {
            state: PetState {
                kb_nodes_at_birth: kb_nodes,
                ..Default::default()
            },
            decay_rate: 0.02,
        }
    }

    pub fn process_conversation(&mut self, user_text: &str) {
        let signal = ConversationMiner::mine(user_text);
        let decay = if signal.visual_deltas.size.abs() < 1e-16
            && signal.visual_deltas.warmth.abs() < 1e-16
            && signal.visual_deltas.softness.abs() < 1e-16
            && signal.visual_deltas.energy.abs() < 1e-16
            && signal.visual_deltas.brightness.abs() < 1e-16
            && signal.visual_deltas.creature.abs() < 1e-16
            && signal.visual_deltas.complexity.abs() < 1e-16
            && signal.visual_deltas.definition.abs() < 1e-16
            && signal.behavior_deltas.curiosity.abs() < 1e-16
            && signal.behavior_deltas.playfulness.abs() < 1e-16
            && signal.behavior_deltas.talkativeness.abs() < 1e-16
            && signal.behavior_deltas.reactivity.abs() < 1e-16
        {
            self.decay_rate * 3.0
        } else {
            self.decay_rate * 0.5
        };

        TraitEvolution::apply_signal(
            &mut self.state.target_visual,
            &mut self.state.behavior,
            &signal,
            decay,
        );

        self.state.conversations_had += 1;
        self.state.age_cycles += 1;
    }

    pub fn tick(&mut self) {
        self.state.tick_transition();
        self.state.age_cycles += 1;
    }

    pub fn set_valence(&mut self, valence: f64, arousal: f64) {
        self.state.expression = match () {
            _ if valence > 0.5 && arousal > 0.6 => PetExpression::Excited,
            _ if valence > 0.0 && arousal > 0.5 => PetExpression::Curious,
            _ if valence > 0.5 && arousal < 0.4 => PetExpression::Content,
            _ if valence < -0.3 && arousal > 0.4 => PetExpression::Frustrated,
            _ if valence.abs() < 0.2 && arousal > 0.5 => PetExpression::Confused,
            _ if arousal < 0.2 => PetExpression::Sleepy,
            _ => PetExpression::Neutral,
        };
    }

    pub fn set_energy(&mut self, curiosity: f64) {
        self.state.energy = curiosity.max(0.1).min(1.0);
    }

    pub fn check_growth(&mut self, total_kb_nodes: u64) -> Vec<LevelUpReward> {
        GrowthTable::check_level_up(&mut self.state, total_kb_nodes)
    }

    pub fn decay(&mut self) {
        let empty = TraitSignal::new();
        TraitEvolution::apply_signal(
            &mut self.state.target_visual,
            &mut self.state.behavior,
            &empty,
            self.decay_rate,
        );
    }

    pub fn state_snapshot(&self) -> &PetState {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = PetEngine::new();
        assert_eq!(engine.state.level, 0);
        assert!((engine.state.visual.size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_conversation_changes_traits() {
        let mut engine = PetEngine::new();
        engine.process_conversation("你就像一只猫");
        assert!(engine.state.target_visual.creature < 0.3);
    }

    #[test]
    fn test_tick_transition() {
        let mut engine = PetEngine::new();
        engine.state.target_visual.size = 0.9;
        engine.tick();
        assert!(engine.state.visual.size < 0.9);
        assert!(engine.state.visual.size > 0.5);
    }

    #[test]
    fn test_valence_mapping() {
        let mut engine = PetEngine::new();
        engine.set_valence(0.8, 0.9);
        assert_eq!(engine.state.expression, PetExpression::Excited);
        engine.set_valence(0.7, 0.2);
        assert_eq!(engine.state.expression, PetExpression::Content);
        engine.set_valence(-0.5, 0.6);
        assert_eq!(engine.state.expression, PetExpression::Frustrated);
    }

    #[test]
    fn test_growth() {
        let mut engine = PetEngine::with_kb_snapshot(0);
        engine.state.conversations_had = 10;
        let rewards = engine.check_growth(500);
        assert_eq!(engine.state.level, 1);
        assert!(!rewards.is_empty());
    }

    #[test]
    fn test_decay_over_time() {
        let mut engine = PetEngine::new();
        engine.state.target_visual.size = 0.9;
        engine.decay();
        assert!(engine.state.target_visual.size < 0.9);
    }

    #[test]
    fn test_multi_conversation_accumulates() {
        let mut engine = PetEngine::new();
        engine.process_conversation("你好温暖");
        engine.process_conversation("你好温暖");
        assert!(engine.state.target_visual.warmth > 0.6);
    }

    #[test]
    fn test_empty_text_no_change() {
        let mut engine = PetEngine::new();
        let before = engine.state.target_visual.size;
        engine.process_conversation("今天天气不错");
        assert!((engine.state.target_visual.size - before).abs() < 0.01);
    }
}
