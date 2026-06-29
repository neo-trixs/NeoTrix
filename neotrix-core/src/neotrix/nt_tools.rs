//! 工具箱 — consciousness tool primitives
//!
//! 供 SEAL pipeline stage 内部调用的工具函数:
//!   read / write / edit / bash / glob / grep / webfetch / websearch

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::core::nt_core_agent::UserAgentRotation;
use crate::core::nt_core_search::{self, format_results, search_file_content};
use crate::neotrix::nt_shield::agent_anomaly::{record_action, AgentActionType};

/// A tool invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    pub args: serde_json::Value,
}

use crate::core::nt_core_agent::tool_result::ToolResult;

/// Dispatch a tool call to the appropriate handler.
pub fn dispatch_tool(tool: &str, args: &serde_json::Value) -> ToolResult {
    let start = std::time::Instant::now();
    record_action(
        AgentActionType::ToolCall,
        tool,
        0,
        args.to_string().len() as u64,
        true,
    );
    let result = match tool {
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
            let old = args
                .get("old_string")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let new = args
                .get("new_string")
                .and_then(|v| v.as_str())
                .unwrap_or("");
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
        "image_gen" => {
            let prompt = args
                .get("prompt")
                .and_then(|v| v.as_str())
                .unwrap_or("abstract");
            let width = args
                .get("width")
                .and_then(|v| v.as_u64())
                .unwrap_or(512)
                .min(2048)
                .max(32) as u32;
            let height = args
                .get("height")
                .and_then(|v| v.as_u64())
                .unwrap_or(512)
                .min(2048)
                .max(32) as u32;
            let seed = args.get("seed").and_then(|v| v.as_u64());
            tool_image_gen(prompt, width, height, seed)
        }
        "minimax_t2i" => {
            let prompt = args
                .get("prompt")
                .and_then(|v| v.as_str())
                .unwrap_or("abstract art");
            let width = args
                .get("width")
                .and_then(|v| v.as_u64())
                .unwrap_or(1024)
                .min(2048)
                .max(64) as u32;
            let height = args
                .get("height")
                .and_then(|v| v.as_u64())
                .unwrap_or(1024)
                .min(2048)
                .max(64) as u32;
            let n = args
                .get("n")
                .and_then(|v| v.as_u64())
                .unwrap_or(1)
                .min(4)
                .max(1) as u32;
            let seed = args.get("seed").and_then(|v| v.as_u64());
            tool_minimax_t2i(prompt, width, height, n, seed)
        }
        _ => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResult::err(format!("Unknown tool: {tool}")).with_duration(elapsed)
        }
    };
    nt_core_search::record_tool_call(tool, result.duration_ms, result.success);
    if !result.success {
        crate::core::nt_core_search::record_failure_pattern(
            tool,
            "execution_error",
            &result.output,
        );
    }
    result
}

// ─── individual tools ───────────────────────────────────────────────

pub fn tool_read(path: &str) -> ToolResult {
    let start = std::time::Instant::now();
    if path.is_empty() {
        return ToolResult::err("path is required").with_duration(0);
    }
    let result = match std::fs::read_to_string(path) {
        Ok(content) => ToolResult::ok(content).with_duration(start.elapsed().as_millis() as u64),
        Err(e) => ToolResult::err(format!("read {path}: {e}"))
            .with_duration(start.elapsed().as_millis() as u64),
    };
    if result.success {
        nt_core_search::record_file_access(path);
    }
    result
}

pub fn tool_write(path: &str, content: &str) -> ToolResult {
    let start = std::time::Instant::now();
    if path.is_empty() {
        return ToolResult::err("path is required").with_duration(0);
    }
    if let Some(parent) = Path::new(path).parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return ToolResult::err(format!("create dir {parent:?}: {e}"))
                    .with_duration(start.elapsed().as_millis() as u64);
            }
        }
    }
    let p = std::path::Path::new(path);
    let tmp = p.with_extension("tmp");
    match std::fs::write(&tmp, content) {
        Ok(()) => {
            if let Err(e) = std::fs::rename(&tmp, p) {
                return ToolResult::err(format!("rename {path}: {e}"))
                    .with_duration(start.elapsed().as_millis() as u64);
            }
            ToolResult::ok(format!("Written {} bytes to {path}", content.len()))
                .with_duration(start.elapsed().as_millis() as u64)
        }
        Err(e) => ToolResult::err(format!("write {path}: {e}"))
            .with_duration(start.elapsed().as_millis() as u64),
    }
}

