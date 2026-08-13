//! nt_act::rcm::chain — 因果链构建 (症状→根因回溯)
//!
//! 节点: nt_act::rcm::chain (L0)
//! Provides: root_cause_method, causal_chain

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ChainStep {
    Symptom(String),
    Hypothesis(String),
    Evidence(String),
    RootCause(String),
}

/// 因果链 — 按 症状→假设→证据→根因 顺序构建
#[derive(Debug, Clone, Default)]
pub struct CausalChain {
    steps: Vec<ChainStep>,
    settled: bool,
}

impl CausalChain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, step: ChainStep) -> Result<(), NeoTrixError> {
        if self.settled {
            return Err(NeoTrixError::InvalidState(
                "因果链已定案, 不可再追加".into(),
            ));
        }
        // 顺序约束: 根因必须最后
        if step == ChainStep::RootCause("".into()) {
            return Err(NeoTrixError::InvalidInput("根因不能为空".into()));
        }
        self.steps.push(step);
        Ok(())
    }

    /// 定案: 链必须以根因结尾
    pub fn settle(&mut self) -> Result<(), NeoTrixError> {
        let has_root = self
            .steps
            .last()
            .map(|s| matches!(s, ChainStep::RootCause(_)))
            .unwrap_or(false);
        if !has_root {
            return Err(NeoTrixError::InvalidState("定案要求链尾为根因".into()));
        }
        self.settled = true;
        Ok(())
    }

    pub fn steps(&self) -> &[ChainStep] {
        &self.steps
    }

    pub fn is_settled(&self) -> bool {
        self.settled
    }

    /// 返回根因 (若已定案)
    pub fn root_cause(&self) -> Option<&str> {
        if !self.settled {
            return None;
        }
        match self.steps.last()? {
            ChainStep::RootCause(rc) => Some(rc),
            _ => None,
        }
    }
}

impl CapabilityNode for CausalChain {
    fn node_id(&self) -> &str {
        "nt_act::rcm::chain"
    }
    fn provides(&self) -> Vec<String> {
        vec!["root_cause_method".into(), "causal_chain".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec![]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Indigo, RuneSocket::Golden]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for CausalChain {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut c = CausalChain::new();
        c.add(ChainStep::Symptom("panic 崩溃".into()))
            .map_err(|e| vec![e.to_string()])?;
        c.add(ChainStep::Hypothesis("内存越界".into()))
            .map_err(|e| vec![e.to_string()])?;
        c.add(ChainStep::Evidence("asan 报告越界写".into()))
            .map_err(|e| vec![e.to_string()])?;
        assert!(c.settle().is_err(), "无根因不能定案");
        c.add(ChainStep::RootCause("缓冲区长度计算错误".into()))
            .map_err(|e| vec![e.to_string()])?;
        c.settle().map_err(|e| vec![e.to_string()])?;
        assert_eq!(c.root_cause(), Some("缓冲区长度计算错误"));
        assert!(
            c.add(ChainStep::Symptom("x".into())).is_err(),
            "定案后不可追加"
        );
        Ok(())
    }

    fn name(&self) -> &str {
        "nt_act_rcm_chain"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_chain() {
        let mut c = CausalChain::new();
        c.add(ChainStep::Symptom("服务 500".into())).unwrap();
        c.add(ChainStep::Hypothesis("DB 连接池耗尽".into()))
            .unwrap();
        c.add(ChainStep::Evidence("连接数达上限".into())).unwrap();
        c.add(ChainStep::RootCause("连接未归还".into())).unwrap();
        c.settle().unwrap();
        assert!(c.is_settled());
        assert_eq!(c.steps().len(), 4);
    }

    #[test]
    fn test_settle_requires_root_cause() {
        let mut c = CausalChain::new();
        c.add(ChainStep::Symptom("x".into())).unwrap();
        assert!(c.settle().is_err());
        assert!(c.root_cause().is_none());
    }

    #[test]
    fn test_no_append_after_settle() {
        let mut c = CausalChain::new();
        c.add(ChainStep::RootCause("rc".into())).unwrap();
        c.settle().unwrap();
        assert!(c.add(ChainStep::Symptom("y".into())).is_err());
    }

    #[test]
    fn test_empty_root_cause_rejected() {
        let mut c = CausalChain::new();
        assert!(c.add(ChainStep::RootCause("".into())).is_err());
    }

    #[test]
    fn test_root_cause_only_after_settle() {
        let mut c = CausalChain::new();
        c.add(ChainStep::RootCause("rc".into())).unwrap();
        assert!(c.root_cause().is_none(), "未定案不暴露根因");
    }
}
