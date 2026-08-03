use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_memory_kb::KnowledgeBase;
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
        "Knowledge base operations: /kb stats | /kb search <query> | /kb explore <node_id> | /kb find <src> <tgt> | /kb cluster | /kb central | /kb serve | /kb export <node_id> | /kb import-assets"
    }
    fn execute(
        &self,
        args: &[String],
        _brain: Option<
            &std::sync::Arc<tokio::sync::RwLock<crate::neotrix::nt_mind::SelfIteratingBrain>>,
        >,
    ) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok(
                "Knowledge Base (KB) commands:\n\
                  /kb stats                      显示 KB 统计\n\
                  /kb search <query>             搜索知识库\n\
                  /kb explore <node_id>          查看节点详情及关联\n\
                  /kb find <src> <tgt>           查找两个节点间最短路径\n\
                  /kb cluster [--min-size 3]     社区发现分析\n\
                  /kb central [--top-k 20]       中心性分析 (PageRank)\n\
                  /kb serve [--port 8337]        启动 MCP 知识服务\n\
                  /kb export <node_id> [--format json|svg]  导出子图\n\
                  /kb import-assets [path]       导入 assets/knowledge_data.json 到 KB\n\
                  /kb import-review [path]      导入 review-findings.json 缺陷记录到 KB",
            );
        }

        let sub = args[0].as_str();
        let rest = &args[1..];

        match sub {
            "stats" => cmd_stats(rest),
            "search" => cmd_search(rest),
            "explore" => cmd_explore(rest),
            "find" => cmd_find(rest),
            "cluster" => cmd_cluster(rest),
            "central" => cmd_central(rest),
            "serve" => cmd_serve(rest),
            "export" => cmd_export(rest),
            "import-assets" => cmd_import_assets(rest),
            "import-review" => cmd_import_review(rest),
            "embed" => cmd_embed(rest),
            _ => CommandOutput::err(&format!(
                "未知子命令: {}. 可用: stats, search, explore, find, cluster, central, serve, export, import-assets, import-review, embed",
                sub
            )),
        }
    }
}

fn cmd_embed(_args: &[String]) -> CommandOutput {
    let kb = match KnowledgeBase::open(None) {
        Ok(kb) => kb,
        Err(e) => return CommandOutput::err(&format!("无法打开知识库: {}", e)),
    };
    let cfg = crate::neotrix::nt_memory_kb::nt_memory_embed::EmbeddingConfig::default();
    let bundled = kb.with_embedding(cfg);
    match bundled.ensure_embeddings() {
        Ok(n) => CommandOutput::ok(&format!(
            "Embedding 补跑完成: 本轮生成 {} 条向量 (model all-MiniLM-L6-v2, 384-dim)\n\
             提示: 需先启动本地服务 scripts/kb-embed-server.py",
            n
        )),
        Err(e) => CommandOutput::err(&format!("Embedding 补跑失败: {}", e)),
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
    match kb.search(&query, 10) {
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
