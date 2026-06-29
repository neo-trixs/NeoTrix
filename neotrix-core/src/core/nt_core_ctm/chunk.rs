use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreWeights {
    pub relevance: f64,
    pub confidence: f64,
    pub surprise: f64,
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            relevance: 1.0,
            confidence: 1.0,
            surprise: 0.2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub processor_name: String,
    pub time_step: usize,
    pub gist: Vec<u8>,
    pub additional_questions: Vec<String>,
    pub relevance: f64,
    pub confidence: f64,
    pub surprise: f64,
}

impl Chunk {
    pub fn new(processor_name: &str, time_step: usize, gist: Vec<u8>) -> Self {
        Self {
            processor_name: processor_name.to_string(),
            time_step,
            gist,
            additional_questions: Vec::new(),
            relevance: 0.0,
            confidence: 0.0,
            surprise: 0.0,
        }
    }

    pub fn weight(&self, weights: &ScoreWeights) -> f64 {
        self.relevance * weights.relevance
            + self.confidence * weights.confidence
            + self.surprise * weights.surprise
    }

    pub fn apply_external(&mut self, external: &ExternalScores, _name: &str) {
        if let Some(r) = external.relevance {
            self.relevance = r;
        }
        if let Some(c) = external.confidence {
            self.confidence = c;
        }
        if let Some(s) = external.surprise {
            self.surprise = s;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ExternalScores {
    pub relevance: Option<f64>,
    pub confidence: Option<f64>,
    pub surprise: Option<f64>,
}

impl ExternalScores {
    pub fn none() -> Self {
        Self {
            relevance: None,
            confidence: None,
            surprise: None,
        }
    }
}

pub fn sample_categorical<'a>(chunks: &'a [Chunk], probs: &[f64]) -> &'a Chunk {
    let mut rng = rand::thread_rng();
    let draw: f64 = rng.gen();
    let mut cum = 0.0;
    for (chunk, p) in chunks.iter().zip(probs.iter()) {
        cum += p;
        if draw <= cum {
            return chunk;
        }
    }
    chunks.last().unwrap_or_else(|| &chunks[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_new() {
        let c = Chunk::new("test", 0, vec![1, 2, 3]);
        assert_eq!(c.processor_name, "test");
        assert_eq!(c.time_step, 0);
        assert_eq!(c.gist, vec![1u8, 2, 3]);
    }

    #[test]
    fn test_chunk_weight_defaults() {
        let c = Chunk::new("p", 1, vec![0u8; 16]);
        let w = ScoreWeights::default();
        assert!((c.weight(&w) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_weight_with_values() {
        let mut c = Chunk::new("p", 1, vec![0u8; 16]);
        c.relevance = 0.8;
        c.confidence = 0.7;
        c.surprise = 0.5;
        assert!((c.weight(&ScoreWeights::default()) - 1.6).abs() < 1e-6);
    }

    #[test]
    fn test_score_weights_defaults() {
        let w = ScoreWeights::default();
        assert!((w.relevance - 1.0).abs() < 1e-6);
        assert!((w.confidence - 1.0).abs() < 1e-6);
        assert!((w.surprise - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_sample_categorical_deterministic() {
        let c1 = Chunk::new("a", 0, vec![0u8; 16]);
        let c2 = Chunk::new("b", 1, vec![1u8; 16]);
        let chunks = vec![c1, c2];
        let probs = vec![1.0, 0.0];
        let picked = sample_categorical(&chunks, &probs);
        assert_eq!(picked.processor_name, "a");
    }
}
