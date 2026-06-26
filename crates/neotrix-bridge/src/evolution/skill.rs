use std::collections::HashMap;
use crate::types::{Domain, IntentionVsa, WorldEffect, VsaLight};

const SKILL_BANK_CAPACITY: usize = 100;

#[derive(Debug, Clone)]
pub struct ExtractedSkill {
    pub id: u64,
    pub domain: Domain,
    pub action_pattern: String,
    pub trigger_conditions: Vec<String>,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub usage_count: u64,
    pub last_used_ms: i64,
    pub effectiveness_score: f64,
    pub parameter_template: serde_json::Value,
}

impl ExtractedSkill {
    pub fn utility_score(&self) -> f64 {
        self.success_rate * (self.usage_count as f64).ln_1p() * self.effectiveness_score
    }
}

#[derive(Debug)]
pub struct SkillExtractor {
    pub skills: Vec<ExtractedSkill>,
    pub vsa: VsaLight,
    pub next_id: u64,
    pub evolution_rounds: u64,
    usage_counts: HashMap<(Domain, String), u64>,
    success_counts: HashMap<(Domain, String), u64>,
    latency_samples: HashMap<(Domain, String), Vec<u64>>,
}

impl Default for SkillExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillExtractor {
    pub fn new() -> Self {
        Self {
            skills: Vec::with_capacity(SKILL_BANK_CAPACITY),
            vsa: VsaLight::new(256),
            next_id: 1,
            evolution_rounds: 0,
            usage_counts: HashMap::new(),
            success_counts: HashMap::new(),
            latency_samples: HashMap::new(),
        }
    }

    pub fn observe_actuation(&mut self, intention: &IntentionVsa, effect: &WorldEffect, latency_ms: u64) {
        let key = (intention.domain, intention.action.clone());
        *self.usage_counts.entry(key.clone()).or_insert(0) += 1;
        if effect.success {
            *self.success_counts.entry(key.clone()).or_insert(0) += 1;
        }
        let samples = self.latency_samples.entry(key.clone()).or_default();
        const MAX_LATENCY_SAMPLES: usize = 10000;
        if samples.len() > MAX_LATENCY_SAMPLES {
            samples.drain(0..MAX_LATENCY_SAMPLES / 5);
        }
        samples.push(latency_ms);
    }

    pub fn evolve(&mut self) -> Vec<ExtractedSkill> {
        self.evolution_rounds += 1;
        let mut new_skills = Vec::new();

        let mut domain_actions: HashMap<Domain, Vec<String>> = HashMap::new();
        for key in self.usage_counts.keys() {
            domain_actions.entry(key.0).or_default().push(key.1.clone());
        }

        for (domain, actions) in &domain_actions {
            for action in actions {
                let key = (*domain, action.clone());
                let total = self.usage_counts.get(&key).copied().unwrap_or(0);
                let successes = self.success_counts.get(&key).copied().unwrap_or(0);
                let success_rate = if total > 0 { successes as f64 / total as f64 } else { 0.0 };

                let avg_lat = self.latency_samples.get(&key)
                    .map(|s: &Vec<u64>| if s.is_empty() { 0.0 } else { s.iter().sum::<u64>() as f64 / s.len() as f64 })
                    .unwrap_or(0.0);

                if total >= 3 && success_rate > 0.3 {
                    let effectiveness = success_rate * (1.0 - (avg_lat / 10000.0).min(0.5));
                    let skill = ExtractedSkill {
                        id: self.next_id,
                        domain: *domain,
                        action_pattern: action.clone(),
                        trigger_conditions: vec![format!("domain:{}", domain.as_str())],
                        success_rate,
                        avg_latency_ms: avg_lat,
                        usage_count: total,
                        last_used_ms: chrono::Utc::now().timestamp_millis(),
                        effectiveness_score: effectiveness,
                        parameter_template: serde_json::json!({}),
                    };

                    self.next_id += 1;
                    new_skills.push(skill);
                }
            }
        }

        // Merge new skills into bank (update existing + add new, keep top K)
        for ns in new_skills {
            if let Some(existing) = self.skills.iter_mut()
                .find(|s| s.domain == ns.domain && s.action_pattern == ns.action_pattern)
            {
                existing.success_rate = existing.success_rate * 0.7 + ns.success_rate * 0.3;
                existing.avg_latency_ms = existing.avg_latency_ms * 0.7 + ns.avg_latency_ms * 0.3;
                existing.usage_count = ns.usage_count;
                existing.last_used_ms = ns.last_used_ms;
                existing.effectiveness_score = existing.effectiveness_score * 0.7 + ns.effectiveness_score * 0.3;
            } else {
                self.skills.push(ns);
            }
        }

        // Prune low-utility skills
        self.skills.sort_by(|a, b| b.utility_score().partial_cmp(&a.utility_score()).unwrap_or(std::cmp::Ordering::Equal));
        if self.skills.len() > SKILL_BANK_CAPACITY {
            self.skills.truncate(SKILL_BANK_CAPACITY);
        }

        self.skills.clone()
    }

