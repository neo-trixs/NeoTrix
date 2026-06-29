use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// Ebbinghaus exponential decay: R = e^(-t/S)
pub fn ebbinghaus_decay(elapsed_hours: f64, stability_hours: f64) -> f64 {
    if stability_hours <= 0.0 {
        return 0.0;
    }
    (-elapsed_hours / stability_hours).exp()
}

/// Importance score combining access frequency, recency, and intrinsic value
pub fn importance_score(access_count: u64, hours_since_access: f64, base_importance: f64) -> f64 {
    let freq_factor = (1.0 + (access_count as f64).ln()).min(5.0) / 5.0;
    let recency_factor = ebbinghaus_decay(hours_since_access, 72.0);
    let importance_weight = base_importance.clamp(0.0, 1.0);
    0.25 * freq_factor + 0.35 * recency_factor + 0.40 * importance_weight
}

#[derive(Debug, Clone)]
pub struct ForgettingReport {
    pub evicted: usize,
    pub archived: usize,
    pub decayed: usize,
    pub total_before: usize,
    pub total_after: usize,
}

impl ForgettingReport {
    pub fn new() -> Self {
        Self {
            evicted: 0,
            archived: 0,
            decayed: 0,
            total_before: 0,
            total_after: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ForgettingStrategy {
    pub max_entries: usize,
    pub min_importance_threshold: f64,
    pub decay_stability_hours: f64,
    pub archive_dir: Option<PathBuf>,
}

impl ForgettingStrategy {
    pub fn new(max_entries: usize, archive_dir: Option<PathBuf>) -> Self {
        Self {
            max_entries,
            min_importance_threshold: 0.15,
            decay_stability_hours: 168.0,
            archive_dir,
        }
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.min_importance_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    pub fn with_decay_stability(mut self, hours: f64) -> Self {
        self.decay_stability_hours = hours.max(1.0);
        self
    }

    pub fn apply(
        &mut self,
        entries: &mut HashMap<String, crate::core::nt_core_knowledge::KnowledgeEntry>,
    ) -> ForgettingReport {
        let mut report = ForgettingReport::new();
        report.total_before = entries.len();

        let now = chrono::Utc::now().timestamp();

        let mut scored: Vec<(String, f64)> = entries
            .iter()
            .map(|(id, e)| {
                let hours_since_access = if e.updated_at > 0 {
                    (now - e.updated_at) as f64 / 3600.0
                } else {
                    0.0
                };
                let score = importance_score(e.access_count, hours_since_access, e.importance);
                (id.clone(), score)
            })
            .collect();

        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut to_evict: Vec<String> = Vec::new();
        let mut to_archive: Vec<String> = Vec::new();

        if scored.len() > self.max_entries {
            let excess = scored.len() - self.max_entries;
            for (id, score) in &scored[..excess] {
                if *score < self.min_importance_threshold {
                    to_evict.push(id.clone());
                } else {
                    to_archive.push(id.clone());
                }
            }
        }

        for (id, _) in &scored {
            if id.len() > 1 {
                let score = scored
                    .iter()
                    .find(|(i, _)| i == id)
                    .map(|(_, s)| *s)
                    .unwrap_or(0.0);
                if score < self.min_importance_threshold * 0.5
                    && entries.len() > self.max_entries / 2
                {
                    if !to_evict.contains(id) {
                        to_evict.push(id.clone());
                    }
                }
            }
        }

        to_evict.dedup();
        to_archive.dedup();

        for id in &to_archive {
            if let Some(entry) = entries.get(id) {
                if self.archive_dir.is_some() {
                    if self.archive_entry(entry).is_ok() {
                        report.archived += 1;
                    }
                }
            }
        }

        for id in &to_evict {
            entries.remove(id);
            report.evicted += 1;
        }

        report.decayed = to_archive.len();
        report.total_after = entries.len();
        report
    }

    fn archive_entry(
        &self,
        entry: &crate::core::nt_core_knowledge::KnowledgeEntry,
    ) -> Result<(), io::Error> {
        let dir = match &self.archive_dir {
            Some(d) => d,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "no archive dir")),
        };
        fs::create_dir_all(dir)?;
        let filename = format!(
            "entry_{}.json",
            entry.id.replace(std::path::MAIN_SEPARATOR, "_")
        );
        let path = dir.join(&filename);
        let json = serde_json::to_string_pretty(entry)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let mut file = fs::File::create(&path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn threshold_report(
        &self,
        entries: &HashMap<String, crate::core::nt_core_knowledge::KnowledgeEntry>,
    ) -> Vec<(String, f64)> {
        let now = chrono::Utc::now().timestamp();
        let mut results = Vec::new();
        for (id, e) in entries {
            let hours_since_access = if e.updated_at > 0 {
                (now - e.updated_at) as f64 / 3600.0
            } else {
                0.0
            };
            let score = importance_score(e.access_count, hours_since_access, e.importance);
            if score < self.min_importance_threshold {
                results.push((id.clone(), score));
            }
        }
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(
        id: &str,
        access_count: u64,
        hours_ago: f64,
        importance: f64,
    ) -> (String, crate::core::nt_core_knowledge::KnowledgeEntry) {
        let now = chrono::Utc::now().timestamp();
        let updated = now - (hours_ago * 3600.0) as i64;
        let entry = crate::core::nt_core_knowledge::KnowledgeEntry {
            id: id.to_string(),
            title: format!("entry_{}", id),
            body: "test body".to_string(),
            summary: "test".to_string(),
            source: crate::core::nt_core_knowledge::KnowledgeSourceType::UserInput,
            source_url: String::new(),
            tags: Vec::new(),
            dimensions: Vec::new(),
            embedding: None,
            confidence: 1.0,
            importance,
            created_at: updated - 3600,
            updated_at: updated,
            access_count,
            related_ids: Vec::new(),
            provenance_hash: None,
            cross_references: Vec::new(),
            evidence_ids: vec![],
            valid_from: None,
            valid_to: None,
        };
        (id.to_string(), entry)
    }

    #[test]
    fn test_ebbinghaus_decay_immediate() {
        let r = ebbinghaus_decay(0.0, 24.0);
        assert!((r - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_ebbinghaus_decay_one_stability() {
        let r = ebbinghaus_decay(24.0, 24.0);
        assert!((r - 0.36787944117).abs() < 1e-6);
    }

    #[test]
    fn test_ebbinghaus_decay_long_elapsed() {
        let r = ebbinghaus_decay(720.0, 24.0);
        assert!(r < 0.001);
    }

    #[test]
    fn test_ebbinghaus_decay_zero_stability() {
        let r = ebbinghaus_decay(10.0, 0.0);
        assert!((r - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_importance_score_high_frequency() {
        let score = importance_score(1000, 1.0, 0.9);
        assert!(score > 0.5);
    }

    #[test]
    fn test_importance_score_low_decay() {
        let score = importance_score(0, 720.0, 0.1);
        assert!(score < 0.3);
    }

    #[test]
    fn test_importance_score_clamped() {
        let score = importance_score(0, 10000.0, 0.0);
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_strategy_new() {
        let s = ForgettingStrategy::new(100, None);
        assert_eq!(s.max_entries, 100);
        assert!((s.min_importance_threshold - 0.15).abs() < 1e-9);
    }

    #[test]
    fn test_apply_no_entries() {
        let mut s = ForgettingStrategy::new(100, None);
        let mut entries: HashMap<String, crate::core::nt_core_knowledge::KnowledgeEntry> =
            HashMap::new();
        let report = s.apply(&mut entries);
        assert_eq!(report.total_before, 0);
        assert_eq!(report.total_after, 0);
    }

    #[test]
    fn test_apply_under_max() {
        let mut s = ForgettingStrategy::new(100, None);
        let mut entries: HashMap<String, crate::core::nt_core_knowledge::KnowledgeEntry> =
            HashMap::new();
        let (k, v) = make_entry("a", 10, 1.0, 0.9);
        entries.insert(k, v);
        let (k, v) = make_entry("b", 5, 2.0, 0.8);
        entries.insert(k, v);
        let report = s.apply(&mut entries);
        assert_eq!(report.total_before, 2);
        assert_eq!(report.total_after, 2);
        assert_eq!(report.evicted, 0);
    }

    #[test]
    fn test_apply_evicts_low_importance() {
        let mut s = ForgettingStrategy::new(10, None).with_threshold(0.5);
        let mut entries: HashMap<String, crate::core::nt_core_knowledge::KnowledgeEntry> =
            HashMap::new();
        for i in 0..20 {
            let (k, v) = make_entry(&format!("id_{}", i), 0, 1000.0, 0.01);
            entries.insert(k, v);
        }
        let report = s.apply(&mut entries);
        assert!(report.evicted > 0);
        assert!(entries.len() < 20);
    }

    #[test]
    fn test_threshold_report() {
        let s = ForgettingStrategy::new(100, None).with_threshold(0.8);
        let mut entries: HashMap<String, crate::core::nt_core_knowledge::KnowledgeEntry> =
            HashMap::new();
        let (k, v) = make_entry("high", 100, 1.0, 0.9);
        entries.insert(k, v);
        let (k, v) = make_entry("low", 0, 1000.0, 0.01);
        entries.insert(k, v);
        let low = s.threshold_report(&entries);
        assert_eq!(low.len(), 1);
        assert_eq!(low[0].0, "low");
    }

    #[test]
    fn test_archive_dir_creation() {
        let tmp = std::env::temp_dir().join("neotrix_forget_test_archive");
        let _ = fs::remove_dir_all(&tmp);
        let mut s = ForgettingStrategy::new(5, Some(tmp.clone()));
        let mut entries: HashMap<String, crate::core::nt_core_knowledge::KnowledgeEntry> =
            HashMap::new();
        for i in 0..10 {
            let (k, v) = make_entry(&format!("a{}", i), 0, 500.0, 0.05);
            entries.insert(k, v);
        }
        s.apply(&mut entries);
        let read = fs::read_dir(&tmp);
        assert!(read.is_ok());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_decay_stability_config() {
        let s = ForgettingStrategy::new(100, None).with_decay_stability(336.0);
        assert!((s.decay_stability_hours - 336.0).abs() < 1e-9);
    }

    #[test]
    fn test_report_counts() {
        let mut s = ForgettingStrategy::new(5, None);
        let mut entries: HashMap<String, crate::core::nt_core_knowledge::KnowledgeEntry> =
            HashMap::new();
        for i in 0..20 {
            let (k, v) = make_entry(&format!("x{}", i), 0, 999.0, 0.02);
            entries.insert(k, v);
        }
        let report = s.apply(&mut entries);
        assert_eq!(report.total_before, 20);
        assert!(report.total_after <= 5);
        assert_eq!(report.evicted + report.archived, 20 - report.total_after);
    }
}
