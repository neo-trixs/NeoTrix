const IDLE_THRESHOLD_MS: u64 = 2000;
const MAX_REVERBERATION_STEPS: usize = 20;
const DEFAULT_MIN_IDLE_SECS: u64 = 5;
const COGNITIVE_LOAD_THRESHOLD: f64 = 0.3;
const VSA_SIMILARITY_THRESHOLD: f64 = 0.85;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

/// Trait for reading cross-session memory entries for DMN association discovery.
/// Defined locally to avoid reverse architecture dependency (core/ → neotrix/).
pub trait CrossSessionMemoryReader {
    type Entry: MemoryEntryReader;
    fn entries(&self) -> Vec<&Self::Entry>;
}

/// Trait for inspecting a single memory entry during association discovery.
pub trait MemoryEntryReader {
    fn key(&self) -> &str;
    fn category_name(&self) -> String;
    fn vsa_bytes(&self) -> Option<&[u8]>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DMNActivity {
    Idle,
    Reverberating,
    Consolidating,
    Exploring,
}

impl DMNActivity {
    pub fn name(&self) -> &'static str {
        match self {
            DMNActivity::Idle => "idle",
            DMNActivity::Reverberating => "reverberating",
            DMNActivity::Consolidating => "consolidating",
            DMNActivity::Exploring => "exploring",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReverberationSample {
    pub timestamp: Instant,
    pub coherence: f64,
    pub novelty: f64,
}

#[derive(Debug, Clone)]
pub struct DmnReport {
    pub activity: DMNActivity,
    pub insights_generated: u64,
    pub associations_discovered: u64,
    pub vectors_defragmented: usize,
    pub duplicates_removed: usize,
}

#[derive(Debug, Clone)]
pub struct DefragReport {
    pub merged_clusters: usize,
    pub duplicates_removed: usize,
    pub vectors_before: usize,
    pub vectors_after: usize,
}

#[derive(Debug, Clone)]
pub struct Association {
    pub source_key: String,
    pub target_key: String,
    pub strength: f64,
    pub category: String,
}

#[derive(Debug, Clone)]
pub struct DefaultModeNetwork {
    pub activity: DMNActivity,
    pub last_external_input: Instant,
    pub reverberation_samples: Vec<ReverberationSample>,
    pub reverberation_step: usize,
    pub idle_since: Option<Instant>,
    pub total_reverberations: u64,
    pub novelty_threshold: f64,
    pub insights_generated: u64,
    pub associations_discovered: u64,
    pub min_idle_seconds: u64,
}

impl Default for DefaultModeNetwork {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultModeNetwork {
    pub fn new() -> Self {
        Self {
            activity: DMNActivity::Idle,
            last_external_input: Instant::now(),
            reverberation_samples: Vec::with_capacity(MAX_REVERBERATION_STEPS),
            reverberation_step: 0,
            idle_since: None,
            total_reverberations: 0,
            novelty_threshold: 0.3,
            insights_generated: 0,
            associations_discovered: 0,
            min_idle_seconds: DEFAULT_MIN_IDLE_SECS,
        }
    }

    pub fn tick(&mut self, has_external_input: bool) -> DMNActivity {
        if has_external_input {
            self.last_external_input = Instant::now();
            self.idle_since = None;
            self.activity = DMNActivity::Idle;
            self.reverberation_step = 0;
            return DMNActivity::Idle;
        }

        let idle_duration = Instant::now().duration_since(self.last_external_input);
        if idle_duration < Duration::from_millis(IDLE_THRESHOLD_MS) {
            self.activity = DMNActivity::Idle;
            return DMNActivity::Idle;
        }

        if self.idle_since.is_none() {
            self.idle_since = Some(Instant::now());
        }

        if self.reverberation_step < MAX_REVERBERATION_STEPS {
            self.activity = DMNActivity::Reverberating;
            self.reverberation_step += 1;
        } else {
            self.activity = DMNActivity::Consolidating;
        }

        self.activity
    }

    /// Try to run DMN if idle conditions are met:
    /// cognitive_load < 0.3 AND enough time since last external input
    pub fn try_run(
        &mut self,
        cognitive_load: f64,
        elapsed_since_last: Duration,
    ) -> Option<DmnReport> {
        if cognitive_load >= COGNITIVE_LOAD_THRESHOLD {
            self.activity = DMNActivity::Idle;
            return None;
        }

        if elapsed_since_last < Duration::from_secs(self.min_idle_seconds) {
            self.activity = DMNActivity::Idle;
            return None;
        }

        let has_input = elapsed_since_last < Duration::from_millis(IDLE_THRESHOLD_MS);
        let activity = self.tick(!has_input);

        Some(DmnReport {
            activity,
            insights_generated: self.insights_generated,
            associations_discovered: self.associations_discovered,
            vectors_defragmented: 0,
            duplicates_removed: 0,
        })
    }

    /// Defragment VSA store: merge similar vectors, remove duplicates
    pub fn defragment(&self, store: &mut HashMap<String, Vec<u8>>) -> DefragReport {
        let vectors_before = store.len();
        let mut merged = 0usize;
        let mut removed = 0usize;

        let keys: Vec<String> = store.keys().cloned().collect();
        let mut to_remove: Vec<String> = Vec::new();

        for i in 0..keys.len() {
            if to_remove.contains(&keys[i]) {
                continue;
            }
            let a = match store.get(&keys[i]) {
                Some(v) => v,
                None => continue,
            };
            for j in (i + 1)..keys.len() {
                if to_remove.contains(&keys[j]) {
                    continue;
                }
                let b = match store.get(&keys[j]) {
                    Some(v) => v,
                    None => continue,
                };
                let sim = QuantizedVSA::similarity(a, b);
                if sim >= VSA_SIMILARITY_THRESHOLD {
                    to_remove.push(keys[j].clone());
                    merged += 1;
                }
            }
        }

        for k in &to_remove {
            store.remove(k);
            removed += 1;
        }

        DefragReport {
            merged_clusters: merged,
            duplicates_removed: removed,
            vectors_before,
            vectors_after: store.len(),
        }
    }

    /// Discover cross-session associations in memory
    pub fn discover_associations(
        &self,
        memory: &impl CrossSessionMemoryReader,
    ) -> Vec<Association> {
        let all: Vec<_> = memory
            .entries()
            .into_iter()
            .filter(|e| e.vsa_bytes().is_some())
            .collect();

        let mut associations = Vec::new();
        let strength_threshold = 0.6;

        for i in 0..all.len() {
            let a_vsa = match all[i].vsa_bytes() {
                Some(v) => v,
                None => continue,
            };
            for j in (i + 1)..all.len() {
                let b_vsa = match all[j].vsa_bytes() {
                    Some(v) => v,
                    None => continue,
                };
                let sim = QuantizedVSA::similarity(a_vsa, b_vsa);
                if sim >= strength_threshold {
                    let cat_a = all[i].category_name();
                    let cat_b = all[j].category_name();
                    let category = if cat_a == cat_b {
                        cat_a
                    } else {
                        format!("{}_{}", cat_a, cat_b)
                    };
                    associations.push(Association {
                        source_key: all[i].key().to_string(),
                        target_key: all[j].key().to_string(),
                        strength: sim,
                        category,
                    });
                }
            }
        }

        associations
    }

    /// Generate spontaneous insight from defragmentation
    pub fn generate_insight(&self, store: &HashMap<String, Vec<u8>>) -> Option<String> {
        if store.len() < 3 {
            return None;
        }

        let keys: Vec<&String> = store.keys().take(50).collect();
        let mut max_sim = 0.0f64;
        let mut insight_pair: Option<(&String, &String)> = None;

        for i in 0..keys.len() {
            let a = match store.get(keys[i]) {
                Some(v) => v,
                None => continue,
            };
            for j in (i + 1)..keys.len() {
                let b = match store.get(keys[j]) {
                    Some(v) => v,
                    None => continue,
                };
                let sim = QuantizedVSA::similarity(a, b);
                if sim > max_sim && sim < VSA_SIMILARITY_THRESHOLD {
                    max_sim = sim;
                    insight_pair = Some((keys[i], keys[j]));
                }
            }
        }

        insight_pair.map(|(a, b)| {
            format!(
                "DMN insight: latent association ({:.0}%) between '{}' and '{}'",
                max_sim * 100.0,
                a,
                b
            )
        })
    }

    pub fn record_reverberation(&mut self, coherence: f64, novelty: f64) {
        if self.reverberation_samples.len() >= MAX_REVERBERATION_STEPS {
            self.reverberation_samples.remove(0);
        }
        self.reverberation_samples.push(ReverberationSample {
            timestamp: Instant::now(),
            coherence,
            novelty,
        });
        self.total_reverberations += 1;
    }

    pub fn average_reverberation_coherence(&self) -> f64 {
        if self.reverberation_samples.is_empty() {
            return 0.0;
        }
        self.reverberation_samples
            .iter()
            .map(|s| s.coherence)
            .sum::<f64>()
            / self.reverberation_samples.len() as f64
    }

    pub fn average_novelty(&self) -> f64 {
        if self.reverberation_samples.is_empty() {
            return 0.0;
        }
        self.reverberation_samples
            .iter()
            .map(|s| s.novelty)
            .sum::<f64>()
            / self.reverberation_samples.len() as f64
    }

    pub fn is_idle(&self) -> bool {
        self.activity == DMNActivity::Idle
    }

    pub fn is_reverberating(&self) -> bool {
        self.activity == DMNActivity::Reverberating
    }

    pub fn reset(&mut self) {
        self.activity = DMNActivity::Idle;
        self.last_external_input = Instant::now();
        self.reverberation_samples.clear();
        self.reverberation_step = 0;
        self.idle_since = None;
    }

    pub fn idle_duration(&self) -> Duration {
        self.idle_since
            .map(|t| Instant::now().duration_since(t))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dmn_starts_idle() {
        let dmn = DefaultModeNetwork::new();
        assert_eq!(dmn.activity, DMNActivity::Idle);
        assert!(dmn.is_idle());
    }

    #[test]
    fn test_external_input_keeps_idle() {
        let mut dmn = DefaultModeNetwork::new();
        let activity = dmn.tick(true);
        assert_eq!(activity, DMNActivity::Idle);
    }

    #[test]
    fn test_no_input_triggers_reverberation() {
        let mut dmn = DefaultModeNetwork::new();
        dmn.last_external_input = Instant::now() - Duration::from_millis(IDLE_THRESHOLD_MS + 100);
        let activity = dmn.tick(false);
        assert_eq!(activity, DMNActivity::Reverberating);
    }

    #[test]
    fn test_reverberation_steps_limited() {
        let mut dmn = DefaultModeNetwork::new();
        dmn.last_external_input = Instant::now() - Duration::from_millis(IDLE_THRESHOLD_MS + 100);
        for _ in 0..MAX_REVERBERATION_STEPS {
            dmn.tick(false);
        }
        assert_eq!(dmn.activity, DMNActivity::Consolidating);
    }

    #[test]
    fn test_record_reverberation() {
        let mut dmn = DefaultModeNetwork::new();
        dmn.record_reverberation(0.8, 0.3);
        assert_eq!(dmn.reverberation_samples.len(), 1);
        assert_eq!(dmn.total_reverberations, 1);
    }

    #[test]
    fn test_average_coherence() {
        let mut dmn = DefaultModeNetwork::new();
        dmn.record_reverberation(0.8, 0.2);
        dmn.record_reverberation(0.9, 0.3);
        assert!((dmn.average_reverberation_coherence() - 0.85).abs() < 1e-9);
    }

    #[test]
    fn test_reset_clears_state() {
        let mut dmn = DefaultModeNetwork::new();
        dmn.last_external_input = Instant::now() - Duration::from_millis(IDLE_THRESHOLD_MS + 100);
        dmn.tick(false);
        dmn.record_reverberation(0.8, 0.3);
        dmn.reset();
        assert!(dmn.is_idle());
        assert_eq!(dmn.reverberation_samples.len(), 0);
    }

    #[test]
    fn test_dmn_activity_names() {
        assert_eq!(DMNActivity::Idle.name(), "idle");
        assert_eq!(DMNActivity::Reverberating.name(), "reverberating");
        assert_eq!(DMNActivity::Exploring.name(), "exploring");
    }

    #[test]
    fn test_try_run_high_cog_load_returns_none() {
        let mut dmn = DefaultModeNetwork::new();
        let result = dmn.try_run(0.8, Duration::from_secs(10));
        assert!(result.is_none());
    }

    #[test]
    fn test_try_run_short_idle_returns_none() {
        let mut dmn = DefaultModeNetwork::new();
        let result = dmn.try_run(0.1, Duration::from_millis(100));
        assert!(result.is_none());
    }

    #[test]
    fn test_try_run_low_load_long_idle_returns_report() {
        let mut dmn = DefaultModeNetwork::new();
        dmn.last_external_input = Instant::now() - Duration::from_secs(10);
        let result = dmn.try_run(0.1, Duration::from_secs(10));
        assert!(result.is_some());
        let report = result.unwrap();
        assert_eq!(report.activity, DMNActivity::Reverberating);
    }

    #[test]
    fn test_defragment_no_duplicates() {
        let dmn = DefaultModeNetwork::new();
        let mut store: HashMap<String, Vec<u8>> = HashMap::new();
        store.insert("a".to_string(), QuantizedVSA::seeded_random(1, 64));
        store.insert("b".to_string(), QuantizedVSA::seeded_random(2, 64));
        let report = dmn.defragment(&mut store);
        assert_eq!(report.duplicates_removed, 0);
        assert_eq!(report.vectors_before, 2);
        assert_eq!(report.vectors_after, 2);
    }

    #[test]
    fn test_defragment_identical_removes_duplicate() {
        let dmn = DefaultModeNetwork::new();
        let mut store: HashMap<String, Vec<u8>> = HashMap::new();
        let v = QuantizedVSA::seeded_random(42, 64);
        store.insert("a".to_string(), v.clone());
        store.insert("b".to_string(), v);
        let report = dmn.defragment(&mut store);
        assert_eq!(report.duplicates_removed, 1);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_generate_insight_small_store_returns_none() {
        let dmn = DefaultModeNetwork::new();
        let mut store: HashMap<String, Vec<u8>> = HashMap::new();
        store.insert("a".to_string(), QuantizedVSA::seeded_random(1, 64));
        store.insert("b".to_string(), QuantizedVSA::seeded_random(2, 64));
        assert!(dmn.generate_insight(&store).is_none());
    }

    #[test]
    fn test_try_run_updates_activity() {
        let mut dmn = DefaultModeNetwork::new();
        dmn.last_external_input = Instant::now() - Duration::from_secs(10);
        let _ = dmn.try_run(0.1, Duration::from_secs(10));
        assert_eq!(dmn.activity, DMNActivity::Reverberating);
    }
}
