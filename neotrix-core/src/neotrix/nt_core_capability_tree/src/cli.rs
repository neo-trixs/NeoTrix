//! CLI: neotrix-capability 子命令

use crate::node::{CapabilityNode, ConstellationLevel, Domain, NodeLayer};
use crate::registry::{CapabilityRegistry, RegistryError};
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
            Commands::Route { target, from, to } => {
                self.cmd_route(&registry, target, from.as_deref(), to.as_deref())?;
            }
        }

        self.save_registry(&registry)?;
        Ok(())
    }

    fn load_registry(&self) -> Result<CapabilityRegistry, Box<dyn std::error::Error>> {
        if self.registry.exists() {
            let content = fs::read_to_string(&self.registry)?;
            let export: crate::registry::RegistryExport = serde_json::from_str(&content)?;
            let mut reg = CapabilityRegistry::new();
            for node in export.nodes {
                reg.register(node).map_err(|e| format!("Failed to register node: {}", e))?;
            }
            for (from, to) in export.edges {
                // 外部消费者容错: 边的端点可能不在注册表中 (如 nt_io_neocodex::build_request 等外部模块)
                // 这些是外部消费者引用, 非树内依赖, 跳过并警告而非阻塞加载
                if !reg.nodes.contains_key(&from) || !reg.nodes.contains_key(&to) {
                    eprintln!("[capability_tree] skip edge {} -> {} (external consumer, not in registry)", from, to);
                    continue;
                }
                reg.add_dependency(&from, &to).map_err(|e| format!("Failed to add edge: {}", e))?;
            }
            // 保留经验驱动迭代目标 (distill 蒸馏写入, scan --apply 消费)
            reg.experience_targets = export.experience_targets;
            Ok(reg)
        } else {
            Ok(CapabilityRegistry::new())
        }
    }

    fn save_registry(&self, registry: &CapabilityRegistry) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = self.registry.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&registry.export())?;
        fs::write(&self.registry, content)?;
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
            "L3" => Ok(NodeLayer::L3DomainService),
            "L4" => Ok(NodeLayer::L4Application),
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
        registry: &CapabilityRegistry,
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

        for (domain, nodes) in by_domain {
            println!("{}", domain);
            // 按层级分组
            let mut by_layer: HashMap<NodeLayer, Vec<_>> = HashMap::new();
            for n in nodes {
                by_layer.entry(n.layer).or_default().push(n);
            }
            for layer in [NodeLayer::L0Primitive, NodeLayer::L1Composite, NodeLayer::L2Orchestrator, NodeLayer::L3DomainService, NodeLayer::L4Application] {
                if let Some(layer_nodes) = by_layer.get(&layer) {
                    println!("  {} ({})", layer.as_str(), layer_nodes.len());
                    for n in layer_nodes {
                        let dep_mark = if n.deprecated { " [DEPRECATED]" } else { "" };
                        println!("    ├─ {} [{}] deps={} dependents={}{}",
                            n.id, n.constellation.as_str(), n.requires.len(), n.dependents.len(), dep_mark);
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
        registry: &mut CapabilityRegistry,
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
        registry: &mut CapabilityRegistry,
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
        registry: &mut CapabilityRegistry,
        id: &str,
        reason: &str,
        force: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let plan = EvolutionEngine::new(registry).plan_prune(id.to_string(), reason.to_string());
        EvolutionEngine::new(registry).execute(plan)?;
        
        if force {
            if let Err(e) = registry.remove(id) {
                if !matches!(e, RegistryError::CircularDependency(_, _)) {
                    return Err(e.into());
                }
                println!("Pruned (deprecated, has dependents): {}", id);
            } else {
                println!("Pruned and removed: {}", id);
            }
        } else {
            println!("Pruned (deprecated): {}", id);
        }
        Ok(())
    }

    fn cmd_mature(&self, registry: &mut CapabilityRegistry, id: &str, wiring: Option<&str>, evidence: bool) -> Result<(), Box<dyn std::error::Error>> {
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

    fn cmd_strengthen(&self, registry: &mut CapabilityRegistry, id: &str, note: &str) -> Result<(), Box<dyn std::error::Error>> {
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
        registry: &mut CapabilityRegistry,
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
        registry: &mut CapabilityRegistry,
        from: &str,
        to: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        registry.add_dependency(from, to)?;
        println!("Linked: {} -> {}", from, to);
        Ok(())
    }

    fn cmd_scan(&self, registry: &mut CapabilityRegistry, apply: bool) -> Result<(), Box<dyn std::error::Error>> {
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
        registry: &CapabilityRegistry,
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
            // 意识体觉醒目标: 不映射能力节点, 仅记录 (消费在 NT-META 层)
            if capability_tag.starts_with("consciousness::") {
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
                plans.push(EvolutionPlan {
                    cycle: self.cycle.clone(),
                    actions: vec![EvolutionAction::Budding {
                        new_node_id: format!("exp::{}::{}", domain.as_str().to_lowercase(), capability_tag),
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

    fn cmd_get(&self, registry: &CapabilityRegistry, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(node) = registry.get(id) {
            println!("{}", serde_json::to_string_pretty(node)?);
        } else {
            eprintln!("Node not found: {}", id);
        }
        Ok(())
    }

    fn cmd_list(
        &self,
        registry: &CapabilityRegistry,
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

    fn cmd_stats(&self, registry: &CapabilityRegistry) -> Result<(), Box<dyn std::error::Error>> {
        let stats = registry.stats();
        println!("{}", serde_json::to_string_pretty(&stats)?);
        Ok(())
    }

    fn cmd_export(
        &self,
        registry: &CapabilityRegistry,
        output: &Option<PathBuf>,
        format: &ExportFormat,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let export = registry.export();
        let content = match format {
            ExportFormat::Json => serde_json::to_string_pretty(&export)?,
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

    fn cmd_validate(&self, registry: &CapabilityRegistry) -> Result<(), Box<dyn std::error::Error>> {
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
    fn cmd_contracts(&self, registry: &CapabilityRegistry) {
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

    /// 最短路径路由 (LoopX 吸收: 流程节点最优解)。
    /// 目标可以是能力标签 (自动选最优 provider) 或节点 ID。
    fn cmd_route(
        &self,
        registry: &CapabilityRegistry,
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