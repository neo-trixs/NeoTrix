//! StreamPipeline — block-causal VSA attention for real-time streaming interaction.
//!
//! Inspired by Wan Streamer (2026): single-Transformer real-time audio-visual interaction
//! with 200ms latency, 25fps. Core innovations absorbed:
//!
//! 1. Block-causal attention: Each block attends to itself + all prior blocks (no token-level causal mask).
//! 2. Thinker-performer separation: Thinker (slow, deep) updates latent state; Performer (fast, shallow)
//!    produces output from latent state. Thinker runs at lower frequency.
//! 3. VSA block encoding: Each temporal block of perception is encoded into a VSA vector via bundling.
//!
//! Architecture:
//! ```text
//! Input frames → BlockEncoder (VSA bundling per window) → VSA Block Buffer
//! ┌─ ThinkerChannel (every N blocks): block-causal attention over buffer → latent update
//! └─ PerformerChannel (every block): read latent → produce output VSA
//! ```

use std::collections::VecDeque;

const VSA_DIM: usize = 512;

/// A single block of streamed perception, encoded as VSA
#[derive(Debug, Clone)]
pub struct StreamBlock {
    pub block_id: u64,
    pub vsa_vector: Vec<u8>,
    pub modality: StreamModality,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamModality {
    Visual,
    Audio,
    Text,
    Mixed,
}

/// Thinker-performer dual-channel state
#[derive(Debug, Clone)]
pub struct ThinkerPerformerState {
    pub latent: Vec<f64>,
    pub last_thinker_update: u64,
    pub performer_output: Vec<u8>,
}

/// Block-causal attention weights
#[derive(Debug, Clone)]
pub struct BlockAttentionWeights {
    pub self_weight: f64,
    pub prev_weights: Vec<f64>,
    pub temperature: f64,
}

impl Default for BlockAttentionWeights {
    fn default() -> Self {
        BlockAttentionWeights {
            self_weight: 0.4,
            prev_weights: Vec::new(),
            temperature: 1.0,
        }
    }
}

/// StreamPipeline — block-causal VSA attention pipeline
#[derive(Debug, Clone)]
pub struct StreamPipeline {
    pub block_buffer: VecDeque<StreamBlock>,
    pub max_blocks: usize,
    pub block_window: usize,
    pub thinker_interval: u64,
    pub performer_interval: u64,
    pub state: ThinkerPerformerState,
    pub attention: BlockAttentionWeights,
    pub block_counter: u64,
}

impl StreamPipeline {
    pub fn new(max_blocks: usize, block_window: usize) -> Self {
        StreamPipeline {
            block_buffer: VecDeque::with_capacity(max_blocks),
            max_blocks,
            block_window,
            thinker_interval: 5,
            performer_interval: 1,
            state: ThinkerPerformerState {
                latent: vec![0.0; 16],
                last_thinker_update: 0,
                performer_output: vec![0; VSA_DIM],
            },
            attention: BlockAttentionWeights::default(),
            block_counter: 0,
        }
    }

    pub fn with_thinker_interval(mut self, interval: u64) -> Self {
        self.thinker_interval = interval;
        self
    }

    pub fn with_performer_interval(mut self, interval: u64) -> Self {
        self.performer_interval = interval;
        self
    }

    /// Encode raw bytes into a VSA block and push to buffer
    pub fn push_block(&mut self, data: &[u8], modality: StreamModality, timestamp_ms: u64) {
        let seed: u64 = data.iter().fold(self.block_counter, |acc, &b| {
            acc.wrapping_mul(31).wrapping_add(b as u64)
        });
        let vsa = self.vsa_encode(seed);
        self.block_counter += 1;
        let block = StreamBlock {
            block_id: self.block_counter,
            vsa_vector: vsa,
            modality,
            timestamp_ms,
        };
        if self.block_buffer.len() >= self.max_blocks {
            self.block_buffer.pop_front();
        }
        self.block_buffer.push_back(block);
    }

    /// VSA encoding from a seed value
    fn vsa_encode(&self, seed: u64) -> Vec<u8> {
        // Simple LCG-based deterministic VSA generation (no external QuantizedVSA dependency)
        let mut state = seed;
        let mut vec = Vec::with_capacity(VSA_DIM);
        for _ in 0..VSA_DIM {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            vec.push(((state >> 32) & 0xFF) as u8);
        }
        vec
    }

