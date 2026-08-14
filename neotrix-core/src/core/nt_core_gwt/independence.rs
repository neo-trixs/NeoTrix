//! §3.3.2 Theorem 42 可交换性判定 — 吸收自 cordiverse/paper:
//! "若每个 key 可交换 (Def 39), 则任何 coeffect-mediated effect functions
//! 互相独立" → 为 §3.1.3 independence 提供系统性供给。
//!
//! 分解 (P26): 计算分成可交换部分 (由 effects 承担, Corollary 21 任意顺序
//! 撤回) 与顺序敏感部分 (由 coeffects 承担 — 非交换 key 次序须从外部强加)。
//!
//! NeoTrix 消费者 (R-P79): **GWT 路由前提强化** — 广播前判定 specialist
//! effect 的可交换性, 决定 winner 集群能否任意序撤回 (独立) 或须由
//! 外部声明次序 (非交换)。

use std::collections::{HashMap, HashSet};

/// key 交换律分类 (Def 39)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum KeyCommutativity {
    /// 交换律成立: 两个 effect 操作同一 key 时顺序无关 (注册路由/事件监听)。
    Commutative,
    /// 顺序敏感: 非交换 key (middleware 链/累积器), 次序须外部强加。
    Ordered,
}

/// 单个 effect 的 key 声明。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EffectSpec {
    pub id: String,
    /// (key, 交换律)。同一 key 可被多 effect 引用。
    pub keys: Vec<(String, KeyCommutativity)>,
}

impl EffectSpec {
    /// 全交换 effect — 所有 key 声明为 Commutative。
    pub fn commutative(id: impl Into<String>, keys: &[&str]) -> Self {
        Self {
            id: id.into(),
            keys: keys.iter().map(|k| (k.to_string(), KeyCommutativity::Commutative)).collect(),
        }
    }

    /// 含顺序敏感 key 的 effect (非交换部分)。
    pub fn with_ordered(id: impl Into<String>, keys: &[(&str, KeyCommutativity)]) -> Self {
        Self {
            id: id.into(),
            keys: keys.iter().map(|(k, c)| (k.to_string(), *c)).collect(),
        }
    }

    fn commutativity_for(&self, key: &str) -> Option<KeyCommutativity> {
        self.keys.iter().find(|(k, _)| k == key).map(|(_, c)| *c)
    }
}

/// independence 判定结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndependenceVerdict {
    /// pairwise independent → 可任意顺序撤回 (Corollary 21)。
    Independent,
    /// 存在非交换 key 冲突 → 需外部强加次序。
    OrderedRequired { conflicts: Vec<(String, String)> },
}

/// 判定两个 effect 是否独立 (Def 19 + Theorem 42)。
///
/// 规则:
/// - key 无交集 → 独立。
/// - 有交集且交集中每个 key 在两侧都是 Commutative → 独立。
/// - 任一交集 key 为 Ordered → OrderedRequired, 列出冲突对。
pub fn independent(a: &EffectSpec, b: &EffectSpec) -> IndependenceVerdict {
    if a.id == b.id {
        return IndependenceVerdict::Independent;
    }
    let a_keys: HashSet<&str> = a.keys.iter().map(|(k, _)| k.as_str()).collect();
    let b_keys: HashSet<&str> = b.keys.iter().map(|(k, _)| k.as_str()).collect();
    let mut conflicts = Vec::new();
    for shared in a_keys.intersection(&b_keys) {
        let a_c = a.commutativity_for(shared).unwrap_or(KeyCommutativity::Ordered);
        let b_c = b.commutativity_for(shared).unwrap_or(KeyCommutativity::Ordered);
        if a_c == KeyCommutativity::Ordered || b_c == KeyCommutativity::Ordered {
            conflicts.push((a.id.clone(), b.id.clone()));
        }
    }
    if conflicts.is_empty() {
        IndependenceVerdict::Independent
    } else {
        IndependenceVerdict::OrderedRequired { conflicts }
    }
}

