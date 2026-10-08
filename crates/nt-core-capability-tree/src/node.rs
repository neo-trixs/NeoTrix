//! 能力树节点定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// NT-* 领域轴 (X 轴)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    Core,
    Mind,
    Memory,
    World,
    Act,
    Shield,
    Io,
    Meta,
    Nexus,
    Governance,
    Repair,
    /// `neobot` 独立 crate 的能力域。
    ///
    /// 2026-10-08：本变体不是「发明」，而是**补上代码与数据的漂移** ——
    /// `.neotrix/capability_registry.json` 里本就有 4 个 `domain: "neobot"`
    /// 节点（`neobot::nt_routing::quota_mode` / `nt_channel_wecom` /
    /// `nt_llama::ssd_offload` / `nt_agent_home`，均带完整 evolution_log），
    /// 而枚举里没有对应值 ⇒ 整份注册表 `serde` 解析失败 ⇒
    /// **326 个节点在生产里全部不可见**（`skill_tree.rs` 用 `.ok()?` 静默吞掉，
    /// 后台维护循环 `log::warn` 后 `return None`）。
    ///
    /// ⛔ 与 AGENTS.md §6 的裁决不冲突：那条讲的是**架构台账**
    /// （R-P199 口径限 neotrix-core L1–L6，neobot 不占 L 层），
    /// 不是能力树的 Domain 取值集。
    Neobot,
}

impl Domain {
    pub fn as_str(&self) -> &'static str {
        match self {
            Domain::Core => "core",
            Domain::Mind => "mind",
            Domain::Memory => "memory",
            Domain::World => "world",
            Domain::Act => "act",
            Domain::Shield => "shield",
            Domain::Io => "io",
            Domain::Meta => "meta",
            Domain::Nexus => "nexus",
            Domain::Governance => "governance",
            Domain::Repair => "repair",
            Domain::Neobot => "neobot",
        }
    }

    /// 从域名字符串解析 (大小写不敏感); 兼容新格式裸名 (core/act/...) 与老格式 "NT-*"; 无法识别时返回 None。
    pub fn parse(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            "CORE" | "NT-CORE" => Some(Domain::Core),
            "MIND" | "NT-MIND" => Some(Domain::Mind),
            "MEMORY" | "NT-MEMORY" => Some(Domain::Memory),
            "WORLD" | "NT-WORLD" => Some(Domain::World),
            "ACT" | "NT-ACT" => Some(Domain::Act),
            "SHIELD" | "NT-SHIELD" => Some(Domain::Shield),
            "IO" | "NT-IO" => Some(Domain::Io),
            "META" | "NT-META" => Some(Domain::Meta),
            "NEXUS" | "NT-NEXUS" => Some(Domain::Nexus),
            "GOVERNANCE" | "NT-GOVERNANCE" => Some(Domain::Governance),
            "REPAIR" | "NT-REPAIR" => Some(Domain::Repair),
            "NEOBOT" | "NT-NEOBOT" => Some(Domain::Neobot),
            _ => None,
        }
    }
}

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 抽象层轴 (Z 轴)
/// **能力类型**（2026-10-04，吸收 `hermes-desktop` 的
/// `IndexEntry.type`）—— **「一切为插件」的分类维度**。
///
/// **为什么必须是显式字段、⛔ 不能从 id 猜**（本轮实测）：
/// 本仓 83 处生产侧能力构造点里，**只有 30 处用了可读域前缀**
/// （`NT-MIND` 20 / `NT-MEMORY` 6 / `consciousness` 3 / `exp` 1）
/// ⇒ **从 id 推断类型会漏掉一大半**，那是**结构性脆弱**。
///
/// 与 hermes 的对应关系（语义不同，⛔ 不是照抄枚举值）：
/// | hermes `IndexEntry.type` | 本仓 `CapabilityKind` |
/// |---|---|
/// | `mcp` | `Tool`（本仓工具统一经 MCP 桥暴露） |
/// | `skill` | `Skill` |
/// | `workflow` | `Workflow` |
/// | `agent` | `Agent` |
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CapabilityKind {
    /// 工具（经 `McpBridge` 派发）
    Tool,
    /// 技能（可复用的领域能力）
    Skill,
    /// 工作流（多能力编排）
    Workflow,
    /// 代理（有自身循环/状态者）
    Agent,
    /// **缺口**（意识观察到「自己不会」而登记的节点）
    /// **它不是插件** ⇒ 市场的「可安装项」视图**必须排除它**
    Gap,
}

