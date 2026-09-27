//! NT-GEN-MODEL — 生成式因果 LM 端内推理（MiniMind-3 / Qwen3 架构映射）
//!
//! P3 落地：Dense decoder（RMSNorm + GQA + NeoX-RoPE + SwiGLU + 可选
//! QK-Norm），safetensors 权重 + tokenizers 分词，温度/top-k/top-p/
//! 重复惩罚采样（MiniMind Pattern 8 采样映射），CPU（Metal 后续）。
//! 无真实权重时全部单测走合成权重形状验证；
//! `models/minimind-3/` 落盘即激活 live 测试。

use candle_core::{DType, Device, Tensor};
use candle_nn::{Module, VarBuilder};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// 生成模型配置（默认 = MiniMind-3 Dense 档）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NtGenConfig {
    pub vocab_size: usize,
    pub hidden_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub num_key_value_heads: usize,
    pub intermediate_size: usize,
    pub rms_norm_eps: f64,
    pub rope_theta: f32,
    pub max_position_embeddings: usize,
    pub tie_word_embeddings: bool,
    pub bos_token_id: u32,
    pub eos_token_id: u32,
}

impl Default for NtGenConfig {
    fn default() -> Self {
        Self {
            vocab_size: 6400,
            hidden_size: 768,
            num_hidden_layers: 8,
            num_attention_heads: 8,
            num_key_value_heads: 4,
            intermediate_size: 2048,
            rms_norm_eps: 1e-6,
            rope_theta: 1e6,
            max_position_embeddings: 32768,
            tie_word_embeddings: true,
            bos_token_id: 1,
            eos_token_id: 2,
        }
    }
}

/// RoPE cos/sin 表：[1, 1, seq_len, head_dim]
fn build_rope_tables(
    seq_len: usize,
    head_dim: usize,
    theta: f32,
    device: &Device,
) -> Result<(Tensor, Tensor)> {
    let half = head_dim / 2;
    let inv_freq: Vec<f32> = (0..half)
        .map(|i| theta.powf(-((2 * i) as f32) / head_dim as f32))
        .collect();
    let mut freqs = Vec::with_capacity(seq_len * half);
    for t in 0..seq_len {
        for f in &inv_freq {
            freqs.push(t as f32 * *f);
        }
    }
    let freqs = Tensor::from_vec(freqs, (seq_len, half), device)
        .map_err(|e| Error::InferenceError(e.to_string()))?;
    // NeoX 式：emb = cat(freqs, freqs)
    let emb = Tensor::cat(&[&freqs, &freqs], 1)
        .map_err(|e| Error::InferenceError(e.to_string()))?;
    let cos = emb
        .cos()
        .map_err(|e| Error::InferenceError(e.to_string()))?;
    let sin = emb
        .sin()
        .map_err(|e| Error::InferenceError(e.to_string()))?;
    // [1, 1, T, D] 便于广播到 [B, H, T, D]
    let cos = cos
        .unsqueeze(0)
        .and_then(|t| t.unsqueeze(0))
        .map_err(|e| Error::InferenceError(e.to_string()))?;
    let sin = sin
        .unsqueeze(0)
        .and_then(|t| t.unsqueeze(0))
        .map_err(|e| Error::InferenceError(e.to_string()))?;
    Ok((cos, sin))
}

/// NeoX 式 RoPE 应用
fn apply_rope(q: &Tensor, k: &Tensor, cos: &Tensor, sin: &Tensor) -> Result<(Tensor, Tensor)> {
    let rotate = |x: &Tensor| -> Result<Tensor> {
        let d = x.dim(3).map_err(|e| Error::InferenceError(e.to_string()))?;
        let x1 = x
            .narrow(3, 0, d / 2)
            .map_err(|e| Error::InferenceError(e.to_string()))?;
        let x2 = x
            .narrow(3, d / 2, d / 2)
            .map_err(|e| Error::InferenceError(e.to_string()))?;
        let neg_x2 = x2
            .neg()
            .map_err(|e| Error::InferenceError(e.to_string()))?;
        Tensor::cat(&[&neg_x2, &x1], 3)
            .map_err(|e| Error::InferenceError(e.to_string()))
    };
    let map2 = |a: &Tensor, b: &Tensor| -> Result<Tensor> {
        a.broadcast_mul(b)
            .map_err(|e| Error::InferenceError(e.to_string()))
    };
    let q_embed = map2(q, cos)?
        .broadcast_add(&map2(&rotate(q)?, sin)?)
        .map_err(|e| Error::InferenceError(e.to_string()))?;
    let k_embed = map2(k, cos)?
        .broadcast_add(&map2(&rotate(k)?, sin)?)
        .map_err(|e| Error::InferenceError(e.to_string()))?;
    Ok((q_embed, k_embed))
}

