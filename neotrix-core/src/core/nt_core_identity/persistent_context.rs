use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ContextFingerprint {
    pub topic_hashes: Vec<u64>,
    pub interaction_count: u64,
    pub last_active: u64,
}

#[derive(Debug, Clone)]
pub struct UserModel {
    pub preferences: HashMap<String, f64>,
    pub interaction_history: Vec<String>,
    pub relationship_quality: f64,
}

#[derive(Debug, Clone)]
pub struct PersistentContext {
    pub user_model: UserModel,
    pub fingerprint: ContextFingerprint,
    pub coherence_score: f64,
}

impl PersistentContext {
    pub fn new() -> Self {
        Self {
            user_model: UserModel {
                preferences: HashMap::new(),
                interaction_history: Vec::new(),
                relationship_quality: 0.5,
            },
            fingerprint: ContextFingerprint {
                topic_hashes: Vec::new(),
                interaction_count: 0,
                last_active: 0,
            },
            coherence_score: 1.0,
        }
    }

    pub fn record_interaction(&mut self, topic: &str, quality: f64) {
        self.fingerprint.interaction_count += 1;
        let hash: u64 = topic
            .bytes()
            .fold(0u64, |h, b| h.wrapping_mul(31).wrapping_add(b as u64));
        self.fingerprint.topic_hashes.push(hash);
        self.user_model.interaction_history.push(topic.to_string());
        self.user_model.relationship_quality =
            self.user_model.relationship_quality * 0.9 + quality * 0.1;
        self.update_coherence();
    }

    pub fn set_preference(&mut self, key: &str, value: f64) {
        self.user_model
            .preferences
            .insert(key.to_string(), value.clamp(0.0, 1.0));
    }

    fn update_coherence(&mut self) {
        let topic_diversity = self.fingerprint.topic_hashes.len() as f64
            / self.fingerprint.interaction_count.max(1) as f64;
        self.coherence_score = (self.user_model.relationship_quality + topic_diversity) / 2.0;
    }

    pub fn continuity_score(&self, other: &Self) -> f64 {
        let overlap: usize = self
            .fingerprint
            .topic_hashes
            .iter()
            .filter(|h| other.fingerprint.topic_hashes.contains(h))
            .count();
        let max = self
            .fingerprint
            .topic_hashes
            .len()
            .max(other.fingerprint.topic_hashes.len());
        if max == 0 {
            0.0
        } else {
            overlap as f64 / max as f64
        }
    }
}

impl Default for PersistentContext {
    fn default() -> Self {
        Self::new()
    }
}