impl Default for CapabilityKind {
    /// **缺 `kind` 的旧节点一律按 `Skill`**。
    ///
    /// ## 为什么是 `Skill` 而不是 `Gap`
    ///
    /// 市场的 fail-closed 依赖 `Gap` 被排除在可上架清单外，所以默认值
    /// 若取 `Gap` 会让**所有**能力都不可上架（市场恒空）；
    /// 若取 `Skill`，一个本该是 `Gap` 的节点会被错误上架。
    ///
    /// ## 为什么这个风险在当前数据上**不成立**（实测，非推测）
    ///
    /// 已提交的注册表 318 个节点里，`kind` 键**一个都没有**，而
    /// `consciousness::gap::*` 节点**一个都不在该文件里** —— 缺口节点由
    /// `ConsciousnessRuntime` 在**运行期**注册并显式带
    /// `CapabilityKind::Gap`（见 `ea5b5424`）。
    /// ⇒ 旧文件里的节点都不是缺口，取 `Skill` 不会让缺口混入市场。
    ///
    /// ⛔ 若将来把缺口节点**持久化进这个文件**，必须先给它们写显式
    ///   `kind: "gap"`，否则默认值会把「我不会」伪装成「我会」。
    fn default() -> Self {
        Self::Skill
    }
}

impl CapabilityKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tool => "tool",
            Self::Skill => "skill",
            Self::Workflow => "workflow",
            Self::Agent => "agent",
            Self::Gap => "gap",
        }
    }

    /// **它是不是一个「可被市场列出的插件」**（关键区分）。
    ///
    /// `Gap` **不是插件**：它是意识登记的**缺口**
    /// （`consciousness::gap::q*`，来自 `observe_from_critique`）
    /// ⇒ **市场的「已安装能力」视图若含它，就是把
    /// 「我不会」当成「我有」⇒ **语义反了**。
    pub fn is_marketable(&self) -> bool {
        !matches!(self, Self::Gap)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeLayer {
    L0Primitive,
    L1Composite,
    L2Orchestrator,
    L2World,
    L3DomainService,
    L3Memory,
    L4Application,
    L4Cognition,
    L5Conscious,
    L6Self,
    L7Capability,
    L8Autonomic,
}

impl NodeLayer {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeLayer::L0Primitive => "l0primitive",
            NodeLayer::L1Composite => "l1composite",
            NodeLayer::L2Orchestrator => "l2orchestrator",
            NodeLayer::L3DomainService => "l3domainservice",
            NodeLayer::L4Application => "l4application",
            NodeLayer::L4Cognition => "l4cognition",
            NodeLayer::L2World => "l2world",
            NodeLayer::L3Memory => "l3memory",
            NodeLayer::L5Conscious => "l5conscious",
            NodeLayer::L6Self => "l6self",
            NodeLayer::L7Capability => "l7capability",
            NodeLayer::L8Autonomic => "l8autonomic",
        }
    }
}

/// 星座成熟度 (Y 轴) - C0 到 C6
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ConstellationLevel {
    #[default]
    C0Compile,
    C1UnitTest,
    #[serde(alias = "c2integration")]
    C2IntegrationTest,
    C3Benchmark,
    C4MainPipeline,
    C5SelfHealing,
    C6EvolutionLoop,
}

