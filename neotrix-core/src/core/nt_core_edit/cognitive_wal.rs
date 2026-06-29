#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum WalEntryType {
    StateSnapshot,
    MutationProposal,
    MutationApproved,
    MutationApplied,
    MutationRolledBack,
    Checkpoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalEntry {
    pub id: u64,
    pub entry_type: WalEntryType,
    pub timestamp: u64,
    pub key: String,
    pub data: Vec<u8>,
    pub checksum: u64,
    pub prev_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveWal {
    pub entries: Vec<WalEntry>,
    pub next_id: u64,
    pub max_entries: usize,
    pub persist_path: Option<String>,
}

fn compute_checksum(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

impl CognitiveWal {
    pub fn new() -> Self {
        CognitiveWal {
            entries: Vec::new(),
            next_id: 1,
            max_entries: 10000,
            persist_path: None,
        }
    }

    pub fn append(&mut self, entry_type: WalEntryType, key: &str, data: Vec<u8>) -> u64 {
        let id = self.next_id;
        let checksum = compute_checksum(&data);
        let entry = WalEntry {
            id,
            entry_type,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            key: key.to_string(),
            data,
            checksum,
            prev_id: if id > 1 { id - 1 } else { 0 },
        };
        self.entries.push(entry);
        self.next_id += 1;
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
        id
    }

    pub fn read(&self, id: u64) -> Option<&WalEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    pub fn read_latest(&self, key: &str) -> Option<&WalEntry> {
        self.entries.iter().rev().find(|e| e.key == key)
    }

    pub fn replay(&self, since_id: u64) -> Vec<&WalEntry> {
        self.entries.iter().filter(|e| e.id >= since_id).collect()
    }

    pub fn truncate(&mut self, before_id: u64) {
        self.entries.retain(|e| e.id >= before_id);
    }

    pub fn integrity_check(&self) -> Vec<u64> {
        self.entries
            .iter()
            .filter(|e| compute_checksum(&e.data) != e.checksum)
            .map(|e| e.id)
            .collect()
    }

    pub fn count_by_type(&self, entry_type: WalEntryType) -> usize {
        self.entries
            .iter()
            .filter(|e| e.entry_type == entry_type)
            .count()
    }

    pub fn crash_recovery(&mut self) -> usize {
        let incomplete: Vec<u64> = self
            .entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::MutationProposal)
            .filter(|proposal| {
                !self.entries.iter().any(|e| {
                    e.key == proposal.key
                        && e.id > proposal.id
                        && (e.entry_type == WalEntryType::MutationApplied
                            || e.entry_type == WalEntryType::MutationRolledBack)
                })
            })
            .map(|e| e.id)
            .collect();

        let count = incomplete.len();
        self.entries.retain(|e| !incomplete.contains(&e.id));
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_and_read() {
        let mut wal = CognitiveWal::new();
        let id1 = wal.append(WalEntryType::StateSnapshot, "hcube", vec![0x01, 0x02]);
        let id2 = wal.append(WalEntryType::Checkpoint, "state", vec![0x03]);

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        let entry = wal.read(1).expect("entry 1 should exist");
        assert_eq!(entry.entry_type, WalEntryType::StateSnapshot);
        assert_eq!(entry.key, "hcube");
        assert_eq!(entry.data, vec![0x01, 0x02]);
        assert!(entry.checksum != 0);

        assert!(wal.read(999).is_none());
    }

    #[test]
    fn test_read_latest() {
        let mut wal = CognitiveWal::new();
        wal.append(WalEntryType::StateSnapshot, "vec", vec![1]);
        wal.append(WalEntryType::MutationApplied, "vec", vec![2]);
        wal.append(WalEntryType::MutationApplied, "vec", vec![3]);

        let latest = wal.read_latest("vec").expect("should find entry");
        assert_eq!(latest.data, vec![3]);
        assert_eq!(latest.id, 3);

        assert!(wal.read_latest("nonexistent").is_none());
    }

    #[test]
    fn test_replay() {
        let mut wal = CognitiveWal::new();
        wal.append(WalEntryType::Checkpoint, "ckpt", vec![0]);
        wal.append(WalEntryType::MutationApplied, "a", vec![1]);
        wal.append(WalEntryType::MutationApplied, "b", vec![2]);
        wal.append(WalEntryType::MutationApplied, "c", vec![3]);

        let replayed = wal.replay(3);
        assert_eq!(replayed.len(), 2);
        assert_eq!(replayed[0].key, "b");
        assert_eq!(replayed[1].key, "c");
    }

    #[test]
    fn test_truncate() {
        let mut wal = CognitiveWal::new();
        wal.append(WalEntryType::StateSnapshot, "s1", vec![1]);
        wal.append(WalEntryType::StateSnapshot, "s2", vec![2]);
        wal.append(WalEntryType::StateSnapshot, "s3", vec![3]);

        wal.truncate(2);
        assert_eq!(wal.entries.len(), 2);
        assert_eq!(wal.entries[0].id, 2);
        assert_eq!(wal.entries[1].id, 3);
    }

    #[test]
    fn test_integrity_check() {
        let mut wal = CognitiveWal::new();
        wal.append(WalEntryType::StateSnapshot, "good", vec![1, 2, 3]);
        let bad_id = wal.append(WalEntryType::MutationProposal, "bad", vec![4, 5, 6]);
        wal.append(WalEntryType::Checkpoint, "good2", vec![7]);

        // corrupt the data of the second entry
        wal.entries[1].data = vec![0xFF, 0xFF];

        let corrupted = wal.integrity_check();
        assert_eq!(corrupted, vec![bad_id]);
    }

    #[test]
    fn test_crash_recovery() {
        let mut wal = CognitiveWal::new();
        // complete transaction for key1
        wal.append(WalEntryType::MutationProposal, "key1", vec![1]);
        wal.append(WalEntryType::MutationApplied, "key1", vec![2]);
        // incomplete transaction for key2
        wal.append(WalEntryType::MutationProposal, "key2", vec![3]);
        // incomplete transaction for key3
        wal.append(WalEntryType::MutationProposal, "key3", vec![4]);

        let removed = wal.crash_recovery();
        assert_eq!(removed, 2);
        // entries 1 + 2 should remain (key1 proposal + key1 applied)
        assert_eq!(wal.entries.len(), 2);
    }

    #[test]
    fn test_count_by_type() {
        let mut wal = CognitiveWal::new();
        wal.append(WalEntryType::StateSnapshot, "s", vec![]);
        wal.append(WalEntryType::MutationProposal, "p1", vec![]);
        wal.append(WalEntryType::MutationProposal, "p2", vec![]);
        wal.append(WalEntryType::MutationApplied, "a", vec![]);

        assert_eq!(wal.count_by_type(WalEntryType::StateSnapshot), 1);
        assert_eq!(wal.count_by_type(WalEntryType::MutationProposal), 2);
        assert_eq!(wal.count_by_type(WalEntryType::MutationApplied), 1);
        assert_eq!(wal.count_by_type(WalEntryType::Checkpoint), 0);
    }

    #[test]
    fn test_large_sequence() {
        let mut wal = CognitiveWal::new();
        let n = 1000;
        for i in 0..n {
            wal.append(
                WalEntryType::MutationApplied,
                &format!("key{}", i),
                vec![i as u8],
            );
        }

        assert_eq!(wal.entries.len(), n);
        assert_eq!(wal.next_id, n as u64 + 1);

        // read the last entry
        let last = wal.read(n as u64).expect("last entry should exist");
        assert_eq!(last.key, format!("key{}", n - 1));

        // read_latest for a specific key
        let latest = wal.read_latest("key500").expect("should find key500");
        assert_eq!(latest.data, vec![244]);

        // replay from halfway
        let replayed = wal.replay(n as u64 / 2);
        assert_eq!(replayed.len(), n as usize / 2 + 1);
    }
}