pub fn tool_edit(path: &str, old_string: &str, new_string: &str) -> ToolResult {
    let start = std::time::Instant::now();
    if path.is_empty() || old_string.is_empty() {
        return ToolResult::err("path and old_string are required").with_duration(0);
    }
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return ToolResult::err(format!("read {path}: {e}"))
                .with_duration(start.elapsed().as_millis() as u64)
        }
    };
    if !content.contains(old_string) {
        return ToolResult::err(format!("old_string not found in {path}"))
            .with_duration(start.elapsed().as_millis() as u64);
    }
    let new_content = content.replace(old_string, new_string);
    let p = std::path::Path::new(path);
    let tmp = p.with_extension("tmp");
    match std::fs::write(&tmp, &new_content) {
        Ok(()) => {
            if let Err(e) = std::fs::rename(&tmp, p) {
                return ToolResult::err(format!("rename {path}: {e}"))
                    .with_duration(start.elapsed().as_millis() as u64);
            }
            ToolResult::ok(format!(
                "Edited {path}: {} → {} chars",
                content.len(),
                new_content.len()
            ))
            .with_duration(start.elapsed().as_millis() as u64)
        }
        Err(e) => ToolResult::err(format!("write {path}: {e}"))
            .with_duration(start.elapsed().as_millis() as u64),
    }
}

pub fn tool_bash(command: &str) -> ToolResult {
    let start = std::time::Instant::now();
    if command.is_empty() {
        return ToolResult::err("command is required").with_duration(0);
    }

    match crate::cli::execute_guarded(command) {
        Ok(output) => {
            if output.contains("(exit code: ") {
                let exit_code = output
                    .rsplit_once("(exit code: ")
                    .and_then(|(_, rest): (&str, &str)| {
                        rest.trim_end_matches(')').parse::<i32>().ok()
                    })
                    .unwrap_or(-1);
                let clean = output
                    .rsplit_once("\n(exit code: ")
                    .map(|(before, _): (&str, &str)| before.to_string())
                    .unwrap_or_else(|| output.clone());
                if exit_code == 0 {
                    ToolResult::ok(clean).with_duration(start.elapsed().as_millis() as u64)
                } else {
                    ToolResult::err(format!("exit={exit_code}: {clean}"))
                        .with_duration(start.elapsed().as_millis() as u64)
                }
            } else {
                ToolResult::ok(output).with_duration(start.elapsed().as_millis() as u64)
            }
        }
        Err(e) => ToolResult::err(e).with_duration(start.elapsed().as_millis() as u64),
    }
}

pub fn tool_glob(pattern: &str) -> ToolResult {
    let start = std::time::Instant::now();
    if pattern.is_empty() {
        return ToolResult::err("pattern is required").with_duration(0);
    }
    let mut entries: Vec<String> = match glob::glob(pattern) {
        Ok(paths) => paths
            .filter_map(|p| p.ok().map(|p| p.display().to_string()))
            .collect(),
        Err(e) => {
            return ToolResult::err(format!("glob pattern: {e}"))
                .with_duration(start.elapsed().as_millis() as u64)
        }
    };
    nt_core_search::rank_paths(&mut entries);
    let elapsed = start.elapsed().as_millis() as u64;
    let summary = format!("{} files\n{}", entries.len(), entries.join("\n"));
    ToolResult::ok(summary).with_duration(elapsed)
}

