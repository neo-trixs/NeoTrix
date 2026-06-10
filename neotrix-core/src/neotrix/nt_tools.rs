//! 工具箱 — consciousness tool primitives
//!
//! 供 SEAL pipeline stage 内部调用的工具函数:
//!   read / write / edit / bash / glob / grep / webfetch / websearch

use serde::{Deserialize, Serialize};
use std::path::Path;

/// A tool invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    pub args: serde_json::Value,
}

/// Result of a tool invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub duration_ms: u64,
}

impl ToolResult {
    pub fn ok(output: impl Into<String>, duration_ms: u64) -> Self {
        Self { success: true, output: output.into(), error: None, duration_ms }
    }
    pub fn err(error: impl Into<String>, duration_ms: u64) -> Self {
        Self { success: false, output: String::new(), error: Some(error.into()), duration_ms }
    }
}

/// Dispatch a tool call to the appropriate handler.
pub fn dispatch_tool(tool: &str, args: &serde_json::Value) -> ToolResult {
    let start = std::time::Instant::now();
    match tool {
        "read" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            tool_read(path)
        }
        "write" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
            tool_write(path, content)
        }
        "edit" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let old = args.get("old_string").and_then(|v| v.as_str()).unwrap_or("");
            let new = args.get("new_string").and_then(|v| v.as_str()).unwrap_or("");
            tool_edit(path, old, new)
        }
        "bash" => {
            let cmd = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
            tool_bash(cmd)
        }
        "glob" => {
            let pattern = args.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
            tool_glob(pattern)
        }
        "grep" => {
            let pattern = args.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
            tool_grep(pattern, path)
        }
        "webfetch" => {
            let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
            tool_webfetch(url)
        }
        "websearch" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            tool_websearch(query)
        }
        _ => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResult::err(format!("Unknown tool: {tool}"), elapsed)
        }
    }
}

// ─── individual tools ───────────────────────────────────────────────

pub fn tool_read(path: &str) -> ToolResult {
    let start = std::time::Instant::now();
    if path.is_empty() {
        return ToolResult::err("path is required", 0);
    }
    match std::fs::read_to_string(path) {
        Ok(content) => ToolResult::ok(content, start.elapsed().as_millis() as u64),
        Err(e) => ToolResult::err(format!("read {path}: {e}"), start.elapsed().as_millis() as u64),
    }
}

pub fn tool_write(path: &str, content: &str) -> ToolResult {
    let start = std::time::Instant::now();
    if path.is_empty() {
        return ToolResult::err("path is required", 0);
    }
    if let Some(parent) = Path::new(path).parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return ToolResult::err(format!("create dir {parent:?}: {e}"), start.elapsed().as_millis() as u64);
            }
        }
    }
    match std::fs::write(path, content) {
        Ok(()) => ToolResult::ok(format!("Written {} bytes to {path}", content.len()), start.elapsed().as_millis() as u64),
        Err(e) => ToolResult::err(format!("write {path}: {e}"), start.elapsed().as_millis() as u64),
    }
}

pub fn tool_edit(path: &str, old_string: &str, new_string: &str) -> ToolResult {
    let start = std::time::Instant::now();
    if path.is_empty() || old_string.is_empty() {
        return ToolResult::err("path and old_string are required", 0);
    }
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return ToolResult::err(format!("read {path}: {e}"), start.elapsed().as_millis() as u64),
    };
    if !content.contains(old_string) {
        return ToolResult::err(format!("old_string not found in {path}"), start.elapsed().as_millis() as u64);
    }
    let new_content = content.replace(old_string, new_string);
    match std::fs::write(path, &new_content) {
        Ok(()) => ToolResult::ok(
            format!("Edited {path}: {} → {} chars", content.len(), new_content.len()),
            start.elapsed().as_millis() as u64,
        ),
        Err(e) => ToolResult::err(format!("write {path}: {e}"), start.elapsed().as_millis() as u64),
    }
}

pub fn tool_bash(command: &str) -> ToolResult {
    let start = std::time::Instant::now();
    if command.is_empty() {
        return ToolResult::err("command is required", 0);
    }

    match crate::cli::execute_guarded(command) {
        Ok(output) => {
            if output.contains("(exit code: ") {
                let exit_code = output.rsplit_once("(exit code: ")
                    .and_then(|(_, rest)| rest.trim_end_matches(')').parse::<i32>().ok())
                    .unwrap_or(-1);
                let clean = output.rsplit_once("\n(exit code: ")
                    .map(|(before, _)| before.to_string())
                    .unwrap_or_else(|| output.clone());
                if exit_code == 0 {
                    ToolResult::ok(clean, start.elapsed().as_millis() as u64)
                } else {
                    ToolResult::err(
                        format!("exit={exit_code}: {clean}"),
                        start.elapsed().as_millis() as u64,
                    )
                }
            } else {
                ToolResult::ok(output, start.elapsed().as_millis() as u64)
            }
        }
        Err(e) => ToolResult::err(e, start.elapsed().as_millis() as u64),
    }
}

