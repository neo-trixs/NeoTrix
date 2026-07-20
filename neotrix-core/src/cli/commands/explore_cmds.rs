use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::l2_world_impl::nt_world_exploration_engine::{ExplorationEngine, ExplorationConfig};
use crate::neotrix::l8_autonomic_impl::nt_mind_knowledge_pipeline::KnowledgeAbsorptionPipeline;

pub struct ExploreCmd;

impl CliCommand for ExploreCmd {
    fn name(&self) -> &str { "/explore" }
    fn aliases(&self) -> Vec<&str> { vec!["/discover", "/ingest"] }
    fn description(&self) -> &str {
        "外部知识探索: /explore [cycle|status|absorb <url>|github <url>|search <q>]"
    }

    fn execute(&self, args: &[String], _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::neotrix::nt_mind::SelfIteratingBrain>>>) -> CommandOutput {
        let mode = args.first().map(|s| s.as_str()).unwrap_or("status");

        // Try to open KB connection for pipeline ops
        fn try_open_kb() -> Option<crate::neotrix::nt_memory_kb::KnowledgeBase> {
            crate::neotrix::nt_memory_kb::KnowledgeBase::open(None).ok()
        }
        fn try_prepare_pipeline() -> KnowledgeAbsorptionPipeline {
            let mut p = KnowledgeAbsorptionPipeline::new();
            if let Some(kb) = try_open_kb() {
                p.attach_kb(std::sync::Arc::new(kb));
            }
            p
        }

        match mode {
            "cycle" | "run" => {
                let config = ExplorationConfig::default();
                let mut engine = ExplorationEngine::new(config);
                if let Some(kb) = try_open_kb() {
                    engine.attach_kb(kb.into());
                }
                let report = engine.run_cycle();
                CommandOutput::ok(&format!(
                    "🔍 探索完成: 发现 {} | 摄入 {} | 跳过 {} | 失败 {} | KB 总计 {}",
                    report.discovered, report.ingested, report.skipped, report.failed, report.total_in_kb
                ))
            }

            "status" | "stat" => {
                let engine = ExplorationEngine::new(ExplorationConfig::default());
                let s = engine.stats();
                let p = try_prepare_pipeline();
                let pstats = p.stats();
                CommandOutput::ok(&format!(
                    "📊 探索状态:\n  探索引擎: {} 轮, {} 已发现, {} 已摄入\n  知识管道: {} 源吸收",
                    s.total_cycles, s.discovered, s.ingested_urls, pstats.total_sources
                ))
            }

            "absorb" | "ingest" => {
                let url = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if url.is_empty() {
                    return CommandOutput::err("用法: /explore absorb <url>");
                }
                let mut pipeline = try_prepare_pipeline();
                match pipeline.absorb_url(url) {
                    Ok(r) => CommandOutput::ok(&format!(
                        "📥 {}: {} 节点, {} 边 ({})",
                        r.action, r.nodes_created, r.edges_created, r.url
                    )),
                    Err(e) => CommandOutput::err(&format!("吸收失败: {}", e)),
                }
            }

            "github" => {
                let url = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if url.is_empty() {
                    return CommandOutput::err("用法: /explore github <repo_url>");
                }
                let mut pipeline = try_prepare_pipeline();
                match pipeline.absorb_github(url) {
                    Ok(r) => {
                        let mut msg = format!("📦 GitHub {}: {} 节点, {} 边", r.action, r.nodes_created, r.edges_created);
                        if let Some(ref summary) = r.distil_summary {
                            msg.push_str(&format!("\n  蒸馏: {}", summary.chars().take(120).collect::<String>()));
                        }
                        CommandOutput::ok(&msg)
                    }
                    Err(e) => CommandOutput::err(&format!("GitHub 吸收失败: {}", e)),
                }
            }

            "panorama" | "map" => {
                let mut pipeline = try_prepare_pipeline();
                match pipeline.update_panorama() {
                    Ok(p) => {
                        let types: Vec<String> = p.by_type.iter()
                            .map(|(k, v)| format!("{}: {}", k, v)).collect();
                        CommandOutput::ok(&format!(
                            "🗺️ 知识全景: {} 源 | {}\n  更新于 {}",
                            p.total_sources, types.join(" | "),
                            chrono::DateTime::from_timestamp(p.updated_at, 0)
                                .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                                .unwrap_or_else(|| "unknown".into())
                        ))
                    }
                    Err(e) => CommandOutput::err(&format!("全景更新失败: {}", e)),
                }
            }

            "recent" | "last" => {
                let pipeline = try_prepare_pipeline();
                let recent = pipeline.recent_sources(10);
                if recent.is_empty() {
                    return CommandOutput::ok("📭 暂无吸收记录");
                }
                let mut lines = vec!["📋 最近吸收:".to_string()];
                for e in recent {
                    lines.push(format!("  {} | {} | {}", e.source_type, e.title, e.url));
                }
                CommandOutput::ok(&lines.join("\n"))
            }

            _ => {
                CommandOutput::ok(
                    "🔍 外部探索命令:\n\
                    /explore status         探索状态\n\
                    /explore cycle          执行一次探索循环\n\
                    /explore absorb <url>   吸收 URL 到知识库\n\
                    /explore github <url>   吸收 GitHub 仓库 + 蒸馏\n\
                    /explore panorama       查看知识全景\n\
                    /explore recent         最近吸收记录"
                )
            }
        }
    }
}
