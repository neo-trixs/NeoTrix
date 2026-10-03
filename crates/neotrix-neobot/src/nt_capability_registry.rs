//! ⭐⭐ **运行期能力注册表** —— 让「能力树」从**被测量的对象**变成**被使用的对象**。
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
//! ⭐ 可验证性**不靠新造度量**，而靠**已存在**的 `maturity_audit()` —— 它已经拒绝
//! 「声称成熟度超过证据支撑」的节点。
//!
//! ## ⭐ 为什么放在 `crates/neotrix-neobot/`（层归属裁决）
//!
//! 候选 A：`neotrix-core/src/l1_action/` —— ⛔ 会被 L2–L6 依赖，
//! 且 `check-layer-deps.sh` 的 `SRC` **就是** `neotrix-core/src` ⇒ 选错会被门拦。
//!
//! 候选 B（**采纳**）：`crates/neotrix-neobot/` ——
//! ⭐ `check-layer-deps` 只扫 `neotrix-core/src` ⇒ 本 crate 不受它管；
//! ⭐⭐ **且 `neotrix-core/Cargo.toml:104` 依赖 `neotrix-neobot`**
//! ⇒ 意识侧的生产者（`neotrix-core/src/l4_emotion/nt_feel_facade.rs`，
//! 实测已有生产消费者）**可以直接调用本模块**，**零层违规**。
//!
//! ⛔ 刻意**不重造** maturity 词汇表 / 节点判据 —— 一律用 `nt-core-capability-tree`
//! 的真类型。理由：重造会产生第二套判据，而那正是「反虚标」要防的东西。

use std::sync::{Mutex, OnceLock};

use nt_core_capability_tree::node::CapabilityNode;
use nt_core_capability_tree::registry::{CapabilityTreeRegistry, RegistryError};

/// 进程级注册表。
///
/// ⭐ 用 `OnceLock<Mutex<…>>` 而非 `static mut` / `lazy_static`：
/// - `OnceLock` 是 std（MSRV 1.70+），本仓已在 `nt_echo` 等处用同一形态；
/// - ⭐ **不引任何新依赖**（`once_cell` / `lazy_static` 在本 crate 里零使用）。
static REGISTRY: OnceLock<Mutex<CapabilityTreeRegistry>> = OnceLock::new();

fn slot() -> &'static Mutex<CapabilityTreeRegistry> {
    REGISTRY.get_or_init(|| Mutex::new(CapabilityTreeRegistry::default()))
}

/// ⭐ 把一个能力节点登记进运行期能力树。
///
/// ## 语义
/// - ⭐ **同 id 重复登记 = 幂等成功**（底层 `register()` 返回 `AlreadyExists`），
///   且 `node_count()` **不变** ⇒ 「同一能力被多处发现」天然收敛。
/// - 结构性失败 ⇒ 返回 `Err`（如锁投毒），由调用方决定是否上报。
///   ⛔ 刻意**不**在这里 `log::warn!` 后返回 `Ok` —— 那会让「登记失败」
///   与「登记成功」不可区分，正是 `check-silent-failure` 要抓的形态。
///
/// # Errors
/// `CapabilityTreeRegistry::register` 的错误（重复以外的结构性冲突）。
pub fn register_node(node: CapabilityNode) -> Result<(), String> {
    let mut reg = slot().lock().map_err(|e| format!("能力注册表锁投毒: {e}"))?;
    // ⭐⭐ `register()` 对同 id **直接返回 `AlreadyExists`**（实测 `registry.rs:169-171`）。
    //    ⓰ 我一度以为它是 upsert —— 那是 **`merge_overlay`** 的行为（`:552-556` 就地更新），
    //    两者职责不同。⇒ 「同一能力被多处发现」必须在此**显式**收敛成幂等成功。
    // ⭐ 按**错误变体**匹配（`RegistryError::AlreadyExists`），⛔ 不用字符串 ——
    //   字符串匹配会在文案改动时静默失效（我第一版就踩了：写的是英文
    //   `already exists`，而真实文案是中文，且我连变体名都没核实）。
    match reg.register(node) {
        Ok(()) => Ok(()),
        Err(RegistryError::AlreadyExists(_)) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// ⭐⭐ 把运行期注册表**借给**一个需要 `&mut CapabilityTreeRegistry` 的生产者。
///
/// ## 为什么要这个入口（2026-10-03 实测）
/// 仓里现成的节点生产者签名统一是：
/// ```ignore
/// pub fn register_xxx_capability(registry: &mut CapabilityTreeRegistry) -> CapabilityNode
/// ```
/// ⇒ 它们**只认 `&mut`**，⛔ 无法通过 `register_node(node)` 那个「传成品」的入口。
/// ⭐ 若为此给每个生产者改签名，就是**为接线改 8 处生产代码**；
/// 而本函数让它们**原样调用**，接线成本降为一个 `with_registry`。
///
/// ⭐⭐ **为什么这个入口必须诚实地暴露 `&mut`**：调用方能拿到完整注册表，
/// 也就**能绕过本模块的 `AlreadyExists` 幂等语义**（`register_node` 会把
/// `AlreadyExists` 吞成 `Ok`，而这里不会）。
/// ⇒ ⭐ 因此 `bootstrap` 类调用方**必须自己断言结果**，
/// ⛔ 否则拓扑序写错会**静默失败**（详见 `bootstrap_trade_capabilities`）。
///
/// # Errors
/// 注册表锁投毒。
pub fn with_registry<R>(f: impl FnOnce(&mut CapabilityTreeRegistry) -> R) -> Result<R, String> {
    let mut reg = slot().lock().map_err(|e| format!("能力注册表锁投毒: {e}"))?;
    Ok(f(&mut reg))
}

/// 当前已登记的节点数 —— ⭐ **涌现门要盯的就是这个数**（见路线 §5 第 2 步）。
///
/// ⭐ 「节点数不增长即判红」是**反向护栏**：钉住涌现必须是**行为**，
/// 否则「接线完成」会退化成「一次性注册固定几个节点」。
pub fn node_count() -> usize {
    slot().lock().map(|r| r.nodes.len()).unwrap_or(0)
}

/// ⭐ 跑一次成熟度审计 —— 返回**虚标节点**列表（空 = 无虚标）。
///
/// ⭐ 这是判据的另一半：新增节点必须**过这一关**，否则「涌现」退化为刷节点。
pub fn maturity_findings() -> Vec<String> {
    match slot().lock() {
        Ok(reg) => reg.maturity_audit().into_iter().map(|f| f.id).collect(),
        Err(_) => vec!["<lock poisoned>".to_owned()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nt_core_capability_tree::node::Domain;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static SEQ: AtomicUsize = AtomicUsize::new(0);
    /// ⭐⭐ 注册表是**进程全局**的（`OnceLock<Mutex<…>>`），而 `cargo test` **并行跑**
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

    /// ⭐ 接线本身能被执行 —— 这正是「运行时不消费它」那个缺陷的**回归**。
    #[test]
    fn 节点可被运行期登记且计数增长() {
        serial(|| {
            let before = node_count();
            register_node(probe_node("wire")).expect("register");
            assert_eq!(node_count(), before + 1, "登记后节点数必须 +1");
        });
    }

    /// ⭐ **幂等语义**：同 id 重复登记**不报错、也不使节点数增长**。
    ///
    /// ⭐ 这条测试的由来：我第一版按「错误串含 `already exists` ⇒ 幂等成功」
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

    /// ⭐ 新增节点不得引入成熟度虚标 —— 判据的后半段。
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
}
