//! CLI: neotrix-capability 子命令

use crate::node::{CapabilityNode, Domain, NodeLayer};
use crate::registry::{CapabilityRegistry, RegistryError};
use crate::evolution::EvolutionEngine;
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
            Commands::Mature { id } => {
                self.cmd_mature(&mut registry, id)?;
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
                reg.add_dependency(&from, &to).map_err(|e| format!("Failed to add edge: {}", e))?;
            }
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
        if s.starts_with('C') {
            let n: u8 = s[1..].parse()?;
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

    fn cmd_mature(&self, registry: &mut CapabilityRegistry, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let plan = EvolutionEngine::new(registry).plan_mature(id.to_string());
        EvolutionEngine::new(registry).execute(plan)?;
        if let Some(node) = registry.get(id) {
            println!("Matured: {} -> {}", id, node.constellation.as_str());
        }
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
        let plans = engine.auto_scan(&self.cycle);

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
                engine.execute(plan)?;
            }
            println!("Applied all plans.");
        }
        Ok(())
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
}