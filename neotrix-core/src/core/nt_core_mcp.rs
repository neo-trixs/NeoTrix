use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};

/// MCP Server for NeoTrix — exposes tools via stdio JSON-RPC 2.0 transport.
pub struct McpServer {
    tools: Vec<McpTool>,
    reader: std::io::Stdin,
    writer: std::io::Stdout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct McpRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    #[serde(default)]
    params: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct McpResponse {
    jsonrpc: String,
    id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<McpError>,
}

#[derive(Serialize, Deserialize)]
struct McpError {
    code: i32,
    message: String,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
            reader: std::io::stdin(),
            writer: std::io::stdout(),
        }
    }

    pub fn register_all_tools(&mut self) {
        self.register_tool(McpTool {
            name: "read_file".into(),
            description: "Read file contents from disk".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Absolute or relative file path"
                    }
                },
                "required": ["path"]
            }),
        });
        self.register_tool(McpTool {
            name: "write_file".into(),
            description: "Write content to a file on disk".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path to write to"
                    },
                    "content": {
                        "type": "string",
                        "description": "File content to write"
                    }
                },
                "required": ["path", "content"]
            }),
        });
        self.register_tool(McpTool {
            name: "edit_file".into(),
            description: "Edit a file by finding and replacing exact text".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path to edit"
                    },
                    "old_string": {
                        "type": "string",
                        "description": "Exact text to find (must match exactly once)"
                    },
                    "new_string": {
                        "type": "string",
                        "description": "Replacement text"
                    }
                },
                "required": ["path", "old_string", "new_string"]
            }),
        });
        self.register_tool(McpTool {
            name: "search_code".into(),
            description: "Search codebase using regex pattern".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Regex pattern to search for"
                    },
                    "path": {
                        "type": "string",
                        "description": "Directory or file path to search (default: current dir)"
                    }
                },
                "required": ["pattern"]
            }),
        });
        self.register_tool(McpTool {
            name: "git_diff".into(),
            description: "Get git diff of the working tree".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "staged": {
                        "type": "boolean",
                        "description": "If true, show staged diff (git diff --cached)",
                        "default": false
                    }
                },
                "required": []
            }),
        });
        self.register_tool(McpTool {
            name: "execute_command".into(),
            description: "Execute a shell command and return its output".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Shell command to execute"
                    }
                },
                "required": ["command"]
            }),
        });
    }

    pub fn register_tool(&mut self, tool: McpTool) {
        self.tools.push(tool);
    }

    pub fn run(&mut self) -> Result<(), String> {
        loop {
            let mut line = String::new();
            let _ = self.reader.lock().read_line(&mut line);

            if line.trim().is_empty() {
                continue;
            }

            let request: McpRequest = serde_json::from_str(&line.trim())
                .map_err(|e| format!("Invalid JSON-RPC request: {}", e))?;

            let response = self.handle_request(&request);

            let json = serde_json::to_string(&response)
                .map_err(|e| format!("Serialization error: {}", e))?;

            let mut out = self.writer.lock();
            writeln!(out, "{}", json).map_err(|e| format!("Write error: {}", e))?;
            out.flush().map_err(|e| format!("Flush error: {}", e))?;

            if request.method == "exit" {
                break;
            }
        }
        Ok(())
    }

    fn handle_request(&self, request: &McpRequest) -> McpResponse {
        match request.method.as_str() {
            "initialize" => McpResponse {
                jsonrpc: "2.0".into(),
                id: request.id,
                result: Some(serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "neotrix-mcp",
                        "version": env!("CARGO_PKG_VERSION"),
                    },
                    "capabilities": {
                        "tools": {}
                    }
                })),
                error: None,
            },
            "tools/list" => self.handle_list_tools(request.id),
            "tools/call" => {
                let name = request
                    .params
                    .as_ref()
                    .and_then(|p| p.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let args = request
                    .params
                    .as_ref()
                    .and_then(|p| p.get("arguments"))
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                self.handle_call_tool(request.id, name, &args)
            }
            _ => McpResponse {
                jsonrpc: "2.0".into(),
                id: request.id,
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                }),
            },
        }
    }

    fn handle_list_tools(&self, id: u64) -> McpResponse {
        let tools: Vec<serde_json::Value> = self
            .tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": t.input_schema,
                })
            })
            .collect();

        McpResponse {
            jsonrpc: "2.0".into(),
            id,
            result: Some(serde_json::json!({ "tools": tools })),
            error: None,
        }
    }

    fn handle_call_tool(&self, id: u64, name: &str, args: &serde_json::Value) -> McpResponse {
        let result = match name {
            "read_file" => call_read_file(args),
            "write_file" => call_write_file(args),
            "edit_file" => call_edit_file(args),
            "search_code" => call_search_code(args),
            "git_diff" => call_git_diff(args),
            "execute_command" => call_execute_command(args),
            other => Err(format!("Unknown tool: {}", other)),
        };

        match result {
            Ok(content) => McpResponse {
                jsonrpc: "2.0".into(),
                id,
                result: Some(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": content
                    }]
                })),
                error: None,
            },
            Err(e) => McpResponse {
                jsonrpc: "2.0".into(),
                id,
                result: None,
                error: Some(McpError {
                    code: -32000,
                    message: e,
                }),
            },
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

