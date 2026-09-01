use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::core::nt_core_memory_asset::MemoryAssetKind;
use crate::l1_action::nt_memory::nt_memory_kb::{
    diff_snapshots, kb_write_guard, record_write_evidence, snapshot_from_file, snapshot_kb,
    snapshot_to_file, KnowledgeBase, KnowledgeNode, NodeType, RelationType, WriteGuardVerdict,
};
use rusqlite::Connection;
use std::collections::{HashMap, HashSet, VecDeque};

fn open_kb() -> Option<KnowledgeBase> {
    KnowledgeBase::open(None).ok()
}

fn kb_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(home).join(".neotrix").join("knowledge.db")
}

fn open_raw_conn() -> Option<Connection> {
    Connection::open(kb_path()).ok()
}

fn load_edges(conn: &Connection) -> Result<Vec<(String, String)>, String> {
    let mut stmt = conn
        .prepare("SELECT source_id, target_id FROM edges")
        .map_err(|e| format!("Failed to query edges: {}", e))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| format!("Failed to read edges: {}", e))?;
    let mut edges = Vec::new();
    for row in rows {
        edges.push(row.map_err(|e| format!("Failed to read edge row: {}", e))?);
    }
    Ok(edges)
}

fn node_title(conn: &Connection, id: &str) -> String {
    conn.query_row("SELECT title FROM nodes WHERE id = ?1", [id], |row| {
        row.get::<_, String>(0)
    })
    .unwrap_or_else(|_| id.to_string())
}

fn node_type_str(conn: &Connection, id: &str) -> String {
    conn.query_row(
        "SELECT node_type FROM nodes WHERE id = ?1",
        [id],
        |row| row.get::<_, String>(0),
    )
    .unwrap_or_else(|_| "?".to_string())
}

fn parse_usize(args: &[String], flag: &str, default: usize) -> usize {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(default)
}

fn parse_str<'a>(args: &'a [String], flag: &'a str, default: &'a str) -> &'a str {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or(default)
}

pub struct KbCmd;
impl CliCommand for KbCmd {
    fn name(&self) -> &str {
        "/kb"
    }
    fn aliases(&self) -> Vec<&str> {
        vec!["/knowledge", "/knowledge-base"]
    }
    fn description(&self) -> &str {
        "Knowledge base operations: /kb stats | /kb search <query> | /kb get <node_id> | /kb query <text> | /kb write <json> | /kb explore <node_id> | /kb find <src> <tgt> | /kb cluster | /kb central | /kb serve | /kb export <node_id> | /kb import-assets | /kb absorb-map | /kb snapshot [--out <path>] | /kb diff <snapA> [snapB] | /kb assets --kind <chat_memory|skill|llm_wiki|code_graph>"
    }
    fn is_primary(&self) -> bool { false }

    fn execute(
        &self,
        args: &[String],
        _brain: Option<
            &std::sync::Arc<tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain>>,
        >,
    ) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok(
                "Knowledge Base (KB) commands:\n\
                  /kb stats                      显示 KB 统计\n\
                  /kb search <query>             搜索知识库\n\
                  /kb get <node_id>              按 ID 取节点 (MCP read)\n\
                  /kb query <text> [--limit N]   高级查询 (混合重排, MCP read)\n\
                  /kb write <json> [--force]     受守卫的 KB 写操作 (node:create/update, edge:upsert, kv:set, node:delete, edge:delete)\n\
                  /kb explore <node_id>          查看节点详情及关联\n\
                  /kb find <src> <tgt>           查找两个节点间最短路径\n\
                  /kb cluster [--min-size 3]     社区发现分析\n\
                  /kb central [--top-k 20]       中心性分析 (PageRank)\n\
                  /kb serve [--port 8337]        启动 MCP 知识服务\n\
                  /kb export <node_id> [--format json|svg]  导出子图\n\
                  /kb import-assets [path]       导入 assets/knowledge_data.json 到 KB\n\
                  /kb import-review [path]      导入 review-findings.json 缺陷记录到 KB\n\
                  /kb absorb-map [--dry-run] [--limit N] [--types a,b] 全库本源溯源+能力映射 (R-P79)\n\
                  /kb consistency               设定一致性检查 (对标网文每卷设定检查)\n\
                  /kb axioms                    架构公理推演树 (公理→定律→模块约束)\n\
                  /kb snapshot [--out <path>]   捕获 KB 全量快照 (节点/边/统计, 默认 ~/.neotrix/snapshots/)\n\
                  /kb diff <snapA> [snapB]      比较两个快照 (缺 snapB 则对当前库); --detail <N> 列出明细\n\
                  /kb assets --kind <k>         四态记忆资产查询 (chat_memory|skill|llm_wiki|code_graph)",
            );
        }

        let sub = args[0].as_str();
        let rest = &args[1..];

        match sub {
            "stats" => cmd_stats(rest),
            "search" => cmd_search(rest),
            "get" => cmd_get(rest),
            "query" => cmd_query(rest),
            "write" => cmd_write(rest),
            "explore" => cmd_explore(rest),
            "find" => cmd_find(rest),
            "cluster" => cmd_cluster(rest),
            "central" => cmd_central(rest),
            "serve" => cmd_serve(rest),
            "export" => cmd_export(rest),
            "import-assets" => cmd_import_assets(rest),
            "import-review" => cmd_import_review(rest),
            "absorb-map" => cmd_absorb_map(rest),
            "embed" => cmd_embed(rest),
            "distill" => cmd_distill(rest),
            "ingest-asset" => cmd_ingest_asset(rest),
            "consistency" => cmd_consistency(rest),
            "axioms" => cmd_axioms(rest),
            "snapshot" => cmd_snapshot(rest),
            "diff" => cmd_diff(rest),
            "assets" => cmd_assets(rest),
            _ => CommandOutput::err(&format!(
                "未知子命令: {}. 可用: stats, search, get, query, write, explore, find, cluster, central, serve, export, import-assets, import-review, absorb-map, embed, distill, ingest-asset, consistency, axioms, snapshot, diff, assets",
                sub
            )),
        }
    }
}

/// /kb consistency — 设定一致性检查 (对标网文每卷设定检查)
fn cmd_consistency(_args: &[String]) -> CommandOutput {
    let conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };
    let mut out = String::new();
    let _ = crate::l1_action::nt_memory::nt_memory_kb::setting_consistency::check_and_report_to_string(&conn, &mut out);
    CommandOutput::ok(&out)
}

/// /kb axioms — 架构公理推演树 (公理→定律→模块约束)
fn cmd_axioms(_args: &[String]) -> CommandOutput {
    let tree = crate::core::nt_core_axiom_tree::AxiomTree::build();
    CommandOutput::ok(&tree.render())
}