pub fn tool_glob(pattern: &str) -> ToolResult {
    let start = std::time::Instant::now();
    if pattern.is_empty() {
        return ToolResult::err("pattern is required", 0);
    }
    let entries: Vec<String> = match glob::glob(pattern) {
        Ok(paths) => paths.filter_map(|p| p.ok().map(|p| p.display().to_string())).collect(),
        Err(e) => return ToolResult::err(format!("glob pattern: {e}"), start.elapsed().as_millis() as u64),
    };
    ToolResult::ok(entries.join("\n"), start.elapsed().as_millis() as u64)
}

pub fn tool_grep(pattern: &str, path: &str) -> ToolResult {
    let start = std::time::Instant::now();
    if pattern.is_empty() {
        return ToolResult::err("pattern is required", 0);
    }
    let re = match regex::Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => return ToolResult::err(format!("invalid regex: {e}"), start.elapsed().as_millis() as u64),
    };
    let search_path = Path::new(path);
    let mut results: Vec<String> = Vec::new();
    if search_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(search_path) {
            for (i, line) in content.lines().enumerate() {
                if re.is_match(line) {
                    results.push(format!("{}:{}:{}", search_path.display(), i + 1, line));
                }
            }
        }
    } else if search_path.is_dir() {
        for entry in walkdir::WalkDir::new(search_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path().to_path_buf();
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for (i, line) in content.lines().enumerate() {
                        if re.is_match(line) {
                            results.push(format!("{}:{}:{}", path.display(), i + 1, line));
                        }
                    }
                }
            }
        }
    }
    if results.is_empty() {
        ToolResult::ok("(no matches)", start.elapsed().as_millis() as u64)
    } else {
        ToolResult::ok(results.join("\n"), start.elapsed().as_millis() as u64)
    }
}

pub fn tool_webfetch(url: &str) -> ToolResult {
    let start = std::time::Instant::now();
    if url.is_empty() {
        return ToolResult::err("url is required", 0);
    }
    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) NeoTrix/0.1")
        .build()
    {
        Ok(c) => c,
        Err(e) => return ToolResult::err(format!("http client: {e}"), start.elapsed().as_millis() as u64),
    };
    match client.get(url).send() {
        Ok(resp) => {
            let status = resp.status();
            match resp.text() {
                Ok(body) => {
                    let preview = format!("[{}] {} bytes\n\n{}", status.as_u16(), body.len(), &body.chars().take(8000).collect::<String>());
                    ToolResult::ok(preview, start.elapsed().as_millis() as u64)
                }
                Err(e) => ToolResult::err(format!("read body: {e}"), start.elapsed().as_millis() as u64),
            }
        }
        Err(e) => ToolResult::err(format!("fetch {url}: {e}"), start.elapsed().as_millis() as u64),
    }
}

