//!  **运行期能力注册表** —— 让「能力树」从**被测量的对象**变成**被使用的对象**。
//!
//! ## 为什么有这个模块（2026-10-03 实测结论）
//!
//! 能力树（`crates/nt-core-capability-tree`）**该有的都有**：节点模型、registry
//! （`register()` 可新增节点）、依赖边、成熟度模型、反虚标审计（`maturity_audit()`）、
//! CLI、以及 CI 的 `audit-maturity --strict` 门。
//!
//! ⛔ 但**运行时不消费它** —— 实测 `neotrix-core/src` 与 `crates/neotrix-neobot/src`
//! 对 `nt-core-capability-tree` **零引用**；全仓只有它自己的 crate、`Cargo.toml`、
//! 两份文档与两个门脚本的基线提到它。
//!
//! ⇒ 准确的差距**不是**「缺一个涌现引擎」，而是「已建好的能力树是一套被测量、
//! 却不被使用与生长的对象」。本模块就是那个缺失的接线。
//!
//! ## 判据（见 `docs/architecture/EMERGENCE-ROADMAP-2026-10-03.md` §1）
//!
//! > **涌现 = 能力树在无人工干预下新增了节点，且该节点通过 `audit-maturity --strict`。**
//!
//!  可验证性**不靠新造度量**，而靠**已存在**的 `maturity_audit()` —— 它已经拒绝
//! 「声称成熟度超过证据支撑」的节点。
//!
//! ##  为什么放在 `crates/neotrix-neobot/`（层归属裁决）
//!
//! 候选 A：`neotrix-core/src/l1_action/` —— ⛔ 会被 L2–L6 依赖，
//! 且 `check-layer-deps.sh` 的 `SRC` **就是** `neotrix-core/src` ⇒ 选错会被门拦。
//!
//! 候选 B（**采纳**）：`crates/neotrix-neobot/` ——
//!  `check-layer-deps` 只扫 `neotrix-core/src` ⇒ 本 crate 不受它管；
//!  **且 `neotrix-core/Cargo.toml:104` 依赖 `neotrix-neobot`**
//! ⇒ 意识侧的生产者（`neotrix-core/src/l4_emotion/nt_feel_facade.rs`，
//! 实测已有生产消费者）**可以直接调用本模块**，**零层违规**。
//!
//! ⛔ 刻意**不重造** maturity 词汇表 / 节点判据 —— 一律用 `nt-core-capability-tree`
//! 的真类型。理由：重造会产生第二套判据，而那正是「反虚标」要防的东西。

use std::sync::{Mutex, OnceLock};

use nt_core_capability_tree::node::CapabilityNode;
use nt_core_capability_tree::registry::{CapabilityTreeRegistry, RegistryError};

use crate::nt_determinism::{sorted_keys, Digest};

/// 进程级注册表。
///
///  用 `OnceLock<Mutex<…>>` 而非 `static mut` / `lazy_static`：
/// - `OnceLock` 是 std（MSRV 1.70+），本仓已在 `nt_echo` 等处用同一形态；
/// -  **不引任何新依赖**（`once_cell` / `lazy_static` 在本 crate 里零使用）。
static REGISTRY: OnceLock<Mutex<CapabilityTreeRegistry>> = OnceLock::new();

fn slot() -> &'static Mutex<CapabilityTreeRegistry> {
    REGISTRY.get_or_init(|| Mutex::new(CapabilityTreeRegistry::default()))
}