/// /kb snapshot [--out <path>] — 捕获 KB 全量快照 (G5, dbx snapshot 对齐)。
fn cmd_snapshot(args: &[String]) -> CommandOutput {
    let kb = match open_kb() {
        Some(kb) => kb,
        None => return CommandOutput::err("无法打开知识库 (KnowledgeBase::open failed)"),
    };
    let snap = match snapshot_kb(&kb) {
        Ok(s) => s,
        Err(e) => return CommandOutput::err(&format!("快照捕获失败: {}", e)),
    };

    let out = if let Some(path) = args
        .iter()
        .position(|a| a == "--out")
        .and_then(|i| args.get(i + 1))
    {
        std::path::PathBuf::from(path)
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let dir = std::path::PathBuf::from(home).join(".neotrix").join("snapshots");
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        dir.join(format!("kb-{}.json", ts))
    };

    if let Err(e) = snapshot_to_file(&snap, &out) {
        return CommandOutput::err(&format!("快照落盘失败: {}", e));
    }

    let msg = format!(
        "KB 快照已捕获\n  文件:   {}\n  节点:   {}\n  边:     {}\n  指纹:   {}\n  DB:     {} bytes",
        out.display(),
        snap.nodes.len(),
        snap.edges.len(),
        snap.checksum(),
        snap.stats.db_size_bytes,
    );
    CommandOutput::ok(&msg).with_json(serde_json::json!({
        "path": out.to_string_lossy(),
        "nodes": snap.nodes.len(),
        "edges": snap.edges.len(),
        "checksum": snap.checksum(),
        "captured_at_ms": snap.captured_at_ms,
    }))
}

/// /kb assets --kind <chat_memory|skill|llm_wiki|code_graph> — 四态记忆资产查询
/// （吸收 TencentDB-Agent-Memory）。派生维度, 实时由 `MemoryAssetKind::classify` 推断。
fn cmd_assets(args: &[String]) -> CommandOutput {
    let mut kind: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--kind" {
            if i + 1 < args.len() {
                kind = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
            return CommandOutput::err("--kind 需要一个取值");
        }
        i += 1;
    }
    let Some(kind_str) = kind else {
        return CommandOutput::err("用法: /kb assets --kind <chat_memory|skill|llm_wiki|code_graph>");
    };
    let Some(k) = MemoryAssetKind::from_str(&kind_str) else {
        return CommandOutput::err(&format!("未知资产类型: {kind_str}"));
    };
    let kb = match KnowledgeBase::open(None).ok() {
        Some(kb) => kb,
        None => return CommandOutput::err("无法打开知识库 (KnowledgeBase::open failed)"),
    };
    let nodes: Vec<KnowledgeNode> = match kb.nodes_by_asset_kind(k) {
        Ok(n) => n,
        Err(e) => return CommandOutput::err(&format!("查询失败: {e}")),
    };
    let titles: Vec<String> = nodes.iter().map(|n| n.title.clone()).collect();
    let preview: Vec<String> = titles.iter().take(20).map(|t| format!("  - {t}")).collect();
    let out = format!(
        "四态资产 [{}]: 命中 {} 个节点\n{}",
        k.as_str(),
        nodes.len(),
        preview.join("\n")
    );
    CommandOutput::ok(&out).with_json(serde_json::json!({
        "kind": k.as_str(),
        "count": nodes.len(),
        "titles": titles,
    }))
}

/// /kb diff <snapA> [snapB] [--detail N] — 比较两个快照; 缺 snapB 时对当前库 (G5)。
fn cmd_diff(args: &[String]) -> CommandOutput {
    let detail = parse_usize(args, "--detail", 10);
    // 位置参数过滤: 跳过取值 flag (--detail N) 及其值, 保留纯路径。
    let mut positional: Vec<&str> = Vec::new();
    let mut skip_next = false;
    for a in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if a == "--detail" {
            skip_next = true;
            continue;
        }
        if !a.starts_with("--") {
            positional.push(a.as_str());
        }
    }
    if positional.is_empty() {
        return CommandOutput::err("用法: /kb diff <snapA.json> [snapB.json] [--detail N]");
    }

    let base = match snapshot_from_file(std::path::Path::new(positional[0])) {
        Ok(s) => s,
        Err(e) => return CommandOutput::err(&format!("读取 snapA 失败: {}", e)),
    };

    let other = if let Some(p) = positional.get(1) {
        match snapshot_from_file(std::path::Path::new(p)) {
            Ok(s) => s,
            Err(e) => return CommandOutput::err(&format!("读取 snapB 失败: {}", e)),
        }
    } else {
        let kb = match open_kb() {
            Some(kb) => kb,
            None => return CommandOutput::err("无法打开知识库 (KnowledgeBase::open failed)"),
        };
        match snapshot_kb(&kb) {
            Ok(s) => s,
            Err(e) => return CommandOutput::err(&format!("当前库快照失败: {}", e)),
        }
    };

    let diff = diff_snapshots(&base, &other);
    let mut out = format!(
        "KB diff (base {} nodes/{} edges → other {} nodes/{} edges)\n",
        base.nodes.len(), base.edges.len(), other.nodes.len(), other.edges.len()
    );
    out.push_str(&format!(
        "  nodes: +{} / -{} / changed {} | edges: +{} / -{}\n",
        diff.nodes_added.len(),
        diff.nodes_removed.len(),
        diff.nodes_changed.len(),
        diff.edges_added.len(),
        diff.edges_removed.len(),
    ));
    if diff.is_empty() {
        out.push_str("  无差异\n");
        return CommandOutput::ok(&out).with_json(serde_json::json!(diff));
    }

    let render_nodes = |items: &[crate::l1_action::nt_memory::nt_memory_kb::DiffNode], tag: &str| {
        let mut s = String::new();
        for item in items.iter().take(detail) {
            s.push_str(&format!(
                "    {} [{}] {} ({})\n",
                tag, item.node_type, item.title, item.id
            ));
            if !item.fields_changed.is_empty() {
                s.push_str(&format!("        changed: {}\n", item.fields_changed.join(", ")));
            }
        }
        if items.len() > detail {
            s.push_str(&format!("    … 还有 {} 条\n", items.len() - detail));
        }
        s
    };
    out.push_str("  新增节点:\n");
    out.push_str(&render_nodes(&diff.nodes_added, "+"));
    out.push_str("  删除节点:\n");
    out.push_str(&render_nodes(&diff.nodes_removed, "-"));
    out.push_str("  变更节点:\n");
    out.push_str(&render_nodes(&diff.nodes_changed, "~"));
    if !diff.edges_added.is_empty() {
        out.push_str("  新增边:\n");
        for e in diff.edges_added.iter().take(detail) {
            out.push_str(&format!("    + {} → {} ({})\n", e.source_id, e.target_id, e.relation_type));
        }
    }
    if !diff.edges_removed.is_empty() {
        out.push_str("  删除边:\n");
        for e in diff.edges_removed.iter().take(detail) {
            out.push_str(&format!("    - {} → {} ({})\n", e.source_id, e.target_id, e.relation_type));
        }
    }
    CommandOutput::ok(&out).with_json(serde_json::json!(diff))
}

fn cmd_embed(_args: &[String]) -> CommandOutput {
//     use crate::l1_action::nt_memory::nt_memory_kb::embed::EmbedMode;
    let kb = match KnowledgeBase::open(None) {
        Ok(kb) => kb,
        Err(e) => return CommandOutput::err(&format!("无法打开知识库: {}", e)),
    };
//     let cfg = crate::l1_action::nt_memory::nt_memory_kb::embed::EmbeddingConfig::default();
    let bundled = kb.with_embedding(cfg);
    let mode_label = match bundled.embedding_config.read().ok().and_then(|c| c.clone()).map(|c| c.mode) {
        Some(EmbedMode::Local) => "本地 hash-kernel (384-dim, 零依赖)",
        _ => "HTTP MiniLM (可选脚本 scripts/kb-embed-server.py)",
    };
    match bundled.ensure_embeddings() {
        Ok(n) => CommandOutput::ok(&format!(
            "Embedding 补跑完成: 本轮处理 {} 个待嵌节点\n\
             模式: {}\n\
             Http 服务不可用时自动降级本地 hash-kernel, 全链路零依赖可跑。\n\
             强制本地: NEOTRIX_EMBEDDING_MODE=local",
            n, mode_label
        )),
        Err(e) => CommandOutput::err(&format!("Embedding 补跑失败: {}", e)),
    }
}

