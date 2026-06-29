// INTERNAL - only used by sibling modules in this directory
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveSequence {
    pub drive: String,
    pub context_bits: u64,
    pub reward: f64,
    pub cycle: u64,
    pub ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeaArchive {
    entries: Vec<DriveSequence>,
    max_entries: usize,
    records_since_save: usize,
}

impl GeaArchive {
    pub fn new() -> Self {
        Self {
            entries: Vec::with_capacity(100),
            max_entries: 100,
            records_since_save: 0,
        }
    }

    pub fn with_max(max: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max.max(1)),
            max_entries: max.max(1),
            records_since_save: 0,
        }
    }

    pub fn record_success(&mut self, drive: &str, context: u64, reward: f64, cycle: u64) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(DriveSequence {
            drive: drive.to_string(),
            context_bits: context,
            reward,
            cycle,
            ttl: 100,
        });
        self.records_since_save += 1;
    }

    pub fn bias_for(&self, drive: &str, context: u64) -> f64 {
        if self.entries.is_empty() {
            return 0.0;
        }
        let total = self.entries.len();
        let matching = self
            .entries
            .iter()
            .filter(|e| e.drive == drive && e.context_bits == context)
            .count();
        if matching == 0 {
            return 0.0;
        }
        let ratio = matching as f64 / total as f64;
        (ratio * 0.5).min(0.5)
    }

    pub fn prune(&mut self, cycle: u64) {
        self.entries.retain(|e| cycle < e.cycle + e.ttl);
    }

    pub fn merge(&mut self, other: &GeaArchive) {
        for entry in &other.entries {
            let exists = self
                .entries
                .iter()
                .any(|e| e.drive == entry.drive && e.context_bits == entry.context_bits);
            if !exists {
                if self.entries.len() >= self.max_entries {
                    self.entries.remove(0);
                }
                self.entries.push(entry.clone());
            }
        }
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        let p = std::path::Path::new(path);
        let tmp = p.with_extension("tmp");
        std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, p).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load(path: &str) -> Result<Self, String> {
        let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }

    pub fn needs_autosave(&self) -> bool {
        self.records_since_save >= 10
    }

    pub fn reset_autosave_counter(&mut self) {
        self.records_since_save = 0;
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

impl Default for GeaArchive {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bias_basic() {
        let mut archive = GeaArchive::with_max(100);
        archive.record_success("explore", 42, 0.9, 1);
        let bias = archive.bias_for("explore", 42);
        assert!(bias > 0.0);
        assert!(bias <= 0.5);

        let bias_mismatch = archive.bias_for("explore", 99);
        assert_eq!(bias_mismatch, 0.0);
    }

    #[test]
    fn test_prune() {
        let mut archive = GeaArchive::with_max(100);
        archive.record_success("explore", 42, 0.9, 1);
        archive.prune(200);
        let bias_pruned = archive.bias_for("explore", 42);
        assert_eq!(bias_pruned, 0.0);
    }

    #[test]
    fn test_ring_buffer_eviction() {
        let mut small = GeaArchive::with_max(3);
        small.record_success("explore", 1, 0.9, 1);
        small.record_success("exploit", 2, 0.8, 2);
        small.record_success("innovate", 3, 0.7, 3);
        small.record_success("prune", 4, 0.9, 4);
        assert_eq!(small.bias_for("explore", 1), 0.0);
        assert!(small.bias_for("prune", 4) > 0.0);
    }

    #[test]
    fn test_save_load_roundtrip() {
        let mut archive = GeaArchive::with_max(50);
        archive.record_success("explore", 10, 0.8, 1);
        archive.record_success("exploit", 20, 0.9, 2);
        archive.record_success("innovate", 30, 0.7, 3);

        let path = "/tmp/neotrix_test_gea_roundtrip.json";
        archive.save(path).unwrap();

        let loaded = GeaArchive::load(path).unwrap();
        assert_eq!(loaded.entry_count(), 3);
        assert!((loaded.bias_for("explore", 10) - archive.bias_for("explore", 10)).abs() < 1e-9);
        assert!((loaded.bias_for("exploit", 20) - archive.bias_for("exploit", 20)).abs() < 1e-9);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_merge_deduplicates() {
        let mut a = GeaArchive::with_max(100);
        a.record_success("explore", 1, 0.8, 1);
        a.record_success("exploit", 2, 0.9, 2);

        let mut b = GeaArchive::with_max(100);
        b.record_success("exploit", 2, 0.9, 2);
        b.record_success("innovate", 3, 0.7, 3);

        a.merge(&b);
        assert_eq!(a.entry_count(), 3);
        assert!((a.bias_for("exploit", 2) - 0.5 * 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_merge_empty_self() {
        let mut a = GeaArchive::with_max(10);
        let b = GeaArchive::with_max(10);
        a.merge(&b);
        assert_eq!(a.entry_count(), 0);
    }

    #[test]
    fn test_autosave_counter() {
        let mut archive = GeaArchive::with_max(100);
        assert!(!archive.needs_autosave());
        for i in 0..10 {
            archive.record_success("explore", i, 0.8, i as u64);
        }
        assert!(archive.needs_autosave());
        archive.reset_autosave_counter();
        assert!(!archive.needs_autosave());
    }

    #[test]
    fn test_default_and_new() {
        let a = GeaArchive::new();
        let b = GeaArchive::default();
        assert_eq!(a.entry_count(), b.entry_count());
        assert_eq!(a.entry_count(), 0);
    }
}
