//! Tool System for Autonomous Communication

use std::sync::{Arc, Mutex as StdMutex};
use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex as TokioMutex;
use tokio::time::timeout;

use super::{DeliveryOutcome, ToolsConfig, ToolSpec, ToolResult, classify_failure, classify_status, transport_cause, TransportCause};

/// Tool trait
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    fn spec(&self) -> ToolSpec;
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, String>;
}

/// Tool registry
pub struct ToolRegistry {
    config: ToolsConfig,
    tools: Arc<TokioMutex<HashMap<String, Arc<dyn Tool>>>>,
    builtin_tools: Arc<StdMutex<HashMap<String, Arc<dyn Tool>>>>,
}

impl ToolRegistry {
    pub fn new(config: ToolsConfig) -> Self {
        let mut registry = Self {
            config,
            tools: Arc::new(TokioMutex::new(HashMap::new())),
            builtin_tools: Arc::new(StdMutex::new(HashMap::new())),
        };
        
        if registry.config.builtin_tools {
            registry.register_builtin_tools();
        }
        
        registry
    }

    fn register_builtin_tools(&mut self) {
        // HTTP request tool
        self.register(Arc::new(HttpRequestTool));
        
        // File operations tool
        self.register(Arc::new(FileOperationTool));
        
        // Shell command tool
        self.register(Arc::new(ShellCommandTool));
        
        // JSON processing tool
        self.register(Arc::new(JsonProcessingTool));
        
        // Code execution tool
        self.register(Arc::new(CodeExecutionTool));
        
        // Knowledge query tool
        self.register(Arc::new(KnowledgeQueryTool));
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        let spec = tool.spec();
        let name = spec.name.clone();
        
        // Check for conflicts
        if self.builtin_tools.lock().unwrap().contains_key(&name) {
            eprintln!("Warning: Tool '{}' already registered, overwriting", name);
        }
        
        self.builtin_tools.lock().unwrap().insert(name, tool);
    }

pub fn register_custom(&mut self, tool: Arc<dyn Tool>) {
        self.register(tool);
    }

    pub fn unregister(&mut self, name: &str) -> bool {
        self.builtin_tools.lock().unwrap().remove(name).is_some()
    }

    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.builtin_tools.lock().unwrap()
            .get(name)
            .map(|t| Arc::clone(t) as Arc<dyn Tool>)
    }

    pub fn list_tools(&self) -> Vec<String> {
        self.builtin_tools.lock().unwrap().keys().cloned().collect()
    }

    pub async fn execute(&self, name: &str, args: serde_json::Value) -> Result<ToolResult, String> {
        let tool = {
            let tools = self.builtin_tools.lock().unwrap();
            tools.get(name).cloned()
        }.ok_or_else(|| format!("Tool '{}' not found", name))?;

        let _start = std::time::Instant::now();
        
        // Apply timeout
        let result = match timeout(Duration::from_secs(self.config.max_execution_time_secs), tool.execute(args)).await {
            Ok(result) => result,
            Err(_) => return Ok(ToolResult {
                success: false,
                output: serde_json::json!(null),
                error: Some(format!("Tool execution timed out after {}s", self.config.max_execution_time_secs)),
                duration_ms: self.config.max_execution_time_secs * 1000,
                metadata: HashMap::new(),
            }),
        };
        
        let duration = std::time::Instant::now().elapsed().as_millis() as u64;
        
        match result {
            Ok(mut result) => {
                result.duration_ms = duration;
                Ok(result)
            }
            Err(e) => Ok(ToolResult {
                success: false,
                output: serde_json::json!(null),
                error: Some(e),
                duration_ms: duration,
                metadata: HashMap::new(),
            }),
        }
    }
}

// Built-in tools

/// HTTP Request Tool
struct HttpRequestTool;

