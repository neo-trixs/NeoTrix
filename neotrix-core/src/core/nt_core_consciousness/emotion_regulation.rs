use std::collections::HashMap;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegulationStrategy {
    Reappraisal,
    Suppression,
    Acceptance,
    Distraction,
    ProblemFocus,
    Mindfulness,
    SocialSharing,
}

#[derive(Debug, Clone)]
pub struct EmotionRegulation {
    pub strategies: HashMap<RegulationStrategy, f64>,
    pub selected_strategy: Option<RegulationStrategy>,
    pub regulation_history: VecDeque<(RegulationStrategy, f64, u64)>,
    pub max_history: usize,
    pub flexibility: f64,
    pub default_strategy: RegulationStrategy,
    pub cycle_count: u64,
    pub intensity_threshold: f64,
    pub reappraisal_bias: f64,
}

impl EmotionRegulation {
    pub fn new() -> Self {
        let mut strategies = HashMap::new();
        for s in [
            RegulationStrategy::Reappraisal,
            RegulationStrategy::Suppression,
            RegulationStrategy::Acceptance,
            RegulationStrategy::Distraction,
            RegulationStrategy::ProblemFocus,
            RegulationStrategy::Mindfulness,
            RegulationStrategy::SocialSharing,
        ] {
            strategies.insert(s, 0.5);
        }
        EmotionRegulation {
            strategies,
            selected_strategy: None,
            regulation_history: VecDeque::new(),
            max_history: 100,
            flexibility: 0.5,
            default_strategy: RegulationStrategy::Acceptance,
            cycle_count: 0,
            intensity_threshold: 0.7,
            reappraisal_bias: 0.6,
        }
    }

    pub fn select_strategy(
        &mut self,
        emotion_label: &str,
        intensity: f64,
        context: &str,
    ) -> RegulationStrategy {
        self.cycle_count += 1;

        if self.flexibility < 0.3 {
            return self.default_strategy;
        }

        let emotion_lower = emotion_label.to_lowercase();
        let context_lower = context.to_lowercase();
        let controllable = context_lower.contains("controllable")
            || context_lower.contains("action")
            || context_lower.contains("solve")
            || context_lower.contains("fix");
        let uncontrollable = context_lower.contains("uncontrollable")
            || context_lower.contains("loss")
            || context_lower.contains("inevitable")
            || context_lower.contains("cannot");

        if intensity > self.intensity_threshold {
            if emotion_lower.contains("fear")
                || emotion_lower.contains("anger")
                || emotion_lower.contains("panic")
            {
                let strategy = if self.reappraisal_bias > 0.5 {
                    RegulationStrategy::Distraction
                } else {
                    RegulationStrategy::Suppression
                };
                self.selected_strategy = Some(strategy);
                return strategy;
            }
        }

        if emotion_lower.contains("sad")
            || emotion_lower.contains("grief")
            || emotion_lower.contains("lonely")
        {
            self.selected_strategy = Some(RegulationStrategy::SocialSharing);
            return RegulationStrategy::SocialSharing;
        }

        if intensity > 0.3 && intensity <= 0.7 && controllable {
            self.selected_strategy = Some(RegulationStrategy::ProblemFocus);
            return RegulationStrategy::ProblemFocus;
        }

        if intensity > 0.0 && intensity <= 0.7 && uncontrollable {
            let strategy = if self.reappraisal_bias > 0.5 {
                RegulationStrategy::Reappraisal
            } else {
                RegulationStrategy::Acceptance
            };
            self.selected_strategy = Some(strategy);
            return strategy;
        }

        self.selected_strategy = Some(self.default_strategy);
        self.default_strategy
    }

    pub fn apply_strategy(&mut self, strategy: RegulationStrategy, outcome_valence: f64) -> f64 {
        let proficiency = self.strategies.get(&strategy).copied().unwrap_or(0.5);
        let effectiveness = proficiency * (1.0 - (outcome_valence - 0.5).abs());
        let noise = ((self
            .cycle_count
            .wrapping_mul(1103515245)
            .wrapping_add(12345)
            % 100) as f64
            / 100.0)
            * 0.1
            - 0.05;
        let final_effectiveness = (effectiveness + noise).clamp(0.0, 1.0);

        self.regulation_history
            .push_back((strategy, final_effectiveness, self.cycle_count));
        while self.regulation_history.len() > self.max_history {
            self.regulation_history.pop_front();
        }

        final_effectiveness
    }

    pub fn update_proficiency(&mut self, strategy: RegulationStrategy, effectiveness: f64) {
        let learning_rate = 0.1;
        let entry = self.strategies.entry(strategy).or_insert(0.5);
        let delta = learning_rate * (effectiveness - *entry);
        *entry = (*entry + delta).clamp(0.0, 1.0);
    }

    pub fn suggest_switch(
        &self,
        current_intensity: f64,
        duration_ms: u64,
    ) -> Option<RegulationStrategy> {
        let current = self.selected_strategy?;
        let recent: Vec<_> = self
            .regulation_history
            .iter()
            .rev()
            .take_while(|(s, _, _)| *s == current)
            .collect();

        if recent.is_empty() {
            return None;
        }

        if duration_ms > 5000 && current_intensity > self.intensity_threshold {
            let switch_to = match current {
                RegulationStrategy::Suppression => RegulationStrategy::Acceptance,
                RegulationStrategy::Distraction => RegulationStrategy::ProblemFocus,
                RegulationStrategy::Reappraisal => RegulationStrategy::Mindfulness,
                _ => self.default_strategy,
            };
            return Some(switch_to);
        }

        None
    }