/// /kb distill [--pairs N] [--epochs N] — DistilVDR 蒸馏学生训练 (R-P79 生产接线)。
/// 从 KB 已有向量采样 (q,d) 对, teacher = 余弦, 点级回归训练双塔对角学生,
/// 落盘 ~/.neotrix/distill_student.json; hybrid_search Tier 3 自动消费。
fn cmd_distill(args: &[String]) -> CommandOutput {
//     use crate::l1_action::nt_memory::nt_memory_kb::embed::load_all_embeddings;
    use crate::l1_action::nt_memory::nt_memory_kb::distill::{
        load_student, sample_contrastive_pairs, sample_training_pairs, save_student,
        train_contrastive, CONTRASTIVE_MARGIN, PointwiseDistillStudent,
    };
    let conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };
    let embeddings = match load_all_embeddings(&conn) {
        Ok(e) => e,
        Err(err) => return CommandOutput::err(&format!("读取 embeddings 失败: {}", err)),
    };
    if embeddings.len() < 2 {
        return CommandOutput::err(&format!(
            "向量不足 ({} 条), 先跑 /kb embed 或确认已有嵌入。",
            embeddings.len()
        ));
    }
    let pairs = parse_usize(args, "--pairs", 400).min(embeddings.len().saturating_mul(8));
    let epochs = parse_usize(args, "--epochs", 12).min(200);

    // 对比/列表式模式 (E5/BGE: listwise + contrastive + hard-negative)
    let contrastive = {
        let mut c = false;
        let mut i = 0;
        while i < args.len() {
            if args[i] == "--mode" {
                if args.get(i + 1).map(|v| v.as_str()) == Some("contrastive") {
                    c = true;
                }
                i += 2;
            } else if args[i] == "--contrastive" {
                c = true;
                i += 1;
            } else if args[i] == "--neg-k" {
                i += 2;
            } else {
                i += 1;
            }
        }
        c
    };
    let neg_k = parse_usize(args, "--neg-k", 4);

    if contrastive {
        let samples = sample_contrastive_pairs(&embeddings, pairs, neg_k, 0x9E37_79B9);
        let dim = embeddings[0].1.len();
        if samples.is_empty() {
            return CommandOutput::err("无法构造对比样本 (向量不足)");
        }
        let before = load_student();
        let student = train_contrastive(&samples, dim, epochs, 0.05, 0.9);
        return match save_student(&student) {
            Ok(()) => {
                let rate = |s: &PointwiseDistillStudent| {
                    let sat = samples
                        .iter()
                        .filter(|(q, p, ns)| {
                            let sp = s.score(q, p);
                            let sn =
                                ns.iter().map(|n| s.score(q, n)).sum::<f64>() / ns.len() as f64;
                            sp - sn >= CONTRASTIVE_MARGIN
                        })
                        .count();
                    sat as f64 / samples.len() as f64
                };
                let before_rate = before.as_ref().map(rate).unwrap_or(0.0);
                let after_rate = rate(&student);
                CommandOutput::ok(&format!(
                    "DistilVDR 对比蒸馏完成: 样本 {} 对 (neg_k={}), 维度 {}, 迭代 {} epoch\n\
                     间隔满足率 (gap>=margin {:.2}): 训练前 {:.3} → 训练后 {:.3}\n\
                     已落盘 ~/.neotrix/distill_student.json\n\
                     hybrid_search Tier 3 将自动消费蒸馏分数。",
                    samples.len(),
                    neg_k,
                    dim,
                    epochs,
                    CONTRASTIVE_MARGIN,
                    before_rate,
                    after_rate
                ))
            }
            Err(e) => CommandOutput::err(&format!("蒸馏落盘失败: {}", e)),
        };
    }

    let (samples, dim) = sample_training_pairs(&embeddings, pairs, 0x9E37_79B9);
    let before = load_student();
    let student = PointwiseDistillStudent::train(&samples, dim, epochs, 0.05, 0.9);
    match save_student(&student) {
        Ok(()) => {
            let before_mse = before
                .as_ref()
                .map(|s| {
                    samples
                        .iter()
                        .map(|(q, d, t)| (s.score(q, d) - t).powi(2))
                        .sum::<f64>()
                        / samples.len() as f64
                });
            let after_mse = samples
                .iter()
                .map(|(q, d, t)| (student.score(q, d) - t).powi(2))
                .sum::<f64>()
                / samples.len() as f64;
            let mut out = format!(
                "DistilVDR 蒸馏完成: 样本 {} 对, 维度 {}, 迭代 {} epoch\n\
                 MSE: 训练前 {:.6} → 训练后 {:.6}\n\
                 已落盘 ~/.neotrix/distill_student.json\n\
                 hybrid_search Tier 3 将自动消费蒸馏分数 (无学生时退化回余弦)。",
                samples.len(),
                dim,
                epochs,
                before_mse.unwrap_or(0.0),
                after_mse
            );
            if before_mse.map(|b| after_mse >= b).unwrap_or(false) {
                out.push_str("\n警告: MSE 未下降 — 向量可能过稀疏或样本不足。");
            }
            CommandOutput::ok(&out)
        }
        Err(e) => CommandOutput::err(&format!("蒸馏落盘失败: {}", e)),
    }
}

/// /kb ingest-asset <url> — ARTEX 资产图手动入口 (R-P79 生产验证)。
/// 把单个 URL 的资产层级 (root_domain→subdomain→service→endpoint) 落 KB。
fn cmd_ingest_asset(args: &[String]) -> CommandOutput {
    use crate::l2_perception::nt_world::nt_world_crawl::asset_graph::AssetGraphWriter;
    let Some(url) = args.first() else {
        return CommandOutput::err("用法: /kb ingest-asset <url>");
    };
    let kb = match KnowledgeBase::open(None) {
        Ok(kb) => kb,
        Err(e) => return CommandOutput::err(&format!("无法打开知识库: {}", e)),
    };
    match AssetGraphWriter::ingest_url(&kb, url) {
        Ok(edges) => CommandOutput::ok(&format!(
            "资产图写入完成: 新增/确认边 {} 条 (幂等, 重复 ingest 不重复计数)。",
            edges
        )),
        Err(e) => CommandOutput::err(&format!("资产图写入失败: {}", e)),
    }
}

