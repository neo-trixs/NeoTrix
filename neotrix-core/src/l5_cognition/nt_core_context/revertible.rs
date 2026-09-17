//! ∂Γ 可回滚上下文 — 吸收自 cordiverse/paper §4.1.1 (spatiotemporal composability,
//! acyclic spatial slices + linear temporal order) 与 §5.3.1 (revertible
//! operations)，以及 deepseek-harness plugin.ts `load_batch` 的事务化语义
//! (all-or-nothing)。
//!
//! 机制:
//! - 每次变更记 `RevertibleEffect` (forward/inverse 对)。
//! - 栈式撤销 (undo_last / revert_key / revert_keys) + 整体 recover。
//! - 状态为泛型 `S` (例如插件事务化加载的累积状态)。
//!
//! NeoTrix 消费方 (R-P79): `nt_io_plugin/registry.rs::load_batch` — 一批插件
//! 装载失败即整体回滚, 与 "Everything is a Plugin" 范式的原子加载语义对齐。

/// 可回滚效果: 一对前向/逆变换。
pub trait RevertibleEffect<S>: Send + Sync {
    fn key(&self) -> &str;
    fn forward(&self, state: &mut S);
    fn inverse(&self, state: &mut S);
}

/// 闭包实现的效果 (名称 + forward/inverse 闭包)。
pub struct ClosureEffect<S> {
    name: String,
    forward: Box<dyn Fn(&mut S) + Send + Sync>,
    inverse: Box<dyn Fn(&mut S) + Send + Sync>,
}

impl<S> ClosureEffect<S> {
    pub fn new<F, I>(name: impl Into<String>, forward: F, inverse: I) -> Self
    where
        F: Fn(&mut S) + Send + Sync + 'static,
        I: Fn(&mut S) + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            forward: Box::new(forward),
            inverse: Box::new(inverse),
        }
    }
}

impl<S> RevertibleEffect<S> for ClosureEffect<S> {
    fn key(&self) -> &str {
        &self.name
    }
    fn forward(&self, state: &mut S) {
        (self.forward)(state);
    }
    fn inverse(&self, state: &mut S) {
        (self.inverse)(state);
    }
}

/// 便捷构造: 单个效果 (track 一次变更)。
pub fn add_effect<S>(
    name: &str,
    forward: impl Fn(&mut S) + Send + Sync + 'static,
    inverse: impl Fn(&mut S) + Send + Sync + 'static,
) -> ClosureEffect<S> {
    ClosureEffect::new(name, forward, inverse)
}

/// 可回滚上下文 (∂Γ)。
///
/// `S` 可为借用状态 (如 `&mut Registry`): 效果闭包生命周期受 `'a` 约束,
/// 使事务批处理能在方法体内持有借用。
pub struct RevertibleContext<'a, S> {
    state: S,
    effects: Vec<Box<dyn RevertibleEffect<S> + 'a>>,
}

