use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

/// 工作流定义
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub created_at: i64,
    pub updated_at: i64,
    pub tags: Vec<String>,
}

/// 工作流步骤
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub kind: String, // llm, tool, condition, parallel
    pub name: String,
    pub params: HashMap<String, serde_json::Value>,
    pub depends_on: Vec<String>,
    pub timeout_secs: u64,
    pub retry_count: u32,
}

/// 工作流运行实例
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowRun {
    pub id: String,
    pub workflow_id: String,
    pub status: String, // pending, running, completed, failed, cancelled
    pub current_step: usize,
    pub progress_pct: f64,
    pub started_at: i64,
    pub results: HashMap<String, serde_json::Value>,
}

/// 工作流域插件
pub struct WorkflowPluginImpl {
    db_path: PathBuf,
    workflows: Mutex<HashMap<String, Workflow>>,
    runs: Mutex<HashMap<String, WorkflowRun>>,
}

impl WorkflowPluginImpl {
    pub fn new() -> Self {
        let db_path = dirs::home_dir()
            .map(|h| h.join(".neotrix").join("workflows.json"))
            .unwrap_or_else(|| PathBuf::from(".neotrix/workflows.json"));

        // 加载已有工作流
        let workflows = if db_path.exists() {
            std::fs::read_to_string(&db_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            HashMap::new()
        };

        Self {
            db_path,
            workflows: Mutex::new(workflows),
            runs: Mutex::new(HashMap::new()),
        }
    }

    fn get_runs(&self, workflow_id: Option<&str>) -> Result<serde_json::Value, DomainError> {
        let runs = self.runs.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: e.to_string(),
            recoverable: true,
        })?;
        let filtered: Vec<&WorkflowRun> = match workflow_id {
            Some(wid) => runs.values().filter(|r| r.workflow_id == wid).collect(),
            None => runs.values().collect(),
        };
        Ok(serde_json::json!({ "runs": filtered }))
    }

    fn save_workflows(&self) -> Result<(), DomainError> {
        let workflows = self.workflows.lock().map_err(|e| DomainError {
            code: "LOCK_ERROR".into(),
            message: e.to_string(),
            recoverable: true,
        })?;

        if let Some(parent) = self.db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        std::fs::write(
            &self.db_path,
            serde_json::to_string_pretty(&*workflows).unwrap_or_default(),
        )
        .map_err(|e| DomainError {
            code: "WRITE_ERROR".into(),
            message: e.to_string(),
            recoverable: true,
        })?;

        Ok(())
    }
}

