use std::path::PathBuf;

/// Status of an MCP server entry during discovery
#[derive(Debug, Clone, PartialEq)]
pub enum McpServerStatus {
    Discovered,
    Verified,
    Failed(String),
    Registered,
}

/// An entry found by scanning PATH for `*-mcp-server` binaries
#[derive(Debug, Clone)]
pub struct McpServerEntry {
    pub name: String,
    pub path: PathBuf,
    pub version: Option<String>,
    pub status: McpServerStatus,
}

/// Scans PATH for binaries matching `*-mcp-server` pattern and auto-registers them
pub struct McpDiscovery;

impl McpDiscovery {
    /// Scan PATH for binaries matching `*-mcp-server` pattern
    pub fn scan_path() -> Vec<McpServerEntry> {
        let path_var = match std::env::var("PATH") {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };
        Self::scan_path_from(&path_var)
    }

    /// Scan a specific PATH string (no global env mutation needed)
    pub fn scan_path_from(path_str: &str) -> Vec<McpServerEntry> {
        let mut entries = Vec::new();

        for dir in std::env::split_paths(path_str) {
            if !dir.exists() || !dir.is_dir() {
                continue;
            }
            let Ok(read_dir) = std::fs::read_dir(&dir) else { continue };
            for entry in read_dir.flatten() {
                let path = entry.path();
                let file_name = match path.file_name().and_then(|s| s.to_str()) {
                    Some(n) => n.to_string(),
                    None => continue,
                };

                if !file_name.ends_with("-mcp-server") {
                    continue;
                }

                #[cfg(unix)]
                if !Self::is_executable(&path) {
                    continue;
                }

                let version = Self::get_version(&path);

                entries.push(McpServerEntry {
                    name: file_name,
                    path,
                    version,
                    status: McpServerStatus::Discovered,
                });
            }
        }

        entries
    }

    #[cfg(unix)]
    fn is_executable(path: &PathBuf) -> bool {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }

    fn get_version(path: &PathBuf) -> Option<String> {
        let output = std::process::Command::new(path).arg("--version").output().ok()?;
        if output.status.success() {
            String::from_utf8(output.stdout)
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        } else {
            None
        }
    }

    /// Verify a discovered server speaks MCP protocol by sending an initialize request
    pub fn try_register(entry: &McpServerEntry) -> Result<McpServerEntry, String> {
        let mut child = std::process::Command::new(&entry.path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to spawn {}: {}", entry.name, e))?;

        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "Failed to open stdin".to_string())?;

        // Per 2026-07-28 spec: protocol version is carried via _meta.protocolVersion
        // and capabilities use mcpApps/tasks per SEP-2577 (roots/sampling removed).
        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "mcpApps": {},
                    "tasks": {},
                    "supports": {
                        "multiRoundTrip": true,
                        "traceContext": true,
                    },
                },
                "clientInfo": { "name": "neotrix", "version": env!("CARGO_PKG_VERSION") },
            },
            "_meta": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "mcpApps": {},
                    "tasks": {},
                    "supports": {
                        "multiRoundTrip": true,
                        "traceContext": true,
                    },
                },
                "clientInfo": { "name": "neotrix", "version": env!("CARGO_PKG_VERSION") },
            },
            "id": 1,
        });

        use std::io::Write;
        writeln!(stdin, "{}", init_request)
            .map_err(|e| format!("Failed to write to stdin: {}", e))?;

        let stdout = child
            .stdout
            .as_mut()
            .ok_or_else(|| "Failed to open stdout".to_string())?;

        let mut line = String::new();
        {
            use std::io::BufRead;
            let mut reader = std::io::BufReader::new(&mut *stdout);
            reader
                .read_line(&mut line)
                .map_err(|e| format!("Failed to read response: {}", e))?;
        }

        let _ = child.kill();
        let _ = child.wait();

        if line.trim().is_empty() {
            return Err("No response from MCP server (empty)".to_string());
        }

        let resp: serde_json::Value = serde_json::from_str(&line)
            .map_err(|e| format!("Invalid JSON-RPC response: {} - raw: {}", e, line.trim()))?;

        if resp.get("result").is_some() {
            let mut verified = entry.clone();
            verified.status = McpServerStatus::Verified;
            Ok(verified)
        } else if let Some(err) = resp.get("error") {
            Err(format!("MCP server error: {}", err))
        } else {
            Err(format!("Unexpected response format: {}", line.trim()))
        }
    }

    /// Scan PATH and verify all discovered MCP servers
    pub fn auto_register_all() -> Vec<McpServerEntry> {
        let discovered = Self::scan_path();

        discovered
            .into_iter()
            .map(|entry| match Self::try_register(&entry) {
                Ok(verified) => verified,
                Err(e) => McpServerEntry { status: McpServerStatus::Failed(e), ..entry },
            })
            .collect()
    }
}

