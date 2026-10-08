//! **能力插件市场** —— 2026-10-04
//!
//! # 对标目标（用户指定）
//! > 「贸易能力这些同类型，作为能力插件汇聚在一个能力插件市场里，供人调用，
//! >  对标产品 hermes 一切为插件能力」
//!
//! **hermes 的形态**（`hermes-desktop/src/main/registry.ts:46`）：
//! ```ts
//! type IndexEntry = {
//!   id: string;
//!   type: "agent" | "mcp" | "skill" | "workflow";   // 分类维度
//!   category?: string; name: string; version?: string;
//!   description?: string; tags?: string[]; author?: string | {...};
//!   license?: string;                               // 许可（商业阻断项）
//!   platforms?: string[]; path?: string; icon?: string;
//! }
//! ```
//!
//! # 三条不可违背的设计纪律（本轮一路吃过的教训）
//!
//! ## ① **零第二真源** —— 市场是**能力树的查询投影**，⛔ 不是第二份数据
//!
//! 本轮我刚刚花大力气消除「第三种状态」（`expected_but_unregistered`）：
//! 「注册表说自己有、金丝雀说自己没验」⇒ **两个真源各说各话，
//! 比「有门没跑」更难查**。
//! ⇒ **若市场再存一份清单，就是第三份** 
//! ⇒ 所以：**市场只提供 `&CapabilityTreeRegistry` 上的查询**，
//! **不持有任何数据**。删掉市场 = 删掉一个视图，不丢事实。
//!
//! ## ② **`Gap` 不是插件** —— 「我不会」⛔ 不能进「我有什么」
//!
//! `CapabilityKind::Gap` 是意识 `observe_from_critique` 登记的**缺口**
//! （`consciousness::gap::q*`）。市场若把它当「已安装能力」列出，
//! **语义反了** ⇒ `is_marketable()` 把它排除。
//!
//! ## ③ **许可缺失 ⇒ ⛔ 不是「未知」，是「不可上架」**
//!
//! hermes 的 `IndexEntry.license` 是**可选**（`license?: string`）。
//! **本仓不能这样**：插件市场 会引入**外部代码**，
//! 而本仓有**商业许可阻断门**（`check-license-js.sh` / `deny.toml`），
//! 但那 **只管构建期**，**运行期上架的插件许可无从审计**。
//! ⇒ `license` 为空 **不是 unknown**，**是「不可上架」**。
//!
//! # 与本仓既有设施的接线（都不是新概念）
//! - `nt_capability_registry::with_registry` ⇒ 拿**进程级真源**
//! - `nt_capability_registry::nodes_providing(tag)` ⇒ 按标签定位
//! - `nt_capability_canary::status(session)` ⇒ **「被调用过吗」** 市场要显示它

use nt_core_capability_tree::node::{CapabilityKind, CapabilityNode};
use nt_core_capability_tree::registry::CapabilityTreeRegistry;

/// **市场条目** —— **纯投影**（不含任何自有字段以外的数据）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketEntry {
    pub id: String,
    pub kind: CapabilityKind,
    pub domain: String,
    /// 分组用（hermes 的 `category`）
    pub category: String,
    /// 版本（hermes 的 `version`）⇒ **从 `metadata` 取**
    pub version: String,
    /// **许可**（hermes 的 `license`）
    /// **⚠️ 空串 意味着「不可上架」**（见纪律③），⛔ 不是 unknown。
    pub license: String,
    pub description: String,
    pub tags: Vec<String>,
    /// **市场成熟度等级**（`C0..C6`）⇒ 供人**判断敢不敢用**
    pub maturity: String,
    /// **被调用过几次**（来自金丝雀的**累计**计数）
    /// **0 是合法且常见的状态**（冷启动）。
    /// **⛔ 不可用它做「上架」判据** —— 
    /// 「调用过 0 次」≠「不该上架」；那是 **金丝雀的健康问题**，
    /// **不是市场的准入问题**。两者职责必须分开，
    /// 否则「没人调过」的能力会被市场**悄悄藏起来** ⇒ 那才是「看不见」。
    pub invoked: usize,
}

impl MarketEntry {
    /// **它能不能被市场列出**（唯一准入门）。
    ///
    /// 三条（与纪律一一对应）：
    /// ① `kind.is_marketable()` ⇒ `Gap` 排除（纪律②）
    /// ② `license` 非空（纪律③）⇒ **缺许可 不是 unknown，是不可上架**
    /// ③ `version` 非空（无版本 **无法判兼容性** ⇒ 同②）
    pub fn is_listable(&self) -> bool {
        self.kind.is_marketable() && !self.license.trim().is_empty() && !self.version.trim().is_empty()
    }

