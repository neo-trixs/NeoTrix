//! NT-ACT 动作缓存自愈 (D16) — 记住先前动作免 LLM 推理, 选择器多组回退。
//!
//! 参照: stagehand (auto-caching + self-healing 记住先前动作免 LLM 推理)。
//! 机制: (状态签名 → 动作) 缓存命中即免推理; 动作执行失败时按选择器组
//! 依次回退 (self-healing), 直到成功或耗尽。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 动作条目 — 一条可缓存的动作。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAction {
    pub id: String,
    /// 语义签名: 当前状态 (URL + 页面摘要哈希 + 意图)
    pub signature: String,
    /// 要执行的动作描述 (如 "click #submit")
    pub action: String,
    /// 该动作对应的选择器组 (回退链, 第一个命中优先)
    pub selectors: Vec<String>,
    /// 命中计数
    pub hits: u64,
}

impl CachedAction {
    pub fn new(id: &str, signature: &str, action: &str, selectors: Vec<String>) -> Self {
        Self {
            id: id.to_string(),
            signature: signature.to_string(),
            action: action.to_string(),
            selectors,
            hits: 0,
        }
    }
}

/// 选择器组回退结果。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FallbackOutcome {
    /// 某选择器成功
    Success(String),
    /// 全部失败
    Failed,
}

/// 动作缓存 — 免推理 + 自愈回退。
#[derive(Debug, Clone, Default)]
pub struct ActionCache {
    by_signature: HashMap<String, CachedAction>,
}

impl ActionCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录一个成功动作 (缓存, 供下次免推理)。
    pub fn remember(&mut self, signature: &str, action: &str, selectors: Vec<String>) {
        let id = format!("act_{}", self.by_signature.len() + 1);
        self.by_signature.insert(
            signature.to_string(),
            CachedAction::new(&id, signature, action, selectors),
        );
    }

    /// 查询缓存 — 命中返回 Some(动作), 免 LLM 推理。
    pub fn lookup(&self, signature: &str) -> Option<&CachedAction> {
        self.by_signature.get(signature)
    }

    /// 记录命中。
    pub fn hit(&mut self, signature: &str) {
        if let Some(ca) = self.by_signature.get_mut(signature) {
            ca.hits += 1;
        }
    }

    /// 执行动作带自愈回退: 依次尝试 selectors, 注入的执行器判断成功与否。
    /// 返回第一个成功的选择器, 全失败则 Failed。
    pub fn execute_with_fallback(
        &self,
        action: &CachedAction,
        executor: &dyn Fn(&str) -> bool,
    ) -> FallbackOutcome {
        for s in &action.selectors {
            if executor(s) {
                return FallbackOutcome::Success(s.clone());
            }
        }
        FallbackOutcome::Failed
    }

    /// 带缓存免推理的完整执行: 命中→直接执行缓存动作; 未命中→由 miss_handler
    /// 产生动作并 remember。返回 (是否缓存命中, 执行结果)。
    pub fn act(
        &mut self,
        signature: &str,
        executor: &dyn Fn(&str) -> bool,
        miss_handler: &mut dyn FnMut() -> (String, Vec<String>),
    ) -> (bool, bool) {
        if let Some(ca) = self.lookup(signature).cloned() {
            self.hit(signature);
            let ok = self.execute_with_fallback(&ca, executor) != FallbackOutcome::Failed;
            (true, ok)
        } else {
            let (action, selectors) = miss_handler();
            self.remember(signature, &action, selectors);
            let ca = self.lookup(signature).cloned().unwrap();
            let ok = self.execute_with_fallback(&ca, executor) != FallbackOutcome::Failed;
            (false, ok)
        }
    }

    pub fn size(&self) -> usize {
        self.by_signature.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remember_then_lookup_hit() {
        let mut c = ActionCache::new();
        c.remember("https://x.dev", "click #go", vec!["#go".into(), "#submit".into()]);
        assert!(c.lookup("https://x.dev").is_some());
        assert!(c.lookup("https://other.dev").is_none());
        assert_eq!(c.size(), 1);
    }

    #[test]
    fn fallback_tries_selector_groups_in_order() {
        let c = ActionCache::new();
        let ca = CachedAction::new("1", "sig", "click", vec!["#a".into(), "#b".into(), "#c".into()]);
        // #a 失败, #b 成功
        let outcome = c.execute_with_fallback(&ca, &|s| s == "#b");
        assert_eq!(outcome, FallbackOutcome::Success("#b".into()));
    }

    #[test]
    fn fallback_exhausts_to_failed() {
        let c = ActionCache::new();
        let ca = CachedAction::new("1", "sig", "click", vec!["#a".into(), "#b".into()]);
        let outcome = c.execute_with_fallback(&ca, &|_| false);
        assert_eq!(outcome, FallbackOutcome::Failed);
    }

    #[test]
    fn act_hit_avoids_reinference() {
        let mut c = ActionCache::new();
        let mut miss_count = 0;
        // 第一次: miss → 用 miss_handler 记忆
        let (was_hit1, ok1) = c.act("sig1", &|_| true, &mut || {
            miss_count += 1;
            ("click #go".into(), vec!["#go".into()])
        });
        assert!(!was_hit1);
        assert!(ok1);
        // 第二次: 命中 → miss_handler 不再调用 (免推理)
        let (was_hit2, ok2) = c.act("sig1", &|_| true, &mut || {
            miss_count += 1;
            ("click #go".into(), vec!["#go".into()])
        });
        assert!(was_hit2);
        assert!(ok2);
        assert_eq!(miss_count, 1, "second act hit cache → no re-inference");
    }
}
