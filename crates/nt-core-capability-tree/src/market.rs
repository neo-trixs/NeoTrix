//! 能力市场清单 —— **纯数据**，两侧共读。
//!
//! ## 为什么清单在共享 crate 而不在 core 或 neobot
//!
//! `capability_invoke` 长期生产不可达，根因是**播种点在 core、挂载点在
//! neobot，而依赖方向固定 `core → neobot`** ⇒ 工作区里**没有任何 crate
//! 同时依赖两侧** ⇒ 注册表在服务进程里恒空。
//!
//! 而两侧**都能依赖本 crate**（`neotrix-neobot/Cargo.toml:42`）⇒ 把
//! 「这个能力是什么」这份**纯事实**放这里，是唯一能让两侧对上号的位置。
//!
//! ## 字段归属按语义切分（不是按「谁方便」）
//!
//! | 字段 | 语义 | 家 |
//! |---|---|---|
//! | `id` / `category` / `description` | **这个能力是什么** | **本清单**（纯数据） |
//! | `version` | **本仓实现版本** | 调用方（core 用 `env!`，neobot 用自己的） |
//! | `license` | **本仓的许可决策** | 调用方，但**共用本文件的串** |
//!
//! ⚠️ `version` 刻意**不在本清单里**：它是「实现所在 crate 的包版本」。
//! 若把它写成 `env!("CARGO_PKG_VERSION")`，在 core 里解析为 core 的版本、
//! 在 neobot 里解析为neobot 的版本 —— **同一份清单会静默报告不同版本**。
//!
//! ## 版本一致性
//!
//! `neotrix-core` 与 `neotrix-neobot` 都是 `version.workspace = true`
//! ⇒ 当前**天然同步**（同为 `0.23.0`）。
//! 由 `scripts/ops/check_version_sync.py` 持续守护：
//! 一旦有人解开继承而两边版本分叉，**门立即变红**。

/// 市场元数据的**规范键名** —— 本 crate 是唯一定义处。
///
/// ⛔ `neotrix-neobot::nt_capability_market::meta_keys` 必须**引用本处**，
/// 不得再硬编码字符串：两份键名会分叉，而 `metadata` 是
/// `HashMap<String, serde_json::Value>` ⇒ **键名拼错不会报错**，
/// 只会让值「读不出来」而静默降级。
pub mod keys {
    /// 版本号（市场必需，缺失即不可上架）
    pub const VERSION: &str = "market.version";
    /// 许可（市场必需，缺失即不可上架）
    pub const LICENSE: &str = "market.license";
    /// 市场分组（**架构轴 domain ≠ 市场轴 category**，混用会让目录随重构重排）
    pub const CATEGORY: &str = "market.category";
    /// 描述（缺失不阻断，但市场会显示为空）
    pub const DESCRIPTION: &str = "market.description";
}

/// 本仓自有实现的许可标识。
///
/// 「许可」是关于**本仓实现**的声明，不是关于能力概念的常量
/// ⇒ 但它必须是**同一个串**（core 与 neobot 两侧都写它），故放这里共用。
pub const TRADE_LICENSE: &str = "LicenseRef-NeoTrix-Internal";

/// 能力**是否可执行** —— 「执行器登记制」的记账字段。
///
/// # 为什么要有这个枚举（2026-10-06）
///
/// 本轮实测抓到 4 处「**上架了但不可用/是桩**」：
/// manifest id 与实现错层 · 金丝雀度量「被查过」而非「被执行过」·
/// 安全工具注册失败静默 · 执行器是阶段脚手架。
///
/// ⇒ 光有「清单」不够，**必须同时记账「它到底能不能干活」**，
/// 且这个记账要**可被机器校验**（见 `scripts/check-executor-registry.sh`），
/// 否则又是一条「写着健康、实际不能跑」的说法。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Executability {
    /// **无可调用入口**（只有 engine 方法 / 只是知识包工厂）
    ///
    /// ⇒ 派发必然返 `None` ⇒ 调用方 fail-closed。
    /// ⚠️ 上架它**不等于**它能干活；描述里必须写明。
    DeclaredOnly,
    /// **有可调用入口，且已通过非桩证据测试**
    Executable,
    /// **有入口，但是脚手架**（各阶段业务逻辑未实现）
    ///
    /// ⚠️ 最危险的一档：能跑通、有产出、却没做实事
    /// （本轮 `foreign_trade_full_cycle` 即此档）。
    Scaffold,
}

/// 清单里的一条：**只含「这个能力是什么」**，不含版本/许可。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManifestEntry {
    /// 能力树节点 id（**与 `expect()` 登记的 id 逐字一致**）
    pub id: &'static str,
    /// 所属域（**架构轴**）—— 刻意**不含**在此的是 `category`（市场轴）之外的
    /// 任何「市场呈现面」字段。
    pub domain: crate::node::Domain,
    /// 市场分组（`trade/...`）
    pub category: &'static str,
    /// 人读描述
    pub description: &'static str,
    /// ⭐ 可执行性记账（见 [`Executability`]）
    pub executability: Executability,
}