    /// 不可上架的原因（供人行动，⛔ 不只说「不行」）
    pub fn blocked_reason(&self) -> Option<String> {
        if !self.kind.is_marketable() {
            return Some(format!(
                "类型 '{}' 不是可安装能力（⭐⭐ 它是意识登记的**缺口**）",
                self.kind.as_str()
            ));
        }
        if self.license.trim().is_empty() {
            return Some("⛔ 缺 license ⇒ ⭐⭐ 本仓有商业许可阻断门，⭐⭐ 许可缺失**不是未知**而是不可上架".to_owned());
        }
        if self.version.trim().is_empty() {
            return Some("⛔ 缺 version ⇒ ⭐⭐ 无法判兼容性（⭐⭐ 与 license 同类：不可上架）".to_owned());
        }
        None
    }
}

/// 市场查询用的 `metadata` 键名（**命名契约**）。
pub mod meta_keys {
    /// ⚠️ **唯一定义处在 `nt_core_capability_tree::market::keys`**。
    ///
    /// 本模块此前**硬编码**这四个字符串，而清单已下沉到共享 crate
    /// ⇒ 两份键名会分叉。`metadata` 是 `HashMap<String, serde_json::Value>`
    /// ⇒ **键名拼错不会报错**，只会让值静默「读不出来」。
    /// ⇒ 故此处改为**引用**，键名只有一份。
    pub use nt_core_capability_tree::market::keys::{CATEGORY, DESCRIPTION, LICENSE, VERSION};
}

/// 把一个能力节点投影成市场条目（**纯函数**，可单测）。
///
/// **⚠️ 不在这里读金丝雀**：本函数签名是
/// `&CapabilityTreeRegistry`，而金丝雀是**进程级另一真源**；
/// 混进来会让本函数**不再可单测**（需要全局状态）。
/// ⇒ **`invoked` 由调用方注入**（见 `list_with_calls`）。
pub fn project(node: &CapabilityNode) -> MarketEntry {
    let m = &node.metadata;
    // `metadata` 是 `HashMap<String, serde_json::Value>`（**不是 String**）
    // ⇒ 取值必须 **显式 `as_str()`**，而 ⛔ 不能 `cloned()`。
    // 而且：`as_str()` 对「值不是字符串」返回 `None`
    // ⇒ 填错类型（`"1.0"` 写成数字）**会落到默认值**
    // ⇒ 然后被 `is_listable()` **当作「缺失」判红** ⇒
    // **填错类型不会被静默接受**，这是想要的行为。
    let meta_str = |key: &str| m.get(key).and_then(|v| v.as_str()).map(str::to_owned);
    MarketEntry {
        id: node.id.clone(),
        kind: node.kind,
        // 分组优先取 `metadata.market.category`，⛔ **不退回 domain** ——
        // domain 是**架构轴**（L0–L6），category 是**市场轴**，
        // 混用会让市场分组**随架构重构而变** ⇒ 用户看到的目录会无故重排。
        category: meta_str(meta_keys::CATEGORY)
            .unwrap_or_else(|| "uncategorized".to_owned()),
        domain: format!("{:?}", node.domain).to_lowercase(),
        version: meta_str(meta_keys::VERSION).unwrap_or_default(),
        license: meta_str(meta_keys::LICENSE).unwrap_or_default(),
        description: meta_str(meta_keys::DESCRIPTION).unwrap_or_default(),
        tags: node.provides.clone(),
        maturity: format!("{:?}", node.constellation),
        invoked: 0,
    }
}