pub fn tool_websearch(query: &str) -> ToolResult {
    let start = std::time::Instant::now();
    if query.is_empty() {
        return ToolResult::err("query is required", 0);
    }
    let engine = crate::neotrix::nt_world_search::WebSearchEngine::default();
    match engine.search(query, 8) {
        Ok(results) => {
            let mut out = format!("Search results for \"{}\":\n\n", query);
            for (i, r) in results.iter().enumerate() {
                out.push_str(&format!("{}. {}\n   URL: {}\n   {}\n\n", i + 1, r.title, r.url, r.snippet));
            }
            ToolResult::ok(out.trim().to_string(), start.elapsed().as_millis() as u64)
        }
        Err(e) => ToolResult::err(format!("search failed: {e}"), start.elapsed().as_millis() as u64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_read_nonexistent() {
        let r = tool_read("/tmp/neotrix_test_nonexistent_file_xyz");
        assert!(!r.success);
        assert!(r.error.as_deref().unwrap_or("").contains("No such file") || r.error.as_deref().unwrap_or("").contains("entity not found"));
    }

    #[test]
    fn test_tool_write_and_read() {
        let dir = std::env::temp_dir().join("neotrix_tool_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test_write.txt");
        let path_str = path.display().to_string();

        let w = tool_write(&path_str, "hello from neotrix tools");
        assert!(w.success, "write failed: {:?}", w.error);

        let r = tool_read(&path_str);
        assert!(r.success, "read failed: {:?}", r.error);
        assert_eq!(r.output, "hello from neotrix tools");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_tool_edit() {
        let dir = std::env::temp_dir().join("neotrix_tool_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test_edit.txt");
        let path_str = path.display().to_string();
        std::fs::write(&path, "foo bar baz").unwrap();

        let r = tool_edit(&path_str, "bar", "qux");
        assert!(r.success, "edit failed: {:?}", r.error);
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "foo qux baz");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_tool_edit_not_found() {
        let r = tool_edit("/tmp/neotrix_nonexistent_edit", "foo", "bar");
        assert!(!r.success);
    }

    #[test]
    fn test_tool_bash_echo() {
        let r = tool_bash("echo hello_tool_test_42");
        assert!(r.success, "bash failed: {:?}", r.error);
        assert!(r.output.contains("hello_tool_test_42"));
    }

    #[test]
    fn test_tool_bash_failure() {
        let r = tool_bash("exit 42");
        assert!(!r.success);
        // On some platforms the exit code may differ
    }

    #[test]
    fn test_tool_glob() {
        let dir = std::env::temp_dir().join("neotrix_tool_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("glob_test.txt");
        std::fs::write(&path, "data").unwrap();

        let r = tool_glob(&format!("{}/*.txt", dir.display()));
        assert!(r.success, "glob failed: {:?}", r.error);
        assert!(r.output.contains("glob_test.txt"));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_tool_grep_file() {
        let dir = std::env::temp_dir().join("neotrix_tool_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("grep_test.txt");
        std::fs::write(&path, "line one\nsearch_target\nline three").unwrap();

        let r = tool_grep("search_target", &path.display().to_string());
        assert!(r.success, "grep failed: {:?}", r.error);
        assert!(r.output.contains("search_target"));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_tool_grep_no_match() {
        let r = tool_grep("__NONEXISTENT_PATTERN_42__", "/tmp");
        assert!(r.success);
        assert_eq!(r.output, "(no matches)");
    }

    #[test]
    fn test_tool_empty_path() {
        let r = tool_read("");
        assert!(!r.success);
    }

    #[test]
    fn test_tool_empty_command() {
        let r = tool_bash("");
        assert!(!r.success);
    }

    #[test]
    fn test_dispatch_unknown() {
        let args = serde_json::json!({});
        let r = dispatch_tool("nonexistent_tool_xyz", &args);
        assert!(!r.success);
    }

    #[test]
    fn test_dispatch_read() {
        let args = serde_json::json!({"path": "/tmp/nonexistent_neotrix_dispatch_test"});
        let r = dispatch_tool("read", &args);
        assert!(!r.success);
    }

    #[test]
    fn test_dispatch_write_then_read() {
        let dir = std::env::temp_dir().join("neotrix_tool_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("dispatch_test.txt");
        let path_str = path.display().to_string();

        let args = serde_json::json!({"path": &path_str, "content": "dispatch works"});
        let w = dispatch_tool("write", &args);
        assert!(w.success, "dispatch write failed: {:?}", w.error);

        let args = serde_json::json!({"path": &path_str});
        let r = dispatch_tool("read", &args);
        assert!(r.success, "dispatch read failed: {:?}", r.error);
        assert_eq!(r.output, "dispatch works");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_dispatch_bash() {
        let args = serde_json::json!({"command": "echo dispatch_bash_test_42"});
        let r = dispatch_tool("bash", &args);
        assert!(r.success, "dispatch bash failed: {:?}", r.error);
        assert!(r.output.contains("dispatch_bash_test_42"));
    }

    #[test]
    fn test_dispatch_glob() {
        let args = serde_json::json!({"pattern": "/tmp/neotrix_*_nonexistent"});
        let r = dispatch_tool("glob", &args);
        assert!(r.success);
    }

    #[test]
    fn test_tool_write_creates_dirs() {
        let dir = std::env::temp_dir().join("neotrix_tool_test").join("nested").join("dirs");
        let path = dir.join("deep_test.txt");
        let path_str = path.display().to_string();
        let w = tool_write(&path_str, "deep write test");
        assert!(w.success, "deep write failed: {:?}", w.error);
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "deep write test");
        let _ = std::fs::remove_dir_all(dir.parent().unwrap());
    }
}