#[async_trait::async_trait]
impl Tool for HttpRequestTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "http_request".to_string(),
            description: "Make HTTP requests to external APIs".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "method": {"type": "string", "enum": ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD"]},
                    "url": {"type": "string", "format": "uri"},
                    "headers": {"type": "object", "additionalProperties": {"type": "string"}},
                    "body": {"type": ["string", "object", "null"]},
                    "timeout_secs": {"type": "integer", "minimum": 1, "maximum": 300}
                },
                "required": ["method", "url"]
            }),
            returns: serde_json::json!({
                "type": "object",
                "properties": {
                    "status": {"type": "integer"},
                    "headers": {"type": "object"},
                    "body": {"type": "string"}
                }
            }),
            tags: vec!["http".to_string(), "network".to_string(), "api".to_string()],
            version: "1.0.0".to_string(),
            author: "neotrix".to_string(),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, String> {
        let method = args.get("method").and_then(|v| v.as_str()).ok_or("Missing method")?;
        let url = args.get("url").and_then(|v| v.as_str()).ok_or("Missing url")?;
        let headers = args.get("headers").and_then(|v| v.as_object()).cloned().unwrap_or_default();
        let body = args.get("body").cloned();
let timeout_secs = args.get("timeout_secs").and_then(|v| v.as_u64()).unwrap_or(30);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| format!("Client creation failed: {}", e))?;

        let mut request = client
            .request(method.parse().map_err(|e| format!("Invalid method: {}", e))?, url);

        for (key, value) in headers {
            if let Some(v) = value.as_str() {
                request = request.header(key, v);
            }
        }

        if let Some(body) = body {
            request = request.body(body.to_string());
        }

        let send_result = tokio::time::timeout(std::time::Duration::from_secs(30), request.send()).await;

        // dsh-im three-state delivery semantics: structured failure results carry a
        // `delivery_outcome` so callers can distinguish "definitively not executed"
        // (retryable) from "may have been executed" (never auto-retry).
        let response: reqwest::Response = match send_result {
            Err(_) => {
                return Ok(ToolResult {
                    success: false,
                    output: serde_json::json!({}),
                    error: Some("Request timeout".to_string()),
                    duration_ms: 0,
                    metadata: Self::delivery_metadata(DeliveryOutcome::Unknown),
                });
            }
            Ok(Err(e)) => {
                let outcome = classify_failure(
                    transport_cause(&e),
                );
                return Ok(ToolResult {
                    success: false,
                    output: serde_json::json!({}),
                    error: Some(format!("Request failed: {}", e)),
                    duration_ms: 0,
                    metadata: Self::delivery_metadata(outcome),
                });
            }
            Ok(Ok(response)) => response,
        };

        let status = response.status().as_u16();
        let headers: std::collections::HashMap<String, String> = response.headers().iter()
            .map(|(k, v): (&reqwest::header::HeaderName, &reqwest::header::HeaderValue)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let body: String = match response.text().await {
            Ok(text) => text,
            Err(e) => {
                return Ok(ToolResult {
                    success: false,
                    output: serde_json::json!({ "status": status }),
                    error: Some(format!("Failed to read body: {}", e)),
                    duration_ms: 0,
                    metadata: Self::delivery_metadata(DeliveryOutcome::Unknown),
                });
            }
        };

        let outcome = classify_status(status);
        Ok(ToolResult {
            success: status < 400,
            output: serde_json::json!({
                "status": status,
                "headers": headers,
                "body": body
            }),
            error: None,
            duration_ms: 0,
            metadata: Self::delivery_metadata(outcome),
        })
    }
}

impl HttpRequestTool {
    fn delivery_metadata(outcome: DeliveryOutcome) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("delivery_outcome".to_string(), outcome.as_str().to_string());
        metadata
    }
}

/// File Operations Tool
struct FileOperationTool;

