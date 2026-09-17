//! NT-IO Output Compressor — Real tool output compression for context efficiency
//!
//! Compresses different content types differently:
//! - Test output: extract pass/fail counts, errors only
//! - Git diffs: summarize files changed, key hunks
//! - File reads: extract relevant sections, skip boilerplate
//! - Error messages: deduplicate, extract root causes
//! - JSON/structured: extract keys, summarize arrays
//! - Stack traces: collapse repeated frames
//!
//! Replaces the stub truncation in memory_consolidation.rs with content-aware compression.

#![forbid(unsafe_code)]

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Content type discriminator for selecting compression strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentType {
    TestOutput,
    GitDiff,
    FileRead,
    ErrorMessage,
    JsonData,
    PlainText,
    Code,
    StackTrace,
}

/// Compression configuration.
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Maximum tokens to keep after compression.
    pub max_tokens: usize,
    /// Always preserve error/warning lines even if over budget.
    pub preserve_errors: bool,
    /// Keep the first N lines verbatim (headers, context).
    pub preserve_first_n_lines: usize,
    /// Deduplicate near-identical lines.
    pub dedup_similar: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            max_tokens: 2048,
            preserve_errors: true,
            preserve_first_n_lines: 3,
            dedup_similar: true,
        }
    }
}

/// Result of a compression operation.
#[derive(Debug, Clone)]
pub struct CompressedOutput {
    pub content: String,
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub ratio: f64,
    pub content_type: ContentType,
    pub preserved_sections: Vec<String>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub struct OutputCompressor;

impl OutputCompressor {
    /// Compress `output` of the given `content_type` under `config`.
    pub fn compress(output: &str, content_type: ContentType, config: &CompressionConfig) -> CompressedOutput {
        let original_tokens = estimate_tokens(output);

        if original_tokens <= config.max_tokens {
            return CompressedOutput {
                content: output.to_string(),
                original_tokens,
                compressed_tokens: original_tokens,
                ratio: 1.0,
                content_type,
                preserved_sections: vec![output.to_string()],
            };
        }

        let compressed = match content_type {
            ContentType::TestOutput => compress_test_output(output, config),
            ContentType::GitDiff => compress_git_diff(output, config),
            ContentType::FileRead => compress_file_read(output, config),
            ContentType::ErrorMessage => compress_error_message(output, config),
            ContentType::JsonData => compress_json_data(output, config),
            ContentType::PlainText => compress_plain_text(output, config),
            ContentType::Code => compress_code(output, config),
            ContentType::StackTrace => compress_stacktrace(output, config),
        };

        let compressed_tokens = estimate_tokens(&compressed);
        let ratio = if compressed_tokens > 0 {
            original_tokens as f64 / compressed_tokens as f64
        } else {
            1.0
        };

        let preserved = extract_preserved_sections(&compressed, config.preserve_first_n_lines);

        CompressedOutput {
            content: compressed,
            original_tokens,
            compressed_tokens,
            ratio,
            content_type,
            preserved_sections: preserved,
        }
    }
}

// ---------------------------------------------------------------------------
// Token estimation
// ---------------------------------------------------------------------------

/// Rough token estimation: ~4 chars per token (works for English/code).
pub fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

// ---------------------------------------------------------------------------
// Line deduplication
// ---------------------------------------------------------------------------

/// Remove near-duplicate lines. Two lines are "similar" if they differ by <=10%
/// of their characters (edit distance heuristic via char-set overlap).
pub fn dedup_lines(lines: &[String]) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut seen: Vec<&str> = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            result.push(line.clone());
            continue;
        }
        if seen.iter().any(|s| lines_similar(s, trimmed)) {
            continue;
        }
        seen.push(trimmed);
        result.push(line.clone());
    }
    result
}

fn lines_similar(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let max_len = a.len().max(b.len());
    if max_len == 0 {
        return true;
    }
    let common = a.chars().filter(|c| b.contains(*c)).count();
    common as f64 / max_len as f64 > 0.9
}

