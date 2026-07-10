use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::neotrix::nt_memory_kb::KnowledgeBase;
use crate::neotrix::nt_memory_kb::nt_memory_types::*;

pub fn sync_directory(kb: &KnowledgeBase, dir: &Path, prefix: &str) -> Result<WikiSyncReport, String> {
    let mut report = WikiSyncReport::default();
    let mut files: Vec<PathBuf> = Vec::new();
    collect_md_files(dir, &mut files, prefix)?;

    for file_path in &files {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("read {}: {}", file_path.display(), e))?;

        let file_stem = file_path
            .strip_prefix(dir)
            .unwrap_or(file_path)
            .with_extension("")
            .display()
            .to_string();

        let node_id = format!("wiki:{}", file_stem);
        let title = extract_title(&content).unwrap_or_else(|| file_stem.clone());
        let summary = extract_summary(&content);
        let wiki_links = extract_wiki_links(&content);
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let node = KnowledgeNode {
            id: node_id.clone(),
            node_type: NodeType::WikiPage,
            title,
            summary,
            content: Some(content),
            url: None,
            domain: Some("project_wiki".to_string()),
            language: "en".to_string(),
            confidence: 1.0,
            importance: 0.7,
            created_at: now,
            updated_at: now,
            access_count: 0,
            metadata: Some(serde_json::json!({
                "file_path": file_path.display().to_string(),
                "file_stem": file_stem,
                "wiki_links": wiki_links,
                "line_count": 0,
            })),
            temporal: None,
            supersedes: None,
            source_episode: None,
        };

        match kb.insert_node(&node) {
            Ok(()) => report.synced += 1,
            Err(e) => {
                match kb.update_node(&node) {
                    Ok(()) => report.synced += 1,
                    Err(e2) => {
                        report.errors.push((file_stem, format!("insert: {}/update: {}", e, e2)));
                        continue;
                    }
                }
            }
        }

        for link in &wiki_links {
            let target_id = format!("wiki:{}", link);
            if let Err(e) = kb.upsert_edge(
                &node_id,
                &target_id,
                RelationType::WikiLink,
                1.0,
                Some("wiki_link"),
            ) {
                report.errors.push((format!("{} -> {}", file_stem, link), e));
            } else {
                report.edges_created += 1;
            }
        }
    }

    Ok(report)
}

pub fn build_graph(kb: &KnowledgeBase) -> Result<WikiGraph, String> {
    let nodes = kb.search_by_type(&NodeType::WikiPage, 10000)?;
    let mut graph = WikiGraph {
        nodes: Vec::new(),
        edges: Vec::new(),
    };

    for n in &nodes {
        let file_stem = n.metadata.as_ref()
            .and_then(|m| m.get("file_stem"))
            .and_then(|v| v.as_str())
            .unwrap_or(&n.id)
            .to_string();

        graph.nodes.push(WikiNode {
            id: n.id.clone(),
            title: n.title.clone(),
            summary: n.summary.clone().unwrap_or_default(),
            file_stem,
            importance: n.importance,
        });

        let conn = match kb.conn.lock() {
            Ok(c) => c,
            Err(e) => return Err(format!("Lock: {}", e)),
        };
        if let Ok(edge_list) = crate::neotrix::nt_memory_kb::nt_memory_store::get_edges_for_node(&conn, &n.id) {
            for e in &edge_list {
                if e.relation_type == RelationType::WikiLink {
                    graph.edges.push(WikiEdge {
                        source: e.source_id.clone(),
                        target: e.target_id.clone(),
                        relation: "wiki_link".to_string(),
                    });
                }
            }
        }
    }

    Ok(graph)
}

