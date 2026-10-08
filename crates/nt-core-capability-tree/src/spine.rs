//! **能力脊柱（Capability Spine）** —— 运行时能力的**唯一真源**。
//!
//! # 为什么要有它（2026-10-08，审计 13 套能力体系后的结论）
//!
//! 取证结论：仓里有 **13 套**并行的能力表示法、**13 个**能力注册表，
//! 而**只有 `dispatch::Table` 一条**有活的 register→list→invoke 回路。
//! 其余（`CapabilityRegistry`、`TradeCapabilityRegistry`、`CapabilityCatalog`、
//! `NativeBus`、`PipelineRegistry`…）要么零生产消费者，要么建了空表从不派发。
//!
//! 更致命的是**元数据与执行面互相不知道对方存在**：
//! - 清单说 `genoffice` 是 `Executable`（`market.rs`），但那是**编译期常量**；
//! - 派发表里有没有它，取决于**本进程**有没有跑过注册；
//! - 两者之间**没有任何派生关系** ⇒ 于是「已上架但调不动」成为常态，
//!   而唯一能发现这件事的机制是一道**用正则刮源码文本**的 bash 门。
//!
//! # 脊柱解决的四件事
//!
//! 1. **一个真源**：`id → 执行器 + 描述 + 健康 + 计数`。派发与上架**读同一份**。
//! 2. **执行器可带状态**：`Arc<dyn CapabilityExecutor>` 而非无捕获 `fn` 指针
//!    —— 后者**带不了**插件必需的 config / 连接句柄（这是热插拔的第一道硬墙）。
//! 3. **可拔可换**：`unregister` / `replace`（事务化，见 [`Spine::replace`]），
//!    并按 `drain_timeout` 等待在途调用结束，避免「拔掉正在跑的能力」。
//! 4. **`executability` 派生**：不再是手写声明，而是由
//!    「登记了? 健康? 清单声称?」**算出来**（见 [`Spine::executability`]）。
//!
//! # 与既有 `dispatch` 端口的关系（不重复造轮子）
//!
//! `dispatch::Table` 是**已接线**的生产通路；本模块是它的**超集**：
//! - [`SpineExecutor`] 把既有 `DispatchFn`（无捕获 `fn`）适配成
//!   `Arc<dyn CapabilityExecutor>` ⇒ **存量 3 个执行器无需改写即可入脊柱**；
//! - 新插件（需要状态）直接实现 `CapabilityExecutor`，不进 `dispatch` 表。
//! - 两侧都能依赖本 crate ⇒ 宿主（core/晶体）与薄客户端（neobot）读同一份。

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::dispatch::BoxFuture;
use crate::market::Executability;

// ─── 描述 ────────────────────────────────────────────────────────────────

/// 能力自述（**市场面与执行面共用的同一份描述**）。
///
/// ⛔ 刻意**不再**各系统各带一份：`CapabilityMeta`（L0）/ `ManifestEntry`（清单）/
/// `MarketEntry`（neobot 投影）/ `Capability`（L5 catalog）曾各带
/// id/version/license/description/tags ⇒ 键名漂移不会报错，只会静默读不出。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityDescriptor {
    pub id: String,
    /// 人读描述
    pub description: String,
    /// 市场分组
    pub category: String,
    /// 实现版本（插件自己的版本，**不是** cargo 包版本）
    #[serde(default)]
    pub version: String,
    /// 本仓对该实现的许可声明
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub tags: Vec<String>,
    /// **真实**入参 schema（驱动模型面生成 tool；`None` = 不声明）
    #[serde(default)]
    pub input_schema: Option<Value>,
    #[serde(default)]
    pub output_schema: Option<Value>,
    /// 期望超时（毫秒）。`None` = 由宿主决定。
    ///
    /// ⚠️ 这里**只是声明**；真正强制在执行路径上（历史教训：
    /// `enforced_timeout_ms` 曾是只喂日志的影子值 ⇒ 卡死能力挂住整轮）。
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

impl CapabilityDescriptor {
    /// 最小构造（只需 id + 描述）。
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            category: String::new(),
            version: String::new(),
            license: String::new(),
            tags: Vec::new(),
            input_schema: None,
            output_schema: None,
            timeout_ms: None,
        }
    }

    pub fn with_category(mut self, c: impl Into<String>) -> Self {
        self.category = c.into();
        self
    }
    pub fn with_version(mut self, v: impl Into<String>) -> Self {
        self.version = v.into();
        self
    }
    pub fn with_license(mut self, l: impl Into<String>) -> Self {
        self.license = l.into();
        self
    }
    pub fn with_tags(mut self, tags: &[&str]) -> Self {
        self.tags = tags.iter().map(|s| (*s).to_owned()).collect();
        self
    }
    pub fn with_input_schema(mut self, s: Value) -> Self {
        self.input_schema = Some(s);
        self
    }
}

