//! nt_mind::align::policy — 对齐策略评分 + 门控
//!
//! 节点: nt_mind::align::policy (L1)
//! Provides: alignment_gating, policy_scoring

#![forbid(unsafe_code)]

use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};

/// 对齐策略评分器 — 依据校准偏置判断是否放行
#[derive(Debug, Clone, Default)]
pub struct AlignPolicy {
    min_bias: f32,
}

impl AlignPolicy {
    pub fn new() -> Self {
        Self { min_bias: 0.0 }
    }

    pub fn with_threshold(min_bias: f32) -> Self {
        Self { min_bias }
    }

    /// 依据偏置分数放行 (偏置不足 → 拒绝, 防止未对齐输出)
    pub fn gate(&self, bias: f32) -> bool {
        bias >= self.min_bias
    }

    pub fn threshold(&self) -> f32 {
        self.min_bias
    }
}

impl CapabilityNode for AlignPolicy {
    fn node_id(&self) -> &str {
        "nt_mind::align::policy"
    }
    fn provides(&self) -> Vec<String> {
        vec!["alignment_gating".into(), "policy_scoring".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["preference_alignment".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Golden, RuneSocket::Alabaster]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for AlignPolicy {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let p = AlignPolicy::with_threshold(0.5);
        assert!(p.gate(1.0), "高分应放行");
        assert!(!p.gate(0.0), "低分应拒绝");
        assert_eq!(p.threshold(), 0.5);
        Ok(())
    }

    fn name(&self) -> &str {
        "nt_mind_align_policy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_high_bias() {
        let p = AlignPolicy::with_threshold(0.2);
        assert!(p.gate(0.5));
        assert!(p.gate(0.2));
    }

    #[test]
    fn test_gate_low_bias() {
        let p = AlignPolicy::with_threshold(0.5);
        assert!(!p.gate(0.0));
        assert!(!p.gate(0.49));
    }

    #[test]
    fn test_default_threshold() {
        let p = AlignPolicy::new();
        assert_eq!(p.threshold(), 0.0);
        assert!(p.gate(-0.1) == false);
        assert!(p.gate(0.0));
    }

    #[test]
    fn test_threshold_getter() {
        let p = AlignPolicy::with_threshold(1.5);
        assert_eq!(p.threshold(), 1.5);
    }
}
