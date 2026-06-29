#[derive(Debug, Clone)]
pub struct KVCacheEntry {
    pub key: Vec<f64>,
    pub value: Vec<f64>,
    pub conflict_score: f64,
}

#[derive(Debug, Clone)]
pub struct KVCacheConsolidation {
    pub entries: Vec<KVCacheEntry>,
    pub forget_gate_threshold: f64,
}

impl KVCacheConsolidation {
    pub fn new(threshold: f64) -> Self {
        Self {
            entries: vec![],
            forget_gate_threshold: threshold,
        }
    }
    pub fn insert(&mut self, key: Vec<f64>, value: Vec<f64>) {
        self.entries.push(KVCacheEntry {
            key,
            value,
            conflict_score: 0.0,
        });
    }
    pub fn consolidate(&mut self) {
        let mut scores: Vec<f64> = self.entries.iter().map(|e| e.conflict_score).collect();
        scores.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        if let Some(threshold) = scores.get(scores.len() / 2) {
            self.entries.retain(|e| e.conflict_score <= *threshold);
        }
    }
}
