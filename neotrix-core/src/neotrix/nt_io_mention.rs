use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

/// Result of resolving a @-mention file reference in user input.
#[derive(Debug, Clone)]
pub struct MentionResult {
    pub path: PathBuf,
    pub content: String,
    pub lines: usize,
    pub truncated: bool,
}

const MAX_MENTIONS: usize = 5;
const MAX_LINES_PER_FILE: usize = 200;
const MAX_TOTAL_LINES: usize = 1000;

/// Parse `@path` references in `input`, resolve against `cwd`, read file contents,
/// and inject them into the text. Returns the modified text and metadata for each mention.
///
/// Smart matching rules:
/// - `@file.rs` → exact file match in cwd
/// - `@src/main.rs` → relative path
/// - `@./src/lib.rs` → explicit relative
/// - `@~/file.rs` → home directory
/// - Ignores `@` inside fenced code blocks (`` ``` ``)
/// - Ignores `@` in URL contexts (e.g. `https://`)
/// - Ignores `@` preceded by a word character (email addresses)
pub fn resolve_mentions(input: &str, cwd: &Path) -> (String, Vec<MentionResult>) {
    // Note: avoid look-around which the regex crate does not support.
    // We manually filter non-mention @-preceded-by-word-char cases below.
    let re = Regex::new(r"@([\w./~:-]+)").unwrap();
    let code_regions = find_code_blocks(input);

    let mut results: Vec<MentionResult> = Vec::new();
    let mut total_lines = 0usize;
    let mut pending: Vec<(usize, usize, String, MentionResult)> = Vec::new();

    for m in re.find_iter(input) {
        if pending.len() >= MAX_MENTIONS {
            break;
        }

        let start = m.start();
        let end = m.end();

        if is_inside_region(start, &code_regions) {
            continue;
        }
        // Skip if preceded by a word character (email addresses like user@domain)
        if start > 0 && input.as_bytes()[start - 1].is_ascii_alphanumeric() {
            continue;
        }
        // Skip if preceded by :// (URL fragment like https://)
        if start >= 3 && &input[start - 3..start] == "://" {
            continue;
        }

        let raw = &m.as_str()[1..];
        let resolved = resolve_path(raw, cwd);
        let pb = match resolved {
            Some(p) if p.is_file() => p,
            _ => continue,
        };

        let (content, lines, truncated) = match read_file_limited(&pb, MAX_LINES_PER_FILE) {
            Ok(t) => t,
            Err(_) => continue,
        };

        let new_total = total_lines + lines;
        let (content, lines, truncated) = if new_total > MAX_TOTAL_LINES {
            let available = MAX_TOTAL_LINES.saturating_sub(total_lines);
            let (c, l, _) = truncate_content(&content, available);
            (c, l, true)
        } else {
            (content, lines, truncated)
        };

        let file_ext = pb.extension().and_then(|e| e.to_str()).unwrap_or("");
        let replacement = format!(
            "\n[📎 {}]\n```{}\n{}\n```\n",
            pb.display(),
            file_ext,
            content,
        );

        pending.push((
            start,
            end,
            replacement,
            MentionResult {
                path: pb,
                content,
                lines,
                truncated,
            },
        ));
        total_lines += lines;
    }

    // Apply replacements in reverse position order to keep byte offsets valid
    pending.sort_by(|a, b| b.0.cmp(&a.0));
    let mut modified = input.to_string();
    for (start, end, repl, result) in pending {
        modified.replace_range(start..end, &repl);
        results.push(result);
    }
    results.reverse();

    (modified, results)
}

/// Find byte-offset ranges of fenced code blocks (```...```).
fn find_code_blocks(input: &str) -> Vec<(usize, usize)> {
    let mut regions = Vec::new();
    let mut pos = 0;
    while let Some(start) = input[pos..].find("```") {
        let abs_start = pos + start;
        let after = abs_start + 3;
        if let Some(end) = input[after..].find("```") {
            let abs_end = after + end + 3;
            regions.push((abs_start, abs_end));
            pos = abs_end;
        } else {
            break;
        }
    }
    regions
}

fn is_inside_region(pos: usize, regions: &[(usize, usize)]) -> bool {
    regions.iter().any(|&(s, e)| pos >= s && pos < e)
}

/// Resolve a path string against cwd, handling `~` and absolute paths.
fn resolve_path(raw: &str, cwd: &Path) -> Option<PathBuf> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    if s.starts_with('~') {
        let expanded = shellexpand::tilde(s);
        Some(PathBuf::from(expanded.as_ref()))
    } else if s.starts_with('/') {
        Some(PathBuf::from(s))
    } else {
        Some(cwd.join(s))
    }
}

/// Read a file, returning (content, line_count, truncated_flag).
fn read_file_limited(path: &Path, max_lines: usize) -> std::io::Result<(String, usize, bool)> {
    let content = fs::read_to_string(path)?;
    let total = content.lines().count();
    if total <= max_lines {
        Ok((content, total, false))
    } else {
        let truncated: String = content
            .lines()
            .take(max_lines)
            .collect::<Vec<_>>()
            .join("\n");
        Ok((truncated + "\n[... truncated]", max_lines, true))
    }
}