// ─── 执行器 ──────────────────────────────────────────────────────────────

/// 能力执行器 —— **可带状态**（这是与 `fn` 指针的本质差别）。
pub trait CapabilityExecutor: Send + Sync {
    fn descriptor(&self) -> CapabilityDescriptor;
    /// 执行。入参/出错都由宿主记录，**不得**自己吞掉错误。
    fn execute(&self, input: Value, session: &str) -> BoxFuture<'static, Result<Value, String>>;
    /// 健康自述（热拔插决策用）。默认健康。
    fn health(&self) -> CapabilityHealth {
        CapabilityHealth::Healthy
    }
    /// **能否降级成无捕获 `fn` 指针**放进 `dispatch::Table`。
    ///
    /// 默认 `None` = **不能**（带状态的插件就属于这种）⇒ `sync_spine_into_dispatch`
    /// 会跳过并**如实报出**，绝不把带状态执行器悄悄塞进无捕获表（那会丢状态）。
    /// 只有 [`SpineExecutor`]（本就持有 `DispatchFn` 的适配器）才返回 `Some`。
    fn as_fn_ptr(&self) -> Option<crate::dispatch::DispatchFn> {
        None
    }
}

/// 执行器健康态。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityHealth {
    Healthy,
    /// 已知故障（仍登记，但拔插/上架应据此判断）
    Unhealthy(String),
    /// 拒绝服务（主动下线）
    Disabled,
}

impl CapabilityHealth {
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }
}

/// 把既有 `DispatchFn`（无捕获 `fn`）适配成 `Arc<dyn CapabilityExecutor>`。
///
/// 用途：让**存量**执行器（`foreign_trade_full_cycle` / `trade_quote_negotiation` /
/// `genoffice`）无需改写即可入脊柱 ⇒ 脊柱可以渐进接管，而不是一次性重写。
pub struct SpineExecutor {
    desc: CapabilityDescriptor,
    f: crate::dispatch::DispatchFn,
}

impl SpineExecutor {
    pub fn new(desc: CapabilityDescriptor, f: crate::dispatch::DispatchFn) -> Self {
        Self { desc, f }
    }
}

impl CapabilityExecutor for SpineExecutor {
    fn descriptor(&self) -> CapabilityDescriptor {
        self.desc.clone()
    }
    fn execute(&self, input: Value, session: &str) -> BoxFuture<'static, Result<Value, String>> {
        (self.f)(&self.desc.id, input, session)
    }
    fn as_fn_ptr(&self) -> Option<crate::dispatch::DispatchFn> {
        Some(self.f)
    }
}

// ─── 错误 ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpineError {
    /// id 已被占用（`register` 语义：不覆盖）
    AlreadyExists(String),
    /// id 不存在
    NotFound(String),
    /// 执行器自述的 id 与注册用的 id 不一致 ⇒ 拒绝（否则打点/计数会记到错的 id）
    IdMismatch { registered: String, declared: String },
    /// 锁投毒
    Poisoned,
}

impl std::fmt::Display for SpineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyExists(id) => write!(f, "能力已存在: {id}（register 不覆盖，请用 replace）"),
            Self::NotFound(id) => write!(f, "能力不存在: {id}"),
            Self::IdMismatch { registered, declared } => write!(
                f,
                "执行器自述 id 与注册 id 不一致: 注册={registered} 自述={declared}"
            ),
            Self::Poisoned => write!(f, "能力脊柱锁投毒"),
        }
    }
}

impl std::error::Error for SpineError {}

// ─── 脊柱 ────────────────────────────────────────────────────────────────

/// 一个已登记的能力（执行器 + 自述 + 计数）。
struct Entry {
    exec: Arc<dyn CapabilityExecutor>,
    /// 在途调用计数（`unregister` 的排空判据）
    inflight: Arc<()>,
}

/// 运行时能力真源。
///
/// ⛔ 本类型**不加运行时依赖**（与本 crate 其余部分一致），异步靠
/// [`crate::dispatch::BoxFuture`]（纯 `std`）。
pub struct Spine {
    entries: BTreeMap<String, Entry>,
    /// 每个 id 的在途计数（`Arc<()>` 的 `strong_count` 即在途数 + 1）
    inflight: BTreeMap<String, Vec<Arc<()>>>,
}

