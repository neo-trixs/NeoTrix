//! Cortex-Brain 外置大脑命令 — 查看 / 注册冷存储离线档案与 E8 因果图。
//! 外置大脑 (/Volumes/NeoTrixBrain) 是 114 GB 离线档案 (ZIM/pmtiles/wikipedia + causal_graph.json)。
//! 本命令使其对 live KB 可见、可连接 (Dark Forest：连接而非惰性堆积)。

use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;

use rusqlite::Connection;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_resource_ingest::register_cortex_brain;

const CORTEX_ROOT: &str = "/Volumes/NeoTrixBrain";
const CORTEX_CAUSAL: &str = "/Volumes/NeoTrixBrain/working/causal_graph.json";

fn kb_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    PathBuf::from(home).join(".neotrix").join("knowledge.db")
}

pub struct CortexCmd;

impl CliCommand for CortexCmd {
    fn name(&self) -> &str { "cortex" }
    fn aliases(&self) -> Vec<&str> { vec!["brain-external", "neotrix-brain"] }
    fn description(&self) -> &str {
        "Cortex-Brain 外置大脑：查看/注册冷存储离线档案与 E8 因果图"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let sub = args.get(0).map(|s| s.as_str()).unwrap_or("status");
        match sub {
            "status" => Self::status(),
            "register" => Self::register(),
            "causal" => Self::causal(),
            "help" | "--help" | "-h" => CommandOutput::ok(
                "用法:\n  /cortex status    查看外置大脑挂载与档案概览\n  /cortex register  将外置大脑注册进 live KB (已挂载时)\n  /cortex causal    显示 E8 因果图 (causal_graph.json) 摘要",
            ),
            other => CommandOutput::err(&format!("未知子命令: {other} (试试 /cortex status)")),
        }
    }
}

impl CortexCmd {
    fn status() -> CommandOutput {
        let root = std::path::Path::new(CORTEX_ROOT);
        if !root.exists() {
            return CommandOutput::warn(&format!(
                "外置大脑未挂载: {CORTEX_ROOT} — 当前为惰性离线档案，未接入 live KB。挂载后执行 /cortex register。"
            ));
        }
        let mut lines = vec![format!("外置大脑已挂载: {CORTEX_ROOT}")];
        for sub in ["zim", "pmtiles", "wikipedia"] {
            let p = root.join("cortex-archive").join(sub);
            if p.exists() {
                let n = std::fs::read_dir(&p).map(|rd| rd.filter_map(|e| e.ok()).count()).unwrap_or(0);
                lines.push(format!("  cortex-archive/{sub}: {n} 文件"));
            } else {
                lines.push(format!("  cortex-archive/{sub}: (缺失)"));
            }
        }
        let causal = std::path::Path::new(CORTEX_CAUSAL);
        lines.push(if causal.exists() {
            format!("  causal_graph.json: 存在 (E8 因果图已加载)")
        } else {
            "  causal_graph.json: (缺失)".to_string()
        });
        // live KB 中的 cortex_brain 注册节点数
        if let Ok(conn) = Connection::open(kb_path()) {
            if let Ok(cnt) = conn.query_row(
                "SELECT COUNT(*) FROM nodes WHERE node_type='cortex_brain'",
                [],
                |r| r.get::<_, i64>(0),
            ) {
                lines.push(format!("  live KB 中 cortex_brain 节点: {cnt}"));
            }
        }
        CommandOutput::ok(&lines.join("\n"))
    }

    fn register() -> CommandOutput {
        let root = std::path::Path::new(CORTEX_ROOT);
        if !root.exists() {
            return CommandOutput::err(&format!("外置大脑未挂载: {CORTEX_ROOT} — 无法注册。"));
        }
        let conn = match Connection::open(kb_path()) {
            Ok(c) => c,
            Err(e) => return CommandOutput::err(&format!("无法打开 live KB: {e}")),
        };
        match register_cortex_brain(&conn, root) {
            Ok(n) if n > 0 => CommandOutput::ok(&format!(
                "已注册 {n} 个 cortex_brain 目录/因果图节点到 live KB。"
            )),
            Ok(_) => CommandOutput::warn("未找到可注册内容 (archive 目录为空或 causal_graph 缺失)。"),
            Err(e) => CommandOutput::err(&format!("注册失败: {e}")),
        }
    }

    fn causal() -> CommandOutput {
        let causal = std::path::Path::new(CORTEX_CAUSAL);
        if !causal.exists() {
            return CommandOutput::warn(&format!("外置大脑未挂载或 causal_graph.json 缺失: {CORTEX_CAUSAL}"));
        }
        match crate::core::nt_core_e8::abduction::causal_graph::CausalGraph::from_cortex_json(causal) {
            Ok(g) => {
                let summary = format!(
                    "E8 因果图: {} 节点 / {} 链路\n挂载源: {CORTEX_CAUSAL}",
                    g.nodes.len(),
                    g.edges.len()
                );
                CommandOutput::ok(&summary).with_json(serde_json::json!({
                    "nodes": g.nodes.len(),
                    "edges": g.edges.len(),
                    "path": CORTEX_CAUSAL,
                }))
            }
            Err(e) => CommandOutput::err(&format!("读取因果图失败: {e}")),
        }
    }
}
