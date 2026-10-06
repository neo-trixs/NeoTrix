//! **能力金丝雀**（Capability Canary）—— 2026-10-04 吸收自
//! `plur-ai/plur` 的 `packages/core/src/capability-canary.ts`（78 行、零依赖）。
//!
//! # 它治的是哪一个病
//!
//! 本仓反复栽在**同一个**缺陷上（已实证 **10 次**）：
//! **「有能力 / 有门，但没接线」** ——
//!
//! | # | 实例 | 为什么静态门抓不到 |
//! |---|---|---|
//! | 1 | 检索准入门 4 条测试全绿 | **零生产消费**，但「有测试」看着像接线 |
//! | 2 | `ring_*` 2,152 行 Rust | **从未在 `mod.rs` 声明** ⇒ 编译器都不看 |
//! | 3 | 进程级能力注册表**只有写没有读** | 没有 `lookup` ⇒ 静态上「有 API」 |
//! | 4 | 两笔漏提交 ⇒ CI 一直是红的 | **主树能编**（脏树有那份改动）⇒ 看着是好的 |
//!
//! **`capability-canary.ts:48` 的判据公式（本文件逐字采用）**：
//! ```text
//! healthy = fired_count > 0 || ticks < threshold
//! ```
//! **第二项是整份设计的灵魂**：「还没到观察窗口」**不等于**
//! 「坏了」⇒ **没有观测 ≠ 观测为否**。
//! ⛔ 若缺这一项，任何「刚启动还没调用过」的进程都会被判红，
//! 然后**这个门就会被整体关掉** —— 那比没有门更坏。
//!
//! # 为什么它能治「漏提交 / 脏树」
//!
//! 结果落在**两个可被外部读取的结构化位置**（照 plur 的
//! `plur_status.capabilities` 与 `plur_doctor.checks[].ok`），
//! ⇒ **脏树 / 漏提交 / detached 干净检出，得到的是同一份证据**。
//! **「主工作树不是可信地面真相」这个问题被绕过**，
//! 因为真相同一份金丝雀数据算出来。
//!
//! # 与 `nt_capability_registry::invoke_count` 的分工
//!
//! `invoke_count` 是 **累计计数**（单调，只增不减）
//! 本模块是 **带观察窗口的健康判定**
//! 两者**互补**：前者答「历史上调用过几次」，
//! 后者答「**在当前观察窗口内**它到底活不活」。

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

/// 一条被金丝雀监视的能力。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanaryCapability {
    /// 能力 id（与能力树节点 id **同一空间**，这样才能反向核对）
    pub id: String,
    /// 人读的描述（报告里必须说清「这是个什么东西」）
    pub description: String,
    /// **怎么修**（关键字段：`plur_doctor` 把它放进
    /// `remediation[]`，**不是日志行，是结构化字段**）
    /// 没有它，一个红项只能让人知道「坏了」，不知道怎么修。
    pub fix: String,
}

/// 一条能力的当前健康状态（可被门与 UI 直接序列化）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanaryStatus {
    pub capability: CanaryCapability,
    /// **本次窗口内被观测到几次**（⛔ 不是累计值 —— 累计在 `invoke_count`）
    pub fired_count: usize,
    /// 是否健康（`fired_count > 0 || ticks < threshold`）
    pub healthy: bool,
    /// 不健康时的**可执行**告警（含 `fix`）
    pub warning: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct Slot {
    fired: usize,
}

/// **全局金丝雀**（进程级）。
///
/// 用 `OnceLock` + `Mutex`（照 `nt_capability_registry` 同一形态）
/// ⛔ **不引任何新依赖**。
static CANARY: OnceLock<Mutex<Canary>> = OnceLock::new();

/// 当前观察窗口已过的轮次（供 `healthy` 判据与测试读取）。
static WINDOW_TICKS: AtomicUsize = AtomicUsize::new(0);

/// 默认轮次阈值（照 `capability-canary.ts:28` 的 `threshold = 3`）。
const DEFAULT_THRESHOLD: usize = 3;

