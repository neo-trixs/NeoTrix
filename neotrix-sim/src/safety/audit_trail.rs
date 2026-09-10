// AuditTrail — logs all self-modifications, tracks capability changes, enables post-hoc analysis
// Maps to NT-SHIELD audit dimensions D1-D50

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Types of events that get audited
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    /// Evolution cycle completed
    EvolutionCycle {
        generation: u64,
        population: usize,
        eliminated: usize,
        offspring: usize,
    },
    /// Mutation applied to a genome
    MutationApplied {
        agent_id: String,
        trait_index: usize,
        old_value: f32,
        new_value: f32,
    },
    /// Capability regression detected
    CapabilityRegression {
        agent_id: String,
        severity: String,
        details: String,
    },
    /// Safety violation detected
    SafetyViolation {
        agent_id: String,
        violation_type: String,
        severity: f32,
    },
    /// Evolution constraint enforced
    ConstraintEnforced {
        constraint_type: String,
        agent_id: String,
        details: String,
    },
    /// Agent death
    AgentDeath {
        agent_id: String,
        cause: String,
        tick: u64,
    },
    /// Agent birth (evolution spawn)
    AgentBirth {
        agent_id: String,
        parent_id: Option<String>,
        tick: u64,
    },
    /// Personality drift event
    PersonalityDrift {
        agent_id: String,
        drift_magnitude: f32,
        old_dominant: String,
        new_dominant: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: u64,
    pub tick: u64,
    pub event: AuditEventType,
    pub timestamp_hint: String,  // human-readable
}

/// Audit trail for self-evolution safety analysis
pub struct AuditTrail {
    entries: Vec<AuditEntry>,
    next_id: u64,
    max_entries: usize,
    /// Index: agent_id → entry indices
    agent_index: HashMap<String, Vec<usize>>,
    /// Index: event type → entry indices
    type_index: HashMap<String, Vec<usize>>,
}