impl<'a, S> RevertibleContext<'a, S> {
    pub fn new(state: S) -> Self {
        Self {
            state,
            effects: Vec::new(),
        }
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut S {
        &mut self.state
    }

    pub fn depth(&self) -> usize {
        self.effects.len()
    }

    pub fn is_clean(&self) -> bool {
        self.effects.is_empty()
    }

    pub fn keys(&self) -> Vec<&str> {
        self.effects.iter().map(|e| e.key()).collect()
    }

    /// 记录并执行一次前向变换。
    pub fn track(&mut self, effect: impl RevertibleEffect<S> + 'a) {
        effect.forward(&mut self.state);
        self.effects.push(Box::new(effect));
    }

    /// 撤销最近一次变更 (栈顶)。
    pub fn undo_last(&mut self) -> Option<()> {
        let effect = self.effects.pop()?;
        effect.inverse(&mut self.state);
        Some(())
    }

    /// 撤销指定 key 的最近一次变更 (从栈顶向下搜索)。
    pub fn revert_key(&mut self, key: &str) -> bool {
        for idx in (0..self.effects.len()).rev() {
            if self.effects[idx].key() == key {
                let effect = self.effects.remove(idx);
                effect.inverse(&mut self.state);
                return true;
            }
        }
        false
    }

    /// 按 key 集合全部撤销 (用于原子批处理失败回滚)。
    pub fn revert_keys(&mut self, keys: &[&str]) -> usize {
        keys.iter().filter(|k| self.revert_key(k)).count()
    }

    /// 整体回滚到初始状态。
    pub fn recover(&mut self) {
        while let Some(effect) = self.effects.pop() {
            effect.inverse(&mut self.state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_and_undo() {
        let mut ctx = RevertibleContext::new(0i32);
        ctx.track(add_effect(
            "inc",
            |s: &mut i32| *s += 1,
            |s: &mut i32| *s -= 1,
        ));
        ctx.track(add_effect(
            "inc2",
            |s: &mut i32| *s += 2,
            |s: &mut i32| *s -= 2,
        ));
        assert_eq!(*ctx.state(), 3);
        assert_eq!(ctx.depth(), 2);
        assert_eq!(ctx.keys(), vec!["inc", "inc2"]);
        ctx.undo_last();
        assert_eq!(*ctx.state(), 1);
        ctx.undo_last();
        assert_eq!(*ctx.state(), 0);
        assert!(ctx.is_clean());
    }

    #[test]
    fn test_revert_key_removes_middle() {
        let mut ctx = RevertibleContext::new(10i32);
        ctx.track(add_effect(
            "a",
            |s: &mut i32| *s += 1,
            |s: &mut i32| *s -= 1,
        ));
        ctx.track(add_effect(
            "b",
            |s: &mut i32| *s += 100,
            |s: &mut i32| *s -= 100,
        ));
        assert!(ctx.revert_key("a"));
        assert_eq!(*ctx.state(), 110);
        assert_eq!(ctx.keys(), vec!["b"]);
    }

    #[test]
    fn test_revert_keys_batch() {
        let mut ctx = RevertibleContext::new(0i32);
        ctx.track(add_effect(
            "x",
            |s: &mut i32| *s += 1,
            |s: &mut i32| *s -= 1,
        ));
        ctx.track(add_effect(
            "y",
            |s: &mut i32| *s += 10,
            |s: &mut i32| *s -= 10,
        ));
        ctx.track(add_effect(
            "z",
            |s: &mut i32| *s += 100,
            |s: &mut i32| *s -= 100,
        ));
        let n = ctx.revert_keys(&["x", "z"]);
        assert_eq!(n, 2);
        assert_eq!(*ctx.state(), 10);
        assert_eq!(ctx.keys(), vec!["y"]);
    }

    #[test]
    fn test_recover_rolls_back_everything() {
        let mut ctx = RevertibleContext::new(7i32);
        ctx.track(add_effect(
            "a",
            |s: &mut i32| *s += 3,
            |s: &mut i32| *s -= 3,
        ));
        ctx.track(add_effect(
            "b",
            |s: &mut i32| *s *= 2,
            |s: &mut i32| *s /= 2,
        ));
        ctx.recover();
        assert_eq!(*ctx.state(), 7);
        assert!(ctx.is_clean());
    }

    #[test]
    fn test_revert_unknown_key() {
        let mut ctx = RevertibleContext::new(0i32);
        ctx.track(add_effect(
            "a",
            |s: &mut i32| *s += 1,
            |s: &mut i32| *s -= 1,
        ));
        assert!(!ctx.revert_key("nope"));
        assert_eq!(ctx.depth(), 1);
    }

    #[test]
    fn test_undo_empty_is_none() {
        let mut ctx: RevertibleContext<i32> = RevertibleContext::new(0);
        assert!(ctx.undo_last().is_none());
    }

    #[test]
    fn test_state_mut_direct() {
        let mut ctx = RevertibleContext::new(vec![1, 2]);
        ctx.state_mut().push(3);
        assert_eq!(*ctx.state(), vec![1, 2, 3]);
        // 直接修改不进入 effects 栈 (调用方需自行 track)
        assert!(ctx.is_clean());
    }

    #[test]
    fn test_closures_are_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RevertibleContext<'static, ()>>();
        assert_send_sync::<ClosureEffect<()>>();
    }

    #[test]
    fn test_string_state() {
        let mut ctx = RevertibleContext::new(String::new());
        ctx.track(add_effect(
            "push_hi",
            |s: &mut String| s.push_str("hi"),
            |s: &mut String| {
                s.truncate(s.len() - 2);
            },
        ));
        ctx.track(add_effect(
            "push_bang",
            |s: &mut String| s.push('!'),
            |s: &mut String| {
                s.pop();
            },
        ));
        assert_eq!(*ctx.state(), "hi!");
        ctx.undo_last();
        assert_eq!(*ctx.state(), "hi");
        ctx.undo_last();
        assert_eq!(*ctx.state(), "");
    }

    // ── C2 集成测试: 生产消费形态 (借用状态 + 事务回滚) ──
    // 复刻 nt_io_plugin/registry.rs::load_batch 的真实消费形态: RevertibleContext
    // 以 `&mut *self` 借用状态, 批内失败 → recover() 整体回滚 (all-or-nothing)。
    // R-P79 验证: revertible_effects 被 registry 生产路径消费 (registry.rs:83/131)。

    /// 模拟注册表: 与 InnerRegistry 同构的最小借用状态。
    #[derive(Default)]
    struct FakeRegistry {
        plugins: std::collections::HashMap<String, String>,
    }

    #[test]
    fn test_borrowed_state_transaction_rollback() {
        // 消费形态 = RevertibleContext::new(&mut *self) — 借用状态 (registry.rs:83)
        let mut reg = FakeRegistry::default();
        {
            let mut ctx = RevertibleContext::new(&mut reg);
            ctx.track(add_effect(
                "load:alpha",
                |r: &mut &mut FakeRegistry| {
                    r.plugins.insert("alpha".into(), "v1".into());
                },
                |r: &mut &mut FakeRegistry| {
                    r.plugins.remove("alpha");
                },
            ));
            assert!(ctx.state().plugins.contains_key("alpha"));
            // 模拟第二个插件 on_load 失败 → 整批回滚
            ctx.recover();
        }
        assert!(
            !reg.plugins.contains_key("alpha"),
            "transaction rolled back"
        );
    }

    #[test]
    fn test_borrowed_state_partial_then_undo() {
        let mut reg = FakeRegistry::default();
        {
            let mut ctx = RevertibleContext::new(&mut reg);
            ctx.track(add_effect(
                "load:a",
                |r: &mut &mut FakeRegistry| {
                    r.plugins.insert("a".into(), "1".into());
                },
                |r: &mut &mut FakeRegistry| {
                    r.plugins.remove("a");
                },
            ));
            ctx.track(add_effect(
                "load:b",
                |r: &mut &mut FakeRegistry| {
                    r.plugins.insert("b".into(), "2".into());
                },
                |r: &mut &mut FakeRegistry| {
                    r.plugins.remove("b");
                },
            ));
            // 撤销 a (栈中间) — 模拟 revert_key 在借用状态下的消费
            assert!(ctx.revert_key("load:a"));
            assert!(!ctx.state().plugins.contains_key("a"));
            assert!(ctx.state().plugins.contains_key("b"));
            ctx.recover();
        }
        assert!(reg.plugins.is_empty(), "full rollback on borrow");
    }
}