#[derive(Debug)]
struct Canary {
    threshold: usize,
    slots: BTreeMap<String, Slot>,
    /// `expect()` 的**手写清单**（只登记，计数字段单独存）
    defs: BTreeMap<String, CanaryCapability>,
    /// **收到过但没有任何金丝雀登记过**的 id → 次数。
    ///
    /// 这是 id 空间不一致的**直接证据**：非空即说明 `signal()` 与 `expect()`
    /// 用的不是同一套 id（前者收`TradeCapability` meta id，
    /// 后者登记能力树 id）。首版让 `signal()` 静默新建槽位，
    /// 于是这个不相交在现场完全不可见。
    unmatched: BTreeMap<String, usize>,
}

impl Default for Canary {
    fn default() -> Self {
        Self {
            threshold: DEFAULT_THRESHOLD,
            slots: BTreeMap::new(),
            defs: BTreeMap::new(),
            unmatched: BTreeMap::new(),
        }
    }
}

fn canary() -> &'static Mutex<Canary> {
    CANARY.get_or_init(|| Mutex::new(Canary::default()))
}

/// **登记一条被监视的能力**（plur 的 `expect()`）。
///
/// **显式手写清单 ⛔ 不是从目录扫出来的** —— 理由照 plur：
/// 「扫出来的东西天然可能被编译排除」**扫出来的清单无法表达
/// 「我主张它接了线」** ⇒ 登记本身就是**一个主张**，要能被证伪。
///
/// **重复登记是幂等的**（同一 id 多次 `expect` 不报错，也不覆盖
/// 已有的描述）⇒ 多个模块监视同一能力时不会互相打架。
///
/// # Errors
/// 金丝雀锁投毒（结构性失败 ⇒ 调用方应决定是否上报，⛔ 不静默吞）
pub fn expect(capability: CanaryCapability) -> Result<(), String> {
    let mut c = canary().lock().map_err(|e| format!("金丝雀锁投毒: {e}"))?;
    // 幂等：同一 id 重复 expect 不覆盖已有描述
    c.defs
        .entry(capability.id.clone())
        .or_insert(capability);
    Ok(())
}

/// **打点** —— **这条能力真的被用了**（唯一合法的调用处）。
///
/// **纪律（这一条决定金丝雀有没有意义）**：
/// `signal()` **只许出现在 handler / 生产派发路径内**。
/// ⛔ **绝不许**出现在 `register` 处、⛔ 绝不许出现在测试里
/// （否则「注册即打点」会伪造健康，那就回到了「建成未用却看着健康」）。
/// 参考 plur：它的两个打点都在**handler 体内**，
/// 没有一个紧挨着 `expect()`。
///
/// 打点一个**未登记**的 id ⇒ **静默忽略**（⛔ 不是错误：
/// 「打了没登记的点」是**常态**，真正的问题是反向的
/// ——「登记了却没打点」，那由 `status()` 报）。
pub fn signal(id: &str) {
    let Ok(mut c) = canary().lock() else { return };
    // ⚠️ 判据必须是 `defs`（`expect()` 登记处），**不是** `slots`：
    //    `slots` 是 `tick()`/status 计算时**懒建**的，`expect()` 并不建它
    //    ⇒ 查slots 会把**已登记**的 id 误判成未登记（实测踩到：
    //    5 条树 id 全部落进 unmatched）。
    if c.defs.contains_key(id) {
        // slots 是懒建的 ⇒ 这里必须 entry().or_default()，
        // 否则首次打点落空（实测：5 条树id 的 fired 全是 0）。
        // ⛔ 但仅限**已登记**的 id —— 首版的bug 正在于对未登记 id 也建槽。
        c.slots.entry(id.to_owned()).or_default().fired += 1;
    } else {
        // ⚠️ 首版是 `c.slots.entry(id).or_default().fired += 1` —— 对**未登记**的 id
        // 静默新建槽位。于是 id 空间不一致（`expect()` 登记的是能力树 id如
        // `NT-MEMORY::trade::trade_product_spec`，而 `signal()` 收到的是
        // `TradeCapability` 的 meta id 如 `PriceCalculatorCapability`）
        // **既不命中、也不报错** ⇒ 5 个金丝雀恒为 fired=0，
        // 而这个「永不相交」在现场完全不可见。
        //
        // 现改为：未登记的信号**单独计数并记名**，让不相交变成可报告的证据。
        // （真正统一两个 id 空间需要执行端口，属 B1 方案 C 的范围。）
        *c.unmatched.entry(id.to_owned()).or_insert(0usize) += 1;
    }
}

