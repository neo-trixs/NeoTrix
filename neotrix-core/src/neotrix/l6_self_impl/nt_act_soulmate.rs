//! L6 / NT-ACT — soulmate (github.com/Soulmate-Halo/qiling-soulmate) 吸收节点 (C1)。
//!
//! 源: qiling-soulmate — "强脑弱手" (strong brain / weak hands) 编排范式:
//! 强模型负责推理与规划 (brain), 弱模型/CLI 负责执行 (hands); 并提供 Qiling
//! 上下文压缩以在有限上下文窗口内保留长程状态。NeoTrix 视角: 强/弱模型路由
//! + 上下文压缩 trait。

use crate::core::nt_core_self_test::SelfTest;

/// 模型角色: 强脑 (推理) 或 弱手 (执行)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelRole {
    StrongBrain,
    WeakHands,
}

/// 强/弱模型路由器。
pub trait BrainHandsRouter {
    /// 给定任务复杂度 (0..1), 选择执行角色。
    fn route(&self, complexity: f32) -> ModelRole;
    /// 对长上下文做 Qiling 压缩, 返回压缩后 token 计数 (stub: 按比例截断)。
    fn qiling_compress(&self, tokens: usize, budget: usize) -> usize;
}

pub struct SoulmateRouter {
    pub threshold: f32,
}

impl Default for SoulmateRouter {
    fn default() -> Self {
        Self { threshold: 0.6 }
    }
}

impl BrainHandsRouter for SoulmateRouter {
    fn route(&self, complexity: f32) -> ModelRole {
        if complexity >= self.threshold {
            ModelRole::StrongBrain
        } else {
            ModelRole::WeakHands
        }
    }

    fn qiling_compress(&self, tokens: usize, budget: usize) -> usize {
        if tokens <= budget {
            tokens
        } else {
            budget
        }
    }
}

#[derive(Default)]
pub struct SoulmateSelfTest;

impl SelfTest for SoulmateSelfTest {
    fn name(&self) -> &str {
        "nt_act_soulmate"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let r = SoulmateRouter::default();
        let mut errs = Vec::new();
        if r.route(0.9) != ModelRole::StrongBrain {
            errs.push("soulmate: high complexity must route to StrongBrain".into());
        }
        if r.route(0.1) != ModelRole::WeakHands {
            errs.push("soulmate: low complexity must route to WeakHands".into());
        }
        if r.qiling_compress(1000, 500) != 500 {
            errs.push("soulmate: compression must cap at budget".into());
        }
        if r.qiling_compress(100, 500) != 100 {
            errs.push("soulmate: compression must not expand".into());
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
    fn routes_strong_on_high() {
        assert_eq!(SoulmateRouter::default().route(0.9), ModelRole::StrongBrain);
    }

    #[test]
    fn routes_weak_on_low() {
        assert_eq!(SoulmateRouter::default().route(0.1), ModelRole::WeakHands);
    }

    #[test]
    fn compress_caps_to_budget() {
        assert_eq!(SoulmateRouter::default().qiling_compress(1000, 500), 500);
    }
}
