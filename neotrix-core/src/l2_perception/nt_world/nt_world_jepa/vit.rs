use serde::{Deserialize, Serialize};

/// Simplified ViT encoder for E8 state sequences.
///
/// Processes a sequence of "patches" (E8 hexagram state embeddings)
/// through transformer encoder layers: patch projection → positional
/// encoding → [LayerNorm + Multi-Head Attention + MLP] × L → output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JepaViTEncoder {
    pub num_patches: usize,
    pub patch_dim: usize,
    pub embed_dim: usize,
    pub num_heads: usize,
    pub num_layers: usize,
    pub mlp_dim: usize,
    /// Linear projection: embed_dim × patch_dim
    pub patch_proj: Vec<Vec<f64>>,
    pub patch_bias: Vec<f64>,
    /// Learned positional embeddings: num_patches × embed_dim
    pub pos_embed: Vec<Vec<f64>>,
    /// CLS token: embed_dim
    pub cls_token: Vec<f64>,
    /// Transformer encoder layers
    pub layers: Vec<TransformerLayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformerLayer {
    pub ln1_gamma: Vec<f64>,
    pub ln1_beta: Vec<f64>,
    pub q_proj: Vec<Vec<f64>>,
    pub k_proj: Vec<Vec<f64>>,
    pub v_proj: Vec<Vec<f64>>,
    pub out_proj: Vec<Vec<f64>>,
    pub ln2_gamma: Vec<f64>,
    pub ln2_beta: Vec<f64>,
    pub fc1: Vec<Vec<f64>>,
    pub fc1_bias: Vec<f64>,
    pub fc2: Vec<Vec<f64>>,
    pub fc2_bias: Vec<f64>,
}

impl TransformerLayer {
    fn new(embed_dim: usize, mlp_dim: usize, rng: &mut SimpleRng) -> Self {
        let std_attn = (2.0 / embed_dim as f64).sqrt();
        let std_mlp = (2.0 / (embed_dim + mlp_dim) as f64).sqrt();

        Self {
            ln1_gamma: vec![1.0; embed_dim],
            ln1_beta: vec![0.0; embed_dim],
            q_proj: random_matrix(embed_dim, embed_dim, std_attn, rng),
            k_proj: random_matrix(embed_dim, embed_dim, std_attn, rng),
            v_proj: random_matrix(embed_dim, embed_dim, std_attn, rng),
            out_proj: random_matrix(embed_dim, embed_dim, std_attn, rng),
            ln2_gamma: vec![1.0; embed_dim],
            ln2_beta: vec![0.0; embed_dim],
            fc1: random_matrix(mlp_dim, embed_dim, std_mlp, rng),
            fc1_bias: vec![0.0; mlp_dim],
            fc2: random_matrix(embed_dim, mlp_dim, std_mlp, rng),
            fc2_bias: vec![0.0; embed_dim],
        }
    }
}

struct SimpleRng(u64);

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn uniform(&mut self) -> f64 {
        (self.next_u64() as f64) / (u64::MAX as f64)
    }
}

fn random_matrix(rows: usize, cols: usize, std: f64, rng: &mut SimpleRng) -> Vec<Vec<f64>> {
    (0..rows)
        .map(|_| (0..cols).map(|_| (rng.uniform() - 0.5) * 2.0 * std).collect())
        .collect()
}

fn layer_norm(x: &[f64], gamma: &[f64], beta: &[f64]) -> Vec<f64> {
    let n = x.len() as f64;
    let mean = x.iter().sum::<f64>() / n;
    let var = x.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let std = (var + 1e-6).sqrt();
    x.iter()
        .zip(gamma.iter())
        .zip(beta.iter())
        .map(|((&v, &g), &b)| g * (v - mean) / std + b)
        .collect()
}

fn linear_transform(input: &[f64], weight: &[Vec<f64>], bias: &[f64]) -> Vec<f64> {
    let out_dim = weight.len();
    (0..out_dim)
        .map(|i| {
            let mut sum = bias[i];
            for (j, &v) in input.iter().enumerate().take(weight[i].len()) {
                sum += weight[i][j] * v;
            }
            sum
        })
        .collect()
}