/// 聚合判定器 — 一组 effect 是否两两独立 (可任意序撤回)。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct IndependenceGate {
    specs: Vec<EffectSpec>,
}

impl IndependenceGate {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个 effect 的 key 声明。
    pub fn register(&mut self, spec: EffectSpec) {
        self.specs.retain(|s| s.id != spec.id);
        self.specs.push(spec);
    }

    pub fn contains(&self, id: &str) -> bool {
        self.specs.iter().any(|s| s.id == id)
    }

    pub fn spec(&self, id: &str) -> Option<&EffectSpec> {
        self.specs.iter().find(|s| s.id == id)
    }

    /// 判定给定 id 集合是否两两独立 (任意序撤回合法)。
    pub fn assert_any_order(&self, ids: &[&str]) -> IndependenceVerdict {
        let present: Vec<&EffectSpec> = ids
            .iter()
            .filter_map(|id| self.spec(id))
            .collect();
        for i in 0..present.len() {
            for j in (i + 1)..present.len() {
                if let IndependenceVerdict::OrderedRequired { conflicts } =
                    independent(present[i], present[j])
                {
                    return IndependenceVerdict::OrderedRequired { conflicts };
                }
            }
        }
        IndependenceVerdict::Independent
    }

    /// 给出推荐执行次序: 非交换 key 的依赖作为边, 拓扑排序。
    /// 无环 → 返回确定的偏序; 有环 → 返回 None (须人工裁决)。
    ///
    /// 语义: 可交换部分任意序 (Corollary 21), 顺序敏感部分由依赖强加
    /// (coeffect 承接次序)。
    pub fn order_for(&self, ids: &[&str]) -> Option<Vec<String>> {
        let present: Vec<&EffectSpec> = ids
            .iter()
            .filter_map(|id| self.spec(id))
            .collect();
        if present.is_empty() {
            return Some(Vec::new());
        }
        // 边: a 依赖 b 当且仅当共享 key 且 b 侧为 Ordered (b 的累积先于 a)。
        let mut dependents: HashMap<String, Vec<String>> = HashMap::new();
        let mut indegree: HashMap<String, usize> = HashMap::new();
        for spec in &present {
            indegree.entry(spec.id.clone()).or_insert(0);
        }
        for a in &present {
            for b in &present {
                if a.id == b.id {
                    continue;
                }
                let shared: Vec<&str> = a
                    .keys
                    .iter()
                    .map(|(k, _)| k.as_str())
                    .filter(|k| b.commutativity_for(k).is_some())
                    .collect();
                if shared.iter().any(|k| {
                    b.commutativity_for(k) == Some(KeyCommutativity::Ordered)
                }) {
                    // a 顺序敏感地依赖 b: b 先执行。
                    dependents.entry(b.id.clone()).or_default().push(a.id.clone());
                    *indegree.entry(a.id.clone()).or_insert(0) += 1;
                }
            }
        }
        // Kahn 拓扑排序。
        let mut queue: Vec<String> = indegree
            .iter()
            .filter(|(_, d)| **d == 0)
            .map(|(id, _)| id.clone())
            .collect();
        queue.sort_unstable();
        let mut order = Vec::new();
        while let Some(id) = queue.pop() {
            order.push(id.clone());
            if let Some(deps) = dependents.get(&id) {
                for dep in deps {
                    let d = indegree.get_mut(dep).unwrap();
                    *d -= 1;
                    if *d == 0 {
                        queue.push(dep.clone());
                        queue.sort_unstable();
                    }
                }
            }
        }
        if order.len() == present.len() {
            Some(order)
        } else {
            None // 有环 — 非交换依赖构成循环, 无法自动排序
        }
    }