// ══════════════════════════════════════════════════════════════════
//  **读侧 + 调用计数**（2026-10-04）
// ══════════════════════════════════════════════════════════════════
//
//  **为什么补这个**（ 本轮实测发现， 不是推理）：
// 改前本模块的公开面是  **只有写、没有读** ——
// `register_node`（写）+ `with_registry` / `node_count` / `maturity_findings`
// / `capability_digest`（全是「关于注册表的查询」）。
// ⇒  **没有任何生产路径能经这个进程级注册表「取到一个能力并调用它」。**
// ⇒  这是「有能力没接线」这一类缺陷的  **第 9 例**， 而且是
//  **结构性的**：注册表被当成**只写日志**， 而不是**可派发的能力面**。
//
//  **补上读侧后，还顺带解锁了一件三家对标仓库都做不到的事**：
//  **「注册了但没调用」的检测**。 对标结论（2026-10-04 深挖）：
//  `os-taxonomy` / `lcu` / `backburner`  **三家全部没有**
//  「注册了却零调用」的直接度量 ——  它们只能证明
//  「声明 == 现实」（计数、摘要、符号、差分对拍），
//  **但「注册了」与「被用了」是两个不同的事实**， 只有后者能靠计数证伪。
//
//  计数放**旁挂表**而非改 `CapabilityNode`：
//  ⛔ 不改另一个 crate 的节点类型（那会把 `nt-core-capability-tree`
//  ⛔ 变成「为了加一个计数器而承担语义」）。
/// 每个能力的调用次数（ **单调递增**， 只增不减）。
///
///  用 `OnceLock` + `Mutex<HashMap<String, u64>>` 而非 per-node `AtomicU64`：
///  节点本身在另一个 crate 里， 旁挂表让本 crate  **零侵入**。
static INVOKE_COUNTS: OnceLock<Mutex<std::collections::HashMap<String, u64>>> =
    OnceLock::new();

fn counts() -> &'static Mutex<std::collections::HashMap<String, u64>> {
    INVOKE_COUNTS.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

///  **取一个能力节点 —— 注册表的「读侧」**（ 改前完全没有）。
///
///  这是  **唯一的生产调用路径**：能力被真正使用， **必须**经过这里。
/// ⇒  因此它同时是  **「有没有被用」这件事的唯一可信观测点**
///  （ 在别处打点都可以被绕过， 这里不行）。
///
/// # Errors
/// · 注册表锁投毒
/// · `id` 未登记 ⇒ `Ok(None)`（ **不是 Err**： 「没这个能力」是正常查询结果，
///   ⛔ 把它变成 Err 会让调用方分不清「不存在」与「系统坏了」）
pub fn lookup(id: &str) -> Result<Option<CapabilityNode>, String> {
    let found = {
        let reg = slot().lock().map_err(|e| format!("注册表锁投毒: {e}"))?;
        reg.nodes.get(id).cloned()
    };
    if found.is_some() {
        //  计数**只对真正取到的能力递增**： 未登记的 id  **不计数**
        //  （否则「查了 100 次不存在的 id」会被误读成「有 100 次真实调用」）。
        let mut c = counts().lock().map_err(|e| format!("计数锁投毒: {e}"))?;
        *c.entry(id.to_owned()).or_insert(0) += 1;
    }
    Ok(found)
}

///  **该 id 是否已登记进能力树**（ 供金丝雀做**反向核对**）。
///
///   **为什么必须有它**： 金丝雀的 `expect()` 是
///  **显式手写的主张清单**， 而能力树是**另一个真源**。
///  ⇒  若无此函数， 就会出现 **第三种状态**：
///  「金丝雀主张它接了线， 能力树里却根本没有它」
///  ⇒  **那比「有门没跑」更难查**（ 两个真源各说各话）。
pub fn has_node(id: &str) -> bool {
    slot()
        .lock()
        .map(|reg| reg.nodes.contains_key(id))
        .unwrap_or(false)
}

///  **按能力标签派发** ——  **两座 id 空间之间那座缺失的桥**。
///
///  能力树节点 id 形如 `NT-MEMORY::trade::trade_product_spec`，
///  而路由标签形如 `hybrid_retrieval` ⇒  **改前两者无法互达**，
///  于是「路由决策」永远变不成「一个可派发的能力」
///  ⇒ 这就是「注册了但没调用」的**结构性根因**。
///  `provides`  **本来就是连接键**， 而索引一直在维护它 ⇒ 只是没有 API。
///
///  **计数语义与 `dispatch_by_capability` 一致：**
///  **零副作用，纯解析**；标签无提供者 ⇒ 返回空 vec 且不计数。
///  计数只能经 `lookup` / `record_dispatch` 写入
/// （ 否则「路由到不存在的能力」或「只解析未执行」会被误读成「有真实调用」）。
pub fn resolve_by_capability(capability_tag: &str) -> Result<Vec<CapabilityNode>, String> {
    let ids = {
        let reg = slot().lock().map_err(|e| format!("注册表锁投毒: {e}"))?;
        reg.nodes_providing(capability_tag)
    };
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        let node = {
            let reg = slot().lock().map_err(|e| format!("注册表锁投毒: {e}"))?;
            reg.nodes.get(&id).cloned()
        };
        if let Some(n) = node {
            out.push(n);
        }
    }
    Ok(out)
}