// ---------------------------------------------------------------------------
// Test output compressor
// ---------------------------------------------------------------------------

fn compress_test_output(output: &str, config: &CompressionConfig) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let mut result = Vec::new();
    let mut total = 0u64;
    let mut passed = 0u64;
    let mut failed = 0u64;
    let mut errors: Vec<String> = Vec::new();
    let mut panics: Vec<String> = Vec::new();

    for line in &lines {
        let lower = line.to_lowercase();
        if lower.contains("test result") || (lower.contains("running") && lower.contains("test")) {
            result.push(line.to_string());
            if let Some((t, p, f)) = parse_test_summary(line) {
                total = t;
                passed = p;
                failed = f;
            }
            continue;
        }
        if lower.contains("error") || lower.contains("fail") || lower.contains("panic") {
            errors.push(line.to_string());
            if lower.contains("panic") {
                panics.push(line.to_string());
            }
        }
    }

    // Also grab lines immediately following error lines (context)
    for i in 0..lines.len() {
        let lower = lines[i].to_lowercase();
        if lower.contains("error") || lower.contains("fail") {
            if i + 1 < lines.len() {
                let next = lines[i + 1].trim();
                if !next.is_empty() && !errors.contains(&lines[i + 1].to_string()) {
                    errors.push(lines[i + 1].to_string());
                }
            }
        }
    }

    let mut out = String::new();
    if !result.is_empty() {
        out.push_str("== Test Summary ==\n");
        for r in &result {
            out.push_str(r);
            out.push('\n');
        }
        out.push_str(&format!("Total: {} | Passed: {} | Failed: {}\n\n", total, passed, failed));
    }

    if !panics.is_empty() {
        out.push_str("== Panics ==\n");
        for p in &panics {
            out.push_str(p);
            out.push('\n');
        }
        out.push('\n');
    }

    if !errors.is_empty() {
        out.push_str("== Errors ==\n");
        let deduped = if config.dedup_similar {
            dedup_lines(&errors)
        } else {
            errors
        };
        for e in &deduped {
            out.push_str(e);
            out.push('\n');
        }
    }

    // Fallback: if nothing was extracted, do a simple line budget
    if out.is_empty() {
        let budget_lines = config.max_tokens / 4;
        out = lines.iter().take(budget_lines.max(10)).cloned().collect::<Vec<_>>().join("\n");
        if lines.len() > budget_lines {
            out.push_str(&format!("\n... [{} lines truncated]", lines.len() - budget_lines));
        }
    }

    out
}

fn parse_test_summary(line: &str) -> Option<(u64, u64, u64)> {
    let lower = line.to_lowercase();
    let passed = extract_count(&lower, "passed").or_else(|| extract_count(&lower, "ok"))?;
    let failed = extract_count(&lower, "failed").unwrap_or(0);
    let total = passed + failed;
    Some((total, passed, failed))
}