impl AuditTrail {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            next_id: 0,
            max_entries,
            agent_index: HashMap::new(),
            type_index: HashMap::new(),
        }
    }

    /// Record an audit event
    pub fn record(&mut self, tick: u64, event: AuditEventType) {
        let entry_id = self.next_id;
        self.next_id += 1;

        let agent_id = Self::extract_agent_id(&event);
        let type_key = Self::event_type_key(&event);

        let entry = AuditEntry {
            id: entry_id,
            tick,
            event,
            timestamp_hint: format!("tick_{}", tick),
        };

        // Index by agent
        if let Some(ref aid) = agent_id {
            self.agent_index
                .entry(aid.clone())
                .or_default()
                .push(self.entries.len());
        }

        // Index by type
        self.type_index
            .entry(type_key)
            .or_default()
            .push(self.entries.len());

        self.entries.push(entry);

        // Evict oldest if over capacity
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
            // Rebuild indices (simple but correct)
            self.rebuild_indices();
        }
    }

    /// Query audit entries for a specific agent
    pub fn for_agent(&self, agent_id: &str) -> Vec<&AuditEntry> {
        self.agent_index
            .get(agent_id)
            .map(|indices| indices.iter().map(|&i| &self.entries[i]).collect())
            .unwrap_or_default()
    }

    /// Query audit entries by event type
    pub fn for_event_type(&self, type_key: &str) -> Vec<&AuditEntry> {
        self.type_index
            .get(type_key)
            .map(|indices| indices.iter().map(|&i| &self.entries[i]).collect())
            .unwrap_or_default()
    }

    /// Query entries within a tick range
    pub fn in_tick_range(&self, start: u64, end: u64) -> Vec<&AuditEntry> {
        self.entries.iter()
            .filter(|e| e.tick >= start && e.tick <= end)
            .collect()
    }

    /// Get total number of audit entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if audit trail is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get summary statistics
    pub fn summary(&self) -> AuditSummary {
        let mut type_counts = HashMap::new();
        for entry in &self.entries {
            let key = Self::event_type_key(&entry.event);
            *type_counts.entry(key).or_insert(0) += 1;
        }

        let agent_counts = self.agent_index.iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect();

        AuditSummary {
            total_entries: self.entries.len(),
            type_counts,
            agent_activity: agent_counts,
            tick_range: {
                let ticks: Vec<u64> = self.entries.iter().map(|e| e.tick).collect();
                if ticks.is_empty() {
                    (0, 0)
                } else {
                    let min = *ticks.iter().min().unwrap();
                    let max = *ticks.iter().max().unwrap();
                    (min, max)
                }
            },
        }
    }

    fn extract_agent_id(event: &AuditEventType) -> Option<String> {
        match event {
            AuditEventType::MutationApplied { agent_id, .. } => Some(agent_id.clone()),
            AuditEventType::CapabilityRegression { agent_id, .. } => Some(agent_id.clone()),
            AuditEventType::SafetyViolation { agent_id, .. } => Some(agent_id.clone()),
            AuditEventType::ConstraintEnforced { agent_id, .. } => Some(agent_id.clone()),
            AuditEventType::AgentDeath { agent_id, .. } => Some(agent_id.clone()),
            AuditEventType::AgentBirth { agent_id, .. } => Some(agent_id.clone()),
            AuditEventType::PersonalityDrift { agent_id, .. } => Some(agent_id.clone()),
            AuditEventType::EvolutionCycle { .. } => None,
        }
    }

    fn event_type_key(event: &AuditEventType) -> String {
        match event {
            AuditEventType::EvolutionCycle { .. } => "evolution_cycle".to_string(),
            AuditEventType::MutationApplied { .. } => "mutation_applied".to_string(),
            AuditEventType::CapabilityRegression { .. } => "capability_regression".to_string(),
            AuditEventType::SafetyViolation { .. } => "safety_violation".to_string(),
            AuditEventType::ConstraintEnforced { .. } => "constraint_enforced".to_string(),
            AuditEventType::AgentDeath { .. } => "agent_death".to_string(),
            AuditEventType::AgentBirth { .. } => "agent_birth".to_string(),
            AuditEventType::PersonalityDrift { .. } => "personality_drift".to_string(),
        }
    }

    fn rebuild_indices(&mut self) {
        self.agent_index.clear();
        self.type_index.clear();
        for (i, entry) in self.entries.iter().enumerate() {
            if let Some(ref aid) = Self::extract_agent_id(&entry.event) {
                self.agent_index.entry(aid.clone()).or_default().push(i);
            }
            let key = Self::event_type_key(&entry.event);
            self.type_index.entry(key).or_default().push(i);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    pub total_entries: usize,
    pub type_counts: HashMap<String, usize>,
    pub agent_activity: HashMap<String, usize>,
    pub tick_range: (u64, u64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_and_retrieve() {
        let mut trail = AuditTrail::new(1000);
        trail.record(0, AuditEventType::AgentBirth {
            agent_id: "agent_0".to_string(),
            parent_id: None,
            tick: 0,
        });
        trail.record(1, AuditEventType::MutationApplied {
            agent_id: "agent_0".to_string(),
            trait_index: 0,
            old_value: 0.5,
            new_value: 0.6,
        });

        let agent_entries = trail.for_agent("agent_0");
        assert_eq!(agent_entries.len(), 2);
    }

    #[test]
    fn query_by_type() {
        let mut trail = AuditTrail::new(1000);
        trail.record(0, AuditEventType::EvolutionCycle {
            generation: 1, population: 10, eliminated: 2, offspring: 2,
        });
        trail.record(1, AuditEventType::AgentBirth {
            agent_id: "agent_0".to_string(), parent_id: None, tick: 1,
        });

        let cycles = trail.for_event_type("evolution_cycle");
        assert_eq!(cycles.len(), 1);
    }

    #[test]
    fn tick_range_query() {
        let mut trail = AuditTrail::new(1000);
        for i in 0..10 {
            trail.record(i, AuditEventType::EvolutionCycle {
                generation: i, population: 10, eliminated: 0, offspring: 0,
            });
        }
        let in_range = trail.in_tick_range(3, 7);
        assert_eq!(in_range.len(), 5);
    }

    #[test]
    fn eviction_works() {
        let mut trail = AuditTrail::new(5);
        for i in 0..10 {
            trail.record(i, AuditEventType::EvolutionCycle {
                generation: i, population: 10, eliminated: 0, offspring: 0,
            });
        }
        assert_eq!(trail.len(), 5);
    }

    #[test]
    fn summary_statistics() {
        let mut trail = AuditTrail::new(1000);
        trail.record(0, AuditEventType::AgentBirth {
            agent_id: "a".to_string(), parent_id: None, tick: 0,
        });
        trail.record(1, AuditEventType::AgentBirth {
            agent_id: "b".to_string(), parent_id: None, tick: 1,
        });
        trail.record(2, AuditEventType::MutationApplied {
            agent_id: "a".to_string(), trait_index: 0, old_value: 0.0, new_value: 0.1,
        });

        let summary = trail.summary();
        assert_eq!(summary.total_entries, 3);
        assert_eq!(summary.type_counts.get("agent_birth"), Some(&2));
    }
}
