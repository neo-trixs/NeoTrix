use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

const MAX_ENTRIES: usize = 10_000;
const FREQ_WEIGHT: f64 = 0.6;
const RECENCY_WEIGHT: f64 = 0.4;

#[derive(Clone, Debug)]
pub struct FrecencyEntry {
    pub access_count: u64,
    pub last_access: Instant,
}

#[derive(Clone, Debug)]
pub struct FrecencyIndex {
    entries: HashMap<PathBuf, FrecencyEntry>,
}

impl Default for FrecencyIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl FrecencyIndex {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn record_access(&mut self, path: PathBuf) {
        let entry = self.entries.entry(path).or_insert(FrecencyEntry {
            access_count: 0,
            last_access: Instant::now(),
        });
        entry.access_count += 1;
        entry.last_access = Instant::now();

        if self.entries.len() > MAX_ENTRIES {
            self.prune();
        }
    }

    pub fn score(&self, path: &PathBuf) -> f64 {
        self.entries.get(path).map_or(0.0, |e| {
            let freq = (e.access_count as f64).min(100.0) / 100.0;
            let recency = Instant::now()
                .duration_since(e.last_access)
                .as_secs_f64()
                .max(0.0);
            let recency_norm = (1.0 / (1.0 + recency / 3600.0)).max(0.0);
            freq * FREQ_WEIGHT + recency_norm * RECENCY_WEIGHT
        })
    }

    pub fn rank(&self, paths: &mut [String]) {
        paths.sort_by(|a, b| {
            let pa = PathBuf::from(a);
            let pb = PathBuf::from(b);
            let sa = self.score(&pa);
            let sb = self.score(&pb);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    fn prune(&mut self) {
        let mut entries: Vec<(PathBuf, &FrecencyEntry)> =
            self.entries.iter().map(|(k, v)| (k.clone(), v)).collect();
        entries.sort_by(|a, b| {
            let sa = (a.1.access_count as f64).min(100.0) / 100.0 * FREQ_WEIGHT
                + (1.0
                    / (1.0
                        + Instant::now().duration_since(a.1.last_access).as_secs_f64() / 3600.0))
                    .max(0.0)
                    * RECENCY_WEIGHT;
            let sb = (b.1.access_count as f64).min(100.0) / 100.0 * FREQ_WEIGHT
                + (1.0
                    / (1.0
                        + Instant::now().duration_since(b.1.last_access).as_secs_f64() / 3600.0))
                    .max(0.0)
                    * RECENCY_WEIGHT;
            sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
        });
        let keep = MAX_ENTRIES / 2;
        let to_remove: Vec<PathBuf> = entries.iter().skip(keep).map(|(k, _)| k.clone()).collect();
        for k in to_remove {
            self.entries.remove(&k);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frecency_record_and_score() {
        let mut idx = FrecencyIndex::new();
        let p = PathBuf::from("/test/file.rs");
        assert_eq!(idx.score(&p), 0.0);
        idx.record_access(p.clone());
        assert!(idx.score(&p) > 0.0);
    }

    #[test]
    fn test_frecency_ranking() {
        let mut idx = FrecencyIndex::new();
        let a = PathBuf::from("/a.rs");
        let b = PathBuf::from("/b.rs");
        idx.record_access(a.clone());
        idx.record_access(a.clone());
        idx.record_access(a.clone());
        idx.record_access(b.clone());
        let mut paths = vec!["/b.rs".to_string(), "/a.rs".to_string()];
        idx.rank(&mut paths);
        assert_eq!(paths[0], "/a.rs");
    }

    #[test]
    fn test_prune_evicts_oldest() {
        let mut idx = FrecencyIndex::new();
        for i in 0..MAX_ENTRIES + 100 {
            idx.record_access(PathBuf::from(format!("/file_{}.rs", i)));
        }
        assert!(idx.entries.len() <= MAX_ENTRIES);
    }
}
