//! Write-Ahead Log for crash-safe multi-file persistence.
//!
//! Ensures atomic multi-file writes: log first, apply second, commit third.
//! On recovery, uncommitted entries are replayed; partially-applied ones are rolled forward.

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

/// A single WAL entry — one logical mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalEntry {
    /// Monotonically increasing sequence number
    pub seq: u64,
    /// Unix timestamp of when this entry was written
    pub timestamp: i64,
    /// Component identifier (e.g., "brain", "bank", "goals", "world_model")
    pub component: String,
    /// Operation type
    pub operation: String,
    /// Path of the persisted file (relative to store root)
    pub file_path: String,
    /// SHA-256 hash of the data (for integrity check)
    pub data_hash: String,
}

/// Current state of the WAL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalState {
    /// Format version
    pub format: String,
    /// Last committed sequence number
    pub last_committed_seq: u64,
    /// Pending (uncommitted) entries
    pub pending: Vec<WalEntry>,
    /// Component to last seq mapping for fast lookup
    pub component_seqs: HashMap<String, u64>,
}

impl Default for WalState {
    fn default() -> Self {
        Self::new()
    }
}

impl WalState {
    pub fn new() -> Self {
        Self {
            format: "neotrix-wal-v1".to_string(),
            last_committed_seq: 0,
            pending: Vec::new(),
            component_seqs: HashMap::new(),
        }
    }
}

/// WAL manager — thread-safe via internal Mutex.
pub struct WriteAheadLog {
    path: PathBuf,
    state: Mutex<WalState>,
    next_seq: Mutex<u64>,
}

impl WriteAheadLog {
    /// Open or create a WAL at the given path.
    pub fn open(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let state = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str::<WalState>(&s).ok())
                .unwrap_or_else(WalState::new)
        } else {
            WalState::new()
        };
        let next_seq = state.last_committed_seq + 1;
        Self {
            path,
            state: Mutex::new(state),
            next_seq: Mutex::new(next_seq),
        }
    }

    /// Log a mutation (write-ahead) and persist the WAL state.
    pub fn log(&self, component: &str, operation: &str, file_path: &str, data: &[u8]) -> Result<u64, String> {
        let seq = {
            let mut seq = self.next_seq.lock().map_err(|e| e.to_string())?;
            *seq += 1;
            *seq
        };
        let data_hash = hex::encode(Sha256::digest(data));
        let entry = WalEntry {
            seq,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            component: component.to_string(),
            operation: operation.to_string(),
            file_path: file_path.to_string(),
            data_hash,
        };

        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        state.pending.push(entry);
        state.component_seqs.insert(component.to_string(), seq);
        self.flush_state(&state)?;
        Ok(seq)
    }

    /// Commit all pending entries up to the given sequence number.
    pub fn commit(&self, seq: u64) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        state.pending.retain(|e| e.seq > seq);
        if seq > state.last_committed_seq {
            state.last_committed_seq = seq;
        }
        self.flush_state(&state)
    }

    /// Get all pending (uncommitted) entries.
    pub fn pending_entries(&self) -> Result<Vec<WalEntry>, String> {
        let state = self.state.lock().map_err(|e| e.to_string())?;
        Ok(state.pending.clone())
    }

    /// Get the last committed sequence number.
    pub fn last_committed(&self) -> Result<u64, String> {
        let state = self.state.lock().map_err(|e| e.to_string())?;
        Ok(state.last_committed_seq)
    }

    /// Replay all pending entries (for crash recovery).
    pub fn recover(&self) -> Result<Vec<WalEntry>, String> {
        self.pending_entries()
    }

    /// Flush WAL state to disk atomically.
    fn flush_state(&self, state: &WalState) -> Result<(), String> {
        let json = serde_json::to_string_pretty(state)
            .map_err(|e| format!("serialize wal: {}", e))?;
        crate::core::fs_util::atomic_write(&self.path, json.as_bytes())
    }

    /// Rotate (truncate) the WAL after all entries are committed.
    pub fn rotate(&self) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        state.pending.clear();
        self.flush_state(&state)
    }
}

impl Drop for WriteAheadLog {
    fn drop(&mut self) {
        if let Ok(state) = self.state.lock() {
            let _ = self.flush_state(&state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_wal_path() -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        PathBuf::from(std::env::temp_dir()).join(format!("test_wal_{}.json", n))
    }

    #[test]
    fn test_wal_multiple_components() {
        let path = test_wal_path();
        let _ = std::fs::remove_file(&path);

        let wal = WriteAheadLog::open(&path);
        let s1 = wal.log("comp_a", "write", "a.json", b"aaa").expect("log should succeed");
        let s2 = wal.log("comp_b", "write", "b.json", b"bbb").expect("log should succeed");
        let s3 = wal.log("comp_a", "update", "a.json", b"aaa_v2").expect("log should succeed");
        assert!(s1 < s2 && s2 < s3, "seq should be monotonic");

        wal.commit(s2).expect("commit should succeed");

        let pending = wal.pending_entries().expect("pending entries should succeed");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].component, "comp_a");

        wal.commit(s3).expect("commit should succeed");
        assert_eq!(wal.pending_entries().expect("pending entries should succeed").len(), 0);

        let _ = std::fs::remove_file(&path);
    }
}
