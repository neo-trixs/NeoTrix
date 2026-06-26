use super::chunk::{sample_categorical, Chunk, ScoreWeights};
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

pub fn uptree_competition<'a>(
    chunks: &'a [Chunk],
    weights: &ScoreWeights,
    temperature: f64,
) -> Option<&'a Chunk> {
    if chunks.is_empty() {
        return None;
    }
    if chunks.len() == 1 {
        return Some(&chunks[0]);
    }

    let raw: Vec<f64> = chunks.iter().map(|c| c.weight(weights)).collect();
    let max_val = raw.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let shifted: Vec<f64> = raw.iter().map(|w| (w - max_val) / temperature).collect();
    let exp_vals: Vec<f64> = shifted.iter().map(|w| w.exp()).collect();
    let sum: f64 = exp_vals.iter().sum();

    if sum == 0.0 || !sum.is_finite() {
        return Some(&chunks[0]);
    }

    let probs: Vec<f64> = exp_vals.iter().map(|e| e / sum).collect();
    Some(sample_categorical(chunks, &probs))
}

pub fn compute_surprise_from_prediction(predicted: &[u8], actual: &[u8]) -> f64 {
    1.0 - QuantizedVSA::similarity(predicted, actual)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uptree_single_chunk() {
        let c = Chunk::new("only", 0, vec![0u8; 16]);
        let chunks = vec![c];
        let winner = uptree_competition(&chunks, &ScoreWeights::default(), 0.1);
        assert_eq!(winner.unwrap().processor_name, "only");
    }

    #[test]
    fn test_uptree_selects_highest_weight() {
        let mut c1 = Chunk::new("low", 0, vec![0u8; 16]);
        c1.relevance = 0.1;
        let mut c2 = Chunk::new("high", 1, vec![1u8; 16]);
        c2.relevance = 0.9;
        let chunks = vec![c1, c2];
        let winner = uptree_competition(&chunks, &ScoreWeights::default(), 0.01);
        assert_eq!(winner.unwrap().processor_name, "high");
    }

    #[test]
    fn test_surprise_identical() {
        let v = vec![0u8; 32];
        let s = compute_surprise_from_prediction(&v, &v);
        assert!((s - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_surprise_different() {
        let a = vec![0u8; 32];
        let b = vec![1u8; 32];
        let s = compute_surprise_from_prediction(&a, &b);
        assert!(s > 0.0);
    }
}