fn extract_count(s: &str, keyword: &str) -> Option<u64> {
    let mut chars = s.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            let mut num = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    num.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            let rest: String = chars.clone().collect();
            if rest.to_lowercase().contains(keyword) {
                if let Ok(n) = num.parse::<u64>() {
                    return Some(n);
                }
            }
        } else {
            chars.next();
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Git diff compressor
// ---------------------------------------------------------------------------

fn compress_git_diff(output: &str, _config: &CompressionConfig) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let mut files: Vec<String> = Vec::new();
    let mut stats = DiffStats::default();
    let mut current_file = String::new();
    let mut current_hunks: Vec<String> = Vec::new();
    let mut in_hunk = false;

    for line in &lines {
        if line.starts_with("diff --git") {
            if !current_file.is_empty() {
                flush_file(&current_file, &current_hunks, &mut files, &mut stats);
            }
            current_file = line.strip_prefix("diff --git a/").unwrap_or(line).to_string();
            let slash_pos = current_file.find(" b/");
            if let Some(pos) = slash_pos {
                current_file = current_file[pos + 3..].to_string();
            }
            current_hunks.clear();
            in_hunk = false;
        } else if line.starts_with("@@") {
            in_hunk = true;
            stats.hunks += 1;
            current_hunks.push(line.to_string());
        } else if line.starts_with('+') && !line.starts_with("+++") {
            stats.additions += 1;
            if in_hunk {
                current_hunks.push(line.to_string());
            }
        } else if line.starts_with('-') && !line.starts_with("---") {
            stats.deletions += 1;
            if in_hunk {
                current_hunks.push(line.to_string());
            }
        } else if in_hunk && !line.starts_with("+++") && !line.starts_with("---") && !line.starts_with("diff") {
            // Context lines in a hunk
        }
    }
    if !current_file.is_empty() {
        flush_file(&current_file, &current_hunks, &mut files, &mut stats);
    }

    let mut out = String::new();
    out.push_str(&format!(
        "== Diff Summary: {} files, +{} -{} lines, {} hunks ==\n\n",
        stats.files_changed, stats.additions, stats.deletions, stats.hunks
    ));
    out.push_str(&files.join("\n\n"));
    out
}

#[derive(Default)]
struct DiffStats {
    files_changed: u32,
    additions: u32,
    deletions: u32,
    hunks: u32,
}

fn flush_file(name: &str, hunks: &[String], files: &mut Vec<String>, stats: &mut DiffStats) {
    stats.files_changed += 1;
    let max_hunks = 3;
    let kept: Vec<&str> = hunks.iter().take(max_hunks).map(|s| s.as_str()).collect();
    let truncated = hunks.len().saturating_sub(max_hunks);

    let mut entry = format!("--- {} ---\n", name);
    for h in &kept {
        entry.push_str(h);
        entry.push('\n');
    }
    if truncated > 0 {
        entry.push_str(&format!("... [{} more hunks]\n", truncated));
    }
    files.push(entry);
}

// ---------------------------------------------------------------------------
// File read compressor
// ---------------------------------------------------------------------------

fn compress_file_read(output: &str, config: &CompressionConfig) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let total_lines = lines.len();

    if total_lines <= config.preserve_first_n_lines + 10 {
        return output.to_string();
    }

    let mut out = Vec::new();

    // Preserve header lines (file path, metadata, etc.)
    let header_end = config.preserve_first_n_lines.min(total_lines);
    for line in &lines[..header_end] {
        out.push(line.to_string());
    }

    // Find and preserve code/function bodies
    let mut interesting: Vec<(usize, String)> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub fn")
            || trimmed.starts_with("fn ")
            || trimmed.starts_with("pub struct")
            || trimmed.starts_with("struct ")
            || trimmed.starts_with("pub enum")
            || trimmed.starts_with("enum ")
            || trimmed.starts_with("impl ")
            || trimmed.starts_with("pub impl")
            || trimmed.starts_with("pub trait")
            || trimmed.starts_with("trait ")
        {
            let end = (i + 6).min(total_lines);
            for j in i..end {
                interesting.push((j, lines[j].to_string()));
            }
        }
    }

    if !interesting.is_empty() {
        out.push(String::new());
        out.push("== Key Definitions ==\n".to_string());
        let mut seen_idx = std::collections::HashSet::new();
        for (idx, line) in interesting {
            if seen_idx.insert(idx) {
                out.push(line);
            }
        }
    }

    // Preserve tail
    let tail_start = total_lines.saturating_sub(5);
    if tail_start > header_end {
        out.push(String::new());
        out.push("== Tail ==\n".to_string());
        for line in &lines[tail_start..] {
            out.push(line.to_string());
        }
    }

    let skipped = total_lines - out.len();
    if skipped > 0 {
        out.push(format!("\n... [{} lines of boilerplate skipped]", skipped));
    }

    out.join("\n")
}