impl ConstellationLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConstellationLevel::C0Compile => "c0compile",
            ConstellationLevel::C1UnitTest => "c1unittest",
            ConstellationLevel::C2IntegrationTest => "c2integrationtest",
            ConstellationLevel::C3Benchmark => "c3benchmark",
            ConstellationLevel::C4MainPipeline => "c4mainpipeline",
            ConstellationLevel::C5SelfHealing => "c5selfhealing",
            ConstellationLevel::C6EvolutionLoop => "c6evolutionloop",
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            ConstellationLevel::C0Compile => Some(ConstellationLevel::C1UnitTest),
            ConstellationLevel::C1UnitTest => Some(ConstellationLevel::C2IntegrationTest),
            ConstellationLevel::C2IntegrationTest => Some(ConstellationLevel::C3Benchmark),
            ConstellationLevel::C3Benchmark => Some(ConstellationLevel::C4MainPipeline),
            ConstellationLevel::C4MainPipeline => Some(ConstellationLevel::C5SelfHealing),
            ConstellationLevel::C5SelfHealing => Some(ConstellationLevel::C6EvolutionLoop),
            ConstellationLevel::C6EvolutionLoop => None,
        }
    }

    /// 晋级条件 — 对标网文境界体系 (每个境界需满足明确晋级条件)。
    /// 来源: novel-causal-chain-analysis.md §启发2 (力量体系 = Constellation 阶梯)
    pub fn promotion_requirement(&self) -> &'static str {
        match self {
            ConstellationLevel::C0Compile => "cargo check 0 errors (可编译)",
            ConstellationLevel::C1UnitTest => "单元测试 ≥3 且全绿",
            ConstellationLevel::C2IntegrationTest => "集成测试通过 + 生产接线",
            ConstellationLevel::C3Benchmark => "benchmark 基线建立 + 无回归",
            ConstellationLevel::C4MainPipeline => "接入 SEAL 主流水线并被消费",
            ConstellationLevel::C5SelfHealing => "自愈回路闭环 (检测→恢复→验证)",
            ConstellationLevel::C6EvolutionLoop => "吸收循环闭环 (快照→蒸馏→落盘→反馈)",
        }
    }

    /// 晋级代价 — 对标网文"每个境界有代价" (扮演法/献祭/反噬)。
    /// 规则: 境界越高, 代价越大; 代价是晋升的治理成本。
    pub fn promotion_cost(&self) -> &'static str {
        match self {
            ConstellationLevel::C0Compile => "依赖治理 + 编译时间",
            ConstellationLevel::C1UnitTest => "测试维护成本",
            ConstellationLevel::C2IntegrationTest => "集成复杂度 + 跨模块契约",
            ConstellationLevel::C3Benchmark => "性能分析成本",
            ConstellationLevel::C4MainPipeline => "故障影响面扩大 (生产路径)",
            ConstellationLevel::C5SelfHealing => "自愈逻辑复杂度",
            ConstellationLevel::C6EvolutionLoop => "演化稳定性治理",
        }
    }

    /// 能力表现 — 对标网文"每个境界的实力表现" (可视化升级节点)。
    pub fn capability_manifest(&self) -> &'static str {
        match self {
            ConstellationLevel::C0Compile => "模块存在且可编译",
            ConstellationLevel::C1UnitTest => "核心逻辑可验证",
            ConstellationLevel::C2IntegrationTest => "跨模块协作可用",
            ConstellationLevel::C3Benchmark => "性能可度量可对比",
            ConstellationLevel::C4MainPipeline => "生产路径消费产出",
            ConstellationLevel::C5SelfHealing => "失败自动恢复",
            ConstellationLevel::C6EvolutionLoop => "自主演化进化",
        }
    }
}

/// Rune 插槽类型 (5 槽)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuneSocket {
    Crimson,    // 数据摄取
    Indigo,     // 变换
    Obsidian,   // 缓存
    Golden,     // 错误恢复
    Alabaster,  // 监控
    #[serde(other)]
    Unknown,    // 容忍注册表中非标准 rune 标记 (如 "indigo:transform")
}

impl RuneSocket {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuneSocket::Crimson => "Crimson",
            RuneSocket::Indigo => "Indigo",
            RuneSocket::Obsidian => "Obsidian",
            RuneSocket::Golden => "Golden",
            RuneSocket::Alabaster => "Alabaster",
            RuneSocket::Unknown => "Unknown",
        }
    }
}

/// 演化操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvolutionOp {
    Budding,           // 萌芽: 新建 Primitive
    Grafting,          // 嫁接: 折叠分散实现到 Primitive/Composite
    Pruning,           // 修剪: 标记废弃/删除无用节点
    CrossPollination,  // 异花授粉: 跨域抽象共享 Primitive
    Maturation,        // 成熟晋升: Cn -> Cn+1
    Strengthen,        // 强化: 吸收经验强化既有节点 (R-P42 吸收强化现有节点, 不新建)
}

/// 演化日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionLogEntry {
    pub cycle: String,
    pub op: EvolutionOp,
    pub from_nodes: Vec<String>,      // 来源节点 (Grafting/Pruning 时)
    pub to_node: Option<String>,      // 目标节点 (Budding/Maturation 时)
    pub note: String,
    #[serde(default)]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runeword_change: Option<String>,
}

