use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// A content fingerprint for stage inputs.
/// Uses a simple hash of the stage's key inputs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Fingerprint(u64);

impl Fingerprint {
    pub fn from_text(text: &str) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut hasher);
        Fingerprint(hasher.finish())
    }

    pub fn from_values(values: &[&str]) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for v in values {
            v.hash(&mut hasher);
        }
        Fingerprint(hasher.finish())
    }
}

/// Tracks the last-seen fingerprint for each stage and determines which to skip.
pub struct FingerprintReconciler {
    last_fingerprints: HashMap<String, Fingerprint>,
    skip_count: HashMap<String, u64>,
    total_bytes_saved: u64,
}

impl FingerprintReconciler {
    pub fn new() -> Self {
        Self {
            last_fingerprints: HashMap::new(),
            skip_count: HashMap::new(),
            total_bytes_saved: 0,
        }
    }

    /// Check if a stage should be skipped based on fingerprint.
    /// Returns true if the stage should be executed (fingerprint changed or new).
    /// Returns false if the stage should be skipped (fingerprint unchanged).
    pub fn should_execute(&mut self, stage_name: &str, inputs: &[&str]) -> bool {
        let fp = Fingerprint::from_values(inputs);
        if let Some(last) = self.last_fingerprints.get(stage_name) {
            if *last == fp {
                *self.skip_count.entry(stage_name.to_string()).or_insert(0) += 1;
                return false;
            }
        }
        self.last_fingerprints.insert(stage_name.to_string(), fp);
        true
    }

    /// Record estimated bytes saved for a skip
    pub fn record_skip(&mut self, stage_name: &str, estimated_bytes: u64) {
        let _ = stage_name;
        self.total_bytes_saved += estimated_bytes;
    }

    /// Stats
    pub fn skip_count(&self, stage_name: &str) -> u64 {
        self.skip_count.get(stage_name).copied().unwrap_or(0)
    }

    pub fn total_bytes_saved(&self) -> u64 {
        self.total_bytes_saved
    }

    pub fn total_skips(&self) -> u64 {
        self.skip_count.values().sum()
    }

    /// Reset tracking for a specific stage
    pub fn invalidate(&mut self, stage_name: &str) {
        self.last_fingerprints.remove(stage_name);
    }

    /// Reset all tracking
    pub fn reset_all(&mut self) {
        self.last_fingerprints.clear();
        self.skip_count.clear();
        self.total_bytes_saved = 0;
    }
}

impl Default for FingerprintReconciler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_call_always_executes() {
        let mut reconciler = FingerprintReconciler::new();
        assert!(reconciler.should_execute("stage_a", &["input1"]));
    }

    #[test]
    fn test_same_fingerprint_returns_false() {
        let mut reconciler = FingerprintReconciler::new();
        assert!(reconciler.should_execute("stage_a", &["hello", "world"]));
        assert!(!reconciler.should_execute("stage_a", &["hello", "world"]));
    }

    #[test]
    fn test_different_fingerprint_returns_true() {
        let mut reconciler = FingerprintReconciler::new();
        assert!(reconciler.should_execute("stage_a", &["hello"]));
        assert!(reconciler.should_execute("stage_a", &["world"]));
    }

    #[test]
    fn test_multiple_stages_independent() {
        let mut reconciler = FingerprintReconciler::new();
        assert!(reconciler.should_execute("stage_a", &["x"]));
        assert!(reconciler.should_execute("stage_b", &["y"]));
        assert!(!reconciler.should_execute("stage_a", &["x"]));
        assert!(!reconciler.should_execute("stage_b", &["y"]));
    }

    #[test]
    fn test_record_skip_accumulates_bytes() {
        let mut reconciler = FingerprintReconciler::new();
        assert_eq!(reconciler.total_bytes_saved(), 0);
        reconciler.record_skip("stage_a", 1024);
        assert_eq!(reconciler.total_bytes_saved(), 1024);
        reconciler.record_skip("stage_b", 2048);
        assert_eq!(reconciler.total_bytes_saved(), 3072);
    }

    #[test]
    fn test_skip_count_tracks_per_stage() {
        let mut reconciler = FingerprintReconciler::new();
        assert_eq!(reconciler.skip_count("stage_a"), 0);
        assert!(reconciler.should_execute("stage_a", &["x"]));
        assert!(!reconciler.should_execute("stage_a", &["x"]));
        assert!(!reconciler.should_execute("stage_a", &["x"]));
        assert_eq!(reconciler.skip_count("stage_a"), 2);
    }

    #[test]
    fn test_total_skips() {
        let mut reconciler = FingerprintReconciler::new();
        assert_eq!(reconciler.total_skips(), 0);
        assert!(reconciler.should_execute("a", &["1"]));
        assert!(!reconciler.should_execute("a", &["1"]));
        assert!(reconciler.should_execute("b", &["2"]));
        assert!(!reconciler.should_execute("b", &["2"]));
        assert!(!reconciler.should_execute("b", &["2"]));
        assert_eq!(reconciler.total_skips(), 3);
    }

    #[test]
    fn test_invalidate_resets_stage() {
        let mut reconciler = FingerprintReconciler::new();
        assert!(reconciler.should_execute("stage_a", &["x"]));
        assert!(!reconciler.should_execute("stage_a", &["x"]));
        reconciler.invalidate("stage_a");
        assert!(reconciler.should_execute("stage_a", &["x"]));
    }

    #[test]
    fn test_reset_all_clears_everything() {
        let mut reconciler = FingerprintReconciler::new();
        assert!(reconciler.should_execute("a", &["1"]));
        assert!(!reconciler.should_execute("a", &["1"]));
        assert!(reconciler.should_execute("b", &["2"]));
        assert!(!reconciler.should_execute("b", &["2"]));
        reconciler.record_skip("a", 512);
        reconciler.record_skip("b", 256);

        reconciler.reset_all();
        assert_eq!(reconciler.total_skips(), 0);
        assert_eq!(reconciler.total_bytes_saved(), 0);
        assert_eq!(reconciler.skip_count("a"), 0);
        // After reset, first call should execute again
        assert!(reconciler.should_execute("a", &["1"]));
    }

    #[test]
    fn test_fingerprint_from_text() {
        let fp1 = Fingerprint::from_text("hello");
        let fp2 = Fingerprint::from_text("hello");
        let fp3 = Fingerprint::from_text("world");
        assert_eq!(fp1, fp2);
        assert_ne!(fp1, fp3);
    }

    #[test]
    fn test_fingerprint_from_values() {
        let fp1 = Fingerprint::from_values(&["a", "b"]);
        let fp2 = Fingerprint::from_values(&["a", "b"]);
        let fp3 = Fingerprint::from_values(&["a", "c"]);
        assert_eq!(fp1, fp2);
        assert_ne!(fp1, fp3);
    }

    #[test]
    fn test_fingerprint_from_values_order_matters() {
        let fp1 = Fingerprint::from_values(&["a", "b"]);
        let fp2 = Fingerprint::from_values(&["b", "a"]);
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_record_skip_does_not_affect_skip_count() {
        let mut reconciler = FingerprintReconciler::new();
        assert!(reconciler.should_execute("s", &["x"]));
        assert!(!reconciler.should_execute("s", &["x"]));
        assert_eq!(reconciler.skip_count("s"), 1);
        assert_eq!(reconciler.total_bytes_saved(), 0);
        reconciler.record_skip("s", 999);
        assert_eq!(reconciler.skip_count("s"), 1);
        assert_eq!(reconciler.total_bytes_saved(), 999);
    }
}
