//! Code review command — staged / unstaged / file / all

use std::fs;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_act_code::code_review_pipeline::{
    CodeReviewPipeline, DiffParser, ReviewCmdConfig,
};
use crate::neotrix::nt_mind::SelfIteratingBrain;

pub struct ReviewCmd;

impl CliCommand for ReviewCmd {
    fn name(&self) -> &str {
        "/review"
    }
    fn aliases(&self) -> Vec<&str> {
        vec!["/r", "/code-review", "review"]
    }
    fn description(&self) -> &str {
        "Code review: staged|unstaged|file <path>|all [--nt_shield] [--full] [--rule <categories>]"
    }
    fn execute(
        &self,
        args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let subcmd = args.first().map(|s| s.as_str()).unwrap_or("staged");
        let full = args.contains(&"--full".to_string());
        let rule_idx = args.iter().position(|a| a == "--rule" || a == "-r");
        let rule_categories = rule_idx
            .and_then(|i| args.get(i + 1))
            .map(|v| {
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let config = ReviewCmdConfig { rule_categories };
        match subcmd {
            "staged" => self.review_staged(),
            "unstaged" => self.review_unstaged(),
            "file" => {
                let path = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if path.is_empty() { return CommandOutput::err("Usage: /review file <path>"); }
                self.review_file(path)
            }
            "all" => self.review_all(full, &config),
            "help" => CommandOutput::ok("Usage: /review staged|unstaged|file <path>|all [--nt_shield] [--full] [--rule <categories>]"),
            _ => CommandOutput::err("Usage: /review staged|unstaged|file <path>|all [--nt_shield] [--full] [--rule <categories>]"),
        }
    }
}

impl ReviewCmd {
    fn review_staged(&self) -> CommandOutput {
        let output = Command::new("git")
            .args(["diff", "--cached", "--no-color"])
            .output();
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                if !stderr.is_empty() {
                    return CommandOutput::warn(&format!("git error: {}", stderr.trim()));
                }
                if stdout.is_empty() {
                    return CommandOutput::ok("No staged changes to review");
                }
                let review = self.analyze_content(&stdout, "Staged Changes");
                CommandOutput::ok(&review)
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    CommandOutput::err("git is not installed")
                } else {
                    CommandOutput::err(&format!("git execution failed: {}", e))
                }
            }
        }
    }

    fn review_unstaged(&self) -> CommandOutput {
        let output = Command::new("git").args(["diff", "--no-color"]).output();
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                if !stderr.is_empty() {
                    return CommandOutput::warn(&format!("git error: {}", stderr.trim()));
                }
                if stdout.is_empty() {
                    return CommandOutput::ok("No unstaged changes to review");
                }
                let review = self.analyze_content(&stdout, "Unstaged Changes");
                CommandOutput::ok(&review)
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    CommandOutput::err("git is not installed")
                } else {
                    CommandOutput::err(&format!("git execution failed: {}", e))
                }
            }
        }
    }

    fn review_file(&self, path: &str) -> CommandOutput {
        match fs::read_to_string(path) {
            Ok(contents) => {
                let line_count = contents.lines().count();
                let review = self.analyze_content(&contents, &format!("File: {}", path));
                let header = format!("📄 Reviewing `{}` ({} lines)\n\n", path, line_count);
                CommandOutput::ok(&format!("{}{}", header, review))
            }
            Err(e) => CommandOutput::err(&format!("Failed to read {}: {}", path, e)),
        }
    }

    fn review_all(&self, full: bool, _config: &ReviewCmdConfig) -> CommandOutput {
        let staged = Command::new("git")
            .args(["diff", "--cached", "--no-color"])
            .output();
        let unstaged = Command::new("git").args(["diff", "--no-color"]).output();

        let mut combined = String::new();

        match staged {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                if !stdout.is_empty() {
                    combined.push_str("=== STAGED ===\n");
                    combined.push_str(&stdout);
                    combined.push('\n');
                }
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::NotFound {
                    return CommandOutput::err(&format!("git execution failed: {}", e));
                }
                return CommandOutput::err("git is not installed");
            }
        }

        match unstaged {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                if !stdout.is_empty() {
                    combined.push_str("=== UNSTAGED ===\n");
                    combined.push_str(&stdout);
                    combined.push('\n');
                }
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::NotFound {
                    return CommandOutput::err(&format!("git execution failed: {}", e));
                }
                return CommandOutput::err("git is not installed");
            }
        }

        if combined.is_empty() {
            return CommandOutput::ok("No changes to review");
        }

        if full {
            return self.review_with_pipeline(&combined);
        }

        let review = self.analyze_content(&combined, "All Changes");
        CommandOutput::ok(&review)
    }

    fn review_with_pipeline(&self, diff_text: &str) -> CommandOutput {
        let diffs = DiffParser::new().parse_diff(diff_text);
        if diffs.is_empty() {
            return CommandOutput::ok("No parseable diff changes to review");
        }
        let pipeline = CodeReviewPipeline::new();
        let result = pipeline.run_deterministic_review(&diffs);
        if result.comments.is_empty() {
            return CommandOutput::ok("✅ Pipeline review: no issues found");
        }
        let mut output = String::new();
        output.push_str(&format!("## 🔍 Pipeline Code Review\n\n"));
        output.push_str(&format!(
            "Files: {} | Comments: {}\n\n",
            result.file_count, result.comment_count
        ));
        for comment in &result.comments {
            let sev = match comment.severity {
                crate::neotrix::nt_act_code::code_review_pipeline::IssueSeverity::Critical => {
                    "CRITICAL"
                }
                crate::neotrix::nt_act_code::code_review_pipeline::IssueSeverity::High => "HIGH",
                crate::neotrix::nt_act_code::code_review_pipeline::IssueSeverity::Medium => {
                    "MEDIUM"
                }
                crate::neotrix::nt_act_code::code_review_pipeline::IssueSeverity::Low => "LOW",
                crate::neotrix::nt_act_code::code_review_pipeline::IssueSeverity::Info => "INFO",
            };
            let line = comment
                .start_line
                .map(|l| format!(" L{}", l))
                .unwrap_or_default();
            output.push_str(&format!(
                "{} [{:?}] {}{}: {}\n",
                sev, comment.category, comment.file, line, comment.message
            ));
            if let Some(s) = &comment.suggestion {
                output.push_str(&format!("   → {}\n", s));
            }
        }
        output.push_str(&format!(
            "\n📊 {} warnings, {} errors\n",
            result.warning_count, result.error_count
        ));
        CommandOutput::ok(&output)
    }

    fn analyze_content(&self, content: &str, label: &str) -> String {
        let mut nt_shield_issues: Vec<String> = Vec::new();
        let mut quality_issues: Vec<String> = Vec::new();
        let mut style_issues: Vec<String> = Vec::new();

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let line_is_code = |idx: usize| -> bool {
            let trimmed = lines[idx].trim();
            if trimmed.is_empty()
                || trimmed.starts_with("//")
                || trimmed.starts_with('#')
                || trimmed.starts_with("/*")
                || trimmed.starts_with('*')
            {
                return false;
            }
            true
        };

        // --- Security checks ---
        let nt_shield_patterns: &[(&str, &str)] = &[
            (
                r#"(?i)(api[_\-]?key|secret|token|password|passwd)\s*[:=]\s*(['"]?[A-Za-z0-9_\-]{16,})"#,
                "Possible hardcoded secret/key",
            ),
            (r"(?i)AKIA[0-9A-Z]{16}", "Possible AWS Access Key"),
            (r"(?i)sk-[a-zA-Z0-9_\-]{32,}", "Possible OpenAI API key"),
            (
                r"(?i)(ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9_]{36}",
                "Possible GitHub token",
            ),
            (
                r"\bunsafe\b",
                "Usage of `unsafe` block \u{2014} verify safety invariants",
            ),
            (
                r"format!\s*\(\s*[^)]*\b(String|str)\b.*\)",
                "Potential format string injection risk",
            ),
            (
                r"(?i)cmd\.(Run|Exec|Output|CombinedOutput).*\$\{",
                "Possible command injection via shell expansion",
            ),
        ];

        for (pattern, desc) in nt_shield_patterns {
            let re = match regex::Regex::new(pattern) {
                Ok(r) => r,
                Err(_) => continue,
            };
            for (i, line) in lines.iter().enumerate() {
                if re.is_match(line) {
                    nt_shield_issues.push(format!("  L{}: {} — {}", i + 1, line.trim(), desc));
                }
            }
        }

        // --- Quality checks ---
        let mut func_lines: Vec<(usize, usize)> = Vec::new();
        let mut func_start: Option<usize> = None;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("fn ")
                && trimmed.contains('(')
                && trimmed.contains(')')
                && trimmed.contains('{')
            {
                if let Some(start) = func_start {
                    let _ = start;
                }
                func_start = Some(i);
            }
            if let Some(start) = func_start {
                if trimmed == "}" {
                    let size = i - start;
                    if size > 1 {
                        func_lines.push((start, size));
                    }
                    func_start = None;
                }
            }
        }
        if let Some(start) = func_start {
            if total_lines - start > 1 {
                func_lines.push((start, total_lines - start));
            }
        }

        for &(start, size) in &func_lines {
            if size > 50 {
                quality_issues.push(format!(
                    "  L{}: Large function ({} lines, threshold 50) — consider refactoring",
                    start + 1,
                    size
                ));
            }
        }

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.ends_with(".unwrap()") || trimmed.ends_with(".unwrap();") {
                quality_issues.push(format!(
                    "  L{}: `.unwrap()` — prefer proper error handling with `?` or match",
                    i + 1
                ));
            }
        }

        // Check for deep nesting
        let mut max_indent = 0;
        for line in &lines {
            let leading_spaces = line.chars().take_while(|c| *c == ' ').count();
            if leading_spaces > max_indent {
                max_indent = leading_spaces;
            }
        }
        if max_indent > 40 {
            quality_issues.push(format!(
                "  Deep nesting detected (max indent: {} spaces) — consider flattening",
                max_indent
            ));
        }

        // --- Style checks ---
        for (i, line) in lines.iter().enumerate() {
            if line.len() > 100 && line_is_code(i) {
                style_issues.push(format!(
                    "  L{}: Line too long ({} chars > 100)",
                    i + 1,
                    line.len()
                ));
            }
        }

        // Check for commented-out code
        let mut commented_lines = 0;
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("// ")
                && (trimmed.contains("fn ")
                    || trimmed.contains("struct ")
                    || trimmed.contains("impl ")
                    || trimmed.contains("if ")
                    || trimmed.contains("for ")
                    || trimmed.contains("while "))
            {
                commented_lines += 1;
            }
        }
        if commented_lines > 3 {
            style_issues.push(format!(
                "  Found {} lines of commented-out code — clean up before commit",
                commented_lines
            ));
        }

        // --- Build output ---
        let mut output = String::new();
        output.push_str(&format!("## 🔍 Code Review: {}\n\n", label));

        // Security section
        output.push_str("### 🔴 Security\n");
        if nt_shield_issues.is_empty() {
            output.push_str("  No nt_shield issues detected.\n");
        } else {
            for issue in &nt_shield_issues {
                output.push_str(issue);
                output.push('\n');
            }
        }
        output.push('\n');

        // Quality section
        output.push_str("### 🟡 Code Quality\n");
        if quality_issues.is_empty() {
            output.push_str("  No quality issues detected.\n");
        } else {
            for issue in &quality_issues {
                output.push_str(issue);
                output.push('\n');
            }
        }
        output.push('\n');

        // Style section
        output.push_str("### 🔵 Style\n");
        if style_issues.is_empty() {
            output.push_str("  No style issues detected.\n");
        } else {
            for issue in &style_issues {
                output.push_str(issue);
                output.push('\n');
            }
        }
        output.push('\n');

        // Summary
        let total_issues = nt_shield_issues.len() + quality_issues.len() + style_issues.len();
        output.push_str(&format!("### 📊 Summary\n"));
        output.push_str(&format!("  Total lines reviewed: {}\n", total_lines));
        output.push_str(&format!("  Security issues: {}\n", nt_shield_issues.len()));
        output.push_str(&format!("  Quality issues: {}\n", quality_issues.len()));
        output.push_str(&format!("  Style issues: {}\n", style_issues.len()));

        let assessment = if total_issues == 0 {
            "**Overall: Looks good — no issues found.**"
        } else if total_issues <= 3 {
            "**Overall: Minor issues — consider addressing them.**"
        } else {
            "**Overall: Needs attention — review and fix issues before committing.**"
        };
        output.push_str(&format!("  {}\n", assessment));

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_cmd_name() {
        let cmd = ReviewCmd;
        assert_eq!(cmd.name(), "/review");
    }

    #[test]
    fn test_review_cmd_aliases() {
        let cmd = ReviewCmd;
        assert!(cmd.aliases().contains(&"/r"));
        assert!(cmd.aliases().contains(&"/code-review"));
    }

    #[test]
    fn test_review_file_not_found() {
        let cmd = ReviewCmd;
        let result = cmd.review_file("/nonexistent/path.rs");
        assert!(!result.success);
    }

    #[test]
    fn test_analyze_content_no_issues() {
        let cmd = ReviewCmd;
        let result = cmd.analyze_content("fn hello() {\n    println!(\"hi\");\n}\n", "test");
        assert!(result.contains("No nt_shield issues"));
        assert!(result.contains("No quality issues"));
        assert!(result.contains("No style issues"));
        assert!(result.contains("Looks good"));
    }

    #[test]
    fn test_analyze_content_detects_unwrap() {
        let cmd = ReviewCmd;
        let result = cmd.analyze_content("let x = foo().unwrap();\n", "test");
        assert!(result.contains(".unwrap()"));
        assert!(
            result.contains("looks good")
                || result.contains("Minor")
                || result.contains("Needs attention")
        );
    }

    #[test]
    fn test_analyze_content_detects_api_key() {
        let cmd = ReviewCmd;
        let result = cmd.analyze_content(
            "api_key = \"sk-abcdefghijklmnopqrstuvwxyz1234567890\"\n",
            "test",
        );
        assert!(result.contains("API key"));
    }

    #[test]
    fn test_analyze_content_detects_long_lines() {
        let cmd = ReviewCmd;
        let long_line = "x".repeat(150);
        let result = cmd.analyze_content(&format!("fn test() {{\n    {}\n}}\n", long_line), "test");
        assert!(result.contains("Line too long"));
    }

    #[test]
    fn test_execute_help() {
        let cmd = ReviewCmd;
        let result = cmd.execute(&["help".to_string()], None);
        assert!(result.success);
        assert!(result.message.contains("Usage"));
    }

    #[test]
    fn test_execute_file_no_path() {
        let cmd = ReviewCmd;
        let result = cmd.execute(&["file".to_string()], None);
        assert!(!result.success);
        assert!(result.message.contains("Usage"));
    }

    #[test]
    fn test_execute_unknown_subcmd() {
        let cmd = ReviewCmd;
        let result = cmd.execute(&["invalid".to_string()], None);
        assert!(!result.success);
    }
}
