use serde::{Deserialize, Serialize};

/// `VsaBackend` 收敛到低层唯一实现（2026-10-02）。
///
/// 与本批前三个类型同法：先逐项核实，再只收敛**能收敛的那部分**。
/// · 该 trait 两侧**逐字相同**（`diff` 实测无输出）⇒ 可安全收敛。
/// · core 侧两个 impl（`VSAEngine` / `HolonBackend`）在收敛后
///   变为「为本地类型实现外来 trait」⇒ 符合 orphan 规则，可编译。
///
/// ⛔ **但同文件的 `VSAEngine` 刻意不收敛**（与 trait 不同处理）：
/// core 的 `impl VsaBackend for VSAEngine` 与
/// `impl crate::neotrix::VsaEmbedding for VSAEngine`
/// **都要读 `self.dim`**，而 `dim` 在两个 crate 里**都是私有字段**（无 `pub`）。
/// ⇒ 若把 `VSAEngine` 收敛到低层，core 这两个 impl **访问不到该字段，编译失败**。
/// 该模块另有 7 个 core 文件同样触达 `VSAEngine` / `VsaBackend`
/// （`nt_memory_vsa_expand.rs`、`hyperdgm.rs`、`vsa_holon.rs`、
///  `nt_file_ability/tests.rs`、`tests/vsa_engine.rs`、`nt_core_hcube/mod.rs` 等）。
/// ⇒ 解开它必须先决定「`dim` 是否属于公开契约」（把私有字段改 `pub`
/// 是**低层 API 面变更**，不是顺手能做的去重）⇒ 留独立批次。
///
/// ⛔ **且上述只是「会编译失败」的那部分；下面 4 条是「会静默改行为」的，
/// 比编译失败更危险，原注释未记录（2026-10-04 逐行 diff 补记）：**
/// 1. **`bundle(&[])` 语义分叉**（两侧逐字读过，不是笔误）：
///    · 低层 `nt_core_hcube::vsa::bundle` 空输入分支 → `vec![0.0; self.dim]`（保维）
///    · core  本文件 `bundle` 同分支            → `Vec::new()`（**0 长**）
///    ⇒ 合并后 core 侧 `bundle(&[])` 会从「0 长」变成「dim 长」。
///    这是**返回值形状变化、不报错**，编译器拦不住 ⇒ 必须显式裁决后手改。
/// 2. **derive 不对等**：core 有 `Clone, Copy, Debug, Serialize, Deserialize`；
///    低层 `pub struct VSAEngine` **一个 derive 都没有**
///    （其上一行仅是 `///` 文档注释，实测）⇒ 直接换类型会丢 serde 能力。
/// 3. **`Default` 只在 core 侧**（dim=4096），低层没有。
/// 4. **`VsaEmbedding` 只在 core 侧**（token hashing），
///    且它有真实消费方 `l1_action/nt_file_ability/embedding.rs`
///    （签名是 `&dyn VsaEmbedding`）⇒ 不是可删的死 impl。
///
/// ⇒ 本笔只收敛 trait：**能收敛的收敛，不能的写明为什么不能。**
/// ⚠️ 下批若真要收敛，顺序必须是：先定 `bundle(&[])` 语义 → 再补低层
/// derive/`Default`/`VsaEmbedding` → 最后才动类型指派。**跳步即静默回归。**
/// 📌 本块**按符号名引用，刻意不写行号**：行号锚点必腐化，且本注释自身
/// 一经增删即令行号失效（本次增补 19 行即自证）。
pub use neotrix_types::core::nt_core_hcube::vsa::VsaBackend;

/// Default MAP-based VSA engine on real-valued vectors.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct VSAEngine {
    dim: usize,
}

impl Default for VSAEngine {
    fn default() -> Self {
        Self { dim: 4096 }
    }
}

impl VSAEngine {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }
}

impl VsaBackend for VSAEngine {
    fn bind(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).collect()
    }

    fn bundle(&self, vectors: &[&[f64]]) -> Vec<f64> {
        let Some(first) = vectors.first() else {
            return Vec::new();
        };
        let dim = first.len();
        let mut result = vec![0.0; dim];
        for v in vectors {
            for (r, x) in result.iter_mut().zip(v.iter()) {
                *r += x;
            }
        }
        result
    }

    fn permute(&self, v: &[f64], shift: isize) -> Vec<f64> {
        let len = v.len();
        if len == 0 {
            return Vec::new();
        }
        let mut result = vec![0.0; len];
        for (i, item) in result.iter_mut().enumerate() {
            let src = ((i as isize - shift).rem_euclid(len as isize)) as usize;
            *item = v[src];
        }
        result
    }

    fn similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let na: f64 = a.iter().map(|x| x * x).sum();
        let nb: f64 = b.iter().map(|x| x * x).sum();
        let norm = na.sqrt() * nb.sqrt();
        if norm < 1e-12 {
            0.0
        } else {
            dot / norm
        }
    }

    fn dimensions(&self) -> usize {
        self.dim
    }

    fn name(&self) -> &str {
        "map-vsa"
    }
}

impl crate::neotrix::VsaEmbedding for VSAEngine {
    fn embed_tokens(&self, tokens: &[&str]) -> Vec<f64> {
        if tokens.is_empty() {
            return vec![0.0; self.dim];
        }
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut result = vec![0.0f64; self.dim];
        for token in tokens {
            let mut hasher = DefaultHasher::new();
            token.hash(&mut hasher);
            let h = hasher.finish();
            for i in 0..self.dim {
                let bit = (h >> (i % 64)) & 1;
                result[i] += if bit == 1 { 1.0 } else { -1.0 };
            }
        }
        let norm: f64 = result.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 1e-12 {
            for r in &mut result {
                *r /= norm;
            }
        }
        result
    }

    fn similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        VsaBackend::similarity(self, a, b)
    }

    fn dimensions(&self) -> usize {
        VsaBackend::dimensions(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> VSAEngine {
        VSAEngine::new(4096)
    }

    #[test]
    fn test_bind_different_from_inputs() {
        let e = engine();
        let a: Vec<f64> = (0..4096).map(|i| (i as f64).sin()).collect();
        let b: Vec<f64> = (0..4096).map(|i| (i as f64).cos()).collect();
        let c = e.bind(&a, &b);
        let sim_a = e.similarity(&c, &a);
        let sim_b = e.similarity(&c, &b);
        assert!(sim_a.abs() < 0.1);
        assert!(sim_b.abs() < 0.1);
    }

    #[test]
    fn test_bundle_similar_to_all_components() {
        let e = engine();
        let a: Vec<f64> = (0..4096).map(|i| (i as f64).sin()).collect();
        let b: Vec<f64> = (0..4096).map(|i| (i as f64).cos()).collect();
        let c = e.bundle(&[&a, &b]);
        assert!(e.similarity(&c, &a) > 0.5);
        assert!(e.similarity(&c, &b) > 0.5);
    }

    #[test]
    fn test_permute_reversible() {
        let e = engine();
        let v: Vec<f64> = (0..4096).map(|i| (i as f64).sin()).collect();
        let p = e.permute(&v, 100);
        let r = e.permute(&p, -100);
        let sim = e.similarity(&r, &v);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_self_similarity_one() {
        let e = engine();
        let v: Vec<f64> = (0..4096).map(|i| (i as f64).sin()).collect();
        let sim = e.similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_dimensions() {
        let e = VSAEngine::new(1024);
        assert_eq!(e.dimensions(), 1024);
    }
}