// ---------------------------------------------------------------------------
// Error message compressor
// ---------------------------------------------------------------------------

fn compress_error_message(output: &str, config: &CompressionConfig) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let mut error_groups: Vec<ErrorGroup> = Vec::new();
    let mut current_group: Option<ErrorGroup> = None;

    for line in &lines {
        let lower = line.to_lowercase();
        let is_error = lower.contains("error")
            || lower.contains("panic")
            || lower.contains("fatal")
            || lower.contains("warning")
            || lower.contains("failed")
            || lower.contains("exception");

        if is_error {
            if let Some(ref mut group) = current_group {
                group.lines.push(line.to_string());
            } else {
                current_group = Some(ErrorGroup {
                    first_line: line.to_string(),
                    lines: vec![line.to_string()],
                });
            }
        } else if let Some(group) = current_group.take() {
            error_groups.push(group);
        }
    }
    if let Some(group) = current_group {
        error_groups.push(group);
    }

    let groups = if config.dedup_similar {
        dedup_error_groups(error_groups)
    } else {
        error_groups
    };

    let mut out = String::new();
    out.push_str(&format!("== {} Error Group(s) ==\n\n", groups.len()));

    for (i, group) in groups.iter().enumerate() {
        out.push_str(&format!("[{}]\n", i + 1));
        out.push_str(&group.first_line);
        out.push('\n');
        for line in group.lines.iter().skip(1).take(3) {
            out.push_str("  ");
            out.push_str(line);
            out.push('\n');
        }
        if group.lines.len() > 4 {
            out.push_str(&format!("  ... [{} more lines]\n", group.lines.len() - 4));
        }
        out.push('\n');
    }

    out
}

struct ErrorGroup {
    first_line: String,
    lines: Vec<String>,
}

fn dedup_error_groups(groups: Vec<ErrorGroup>) -> Vec<ErrorGroup> {
    let mut result = Vec::new();
    let mut seen_signatures: Vec<String> = Vec::new();

    for group in groups {
        let sig: String = group
            .first_line
            .chars()
            .take(80)
            .filter(|c| !c.is_ascii_digit())
            .collect::<String>()
            .to_lowercase()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();

        let mut merged = false;
        for (j, seen_sig) in seen_signatures.iter().enumerate() {
            if lines_similar(seen_sig, &sig) {
                if let Some(existing) = result.get_mut(j) {
                    existing.lines.extend(group.lines.iter().cloned());
                }
                merged = true;
                break;
            }
        }
        if !merged {
            seen_signatures.push(sig);
            result.push(group);
        }
    }
    result
}

// ---------------------------------------------------------------------------
// JSON data compressor
// ---------------------------------------------------------------------------

fn compress_json_data(output: &str, _config: &CompressionConfig) -> String {
    let trimmed = output.trim();

    if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
        return output.to_string();
    }

    let mut out = String::new();

    if trimmed.starts_with('[') {
        out.push_str("== JSON Array ==\n");
        let elements = count_json_array_elements(trimmed);
        out.push_str(&format!("Elements: {}\n", elements));

        let mut depth = 0i32;
        let mut elem_start = None;
        let mut elem_count = 0u32;

        for (i, c) in trimmed.char_indices() {
            match c {
                '[' if depth == 0 => { depth = 1; elem_start = Some(i + 1); }
                '[' => depth += 1,
                ']' if depth == 1 => {
                    if let Some(start) = elem_start.take() {
                        elem_count += 1;
                        if elem_count <= 3 {
                            let elem = &trimmed[start..i];
                            out.push_str(&format!("  [{}]: {}\n", elem_count - 1, truncate_str(elem, 200)));
                        }
                    }
                    depth = 0;
                }
                ']' => depth -= 1,
                ',' if depth == 1 => {
                    if let Some(start) = elem_start.take() {
                        elem_count += 1;
                        if elem_count <= 3 {
                            let elem = &trimmed[start..i];
                            out.push_str(&format!("  [{}]: {}\n", elem_count - 1, truncate_str(elem, 200)));
                        }
                        elem_start = Some(i + 1);
                    }
                }
                _ => {}
            }
        }
        if let Some(start) = elem_start {
            elem_count += 1;
            if elem_count <= 3 {
                let end = trimmed.len().saturating_sub(1);
                if start < end {
                    let elem = trimmed[start..end].trim();
                    out.push_str(&format!("  [{}]: {}\n", elem_count - 1, truncate_str(elem, 200)));
                }
            }
        }
        if elements > 3 {
            out.push_str(&format!("  ... [{} more elements]\n", elements - 3));
        }
    } else {
        out.push_str("== JSON Object ==\n");
        let keys = extract_json_keys(trimmed);
        out.push_str(&format!("Keys ({}): ", keys.len()));
        out.push_str(&keys.join(", "));
        out.push('\n');

        for (i, key) in keys.iter().take(5) {
            if let Some(val) = extract_json_value(trimmed, key) {
                out.push_str(&format!("  {}: {}\n", key, truncate_str(&val, 150)));
            }
        }
        if keys.len() > 5 {
            out.push_str(&format!("  ... [{} more keys]\n", keys.len() - 5));
        }
    }

    out
}

