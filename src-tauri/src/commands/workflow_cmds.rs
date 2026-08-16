use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

static STATE: LazyLock<Mutex<WorkflowState>> = LazyLock::new(|| {
    Mutex::new(WorkflowState {
        workflows: Vec::new(),
        runs: Vec::new(),
        run_steps: Vec::new(),
        schedules: Vec::new(),
    })
});

struct WorkflowState {
    workflows: Vec<Workflow>,
    runs: Vec<WorkflowRun>,
    run_steps: Vec<WorkflowRunStep>,
    schedules: Vec<WorkflowSchedule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub params: HashMap<String, String>,
    pub depends_on: Vec<String>,
    pub timeout_secs: u64,
    pub retry_count: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: u32,
    pub steps: Vec<WorkflowStep>,
    pub created_at: i64,
    pub updated_at: i64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: String,
    pub workflow_id: String,
    pub status: String,
    pub current_step: u32,
    pub progress_pct: f64,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRunStep {
    pub run_id: String,
    pub step_id: String,
    pub status: String,
    pub started_at: i64,
    pub duration_ms: u64,
    pub output: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSchedule {
    pub id: String,
    pub workflow_id: String,
    pub trigger: String,
    pub cron_expr: Option<String>,
    pub enabled: bool,
}

fn short_uid() -> String {
    Uuid::new_v4().to_string()[..8].to_string()
}

fn now_ts() -> i64 {
    Utc::now().timestamp_millis()
}

#[tauri::command]
pub fn workflow_create(
    name: String,
    description: String,
    steps_json: String,
) -> Result<String, String> {
    let steps: Vec<serde_json::Value> =
        serde_json::from_str(&steps_json).map_err(|e| format!("Invalid steps JSON: {e}"))?;

    let parsed_steps: Vec<WorkflowStep> = steps
        .into_iter()
        .map(|s| {
            let kind = s
                .get("kind")
                .and_then(|v| v.as_str())
                .unwrap_or("tool_call")
                .to_string();
            let name = s
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("step")
                .to_string();
            let params = s
                .get("params")
                .and_then(|v| v.as_object())
                .map(|o| {
                    o.iter()
                        .map(|(k, v)| {
                            (
                                k.clone(),
                                v.as_str().map(|s| s.to_string()).unwrap_or_default(),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            let depends_on = s
                .get("depends_on")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let timeout_secs = s
                .get("timeout_secs")
                .and_then(|v| v.as_u64())
                .unwrap_or(300);
            let retry_count = s
                .get("retry_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u8;
            WorkflowStep {
                id: short_uid(),
                kind,
                name,
                params,
                depends_on,
                timeout_secs,
                retry_count,
            }
        })
        .collect();

    let id = format!("wf-{}", short_uid());
    let now = now_ts();

    let wf = Workflow {
        id: id.clone(),
        name,
        description,
        version: 1,
        steps: parsed_steps,
        created_at: now,
        updated_at: now,
        tags: Vec::new(),
    };

    let mut state = STATE.lock().map_err(|e| format!("State lock: {e}"))?;
    if state.workflows.len() >= 100 {
        return Err("Workflow limit reached (max 100)".into());
    }
    state.workflows.push(wf);
    Ok(id)
}

#[tauri::command]
pub fn workflow_list() -> Result<Vec<Workflow>, String> {
    let state = STATE.lock().map_err(|e| format!("State lock: {e}"))?;
    Ok(state.workflows.clone())
}

#[tauri::command]
pub fn workflow_get(id: String) -> Result<Workflow, String> {
    let state = STATE.lock().map_err(|e| format!("State lock: {e}"))?;
    state
        .workflows
        .iter()
        .find(|w| w.id == id)
        .cloned()
        .ok_or_else(|| format!("Workflow not found: {id}"))
}

#[tauri::command]
pub fn workflow_update(
    id: String,
    name: Option<String>,
    description: Option<String>,
    steps_json: Option<String>,
) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("State lock: {e}"))?;
    let wf = state
        .workflows
        .iter_mut()
        .find(|w| w.id == id)
        .ok_or_else(|| format!("Workflow not found: {id}"))?;

    if let Some(n) = name {
        wf.name = n;
    }
    if let Some(d) = description {
        wf.description = d;
    }
    if let Some(json) = steps_json {
        let steps: Vec<serde_json::Value> =
            serde_json::from_str(&json).map_err(|e| format!("Invalid steps JSON: {e}"))?;
        wf.steps = steps
            .into_iter()
            .map(|s| {
                let kind = s
                    .get("kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or("tool_call")
                    .to_string();
                let name = s
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("step")
                    .to_string();
                let params = s
                    .get("params")
                    .and_then(|v| v.as_object())
                    .map(|o| {
                        o.iter()
                            .map(|(k, v)| {
                                (
                                    k.clone(),
                                    v.as_str().map(|s| s.to_string()).unwrap_or_default(),
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let depends_on = s
                    .get("depends_on")
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let timeout_secs = s
                    .get("timeout_secs")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(300);
                let retry_count = s
                    .get("retry_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u8;
                WorkflowStep {
                    id: short_uid(),
                    kind,
                    name,
                    params,
                    depends_on,
                    timeout_secs,
                    retry_count,
                }
            })
            .collect();
        wf.version += 1;
    }
    wf.updated_at = now_ts();
    Ok(())
}

#[tauri::command]
pub fn workflow_delete(id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("State lock: {e}"))?;
    let len_before = state.workflows.len();
    state.workflows.retain(|w| w.id != id);
    if state.workflows.len() == len_before {
        return Err(format!("Workflow not found: {id}"));
    }
    Ok(())
}

#[tauri::command]
pub fn workflow_run(workflow_id: String) -> Result<String, String> {
    let mut state = STATE.lock().map_err(|e| format!("State lock: {e}"))?;

    let wf = state
        .workflows
        .iter()
        .find(|w| w.id == workflow_id)
        .cloned()
        .ok_or_else(|| format!("Workflow not found: {workflow_id}"))?;

    if state.runs.len() >= 200 {
        return Err("Run limit reached (max 200)".into());
    }

    let run_id = format!("run-{}", short_uid());
    let now = now_ts();

    let run = WorkflowRun {
        id: run_id.clone(),
        workflow_id: workflow_id.clone(),
        status: "running".into(),
        current_step: 0,
        progress_pct: 0.0,
        started_at: now,
        completed_at: None,
        error: None,
    };
    state.runs.push(run);

    let total = wf.steps.len() as f64;
    for (i, step) in wf.steps.iter().enumerate() {
        if state.runs.iter().any(|r| r.id == run_id && r.status == "cancelled") {
            break;
        }

        if state.run_steps.len() >= 1000 {
            break;
        }

        let step_start = now_ts();
        let rs = WorkflowRunStep {
            run_id: run_id.clone(),
            step_id: step.id.clone(),
            status: "running".into(),
            started_at: step_start,
            duration_ms: 0,
            output: None,
            error: None,
        };
        state.run_steps.push(rs);

        let simulated_output = format!(
            "Executed step '{}' (kind={}) with {} params",
            step.name,
            step.kind,
            step.params.len()
        );
        let duration = fastrand_u64(1..500);

        let step_idx = state
            .run_steps
            .iter()
            .rposition(|s| s.run_id == run_id && s.step_id == step.id)
            .ok_or_else(|| format!("step {} for run {} not found", step.id, run_id))?;
        state.run_steps[step_idx].status = "completed".into();
        state.run_steps[step_idx].duration_ms = duration;
        state.run_steps[step_idx].output = Some(simulated_output);

        let run_idx = state.runs.iter().rposition(|r| r.id == run_id).ok_or_else(|| format!("run {} not found", run_id))?;
        state.runs[run_idx].current_step = (i + 1) as u32;
        state.runs[run_idx].progress_pct = ((i + 1) as f64 / total * 100.0 * 100.0).round() / 100.0;
    }

    let run_idx = state.runs.iter().rposition(|r| r.id == run_id).ok_or_else(|| format!("run {} not found", run_id))?;
    state.runs[run_idx].status = "completed".into();
    state.runs[run_idx].completed_at = Some(now_ts());
    state.runs[run_idx].progress_pct = 100.0;

    Ok(run_id)
}

#[tauri::command]
pub fn workflow_run_status(run_id: String) -> Result<WorkflowRun, String> {
    let state = STATE.lock().map_err(|e| format!("State lock: {e}"))?;
    state
        .runs
        .iter()
        .find(|r| r.id == run_id)
        .cloned()
        .ok_or_else(|| format!("Run not found: {run_id}"))
}

#[tauri::command]
pub fn workflow_run_list(workflow_id: String) -> Result<Vec<WorkflowRun>, String> {
    let state = STATE.lock().map_err(|e| format!("State lock: {e}"))?;
    let mut runs: Vec<WorkflowRun> = state
        .runs
        .iter()
        .filter(|r| r.workflow_id == workflow_id)
        .cloned()
        .collect();
    runs.truncate(50);
    Ok(runs)
}

#[tauri::command]
pub fn workflow_run_steps(run_id: String) -> Result<Vec<WorkflowRunStep>, String> {
    let state = STATE.lock().map_err(|e| format!("State lock: {e}"))?;
    let steps: Vec<WorkflowRunStep> = state
        .run_steps
        .iter()
        .filter(|s| s.run_id == run_id)
        .cloned()
        .collect();
    Ok(steps)
}

#[tauri::command]
pub fn workflow_run_cancel(run_id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("State lock: {e}"))?;
    let run = state
        .runs
        .iter_mut()
        .find(|r| r.id == run_id)
        .ok_or_else(|| format!("Run not found: {run_id}"))?;
    run.status = "cancelled".into();
    run.completed_at = Some(now_ts());
    Ok(())
}

#[tauri::command]
pub fn workflow_schedule_create(
    workflow_id: String,
    trigger: String,
    cron_expr: Option<String>,
) -> Result<String, String> {
    let mut state = STATE.lock().map_err(|e| format!("State lock: {e}"))?;

    if !state.workflows.iter().any(|w| w.id == workflow_id) {
        return Err(format!("Workflow not found: {workflow_id}"));
    }

    if state.schedules.len() >= 50 {
        return Err("Schedule limit reached (max 50)".into());
    }

    let id = format!("sch-{}", short_uid());
    state.schedules.push(WorkflowSchedule {
        id: id.clone(),
        workflow_id,
        trigger,
        cron_expr,
        enabled: true,
    });
    Ok(id)
}

#[tauri::command]
pub fn workflow_schedule_list() -> Result<Vec<WorkflowSchedule>, String> {
    let state = STATE.lock().map_err(|e| format!("State lock: {e}"))?;
    Ok(state.schedules.clone())
}

#[tauri::command]
pub fn workflow_schedule_delete(schedule_id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("State lock: {e}"))?;
    let len_before = state.schedules.len();
    state.schedules.retain(|s| s.id != schedule_id);
    if state.schedules.len() == len_before {
        return Err(format!("Schedule not found: {schedule_id}"));
    }
    Ok(())
}

#[tauri::command]
pub fn workflow_import_from_json(json: String) -> Result<String, String> {
    let import: serde_json::Value =
        serde_json::from_str(&json).map_err(|e| format!("Invalid JSON: {e}"))?;

    let name = import
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Imported Workflow")
        .to_string();
    let description = import
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let steps_json = import
        .get("steps")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "[]".to_string());

    workflow_create(name, description, steps_json)
}

fn fastrand_u64(range: std::ops::Range<u64>) -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;
    let seed = nanos.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let val = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    range.start + val % (range.end - range.start)
}

#[tauri::command]
pub fn workflow_generate(description: String) -> Result<String, String> {
    let desc_lower = description.to_lowercase();

    let step_defs: Vec<(&str, &str, u64, u8)>;
    let wf_name: String;

    if (desc_lower.contains("review") || desc_lower.contains("pr")) && (desc_lower.contains("code") || desc_lower.contains("pull")) {
        wf_name = "Code Review".into();
        step_defs = vec![
            ("tool_call", "checkout_branch", 60, 1),
            ("tool_call", "run_tests", 120, 2),
            ("sub_agent", "code_review", 300, 0),
            ("tool_call", "create_pr", 60, 1),
            ("notify", "notify_team", 30, 0),
        ];
    } else if desc_lower.contains("deploy") {
        wf_name = "Deploy".into();
        step_defs = vec![
            ("tool_call", "run_tests", 120, 2),
            ("tool_call", "build", 180, 1),
            ("tool_call", "staging_deploy", 120, 1),
            ("sub_agent", "integration_tests", 300, 0),
            ("tool_call", "production_deploy", 120, 2),
            ("notify", "notify_team", 30, 0),
        ];
    } else if desc_lower.contains("research") || desc_lower.contains("api") || desc_lower.contains("document") {
        wf_name = "Research & Documentation".into();
        step_defs = vec![
            ("tool_call", "fetch_docs", 120, 1),
            ("sub_agent", "analyze_endpoints", 180, 0),
            ("tool_call", "generate_docs", 120, 1),
            ("sub_agent", "review", 120, 0),
            ("tool_call", "publish", 60, 1),
        ];
    } else if desc_lower.contains("security") || desc_lower.contains("audit") {
        wf_name = "Security Audit".into();
        step_defs = vec![
            ("tool_call", "scan_dependencies", 120, 2),
            ("tool_call", "check_vulnerabilities", 180, 1),
            ("sub_agent", "review_logs", 120, 0),
            ("tool_call", "generate_report", 60, 1),
            ("notify", "send_notification", 30, 0),
        ];
    } else {
        wf_name = "Generated Workflow".into();
        step_defs = vec![
            ("sub_agent", "analyze", 120, 0),
            ("tool_call", "execute", 120, 1),
            ("sub_agent", "review", 120, 0),
            ("tool_call", "finalize", 60, 1),
        ];
    }

    let mut steps: Vec<WorkflowStep> = step_defs
        .into_iter()
        .map(|(kind, name, timeout, retry)| WorkflowStep {
            id: short_uid(),
            kind: kind.to_string(),
            name: name.to_string(),
            params: HashMap::new(),
            depends_on: Vec::new(),
            timeout_secs: timeout,
            retry_count: retry,
        })
        .collect();

    for i in 1..steps.len() {
        let prev_id = steps[i - 1].id.clone();
        steps[i].depends_on.push(prev_id);
    }

    let id = format!("wf-{}", short_uid());
    let now = now_ts();

    let wf = Workflow {
        id: id.clone(),
        name: wf_name,
        description: description.clone(),
        version: 1,
        steps,
        created_at: now,
        updated_at: now,
        tags: vec!["generated".into()],
    };

    let mut state = STATE.lock().map_err(|e| format!("State lock: {e}"))?;
    if state.workflows.len() >= 100 {
        return Err("Workflow limit reached (max 100)".into());
    }
    state.workflows.push(wf);
    Ok(id)
}

#[tauri::command]
pub fn workflow_generate_suggest(description: String) -> Result<serde_json::Value, String> {
    let desc_lower = description.to_lowercase();

    let step_defs: Vec<(&str, &str, u64, u8)>;

    if (desc_lower.contains("review") || desc_lower.contains("pr")) && (desc_lower.contains("code") || desc_lower.contains("pull")) {
        step_defs = vec![
            ("tool_call", "checkout_branch", 60, 1),
            ("tool_call", "run_tests", 120, 2),
            ("sub_agent", "code_review", 300, 0),
            ("tool_call", "create_pr", 60, 1),
            ("notify", "notify_team", 30, 0),
        ];
    } else if desc_lower.contains("deploy") {
        step_defs = vec![
            ("tool_call", "run_tests", 120, 2),
            ("tool_call", "build", 180, 1),
            ("tool_call", "staging_deploy", 120, 1),
            ("sub_agent", "integration_tests", 300, 0),
            ("tool_call", "production_deploy", 120, 2),
            ("notify", "notify_team", 30, 0),
        ];
    } else if desc_lower.contains("research") || desc_lower.contains("api") || desc_lower.contains("document") {
        step_defs = vec![
            ("tool_call", "fetch_docs", 120, 1),
            ("sub_agent", "analyze_endpoints", 180, 0),
            ("tool_call", "generate_docs", 120, 1),
            ("sub_agent", "review", 120, 0),
            ("tool_call", "publish", 60, 1),
        ];
    } else if desc_lower.contains("security") || desc_lower.contains("audit") {
        step_defs = vec![
            ("tool_call", "scan_dependencies", 120, 2),
            ("tool_call", "check_vulnerabilities", 180, 1),
            ("sub_agent", "review_logs", 120, 0),
            ("tool_call", "generate_report", 60, 1),
            ("notify", "send_notification", 30, 0),
        ];
    } else {
        step_defs = vec![
            ("sub_agent", "analyze", 120, 0),
            ("tool_call", "execute", 120, 1),
            ("sub_agent", "review", 120, 0),
            ("tool_call", "finalize", 60, 1),
        ];
    }

    let steps_json: Vec<serde_json::Value> = step_defs
        .into_iter()
        .map(|(kind, name, timeout, retry)| {
            serde_json::json!({
                "kind": kind,
                "name": name,
                "timeout_secs": timeout,
                "retry_count": retry,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "description": description,
        "steps": steps_json,
        "total_steps": steps_json.len(),
    }))
}

#[tauri::command]
pub fn workflow_export(id: String, format: Option<String>) -> Result<String, String> {
    let state = STATE.lock().map_err(|e| format!("State lock: {e}"))?;
    let wf = state
        .workflows
        .iter()
        .find(|w| w.id == id)
        .cloned()
        .ok_or_else(|| format!("Workflow not found: {id}"))?;

    let fmt = format.unwrap_or_else(|| "json".into());

    match fmt.as_str() {
        "yaml" | "yml" => {
            let mut out = String::new();
            out.push_str(&format!("id: {}\n", wf.id));
            out.push_str(&format!("name: {}\n", wf.name));
            out.push_str(&format!("description: {}\n", wf.description));
            out.push_str(&format!("version: {}\n", wf.version));
            out.push_str(&format!("created_at: {}\n", wf.created_at));
            out.push_str(&format!("updated_at: {}\n", wf.updated_at));
            out.push_str("steps:\n");
            for step in &wf.steps {
                out.push_str(&format!("  - id: {}\n", step.id));
                out.push_str(&format!("    kind: {}\n", step.kind));
                out.push_str(&format!("    name: {}\n", step.name));
                out.push_str(&format!("    timeout_secs: {}\n", step.timeout_secs));
                out.push_str(&format!("    retry_count: {}\n", step.retry_count));
                if !step.params.is_empty() {
                    out.push_str("    params:\n");
                    for (k, v) in &step.params {
                        out.push_str(&format!("      {}: {}\n", k, v));
                    }
                }
                if !step.depends_on.is_empty() {
                    out.push_str(&format!("    depends_on: [{}]\n", step.depends_on.join(", ")));
                }
            }
            Ok(out)
        }
        _ => {
            serde_json::to_string_pretty(&wf).map_err(|e| format!("Serialize error: {e}"))
        }
    }
}

#[tauri::command]
pub fn workflow_duplicate(id: String, new_name: String) -> Result<String, String> {
    let mut state = STATE.lock().map_err(|e| format!("State lock: {e}"))?;

    let original = state
        .workflows
        .iter()
        .find(|w| w.id == id)
        .cloned()
        .ok_or_else(|| format!("Workflow not found: {id}"))?;

    if state.workflows.len() >= 100 {
        return Err("Workflow limit reached (max 100)".into());
    }

    let new_id = format!("wf-{}", short_uid());
    let now = now_ts();

    let duplicate = Workflow {
        id: new_id.clone(),
        name: new_name,
        description: format!("Duplicate of '{}'", original.name),
        version: 1,
        steps: original.steps,
        created_at: now,
        updated_at: now,
        tags: original.tags,
    };

    state.workflows.push(duplicate);
    Ok(new_id)
}

#[tauri::command]
pub fn workflow_batch_run(workflow_ids: Vec<String>, parallel: Option<bool>) -> Result<serde_json::Value, String> {
    let is_parallel = parallel.unwrap_or(false);
    let total = workflow_ids.len();
    let mut run_ids: Vec<String> = Vec::new();
    let mut failed: u32 = 0;

    for wf_id in &workflow_ids {
        match workflow_run(wf_id.clone()) {
            Ok(run_id) => run_ids.push(run_id),
            Err(_) => failed += 1,
        }
    }

    let completed = total as u32 - failed;

    Ok(serde_json::json!({
        "total": total,
        "completed": completed,
        "failed": failed,
        "run_ids": run_ids,
        "parallel": is_parallel,
    }))
}

// All structs are pub and re-exported in mod.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_create_with_two_steps() {
        let steps_json = r#"[
            {"kind": "tool_call", "name": "search", "params": {"query": "rust"}, "depends_on": [], "timeout_secs": 60, "retry_count": 2},
            {"kind": "sub_agent", "name": "analyze", "params": {"depth": "deep"}, "depends_on": ["search"], "timeout_secs": 120, "retry_count": 1}
        ]"#;

        let id = workflow_create("Test WF".into(), "A test workflow".into(), steps_json.into())
            .unwrap();
        assert!(id.starts_with("wf-"), "id: {id}");

        let wf = workflow_get(id.clone()).unwrap();
        assert_eq!(wf.name, "Test WF");
        assert_eq!(wf.steps.len(), 2);
        assert_eq!(wf.steps[0].kind, "tool_call");
        assert_eq!(wf.steps[0].name, "search");
        assert_eq!(wf.steps[1].kind, "sub_agent");
        assert_eq!(wf.steps[1].name, "analyze");
    }

    #[test]
    fn test_workflow_list() {
        // Create a fresh workflow
        let steps = r#"[{"kind": "notify", "name": "alert", "params": {}, "depends_on": [], "timeout_secs": 30, "retry_count": 0}]"#;
        let id = workflow_create("List test".into(), "".into(), steps.into()).unwrap();

        let list = workflow_list().unwrap();
        assert!(list.iter().any(|w| w.id == id));
    }

    #[test]
    fn test_workflow_run_and_status() {
        let steps = r#"[
            {"kind": "tool_call", "name": "fetch", "params": {"url": "https://example.com"}, "depends_on": [], "timeout_secs": 30, "retry_count": 1},
            {"kind": "tool_call", "name": "parse", "params": {}, "depends_on": ["fetch"], "timeout_secs": 30, "retry_count": 0}
        ]"#;
        let wf_id = workflow_create("Run test".into(), "".into(), steps.into()).unwrap();

        let run_id = workflow_run(wf_id.clone()).unwrap();
        assert!(run_id.starts_with("run-"), "run_id: {run_id}");

        let status = workflow_run_status(run_id.clone()).unwrap();
        assert_eq!(status.workflow_id, wf_id);
        assert_eq!(status.status, "completed");
        assert_eq!(status.progress_pct, 100.0);
        assert!(status.completed_at.is_some());
    }

    #[test]
    fn test_workflow_run_cancel() {
        let steps = r#"[{"kind": "wait", "name": "pause", "params": {"duration": "999"}, "depends_on": [], "timeout_secs": 999, "retry_count": 0}]"#;
        let wf_id = workflow_create("Cancel test".into(), "".into(), steps.into()).unwrap();

        let run_id = workflow_run(wf_id.clone()).unwrap();

        workflow_run_cancel(run_id.clone()).unwrap();

        let status = workflow_run_status(run_id.clone()).unwrap();
        assert_eq!(status.status, "cancelled");
        assert!(status.completed_at.is_some());
    }

    #[test]
    fn test_workflow_update_and_delete() {
        let steps = r#"[{"kind": "tool_call", "name": "step1", "params": {}, "depends_on": [], "timeout_secs": 30, "retry_count": 0}]"#;
        let wf_id = workflow_create("Update test".into(), "original".into(), steps.into()).unwrap();

        workflow_update(
            wf_id.clone(),
            Some("Updated".into()),
            Some("updated desc".into()),
            None,
        )
        .unwrap();

        let wf = workflow_get(wf_id.clone()).unwrap();
        assert_eq!(wf.name, "Updated");
        assert_eq!(wf.description, "updated desc");
        assert_eq!(wf.version, 1);

        workflow_delete(wf_id.clone()).unwrap();
        assert!(workflow_get(wf_id.clone()).is_err());
    }

    #[test]
    fn test_workflow_generate() {
        let wf_id = workflow_generate("review my code and create a PR".into()).unwrap();
        assert!(wf_id.starts_with("wf-"), "id: {wf_id}");

        let wf = workflow_get(wf_id.clone()).unwrap();
        assert_eq!(wf.name, "Code Review");
        assert_eq!(wf.steps.len(), 5);
        assert_eq!(wf.steps[0].name, "checkout_branch");
        assert_eq!(wf.steps[1].name, "run_tests");
        assert_eq!(wf.steps[4].name, "notify_team");
        assert!(wf.steps[0].depends_on.is_empty());
        assert_eq!(wf.steps[1].depends_on, vec![wf.steps[0].id.clone()]);
    }

    #[test]
    fn test_workflow_export_json() {
        let steps = r#"[{"kind": "tool_call", "name": "step1", "params": {}, "depends_on": [], "timeout_secs": 30, "retry_count": 0}]"#;
        let wf_id = workflow_create("Export test".into(), "test".into(), steps.into()).unwrap();

        let exported = workflow_export(wf_id.clone(), None).unwrap();
        assert!(exported.contains("Export test"));
        assert!(exported.contains("step1"));

        let yaml = workflow_export(wf_id.clone(), Some("yaml".into())).unwrap();
        assert!(yaml.contains("name: Export test"));
    }

    #[test]
    fn test_workflow_batch_run() {
        let steps = r#"[{"kind": "tool_call", "name": "batch_step", "params": {}, "depends_on": [], "timeout_secs": 10, "retry_count": 0}]"#;
        let wf1 = workflow_create("Batch 1".into(), "".into(), steps.into()).unwrap();
        let wf2 = workflow_create("Batch 2".into(), "".into(), steps.into()).unwrap();

        let result = workflow_batch_run(vec![wf1, wf2], Some(false)).unwrap();
        assert_eq!(result["total"], 2);
        assert_eq!(result["completed"], 2);
        assert_eq!(result["failed"], 0);
        assert_eq!(result["parallel"], false);
        let ids = result["run_ids"].as_array().unwrap();
        assert_eq!(ids.len(), 2);
    }
}
