//! Tauri commands for consciousness tool primitives

use neotrix::neotrix::nt_core_error::NeoTrixError;
use neotrix::neotrix::nt_world_search::WebSearchEngine;
use reqwest::blocking;
use std::fs;
use std::process::Command;
use glob::glob;

#[derive(Clone, serde::Serialize)]
#[allow(dead_code)]
enum ToolOrigin { User, System, Agent }

#[derive(Clone, serde::Serialize)]
struct ToolEvent {
    tool: String, success: bool, duration_ms: u64, origin: ToolOrigin, summary: String,
}

fn emit_event(event: ToolEvent) {
    let _ = event;
}

#[derive(serde::Serialize)]
pub struct ToolResponse {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(serde::Serialize)]
pub struct SearchResultItem {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

fn dispatch_tool(tool: &str, args: &serde_json::Value) -> ToolResponse {
    let start = std::time::Instant::now();
    match tool {
        "websearch" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(8) as usize;
            let engine = WebSearchEngine::default();
            match engine.search(query, count) {
                Ok(results) => {
                    let items: Vec<String> = results.into_iter().map(|r| format!("{}: {}", r.title, r.url)).collect();
                    ToolResponse {
                        success: true,
                        output: items.join("\n"),
                        error: None,
                        duration_ms: start.elapsed().as_millis() as u64,
                    }
                }
                Err(e) => ToolResponse {
                    success: false,
                    output: String::new(),
                    error: Some(e.to_string()),
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
        }
        "webfetch" | "fetch" => {
            let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
            // SSRF 防护：仅允许 http/https scheme
            let scheme_ok = url.starts_with("http://") || url.starts_with("https://");
            if !scheme_ok {
                return ToolResponse {
                    success: false,
                    output: String::new(),
                    error: Some(format!("URL scheme not allowed (http/https only): {}", url)),
                    duration_ms: start.elapsed().as_millis() as u64,
                };
            }
            match blocking::get(url) {
                Ok(response) => {
                    match response.text() {
                        Ok(body) => ToolResponse {
                            success: true,
                            output: body.chars().take(10000).collect(),
                            error: None,
                            duration_ms: start.elapsed().as_millis() as u64,
                        },
                        Err(e) => ToolResponse {
                            success: false,
                            output: String::new(),
                            error: Some(e.to_string()),
                            duration_ms: start.elapsed().as_millis() as u64,
                        }
                    }
                }
                Err(e) => ToolResponse {
                    success: false,
                    output: String::new(),
                    error: Some(e.to_string()),
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
        }
        "read" | "read_file" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            // 路径校验：仅允许主目录内文件（复用 project_cmds 的 resolve_safe_path）
            let safe = match super::project_cmds::resolve_safe_path(path) {
                Ok(p) => p,
                Err(e) => {
                    return ToolResponse {
                        success: false,
                        output: String::new(),
                        error: Some(e.to_string()),
                        duration_ms: start.elapsed().as_millis() as u64,
                    };
                }
            };
            match std::fs::read_to_string(&safe) {
                Ok(content) => ToolResponse {
                    success: true,
                    output: content.chars().take(10000).collect(),
                    error: None,
                    duration_ms: start.elapsed().as_millis() as u64,
                },
                Err(e) => ToolResponse {
                    success: false,
                    output: String::new(),
                    error: Some(e.to_string()),
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
        }
        "write" | "write_file" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
            // 路径校验：仅允许主目录内文件
            let safe = match super::project_cmds::resolve_safe_path(path) {
                Ok(p) => p,
                Err(e) => {
                    return ToolResponse {
                        success: false,
                        output: String::new(),
                        error: Some(e.to_string()),
                        duration_ms: start.elapsed().as_millis() as u64,
                    };
                }
            };
            match fs::write(&safe, content) {
                Ok(()) => ToolResponse {
                    success: true,
                    output: format!("Written to {}", safe.display()),
                    error: None,
                    duration_ms: start.elapsed().as_millis() as u64,
                },
                Err(e) => ToolResponse {
                    success: false,
                    output: String::new(),
                    error: Some(e.to_string()),
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
        }
        "bash" | "shell" => {
            let command = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
            // 命令白名单校验：复用 remote_cmds 的 validate_remote_command
            if let Err(e) = super::remote_cmds::validate_remote_command(command) {
                return ToolResponse {
                    success: false,
                    output: String::new(),
                    error: Some(e),
                    duration_ms: start.elapsed().as_millis() as u64,
                };
            }
            match Command::new("sh").arg("-c").arg(command).output() {
                Ok(output) => ToolResponse {
                    success: output.status.success(),
                    output: String::from_utf8_lossy(&output.stdout).to_string(),
                    error: if output.status.success() { None } else { Some(String::from_utf8_lossy(&output.stderr).to_string()) },
                    duration_ms: start.elapsed().as_millis() as u64,
                },
                Err(e) => ToolResponse {
                    success: false,
                    output: String::new(),
                    error: Some(e.to_string()),
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
        }
        "grep" => {
            let pattern = args.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
            let output = Command::new("grep").args(["-r", pattern, path]).output();
            match output {
                Ok(out) => ToolResponse {
                    success: out.status.success(),
                    output: String::from_utf8_lossy(&out.stdout).to_string(),
                    error: if out.status.success() { None } else { Some(String::from_utf8_lossy(&out.stderr).to_string()) },
                    duration_ms: start.elapsed().as_millis() as u64,
                },
                Err(e) => ToolResponse {
                    success: false,
                    output: String::new(),
                    error: Some(e.to_string()),
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
        }
        "glob" => {
            let pattern = args.get("pattern").and_then(|v| v.as_str()).unwrap_or("*");
            let base = args.get("base").and_then(|v| v.as_str()).unwrap_or(".");
            // base 路径校验：仅允许主目录内
            let safe_base = match super::project_cmds::resolve_safe_path(base) {
                Ok(p) => p,
                Err(e) => {
                    return ToolResponse {
                        success: false,
                        output: String::new(),
                        error: Some(e.to_string()),
                        duration_ms: start.elapsed().as_millis() as u64,
                    };
                }
            };
            let files: Vec<String> = match glob(&format!("{}/{}", safe_base.display(), pattern)) {
                Ok(entries) => entries.filter_map(|e| e.ok()).map(|p| p.display().to_string()).take(100).collect(),
                Err(_) => vec![],
            };
            ToolResponse {
                success: true,
                output: files.join("\n"),
                error: None,
                duration_ms: start.elapsed().as_millis() as u64,
            }
        }
        _ => ToolResponse {
            success: false,
            output: String::new(),
            error: Some(format!("tool '{}' not implemented in desktop mode", tool)),
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}

#[tauri::command]
pub fn tool_execute(tool: String, args: serde_json::Value) -> ToolResponse {
    let result = dispatch_tool(&tool, &args);
    let summary = if result.success {
        let preview: String = result.output.chars().take(120).collect();
        if result.output.len() > 120 {
            format!("{}…", preview)
        } else {
            preview
        }
    } else {
        result.error.clone().unwrap_or_default()
    };
    emit_event(ToolEvent {
        tool: tool.clone(),
        success: result.success,
        duration_ms: result.duration_ms,
        origin: ToolOrigin::User,
        summary,
    });
    ToolResponse {
        success: result.success,
        output: result.output,
        error: result.error.clone(),
        duration_ms: result.duration_ms,
    }
}

#[tauri::command]
pub fn tool_search(query: String, count: Option<usize>) -> Result<Vec<SearchResultItem>, NeoTrixError> {
    let engine = WebSearchEngine::default();
    let results = engine.search(&query, count.unwrap_or(8)).map_err(|e| NeoTrixError::Network(e.to_string()))?;
    let items: Vec<SearchResultItem> = results.into_iter().map(|r| SearchResultItem {
        title: r.title,
        url: r.url,
        snippet: r.snippet,
    }).collect();
    let summary = format!("search: {} results for \"{}\"", items.len(), &query);
    emit_event(ToolEvent {
        tool: "websearch".into(),
        success: true,
        duration_ms: 0,
        origin: ToolOrigin::User,
        summary,
    });
    Ok(items)
}