fn count_json_array_elements(s: &str) -> u32 {
    let mut depth = 0i32;
    let mut count = 0u32;
    let mut in_string = false;
    let mut escape = false;

    for c in s.chars() {
        if escape {
            escape = false;
            continue;
        }
        if c == '\\' && in_string {
            escape = true;
            continue;
        }
        if c == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match c {
            '[' | '{' => depth += 1,
            ']' | '}' => depth -= 1,
            ',' if depth == 1 => count += 1,
            _ => {}
        }
    }
    count + 1
}

fn extract_json_keys(s: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    let mut current_key: Option<String> = None;

    for c in s.chars() {
        if escape {
            escape = false;
            if let Some(ref mut key) = current_key {
                key.push(c);
            }
            continue;
        }
        if c == '\\' && in_string {
            escape = true;
            continue;
        }
        if c == '"' {
            if in_string {
                in_string = false;
                if let Some(key) = current_key.take() {
                    if depth == 1 {
                        keys.push(key);
                    }
                }
            } else {
                in_string = true;
                current_key = Some(String::new());
            }
            continue;
        }
        if in_string {
            if let Some(ref mut key) = current_key {
                key.push(c);
            }
            continue;
        }
        match c {
            '{' | '[' => depth += 1,
            '}' | ']' => depth -= 1,
            _ => {}
        }
    }

    keys
}

fn extract_json_value(s: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let pos = s.find(&pattern)?;
    let after_key = &s[pos + pattern.len()..];
    let colon_pos = after_key.find(':')?;
    let value_start = after_key[colon_pos + 1..].trim_start();
    Some(truncate_str(value_start, 100))
}

// ---------------------------------------------------------------------------
// Stack trace compressor
// ---------------------------------------------------------------------------