/// 全库本源溯源 + 能力映射 (R-P79, Rust 原生取代 scripts/absorb_full_kb.py)。
/// /kb absorb-map [--dry-run] [--limit N] [--types a,b]
fn cmd_absorb_map(args: &[String]) -> CommandOutput {
    let dry_run = !args.contains(&"--apply".to_string());
    let limit = parse_usize(args, "--limit", usize::MAX);
    let types_raw = parse_str(args, "--types", "");

    let conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };
    let types: Option<Vec<String>> = if types_raw.is_empty() {
        None
    } else {
        Some(
            types_raw
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        )
    };
    let limit_opt = if limit == usize::MAX { None } else { Some(limit) };

    let (mapped, report) = match crate::l1_action::nt_memory::nt_memory_kb::map_nodes(&conn, None, types.as_deref(), limit_opt)
    {
        Ok(r) => r,
        Err(e) => return CommandOutput::err(&format!("全库溯源失败: {}", e)),
    };

    let mut lines = vec![format!("=== 全库本源溯源报告 (total {}) ===", report.total)];
    let all_sources = ["E8", "VSA", "GWT", "ConsciousnessTree", "Reality"];
    for core in all_sources {
        let cnt = report.per_source.get(core).copied().unwrap_or(0);
        let pct = if report.total > 0 {
            100 * cnt / report.total
        } else {
            0
        };
        lines.push(format!("  {:<18} {:>7} ({}%)", core, cnt, pct));
    }
    let unknown = report.unknown_source(report.total);
    lines.push(format!("  未映射本源: {}", unknown));
    lines.push(format!("  未映射能力: {}", report.unmapped.len()));
    lines.push(String::new());
    lines.push("=== 能力分布 (top 15) ===".to_string());
    let mut caps: Vec<(&String, &usize)> = report.per_cap.iter().collect();
    caps.sort_by(|a, b| b.1.cmp(a.1));
    for (cap, cnt) in caps.into_iter().take(15) {
        lines.push(format!("  {:<12} {}", cap, cnt));
    }

    if dry_run {
        lines.push(String::new());
        lines.push(format!("[absorb-map] dry-run, 未写库. 已映射 {} 条, 未映射 {} 条 (可加 --apply 写库)",
            report.per_source.values().sum::<usize>(), report.unmapped.len()));
        return CommandOutput::ok(&lines.join("\n"));
    }

    match crate::l1_action::nt_memory::nt_memory_kb::apply_mappings(&conn, &mapped) {
        Ok(n) => {
            lines.push(String::new());
            lines.push(format!("[absorb-map] 已写库: {} 条 absorbed_capability + knowledge_source", n));
            CommandOutput::ok(&lines.join("\n"))
        }
        Err(e) => CommandOutput::err(&format!("写库失败: {}", e)),
    }
}

fn cmd_import_assets(args: &[String]) -> CommandOutput {
    let path = args.first().map(|s| s.as_str()).unwrap_or("assets/knowledge_data.json");
    let path = std::path::Path::new(path);

    if !path.exists() {
        return CommandOutput::err(&format!("File not found: {}", path.display()));
    }

    let kb = match KnowledgeBase::open(None) {
        Ok(kb) => kb,
        Err(e) => return CommandOutput::err(&format!("Cannot open KB: {}", e)),
    };

    match kb.import_knowledge_assets(path) {
        Ok(report) => {
            let mut msg = format!(
                "Knowledge assets import:\n  imported: {} nodes\n  edges created: {}\n",
                report.imported, report.edges_created,
            );
            if !report.errors.is_empty() {
                msg.push_str(&format!("  errors: {}", report.errors.len()));
                for err in report.errors.iter().take(5) {
                    msg.push_str(&format!("\n    - {}", err));
                }
            }
            CommandOutput::ok(&msg)
        }
        Err(e) => CommandOutput::err(&format!("Import failed: {}", e)),
    }
}

fn cmd_import_review(_args: &[String]) -> CommandOutput {
    let path = "design/review-findings.json";
    let path = std::path::Path::new(path);

    if !path.exists() {
        return CommandOutput::err(&format!("File not found: {}", path.display()));
    }

    let kb = match KnowledgeBase::open(None) {
        Ok(kb) => kb,
        Err(e) => return CommandOutput::err(&format!("Cannot open KB: {}", e)),
    };

    match kb.import_review_findings(path) {
        Ok(report) => {
            let mut msg = format!(
                "Review findings import:\n  imported: {} defects\n",
                report.imported,
            );
            if !report.errors.is_empty() {
                msg.push_str(&format!("  errors: {}", report.errors.len()));
                for err in report.errors.iter().take(5) {
                    msg.push_str(&format!("\n    - {}", err));
                }
            }
            CommandOutput::ok(&msg)
        }
        Err(e) => CommandOutput::err(&format!("Import failed: {}", e)),
    }
}

fn cmd_stats(_args: &[String]) -> CommandOutput {
    let kb = match open_kb() {
        Some(kb) => kb,
        None => return CommandOutput::err("无法打开知识库 (KnowledgeBase::open failed)"),
    };
    match kb.stats() {
        Ok(stats) => {
            let mut out = format!(
                "KB 统计信息\n━━━━━━━━━━━━━━━\n  节点总数: {}\n  边总数:   {}\n  待爬取:   {}\n  已爬取:   {}\n  DB 大小:  {} bytes\n",
                stats.total_nodes,
                stats.total_edges,
                stats.crawl_pending,
                stats.crawl_completed,
                stats.db_size_bytes,
            );
            out.push_str("  节点类型分布:\n");
            for (t, c) in &stats.by_type {
                out.push_str(&format!("    {}: {}\n", t, c));
            }
            if !stats.by_domain.is_empty() {
                out.push_str("  域名分布 (Top 20):\n");
                for (d, c) in &stats.by_domain {
                    out.push_str(&format!("    {}: {}\n", d, c));
                }
            }
            CommandOutput::ok(&out)
        }
        Err(e) => CommandOutput::err(&format!("获取 KB 统计失败: {}", e)),
    }
}

fn cmd_search(args: &[String]) -> CommandOutput {
    if args.is_empty() {
        return CommandOutput::err("用法: /kb search <query>");
    }
    let kb = match open_kb() {
        Some(kb) => kb,
        None => return CommandOutput::err("无法打开知识库 (KnowledgeBase::open failed)"),
    };
    let query = args.join(" ");
    // Operator terminal context: Confidential clearance (keeps Secret-tier nodes
    // out unless the operator runs the full tree). Wiring the permission-aware
    // retrieval path (Onyx pattern) instead of raw search.
    let permission = crate::l1_action::nt_memory::nt_memory_kb::types::PermissionLevel::Confidential;
    match kb.search_permission_aware(&query, 10, permission) {
        Ok(results) => {
            if results.is_empty() {
                return CommandOutput::ok(&format!("未找到匹配 \"{}\" 的结果", query));
            }
            let mut out = format!("搜索 \"{}\" ({} 条):\n", query, results.len());
            for (i, r) in results.iter().enumerate() {
                let node = &r.node;
                let summary = node.summary.as_deref().unwrap_or("(无摘要)");
                let preview = if summary.len() > 100 {
                    &summary[..summary.floor_char_boundary(100)]
                } else {
                    summary
                };
                out.push_str(&format!(
                    "  {}. [{}] {} (score: {:.3})\n     {}\n",
                    i + 1,
                    node.node_type.as_str(),
                    node.title,
                    r.score,
                    preview
                ));
            }
            CommandOutput::ok(&out)
        }
        Err(e) => CommandOutput::err(&format!("搜索失败: {}", e)),
    }
}

