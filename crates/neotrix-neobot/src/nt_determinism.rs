//! ⭐⭐ **确定性三件套** —— 无状态可寻址采样 / 无别名状态摘要 / 有序键。
//!
//! ## 为什么有这个模块（2026-10-05 实测事故）
//!
//! 同会话吸收 `luckyyyyy/miu2d`（`engine-wasm/src/ai_search.rs`）与
//! `rust-alert/ra2.exe`（Apache-2.0）时，在**本仓自己的 LLM 网关热路径**上
//! 找到并修掉 3 处 LIVE 非确定性：
//!
//! | 位置 | 病 |
//! |---|---|
//! | `l0_substrate/nt_core_cache.rs::set_exact` | `HashMap::keys().next()` 当淘汰受害者 |
//! | `l0_substrate/nt_core_cache.rs::get_semantic` | `sim > best_sim` 同分时取哈希序首个 |
//! | `…/gateway/routing/selection.rs::build_candidate_chain` | 四级判据全并列，无名字兜底 |
//!
//! 三者同一病根：**用无序容器的遍历序去破平局**。危害到顶 ——
//! `resolve_default_model_sync` 取 `chain.first()`，⇒ **默认模型本身**会漂移。
//!
//! ⭐ 手搓回归测试时实测（`/tmp` 旁路程序，非估算）：单次 2 键淘汰有 **50%**
//! 概率**恰好**选中正确键 ⇒ 第一版测试在回退修复后**仍然通过**，制造了
//! 「已修好」的假象。所以**判别力必须实测**，不能声称。
//!
//! ## 本模块提供什么（都不引新依赖，`#![forbid(unsafe_code)]`）
//!
//! 1. [`stateless_sample`] —— **纯函数** `f(seed, tick, key)`，**无 RNG 游标**。
//!    取自 `ra2.exe` `gameplay/terrain_spawn.rs:170-179`（splitmix64 finalizer）
//!    与 `gameplay/ai_triggers.rs:197-203`（FNV 变体）。
//!    ⭐⭐ **可寻址 = 无需把游标存进状态**：没有游标就没有可失同步的东西，
//!    可重算、可并行、可重放。`luckyyyyy/miu2d` 的
//!    `ai_search.rs` 同理由把确定性写成「桶按 group 升序 ⇒ first-wins 帧间稳定」。
//!
//! 2. [`Digest`] —— 状态指纹。**每字段一次 `mul`+`add`，绝不把多字段移位塞进
//!    同一个累加器**；每个变体一个 `variant` 判别值。
//!
//! 3. [`sorted_keys`] —— 「遍历 `HashMap` 做决策前先排序键」这条纪律的可复用原语。
//!    取自 `ra2.exe` `gameplay/ai_triggers.rs:137-138`（它**不**禁止 `HashMap`，
//!    而是在迭代边界把键序规范化 —— 代价 2 行，收益消除整类 bug）。
//!
//! ## ⛔ 刻意**不**照抄 ra2.exe 摘要实现的两个缺陷（它自己有实测记录）
//!
//! | 缺陷 | 位置 | 后果 | 本模块的做法 |
//! |---|---|---|---|
//! | **位段别名冲突** | `persistence/digest.rs:59-79` | 21 个字段 `wrapping_add` 进**同一个**累加器且移位区间重叠、进位串扰 ⇒ `x=0,health=1` 与 `x=1,health=0` 摘要**相同**，此类失同步**不可见** | [`Digest`] 每字段独立一步 `mul`+`add` ⇒ 结构上不可能别名 |
//! | **字段覆盖靠手维护** | `persistence/digest.rs:15-152` | 加一个字段**不报错、不失败**，只是静默不再被检查；且 `match_seed` / `ai_trigger_runtime` 等整块未覆盖 | 本模块只提供**机制**；覆盖面由调用方显式列举，`variant` 强制声明变体身份 |
//!
//! 另：ra2.exe 全仓**没有** golden hash 常量（`rg EXPECTED_HASH` = 0 命中），
//! 其唯一确定性测试 `tests/engine/persistence/digest.rs:8-61` 是**同进程双胞胎
//! 差分**（两个独立构造 ⇒ Rust 给每个 `HashMap` 不同 hasher 种子 ⇒ 遍历序不同）。
//! ⭐ 该手法正是本仓 3 处 bug 的理想探测器，已用于
//! [`tests::twin_diff_detects_hash_order_tie_break`]。

use std::collections::HashMap;

/// FNV-1a 64 位素数（与 ra2.exe `persistence/digest.rs`、NeoTrix
/// `nt_shield_audit/mod.rs:827` 同值，便于交叉比对）。
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// FNV-1a 64 位偏移基。
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

