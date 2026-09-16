//! NT-MIND — Recuris 吸收 (arXiv:2608.24876).
//!
//! Recuris: 递归经验工作记忆 (recurrent experiential working memory) — 为长程
//! agent 提供可递归重访的经验缓存。本模块实现 `_RecurisWorkingMemory` 经验缓存
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

/// Recuris 递归经验工作记忆实现。
pub struct _RecurisWorkingMemory {
    buffer: VecDeque<Experience>,
    capacity: usize,
}

impl _RecurisWorkingMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity: capacity.max(1),
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn remember(&mut self, input: &str, output: &str) -> usize {
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

    pub fn recall(&self, depth: usize) -> Option<&Experience> {
        let idx = self.buffer.len().checked_sub(1).and_then(|last| last.checked_sub(depth));
        idx.and_then(|i| self.buffer.get(i))
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

impl SelfTest for _RecurisWorkingMemory {
    fn name(&self) -> &'static str {
        "_RecurisWorkingMemory"
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
        let mut m = _RecurisWorkingMemory::new(4);
        let d0 = m.remember("a", "x");
        let d1 = m.remember("b", "y");
        assert_eq!(d0, 0);
        assert_eq!(d1, 1);
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn test_recall_by_recursive_depth() {
        let mut m = _RecurisWorkingMemory::new(4);
        m.remember("a", "x");
        m.remember("b", "y");
        assert_eq!(m.recall(0).unwrap().input, "b");
        assert_eq!(m.recall(1).unwrap().input, "a");
        assert!(m.recall(2).is_none());
    }

    #[test]
    fn test_capacity_eviction() {
        let mut m = _RecurisWorkingMemory::new(2);
        m.remember("a", "x");
        m.remember("b", "y");
        m.remember("c", "z");
        assert_eq!(m.len(), 2);
        assert_eq!(m.recall(0).unwrap().input, "c");
        assert!(m.self_test().is_ok());
    }
}
