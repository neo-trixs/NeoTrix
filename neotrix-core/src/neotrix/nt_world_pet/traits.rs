use crate::neotrix::nt_world_pet::state::{BehaviorParams, VisualParams};

#[derive(Debug, Clone)]
pub struct TraitSignal {
    pub visual_deltas: VisualParams,
    pub behavior_deltas: BehaviorParams,
}

fn zero_visual() -> VisualParams {
    VisualParams {
        size: 0.0,
        warmth: 0.0,
        softness: 0.0,
        energy: 0.0,
        brightness: 0.0,
        creature: 0.0,
        complexity: 0.0,
        definition: 0.0,
    }
}

fn zero_behavior() -> BehaviorParams {
    BehaviorParams {
        curiosity: 0.0,
        playfulness: 0.0,
        talkativeness: 0.0,
        reactivity: 0.0,
    }
}

impl TraitSignal {
    pub fn new() -> Self {
        Self {
            visual_deltas: zero_visual(),
            behavior_deltas: zero_behavior(),
        }
    }

    pub fn with_visual(mut self, field: &str, delta: f64) -> Self {
        match field {
            "size" => self.visual_deltas.size += delta,
            "warmth" => self.visual_deltas.warmth += delta,
            "softness" => self.visual_deltas.softness += delta,
            "energy" => self.visual_deltas.energy += delta,
            "brightness" => self.visual_deltas.brightness += delta,
            "creature" => self.visual_deltas.creature += delta,
            "complexity" => self.visual_deltas.complexity += delta,
            "definition" => self.visual_deltas.definition += delta,
            _ => {}
        }
        self
    }

    pub fn with_behavior(mut self, field: &str, delta: f64) -> Self {
        match field {
            "curiosity" => self.behavior_deltas.curiosity += delta,
            "playfulness" => self.behavior_deltas.playfulness += delta,
            "talkativeness" => self.behavior_deltas.talkativeness += delta,
            "reactivity" => self.behavior_deltas.reactivity += delta,
            _ => {}
        }
        self
    }
}

pub struct TraitEvolution;

impl TraitEvolution {
    pub fn apply_signal(
        target: &mut VisualParams,
        behavior_target: &mut BehaviorParams,
        signal: &TraitSignal,
        decay_rate: f64,
    ) {
        let apply = |current: f64, delta: f64| -> f64 {
            let decayed = if delta.abs() < 1e-6 {
                (current - 0.5) * (1.0 - decay_rate) + 0.5
            } else {
                (current + delta).clamp(0.0, 1.0)
            };
            decayed
        };

        target.size = apply(target.size, signal.visual_deltas.size);
        target.warmth = apply(target.warmth, signal.visual_deltas.warmth);
        target.softness = apply(target.softness, signal.visual_deltas.softness);
        target.energy = apply(target.energy, signal.visual_deltas.energy);
        target.brightness = apply(target.brightness, signal.visual_deltas.brightness);
        target.creature = apply(target.creature, signal.visual_deltas.creature);
        target.complexity = apply(target.complexity, signal.visual_deltas.complexity);
        target.definition = apply(target.definition, signal.visual_deltas.definition);

        behavior_target.curiosity = apply(behavior_target.curiosity, signal.behavior_deltas.curiosity);
        behavior_target.playfulness = apply(behavior_target.playfulness, signal.behavior_deltas.playfulness);
        behavior_target.talkativeness = apply(behavior_target.talkativeness, signal.behavior_deltas.talkativeness);
        behavior_target.reactivity = apply(behavior_target.reactivity, signal.behavior_deltas.reactivity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_moves_trait() {
        let mut visual = VisualParams::default();
        let mut behavior = BehaviorParams::default();
        let signal = TraitSignal::new().with_visual("size", 0.3).with_behavior("curiosity", 0.2);
        TraitEvolution::apply_signal(&mut visual, &mut behavior, &signal, 0.0);
        assert!((visual.size - 0.8).abs() < 1e-6);
        assert!((behavior.curiosity - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_decay_pulls_to_default() {
        let mut visual = VisualParams { size: 0.9, ..Default::default() };
        let mut behavior = BehaviorParams::default();
        let signal = TraitSignal::new();
        TraitEvolution::apply_signal(&mut visual, &mut behavior, &signal, 0.5);
        assert!((visual.size - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_no_overflow() {
        let mut visual = VisualParams { size: 0.95, ..Default::default() };
        let mut behavior = BehaviorParams::default();
        let signal = TraitSignal::new().with_visual("size", 0.3);
        TraitEvolution::apply_signal(&mut visual, &mut behavior, &signal, 0.0);
        assert!(visual.size <= 1.0);
    }

    #[test]
    fn test_no_underflow() {
        let mut visual = VisualParams { size: 0.05, ..Default::default() };
        let mut behavior = BehaviorParams::default();
        let signal = TraitSignal::new().with_visual("size", -0.3);
        TraitEvolution::apply_signal(&mut visual, &mut behavior, &signal, 0.0);
        assert!(visual.size >= 0.0);
    }
}