/// 贸易能力的市场清单（**5 条**）。
///
/// # ⭐ 这 5 条是**既有契约**（`7187e0b9`，2026-10-03，早于本轮），不是编造
///
/// 每个 id 都能在对应实现模块里找到**字面同串 + 注册调用**：
///
/// | id | 实现模块 |
/// |---|---|
/// | `NT-MEMORY::trade::trade_product_spec` | `l4_emotion/nt_memory/nt_trade_product_spec.rs` |
/// | `NT-MIND::trade::trade_quote_negotiation` | `l1_action/nt_act/nt_act_trade/quote_negotiation.rs` |
/// | `NT-MIND::trade::trade_production_logistics` | `…/production_logistics.rs` |
/// | `NT-MIND::trade::trade_finance_compliance` | `…/finance_compliance.rs` |
/// | `NT-MIND::trade::foreign_trade_full_cycle` | `…/full_cycle.rs` |
///
/// # ⚠️⛔ 不要拿`TradeCapabilityRegistry` 来核对本清单
///
/// 那是**另一层**能力：`trade.price_calculator` / `product_matcher` /
/// `risk_assessor` / `supplier_matcher`（点号 id，4 个）——
/// **计价 / 产品匹配 / 风控 / 供应商匹配**，与本清单的 5 个贸易能力**不是同一批东西**。
///
/// 我曾据它把清单改成 `trade.*`（提交 `f67942ea`）⇒ **那是错的，已回退**。
/// 详见 `docs/architecture/FOLLOWUP-TASKS-2026-10-06.md` 的 P0.1 修正。
pub const TRADE_MANIFEST: &[ManifestEntry] = &[
    ManifestEntry {
        id: "NT-MEMORY::trade::trade_product_spec",
        domain: crate::node::Domain::Memory,
        category: "trade/product-spec",
        description: "贸易产品规格生成（L4 memory 侧）",
        // 实测：只有 `get_product_knowledge_pack`（知识包工厂），**无执行器**
        executability: Executability::DeclaredOnly,
    },
    ManifestEntry {
        id: "NT-MIND::trade::trade_quote_negotiation",
        domain: crate::node::Domain::Mind,
        category: "trade/quote",
        description: "报价谈判",
        // 实测：顶层 `execute_quote_negotiation` + serde 三元组 ⇒ 可派发
        executability: Executability::Executable,
    },
    ManifestEntry {
        id: "NT-MIND::trade::trade_production_logistics",
        domain: crate::node::Domain::Mind,
        category: "trade/logistics",
        description: "生产物流",
        // 实测：顶层执行器数为 **0**（只有 engine 方法）⇒ 无单一入口
        executability: Executability::DeclaredOnly,
    },
    ManifestEntry {
        id: "NT-MIND::trade::trade_finance_compliance",
        domain: crate::node::Domain::Mind,
        category: "trade/finance",
        description: "金融合规",
        // 实测：顶层执行器数为 **0**（只有 FinanceEngine 方法）
        executability: Executability::DeclaredOnly,
    },
    ManifestEntry {
        id: "NT-MIND::trade::foreign_trade_full_cycle",
        domain: crate::node::Domain::Mind,
        category: "trade/full-cycle",
        // ⚠️⚠️ **诚实标注**：该能力的执行器目前是**阶段脚手架**——
        // 17 个阶段的业务逻辑全是注释，函数只推进状态机（实测 2026-10-06）。
        // ⇒ 上架它不等于它能干活。详见 FOLLOWUP-TASKS P0.4。
        description: "外贸全链（⚠️ 当前为阶段脚手架，未含各阶段业务逻辑）",
        // ⚠️ 能跑通、有产出、却**没做实事** ⇒ 最危险的一档
        executability: Executability::Scaffold,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 清单id唯一且非空() {
        let mut ids: Vec<&str> = TRADE_MANIFEST.iter().map(|e| e.id).collect();
        let n = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), n, "清单 id 重复会让反向核对永远红");
        assert!(TRADE_MANIFEST.iter().all(|e| !e.id.trim().is_empty()));
    }

    #[test]
    fn 键名与neobot_门面一致() {
        // 本 crate 是唯一定义处；neobot 的 meta_keys 必须引用这里。
        assert_eq!(keys::VERSION, "market.version");
        assert_eq!(keys::LICENSE, "market.license");
        assert_eq!(keys::CATEGORY, "market.category");
        assert_eq!(keys::DESCRIPTION, "market.description");
    }

    /// 清单 id 必须是**能力树 id 形态**（含两个 `::`），否则反向核对照不上。
    #[test]
    /// 清单 id 必须是**能力树 id** 形态（`域::模块::实例`，`::` 恰 2 次）。
    #[test]
    fn 清单id是能力树形态() {
        for e in TRADE_MANIFEST {
            assert_eq!(
                e.id.matches("::").count(),
                2,
                "id 应为 `域::模块::实例` 形态，实得 {:?}",
                e.id
            );
        }
    }

    /// 清单非空。
    #[test]
    fn 清单非空() {
        assert!(!TRADE_MANIFEST.is_empty(), "清单为空 ⇒ 市场无从展示");
    }
}