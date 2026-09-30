//! VSA HyperCube Embedding (Ext-3) — 内容 → 高维超向量嵌入。
//!
//! 通过 `VsaEmbedding` trait 抽象 NT-CORE 的 VSA 引擎能力，
//! 实现 L1 行动层 → L5 认知层的依赖倒置。

use std::path::Path;

use super::core::FileAbility;
use super::types::{Result, VsaEmbedding};

/// 内容 → 高维超向量嵌入。
///
/// 复用 core 既有 VSA 能力 (R-P42，不平行重造 VSA)。
/// 通过 `VsaEmbedding` trait 调用 NT-CORE 的嵌入实现。
///
/// 方法: 对纯文本 token 序列，每个 token 由确定性 xorshift PRNG (seed=token hash)
/// 生成 `dim` 维 ±1 随机超向量；按位置 `permute` 编码顺序；`bundle` 求和后归一化。
/// 相似度经 VsaEmbedding::similarity (余弦) 度量。
pub fn embed_text(text: &str, engine: &dyn VsaEmbedding) -> Vec<f64> {
    if text.trim().is_empty() {
        return vec![0.0; engine.dimensions()];
    }
    let tokens: Vec<&str> = text.split_whitespace().collect();
    engine.embed_tokens(&tokens)
}

/// 两个文件的语义相似度 (内容已嵌入 → 余弦)
pub fn content_similarity(
    path_a: impl AsRef<Path>,
    path_b: impl AsRef<Path>,
    engine: &dyn VsaEmbedding,
) -> Result<f64> {
    let a = FileAbility::open(path_a)?;
    let b = FileAbility::open(path_b)?;
    let _dim = engine.dimensions();
    let va = embed_text(&a.plain_text(), engine);
    let vb = embed_text(&b.plain_text(), engine);
    Ok(engine.similarity(&va, &vb))
}