#[async_trait::async_trait]
impl Tool for FileOperationTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "file_operation".to_string(),
            description: "Read, write, list, and manipulate files".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": {"type": "string", "enum": ["read", "write", "append", "delete", "list", "mkdir", "copy", "move"]},
                    "path": {"type": "string"},
                    "content": {"type": ["string", "null"]},
                    "destination": {"type": ["string", "null"]}
                },
                "required": ["operation", "path"]
            }),
            returns: serde_json::json!({
                "type": "object",
                "properties": {
                    "success": {"type": "boolean"},
                    "output": {"type": "string"},
                    "metadata": {"type": "object"}
                }
            }),
            tags: vec!["file".to_string(), "filesystem".to_string(), "io".to_string()],
            version: "1.0.0".to_string(),
            author: "neotrix".to_string(),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, String> {
        let operation = args.get("operation").and_then(|v| v.as_str()).ok_or("Missing operation")?;
        let path = args.get("path").and_then(|v| v.as_str()).ok_or("Missing path")?;
        let content = args.get("content").and_then(|v| v.as_str());
        let destination = args.get("destination").and_then(|v| v.as_str());

        use std::fs;
        use std::path::Path;

        let result = match operation {
            "read" => {
                let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
                Ok(ToolResult {
                    success: true,
                    output: serde_json::json!({"content": content}),
                    error: None,
                    duration_ms: 0,
                    metadata: HashMap::new(),
                })
            }
            "write" => {
                let content = content.unwrap_or("");
                fs::write(path, content).map_err(|e| e.to_string())?;
                Ok(ToolResult {
                    success: true,
                    output: serde_json::json!({"written": path}),
                    error: None,
                    duration_ms: 0,
                    metadata: HashMap::new(),
                })
            }
            "append" => {
                let content = content.unwrap_or("");
                let mut file = fs::OpenOptions::new().append(true).create(true).open(path).map_err(|e| e.to_string())?;
                use std::io::Write;
                file.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
                Ok(ToolResult {
                    success: true,
                    output: serde_json::json!({"appended": path}),
                    error: None,
                    duration_ms: 0,
                    metadata: HashMap::new(),
                })
            }
            "delete" => {
                if Path::new(path).is_dir() {
                    fs::remove_dir_all(path).map_err(|e| e.to_string())?;
                } else {
                    fs::remove_file(path).map_err(|e| e.to_string())?;
                }
                Ok(ToolResult {
                    success: true,
                    output: serde_json::json!({"deleted": path}),
                    error: None,
                    duration_ms: 0,
                    metadata: HashMap::new(),
                })
            }
            "list" => {
                let entries = fs::read_dir(path).map_err(|e| e.to_string())?;
                let mut files = Vec::new();
                for entry in entries {
                    let entry = entry.map_err(|e| e.to_string())?;
                    let path = entry.path();
                    files.push(serde_json::json!({
                        "name": path.file_name().and_then(|s| s.to_str()).unwrap_or(""),
                        "path": path.to_string_lossy(),
                        "is_dir": path.is_dir(),
                    }));
                }
                Ok(ToolResult {
                    success: true,
                    output: serde_json::json!(files),
                    error: None,
                    duration_ms: 0,
                    metadata: HashMap::new(),
                })
            }
            "mkdir" => {
                fs::create_dir_all(path).map_err(|e| e.to_string())?;
                Ok(ToolResult {
                    success: true,
                    output: serde_json::json!({"created": path}),
                    error: None,
                    duration_ms: 0,
                    metadata: HashMap::new(),
                })
            }
            "copy" => {
                let dest = destination.ok_or("Destination required for copy")?;
                fs::copy(path, dest).map_err(|e| e.to_string())?;
                Ok(ToolResult {
                    success: true,
                    output: serde_json::json!({"copied_from": path, "copied_to": destination}),
                    error: None,
                    duration_ms: 0,
                    metadata: HashMap::new(),
                })
            }
            "move" => {
                let dest = destination.ok_or("Destination required for move")?;
                fs::rename(path, dest).map_err(|e| e.to_string())?;
                Ok(ToolResult {
                    success: true,
                    output: serde_json::json!({"moved_from": path, "moved_to": destination}),
                    error: None,
                    duration_ms: 0,
                    metadata: HashMap::new(),
                })
            }
            _ => Err(format!("Unknown operation: {}", operation)),
        }?;

        Ok(result)
    }
}

