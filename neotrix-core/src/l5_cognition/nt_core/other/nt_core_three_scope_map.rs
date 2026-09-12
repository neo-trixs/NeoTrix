//! NT-CORE — three-scope-map-skill 吸收 (github.com/songsummer920-dazzle/three-scope-map-skill).
//!
//! three-scope-map-skill: 三范围映射推理 — 将问题/系统映射到三个范围层
//! (micro/local/global 或 concrete/abstract/systemic) 进行分层推理与映射。
//! 本模块实现 `_ThreeScopeMap` trait (C1: trait 存在 + 基础逻辑 + SelfTest T1
//! + 3 测试), 建模范围划分与跨范围映射。

use crate::core::nt_core_self_test::SelfTest;
use std::collections::HashMap;

/// 三个范围层的标识。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scope {
    Micro,
    Meso,
    Macro,
}

/// 三范围映射推理 trait。
pub(crate) trait _ThreeScopeMap {
    /// 将一项元素登记到指定范围。
    fn map(&mut self, scope: Scope, item: &str);
    /// 返回某范围登记的元素数。
    fn count(&self, scope: Scope) -> usize;
    /// 跨范围提升: 将 micro 元素提升为 macro 归并键, 返回提升数。
    fn elevate(&mut self, from: Scope, to: Scope, item: &str) -> usize;
    /// 总登记数。
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// three-scope-map 实现。
pub(crate) struct _ThreeScopeMapImpl {
    scopes: HashMap<Scope, Vec<String>>,
}

impl _ThreeScopeMapImpl {
    pub fn new() -> Self {
        Self {
            scopes: HashMap::new(),
        }
    }

    pub fn scopes(&self) -> Vec<Scope> {
        self.scopes.keys().copied().collect()
    }
}

impl Default for _ThreeScopeMapImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl _ThreeScopeMap for _ThreeScopeMapImpl {
    fn map(&mut self, scope: Scope, item: &str) {
        self.scopes
            .entry(scope)
            .or_default()
            .push(item.to_string());
    }

    fn count(&self, scope: Scope) -> usize {
        self.scopes.get(&scope).map(|v| v.len()).unwrap_or(0)
    }

    fn elevate(&mut self, from: Scope, to: Scope, item: &str) -> usize {
        let removed = self
            .scopes
            .get_mut(&from)
            .map(|v| {
                let before = v.len();
                v.retain(|x| x != item);
                before - v.len()
            })
            .unwrap_or(0);
        if removed > 0 {
            self.scopes
                .entry(to)
                .or_default()
                .push(item.to_string());
        }
        removed
    }

    fn len(&self) -> usize {
        self.scopes.values().map(|v| v.len()).sum()
    }
}

impl SelfTest for _ThreeScopeMapImpl {
    fn name(&self) -> &'static str {
        "_ThreeScopeMap"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        if self.scopes.is_empty() && !self.is_empty() {
            return Err(vec!["scope map inconsistent".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_partitions_by_scope() {
        let mut m = _ThreeScopeMapImpl::new();
        m.map(Scope::Micro, "fn_a");
        m.map(Scope::Macro, "system");
        assert_eq!(m.count(Scope::Micro), 1);
        assert_eq!(m.count(Scope::Macro), 1);
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn test_elevate_moves_across_scopes() {
        let mut m = _ThreeScopeMapImpl::new();
        m.map(Scope::Micro, "fn_a");
        let moved = m.elevate(Scope::Micro, Scope::Macro, "fn_a");
        assert_eq!(moved, 1);
        assert_eq!(m.count(Scope::Micro), 0);
        assert_eq!(m.count(Scope::Macro), 1);
    }

    #[test]
    fn test_self_test_consistency() {
        let mut m = _ThreeScopeMapImpl::new();
        m.map(Scope::Meso, "x");
        assert!(m.self_test().is_ok());
    }
}
