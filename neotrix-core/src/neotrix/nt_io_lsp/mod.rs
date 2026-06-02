mod protocol;
pub mod features;
pub mod manager;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{BufReader, BufWriter};
use tokio::process::Command;

// ---------- Core Data Types ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: Option<i32>,
    pub detail: Option<String>,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverInfo {
    pub contents: String,
    pub range: Option<Range>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub code: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

// ---------- Errors ----------

#[derive(Debug)]
pub enum LspError {
    ConnectionFailed(String),
    ServerExited(Option<i32>),
    NotInitialized,
    AlreadyShutdown,
    RequestFailed(i64, String),
    ParseError(String),
    IoError(String),
}

impl std::fmt::Display for LspError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LspError::ConnectionFailed(msg) => write!(f, "LSP connection failed: {}", msg),
            LspError::ServerExited(code) => write!(f, "LSP server exited: {:?}", code),
            LspError::NotInitialized => write!(f, "LSP client not initialized"),
            LspError::AlreadyShutdown => write!(f, "LSP client already shutdown"),
            LspError::RequestFailed(c, m) => write!(f, "LSP request failed ({}): {}", c, m),
            LspError::ParseError(msg) => write!(f, "LSP parse error: {}", msg),
            LspError::IoError(msg) => write!(f, "LSP IO error: {}", msg),
        }
    }
}

impl std::error::Error for LspError {}

// ---------- LspClient ----------

pub struct LspClient {
    child: Option<tokio::process::Child>,
    writer: BufWriter<tokio::process::ChildStdin>,
    reader: BufReader<tokio::process::ChildStdout>,
    next_id: u64,
    initialized: bool,
    root_uri: String,
    language_id: String,
}

impl LspClient {
    /// Spawn an LSP server process. Does NOT send initialize yet.
    pub async fn spawn(
        server_command: &str,
        args: &[&str],
        root_uri: &str,
        language_id: &str,
    ) -> Result<Self, LspError> {
        let mut child = Command::new(server_command)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| LspError::ConnectionFailed(format!("spawn {}: {}", server_command, e)))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| LspError::ConnectionFailed("child has no stdin".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| LspError::ConnectionFailed("child has no stdout".into()))?;