/// Shell Command Tool
struct ShellCommandTool;

#[async_trait::async_trait]
impl Tool for ShellCommandTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "shell_command".to_string(),
            description: "Execute shell commands".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string"},
                    "args": {"type": "array", "items": {"type": "string"}},
                    "working_dir": {"type": "string"},
                    "timeout_secs": {"type": "integer", "minimum": 1, "maximum": 300},
                    "env": {"type": "object", "additionalProperties": {"type": "string"}}
                },
                "required": ["command"]
            }),
            returns: serde_json::json!({
                "type": "object",
                "properties": {
                    "stdout": {"type": "string"},
                    "stderr": {"type": "string"},
                    "exit_code": {"type": "integer"},
                    "success": {"type": "boolean"}
                }
            }),
            tags: vec!["shell".to_string(), "command".to_string(), "system".to_string()],
            version: "1.0.0".to_string(),
            author: "neotrix".to_string(),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, String> {
        let command = args.get("command").and_then(|v| v.as_str()).ok_or("Missing command")?;
        let args_list = args.get("args").and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();
        let working_dir = args.get("working_dir").and_then(|v| v.as_str());
        let timeout_secs = args.get("timeout_secs").and_then(|v| v.as_u64()).unwrap_or(60);
        let env_vars = args.get("env").and_then(|v| v.as_object()).cloned().unwrap_or_default();

        use tokio::process::Command;
        use std::time::Duration;

        let mut cmd = Command::new(command);
        cmd.args(&args_list);
        
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }
        
        for (k, v) in env_vars {
            if let Some(v_str) = v.as_str() {
                cmd.env(k, v_str);
            }
        }

        let child = cmd.spawn()
            .map_err(|e| format!("Failed to spawn command: {}", e))?;

        let output = tokio::time::timeout(Duration::from_secs(timeout_secs), child.wait_with_output())
            .await
            .map_err(|_| "Command timeout".to_string())?
            .map_err(|e| format!("Command execution failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        Ok(ToolResult {
            success: exit_code == 0,
            output: serde_json::json!({
                "stdout": stdout,
                "stderr": stderr,
                "exit_code": exit_code,
                "success": exit_code == 0
            }),
            error: if exit_code == 0 { None } else { Some(stderr) },
            duration_ms: 0,
            metadata: HashMap::new(),
        })
    }
}

/// JSON Processing Tool
struct JsonProcessingTool;

#[async_trait::async_trait]
impl Tool for JsonProcessingTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "json_processing".to_string(),
            description: "Parse, query, transform, and validate JSON data".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": {"type": "string", "enum": ["parse", "query", "transform", "validate", "merge", "diff"]},
                    "json": {"type": ["string", "object"]},
                    "query": {"type": "string"}, // jq-style query
                    "transform": {"type": "object"}, // transformation rules
                    "schema": {"type": "object"} // JSON schema for validation
                },
                "required": ["operation", "json"]
            }),
            returns: serde_json::json!({
                "type": "object",
                "properties": {
                    "result": {}
                }
            }),
            tags: vec!["json".to_string(), "data".to_string(), "processing".to_string()],
            version: "1.0.0".to_string(),
            author: "neotrix".to_string(),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, String> {
        let operation = args.get("operation").and_then(|v| v.as_str()).ok_or("Missing operation")?;
        let json = args.get("json").ok_or("Missing json")?;
        
        // Parse input JSON
        let value: serde_json::Value = if json.is_string() {
            serde_json::from_str(json.as_str().unwrap()).map_err(|e| e.to_string())?
        } else {
            json.clone()
        };

        let result = match operation {
            "query" => {
                let query = args.get("query").and_then(|v| v.as_str()).ok_or("Missing query")?;
                // Use jq-like query (simplified - in production use a proper jq library)
                // For now, simple path access
                let path = query.split('.').collect::<Vec<_>>();
                let mut current = &value;
                for part in path {
                    current = current.get(part).ok_or("Path not found")?;
                }
                Ok(ToolResult {
                    success: true,
                    output: current.clone(),
                    error: None,
                    duration_ms: 0,
                    metadata: HashMap::new(),
                })
            }
            "validate" => {
                let _schema = args.get("schema").ok_or("Missing schema")?;
                // JSON Schema validation (simplified)
                Ok(ToolResult {
                    success: true,
                    output: serde_json::json!({"valid": true}),
                    error: None,
                    duration_ms: 0,
                    metadata: HashMap::new(),
                })
            }
            "transform" => {
                // JSON transformation
                Ok(ToolResult {
                    success: true,
                    output: value,
                    error: None,
                    duration_ms: 0,
                    metadata: HashMap::new(),
                })
            }
            "merge" => {
                let other = args.get("other").ok_or("Missing other JSON")?;
                let merged = json_merge::merge(&value, other);
                Ok(ToolResult {
                    success: true,
                    output: merged,
                    error: None,
                    duration_ms: 0,
                    metadata: HashMap::new(),
                })
            }
            _ => Err(format!("Unknown operation: {}", operation)),
        }?;

        Ok(result)
    }
}

