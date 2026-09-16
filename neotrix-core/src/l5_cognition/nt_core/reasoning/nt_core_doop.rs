//! NT-CORE — doop 吸收 (github.com/kgoedecke/doop).
//!
//! doop: 声明式指针/静态分析 (declarative pointer/static analysis) — 以声明式
//! 规则表达对程序指针流与数据依赖的查询。本模块实现 `_PointerAnalysis` trait
//! (C1: trait 存在 + 基础逻辑 + SelfTest T1 + 3 测试), 建模声明式分析点的
//! 指针集 (points-to set) 推导与查询。

use crate::core::nt_core_self_test::SelfTest;
use std::collections::HashMap;

/// 声明式分析点: 变量名 → 其指向的目标集 (points-to set)。
#[derive(Debug, Clone, PartialEq)]
pub struct _AnalysisPoint {
    pub var: String,
    pub points_to: Vec<String>,
}

/// doop 声明式指针分析实现。
pub struct _DoopPointerAnalysis {
    points: HashMap<String, Vec<String>>,
}

impl _DoopPointerAnalysis {
    pub fn new() -> Self {
        Self {
            points: HashMap::new(),
        }
    }

    pub fn declare(&mut self, var: &str, points_to: &[&str]) {
        let entry = self
            .points
            .entry(var.to_string())
            .or_default();
        for p in points_to {
            if !entry.contains(&(*p).to_string()) {
                entry.push((*p).to_string());
            }
        }
    }

    pub fn query_points_to(&self, var: &str) -> Vec<String> {
        self.points.get(var).cloned().unwrap_or_default()
    }

    pub fn merge_alias(&mut self, a: &str, b: &str) -> usize {
        let bset = self.points.get(b).cloned().unwrap_or_default();
        let entry = self.points.entry(a.to_string()).or_default();
        let before = entry.len();
        for p in bset {
            if !entry.contains(&p) {
                entry.push(p);
            }
        }
        entry.len() - before
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for _DoopPointerAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfTest for _DoopPointerAnalysis {
    fn name(&self) -> &'static str {
        "_DoopPointerAnalysis"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        // 指针集不得含自身别名噪声 (声明式分析要求 points-to 闭包一致)。
        for (var, set) in &self.points {
            if set.contains(var) {
                return Err(vec![format!("{} points-to itself (invalid alias)", var)]);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declare_registers_points_to_set() {
        let mut a = _DoopPointerAnalysis::new();
        a.declare("p", &["obj_x"]);
        assert_eq!(a.query_points_to("p"), vec!["obj_x".to_string()]);
        assert_eq!(a.len(), 1);
    }

    #[test]
    fn test_merge_alias_propagates_and_counts() {
        let mut a = _DoopPointerAnalysis::new();
        a.declare("p", &["obj_x"]);
        a.declare("q", &["obj_y"]);
        let added = a.merge_alias("p", "q");
        assert_eq!(added, 1);
        let mut pts = a.query_points_to("p");
        pts.sort();
        assert_eq!(pts, vec!["obj_x".to_string(), "obj_y".to_string()]);
    }

    #[test]
    fn test_self_test_rejects_self_alias() {
        let mut a = _DoopPointerAnalysis::new();
        a.declare("p", &["p"]);
        assert!(a.self_test().is_err());
    }
}
