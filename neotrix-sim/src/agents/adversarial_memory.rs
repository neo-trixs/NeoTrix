use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A recorded threat event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatEvent {
    pub agent_id: String,
    pub threat_type: ThreatType,
    pub target_id: String,
    pub severity: f32,
    pub tick: u64,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ThreatType {
    Attack,
    Theft,
    Deception,
    Territory,
    Resource,
    Social,
}

/// An identified attack pattern from history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackPattern {
    pub agent_id: String,
    pub threat_type: ThreatType,
    pub frequency: u32,
    pub avg_severity: f32,
    pub last_seen: u64,
    pub preferred_targets: Vec<String>,
    pub time_pattern: TimePattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimePattern {
    Periodic { interval: u64 },
    Triggered { trigger: String },
    Random,
    Unknown,
}

/// A suggested defense strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenseStrategy {
    pub defense_type: DefenseType,
    pub priority: f32,
    pub description: String,
    pub estimated_effectiveness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DefenseType {
    Avoid,
    Fortify,
    Counter,
    Negotiate,
    Alert,
    Monitor,
}

/// Adversarial memory system: tracks threats, patterns, and defenses
pub struct AdversarialMemory {
    threats: Vec<ThreatEvent>,
    patterns: Vec<AttackPattern>,
    #[allow(dead_code)]
    defenses: Vec<DefenseStrategy>,
    threat_counts: HashMap<String, HashMap<ThreatType, u32>>,
    max_threats: usize,
}

impl AdversarialMemory {
    pub fn new(max_threats: usize) -> Self {
        Self {
            threats: Vec::new(),
            patterns: Vec::new(),
            defenses: Vec::new(),
            threat_counts: HashMap::new(),
            max_threats,
        }
    }

    /// Record a new threat event
    pub fn record_threat(&mut self, threat: ThreatEvent) {
        let agent = threat.agent_id.clone();
        let ttype = threat.threat_type.clone();
        *self
            .threat_counts
            .entry(agent)
            .or_default()
            .entry(ttype)
            .or_insert(0) += 1;

        self.threats.push(threat);
        self.prune();
        self.rebuild_patterns();
    }

    /// Predict the next likely attack from a given agent
    pub fn predict_attack(&self, agent_id: &str) -> Option<AttackPattern> {
        self.patterns
            .iter()
            .filter(|p| p.agent_id == agent_id)
            .max_by(|a, b| {
                a.frequency
                    .partial_cmp(&b.frequency)
                    .unwrap()
                    .then(a.avg_severity.partial_cmp(&b.avg_severity).unwrap())
            })
            .cloned()
    }

    /// Suggest a defense based on a threat
    pub fn suggest_defense(&self, threat: &ThreatEvent) -> DefenseStrategy {
        let agent_history = self.threat_counts.get(&threat.agent_id);
        let repeat_offender = agent_history
            .map(|h| h.values().sum::<u32>())
            .unwrap_or(0)
            > 3;

        match threat.threat_type {
            ThreatType::Attack => {
                let dt = if repeat_offender {
                    DefenseType::Counter
                } else {
                    DefenseType::Fortify
                };
                DefenseStrategy {
                    defense_type: dt,
                    priority: threat.severity,
                    description: if repeat_offender {
                        format!(
                            "Agent {} is a repeat attacker. Counter and maintain distance.",
                            threat.agent_id
                        )
                    } else {
                        "Fortify position and prepare to evade".to_string()
                    },
                    estimated_effectiveness: if repeat_offender { 0.7 } else { 0.5 },
                }
            }
            ThreatType::Theft => DefenseStrategy {
                defense_type: DefenseType::Monitor,
                priority: threat.severity * 0.8,
                description: "Monitor resource caches and secure valuable items".to_string(),
                estimated_effectiveness: 0.6,
            },
            ThreatType::Deception => DefenseStrategy {
                defense_type: DefenseType::Alert,
                priority: threat.severity * 0.9,
                description: "Cross-verify information from this agent".to_string(),
                estimated_effectiveness: 0.5,
            },
            ThreatType::Territory => DefenseStrategy {
                defense_type: DefenseType::Negotiate,
                priority: threat.severity * 0.6,
                description: "Negotiate territory boundaries to avoid conflict".to_string(),
                estimated_effectiveness: 0.4,
            },
            ThreatType::Resource => DefenseStrategy {
                defense_type: DefenseType::Avoid,
                priority: threat.severity * 0.7,
                description: "Avoid competing for the same resources".to_string(),
                estimated_effectiveness: 0.6,
            },
            ThreatType::Social => DefenseStrategy {
                defense_type: DefenseType::Alert,
                priority: threat.severity * 0.5,
                description: "Be cautious in social interactions with this agent".to_string(),
                estimated_effectiveness: 0.4,
            },
        }
    }

    /// Get all threats from a specific agent
    pub fn threats_from(&self, agent_id: &str) -> Vec<&ThreatEvent> {
        self.threats.iter().filter(|t| t.agent_id == agent_id).collect()
    }

    /// Get threat count by type for an agent
    pub fn threat_summary(&self, agent_id: &str) -> HashMap<ThreatType, u32> {
        self.threat_counts
            .get(agent_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all attack patterns
    pub fn patterns(&self) -> &[AttackPattern] {
        &self.patterns
    }

    pub fn threat_count(&self) -> usize {
        self.threats.len()
    }

    fn rebuild_patterns(&mut self) {
        self.patterns.clear();
        let mut grouped: HashMap<String, Vec<&ThreatEvent>> = HashMap::new();
        for threat in &self.threats {
            grouped
                .entry(threat.agent_id.clone())
                .or_default()
                .push(threat);
        }

        for (agent_id, agent_threats) in &grouped {
            let mut type_groups: HashMap<&ThreatType, Vec<&ThreatEvent>> = HashMap::new();
            for t in agent_threats {
                type_groups.entry(&t.threat_type).or_default().push(t);
            }

            for (ttype, threats) in &type_groups {
                let avg_severity =
                    threats.iter().map(|t| t.severity).sum::<f32>() / threats.len() as f32;
                let last_seen = threats.iter().map(|t| t.tick).max().unwrap_or(0);

                let mut target_counts: HashMap<String, u32> = HashMap::new();
                for t in threats {
                    *target_counts.entry(t.target_id.clone()).or_insert(0) += 1;
                }
                let mut preferred: Vec<(String, u32)> = target_counts.into_iter().collect();
                preferred.sort_by(|a, b| b.1.cmp(&a.1));
                let preferred_targets: Vec<String> =
                    preferred.into_iter().take(3).map(|(k, _v)| k).collect();

                let time_pattern = if threats.len() >= 3 {
                    let ticks: Vec<u64> = threats.iter().map(|t| t.tick).collect();
                    let mut intervals: Vec<u64> = ticks.windows(2).map(|w| w[1].saturating_sub(w[0])).collect();
                    intervals.sort();
                    let median = intervals[intervals.len() / 2];
                    if median > 0 && intervals.iter().all(|i| (*i as f64 - median as f64).abs() < median as f64 * 0.3) {
                        TimePattern::Periodic { interval: median }
                    } else {
                        TimePattern::Random
                    }
                } else {
                    TimePattern::Unknown
                };

                self.patterns.push(AttackPattern {
                    agent_id: agent_id.clone(),
                    threat_type: (*ttype).clone(),
                    frequency: threats.len() as u32,
                    avg_severity,
                    last_seen,
                    preferred_targets,
                    time_pattern,
                });
            }
        }
    }

    fn prune(&mut self) {
        if self.threats.len() > self.max_threats {
            self.threats.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap());
            self.threats.truncate(self.max_threats);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_threat(agent: &str, ttype: ThreatType, severity: f32, tick: u64) -> ThreatEvent {
        ThreatEvent {
            agent_id: agent.to_string(),
            threat_type: ttype,
            target_id: "target_0".to_string(),
            severity,
            tick,
            context: "test".to_string(),
        }
    }

    #[test]
    fn record_and_predict() {
        let mut mem = AdversarialMemory::new(100);
        for i in 0..5 {
            mem.record_threat(make_threat(
                "attacker_0",
                ThreatType::Attack,
                0.8,
                i * 10,
            ));
        }
        let pattern = mem.predict_attack("attacker_0").unwrap();
        assert_eq!(pattern.frequency, 5);
        assert!(pattern.avg_severity > 0.5);
    }

    #[test]
    fn suggest_defense_for_attack() {
        let mem = AdversarialMemory::new(100);
        let threat = make_threat("attacker_0", ThreatType::Attack, 0.9, 0);
        let defense = mem.suggest_defense(&threat);
        assert!(defense.priority > 0.5);
    }

    #[test]
    fn repeat_offender_gets_counter() {
        let mut mem = AdversarialMemory::new(100);
        for i in 0..5 {
            mem.record_threat(make_threat("bad_agent", ThreatType::Attack, 0.7, i));
        }
        let threat = make_threat("bad_agent", ThreatType::Attack, 0.8, 100);
        let defense = mem.suggest_defense(&threat);
        assert_eq!(defense.defense_type, DefenseType::Counter);
    }

    #[test]
    fn threat_summary() {
        let mut mem = AdversarialMemory::new(100);
        mem.record_threat(make_threat("a", ThreatType::Attack, 0.5, 0));
        mem.record_threat(make_threat("a", ThreatType::Theft, 0.3, 1));
        let summary = mem.threat_summary("a");
        assert_eq!(summary[&ThreatType::Attack], 1);
        assert_eq!(summary[&ThreatType::Theft], 1);
    }

    #[test]
    fn prune_limits_threats() {
        let mut mem = AdversarialMemory::new(5);
        for i in 0..10 {
            mem.record_threat(make_threat("a", ThreatType::Attack, i as f32 * 0.1, i));
        }
        assert!(mem.threat_count() <= 5);
    }
}
