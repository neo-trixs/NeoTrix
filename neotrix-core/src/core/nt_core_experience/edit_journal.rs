use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// A snapshot of a file's content before mutation
#[derive(Debug, Clone)]
pub struct FileSnapshot {
    pub path: String,
    pub content: String,
    pub timestamp: u64,
}

/// Record of a single mutation within a journal transaction
#[derive(Debug, Clone)]
pub struct EditRecord {
    pub file_path: String,
    pub original_source: String,
    pub mutated_source: String,
    pub success: bool,
    pub rolled_back: bool,
}

/// Status of a journal transaction
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Active,
    Committed,
    RolledBack,
}

/// P0.5: MOSS/agent-undo inspired EditJournal
/// Provides batch-level snapshot + rollback for SEAL self-modification
pub struct EditJournal {
    pub active_batch_id: Option<u64>,
    pub snapshots: HashMap<String, FileSnapshot>,
    pub records: Vec<EditRecord>,
    pub max_records: usize,
    pub status: TransactionStatus,
    pub total_committed: u64,
    pub total_rolled_back: u64,
    next_batch_id: u64,
}

impl EditJournal {
    pub fn new() -> Self {
        Self {
            active_batch_id: None,
            snapshots: HashMap::new(),
            records: Vec::with_capacity(50),
            max_records: 200,
            status: TransactionStatus::Committed,
            total_committed: 0,
            total_rolled_back: 0,
            next_batch_id: 1,
        }
    }

    /// Begin a new transaction batch. Takes snapshots of all listed files.
    pub fn begin_transaction(&mut self, file_paths: &[&str]) -> u64 {
        if self.status == TransactionStatus::Active {
            let bid = self.active_batch_id.unwrap_or(0);
            return bid;
        }
        let batch_id = self.next_batch_id;
        self.next_batch_id += 1;
        self.active_batch_id = Some(batch_id);
        self.status = TransactionStatus::Active;
        self.records.clear();

        for path in file_paths {
            let content = fs::read_to_string(path).unwrap_or_default();
            self.snapshots.insert(
                path.to_string(),
                FileSnapshot {
                    path: path.to_string(),
                    content,
                    timestamp: now_ms(),
                },
            );
        }
        batch_id
    }

    /// Snapshot a specific file (called before mutation)
    pub fn snapshot(&mut self, file_path: &str) {
        if self.snapshots.contains_key(file_path) {
            return;
        }
        let content = fs::read_to_string(file_path).unwrap_or_default();
        self.snapshots.insert(
            file_path.to_string(),
            FileSnapshot {
                path: file_path.to_string(),
                content,
                timestamp: now_ms(),
            },
        );
    }

    pub fn record_mutation(
        &mut self,
        file_path: &str,
        original: &str,
        mutated: &str,
        success: bool,
    ) {
        self.records.push(EditRecord {
            file_path: file_path.to_string(),
            original_source: original.to_string(),
            mutated_source: mutated.to_string(),
            success,
            rolled_back: false,
        });
        if self.records.len() > self.max_records {
            self.records.remove(0);
        }
    }

    pub fn commit(&mut self) -> usize {
        let count = self.records.len();
        self.total_committed += count as u64;
        self.status = TransactionStatus::Committed;
        self.active_batch_id = None;
        self.snapshots.clear();
        count
    }

    pub fn rollback(&mut self) -> usize {
        let mut count = 0usize;
        for record in self.records.iter_mut().rev() {
            if record.rolled_back || record.success {
                if let Err(e) = fs::write(&record.file_path, &record.original_source) {
                    eprintln!(
                        "edit_journal: rollback failed for {}: {}",
                        record.file_path, e
                    );
                } else {
                    record.rolled_back = true;
                    count += 1;
                }
            }
        }
        self.total_rolled_back += count as u64;
        self.status = TransactionStatus::RolledBack;
        self.active_batch_id = None;
        self.snapshots.clear();
        count
    }