/// 同 [`resolve_by_capability`]，但**纯解析、不计数**。
///
/// ⚠️ 修复记录（OPEN-DEFECTS #4）：原实现在解析到节点后即 `counts += 1`，
/// ⇒ 「解析但未执行」被记成「已调用」，`registered_never_invoked()` 清单
/// 因此被污染（解析一次就永久消失）。现改为**无副作用的纯解析**。
///
/// 语义闭环：调用方执行成功后应自行调 [`record_dispatch`] 记一次真实派发；
/// 仅解析（路由命中、rationale 附注、探测）**绝不**影响计数。
pub fn dispatch_by_capability(capability_tag: &str) -> Result<Vec<CapabilityNode>, String> {
    resolve_by_capability(capability_tag)
}

///  **「注册了但一次都没被调用」的清单** ——  **三家参考仓库都没有的能力**。
///
///  返回 `(id, 已调用次数)`， **只含次数为 0 的**。
///  ⛔ 刻意**不做**覆盖率百分比： 分母（「本该被调用多少次」） 无从定义，
///  造一个假分母会得到一个 **看起来精确、实则无法证伪**的数字
///  （ 对标 `lcu/tested.py` 的原则：「Informational, never refuses」）。
pub fn registered_never_invoked() -> Result<Vec<String>, String> {
    let reg = slot().lock().map_err(|e| format!("注册表锁投毒: {e}"))?;
    let c = counts().lock().map_err(|e| format!("计数锁投毒: {e}"))?;
    let mut out: Vec<String> = reg
        .nodes
        .keys()
        .filter(|id| c.get(*id).copied().unwrap_or(0) == 0)
        .cloned()
        .collect();
    //  排序 ⇒ 输出确定（ 对标本 crate `nt_determinism::sorted_keys` 的纪律）
    out.sort();
    Ok(out)
}

///  **按 id 记一次真实派发**（2026-06 随 `capability_invoke` 新增）。
///
/// ## 为什么需要它
///
/// `dispatch_by_capability` 只按**标签**解析（`hybrid_retrieval` 这类路由标签），
/// 是纯解析、不计数；而「模型按 id 调用一个已上架能力」这条通路**还需要记账原语**。
///
/// ⛔ 刻意**不**在 executor 里直接操作 `counts()`：那是本模块的私有状态，
///   绕过它就等于把计数规则散到多处 ⇒ 将来改规则会漏改。
///
/// ## 与「查到的次数」的关系
///
/// `invoke_count` 只读，本函数是唯一的按 id 写入口（另一入口是 `lookup`
/// 的「取到即计数」）。二者的分工：**本函数记「派发发生了」，不记「能力被执行
/// 成功」** —— 执行结果由调用方的 `ok` 字段表达，混进计数会让「调用过」
/// 变成一个含混的数字。
///
/// # Errors
/// 计数锁投毒（结构性失败 ⇒ 调用方应决定是否上报）
pub fn record_dispatch(id: &str) -> Result<usize, String> {
    let mut c = counts().lock().map_err(|e| format!("计数锁投毒: {e}"))?;
    let slot = c.entry(id.to_owned()).or_insert(0);
    *slot += 1;
    Ok(*slot as usize)
}

///  某能力的调用次数（ 未登记或从未调用 ⇒ 0）。
pub fn invoke_count(id: &str) -> usize {
    counts()
        .lock()
        .map(|c| c.get(id).copied().unwrap_or(0) as usize)
        .unwrap_or(0)
}