/// Further truncate content to a maximum number of lines.
fn truncate_content(content: &str, max_lines: usize) -> (String, usize, bool) {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() <= max_lines {
        (content.to_string(), lines.len(), false)
    } else {
        let truncated = lines[..max_lines].join("\n");
        (truncated + "\n[... truncated]", max_lines, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_dir() -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("neotrix_mention_test_{id}"));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_mention_resolves_exact_file() {
        let dir = test_dir();
        fs::write(dir.join("hello.rs"), "fn hello() {\n    println!(\"hi\");\n}\n").unwrap();
        let input = "check @hello.rs for style";
        let (modified, results) = resolve_mentions(input, &dir);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path.file_name().unwrap(), "hello.rs");
        assert!(modified.contains("hello.rs"));
        assert!(modified.contains("fn hello()"));
        cleanup(&dir);
    }

    #[test]
    fn test_mention_resolves_relative_path() {
        let dir = test_dir();
        let sub = dir.join("src");
        let _ = fs::create_dir_all(&sub);
        fs::write(sub.join("lib.rs"), "pub fn main() {}").unwrap();
        let input = "check @src/lib.rs";
        let (modified, results) = resolve_mentions(input, &dir);
        assert_eq!(results.len(), 1);
        assert!(modified.contains("pub fn main()"));
        cleanup(&dir);
    }

    #[test]
    fn test_mention_resolves_explicit_relative() {
        let dir = test_dir();
        fs::write(dir.join("mod.rs"), "pub mod test;").unwrap();
        let input = "see @./mod.rs";
        let (modified, results) = resolve_mentions(input, &dir);
        assert_eq!(results.len(), 1);
        assert!(modified.contains("pub mod test"));
        cleanup(&dir);
    }

    #[test]
    fn test_mention_ignores_email() {
        let dir = test_dir();
        let input = "contact me at user@example.com";
        let (modified, results) = resolve_mentions(input, &dir);
        assert!(results.is_empty());
        assert_eq!(modified, input);
        cleanup(&dir);
    }

    #[test]
    fn test_mention_ignores_code_block() {
        let dir = test_dir();
        let input = "```\n@foo.rs\n```";
        let (modified, results) = resolve_mentions(input, &dir);
        assert!(results.is_empty());
        assert_eq!(modified, input);
        cleanup(&dir);
    }

    #[test]
    fn test_mention_ignores_url() {
        let dir = test_dir();
        let input = "see https://example.com/@user";
        let (modified, results) = resolve_mentions(input, &dir);
        assert!(results.is_empty());
        assert_eq!(modified, input);
        cleanup(&dir);
    }

    #[test]
    fn test_mention_max_files() {
        let dir = test_dir();
        for i in 0..8 {
            fs::write(dir.join(format!("f{i}.rs")), "fn f() {}\n").unwrap();
        }
        let input = (0..8).map(|i| format!("@f{i}.rs")).collect::<Vec<_>>().join(" ");
        let (_, results) = resolve_mentions(&input, &dir);
        assert_eq!(results.len(), MAX_MENTIONS);
        cleanup(&dir);
    }

    #[test]
    fn test_mention_max_lines_per_file() {
        let dir = test_dir();
        let content: Vec<String> = (0..300).map(|i| format!("line {}", i)).collect();
        fs::write(dir.join("big.rs"), content.join("\n")).unwrap();
        let input = "check @big.rs";
        let (_, results) = resolve_mentions(input, &dir);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].lines, MAX_LINES_PER_FILE);
        assert!(results[0].truncated);
        assert!(results[0].content.contains("[... truncated]"));
        cleanup(&dir);
    }

    #[test]
    fn test_mention_total_line_budget() {
        let dir = test_dir();
        for i in 0..3 {
            let content: Vec<String> = (0..400).map(|j| format!("f{i} line {j}")).collect();
            fs::write(dir.join(format!("big{i}.rs")), content.join("\n")).unwrap();
        }
        let input = "@big0.rs @big1.rs @big2.rs";
        let (_, results) = resolve_mentions(input, &dir);
        assert_eq!(results.len(), 3);
        let total: usize = results.iter().map(|r| r.lines).sum();
        assert!(total <= MAX_TOTAL_LINES);
        cleanup(&dir);
    }

    #[test]
    fn test_mention_nonexistent_file_skipped() {
        let dir = test_dir();
        let input = "check @nonexistent.rs and @also_missing.py";
        let (modified, results) = resolve_mentions(input, &dir);
        assert!(results.is_empty());
        assert_eq!(modified, input);
        cleanup(&dir);
    }

    #[test]
    fn test_mention_not_preceded_by_word_char() {
        let dir = test_dir();
        fs::write(dir.join("foo.rs"), "content").unwrap();
        let input = "check @foo.rs";
        let (modified, results) = resolve_mentions(input, &dir);
        assert_eq!(results.len(), 1);
        assert!(modified.contains("content"));
        cleanup(&dir);
    }

    #[test]
    fn test_mention_resolves_tilde_path() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let dir = PathBuf::from(&home);
        fs::write(dir.join("tilde_test.rs"), "// tilde file").unwrap();
        let input = "check @~/tilde_test.rs";
        let (modified, results) = resolve_mentions(input, &dir);
        assert_eq!(results.len(), 1);
        assert!(modified.contains("tilde file"));
        let _ = fs::remove_file(dir.join("tilde_test.rs"));
    }
}