pub fn generate_graph_html(kb: &KnowledgeBase) -> Result<String, String> {
    let graph = build_graph(kb)?;
    let nodes_json = serde_json::to_string(&graph.nodes).map_err(|e| format!("serialize nodes: {}", e))?;
    let edges_json = serde_json::to_string(&graph.edges).map_err(|e| format!("serialize edges: {}", e))?;

    let template = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>NeoTrix Wiki Knowledge Graph</title>
<style>
  * { margin:0; padding:0; box-sizing:border-box; }
  body { font-family: system-ui, sans-serif; background:#0d0d1a; color:#e0e0ff; overflow:hidden; }
  svg { width:100vw; height:100vh; }
  .info { position:fixed; bottom:20px; left:50%; transform:translateX(-50%); background:rgba(13,13,26,0.9); padding:8px 16px; border-radius:8px; font-size:13px; border:1px solid #333; z-index:10; }
  .legend { position:fixed; top:20px; right:20px; background:rgba(13,13,26,0.9); padding:12px; border-radius:8px; font-size:12px; border:1px solid #333; z-index:10; }
  .legend-item { display:flex; align-items:center; gap:8px; margin:4px 0; }
  .dot { width:10px; height:10px; border-radius:50%; display:inline-block; }
</style>
</head>
<body>
<div id="graph"></div>
<div class="info">Wiki Knowledge Graph - drag to explore, scroll to zoom</div>
<div class="legend">
  <div class="legend-item"><span class="dot" style="background:#7c3aed;"></span> Wiki Page</div>
  <div class="legend-item"><span style="color:#555;">line</span> Wiki Link</div>
</div>
<script src="https://d3js.org/d3.v7.min.js"></script>
<script>
const nodes = __NODES__;
const links = __EDGES__;

const width = window.innerWidth;
const height = window.innerHeight;

const svg = d3.select("#graph").append("svg")
    .attr("width", width).attr("height", height);

const g = svg.append("g");

d3.zoom().on("zoom", (event) => {
    g.attr("transform", event.transform);
})(svg);

const simulation = d3.forceSimulation(nodes)
    .force("link", d3.forceLink(links).id(d => d.id).distance(120))
    .force("charge", d3.forceManyBody().strength(-200))
    .force("center", d3.forceCenter(width / 2, height / 2))
    .force("collision", d3.forceCollide().radius(30));

const link = g.append("g")
    .selectAll("line")
    .data(links)
    .join("line")
    .attr("stroke", "#444")
    .attr("stroke-width", 1)
    .attr("stroke-opacity", 0.6);

const node = g.append("g")
    .selectAll("g")
    .data(nodes)
    .join("g")
    .call(d3.drag()
        .on("start", (event, d) => { if (!event.active) simulation.alphaTarget(0.3).restart(); d.fx = d.x; d.fy = d.y; })
        .on("drag", (event, d) => { d.fx = event.x; d.fy = event.y; })
        .on("end", (event, d) => { if (!event.active) simulation.alphaTarget(0); d.fx = null; d.fy = null; })
    );

node.append("circle")
    .attr("r", d => 5 + Math.sqrt(d.importance * 20))
    .attr("fill", "#7c3aed")
    .attr("stroke", "#a78bfa")
    .attr("stroke-width", 1.5);

node.append("text")
    .text(d => d.title.length > 25 ? d.title.slice(0, 22) + "..." : d.title)
    .attr("x", 10)
    .attr("y", 4)
    .attr("fill", "#c4b5fd")
    .attr("font-size", "11px")
    .attr("font-family", "system-ui");

simulation.on("tick", () => {
    link.attr("x1", d => d.source.x).attr("y1", d => d.source.y)
        .attr("x2", d => d.target.x).attr("y2", d => d.target.y);
    node.attr("transform", d => "translate(" + d.x + "," + d.y + ")");
});
</script>
</body>
</html>"##;

    let html = template
        .replace("__NODES__", &nodes_json)
        .replace("__EDGES__", &edges_json);

    Ok(html)
}

pub fn query(kb: &KnowledgeBase, query_text: &str, limit: usize) -> Result<Vec<WikiSearchResult>, String> {
    let results = kb.search(query_text, limit)?;
    let wiki_results: Vec<WikiSearchResult> = results
        .into_iter()
        .filter(|r| r.node.node_type == NodeType::WikiPage)
        .map(|r| WikiSearchResult {
            id: r.node.id,
            title: r.node.title,
            summary: r.node.summary.unwrap_or_default(),
            score: r.score,
        })
        .collect();
    Ok(wiki_results)
}

// ---- Helpers ----

fn collect_md_files(dir: &Path, files: &mut Vec<PathBuf>, prefix: &str) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("read_dir {}: {}", dir.display(), e))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_md_files(&path, files, prefix)?;
        } else if path.extension().is_some_and(|ext| ext == "md") {
            let relative = path.strip_prefix(prefix).unwrap_or(&path).display().to_string();
            if !relative.contains("node_modules") && !relative.starts_with('.') {
                files.push(path);
            }
        }
    }
    Ok(())
}

fn extract_title(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(t) = trimmed.strip_prefix("# ") {
            return Some(t.to_string());
        }
    }
    None
}

fn extract_summary(content: &str) -> Option<String> {
    let mut lines = content.lines();
    let _ = lines.next();
    for line in lines {
        let t = line.trim();
        if !t.is_empty() && !t.starts_with('#') && !t.starts_with('>') && !t.starts_with("---") {
            let clean = t.strip_prefix("**").unwrap_or(t);
            let clean = clean.strip_suffix("**").unwrap_or(clean);
            if !clean.is_empty() {
                return Some(clean.chars().take(200).collect());
            }
        }
    }
    None
}

fn extract_wiki_links(content: &str) -> Vec<String> {
    let mut links = Vec::new();
    for line in content.lines() {
        let mut remaining = line;
        while let Some(start) = remaining.find("[[") {
            let after = &remaining[start + 2..];
            if let Some(end) = after.find("]]") {
                let link_text = &after[..end];
                let target = link_text.split('|').next().unwrap_or(link_text).trim().to_string();
                if !target.is_empty() && !target.starts_with("http") {
                    links.push(target);
                }
                remaining = &after[end + 2..];
            } else {
                break;
            }
        }
    }
    links.sort();
    links.dedup();
    links
}

// ---- Types ----

#[derive(Default, Debug)]
pub struct WikiSyncReport {
    pub synced: usize,
    pub edges_created: usize,
    pub errors: Vec<(String, String)>,
}

#[derive(Debug, serde::Serialize)]
pub struct WikiNode {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub file_stem: String,
    pub importance: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct WikiEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
}

#[derive(Debug)]
pub struct WikiGraph {
    pub nodes: Vec<WikiNode>,
    pub edges: Vec<WikiEdge>,
}

#[derive(Debug, serde::Serialize)]
pub struct WikiSearchResult {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub score: f64,
}