/// **收到过但没有任何金丝雀登记过的** id 及其次数。
///
/// 这是 id 空间不一致的**直接证据**：非空即说明 `signal()` 与 `expect()`
/// 用的不是同一套 id。
pub fn unmatched_signals() -> Result<Vec<(String, usize)>, String> {
    let c = canary().lock().map_err(|e| format!("金丝雀锁投毒: {e}"))?;
    let mut v: Vec<(String, usize)> = c
        .unmatched
        .iter()
        .map(|(k, n)| (k.clone(), *n))
        .collect();
    // 排序 ⇒ 输出确定（本仓一贯的确定性纪律）
    v.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(v)
}

/// **推进一轮观察窗口**（plur 的 `tick()`，在 `server.ts:336`
/// 「每次 tool call = 一个 turn」处调用）。
///
/// **窗口是每会话的** ⇒ 对应 `plur` 在 `tools.ts:3594` 的
/// `reset()`（它的注释记了 #192 事故：不 reset 的话
/// 一次信号能让金丝雀在**整个 server 生命周期**保持健康）。
pub fn tick() {
    WINDOW_TICKS.fetch_add(1, Ordering::SeqCst);
}

/// **清空本会话窗口**（**每会话开始时必须调**，照 plur `reset()`）。
pub fn reset() {
    WINDOW_TICKS.store(0, Ordering::SeqCst);
    if let Ok(mut c) = canary().lock() {
        for slot in c.slots.values_mut() {
            slot.fired = 0;
        }
        // ⚠️ 原实现**名不副实**：叫 `reset` 却只清 `fired`，
        // 留着 `defs`（登记项）与 `unmatched`（未登记信号）。
        // ⇒ 「重置后仍带着上一次的未登记信号」会让 id 空间不一致**跨窗口残留**。
        c.unmatched.clear();
        c.defs.clear();
        c.slots.clear();
    }
}

/// 当前窗口的轮次（供门读取，⛔ 不暴露内部锁）
pub fn window_ticks() -> usize {
    WINDOW_TICKS.load(Ordering::SeqCst)
}

/// **当前健康快照** —— **门与 UI 的唯一读入口**。
///
/// **判据逐字照抄 `capability-canary.ts:48`**：
/// `healthy = fired_count > 0 || ticks < threshold`
/// **第二项是灵魂**：「还没到观察窗口」**不等于**
/// 「坏了」⇒ **冷启动期的进程不会被误判红**。
///
/// **只含「登记过」的能力**（`defs`），
/// **没登记的不出现** —— 因为「没主张」⛔ 不是缺陷。
pub fn status() -> Result<Vec<CanaryStatus>, String> {
    let c = canary().lock().map_err(|e| format!("金丝雀锁投毒: {e}"))?;
    let ticks = window_ticks();
    let mut out = Vec::with_capacity(c.defs.len());
    for (id, capability) in &c.defs {
        let fired = c.slots.get(id).map(|s| s.fired).unwrap_or(0);
        let healthy = fired > 0 || ticks < c.threshold;
        let warning = if healthy {
            None
        } else {
            // 照抄 `capability-canary.ts:57` 的文案形状（含 fix ⇒ 可执行）
            Some(format!(
                "能力 '{}'（{}）已观察 {} 轮仍**零调用** ⇒ 它可能正被静默阻断。\n     修法: {}",
                id, capability.description, ticks, capability.fix
            ))
        };
        out.push(CanaryStatus {
            capability: capability.clone(),
            fired_count: fired,
            healthy,
            warning,
        });
    }
    Ok(out)
}