    /// Safe rollback: only rollback if the file content matches the expected mutated source
    pub fn safe_rollback(&mut self) -> usize {
        let mut count = 0usize;
        for record in self.records.iter_mut().rev() {
            if record.rolled_back || !record.success {
                continue;
            }
            let current = fs::read_to_string(&record.file_path).unwrap_or_default();
            if current == record.mutated_source {
                if let Err(e) = fs::write(&record.file_path, &record.original_source) {
                    eprintln!(
                        "edit_journal: safe_rollback failed for {}: {}",
                        record.file_path, e
                    );
                } else {
                    record.rolled_back = true;
                    count += 1;
                }
            }
        }
        self.total_rolled_back += count as u64;
        self.status = TransactionStatus::RolledBack;
        self.active_batch_id = None;
        self.snapshots.clear();
        count
    }

    pub fn summary(&self) -> String {
        format!(
            "edit_journal: status={:?} records={} committed={} rolled_back={}",
            self.status,
            self.records.len(),
            self.total_committed,
            self.total_rolled_back,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn temp_file(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "{}", content).unwrap();
        f
    }

    #[test]
    fn test_begin_transaction() {
        let mut j = EditJournal::new();
        let bid = j.begin_transaction(&["src/main.rs", "src/lib.rs"]);
        assert!(bid >= 1);
        assert_eq!(j.status, TransactionStatus::Active);
        assert!(j.snapshots.contains_key("src/main.rs"));
    }

    #[test]
    fn test_record_and_commit() {
        let mut j = EditJournal::new();
        j.begin_transaction(&["nonexistent.rs"]);
        j.record_mutation("file.rs", "old", "new", true);
        assert_eq!(j.records.len(), 1);
        let c = j.commit();
        assert_eq!(c, 1);
        assert_eq!(j.status, TransactionStatus::Committed);
    }

    #[test]
    fn test_rollback_restores_file() {
        let mut f = temp_file("original content");
        let path = f.path().to_str().unwrap().to_string();
        let mut j = EditJournal::new();
        j.snapshot(&path);
        j.record_mutation(&path, "original content", "modified content", true);
        let c = j.rollback();
        assert!(c >= 1);
        let restored = fs::read_to_string(&path).unwrap();
        assert_eq!(restored, "original content");
    }

    #[test]
    fn test_safe_rollback_noop_on_diverged() {
        let mut f = temp_file("original");
        let path = f.path().to_str().unwrap().to_string();
        let mut j = EditJournal::new();
        j.snapshot(&path);
        j.record_mutation(&path, "original", "modified", true);
        // Someone else changed the file
        fs::write(&path, "someone else changed this").unwrap();
        let c = j.safe_rollback();
        assert_eq!(c, 0); // Should NOT rollback — mutated_source doesn't match current
        let current = fs::read_to_string(&path).unwrap();
        assert_eq!(current, "someone else changed this");
    }

    #[test]
    fn test_begin_noop_when_active() {
        let mut j = EditJournal::new();
        let bid1 = j.begin_transaction(&["a.rs"]);
        let bid2 = j.begin_transaction(&["b.rs"]);
        assert_eq!(bid1, bid2);
        assert!(j.snapshots.contains_key("a.rs"));
        assert!(!j.snapshots.contains_key("b.rs")); // second begin was noop
    }

    #[test]
    fn test_summary() {
        let mut j = EditJournal::new();
        j.begin_transaction(&["f.rs"]);
        j.record_mutation("f.rs", "a", "b", true);
        j.commit();
        let s = j.summary();
        assert!(s.contains("Committed"));
        assert!(s.contains("committed=1"));
    }

    #[test]
    fn test_max_records_prunes() {
        let mut j = EditJournal::new();
        j.max_records = 3;
        j.begin_transaction(&["x.rs"]);
        for i in 0..5 {
            j.record_mutation("x.rs", &format!("old{}", i), &format!("new{}", i), true);
        }
        assert_eq!(j.records.len(), 3);
        assert_eq!(j.records[0].file_path, "x.rs");
    }
}