/// Runeword 配置 — 基于 constellation level 自动分配的 rune 槽
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunewordConfig {
    /// 当前填满的 rune 槽 (按 constellation level 自动分配)
    pub sockets: Vec<RuneSocket>,
    /// 当前 runeword 名称 (随 constellation 晋升而变化)
    pub runeword: String,
    /// 是否启用 Scry (完整 ETL, 5 槽全开)
    pub scry_enabled: bool,
    /// 上一次晋升时的 constellation level
    pub last_promotion_level: ConstellationLevel,
}

/// 能力节点核心定义
#[derive(Debug, Clone, Serialize, Deserialize)]
/// 能力树节点（**数据模型**）。
///
/// ⚠️ 与 `neotrix_core::l0_substrate::nt_core_traits::CapabilityNode`（**同名 trait**）
/// 不是同一个东西 —— 那是「运行期能力提供者」的行为接口，本结构体是树的数据模型。
/// 见 `docs/architecture/FOLLOWUP-TASKS-2026-10-06.md` T0.1。
///
/// 📌 **这里是能力市场描述性元数据的家**：`category` / `version` / `license` /
/// `description` / `tags` / `maturity` 应下沉到本结构体（neobot 已依赖本 crate），
/// 而**不是**复制到 neobot —— 复制会产生第二个真身（本仓已记录四起同源数据腐化）。
/// 当前这 6 个字段仍在别处 ⇒ 这是 `capability_invoke` 生产不可达的根因（T0.1）。
pub struct CapabilityNode {
    pub id: String,                           // 全局唯一 ID: "domain::module::function"
    pub domain: Domain,
    pub layer: NodeLayer,
    pub constellation: ConstellationLevel,
    /// **能力类型**（**市场枚举的第一维度**，见 `CapabilityKind`）
    // 2026-10-06 P0 数据丢失修复：`kind` 曾是**唯一没有 `#[serde(default)]`**
    // 的可选语义字段，而已提交的 `.neotrix/capability_registry.json`
    // 318 个节点**全部**没有这个键 ⇒ `from_str::<RegistryExport>` 整体失败。
    // `cli::load_registry` 用 `Err(_)` 吞掉错误并误走「老 schema 迁移」，
    // 迁移结果 **0 节点**；随后 `save_registry` 把 41 个 roadmap 节点
    // 写回 ⇒ **318 节点 / 43 边被一次性销毁**（实测 3/3 复现）。
    // 加 `default` 让旧文件仍可解析；默认值语义见 `Default for CapabilityKind`。
    #[serde(default)]
    pub kind: CapabilityKind,
    pub provides: Vec<String>,                // 提供的能力标签
    /// **树内**依赖：可解析为节点 id **或**某个节点的 `provides` 标签
    /// （双命名空间，见 `registry::CapabilityTreeRegistry::register`）。
    /// ⛔ 语义收敛为「**必须能解析**」：注册表自洽性的判据就是本字段全解析。
    #[serde(default)]
    pub requires: Vec<String>,
    /// **树外**依赖：指向注册表**之外**的组件/概念（如 `memory.kv_store`、
    /// `shield.policy_engine` 这类代码子系统名），或「已声明但尚未在树内落地」
    /// 的能力（如 `neobot::nt_store_quota`）。
    ///
    /// # 为什么要有这个字段（2026-10-08）
    ///
    /// 实测：注册表里 **30** 条 `requires` 既不匹配节点 id 也不匹配任何
    /// `provides` 标签（另有 4 条形似节点 id 但同样不存在）。它们**不是垃圾**：
    /// 记的是真实的架构依赖意图（`memory.kv_store` 确实存在，只是不在能力树里）。
    ///
    /// 但混在 `requires` 里有两害：
    /// 1. **永久噪音**：每次 `cli::load_registry` 都刷 30 行
    ///    `eprintln WARNING`，且永远无法消除 ⇒ 训练人忽略告警
    ///    （与「死边」的处理先例同源：`cli.rs` 对边已是「跳过并警告」）。
    /// 2. **真回归被淹没**：日后真的写错一个 `requires`，与这 30 条静态噪音混在一起，
    ///    判据失效。
    ///
    /// ⇒ 显式分流：**`requires` 只放能解析的**（于是
    ///   `validate_dependencies` 恢复为有意义的自洽性信号），
    ///   树外的意图原样搬进本字段，**一条信息都不删**。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_requires: Vec<String>,
    #[serde(default)]
    pub rune_sockets: Vec<RuneSocket>,        // 占用的 Rune 槽
    #[serde(default)]
    pub dependents: Vec<String>,              // 反向依赖 (谁在用我)
    #[serde(default)]
    pub evolution_log: Vec<EvolutionLogEntry>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub deprecated: bool,
    #[serde(default)]
    pub deprecated_reason: Option<String>,
    #[serde(default)]
    pub runeword_config: RunewordConfig,
}