pub fn tool_grep(pattern: &str, path: &str) -> ToolResult {
    let start = std::time::Instant::now();
    if pattern.is_empty() {
        return ToolResult::err("pattern is required").with_duration(0);
    }
    let search_path = Path::new(path);
    let mut all_results = Vec::new();
    if search_path.is_file() {
        let r = search_file_content(search_path, pattern);
        all_results.extend(r);
    } else if search_path.is_dir() {
        for entry in walkdir::WalkDir::new(search_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let r = search_file_content(entry.path(), pattern);
                all_results.extend(r);
            }
        }
    }
    if all_results.is_empty() {
        let pattern_lower = pattern.to_lowercase();
        if search_path.is_file() {
            if let Ok(content) = std::fs::read_to_string(search_path) {
                for (i, line) in content.lines().enumerate() {
                    let line_lower = line.to_lowercase();
                    let dist = strsim::levenshtein(&pattern_lower, &line_lower);
                    let max_dist = (pattern_lower.len() as f64 * 0.4).ceil() as usize;
                    if dist <= max_dist.max(2) {
                        all_results.push(nt_core_search::Match {
                            path: search_path.display().to_string(),
                            line: i + 1,
                            content: line.to_string(),
                            is_definition: false,
                        });
                        if all_results.len() >= 5 {
                            break;
                        }
                    }
                }
            }
        } else if search_path.is_dir() {
            for entry in walkdir::WalkDir::new(search_path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if !entry.file_type().is_file() {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    for (i, line) in content.lines().enumerate() {
                        let line_lower = line.to_lowercase();
                        let dist = strsim::levenshtein(&pattern_lower, &line_lower);
                        let max_dist = (pattern_lower.len() as f64 * 0.4).ceil() as usize;
                        if dist <= max_dist.max(2) {
                            all_results.push(nt_core_search::Match {
                                path: entry.path().display().to_string(),
                                line: i + 1,
                                content: line.to_string(),
                                is_definition: false,
                            });
                            if all_results.len() >= 5 {
                                break;
                            }
                        }
                    }
                }
                if all_results.len() >= 5 {
                    break;
                }
            }
        }
    }
    if all_results.is_empty() {
        ToolResult::ok("(no matches)").with_duration(start.elapsed().as_millis() as u64)
    } else {
        let out = format_results(&all_results);
        let elapsed = start.elapsed().as_millis() as u64;
        let summary = format!("{} matches in {}ms\n{}", all_results.len(), elapsed, out);
        ToolResult::ok(summary).with_duration(elapsed)
    }
}

pub fn tool_webfetch(url: &str) -> ToolResult {
    let start = std::time::Instant::now();
    if url.is_empty() {
        return ToolResult::err("url is required").with_duration(0);
    }
    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent(UserAgentRotation::default().next())
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return ToolResult::err(format!("http client: {e}"))
                .with_duration(start.elapsed().as_millis() as u64)
        }
    };
    match client.get(url).send() {
        Ok(resp) => {
            let status = resp.status();
            match resp.text() {
                Ok(body) => {
                    let preview = format!(
                        "[{}] {} bytes\n\n{}",
                        status.as_u16(),
                        body.len(),
                        &body.chars().take(8000).collect::<String>()
                    );
                    ToolResult::ok(preview).with_duration(start.elapsed().as_millis() as u64)
                }
                Err(e) => ToolResult::err(format!("read body: {e}"))
                    .with_duration(start.elapsed().as_millis() as u64),
            }
        }
        Err(e) => ToolResult::err(format!("fetch {url}: {e}"))
            .with_duration(start.elapsed().as_millis() as u64),
    }
}

pub fn tool_websearch(query: &str) -> ToolResult {
    let start = std::time::Instant::now();
    if query.is_empty() {
        return ToolResult::err("query is required").with_duration(0);
    }
    let engine = crate::neotrix::nt_world_search::WebSearchEngine::default();
    match engine.search(query, 8) {
        Ok(results) => {
            let mut out = format!("Search results for \"{}\":\n\n", query);
            for (i, r) in results.iter().enumerate() {
                out.push_str(&format!(
                    "{}. {}\n   URL: {}\n   {}\n\n",
                    i + 1,
                    r.title,
                    r.url,
                    r.snippet
                ));
            }
            ToolResult::ok(out.trim().to_string()).with_duration(start.elapsed().as_millis() as u64)
        }
        Err(e) => ToolResult::err(format!("search failed: {e}"))
            .with_duration(start.elapsed().as_millis() as u64),
    }
}

// ─── Image Generation Tools ─────────────────────────────────────────

