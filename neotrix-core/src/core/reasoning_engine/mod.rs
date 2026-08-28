//! reasoning_engine 模块入口 — 前向/反向/归因推理对外导出.

pub mod abductive;
pub mod backward;
pub mod chain_executor;
pub mod forward;

pub use chain_executor::{
    CausalRule, ChainExecutor, EdgeStep, ReasoningChainRecord, ReasoningResult, ReasoningStepNode,
    REASONING_RELATIONS,
};

use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};
use crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase;
use std::path::PathBuf;
use std::sync::Arc;

/// T3 SelfTest 接线 (NT-CORE reasoning_engine): 校验三种推理模式 (前向/反向/归因)
/// 均已接线且返回有效结果, 以及推理关系词汇非空 (核心不变量)。
pub struct ReasoningEngineSelfTest;

impl SelfTest for ReasoningEngineSelfTest {
    fn name(&self) -> &str {
        "reasoning_engine"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        if REASONING_RELATIONS.is_empty() {
            failures.push("reasoning_engine: REASONING_RELATIONS vocabulary empty".into());
        }
        let kb = match KnowledgeBase::open(Some(PathBuf::from(":memory:"))) {
            Ok(kb) => Arc::new(kb),
            Err(e) => {
                failures.push(format!("reasoning_engine: in-memory KB open failed: {e}"));
                return Err(failures);
            }
        };
        let ce = ChainExecutor::new(kb);
        let forward = ce.execute_forward("rust unsafe", 4);
        if forward.mode.is_empty() {
            failures.push("reasoning_engine: forward reasoning produced empty mode".into());
        }
        if !forward.confidence.is_finite() {
            failures.push("reasoning_engine: forward confidence is NaN/non-finite".into());
        }
        let backward = ce.execute_backward("memory safe");
        if backward.mode.is_empty() {
            failures.push("reasoning_engine: backward reasoning produced empty mode".into());
        }
        let abductive = ce.execute_abductive("slow build");
        if abductive.mode.is_empty() {
            failures.push("reasoning_engine: abductive reasoning produced empty mode".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

pub fn register_reasoning_engine_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(ReasoningEngineSelfTest));
}

#[cfg(test)]
mod selftest_tests {
    use super::*;

    #[test]
    fn test_reasoning_engine_self_test_passes() {
        let t = super::ReasoningEngineSelfTest;
        assert!(
            t.self_test().is_ok(),
            "ReasoningEngineSelfTest failed: {:?}",
            t.self_test().err()
        );
    }
}