impl CapabilityNode {
    /// 创建 L0 Primitive (Root)
    pub fn new_primitive(
        id: String,
        domain: Domain,
        provides: Vec<String>,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            domain,
            // 默认 `Skill`：`new_primitive` 是最常用的构造器，
            // 而绝大多数能力确实既非工具也非工作流 
            // （⛔ 默认成 `Gap` 会让市场把它们全排除 ⇒ 那是更坏的错）
            kind: CapabilityKind::Skill,
            layer: NodeLayer::L0Primitive,
            constellation: ConstellationLevel::C0Compile,
            provides,
            requires: vec![],
            external_requires: vec![],
            rune_sockets: vec![],
            dependents: vec![],
            evolution_log: vec![],
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            deprecated: false,
            deprecated_reason: None,
            runeword_config: RunewordConfig::default(),
        }
    }

    /// 创建 Composite Node (L1-L2)
    pub fn new_composite(
        id: String,
        domain: Domain,
        layer: NodeLayer,
        provides: Vec<String>,
        requires: Vec<String>,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            domain,
            // 与 `new_primitive` 同一默认值（两处不一致 = 第三种状态）
            kind: CapabilityKind::Skill,
            layer,
            constellation: ConstellationLevel::C0Compile,
            provides,
            requires,
            external_requires: Vec::new(),
            rune_sockets: vec![],
            dependents: vec![],
            evolution_log: vec![],
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            deprecated: false,
            deprecated_reason: None,
            runeword_config: RunewordConfig::default(),
        }
    }

    /// 创建 Constellation (L3-L4)
    pub fn new_constellation(
        id: String,
        domain: Domain,
        layer: NodeLayer,
        provides: Vec<String>,
        requires: Vec<String>,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            domain,
            // 与 `new_primitive` 同一默认值（两处不一致 = 第三种状态）
            kind: CapabilityKind::Skill,
            layer,
            constellation: ConstellationLevel::C0Compile,
            provides,
            requires,
            external_requires: Vec::new(),
            rune_sockets: vec![],
            dependents: vec![],
            evolution_log: vec![],
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            deprecated: false,
            deprecated_reason: None,
            runeword_config: RunewordConfig::default(),
        }
    }

    /// 记录演化操作
    pub fn record_evolution(&mut self, entry: EvolutionLogEntry) {
        self.evolution_log.push(entry);
        self.updated_at = chrono::Utc::now();
    }

    /// 添加依赖者
    pub fn add_dependent(&mut self, dependent_id: String) {
        if !self.dependents.contains(&dependent_id) {
            self.dependents.push(dependent_id);
            self.updated_at = chrono::Utc::now();
        }
    }

    /// 移除依赖者
    pub fn remove_dependent(&mut self, dependent_id: &str) {
        self.dependents.retain(|d| d != dependent_id);
        self.updated_at = chrono::Utc::now();
    }

    /// 晋升证据门禁 (D16 自欺防线) — 阻止无证据晋升。
    ///
    /// 判定标准 (与 Constellation 晋级条件对齐):
    /// - C0→C1: 必须提供能力标签 (provides 非空), 证明节点有真实职责
    /// - C1→C2: 必须存在生产接线证据 (metadata.wiring_evidence 非空, 描述
    ///   生产消费路径 file:line)。注意 dependents 仅记录设计依赖 (DAG 边),
    ///   不等于运行时接线 — 接线证据必须显式声明 (审查 D2/D16 发现)。
    /// - ≥C2: 需 evidence_gated='passed' 显式标记 (集成测试 + benchmark + 流水线)。
    ///
    /// 返回 (通过?, 拒绝原因)。原因非空即拒绝。
    pub fn promotion_evidence_gate(&self) -> (bool, Option<String>) {
        match self.constellation {
            ConstellationLevel::C0Compile => {
                if self.provides.is_empty() {
                    (false, Some("C0→C1 gate: node provides no capability labels".into()))
                } else {
                    (true, None)
                }
            }
            ConstellationLevel::C1UnitTest => {
                let has_wiring = self
                    .metadata
                    .get("wiring_evidence")
                    .map(|v| v.is_string() && !v.as_str().unwrap_or("").is_empty())
                    .unwrap_or(false);
                if !has_wiring {
                    (false, Some("C1→C2 gate: no production wiring evidence (set metadata.wiring_evidence)".into()))
                } else {
                    (true, None)
                }
            }
            ConstellationLevel::C2IntegrationTest
            | ConstellationLevel::C3Benchmark
            | ConstellationLevel::C4MainPipeline => {
                if self
                    .metadata
                    .get("evidence_gated")
                    .map(|v| v == "passed")
                    .unwrap_or(false)
                {
                    (true, None)
                } else {
                    (false, Some("C2+ gate: requires evidence_gated='passed' in metadata".into()))
                }
            }
            ConstellationLevel::C5SelfHealing | ConstellationLevel::C6EvolutionLoop => {
                // D16 自欺防线收紧: C5/C6 亦须 evidence_gated='passed' + 生产接线证据,
                // 防止无证据节点被晋升到最高阶 (历史上 C5/C6 无门禁直接放行)。
                let gated = self
                    .metadata
                    .get("evidence_gated")
                    .map(|v| v == "passed")
                    .unwrap_or(false);
                let has_wiring = self
                    .metadata
                    .get("wiring_evidence")
                    .map(|v| v.is_string() && !v.as_str().unwrap_or("").is_empty())
                    .unwrap_or(false);
                if gated && has_wiring {
                    (true, None)
                } else {
                    let mut reasons = Vec::new();
                    if !gated {
                        reasons.push("evidence_gated='passed' required");
                    }
                    if !has_wiring {
                        reasons.push("production wiring_evidence (file:line) required");
                    }
                    (
                        false,
                        Some(format!("C5+ gate: {}", reasons.join(", "))),
                    )
                }
            }
        }
    }

    /// 复算"证据链实际支撑"的最高 ConstellationLevel (用于 E2 虚标审计)。
    ///
    /// 与 `promotion_evidence_gate` 互补: 后者决定"能否再晋升",
    /// 本方法倒推"当前声称值是否已被证据支撑"。
    /// C0 默认达成; C1 需 `provides` 非空; 更高等级需对应证据字段
    /// (wiring_evidence / evidence_gated / self_healing_evidence)。
    pub fn evidence_supported_constellation(&self) -> ConstellationLevel {
        // 严格对齐 D16 promotion_evidence_gate 的逐步门禁:
        // C0→C1: provides 非空; C1→C2: wiring_evidence (file:line);
        // C2→C3 / C3→C4: evidence_gated='passed'; C4→C5 / C5→C6: gated && wiring.
        let has_provides = !self.provides.is_empty();
        let has_wiring = self
            .metadata
            .get("wiring_evidence")
            .map(|v| v.is_string() && !v.as_str().unwrap_or("").is_empty())
            .unwrap_or(false);
        let gated = self
            .metadata
            .get("evidence_gated")
            .map(|v| v == "passed")
            .unwrap_or(false);

        let mut level = ConstellationLevel::C0Compile;
        if has_provides {
            level = ConstellationLevel::C1UnitTest;
        }
        if matches!(level, ConstellationLevel::C1UnitTest) && has_wiring {
            level = ConstellationLevel::C2IntegrationTest;
        }
        if matches!(level, ConstellationLevel::C2IntegrationTest) && gated {
            level = ConstellationLevel::C3Benchmark;
        }
        if matches!(level, ConstellationLevel::C3Benchmark) && gated {
            level = ConstellationLevel::C4MainPipeline;
        }
        if matches!(level, ConstellationLevel::C4MainPipeline) && gated && has_wiring {
            level = ConstellationLevel::C5SelfHealing;
        }
        if matches!(level, ConstellationLevel::C5SelfHealing) && gated && has_wiring {
            level = ConstellationLevel::C6EvolutionLoop;
        }
        level
    }

    /// 晋升星座等级 (受证据门禁约束)
    pub fn promote_constellation(&mut self) -> Result<bool, String> {
        let (gate_ok, reason) = self.promotion_evidence_gate();
        if !gate_ok {
            return Err(reason.unwrap_or_else(|| "evidence gate rejected".into()));
        }
        if let Some(next) = self.constellation.next() {
            let prev_runeword = self.runeword_config.runeword.clone();
            self.constellation = next;
            self.update_runeword_config();
            let new_runeword = self.runeword_config.runeword.clone();
            let runeword_change = if prev_runeword != new_runeword {
                Some(format!("{} → {}", prev_runeword, new_runeword))
            } else {
                None
            };
            self.record_evolution(EvolutionLogEntry {
                cycle: "auto".into(),
                op: EvolutionOp::Maturation,
                from_nodes: vec![],
                to_node: Some(self.id.clone()),
                note: format!("Promoted to {}", next.as_str()),
                timestamp: chrono::Utc::now(),
                runeword_change,
            });
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 标记废弃
    pub fn deprecate(&mut self, reason: String) {
        self.deprecated = true;
        self.deprecated_reason = Some(reason.clone());
        self.record_evolution(EvolutionLogEntry {
            cycle: "auto".into(),
            op: EvolutionOp::Pruning,
            from_nodes: vec![],
            to_node: Some(self.id.clone()),
            note: format!("Deprecated: {}", reason),
            timestamp: chrono::Utc::now(),
            runeword_change: None,
        });
    }

    /// 根据 constellation level 计算 rune 槽位
    ///
    /// C0: 1 slot (Crimson), C1: +Indigo, C2: +Obsidian, C3: +Golden, C4+: +Alabaster (Scry enabled)
    pub fn compute_rune_sockets(&self) -> Vec<RuneSocket> {
        use RuneSocket::*;
        match self.constellation {
            ConstellationLevel::C0Compile => vec![Crimson],
            ConstellationLevel::C1UnitTest => vec![Crimson, Indigo],
            ConstellationLevel::C2IntegrationTest => vec![Crimson, Indigo, Obsidian],
            ConstellationLevel::C3Benchmark => vec![Crimson, Indigo, Obsidian, Golden],
            ConstellationLevel::C4MainPipeline
            | ConstellationLevel::C5SelfHealing
            | ConstellationLevel::C6EvolutionLoop => {
                vec![Crimson, Indigo, Obsidian, Golden, Alabaster]
            }
        }
    }

    /// Runeword 名称随 constellation level 变化
    fn runeword_name_for_level(level: ConstellationLevel) -> &'static str {
        match level {
            ConstellationLevel::C0Compile => "Fehu",
            ConstellationLevel::C1UnitTest => "Uruz",
            ConstellationLevel::C2IntegrationTest => "Thurisaz",
            ConstellationLevel::C3Benchmark => "Ansuz",
            ConstellationLevel::C4MainPipeline => "Raidho",
            ConstellationLevel::C5SelfHealing => "Kaunan",
            ConstellationLevel::C6EvolutionLoop => "Scry",
        }
    }

    /// 更新 runeword_config (根据当前 constellation level 自动计算槽位和 runeword 名称)
    pub fn update_runeword_config(&mut self) {
        self.runeword_config.sockets = self.compute_rune_sockets();
        self.runeword_config.runeword = Self::runeword_name_for_level(self.constellation).to_string();
        self.runeword_config.scry_enabled = matches!(
            self.constellation,
            ConstellationLevel::C4MainPipeline
                | ConstellationLevel::C5SelfHealing
                | ConstellationLevel::C6EvolutionLoop
        );
        self.runeword_config.last_promotion_level = self.constellation;
    }

    /// 检查是否为 L0 Primitive
    pub fn is_primitive(&self) -> bool {
        self.layer == NodeLayer::L0Primitive
    }

    /// 检查是否为 Composite
    pub fn is_composite(&self) -> bool {
        matches!(self.layer, NodeLayer::L1Composite | NodeLayer::L2Orchestrator)
    }

    /// 检查是否为 Constellation
    pub fn is_constellation(&self) -> bool {
        matches!(self.layer, NodeLayer::L3DomainService | NodeLayer::L4Application | NodeLayer::L4Cognition)
    }
}