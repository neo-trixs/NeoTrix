//! nt_mind::testtime::verify — 验证循环 (生成-校验-重试)
//!
//! 节点: nt_mind::testtime::verify (L0)
//! Provides: testtime_reasoning, verify_loop

#![forbid(unsafe_code)]

use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};

/// 验证循环 — 对生成答案执行校验器, 失败则重试 (max_retries 封顶)
#[derive(Debug, Clone, Default)]
pub struct VerifyLoop {
    max_retries: usize,
}

impl VerifyLoop {
    pub fn new() -> Self {
        Self { max_retries: 3 }
    }

    pub fn with_max_retries(max_retries: usize) -> Self {
        Self { max_retries }
    }

    /// 迭代验证: 生成器产出候选, 校验器判定; 通过返回 Some(通过轮次)
    pub fn verify(
        &self,
        mut generate: impl FnMut(usize) -> String,
        mut check: impl FnMut(&str) -> bool,
    ) -> Option<(String, usize)> {
        let mut last = String::new();
        for round in 0..=self.max_retries {
            last = generate(round);
            if check(&last) {
                return Some((last, round));
            }
        }
        None
    }
}

impl CapabilityNode for VerifyLoop {
    fn node_id(&self) -> &str {
        "nt_mind::testtime::verify"
    }
    fn provides(&self) -> Vec<String> {
        vec!["testtime_reasoning".into(), "verify_loop".into()]
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

impl SelfTest for VerifyLoop {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let v = VerifyLoop::with_max_retries(3);
        let mut attempts = 0;
        let res = v
            .verify(
                |_| {
                    attempts += 1;
                    format!("candidate-{attempts}")
                },
                |s| s == "candidate-3",
            )
            .ok_or_else(|| vec!["应在第 3 轮通过".into()])?;
        assert_eq!(res.1, 2, "0-indexed 第 3 轮");
        assert_eq!(attempts, 3);
        // 全部失败 → None
        let none = v.verify(|_| "x".into(), |_| false);
        assert!(none.is_none());
        Ok(())
    }

    fn name(&self) -> &str {
        "nt_mind_testtime_verify"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passes_first_round() {
        let v = VerifyLoop::new();
        let res = v.verify(|_| "ok".into(), |_| true).unwrap();
        assert_eq!(res.1, 0);
    }

    #[test]
    fn test_retries_until_pass() {
        let v = VerifyLoop::with_max_retries(5);
        let mut n = 0;
        let res = v
            .verify(
                |_| {
                    n += 1;
                    n.to_string()
                },
                |s| s == "4",
            )
            .unwrap();
        assert_eq!(res.0, "4");
        assert_eq!(res.1, 3);
        assert_eq!(n, 4);
    }

    #[test]
    fn test_never_passes_returns_none() {
        let v = VerifyLoop::with_max_retries(2);
        let res = v.verify(|_| "bad".into(), |_| false);
        assert!(res.is_none());
    }

    #[test]
    fn test_zero_retries_single_try() {
        let v = VerifyLoop::with_max_retries(0);
        let mut calls = 0;
        let res = v.verify(
            |_| {
                calls += 1;
                "x".into()
            },
            |_| false,
        );
        assert!(res.is_none());
        assert_eq!(calls, 1);
    }
}