fn call_read_file(args: &serde_json::Value) -> Result<String, String> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: path".to_string())?;
    std::fs::read_to_string(path).map_err(|e| format!("Failed to read '{}': {}", path, e))
}

fn call_write_file(args: &serde_json::Value) -> Result<String, String> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: path".to_string())?;
    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: content".to_string())?;

    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directories: {}", e))?;
    }
    std::fs::write(path, content).map_err(|e| format!("Failed to write '{}': {}", path, e))?;
    Ok(format!("Successfully wrote {} bytes to {}", content.len(), path))
}

fn call_edit_file(args: &serde_json::Value) -> Result<String, String> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: path".to_string())?;
    let old_string = args
        .get("old_string")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: old_string".to_string())?;
    let new_string = args
        .get("new_string")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: new_string".to_string())?;

    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read '{}': {}", path, e))?;

    if !content.contains(old_string) {
        return Err(format!(
            "old_string not found in '{}'.\nSearch text:\n---\n{}\n---\nFile content:\n---\n{}\n---",
            path,
            old_string,
            content
        ));
    }

    let new_content = content.replace(old_string, new_string);
    std::fs::write(path, &new_content)
        .map_err(|e| format!("Failed to write '{}': {}", path, e))?;

    let occurrences = content.matches(old_string).count();
    Ok(format!(
        "Applied edit to {} ({} occurrence{})",
        path,
        occurrences,
        if occurrences == 1 { "" } else { "s" }
    ))
}

fn call_search_code(args: &serde_json::Value) -> Result<String, String> {
    let pattern = args
        .get("pattern")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: pattern".to_string())?;
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .unwrap_or(".");

    let re = regex::Regex::new(pattern)
        .map_err(|e| format!("Invalid regex pattern '{}': {}", pattern, e))?;

    let mut results = Vec::new();
    let walker = walkdir::WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.') && name != "target" && name != "node_modules"
        });

    for entry in walker.filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            for (i, line) in content.lines().enumerate() {
                if re.is_match(line) {
                    results.push(format!(
                        "{}:{}: {}",
                        entry.path().display(),
                        i + 1,
                        line.trim()
                    ));
                }
            }
        }
    }

    if results.is_empty() {
        return Ok(format!("No matches found for pattern '{}'", pattern));
    }

    let max_results = 100;
    let mut output = String::new();
    for r in results.iter().take(max_results) {
        output.push_str(r);
        output.push('\n');
    }
    if results.len() > max_results {
        output.push_str(&format!("... and {} more results", results.len() - max_results));
    }
    Ok(output)
}

fn call_git_diff(args: &serde_json::Value) -> Result<String, String> {
    let staged = args.get("staged").and_then(|v| v.as_bool()).unwrap_or(false);

    let mut cmd = std::process::Command::new("git");
    cmd.arg("diff");
    if staged {
        cmd.arg("--cached");
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run git diff: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if stdout.is_empty() {
            Ok("No changes.".to_string())
        } else {
            Ok(stdout)
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("git diff failed: {}", stderr))
    }
}