fn err(e: candle_core::Error) -> Error {
    Error::InferenceError(e.to_string())
}

/// bias 可选的 Linear（MiniMind/Qwen 系权重常无 bias，有则用之）
fn linear_opt_bias(
    in_dim: usize,
    out_dim: usize,
    vb: VarBuilder,
) -> candle_core::Result<candle_nn::Linear> {
    let w = vb.get((out_dim, in_dim), "weight")?;
    let b = vb.get(out_dim, "bias").ok();
    Ok(candle_nn::Linear::new(w, b))
}

struct Mlp {
    gate: candle_nn::Linear,
    up: candle_nn::Linear,
    down: candle_nn::Linear,
}

impl Mlp {
    fn load(vb: VarBuilder, hidden: usize, intermediate: usize) -> Result<Self> {
        let gate = linear_opt_bias(hidden, intermediate, vb.pp("gate_proj")).map_err(err)?;
        let up = linear_opt_bias(hidden, intermediate, vb.pp("up_proj")).map_err(err)?;
        let down = linear_opt_bias(intermediate, hidden, vb.pp("down_proj")).map_err(err)?;
        Ok(Self { gate, up, down })
    }

    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let g = self.gate.forward(xs).map_err(err)?;
        let u = self.up.forward(xs).map_err(err)?;
        let silu = candle_nn::ops::sigmoid(&g)
            .and_then(|s| g.broadcast_mul(&s))
            .map_err(err)?;
        let gated = silu.broadcast_mul(&u).map_err(err)?;
        self.down.forward(&gated).map_err(err)
    }
}

struct Attention {
    q_proj: candle_nn::Linear,
    k_proj: candle_nn::Linear,
    v_proj: candle_nn::Linear,
    o_proj: candle_nn::Linear,
    q_norm: Option<candle_nn::RmsNorm>,
    k_norm: Option<candle_nn::RmsNorm>,
    n_head: usize,
    n_kv: usize,
    head_dim: usize,
}

