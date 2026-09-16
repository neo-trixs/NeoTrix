use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f64,
}

pub struct VectorIndex {
    dimension: usize,
    entries: Vec<VectorEntry>,
}

impl VectorIndex {
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, entry: VectorEntry) {
        if entry.vector.len() == self.dimension {
            self.entries.push(entry);
        }
    }

    pub fn search(&self, query: &[f32], top_k: usize) -> Vec<SearchResult> {
        if query.len() != self.dimension {
            return Vec::new();
        }
        let mut scores: Vec<(usize, f64)> = self.entries.iter().enumerate().map(|(i, e)| {
            let dot: f64 = query.iter().zip(e.vector.iter()).map(|(a, b)| *a as f64 * *b as f64).sum();
            let norm_a: f64 = query.iter().map(|a| *a as f64 * *a as f64).sum::<f64>().sqrt();
            let norm_b: f64 = e.vector.iter().map(|b| *b as f64 * *b as f64).sum::<f64>().sqrt();
            let score = if norm_a == 0.0 || norm_b == 0.0 { 0.0 } else { dot / (norm_a * norm_b) };
            (i, score)
        }).collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.iter().take(top_k).map(|(i, score)| SearchResult {
            id: self.entries[*i].id.clone(),
            score: *score,
        }).collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_index() {
        let idx = VectorIndex::new(3);
        assert!(idx.is_empty());
        let results = idx.search(&[1.0, 0.0, 0.0], 5);
        assert!(results.is_empty());
    }
}
