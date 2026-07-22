use std::fs;
use std::path::Path;

use serde_json::Value;

use super::KnowledgeBase;
use super::nt_memory_types::{NodeType, RelationType};

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

        let summary = Some(&body[..body.len().min(200)]);

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
        let summary = Some(&description[..description.len().min(200)]);

        let priority = match severity {
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

        let kb = match super::super::KnowledgeBase::open(None) {
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