    /// Block-causal attention: each block attends to itself + all prior blocks
    pub fn block_causal_attention(&self, target_block: &StreamBlock) -> Vec<f64> {
        let n = self.block_buffer.len();
        if n == 0 {
            return vec![0.0; 16];
        }

        let target_vsa_f64: Vec<f64> = target_block.vsa_vector.iter().map(|&b| b as f64).collect();
        let mut context = vec![0.0f64; 16];
        let mut total_weight = 0.0;

        // Self-attention
        let self_sim = self.vsa_similarity_f64(&target_vsa_f64, &target_vsa_f64);
        let self_w = self.attention.self_weight * self_sim;
        for i in 0..16.min(target_vsa_f64.len()) {
            context[i] += self_w * target_vsa_f64[i];
        }
        total_weight += self_w;

        // Attend to all prior blocks
        let mut prev_weights: Vec<f64> = Vec::new();
        for block in self.block_buffer.iter() {
            if block.block_id >= target_block.block_id {
                continue;
            }
            let b_vsa_f64: Vec<f64> = block.vsa_vector.iter().map(|&b| b as f64).collect();
            let sim = self.vsa_similarity_f64(&target_vsa_f64, &b_vsa_f64);
            let w = sim * (1.0 - self.attention.self_weight) / n.max(1) as f64;
            for i in 0..16.min(b_vsa_f64.len()) {
                context[i] += w * b_vsa_f64[i];
            }
            total_weight += w;
            prev_weights.push(w);
        }

        // Normalize
        if total_weight > 0.0 {
            for c in context.iter_mut() {
                *c /= total_weight;
            }
        }

        context
    }

    /// Thinker channel: update latent state from block-causal attention (runs every thinker_interval blocks)
    pub fn thinker_step(&mut self) {
        if self.block_buffer.is_empty() {
            return;
        }
        if let Some(latest) = self.block_buffer.back() {
            let context = self.block_causal_attention(latest);
            // Simple exponential moving average update of latent state
            let alpha = 0.3;
            for i in 0..context.len().min(self.state.latent.len()) {
                self.state.latent[i] = self.state.latent[i] * (1.0 - alpha) + context[i] * alpha;
            }
            self.state.last_thinker_update = self.block_counter;
        }
    }

    /// Performer channel: read latent state and produce output VSA (runs every performer_interval blocks)
    pub fn performer_step(&mut self) -> Vec<u8> {
        // Project latent state to VSA space via simple linear transform
        let mut output = vec![0u8; VSA_DIM];
        for i in 0..VSA_DIM {
            let idx = i % self.state.latent.len();
            let val = (self.state.latent[idx] * 255.0).round() as i32;
            output[i] = val.clamp(0, 255) as u8;
        }
        self.state.performer_output = output.clone();
        output
    }

    /// Process a new perception block through the full pipeline.
    /// Returns (performer_output, thinker_ran_this_step)
    pub fn process_block(
        &mut self,
        data: &[u8],
        modality: StreamModality,
        timestamp_ms: u64,
    ) -> (Vec<u8>, bool) {
        self.push_block(data, modality, timestamp_ms);

        let mut thinker_ran = false;
        if self.block_counter % self.thinker_interval == 0 {
            self.thinker_step();
            thinker_ran = true;
        }

        let output = self.performer_step();
        (output, thinker_ran)
    }

    /// VSA cosine similarity between two f64 vectors
    fn vsa_similarity_f64(&self, a: &[f64], b: &[f64]) -> f64 {
        let min_len = a.len().min(b.len());
        if min_len == 0 {
            return 0.0;
        }
        let dot: f64 = a[..min_len]
            .iter()
            .zip(b[..min_len].iter())
            .map(|(x, y)| x * y)
            .sum();
        let norm_a: f64 = a[..min_len].iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b[..min_len].iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm_a > 0.0 && norm_b > 0.0 {
            (dot / (norm_a * norm_b) + 1.0) / 2.0 // normalize to [0, 1]
        } else {
            0.0
        }
    }

    /// Current buffer stats
    pub fn stats(&self) -> (usize, u64, f64) {
        (
            self.block_buffer.len(),
            self.block_counter,
            self.state.latent.iter().map(|x| x.abs()).sum::<f64>()
                / self.state.latent.len().max(1) as f64,
        )
    }

    /// Latent coherence: measure of how consistent the latent state is
    pub fn latent_coherence(&self) -> f64 {
        if self.state.latent.is_empty() {
            return 0.0;
        }
        let mean = self.state.latent.iter().sum::<f64>() / self.state.latent.len() as f64;
        let var = self
            .state
            .latent
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / self.state.latent.len() as f64;
        if var > 0.0 {
            (1.0 + var.recip()).min(1.0)
        } else {
            1.0
        }
    }
}

impl Default for StreamPipeline {
    fn default() -> Self {
        Self::new(64, 8)
    }
}