fn scaled_dot_product_attention(
    q: &[Vec<f64>],
    k: &[Vec<f64>],
    v: &[Vec<f64>],
    scale: f64,
) -> Vec<Vec<f64>> {
    let seq = q.len();
    let dim = q[0].len();
    let mut attn = vec![vec![0.0f64; seq]; seq];
    for i in 0..seq {
        for j in 0..seq {
            let dot: f64 = (0..dim).map(|d| q[i][d] * k[j][d]).sum();
            attn[i][j] = dot * scale;
        }
    }
    for i in 0..seq {
        let max_val = attn[i]
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let mut sum = 0.0;
        for j in 0..seq {
            attn[i][j] = (attn[i][j] - max_val).exp();
            sum += attn[i][j];
        }
        if sum > 1e-12 {
            for j in 0..seq {
                attn[i][j] /= sum;
            }
        }
    }
    let mut out = vec![vec![0.0f64; dim]; seq];
    for i in 0..seq {
        for d in 0..dim {
            let mut s = 0.0;
            for j in 0..seq {
                s += attn[i][j] * v[j][d];
            }
            out[i][d] = s;
        }
    }
    out
}

fn multi_head_attention(
    x: &[Vec<f64>],
    q_w: &[Vec<f64>],
    k_w: &[Vec<f64>],
    v_w: &[Vec<f64>],
    out_w: &[Vec<f64>],
    num_heads: usize,
) -> Vec<Vec<f64>> {
    let seq = x.len();
    let dim = x[0].len();
    let head_dim = dim / num_heads;
    if head_dim == 0 {
        return x.to_vec();
    }

    let q: Vec<Vec<f64>> = x.iter().map(|xi| linear_transform(xi, q_w, &vec![0.0; dim])).collect();
    let k: Vec<Vec<f64>> = x.iter().map(|xi| linear_transform(xi, k_w, &vec![0.0; dim])).collect();
    let v: Vec<Vec<f64>> = x.iter().map(|xi| linear_transform(xi, v_w, &vec![0.0; dim])).collect();

    let scale = (head_dim as f64).sqrt().recip();
    let mut head_outputs = Vec::with_capacity(num_heads);
    for h in 0..num_heads {
        let start = h * head_dim;
        let end = start + head_dim;
        let q_h: Vec<Vec<f64>> = q.iter().map(|qi| qi[start..end].to_vec()).collect();
        let k_h: Vec<Vec<f64>> = k.iter().map(|ki| ki[start..end].to_vec()).collect();
        let v_h: Vec<Vec<f64>> = v.iter().map(|vi| vi[start..end].to_vec()).collect();
        head_outputs.push(scaled_dot_product_attention(&q_h, &k_h, &v_h, scale));
    }

    let mut concat = vec![vec![0.0f64; dim]; seq];
    for i in 0..seq {
        for h in 0..num_heads {
            let start = h * head_dim;
            for d in 0..head_dim {
                concat[i][start + d] = head_outputs[h][i][d];
            }
        }
    }

    concat
        .iter()
        .map(|ci| linear_transform(ci, out_w, &vec![0.0; dim]))
        .collect()
}

impl JepaViTEncoder {
    pub fn new(num_patches: usize, patch_dim: usize, embed_dim: usize, num_heads: usize, num_layers: usize, mlp_dim: usize, seed: u64) -> Self {
        let mut rng = SimpleRng::new(seed);
        let std_proj = (2.0 / (patch_dim + embed_dim) as f64).sqrt();
        let patch_proj = random_matrix(embed_dim, patch_dim, std_proj, &mut rng);
        let patch_bias = vec![0.0; embed_dim];
        let pos_embed: Vec<Vec<f64>> = (0..num_patches)
            .map(|_| {
                (0..embed_dim)
                    .map(|_| (rng.uniform() - 0.5) * 0.1)
                    .collect()
            })
            .collect();
        let cls_token = (0..embed_dim).map(|_| (rng.uniform() - 0.5) * 0.1).collect();
        let layers = (0..num_layers)
            .map(|_| TransformerLayer::new(embed_dim, mlp_dim, &mut rng))
            .collect();

        Self {
            num_patches,
            patch_dim,
            embed_dim,
            num_heads,
            num_layers,
            mlp_dim,
            patch_proj,
            patch_bias,
            pos_embed,
            cls_token,
            layers,
        }
    }

