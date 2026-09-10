use std::collections::HashMap;

use crate::agents::sim_agent::{AgentAction, Personality};
use crate::society::culture::Culture;

/// Record of an observed action from another agent
#[derive(Debug, Clone)]
pub struct ObservedAction {
    pub observer_id: String,
    pub actor_id: String,
    pub action: AgentAction,
    pub success: bool,
    pub tick: u64,
}

/// SocialLearning — Adapter 4
///
/// When an agent observes another agent's action:
/// - If the action was successful, copy the action pattern (bias future decisions)
/// - Update personality traits based on observed success
/// - Share knowledge through culture.memes
///
/// Sources:
/// - Generative Agents: social learning through observation and imitation
/// - Project Sid (PIANO): agents learn from successful behaviors of peers
/// - BDI: social beliefs updated through interaction observation
pub struct SocialLearning {
    /// action_pattern -> success_count (per observer agent)
    action_patterns: HashMap<String, HashMap<String, u32>>,
    /// agent_id -> (trait_name -> adjustment)
    personality_adjustments: HashMap<String, Vec<(String, f32)>>,
    _max_pattern_history: usize,
    learning_rate: f32,
    mimicry_threshold: f32,
}

impl SocialLearning {
    pub fn new() -> Self {
        Self {
            action_patterns: HashMap::new(),
            personality_adjustments: HashMap::new(),
            _max_pattern_history: 50,
            learning_rate: 0.1,
            mimicry_threshold: 0.6,
        }
    }

    /// Record an observed action from another agent.
    pub fn observe_action(
        &mut self,
        observer_id: &str,
        actor_id: &str,
        action: &AgentAction,
        success: bool,
        tick: u64,
    ) -> ObservedAction {
        let record = ObservedAction {
            observer_id: observer_id.to_string(),
            actor_id: actor_id.to_string(),
            action: action.clone(),
            success,
            tick,
        };

        // Track action pattern success
        let pattern = action_pattern_key(action);
        let patterns = self.action_patterns
            .entry(observer_id.to_string())
            .or_insert_with(HashMap::new);

        if success {
            *patterns.entry(pattern).or_insert(0) += 1;
        }

        record
    }

    /// Get the bias toward a specific action based on observed success.
    /// Returns a value 0.0-1.0 indicating how likely the agent should be
    /// to mimic this action.
    pub fn mimicry_bias(&self, observer_id: &str, action: &AgentAction) -> f32 {
        let pattern = action_pattern_key(action);
        if let Some(patterns) = self.action_patterns.get(observer_id) {
            if let Some(&success_count) = patterns.get(&pattern) {
                let total: u32 = patterns.values().sum();
                if total > 0 {
                    return (success_count as f32 / total as f32).min(self.mimicry_threshold);
                }
            }
        }
        0.0
    }

    /// Update personality based on observed successful action.
    /// Cooperative actions increase cooperativeness, aggressive actions increase aggression, etc.
    pub fn update_personality_from_observation(
        &mut self,
        observer_id: &str,
        action: &AgentAction,
        success: bool,
    ) -> Vec<(String, f32)> {
        if !success {
            return vec![];
        }

        let mut adjustments = Vec::new();

        match action {
            AgentAction::Talk { .. } | AgentAction::Trade { .. } => {
                // Social actions → boost cooperativeness
                let adj = ("cooperativeness".to_string(), self.learning_rate);
                adjustments.push(adj.clone());
                self.personality_adjustments
                    .entry(observer_id.to_string())
                    .or_default()
                    .push(adj);
            }
            AgentAction::Attack { .. } => {
                // Aggressive actions → boost aggression (if successful)
                let adj = ("aggression".to_string(), self.learning_rate * 0.5);
                adjustments.push(adj.clone());
                self.personality_adjustments
                    .entry(observer_id.to_string())
                    .or_default()
                    .push(adj);
            }
            AgentAction::Explore { .. } => {
                // Exploration → boost curiosity
                let adj = ("curiosity".to_string(), self.learning_rate);
                adjustments.push(adj.clone());
                self.personality_adjustments
                    .entry(observer_id.to_string())
                    .or_default()
                    .push(adj);
            }
            AgentAction::Build { .. } => {
                // Building → boost openness
                let adj = ("openness".to_string(), self.learning_rate);
                adjustments.push(adj.clone());
                self.personality_adjustments
                    .entry(observer_id.to_string())
                    .or_default()
                    .push(adj);
            }
            _ => {}
        }

        adjustments
    }

    /// Apply accumulated personality adjustments to a Personality.
    pub fn apply_adjustments(&self, observer_id: &str, personality: &mut Personality) {
        if let Some(adj) = self.personality_adjustments.get(observer_id) {
            for (trait_name, delta) in adj {
                match trait_name.as_str() {
                    "cooperativeness" => {
                        personality.cooperativeness = (personality.cooperativeness + delta).clamp(0.0, 1.0);
                    }
                    "aggression" => {
                        personality.aggression = (personality.aggression + delta).clamp(0.0, 1.0);
                    }
                    "curiosity" => {
                        personality.curiosity = (personality.curiosity + delta).clamp(0.0, 1.0);
                    }
                    "openness" => {
                        personality.openness = (personality.openness + delta).clamp(0.0, 1.0);
                    }
                    "sociability" => {
                        personality.sociability = (personality.sociability + delta).clamp(0.0, 1.0);
                    }
                    _ => {}
                }
            }
        }
    }

    /// Share knowledge through culture memes.
    /// Creates a meme describing the observed successful action.
    pub fn share_knowledge(
        &self,
        culture: &mut Culture,
        actor_id: &str,
        action: &AgentAction,
        observer_id: &str,
        tick: u64,
    ) {
        let action_desc = format!("{:?}", action);
        let meme_content = format!("learned:{}:{}", actor_id, action_desc);
        let meme = culture.create_meme(&meme_content, observer_id, tick);
        let meme_id = meme.id.clone();
        culture.spread_meme(&meme_id, actor_id, 0.3);
    }

