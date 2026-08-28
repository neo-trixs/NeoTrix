//! NT-CORE — simplify-codebase 吸收 (github.com/tt-a1i/simplify-codebase).
//!
//! simplify-codebase: 代码库简化/重构 (codebase simplification/refactoring) —
//! 识别冗余、死代码与可合并结构, 产出重构候选。本模块实现 `CodeSimplifier`
//! trait (C1: trait 存在 + 基础逻辑 + SelfTest T1 + 3 测试), 建模冗余块
//! (redundancy) 检测与重构候选生成。

use crate::core::nt_core_self_test::SelfTest;
use std::collections::HashMap;

/// 单条重构候选: 冗余签名/建议操作。
#[derive(Debug, Clone, PartialEq)]
pub struct RefactorCandidate {
    pub kind: String,
    pub target: String,
    pub reason: String,
}

/// 代码库简化/重构 trait。
pub trait CodeSimplifier {
    /// 注册一个代码单元 (id) 及其内容指纹。
    fn register_unit(&mut self, id: &str, fingerprint: &str);
    /// 检测指纹重复 (冗余单元), 返回重构候选。
    fn detect_redundancy(&mut self) -> Vec<RefactorCandidate>;
    /// 已注册单元数。
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// simplify-codebase 实现。
pub struct SimplifyCodebase {
    units: HashMap<String, String>,
    fingerprints: HashMap<String, Vec<String>>,
}

impl SimplifyCodebase {
    pub fn new() -> Self {
        Self {
            units: HashMap::new(),
            fingerprints: HashMap::new(),
        }
    }
}

impl Default for SimplifyCodebase {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeSimplifier for SimplifyCodebase {
    fn register_unit(&mut self, id: &str, fingerprint: &str) {
        self.units.insert(id.to_string(), fingerprint.to_string());
        self.fingerprints
            .entry(fingerprint.to_string())
            .or_default()
            .push(id.to_string());
    }

    fn detect_redundancy(&mut self) -> Vec<RefactorCandidate> {
        let mut candidates = Vec::new();
        for (fp, ids) in &self.fingerprints {
            if ids.len() > 1 {
                // 首单元保留, 其余标记为可合并冗余。
                for dup in &ids[1..] {
                    candidates.push(RefactorCandidate {
                        kind: "duplicate_fingerprint".into(),
                        target: dup.clone(),
                        reason: format!(
                            "identical fingerprint '{}' with {} other units",
                            fp,
                            ids.len() - 1
                        ),
                    });
                }
            }
        }
        candidates
    }

    fn len(&self) -> usize {
        self.units.len()
    }
}

impl SelfTest for SimplifyCodebase {
    fn name(&self) -> &'static str {
        "SimplifyCodebase"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        for id in self.units.keys() {
            if id.is_empty() {
                return Err(vec!["unit id must not be empty".into()]);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_unit_tracks_count() {
        let mut s = SimplifyCodebase::new();
        s.register_unit("a", "fp1");
        s.register_unit("b", "fp2");
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn test_detect_redundancy_finds_duplicates() {
        let mut s = SimplifyCodebase::new();
        s.register_unit("a", "fp1");
        s.register_unit("b", "fp1");
        s.register_unit("c", "fp2");
        let cands = s.detect_redundancy();
        assert_eq!(cands.len(), 1);
        assert_eq!(cands[0].target, "b");
        assert_eq!(cands[0].kind, "duplicate_fingerprint");
    }

    #[test]
    fn test_self_test_rejects_empty_id() {
        let mut s = SimplifyCodebase::new();
        s.register_unit("", "fp1");
        assert!(s.self_test().is_err());
    }
}
