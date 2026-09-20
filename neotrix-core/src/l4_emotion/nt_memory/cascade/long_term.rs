#![forbid(unsafe_code)]

//! Long-Term Store — distilled rules with confidence decay and reinforcement.
//!
//! Tier 5 of the five-tier cascade. Holds distilled, generalised rules
//! that have survived multiple promotion cycles. Confidence decays over time
//! but is reinforced each time the rule is confirmed.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

/// A distilled long-term rule with confidence tracking.
#[derive(Debug, Clone)]
pub struct Rule {
    pub id: u64,
    pub rule: String,
    pub confidence: f64,
    pub created_at: Instant,
    pub last_reinforced: Instant,
    pub reinforcement_count: u32,
    pub decay_rate: f64,
    pub source_episodes: Vec<u64>,
}

/// LongTermStore: distilled rules, confidence decay + reinforcement.
#[derive(Debug)]
pub struct LongTermStore {
    rules: BTreeMap<u64, Rule>,
    max_capacity: usize,
    next_id: u64,
    default_decay_rate: f64,
    decay_interval: Duration,
}

impl Default for LongTermStore {
    fn default() -> Self {
        Self {
            rules: BTreeMap::new(),
            max_capacity: 10_000,
            next_id: 1,
            default_decay_rate: 0.01,
            decay_interval: Duration::from_secs(3600),
        }
    }
}

impl LongTermStore {
    pub fn new(max_capacity: usize, default_decay_rate: f64, decay_interval: Duration) -> Self {
        Self {
            rules: BTreeMap::new(),
            max_capacity,
            next_id: 1,
            default_decay_rate,
            decay_interval,
        }
    }

    /// Distill and store a new rule.
    pub fn distill(
        &mut self,
        rule_text: String,
        initial_confidence: f64,
        source_episodes: Vec<u64>,
    ) -> u64 {
        let now = Instant::now();
        let id = self.next_id;
        self.next_id += 1;

        self.rules.insert(
            id,
            Rule {
                id,
                rule: rule_text,
                confidence: initial_confidence,
                created_at: now,
                last_reinforced: now,
                reinforcement_count: 0,
                decay_rate: self.default_decay_rate,
                source_episodes,
            },
        );
        id
    }

    /// Reinforce a rule, boosting its confidence.
    pub fn reinforce(&mut self, id: u64, boost: f64) -> bool {
        if let Some(rule) = self.rules.get_mut(&id) {
            rule.confidence = (rule.confidence + boost).min(1.0);
            rule.last_reinforced = Instant::now();
            rule.reinforcement_count += 1;
            true
        } else {
            false
        }
    }

    /// Recall a rule by id, applying any pending decay first.
    pub fn recall(&mut self, id: u64) -> Option<&Rule> {
        self.apply_decay(id);
        self.rules.get(&id)
    }

    /// Recall a rule by id (immutable — no decay applied).
    pub fn peek(&self, id: u64) -> Option<&Rule> {
        self.rules.get(&id)
    }

    /// Return all rules above a confidence threshold.
    pub fn confident_rules(&self, threshold: f64) -> Vec<&Rule> {
        self.rules
            .values()
            .filter(|r| r.confidence >= threshold)
            .collect()
    }

    /// Search rules by substring match on the rule text.
    pub fn search(&self, query: &str) -> Vec<&Rule> {
        self.rules
            .values()
            .filter(|r| r.rule.contains(query))
            .collect()
    }

    /// Apply global decay to all rules based on elapsed time.
    pub fn decay_all(&mut self) {
        let now = Instant::now();
        for rule in self.rules.values_mut() {
            let elapsed_secs = now.duration_since(rule.last_reinforced).as_secs_f64();
            let intervals = elapsed_secs / self.decay_interval.as_secs_f64();
            let decay = rule.decay_rate * intervals;
            rule.confidence = (rule.confidence - decay).max(0.0);
        }
        self.prune_low_confidence(0.05);
    }

    /// Remove a rule by id.
    pub fn remove(&mut self, id: u64) -> Option<Rule> {
        self.rules.remove(&id)
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    fn apply_decay(&mut self, id: u64) {
        if let Some(rule) = self.rules.get_mut(&id) {
            let now = Instant::now();
            let elapsed_secs = now.duration_since(rule.last_reinforced).as_secs_f64();
            let intervals = elapsed_secs / self.decay_interval.as_secs_f64();
            let decay = rule.decay_rate * intervals;
            rule.confidence = (rule.confidence - decay).max(0.0);
        }
    }

    fn prune_low_confidence(&mut self, threshold: f64) {
        let ids_to_remove: Vec<u64> = self
            .rules
            .iter()
            .filter(|(_, r)| r.confidence < threshold)
            .map(|(id, _)| *id)
            .collect();
        for id in ids_to_remove {
            self.rules.remove(&id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distill_and_recall() {
        let mut lt = LongTermStore::default();
        let id = lt.distill("always validate input".into(), 0.9, vec![]);
        let rule = lt.recall(id).unwrap();
        assert_eq!(rule.rule, "always validate input");
        assert_eq!(rule.reinforcement_count, 0);
    }

    #[test]
    fn reinforce_boosts_confidence() {
        let mut lt = LongTermStore::default();
        let id = lt.distill("rule".into(), 0.5, vec![]);
        lt.reinforce(id, 0.3);
        let rule = lt.peek(id).unwrap();
        assert!((rule.confidence - 0.8).abs() < 0.01);
        assert_eq!(rule.reinforcement_count, 1);
    }

    #[test]
    fn confident_rules_filter() {
        let mut lt = LongTermStore::default();
        lt.distill("weak".into(), 0.2, vec![]);
        lt.distill("strong".into(), 0.9, vec![]);
        let strong = lt.confident_rules(0.5);
        assert_eq!(strong.len(), 1);
        assert_eq!(strong[0].rule, "strong");
    }
}
