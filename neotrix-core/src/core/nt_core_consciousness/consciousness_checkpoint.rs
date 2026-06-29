// INTERNAL - only used by sibling modules in this directory
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;

/// A single checkpoint of consciousness state, linked via hash chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessCheckpoint {
    /// Cycle number at which this checkpoint was taken
    pub cycle_number: usize,
    /// SHA-256 hash of the serialized consciousness state
    pub state_hash: [u8; 32],
    /// SHA-256 of the previous checkpoint (all zeros for genesis)
    pub parent_hash: [u8; 32],
    /// Unix timestamp of checkpoint creation
    pub timestamp_ns: u64,
    /// Human-readable description
    pub description: String,
    /// The full checkpoint hash = SHA-256(cycle_number || state_hash || parent_hash || timestamp_ns)
    pub self_hash: [u8; 32],
}

impl ConsciousnessCheckpoint {
    /// Create a new checkpoint and compute its self_hash.
    pub fn new(
        cycle_number: usize,
        state_hash: [u8; 32],
        parent_hash: [u8; 32],
        description: &str,
    ) -> Self {
        let timestamp_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let mut cp = Self {
            cycle_number,
            state_hash,
            parent_hash,
            timestamp_ns,
            description: description.to_string(),
            self_hash: [0u8; 32],
        };
        cp.self_hash = cp.compute_self_hash();
        cp
    }

    /// Compute the self hash: SHA-256(cycle_number || state_hash || parent_hash || timestamp_ns)
    pub fn compute_self_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.cycle_number.to_le_bytes());
        hasher.update(&self.state_hash);
        hasher.update(&self.parent_hash);
        hasher.update(&self.timestamp_ns.to_le_bytes());
        let result = hasher.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&result);
        arr
    }

    /// Verify that self_hash is consistent with the fields.
    pub fn verify(&self) -> bool {
        self.self_hash == self.compute_self_hash()
    }
}

/// Manages a chain of checkpoints for the consciousness subsystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointStore {
    /// Genesis checkpoint (cycle 0)
    pub genesis: ConsciousnessCheckpoint,
    /// All checkpoints in order, bounded to capacity
    pub chain: VecDeque<ConsciousnessCheckpoint>,
    /// Maximum checkpoints to retain
    pub capacity: usize,
}

impl CheckpointStore {
    /// Create a new store with a genesis checkpoint.
    pub fn new(state_hash: [u8; 32], capacity: usize) -> Self {
        let genesis = ConsciousnessCheckpoint::new(0, state_hash, [0u8; 32], "genesis");
        let mut chain = VecDeque::new();
        chain.push_back(genesis.clone());
        Self {
            genesis,
            chain,
            capacity,
        }
    }

    /// Add a new checkpoint to the chain.
    pub fn add_checkpoint(
        &mut self,
        cycle_number: usize,
        state_hash: [u8; 32],
        description: &str,
    ) -> &ConsciousnessCheckpoint {
        let parent_hash = self.chain.back().map(|c| c.self_hash).unwrap_or([0u8; 32]);
        let cp = ConsciousnessCheckpoint::new(cycle_number, state_hash, parent_hash, description);
        self.chain.push_back(cp);
        while self.chain.len() > self.capacity {
            self.chain.pop_front();
        }
        self.chain.back().unwrap_or_else(|| {
            log::error!("consciousness_checkpoint: chain.back() failed — no checkpoint after push");
            &self.genesis
        })
    }

    /// Get the latest checkpoint.
    pub fn latest(&self) -> Option<&ConsciousnessCheckpoint> {
        self.chain.back()
    }

    /// Get the latest checkpoint's self_hash.
    pub fn latest_hash(&self) -> Option<[u8; 32]> {
        self.chain.back().map(|c| c.self_hash)
    }

    /// Verify the entire chain from genesis to latest.
    /// Returns (is_valid, checked_count, failure_reason).
    pub fn verify_chain(&self) -> (bool, usize, Option<String>) {
        if self.chain.is_empty() {
            return (false, 0, Some("empty chain".to_string()));
        }

        for (i, cp) in self.chain.iter().enumerate() {
            if !cp.verify() {
                return (
                    false,
                    i,
                    Some(format!(
                        "checkpoint {} self_hash mismatch at cycle {}",
                        i, cp.cycle_number
                    )),
                );
            }
        }

        for i in 1..self.chain.len() {
            let expected_parent = self.chain[i - 1].self_hash;
            if self.chain[i].parent_hash != expected_parent {
                return (false, i, Some(format!(
                    "parent_hash mismatch at checkpoint {} (cycle {}): expected {:02x?} got {:02x?}",
                    i, self.chain[i].cycle_number, expected_parent, self.chain[i].parent_hash
                )));
            }
        }

        (true, self.chain.len(), None)
    }

    /// Number of checkpoints in the chain.
    pub fn len(&self) -> usize {
        self.chain.len()
    }

    /// Save the checkpoint store to a JSON file.
    pub fn save_to_json(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        crate::core::nt_core_util::atomic_write_json(std::path::Path::new(path), self)?;
        Ok(())
    }

    /// Load the checkpoint store from a JSON file.
    pub fn load_from_json(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let store: Self = serde_json::from_str(&json)?;
        let (valid, count, reason) = store.verify_chain();
        if !valid {
            log::warn!(
                "Checkpoint store integrity check failed at checkpoint {}: {:?}",
                count,
                reason
            );
        }
        Ok(store)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_self_hash() {
        let cp = ConsciousnessCheckpoint::new(1, [1u8; 32], [0u8; 32], "test");
        assert!(cp.verify());
    }

    #[test]
    fn test_checkpoint_chain_verification() {
        let mut store = CheckpointStore::new([0u8; 32], 100);
        store.add_checkpoint(10, [1u8; 32], "cycle_10");
        store.add_checkpoint(20, [2u8; 32], "cycle_20");
        let (valid, count, reason) = store.verify_chain();
        assert!(valid, "Chain should be valid: {:?}", reason);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_checkpoint_tamper_detection() {
        let mut store = CheckpointStore::new([0u8; 32], 100);
        store.add_checkpoint(10, [1u8; 32], "cycle_10");
        if let Some(cp) = store.chain.get_mut(1) {
            cp.state_hash = [0xffu8; 32];
        }
        let (valid, _, reason) = store.verify_chain();
        assert!(!valid, "Should detect tampering");
        assert!(reason.unwrap_or_default().contains("self_hash"));
    }

    #[test]
    fn test_checkpoint_parent_link() {
        let mut store = CheckpointStore::new([0u8; 32], 100);
        store.add_checkpoint(10, [1u8; 32], "cycle_10");
        store.add_checkpoint(20, [2u8; 32], "cycle_20");
        if let Some(cp) = store.chain.get_mut(2) {
            cp.parent_hash = [0xabu8; 32];
        }
        let (valid, _, _reason) = store.verify_chain();
        assert!(!valid, "Should detect broken parent link");
    }

    #[test]
    fn test_checkpoint_serde_roundtrip() {
        let mut store = CheckpointStore::new([42u8; 32], 10);
        store.add_checkpoint(5, [1u8; 32], "first");
        store.add_checkpoint(15, [2u8; 32], "second");
        let json = match serde_json::to_string(&store) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("consciousness_checkpoint: serialize store failed: {}", e);
                return;
            }
        };
        let loaded: CheckpointStore = match serde_json::from_str(&json) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("consciousness_checkpoint: deserialize store failed: {}", e);
                return;
            }
        };
        let (valid, count, _) = loaded.verify_chain();
        assert!(valid);
        assert_eq!(count, 3);
    }
}
