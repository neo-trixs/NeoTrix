use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct BinaryHDVector {
    pub bits: Vec<u64>,
    pub dims: usize,
}

fn random_binary(dims: usize) -> BinaryHDVector {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let words = (dims + 63) / 64;
    let mut bits = Vec::with_capacity(words);
    let mut state = seed;
    for _ in 0..words {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        bits.push(state);
    }
    BinaryHDVector { bits, dims }
}

impl BinaryHDVector {
    pub fn new(dims: usize) -> Self {
        random_binary(dims)
    }
    pub fn from_float(v: &[f64], threshold: f64) -> Self {
        let dims = v.len();
        let words = (dims + 63) / 64;
        let mut bits = vec![0u64; words];
        for (i, &val) in v.iter().enumerate() {
            if val > threshold {
                bits[i / 64] |= 1u64 << (i % 64);
            }
        }
        BinaryHDVector { bits, dims }
    }
    pub fn xor(&self, other: &Self) -> Self {
        let words = self.bits.len().min(other.bits.len());
        let mut bits = Vec::with_capacity(words);
        for i in 0..words {
            bits.push(self.bits[i] ^ other.bits[i]);
        }
        BinaryHDVector {
            bits,
            dims: self.dims,
        }
    }
    pub fn popcount(&self) -> f64 {
        self.bits.iter().map(|w| w.count_ones() as f64).sum::<f64>() / self.dims as f64
    }
    pub fn similarity(&self, other: &Self) -> f64 {
        1.0 - (2.0 * (self.xor(other).popcount() - 0.5).abs())
    }
}

#[derive(Debug, Clone)]
pub struct BinaryAttentionHead {
    pub query: BinaryHDVector,
    pub key: BinaryHDVector,
}

impl BinaryAttentionHead {
    pub fn new(dims: usize) -> Self {
        Self {
            query: BinaryHDVector::new(dims),
            key: BinaryHDVector::new(dims),
        }
    }
    pub fn forward(&self, x: &BinaryHDVector) -> f64 {
        self.query.similarity(x) * self.key.similarity(x)
    }
}

#[derive(Debug, Clone)]
pub struct LARSVSAttention {
    pub heads: Vec<BinaryAttentionHead>,
    pub dims: usize,
}

impl LARSVSAttention {
    pub fn new(dims: usize, num_heads: usize) -> Self {
        Self {
            heads: (0..num_heads)
                .map(|_| BinaryAttentionHead::new(dims))
                .collect(),
            dims,
        }
    }
    pub fn attend(&self, x: &BinaryHDVector, values: &[f64]) -> f64 {
        let mut ws = 0.0;
        let mut tw = 0.0;
        for (i, head) in self.heads.iter().enumerate() {
            let w = head.forward(x);
            ws += w * values.get(i).copied().unwrap_or(0.0);
            tw += w;
        }
        if tw > 0.0 {
            ws / tw
        } else {
            0.0
        }
    }
}