fn compress_stacktrace(output: &str, config: &CompressionConfig) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let mut frame_counts: HashMap<String, u32> = HashMap::new();
    let mut unique_frames: Vec<String> = Vec::new();
    let mut preamble: Vec<String> = Vec::new();
    let mut in_trace = false;

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.contains("stack trace")
            || trimmed.contains("backtrace")
            || trimmed.contains("traceback")
        {
            in_trace = true;
            preamble.push(line.to_string());
            continue;
        }

        if in_trace {
            let is_frame = trimmed.starts_with('#')
                || trimmed.starts_with("at ")
                || trimmed.contains(".rs:")
                || trimmed.contains(".js:")
                || trimmed.contains(".py:");

            if is_frame {
                let normalized = normalize_frame(trimmed);
                *frame_counts.entry(normalized.clone()).or_insert(0) += 1;
                if !unique_frames.contains(&normalized) {
                    unique_frames.push(normalized);
                }
            } else {
                preamble.push(line.to_string());
            }
        } else {
            preamble.push(line.to_string());
        }
    }

    let mut out = preamble.join("\n");
    if !unique_frames.is_empty() {
        out.push_str("\n\n== Stack Trace ==\n");
        let max_frames = config.max_tokens / 10;

        for frame in unique_frames.iter().take(max_frames) {
            let count = frame_counts.get(frame).copied().unwrap_or(1);
            if count > 1 {
                out.push_str(&format!("  {} (x{})\n", frame, count));
            } else {
                out.push_str(&format!("  {}\n", frame));
            }
        }
        if unique_frames.len() > max_frames {
            out.push_str(&format!(
                "  ... [{} more frames]\n",
                unique_frames.len() - max_frames
            ));
        }
        let dupes: u32 = frame_counts.values().map(|c| c - 1).sum();
        if dupes > 0 {
            out.push_str(&format!("  ({} duplicate frames collapsed)\n", dupes));
        }
    }

    out
}

fn normalize_frame(frame: &str) -> String {
    let mut result = String::new();
    let mut chars = frame.chars().peekable();
    while let Some(c) = chars.next() {
        if c == ':' {
            let rest: String = chars.clone().take(6).collect();
            if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
                for _ in 0..rest.len() {
                    chars.next();
                }
                result.push_str(":N");
                continue;
            }
        }
        result.push(c);
    }
    result
}

// ---------------------------------------------------------------------------
// Plain text compressor
// ---------------------------------------------------------------------------

fn compress_plain_text(output: &str, config: &CompressionConfig) -> String {
    let mut lines: Vec<String> = output.lines().map(|s| s.to_string()).collect();

    if config.dedup_similar {
        lines = dedup_lines(&lines);
    }

    let budget_lines = config.max_tokens / 4;
    let first_n = config.preserve_first_n_lines;

    if lines.len() <= budget_lines {
        return lines.join("\n");
    }

    let mut out = Vec::new();

    for line in lines.iter().take(first_n) {
        out.push(line.clone());
    }

    let middle_budget = budget_lines.saturating_sub(first_n + 5);
    let total_middle = lines.len() - first_n - 5;
    if middle_budget > 0 && total_middle > 0 {
        let step = (total_middle / middle_budget).max(1);
        let mut taken = 0;
        for (i, line) in lines.iter().enumerate().skip(first_n) {
            if i >= lines.len() - 5 {
                break;
            }
            if taken % step == 0 {
                out.push(line.clone());
            }
            taken += 1;
        }
    }

    let tail_start = lines.len().saturating_sub(5);
    for line in lines.iter().skip(tail_start) {
        out.push(line.clone());
    }

    let skipped = lines.len() - out.len();
    out.push(format!("\n... [{} lines compressed]", skipped));
    out.join("\n")
}

// ---------------------------------------------------------------------------
// Code compressor
// ---------------------------------------------------------------------------

