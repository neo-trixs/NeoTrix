#![allow(dead_code)]
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};

/// A lightweight record of KPI values extracted from MetaCycleResult.
/// This is the persisted form — smaller than the full MetaCycleResult,
/// focused on metrics that are useful for historical trend analysis.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KpiRecord {
    pub cycle: u64,
    pub iteration: usize,
    pub meta_accuracy: f64,
    pub meta_accuracy_trend: f64,
    pub alert_count: usize,
    pub plan_count: usize,
    pub weakness_count: usize,
    pub compilation_ok: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Bounded ring buffer that stores the last N KPI records in memory
/// and can persist/load them to/from disk as JSON.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KpiRingBuffer {
    entries: VecDeque<KpiRecord>,
    max_entries: usize,
    #[serde(skip)]
    persist_path: PathBuf,
}

impl KpiRingBuffer {
    pub fn new(max_entries: usize, persist_path: PathBuf) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_entries.min(100)),
            max_entries,
            persist_path,
        }
    }

    /// Push a record into the ring buffer. If at capacity, evicts the oldest.
    pub fn push(&mut self, record: KpiRecord) {
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(record);
    }

    /// Read-only access to the full history (oldest first).
    pub fn history(&self) -> impl Iterator<Item = &KpiRecord> {
        self.entries.iter()
    }

    /// Return history as a Vec (oldest first).
    pub fn history_vec(&self) -> Vec<KpiRecord> {
        self.entries.iter().cloned().collect()
    }

    /// Return the last N records (most recent first).
    pub fn recent(&self, n: usize) -> Vec<&KpiRecord> {
        let n = n.min(self.entries.len());
        self.entries.iter().rev().take(n).collect()
    }

    /// Number of entries currently in the buffer.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Persist the ring buffer to disk as JSON (atomic write: tmp + rename).
    pub fn persist(&self) -> Result<(), String> {
        if let Some(parent) = self.persist_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("failed to create parent dir: {e}"))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("failed to serialize KPI buffer: {e}"))?;
        let tmp_path = self.persist_path.with_extension("tmp");
        fs::write(&tmp_path, &json).map_err(|e| format!("failed to write KPI tmp file: {e}"))?;
        fs::rename(&tmp_path, &self.persist_path)
            .map_err(|e| format!("failed to rename KPI tmp file: {e}"))?;
        Ok(())
    }

    /// Load a KpiRingBuffer from a JSON file on disk.
    /// Returns a default (empty) buffer if the file does not exist or is corrupt.
    pub fn load(path: &Path, max_entries: usize) -> Self {
        if !path.exists() {
            return Self::new(max_entries, path.to_path_buf());
        }
        match fs::read_to_string(path) {
            Ok(json) => match serde_json::from_str::<KpiRingBufferData>(&json) {
                Ok(data) => Self {
                    entries: data.entries.into(),
                    max_entries,
                    persist_path: path.to_path_buf(),
                },
                Err(e) => {
                    log::warn!("[kpi] failed to deserialize KPI buffer: {e} — starting fresh");
                    Self::new(max_entries, path.to_path_buf())
                }
            },
            Err(e) => {
                log::warn!("[kpi] failed to read KPI buffer file: {e} — starting fresh");
                Self::new(max_entries, path.to_path_buf())
            }
        }
    }

    /// Summary string for logging / debugging.
    pub fn summary(&self) -> String {
        if self.entries.is_empty() {
            return "kpi_buffer:empty".into();
        }
        let latest = self.entries.back().unwrap();
        let min_acc = self
            .entries
            .iter()
            .map(|e| e.meta_accuracy)
            .fold(f64::MAX, f64::min);
        let max_acc = self
            .entries
            .iter()
            .map(|e| e.meta_accuracy)
            .fold(f64::MIN, f64::max);
        format!(
            "kpi_buffer:entries={}_latest_acc={:.3}_acc_range=[{:.3},{:.3}]",
            self.entries.len(),
            latest.meta_accuracy,
            min_acc,
            max_acc
        )
    }
}

/// Helper struct for deserialization (persist_path is skipped, so we load separately).
#[derive(serde::Deserialize)]
struct KpiRingBufferData {
    entries: Vec<KpiRecord>,
    max_entries: usize,
}
