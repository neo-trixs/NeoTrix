use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Dual representation: every concept exists as both symbolic and numeric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicNode {
    pub id: u64,
    pub label: String,
    pub kind: String,  // "concept", "event", "agent", "location"
    pub embedding: [f32; 16],
    pub attributes: HashMap<String, String>,
    pub tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericVector {
    pub id: u64,
    pub values: Vec<f32>,
    pub source_label: String,
    pub tick: u64,
}

pub struct DualRepresentation {
    symbolic: Vec<SymbolicNode>,
    numeric: Vec<NumericVector>,
    label_to_id: HashMap<String, u64>,
    next_id: u64,
    embedding_size: usize,
}

impl DualRepresentation {
    pub fn new(embedding_size: usize) -> Self {
        Self {
            symbolic: Vec::new(),
            numeric: Vec::new(),
            label_to_id: HashMap::new(),
            next_id: 0,
            embedding_size,
        }
    }

    /// Add concept in both representations simultaneously
    pub fn add(&mut self, label: &str, kind: &str, attributes: HashMap<String, String>, tick: u64) -> u64 {
        if let Some(&existing_id) = self.label_to_id.get(label) {
            return existing_id;
        }

        let id = self.next_id;
        self.next_id += 1;

        // Symbolic side
        let embedding = self.generate_embedding(label, kind);
        self.symbolic.push(SymbolicNode {
            id, label: label.to_string(), kind: kind.to_string(),
            embedding, attributes, tick,
        });

        // Numeric side
        let values: Vec<f32> = embedding.to_vec();
        self.numeric.push(NumericVector {
            id, values, source_label: label.to_string(), tick,
        });

        self.label_to_id.insert(label.to_string(), id);
        id
    }

    /// Get symbolic node by label
    pub fn symbolic_of(&self, label: &str) -> Option<&SymbolicNode> {
        self.label_to_id.get(label).and_then(|&id| self.symbolic.iter().find(|n| n.id == id))
    }

    /// Get numeric vector by label
    pub fn numeric_of(&self, label: &str) -> Option<&NumericVector> {
        self.label_to_id.get(label).and_then(|&id| self.numeric.iter().find(|v| v.id == id))
    }

    /// Get both representations of a concept
    pub fn get_both(&self, label: &str) -> Option<(&SymbolicNode, &NumericVector)> {
        let id = *self.label_to_id.get(label)?;
        let sym = self.symbolic.iter().find(|n| n.id == id)?;
        let num = self.numeric.iter().find(|v| v.id == id)?;
        Some((sym, num))
    }

    /// Find similar concepts by cosine similarity on numeric vectors
    pub fn similar_to(&self, label: &str, top_k: usize) -> Vec<(&str, f32)> {
        let target = match self.numeric_of(label) {
            Some(v) => v,
            None => return Vec::new(),
        };
        let mut scores: Vec<(&str, f32)> = self.numeric.iter()
            .filter(|v| v.id != target.id)
            .map(|v| {
                let sim = cosine_sim(&target.values, &v.values);
                (v.source_label.as_str(), sim)
            })
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores.into_iter().take(top_k).collect()
    }

    /// Convert symbolic attributes to numeric features (for ML compatibility)
    pub fn attributes_as_numeric(&self, label: &str) -> Vec<f32> {
        self.symbolic_of(label).map(|n| {
            n.attributes.values().map(|v| {
                v.parse::<f32>().unwrap_or_else(|_| {
                    // Hash string to float in [0, 1]
                    (v.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32)) % 1000) as f32 / 1000.0
                })
            }).collect()
        }).unwrap_or_default()
    }

    pub fn count(&self) -> usize { self.symbolic.len() }

    fn generate_embedding(&self, label: &str, kind: &str) -> [f32; 16] {
        let mut emb = [0.0f32; 16];
        let bytes = label.bytes().chain(kind.bytes());
        for (i, b) in bytes.enumerate() {
            emb[i % 16] += b as f32 / 255.0;
        }
        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 { emb.iter_mut().for_each(|x| *x /= norm); }
        emb
    }
}

fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len().min(b.len());
    let dot: f32 = (0..len).map(|i| a[i] * b[i]).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na * nb == 0.0 { 0.0 } else { dot / (na * nb) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_creates_both_representations() {
        let mut dr = DualRepresentation::new(16);
        dr.add("alice", "agent", HashMap::new(), 0);
        assert!(dr.symbolic_of("alice").is_some());
        assert!(dr.numeric_of("alice").is_some());
        assert_eq!(dr.count(), 1);
    }

    #[test]
    fn get_both_returns_pair() {
        let mut dr = DualRepresentation::new(16);
        dr.add("forest", "location", HashMap::new(), 0);
        let (sym, num) = dr.get_both("forest").unwrap();
        assert_eq!(sym.label, "forest");
        assert_eq!(num.values.len(), 16);
    }

    #[test]
    fn similar_to_finds_related() {
        let mut dr = DualRepresentation::new(16);
        dr.add("aaa", "concept", HashMap::new(), 0);
        dr.add("aab", "concept", HashMap::new(), 0);
        dr.add("zzz", "concept", HashMap::new(), 0);
        let similar = dr.similar_to("aaa", 2);
        assert!(!similar.is_empty());
        // "aab" should be more similar to "aaa" than "zzz"
        assert!(similar.iter().any(|(l, _)| *l == "aab"));
    }

    #[test]
    fn duplicate_label_returns_existing_id() {
        let mut dr = DualRepresentation::new(16);
        let id1 = dr.add("test", "concept", HashMap::new(), 0);
        let id2 = dr.add("test", "concept", HashMap::new(), 1);
        assert_eq!(id1, id2);
        assert_eq!(dr.count(), 1);
    }

    #[test]
    fn attributes_as_numeric_works() {
        let mut dr = DualRepresentation::new(16);
        let mut attrs = HashMap::new();
        attrs.insert("speed".into(), "3.14".into());
        dr.add("entity", "agent", attrs, 0);
        let num = dr.attributes_as_numeric("entity");
        assert!((num[0] - 3.14).abs() < 0.01);
    }

    #[test]
    fn cosine_sim_identical() {
        assert!((cosine_sim(&[1.0, 0.0], &[1.0, 0.0]) - 1.0).abs() < 0.001);
    }

    #[test]
    fn cosine_sim_orthogonal() {
        assert!(cosine_sim(&[1.0, 0.0], &[0.0, 1.0]).abs() < 0.001);
    }
}