/// **列出市场**（纯查询，不持有数据）。
///
/// 参数 `invoked_of` 是 「id → 累计调用次数」的注入接口
/// （生产传 `nt_capability_registry::invoke_count`，
/// 测试传闭包 ⇒ **无需全局状态即可测**）。
pub fn list_with_calls<F>(reg: &CapabilityTreeRegistry, invoked_of: F) -> Vec<MarketEntry>
where
    F: Fn(&str) -> usize,
{
    let mut out: Vec<MarketEntry> = reg
        .nodes
        .values()
        .map(|n| {
            let mut e = project(n);
            e.invoked = invoked_of(&e.id);
            e
        })
        .collect();
    // 排序 ⇒ 输出确定（本仓一贯的确定性纪律；也是门能稳定 diff 的前提）
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

/// **「可上架」视图** —— 人真正看到的东西。
pub fn listable(reg: &CapabilityTreeRegistry) -> Vec<MarketEntry> {
    let all = list_with_calls(reg, |_| 0);
    let mut v: Vec<MarketEntry> = all.into_iter().filter(MarketEntry::is_listable).collect();
    v.sort_by(|a, b| a.id.cmp(&b.id));
    v
}

/// **「不可上架」清单** —— **这才是开发期最有用的一张表**。
///
/// 为什么必须暴露它：若市场只显示「可上架」，
/// 那 83 处能力构造点里**没填 market.* 的会静默消失**
/// ⇒ **「不可见」又回来了**（与本轮治的病同型）。
/// ⇒ **`blocked` 与 `listable` 必须成对出现**。
pub fn blocked(reg: &CapabilityTreeRegistry) -> Vec<(String, String)> {
    let all = list_with_calls(reg, |_| 0);
    let mut v: Vec<(String, String)> = all
        .into_iter()
        .filter_map(|e| e.blocked_reason().map(|r| (e.id, r)))
        .collect();
    v.sort();
    v
}

/// 按类型分组（对标 hermes 的 `type` 过滤）—— 市场的**第一维度**。
pub fn by_kind(reg: &CapabilityTreeRegistry) -> Vec<(CapabilityKind, usize)> {
    let all = list_with_calls(reg, |_| 0);
    let mut counts: std::collections::BTreeMap<CapabilityKind, usize> =
        std::collections::BTreeMap::new();
    for e in all.into_iter().filter(MarketEntry::is_listable) {
        *counts.entry(e.kind).or_insert(0) += 1;
    }
    counts.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nt_core_capability_tree::node::Domain;

    fn node_with_meta(id: &str, kv: &[(&str, &str)], kind: CapabilityKind) -> CapabilityNode {
        let mut n = CapabilityNode::new_primitive(
            id.to_owned(),
            Domain::Act,
            vec![format!("{id}::tag")],
        );
        n.kind = kind;
        for (k, v) in kv {
            // `metadata` 的值类型是 `serde_json::Value`（编译器会纠正人）
            n.metadata.insert(
                (*k).to_owned(),
                serde_json::Value::String((*v).to_owned()),
            );
        }
        n
    }

    fn reg_with(nodes: Vec<CapabilityNode>) -> CapabilityTreeRegistry {
        let mut r = CapabilityTreeRegistry::default();
        for n in nodes {
            let id = n.id.clone();
            r.register(n).expect("register");
            let _ = id;
        }
        r
    }

    /// **纪律③ 的可证伪测试**：缺 license **不可上架**
    /// （⛔ **不是** unknown）。
    #[test]
    fn 缺license或version即不可上架_而不是unknown() {
        let reg = reg_with(vec![
            node_with_meta("a::ok", &[("market.license", "MIT"), ("market.version", "1.0.0")], CapabilityKind::Skill),
            node_with_meta("b::no-license", &[("market.version", "1.0.0")], CapabilityKind::Skill),
            node_with_meta("c::no-version", &[("market.license", "MIT")], CapabilityKind::Skill),
        ]);
        let ids: Vec<String> = listable(&reg).into_iter().map(|e| e.id).collect();
        assert_eq!(ids, vec!["a::ok".to_owned()], "⭐⭐ 只有两项齐全的才可上架");
        let blocked = blocked(&reg);
        assert_eq!(blocked.len(), 2, "⭐⭐⭐ ⭐⭐ 不可见 = 静默消失 ⇒ 必须成对暴露");
        assert!(blocked.iter().any(|(id, r)| id == "b::no-license" && r.contains("license")));
        assert!(blocked.iter().any(|(id, r)| id == "c::no-version" && r.contains("version")));
    }

    /// **纪律② 的可证伪测试**：`Gap` 绝不出现在「我有什么」里。
    #[test]
    fn gap类型绝不出现在可上架视图() {
        let reg = reg_with(vec![
            node_with_meta("real::skill", &[("market.license", "MIT"), ("market.version", "1")], CapabilityKind::Skill),
            node_with_meta("consciousness::gap::q30", &[("market.license", "MIT"), ("market.version", "1")], CapabilityKind::Gap),
        ]);
        let ids: Vec<String> = listable(&reg).into_iter().map(|e| e.id).collect();
        assert_eq!(ids, vec!["real::skill".to_owned()], "⭐⭐⭐ 缺口 ⭐⭐ 不得出现在「我有什么」");
        let blocked = blocked(&reg);
        assert!(blocked.iter().any(|(id, r)| id.contains("gap") && r.contains("缺口")));
    }

    /// **纪律① 的可证伪测试**：市场**不持有数据** 是投影。
    #[test]
    fn 市场是投影_节点删掉则条目同步消失() {
        let mut reg = reg_with(vec![node_with_meta(
            "x::y",
            &[("market.license", "MIT"), ("market.version", "1")],
            CapabilityKind::Skill,
        )]);
        assert_eq!(listable(&reg).len(), 1);
        // 市场**没有** `remove`、没有自己的存储 ⇒ 删节点即消失
        reg.nodes.shift_remove("x::y");
        assert_eq!(listable(&reg).len(), 0, "⭐⭐⭐ 市场必须是能力树的投影，⭐⭐ 不是第二份清单");
    }

    /// **`invoked` 注入**：冷启动 0 次 **仍然可上架**。
    /// **这是职责分离的关键**：「调用过 0 次」是 **金丝雀的健康问题**，
    /// **不是市场的准入问题** 
    /// 否则没人调过的能力会被市场**悄悄藏起来** ⇒ 那才是「不可见」。
    #[test]
    fn 零调用仍可上架_因为调用数不是准入判据() {
        let reg = reg_with(vec![node_with_meta(
            "cold::skill",
            &[("market.license", "MIT"), ("market.version", "1")],
            CapabilityKind::Skill,
        )]);
        let all = list_with_calls(&reg, |_| 0);
        assert_eq!(all[0].invoked, 0, "⭐⭐ 零调用是合法状态");
        assert!(all[0].is_listable(), "⭐⭐⭐⭐ **零调用 ⛔ 不得影响上架**（职责分离）");
    }

    /// **`market.*` 值类型填错 ⇒ 判红而非静默接受**。
    ///
    /// 因为 `metadata` 是 `serde_json::Value`，而取值走
    /// `as_str()` ⇒ 「写成数字」会拿到 `None`
    /// ⇒ 落到默认值 ⇒ 被 `is_listable()` 判红。
    /// **这正是想要的行为**：一个「看起来填了」的
    /// 错类型，**不该被当成合规**。
    #[test]
    fn 元数据类型填错判红而非静默接受() {
        let mut n = node_with_meta("t::y", &[], CapabilityKind::Skill);
        // 故意把 version 写成**数字**而不是字符串
        n.metadata.insert(
            meta_keys::VERSION.to_owned(),
            serde_json::json!(1.0),
        );
        n.metadata.insert(
            meta_keys::LICENSE.to_owned(),
            serde_json::Value::String("MIT".to_owned()),
        );
        let reg = reg_with(vec![n]);
        assert!(listable(&reg).is_empty(), "⭐⭐⭐⭐ 错类型 ⇒ **不可上架**（⭐⭐ ⛔ 不静默接受）");
        let b = blocked(&reg);
        assert!(b.iter().any(|(id, r)| id == "t::y" && r.contains("version")),
            "⭐⭐⭐⭐ ⭐⭐ 且必须说清是 version 的问题");
    }

    /// **输出确定**：同样的注册顺序不同 ⇒ 条目顺序必须相同。
    #[test]
    fn 输出确定_与注册顺序无关() {
        let a = node_with_meta("z::z", &[("market.license", "MIT"), ("market.version", "1")], CapabilityKind::Skill);
        let b = node_with_meta("a::a", &[("market.license", "MIT"), ("market.version", "1")], CapabilityKind::Tool);
        let r1 = reg_with(vec![a.clone(), b.clone()]);
        let r2 = reg_with(vec![b, a]);
        let ids1: Vec<String> = listable(&r1).into_iter().map(|e| e.id).collect();
        let ids2: Vec<String> = listable(&r2).into_iter().map(|e| e.id).collect();
        assert_eq!(ids1, ids2, "⭐⭐ 市场列表必须确定（⭐⭐ 门才能稳定 diff）");
    }

    /// **`category` ⛔ 不退回 `domain`** ——架构轴与市场轴必须分开。
    #[test]
    fn category不回退到domain_否则架构重构会重排市场目录() {
        let reg = reg_with(vec![node_with_meta(
            "q::r",
            &[("market.license", "MIT"), ("market.version", "1")],
            CapabilityKind::Skill,
        )]);
        let e = &listable(&reg)[0];
        assert_eq!(e.category, "uncategorized", "⭐⭐ 未填 category ⇒ 明写 uncategorized");
        assert_ne!(e.category, e.domain, "⭐⭐⭐ category ⛔ 不得等于 domain（⭐⭐ 两轴不同）");
    }
}