use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

/// A single entry in the SEAL governance hash chain.
///
/// Every SEAL self-modification produces one entry, forming
/// an append-only audit trail that can be verified from genesis.
#[derive(Debug, Clone)]
pub struct GovernanceEntry {
    pub phase: String,
    pub description: String,
    pub chain_hash: u64,
    pub prior_hash: u64,
    pub timestamp: u64,
}

/// SEALGovernance — append-only hash chain for SEAL self-modification audit.
///
/// Inspired by consciousness-kernel's GovernanceLedger + ReceiptChain.
/// Every SEAL mutation produces an entry that chains to the prior entry,
/// making the entire evolution history tamper-evident.
#[derive(Debug, Clone)]
pub struct SEALGovernance {
    chain: Vec<GovernanceEntry>,
    genesis_hash: u64,
}

impl SEALGovernance {
    pub fn new() -> Self {
        let genesis_hash = compute_hash("genesis:SEALGovernance:v1");
        Self {
            chain: Vec::new(),
            genesis_hash,
        }
    }

    pub fn record_entry(&mut self, phase: &str, description: &str) -> &GovernanceEntry {
        let prior_hash = self
            .chain
            .last()
            .map(|e| e.chain_hash)
            .unwrap_or(self.genesis_hash);
        let input = format!("{}|{}|{}", phase, description, prior_hash);
        let chain_hash = compute_hash(&input);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.chain.push(GovernanceEntry {
            phase: phase.to_string(),
            description: description.to_string(),
            chain_hash,
            prior_hash,
            timestamp,
        });
        self.chain.last().unwrap()
    }

    pub fn verify_chain(&self) -> bool {
        let mut prev = self.genesis_hash;
        for entry in &self.chain {
            if entry.prior_hash != prev {
                return false;
            }
            let input = format!("{}|{}|{}", entry.phase, entry.description, entry.prior_hash);
            let expected = compute_hash(&input);
            if entry.chain_hash != expected {
                return false;
            }
            prev = entry.chain_hash;
        }
        true
    }

    pub fn chain_len(&self) -> usize {
        self.chain.len()
    }

    pub fn last_entry(&self) -> Option<&GovernanceEntry> {
        self.chain.last()
    }

    pub fn all_entries(&self) -> &[GovernanceEntry] {
        &self.chain
    }

    pub fn genesis_hash(&self) -> u64 {
        self.genesis_hash
    }
}

fn compute_hash(input: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_chain_verifies() {
        let gov = SEALGovernance::new();
        assert!(gov.verify_chain());
        assert_eq!(gov.chain_len(), 0);
    }

    #[test]
    fn single_entry_verifies() {
        let mut gov = SEALGovernance::new();
        gov.record_entry("commit", "initial mutation");
        assert_eq!(gov.chain_len(), 1);
        assert!(gov.verify_chain());
    }

    #[test]
    fn multiple_entries_chain_correctly() {
        let mut gov = SEALGovernance::new();
        gov.record_entry("distill", "extracted heuristic");
        gov.record_entry("apply", "patched reasoning module");
        gov.record_entry("verify", "tests passed");
        gov.record_entry("commit", "finalized");
        assert_eq!(gov.chain_len(), 4);
        assert!(gov.verify_chain());
    }

    #[test]
    fn tampered_entry_detected() {
        let mut gov = SEALGovernance::new();
        gov.record_entry("distill", "first");
        gov.record_entry("apply", "second");
        gov.chain[0].description = "tampered".to_string();
        assert!(!gov.verify_chain());
    }

    #[test]
    fn genesis_hash_stable() {
        let gov1 = SEALGovernance::new();
        let gov2 = SEALGovernance::new();
        assert_eq!(gov1.genesis_hash(), gov2.genesis_hash());
    }
}