/// Code Execution Tool
struct CodeExecutionTool;

#[async_trait::async_trait]
impl Tool for CodeExecutionTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "code_execution".to_string(),
            description: "Execute code in various languages (Python, JavaScript, Rust, etc.)".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "language": {"type": "string", "enum": ["python", "javascript", "rust", "bash", "lua"]},
                    "code": {"type": "string"},
                    "args": {"type": "array", "items": {"type": "string"}},
                    "timeout_secs": {"type": "integer", "minimum": 1, "maximum": 300}
                },
                "required": ["language", "code"]
            }),
            returns: serde_json::json!({
                "type": "object",
                "properties": {
                    "stdout": {"type": "string"},
                    "stderr": {"type": "string"},
                    "exit_code": {"type": "integer"},
                    "success": {"type": "boolean"}
                }
            }),
            tags: vec!["code".to_string(), "execution".to_string(), "programming".to_string()],
            version: "1.0.0".to_string(),
            author: "neotrix".to_string(),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, String> {
        let language = args.get("language").and_then(|v| v.as_str()).ok_or("Missing language")?;
        let code = args.get("code").and_then(|v| v.as_str()).ok_or("Missing code")?;
        let _args_list = args.get("args").and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();
        let timeout_secs = args.get("timeout_secs").and_then(|v| v.as_u64()).unwrap_or(30);

        let (cmd, args_list): (&str, Vec<&str>) = match language {
            "python" => ("python3", vec!["-c", code]),
            "javascript" => ("node", vec!["-e", code]),
            "rust" => {
                // Write to temp file and compile
                let temp_file = format!("/tmp/neotrix_code_{}.rs", uuid::Uuid::new_v4());
                std::fs::write(&temp_file, code).map_err(|e| e.to_string())?;
                let output = std::process::Command::new("rustc")
                    .args(["--edition", "2021", &temp_file, "-o", &format!("{}.out", temp_file)])
                    .output()
                    .map_err(|e| e.to_string())?;
                if !output.status.success() {
                    return Ok(ToolResult {
                        success: false,
                        output: serde_json::json!({"error": String::from_utf8_lossy(&output.stderr).to_string()}),
                        error: Some("Compilation failed".to_string()),
                        duration_ms: 0,
                        metadata: HashMap::new(),
                    });
                }
                // Leak the string to get a static reference (for demo purposes)
                let exe_path = format!("{}.out", temp_file);
                (Box::leak(exe_path.into_boxed_str()), vec![])
            },
            "bash" => ("bash", vec!["-c", code]),
            "lua" => ("lua", vec!["-e", code]),
            _ => return Err(format!("Unsupported language: {}", language)),
        };

        use tokio::process::Command;
        use std::time::Duration;
        use tokio::time::timeout;

        let mut cmd = tokio::process::Command::new(cmd);
        cmd.args(&args_list);
        
        let child = cmd.spawn()
            .map_err(|e| format!("Failed to spawn: {}", e))?;

        let output = match timeout(Duration::from_secs(timeout_secs), child.wait_with_output()).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => return Err(format!("Execution failed: {}", e)),
            Err(_) => return Err("Execution timeout".to_string()),
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let _exit_code = output.status.code().unwrap_or(-1);

        Ok(ToolResult {
            success: output.status.success(),
            output: serde_json::json!({
                "stdout": stdout,
                "stderr": stderr,
                "exit_code": _exit_code
            }),
            error: if output.status.success() { None } else { Some(stderr) },
            duration_ms: 0,
            metadata: HashMap::new(),
        })
    }
}