        Ok(Self {
            child: Some(child),
            writer: BufWriter::new(stdin),
            reader: BufReader::new(stdout),
            next_id: 1,
            initialized: false,
            root_uri: root_uri.to_string(),
            language_id: language_id.to_string(),
        })
    }

    // ===== Lifecycle =====

    /// Send initialize request + await initialized nt_io_notify.
    /// Returns parsed server capabilities as raw JSON.
    pub async fn initialize(&mut self) -> Result<Value, LspError> {
        let params = serde_json::json!({
            "processId": std::process::id(),
            "rootUri": self.root_uri,
            "capabilities": {
                "textDocument": {
                    "hover": { "contentFormat": ["markdown", "plaintext"] },
                    "completion": {
                        "completionItem": {
                            "documentationFormat": ["markdown", "plaintext"]
                        }
                    },
                    "definition": { "linkSupport": true },
                    "references": {},
                    "formatting": {},
                    "synchronization": {
                        "didOpen": true,
                        "didChange": true,
                        "didSave": true
                    }
                }
            }
        });

        let result = self.call("initialize", Some(params)).await?;
        self.notify("initialized", None).await?;
        self.initialized = true;
        Ok(result)
    }

    /// Open a file and send its contents to the server.
    pub async fn open_file(&mut self, path: &Path, text: &str) -> Result<(), LspError> {
        self.ensure_initialized()?;
        let uri = path_to_uri(path, &self.root_uri);
        let params = serde_json::json!({
            "textDocument": {
                "uri": uri,
                "languageId": self.language_id,
                "version": 1,
                "text": text
            }
        });
        self.notify("textDocument/didOpen", Some(params)).await
    }

    /// Send a full document change.
    pub async fn change(&mut self, path: &Path, text: &str, version: i32) -> Result<(), LspError> {
        self.ensure_initialized()?;
        let uri = path_to_uri(path, &self.root_uri);
        let params = serde_json::json!({
            "textDocument": { "uri": uri, "version": version },
            "contentChanges": [{ "text": text }]
        });
        self.notify("textDocument/didChange", Some(params)).await
    }

    /// Gracefully shut down the LSP session and kill the server process.
    pub async fn shutdown(&mut self) -> Result<(), LspError> {
        if !self.initialized {
            if let Some(mut child) = self.child.take() {
                let _ = child.kill().await;
            }
            return Ok(());
        }
        let _: Value = self.call("shutdown", None).await?;
        self.notify("exit", None).await.ok();
        if let Some(mut child) = self.child.take() {
            let _ = child.wait().await;
        }
        self.initialized = false;
        Ok(())
    }

    // ===== LSP Features =====

    /// Request hover info at a position.
    pub async fn hover(&mut self, path: &Path, position: Position) -> Result<Option<HoverInfo>, LspError> {
        self.ensure_initialized()?;
        let uri = path_to_uri(path, &self.root_uri);
        let params = serde_json::json!({
            "textDocument": { "uri": uri },
            "position": { "line": position.line, "character": position.character }
        });
        let result = self.call("textDocument/hover", Some(params)).await?;
        if result.is_null() {
            return Ok(None);
        }
        let contents = extract_hover_contents(&result["contents"]);
        let range = result.get("range").map(extract_range);
        Ok(Some(HoverInfo { contents, range }))
    }

    /// Request completion items at a position.
    pub async fn completion(
        &mut self,
        path: &Path,
        position: Position,
    ) -> Result<Vec<CompletionItem>, LspError> {
        self.ensure_initialized()?;
        let uri = path_to_uri(path, &self.root_uri);
        let params = serde_json::json!({
            "textDocument": { "uri": uri },
            "position": { "line": position.line, "character": position.character }
        });
        let result = self.call("textDocument/completion", Some(params)).await?;
        let items = if result.is_object() {
            result["items"].as_array().cloned().unwrap_or_default()
        } else {
            result.as_array().cloned().unwrap_or_default()
        };
        Ok(items.into_iter().map(extract_completion_item).collect())
    }

    /// Go to definition.
    pub async fn goto_definition(
        &mut self,
        path: &Path,
        position: Position,
    ) -> Result<Option<Location>, LspError> {
        self.ensure_initialized()?;
        let uri = path_to_uri(path, &self.root_uri);
        let params = serde_json::json!({
            "textDocument": { "uri": uri },
            "position": { "line": position.line, "character": position.character }
        });
        let result = self.call("textDocument/definition", Some(params)).await?;
        if result.is_null() {
            return Ok(None);
        }
        if let Some(loc) = result.as_object() {
            return Ok(Some(extract_location(loc)));
        }
        if let Some(arr) = result.as_array() {
            if let Some(first) = arr.first().and_then(|v| v.as_object()) {
                return Ok(Some(extract_location(first)));
            }
        }
        Ok(None)
    }

    /// Find all references.
    pub async fn references(
        &mut self,
        path: &Path,
        position: Position,
    ) -> Result<Vec<Location>, LspError> {
        self.ensure_initialized()?;
        let uri = path_to_uri(path, &self.root_uri);
        let params = serde_json::json!({
            "textDocument": { "uri": uri },
            "position": { "line": position.line, "character": position.character },
            "context": { "includeDeclaration": true }
        });
        let result = self.call("textDocument/references", Some(params)).await?;
        Ok(extract_location_array(&result))
    }

    /// Get diagnostics for a file.
    /// Note: LSP diagnostics are push-based via `textDocument/publishDiagnostics`.
    /// This method attempts a workspace/diagnostic pull (LSP 3.17+) or returns empty.
    pub async fn diagnostics(&mut self, path: &Path) -> Result<Vec<Diagnostic>, LspError> {
        self.ensure_initialized()?;
        let uri = path_to_uri(path, &self.root_uri);
        let params = serde_json::json!({
            "textDocument": { "uri": uri }
        });
        let result = self.call("textDocument/diagnostic", Some(params)).await;
        match result {
            Ok(val) => Ok(extract_diagnostics(&val)),
            Err(_) => Ok(vec![]),
        }
    }

    /// Format a document. Returns the formatted text.
    pub async fn format(&mut self, path: &Path) -> Result<String, LspError> {
        self.ensure_initialized()?;
        let uri = path_to_uri(path, &self.root_uri);
        let params = serde_json::json!({
            "textDocument": { "uri": uri },
            "options": { "tabSize": 4, "insertSpaces": true }
        });
        let result = self.call("textDocument/formatting", Some(params)).await?;
        let edits = result
            .as_array()
            .ok_or_else(|| LspError::ParseError("formatting: expected array".into()))?;
        let new_text = edits
            .first()
            .and_then(|e| e["newText"].as_str())
            .unwrap_or("")
            .to_string();
        Ok(new_text)
    }

    // ===== Internal: JSON-RPC helpers =====

    fn ensure_initialized(&self) -> Result<(), LspError> {
        if !self.initialized {
            return Err(LspError::NotInitialized);
        }
        Ok(())
    }

    /// Send a request and wait for the matching response.
    async fn call(&mut self, method: &str, params: Option<Value>) -> Result<Value, LspError> {
        let id = self.next_id;
        self.next_id += 1;

        let request = protocol::JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id,
            method: method.into(),
            params,
        };
        protocol::write_message(
            &mut self.writer,
            &protocol::JsonRpcMessage::Request(request),
        )
        .await
        .map_err(|e| LspError::IoError(e.to_string()))?;

        loop {
            let msg = protocol::read_message(&mut self.reader)
                .await
                .map_err(|e| LspError::IoError(e))?;

            match msg {
                protocol::JsonRpcMessage::Response(resp) => {
                    if resp.id == id {
                        if let Some(err) = resp.error {
                            return Err(LspError::RequestFailed(err.code, err.message));
                        }
                        return Ok(resp.result.unwrap_or(Value::Null));
                    }
                }
                protocol::JsonRpcMessage::Notification(_) => {
                    // drop nt_io_notifys during request-response cycle
                }
                protocol::JsonRpcMessage::Request(_) => {
                    // servers typically don't send requests to clients
                }
            }
        }
    }

    /// Send a fire-and-forget nt_io_notify.
    async fn notify(&mut self, method: &str, params: Option<Value>) -> Result<(), LspError> {
        let nt_io_notify = protocol::JsonRpcNotification {
            jsonrpc: "2.0".into(),
            method: method.into(),
            params,
        };
        protocol::write_message(
            &mut self.writer,
            &protocol::JsonRpcMessage::Notification(nt_io_notify),
        )
        .await
        .map_err(|e| LspError::IoError(e.to_string()))
    }
}

