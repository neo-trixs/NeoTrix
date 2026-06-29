use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalEntry {
    pub seq: u64,
    pub timestamp: i64,
    pub component: String,
    pub operation: String,
    pub file_path: String,
    pub data_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalState {
    pub format: String,
    pub last_committed_seq: u64,
    pub pending: Vec<WalEntry>,
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

pub struct WriteAheadLog {
    path: PathBuf,
    state: Mutex<WalState>,
    next_seq: Mutex<u64>,
}

impl WriteAheadLog {
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

    pub fn log(
        &self,
        component: &str,
        operation: &str,
        file_path: &str,
        data: &[u8],
    ) -> Result<u64, String> {
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

    pub fn commit(&self, seq: u64) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        state.pending.retain(|e| e.seq > seq);
        if seq > state.last_committed_seq {
            state.last_committed_seq = seq;
        }
        self.flush_state(&state)
    }

    pub fn pending_entries(&self) -> Result<Vec<WalEntry>, String> {
        let state = self.state.lock().map_err(|e| e.to_string())?;
        Ok(state.pending.clone())
    }

    pub fn last_committed(&self) -> Result<u64, String> {
        let state = self.state.lock().map_err(|e| e.to_string())?;
        Ok(state.last_committed_seq)
    }

    pub fn recover(&self) -> Result<Vec<WalEntry>, String> {
        self.pending_entries()
    }

    fn flush_state(&self, state: &WalState) -> Result<(), String> {
        let json =
            serde_json::to_string_pretty(state).map_err(|e| format!("serialize wal: {}", e))?;
        crate::core::nt_core_util::atomic_write_bytes(&self.path, json.as_bytes())
            .map_err(|e| format!("atomic write: {}", e))
    }

    pub fn rotate(&self) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        state.pending.clear();
        self.flush_state(&state)
    }
}

impl Drop for WriteAheadLog {
    fn drop(&mut self) {
        if let Ok(state) = self.state.lock() {
            if let Err(e) = self.flush_state(&state) {
                log::warn!("[wal] flush_state in Drop failed: {e}");
            }
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
        let s1 = wal
            .log("comp_a", "write", "a.json", b"aaa")
            .expect("log should succeed");
        let s2 = wal
            .log("comp_b", "write", "b.json", b"bbb")
            .expect("log should succeed");
        let s3 = wal
            .log("comp_a", "update", "a.json", b"aaa_v2")
            .expect("log should succeed");
        assert!(s1 < s2 && s2 < s3, "seq should be monotonic");

        wal.commit(s2).expect("commit should succeed");

        let pending = wal
            .pending_entries()
            .expect("pending entries should succeed");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].component, "comp_a");

        wal.commit(s3).expect("commit should succeed");
        assert_eq!(
            wal.pending_entries()
                .expect("pending entries should succeed")
                .len(),
            0
        );

        let _ = std::fs::remove_file(&path);
    }
}