/// 概率分母（百万分之一），与 RA2 原作的 `PROBABILITY_DENOMINATOR` 同口径。
pub const PROBABILITY_DENOMINATOR: u32 = 1_000_000;

/// splitmix64 finalizer（Vigna）。⭐ 只用 `wrapping_*`，**无 panic 路径**。
#[must_use]
pub fn splitmix64_finalize(mut h: u64) -> u64 {
    h ^= h >> 30;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^= h >> 31;
    h
}

/// ⭐ 无状态掷骰 —— `(seed, tick, key)` 的**纯函数**。
///
/// ⭐⭐ 与顺序 RNG 的关键差别：**没有游标**。同一个 `(seed, tick, key)`
/// 永远给同一个值，与「之前抽过什么」「谁先抽」**完全无关**。
/// ⇒ 可并行、可重试、可重放、可在任意子集上重算，均不失同步。
#[must_use]
pub fn mix(seed: u64, tick: u64, key: &str) -> u64 {
    let mut h = seed ^ tick.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    for b in key.as_bytes() {
        h = h.wrapping_mul(FNV_PRIME).wrapping_add(u64::from(*b));
    }
    splitmix64_finalize(h)
}

/// 概率样本：`0..PROBABILITY_DENOMINATOR`。`pct = sample_1m(..) as f64 / 1e6`。
#[must_use]
pub fn sample_1m(seed: u64, tick: u64, key: &str) -> u32 {
    (mix(seed, tick, key) % u64::from(PROBABILITY_DENOMINATOR)) as u32
}

/// ⭐⭐ 无别名状态摘要。
///
/// ## 与 ra2.exe `persistence/digest.rs` 的**关键**区别
///
/// ra2.exe 把 21 个字段 `wrapping_add` 进同一个累加器并各自移位，位段**重叠**
/// 且进位串扰 ⇒ `x=0,health=1` 与 `x=1,health=0` 摘要相同（该仓自述缺陷）。
///
/// 本类型**每字段一次** `h = h * FNV_PRIME + v`，**没有移位、没有共享累加器**
/// ⇒ 结构上不可能出现位段别名。
///
/// ## 用法纪律
///
/// - 每个**枚举变体**先调 [`Digest::variant`](Self::variant)，给唯一判别值
///   （抄 ra2.exe `digest.rs:163-252` 唯一的好设计：变体标签使编码单射，
///   换序或替换的命令无法别名）。
/// - 字段**必须逐个显式列出**。⚠️ 本类型**不**提供反射，因此「加了字段忘了
///   纳入摘要」仍是可能的 —— 那正是 ra2.exe 的缺陷 2。缓解手段只有一个：
///   把摘要构造放在**被测状态构造的同一函数**里，让二者同步修改。
#[derive(Debug, Clone)]
pub struct Digest {
    h: u64,
    fields: u32,
}

impl Default for Digest {
    fn default() -> Self {
        Self::new()
    }
}

impl Digest {
    /// 空摘要（以 [`FNV_OFFSET`] 起算）。
    #[must_use]
    pub fn new() -> Self {
        Self { h: FNV_OFFSET, fields: 0 }
    }

    /// 命名一个被摘要的**段**（如 `"entities"` / `"routing"`）。
    #[must_use]
    pub fn section(self, name: &str) -> Self {
        self.field_bytes(name.as_bytes())
    }

    /// ⭐ 枚举变体判别值。**同一次摘要里每个变体位置必须给唯一值。**
    #[must_use]
    pub fn variant(self, tag: u8) -> Self {
        self.field_u64(u64::from(tag))
    }

    #[must_use]
    pub fn field_u64(self, v: u64) -> Self {
        self.field_bytes(&v.to_le_bytes())
    }

    #[must_use]
    pub fn field_i64(self, v: i64) -> Self {
        // ⭐ `as u64` 保持位模式相同 ⇒ `field_i64` 与 `field_u64` 对同一数值等价，
        //   避免「同一事实两条编码路径」这种新的分歧源。
        self.field_u64(v as u64)
    }

    #[must_use]
    pub fn field_u32(self, v: u32) -> Self {
        self.field_u64(u64::from(v))
    }

    #[must_use]
    pub fn field_bool(self, v: bool) -> Self {
        self.field_u64(u64::from(v))
    }

    #[must_use]
    pub fn field_str(self, s: &str) -> Self {
        // ⭐ 长度也纳入 ⇒ `("ab","c")` 与 `("a","bc")` 不别名。
        self.field_u64(s.len() as u64).field_bytes(s.as_bytes())
    }