impl Attention {
    fn load(vb: VarBuilder, cfg: &NtGenConfig) -> Result<Self> {
        let hidden = cfg.hidden_size;
        let n_head = cfg.num_attention_heads;
        let n_kv = cfg.num_key_value_heads;
        let head_dim = hidden / n_head;
        let q_proj = linear_opt_bias(hidden, n_head * head_dim, vb.pp("q_proj")).map_err(err)?;
        let k_proj = linear_opt_bias(hidden, n_kv * head_dim, vb.pp("k_proj")).map_err(err)?;
        let v_proj = linear_opt_bias(hidden, n_kv * head_dim, vb.pp("v_proj")).map_err(err)?;
        let o_proj = linear_opt_bias(n_head * head_dim, hidden, vb.pp("o_proj")).map_err(err)?;
        // QK-Norm（Qwen3 有；缺失则跳过，兼容无该权重的旧档）
        let q_norm = vb
            .pp("q_norm")
            .get(head_dim, "weight")
            .ok()
            .map(|w| candle_nn::RmsNorm::new(w, cfg.rms_norm_eps));
        let k_norm = vb
            .pp("k_norm")
            .get(head_dim, "weight")
            .ok()
            .map(|w| candle_nn::RmsNorm::new(w, cfg.rms_norm_eps));
        Ok(Self {
            q_proj,
            k_proj,
            v_proj,
            o_proj,
            q_norm,
            k_norm,
            n_head,
            n_kv,
            head_dim,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn forward(
        &self,
        xs: &Tensor,
        cos: &Tensor,
        sin: &Tensor,
        mask: Option<&Tensor>,
        k_cache: &mut Tensor,
        v_cache: &mut Tensor,
    ) -> Result<Tensor> {
        let (b, t, _) = xs.dims3().map_err(err)?;
        let q = self.q_proj.forward(xs).map_err(err)?;
        let k = self.k_proj.forward(xs).map_err(err)?;
        let v = self.v_proj.forward(xs).map_err(err)?;
        // Qwen3 式 head-wise QK-Norm（[B*H*T, D] 展平做 RMSNorm）
        let q = self.norm_heads(&q, self.n_head, &self.q_norm)?;
        let k = self.norm_heads(&k, self.n_kv, &self.k_norm)?;
        // reshape → [B, H, T, D]
        let q = q
            .reshape((b, t, self.n_head, self.head_dim))
            .map_err(err)?
            .transpose(1, 2)
            .map_err(err)?;
        let k4 = k
            .reshape((b, t, self.n_kv, self.head_dim))
            .map_err(err)?
            .transpose(1, 2)
            .map_err(err)?;
        let v4 = v
            .reshape((b, t, self.n_kv, self.head_dim))
            .map_err(err)?
            .transpose(1, 2)
            .map_err(err)?;
        let (q, k4) = apply_rope(&q, &k4, cos, sin)?;
        // 先拼 cache（都是 kv 头数，未重复），存回未重复版本
        let kc_ref: &Tensor = k_cache;
        let vc_ref: &Tensor = v_cache;
        let k_full = Tensor::cat(&[kc_ref, &k4], 2).map_err(err)?;
        let v_full = Tensor::cat(&[vc_ref, &v4], 2).map_err(err)?;
        *k_cache = k_full.clone();
        *v_cache = v_full.clone();
        // GQA：完整 kv 交错重复到 n_head 后再做 attention
        let rep = self.n_head / self.n_kv;
        let kv_len = k_full.dim(2).map_err(err)?;
        let k_rep = repeat_kv_heads(&k_full, b, self.n_kv, rep, kv_len, self.head_dim)?;
        let v_rep = repeat_kv_heads(&v_full, b, self.n_kv, rep, kv_len, self.head_dim)?;
        let scale = 1.0 / (self.head_dim as f64).sqrt();
        let mut scores = (q.matmul(&k_rep.t().map_err(err)?).map_err(err)? * scale)
            .map_err(err)?;
        if let Some(m) = mask {
            scores = scores.broadcast_add(m).map_err(err)?;
        }
        let probs = candle_nn::ops::softmax_last_dim(&scores).map_err(err)?;
        let out = probs.matmul(&v_rep).map_err(err)?;
        let out = out.transpose(1, 2).map_err(err)?.reshape((b, t, self.n_head * self.head_dim)).map_err(err)?;
        self.o_proj.forward(&out).map_err(err)
    }

    /// head-wise RMSNorm（无 norm 配置时原样返回）
    fn norm_heads(
        &self,
        x: &Tensor,
        n_heads: usize,
        norm: &Option<candle_nn::RmsNorm>,
    ) -> Result<Tensor> {
        match norm {
            Some(n) => {
                let (b, t, _) = x.dims3().map_err(err)?;
                let flat = x
                    .reshape((b * t * n_heads, self.head_dim))
                    .map_err(err)?;
                n.forward(&flat).map_err(err)?.reshape((b, t, n_heads * self.head_dim)).map_err(err)
            }
            None => Ok(x.clone()),
        }
    }
}

/// kv head 交错重复 [B, kvH, T, D] → [B, kvH*rep, T, D]
fn repeat_kv_heads(
    x: &Tensor,
    b: usize,
    kv_heads: usize,
    rep: usize,
    t: usize,
    head_dim: usize,
) -> Result<Tensor> {
    x.unsqueeze(2)
        .map_err(err)?
        .broadcast_as((b, kv_heads, rep, t, head_dim))
        .map_err(err)?
        .reshape((b, kv_heads * rep, t, head_dim))
        .map_err(err)
}

struct DecoderLayer {
    attn_norm: candle_nn::RmsNorm,
    attn: Attention,
    mlp_norm: candle_nn::RmsNorm,
    mlp: Mlp,
}

impl DecoderLayer {
    fn load(vb: VarBuilder, cfg: &NtGenConfig) -> Result<Self> {
        let attn_norm =
            candle_nn::rms_norm(cfg.hidden_size, cfg.rms_norm_eps, vb.pp("input_layernorm"))
                .map_err(err)?;
        let attn = Attention::load(vb.pp("self_attn"), cfg)?;
        let mlp_norm = candle_nn::rms_norm(
            cfg.hidden_size,
            cfg.rms_norm_eps,
            vb.pp("post_attention_layernorm"),
        )
        .map_err(err)?;
        let mlp = Mlp::load(vb.pp("mlp"), cfg.hidden_size, cfg.intermediate_size)?;
        Ok(Self {
            attn_norm,
            attn,
            mlp_norm,
            mlp,
        })
    }
}

/// 生成模型（Dense decoder + KV cache）
pub struct NtGenModel {
    embed: candle_nn::Embedding,
    /// tie 路径用：embedding 权重 [V, H]（lm_head 缺失时）
    embed_weight: Option<Tensor>,
    layers: Vec<DecoderLayer>,
    norm: candle_nn::RmsNorm,
    lm_head: Option<candle_nn::Linear>,
    cfg: NtGenConfig,
    device: Device,
}

impl NtGenModel {
    pub fn load(vb: VarBuilder, cfg: NtGenConfig, device: &Device) -> Result<Self> {
        let embed = candle_nn::embedding(cfg.vocab_size, cfg.hidden_size, vb.pp("model.embed_tokens"))
            .map_err(err)?;
        let mut layers = Vec::with_capacity(cfg.num_hidden_layers);
        for i in 0..cfg.num_hidden_layers {
            layers.push(DecoderLayer::load(
                vb.pp(format!("model.layers.{i}")),
                &cfg,
            )?);
        }
        let norm = candle_nn::rms_norm(cfg.hidden_size, cfg.rms_norm_eps, vb.pp("model.norm"))
            .map_err(err)?;
        // lm_head 缺失 + tie=true → 用 embedding 转置
        let lm_head = match vb.pp("lm_head").get(cfg.vocab_size, "weight") {
            Ok(_) => Some(
                linear_opt_bias(cfg.hidden_size, cfg.vocab_size, vb.pp("lm_head")).map_err(err)?,
            ),
            Err(_) => None,
        };
        let embed_weight = vb
            .pp("model.embed_tokens")
            .get((cfg.vocab_size, cfg.hidden_size), "weight")
            .ok();
        Ok(Self {
            embed,
            embed_weight,
            layers,
            norm,
            lm_head,
            cfg,
            device: device.clone(),
        })
    }

    /// 前向：tokens [T]，start_pos 为 cache 已有长度；返回 logits [1, T, V]
    pub fn forward(
        &self,
        tokens: &[u32],
        start_pos: usize,
        cache: &mut Vec<(Tensor, Tensor)>,
    ) -> Result<Tensor> {
        let t = tokens.len();
        let input = Tensor::new(tokens, &self.device).map_err(err)?.unsqueeze(0).map_err(err)?;
        let mut xs = self.embed.forward(&input).map_err(err)?;
        let (cos, sin) = build_rope_tables(
            start_pos + t,
            self.cfg.hidden_size / self.cfg.num_attention_heads,
            self.cfg.rope_theta,
            &self.device,
        )?;
        // 切出本段位置的 cos/sin
        let cos = cos.narrow(2, start_pos, t).map_err(err)?;
        let sin = sin.narrow(2, start_pos, t).map_err(err)?;
        // 因果 mask [1, 1, T, K]
        let kv_len = start_pos + t;
        let mask = causal_mask(t, kv_len, &self.device)?;
        for (i, layer) in self.layers.iter().enumerate() {
            let residual = xs.clone();
            let h = layer.attn_norm.forward(&xs).map_err(err)?;
            let (k_empty, v_empty) = (&cache[i].0, &cache[i].1);
            let mut kc = k_empty.clone();
            let mut vc = v_empty.clone();
            let a = layer.attn.forward(&h, &cos, &sin, Some(&mask), &mut kc, &mut vc)?;
            cache[i] = (kc, vc);
            xs = (residual + a).map_err(err)?;
            let residual = xs.clone();
            let h = layer.mlp_norm.forward(&xs).map_err(err)?;
            let m = layer.mlp.forward(&h)?;
            xs = (residual + m).map_err(err)?;
        }
        let xs = self.norm.forward(&xs).map_err(err)?;
        match &self.lm_head {
            Some(head) => head.forward(&xs).map_err(err),
            None => {
                // tie：logits = hidden · embed^T（matmul 要求同 rank，先展平）
                let w = self.embed_weight.as_ref().ok_or_else(|| {
                    Error::ModelLoadError("tie enabled but embed weight missing".to_string())
                })?;
                let (b, t, _) = xs.dims3().map_err(err)?;
                let flat = xs.reshape((b * t, self.cfg.hidden_size)).map_err(err)?;
                let logits = flat.matmul(&w.t().map_err(err)?).map_err(err)?;
                logits.reshape((b, t, self.cfg.vocab_size)).map_err(err)
            }
        }
    }

    /// 空 KV cache（每层一对 [1, H|kvH, 0, D]）
    pub fn empty_cache(&self) -> Result<Vec<(Tensor, Tensor)>> {
        let hd = self.cfg.hidden_size / self.cfg.num_attention_heads;
        let mut out = Vec::with_capacity(self.layers.len());
        for _ in &self.layers {
            let k = Tensor::zeros(
                (1, self.cfg.num_key_value_heads, 0, hd),
                DType::F32,
                &self.device,
            )
            .map_err(err)?;
            let v = Tensor::zeros(
                (1, self.cfg.num_key_value_heads, 0, hd),
                DType::F32,
                &self.device,
            )
            .map_err(err)?;
            out.push((k, v));
        }
        Ok(out)
    }

    pub fn config(&self) -> &NtGenConfig {
        &self.cfg
    }
}

/// 因果 mask [1, 1, T, K]：允许 j <= kv_len - T + i
fn causal_mask(t: usize, kv_len: usize, device: &Device) -> Result<Tensor> {
    let mut data = Vec::with_capacity(t * kv_len);
    for i in 0..t {
        for j in 0..kv_len {
            let allowed = j <= kv_len - t + i;
            data.push(if allowed { 0.0f32 } else { f32::NEG_INFINITY });
        }
    }
    Tensor::from_vec(data, (1, 1, t, kv_len), device).map_err(err)
}

/// 采样参数（MiniMind Pattern 8 映射）
#[derive(Debug, Clone)]
pub struct NtSampleParams {    pub temperature: f64,
    pub top_k: Option<usize>,
    pub top_p: Option<f64>,
    pub repetition_penalty: f32,
    pub seed: u64,
}

impl Default for NtSampleParams {
    fn default() -> Self {
        Self {
            temperature: 0.8,
            top_k: Some(40),
            top_p: Some(0.9),
            repetition_penalty: 1.1,
            seed: 0x243F_6A88_85A3_08D3,
        }
    }
}

/// 小确定性 RNG（xorshift64*，免 rand 依赖）
struct TinyRng(u64);

impl TinyRng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn next_f64(&mut self) -> f64 {
        const DIV: f64 = (u64::MAX as f64) + 1.0;
        self.next_u64() as f64 / DIV
    }
}

pub struct NtSampler {
    params: NtSampleParams,
    rng: TinyRng,
}

impl NtSampler {
    pub fn new(params: NtSampleParams) -> Self {
        let seed = params.seed;
        Self {
            params,
            rng: TinyRng(seed),
        }
    }

    /// 采样：greedy（temp<=0）或 温度/top-k/top-p/重复惩罚
    pub fn sample(&mut self, logits: &[f32], seen: &[u32]) -> u32 {
        if logits.is_empty() {
            return 0;
        }
        if self.params.temperature <= 0.0 {
            return argmax(logits);
        }
        let mut adjusted: Vec<f32> = logits.to_vec();
        if (self.params.repetition_penalty - 1.0).abs() > f32::EPSILON {
            for s in seen {
                if let Some(v) = adjusted.get_mut(*s as usize) {
                    if *v > 0.0 {
                        *v /= self.params.repetition_penalty;
                    } else {
                        *v *= self.params.repetition_penalty;
                    }
                }
            }
        }
        let temp = self.params.temperature.max(1e-6) as f32;
        let mut probs: Vec<f32> = adjusted.iter().map(|l| l / temp).collect();
        // top-k：只留前 k（并列截断不影响正确性：多留的进不了采样即弃）
        if let Some(k) = self.params.top_k {
            if k < probs.len() {
                let mut idx: Vec<usize> = (0..probs.len()).collect();
                idx.sort_by(|a, b| {
                    probs[*b]
                        .partial_cmp(&probs[*a])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                for (i, p) in probs.iter_mut().enumerate() {
                    if !idx[..k.min(idx.len())].contains(&i) {
                        *p = f32::NEG_INFINITY;
                    }
                }
            }
        }
        // softmax
        let max = probs
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);
        let mut sum = 0.0f32;
        for p in probs.iter_mut() {
            *p = (*p - max).exp();
            sum += *p;
        }
        if sum > 0.0 {
            for p in probs.iter_mut() {
                *p /= sum;
            }
        }
        // top-p
        let mut order: Vec<usize> = (0..probs.len()).collect();
        order.sort_by(|a, b| {
            probs[*b]
                .partial_cmp(&probs[*a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut allowed = vec![false; probs.len()];
        if let Some(top_p) = self.params.top_p {
            let mut cum = 0.0f32;
            for i in &order {
                allowed[*i] = true;
                cum += probs[*i];
                if cum >= top_p as f32 {
                    break;
                }
            }
        } else {
            for i in &order {
                allowed[*i] = true;
            }
        }
        // 按过滤后分布采样
        let total: f32 = order.iter().filter(|i| allowed[**i]).map(|i| probs[*i]).sum();
        let mut r = (self.rng.next_f64() as f32) * total;
        for i in &order {
            if !allowed[*i] {
                continue;
            }
            r -= probs[*i];
            if r <= 0.0 {
                return *i as u32;
            }
        }
        order.first().copied().unwrap_or(0) as u32
    }
}

fn argmax(logits: &[f32]) -> u32 {
    let mut best = 0usize;
    for (i, v) in logits.iter().enumerate() {
        if *v > logits[best] {
            best = i;
        }
    }
    best as u32
}

/// 自动设备：metal feature 开启且可用则 M5 GPU，否则 CPU。
/// 纯加法，不碰既有 CPU 路径；Metal 失败静默回落（server 场景不崩）。
///
/// 已知限制（2026-09-23 实测）：candle-metal 0.8 无 rms-norm 算子，
/// Qwen3 系每层 RMSNorm → 前向报 "no metal implementation for rms-norm"。
/// 在 candle 补齐前 Qwen3 模型仍走 CPU；auto_device 只解决设备获取，
/// 不保证算子覆盖（调用方以 generate 实测为准）。
pub fn auto_device() -> Device {
    #[cfg(feature = "metal")]
    {
        match Device::new_metal(0) {
            Ok(d) => return d,
            Err(_) => {}
        }
    }
    Device::Cpu
}

/// 生成引擎：分词 + 模型 + 采样
pub struct NtGenEngine {
    model: NtGenModel,
    tokenizer: tokenizers::Tokenizer,
    eos: u32,
}

impl NtGenEngine {
    /// 从目录加载（config.json + model.safetensors + tokenizer.json）
    pub fn load_dir(dir: &std::path::Path, device: &Device) -> Result<Self> {
        let cfg_data = std::fs::read(dir.join("config.json"))
            .map_err(|_| Error::ModelNotFound(format!("config.json missing in {}", dir.display())))?;
        // 先解析为 Value，再合并默认值（HF config 常缺 bos/eos_token_id）
        let raw: serde_json::Value = serde_json::from_slice(&cfg_data)
            .map_err(|e| Error::ModelLoadError(format!("bad config.json: {e}")))?;
        let mut cfg = NtGenConfig::default();
        if let Some(v) = raw.get("vocab_size").and_then(|v| v.as_u64()) {
            cfg.vocab_size = v as usize;
        }
        if let Some(v) = raw.get("hidden_size").and_then(|v| v.as_u64()) {
            cfg.hidden_size = v as usize;
        }
        if let Some(v) = raw.get("num_hidden_layers").and_then(|v| v.as_u64()) {
            cfg.num_hidden_layers = v as usize;
        }
        if let Some(v) = raw.get("num_attention_heads").and_then(|v| v.as_u64()) {
            cfg.num_attention_heads = v as usize;
        }
        if let Some(v) = raw.get("num_key_value_heads").and_then(|v| v.as_u64()) {
            cfg.num_key_value_heads = v as usize;
        }
        if let Some(v) = raw.get("intermediate_size").and_then(|v| v.as_u64()) {
            cfg.intermediate_size = v as usize;
        }
        if let Some(v) = raw.get("rms_norm_eps").and_then(|v| v.as_f64()) {
            cfg.rms_norm_eps = v;
        }
        if let Some(v) = raw.get("rope_theta").and_then(|v| v.as_f64()) {
            cfg.rope_theta = v as f32;
        }
        if let Some(v) = raw.get("max_position_embeddings").and_then(|v| v.as_u64()) {
            cfg.max_position_embeddings = v as usize;
        }
        if let Some(v) = raw.get("tie_word_embeddings").and_then(|v| v.as_bool()) {
            cfg.tie_word_embeddings = v;
        }
        cfg.bos_token_id = raw
            .get("bos_token_id")
            .and_then(|v| v.as_u64())
            .unwrap_or(cfg.bos_token_id as u64) as u32;
        cfg.eos_token_id = raw
            .get("eos_token_id")
            .and_then(|v| v.as_u64())
            .unwrap_or(cfg.eos_token_id as u64) as u32;
        let weights = std::fs::read(dir.join("model.safetensors")).map_err(|_| {
            Error::ModelNotFound(format!("model.safetensors missing in {}", dir.display()))
        })?;
        let vb = VarBuilder::from_slice_safetensors(&weights, DType::F32, device)
            .map_err(|e| Error::ModelLoadError(e.to_string()))?;
        let model = NtGenModel::load(vb, cfg.clone(), device)?;
        let tokenizer = tokenizers::Tokenizer::from_file(dir.join("tokenizer.json"))
            .map_err(|e| Error::ModelLoadError(format!("bad tokenizer.json: {e}")))?;
        let eos = cfg.eos_token_id;
        Ok(Self {
            model,
            tokenizer,
            eos,
        })
    }

    /// 生成：prefill + 自回归解码（遇 eos 或达上限停）
    pub fn generate(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        params: NtSampleParams,
    ) -> Result<String> {
        let ids = self
            .tokenizer
            .encode(prompt, true)
            .map_err(|e| Error::InferenceError(e.to_string()))?
            .get_ids()
            .to_vec();
        if ids.is_empty() {
            return Err(Error::InferenceError("empty prompt tokens".to_string()));
        }
        let mut sampler = NtSampler::new(params);
        let mut cache = self.model.empty_cache()?;
        let mut out: Vec<u32> = Vec::new();
        let mut seen: Vec<u32> = ids.clone();
        // prefill 全量 prompt，只取最后位置 logits
        let logits = self.model.forward(&ids, 0, &mut cache)?;
        let last = last_logits(&logits)?;
        let mut next = sampler.sample(&last, &seen);
        out.push(next);
        seen.push(next);
        let mut pos = ids.len();
        while out.len() < max_tokens.max(1) {
            if next == self.eos {
                break;
            }
            let logits = self.model.forward(&[next], pos, &mut cache)?;
            pos += 1;
            let last = last_logits(&logits)?;
            next = sampler.sample(&last, &seen);
            out.push(next);
            seen.push(next);
        }
        self.tokenizer
            .decode(&out, true)
            .map_err(|e| Error::InferenceError(e.to_string()))
    }
}

/// 取最后一个位置的 logits → Vec<f32>
fn last_logits(logits: &Tensor) -> Result<Vec<f32>> {
    let t = logits.dim(1).map_err(err)?;
    let last = logits
        .narrow(1, t.saturating_sub(1), 1)
        .map_err(err)?
        .squeeze(0)
        .map_err(err)?
        .squeeze(0)
        .map_err(err)?;
    last.to_vec1::<f32>().map_err(err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// 合成小权重（vocab 32 / hidden 16 / 1 层 / 2 头），走真实加载路径。
    /// 无 lm_head（测 tie）、无 q/k_norm（测跳过）。
    fn tiny_setup() -> (NtGenModel, NtGenConfig) {
        use safetensors::tensor::{Dtype, TensorView};
        let cfg = NtGenConfig {
            vocab_size: 32,
            hidden_size: 16,
            num_hidden_layers: 1,
            num_attention_heads: 2,
            num_key_value_heads: 2,
            intermediate_size: 32,
            rms_norm_eps: 1e-6,
            rope_theta: 10000.0,
            max_position_embeddings: 64,
            tie_word_embeddings: true,
            bos_token_id: 1,
            eos_token_id: 2,
        };
        // 拥有所有权的 (名, 形状, 字节)，views 借用它们，serialize 一次性拷出
        let mut owned: Vec<(String, Vec<usize>, Vec<u8>)> = Vec::new();
        let mut push = |name: &str, shape: &[usize]| {
            let n: usize = shape.iter().product();
            let mut bytes = Vec::with_capacity(n * 4);
            for i in 0..n {
                let v = 0.02 * ((i % 13) as f32 - 6.0);
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            owned.push((name.to_string(), shape.to_vec(), bytes));
        };
        push("model.embed_tokens.weight", &[32, 16]);
        push("model.layers.0.input_layernorm.weight", &[16]);
        // head_dim = hidden/heads = 8；k/v 走 GQA（kv=2 → [16,16]）
        push("model.layers.0.self_attn.q_proj.weight", &[16, 16]);
        push("model.layers.0.self_attn.k_proj.weight", &[16, 16]);
        push("model.layers.0.self_attn.v_proj.weight", &[16, 16]);
        push("model.layers.0.self_attn.o_proj.weight", &[16, 16]);
        push("model.layers.0.post_attention_layernorm.weight", &[16]);
        push("model.layers.0.mlp.gate_proj.weight", &[32, 16]);
        push("model.layers.0.mlp.up_proj.weight", &[32, 16]);
        push("model.layers.0.mlp.down_proj.weight", &[16, 32]);
        push("model.norm.weight", &[16]);
        let views: Vec<(String, TensorView)> = owned
            .iter()
            .map(|(n, s, b)| {
                (
                    n.clone(),
                    TensorView::new(Dtype::F32, s.clone(), b).unwrap(),
                )
            })
            .collect();
        let blob = safetensors::serialize(views, &None).unwrap();
        let device = Device::Cpu;
        let vb =
            VarBuilder::from_slice_safetensors(&blob, DType::F32, &device).unwrap();
        let model = NtGenModel::load(vb, cfg.clone(), &device).unwrap();
        (model, cfg)
    }

    #[test]
    fn test_config_default_minimind3() {
        let c = NtGenConfig::default();
        assert_eq!((c.num_hidden_layers, c.hidden_size), (8, 768));
        assert_eq!((c.num_attention_heads, c.num_key_value_heads), (8, 4));
        assert_eq!(c.vocab_size, 6400);
    }

    #[test]
    fn test_forward_shapes_and_cache_growth() {
        let (model, cfg) = tiny_setup();
        let mut cache = model.empty_cache().unwrap();
        let logits = model.forward(&[1, 5, 9], 0, &mut cache).unwrap();
        assert_eq!(logits.dims(), &[1, 3, cfg.vocab_size]);
        // cache 长到 3
        assert_eq!(cache[0].0.dim(2).unwrap(), 3);
        // 续解 1 个 token，cache 到 4，形状对
        let logits2 = model.forward(&[7], 3, &mut cache).unwrap();
        assert_eq!(logits2.dims(), &[1, 1, cfg.vocab_size]);
        assert_eq!(cache[0].0.dim(2).unwrap(), 4);
    }

    #[test]
    fn test_forward_deterministic_and_finite() {
        let (model, _) = tiny_setup();
        let run = || {
            let mut cache = model.empty_cache().unwrap();
            let logits = model.forward(&[3, 8], 0, &mut cache).unwrap();
            last_logits(&logits).unwrap()
        };
        let a = run();
        let b = run();
        assert_eq!(a, b, "same input must give identical logits");
        assert!(a.iter().all(|v: &f32| v.is_finite()), "no NaN/Inf allowed");
    }

    #[test]
    fn test_sampler_greedy_and_penalty() {
        let greedy = NtSampleParams {
            temperature: 0.0,
            top_k: None,
            top_p: None,
            repetition_penalty: 1.0,
            seed: 7,
        };
        assert_eq!(NtSampler::new(greedy.clone()).sample(&[1.0, 5.0, 3.0], &[]), 1);
        // 重复惩罚翻转 argmax（top_k=1 时采样退化为受罚 argmax，种子无关）
        let penalized = NtSampleParams {
            temperature: 1.0,
            top_k: Some(1),
            top_p: None,
            repetition_penalty: 2.0,
            seed: 7,
        };
        assert_eq!(
            NtSampler::new(penalized).sample(&[5.0, 4.0], &[0]),
            1,
            "token0 被惩罚后应选 token1"
        );
        // top-p 截断：尖峰分布只剩首位
        let nucleus = NtSampleParams {
            temperature: 1.0,
            top_k: None,
            top_p: Some(0.5),
            repetition_penalty: 1.0,
            seed: 7,
        };
        assert_eq!(NtSampler::new(nucleus).sample(&[10.0, 0.0, 0.0, 0.0], &[]), 0);
        // 空 logits 兜底
        assert_eq!(NtSampler::new(greedy).sample(&[], &[]), 0);
    }

    #[test]
    fn test_load_dir_missing_errors() {
        let r = NtGenEngine::load_dir(
            std::path::Path::new("/nonexistent-xyz-no-model"),
            &Device::Cpu,
        );
        assert!(r.is_err());
    }

    /// 活权重冒烟（默认忽略）：models/minimind-3 落盘即激活。
    ///
    /// 取权重（网络恢复后）：
    /// ```sh
    /// mkdir -p models/minimind-3 && cd models/minimind-3
    /// # MiniMind-3: https://github.com/jingyaogong/minimind
    /// huggingface-cli download <minimind-3-repo> --local-dir .
    /// # 需要：config.json + model.safetensors(F32) + tokenizer.json
    /// ```
    /// 跑：`cargo test -p neotrix-decision-engine live_minimind3 -- --ignored --nocapture`
    #[test]
    #[ignore = "needs real models/minimind-3 weights"]
    fn live_minimind3_generate() {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        root.pop();
        root.pop();
        let dir = root.join("models").join("minimind-3");
        if !dir.join("model.safetensors").exists() {
            return;
        }
        let mut eng = NtGenEngine::load_dir(&dir, &Device::Cpu).expect("load weights");
        let greedy = NtSampleParams {
            temperature: 0.0,
            top_k: None,
            top_p: None,
            repetition_penalty: 1.0,
            seed: 0,
        };
        let out = eng.generate("你好", 8, greedy).expect("generate");
        eprintln!("[live] output={out}");
        assert!(!out.trim().is_empty());
    }

    /// Metal 活权重冒烟（默认忽略，需 --features metal 编译）：
    /// `cargo test -p neotrix-decision-engine --features metal live_minimind3_metal -- --ignored --nocapture`
    /// 现状（2026-09-23）：device/权重加载 OK，前向卡在 rms-norm 无 metal 算子。
    /// 本测试保留为能力探针，转绿之日即 candle 补齐之时。
    #[test]
    #[ignore = "needs models/minimind-3 weights + metal feature + rms-norm kernel (missing in candle-metal 0.8)"]
    fn live_minimind3_metal() {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        root.pop();
        root.pop();
        let dir = root.join("models").join("minimind-3");
        if !dir.join("model.safetensors").exists() {
            return;
        }
        let device = auto_device();
        eprintln!("[live-metal] device={device:?}");
        let mut eng = NtGenEngine::load_dir(&dir, &device).expect("load weights");
        let greedy = NtSampleParams {
            temperature: 0.0,
            top_k: None,
            top_p: None,
            repetition_penalty: 1.0,
            seed: 0,
        };
        let out = eng.generate("你好", 8, greedy).expect("generate");
        eprintln!("[live-metal] output={out}");
        assert!(!out.trim().is_empty());
    }
}