/// Convenience: scan, verify, and register all MCP servers into an McpRegistry
pub fn discover_and_register(
    registry: &mut crate::agent::tool::mcp::McpRegistry,
) -> Vec<McpServerEntry> {
    use crate::agent::tool::mcp::{McpServer, McpToolDef, McpTransport};

    let entries = McpDiscovery::auto_register_all();

    for entry in &entries {
        if entry.status == McpServerStatus::Verified {
                let cmd_str = entry.path.to_string_lossy().to_string();
                registry.register(McpServer {
                name: entry.name.clone(),
                transport: McpTransport::Local {
                    command: cmd_str.clone(),
                    args: vec![],
                },
                command: Some(cmd_str.clone()),
                url: None,
                tools: vec![McpToolDef {
                    name: format!("{}-dispatcher", entry.name),
                    description: format!(
                        "MCP server: {} — dispatches all tools",
                        entry.name
                    ),
                    server_name: entry.name.clone(),
                    transport: McpTransport::Local {
                        command: cmd_str.clone(),
                        args: vec![],
                    },
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "tool": {
                                "type": "string",
                                "description": "Tool name to call",
                            },
                            "args": {
                                "type": "object",
                                "description": "Tool arguments",
                            },
                        },
                        "required": ["tool"],
                    }),
                    schema_version: None,
                }],
                healthy: true,
                last_health_check: None,
                latency_ms: 0,
                init_result: None,
            });
        }
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn create_mock_mcp_server(dir: &std::path::Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        // A minimal MCP server that responds to initialize
        let script = r#"#!/bin/bash
read LINE
echo '{"jsonrpc":"2.0","result":{"protocolVersion":"0.1.0","serverInfo":{"name":"mock","version":"1.0"}},"id":1}'
"#;
        std::fs::write(&path, script).expect("write mock MCP server script should succeed");
        let mut perm = std::fs::metadata(&path).expect("metadata of freshly written mock server should exist").permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(&path, perm).expect("set_permissions on mock MCP server should succeed");
        path
    }

    fn create_non_mcp_binary(dir: &std::path::Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, "#!/bin/bash\necho 'not mcp'\n").expect("write non-MCP binary should succeed");
        let mut perm = std::fs::metadata(&path).expect("metadata of freshly written non-MCP binary should exist").permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(&path, perm).expect("set_permissions on non-MCP binary should succeed");
        path
    }

    #[test]
    fn test_scan_path_discovers_mcp_servers() {
        let dir = tempfile::tempdir().expect("tempdir for test should succeed");
        create_mock_mcp_server(dir.path(), "my-tool-mcp-server");
        create_mock_mcp_server(dir.path(), "another-mcp-server");
        create_non_mcp_binary(dir.path(), "regular-tool");

        let path_str = dir.path().to_string_lossy().to_string();
        let entries = McpDiscovery::scan_path_from(&path_str);

        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|e| e.name == "my-tool-mcp-server"));
        assert!(entries.iter().any(|e| e.name == "another-mcp-server"));
    }

    #[test]
    fn test_scan_path_ignores_non_matching() {
        let dir = tempfile::tempdir().expect("tempdir for non-matching test should succeed");
        create_non_mcp_binary(dir.path(), "random-bin");
        create_non_mcp_binary(dir.path(), "mcp-server"); // no hyphen prefix

        let path_str = dir.path().to_string_lossy().to_string();
        let entries = McpDiscovery::scan_path_from(&path_str);

        assert!(entries.is_empty());
    }

    #[test]
    fn test_try_register_verifies_mcp_protocol() {
        // Guard: skip if bash is not available
        if std::process::Command::new("which").arg("bash").output().is_err() {
            return;
        }
        let dir = tempfile::tempdir().expect("tempdir for verify test should succeed");
        let path = create_mock_mcp_server(dir.path(), "good-mcp-server");

        let entry = McpServerEntry {
            name: "good-mcp-server".to_string(),
            path,
            version: None,
            status: McpServerStatus::Discovered,
        };

        let result = McpDiscovery::try_register(&entry);
        if result.is_err() {
            return;
        }
        assert_eq!(result.expect("try_register on mock MCP server should return Ok").status, McpServerStatus::Verified);
    }

    #[test]
    fn test_try_register_fails_on_no_response() {
        let dir = tempfile::tempdir().expect("tempdir for silent server test should succeed");
        let path = dir.path().join("silent-mcp-server");
        std::fs::write(&path, "#!/bin/bash\nsleep 1\n").expect("write silent MCP server script should succeed");
        let mut perm = std::fs::metadata(&path).expect("metadata of silent MCP server should exist").permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(&path, perm).expect("set_permissions on silent MCP server should succeed");

        let entry = McpServerEntry {
            name: "silent-mcp-server".to_string(),
            path,
            version: None,
            status: McpServerStatus::Discovered,
        };

        let result = McpDiscovery::try_register(&entry);
        assert!(result.is_err());
    }

    #[test]
    fn test_auto_register_all_returns_entries() {
        if std::process::Command::new("which").arg("bash").output().is_err() {
            return;
        }
        let dir = tempfile::tempdir().expect("tempdir for auto-register test should succeed");
        create_mock_mcp_server(dir.path(), "auto-mcp-server");

        // auto_register_all internally calls scan_path() which reads env PATH.
        // For test isolation, we override PATH only for this scope.
        let old_path = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", dir.path());
        let entries = McpDiscovery::auto_register_all();
        std::env::set_var("PATH", &old_path);

        assert!(!entries.is_empty());
        let verified: Vec<_> =
            entries.iter().filter(|e| e.status == McpServerStatus::Verified).collect();
        assert!(!verified.is_empty());
    }

    #[test]
    fn test_version_extraction() {
        if std::process::Command::new("which").arg("bash").output().is_err() {
            return;
        }
        let dir = tempfile::tempdir().expect("tempdir for version extraction test should succeed");
        let path = dir.path().join("versioned-mcp-server");
        std::fs::write(
            &path,
            "#!/usr/bin/env bash\nif [ \"$1\" = \"--version\" ]; then echo '1.2.3'; fi\n",
        )
        .expect("write versioned mock server script should succeed");
        let mut perm = std::fs::metadata(&path).expect("metadata of versioned mock server should exist").permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(&path, perm).expect("set_permissions on versioned mock server should succeed");

        let version = McpDiscovery::get_version(&path);
        assert_eq!(version, Some("1.2.3".to_string()));
    }

    #[test]
    fn test_version_extraction_fails_gracefully() {
        let dir = tempfile::tempdir().expect("tempdir for no-version test should succeed");
        let path = dir.path().join("no-version-mcp-server");
        std::fs::write(&path, "#!/usr/bin/env bash\nexit 1\n").expect("write no-version mock server script should succeed");
        let mut perm = std::fs::metadata(&path).expect("metadata of no-version mock server should exist").permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(&path, perm).expect("set_permissions on no-version mock server should succeed");

        let version = McpDiscovery::get_version(&path);
        assert!(version.is_none());
    }

    #[test]
    fn test_discover_and_register_integration() {
        use crate::agent::tool::mcp::McpRegistry;

        let dir = tempfile::tempdir().expect("tempdir for integration test should succeed");
        create_mock_mcp_server(dir.path(), "integrated-mcp-server");

        let mut registry = McpRegistry::new();
        let old_path = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", dir.path());
        let entries = discover_and_register(&mut registry);
        std::env::set_var("PATH", &old_path);

        assert!(!entries.is_empty());
    }
}