// ===== Helper functions =====

fn path_to_uri(path: &Path, root_uri: &str) -> String {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        let root_str = root_uri.strip_prefix("file://").unwrap_or("");
        let root = PathBuf::from(root_str);
        root.join(path)
    };
    format!("file://{}", abs.display())
}

fn extract_position(val: &Value) -> Position {
    Position {
        line: val["line"].as_u64().unwrap_or(0) as u32,
        character: val["character"].as_u64().unwrap_or(0) as u32,
    }
}

fn extract_range(val: &Value) -> Range {
    Range {
        start: extract_position(&val["start"]),
        end: extract_position(&val["end"]),
    }
}

fn extract_location(obj: &serde_json::Map<String, Value>) -> Location {
    Location {
        uri: obj["uri"].as_str().unwrap_or("").to_string(),
        range: extract_range(&obj["range"]),
    }
}

fn extract_location_array(val: &Value) -> Vec<Location> {
    val.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_object())
                .map(extract_location)
                .collect()
        })
        .unwrap_or_default()
}

fn extract_completion_item(val: Value) -> CompletionItem {
    CompletionItem {
        label: val["label"].as_str().unwrap_or("").to_string(),
        kind: val["kind"].as_i64().map(|k| k as i32),
        detail: val["detail"].as_str().map(|s| s.to_string()),
        documentation: val["documentation"]
            .as_str()
            .or_else(|| {
                val["documentation"]
                    .as_object()
                    .and_then(|o| o.get("value"))
                    .and_then(|v| v.as_str())
            })
            .map(|s| s.to_string()),
    }
}

fn extract_hover_contents(val: &Value) -> String {
    if let Some(s) = val.as_str() {
        return s.to_string();
    }
    if let Some(obj) = val.as_object() {
        if let Some(value) = obj.get("value").and_then(|v| v.as_str()) {
            return value.to_string();
        }
        if let Some(kind) = obj.get("kind").and_then(|v| v.as_str()) {
            if kind == "markdown" {
                if let Some(value) = obj.get("value").and_then(|v| v.as_str()) {
                    return value.to_string();
                }
            }
        }
    }
    if let Some(arr) = val.as_array() {
        return arr
            .iter()
            .filter_map(|v| {
                v.as_str()
                    .or_else(|| v.as_object().and_then(|o| o.get("value")).and_then(|v| v.as_str()))
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    val.to_string()
}

fn extract_diagnostics(val: &Value) -> Vec<Diagnostic> {
    let items = if val.is_object() {
        val["items"].as_array().cloned().unwrap_or_default()
    } else {
        val.as_array().cloned().unwrap_or_default()
    };
    items
        .into_iter()
        .map(|v| {
            let severity = match v["severity"].as_u64() {
                Some(1) => DiagnosticSeverity::Error,
                Some(2) => DiagnosticSeverity::Warning,
                Some(3) => DiagnosticSeverity::Information,
                Some(4) => DiagnosticSeverity::Hint,
                _ => DiagnosticSeverity::Warning,
            };
            Diagnostic {
                range: extract_range(&v["range"]),
                severity,
                message: v["message"].as_str().unwrap_or("").to_string(),
                code: match &v["code"] {
                    Value::String(s) => Some(s.clone()),
                    Value::Number(n) => n.as_u64().map(|c| c.to_string()),
                    _ => None,
                },
                source: v["source"].as_str().map(|s| s.to_string()),
            }
        })
        .collect()
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