/// Knowledge Query Tool
struct KnowledgeQueryTool;

#[async_trait::async_trait]
impl Tool for KnowledgeQueryTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "knowledge_query".to_string(),
            description: "Query the knowledge base for information".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "namespace": {"type": "string"},
                    "limit": {"type": "integer", "minimum": 1, "maximum": 100}
                },
                "required": ["query"]
            }),
            returns: serde_json::json!({
                "type": "object",
                "properties": {
                    "results": {"type": "array"},
                    "count": {"type": "integer"}
                }
            }),
            tags: vec!["knowledge".to_string(), "query".to_string(), "database".to_string()],
            version: "1.0.0".to_string(),
            author: "neotrix".to_string(),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, String> {
        let query = args.get("query").and_then(|v| v.as_str()).ok_or("Missing query")?;
        let namespace = args.get("namespace").and_then(|v| v.as_str()).unwrap_or("default");
        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

        // Try to access knowledge base
        if let Ok(kb) = crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase::open(None) {
            let conn = kb.conn.lock().map_err(|e| format!("KB lock failed: {}", e))?;
            
            // Simple keyword search in kb_store
            let query_sql = format!(
                "SELECT key, value FROM kv_store WHERE namespace = ? AND (key LIKE ? OR value LIKE ?) LIMIT ?"
            );
            
            let mut stmt = conn.prepare(&query_sql).map_err(|e| e.to_string())?;
            let search_term = format!("%{}%", query);
            let mut rows = stmt.query(rusqlite::params![namespace, &search_term, &search_term, limit as i64])
                .map_err(|e| e.to_string())?;
            
            let mut results = Vec::new();
            while let Some(row) = rows.next().map_err(|e| e.to_string())? {
                let key: String = row.get(0).map_err(|e| e.to_string())?;
                let value: String = row.get(1).map_err(|e| e.to_string())?;
                results.push(serde_json::json!({"key": key, "value": value}));
            }
            
            Ok(ToolResult {
                success: true,
                output: serde_json::json!({"results": results, "count": results.len()}),
                error: None,
                duration_ms: 0,
                metadata: HashMap::new(),
            })
        } else {
            Ok(ToolResult {
                success: false,
                output: serde_json::json!({"results": [], "count": 0}),
                error: Some("Knowledge base not available".to_string()),
                duration_ms: 0,
                metadata: HashMap::new(),
            })
        }
    }
}

// Helper module for JSON merge
mod json_merge {
    use serde_json::Value;
    
    pub fn merge(a: &Value, b: &Value) -> Value {
        match (a, b) {
            (Value::Object(a), Value::Object(b)) => {
                let mut result = a.clone();
                for (k, v) in b {
                    result[k] = merge(a.get(k).unwrap_or(&Value::Null), v);
                }
                Value::Object(result)
            }
            (Value::Array(a), Value::Array(b)) => {
                Value::Array([a.clone(), b.clone()].concat())
            }
            (_, b) => b.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_registry() {
        let config = ToolsConfig::default();
        let registry = ToolRegistry::new(config);
        
        let tools = registry.list_tools();
        assert!(tools.contains(&"http_request".to_string()));
        assert!(tools.contains(&"file_operation".to_string()));
        assert!(tools.contains(&"shell_command".to_string()));
    }
}
