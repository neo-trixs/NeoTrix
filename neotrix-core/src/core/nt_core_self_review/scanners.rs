//! Self-review scanning engine — extracted from `mod.rs` (Element 协议化拆分).
//! Private `scan_*`/`count_*`/`collect_*` mechanical-check helpers.
//! Public API (types, `scan_for_patterns`, `SelfReviewGate`) stays in `mod.rs`;
//! this module is `pub(crate)` so the parent calls helpers by name via `use scanners::*;`.

use std::collections::HashMap;
use std::path::Path;
use syn;

use super::*;

pub(crate) fn collect_rs_files_recursive(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.ends_with("target") || path.ends_with("bin-archive") {
                    continue;
                }
                collect_rs_files_recursive(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
}

/// Cached listing of `.rs` files under `root`, invalidated by root directory mtime.
///
/// Nearly every check in `run_all` re-walks the whole source tree via `read_dir`
/// (then stats each file through `read_source_cached`/`production_lines_cached`).
/// Walking once and reusing the path list removes ~50 redundant tree traversals.
pub(crate) fn cached_rs_files(root: &Path) -> Arc<Vec<PathBuf>> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, (SystemTime, Arc<Vec<PathBuf>>)>>> =
        OnceLock::new();
    let mtime = std::fs::metadata(root)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((cached_mtime, files)) = guard.get(root) {
        if *cached_mtime == mtime {
            return files.clone();
        }
    }
    let mut files = Vec::new();
    collect_rs_files_recursive(root, &mut files);
    let files = Arc::new(files);
    guard.insert(root.to_path_buf(), (mtime, files.clone()));
    files
}

/// Cached source-file reader.
///
/// Self-review `run_all()` issues ~50 full-tree scans (PA00x pattern checks),
/// each re-reading every `.rs` file in `src/` (1119 files / 12MB). In a debug
/// build a single scan costs ~0.36s, so the full audit runs to tens of seconds —
/// and in `SelfReviewStage` (frequency 1) it runs on *every* seal-loop iteration.
/// Memoizing contents by (mtime, len) collapses the audit to one disk read per
/// file for the whole process lifetime.
pub(crate) fn read_source_cached(path: &Path) -> std::io::Result<String> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, (SystemTime, u64, Arc<str>)>>> = OnceLock::new();
    let metadata = std::fs::metadata(path)?;
    let mtime = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let len = metadata.len();

    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((cached_mtime, cached_len, content)) = guard.get(path) {
        if *cached_mtime == mtime && *cached_len == len {
            return Ok(content.to_string());
        }
    }
    let content = std::fs::read_to_string(path)?;
    guard.insert(
        path.to_path_buf(),
        (mtime, len, Arc::from(content.as_str())),
    );
    Ok(content)
}

/// Production (non-test) source lines, computed once per file and cached by
/// (mtime, len).
///
/// `scan_for_pattern_excluding_tests` is called ~25× per `run_all()`; without a
/// cache each call re-runs the per-line test-context brace tracking over every
/// file (344K lines in this repo). Stripping test contexts once per file turns
/// the hot loop into a plain `line.matches(pattern)` scan over cached lines.
pub(crate) fn production_lines_cached(path: &Path) -> Option<Arc<Vec<String>>> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, (SystemTime, u64, Arc<Vec<String>>)>>> =
        OnceLock::new();
    let metadata = std::fs::metadata(path).ok()?;
    let mtime = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let len = metadata.len();

    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((cached_mtime, cached_len, lines)) = guard.get(path) {
        if *cached_mtime == mtime && *cached_len == len {
            return Some(lines.clone());
        }
    }
    let content = std::fs::read_to_string(path).ok()?;
    let mut prod = Vec::new();
    let mut in_test = false;
    let mut depth = 0usize;
    for line in content.lines() {
        if !in_test {
            // 进入测试上下文: #[cfg(test)] mod 块 或 独立 #[test]/#[tokio::test] fn
            if line.trim().starts_with("#[cfg(test)]")
                || line.trim().starts_with("#[test]")
                || line.trim().starts_with("#[tokio::test]")
            {
                // 仅当后续行出现 `{` 才进入 — 避免属性多行组合误判
                in_test = true;
                depth = 0;
                continue;
            }
        }
        if in_test {
            // brace 深度追踪: 遇 `{` 深度+1, 遇 `}` 深度-1, 归零退出测试上下文
            for ch in line.chars() {
                match ch {
                    '{' => depth += 1,
                    '}' => {
                        if depth == 0 {
                            in_test = false;
                            break;
                        }
                        depth -= 1;
                    }
                    _ => {}
                }
            }
            if in_test {
                continue;
            }
        }
        prod.push(line.to_string());
    }
    let lines = Arc::new(prod);
    guard.insert(path.to_path_buf(), (mtime, len, lines.clone()));
    Some(lines)
}

