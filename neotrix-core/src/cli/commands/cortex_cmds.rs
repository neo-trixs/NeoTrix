//! Cortex-Brain 外置大脑命令 — 查看 / 注册冷存储离线档案与 E8 因果图。
//! 外置大脑 (/Volumes/NeoTrixBrain) 是 114 GB 离线档案 (ZIM/pmtiles/wikipedia + causal_graph.json)。
//! 本命令使其对 live KB 可见、可连接 (Dark Forest：连接而非惰性堆积)。

use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;

use rusqlite::Connection;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_resource_ingest::{
    corpus_archive_path, migrate_cortex_corpus, prune_cortex_orphans, register_cortex_brain,
};
use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_cortex_sync::{
    digest_sample, export_delta, report_lineage,
};

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
            "lineage" => Self::lineage(),
            "sync" => Self::sync(&args[1..]),
            "digest" => Self::digest(&args[1..]),
            "prune" => Self::prune(args.contains(&"--force".to_string())),
            "corpus" => Self::corpus(&args[1..]),
            "help" | "--help" | "-h" => CommandOutput::ok(
                "用法:\n  /cortex status    查看外置大脑挂载与档案概览\n  /cortex register  将外置大脑注册进 live KB (已挂载时)\n  /cortex causal    显示 E8 因果图 (causal_graph.json) 摘要\n  /cortex prune     回收盘上已消失归档对应的悬空 KB 节点 (dry-run)\n  /cortex prune --force  真正删除上述悬空节点\n  /cortex corpus status   查看 68GB corpus 本地/外置副本状态\n  /cortex corpus migrate  [--force] 将 64GB corpus 拷到外置大脑冷存档 (dry-run)\n  /cortex lineage  查看外置大脑各节点的血缘 (sha256/上次同步/循环/方向)\n  /cortex sync [--dry-run]  把 live KB 的 SEAL 吸收增量回写外置大脑 (双向边界 G1)\n  /cortex digest [--top-k N] [--domain D] [path]  有界采样外置 corpus, 把冷节点激活进 live KB (Phase 1)",
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
        // local 68GB corpus cold archive
        let home = std::env::var("HOME").unwrap_or_default();
        let corpus = std::path::Path::new(&home)
            .join(".neotrix")
            .join("knowledge-archive-corpus-20260825.db");
        lines.push(if corpus.exists() {
            let sz = std::fs::metadata(&corpus).map(|m| m.len()).unwrap_or(0);
            format!(
                "  本地 68GB corpus: 存在 ({:.1} GB) — 冷存档 (live KB 的超集，不合并)",
                sz as f64 / 1e9
            )
        } else {
            "  本地 68GB corpus: (缺失)".to_string()
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

    fn prune(force: bool) -> CommandOutput {
        let root = std::path::Path::new(CORTEX_ROOT);
        if !root.exists() {
            return CommandOutput::err(&format!("外置大脑未挂载: {CORTEX_ROOT} — 拒绝 prune (否则会误删全部 zim 节点)。"));
        }
        let conn = match Connection::open(kb_path()) {
            Ok(c) => c,
            Err(e) => return CommandOutput::err(&format!("无法打开 live KB: {e}")),
        };
        let (sources, nodes) = match prune_cortex_orphans(&conn, root, !force) {
            Ok(v) => v,
            Err(e) => return CommandOutput::err(&format!("prune 失败: {e}")),
        };
        if sources == 0 {
            return CommandOutput::ok("无悬空 zim 源 — KB 中所有 zimid 节点在盘上均有对应归档，无需回收。");
        }
        if force {
            CommandOutput::ok(&format!(
                "已回收 {sources} 个悬空 zim 源 / {nodes} 个文章节点 (含其边)。"
            ))
        } else {
            CommandOutput::warn(&format!(
                "[dry-run] 将回收 {sources} 个悬空 zim 源 / {nodes} 个文章节点。加 --force 真正删除。"
            ))
        }
    }

    fn corpus(args: &[String]) -> CommandOutput {
        let sub = args.first().map(|s| s.as_str()).unwrap_or("status");
        let root = std::path::Path::new(CORTEX_ROOT);
        match sub {
            "migrate" => {
                if !root.exists() {
                    return CommandOutput::err(&format!(
                        "外置大脑未挂载: {CORTEX_ROOT} — 无法迁移。"
                    ));
                }
                let conn = match Connection::open(kb_path()) {
                    Ok(c) => c,
                    Err(e) => return CommandOutput::err(&format!("无法打开 live KB: {e}")),
                };
                let force = args.contains(&"--force".to_string());
                match migrate_cortex_corpus(&conn, root, !force) {
                    Ok(msg) => {
                        if force {
                            CommandOutput::ok(&msg)
                        } else {
                            CommandOutput::warn(&msg)
                        }
                    }
                    Err(e) => CommandOutput::err(&format!("迁移失败: {e}")),
                }
            }
            "status" | _ => {
                let home = std::env::var("HOME").unwrap_or_default();
                let local = std::path::Path::new(&home)
                    .join(".neotrix")
                    .join("knowledge-archive-corpus-20260825.db");
                let ext = root.join("knowledge-archive-corpus-20260825.db");
                let mut lines = vec!["68GB corpus 冷存档状态:".to_string()];
                lines.push(if local.exists() {
                    let sz = std::fs::metadata(&local).map(|m| m.len()).unwrap_or(0);
                    format!("  本地源副本: 存在 ({:.1} GB)", sz as f64 / 1e9)
                } else {
                    "  本地源副本: (缺失)".to_string()
                });
                lines.push(if ext.exists() {
                    let sz = std::fs::metadata(&ext).map(|m| m.len()).unwrap_or(0);
                    format!("  外置大脑副本: 存在 ({:.1} GB)", sz as f64 / 1e9)
                } else {
                    "  外置大脑副本: (缺失) — 运行 /cortex corpus migrate --force".to_string()
                });
                CommandOutput::ok(&lines.join("\n"))
            }
        }
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

    fn lineage() -> CommandOutput {
        let conn = match Connection::open(kb_path()) {
            Ok(c) => c,
            Err(e) => return CommandOutput::err(&format!("无法打开 live KB: {e}")),
        };
        match report_lineage(&conn) {
            Ok(reps) if reps.is_empty() => {
                CommandOutput::warn("live KB 中无 cortex_brain 节点 — 先 /cortex register。")
            }
            Ok(reps) => {
                let mut lines = vec!["外置大脑血缘 (lineage):".to_string()];
                for r in reps {
                    lines.push(format!(
                        "  {} [{}] dir={} sha={} synced={:?} cycle={:?}",
                        r.url,
                        r.kind,
                        r.direction,
                        r.external_sha256.unwrap_or_else(|| "-".to_string()),
                        r.last_synced_at,
                        r.last_seal_cycle,
                    ));
                }
                CommandOutput::ok(&lines.join("\n"))
            }
            Err(e) => CommandOutput::err(&format!("lineage 失败: {e}")),
        }
    }

    fn sync(args: &[String]) -> CommandOutput {
        let root = std::path::Path::new(CORTEX_ROOT);
        if !root.exists() {
            return CommandOutput::err(&format!("外置大脑未挂载: {CORTEX_ROOT} — 拒绝回写。"));
        }
        let dry = args.contains(&"--dry-run".to_string());
        let conn = match Connection::open(kb_path()) {
            Ok(c) => c,
            Err(e) => return CommandOutput::err(&format!("无法打开 live KB: {e}")),
        };
        match export_delta(&conn, root, dry) {
            Ok(r) => {
                let msg = format!(
                    "[{}] 回写 {} 条 experience 增量 → {}/working/nt_cortex_delta.jsonl (since={}, version_ok={})",
                    if dry { "dry-run" } else { "sync" },
                    r.entries,
                    CORTEX_ROOT,
                    r.since,
                    r.version_ok
                );
                if dry {
                    CommandOutput::warn(&msg)
                } else {
                    CommandOutput::ok(&msg)
                }
            }
            Err(e) => CommandOutput::err(&format!("sync 失败: {e}")),
        }
    }

    fn digest(args: &[String]) -> CommandOutput {
        let mut top_k = 50usize;
        let mut domain: Option<String> = None;
        let mut path_arg: Option<String> = None;
        let mut it = args.iter();
        while let Some(a) = it.next() {
            match a.as_str() {
                "--top-k" => {
                    if let Some(v) = it.next() {
                        top_k = v.parse().unwrap_or(50);
                    }
                }
                "--domain" => {
                    if let Some(v) = it.next() {
                        domain = Some(v.clone());
                    }
                }
                other if !other.starts_with('-') => path_arg = Some(other.to_string()),
                _ => {}
            }
        }
        let corpus = match path_arg {
            Some(p) => std::path::PathBuf::from(p),
            None => match corpus_archive_path() {
                Some(p) => p,
                None => {
                    return CommandOutput::err(
                        "外置大脑 corpus 未挂载, 且无位置参数路径 (用法: /cortex digest [path])",
                    )
                }
            },
        };
        let conn = match Connection::open(kb_path()) {
            Ok(c) => c,
            Err(e) => return CommandOutput::err(&format!("无法打开 live KB: {e}")),
        };
        match digest_sample(&conn, &corpus, top_k, domain.as_deref()) {
            Ok(r) => CommandOutput::ok(&format!(
                "digest: 取样 {} 个冷节点 → 激活 {} 个进 live KB (已 live 跳过 {} 个), {} bytes, 覆盖类型 {:?}",
                r.sampled, r.activated, r.already_live, r.bytes, r.node_types
            )),
            Err(e) => CommandOutput::err(&format!("digest 失败: {e}")),
        }
    }
}