pub fn tool_image_gen(prompt: &str, width: u32, height: u32, seed: Option<u64>) -> ToolResult {
    let start = std::time::Instant::now();

    use crate::agent::tool::impls::image_gen::{
        self, generate_combined, generate_geometric, generate_mandelbrot, generate_perlin_art,
        png_encode,
    };

    let actual_seed = seed.unwrap_or_else(|| {
        if prompt.is_empty() {
            42
        } else {
            prompt
                .bytes()
                .fold(42u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
        }
    });

    let style = image_gen::parse_style(prompt);
    let pixels = match style {
        image_gen::ImageStyle::Fractal => generate_mandelbrot(width, height, actual_seed),
        image_gen::ImageStyle::Geometric => generate_geometric(width, height, actual_seed),
        image_gen::ImageStyle::Perlin => generate_perlin_art(width, height, actual_seed),
        image_gen::ImageStyle::Combined => generate_combined(width, height, actual_seed),
    };

    let png = png_encode(width, height, &pixels);
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png);

    let result = serde_json::json!({
        "tool": "image_gen",
        "format": "png",
        "width": width,
        "height": height,
        "seed": actual_seed,
        "prompt": prompt,
        "size_bytes": png.len(),
        "data": b64,
    });

    ToolResult::ok(serde_json::to_string_pretty(&result).unwrap_or_default())
        .with_duration(start.elapsed().as_millis() as u64)
}

pub fn tool_minimax_t2i(
    prompt: &str,
    width: u32,
    height: u32,
    n: u32,
    seed: Option<u64>,
) -> ToolResult {
    let start = std::time::Instant::now();

    let api_host = match std::env::var("MINIMAX_API_HOST") {
        Ok(h) => h,
        Err(_) => {
            return ToolResult::err("MINIMAX_API_HOST not set. Try: https://api.minimaxi.com")
                .with_duration(start.elapsed().as_millis() as u64)
        }
    };
    let api_key = match std::env::var("MINIMAX_API_KEY") {
        Ok(k) => k,
        Err(_) => {
            return ToolResult::err(
                "MINIMAX_API_KEY not set. Set via export MINIMAX_API_KEY=sk-...",
            )
            .with_duration(start.elapsed().as_millis() as u64)
        }
    };

    let url = format!("{}/v1/image/generation", api_host);
    let size = format!("{}x{}", width, height);

    let body = serde_json::json!({
        "model": "image-01",
        "prompt": prompt,
        "n": n.min(4),
        "image_size": size,
        "seed": seed,
    });

    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return ToolResult::err(format!("HTTP client: {e}"))
                .with_duration(start.elapsed().as_millis() as u64)
        }
    };

    let resp = match client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
    {
        Ok(r) => r,
        Err(e) => {
            return ToolResult::err(format!("MiniMax API: {e}"))
                .with_duration(start.elapsed().as_millis() as u64)
        }
    };

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return ToolResult::err(format!("MiniMax API {status}: {text}"))
            .with_duration(start.elapsed().as_millis() as u64);
    }

    let text = match resp.text() {
        Ok(t) => t,
        Err(e) => {
            return ToolResult::err(format!("Response read: {e}"))
                .with_duration(start.elapsed().as_millis() as u64)
        }
    };

    let result = serde_json::json!({
        "tool": "minimax_t2i",
        "format": "png",
        "width": width,
        "height": height,
        "model": "image-01",
        "prompt": prompt,
        "raw_response": text,
    });

    ToolResult::ok(serde_json::to_string_pretty(&result).unwrap_or_default())
        .with_duration(start.elapsed().as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_util::{TestDir, GLOBAL_TEST_LOCK};

    #[test]
    fn test_tool_read_nonexistent() {
        let _guard = GLOBAL_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let td = TestDir::new();
        let path = td
            .path()
            .join("nonexistent_file_xyz")
            .to_string_lossy()
            .to_string();
        let r = tool_read(&path);
        assert!(!r.success);
        assert!(r.error.contains("No such file") || r.error.contains("entity not found"));
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
        let _guard = GLOBAL_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let td = TestDir::new();
        let path = td
            .path()
            .join("nonexistent_edit")
            .to_string_lossy()
            .to_string();
        let r = tool_edit(&path, "foo", "bar");
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
        let _guard = GLOBAL_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let td = TestDir::new();
        let path = td
            .path()
            .join("nonexistent_dispatch_test")
            .to_string_lossy()
            .to_string();
        let args = serde_json::json!({"path": path});
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
        let _guard = GLOBAL_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let td = TestDir::new();
        let pattern = td
            .path()
            .join("neotrix_*_nonexistent")
            .to_string_lossy()
            .to_string();
        let args = serde_json::json!({"pattern": pattern});
        let r = dispatch_tool("glob", &args);
        assert!(r.success);
    }

    #[test]
    fn test_tool_write_creates_dirs() {
        let dir = std::env::temp_dir()
            .join("neotrix_tool_test")
            .join("nested")
            .join("dirs");
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
