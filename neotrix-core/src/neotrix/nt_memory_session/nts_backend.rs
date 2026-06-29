use std::path::Path;

use crate::core::nt_core_session::unified_record::UnifiedSessionRecord;
use nt_segstore::{Record as SegRecord, StorageConfig, StorageEngine, VsaTag, RT_SESSION};

/// NTSSEG-backed persistent store for `UnifiedSessionRecord`.
/// Uses the canonical append-only segment storage with RT_SESSION record type.
pub struct UnifiedSessionStore {
    engine: StorageEngine,
}

impl UnifiedSessionStore {
    /// Open or create a session store at the given directory path.
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let config = StorageConfig {
            data_dir: path.as_ref().to_path_buf(),
            ..Default::default()
        };
        let engine = StorageEngine::new(config)?;
        Ok(Self { engine })
    }

    /// Append a single unified record.
    pub fn append(&mut self, record: UnifiedSessionRecord) -> std::io::Result<()> {
        let bytes = serde_json::to_vec(&record)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        let seg = SegRecord::new(VsaTag::SelfMemory, RT_SESSION, &record.session_id, bytes);
        self.engine.put(seg)
    }

    /// Append many records in a batch (more efficient than repeated `append`).
    pub fn append_batch(&mut self, records: Vec<UnifiedSessionRecord>) -> std::io::Result<()> {
        for r in records {
            self.append(r)?;
        }
        Ok(())
    }

    /// Query records for a given session_id.
    /// Returns up to `limit` records matching the session_id, newest-first.
    pub fn query(&self, session_id: &str, limit: usize) -> Vec<UnifiedSessionRecord> {
        self.engine
            .find_by_type(RT_SESSION)
            .into_iter()
            .filter(|r| r.key == session_id && !r.tombstone)
            .filter_map(|r| serde_json::from_slice(&r.data).ok())
            .rev()
            .take(limit)
            .collect()
    }

    /// Query all records matching a record type filter.
    pub fn query_by_type(&self, rt: &str, limit: usize) -> Vec<UnifiedSessionRecord> {
        let rt_lower = rt.to_lowercase();
        self.engine
            .find_by_type(RT_SESSION)
            .into_iter()
            .filter(|r| !r.tombstone)
            .filter_map(|r| {
                let rec: UnifiedSessionRecord = serde_json::from_slice(&r.data).ok()?;
                if rec.record_type.as_str() == rt_lower {
                    Some(rec)
                } else {
                    None
                }
            })
            .rev()
            .take(limit)
            .collect()
    }

    /// List all distinct session IDs in the store.
    pub fn list_sessions(&self) -> Vec<String> {
        let mut seen = std::collections::BTreeSet::new();
        for r in self.engine.find_by_type(RT_SESSION) {
            if !r.tombstone {
                seen.insert(r.key.clone());
            }
        }
        seen.into_iter().collect()
    }

    /// Total record count across all sessions.
    pub fn record_count(&self) -> usize {
        self.engine.find_by_type(RT_SESSION).len()
    }

    /// Delegate to the underlying engine stats.
    pub fn stats(&self) -> nt_segstore::StoreStats {
        self.engine.stats()
    }
}
