//! L4 — Cognition (认知层) — 推理核心。
//!
//! 本模块原为独立 `ReasoningKernel` stub（固定返回 Deductive/0.5），与
//! L1 `nt_io_standalone` 平行重复且零生产消费方（仅 mod 声明 + re-export，
//! 违反 R-P42 平行适配器 + Dark Forest 孤儿规则）。
//!
//! 决策：删除重复 stub，改为 re-export L1 权威实现（`nt_io_standalone`），
//! 保留公共 API 路径 `neotrix::nt_core_kernel::*` 不变，消除双份漂移。
//! 真实推理能力（多步状态演化 + 方法选择 + self-consistency + 验证器）
//! 由 L1 实现提供，L8 生产路径已接线消费。

pub use crate::neotrix::l1_body_impl::nt_io_standalone::{
    EVOLUTION, KERNEL_DIM, ReasoningKernel, ReasoningMethod, ReasoningOutput,
    StageInfo, KernelStats, SelfConsistencyResult, verify_answer,
    text_to_vector, format_kernel_output,
};
pub use crate::core::nt_core_reasoning::{ReasoningTrace, TraceSource};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reasoning_kernel_new() {
        let k = ReasoningKernel::new(5);
        assert_eq!(k.stage, 5);
        assert_eq!(k.state.len(), KERNEL_DIM);
    }

    #[test]
    fn test_reasoning_kernel_stage_clamped() {
        let k = ReasoningKernel::new(100);
        assert_eq!(k.stage, EVOLUTION.len() - 1);
    }

    #[test]
    fn test_reasoning_kernel_evolve_stage() {
        let mut k = ReasoningKernel::new(0);
        k.evolve_stage();
        assert_eq!(k.stage, 1);
    }

    #[test]
    fn test_reasoning_kernel_evolve_stage_max() {
        let mut k = ReasoningKernel::new(EVOLUTION.len() - 1);
        k.evolve_stage();
        assert_eq!(k.stage, EVOLUTION.len() - 1);
    }

    #[test]
    fn test_reasoning_kernel_reason_real() {
        // P0: re-export 的 L1 实现产出真实多步 trace（非固定 0.5 stub）。
        let k = ReasoningKernel::new(3);
        let query = vec![0.5; KERNEL_DIM];
        let output = k.reason(&query, None, None);
        assert!(output.trace.intermediate_states.len() >= 2, "must evolve multiple steps");
        assert!(!output.trace.intermediate_states.is_empty());
        assert!(output.confidence > 0.0 && output.confidence <= 1.0);
    }

    #[test]
    fn test_reasoning_kernel_stats() {
        let k = ReasoningKernel::new(2);
        let stats = k.stats();
        assert_eq!(stats.stage, 2);
        assert_eq!(stats.state_dim, KERNEL_DIM);
    }

    #[test]
    fn test_stage_info_const() {
        assert_eq!(EVOLUTION.len(), 19);
        assert_eq!(EVOLUTION[0].label, "Stage 0");
        assert_eq!(EVOLUTION[0].description, "Initial");
    }

    #[test]
    fn test_kernel_dim_constant() {
        assert_eq!(KERNEL_DIM, 128);
    }

    #[test]
    fn test_verify_answer_reexport() {
        // 验证器经 re-export 可用（RLVR 锚）。
        assert!((verify_answer("42", "42") - 1.0).abs() < 1e-9);
        assert_eq!(verify_answer("", "42"), 0.0);
    }
}