    /// Encode a sequence of patches into latent representations.
    ///
    /// `patches`: `num_patches` vectors each of size `patch_dim`.
    /// Returns `(cls_token_embedding, patch_embeddings)` where each is embed_dim-sized.
    pub fn encode(&self, patches: &[Vec<f64>]) -> (Vec<f64>, Vec<Vec<f64>>) {
        let n = patches.len().min(self.num_patches);
        let mut token_seq: Vec<Vec<f64>> = Vec::with_capacity(n + 1);

        let cls = self.cls_token.clone();
        token_seq.push(cls);

        for i in 0..n {
            let mut tok = linear_transform(&patches[i], &self.patch_proj, &self.patch_bias);
            for d in 0..self.embed_dim {
                tok[d] += self.pos_embed[i][d];
            }
            token_seq.push(tok);
        }

        for _ in n..self.num_patches {
            token_seq.push(vec![0.0; self.embed_dim]);
        }

        for layer in &self.layers {
            let ln1_out: Vec<Vec<f64>> = token_seq.iter().map(|t| layer_norm(t, &layer.ln1_gamma, &layer.ln1_beta)).collect();
            let attn_out = multi_head_attention(&ln1_out, &layer.q_proj, &layer.k_proj, &layer.v_proj, &layer.out_proj, self.num_heads);
            for i in 0..token_seq.len() {
                for d in 0..self.embed_dim {
                    token_seq[i][d] += attn_out[i][d];
                }
            }

            let ln2_out: Vec<Vec<f64>> = token_seq.iter().map(|t| layer_norm(t, &layer.ln2_gamma, &layer.ln2_beta)).collect();
            let mlp_out: Vec<Vec<f64>> = ln2_out.iter().map(|t| {
                let h = linear_transform(t, &layer.fc1, &layer.fc1_bias);
                let activated: Vec<f64> = h.into_iter().map(|v| v.tanh()).collect();
                linear_transform(&activated, &layer.fc2, &layer.fc2_bias)
            }).collect();
            for i in 0..token_seq.len() {
                for d in 0..self.embed_dim {
                    token_seq[i][d] += mlp_out[i][d];
                }
            }
        }

        let cls_out = token_seq[0].clone();
        let patch_out: Vec<Vec<f64>> = token_seq[1..=n.min(self.num_patches)].to_vec();

        (cls_out, patch_out)
    }

    /// Encode and return the CLS token (pooled representation).
    pub fn encode_cls(&self, patches: &[Vec<f64>]) -> Vec<f64> {
        self.encode(patches).0
    }

    /// Random projection init for smaller JEPA variants.
    pub fn new_mlp(patch_dim: usize, embed_dim: usize, seed: u64) -> Self {
        Self::new(1, patch_dim, embed_dim, 4, 2, embed_dim * 4, seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vit_encoder_output_dim() {
        let vit = JepaViTEncoder::new(8, 16, 32, 4, 2, 64, 42);
        let patches: Vec<Vec<f64>> = (0..8).map(|_| (0..16).map(|i| i as f64 / 16.0).collect()).collect();
        let (cls, patch_out) = vit.encode(&patches);
        assert_eq!(cls.len(), 32);
        assert_eq!(patch_out.len(), 8);
        for p in &patch_out {
            assert_eq!(p.len(), 32);
        }
    }

    #[test]
    fn test_vit_handles_fewer_patches() {
        let vit = JepaViTEncoder::new(8, 16, 32, 4, 2, 64, 42);
        let patches: Vec<Vec<f64>> = (0..3).map(|_| (0..16).map(|i| i as f64 / 16.0).collect()).collect();
        let (cls, patch_out) = vit.encode(&patches);
        assert_eq!(cls.len(), 32);
        assert_eq!(patch_out.len(), 3);
    }

    #[test]
    fn test_layer_norm_basic() {
        let x = vec![1.0, 2.0, 3.0, 4.0];
        let gamma = vec![1.0; 4];
        let beta = vec![0.0; 4];
        let y = layer_norm(&x, &gamma, &beta);
        let mean = y.iter().sum::<f64>() / 4.0;
        assert!((mean).abs() < 1e-6);
        let var = y.iter().map(|v| v.powi(2)).sum::<f64>() / 4.0;
        assert!((var - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_multi_head_attention_output_dim() {
        let dim = 8;
        let seq = 4;
        let std = (2.0 / dim as f64).sqrt();
        let mut rng = SimpleRng::new(42);
        let x: Vec<Vec<f64>> = (0..seq).map(|_| (0..dim).map(|i| i as f64 / dim as f64).collect()).collect();
        let q_w = random_matrix(dim, dim, std, &mut rng);
        let k_w = random_matrix(dim, dim, std, &mut rng);
        let v_w = random_matrix(dim, dim, std, &mut rng);
        let out_w = random_matrix(dim, dim, std, &mut rng);
        let out = multi_head_attention(&x, &q_w, &k_w, &v_w, &out_w, 2);
        assert_eq!(out.len(), seq);
        for o in &out {
            assert_eq!(o.len(), dim);
        }
    }
}
