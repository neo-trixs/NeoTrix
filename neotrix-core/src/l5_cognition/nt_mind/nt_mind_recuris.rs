//! NT-MIND — Recuris 吸收 (arXiv:2608.24876).
//!
//! Recuris: 递归经验工作记忆 (recurrent experiential working memory) — 为长程
//! agent 提供可递归重访的经验缓存。本模块实现 `RecurisWorkingMemory` 经验缓存
//! trait (C1: trait 存在 + 基础逻辑 + SelfTest T1 + 3 测试)。

use crate::core::nt_core_self_test::SelfTest;
use std::collections::VecDeque;

/// 单条经验记录: 输入/输出/递归深度。
#[derive(Debug, Clone, PartialEq)]
pub struct Experience {
    pub input: String,
    pub output: String,
    pub depth: usize,
}

/// 递归经验工作记忆 trait。
pub trait WorkingMemory {
    /// 写入一条经验, 返回其递归深度 (当前缓存长度)。
    fn remember(&mut self, input: &str, output: &str) -> usize;
    /// 按递归深度重访最近经验 (0 = 最新), 无则 None。
    fn recall(&self, depth: usize) -> Option<&Experience>;
    /// 缓存容量。
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Recuris 递归经验工作记忆实现。
pub struct RecurisWorkingMemory {
    buffer: VecDeque<Experience>,
    capacity: usize,
}

impl RecurisWorkingMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity: capacity.max(1),
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl WorkingMemory for RecurisWorkingMemory {
    fn remember(&mut self, input: &str, output: &str) -> usize {
        let depth = self.buffer.len();
        self.buffer.push_back(Experience {
            input: input.to_string(),
            output: output.to_string(),
            depth,
        });
        while self.buffer.len() > self.capacity {
            self.buffer.pop_front();
        }
        depth
    }

    fn recall(&self, depth: usize) -> Option<&Experience> {
        let idx = self.buffer.len().checked_sub(1).and_then(|last| last.checked_sub(depth));
        idx.and_then(|i| self.buffer.get(i))
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }
}

impl SelfTest for RecurisWorkingMemory {
    fn name(&self) -> &'static str {
        "RecurisWorkingMemory"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        if self.capacity == 0 {
            return Err(vec!["capacity must be >= 1".into()]);
        }
        if self.buffer.len() > self.capacity {
            return Err(vec![format!(
                "buffer overflow: {} > {}",
                self.buffer.len(),
                self.capacity
            )]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remember_assigns_recursive_depth() {
        let mut m = RecurisWorkingMemory::new(4);
        let d0 = m.remember("a", "x");
        let d1 = m.remember("b", "y");
        assert_eq!(d0, 0);
        assert_eq!(d1, 1);
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn test_recall_by_recursive_depth() {
        let mut m = RecurisWorkingMemory::new(4);
        m.remember("a", "x");
        m.remember("b", "y");
        assert_eq!(m.recall(0).unwrap().input, "b");
        assert_eq!(m.recall(1).unwrap().input, "a");
        assert!(m.recall(2).is_none());
    }

    #[test]
    fn test_capacity_eviction() {
        let mut m = RecurisWorkingMemory::new(2);
        m.remember("a", "x");
        m.remember("b", "y");
        m.remember("c", "z");
        assert_eq!(m.len(), 2);
        assert_eq!(m.recall(0).unwrap().input, "c");
        assert!(m.self_test().is_ok());
    }
}