    /// Get the most successful action pattern for an observer.
    pub fn best_pattern(&self, observer_id: &str) -> Option<String> {
        self.action_patterns.get(observer_id)
            .and_then(|patterns| {
                patterns.iter()
                    .max_by_key(|(_, &count)| count)
                    .map(|(pattern, _)| pattern.clone())
            })
    }

    /// Get action pattern stats for an observer
    pub fn pattern_stats(&self, observer_id: &str) -> Option<&HashMap<String, u32>> {
        self.action_patterns.get(observer_id)
    }

    pub fn len(&self) -> usize {
        self.action_patterns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.action_patterns.is_empty()
    }
}

impl Default for SocialLearning {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert an AgentAction to a pattern key for tracking
fn action_pattern_key(action: &AgentAction) -> String {
    match action {
        AgentAction::Move { .. } => "move".to_string(),
        AgentAction::Eat { .. } => "eat".to_string(),
        AgentAction::Rest => "rest".to_string(),
        AgentAction::Talk { .. } => "talk".to_string(),
        AgentAction::Attack { .. } => "attack".to_string(),
        AgentAction::Trade { .. } => "trade".to_string(),
        AgentAction::Explore { .. } => "explore".to_string(),
        AgentAction::Build { .. } => "build".to_string(),
        AgentAction::Harvest { .. } => "harvest".to_string(),
        AgentAction::Think => "think".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::sim_agent::{AgentAction, Personality};
    use crate::society::culture::Culture;
    use crate::foundation::math_bridge::Vec2;

    #[test]
    fn new_has_no_patterns() {
        let sl = SocialLearning::new();
        assert!(sl.is_empty());
        assert_eq!(sl.len(), 0);
    }

    #[test]
    fn observe_action_records_success() {
        let mut sl = SocialLearning::new();
        sl.observe_action("obs", "actor", &AgentAction::Rest, true, 10);
        assert!(!sl.is_empty());
        assert_eq!(sl.len(), 1);
    }

    #[test]
    fn mimicry_bias_returns_zero_when_no_data() {
        let sl = SocialLearning::new();
        assert_eq!(sl.mimicry_bias("obs", &AgentAction::Rest), 0.0);
    }

    #[test]
    fn mimicry_bias_returns_value_after_observation() {
        let mut sl = SocialLearning::new();
        for _ in 0..5 {
            sl.observe_action("obs", "actor", &AgentAction::Rest, true, 10);
        }
        let bias = sl.mimicry_bias("obs", &AgentAction::Rest);
        assert!(bias > 0.0);
    }

    #[test]
    fn mimicry_bias_capped_at_threshold() {
        let mut sl = SocialLearning::new();
        sl.mimicry_threshold = 0.3;
        for _ in 0..100 {
            sl.observe_action("obs", "actor", &AgentAction::Rest, true, 10);
        }
        let bias = sl.mimicry_bias("obs", &AgentAction::Rest);
        assert!(bias <= 0.3);
    }

    #[test]
    fn personality_update_from_social_action() {
        let mut sl = SocialLearning::new();
        let adj = sl.update_personality_from_observation(
            "obs",
            &AgentAction::Talk { target_id: "a1".into(), message: "hi".into() },
            true,
        );
        assert!(!adj.is_empty());
        assert_eq!(adj[0].0, "cooperativeness");
    }

    #[test]
    fn personality_no_update_on_failure() {
        let mut sl = SocialLearning::new();
        let adj = sl.update_personality_from_observation(
            "obs",
            &AgentAction::Talk { target_id: "a1".into(), message: "hi".into() },
            false,
        );
        assert!(adj.is_empty());
    }

    #[test]
    fn apply_adjustments_modifies_personality() {
        let mut sl = SocialLearning::new();
        sl.update_personality_from_observation("obs", &AgentAction::Rest, true);
        // rest doesn't trigger personality change, so let's use explore
        sl.update_personality_from_observation("obs", &AgentAction::Explore { direction: Vec2::new(1.0, 0.0) }, true);

        let mut p = Personality::default();
        let orig = p.curiosity;
        sl.apply_adjustments("obs", &mut p);
        assert!(p.curiosity > orig);
    }

    #[test]
    fn share_knowledge_creates_meme() {
        let sl = SocialLearning::new();
        let mut culture = Culture::new();
        sl.share_knowledge(
            &mut culture,
            "actor",
            &AgentAction::Rest,
            "observer",
            10,
        );
        assert_eq!(culture.memes.len(), 1);
        assert!(culture.memes[0].content.contains("learned"));
    }

    #[test]
    fn best_pattern_returns_most_successful() {
        let mut sl = SocialLearning::new();
        for _ in 0..5 {
            sl.observe_action("obs", "actor", &AgentAction::Rest, true, 10);
        }
        sl.observe_action("obs", "actor", &AgentAction::Think, true, 10);

        let best = sl.best_pattern("obs");
        assert_eq!(best.as_deref(), Some("rest"));
    }

    #[test]
    fn action_pattern_keys() {
        assert_eq!(action_pattern_key(&AgentAction::Rest), "rest");
        assert_eq!(action_pattern_key(&AgentAction::Think), "think");
        assert_eq!(action_pattern_key(&AgentAction::Explore { direction: Vec2::new(1.0, 0.0) }), "explore");
        assert_eq!(
            action_pattern_key(&AgentAction::Talk { target_id: "a".into(), message: "hi".into() }),
            "talk"
        );
    }
}
