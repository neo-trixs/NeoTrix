//! CLI: neotrix-capability 子命令

use nt_term_viz::tree::{sibling_prefixes, TreeStyle};
use crate::node::{CapabilityNode, ConstellationLevel, Domain, NodeLayer};
use crate::registry::{CapabilityTreeRegistry, RegistryError};
use crate::evolution::{EvolutionAction, EvolutionEngine, EvolutionPlan};
use clap::{Parser, Subcommand};
use serde_json;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "neotrix-capability", version, about = "NeoTrix Capability Tree CLI")]
pub struct CapabilityCli {
    #[command(subcommand)]
    pub command: Commands,

    /// 注册表文件路径
    #[arg(long, default_value = ".neotrix/capability_registry.json")]
    pub registry: PathBuf,

    /// 当前 cycle 标识
    #[arg(long, default_value = "auto")]
    pub cycle: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 显示能力树 (ASCII/Mermaid)
    Tree {
        /// 输出格式
        #[arg(long, default_value = "ascii")]
        format: TreeFormat,
        /// 仅显示指定域
        #[arg(long)]
        domain: Option<String>,
        /// 仅显示指定层级
        #[arg(long)]
        layer: Option<String>,
        /// 仅显示指定星座等级
        #[arg(long)]
        constellation: Option<String>,
    },

    /// 萌芽: 创建新节点
    Bud {
        /// 节点 ID (domain::module::name)
        #[arg(long)]
        id: String,
        /// 领域
        #[arg(long)]
        domain: String,
        /// 层级 (L0/L1/L2/L3/L4)
        #[arg(long, default_value = "L0")]
        layer: String,
        /// 提供的能力标签 (逗号分隔)
        #[arg(long)]
        provides: String,
        /// 备注
        #[arg(long)]
        note: String,
    },

    /// 嫁接: 折叠分散实现到目标节点
    Graft {
        /// 目标节点 ID
        #[arg(long)]
        target: String,
        /// 被折叠的节点 ID 列表 (逗号分隔)
        #[arg(long)]
        folded: String,
        /// 备注
        #[arg(long)]
        note: String,
    },

    /// 修剪: 标记废弃/删除
    Prune {
        /// 节点 ID
        #[arg(long)]
        id: String,
        /// 原因
        #[arg(long)]
        reason: String,
        /// 强制删除 (无 dependents 时)
        #[arg(long)]
        force: bool,
    },

    /// 成熟晋升
    Mature {
        /// 节点 ID
        #[arg(long)]
        id: String,
        /// 生产接线证据 (file:line 描述生产消费路径, C1→C2 必填)
        #[arg(long)]
        wiring: Option<String>,
        /// 显式确认证据门禁已通过 (C2→C3 及以上)
        #[arg(long)]
        evidence: bool,
    },

    /// 强化: 吸收经验强化既有节点 (R-P42)
    Strengthen {
        /// 节点 ID
        #[arg(long)]
        id: String,
        /// 强化备注 (吸收的经验)
        #[arg(long)]
        note: String,
    },

    /// 异花授粉: 跨域共享
    CrossPollinate {
        /// 共享节点 ID
        #[arg(long)]
        shared: String,
        /// 域 A
        #[arg(long)]
        domain_a: String,
        /// 域 B
        #[arg(long)]
        domain_b: String,
        /// 备注
        #[arg(long)]
        note: String,
    },

    /// 建立依赖边: from 依赖 to
    Link {
        /// 依赖方节点 ID
        #[arg(long)]
        from: String,
        /// 被依赖节点 ID
        #[arg(long)]
        to: String,
    },

    /// 自动扫描并建议演化计划
    Scan {
        /// 执行建议的计划
        #[arg(long)]
        apply: bool,
    },

    /// 查询节点详情
    Get {
        /// 节点 ID
        id: String,
    },

    /// 列出节点
    List {
        /// 按域过滤
        #[arg(long)]
        domain: Option<String>,
        /// 按层级过滤
        #[arg(long)]
        layer: Option<String>,
        /// 按星座等级过滤
        #[arg(long)]
        constellation: Option<String>,
        /// 仅显示废弃
        #[arg(long)]
        deprecated: bool,
        /// 仅显示孤儿
        #[arg(long)]
        orphans: bool,
    },

    /// 统计信息
    Stats,

    /// 导出注册表
    Export {
        /// 输出文件
        #[arg(long)]
        output: Option<PathBuf>,
        /// 格式
        #[arg(long, default_value = "json")]
        format: ExportFormat,
    },

    /// 导入注册表
    Import {
        /// 输入文件
        input: PathBuf,
    },

    /// 验证注册表 (检查循环依赖、层级跨度等)
    Validate,

    /// 契约审计 (P1): 报告缺失 input/output_schema + fallback_chain 的节点
    Contracts,

    /// 成熟度真相反查 (E2): 报告声称 Constellation > 证据支撑的虚标节点。
    /// 默认只读; --apply 将虚标节点降标到证据支撑等级并写回 (可逆转)。
    AuditMaturity {
        /// 执行降标写回 (默认仅报告)
        #[arg(long)]
        apply: bool,
        /// 存在虚标节点时以非零码退出 (供 CI 门使用, 2026-09-27 新增)
        #[arg(long)]
        strict: bool,
    },

