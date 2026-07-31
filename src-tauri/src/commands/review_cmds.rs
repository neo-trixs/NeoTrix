use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReviewFile {
    pub path: String,
    pub additions: usize,
    pub deletions: usize,
    pub issues: Vec<ReviewIssue>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReviewIssue {
    pub line: usize,
    pub severity: String,
    pub category: String,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ReviewResult {
    pub pr_title: String,
    pub total_files: usize,
    pub total_issues: usize,
    pub critical: usize,
    pub warning: usize,
    pub info: usize,
    pub files: Vec<ReviewFile>,
    pub summary: String,
    pub score: u8,
}

fn severity_priority(s: &str) -> u8 {
    match s {
        "critical" => 3,
        "warning" => 2,
        "info" => 1,
        _ => 0,
    }
}

fn check_line_issues(line: &str, line_num: usize) -> Vec<ReviewIssue> {
    let mut issues = Vec::new();

    let trimmed = line.trim();

    // Hardcoded credentials
    if trimmed.contains("password") || trimmed.contains("api_key") || trimmed.contains("secret") {
        if trimmed.contains('=') || trimmed.contains(':') {
            issues.push(ReviewIssue {
                line: line_num,
                severity: "critical".into(),
                category: "security".into(),
                message: "Possible hardcoded credential".into(),
                suggestion: Some("Use environment variables or a secrets manager".into()),
            });
        }
    }

    // TODO/FIXME markers
    if trimmed.starts_with("// TODO") || trimmed.starts_with("// FIXME") || trimmed.starts_with("/* TODO") {
        issues.push(ReviewIssue {
            line: line_num,
            severity: "warning".into(),
            category: "maintainability".into(),
            message: format!("Unresolved {}", trimmed.split_whitespace().next().unwrap_or("marker")),
            suggestion: Some("Resolve before merge".into()),
        });
    }

    // Overly long lines
    if line.len() > 120 && !trimmed.starts_with("//") && !trimmed.starts_with("/*") && !trimmed.starts_with('*') {
        issues.push(ReviewIssue {
            line: line_num,
            severity: "info".into(),
            category: "style".into(),
            message: format!("Line too long ({} chars)", line.len()),
            suggestion: Some("Consider breaking into multiple lines (< 120 chars)".into()),
        });
    }

    // Debug print statements
    if trimmed.contains("println!") || trimmed.contains("dbg!") {
        issues.push(ReviewIssue {
            line: line_num,
            severity: "warning".into(),
            category: "debug".into(),
            message: "Debug print statement left in code".into(),
            suggestion: Some("Remove or replace with proper logging".into()),
        });
    }

    // Unwrapped unwrap/expect
    if trimmed.contains(".unwrap()") || trimmed.contains(".expect(") {
        issues.push(ReviewIssue {
            line: line_num,
            severity: "warning".into(),
            category: "error-handling".into(),
            message: "Use of unwrap/expect without error handling".into(),
            suggestion: Some("Use ? operator or proper error handling".into()),
        });
    }

    issues
}

fn parse_diff_file(diff_content: &str) -> Vec<(String, Vec<String>)> {
    let mut files = Vec::new();
    let mut current_file = String::new();
    let mut current_lines: Vec<String> = Vec::new();

    for line in diff_content.lines() {
        if line.starts_with("+++ b/") {
            if !current_file.is_empty() {
                files.push((std::mem::take(&mut current_file), std::mem::take(&mut current_lines)));
            }
            current_file = line[6..].to_string();
        } else if line.starts_with('+') && !line.starts_with("+++") {
            current_lines.push(line[1..].to_string());
        }
    }
    if !current_file.is_empty() {
        files.push((current_file, current_lines));
    }
    files
}

#[tauri::command]
pub fn review_diff(diff_content: String, pr_title: String) -> Result<ReviewResult, String> {
    let files = parse_diff_file(&diff_content);
    let mut review_files = Vec::new();
    let mut total_issues = 0;
    let mut critical = 0;
    let mut warning = 0;
    let mut info = 0;

    for (path, lines) in &files {
        let mut issues = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let line_issues = check_line_issues(line, i + 1);
            for issue in &line_issues {
                match issue.severity.as_str() {
                    "critical" => critical += 1,
                    "warning" => warning += 1,
                    _ => info += 1,
                }
            }
            total_issues += line_issues.len();
            issues.extend(line_issues);
        }
        review_files.push(ReviewFile {
            path: path.clone(),
            additions: lines.len(),
            deletions: 0,
            issues,
        });
    }

    let score = if total_issues == 0 {
        100
    } else {
        let raw = 100.0 - (critical as f64 * 15.0 + warning as f64 * 5.0 + info as f64 * 1.0);
        raw.max(0.0).min(100.0) as u8
    };

    let summary = format!(
        "Found {} issues ({} critical, {} warning, {} info) across {} files. Score: {}/100",
        total_issues, critical, warning, info, files.len(), score
    );

    Ok(ReviewResult {
        pr_title,
        total_files: files.len(),
        total_issues,
        critical,
        warning,
        info,
        files: review_files,
        summary,
        score,
    })
}

#[tauri::command]
pub fn review_get_issue_detail(file_path: String, line: usize) -> Result<ReviewIssue, String> {
    Ok(ReviewIssue {
        line,
        severity: "info".into(),
        category: "general".into(),
        message: format!("Reviewing {} at line {}", file_path, line),
        suggestion: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_diff_empty() {
        let files = parse_diff_file("");
        assert!(files.is_empty());
    }

    #[test]
    fn test_parse_diff_one_file() {
        let diff = "+++ b/src/main.rs\n+fn main() {}\n+let x = 1;\n";
        let files = parse_diff_file(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, "src/main.rs");
        assert_eq!(files[0].1.len(), 2);
    }

    #[test]
    fn test_check_line_password() {
        let issues = check_line_issues("let password = \"secret123\"", 5);
        assert!(issues.iter().any(|i| i.severity == "critical"));
    }

    #[test]
    fn test_check_line_todo() {
        let issues = check_line_issues("// TODO: fix this", 3);
        assert!(issues.iter().any(|i| i.severity == "warning"));
    }

    #[test]
    fn test_full_review() {
        let diff = "+++ b/src/lib.rs\n+fn main() {\n+    let password = \"hunter2\";\n+    // TODO: cleanup\n+    dbg!(\"test\");\n+    x.unwrap();\n+}\n";
        let result = review_diff(diff.into(), "Test PR".into()).unwrap();
        assert_eq!(result.total_files, 1);
        assert!(result.total_issues >= 4);
        assert!(result.score < 100);
    }

    #[test]
    fn test_empty_diff_100_score() {
        let result = review_diff("".into(), "Empty PR".into()).unwrap();
        assert_eq!(result.score, 100);
        assert_eq!(result.total_files, 0);
    }
}
