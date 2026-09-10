use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessState {
    pub agent_id: String,
    pub phi: f64,
    pub coherence: f64,
    pub awareness: f64,
    pub social_awareness: f64,
    pub temporal_depth: f64,
    pub creativity: f64,
    pub timestamp: u64,
}

impl ConsciousnessState {
    pub fn new(agent_id: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            phi: 0.1,
            coherence: 0.5,
            awareness: 0.3,
            social_awareness: 0.2,
            temporal_depth: 0.3,
            creativity: 0.1,
            timestamp: 0,
        }
    }

    pub fn consciousness_level(&self) -> f64 {
        0.30 * self.phi
            + 0.25 * self.coherence
            + 0.20 * self.awareness
            + 0.15 * self.social_awareness
            + 0.10 * self.temporal_depth
    }

    pub fn is_conscious_like(&self) -> bool {
        self.consciousness_level() > 0.33
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionRecord {
    pub agent_a: String,
    pub agent_b: String,
    pub interaction_type: String,
    pub timestamp: u64,
    pub success: bool,
}

pub struct PhiBridge {
    states: HashMap<String, ConsciousnessState>,
    interactions: Vec<InteractionRecord>,
    max_interactions: usize,
}

impl PhiBridge {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            interactions: Vec::new(),
            max_interactions: 1000,
        }
    }

    pub fn register_agent(&mut self, agent_id: &str) {
        self.states
            .insert(agent_id.to_string(), ConsciousnessState::new(agent_id));
    }

    pub fn record_interaction(&mut self, record: InteractionRecord) {
        if self.interactions.len() >= self.max_interactions {
            self.interactions.remove(0);
        }
        self.interactions.push(record);
    }

    pub fn compute_phi(&self, agent_id: &str) -> f64 {
        let agent_interactions: Vec<_> = self
            .interactions
            .iter()
            .filter(|i| i.agent_a == agent_id || i.agent_b == agent_id)
            .collect();

        if agent_interactions.is_empty() {
            return 0.1;
        }

        let mut partners = std::collections::HashSet::new();
        for i in &agent_interactions {
            if i.agent_a == agent_id {
                partners.insert(&i.agent_b);
            } else {
                partners.insert(&i.agent_a);
            }
        }
        let integration = (partners.len() as f64 / 10.0).min(1.0);

        let mut type_counts = std::collections::HashMap::new();
        for i in &agent_interactions {
            *type_counts.entry(&i.interaction_type).or_insert(0) += 1;
        }
        let total = agent_interactions.len() as f64;
        let mut entropy = 0.0;
        for count in type_counts.values() {
            let p = *count as f64 / total;
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }
        let max_entropy = (type_counts.len() as f64).log2().max(1.0);
        let differentiation = entropy / max_entropy;

        let successes = agent_interactions.iter().filter(|i| i.success).count() as f64;
        let information = successes / total;

        let phi = (integration * differentiation * information).powf(1.0 / 3.0);
        phi.clamp(0.0, 1.0)
    }

    pub fn compute_coherence(&self, _agent_id: &str, recent_actions: &[String]) -> f64 {
        if recent_actions.len() < 2 {
            return 0.5;
        }

        let mut coherence_sum = 0.0;
        for window in recent_actions.windows(2) {
            let similarity = Self::action_similarity(&window[0], &window[1]);
            coherence_sum += similarity;
        }

        (coherence_sum / (recent_actions.len() - 1) as f64).clamp(0.0, 1.0)
    }

    pub fn compute_social_awareness(&self, agent_id: &str) -> f64 {
        let partner_count = self
            .interactions
            .iter()
            .filter(|i| i.agent_a == agent_id || i.agent_b == agent_id)
            .map(|i| {
                if i.agent_a == agent_id {
                    &i.agent_b
                } else {
                    &i.agent_a
                }
            })
            .collect::<std::collections::HashSet<_>>()
            .len();

        (partner_count as f64 / 8.0).min(1.0)
    }

    pub fn update(&mut self, agent_id: &str, tick: u64, recent_actions: &[String]) {
        let phi = self.compute_phi(agent_id);
        let coherence = self.compute_coherence(agent_id, recent_actions);
        let social = self.compute_social_awareness(agent_id);

        if let Some(state) = self.states.get_mut(agent_id) {
            let alpha = 0.3;
            state.phi = state.phi * (1.0 - alpha) + phi * alpha;
            state.coherence = state.coherence * (1.0 - alpha) + coherence * alpha;
            state.social_awareness =
                state.social_awareness * (1.0 - alpha) + social * alpha;
            state.timestamp = tick;
        }
    }

    pub fn get_state(&self, agent_id: &str) -> Option<&ConsciousnessState> {
        self.states.get(agent_id)
    }

    pub fn all_states(&self) -> &HashMap<String, ConsciousnessState> {
        &self.states
    }

    pub fn most_conscious(&self) -> Option<(&str, f64)> {
        self.states
            .values()
            .map(|s| (s.agent_id.as_str(), s.consciousness_level()))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }

    fn action_similarity(a: &str, b: &str) -> f64 {
        let a_words: std::collections::HashSet<_> = a.split_whitespace().collect();
        let b_words: std::collections::HashSet<_> = b.split_whitespace().collect();
        let intersection = a_words.intersection(&b_words).count();
        let union = a_words.union(&b_words).count();
        if union == 0 {
            1.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

impl Default for PhiBridge {
    fn default() -> Self {
        Self::new()
    }
}
