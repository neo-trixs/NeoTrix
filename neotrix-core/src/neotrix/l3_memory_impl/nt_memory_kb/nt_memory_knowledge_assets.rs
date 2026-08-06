use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde_json::Value;

use super::KnowledgeBase;
use super::nt_memory_types::{NodeType, RelationType};

/// A registered executable skill derived from external resources.
/// Follows Resource2Skill pattern: resource → skill → tool.
#[derive(Debug, Clone)]
pub struct SkillEntry {
    pub name: String,
    pub description: String,
    pub source_resource: String,
    pub domain: String,
    pub tool_name: Option<String>,
    pub confidence: f64,
}

/// Skills library — registry of skills distilled from external resources.
/// In-memory store that can be rebuilt from KB nodes.
#[derive(Debug, Clone)]
pub struct SkillsLibrary {
    skills: HashMap<String, SkillEntry>,
}

impl SkillsLibrary {
    pub fn new() -> Self {
        Self { skills: HashMap::new() }
    }

    pub fn register(&mut self, entry: SkillEntry) {
        self.skills.insert(entry.name.clone(), entry);
    }

    pub fn get(&self, name: &str) -> Option<&SkillEntry> {
        self.skills.get(name)
    }

    pub fn all(&self) -> Vec<&SkillEntry> {
        self.skills.values().collect()
    }

    pub fn by_domain(&self, domain: &str) -> Vec<&SkillEntry> {
        self.skills.values().filter(|s| s.domain == domain).collect()
    }

    pub fn rebuild_from_kb(&mut self, kb: &KnowledgeBase) -> Result<usize, String> {
        let conn = kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let query = "SELECT id, title, summary, url, domain FROM nodes WHERE node_type = 'skill'";
        let mut stmt = conn.prepare(query).map_err(|e| format!("Prepare: {}", e))?;
        let rows = stmt.query_map([], |row| {
            let title: String = row.get(1)?;
            let summary: Option<String> = row.get(2)?;
            let url: Option<String> = row.get(3)?;
            let domain: Option<String> = row.get(4)?;
            Ok((title, summary, url, domain))
        }).map_err(|e| format!("Query: {}", e))?;
        let mut count = 0;
        for (name, summary, _url, domain) in rows.flatten() {
            self.skills.insert(name.clone(), SkillEntry {
                name,
                description: summary.unwrap_or_default(),
                source_resource: String::new(),
                domain: domain.unwrap_or_else(|| "unknown".into()),
                tool_name: None,
                confidence: 0.5,
            });
            count += 1;
        }
        Ok(count)
    }
}

impl Default for SkillsLibrary {
    fn default() -> Self {
        Self::new()
    }
}

fn prefix(s: &str, max: usize) -> &str {
    if s.len() <= max { s }
    else {
        let idx = s.char_indices().nth(max).map(|(i, _)| i).unwrap_or(s.len());
        &s[..idx]
    }
}

