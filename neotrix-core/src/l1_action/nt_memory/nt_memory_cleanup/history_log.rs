//! History Log - 清理历史日志
//!
//! 记录清理操作历史
//! 域: NT-MEMORY (知识守护者)
//! 层: L1 Action

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupHistoryEntry {
    pub timestamp: String,
    pub operation: String,
    pub paths: Vec<String>,
    pub total_size_freed: u64,
    pub success: bool,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStats {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub total_size_freed: u64,
    pub total_duration_ms: u64,
    pub average_duration_ms: u64,
}

pub struct HistoryLog {
    entries: Vec<CleanupHistoryEntry>,
    log_path: PathBuf,
    max_entries: usize,
}

impl HistoryLog {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        let log_path = home.join(".config/neotrix/cleanup_history.json");
        let mut log = Self { entries: Vec::new(), log_path, max_entries: 1000 };
        let _ = log.load();
        log
    }

    pub(crate) fn _log_operation(&mut self, entry: CleanupHistoryEntry) {
        self.entries.push(entry);
        if self.entries.len() > self.max_entries { self.entries.remove(0); }
        let _ = self.save();
    }

    pub(crate) fn _get_recent(&self, count: usize) -> Vec<&CleanupHistoryEntry> {
        self.entries.iter().rev().take(count).collect()
    }

    pub fn get_stats(&self) -> HistoryStats {
        let total = self.entries.len();
        let successful = self.entries.iter().filter(|e| e.success).count();
        let total_freed = self.entries.iter().map(|e| e.total_size_freed).sum();
        let total_duration = self.entries.iter().map(|e| e.duration_ms).sum::<u64>();
        HistoryStats {
            total_operations: total,
            successful_operations: successful,
            failed_operations: total - successful,
            total_size_freed: total_freed,
            total_duration_ms: total_duration,
            average_duration_ms: if total > 0 { total_duration / total as u64 } else { 0 },
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.entries).map_err(|e| e.to_string())?;
        if let Some(parent) = self.log_path.parent() { std::fs::create_dir_all(parent).ok(); }
        std::fs::write(&self.log_path, json).map_err(|e| e.to_string())
    }

    pub fn load(&mut self) -> Result<(), String> {
        if self.log_path.exists() {
            let json = std::fs::read_to_string(&self.log_path).map_err(|e| e.to_string())?;
            self.entries = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn clear(&mut self) { self.entries.clear(); let _ = self.save(); }
}

impl Default for HistoryLog { fn default() -> Self { Self::new() } }