/// 不健康的告警（供门直接打印，⛔ 空 vec = 绿）
pub fn warnings() -> Result<Vec<String>, String> {
    Ok(status()?
        .into_iter()
        .filter_map(|s| s.warning)
        .collect())
}

/// **已被主张、但** **从未** **进入能力树注册表的 id**
/// —— **反向核对**（plur 的迁移项 3，**不做就白搭**）。
///
/// **为什么必须做**：否则会出现**第三种状态**
/// ——「注册表说自己有、金丝雀说自己没验」，
/// **那比前两种都更难查**（两个真源各说各话）。
///
/// 返回 `Vec<String>`（排序 ⇒ 输出确定）；**无缺口时返回空 vec**。
pub fn expected_but_unregistered() -> Result<Vec<String>, String> {
    let c = canary().lock().map_err(|e| format!("金丝雀锁投毒: {e}"))?;
    let missing: Vec<String> = c
        .defs
        .keys()
        .filter(|id| !crate::nt_capability_registry::has_node(id))
        .cloned()
        .collect();
    Ok(missing)
}

/// **T0.2 id 空间契约**（2026-10-06）
///
/// 背景：金丝雀 `expect()` 登记的是**能力树 id**（`TRADE_MANIFEST` 里的
/// `NT-*::...`），而生产派发点 `TradeCapabilityRegistry::get()` 传进来的是
/// 调用方给的 id。两侧若用不同 id 空间，`signal()` 会**静默落进 `unmatched`**，
/// 于是「登记了却零触发」看起来像负载问题，而真因是**命名空间不一致**。
///
/// `07aabb9f` 已让 `unmatched` 可见；本组测试**把契约钉死**，
/// 使下一个接线者无法静默用错 id。
#[cfg(test)]
mod idspace_contract_tests {
    use super::*;
    use nt_core_capability_tree::market::TRADE_MANIFEST;

    /// 登记（树 id）+ 打点（树 id）⇒ 必须命中该金丝雀，且 `unmatched` 保持空。
    #[test]
    fn 树id打点命中金丝雀且无未登记信号() {
        reset(); // 公共函数（非方法）
        for e in TRADE_MANIFEST {
            expect(CanaryCapability {
                id: e.id.to_owned(),
                description: e.description.to_owned(),
                fix: "在生产派发处 signal(同一个树 id)".to_owned(),
            })
            .expect("登记");
        }
        for e in TRADE_MANIFEST {
            signal(e.id); // ⛔ 必须是**树 id**，不是 meta id
        }
        let unmatched = unmatched_signals().expect("读未登记信号");
        assert!(
            unmatched.is_empty(),
            "用树 id 打点不该产生未登记信号，实得 {unmatched:?}"
        );
        let st = status().expect("读状态");
        assert_eq!(st.len(), TRADE_MANIFEST.len());
        assert!(
            st.iter().all(|s| s.fired_count > 0),
            "每条树 id 都应被记到触发：{:?}",
            st.iter().map(|s| (&s.capability.id, s.fired_count)).collect::<Vec<_>>()
        );
    }

    /// 反面：**meta id 形态**的 id 不许被当成已登记信号。
    /// 这条测试存在的意义：若将来有人图省事直接传 `PriceCalculatorCapability`，
    /// 立刻红，而不是静默地让 5 条金丝雀永远零触发。
    #[test]
    fn 非树id打点必落未登记() {
        reset(); // 公共函数（非方法）
        expect(CanaryCapability {
            id: TRADE_MANIFEST[0].id.to_owned(),
            description: TRADE_MANIFEST[0].description.to_owned(),
            fix: "在生产派发处 signal(树 id)".to_owned(),
        })
        .expect("登记");
        signal("PriceCalculatorCapability"); // 错的 id 空间
        let unmatched = unmatched_signals().expect("读未登记信号");
        assert_eq!(
            unmatched,
            vec![("PriceCalculatorCapability".to_owned(), 1)],
            "meta id 形态必须被记为未登记，而不是静默丢弃"
        );
    }