pub fn import_knowledge_assets(kb: &KnowledgeBase, path: &Path) -> Result<ImportReport, String> {
    let data = fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
    let entries: Vec<Value> = serde_json::from_str(&data).map_err(|e| format!("JSON parse error: {}", e))?;

    let mut report = ImportReport::default();

    for entry in &entries {
        let title = entry["title"].as_str().unwrap_or("untitled");
        let body = entry["body"].as_str().unwrap_or("");
        let tags: Vec<&str> = entry["tags"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();
        let importance = entry["importance"].as_f64().unwrap_or(0.5);
        let domain = tags.first().copied().unwrap_or("knowledge");
        let asset_url = format!("asset:knowledge_data:{}", title);

        let summary = Some(prefix(body, 200));

        let node_id = match kb.insert_or_get_node(title, NodeType::Concept, summary, Some(&asset_url), Some(domain)) {
            Ok(id) => id,
            Err(e) => {
                report.errors.push(format!("{}: {}", title, e));
                continue;
            }
        };

        if let Err(e) = kb.update_node_content(&node_id, body) {
            report.errors.push(format!("{} content: {}", title, e));
        }

        let meta = serde_json::json!({
            "tags": tags,
            "importance": importance,
            "source": "knowledge_assets",
            "char_count": body.len(),
        });
        if let Err(e) = kb.update_node_metadata(&node_id, &meta) {
            report.errors.push(format!("{} metadata: {}", title, e));
        }

        report.imported += 1;

        for seen in &report.seen_titles {
            let seen_url = format!("asset:knowledge_data:{}", seen);
            if let Ok(Some(other_node)) = kb.find_node_by_url(&seen_url) {
                let other_tags: Vec<String> = other_node
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("tags"))
                    .and_then(|t| t.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                let shared: Vec<&str> = tags.iter().filter(|t| other_tags.contains(&t.to_string())).copied().collect();
                for tag in &shared {
                    let weight = 0.5 + (importance.min(other_node.importance)) * 0.5;
                    let _ = kb.upsert_edge(&node_id, &other_node.id, RelationType::Related, weight, Some(&format!("shared tag: {}", tag)));
                    report.edges_created += 1;
                }
            }
        }

        report.seen_titles.push(title.to_string());
    }

    Ok(report)
}

pub fn import_review_findings(kb: &KnowledgeBase, path: &Path) -> Result<ImportReport, String> {
    let data = fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
    let root: Value = serde_json::from_str(&data).map_err(|e| format!("JSON parse error: {}", e))?;

    let mut report = ImportReport::default();
    let defects = root["defects"].as_array().ok_or("No 'defects' array found")?;
    let review_time = root["review_summary"]
        .as_object()
        .and_then(|s| s.get("timestamp"))
        .and_then(|t| t.as_str())
        .unwrap_or("unknown")
        .to_string();

    for entry in defects {
        let file_path = entry["file_path"].as_str().unwrap_or("");
        let line_number = entry["line_number"].as_i64().unwrap_or(0);
        let severity = entry["severity"].as_str().unwrap_or("P3");
        let defect_type = entry["defect_type"].as_str().unwrap_or("unknown");
        let description = entry["description"].as_str().unwrap_or("");
        let title = format!("[{}] {} at {}:{}", severity, defect_type, file_path, line_number);
        let dedup_url = format!("asset:review_finding:{}:{}:{}", file_path, line_number, defect_type);
        let summary = Some(prefix(description, 200));

        let _priority = match severity {
            "P0" => 0.98, "P1" => 0.90, "P2" => 0.70, _ => 0.50,
        };

        let node_id = match kb.insert_or_get_node(&title, NodeType::DetectionFinding, summary, Some(&dedup_url), Some("architecture_review")) {
            Ok(id) => id,
            Err(e) => {
                report.errors.push(format!("{}: {}", title, e));
                continue;
            }
        };

        let meta = serde_json::json!({
            "file_path": file_path,
            "line_number": line_number,
            "severity": severity,
            "defect_type": defect_type,
            "description": description,
            "source": "review-findings.json",
            "review_time": review_time,
        });
        if let Err(e) = kb.update_node_metadata(&node_id, &meta) {
            report.errors.push(format!("metadata for {}: {}", title, e));
        }

        report.imported += 1;
    }

    Ok(report)
}

pub fn import_reasoning_memories(kb: &KnowledgeBase, path: &Path) -> Result<ImportReport, String> {
    let data = fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
    let root: Value = serde_json::from_str(&data).map_err(|e| format!("JSON parse error: {}", e))?;

    let mut report = ImportReport::default();
    let memories = root["memories"].as_array().ok_or("No 'memories' array found")?;

    for m in memories {
        let tid = m["id"].as_str().unwrap_or("");
        let task_desc = m["task_description"].as_str().unwrap_or("untitled");
        let task_type = m["task_type"].as_str().unwrap_or("unknown");
        let success = m["success"].as_bool().unwrap_or(false);
        let reward = m["reward"].as_f64().unwrap_or(0.0);
        let reward_source = m["reward_source"].as_str().unwrap_or("unknown");
        let lifecycle = m["lifecycle"].as_str().unwrap_or("");

        let dedup_key = if tid.is_empty() {
            format!("reasoning_memory:unnamed:{}", prefix(task_desc, 32))
        } else {
            format!("reasoning_memory:{}", tid)
        };

        let summary = prefix(task_desc, 200);
        let title = format!("[{}] {}", task_type, prefix(task_desc, 80));

        let node_id = match kb.insert_or_get_node(&title, NodeType::ThinkingTrace,
            Some(summary), Some(&dedup_key), Some(task_type))
        {
            Ok(id) => id,
            Err(e) => {
                report.errors.push(format!("mem: {}", e));
                continue;
            }
        };

        let meta = serde_json::json!({
            "task_type": task_type,
            "task_description": task_desc,
            "success": success,
            "reward": reward,
            "reward_source": reward_source,
            "lifecycle": lifecycle,
            "_reasoning_id": tid,
            "source": "reasoning_bank.json",
        });
        let _ = kb.update_node_metadata(&node_id, &meta);

        report.imported += 1;
    }

    Ok(report)
}

pub fn import_knowledge_engine(kb: &KnowledgeBase, path: &Path) -> Result<ImportReport, String> {
    let data = fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
    let root: Value = serde_json::from_str(&data).map_err(|e| format!("JSON parse error: {}", e))?;

    let mut report = ImportReport::default();
    let entries = root["entries"].as_object().ok_or("No 'entries' object found")?;

    // ── Import each entry ──
    for (uuid, entry) in entries {
        let title = entry["title"].as_str().unwrap_or("untitled");
        let body = entry["body"].as_str().unwrap_or("");
        let summary = entry["summary"].as_str().unwrap_or(body);
        let tags: Vec<&str> = entry["tags"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();
        let source = entry["source"].as_str().unwrap_or("unknown");
        let source_url = entry["source_url"].as_str().unwrap_or("");
        let confidence = entry["confidence"].as_f64().unwrap_or(0.5);
        let importance = entry["importance"].as_f64().unwrap_or(0.5);
        let first_tag = tags.first().copied().unwrap_or(source);

        // Map source to node type
        let node_type = match source {
            "ArXiv" => NodeType::Paper,
            "Wikipedia" => NodeType::Concept,
            "SemanticScholar" => NodeType::Paper,
            "WebPage" => NodeType::Source,
            _ => NodeType::Concept,
        };

        let dedup_key = if source_url.is_empty() {
            format!("ke:title:{}", title)
        } else {
            format!("ke:{}", source_url)
        };

        let sid = match kb.insert_or_get_node(title, node_type,
            Some(prefix(summary, 200)),
            Some(&dedup_key), Some(first_tag))
        {
            Ok(id) => id,
            Err(e) => {
                report.errors.push(format!("{}: {}", title, e));
                continue;
            }
        };

        // Store body
        if body.len() > 20 {
            let _ = kb.update_node_content(&sid, body);
        }

        // Store metadata
        let meta = serde_json::json!({
            "source": source,
            "tags": tags,
            "confidence": confidence,
            "importance": importance,
            "has_body": body.len() > 20,
            "_ke_uuid": uuid,
        });
        let _ = kb.update_node_metadata(&sid, &meta);

        report.imported += 1;
    }

    Ok(report)
}

pub fn import_absorption_report(kb: &KnowledgeBase, path: &Path) -> Result<ImportReport, String> {
    let data = fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
    let root: Value = serde_json::from_str(&data).map_err(|e| format!("JSON parse error: {}", e))?;

    let mut report = ImportReport::default();
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let ts = root["timestamp"].as_str().unwrap_or(&now);
    let total_gaps = root["total_gaps"].as_i64().unwrap_or(0);
    let total_plans = root["total_plans"].as_i64().unwrap_or(0);
    let p0 = root["p0_count"].as_i64().unwrap_or(0);
    let p1 = root["p1_count"].as_i64().unwrap_or(0);

    // ── Summary node ──
    let summary_title = format!("Absorption Report ({} projects, {} gaps, {} plans)", 
        root["projects_analyzed"].as_i64().unwrap_or(0), total_gaps, total_plans);
    let summary_url = "asset:absorption_report:latest";
    let sid = match kb.insert_or_get_node(&summary_title, NodeType::Concept, 
        Some(&format!("Absorption report from {}: {} projects, {} gaps ({} P0, {} P1), {} plans", 
            ts, root["projects_analyzed"].as_i64().unwrap_or(0), total_gaps, p0, p1, total_plans)),
        Some(summary_url), Some("absorption"))
    {
        Ok(id) => id,
        Err(e) => { report.errors.push(format!("summary: {}", e)); return Ok(report); }
    };

    let mut heatmap = Vec::new();
    if let Some(arr) = root["domain_gap_heatmap"].as_array() {
        for entry in arr {
            heatmap.push(format!("{}:{}", entry["domain"].as_str().unwrap_or("?"), entry["gaps"].as_i64().unwrap_or(0)));
        }
    }

    let smeta = serde_json::json!({
        "timestamp": ts,
        "projects_analyzed": root["projects_analyzed"].as_i64().unwrap_or(0),
        "total_gaps": total_gaps,
        "total_plans": total_plans,
        "p0_count": p0,
        "p1_count": p1,
        "estimated_lines": root["estimated_total_lines"].as_i64().unwrap_or(0),
        "domain_gap_heatmap": heatmap,
        "source": "absorption_report.json",
        "synced_at": now,
    });
    let _ = kb.update_node_metadata(&sid, &smeta);
    report.imported += 1;

    // ── Project nodes ──
    if let Some(projects) = root["projects"].as_array() {
        for pval in projects {
            let name = pval.as_str().or_else(|| pval.get("name").and_then(|v| v.as_str())).unwrap_or("unknown");
            let purl = format!("asset:absorption_project:{}", name);
            let pid = match kb.find_node_by_url(&purl) {
                Ok(Some(n)) => n.id,
                _ => {
                    match kb.insert_or_get_node(name, NodeType::Repository, 
                        Some(&format!("Project analyzed during absorption: {}", name)),
                        Some(&purl), Some("absorption"))
                    {
                        Ok(id) => id,
                        Err(e) => { report.errors.push(format!("project {}: {}", name, e)); continue; }
                    }
                }
            };
            let _ = kb.update_node_metadata(&pid, &serde_json::json!({
                "type": "absorption_project",
                "source": "absorption_report.json",
            }));
            let _ = kb.upsert_edge(&sid, &pid, RelationType::Related, 0.5, Some("absorbed_project"));
            report.edges_created += 1;
        }
    }

    // ── Top plan nodes ──
    if let Some(plans) = root["top_plans"].as_array() {
        for pval in plans {
            let pname = pval["name"].as_str().unwrap_or("unknown_plan");
            let domain = pval["domain"].as_str().unwrap_or("GENERAL");
            let priority = pval["priority"].as_str().unwrap_or("P2");
            let lines = pval["lines"].as_i64().unwrap_or(0);
            let purl = format!("asset:absorption_plan:{}", pname);
            let pid = match kb.find_node_by_url(&purl) {
                Ok(Some(n)) => n.id,
                _ => {
                    match kb.insert_or_get_node(pname, NodeType::GoalResult,
                        Some(&format!("[{}] {} plan: {} (~{} lines)", priority, domain, pname, lines)),
                        Some(&purl), Some("absorption"))
                    {
                        Ok(id) => id,
                        Err(e) => { report.errors.push(format!("plan {}: {}", pname, e)); continue; }
                    }
                }
            };
            let _ = kb.update_node_metadata(&pid, &serde_json::json!({
                "domain": domain,
                "priority": priority,
                "estimated_lines": lines,
                "source": "absorption_report.json",
            }));
            let _ = kb.upsert_edge(&sid, &pid, RelationType::Related, 
                if priority == "P0" { 0.95 } else { 0.7 }, Some(&format!("{}_plan:{}", priority, pname)));
            report.edges_created += 1;
        }
    }

    Ok(report)
}

pub fn import_brain_state(kb: &KnowledgeBase, base_path: &Path) -> Result<ImportReport, String> {
    let brain_path = base_path.join("brain.json");
    let meta_path = base_path.join("brain_metadata.json");

    let brain_data = fs::read_to_string(&brain_path)
        .map_err(|e| format!("Cannot read {}: {}", brain_path.display(), e))?;
    let capabilities: Value = serde_json::from_str(&brain_data)
        .map_err(|e| format!("Parse {}: {}", brain_path.display(), e))?;

    let meta_obj = if meta_path.exists() {
        fs::read_to_string(&meta_path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or_default()
    } else {
        Value::default()
    };

    let mut report = ImportReport::default();
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    // Count non-zero capability dimensions
    let active_dims: Vec<&str> = capabilities.as_object()
        .map(|obj| obj.iter()
            .filter(|(_, v)| v.as_f64().unwrap_or(0.0) > 0.01)
            .map(|(k, _)| k.as_str())
            .collect())
        .unwrap_or_default();

    // ── Overall brain state node ──
    let title = format!("Agent Brain State ({} active dims)", active_dims.len());
    let dedup_url = "asset:brain_state:latest";
    let learning_rate = meta_obj.get("learning_rate").and_then(|v| v.as_f64()).unwrap_or(0.05);
    let total_absorb = meta_obj.get("total_absorb_count").and_then(|v| v.as_i64()).unwrap_or(0);

    let node_id = match kb.insert_or_get_node(&title, NodeType::Concept, Some(&format!("Agent self-model with {}/23 dimensions active, learning_rate={}, absorb_count={}", active_dims.len(), learning_rate, total_absorb)), Some(dedup_url), Some("agent_state")) {
        Ok(id) => id,
        Err(e) => {
            report.errors.push(format!("brain_state node: {}", e));
            return Ok(report);
        }
    };

    // Store full capability vector as metadata
    let mut full_meta = serde_json::json!({
        "capabilities": capabilities,
        "active_dim_count": active_dims.len(),
        "learning_rate": learning_rate,
        "total_absorb_count": total_absorb,
        "source": "brain.json",
        "synced_at": now,
    });

    // Merge brain_metadata fields if present
    if let Some(task_aff) = meta_obj.get("task_affinity") {
        full_meta["task_affinity"] = task_aff.clone();
    }
    if let Some(abs_hist) = meta_obj.get("absorption_history") {
        full_meta["absorption_history"] = abs_hist.clone();
    }
    if let Some(custom_src) = meta_obj.get("custom_sources") {
        full_meta["custom_sources"] = custom_src.clone();
    }

    if let Err(e) = kb.update_node_metadata(&node_id, &full_meta) {
        report.errors.push(format!("brain_state metadata: {}", e));
    }

    report.imported += 1;

    // ── Per-dimension skill nodes for active capabilities ──
    if let Some(obj) = capabilities.as_object() {
        for (dim_name, dim_val) in obj {
            let val = dim_val.as_f64().unwrap_or(0.0);
            if val <= 0.01 { continue; }
            let dim_title = format!("Capability: {}", dim_name);
            let dim_url = format!("asset:brain_capability:{}", dim_name);
            let dim_id = match kb.find_node_by_url(&dim_url) {
                Ok(Some(n)) => n.id,
                _ => {
                    match kb.insert_or_get_node(&dim_title, NodeType::Skill, Some(&format!("Agent capability dimension with level {:.2}", val)), Some(&dim_url), Some("agent_state")) {
                        Ok(id) => id,
                        Err(e) => {
                            report.errors.push(format!("dim {}: {}", dim_name, e));
                            continue;
                        }
                    }
                }
            };
            if let Err(e) = kb.update_node_metadata(&dim_id, &serde_json::json!({
                "dimension": dim_name,
                "level": val,
                "source": "brain.json",
                "synced_at": now,
            })) {
                report.errors.push(format!("dim metadata {}: {}", dim_name, e));
            }
            // Edge: brain_state → capability (has_skill)
            let _ = kb.upsert_edge(&node_id, &dim_id, RelationType::Related, val, Some(&format!("capability:{}={}", dim_name, val)));
            report.edges_created += 1;
        }
    }

    Ok(report)
}

pub fn import_bandit_data(kb: &KnowledgeBase, path: &Path) -> Result<ImportReport, String> {
    let data = fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
    let entries: Vec<Value> = serde_json::from_str(&data).map_err(|e| format!("JSON parse error: {}", e))?;

    let mut report = ImportReport::default();
    for entry in &entries {
        if !entry.is_array() || entry.as_array().is_none_or(|a| a.len() < 3) {
            continue;
        }
        let arr = entry.as_array().expect("guarded array check");
        let config = &arr[0];
        let visits = arr[1].as_i64().unwrap_or(0);
        let wins = arr[2].as_i64().unwrap_or(0);

        let tls = config["tls"].as_str().unwrap_or("unknown");
        let platform = config["platform"].as_str().unwrap_or("unknown");
        let h2 = config["h2_profile"].as_str().unwrap_or("unknown");
        let geo = config["geo_tag"].as_str().unwrap_or("");

        let title = format!("Bandit: {} {} {}", tls, platform, h2);
        let dedup_url = format!("asset:bandit:{}:{}:{}", tls, platform, h2);
        let summary = format!("{} profile: tls={} platform={} h2={} geo={} visits={} wins={} rate={:.2}",
            tls, tls, platform, h2, geo, visits, wins, wins as f64 / visits.max(1) as f64);
        let importance = (wins as f64 / visits.max(1) as f64).min(1.0);

        let node_id = match kb.insert_or_get_node(&title, NodeType::Concept,
            Some(&summary), Some(&dedup_url), Some("routing"))
        {
            Ok(id) => id,
            Err(_) => continue,
        };
        let _ = kb.update_node_metadata(&node_id, &serde_json::json!({
            "tls": tls, "platform": platform, "h2_profile": h2, "geo_tag": geo,
            "visits": visits, "wins": wins, "win_rate": importance,
            "source": "bandit.json",
        }));
        report.imported += 1;
    }
    Ok(report)
}

pub fn import_e8_state(kb: &KnowledgeBase, path: &Path) -> Result<ImportReport, String> {
    let data = fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
    let root: Value = serde_json::from_str(&data).map_err(|e| format!("JSON parse error: {}", e))?;

    let mut report = ImportReport::default();
    let dedup_url = "asset:e8_state:latest";
    let mode = root["current_mode"].as_i64().unwrap_or(0);
    let meta = root["current_meta"].as_i64().unwrap_or(0);
    let confidence = root["last_e8_confidence"].as_f64().unwrap_or(0.0);
    let prm_count = root["prm_learning_count"].as_i64().unwrap_or(0);
    let title = format!("E8 Engine State (mode={} meta={} confidence={:.2})", mode, meta, confidence);

    let node_id = match kb.insert_or_get_node(&title, NodeType::Concept,
        Some(&format!("E8 hexagram engine runtime state with PRM learning count={}", prm_count)),
        Some(dedup_url), Some("e8"))
    {
        Ok(id) => id,
        Err(e) => { report.errors.push(format!("e8: {}", e)); return Ok(report); }
    };
    let _ = kb.update_node_metadata(&node_id, &root);
    report.imported += 1;
    Ok(report)
}

pub fn import_avatar_chain(kb: &KnowledgeBase, path: &Path) -> Result<ImportReport, String> {
    let data = fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
    let root: Value = serde_json::from_str(&data).map_err(|e| format!("JSON parse error: {}", e))?;

    let mut report = ImportReport::default();
    let genesis = root["genesis_hash"].as_str().unwrap_or("");
    let entries = root["entries"].as_array().map(|a| a.len()).unwrap_or(0);
    let title = format!("Avatar Chain ({} entries, genesis={})", entries, &genesis[..genesis.len().min(16)]);

    let dedup_url = "asset:avatar_chain:latest";
    let node_id = match kb.insert_or_get_node(&title, NodeType::Concept,
        Some(&format!("Avatar personality blockchain with {} entries", entries)),
        Some(dedup_url), Some("avatar"))
    {
        Ok(id) => id,
        Err(e) => { report.errors.push(format!("avatar: {}", e)); return Ok(report); }
    };
    let _ = kb.update_node_metadata(&node_id, &serde_json::json!({
        "genesis_hash": genesis, "entry_count": entries, "source": "avatar_chain.json",
    }));

    // Per-entry nodes for chain history
    if let Some(arr) = root["entries"].as_array() {
        for (i, entry) in arr.iter().enumerate() {
            let idx = entry["index"].as_i64().unwrap_or(i as i64);
            let ts = entry["timestamp"].as_i64().unwrap_or(0);
            let prev_hash = entry["previous_hash"].as_str().unwrap_or("");
            let sig = entry["signature"].as_str().unwrap_or("");
            let e_url = format!("asset:avatar_entry:{}", prev_hash.get(..8).unwrap_or("?"));
            let summary = format!("Avatar chain entry {} timestamp={} hash={}",
                idx, ts, prev_hash.get(..8).unwrap_or("?"));
            let eid = match kb.find_node_by_url(&e_url) {
                Ok(Some(n)) => n.id,
                _ => match kb.insert_or_get_node(&format!("Avatar Entry {}", idx),
                    NodeType::EventRecord, Some(&summary), Some(&e_url), Some("avatar"))
                {
                    Ok(id) => id,
                    Err(_) => continue,
                }
            };
            let _ = kb.update_node_metadata(&eid, &serde_json::json!({
                "index": idx, "timestamp": ts, "previous_hash": prev_hash,
                "signature": sig, "source": "avatar_chain.json",
            }));
            let _ = kb.upsert_edge(&node_id, &eid, RelationType::Related, 1.0 - (i as f64 * 0.001), Some("chain_entry"));
            report.edges_created += 1;
        }
    }

    report.imported += if entries > 0 { 1 + entries } else { 1 };
    Ok(report)
}

pub fn import_proxy_pool(kb: &KnowledgeBase, path: &Path) -> Result<ImportReport, String> {
    let data = fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
    let root: Value = serde_json::from_str(&data).map_err(|e| format!("JSON parse error: {}", e))?;

    let mut report = ImportReport::default();
    let proxy_entries = root["entries"].as_array().map(|a| a.len()).unwrap_or(0);
    let stats = &root["stats"];

    let direct = stats["direct_usable_count"].as_i64().unwrap_or(0);
    let encrypted = stats["encrypted_count"].as_i64().unwrap_or(0);
    let total_checks = stats["total_checks"].as_i64().unwrap_or(0);
    let title = format!("Proxy Pool ({} proxies, {} direct, {} encrypted)", proxy_entries, direct, encrypted);

    let dedup_url = "asset:proxy_pool:latest";
    let sid = match kb.insert_or_get_node(&title, NodeType::Concept,
        Some(&format!("Proxy pool state: {} entries, {} checks, {} fetch errors",
            proxy_entries, stats["fetch_errors"].as_i64().unwrap_or(0), total_checks)),
        Some(dedup_url), Some("proxy"))
    {
        Ok(id) => id,
        Err(e) => { report.errors.push(format!("proxy: {}", e)); return Ok(report); }
    };
    let _ = kb.update_node_metadata(&sid, &serde_json::json!({
        "total_entries": proxy_entries, "direct_usable": direct,
        "encrypted_count": encrypted, "total_checks": total_checks,
        "fetch_errors": stats["fetch_errors"], "source": "proxy-pool-state.json",
    }));
    report.imported += 1;

    // Top 100 proxies as individual nodes (skip all 7666 — too many for KB)
    if let Some(arr) = root["entries"].as_array() {
        for entry in arr.iter().take(100) {
            let host = entry["node"]["host"].as_str().unwrap_or("");
            let port = entry["node"]["port"].as_i64().unwrap_or(0);
            let scheme = entry["node"]["scheme"].as_str().unwrap_or("");
            if host.is_empty() { continue; }
            let purl = format!("asset:proxy:{}:{}", host, port);
            let pname = format!("Proxy {}:{}", host, port);
            let pid = match kb.find_node_by_url(&purl) {
                Ok(Some(n)) => n.id,
                _ => match kb.insert_or_get_node(&pname, NodeType::Source,
                    Some(&format!("{} proxy {}:{}", scheme, host, port)),
                    Some(&purl), Some("proxy"))
                {
                    Ok(id) => id,
                    Err(_) => continue,
                }
            };
            let _ = kb.upsert_edge(&sid, &pid, RelationType::Related, 0.5, Some("proxy_pool_member"));
            report.edges_created += 1;
        }
    }

    Ok(report)
}

#[derive(Default, Debug)]
pub struct ImportReport {
    pub imported: usize,
    pub edges_created: usize,
    pub errors: Vec<String>,
    pub seen_titles: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_import_knowledge_assets() {
        let candidates = [
            "assets/knowledge_data.json",
            "../assets/knowledge_data.json",
        ];
        let path = candidates.iter().find(|s| Path::new(s).exists());
        let path = match path {
            Some(p) => Path::new(p),
            None => {
                eprintln!("Skipping test: assets/knowledge_data.json not found");
                return;
            }
        };

        // B1 测试隔离: 用临时路径而非 open(None) (后者会打开生产 ~/.neotrix/knowledge.db,
        // 并行测试时 database is locked 且污染真实知识库)。
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_kbtest_assets_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = match super::super::KnowledgeBase::open(Some(tmp)) {
            Ok(kb) => kb,
            Err(e) => {
                eprintln!("Skipping test: cannot open KB: {}", e);
                return;
            }
        };

        let report = import_knowledge_assets(&kb, path).expect("import should succeed");
        assert!(report.imported > 0, "should import at least 1 entry");
        assert!(report.imported <= 250, "at most 250 entries");
        assert!(report.errors.is_empty(), "errors: {:?}", report.errors);
        eprintln!(
            "Imported {} knowledge assets, {} edges created",
            report.imported, report.edges_created,
        );
    }
}