fn cmd_explore(args: &[String]) -> CommandOutput {
    if args.is_empty() {
        return CommandOutput::err("用法: /kb explore <node_id>");
    }
    let kb = match open_kb() {
        Some(kb) => kb,
        None => return CommandOutput::err("无法打开知识库 (KnowledgeBase::open failed)"),
    };
    let node_id = &args[0];
    match kb.get_node(node_id) {
        Ok(Some(node)) => {
            let mut out = format!(
                "节点详情\n━━━━━━━━━━━━━━━\n  ID:       {}\n  类型:     {}\n  标题:     {}\n",
                node.id,
                node.node_type.as_str(),
                node.title
            );
            if let Some(ref s) = node.summary {
                out.push_str(&format!("  摘要:     {}\n", s));
            }
            if let Some(ref c) = node.content {
                let preview = if c.len() > 200 { &c[..c.floor_char_boundary(200)] } else { c };
                out.push_str(&format!("  内容预览: {}\n", preview));
            }
            if let Some(ref u) = node.url {
                out.push_str(&format!("  URL:      {}\n", u));
            }
            if let Some(ref d) = node.domain {
                out.push_str(&format!("  域名:     {}\n", d));
            }
            out.push_str(&format!(
                "  置信度:   {:.2}\n  重要性:   {:.2}\n  访问数:   {}\n",
                node.confidence, node.importance, node.access_count
            ));
            if let Ok(related) = kb.get_related(node_id, None, 10) {
                if !related.is_empty() {
                    out.push_str("  关联节点:\n");
                    for r in &related {
                        out.push_str(&format!(
                            "    [{}] {} (score: {:.3})\n",
                            r.node.node_type.as_str(),
                            r.node.title,
                            r.score
                        ));
                    }
                }
            }
            CommandOutput::ok(&out)
        }
        Ok(None) => CommandOutput::not_found(&format!("未找到节点: {}", node_id)),
        Err(e) => CommandOutput::err(&format!("查询节点失败: {}", e)),
    }
}

/// /kb get <node_id> — 按 ID 取节点 (MCP read 通道, JSON 友好)
fn cmd_get(args: &[String]) -> CommandOutput {
    if args.is_empty() {
        return CommandOutput::err("用法: /kb get <node_id>");
    }
    let kb = match open_kb() {
        Some(kb) => kb,
        None => return CommandOutput::err("无法打开知识库 (KnowledgeBase::open failed)"),
    };
    let node_id = &args[0];
    match kb.get_node(node_id) {
        Ok(Some(node)) => {
            let payload = serde_json::json!({
                "id": node.id,
                "type": node.node_type.as_str(),
                "title": node.title,
                "summary": node.summary,
                "url": node.url,
                "domain": node.domain,
                "confidence": node.confidence,
                "importance": node.importance,
                "content": node.content.as_deref().map(|c| {
                    if c.len() > 2000 { &c[..c.floor_char_boundary(2000)] } else { c }
                }),
            });
            match serde_json::to_string_pretty(&payload) {
                Ok(s) => CommandOutput::ok(&s).with_json(payload),
                Err(_) => CommandOutput::err("序列化失败"),
            }
        }
        Ok(None) => CommandOutput::not_found(&format!("未找到节点: {}", node_id)),
        Err(e) => CommandOutput::err(&format!("查询节点失败: {}", e)),
    }
}

/// /kb query <text> [--limit N] — 高级混合重排查询 (MCP read 通道)
fn cmd_query(args: &[String]) -> CommandOutput {
    if args.is_empty() {
        return CommandOutput::err("用法: /kb query <text> [--limit N]");
    }
    let limit = parse_usize(args, "--limit", 10).min(50);
    let text: Vec<&str> = args
        .iter()
        .filter(|a| !a.starts_with("--"))
        .map(|a| a.as_str())
        .collect();
    if text.is_empty() {
        return CommandOutput::err("用法: /kb query <text> [--limit N]");
    }
    let query = text.join(" ");
    let kb = match open_kb() {
        Some(kb) => kb,
        None => return CommandOutput::err("无法打开知识库 (KnowledgeBase::open failed)"),
    };
    match kb.hybrid_rerank_search(&query, limit) {
        Ok(results) => {
            if results.is_empty() {
                return CommandOutput::ok(&format!("未找到匹配 \"{}\" 的结果", query));
            }
            let mut out = format!("查询 \"{}\" ({} 条):\n", query, results.len());
            for (i, r) in results.iter().enumerate() {
                let node = &r.node;
                out.push_str(&format!(
                    "  {}. [{}] {} (score: {:.3})\n",
                    i + 1,
                    node.node_type.as_str(),
                    node.title,
                    r.score
                ));
            }
            CommandOutput::ok(&out)
        }
        Err(e) => CommandOutput::err(&format!("查询失败: {}", e)),
    }
}

