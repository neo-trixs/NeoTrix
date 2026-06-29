use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
pub enum AuditEventType {
    SecurityThreat,
    EvolutionProposal,
    EvolutionApplied,
    EvolutionRejected,
    RiskAlert,
    DefenseTriggered,
    SystemChange,
    Anomaly,
    Info,
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub id: u64,
    pub timestamp: u64,
    pub event_type: AuditEventType,
    pub subsystem: String,
    pub description: String,
    pub severity: f64,
    pub details: String,
    pub hash_chain: String,
}

#[derive(Clone)]
pub struct AuditTrail {
    pub entries: VecDeque<AuditEntry>,
    pub counter: u64,
    pub max_entries: usize,
    pub last_hash: String,
}

impl AuditTrail {
    pub fn new(max_entries: usize) -> Self {
        AuditTrail {
            entries: VecDeque::with_capacity(max_entries.min(100)),
            counter: 0,
            max_entries,
            last_hash: "0000000000000000000000000000000000000000".to_string(),
        }
    }

    pub fn record(
        &mut self,
        event_type: AuditEventType,
        subsystem: &str,
        description: &str,
        severity: f64,
        details: &str,
    ) -> u64 {
        self.counter += 1;
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let entry_content = format!(
            "{}:{}:{}:{}:{}:{}",
            self.counter, ts, subsystem, description, severity, self.last_hash
        );
        let hash_chain = simple_hash(&entry_content);

        let entry = AuditEntry {
            id: self.counter,
            timestamp: ts,
            event_type,
            subsystem: subsystem.to_string(),
            description: description.to_string(),
            severity: severity.clamp(0.0, 1.0),
            details: details.to_string(),
            hash_chain: hash_chain.clone(),
        };

        self.last_hash = hash_chain;
        self.entries.push_back(entry.clone());

        if self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }

        self.counter
    }

    pub fn record_security_threat(
        &mut self,
        subsystem: &str,
        description: &str,
        severity: f64,
        details: &str,
    ) -> u64 {
        self.record(
            AuditEventType::SecurityThreat,
            subsystem,
            description,
            severity,
            details,
        )
    }

    pub fn record_evolution(&mut self, subsystem: &str, description: &str, applied: bool) -> u64 {
        let event_type = if applied {
            AuditEventType::EvolutionApplied
        } else {
            AuditEventType::EvolutionRejected
        };
        self.record(
            event_type,
            subsystem,
            description,
            if applied { 0.3 } else { 0.6 },
            "",
        )
    }

    pub fn recent_events(&self, n: usize) -> Vec<&AuditEntry> {
        self.entries.iter().rev().take(n).collect()
    }

    pub fn events_since(&self, since_id: u64) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.id > since_id).collect()
    }

    pub fn verify_chain(&self) -> bool {
        let mut prev_hash = "0000000000000000000000000000000000000000".to_string();
        for entry in &self.entries {
            let content = format!(
                "{}:{}:{}:{}:{}:{}",
                entry.id,
                entry.timestamp,
                entry.subsystem,
                entry.description,
                entry.severity,
                prev_hash
            );
            let expected = simple_hash(&content);
            if entry.hash_chain != expected {
                return false;
            }
            prev_hash = entry.hash_chain.clone();
        }
        true
    }

    pub fn security_threat_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.event_type == AuditEventType::SecurityThreat)
            .count()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.last_hash = "0000000000000000000000000000000000000000".to_string();
    }
}

fn simple_hash(input: &str) -> String {
    let mut hash: u64 = 5381;
    for b in input.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u64);
    }
    format!("{:020x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_integrity() {
        let mut trail = AuditTrail::new(100);
        trail.record(AuditEventType::Info, "test", "entry 1", 0.1, "");
        trail.record(AuditEventType::Info, "test", "entry 2", 0.2, "");
        assert!(trail.verify_chain());
    }

    #[test]
    fn test_tamper_detection() {
        let mut trail = AuditTrail::new(100);
        trail.record(AuditEventType::Info, "test", "entry 1", 0.1, "");
        trail.record(AuditEventType::Info, "test", "entry 2", 0.2, "");
        if let Some(mut entry) = trail.entries.back_mut() {
            entry.severity = 0.9;
        }
        assert!(!trail.verify_chain());
    }
}