    pub fn set_flexibility(&mut self, flexibility: f64) {
        self.flexibility = flexibility.clamp(0.0, 1.0);
    }

    pub fn set_strategy(&mut self, strategy: RegulationStrategy) {
        self.selected_strategy = Some(strategy);
    }

    pub fn reset(&mut self) {
        for v in self.strategies.values_mut() {
            *v = 0.5;
        }
        self.selected_strategy = None;
        self.regulation_history.clear();
        self.cycle_count = 0;
        self.flexibility = 0.5;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_regulation_defaults() {
        let er = EmotionRegulation::new();
        assert_eq!(er.max_history, 100);
        assert_eq!(er.flexibility, 0.5);
        assert_eq!(er.default_strategy, RegulationStrategy::Acceptance);
        assert_eq!(er.intensity_threshold, 0.7);
        assert_eq!(er.reappraisal_bias, 0.6);
        assert_eq!(er.strategies.len(), 7);
        for v in er.strategies.values() {
            assert!((*v - 0.5).abs() < 1e-6);
        }
        assert!(er.selected_strategy.is_none());
        assert!(er.regulation_history.is_empty());
        assert_eq!(er.cycle_count, 0);
    }

    #[test]
    fn test_select_strategy_high_intensity_fear_uses_suppression_or_distraction() {
        let mut er = EmotionRegulation::new();
        let s = er.select_strategy("fear", 0.9, "uncontrollable threat");
        assert!(s == RegulationStrategy::Suppression || s == RegulationStrategy::Distraction);
        assert_eq!(er.selected_strategy, Some(s));
    }

    #[test]
    fn test_select_strategy_sadness_uses_social_sharing() {
        let mut er = EmotionRegulation::new();
        let s = er.select_strategy("sadness", 0.5, "loss of opportunity");
        assert_eq!(s, RegulationStrategy::SocialSharing);
    }

    #[test]
    fn test_select_strategy_moderate_controllable_uses_problem_focus() {
        let mut er = EmotionRegulation::new();
        let s = er.select_strategy("frustration", 0.5, "controllable situation");
        assert_eq!(s, RegulationStrategy::ProblemFocus);
    }

    #[test]
    fn test_select_strategy_moderate_uncontrollable_uses_acceptance_or_reappraisal() {
        let mut er = EmotionRegulation::new();
        let s = er.select_strategy("sadness", 0.4, "uncontrollable loss");
        assert!(s == RegulationStrategy::Acceptance || s == RegulationStrategy::Reappraisal);
    }

    #[test]
    fn test_apply_strategy_returns_effectiveness() {
        let mut er = EmotionRegulation::new();
        let eff = er.apply_strategy(RegulationStrategy::Reappraisal, 0.7);
        assert!(eff >= 0.0 && eff <= 1.0);
    }

    #[test]
    fn test_apply_strategy_records_history() {
        let mut er = EmotionRegulation::new();
        er.apply_strategy(RegulationStrategy::Mindfulness, 0.6);
        assert_eq!(er.regulation_history.len(), 1);
        let (s, _, _) = er.regulation_history.front().unwrap();
        assert_eq!(*s, RegulationStrategy::Mindfulness);
    }

    #[test]
    fn test_update_proficiency_increases() {
        let mut er = EmotionRegulation::new();
        let prev = er.strategies[&RegulationStrategy::Reappraisal];
        er.update_proficiency(RegulationStrategy::Reappraisal, 0.9);
        let curr = er.strategies[&RegulationStrategy::Reappraisal];
        assert!(curr > prev);
    }

    #[test]
    fn test_update_proficiency_decreases() {
        let mut er = EmotionRegulation::new();
        let prev = er.strategies[&RegulationStrategy::Suppression];
        er.update_proficiency(RegulationStrategy::Suppression, 0.1);
        let curr = er.strategies[&RegulationStrategy::Suppression];
        assert!(curr < prev);
    }

    #[test]
    fn test_suggest_switch_long_duration() {
        let mut er = EmotionRegulation::new();
        er.select_strategy("fear", 0.9, "threat");
        er.apply_strategy(RegulationStrategy::Suppression, 0.3);
        let suggestion = er.suggest_switch(0.8, 6000);
        assert!(suggestion.is_some());
    }

    #[test]
    fn test_set_flexibility() {
        let mut er = EmotionRegulation::new();
        er.set_flexibility(0.8);
        assert!((er.flexibility - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_strategy_manual_override() {
        let mut er = EmotionRegulation::new();
        er.set_strategy(RegulationStrategy::Mindfulness);
        assert_eq!(er.selected_strategy, Some(RegulationStrategy::Mindfulness));
    }

    #[test]
    fn test_low_flexibility_prefers_default() {
        let mut er = EmotionRegulation::new();
        er.set_flexibility(0.2);
        let s = er.select_strategy("fear", 0.9, "threat");
        assert_eq!(s, er.default_strategy);
    }

    #[test]
    fn test_reset() {
        let mut er = EmotionRegulation::new();
        er.select_strategy("fear", 0.9, "threat");
        er.apply_strategy(RegulationStrategy::Suppression, 0.3);
        er.update_proficiency(RegulationStrategy::Reappraisal, 0.9);
        er.reset();
        assert_eq!(er.cycle_count, 0);
        assert!(er.selected_strategy.is_none());
        assert!(er.regulation_history.is_empty());
        for v in er.strategies.values() {
            assert!((*v - 0.5).abs() < 1e-6);
        }
        assert!((er.flexibility - 0.5).abs() < 1e-6);
    }
}