impl DomainPlugin for WorkflowPluginImpl {
    fn name(&self) -> &str {
        "workflow"
    }
    fn description(&self) -> &str {
        "工作流：DAG 编排、LLM 步骤、工具调用"
    }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec {
                name: "list".into(),
                description: "列出所有工作流".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "get".into(),
                description: "获取工作流详情".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "create".into(),
                description: "创建工作流".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "update".into(),
                description: "更新工作流".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "delete".into(),
                description: "删除工作流".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "run".into(),
                description: "执行工作流".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "status".into(),
                description: "查询运行状态".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "cancel".into(),
                description: "取消运行".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "runs".into(),
                description: "获取工作流执行记录".into(),
                params: vec![],
                returns: "Value".into(),
            },
        ]
    }

    fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            "list" => {
                let workflows = self.workflows.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                let list: Vec<&Workflow> = workflows.values().collect();
                Ok(serde_json::json!(list))
            }
            "get" => {
                let id = args
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'id'".into(),
                        recoverable: true,
                    })?;
                let workflows = self.workflows.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                match workflows.get(id) {
                    Some(w) => Ok(serde_json::json!(w)),
                    None => Err(DomainError {
                        code: "NOT_FOUND".into(),
                        message: format!("Workflow {} not found", id),
                        recoverable: true,
                    }),
                }
            }
            "create" => {
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled");
                let description = args
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let steps: Vec<WorkflowStep> = args
                    .get("steps")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();
                let tags: Vec<String> = args
                    .get("tags")
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                let now = chrono::Utc::now().timestamp();
                let workflow = Workflow {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: name.to_string(),
                    description: description.to_string(),
                    steps,
                    created_at: now,
                    updated_at: now,
                    tags,
                };

                let mut workflows = self.workflows.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                workflows.insert(workflow.id.clone(), workflow.clone());
                drop(workflows);
                self.save_workflows()?;

                Ok(serde_json::json!(workflow))
            }
            "update" => {
                let id = args
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'id'".into(),
                        recoverable: true,
                    })?;
                let mut workflows = self.workflows.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                let workflow = workflows.get_mut(id).ok_or_else(|| DomainError {
                    code: "NOT_FOUND".into(),
                    message: format!("Workflow {} not found", id),
                    recoverable: true,
                })?;

                if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
                    workflow.name = name.to_string();
                }
                if let Some(desc) = args.get("description").and_then(|v| v.as_str()) {
                    workflow.description = desc.to_string();
                }
                if let Some(steps) = args.get("steps").and_then(|v| v.as_array()) {
                    workflow.steps =
                        serde_json::from_value(serde_json::json!(steps)).unwrap_or_default();
                }
                workflow.updated_at = chrono::Utc::now().timestamp();

                let updated = workflow.clone();
                drop(workflows);
                self.save_workflows()?;

                Ok(serde_json::json!(updated))
            }
            "delete" => {
                let id = args
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'id'".into(),
                        recoverable: true,
                    })?;
                let mut workflows = self.workflows.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                workflows.remove(id);
                drop(workflows);
                self.save_workflows()?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "run" => {
                let workflow_id = args
                    .get("workflow_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'workflow_id'".into(),
                        recoverable: true,
                    })?;

                // 验证工作流存在
                {
                    let workflows = self.workflows.lock().map_err(|e| DomainError {
                        code: "LOCK_ERROR".into(),
                        message: e.to_string(),
                        recoverable: true,
                    })?;
                    if !workflows.contains_key(workflow_id) {
                        return Err(DomainError {
                            code: "NOT_FOUND".into(),
                            message: format!("Workflow {} not found", workflow_id),
                            recoverable: true,
                        });
                    }
                }

                let run_id = uuid::Uuid::new_v4().to_string();
                let run = WorkflowRun {
                    id: run_id.clone(),
                    workflow_id: workflow_id.to_string(),
                    status: "running".to_string(),
                    current_step: 0,
                    progress_pct: 0.0,
                    started_at: chrono::Utc::now().timestamp(),
                    results: HashMap::new(),
                };

                let mut runs = self.runs.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                runs.insert(run_id.clone(), run.clone());

                Ok(serde_json::json!(run))
            }
            "status" => {
                let run_id =
                    args.get("run_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "missing 'run_id'".into(),
                            recoverable: true,
                        })?;
                let runs = self.runs.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                match runs.get(run_id) {
                    Some(r) => Ok(serde_json::json!(r)),
                    None => Err(DomainError {
                        code: "NOT_FOUND".into(),
                        message: format!("Run {} not found", run_id),
                        recoverable: true,
                    }),
                }
            }
            "cancel" => {
                let run_id =
                    args.get("run_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "missing 'run_id'".into(),
                            recoverable: true,
                        })?;
                let mut runs = self.runs.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                if let Some(run) = runs.get_mut(run_id) {
                    run.status = "cancelled".to_string();
                    Ok(serde_json::json!({ "ok": true }))
                } else {
                    Err(DomainError {
                        code: "NOT_FOUND".into(),
                        message: format!("Run {} not found", run_id),
                        recoverable: true,
                    })
                }
            }
            "runs" => {
                let workflow_id = args.get("workflow_id").and_then(|v| v.as_str());
                self.get_runs(workflow_id)
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }
}