    /// 不注册即视为 Commutative 独立 (无声明 = 无顺序约束)。
    pub fn known_ids(&self) -> Vec<String> {
        self.specs.iter().map(|s| s.id.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_shared_keys_are_independent() {
        let a = EffectSpec::commutative("a", &["cfg"]);
        let b = EffectSpec::commutative("b", &["logs"]);
        assert_eq!(independent(&a, &b), IndependenceVerdict::Independent);
    }

    #[test]
    fn test_shared_commutative_key_independent() {
        let a = EffectSpec::commutative("a", &["events"]);
        let b = EffectSpec::commutative("b", &["events"]);
        // 注册路由/事件监听共享 key 但可交换 → 独立
        assert_eq!(independent(&a, &b), IndependenceVerdict::Independent);
    }

    #[test]
    fn test_shared_ordered_key_requires_order() {
        let a = EffectSpec::commutative("a", &["accum"]);
        let b = EffectSpec::with_ordered("b", &[("accum", KeyCommutativity::Ordered)]);
        // middleware 链共享累积器 → 非交换, 须外部强加次序
        match independent(&a, &b) {
            IndependenceVerdict::Independent => panic!("should require order"),
            IndependenceVerdict::OrderedRequired { conflicts } => {
                assert_eq!(conflicts, vec![("a".to_string(), "b".to_string())]);
            }
        }
    }

    #[test]
    fn test_assert_any_order_pairwise() {
        let mut gate = IndependenceGate::new();
        gate.register(EffectSpec::commutative("r1", &["events"]));
        gate.register(EffectSpec::commutative("r2", &["events"]));
        // mw 与 r1 共享 events 但声明为 Ordered (middleware 链非交换)
        gate.register(EffectSpec::with_ordered("mw", &[("events", KeyCommutativity::Ordered)]));
        // r1+r2 独立 (共享可交换 key)
        assert_eq!(
            gate.assert_any_order(&["r1", "r2"]),
            IndependenceVerdict::Independent
        );
        // r1+mw 冲突 (mw 侧 events 为 Ordered)
        assert!(matches!(
            gate.assert_any_order(&["r1", "mw"]),
            IndependenceVerdict::OrderedRequired { .. }
        ));
    }

    #[test]
    fn test_order_for_commutative_any_order() {
        let mut gate = IndependenceGate::new();
        gate.register(EffectSpec::commutative("x", &["events"]));
        gate.register(EffectSpec::commutative("y", &["events"]));
        let order = gate.order_for(&["y", "x"]).unwrap();
        // 可交换: 任意序均合法 (拓扑排序给出确定偏序)
        assert_eq!(order.len(), 2);
    }

    #[test]
    fn test_order_for_ordered_dependency() {
        let mut gate = IndependenceGate::new();
        gate.register(EffectSpec::with_ordered("init", &[("accum", KeyCommutativity::Ordered)]));
        gate.register(EffectSpec::commutative("read", &["accum"]));
        // init 声明 accum 为 Ordered → read 依赖 init, 必须 init 先
        let order = gate.order_for(&["read", "init"]).unwrap();
        let pos = |id: &str| order.iter().position(|x| x == id).unwrap();
        assert!(pos("init") < pos("read"), "init 必须先于 read: {order:?}");
    }

    #[test]
    fn test_order_for_cycle_returns_none() {
        let mut gate = IndependenceGate::new();
        gate.register(EffectSpec::with_ordered("a", &[("k", KeyCommutativity::Ordered)]));
        gate.register(EffectSpec::with_ordered("b", &[("k", KeyCommutativity::Ordered)]));
        // 双方都声明 k 为 Ordered → 相互依赖环, 无法自动排序
        assert!(gate.order_for(&["a", "b"]).is_none());
    }

    #[test]
    fn test_register_replaces_same_id() {
        let mut gate = IndependenceGate::new();
        gate.register(EffectSpec::commutative("a", &["old"]));
        gate.register(EffectSpec::commutative("a", &["new"]));
        assert_eq!(gate.spec("a").unwrap().keys[0].0, "new");
        assert_eq!(gate.known_ids(), vec!["a"]);
    }
}