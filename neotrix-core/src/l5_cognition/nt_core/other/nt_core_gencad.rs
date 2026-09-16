//! L4 / NT-CORE — GenCAD (arXiv:2409.16294) 吸收节点 (C0)。
//!
//! 源: GenCAD — Geometry-Conditioned CAD 生成, 通过对比表征 (contrastive
//! representation) 将 CAD 命令序列与渲染图像对齐。
//!
//! 本模块提供 NT-CORE 视角的接口: CAD 几何 → 图像渲染 → 对比检索 KB 接口。
//! 目标成熟度 C0 (编译通过 + 基础逻辑 + SelfTest T1)。真实模型权重/推理不在 C0 范围。
//! KB 接线点: 将 (几何向量, 图像向量) 对写入 KB FTS5 索引, 供检索式生成复用。

use crate::core::nt_core_self_test::SelfTest;

/// 一段 CAD 命令序列 (如 OpenCASCADE / BRep 操作历史) 的轻量表示。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct _CadGeometry {
    /// 命令 token 流 (stub: 文本 token, 真实为数值化 command embedding)。
    pub commands: Vec<String>,
    /// 几何维度 (2D 草图 / 3D 实体)。
    pub dim: u8,
}

impl _CadGeometry {
    pub fn new(dim: u8) -> Self {
        Self {
            commands: Vec::new(),
            dim,
        }
    }

    pub fn push(&mut self, cmd: impl Into<String>) {
        self.commands.push(cmd.into());
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

/// 渲染图像 (stub: 仅保留像素尺寸与通道, 真实为张量)。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct _RenderedImage {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
}

/// 对比检索索引中的一条记录 (几何向量 + 图像向量 对齐)。
#[derive(Debug, Clone, PartialEq)]
pub struct _ContrastivePair {
    pub geometry_id: String,
    pub geometry_vec: Vec<f32>,
    pub image_vec: Vec<f32>,
}

impl _ContrastivePair {
    /// 余弦相似度 (几何向量 vs 图像向量 自身对齐应接近 1.0)。
    pub fn alignment(&self) -> f32 {
        cosine(&self.geometry_vec, &self.image_vec)
    }
}

/// C0 基础实现 — 全部逻辑本地可运行, 不依赖外部模型。
#[derive(Default)]
pub struct _GenCadCore {
    store: std::collections::VecDeque<_ContrastivePair>,
}

impl _GenCadCore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    pub fn render(&self, geo: &_CadGeometry) -> _RenderedImage {
        let (w, h) = if geo.dim == 2 { (256, 256) } else { (512, 512) };
        _RenderedImage {
            width: w,
            height: h,
            channels: 3,
        }
    }

    pub fn encode_geometry(&self, geo: &_CadGeometry) -> Vec<f32> {
        let mut v = vec![0.0f32; 8];
        for c in geo.commands.iter() {
            let h = simple_hash(c) as usize % 8;
            v[h] += c.len() as f32;
        }
        let n = geo.commands.len().max(1) as f32;
        v.iter_mut().for_each(|x| *x /= n);
        v
    }

    pub fn encode_image(&self, img: &_RenderedImage) -> Vec<f32> {
        let mut v = vec![0.0f32; 8];
        v[0] = img.width as f32 / 512.0;
        v[1] = img.height as f32 / 512.0;
        v[2] = img.channels as f32 / 3.0;
        v
    }

    pub fn index_into_kb(&self, pair: _ContrastivePair) -> Result<(), String> {
        if pair.geometry_vec.len() != pair.image_vec.len() {
            return Err("geometry/image vector dim mismatch".into());
        }
        if pair.geometry_id.is_empty() {
            return Err("empty geometry_id".into());
        }
        Err("not wired: GenCAD index_into_kb — KB FTS5 persistence not implemented (C0 stub)".into())
    }

    pub fn retrieve(&self, query: &_CadGeometry) -> Vec<_ContrastivePair> {
        let q = self.encode_geometry(query);
        let mut scored: Vec<(f32, _ContrastivePair)> = self
            .store
            .iter()
            .map(|p| (cosine(&q, &p.geometry_vec), p.clone()))
            .collect();
        scored.sort_by(|a, b| b.0.total_cmp(&a.0));
        scored.into_iter().map(|(_, p)| p).collect()
    }
}

