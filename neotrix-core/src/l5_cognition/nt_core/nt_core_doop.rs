//! NT-CORE — doop 吸收 (github.com/kgoedecke/doop).
//!
//! doop: 声明式指针/静态分析 (declarative pointer/static analysis) — 以声明式
//! 规则表达对程序指针流与数据依赖的查询。本模块实现 `PointerAnalysis` trait
//! (C1: trait 存在 + 基础逻辑 + SelfTest T1 + 3 测试), 建模声明式分析点的
//! 指针集 (points-to set) 推导与查询。

use crate::core::nt_core_self_test::SelfTest;
use std::collections::HashMap;

/// 声明式分析点: 变量名 → 其指向的目标集 (points-to set)。
#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisPoint {
    pub var: String,
    pub points_to: Vec<String>,
}

/// 声明式指针/静态分析 trait。
pub trait PointerAnalysis {
    /// 声明一个分析点并登记其初始指针集。
    fn declare(&mut self, var: &str, points_to: &[&str]);
    /// 查询某变量的声明式 points-to set, 无登记则空。
    fn query_points_to(&self, var: &str) -> Vec<String>;
    /// 合并两个变量的指针集 (传播/别名推导), 返回新增的别名数。
    fn merge_alias(&mut self, a: &str, b: &str) -> usize;
    /// 已登记变量数。
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// doop 声明式指针分析实现。
pub struct DoopPointerAnalysis {
    points: HashMap<String, Vec<String>>,
}

impl DoopPointerAnalysis {
    pub fn new() -> Self {
        Self {
            points: HashMap::new(),
        }
    }
}

impl Default for DoopPointerAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

impl PointerAnalysis for DoopPointerAnalysis {
    fn declare(&mut self, var: &str, points_to: &[&str]) {
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

    fn query_points_to(&self, var: &str) -> Vec<String> {
        self.points.get(var).cloned().unwrap_or_default()
    }

    fn merge_alias(&mut self, a: &str, b: &str) -> usize {
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

    fn len(&self) -> usize {
        self.points.len()
    }
}

impl SelfTest for DoopPointerAnalysis {
    fn name(&self) -> &'static str {
        "DoopPointerAnalysis"
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
        let mut a = DoopPointerAnalysis::new();
        a.declare("p", &["obj_x"]);
        assert_eq!(a.query_points_to("p"), vec!["obj_x".to_string()]);
        assert_eq!(a.len(), 1);
    }

    #[test]
    fn test_merge_alias_propagates_and_counts() {
        let mut a = DoopPointerAnalysis::new();
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
        let mut a = DoopPointerAnalysis::new();
        a.declare("p", &["p"]);
        assert!(a.self_test().is_err());
    }
}