pub(crate) fn scan_for_pattern_excluding_tests(dir: &Path, pattern: &str) -> usize {
    let mut count = 0usize;
    for path in cached_rs_files(dir).iter() {
        if let Some(lines) = production_lines_cached(path) {
            for line in lines.iter() {
                count += line.matches(pattern).count();
            }
        }
    }
    count
}

/// Simple heuristic: detect files where an import appears only in the import statement itself.
/// Counts patterns like `use foo` when `foo` doesn't appear elsewhere.
pub(crate) fn scan_for_unused_import_patterns(dir: &Path) -> usize {
    let mut count = 0usize;
    for path in cached_rs_files(dir).iter() {
        if let Ok(content) = read_source_cached(path) {
            for line in content.lines() {
                if let Some(stripped) = line.trim().strip_prefix("use ") {
                    let import_name: String = stripped
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == ':')
                        .collect();
                    if import_name.contains("::") {
                        continue;
                    }
                    // Check if the imported name appears outside of use statements
                    let usage_count = content.matches(&import_name).count();
                    if import_name.len() > 3 && usage_count <= 1 {
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

/// Helper: collect all .rs file stems recursively
pub(crate) fn collect_rs_stems(dir: &Path, out: &mut Vec<String>) {
    for path in cached_rs_files(dir).iter() {
        if let Some(stem) = path.file_stem() {
            out.push(stem.to_string_lossy().to_string());
        }
    }
}

/// Scan a directory tree for .rs files without #[test].
/// Appends uncovered file stems to `uncovered`.
pub(crate) fn scan_file_test_coverage(dir: &Path, uncovered: &mut Vec<String>) {
    for path in cached_rs_files(dir).iter() {
        if let Ok(content) = read_source_cached(path) {
            let line_count = content.lines().count();
            if line_count > SelfReviewConfig::default().min_test_line_count
                && !content.contains("#[test]")
            {
                if let Some(stem) = path.file_stem() {
                    uncovered.push(stem.to_string_lossy().to_string());
                }
            }
        }
    }
}

/// Scan for .unwrap() or .expect() within LazyLock initializer closures.
pub(crate) fn scan_for_pattern_in_lazy_init(dir: &Path) -> usize {
    let mut count = 0usize;
    for path in cached_rs_files(dir).iter() {
        if let Ok(content) = read_source_cached(path) {
            let lines: Vec<&str> = content.lines().collect();
            for i in 0..lines.len() {
                if lines[i].contains("LazyLock::new(") {
                    // Check following lines for unwrap/expect in the closure
                    for j in (i + 1)..lines.len().min(i + 20) {
                        let trimmed = lines[j].trim();
                        if trimmed.starts_with('}') || trimmed.starts_with(");") {
                            break;
                        }
                        if trimmed.contains(".unwrap(") || trimmed.contains(".expect(") {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    count
}

// ─── Python scanner helpers ───

/// Scan Python files for a simple pattern match.
pub(crate) fn scan_python_for_pattern(dir: &Path, pattern: &str) -> usize {
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            if path.extension().is_some_and(|e| e == "py") {
                if let Ok(content) = read_source_cached(&path) {
                    count += content.matches(pattern).count();
                }
            }
        }
    }
    count
}

/// Count bare `except:` clauses (not `except Exception:`, `except ValueError:`, etc.)
pub(crate) fn scan_python_for_bare_except(dir: &Path) -> usize {
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            if path.extension().is_some_and(|e| e == "py") {
                if let Ok(content) = read_source_cached(&path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        // Match bare `except:` with nothing after `except`
                        if trimmed.starts_with("except:")
                            || trimmed == "except :"
                            || trimmed.starts_with("except: ")
                        {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    count
}

/// Scan Python files for f-string patterns in SQL statements (like `f"SELECT ... {value}"`)
pub(crate) fn scan_python_fstring_in_sql(dir: &Path) -> usize {
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            if path.extension().is_some_and(|e| e == "py") {
                if let Ok(content) = read_source_cached(&path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        // Detect f-strings containing SQL keywords with curly-brace interpolation
                        if (trimmed.starts_with("f\"") || trimmed.starts_with("f'"))
                            && (trimmed.contains("SELECT")
                                || trimmed.contains("INSERT")
                                || trimmed.contains("UPDATE")
                                || trimmed.contains("DELETE"))
                            && trimmed.contains('{')
                            && trimmed.contains('}')
                        {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    count
}

// ─── Cycle 29 scanner helpers — Karpathy-inspired code principles (PA020-PA023) ───

/// Scan for function declarations with single-uppercase-letter generic params
/// (`fn foo<T>`) where the param name doesn't appear in the function body,
/// suggesting they may be unused or over-abstracted.
pub(crate) fn scan_for_unused_generic_params(dir: &Path) -> usize {
    let mut count = 0usize;
    for path in cached_rs_files(dir).iter() {
        if let Ok(content) = read_source_cached(path) {
            let lines: Vec<&str> = content.lines().collect();
            for i in 0..lines.len() {
                let trimmed = lines[i].trim();
                if !trimmed.starts_with("fn ") && !trimmed.starts_with("pub fn ") {
                    continue;
                }
                let sig = &lines[i];
                let open_angle = match sig.find('<') {
                    Some(pos) => pos,
                    None => continue,
                };
                let close_angle = match sig[open_angle..].find('>') {
                    Some(pos) => open_angle + pos,
                    None => continue,
                };
                let generics_section = &sig[open_angle + 1..close_angle];
                for param in generics_section.split(',') {
                    let p = param.trim().split(':').next().unwrap_or("").trim();
                    if p.is_empty() || !p.chars().all(|c| c.is_uppercase() || c == '_') {
                        continue;
                    }
                    let mut used = false;
                    for j in i + 1..lines.len().min(i + 60) {
                        if lines[j].trim().starts_with("fn ")
                            || lines[j].trim().starts_with("pub fn ")
                        {
                            break;
                        }
                        if lines[j].contains(p) {
                            used = true;
                            break;
                        }
                    }
                    if !used {
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

/// Count lines with deep indentation (≥ `levels * 4` spaces, indicating >5 levels of nesting).
pub(crate) fn count_deeply_nested_lines(dir: &Path, levels: usize) -> usize {
    let threshold = levels * 4;
    let mut count = 0usize;
    for path in cached_rs_files(dir).iter() {
        if let Ok(content) = read_source_cached(path) {
            for line in content.lines() {
                let leading_spaces = line.len() - line.trim_start().len();
                if leading_spaces >= threshold {
                    count += 1;
                }
            }
        }
    }
    count
}

/// Count functions that span more than `max_lines` lines.
/// Uses simple brace-depth tracking to find matching closing braces.
pub(crate) fn count_long_functions(dir: &Path, max_lines: usize) -> usize {
    let mut count = 0usize;
    for path in cached_rs_files(dir).iter() {
        if let Ok(content) = read_source_cached(path) {
            count += count_long_functions_in_content(&content, max_lines);
        }
    }
    count
}

pub(crate) fn count_long_functions_in_content(content: &str, max_lines: usize) -> usize {
    let lines: Vec<&str> = content.lines().collect();
    let mut count = 0usize;
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        let is_fn_start = trimmed.starts_with("fn ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("pub(crate) fn ")
            || trimmed.starts_with("pub(super) fn ");
        if !is_fn_start {
            i += 1;
            continue;
        }
        if trimmed.ends_with(';') {
            i += 1;
            continue;
        }
        let mut brace_depth = 0i32;
        let fn_start = i;
        let mut found_open = false;
        let mut closed = false;
        #[allow(clippy::mut_range_bound)]
        for j in i..lines.len() {
            for ch in lines[j].chars() {
                if ch == '{' {
                    brace_depth += 1;
                    found_open = true;
                } else if ch == '}' {
                    brace_depth -= 1;
                }
            }
            if found_open && brace_depth == 0 {
                let fn_len = j - fn_start;
                if fn_len > max_lines {
                    count += 1;
                }
                i = j + 1;
                closed = true;
                break;
            }
            if brace_depth < 0 {
                i = j + 1;
                closed = true;
                break;
            }
            if j - fn_start > max_lines * 2 {
                i = j + 1;
                closed = true;
                break;
            }
        }
        if !closed {
            i = fn_start + 1;
        }
    }
    count
}

/// Count traits that have only one implementation (over-abstracted pattern).
pub(crate) fn count_single_impl_traits(dir: &Path) -> usize {
    let mut names: Vec<(String, String)> = Vec::new(); // (trait_name, file_path)
    let mut impl_counts: HashMap<String, usize> = HashMap::new();
    for path in cached_rs_files(dir).iter() {
        if let Ok(content) = read_source_cached(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some(trait_name) = trimmed
                    .strip_prefix("pub trait ")
                    .or_else(|| trimmed.strip_prefix("trait "))
                {
                    let name: String = trait_name
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if !name.is_empty() {
                        names.push((name, path.to_string_lossy().to_string()));
                    }
                }
                if let Some(impl_str) = trimmed
                    .strip_prefix("impl ")
                    .or_else(|| trimmed.strip_prefix("pub impl "))
                {
                    if let Some(for_str) = impl_str.find(" for ") {
                        let trait_part = &impl_str[..for_str].trim();
                        if let Some(name) = trait_part.split('<').next() {
                            let name = name.trim();
                            if !name.is_empty() && !name.contains(' ') {
                                *impl_counts.entry(name.to_string()).or_default() += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    // Count declared traits with exactly one implementation
    names
        .iter()
        .filter(|(name, _)| impl_counts.get(name.as_str()).copied().unwrap_or(0) == 1)
        .count()
}

/// Count if-else chains longer than `max_chain` (consecutive `else if` / `else` lines).
pub(crate) fn count_long_if_chains(dir: &Path, max_chain: usize) -> usize {
    let mut count = 0usize;
    for path in cached_rs_files(dir).iter() {
        if let Ok(content) = read_source_cached(path) {
            let lines: Vec<&str> = content.lines().collect();
            let mut chain = 0usize;
            for line in &lines {
                let trimmed = line.trim();
                if trimmed.starts_with("} else if ")
                    || trimmed.starts_with("else if ")
                    || trimmed == "} else {"
                    || trimmed.starts_with("else {")
                {
                    chain += 1;
                } else {
                    if chain > max_chain {
                        count += 1;
                    }
                    chain = 0;
                }
            }
        }
    }
    count
}

/// Count match expressions with more than `max_arms` arms.
pub(crate) fn count_excessive_match_arms(dir: &Path, max_arms: usize) -> usize {
    let mut count = 0usize;
    for path in cached_rs_files(dir).iter() {
        if let Ok(content) = read_source_cached(path) {
            let lines: Vec<&str> = content.lines().collect();
            let mut i = 0;
            while i < lines.len() {
                if lines[i].contains("match ") && !lines[i].trim().starts_with("//") {
                    let mut brace_depth = 0i32;
                    let mut arms = 0usize;
                    let mut in_match = false;
                    let start = i;
                    for j in i..lines.len() {
                        for ch in lines[j].chars() {
                            if ch == '{' {
                                brace_depth += 1;
                                if !in_match {
                                    in_match = true;
                                }
                            } else if ch == '}' {
                                brace_depth -= 1;
                            }
                        }
                        if in_match && brace_depth == 0 {
                            i = j;
                            break;
                        }
                        if in_match && j > start {
                            let tl = lines[j].trim();
                            if tl.starts_with('|') || tl.contains("=>") {
                                arms += 1;
                            }
                        }
                        if j - i > SelfReviewConfig::default().scan_safety_bound {
                            i = j;
                            break;
                        }
                    }
                    if arms > max_arms {
                        count += 1;
                    }
                }
                i += 1;
            }
        }
    }
    count
}

/// Count function declarations with more than `max_params` parameters.
pub(crate) fn count_excessive_param_count(dir: &Path, max_params: usize) -> usize {
    let mut count = 0usize;
    for path in cached_rs_files(dir).iter() {
        if let Ok(content) = read_source_cached(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if !trimmed.starts_with("fn ") && !trimmed.starts_with("pub fn ") {
                    continue;
                }
                if let Some(paren_open) = trimmed.find('(') {
                    if let Some(paren_close) = trimmed[paren_open..].find(')') {
                        let params_str = &trimmed[paren_open + 1..paren_open + paren_close];
                        if params_str.is_empty() {
                            continue;
                        }
                        let params: Vec<&str> = params_str
                            .split(',')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty() && !s.contains("self"))
                            .collect();
                        if params.len() > max_params {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    count
}

/// Count `// TODO` / `// FIXME` comments that aren't in files with a `#[test]`.
pub(crate) fn count_todos_without_nearby_test(dir: &Path) -> usize {
    let mut count = 0usize;
    for path in cached_rs_files(dir).iter() {
        if let Ok(content) = read_source_cached(path) {
            let has_test = content.contains("#[test]");
            let todo_count = content.matches("// TODO").count()
                + content.matches("//TODO").count()
                + content.matches("// FIXME").count();
            if todo_count > 0 && !has_test {
                count += todo_count;
            }
        }
    }
    count
}

/// Count `&mut self` methods that do not return `Result` (suggesting no error handling).
pub(crate) fn count_state_mutation_no_result(dir: &Path) -> usize {
    let mut count = 0usize;
    for path in cached_rs_files(dir).iter() {
        if let Ok(content) = read_source_cached(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if (trimmed.starts_with("fn ") || trimmed.starts_with("pub fn "))
                    && trimmed.contains("&mut self")
                    && !trimmed.contains("Result")
                {
                    count += 1;
                }
            }
        }
    }
    count
}

/// Count `pub fn` declarations without doc comments indicating success criteria
/// (no "Returns", "Goal", "Purpose", or "Success" in the preceding doc block).
pub(crate) fn count_pub_fn_without_goal_doc(dir: &Path) -> usize {
    let mut count = 0usize;
    for path in cached_rs_files(dir).iter() {
        if let Ok(content) = read_source_cached(path) {
            let lines: Vec<&str> = content.lines().collect();
            for i in 0..lines.len() {
                let trimmed = lines[i].trim();
                if !trimmed.starts_with("pub fn ") {
                    continue;
                }
                let mut has_goal = false;
                for j in (0.max(i.saturating_sub(10)))..i {
                    let d = lines[j].trim();
                    if d.starts_with("///") {
                        if d.contains("Returns")
                            || d.contains("Goal")
                            || d.contains("Purpose")
                            || d.contains("Success")
                            || d.contains("Output")
                        {
                            has_goal = true;
                            break;
                        }
                    } else if !d.starts_with("///") && !d.is_empty() {
                        break;
                    }
                }
                if !has_goal {
                    count += 1;
                }
            }
        }
    }
    count
}

/// Count files changed in the last commit (git diff --stat).
pub(crate) fn count_files_in_last_commit(repo_dir: &Path) -> usize {
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", "HEAD~1..HEAD"])
        .current_dir(repo_dir)
        .output();
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.lines().filter(|l| !l.is_empty()).count()
        }
        Err(_) => 0,
    }
}

/// Count lines added in the last commit (git diff --shortstat).
pub(crate) fn count_lines_in_last_commit(repo_dir: &Path) -> usize {
    let output = std::process::Command::new("git")
        .args(["diff", "--shortstat", "HEAD~1..HEAD"])
        .current_dir(repo_dir)
        .output();
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let s = stdout.trim();
            if s.is_empty() {
                return 0;
            }
            if let Some(ins_part) = s.split(',').nth(1) {
                let cleaned: String = ins_part.chars().filter(|c| c.is_ascii_digit()).collect();
                return cleaned.parse().unwrap_or(0);
            }
            0
        }
        Err(_) => 0,
    }
}

pub(crate) fn estimate_brace_depth(code: &str) -> usize {
    let mut depth = 0usize;
    let mut max_depth = 0usize;
    for c in code.chars() {
        if c == '{' {
            depth += 1;
            max_depth = max_depth.max(depth);
        } else if c == '}' {
            depth = depth.saturating_sub(1);
        }
    }
    max_depth
}

pub(crate) fn syn_file_max_depth(file: &syn::File) -> usize {
    file.items
        .iter()
        .map(|item| syn_item_max_depth(item, 0))
        .max()
        .unwrap_or(0)
}

pub(crate) fn syn_item_max_depth(item: &syn::Item, depth: usize) -> usize {
    match item {
        syn::Item::Fn(f) => syn_block_max_depth(&f.block, depth),
        syn::Item::Impl(imp) => {
            let d = depth + 1;
            d.max(
                imp.items
                    .iter()
                    .map(|ii| syn_impl_item_max_depth(ii, d))
                    .max()
                    .unwrap_or(d),
            )
        }
        syn::Item::Trait(t) => {
            let d = depth + 1;
            d.max(
                t.items
                    .iter()
                    .map(|ti| syn_trait_item_max_depth(ti, d))
                    .max()
                    .unwrap_or(d),
            )
        }
        syn::Item::Mod(m) => match &m.content {
            Some((_, items)) => {
                let d = depth + 1;
                d.max(
                    items
                        .iter()
                        .map(|i| syn_item_max_depth(i, d))
                        .max()
                        .unwrap_or(d),
                )
            }
            None => depth,
        },
        syn::Item::ForeignMod(fm) => {
            let d = depth + 1;
            d.max(
                fm.items
                    .iter()
                    .map(|fi| syn_foreign_item_max_depth(fi, d))
                    .max()
                    .unwrap_or(d),
            )
        }
        syn::Item::Const(c) => syn_expr_max_depth(&c.expr, depth),
        syn::Item::Static(s) => syn_expr_max_depth(&s.expr, depth),
        _ => depth,
    }
}

pub(crate) fn syn_impl_item_max_depth(item: &syn::ImplItem, depth: usize) -> usize {
    match item {
        syn::ImplItem::Fn(f) => syn_block_max_depth(&f.block, depth),
        syn::ImplItem::Const(c) => syn_expr_max_depth(&c.expr, depth),
        _ => depth,
    }
}

pub(crate) fn syn_trait_item_max_depth(item: &syn::TraitItem, depth: usize) -> usize {
    match item {
        syn::TraitItem::Fn(f) => f
            .default
            .as_ref()
            .map(|b| syn_block_max_depth(b, depth))
            .unwrap_or(depth),
        syn::TraitItem::Const(c) => c
            .default
            .as_ref()
            .map(|(_, e)| syn_expr_max_depth(e, depth))
            .unwrap_or(depth),
        _ => depth,
    }
}

pub(crate) fn syn_foreign_item_max_depth(_item: &syn::ForeignItem, depth: usize) -> usize {
    depth
}

pub(crate) fn syn_block_max_depth(block: &syn::Block, depth: usize) -> usize {
    let d = depth + 1;
    d.max(syn_stmts_max_depth(&block.stmts, d))
}

pub(crate) fn syn_stmts_max_depth(stmts: &[syn::Stmt], depth: usize) -> usize {
    stmts
        .iter()
        .map(|s| syn_stmt_max_depth(s, depth))
        .max()
        .unwrap_or(depth)
}

pub(crate) fn syn_stmt_max_depth(stmt: &syn::Stmt, depth: usize) -> usize {
    match stmt {
        syn::Stmt::Item(item) => syn_item_max_depth(item, depth),
        syn::Stmt::Expr(expr, _) => syn_expr_max_depth(expr, depth),
        syn::Stmt::Local(local) => local
            .init
            .as_ref()
            .map(|init| syn_expr_max_depth(&init.expr, depth))
            .unwrap_or(depth),
        _ => depth,
    }
}

pub(crate) fn syn_expr_max_depth(expr: &syn::Expr, depth: usize) -> usize {
    match expr {
        syn::Expr::Block(eb) => syn_block_max_depth(&eb.block, depth),
        syn::Expr::If(ei) => {
            let then_max = syn_block_max_depth(&ei.then_branch, depth);
            let else_max = ei
                .else_branch
                .as_ref()
                .map(|(_, e)| syn_expr_max_depth(e, depth))
                .unwrap_or(0);
            then_max.max(else_max)
        }
        syn::Expr::While(w) => syn_block_max_depth(&w.body, depth),
        syn::Expr::ForLoop(fl) => syn_block_max_depth(&fl.body, depth),
        syn::Expr::Loop(l) => syn_block_max_depth(&l.body, depth),
        syn::Expr::Match(m) => {
            let d = depth + 1;
            d.max(
                m.arms
                    .iter()
                    .map(|arm| syn_expr_max_depth(&arm.body, d))
                    .max()
                    .unwrap_or(d),
            )
        }
        syn::Expr::Unsafe(u) => syn_block_max_depth(&u.block, depth),
        syn::Expr::Closure(c) => syn_expr_max_depth(&c.body, depth),
        _ => depth,
    }
}