    /// 纳入一个**已排序**的字符串序列（配合 [`sorted_keys`] 使用）。
    #[must_use]
    pub fn field_sorted_keys(self, keys: &[String]) -> Self {
        let base = self.field_u64(keys.len() as u64);
        keys.iter().fold(base, |acc, k| acc.field_str(k))
    }

    fn field_bytes(mut self, bytes: &[u8]) -> Self {
        for b in bytes {
            self.h = self.h.wrapping_mul(FNV_PRIME).wrapping_add(u64::from(*b));
        }
        self.fields = self.fields.saturating_add(1);
        self
    }

    /// 摘要值。
    #[must_use]
    pub fn finish(&self) -> u64 {
        // 混入字段计数 ⇒ 「少摘一个字段」也会改变摘要（缓解 ra2 缺陷 2 的前半）。
        self.h ^ u64::from(self.fields).rotate_left(32)
    }

    /// 已纳入的字段数（自证用：可断言「我以为摘了 N 个」）。
    #[must_use]
    pub fn field_count(&self) -> u32 {
        self.fields
    }
}

/// ⭐⭐ 「遍历无序容器做决策前先排序键」这条纪律的可复用原语。
///
/// 抄 `ra2.exe` `gameplay/ai_triggers.rs:137-138`。它**不**禁止 `HashMap`
/// （那会与性能冲突），而是在**迭代边界**把键序规范化 —— 2 行代价，
/// 消除整类「同分取哈希序首个」的不可复现行为。
///
/// ## ⭐ 签名为什么是「键的迭代器」而不是 `&HashMap<K,V>`
///
/// ⓰ 我第一版写成 `&HashMap<K,V>`，**编译即失败**才发现自己想当然：本仓
/// `CapabilityTreeRegistry::nodes` 实际是 `indexmap::IndexMap`
/// （见 `nt_capability_registry.rs::capability_digest` 的实测）。
/// ⇒ 改成接收 `IntoIterator<Item = K>`，对 `HashMap` / `IndexMap` /
/// `BTreeMap` 一律可用，且**不给本 crate 引入 `indexmap` 依赖**。
///
/// 调用：`sorted_keys(map.keys().cloned())`。
#[must_use]
pub fn sorted_keys<K, I>(keys: I) -> Vec<K>
where
    K: Ord + Clone,
    I: IntoIterator<Item = K>,
{
    let mut v: Vec<K> = keys.into_iter().collect();
    v.sort();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 无状态采样 ────────────────────────────────────────────────────────

    #[test]
    fn stateless_sample_is_pure_and_addressable() {
        assert_eq!(sample_1m(7, 3, "a"), sample_1m(7, 3, "a"), "同参必须同值");
        assert_ne!(sample_1m(7, 3, "a"), sample_1m(7, 4, "a"), "tick 是寻址的一部分");
        assert_ne!(sample_1m(7, 3, "a"), sample_1m(8, 3, "a"), "seed 是寻址的一部分");
        assert_ne!(sample_1m(7, 3, "a"), sample_1m(7, 3, "b"), "key 是寻址的一部分");
    }

    #[test]
    fn stateless_sample_is_within_denominator() {
        for i in 0..2_000u64 {
            assert!(sample_1m(i, i * 7, "k") < PROBABILITY_DENOMINATOR);
        }
    }

    /// ⭐ 与顺序 RNG 的**本质**差别：求值顺序不影响结果集合。
    /// 顺序 RNG 抽第 k 次的值依赖前 k-1 次 ⇒ 换序即换值；本实现不可能。
    #[test]
    fn stateless_sample_is_order_independent() {
        let keys: Vec<String> = (0..64).map(|i| format!("k{}", i)).collect();
        let forward: Vec<u32> = keys.iter().map(|k| sample_1m(99, 5, k)).collect();
        let mut reversed_keys = keys.clone();
        reversed_keys.reverse();
        let backward: Vec<u32> = reversed_keys.iter().map(|k| sample_1m(99, 5, k)).collect();
        let mut backward_sorted = backward;
        backward_sorted.sort_unstable();
        let mut forward_sorted = forward.clone();
        forward_sorted.sort_unstable();
        assert_eq!(forward_sorted, backward_sorted, "正序与逆序应得同一多重集");
    }

    // ── 摘要：字段顺序敏感 ────────────────────────────────────────────────

    #[test]
    fn digest_is_field_order_sensitive() {
        let a = Digest::new().section("s").field_u64(1).field_u64(2).finish();
        let b = Digest::new().section("s").field_u64(2).field_u64(1).finish();
        assert_ne!(a, b, "字段顺序不同 ⇒ 摘要必须不同");
    }

    /// ⭐⭐ 直接钉住 ra2.exe `digest.rs:59-79` 的**位段别名**缺陷。
    /// 那份实现下 `x=0,health=1` 与 `x=1,health=0` 摘要相同；本实现必须不同。
    #[test]
    fn digest_has_no_bit_packing_alias() {
        let a = Digest::new().field_u64(0).field_u64(1).finish();
        let b = Digest::new().field_u64(1).field_u64(0).finish();
        assert_ne!(a, b, "字段值在字段间换位必须改变摘要（无位段别名）");
    }

    #[test]
    fn digest_field_count_participates() {
        let one = Digest::new().field_u64(7);
        assert_eq!(one.field_count(), 1);
        let two = one.clone().field_u64(7);
        assert_eq!(two.field_count(), 2);
        assert_ne!(one.finish(), two.finish(), "少摘一个字段也必须改变摘要");
    }

    #[test]
    fn digest_variant_disambiguates() {
        let a = Digest::new().variant(1).field_u64(9).finish();
        let b = Digest::new().variant(2).field_u64(9).finish();
        assert_ne!(a, b, "变体判别值必须参与摘要（换序/替换不可别名）");
    }

    #[test]
    fn digest_str_length_participates() {
        let a = Digest::new().field_str("ab").field_str("c").finish();
        let b = Digest::new().field_str("a").field_str("bc").finish();
        assert_ne!(a, b, "字符串切分不同不得别名（长度已纳入）");
    }

    #[test]
    fn digest_i64_matches_u64_bit_pattern() {
        let a = Digest::new().field_i64(-1).finish();
        let b = Digest::new().field_u64(u64::MAX).finish();
        assert_eq!(a, b, "同一数值不得因编码路径不同而产生两个摘要");
    }

    // ── 有序键纪律 ────────────────────────────────────────────────────────

    #[test]
    fn sorted_keys_is_deterministic_across_instances() {
        let build = || {
            let mut m = HashMap::new();
            for k in ["delta", "alpha", "charlie", "bravo"] {
                m.insert(k.to_owned(), 0u8);
            }
            m
        };
        // ⭐ 两个独立构造的 HashMap 种子不同 ⇒ 无序遍历序不同；排序后必须一致。
        assert_eq!(
            sorted_keys(build().keys().cloned()),
            sorted_keys(build().keys().cloned())
        );
    }

    /// ⭐⭐ **反例（oracle）测试**：证明双胞胎差分确实能抓到本仓那类 bug。
    ///
    /// 手法抄 `ra2.exe` `tests/engine/persistence/digest.rs:8-61`（同进程双胞胎
    /// 差分）：Rust 的 `RandomState` 每次实例化自增线程局部种子 ⇒ 两个 `HashMap`
    /// 键集相同而遍历序不同 ⇒ **若代码用遍历序破平局，双胞胎必然分叉**。
    ///
    /// 这里显式构造一个**故意不确定**的选择器，并断言它被抓到；
    /// 再断言 [`sorted_keys`] 版本稳定。⇒ 判别力是被证明的，不是声称的。
    #[test]
    fn twin_diff_detects_hash_order_tie_break() {
        // 故意坏：同分时取「哈希序首个」。
        let pick_buggy = |m: &HashMap<String, u8>| {
            let mut best: Option<(&String, &u8)> = None;
            for (k, v) in m {
                let better = best.is_none_or(|(_, bv)| v < bv);
                if better {
                    best = Some((k, v));
                }
            }
            best.map(|(k, _)| k.clone())
        };
        let build = || {
            let mut m = HashMap::new();
            // 全部同分（值全为 0）⇒ 胜者完全由遍历序决定。
            for k in ["a1", "b2", "c3", "d4", "e5", "f6", "g7", "h8"] {
                m.insert(k.to_owned(), 0u8);
            }
            m
        };

        let mut buggy_diverged = false;
        for _ in 0..64 {
            let (x, y) = (build(), build());
            if pick_buggy(&x) != pick_buggy(&y) {
                buggy_diverged = true;
                break;
            }
        }
        assert!(buggy_diverged, "双胞胎差分必须抓到哈希序破平局（否则该手法无判别力）");

        // 修好后：排序键 ⇒ 稳定。
        for _ in 0..64 {
            let (x, y) = (build(), build());
            assert_eq!(
                sorted_keys(x.keys().cloned()),
                sorted_keys(y.keys().cloned()),
                "排序键后必须稳定"
            );
        }
    }
}