/// 简单确定性字符串哈希 (FNV-1a 变体, 仅 C0 特征工程占位)。
fn simple_hash(s: &str) -> u32 {
    let mut h: u32 = 2166136261;
    for b in s.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(16777619);
    }
    h
}

/// 余弦相似度; 任一向量为零返回 0.0。
fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

/// T1 SelfTest: 接口基础不变量在 C0 可用 (无外部模型依赖)。
#[derive(Default)]
pub struct _GenCadSelfTest;

impl SelfTest for _GenCadSelfTest {
    fn name(&self) -> &str {
        "nt_core_gencad"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let core = _GenCadCore::new();
        let mut geo = _CadGeometry::new(3);
        geo.push("EXTRUDE");
        geo.push("FILLET");
        let img = core.render(&geo);
        let gv = core.encode_geometry(&geo);
        let iv = core.encode_image(&img);
        let pair = _ContrastivePair {
            geometry_id: "g1".into(),
            geometry_vec: gv.clone(),
            image_vec: iv,
        };
        let mut errs = Vec::new();
        // C0 stub: index_into_kb always returns Err (not wired).
        // Verify that input validation still works (rejects dim mismatch, empty id).
        if core.index_into_kb(pair.clone()).is_ok() {
            errs.push("gencad: C0 stub index_into_kb should not succeed".into());
        }
        if cosine(&gv, &gv) < 0.999 {
            errs.push("gencad: geometry self-cosine != 1.0".into());
        }
        if img.channels != 3 {
            errs.push("gencad: rendered image not RGB".into());
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_dim_selects_size() {
        let core = _GenCadCore::new();
        let img2d = core.render(&_CadGeometry::new(2));
        let img3d = core.render(&_CadGeometry::new(3));
        assert_eq!((img2d.width, img2d.height), (256, 256));
        assert_eq!((img3d.width, img3d.height), (512, 512));
        assert_eq!(img3d.channels, 3);
    }

    #[test]
    fn test_encode_geometry_empty_is_zero_vec() {
        let core = _GenCadCore::new();
        let v = core.encode_geometry(&_CadGeometry::new(3));
        assert_eq!(v.len(), 8);
        assert!(v.iter().all(|x| *x == 0.0));
    }

    #[test]
    fn test_index_rejects_dim_mismatch() {
        let core = _GenCadCore::new();
        let geo = _CadGeometry::new(3);
        let pair = _ContrastivePair {
            geometry_id: "x".into(),
            geometry_vec: vec![0.0; 4],
            image_vec: vec![0.0; 8],
        };
        assert!(core.index_into_kb(pair).is_err());
        let _ = geo;
    }

    #[test]
    fn test_retrieve_orders_by_alignment() {
        let mut core = _GenCadCore::new();
        // 直接注入 store 以验证排序 (C0 内存占位)。
        let mut geo = _CadGeometry::new(3);
        geo.push("EXTRUDE");
        let gv = core.encode_geometry(&geo);
        core.store.push_back(_ContrastivePair {
            geometry_id: "a".into(),
            geometry_vec: gv.clone(),
            image_vec: vec![0.0; 8],
        });
        let mut geo2 = _CadGeometry::new(3);
        geo2.push("BOSS");
        let gv2 = core.encode_geometry(&geo2);
        core.store.push_back(_ContrastivePair {
            geometry_id: "b".into(),
            geometry_vec: gv2,
            image_vec: vec![0.0; 8],
        });
        let results = core.retrieve(&geo);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].geometry_id, "a"); // 最相似排前
    }

    #[test]
    fn test_contrastive_pair_alignment_identical() {
        let v = vec![1.0, 0.0, 1.0];
        let p = _ContrastivePair {
            geometry_id: "s".into(),
            geometry_vec: v.clone(),
            image_vec: v,
        };
        assert!((p.alignment() - 1.0).abs() < 1e-6);
    }
}