fn compress_code(output: &str, config: &CompressionConfig) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let mut out = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let keep = trimmed.starts_with("use ")
            || trimmed.starts_with("pub use")
            || trimmed.starts_with("mod ")
            || trimmed.starts_with("pub mod")
            || trimmed.starts_with("pub fn")
            || trimmed.starts_with("fn ")
            || trimmed.starts_with("pub struct")
            || trimmed.starts_with("pub enum")
            || trimmed.starts_with("pub trait")
            || trimmed.starts_with("pub type")
            || trimmed.starts_with("impl ")
            || trimmed.starts_with("pub impl")
            || trimmed.starts_with("///")
            || trimmed.starts_with("#[")
            || trimmed.starts_with("//!");

        if keep {
            out.push(line.to_string());
            if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn") || trimmed.starts_with("impl ") {
                for extra in lines.iter().skip(i + 1).take(4) {
                    out.push(extra.to_string());
                }
            }
        }
    }

    let budget_lines = config.max_tokens / 4;
    if out.len() > budget_lines {
        out.truncate(budget_lines);
        out.push(format!("... [{} more lines truncated]", lines.len() - budget_lines));
    }

    out.join("\n")
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn extract_preserved_sections(text: &str, n: usize) -> Vec<String> {
    text.lines().take(n).map(|s| s.to_string()).collect()
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("abcd"), 1);
        assert_eq!(estimate_tokens("abcdefgh"), 2);
    }

    #[test]
    fn test_dedup_lines() {
        let lines = vec![
            "error: cannot find type `Foo`".to_string(),
            "error: cannot find type `Bar`".to_string(),
            "error: cannot find type `Foo`".to_string(),
            "".to_string(),
            "warning: unused variable".to_string(),
        ];
        let result = dedup_lines(&lines);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_compress_test_output_passing() {
        let output = "running 3 tests\ntest test_a ... ok\ntest test_b ... ok\ntest test_c ... ok\n\ntest result: ok. 3 passed; 0 failed; 0 ignored";
        let config = CompressionConfig::default();
        let result = OutputCompressor::compress(output, ContentType::TestOutput, &config);
        assert!(result.content.contains("3 passed"));
        assert!(result.content.contains("Test Summary"));
    }

    #[test]
    fn test_compress_test_output_with_errors() {
        let output = "running 2 tests\ntest test_a ... FAILED\ntest test_b ... ok\n\nfailures:\n\nerror[E0308]: mismatched types\n  --> src/main.rs:5:10\n   |\n5  |     let x: i32 = \"hello\";\n   |            ---   ^^^^^^^ expected `i32`, found `&str`\n\ntest result: FAILED. 1 passed; 1 failed";
        let config = CompressionConfig::default();
        let result = OutputCompressor::compress(output, ContentType::TestOutput, &config);
        assert!(result.content.contains("FAILED") || result.content.contains("Errors"));
        assert!(result.compressed_tokens <= result.original_tokens);
    }

    #[test]
    fn test_compress_git_diff() {
        let output = "diff --git a/src/main.rs b/src/main.rs\nindex abc..def 100644\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,5 +1,6 @@\n fn main() {\n+    let x = 1;\n     println!(\"hello\");\n }\n\ndiff --git a/Cargo.toml b/Cargo.toml\nindex 123..456 100644\n--- a/Cargo.toml\n+++ b/Cargo.toml\n@@ -1,3 +1,4 @@\n [package]\n+version = \"0.1.0\"\n name = \"test\"\n";
        let config = CompressionConfig::default();
        let result = OutputCompressor::compress(output, ContentType::GitDiff, &config);
        assert!(result.content.contains("Diff Summary"));
        assert!(result.content.contains("2 files"));
    }

    #[test]
    fn test_compress_file_read_skips_boilerplate() {
        let mut lines: Vec<String> = Vec::new();
        lines.push("src/main.rs".to_string());
        lines.push("=".repeat(60));
        for _ in 0..50 {
            lines.push("    // filler line with no useful content".to_string());
        }
        lines.push("pub fn main() {".to_string());
        lines.push("    println!(\"hello\");".to_string());
        lines.push("    let x = 42;".to_string());
        lines.push("}".to_string());
        for _ in 0..5 {
            lines.push("    // trailing".to_string());
        }

        let output = lines.join("\n");
        let config = CompressionConfig::default();
        let result = OutputCompressor::compress(&output, ContentType::FileRead, &config);
        assert!(result.content.contains("Key Definitions"));
        assert!(result.compressed_tokens < result.original_tokens);
    }

    #[test]
    fn test_compress_error_message_dedup() {
        let output = "error[E0308]: mismatched types\n  --> src/a.rs:5\nerror[E0308]: mismatched types\n  --> src/b.rs:10\nerror[E0592]: duplicate definitions\n  --> src/c.rs:20\nerror[E0308]: mismatched types\n  --> src/d.rs:30\nwarning: unused variable\n  --> src/e.rs:1";
        let config = CompressionConfig::default();
        let result = OutputCompressor::compress(output, ContentType::ErrorMessage, &config);
        assert!(result.content.contains("Error Group"));
        let e0308_count = result.content.matches("E0308").count();
        assert!(e0308_count <= 2, "Expected merged E0308 errors, got {}", e0308_count);
    }

    #[test]
    fn test_compress_json_object() {
        let output = r#"{"name":"test","version":"1.0","dependencies":{"dep1":"1.0","dep2":"2.0"},"scripts":{"build":"cargo build"}}"#;
        let config = CompressionConfig::default();
        let result = OutputCompressor::compress(output, ContentType::JsonData, &config);
        assert!(result.content.contains("JSON Object"));
        assert!(result.content.contains("name"));
    }

    #[test]
    fn test_compress_json_array() {
        let output = r#"[{"id":1,"name":"a"},{"id":2,"name":"b"},{"id":3,"name":"c"},{"id":4,"name":"d"},{"id":5,"name":"e"}]"#;
        let config = CompressionConfig::default();
        let result = OutputCompressor::compress(output, ContentType::JsonData, &config);
        assert!(result.content.contains("JSON Array"));
        assert!(result.content.contains("Elements: 5"));
    }

    #[test]
    fn test_compress_stacktrace_collapses_repeats() {
        let output = "thread 'main' panicked at 'index out of bounds'\nstack trace:\n  #0 foo::bar at src/foo.rs:10\n  #1 foo::bar at src/foo.rs:10\n  #2 foo::bar at src/foo.rs:10\n  #3 baz::qux at src/baz.rs:20\n  #4 main at src/main.rs:5";
        let config = CompressionConfig::default();
        let result = OutputCompressor::compress(output, ContentType::StackTrace, &config);
        assert!(result.content.contains("x3") || result.content.contains("duplicate"));
    }

    #[test]
    fn test_compress_plain_text_dedup() {
        let mut lines: Vec<String> = Vec::new();
        for _ in 0..20 {
            lines.push("INFO: connection established to server".to_string());
        }
        lines.push("INFO: unique event happened".to_string());
        let output = lines.join("\n");
        let config = CompressionConfig::default();
        let result = OutputCompressor::compress(output, ContentType::PlainText, &config);
        let occurrences = result.content.matches("connection established").count();
        assert!(occurrences <= 2, "Expected dedup, got {} occurrences", occurrences);
    }

    #[test]
    fn test_compress_code_preserves_signatures() {
        let output = r#"use std::io;

fn main() {
    let x = 1;
    let y = 2;
    println!("{}", x + y);
}

pub fn helper() -> String {
    "hello".to_string()
}

struct Foo {
    field: i32,
}
"#;
        let config = CompressionConfig::default();
        let result = OutputCompressor::compress(output, ContentType::Code, &config);
        assert!(result.content.contains("pub fn helper"));
        assert!(result.content.contains("struct Foo"));
        assert!(result.content.contains("use std::io"));
    }

    #[test]
    fn test_small_output_not_compressed() {
        let output = "short output";
        let config = CompressionConfig {
            max_tokens: 1000,
            ..Default::default()
        };
        let result = OutputCompressor::compress(output, ContentType::PlainText, &config);
        assert_eq!(result.ratio, 1.0);
        assert_eq!(result.content, output);
    }

    #[test]
    fn test_normalize_frame() {
        assert_eq!(normalize_frame("#0 foo::bar at src/foo.rs:10"), "#0 foo::bar at src/foo.rs:N");
        assert_eq!(normalize_frame("#1 main at src/main.rs:42"), "#1 main at src/main.rs:N");
    }

    #[test]
    fn test_lines_similar() {
        assert!(lines_similar("hello", "hello"));
        assert!(lines_similar("error: cannot find type", "error: cannot find tyep"));
        assert!(!lines_similar("error: type A", "warning: unused"));
    }
}