impl Default for Spine {
    fn default() -> Self {
        Self::new()
    }
}

impl Spine {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            inflight: BTreeMap::new(),
        }
    }

    /// **登记**（不覆盖已存在的 id ⇒ 要覆盖请用 [`Spine::replace`]）。
    pub fn register(&mut self, exec: Arc<dyn CapabilityExecutor>) -> Result<(), SpineError> {
        let id = exec.descriptor().id;
        if id.is_empty() {
            return Err(SpineError::IdMismatch {
                registered: "<empty>".to_owned(),
                declared: id,
            });
        }
        if self.entries.contains_key(&id) {
            return Err(SpineError::AlreadyExists(id));
        }
        self.entries.insert(
            id.clone(),
            Entry {
                exec,
                inflight: Arc::new(()),
            },
        );
        Ok(())
    }

    /// **拔掉**一个能力。返回被拔掉的执行器（调用方若要回滚可再 `register`）。
    ///
    /// `drain` = 最多等待在途调用结束多久（毫秒）。`0` = 不等（立即拔）。
    ///
    /// ⚠️ **在途调用不会被杀掉**（它们各自持有 `Arc`）⇒ 拔除只保证
    /// 「不再有新调用进来」，已在跑的会跑完。等 `drain` 是给调用方一个
    /// 「此刻大致没人正在用它」的信号。
    pub fn unregister(&mut self, id: &str, drain_ms: u64) -> Result<Arc<dyn CapabilityExecutor>, SpineError> {
        let entry = self
            .entries
            .remove(id)
            .ok_or_else(|| SpineError::NotFound(id.to_owned()))?;
        self.wait_drained(id, drain_ms);
        self.inflight.remove(id);
        Ok(entry.exec)
    }

    /// **事务化替换**：装上新的、旧的取回，失败则**完全不变**。
    ///
    /// 「失败则完全不变」的保证来自：先在旁路构造好新条目，再做**单次**
    /// `BTreeMap::insert`（不可能中途失败）⇒ 不像
    /// `CapabilityTreeRegistry::remove` 那样「先删后校验」留下损坏状态
    /// （那是 2026-10-08 修掉的缺陷）。
    pub fn replace(
        &mut self,
        exec: Arc<dyn CapabilityExecutor>,
    ) -> Result<Option<Arc<dyn CapabilityExecutor>>, SpineError> {
        let id = exec.descriptor().id;
        let old = self
            .entries
            .insert(
                id.clone(),
                Entry {
                    exec,
                    inflight: Arc::new(()),
                },
            )
            .map(|e| e.exec);
        Ok(old)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.entries.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn ids(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }

    pub fn get(&self, id: &str) -> Option<Arc<dyn CapabilityExecutor>> {
        self.entries.get(id).map(|e| Arc::clone(&e.exec))
    }

    pub fn descriptor(&self, id: &str) -> Option<CapabilityDescriptor> {
        self.entries.get(id).map(|e| e.exec.descriptor())
    }

    pub fn descriptors(&self) -> Vec<CapabilityDescriptor> {
        self.entries.values().map(|e| e.exec.descriptor()).collect()
    }

    pub fn health(&self, id: &str) -> Option<CapabilityHealth> {
        self.entries.get(id).map(|e| e.exec.health())
    }

    /// 在途调用数（用于观测/排空判据）。
    pub fn inflight_count(&self, id: &str) -> usize {
        self.inflight.get(id).map(|v| v.len()).unwrap_or(0)
    }

    /// **`executability` 派生** —— 不再是清单里的手写声明。
    ///
    /// | 情形 | 派生结果 |
    /// |---|---|
    /// | 未登记 | `DeclaredOnly`（清单声称可执行但本进程没有 ⇒ 就是声明而已） |
    /// | 已登记但 `health()` 非健康 | `DeclaredOnly`（挂了的不算能跑） |
    /// | 已登记且健康，但清单声称 `Scaffold` | `Scaffold`（**尊重更悲观的声明**） |
    /// | 已登记且健康，清单也未降级 | `Executable` |
    ///
    /// ⛔ 不设 `Unhealthy` 之外的新枚举值：`Executability` 是市场契约
    /// （bash 门 `check-executor-registry.sh` 按它对账），派生只在其既有
    /// 三个值之间做选择。
    pub fn executability(&self, id: &str) -> Executability {
        let Some(entry) = self.entries.get(id) else {
            return Executability::DeclaredOnly;
        };
        if !entry.exec.health().is_healthy() {
            return Executability::DeclaredOnly;
        }
        match crate::market::manifest_entry(id).map(|e| e.executability) {
            Some(Executability::Scaffold) => Executability::Scaffold,
            _ => Executability::Executable,
        }
    }

    /// 真正调得动 + 可上架的 id（模型面与 CLI 共用这个口径）。
    pub fn dispatchable_ids(&self) -> Vec<String> {
        self.entries
            .keys()
            .filter(|id| self.executability(id) == Executability::Executable)
            .cloned()
            .collect()
    }

    fn wait_drained(&self, id: &str, drain_ms: u64) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(drain_ms);
        while std::time::Instant::now() < deadline {
            if self.inflight_count(id) == 0 {
                return;
            }
            std::thread::yield_now();
        }
    }
}

