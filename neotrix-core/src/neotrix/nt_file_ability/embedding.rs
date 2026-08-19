//! VSA HyperCube Embedding (Ext-3) — 内容 → 高维超向量嵌入。

use std::path::Path;

use crate::core::nt_core_hcube::vsa::{VSAEngine, VsaBackend};

use super::core::FileAbility;
use super::types::Result;

/// 内容 → 高维超向量嵌入。
///
/// 复用 core 既有 `VSAEngine`/`VsaBackend` (R-P42，不平行重造 VSA)。
/// 方法: 对纯文本 token 序列，每个 token 由确定性 xorshift PRNG (seed=token hash)
/// 生成 `dim` 维 ±1 随机超向量；按位置 `permute` 编码顺序；`bundle` 求和后归一化。
/// 相似度经 `VsaBackend::similarity` (余弦) 度量。
pub fn embed_text(text: &str, dim: usize) -> Vec<f64> {
    let engine = VSAEngine::new(dim);
    if text.trim().is_empty() {
        return vec![0.0; dim];
    }
    let tokens: Vec<&str> = text.split_whitespace().collect();
    let mut occupancy = 0usize;
    let mut accumulator = vec![0.0; dim];
    for (idx, tok) in tokens.iter().enumerate() {
        let mut seed: u64 = 0xcbf29ce484222325;
        for b in tok.as_bytes() {
            seed ^= *b as u64;
            seed = seed.wrapping_mul(0x100000001b3);
        }
        let v = token_vector(&mut seed, dim);
        let shifted = if idx > 0 {
            engine.permute(&v, idx as isize)
        } else {
            v
        };
        for (a, x) in accumulator.iter_mut().zip(shifted.iter()) {
            *a += x;
        }
        occupancy += 1;
    }
    if occupancy == 0 {
        return vec![0.0; dim];
    }
    let norm = accumulator.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm < 1e-12 {
        vec![0.0; dim]
    } else {
        accumulator.iter().map(|x| x / norm).collect()
    }
}

/// xorshift64 PRNG 生成 ±1 hypervector (确定性, OS 零依赖)
fn token_vector(seed: &mut u64, dim: usize) -> Vec<f64> {
    let mut v = Vec::with_capacity(dim);
    for _ in 0..dim {
        let mut x = *seed;
        if x == 0 {
            x = 0x9e3779b97f4a7c15;
        }
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *seed = x;
        v.push(if x & 1 == 1 { 1.0 } else { -1.0 });
    }
    v
}

/// 两个文件的语义相似度 (内容已嵌入 → 余弦)
pub fn content_similarity(path_a: impl AsRef<Path>, path_b: impl AsRef<Path>) -> Result<f64> {
    let engine = VSAEngine::default();
    let dim = engine.dimensions();
    let a = FileAbility::open(path_a)?;
    let b = FileAbility::open(path_b)?;
    let va = embed_text(&a.plain_text(), dim);
    let vb = embed_text(&b.plain_text(), dim);
    Ok(engine.similarity(&va, &vb))
}