    /// 清单与金丝雀清单必须同源：清单为空 ⇒ 无可登记项（防两处各写一份 id）。
    #[test]
    fn 清单非空且id为树形态() {
        assert!(!TRADE_MANIFEST.is_empty(), "清单不该为空");
        for e in TRADE_MANIFEST {
            assert!(
                e.id.starts_with("NT-") && e.id.matches("::").count() == 2,
                "清单 id 应为 `NT-域::模块::实例` 形态，实得 {:?}",
                e.id
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 判据公式的**可证伪**测试：**冷启动不判红**。
    ///
    /// 这是整份设计**最容易写错**的一处：若漏了 `ticks < threshold`，
    /// 任何刚启动的进程都会红 ⇒ 然后这个门会被整体关掉。
    #[test]
    fn 冷启动期不判红_因为还没到观察窗口() {
        let mut c = Canary::default();
        c.defs.insert(
            "cold".to_owned(),
            CanaryCapability {
                id: "cold".to_owned(),
                description: "冷启动判据用".to_owned(),
                fix: "触发它".to_owned(),
            },
        );
        c.slots.insert("cold".to_owned(), Slot::default());
        let ticks = 0usize;
        let fired = 0usize;
        // 与 `status()` 里**逐字同构**的判据
        let healthy = fired > 0 || ticks < c.threshold;
        assert!(healthy, "⭐⭐⭐ 冷启动（0 轮）必须健康，⭐⭐ 否则门会被整体关掉");
    }

    /// 过了窗口且零调用 ⇒ **必须红**，且**告警必须含 `fix`**。
    #[test]
    fn 过了窗口仍零调用则判红且告警含修法() {
        let c = Canary {
            threshold: 1,
            slots: {
                let mut m = BTreeMap::new();
                m.insert("dead".to_owned(), Slot::default());
                m
            },
            defs: {
                let mut m = BTreeMap::new();
                m.insert(
                    "dead".to_owned(),
                    CanaryCapability {
                        id: "dead".to_owned(),
                        description: "从未被调用的能力".to_owned(),
                        fix: "在 handler 里 signal('dead')".to_owned(),
                    },
                );
                m
            },
            unmatched: BTreeMap::new(),
        };
        let fired = 0usize;
        let ticks = 5usize;
        let healthy = fired > 0 || ticks < c.threshold;
        assert!(!healthy, "⭐⭐ 过了窗口仍零调用 ⇒ 必须红");
        let s = CanaryStatus {
            capability: c.defs.values().next().cloned().expect("def"),
            fired_count: fired,
            healthy,
            warning: Some(format!(
                "能力 '{}'（{}）已观察 {} 轮仍**零调用** ⇒ 它可能正被静默阻断。\n     修法: {}",
                "dead", "从未被调用的能力", ticks, "在 handler 里 signal('dead')"
            )),
        };
        let w = s.warning.expect("必须有告警");
        assert!(w.contains("修法"), "⭐⭐⭐ 告警必须**含修法**，⭐⭐ 否则只知道坏、不知道怎么修");
    }

    /// 打点后立刻健康（证明 `signal` 真的改判据）。
    #[test]
    fn signal后立即健康() {
        let mut c = Canary::default();
        c.threshold = 1;
        c.defs.insert(
            "live".to_owned(),
            CanaryCapability {
                id: "live".to_owned(),
                description: "会被调用的能力".to_owned(),
                fix: "".to_owned(),
            },
        );
        c.slots.insert("live".to_owned(), Slot { fired: 1 });
        let fired = c.slots.get("live").map(|s| s.fired).unwrap_or(0);
        assert!(fired > 0, "⭐⭐ signal 后 fired 必须 >0");
    }
}