/// **全局脊柱**（宿主与薄客户端读同一份）。
///
/// ⚠️ 这是**有意**的进程级单例：能力宿主必须让同进程的所有消费者
/// （neobot 的上架面、晶体的派发面）看到**同一份**状态 —— 否则又变成
/// 「两套能力宇宙」（2026-10-08 审计的头号发现：磁盘 326 节点树与
/// 6 个清单 id 互不相干，只在某个进程全局里偶然相遇）。
pub fn global_spine() -> &'static Mutex<Spine> {
    static S: OnceLock<Mutex<Spine>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(Spine::new()))
}

/// 便捷入口：在脊柱上操作（锁投毒返回 `Err`）。
pub fn with_spine<R>(f: impl FnOnce(&mut Spine) -> R) -> Result<R, SpineError> {
    let mut g = global_spine().lock().map_err(|_| SpineError::Poisoned)?;
    Ok(f(&mut g))
}

// ─── 测试 ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn exec_ok(id: &'static str) -> Arc<dyn CapabilityExecutor> {
        Arc::new(SpineExecutor::new(
            CapabilityDescriptor::new(id, format!("{id} 的测试实现")),
            |_id: &str, input: Value, _s: &str| {
                Box::pin(async move { Ok(input) })
            },
        ))
    }

    #[test]
    fn register_then_get() {
        let mut sp = Spine::new();
        sp.register(exec_ok("a::b::c")).expect("登记应成功");
        assert!(sp.contains("a::b::c"));
        assert_eq!(sp.len(), 1);
        assert_eq!(sp.get("a::b::c").map(|e| e.descriptor().id), Some("a::b::c".to_owned()));
    }

    #[test]
    fn register不覆盖已存在() {
        let mut sp = Spine::new();
        sp.register(exec_ok("dup")).expect("首次应成功");
        let e = sp.register(exec_ok("dup")).expect_err("重复登记必须被拒");
        assert_eq!(e, SpineError::AlreadyExists("dup".to_owned()));
        assert_eq!(sp.len(), 1, "被拒的登记不得改变状态");
    }

    /// 热拔插的核心保证：拔掉后再拔要报错，且**状态不留残**。
    #[test]
    fn unregister移除并可再换回() {
        let mut sp = Spine::new();
        sp.register(exec_ok("plug")).unwrap();
        let back = sp.unregister("plug", 0).expect("应拔掉");
        assert_eq!(back.descriptor().id, "plug");
        assert!(!sp.contains("plug"), "拔掉后不得仍在册");
        match sp.unregister("plug", 0) {
            Err(SpineError::NotFound(i)) => assert_eq!(i, "plug"),
            Err(other) => panic!("应为 NotFound，实得 {other:?}"),
            Ok(_) => panic!("重复拔除必须失败"),
        }
        // 换回（模拟回滚 / 重新装载）
        sp.register(back).expect("换回应成功");
        assert!(sp.contains("plug"));
    }

    /// 事务化替换：旧的可取回，且中途不可能留下半状态。
    #[test]
    fn replace取回旧的() {
        let mut sp = Spine::new();
        sp.register(exec_ok("slot")).unwrap();
        let old = sp.replace(exec_ok("slot")).expect("replace 应成功");
        assert!(old.is_some(), "替换已有项必须取回旧的");
        assert_eq!(old.unwrap().descriptor().id, "slot");
        assert_eq!(sp.len(), 1, "替换不得让条目数变化");
        // 替换不存在的 id = 直接登记
        let old2 = sp.replace(exec_ok("fresh")).expect("替换新 id 应成功");
        assert!(old2.is_none(), "新 id 不该有旧值");
        assert!(sp.contains("fresh"));
    }

    /// **`executability` 是派生的**：未登记 ⇒ 哪怕清单说它是 Executable，
    /// 派生结果也必须是 `DeclaredOnly`（这正是「已上架却调不动」的修复点）。
    #[test]
    fn executability由登记事实派生() {
        let mut sp = Spine::new();
        // genoffice 在清单里声明为 Executable
        let id = "NT-ACT::nt_file_ability::genoffice";
        assert_eq!(
            crate::market::manifest_entry(id).map(|e| e.executability),
            Some(Executability::Executable),
            "前置：清单声明 genoffice 为 Executable"
        );
        assert_eq!(
            sp.executability(id),
            Executability::DeclaredOnly,
            "未登记时不得声称可执行"
        );
        sp.register(exec_ok(id)).unwrap();
        assert_eq!(
            sp.executability(id),
            Executability::Executable,
            "登记且健康后才是 Executable"
        );
        assert!(sp.dispatchable_ids().contains(&id.to_owned()));
    }

    /// 尊重清单里更悲观的声明（脚手架不该因被登记就变成「可执行」）。
    #[test]
    fn executability尊重清单的Scaffold声明() {
        let mut sp = Spine::new();
        let id = "NT-MIND::trade::foreign_trade_full_cycle";
        assert_eq!(
            crate::market::manifest_entry(id).map(|e| e.executability),
            Some(Executability::Scaffold),
            "前置：清单把它标为 Scaffold"
        );
        sp.register(exec_ok(id)).unwrap();
        assert_eq!(
            sp.executability(id),
            Executability::Scaffold,
            "脚手架被登记后仍须是 Scaffold（不得升格为 Executable）"
        );
        assert!(
            !sp.dispatchable_ids().contains(&id.to_owned()),
            "Scaffold 不得进入可调度集合"
        );
    }

    /// 不健康 ⇒ 派生降级（拔插/上架据此判断）。
    #[test]
    fn 不健康则降级为DeclaredOnly() {
        struct Bad;
        impl CapabilityExecutor for Bad {
            fn descriptor(&self) -> CapabilityDescriptor {
                CapabilityDescriptor::new("NT-ACT::nt_file_ability::genoffice", "坏掉的")
            }
            fn execute(&self, _i: Value, _s: &str) -> BoxFuture<'static, Result<Value, String>> {
                Box::pin(async { Ok(Value::Null) })
            }
            fn health(&self) -> CapabilityHealth {
                CapabilityHealth::Unhealthy("boom".to_owned())
            }
        }
        let mut sp = Spine::new();
        sp.register(Arc::new(Bad)).unwrap();
        assert_eq!(sp.executability("NT-ACT::nt_file_ability::genoffice"), Executability::DeclaredOnly);
    }

    /// 脊柱是**唯一真源**：派发与上架读同一份 ⇒ 不可能再分叉。
    #[test]
    fn 派发与上架读同一份() {
        let mut sp = Spine::new();
        sp.register(exec_ok("NT-ACT::nt_file_ability::genoffice")).unwrap();
        assert!(sp.contains("NT-ACT::nt_file_ability::genoffice"));
        assert!(sp.dispatchable_ids().contains(&"NT-ACT::nt_file_ability::genoffice".to_owned()));
        let d = sp.descriptor("NT-ACT::nt_file_ability::genoffice").expect("描述可得");
        assert!(!d.id.is_empty());
        assert!(matches!(sp.health("NT-ACT::nt_file_ability::genoffice"), Some(CapabilityHealth::Healthy)));
    }

    #[test]
    fn 全局脊柱可复用且幂等() {
        let id = "test::global::spine";
        with_spine(|sp| sp.register(exec_ok(id))).expect("首次登记");
        // 幂等：重复登记报 AlreadyExists，但脊柱仍可用
        match with_spine(|sp| sp.register(exec_ok(id))).expect("锁不应投毒") {
            Err(SpineError::AlreadyExists(i)) => assert_eq!(i, id),
            Err(other) => panic!("应为 AlreadyExists，实得 {other:?}"),
            Ok(()) => panic!("重复登记必须失败"),
        }
        with_spine(|sp| assert!(sp.contains(id)));
        // 清理，避免污染同进程其它用例
        with_spine(|sp| {
            sp.unregister(id, 0).expect("清理");
        });
    }
}