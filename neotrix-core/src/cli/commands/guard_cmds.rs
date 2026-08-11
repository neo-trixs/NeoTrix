//! AGENTS.md Pointer-Conservation Guard
//!
//! Rust-native replacement for the former .opencode/plugins/agents-guard.js.
//! Enforces the HARD RULE: AGENTS.md is a pure guidance document with fixed ceilings.
//! No per-cycle growth areas (Experience Index) allowed — cycle pointers live in KB only.

use std::fs;
use std::path::Path;

/// AGENTS.md 结构守卫配置（与原 agents-guard.js 保持一致）
const MAX_LINES: usize = 130;
const MAX_BYTES: usize = 22_000;

const ALLOWED_SECTIONS: &[&str] = &[
    "Skill Routing",
    "Architecture",
    "Always-On Core Rules",
    "Shared Language",
    "Build",
    "Test",
    "Key Locations",
];

const FORBIDDEN_SECTIONS: &[&str] = &["Experience Index"];

/// 守卫检查结果
#[derive(Debug, Clone)]
pub struct GuardResult {
    pub ok: bool,
    pub violations: Vec<String>,
    pub lines: usize,
    pub bytes: usize,
    pub sections: Vec<String>,
}

impl GuardResult {
    fn new() -> Self {
        Self {
            ok: true,
            violations: Vec::new(),
            lines: 0,
            bytes: 0,
            sections: Vec::new(),
        }
    }

    fn add_violation(&mut self, msg: String) {
        self.ok = false;
        self.violations.push(msg);
    }
}

/// 执行 AGENTS.md 结构守卫检查
///
/// # Arguments
/// * `path` - AGENTS.md 文件路径（默认当前目录）
/// * `strict` - 严格模式：违规时返回非零退出码
pub fn run_guard(path: &Path, strict: bool) -> Result<GuardResult, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let mut result = GuardResult::new();
    result.bytes = content.len();
    // 忽略末尾单个换行, 与 wc -l 语义一致
    result.lines = content.trim_end_matches('\n').split('\n').count();

    // 1. Total line ceiling
    if result.lines > MAX_LINES {
        result.add_violation(format!(
            "AGENTS.md exceeded {} lines (now {}). AGENTS.md is guidance-only; cycle content must go to KB, never here.",
            MAX_LINES, result.lines
        ));
    }

    // 2. Total byte ceiling
    if result.bytes > MAX_BYTES {
        result.add_violation(format!(
            "AGENTS.md exceeded {} bytes. Violation of pointer-conservation HARD RULE.",
            MAX_BYTES
        ));
    }

    // 3. Section whitelist
    let h2_sections: Vec<String> = content
        .lines()
        .filter(|l| l.starts_with("## "))
        .map(|l| l.trim_start_matches("## ").trim().to_string())
        .collect();
    result.sections = h2_sections.clone();

    let unknown: Vec<&String> = h2_sections
        .iter()
        .filter(|s| !ALLOWED_SECTIONS.contains(&s.as_str()))
        .collect();
    if !unknown.is_empty() {
        let unknown_str: Vec<&str> = unknown.iter().map(|s| s.as_str()).collect();
        result.add_violation(format!(
            "AGENTS.md contains non-whitelisted sections: {}. Allowed: {}.",
            unknown_str.join(", "),
            ALLOWED_SECTIONS.join(", ")
        ));
    }

    // 4. Explicitly forbidden growth areas
    let growth: Vec<&String> = h2_sections
        .iter()
        .filter(|s| FORBIDDEN_SECTIONS.contains(&s.as_str()))
        .collect();
    if !growth.is_empty() {
        let growth_str: Vec<&str> = growth.iter().map(|s| s.as_str()).collect();
        result.add_violation(format!(
            "AGENTS.md contains forbidden per-cycle growth area: {}. Cycle pointers live in KB (neotrix-experience hub/query), never in AGENTS.md.",
            growth_str.join(", ")
        ));
    }

// 5. No inline Experience Index table
    if content.contains("## Experience Index")
        || content.contains("| Cycle | Domain | Summary |")
        || content.contains("| Cycle | Date |")
        || content.contains("| Cycle | Session |")
    {
        result.add_violation(
            "AGENTS.md inlines an Experience Index table. Cycle pointers live in the KB experience hub, never in AGENTS.md."
                .to_string(),
        );
    }

    // Output
    if result.ok {
        println!("✅ AGENTS.md pointer-conservation check passed");
        println!("   Lines: {}/{} | Bytes: {}/{} | Sections: {}", result.lines, MAX_LINES, result.bytes, MAX_BYTES, result.sections.len());
    } else {
        eprintln!("❌ AGENTS.md pointer-conservation violated:");
        for v in &result.violations {
            eprintln!("   - {}", v);
        }
        eprintln!("   Inspect ~/.neotrix/agents-guard-violations.log and revert AGENTS.md to HEAD if needed.");
    }

    if strict && !result.ok {
        return Err("AGENTS.md guard failed".to_string());
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_temp(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f.flush().unwrap();
        f
    }

    #[test]
    fn test_guard_passes_valid_agents() {
        let content = r#"# NeoTrix — AI-Native Developer Toolkit

## Skill Routing
| Task | Load |
|------|------|

## Architecture
Some text.

## Always-On Core Rules
- R-P1: forbid unsafe

## Shared Language
Terms.

## Build
cargo build

## Test
cargo test

## Key Locations
Paths.
"#;
        let f = write_temp(content);
        let res = run_guard(f.path(), false).unwrap();
        assert!(res.ok, "valid AGENTS.md should pass: {:?}", res.violations);
    }

    #[test]
    fn test_guard_fails_line_ceiling() {
        let content = "# Title\n".to_string() + &"## Section\n".repeat(140);
        let f = write_temp(&content);
        let res = run_guard(f.path(), false).unwrap();
        assert!(!res.ok);
        assert!(res.violations.iter().any(|v| v.contains("exceeded 130 lines")));
    }

    #[test]
    fn test_guard_fails_byte_ceiling() {
        let content = "# Title\n".to_string() + &"x".repeat(23_000);
        let f = write_temp(&content);
        let res = run_guard(f.path(), false).unwrap();
        assert!(!res.ok);
        assert!(res.violations.iter().any(|v| v.contains("exceeded 22000 bytes")));
    }

    #[test]
    fn test_guard_fails_unknown_section() {
        let content = r#"# Title
## Skill Routing
## Unknown Section
"#;
        let f = write_temp(content);
        let res = run_guard(f.path(), false).unwrap();
        assert!(!res.ok);
        assert!(res.violations.iter().any(|v| v.contains("non-whitelisted sections")));
    }

    #[test]
    fn test_guard_fails_experience_index_section() {
        let content = r#"# Title
## Skill Routing
## Experience Index
| Cycle | Domain | Summary |
"#;
        let f = write_temp(content);
        let res = run_guard(f.path(), false).unwrap();
        assert!(!res.ok);
        assert!(res.violations.iter().any(|v| v.contains("forbidden per-cycle growth area")));
    }

    #[test]
    fn test_guard_fails_inline_experience_table() {
        let content = r#"# Title
## Skill Routing
| Cycle | Domain | Summary |
| 1 | NT-CORE | test |
"#;
        let f = write_temp(content);
        let res = run_guard(f.path(), false).unwrap();
        assert!(!res.ok);
        assert!(res.violations.iter().any(|v| v.contains("Experience Index table")));
    }
}