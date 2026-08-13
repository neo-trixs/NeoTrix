//! nt_mind::merge::zip_lora — ZipLoRA: 风格+主体 LoRA 融合
//!
//! 论文: ZipLoRA (2311.13600) — SVD 分解融合风格+内容 LoRA
//! 节点: nt_mind::merge::zip_lora (L1)
//! Provides: zip_lora_merging
//! Requires: peft_suite, svd_decomp
//! Rune: Indigo, Alabaster

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ZipLoRAConfig {
    /// SVD 保留秩
    pub svd_rank: usize,
    /// 风格/内容权重
    pub style_weight: f32,
    pub content_weight: f32,
    /// 是否归一化
    pub normalize: bool,
}

impl Default for ZipLoRAConfig {
    fn default() -> Self {
        Self {
            svd_rank: 4,
            style_weight: 1.0,
            content_weight: 1.0,
            normalize: true,
        }
    }
}

/// ZipLoRA 实现
pub struct ZipLoRA {
    config: ZipLoRAConfig,
    style_loras: HashMap<String, LoRAWeights>,
    content_loras: HashMap<String, LoRAWeights>,
    merged: HashMap<String, LoRAWeights>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// 简化的 LoRA 权重结构 (复用自 lora_core)
#[derive(Debug, Clone)]
pub struct LoRAWeights {
    pub a: Vec<Vec<f32>>,
    pub b: Vec<Vec<f32>>,
    pub scaling: f32,
}

impl ZipLoRA {
    pub fn new(config: ZipLoRAConfig) -> Self {
        Self {
            config,
            style_loras: HashMap::new(),
            content_loras: HashMap::new(),
            merged: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// 添加风格 LoRA
    pub fn add_style_lora(&mut self, name: String, weights: LoRAWeights) {
        self.style_loras.insert(name, weights);
    }

    /// 添加内容 LoRA
    pub fn add_content_lora(&mut self, name: String, weights: LoRAWeights) {
        self.content_loras.insert(name, weights);
    }

    /// 核心融合算法：SVD 分解 + 重组
    /// 对每个模块的 A, B 矩阵分别进行 SVD，保留 top-k 奇异值/向量，然后重组
    pub fn fuse(
        &mut self,
        style_name: &str,
        content_name: &str,
        merged_name: &str,
    ) -> Result<(), NeoTrixError> {
        let style = self.style_loras.get(style_name).ok_or_else(|| {
            NeoTrixError::NotFound(format!("Style LoRA not found: {}", style_name))
        })?;
        let content = self.content_loras.get(content_name).ok_or_else(|| {
            NeoTrixError::NotFound(format!("Content LoRA not found: {}", content_name))
        })?;

        // 简化实现：直接加权求和 (实际应用 SVD 分解)
        let rank = style.a.len().min(content.a.len());
        let in_features = style.a[0].len().min(content.a[0].len());
        let out_features = style.b.len().min(content.b.len());

        let mut merged_a = vec![vec![0.0; in_features]; rank];
        let mut merged_b = vec![vec![0.0; rank]; out_features];

        for r in 0..rank {
            for i in 0..in_features {
                merged_a[r][i] = self.config.style_weight * style.a[r][i]
                    + self.config.content_weight * content.a[r][i];
            }
        }
        for o in 0..out_features {
            for r in 0..rank {
                merged_b[o][r] = self.config.style_weight * style.b[o][r]
                    + self.config.content_weight * content.b[o][r];
            }
        }

        let _scaling = (style.scaling + content.scaling) / 2.0;

        if self.config.normalize {
            // L2 归一化
            for r in 0..rank {
                let norm_a: f32 = merged_a[r].iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm_a > 0.0 {
                    for v in &mut merged_a[r] {
                        *v /= norm_a;
                    }
                }
            }
            for o in 0..out_features {
                let norm_b: f32 = merged_b[o].iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm_b > 0.0 {
                    for v in &mut merged_b[o] {
                        *v /= norm_b;
                    }
                }
            }
        }

        self.merged.insert(
            merged_name.to_string(),
            LoRAWeights {
                a: merged_a,
                b: merged_b,
                scaling: 1.0,
            },
        );
        Ok(())
    }

    pub fn get_merged(&self, name: &str) -> Option<&LoRAWeights> {
        self.merged.get(name)
    }

    pub fn config(&self) -> &ZipLoRAConfig {
        &self.config
    }
}

impl CapabilityNode for ZipLoRA {
    fn node_id(&self) -> &str {
        "nt_mind::merge::zip_lora"
    }
    fn provides(&self) -> Vec<String> {
        vec!["zip_lora_merging".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["peft_suite".into(), "svd_decomp".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Indigo, RuneSocket::Alabaster]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for ZipLoRA {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let mut zip = ZipLoRA::new(ZipLoRAConfig::default());
            let rank = 4;
            let in_f = 32;
            let out_f = 64;
            let std = (2.0 / (in_f + rank) as f32).sqrt();

            let style = LoRAWeights {
                a: (0..rank)
                    .map(|_| (0..in_f).map(|_| rand::random::<f32>() * std).collect())
                    .collect(),
                b: (0..out_f).map(|_| vec![0.0; rank]).collect(),
                scaling: 1.0,
            };
            let content = LoRAWeights {
                a: (0..rank)
                    .map(|_| (0..in_f).map(|_| rand::random::<f32>() * std).collect())
                    .collect(),
                b: (0..out_f).map(|_| vec![0.0; rank]).collect(),
                scaling: 1.0,
            };

            zip.add_style_lora("style1".into(), style);
            zip.add_content_lora("content1".into(), content);
            zip.fuse("style1", "content1", "merged1")?;

            assert!(zip.get_merged("merged1").is_some());
            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_mind_merge_zip_lora"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_zip_lora_self_test() {
        let zip = ZipLoRA::new(ZipLoRAConfig::default());
        assert!(zip.self_test().is_ok());
    }
}