///  把一个能力节点登记进运行期能力树。
///
/// ## 语义
/// -  **同 id 重复登记 = 幂等成功**（底层 `register()` 返回 `AlreadyExists`），
///   且 `node_count()` **不变** ⇒ 「同一能力被多处发现」天然收敛。
/// - 结构性失败 ⇒ 返回 `Err`（如锁投毒），由调用方决定是否上报。
///   ⛔ 刻意**不**在这里 `log::warn!` 后返回 `Ok` —— 那会让「登记失败」
///   与「登记成功」不可区分，正是 `check-silent-failure` 要抓的形态。
///
/// # Errors
/// `CapabilityTreeRegistry::register` 的错误（重复以外的结构性冲突）。
pub fn register_node(node: CapabilityNode) -> Result<(), String> {
    let mut reg = slot().lock().map_err(|e| format!("能力注册表锁投毒: {e}"))?;
    //  `register()` 对同 id **直接返回 `AlreadyExists`**（实测 `registry.rs:169-171`）。
    //    ⓰ 我一度以为它是 upsert —— 那是 **`merge_overlay`** 的行为（`:552-556` 就地更新），
    //    两者职责不同。⇒ 「同一能力被多处发现」必须在此**显式**收敛成幂等成功。
    //  按**错误变体**匹配（`RegistryError::AlreadyExists`），⛔ 不用字符串 ——
    //   字符串匹配会在文案改动时静默失效（我第一版就踩了：写的是英文
    //   `already exists`，而真实文案是中文，且我连变体名都没核实）。
    match reg.register(node) {
        Ok(()) => Ok(()),
        Err(RegistryError::AlreadyExists(_)) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

///  把运行期注册表**借给**一个需要 `&mut CapabilityTreeRegistry` 的生产者。
///
/// ## 为什么要这个入口（2026-10-03 实测）
/// 仓里现成的节点生产者签名统一是：
/// ```ignore
/// pub fn register_xxx_capability(registry: &mut CapabilityTreeRegistry) -> CapabilityNode
/// ```
/// ⇒ 它们**只认 `&mut`**，⛔ 无法通过 `register_node(node)` 那个「传成品」的入口。
///  若为此给每个生产者改签名，就是**为接线改 8 处生产代码**；
/// 而本函数让它们**原样调用**，接线成本降为一个 `with_registry`。
///
///  **为什么这个入口必须诚实地暴露 `&mut`**：调用方能拿到完整注册表，
/// 也就**能绕过本模块的 `AlreadyExists` 幂等语义**（`register_node` 会把
/// `AlreadyExists` 吞成 `Ok`，而这里不会）。
/// ⇒  因此 `bootstrap` 类调用方**必须自己断言结果**，
/// ⛔ 否则拓扑序写错会**静默失败**（详见 `bootstrap_trade_capabilities`）。
///
/// # Errors
/// 注册表锁投毒。
pub fn with_registry<R>(f: impl FnOnce(&mut CapabilityTreeRegistry) -> R) -> Result<R, String> {
    let mut reg = slot().lock().map_err(|e| format!("能力注册表锁投毒: {e}"))?;
    Ok(f(&mut reg))
}

/// 当前已登记的节点数 ——  **涌现门要盯的就是这个数**（见路线 §5 第 2 步）。
///
///  「节点数不增长即判红」是**反向护栏**：钉住涌现必须是**行为**，
/// 否则「接线完成」会退化成「一次性注册固定几个节点」。
pub fn node_count() -> usize {
    slot().lock().map(|r| r.nodes.len()).unwrap_or(0)
}

///  跑一次成熟度审计 —— 返回**虚标节点**列表（空 = 无虚标）。
///
///  这是判据的另一半：新增节点必须**过这一关**，否则「涌现」退化为刷节点。
pub fn maturity_findings() -> Vec<String> {
    match slot().lock() {
        Ok(reg) => reg.maturity_audit().into_iter().map(|f| f.id).collect(),
        Err(_) => vec!["<lock poisoned>".to_owned()],
    }
}

///  **涌现指纹** —— 运行期能力树状态的**可复现单值摘要**。
///
/// ## 为什么需要它（2026-10-05）
///
/// 涌现探针原先只能报 `node_count()` 与若干计数。⛔ **计数不足以判定涌现**：
/// 「5 个能力节点」既可能是 5 次真实生长，也可能是同一次 bootstrap 的重复登记；
/// 更糟的是**节点换了一对而数量不变**时，计数完全看不见 —— 而那正是
/// 「能力被悄悄换掉」最需要被发现的形态。
///
///  指纹把「能力树当前是什么」压成一个 `u64`：同一状态 ⇒ 同一值；
/// 任一节点 id 或虚标结论变化 ⇒ 值变。⇒ 涌现从「数得出来」变成「比得了」。
///
/// ##  键序：⓰ 我第一版的理由是**错的**，实测修正
///
/// ⓰ 我原写「本函数必须先排序，否则指纹会漂移」—— 那是**想当然**。
/// 编译实测发现 `CapabilityTreeRegistry::nodes` 是 **`indexmap::IndexMap`**
/// 而非 `HashMap`，而 `IndexMap` **保持插入序** ⇒ 直接遍历**本来就是确定的**。
/// ⇒ 所以此处排序**不是**在修一个现存 bug。
///
///  那为什么仍然排序？（这才是它现在的正当理由）
/// 1. **防未来换容器**：本函数的价值就是「可复现」这个契约，而契约不该
///    依赖「某个字段恰好是插入序映射」这一实现细节。哪天有人为了性能把它
///    换成 `HashMap`，指纹会**静默**开始漂移，而**没有任何测试会红**。
///    排序把这个隐患变成不可能，代价 2 行。
/// 2. 与 [`crate::nt_determinism::sorted_keys`] 同款纪律，跨调用方可读。
/// 3. ⚠️ 附带收益：排序对**顺序敏感型容器**同样正确，不依赖「有序映射」假设。
///
/// ## ⛔ 锁投毒返回 `Err` 而非 `0`
///
/// 「无法测量」绝不能长得像「空树」—— 那会让探针把测量失败读成「涌现尚未发生」。
///
/// # Errors
/// 注册表锁投毒。
pub fn capability_digest() -> Result<u64, String> {
    let reg = slot().lock().map_err(|e| format!("能力注册表锁投毒: {e}"))?;
    let ids = sorted_keys(reg.nodes.keys().cloned());
    let mut d = Digest::new()
        .section("consciousness-capability-tree")
        .variant(1)
        .field_u64(ids.len() as u64)
        .field_sorted_keys(&ids);
    let mut findings: Vec<String> = reg.maturity_audit().into_iter().map(|f| f.id).collect();
    findings.sort();
    d = d.field_u64(findings.len() as u64);
    for f in &findings {
        d = d.field_str(f);
    }
    Ok(d.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nt_core_capability_tree::node::Domain;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static SEQ: AtomicUsize = AtomicUsize::new(0);
    ///  注册表是**进程全局**的（`OnceLock<Mutex<…>>`），而 `cargo test` **并行跑**
    /// ⇒ 断言 `before + 1` 会被别的测试同时 `register` 打破。
    /// ⓰ 我第一版就是这么写的，实测报 `left: 2, right: 1`
    /// ⇒ 本模块内所有测试**必须**持这把锁才能对节点数下断言。
    static SERIAL: Mutex<()> = Mutex::new(());

    fn serial<T>(f: impl FnOnce() -> T) -> T {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        f()
    }

    fn probe_node(tag: &str) -> CapabilityNode {
        let n = SEQ.fetch_add(1, Ordering::SeqCst);
        CapabilityNode::new_primitive(
            format!("probe::{tag}::{n}"),
            Domain::Core,
            vec![format!("probe.capability.{tag}")],
        )
    }

    ///  接线本身能被执行 —— 这正是「运行时不消费它」那个缺陷的**回归**。
    #[test]
    fn 节点可被运行期登记且计数增长() {
        serial(|| {
            let before = node_count();
            register_node(probe_node("wire")).expect("register");
            assert_eq!(node_count(), before + 1, "登记后节点数必须 +1");
        });
    }

    ///  **幂等语义**：同 id 重复登记**不报错、也不使节点数增长**。
    ///
    ///  这条测试的由来：我第一版按「错误串含 `already exists` ⇒ 幂等成功」
    /// 写特判，而**真实错误变体是 `RegistryError::AlreadyExists`**、
    /// 错误文案是中文 ⇒ 字符串匹配永不命中 = 死代码，被测试当场抓到。
    #[test]
    fn 同id重复登记幂等成功且计数不变() {
        serial(|| {
            let node = probe_node("idem2");
            register_node(node.clone()).expect("first");
            let after = node_count();
            register_node(node).expect("重复登记应幂等成功");
            assert_eq!(node_count(), after, "重复登记不应使节点数增长");
        });
    }

    ///  新增节点不得引入成熟度虚标 —— 判据的后半段。
    #[test]
    fn 新增节点不引入成熟度虚标() {
        serial(|| {
            register_node(probe_node("maturity")).expect("register");
            assert!(
                maturity_findings().is_empty(),
                "新增节点引入了虚标：{:?}",
                maturity_findings()
            );
        });
    }

    ///  涌现指纹必须**可复现** —— 同一状态两次取值必须相等。
    ///
    ///  这是 `capability_digest` 存在的全部理由：若它不可复现，
    /// 「比指纹判涌现」就退化成「比噪声」，比没有指纹更坏。
    ///
    /// ⛔ 注册表是**进程全局**且 `cargo test` **并行跑** ⇒ 必须持 `SERIAL`
    /// 才能对「两次取值相等」下断言（否则别的测试中途 register 会打破它）。
    #[test]
    fn 涌现指纹可复现() {
        serial(|| {
            let a = capability_digest().expect("digest");
            let b = capability_digest().expect("digest");
            assert_eq!(a, b, "同一状态下涌现指纹必须可复现");
        });
    }

    ///  指纹必须对**节点集变化**敏感 —— 否则它测不出涌现。
    ///
    /// ⚠️ 这里断言的是**敏感**（指纹变了），不是「等于某个常量」：
    /// 注册表是进程全局的，具体数值依赖其它测试是否已 register ⇒ 钉常量必假失败。
    /// golden 常量对「全局可变状态」本就不适用（`ra2.exe` 全仓也没有）。
    #[test]
    fn 涌现指纹对新增节点敏感() {
        serial(|| {
            let before = capability_digest().expect("digest");
            register_node(probe_node("digest-sensitive")).expect("register");
            let after = capability_digest().expect("digest");
            assert_ne!(
                before, after,
                "新增能力节点后指纹必须变化，否则测不出涌现"
            );
        });
    }
}

#[cfg(test)]
mod read_side_tests {
    use super::*;

    ///  **「注册了但没调用」必须可测** ——  这是本轮补读侧的核心目的。
    ///
    ///  三家对标仓库（`os-taxonomy` / `lcu` / `backburner`）
    /// **全部没有**这个度量：它们只能证明「声明 == 现实」，
    ///  **但「注册了」与「被用了」是两个不同的事实。**
    #[test]
    fn 注册后未被lookup的能力会出现在never_invoked清单里() {
        let id = "test::read_side::never_invoked_probe";
        let node = CapabilityNode::new_primitive(
            id.to_owned(),
            nt_core_capability_tree::node::Domain::Mind,
            vec!["test.read_side".to_owned()],
        );
        //  幂等登记（ 重复跑不会因 AlreadyExists 而红）
        let _ = register_node(node.clone());

        //  此刻它**已注册、零调用** ⇒  必须在清单里
        let never = registered_never_invoked().expect("锁");
        assert!(never.contains(&id.to_owned()), " 已注册未调用的能力应被列出");

        //  **读侧一次** ⇒  立刻从清单里消失（ 这是「可被调用」的证明）
        let got = lookup(id).expect("锁").expect("应能取到");
        assert_eq!(got.id, id, " lookup 必须返回该节点");
        assert_eq!(invoke_count(id), 1, " 取到即计数");
        let never2 = registered_never_invoked().expect("锁");
        assert!(
            !never2.contains(&id.to_owned()),
            " 被取到过 ⇒  **不再是**「注册了但没调用」"
        );
    }

    ///  **未登记的 id  不许被计数** ——  否则「查了 100 次
    /// 不存在的 id」会被误读成「有 100 次真实调用」。
    #[test]
    fn 未登记的id_lookup返回None且不计数() {
        let ghost = "test::read_side::ghost";
        assert!(lookup(ghost).expect("锁").is_none(), " 未登记 ⇒ None（⛔ 不是 Err）");
        assert_eq!(invoke_count(ghost), 0, " 未登记  **不得**计数");
    }

    ///  计数必须**单调**：重复 lookup ⇒ 计数累加（ 不是置位）。
    #[test]
    fn 计数单调累加() {
        let id = "test::read_side::monotonic";
        let node = CapabilityNode::new_primitive(
            id.to_owned(),
            nt_core_capability_tree::node::Domain::Mind,
            vec!["test.read_side".to_owned()],
        );
        let _ = register_node(node);
        for expect in 1..=3 {
            let _ = lookup(id).expect("锁").expect("已登记");
            assert_eq!(invoke_count(id), expect, " 计数应单调累加到 {expect}");
        }
    }

    /// **dispatch 是纯解析、不计数；record_dispatch 才是派发记账** ——
    /// OPEN-DEFECTS #4 回归：解析即计数会让「parsed but never executed」
    /// 污染 `registered_never_invoked()`。
    #[test]
    fn dispatch只解析不计数_record_dispatch才计数() {
        let id = "test::read_side::dispatch_no_count";
        let node = CapabilityNode::new_primitive(
            id.to_owned(),
            nt_core_capability_tree::node::Domain::Mind,
            vec!["test.read_side.dispatch".to_owned()],
        );
        let _ = register_node(node);
        let tag = "test.read_side.dispatch";

        let before = invoke_count(id);
        let resolved = dispatch_by_capability(tag).expect("锁");
        assert!(
            resolved.iter().any(|n| n.id == id),
            "dispatch 必须解析出该节点"
        );
        assert_eq!(
            invoke_count(id),
            before,
            "dispatch 是纯解析 ⇒ 计数不得变"
        );

        let after = record_dispatch(id).expect("锁");
        assert_eq!(after, before + 1, "record_dispatch 必须计数 +1");
        assert_eq!(invoke_count(id), before + 1);
    }
}

/// **从共享清单惰性播种**（2026-10-06，方案 A）。
///
/// ## 为什么需要它
///
/// 播种的真身在 `neotrix-core`（`bootstrap_trade_capabilities`，且
/// `apply_market_meta` 会写 `market.*` 键），而 `tool_schemas` 在本 crate。
/// 依赖方向固定 `core → neobot` ⇒ **工作区里没有任何 crate 同时依赖两侧**
/// ⇒ 服务进程里注册表恒空 ⇒ `capability_invoke` **永不上桌**。
///
/// ## 它做什么 / 不做什么
///
/// - ✅ 把**「这个能力是什么」**（`id` / `category` / `description`）播种进注册表 ——
///   这些是 `nt_core_capability_tree::market::TRADE_MANIFEST` 里的**纯数据**；
/// - ✅ 补 `version`（本 crate 的 `env!("CARGO_PKG_VERSION")`）与
///   `license`（共享串）—— `is_listable()` 要求二者齐备；
/// - ⛔ **不**在这里注册可执行实现（那在 core）⇒ 本函数**只让能力「可见」**，
///   调用仍会 fail-closed（`CAPABILITY_BODY_NOT_EXECUTED`）。
///
/// ## 为什么 version 可以取本 crate 的
///
/// 两个 crate 都是 `version.workspace = true` ⇒ 当前同为 `0.23.0`，
/// 由 `scripts/ops/check_version_sync.py` 守护；一旦分叉门会红。
/// ⚠️ 这**不是**「谁离实现近谁报」——而是两侧共享同一个 workspace 版本号，
/// 故本 crate 的 `env!` 与 core 的 `env!` **解析结果相同**。
///
/// **幂等**：已存在的 id 直接跳过，可反复调用。
pub fn seed_from_market_manifest() -> Result<usize, String> {
    use nt_core_capability_tree::market::{keys, TRADE_LICENSE, TRADE_MANIFEST};
    use nt_core_capability_tree::node::{CapabilityKind, CapabilityNode};
    let ver = env!("CARGO_PKG_VERSION").to_owned();
    let mut added = 0usize;
    for e in TRADE_MANIFEST {
        with_registry(|reg| {
            if reg.nodes.contains_key(e.id) {
                return;
            }
            let mut meta = std::collections::HashMap::new();
            meta.insert(
                keys::VERSION.to_owned(),
                serde_json::Value::String(ver.clone()),
            );
            meta.insert(
                keys::LICENSE.to_owned(),
                serde_json::Value::String(TRADE_LICENSE.to_owned()),
            );
            meta.insert(
                keys::CATEGORY.to_owned(),
                serde_json::Value::String(e.category.to_owned()),
            );
            meta.insert(
                keys::DESCRIPTION.to_owned(),
                serde_json::Value::String(e.description.to_owned()),
            );
            let mut node =
                CapabilityNode::new_primitive(e.id.to_owned(), e.domain, Vec::new());
            node.kind = CapabilityKind::Skill;
            node.metadata = meta;
            reg.nodes.insert(e.id.to_owned(), node);
            added += 1;
        })?;
    }
    Ok(added)
}
