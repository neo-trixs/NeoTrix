use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorRecord {
    pub id: String,
    pub vector: Vec<u8>,
    pub metadata: HashMap<String, String>,
    pub timestamp: u64,
}

impl VectorRecord {
    pub fn new(id: String, vector: Vec<u8>) -> Self {
        Self {
            id,
            vector,
            metadata: HashMap::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DistanceMetric {
    Hamming,
    Cosine,
    Euclidean,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub distance: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    pub max_vectors: usize,
    pub distance_metric: DistanceMetric,
    pub num_partitions: usize,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            max_vectors: 100_000,
            distance_metric: DistanceMetric::Hamming,
            num_partitions: 16,
        }
    }
}
