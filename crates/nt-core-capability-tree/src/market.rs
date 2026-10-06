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
}

/// 贸易能力的市场清单（**4 条**）。
///
/// # ⭐ 本清单曾被**编造**过，教训在案（2026-10-06）
///
/// 首版我凭空写了 5 条 `NT-MEMORY::trade::*` / `NT-MIND::trade::*`
/// 的 id（命名方案自创、数量自创）。
/// 而真实注册表 `create_default_registry()` 只有 **4** 个实现，id 形如
/// `trade.price_calculator`。**零重叠**——却因两侧都读同一份清单，
/// `neobot capability list` 一度**上架 5 项不存在的能力**，
/// 看上去完全健康。
///
/// ⇒ 本清单的每一条都必须由 `capability_registry` 里的真实 id 逐字对照，
/// 而「对照」由 core 侧测试 `manifest每个id都能在全局注册表查到` 承重：
/// id 一旦漂移，**测试立刻红**（不是运行时静默降级）。
///
/// # 为什么不含 `StepHandlerCapability`
///
/// 它**有** `execute_trade` 实现，但 id 是**动态的**
/// `trade.step.{handler_name}` ⇒ 无法进静态清单
/// （要进，得先把 handler 名集合固化 —— 那是独立的一件事）。
pub const TRADE_MANIFEST: &[ManifestEntry] = &[
    ManifestEntry {
        id: "trade.price_calculator",
        domain: crate::node::Domain::Mind,
        category: "trade/pricing",
        description: "按配置计算价格（PriceCalcRequest → 计价结果）",
    },
    ManifestEntry {
        id: "trade.product_matcher",
        domain: crate::node::Domain::Mind,
        category: "trade/product",
        description: "按配置做产品匹配（ProductMatchRequest → 匹配结果）",
    },
    ManifestEntry {
        id: "trade.risk_assessor",
        domain: crate::node::Domain::Mind,
        category: "trade/risk",
        description: "按配置评估风险（RiskAssessRequest → 风险结论）",
    },
    ManifestEntry {
        id: "trade.supplier_matcher",
        domain: crate::node::Domain::Mind,
        category: "trade/supplier",
        description: "按配置匹配供应商（SupplierMatchRequest → 供应商候选）",
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
    /// 清单 id 必须用**实现注册表**的 id 形态（点号 `trade.*`）。
    ///
    /// ⚠️ 首版此处断言的是 `域::模块::实例`（`::` 出现 2 次）——
    /// 那是我照抄 **canary 的 id 空间**写下的，而 canary 与实现
    /// **是两个注册表、两套 id**。真实派发走 `capability_registry`，
    /// 故清单必须用**它**的形态。
    #[test]
    fn 清单id是实现注册表形态() {
        for e in TRADE_MANIFEST {
            assert!(
                e.id.starts_with("trade.") && !e.id.contains("::"),
                "id 应为实现注册表形态 `trade.*`，实得 {:?}",
                e.id
            );
        }
    }

    /// 清单**非空**（市场有东西可展示）。
    #[test]
    fn 清单非空() {
        assert!(!TRADE_MANIFEST.is_empty(), "清单为空 ⇒ 市场无从展示");
    }
}