    /// 最短路径路由: 计算目标能力的最优依赖链 (LoopX 吸收: 流程节点最优解)
    Route {
        /// 目标能力标签 (如 websearch) 或节点 ID
        target: String,
        /// 指定起点节点 ID (默认: 自动找最近 primitive)
        #[arg(long)]
        from: Option<String>,
        /// 指定终点节点 ID (默认: 目标能力的最优 provider)
        #[arg(long)]
        to: Option<String>,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum TreeFormat {
    Ascii,
    Mermaid,
    Json,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ExportFormat {
    Json,
    Mermaid,
    Dot,
}

impl CapabilityCli {
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut registry = self.load_registry()?;

        match &self.command {
            Commands::Tree { format, domain, layer, constellation } => {
                self.cmd_tree(&registry, format, domain, layer, constellation)?;
            }
            Commands::Bud { id, domain, layer, provides, note } => {
                self.cmd_bud(&mut registry, id, domain, layer, provides, note)?;
            }
            Commands::Graft { target, folded, note } => {
                self.cmd_graft(&mut registry, target, folded, note)?;
            }
            Commands::Prune { id, reason, force } => {
                self.cmd_prune(&mut registry, id, reason, *force)?;
            }
            Commands::Mature { id, wiring, evidence } => {
                self.cmd_mature(&mut registry, id, wiring.as_deref(), *evidence)?;
            }
            Commands::Strengthen { id, note } => {
                self.cmd_strengthen(&mut registry, id, note)?;
            }
            Commands::CrossPollinate { shared, domain_a, domain_b, note } => {
                self.cmd_cross_pollinate(&mut registry, shared, domain_a, domain_b, note)?;
            }
            Commands::Link { from, to } => {
                self.cmd_link(&mut registry, from, to)?;
            }
            Commands::Scan { apply } => {
                self.cmd_scan(&mut registry, *apply)?;
            }
            Commands::Get { id } => {
                self.cmd_get(&registry, id)?;
            }
            Commands::List { domain, layer, constellation, deprecated, orphans } => {
                self.cmd_list(&registry, domain, layer, constellation, *deprecated, *orphans)?;
            }
            Commands::Stats => {
                self.cmd_stats(&registry)?;
            }
            Commands::Export { output, format } => {
                self.cmd_export(&registry, output, format)?;
            }
            Commands::Import { input } => {
                self.cmd_import(input.clone())?;
            }
            Commands::Validate => {
                self.cmd_validate(&registry)?;
            }
            Commands::Contracts => {
                self.cmd_contracts(&registry);
            }
            Commands::AuditMaturity { apply, strict } => {
                self.cmd_audit_maturity(&mut registry, *apply, *strict)?;
            }
            Commands::Route { target, from, to } => {
                self.cmd_route(&registry, target, from.as_deref(), to.as_deref())?;
            }
        }

        self.save_registry(&registry)?;
        Ok(())
    }

    fn load_registry(&self) -> Result<CapabilityTreeRegistry, Box<dyn std::error::Error>> {
        if self.registry.exists() {
            let content = fs::read_to_string(&self.registry)?;
            let mut reg = CapabilityTreeRegistry::new();
            match serde_json::from_str::<crate::registry::RegistryExport>(&content) {
                Ok(export) => {
                    // 两阶段加载: 先全量注册 (延迟依赖警告, 允许前向声明),
                    // 再统一 validate_dependencies — 消除 JSON 数组顺序导致的误报。
                    reg.set_defer_dep_warnings(true);
                    for node in export.nodes {
                        reg.register(node).map_err(|e| format!("Failed to register node: {}", e))?;
                    }
                    reg.validate_dependencies();
                    for (from, to) in export.edges {
                        // 外部消费者容错: 边的端点可能不在注册表中 (如 nt_io_neocodex::build_request 等外部模块)
                        // 这些是外部消费者引用, 非树内依赖, 跳过并警告而非阻塞加载
                        if !reg.nodes.contains_key(&from) || !reg.nodes.contains_key(&to) {
                            eprintln!("[capability_tree] skip edge {} -> {} (external consumer, not in registry)", from, to);
                            continue;
                        }
                        reg.add_dependency(&from, &to).map_err(|e| format!("Failed to add edge: {}", e))?;
                    }
                    reg.set_defer_dep_warnings(false);
                    // 保留经验驱动迭代目标 (distill 蒸馏写入, scan --apply 消费)
                    reg.experience_targets = export.experience_targets;
                }
                Err(parse_err) => {
                    // ⛔⛔ 2026-10-06 P0：**只在文件确实是老 schema 时**才允许走迁移。
                    //
                    // 原代码是 `Err(_)` ⇒ **任何**解析失败都被当成「老 schema」。
                    // 实测后果：已提交的新-schema 文件因缺 `kind` 解析失败 ⇒
                    // 迁移出 **0 节点** ⇒ `save_registry` 写回 41 个 roadmap 节点
                    // ⇒ **318 节点 / 43 边一次性销毁**，而退出码是 0（伪装成成功）。
                    //
                    // 判别「是不是老 schema」用**结构**而非「解析成功与否」：
                    // 老 schema 顶层是 `domains` 形且**无 `nodes` 键**。
                    // 新 schema 文件一旦损坏 ⇒ **响亮报错并拒绝继续**，
                    // 因为此时任何"迁移"都等于用空注册表覆盖真实数据。
                    let looks_legacy = Self::looks_like_legacy_schema(&content);
                    if !looks_legacy {
                        return Err(format!(
                            "注册表解析失败，且**不是**老 schema ⇒ 拒绝迁移（否则会销毁数据）。\n\
                             原始错误: {parse_err}\n\
                             文件: {}\n\
                             修法: 修好该文件；**不要**靠迁移绕过。",
                            self.registry.display()
                        )
                        .into());
                    }
                    // 老 schema 文件（domains 形，无 nodes）：内存迁移。
                    // 先落一次性备份，下一次写命令 save_registry 即转正新 schema。
                    reg = CapabilityTreeRegistry::migrate_legacy(&content).map_err(|e| {
                        format!("registry 既非新 schema 也非老 schema: {e}")
                    })?;
                    let bak = self.registry.with_extension("json.bak-legacy");
                    if !bak.exists() {
                        let _ = fs::copy(&self.registry, &bak);
                    }
                    eprintln!(
                        "[capability_tree] 老 schema 已内存迁移（{} 节点，{} 经验目标），备份 {:?}，下次写入转正",
                        reg.nodes.len(),
                        reg.experience_targets.len(),
                        bak
                    );
                }
            }
            // Durable 覆盖层: 合并提交的 overlay, 使手动写入在基础重新生成后仍生效。
            if let Some(ov) = CapabilityTreeRegistry::load_overlay_file(&self.overlay_path()) {
                reg.merge_overlay(&ov);
            }
            // 架构演进路线图 18 模块批量注册 (R-P100) — 幂等
            let _ = crate::roadmap::register_from_default_path(&mut reg);
            Ok(reg)
        } else {
            let mut reg = CapabilityTreeRegistry::new();
            if let Some(ov) = CapabilityTreeRegistry::load_overlay_file(&self.overlay_path()) {
                reg.merge_overlay(&ov);
            }
            // 架构演进路线图 18 模块批量注册 (R-P100) — 幂等
            let _ = crate::roadmap::register_from_default_path(&mut reg);
            Ok(reg)
        }
    }

    /// 覆盖层路径 — 与注册表同目录的 `capability_overrides.json` (提交进 git,
    /// 不被 .gitignore 屏蔽), 承载手动 durable 写入 (bud/strengthen/graft/...)。
    fn overlay_path(&self) -> std::path::PathBuf {
        self.registry
            .parent()
            .map(|p| p.join("capability_overrides.json"))
            .unwrap_or_else(|| std::path::PathBuf::from("capability_overrides.json"))
    }

    /// 规范序列化：递归把**所有对象**的 key 按字节序排好，再出字符串。
    ///
    /// ## 为什么必须有（实测缺陷，非推测）
    ///
    /// `CapabilityNode::metadata` 是 `HashMap<String, serde_json::Value>`，
    /// 而工作区 `Cargo.toml:31` 开了 serde_json 的 `preserve_order`。
    /// ⇒ 序列化**照抄 HashMap 的迭代顺序**，而 Rust 的 `HashMap` 用
    /// `RandomState` ⇒ **每个进程内同一个 map 的迭代顺序都不同**。
    ///
    /// 实测后果（2026-10-06 取证）：`.neotrix/capability_registry.json`
    /// 与 `capability_overrides.json` 两个**已跟踪**文件，每次 CLI 运行
    /// 都产生 706 增 / 706 删的 diff，而**语义差异 0 条**、318 个节点
    /// 逐字段完全一致 —— 纯粹是 key 顺序抖动。
    ///
    /// ⇒ 它让主工作树**永久脏**，`nt_worktree_gate.sh check` 于是反复报
    ///   「主树未提交改动不在任何提交里」，而那条报警**每次都是假的**。
    ///   一个恒假的门比没有门更危险：它训练人忽略报警。
    ///
    /// ## 为什么在**写盘边界**修，而不是把 `metadata` 改成 `BTreeMap`
    ///
    /// 后者是类型级修法（更彻底），但 `CapabilityNode` 被 core / fusion /
    /// neobot 多处构造与读取，改类型会外溢到这些 crate —— 而共享树上
    /// 另有窗口在改同样的文件。写盘边界是**收口点**：已证实 cli.rs 是
    /// 这两个文件的**唯一写者**（registry.rs 的 565/606/615 只读）。
    /// ⇒ 一处改动即根治，且不扩大爆炸半径。
    ///
    /// ## 只排 key，**不动数组顺序**
    ///
    /// 数组元素顺序是数据本身（`requires`/`depends_on` 有语义），排序会
    /// 改变内容。对象 key 无序 ⇒ 排序是恒等变换。
    /// ⛔ **判别「是不是老 schema」的唯一入口**（抽成函数是为了可测）。
    ///
    /// ## 为什么必须抽出来
    ///
    /// 它起初是内联在 `load_registry` 里的一行闭包，而我第一版测试在
    /// **测试里重写了一遍同样的判别** ⇒ 变异验证时把生产代码改成恒真，
    /// **测试依然全绿**（实测）⇒ 零区分力。
    /// ⇒ 判别逻辑必须**只有一份**，测试直接调它。
    ///
    /// 判据是**结构**（顶层 `domains` 存在且 `nodes` 不存在），而不是
    /// 「解析成功与否」—— 后者正是 P0 里把损坏文件误判成老 schema 的原因。
    fn looks_like_legacy_schema(content: &str) -> bool {
        serde_json::from_str::<serde_json::Value>(content)
            .ok()
            .and_then(|v| v.as_object().cloned())
            .is_some_and(|o| !o.contains_key("nodes") && o.contains_key("domains"))
    }

    fn canonical_json<T: serde::Serialize>(value: &T) -> Result<String, serde_json::Error> {
        let v = serde_json::to_value(value)?;
        serde_json::to_string_pretty(&v)
    }

    fn save_registry(&self, registry: &CapabilityTreeRegistry) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = self.registry.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = Self::canonical_json(&registry.export())?;
        fs::write(&self.registry, &content)?;
        // Durable 层: 把完整注册表镜像写入提交的 overlay (capability_overrides.json),
        // 使手动 durable 写入在基础被重新生成后仍生效。overlay 与基础文件同步,
        // 加载时 overlay 节点优先合并 (merge_overlay), 故手动变更永不被覆盖丢弃。
        // 2026-09-30: 原为两处 `let _ =`（create_dir_all + write）后无条件 `Ok(())`。
        // 上方注释承诺「手动变更永不被覆盖丢弃」—— 写失败时 overlay 没落盘，
        // 手动变更在基础重生成后即丢失，而调用方拿到 Ok。**注释的承诺被代码违反。**
        // 本函数本就返回 Result，且 3 行之上主文件写入即用 `?` —— 照既有纪律改。
        let overlay = self.overlay_path();
        if let Some(parent) = overlay.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("overlay 目录不可用 {}: {e}", parent.display()))?;
        }
        fs::write(&overlay, &content)
            .map_err(|e| format!("overlay 落盘失败 {}: {e}", overlay.display()))?;
        Ok(())
    }

    fn parse_domain(&self, s: &str) -> Result<Domain, Box<dyn std::error::Error>> {
        match s.to_uppercase().as_str() {
            "CORE" | "NT-CORE" => Ok(Domain::Core),
            "MIND" | "NT-MIND" => Ok(Domain::Mind),
            "MEMORY" | "NT-MEMORY" => Ok(Domain::Memory),
            "WORLD" | "NT-WORLD" => Ok(Domain::World),
            "ACT" | "NT-ACT" => Ok(Domain::Act),
            "SHIELD" | "NT-SHIELD" => Ok(Domain::Shield),
            "IO" | "NT-IO" => Ok(Domain::Io),
            "META" | "NT-META" => Ok(Domain::Meta),
            "NEXUS" | "NT-NEXUS" => Ok(Domain::Nexus),
            "GOVERNANCE" | "NT-GOVERNANCE" => Ok(Domain::Governance),
            "REPAIR" | "NT-REPAIR" => Ok(Domain::Repair),
            _ => Err(format!("Unknown domain: {}", s).into()),
        }
    }

    fn parse_layer(&self, s: &str) -> Result<NodeLayer, Box<dyn std::error::Error>> {
        match s.to_uppercase().as_str() {
            "L0" => Ok(NodeLayer::L0Primitive),
            "L1" => Ok(NodeLayer::L1Composite),
            "L2" => Ok(NodeLayer::L2Orchestrator),
            "L2W" => Ok(NodeLayer::L2World),
            "L3" => Ok(NodeLayer::L3DomainService),
            "L3M" => Ok(NodeLayer::L3Memory),
            "L4" => Ok(NodeLayer::L4Application),
            "L4C" => Ok(NodeLayer::L4Cognition),
            "L5" => Ok(NodeLayer::L5Conscious),
            "L6" => Ok(NodeLayer::L6Self),
            "L7" => Ok(NodeLayer::L7Capability),
            "L8" => Ok(NodeLayer::L8Autonomic),
            _ => Err(format!("Unknown layer: {}", s).into()),
        }
    }

    fn parse_constellation(&self, s: &str) -> Result<u8, Box<dyn std::error::Error>> {
        let s = s.to_uppercase();
        if let Some(digits) = s.strip_prefix('C') {
            let n: u8 = digits.parse()?;
            if n <= 6 { Ok(n) } else { Err("Constellation must be C0-C6".into()) }
        } else {
            s.parse().map_err(|_| "Invalid constellation".into())
        }
    }

    fn cmd_tree(
        &self,
        registry: &CapabilityTreeRegistry,
        format: &TreeFormat,
        domain: &Option<String>,
        layer: &Option<String>,
        constellation: &Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut nodes: Vec<_> = registry.nodes.values().collect();

        if let Some(d) = domain {
            let dom = self.parse_domain(d)?;
            nodes.retain(|n| n.domain == dom);
        }
        if let Some(l) = layer {
            let lay = self.parse_layer(l)?;
            nodes.retain(|n| n.layer == lay);
        }
        if let Some(c) = constellation {
            let c = self.parse_constellation(c)?;
            nodes.retain(|n| n.constellation as u8 == c);
        }

        match format {
            TreeFormat::Ascii => self.print_ascii_tree(&nodes),
            TreeFormat::Mermaid => self.print_mermaid_tree(&nodes),
            TreeFormat::Json => println!("{}", serde_json::to_string_pretty(&nodes)?),
        }
        Ok(())
    }

    fn print_ascii_tree(&self, nodes: &[&CapabilityNode]) {
        // 按域分组打印
        use std::collections::HashMap;
        let mut by_domain: HashMap<Domain, Vec<_>> = HashMap::new();
        for n in nodes {
            by_domain.entry(n.domain).or_default().push(n);
        }

        // ⚠️ 2026-10-02 修正（**顺序不确定** + **硬编码框线字符**）：
        //
        // ① 原代码 `for (domain, nodes) in by_domain` 直接遍历 **HashMap**
        //    ⇒ **输出顺序不确定** ⇒ 同一命令两次运行结果不同
        //    ⇒ 能力树是**架构视图**，无法 diff 就失去意义。
        //    （`Domain` 只 derive 了 `Eq/Hash`，**没有 `Ord`**
        //      ⇒ 不能直接 `sort()`，改按 `as_str()` 排序，顺序稳定且可预期。）
        //    ⓘ `by_layer` 的外层循环本来就是固定数组 `[L0Primitive, ...]`
        //      ⇒ 层级顺序**已经**是确定的，无需改。
        //
        // ② 原代码对**每一项**都用 `├─`（含最后一项）⇒ 末项没有 `└─`
        //    ⇒ 视觉上每项后面都跟着兄弟节点。改用 `tree_connector`。
        let mut domains: Vec<Domain> = by_domain.keys().copied().collect();
        domains.sort_by_key(|d| d.as_str());

        for domain in domains {
            println!("{}", domain);
            let empty: Vec<&&CapabilityNode> = Vec::new();
            let nodes = by_domain.get(&domain).unwrap_or(&empty);
            // 按层级分组
            let mut by_layer: HashMap<NodeLayer, Vec<&CapabilityNode>> = HashMap::new();
            for n in nodes {
                by_layer.entry(n.layer).or_default().push(n);
            }
            for layer in [NodeLayer::L0Primitive, NodeLayer::L1Composite, NodeLayer::L2Orchestrator, NodeLayer::L3DomainService, NodeLayer::L4Application] {
                if let Some(layer_nodes) = by_layer.get(&layer) {
                    println!("  {} ({})", layer.as_str(), layer_nodes.len());
                    // `sibling_prefixes` 把「谁最后」收进原语，
                    // 避免每个站点各写一遍 `i + 1 == len`（易错）。
                    let prefixes = sibling_prefixes(layer_nodes.len(), TreeStyle::Unicode);
                    for (i, n) in layer_nodes.iter().enumerate() {
                        let dep_mark = if n.deprecated { " [DEPRECATED]" } else { "" };
                        // 缩进 4 列 + 树连线；连线占 4 列 ⇒ 合计 8 列
                        println!("    {}{} [{}] deps={} dependents={}{}",
                            prefixes[i], n.id, n.constellation.as_str(),
                            n.requires.len(), n.dependents.len(), dep_mark);
                    }
                }
            }
        }
    }

    fn print_mermaid_tree(&self, nodes: &[&CapabilityNode]) {
        println!("```mermaid");
        println!("graph TD");
        for n in nodes {
        let shape = match n.layer {
            NodeLayer::L0Primitive => "(()",
            NodeLayer::L1Composite | NodeLayer::L2Orchestrator => "(())",
            NodeLayer::L3DomainService | NodeLayer::L4Application => "((()))",
            _ => "((()))",
        };
            let color = match n.constellation as u8 {
                0 => "fill:#ffcccc",
                1 => "fill:#ffe0cc",
                2 => "fill:#ffffcc",
                3 => "fill:#ccffcc",
                4 => "fill:#cceeff",
                5 => "fill:#ddccff",
                6 => "fill:#eeccff",
                _ => "fill:#ffffff",
            };
            println!("  {}{}[{} {}]{}", 
                n.id.replace("::", "_").replace("-", "_"),
                shape,
                n.id.split("::").last().unwrap_or(&n.id),
                n.constellation.as_str(),
                shape.chars().rev().collect::<String>());
            println!("  style {} {}", n.id.replace("::", "_").replace("-", "_"), color);
            
            for req in &n.requires {
                println!("  {} --> {}", n.id.replace("::", "_").replace("-", "_"), req.replace("::", "_").replace("-", "_"));
            }
        }
        println!("```");
    }

    fn cmd_bud(
        &self,
        registry: &mut CapabilityTreeRegistry,
        id: &str,
        domain: &str,
        layer: &str,
        provides: &str,
        note: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let domain = self.parse_domain(domain)?;
        let layer = self.parse_layer(layer)?;
        let provides: Vec<String> = provides.split(',').map(|s| s.trim().to_string()).collect();

        let plan = EvolutionEngine::new(registry).plan_bud(
            id.to_string(), domain, provides, layer, note.to_string(),
        );
        EvolutionEngine::new(registry).execute(plan)?;
        println!("Budded: {}", id);
        Ok(())
    }

    fn cmd_graft(
        &self,
        registry: &mut CapabilityTreeRegistry,
        target: &str,
        folded: &str,
        note: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let folded_nodes: Vec<String> = folded.split(',').map(|s| s.trim().to_string()).collect();

        let plan = EvolutionEngine::new(registry).plan_graft(
            target.to_string(), folded_nodes, note.to_string(),
        );
        EvolutionEngine::new(registry).execute(plan)?;
        println!("Grafted into: {}", target);
        Ok(())
    }

    fn cmd_prune(
        &self,
        registry: &mut CapabilityTreeRegistry,
        id: &str,
        reason: &str,
        force: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let plan = EvolutionEngine::new(registry).plan_prune(id.to_string(), reason.to_string());
        EvolutionEngine::new(registry).execute(plan)?;
        
        if force {
            match registry.remove(id) {
                Ok(_) => println!("Pruned and removed: {}", id),
                // execute(Prune) 已删无 dependents 节点（见 evolution.rs），
                // 此处再删必中 NotFound——视为已完成而非失败，否则 save 跑不成。
                Err(RegistryError::NotFound(_)) => println!("Pruned and removed: {}", id),
                Err(RegistryError::CircularDependency(_, _)) => {
                    println!("Pruned (deprecated, has dependents): {}", id)
                }
                Err(e) => return Err(e.into()),
            }
        } else {
            println!("Pruned (deprecated): {}", id);
        }
        Ok(())
    }

    fn cmd_mature(&self, registry: &mut CapabilityTreeRegistry, id: &str, wiring: Option<&str>, evidence: bool) -> Result<(), Box<dyn std::error::Error>> {
        // 写入晋升证据 (D16 门禁): C1→C2 需 wiring_evidence; C2+ 需 evidence_gated
        if let Some(node) = registry.get_mut(id) {
            if let Some(w) = wiring {
                node.metadata.insert("wiring_evidence".into(), serde_json::Value::String(w.to_string()));
            }
            if evidence {
                node.metadata.insert("evidence_gated".into(), serde_json::Value::String("passed".into()));
            }
            if node.constellation == ConstellationLevel::C1UnitTest && wiring.is_none() {
                return Err(format!(
                    "mature {} requires --wiring '<file:line> production wiring evidence' for C1→C2 (D16 gate)",
                    id
                ).into());
            }
            if node.constellation >= ConstellationLevel::C2IntegrationTest && !evidence {
                return Err(format!(
                    "mature {} requires --evidence for C2+ promotion (benchmark/pipeline/self-healing proof, D16 gate)",
                    id
                ).into());
            }
        }
        let plan = EvolutionEngine::new(registry).plan_mature(id.to_string());
        EvolutionEngine::new(registry).execute(plan)?;
        if let Some(node) = registry.get(id) {
            println!("Matured: {} -> {}", id, node.constellation.as_str());
        }
        Ok(())
    }

    fn cmd_strengthen(&self, registry: &mut CapabilityTreeRegistry, id: &str, note: &str) -> Result<(), Box<dyn std::error::Error>> {
        if registry.get(id).is_none() {
            return Err(format!("Node '{}' not found", id).into());
        }
        let plan = EvolutionEngine::new(registry).plan_strengthen(id.to_string(), note.to_string());
        EvolutionEngine::new(registry).execute(plan)?;
        println!("Strengthened: {} <- {}", id, note);
        Ok(())
    }

    fn cmd_cross_pollinate(
        &self,
        registry: &mut CapabilityTreeRegistry,
        shared: &str,
        domain_a: &str,
        domain_b: &str,
        note: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let domain_a = self.parse_domain(domain_a)?;
        let domain_b = self.parse_domain(domain_b)?;

        let plan = EvolutionEngine::new(registry).plan_cross_pollinate(
            shared.to_string(), domain_a, domain_b, note.to_string(),
        );
        EvolutionEngine::new(registry).execute(plan)?;
        println!("Cross-pollinated: {} between {} and {}", shared, domain_a, domain_b);
        Ok(())
    }

    fn cmd_link(
        &self,
        registry: &mut CapabilityTreeRegistry,
        from: &str,
        to: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        registry.add_dependency(from, to)?;
        println!("Linked: {} -> {}", from, to);
        Ok(())
    }

    fn cmd_scan(&self, registry: &mut CapabilityTreeRegistry, apply: bool) -> Result<(), Box<dyn std::error::Error>> {
        let engine = EvolutionEngine::new(registry);
        let mut plans = engine.auto_scan(&self.cycle);

        // 经验驱动迭代目标: 消费 distill 写入的 experience_targets 区
        // (经验升维闭环: 蒸馏经验 → experience_targets → 能力树 Strengthen/Bud 计划)
        plans.extend(self.experience_target_plans(registry)?);

        if plans.is_empty() {
            println!("No evolution actions suggested.");
            return Ok(());
        }

        println!("Suggested evolution plans for cycle {}:", self.cycle);
        for (i, plan) in plans.iter().enumerate() {
            println!("  {}. {}", i + 1, plan.rationale);
            for action in &plan.actions {
                println!("     {:?}", action);
            }
        }

        if apply {
            for plan in plans {
                let mut engine = EvolutionEngine::new(registry);
                if let Err(e) = engine.execute(plan) {
                    // 单计划失败不中断: 记录并继续 (幂等容错, 防重复 id 等已存在错误阻断整批)
                    eprintln!("[capability_tree] plan failed (skipped): {}", e);
                }
            }
            // 已消费的经验目标清空 (防重复执行累积)
            registry.experience_targets.clear();
            println!("Applied all plans.");
        }
        Ok(())
    }

    /// 读取蒸馏写入的 experience_targets (capability_registry.json) 并生成迭代计划。
    /// 闭环: distill_promote_to_capability 写入 → 此处消费 → 能力树 Strengthen/Bud 执行。
    pub(crate) fn experience_target_plans(
        &self,
        registry: &CapabilityTreeRegistry,
    ) -> Result<Vec<EvolutionPlan>, Box<dyn std::error::Error>> {
        let mut plans = Vec::new();
        let targets = &registry.experience_targets;
        if targets.is_empty() {
            return Ok(plans);
        }
        // 去重: 同域同标签只生成一个 Bud/Strengthen 计划 (防止重复 id 注册失败中断 apply)
        let mut already_planned: std::collections::HashSet<String> = std::collections::HashSet::new();
        for t in targets {
            let Some(domain_s) = t.get("domain").and_then(|d| d.as_str()) else { continue };
            let Some(signal) = t.get("signal").and_then(|s| s.as_f64()) else { continue };
            let Some(rationale) = t.get("rationale").and_then(|r| r.as_str()) else { continue };
            let domain = match Domain::parse(domain_s) {
                Some(d) => d,
                None => continue,
            };
            let capability_tag = t.get("capability").and_then(|c| c.as_str()).unwrap_or("").to_string();
            if capability_tag.is_empty() {
                continue;
            }
            // 意识体觉醒目标 (consciousness:: 前缀): 去前缀后按普通能力参与映射,
            // 在对应域 (如 NT-META) Bud/Strengthen 能力节点 — 修复 60 条 META targets 永不消费的缺陷。
            let capability_tag = capability_tag.strip_prefix("consciousness::").unwrap_or(&capability_tag).to_string();
            if capability_tag.is_empty() {
                continue;
            }
            if !already_planned.insert(format!("{}::{}", domain_s, capability_tag)) {
                continue;
            }
            // 找域内提供该标签的现有节点 → Strengthen; 缺失 → Bud
            let candidates: Vec<&CapabilityNode> = registry
                .by_domain(domain)
                .into_iter()
                .filter(|n| n.provides.iter().any(|p| p == &capability_tag) && !n.deprecated)
                .collect();
            if let Some(target) = candidates.first() {
                plans.push(EvolutionPlan {
                    cycle: self.cycle.clone(),
                    actions: vec![EvolutionAction::Strengthen {
                        node_id: target.id.clone(),
                        note: format!("{} | signal={:.2}", rationale, signal),
                    }],
                    rationale: format!("经验驱动: 强化 {} | {}", capability_tag, rationale),
                });
            } else {
                let new_id = format!("exp::{}::{}", domain.as_str().to_lowercase(), capability_tag);
                // 同名 exp:: 节点已存在 (含 deprecated): 不重复 Bud。
                // 该能力已沉淀为真实模块节点时由经验蒸馏切换目标, deprecated 占位不应复活。
                if registry.get(&new_id).is_some() {
                    continue;
                }
                plans.push(EvolutionPlan {
                    cycle: self.cycle.clone(),
                    actions: vec![EvolutionAction::Budding {
                        new_node_id: new_id,
                        domain,
                        provides: vec![capability_tag.clone()],
                        layer: crate::node::NodeLayer::L0Primitive,
                        note: format!("经验驱动新节点: {}", rationale),
                    }],
                    rationale: format!("经验驱动: 新建 {} | {}", capability_tag, rationale),
                });
            }
        }
        Ok(plans)
    }

    fn cmd_get(&self, registry: &CapabilityTreeRegistry, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(node) = registry.get(id) {
            println!("{}", serde_json::to_string_pretty(node)?);
        } else {
            eprintln!("Node not found: {}", id);
        }
        Ok(())
    }

    fn cmd_list(
        &self,
        registry: &CapabilityTreeRegistry,
        domain: &Option<String>,
        layer: &Option<String>,
        constellation: &Option<String>,
        deprecated: bool,
        orphans: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut nodes: Vec<_> = registry.nodes.values().collect();

        if let Some(d) = domain {
            let dom = self.parse_domain(d)?;
            nodes.retain(|n| n.domain == dom);
        }
        if let Some(l) = layer {
            let lay = self.parse_layer(l)?;
            nodes.retain(|n| n.layer == lay);
        }
        if let Some(c) = constellation {
            let c = self.parse_constellation(c)?;
            nodes.retain(|n| n.constellation as u8 == c);
        }
        if deprecated {
            nodes.retain(|n| n.deprecated);
        }
        if orphans {
            nodes.retain(|n| n.dependents.is_empty() && !n.is_constellation());
        }

        for n in nodes {
            let dep_mark = if n.deprecated { " [DEP]" } else { "" };
            println!("{} [{}] {}{} deps={} dependents={}",
                n.id, n.constellation.as_str(), n.layer.as_str(), dep_mark, n.requires.len(), n.dependents.len());
        }
        Ok(())
    }

    fn cmd_stats(&self, registry: &CapabilityTreeRegistry) -> Result<(), Box<dyn std::error::Error>> {
        let stats = registry.stats();
        println!("{}", serde_json::to_string_pretty(&stats)?);
        Ok(())
    }

    fn cmd_export(
        &self,
        registry: &CapabilityTreeRegistry,
        output: &Option<PathBuf>,
        format: &ExportFormat,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let export = registry.export();
        let content = match format {
            // 同 save_registry：canonical 化 ⇒ 导出可 diff、可复现。
            ExportFormat::Json => Self::canonical_json(&export)?,
            ExportFormat::Mermaid => {
                let mut out = String::from("```mermaid\ngraph TD\n");
                for (from, to) in &export.edges {
                    out.push_str(&format!("  {} --> {}\n", from.replace("::", "_").replace("-", "_"), to.replace("::", "_").replace("-", "_")));
                }
                out.push_str("```\n");
                out
            }
            ExportFormat::Dot => {
                let mut out = String::from("digraph capability_tree {\n");
                for (from, to) in &export.edges {
                    out.push_str(&format!("  \"{}\" -> \"{}\";\n", from, to));
                }
                out.push_str("}\n");
                out
            }
        };

        if let Some(path) = output {
            fs::write(path, content)?;
            println!("Exported to {:?}", output);
        } else {
            println!("{}", content);
        }
        Ok(())
    }

    fn cmd_import(&self, input: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(input)?;
        let export: crate::registry::RegistryExport = serde_json::from_str(&content)?;
        
        // 这里简化: 实际应合并到现有注册表
        println!("Imported {} nodes, {} edges", export.nodes.len(), export.edges.len());
        Ok(())
    }

    fn cmd_validate(&self, registry: &CapabilityTreeRegistry) -> Result<(), Box<dyn std::error::Error>> {
        let mut errors = 0;
        
        // 检查循环依赖
        if registry.has_cycles() {
            eprintln!("ERROR: Circular dependency detected in registry");
            errors += 1;
        } else {
            println!("OK: No circular dependencies");
        }

        // 检查层级跨度
        for (from_id, node) in &registry.nodes {
            for req in &node.requires {
                if let Some(req_node) = registry.get(req) {
                    let from_layer = node.layer as u8;
                    let to_layer = req_node.layer as u8;
                    if to_layer > from_layer + 1 {
                        eprintln!("WARN: {} (L{}) depends on {} (L{}) - layer span > 1", from_id, from_layer, req, to_layer);
                    }
                }
            }
        }

        // 检查孤儿
        let orphans = registry.orphan_nodes();
        if !orphans.is_empty() {
            println!("INFO: {} orphan nodes (no dependents, not constellation)", orphans.len());
        }

        // 检查过期
        let stale = registry.stale_nodes(3);
        if !stale.is_empty() {
            println!("INFO: {} stale nodes (C0/C1 for 3+ cycles)", stale.len());
        }

        if errors == 0 {
            println!("Validation passed.");
        }
        Ok(())
    }

    /// 契约审计 (P1): 报告缺失 input/output_schema + fallback_chain 的节点。
    /// 履约率 = 合规节点 / 总节点。只读审计, 不阻塞 (既有节点向后兼容)。
    fn cmd_contracts(&self, registry: &CapabilityTreeRegistry) {
        let violations = registry.contract_violations();
        let compliance = registry.contract_compliance();
        println!(
            "契约履约率: {:.1}% ({} / {} 节点)",
            compliance * 100.0,
            registry.nodes.len() - violations.len(),
            registry.nodes.len()
        );
        if violations.is_empty() {
            println!("OK: 全部节点已声明 input_schema / output_schema / fallback_chain 契约");
            return;
        }
        println!("\n缺失契约节点 ({}):", violations.len());
        for (id, missing) in violations.iter().take(40) {
            println!("  - {}: {}", id, missing.join(", "));
        }
        if violations.len() > 40 {
            println!("  ... 其余 {} 个省略 (共 {})", violations.len() - 40, violations.len());
        }
    }

    /// 成熟度真相反查 (E2 虚标治理): 报告并可选降标虚标节点。
    fn cmd_audit_maturity(
        &self,
        registry: &mut CapabilityTreeRegistry,
        apply: bool,
        strict: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let findings = registry.maturity_audit();
        if findings.is_empty() {
            println!("OK: 无成熟度虚标 (声称值均被证据支撑)");
            return Ok(());
        }
        let mut by_domain: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for f in &findings {
            // TODO 4.2: 把「未解析」带进 CI 输出 —— supported 只是**能证明的
            // 地板**, 不加此轴会被读成「真实上限就是这么大」。
            println!(
                "  {}  claimed={} supported={} ({}) [{}]",
                f.id, f.claimed.as_str(), f.supported.as_str(), f.domain.as_str(),
                f.epistemic.label()
            );
            *by_domain.entry(f.domain.as_str().to_string()).or_insert(0) += 1;
        }
        println!("\n虚标节点总数: {}", findings.len());
        for (d, c) in by_domain.iter() {
            println!("  {}: {}", d, c);
        }
        if apply {
            let n = registry.demote_mislabeled(&self.cycle);
            println!("\n已降标 {} 个节点到证据支撑等级 (可逆转: 补 evidence 后可 re-mature)", n);
            // 即使已修复写回, strict 仍以**发现前**的状态判定: 门禁要拦的是
            // "仓库里存在虚标" 这一事实, --apply 只是修复手段而非豁免。
        } else {
            println!("\n(只读报告; 加 --apply 执行降标写回)");
        }
        if strict {
            // CI 门: 虚标存在即失败。此前本命令恒返回 Ok(()) —— 机制齐备却
            // 无法被任何流水线引用, 于是自愈路径 (demote_mislabeled) 从不触发。
            return Err(format!(
                "成熟度虚标: {} 个节点声称等级高于证据支撑 (--apply 可自动降标, 可逆转)",
                findings.len()
            )
            .into());
        }
        Ok(())
    }

    /// 最短路径路由 (LoopX 吸收: 流程节点最优解)。
    /// 目标可以是能力标签 (自动选最优 provider) 或节点 ID。
    fn cmd_route(
        &self,
        registry: &CapabilityTreeRegistry,
        target: &str,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 目标解析: 若 target 是能力标签 → 找最优 provider; 否则视为节点 ID
        let target_node = if registry.get(target).is_some() {
            target.to_string()
        } else {
            match registry.optimal_provider(target) {
                Some(sp) => {
                    println!("[route] capability '{}' → optimal provider: {}", target, sp.path[0]);
                    println!("[route]   path: {} (hops={}, cost={:.2})", sp.path.join(" → "), sp.hops, sp.cost);
                    sp.path[0].clone()
                }
                None => {
                    eprintln!("ERROR: target '{}' is neither a node nor a provided capability", target);
                    return Ok(());
                }
            }
        };

        // 显式 from/to 路由
        match (from, to) {
            (Some(f), Some(t)) => {
                match registry.optimal_path_between(f, t) {
                    Some(sp) => {
                        println!("[route] {} → {}: {} (hops={}, cost={:.2})", f, t, sp.path.join(" → "), sp.hops, sp.cost);
                    }
                    None => eprintln!("ERROR: no path from '{}' to '{}'", f, t),
                }
            }
            (Some(f), None) => {
                match registry.optimal_path_between(f, &target_node) {
                    Some(sp) => {
                        println!("[route] {} → {}: {} (hops={}, cost={:.2})", f, target_node, sp.path.join(" → "), sp.hops, sp.cost);
                    }
                    None => eprintln!("ERROR: no path from '{}' to '{}'", f, target_node),
                }
            }
            _ => {
                // 默认: 目标到最近 primitive 的最优依赖链
                match registry.shortest_path_to_primitive(&target_node) {
                    Some(sp) => {
                        println!("[route] {} → primitive: {} (hops={}, cost={:.2})", target_node, sp.path.join(" → "), sp.hops, sp.cost);
                    }
                    None => eprintln!("ERROR: '{}' has no path to any primitive", target_node),
                }
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod canonical_json_tests {
    use super::CapabilityCli;

    /// 递归断言 `Value` 里**每个对象**的 key 都已升序。
    fn assert_keys_sorted(v: &serde_json::Value, path: &str) {
        match v {
            serde_json::Value::Object(map) => {
                let keys: Vec<String> = map.keys().cloned().collect();
                let mut sorted = keys.clone();
                sorted.sort();
                assert_eq!(keys, sorted, "路径 {path}: 对象 key 未排序 ⇒ 输出会随 HashMap 漂移");
                for (k, val) in map {
                    assert_keys_sorted(val, &format!("{path}/{k}"));
                }
            }
            serde_json::Value::Array(items) => {
                for (i, it) in items.iter().enumerate() {
                    assert_keys_sorted(it, &format!("{path}[{i}]"));
                }
            }
            _ => {}
        }
    }

    /// ⭐⭐⭐ **canonical_json 的输出对象 key 必须全有序。**
    ///
    /// ## 修复机制（实测得出，与最初设想不同）
    ///
    /// 最初我额外写了一个 `sort_json_keys` 递归排序，并以为它在干活。
    /// **实测证明它是死代码**：把 `canonical_json` 改成
    /// `to_value(...)` + `to_string_pretty(&v)`、把 `sort_json_keys`
    /// 掏成空操作，输出**依然是有序的**。
    ///
    /// 原因：本 crate 的 `Cargo.toml` 写的是 `serde_json = "1.0"`
    /// （**没有** `preserve_order` feature，尽管工作区根声明了它）
    /// ⇒ `serde_json::Value::Object` 是 `BTreeMap` ⇒ **`to_value` 这一步
    /// 本身就完成排序**。
    ///
    /// ⇒ 真正的修复是「**不要直接序列化结构体，改为经 `Value` 中转**」：
    /// 直接 `to_string_pretty(&export)` 会照抄 `metadata: HashMap` 的
    /// 迭代顺序，而 Rust 的 `HashMap` 每进程重新播种 ⇒ **同一输入 5 次
    /// 写盘产出 5 个不同文件**（实测 5 个互不相同的 md5）。
    ///
    /// ⛔ 因此本 crate 若将来启用 `preserve_order`，本修复会**静默失效**
    ///   ⇒ 唯一能抓住它的是 `scripts/ops/neobot-check-registry-determinism.sh`
    ///   （跨进程跑真二进制并比对 md5）—— 单进程测试**抓不到**这个缺陷，
    ///   因为同一进程内 HashMap 迭代序是稳定的。
    #[test]
    fn canonical_json_输出key全有序() {
        let v = serde_json::json!({
            "zeta": 1,
            "alpha": {"z": 1, "a": 2},
            "mid": [{"q": 1, "b": 2}],
        });
        let out = CapabilityCli::canonical_json(&v).expect("序列化应成功");
        let parsed: serde_json::Value = serde_json::from_str(&out).expect("输出应是合法 JSON");
        assert_keys_sorted(&parsed, "$");

        let ia = out.find("\"alpha\"").expect("alpha 应出现");
        let im = out.find("\"mid\"").expect("mid 应出现");
        let iz = out.find("\"zeta\"").expect("zeta 应出现");
        assert!(ia < im && im < iz, "顶层 key 应 alpha<mid<zeta，实际 {ia}/{im}/{iz}");
        let ia2 = out.find("\"a\"").expect("嵌套 a 应出现");
        let iz2 = out.find("\"z\"").expect("嵌套 z 应出现");
        assert!(ia2 < iz2, "嵌套对象 key 也应排序");
    }

    /// ⭐⭐⭐ **数组顺序必须原样保留**（不得被排序）。
    ///
    /// `requires` 之类字段的数组顺序是**数据本身**。「顺手把数组也排了」
    /// 会静默改变语义 ⇒ 用测试钉死这条边界。
    #[test]
    fn 数组顺序不被排序() {
        let v = serde_json::json!(["zebra", "apple", "mango"]);
        let out = CapabilityCli::canonical_json(&v).expect("序列化应成功");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&out).expect("应可解析"),
            v,
            "数组顺序是数据，canonical 化不得改变它"
        );
        let iz = out.find("zebra").expect("zebra");
        let ia = out.find("apple").expect("apple");
        let im = out.find("mango").expect("mango");
        assert!(iz < ia && ia < im, "数组应保持原序 zebra/apple/mango，实际 {iz}/{ia}/{im}");
    }
}

#[cfg(test)]
mod registry_roundtrip_tests {
    use super::CapabilityCli;
    use crate::registry::CapabilityTreeRegistry;

    /// ⭐⭐⭐⭐⭐ **缺 `kind` 的注册表必须能加载，且往返零丢失。**
    ///
    /// ## 2026-10-06 P0 数据丢失的回归锁
    ///
    /// `CapabilityNode::kind` 曾是唯一没有 `#[serde(default)]` 的语义字段，
    /// 而已提交的 `.neotrix/capability_registry.json` 里 **318 个节点全部
    /// 没有这个键** ⇒ `from_str::<RegistryExport>` 整体失败 ⇒
    /// `load_registry` 的 `Err(_)` 把它当成「老 schema」⇒ 迁移出 **0 节点**
    /// ⇒ `save_registry`（`run()` 末尾**无条件**执行）写回 41 个 roadmap
    /// 节点 ⇒ **318 节点 / 43 边一次性销毁，而退出码是 0**（实测 3/3 复现）。
    ///
    /// ## 判据为什么是「往返不丢节点」而不是「能解析」
    ///
    /// 缺陷的杀伤力全在**保存时覆盖** ⇒ 只断言「能解析」锁不住它。
    #[test]
    fn 缺kind的旧形态注册表_能解析且往返零丢失() {
        let raw = serde_json::json!({
            "nodes": [{
                "id": "exp::legacy::no_kind",
                "domain": "act",
                "layer": "l0primitive",
                "constellation": "c1unittest",
                "provides": ["legacy"],
                "requires": []
            }],
            "edges": [],
            "experience_targets": []
        });
        let export: crate::registry::RegistryExport = serde_json::from_value(raw)
            .unwrap_or_else(|e| panic!("★ 旧形态 JSON 应能解析（修复前报 {e}）⇒ kind 的 serde default 未生效"));
        assert_eq!(export.nodes.len(), 1);

        let mut reg = CapabilityTreeRegistry::new();
        for n in export.nodes {
            reg.register(n).expect("登记应成功");
        }
        let out = reg.export();
        assert_eq!(out.nodes.len(), 1, "★ 往返后节点数变了 ⇒ 静默截断回归");
        assert_eq!(out.nodes[0].id, "exp::legacy::no_kind");
        assert_eq!(
            out.nodes[0].kind,
            crate::node::CapabilityKind::Skill,
            "★ 缺省 kind 应为 Skill（市场 fail-closed 的依据，见 Default impl）"
        );
    }

    /// ⭐⭐⭐⭐⭐ **真实已提交注册表：解析 + 往返零丢失。**
    ///
    /// P0 的**最忠实**回归锁 —— 直接吃出事的那份文件。
    /// ⛔ 文件不存在则跳过：单测不该依赖工作区布局。
    #[test]
    fn 真实注册表文件_解析并往返零丢失() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.neotrix/capability_registry.json");
        let Ok(content) = std::fs::read_to_string(&path) else {
            eprintln!("跳过：{} 不在工作区", path.display());
            return;
        };
        let export: crate::registry::RegistryExport = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("★ 真实注册表解析失败（这正是 P0 的成因）: {e}"));
        assert!(export.nodes.len() > 100, "真实注册表不该只有 {} 个节点", export.nodes.len());

        let expected = export.nodes.len();
        let mut reg = CapabilityTreeRegistry::new();
        for n in export.nodes {
            reg.register(n).expect("登记应成功");
        }
        let out = reg.export();
        assert_eq!(out.nodes.len(), expected, "★ 往返后节点数变了 ⇒ 静默截断回归");
    }

    /// ⭐⭐⭐ **非老 schema 的坏文件不得被判为可迁移。**
    ///
    /// 缺陷机制：`Err(_)` 把**任何**解析失败都当成「老 schema」。
    /// 判别用**结构**（顶层有 `nodes` 键 ⇒ 是新 schema 的损坏文件）。
    #[test]
    fn 新schema坏文件不被误判为老schema() {
        let broken = serde_json::json!({"nodes": "这不是数组", "edges": []});
        assert!(
            !CapabilityCli::looks_like_legacy_schema(&broken.to_string()),
            "★ 含 nodes 键的文件不得走迁移（否则空注册表会覆盖真实数据）"
        );
    }

    /// ⭐⭐ **真正的老 schema（domains 形、无 nodes）仍须判为可迁移。**
    ///
    /// 反向锁：防止把「消除破坏性回退」做成「彻底不许迁移」。
    #[test]
    fn 老schema仍判为可迁移() {
        let legacy = serde_json::json!({"domains": {"Mind": ["a", "b"]}});
        assert!(
            CapabilityCli::looks_like_legacy_schema(&legacy.to_string()),
            "★ domains 形且无 nodes 应判为老 schema"
        );
    }
}