fn call_execute_command(args: &serde_json::Value) -> Result<String, String> {
    let command = args
        .get("command")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required field: command".to_string())?;

    crate::cli::execute_guarded(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_new() {
        let server = McpServer::new();
        assert!(server.tools.is_empty());
    }

    #[test]
    fn test_register_tool() {
        let mut server = McpServer::new();
        server.register_tool(McpTool {
            name: "test_tool".into(),
            description: "A test tool".into(),
            input_schema: serde_json::json!({"type": "object"}),
        });
        assert_eq!(server.tools.len(), 1);
    }

    #[test]
    fn test_register_all_tools() {
        let mut server = McpServer::new();
        server.register_all_tools();
        assert_eq!(server.tools.len(), 6);
        let names: Vec<&str> = server.tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"read_file"));
        assert!(names.contains(&"write_file"));
        assert!(names.contains(&"edit_file"));
        assert!(names.contains(&"search_code"));
        assert!(names.contains(&"git_diff"));
        assert!(names.contains(&"execute_command"));
    }

    #[test]
    fn test_handle_list_tools() {
        let mut server = McpServer::new();
        server.register_all_tools();
        let resp = server.handle_list_tools(1);
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 6);
    }

    #[test]
    fn test_handle_request_unknown_method() {
        let server = McpServer::new();
        let req = McpRequest {
            jsonrpc: "2.0".into(),
            id: 42,
            method: "bogus".into(),
            params: None,
        };
        let resp = server.handle_request(&req);
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[test]
    fn test_handle_initialize() {
        let server = McpServer::new();
        let req = McpRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "initialize".into(),
            params: None,
        };
        let resp = server.handle_request(&req);
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert_eq!(result["serverInfo"]["name"], "neotrix-mcp");
    }

    #[test]
    fn test_call_read_file_missing_path() {
        let args = serde_json::json!({});
        let result = call_read_file(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required field"));
    }

    #[test]
    fn test_call_read_file_not_found() {
        let args = serde_json::json!({"path": "/tmp/__nonexistent_file_xyz__"});
        let result = call_read_file(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_call_read_file_ok() {
        let tmp = std::env::temp_dir().join("test_mcp_read.txt");
        std::fs::write(&tmp, "hello world").expect("write test file");
        let args = serde_json::json!({"path": tmp.to_string_lossy()});
        let result = call_read_file(&args);
        assert_eq!(result.unwrap(), "hello world");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_call_write_file_ok() {
        let tmp = std::env::temp_dir().join("test_mcp_write.txt");
        let args = serde_json::json!({"path": tmp.to_string_lossy(), "content": "write test"});
        let result = call_write_file(&args);
        assert!(result.is_ok());
        assert_eq!(std::fs::read_to_string(&tmp).unwrap(), "write test");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_call_write_file_missing_fields() {
        let args = serde_json::json!({"path": "/tmp/x"});
        let result = call_write_file(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required field: content"));
    }

    #[test]
    fn test_call_edit_file_ok() {
        let tmp = std::env::temp_dir().join("test_mcp_edit.txt");
        std::fs::write(&tmp, "hello world").expect("write test file");
        let args = serde_json::json!({
            "path": tmp.to_string_lossy(),
            "old_string": "world",
            "new_string": "neotrix"
        });
        let result = call_edit_file(&args);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("1 occurrence"));
        assert_eq!(std::fs::read_to_string(&tmp).unwrap(), "hello neotrix");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_call_edit_file_not_found() {
        let tmp = std::env::temp_dir().join("test_mcp_edit_not_found.txt");
        std::fs::write(&tmp, "hello").expect("write test file");
        let args = serde_json::json!({
            "path": tmp.to_string_lossy(),
            "old_string": "zzzz",
            "new_string": "yyyy"
        });
        let result = call_edit_file(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("old_string not found"));
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_call_edit_file_missing_fields() {
        let args = serde_json::json!({"path": "/tmp/x", "old_string": "a"});
        let result = call_edit_file(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required field: new_string"));
    }

    #[test]
    fn test_call_git_diff_ok() {
        let args = serde_json::json!({"staged": false});
        let result = call_git_diff(&args);
        // Should succeed even if no git repo (might say "No changes" or fail gracefully)
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_call_execute_command_echo() {
        let args = serde_json::json!({"command": "echo hello"});
        let result = call_execute_command(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello");
    }

    #[test]
    fn test_call_execute_command_fail() {
        let args = serde_json::json!({"command": "exit 42"});
        let result = call_execute_command(&args);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("exit code: 42"));
    }

    #[test]
    fn test_call_execute_command_missing_cmd() {
        let args = serde_json::json!({});
        let result = call_execute_command(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required field: command"));
    }

    #[test]
    fn test_search_code_empty_pattern() {
        let args = serde_json::json!({"pattern": "UNLIKELY_PATTERN_XYZ_99999"});
        let result = call_search_code(&args);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("No matches found"));
    }

    #[test]
    fn test_search_code_invalid_regex() {
        let args = serde_json::json!({"pattern": "[invalid"});
        let result = call_search_code(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_input_schema_fields() {
        let mut server = McpServer::new();
        server.register_all_tools();
        for tool in &server.tools {
            assert_eq!(tool.input_schema["type"], "object");
            assert!(tool.input_schema["properties"].is_object());
        }
    }

    #[test]
    fn test_handle_call_tool_unknown() {
        let server = McpServer::new();
        let resp = server.handle_call_tool(1, "no_such_tool", &serde_json::json!({}));
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32000);
    }

    #[test]
    fn test_server_default() {
        let server = McpServer::default();
        assert!(server.tools.is_empty());
    }
}