    pub fn top_skills(&self, domain: Option<&Domain>, n: usize) -> Vec<&ExtractedSkill> {
        let mut filtered: Vec<&ExtractedSkill> = match domain {
            Some(d) => self.skills.iter().filter(|s| &s.domain == d).collect(),
            None => self.skills.iter().collect(),
        };
        filtered.sort_by(|a, b| b.utility_score().partial_cmp(&a.utility_score()).unwrap_or(std::cmp::Ordering::Equal));
        filtered.truncate(n);
        filtered
    }

    pub fn best_action(&self, domain: &Domain) -> Option<String> {
        self.top_skills(Some(domain), 1).first().map(|s| s.action_pattern.clone())
    }

    pub fn skill_count(&self) -> usize {
        self.skills.len()
    }

    pub fn summary(&self) -> HashMap<Domain, usize> {
        let mut m: HashMap<Domain, usize> = HashMap::new();
        for s in &self.skills {
            *m.entry(s.domain).or_insert(0) += 1;
        }
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::WorldEffect;

    fn intention(domain: Domain, action: &str) -> IntentionVsa {
        IntentionVsa {
            domain,
            action: action.into(),
            parameters: serde_json::json!({}),
            confidence: 0.9,
            urgency: 0.5,
        }
    }

    fn effect(success: bool) -> WorldEffect {
        WorldEffect { domain: Domain::System, description: "".into(), success, latency_ms: 0 }
    }

    #[test]
    fn test_observe_and_evolve() {
        let mut ex = SkillExtractor::new();
        let intent = intention(Domain::Crawl, "explore");
        for i in 0..10 {
            ex.observe_actuation(&intent, &effect(i < 8), 100);
        }
        let skills = ex.evolve();
        assert!(skills.len() >= 1);
        let s = &skills[0];
        assert!((s.success_rate - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_top_skills_filter() {
        let mut ex = SkillExtractor::new();
        for a in &["explore", "deepen", "seed"] {
            let intent = intention(Domain::Crawl, a);
            for _ in 0..5 {
                ex.observe_actuation(&intent, &effect(true), 50);
            }
        }
        ex.evolve();
        let top = ex.top_skills(Some(&Domain::Crawl), 2);
        assert_eq!(top.len(), 2);
        assert!(top[0].usage_count >= top[1].usage_count);
    }

    #[test]
    fn test_low_usage_does_not_create_skill() {
        let mut ex = SkillExtractor::new();
        let intent = intention(Domain::Network, "rotate");
        ex.observe_actuation(&intent, &effect(true), 10);
        let skills = ex.evolve();
        let found = skills.iter().any(|s| s.action_pattern == "rotate");
        assert!(!found, "should not create skill with usage < 3");
    }

    #[test]
    fn test_skill_capacity_pruning() {
        let mut ex = SkillExtractor { skills: Vec::with_capacity(200), ..SkillExtractor::new() };
        for i in 0..150 {
            let intent = intention(Domain::Network, &format!("action_{}", i));
            for _ in 0..5 { ex.observe_actuation(&intent, &effect(true), 5); }
        }
        ex.evolve();
        assert!(ex.skills.len() <= SKILL_BANK_CAPACITY);
    }

    #[test]
    fn test_utility_score() {
        let mut ex = SkillExtractor::new();
        let intent = intention(Domain::Crawl, "search");
        for _ in 0..20 { ex.observe_actuation(&intent, &effect(true), 10); }
        ex.evolve();
        let skill = ex.skills.first().unwrap();
        assert!(skill.utility_score() > 0.0);
        assert!(skill.usage_count >= 20);
    }

    #[test]
    fn test_summary() {
        let mut ex = SkillExtractor::new();
        for d in &[Domain::Crawl, Domain::Network, Domain::Crypto] {
            let intent = intention(d.clone(), "test");
            for _ in 0..5 { ex.observe_actuation(&intent, &effect(true), 5); }
        }
        ex.evolve();
        let summary = ex.summary();
        assert_eq!(summary.len(), 3);
    }

    #[test]
    fn test_best_action() {
        let mut ex = SkillExtractor::new();
        for a in &["slow", "medium", "fast"] {
            let intent = intention(Domain::Crawl, a);
            for _ in 0..5 { ex.observe_actuation(&intent, &effect(a == &"fast"), 10); }
        }
        ex.evolve();
        let best = ex.best_action(&Domain::Crawl);
        assert!(best.is_some());
    }

    #[test]
    fn test_merge_updates_existing() {
        let mut ex = SkillExtractor::new();
        let intent = intention(Domain::Crawl, "explore");
        for _ in 0..5 { ex.observe_actuation(&intent, &effect(true), 100); }
        ex.evolve();
        let first_score = ex.skills.first().unwrap().success_rate;
        for _ in 0..5 { ex.observe_actuation(&intent, &effect(false), 200); }
        ex.evolve();
        let merged_score = ex.skills.first().unwrap().success_rate;
        assert!(merged_score < first_score);
    }
}
