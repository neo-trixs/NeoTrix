use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A causal link between cause and effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalLink {
    pub cause: String,
    pub effect: String,
    pub confidence: f32,
    pub strength: f32,
    pub observations: u32,
    pub last_observed: u64,
}

/// An intervention and its observed effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionRecord {
    pub action: String,
    pub effects: Vec<(String, f32)>,
    pub tick: u64,
    pub context: String,
}

/// Causal memory: tracks cause-effect relationships and intervention effects
pub struct CausalMemory {
    causal_chains: Vec<CausalLink>,
    intervention_effects: Vec<InterventionRecord>,
    effect_index: HashMap<String, Vec<usize>>,
    cause_index: HashMap<String, Vec<usize>>,
    max_links: usize,
}

impl CausalMemory {
    pub fn new(max_links: usize) -> Self {
        Self {
            causal_chains: Vec::new(),
            intervention_effects: Vec::new(),
            effect_index: HashMap::new(),
            cause_index: HashMap::new(),
            max_links,
        }
    }

    /// Record that a cause led to an effect
    pub fn record_cause(&mut self, cause: String, effect: String, confidence: f32, tick: u64) {
        // Check if this link already exists
        if let Some(&idx) = self.cause_index.get(&cause).and_then(|indices| {
            indices.iter().find(|&&i| self.causal_chains[i].effect == effect)
        }) {
            let link = &mut self.causal_chains[idx];
            link.observations += 1;
            link.confidence = (link.confidence + confidence).min(1.0);
            link.strength = (link.strength + 0.05).min(1.0);
            link.last_observed = tick;
            return;
        }

        let idx = self.causal_chains.len();
        self.causal_chains.push(CausalLink {
            cause: cause.clone(),
            effect: effect.clone(),
            confidence,
            strength: 0.5,
            observations: 1,
            last_observed: tick,
        });
        self.cause_index.entry(cause).or_default().push(idx);
        self.effect_index.entry(effect).or_default().push(idx);
        self.prune();
    }

    /// Predict what effects a cause produces
    pub fn predict_effect(&self, cause: &str) -> Vec<(String, f32)> {
        self.cause_index
            .get(cause)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&i| {
                        let link = &self.causal_chains[i];
                        let score = link.confidence * link.strength;
                        if score > 0.1 {
                            Some((link.effect.clone(), score))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Predict what causes lead to a given effect
    pub fn predict_cause(&self, effect: &str) -> Vec<(String, f32)> {
        self.effect_index
            .get(effect)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&i| {
                        let link = &self.causal_chains[i];
                        let score = link.confidence * link.strength;
                        if score > 0.1 {
                            Some((link.cause.clone(), score))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Record an intervention and its observed effects
    pub fn intervene(&mut self, action: String, effects: Vec<(String, f32)>, tick: u64, context: &str) {
        self.intervention_effects.push(InterventionRecord {
            action: action.clone(),
            effects: effects.clone(),
            tick,
            context: context.to_string(),
        });

        // Also record causal links from the intervention
        for (effect, confidence) in &effects {
            self.record_cause(action.clone(), effect.clone(), *confidence, tick);
        }
    }

    /// Get predicted effects of an action based on intervention history
    pub fn intervention_effects(&self, action: &str) -> Vec<(String, f32)> {
        let records: Vec<&InterventionRecord> = self
            .intervention_effects
            .iter()
            .filter(|r| r.action == action)
            .collect();

        if records.is_empty() {
            return Vec::new();
        }

        // Aggregate effects across all interventions of this action
        let mut effect_scores: HashMap<String, (f32, u32)> = HashMap::new();
        for record in &records {
            for (effect, score) in &record.effects {
                let entry = effect_scores.entry(effect.clone()).or_insert((0.0, 0));
                entry.0 += score;
                entry.1 += 1;
            }
        }

        effect_scores
            .into_iter()
            .map(|(effect, (total, count))| (effect, total / count as f32))
            .collect()
    }

    /// Get all causal chains
    pub fn all_chains(&self) -> &[CausalLink] {
        &self.causal_chains
    }

    pub fn chain_count(&self) -> usize {
        self.causal_chains.len()
    }

    pub fn intervention_count(&self) -> usize {
        self.intervention_effects.len()
    }

    fn prune(&mut self) {
        if self.causal_chains.len() <= self.max_links {
            return;
        }
        self.causal_chains.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap()
                .then(b.observations.cmp(&a.observations))
        });
        self.causal_chains.truncate(self.max_links * 8 / 10);
        self.rebuild_indices();
    }

    fn rebuild_indices(&mut self) {
        self.cause_index.clear();
        self.effect_index.clear();
        for (i, link) in self.causal_chains.iter().enumerate() {
            self.cause_index
                .entry(link.cause.clone())
                .or_default()
                .push(i);
            self.effect_index
                .entry(link.effect.clone())
                .or_default()
                .push(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_and_predict_effect() {
        let mut mem = CausalMemory::new(100);
        mem.record_cause("rain".into(), "wet_ground".into(), 0.9, 0);
        mem.record_cause("rain".into(), "puddles".into(), 0.7, 0);
        let effects = mem.predict_effect("rain");
        assert_eq!(effects.len(), 2);
    }

    #[test]
    fn predict_cause_reverse() {
        let mut mem = CausalMemory::new(100);
        mem.record_cause("fire".into(), "smoke".into(), 0.9, 0);
        mem.record_cause("dust".into(), "smoke".into(), 0.4, 1);
        let causes = mem.predict_cause("smoke");
        assert_eq!(causes.len(), 2);
        assert!(causes[0].1 > causes[1].1);
    }

    #[test]
    fn duplicate_cause_strengthens() {
        let mut mem = CausalMemory::new(100);
        mem.record_cause("a".into(), "b".into(), 0.5, 0);
        mem.record_cause("a".into(), "b".into(), 0.5, 1);
        assert_eq!(mem.chain_count(), 1);
        assert!(mem.all_chains()[0].observations >= 2);
    }

    #[test]
    fn intervene_records_effects() {
        let mut mem = CausalMemory::new(100);
        mem.intervene(
            "build_wall".into(),
            vec![("safety".into(), 0.8), ("cost".into(), 0.3)],
            10,
            "defending base",
        );
        assert_eq!(mem.intervention_count(), 1);
        let effects = mem.intervention_effects("build_wall");
        assert_eq!(effects.len(), 2);
    }

    #[test]
    fn prune_respects_limit() {
        let mut mem = CausalMemory::new(5);
        for i in 0..10 {
            mem.record_cause(
                format!("cause_{}", i),
                format!("effect_{}", i),
                0.5,
                i,
            );
        }
        assert!(mem.chain_count() <= 5);
    }
}