/// /kb write <json> [--force] — 受守卫的 KB 写操作。
/// action ∈ node:create | node:update | edge:upsert | node:delete | edge:delete | kv:set
fn cmd_write(args: &[String]) -> CommandOutput {
    let force = args.contains(&"--force".to_string());
    let json_text = args
        .iter()
        .filter(|a| !a.starts_with("--"))
        .map(|a| a.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    if json_text.trim().is_empty() {
        return CommandOutput::err(
            "用法: /kb write <json> [--force]\n\
             例: /kb write '{\"action\":\"node:create\",\"title\":\"Rust\",\"url\":\"https://rust-lang.org\"}'",
        );
    }
    let payload: serde_json::Value = match serde_json::from_str(&json_text) {
        Ok(v) => v,
        Err(e) => return CommandOutput::err(&format!("JSON 解析失败: {}", e)),
    };
    let action = match payload.get("action").and_then(|v| v.as_str()) {
        Some(a) => a.to_string(),
        None => return CommandOutput::err("payload 缺少 action 字段"),
    };

    let kb = match open_kb() {
        Some(kb) => kb,
        None => return CommandOutput::err("无法打开知识库 (KnowledgeBase::open failed)"),
    };

    let mut verdict = kb_write_guard(&action, &payload);
    if verdict == WriteGuardVerdict::RequiresApproval && force {
        // force 视为人工审批 (operator 显式放行 Tier3/Tier4)
        verdict = WriteGuardVerdict::Allow;
    }

    match &verdict {
        WriteGuardVerdict::Allow => {}
        WriteGuardVerdict::RequiresApproval => {
            record_write_evidence(&kb, &action, &payload, &verdict, false);
            return CommandOutput::err(&format!(
                "KB 写操作需要审批 (force 未置位): {}",
                verdict.reasons().join("; ")
            ));
        }
        WriteGuardVerdict::Reject(reasons) => {
            record_write_evidence(&kb, &action, &payload, &verdict, false);
            return CommandOutput::err(&format!(
                "KB 写操作被守卫拒绝: {}",
                reasons.join("; ")
            ));
        }
    }

    let result: Result<String, String> = (|| {
        match action.as_str() {
            "node:create" => {
                let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("");
                let ntype = payload
                    .get("node_type")
                    .and_then(|v| v.as_str())
                    .map(NodeType::from_str)
                    .unwrap_or(NodeType::Concept);
                let summary = payload.get("summary").and_then(|v| v.as_str());
                let url = payload.get("url").and_then(|v| v.as_str());
                let domain = payload.get("domain").and_then(|v| v.as_str());
                let id = kb.insert_or_get_node(title, ntype, summary, url, domain)?;
                Ok(format!("节点已写入: {}", id))
            }
            "node:update" => {
                let id = payload.get("id").and_then(|v| v.as_str()).ok_or("缺少 id")?;
                let content = payload
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or("缺少 content")?;
                kb.update_node_content(id, content)?;
                Ok(format!("节点已更新: {}", id))
            }
            "edge:upsert" => {
                let src = payload.get("source_id").and_then(|v| v.as_str()).ok_or("缺少 source_id")?;
                let tgt = payload.get("target_id").and_then(|v| v.as_str()).ok_or("缺少 target_id")?;
                let rt = payload
                    .get("relation_type")
                    .and_then(|v| v.as_str())
                    .map(RelationType::from_str)
                    .unwrap_or(RelationType::Related);
                let weight = payload.get("weight").and_then(|v| v.as_f64()).unwrap_or(1.0);
                let desc = payload.get("description").and_then(|v| v.as_str());
                kb.upsert_edge(src, tgt, rt, weight, desc)?;
                Ok(format!("边已写入: {} -> {}", src, tgt))
            }
            "node:delete" | "edge:delete" => {
                let id = payload.get("id").and_then(|v| v.as_str()).ok_or("缺少 id")?;
                let deleted = if action == "node:delete" {
                    kb.delete_node(id)?
                } else {
                    kb.delete_edge(id)?
                };
                if deleted {
                    Ok(format!("已删除: {}", id))
                } else {
                    Err(format!("未找到待删除对象: {}", id))
                }
            }
            "kv:set" => {
                let ns = payload.get("namespace").and_then(|v| v.as_str()).ok_or("缺少 namespace")?;
                let key = payload.get("key").and_then(|v| v.as_str()).ok_or("缺少 key")?;
                let value = payload.get("value").map(|v| v.to_string()).ok_or("缺少 value")?;
                kb.kv_set(ns, key, &value)?;
                Ok(format!("kv 已写入: {}/{}", ns, key))
            }
            other => Err(format!("未知写操作: {}", other)),
        }
    })();

    match result {
        Ok(msg) => {
            record_write_evidence(&kb, &action, &payload, &verdict, true);
            CommandOutput::ok(&msg)
        }
        Err(e) => {
            record_write_evidence(&kb, &action, &payload, &WriteGuardVerdict::Reject(vec![e.clone()]), false);
            CommandOutput::err(&e)
        }
    }
}

fn cmd_find(args: &[String]) -> CommandOutput {
    if args.len() < 2 {
        return CommandOutput::err("用法: /kb find <source_id> <target_id> [--algo bfs]");
    }
    let source_id = &args[0];
    let target_id = &args[1];
    let _algo = parse_str(args, "--algo", "bfs");

    let conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开 KB 数据库"),
    };

    let edges = match load_edges(&conn) {
        Ok(e) => e,
        Err(e) => return CommandOutput::err(&format!("读取边失败: {}", e)),
    };

    if edges.is_empty() {
        return CommandOutput::ok("知识库中没有边，无法查找路径");
    }

    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for (src, tgt) in &edges {
        adj.entry(src.clone()).or_default().push(tgt.clone());
        adj.entry(tgt.clone()).or_default().push(src.clone());
    }

    if !adj.contains_key(source_id) {
        return CommandOutput::not_found(&format!("源节点不存在: {}", source_id));
    }
    if !adj.contains_key(target_id) {
        return CommandOutput::not_found(&format!("目标节点不存在: {}", target_id));
    }

    let mut visited: HashSet<String> = HashSet::new();
    let mut parent: HashMap<String, String> = HashMap::new();
    let mut queue: VecDeque<String> = VecDeque::new();

    visited.insert(source_id.to_string());
    queue.push_back(source_id.to_string());

    let mut found = false;
    while let Some(current) = queue.pop_front() {
        if current == *target_id {
            found = true;
            break;
        }
        if let Some(neighbors) = adj.get(&current) {
            for neighbor in neighbors {
                if visited.insert(neighbor.clone()) {
                    parent.insert(neighbor.clone(), current.clone());
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    if !found {
        return CommandOutput::ok(&format!(
            "No path found between '{}' and '{}'",
            node_title(&conn, source_id),
            node_title(&conn, target_id)
        ));
    }

    let mut path = Vec::new();
    let mut cur = target_id.to_string();
    path.push(cur.clone());
    while cur != *source_id {
        if let Some(p) = parent.get(&cur) {
            cur = p.clone();
            path.push(cur.clone());
        } else {
            break;
        }
    }
    path.reverse();

    let hops = path.len() - 1;
    let mut out = format!("Shortest path ({} hops):\n", hops);
    out.push_str("  ");
    for (i, id) in path.iter().enumerate() {
        if i > 0 {
            out.push_str(" → ");
        }
        let title = node_title(&conn, id);
        let ntype = node_type_str(&conn, id);
        out.push_str(&format!("{} '{}'", ntype, title));
    }
    out.push('\n');
    CommandOutput::ok(&out)
}

fn cmd_cluster(args: &[String]) -> CommandOutput {
    let _algo = parse_str(args, "--algo", "louvain");
    let min_size = parse_usize(args, "--min-size", 3);

    let conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开 KB 数据库"),
    };

    let edges = match load_edges(&conn) {
        Ok(e) => e,
        Err(e) => return CommandOutput::err(&format!("读取边失败: {}", e)),
    };

    if edges.is_empty() {
        return CommandOutput::ok("知识库中没有边，无法进行社区发现");
    }

    let mut adj: HashMap<String, HashSet<String>> = HashMap::new();
    for (src, tgt) in &edges {
        adj.entry(src.clone()).or_default().insert(tgt.clone());
        adj.entry(tgt.clone()).or_default().insert(src.clone());
    }

    let nodes: Vec<String> = adj.keys().cloned().collect();
    let _m = edges.len() as f64;

    let mut community: HashMap<String, usize> = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        community.insert(node.clone(), i);
    }

    loop {
        let mut changed = false;
        for node in &nodes {
            let current = community[node];
            let _degree = adj.get(node).map_or(0, |v| v.len());

            let mut comm_links: HashMap<usize, usize> = HashMap::new();
            if let Some(neighbors) = adj.get(node) {
                for nb in neighbors {
                    let nc = community[nb];
                    *comm_links.entry(nc).or_default() += 1;
                }
            }

            let best = comm_links
                .into_iter()
                .max_by_key(|&(_, count)| count)
                .map(|(c, _)| c);

            let best_comm = best.unwrap_or(current);
            if best_comm != current {
                community.insert(node.clone(), best_comm);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let mut comm_map: HashMap<usize, Vec<String>> = HashMap::new();
    for (node, comm) in &community {
        comm_map.entry(*comm).or_default().push(node.clone());
    }

    let mut communities: Vec<(usize, Vec<String>)> = comm_map.into_iter().collect();
    communities.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    let filtered: Vec<(usize, Vec<String>)> = communities
        .into_iter()
        .filter(|(_, members)| members.len() >= min_size)
        .collect();

    if filtered.is_empty() {
        return CommandOutput::ok(&format!(
            "未发现大于等于 {} 个成员的社区 (共 {} 个节点, {} 条边)",
            min_size,
            nodes.len(),
            edges.len()
        ));
    }

    let sizes: Vec<String> = filtered.iter().map(|(_, m)| m.len().to_string()).collect();
    let mut out = format!(
        "Found {} communities (sizes: {})\n",
        filtered.len(),
        sizes.join(", ")
    );

    for (i, (_, members)) in filtered.iter().enumerate() {
        let msize = members.len();
        let mut sorted = members.clone();
        sorted.sort_by(|a, b| {
            let imp_a = conn
                .query_row(
                    "SELECT importance FROM nodes WHERE id = ?1",
                    [a],
                    |row| row.get::<_, f64>(0),
                )
                .unwrap_or(0.0);
            let imp_b = conn
                .query_row(
                    "SELECT importance FROM nodes WHERE id = ?1",
                    [b],
                    |row| row.get::<_, f64>(0),
                )
                .unwrap_or(0.0);
            imp_b
                .partial_cmp(&imp_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let top3: Vec<String> = sorted
            .iter()
            .take(3)
            .map(|id| {
                let title = node_title(&conn, id);
                if title.len() > 30 {
                    format!("{}...", &title[..title.floor_char_boundary(30)])
                } else {
                    title
                }
            })
            .collect();

        let preview = if sorted.len() > 3 {
            format!(
                "{} members, top: {}",
                msize,
                top3.join(", ")
            )
        } else {
            format!("{} members: {}", msize, top3.join(", "))
        };
        out.push_str(&format!("  Community {} ({})\n", i + 1, preview));
    }
    CommandOutput::ok(&out)
}

fn cmd_central(args: &[String]) -> CommandOutput {
    let _algo = parse_str(args, "--algo", "pagerank");
    let top_k = parse_usize(args, "--top-k", 20);

    let conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开 KB 数据库"),
    };

    let edges = match load_edges(&conn) {
        Ok(e) => e,
        Err(e) => return CommandOutput::err(&format!("读取边失败: {}", e)),
    };

    if edges.is_empty() {
        return CommandOutput::ok("知识库中没有边，无法计算 PageRank");
    }

    let mut out_edges: HashMap<String, Vec<String>> = HashMap::new();
    let mut all_nodes: HashSet<String> = HashSet::new();

    for (src, tgt) in &edges {
        all_nodes.insert(src.clone());
        all_nodes.insert(tgt.clone());
        out_edges.entry(src.clone()).or_default().push(tgt.clone());
    }

    let n = all_nodes.len();
    let d = 0.85_f64;
    let nodes: Vec<String> = all_nodes.into_iter().collect();

    let mut pr: HashMap<String, f64> = HashMap::new();
    for node in &nodes {
        pr.insert(node.clone(), 1.0 / n as f64);
    }

    for _ in 0..20 {
        let mut dangling_sum = 0.0;
        for node in &nodes {
            let deg = out_edges.get(node).map_or(0, |v| v.len());
            if deg == 0 {
                dangling_sum += pr[node];
            }
        }
        let dangling_contrib = d * dangling_sum / n as f64;

        let mut new_pr: HashMap<String, f64> = HashMap::new();
        for node in &nodes {
            let mut sum = 0.0;
            if let Some(edges_from_u) = out_edges.get(node) {
                let deg = edges_from_u.len() as f64;
                if deg > 0.0 {
                    for v in edges_from_u {
                        sum += pr.get(v).copied().unwrap_or(0.0) / deg;
                    }
                }
            }
            let pr_val = (1.0 - d) / n as f64 + d * sum + dangling_contrib;
            new_pr.insert(node.clone(), pr_val);
        }
        pr = new_pr;
    }

    let mut sorted: Vec<(String, f64)> = pr.into_iter().collect();
    sorted.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let k = top_k.min(sorted.len());
    let mut out = format!("Top {} by PageRank (d={}):\n", k, d);
    for (i, (id, score)) in sorted.iter().take(k).enumerate() {
        let title = node_title(&conn, id);
        out.push_str(&format!("  {:>3}. {} ({:.4})\n", i + 1, title, score));
    }
    CommandOutput::ok(&out)
}

fn cmd_serve(_args: &[String]) -> CommandOutput {
    let port = parse_usize(_args, "--port", 8337);
    CommandOutput::warn(&format!(
        "MCP server stub on port {} — use Phase 3 OpenAPI for HTTP access",
        port
    ))
}

fn cmd_export(args: &[String]) -> CommandOutput {
    if args.is_empty() {
        return CommandOutput::err("用法: /kb export <node_id> [--format json|svg]");
    }
    let node_id = &args[0];
    let format = parse_str(args, "--format", "json");

    let kb = match open_kb() {
        Some(kb) => kb,
        None => return CommandOutput::err("无法打开知识库 (KnowledgeBase::open failed)"),
    };

    let node = match kb.get_node(node_id) {
        Ok(Some(n)) => n,
        Ok(None) => return CommandOutput::not_found(&format!("未找到节点: {}", node_id)),
        Err(e) => return CommandOutput::err(&format!("查询节点失败: {}", e)),
    };

    let related = kb.get_related(node_id, None, 100).unwrap_or_default();

    match format {
        "json" => {
            let mut entries: Vec<serde_json::Value> = Vec::new();
            entries.push(serde_json::json!({
                "id": node.id,
                "type": node.node_type.as_str(),
                "title": node.title,
                "summary": node.summary,
                "url": node.url,
                "domain": node.domain,
                "confidence": node.confidence,
                "importance": node.importance
            }));

            for r in &related {
                entries.push(serde_json::json!({
                    "id": r.node.id,
                    "type": r.node.node_type.as_str(),
                    "title": r.node.title,
                    "relation_score": r.score
                }));
            }

            let output = serde_json::to_string_pretty(&serde_json::json!({
                "node": {
                    "id": node.id,
                    "type": node.node_type.as_str(),
                    "title": node.title
                },
                "related_count": related.len(),
                "subgraph": entries
            }))
            .unwrap_or_else(|_| "{}".to_string());

            CommandOutput::ok(&output).with_json(serde_json::json!({ "format": "json" }))
        }
        "svg" => {
            let related_titles: Vec<String> = related
                .iter()
                .map(|r| {
                    let t = &r.node.title;
                    if t.len() > 20 {
                        format!("{}...", &t[..t.floor_char_boundary(20)])
                    } else {
                        t.clone()
                    }
                })
                .collect();

            let node_title_escaped = node_title_escape(&node.title);
            let total = 1 + related.len();
            let _radius = 120.0_f64;
            let cx = 400.0_f64;
            let cy = 300.0_f64;
            let spread = 220.0_f64;

            let mut svg = format!(
                r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 600" width="800" height="600">
  <rect width="800" height="600" fill="#f8f9fa" rx="12"/>
  <defs>
    <marker id="arrow" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="6" markerHeight="6" orient="auto">
      <path d="M 0 0 L 10 5 L 0 10 z" fill="#adb5bd"/>
    </marker>
    <radialGradient id="center" cx="50%" cy="50%" r="50%">
      <stop offset="0%" stop-color="#e85454"/>
      <stop offset="100%" stop-color="#c0392b"/>
    </radialGradient>
    <radialGradient id="rel" cx="50%" cy="50%" r="50%">
      <stop offset="0%" stop-color="#6c5ce7"/>
      <stop offset="100%" stop-color="#4a3cb5"/>
    </radialGradient>
  </defs>
  <text x="400" y="30" text-anchor="middle" font-family="sans-serif" font-size="16" fill="#495057" font-weight="bold">Knowledge Graph — "{}"</text>
  <circle cx="{}" cy="{}" r="40" fill="url(#center)" stroke="#fff" stroke-width="3"/>
  <text x="{}" y="{}" text-anchor="middle" fill="#fff" font-family="sans-serif" font-size="11" font-weight="bold">{}"##,
                node_title_escaped, cx, cy, cx, cy + 4.0,
                if node.title.len() > 12 {
                    &node.title[..node.title.floor_char_boundary(12)]
                } else {
                    &node.title
                }
            );

            let angle_step = if total > 1 {
                std::f64::consts::TAU / total as f64
            } else {
                0.0
            };

            for (i, rt) in related_titles.iter().enumerate() {
                let angle = angle_step * i as f64 - std::f64::consts::FRAC_PI_2;
                let rx = cx + angle.cos() * spread;
                let ry = cy + angle.sin() * spread;

                svg.push_str(&format!(
                    r##"
  <line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="#adb5bd" stroke-width="1.5" marker-end="url(#arrow)"/>"##,
                    cx, cy, rx, ry
                ));

                svg.push_str(&format!(
                    r##"
  <circle cx="{:.1}" cy="{:.1}" r="28" fill="url(#rel)" stroke="#fff" stroke-width="2"/>
  <text x="{:.1}" y="{:.1}" text-anchor="middle" fill="#fff" font-family="sans-serif" font-size="9">{}"##,
                    rx, ry, rx, ry + 3.0, rt
                ));
            }

            svg.push_str("\n</svg>");

            let mut out = String::from("Exported SVG subgraph:\n");
            out.push_str(&svg);
            CommandOutput::ok(&out)
        }
        _ => CommandOutput::err(&format!(
            "Unknown format: {}. Use --format json or --format svg",
            format
        )),
    }
}

fn node_title_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    static HOME_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn with_temp_home<T>(f: impl FnOnce() -> T) -> T {
        // 双重持锁: 私有锁串行 kb_cmds 内部; 共享 TEST_ENV_LOCK 与其它模块
        // (consciousness_core::isolate_home_once / cipher) 互斥, 防 HOME 窗口竞争。
        let _g = HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _ge = crate::core::nt_core_self_test::TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let old = std::env::var("HOME").ok();
        let dir = std::env::temp_dir().join(format!("nt_kb_cmds_{}", std::process::id()));
        // Purge any db left by an earlier test in this process (tests share the
        // pid-named temp dir; stale nodes would violate the UNIQUE constraint).
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".neotrix")).unwrap();
        std::env::set_var("HOME", &dir);
        let r = f();
        match old {
            Some(o) => std::env::set_var("HOME", o),
            None => std::env::remove_var("HOME"),
        }
        r
    }

    fn seed_node(conn: &Connection, id: &str, ntype: &str, title: &str, url: &str) {
        let now = 1750000000i64;
        conn.execute(
            "INSERT INTO nodes(id,node_type,title,summary,content,url,domain,language,confidence,importance,created_at,updated_at,access_count,metadata,data_tier,temporal,supersedes,source_episode,tier) VALUES(?1,?2,?3,'s','c',?4,'github.com','en',1.0,0.7,?5,?6,0,'{}','cache',NULL,NULL,NULL,'warm')",
            rusqlite::params![id, ntype, title, url, now, now],
        )
        .unwrap();
    }

    #[test]
    fn test_absorb_map_dry_run() {
        with_temp_home(|| {
            let conn = Connection::open(kb_path()).unwrap();
            crate::l1_action::nt_memory::nt_memory_kb::schema::initialize(&conn).unwrap();
            seed_node(&conn, "u_1", "repository", "GitHub - openai/codex: desc", "https://github.com/openai/codex");
            seed_node(&conn, "u_2", "paper", "Attention Is All You Need", "https://arxiv.org/abs/1706.03762");
            drop(conn);

            let out = cmd_absorb_map(&["--dry-run".to_string(), "--limit".to_string(), "10".to_string()]);
            assert!(out.success, "{}", out.message);
            assert!(out.message.contains("全库本源溯源报告"));
            assert!(out.message.contains("Reality"));
            assert!(out.message.contains("未写库"));
        });
    }

    #[test]
    fn test_absorb_map_apply_writes_metadata() {
        with_temp_home(|| {
            let conn = Connection::open(kb_path()).unwrap();
            crate::l1_action::nt_memory::nt_memory_kb::schema::initialize(&conn).unwrap();
            seed_node(&conn, "u_1", "repository", "GitHub - openai/codex: desc", "https://github.com/openai/codex");
            drop(conn);

            let out = cmd_absorb_map(&["--apply".to_string(), "--limit".to_string(), "10".to_string()]);
            assert!(out.success, "{}", out.message);
            assert!(out.message.contains("已写库"));

            let conn = open_raw_conn().unwrap();
            let meta: Option<String> = conn
                .query_row("SELECT metadata FROM nodes WHERE id = 'u_1'", [], |r| r.get(0))
                .unwrap();
            let v: serde_json::Value = serde_json::from_str(&meta.unwrap()).unwrap();
            assert_eq!(v["absorbed_capability"]["capability"], "execute");
            assert_eq!(v["knowledge_source"]["source_core"], "Reality");
        });
    }

    #[test]
    fn test_snapshot_then_diff_reports_added_node() {
        with_temp_home(|| {
            let conn = Connection::open(kb_path()).unwrap();
            crate::l1_action::nt_memory::nt_memory_kb::schema::initialize(&conn).unwrap();
            seed_node(&conn, "u_1", "repository", "GitHub - openai/codex: desc", "https://github.com/openai/codex");
            drop(conn);

            let dir = std::env::temp_dir().join(format!("nt_kb_cmds_snap_{}", std::process::id()));
            std::fs::create_dir_all(dir.join(".neotrix")).unwrap();
            let snap_path = dir.join("snap.json");
            let out = cmd_snapshot(&["--out".to_string(), snap_path.to_string_lossy().into_owned()]);
            assert!(out.success, "{}", out.message);
            assert!(snap_path.exists(), "快照文件应落盘");

            // 再写入一个节点 → 与快照对比应报告新增。
            let conn = open_raw_conn().unwrap();
            seed_node(&conn, "u_2", "paper", "Attention Is All You Need", "https://arxiv.org/abs/1706.03762");
            drop(conn);

            let diff = cmd_diff(&[
                snap_path.to_string_lossy().into_owned(),
                "--detail".to_string(),
                "5".to_string(),
            ]);
            assert!(diff.success, "{}", diff.message);
            assert!(diff.message.contains("+1"), "应报告新增 1 节点: {}", diff.message);
            let json = diff.json.unwrap();
            assert_eq!(json["nodes_added"].as_array().unwrap().len(), 1);
        });
    }

    #[test]
    fn test_snapshot_roundtrip_via_diff_same_db() {
        with_temp_home(|| {
            let conn = Connection::open(kb_path()).unwrap();
            crate::l1_action::nt_memory::nt_memory_kb::schema::initialize(&conn).unwrap();
            seed_node(&conn, "u_1", "repository", "GitHub - openai/codex: desc", "https://github.com/openai/codex");
            drop(conn);

            let dir = std::env::temp_dir().join(format!("nt_kb_cmds_snap2_{}", std::process::id()));
            std::fs::create_dir_all(dir.join(".neotrix")).unwrap();
            let snap_path = dir.join("snap.json");
            let out = cmd_snapshot(&["--out".to_string(), snap_path.to_string_lossy().into_owned()]);
            assert!(out.success);

            let diff = cmd_diff(&[snap_path.to_string_lossy().into_owned()]);
            assert!(diff.success, "{}", diff.message);
            assert!(diff.message.contains("无差异"), "同库快照应无差异: {}", diff.message);
        });
    }
}

