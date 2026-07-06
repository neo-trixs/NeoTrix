//! Rust compiler error/diagnostic parser.
//!
//! Parses `cargo check` / `rustc` structured output into
//! actionable `CompilerDiagnostic` records with file, line,
//! column, severity, error code, and message.

#[derive(Debug, Clone, PartialEq)]
pub struct CompilerDiagnostic {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub severity: DiagnosticSeverity,
    pub code: Option<String>,
    pub message: String,
    pub span_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Note,
    Help,
}

/// Parse full `cargo check` / `rustc` stderr output into diagnostics.
pub fn parse_compiler_output(output: &str) -> Vec<CompilerDiagnostic> {
    let mut diagnostics = Vec::new();
    let lines: Vec<&str> = output.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        // Pattern: error[E0425]: message
        // Pattern: warning[dead_code]: message
        if let Some(diag) = try_parse_diagnostic_header(line) {
            let (sev, code, msg) = diag;
            // Next line should be the location:   --> file.rs:42:17
            let (file, lineno, col, span_text) = if i + 1 < lines.len() {
                parse_location_line(lines[i + 1])
            } else {
                (String::new(), 0, 0, None)
            };
            diagnostics.push(CompilerDiagnostic {
                file,
                line: lineno,
                column: col,
                severity: sev,
                code,
                message: msg,
                span_text,
            });
        }
        i += 1;
    }
    diagnostics
}

fn try_parse_diagnostic_header(line: &str) -> Option<(DiagnosticSeverity, Option<String>, String)> {
    let trimmed = line.trim();
    // error[E0425]: cannot find value...
    if let Some(rest) = trimmed.strip_prefix("error") {
        let (code, msg) = extract_code_and_message(rest, "error");
        return Some((DiagnosticSeverity::Error, code, msg));
    }
    // warning[dead_code]: function is never used...
    if let Some(rest) = trimmed.strip_prefix("warning") {
        let (code, msg) = extract_code_and_message(rest, "warning");
        return Some((DiagnosticSeverity::Warning, code, msg));
    }
    // note: something
    if let Some(msg) = trimmed.strip_prefix("note:") {
        return Some((DiagnosticSeverity::Note, None, msg.trim().to_string()));
    }
    // help: something
    if let Some(msg) = trimmed.strip_prefix("help:") {
        return Some((DiagnosticSeverity::Help, None, msg.trim().to_string()));
    }
    None
}

fn extract_code_and_message(rest: &str, _kind: &str) -> (Option<String>, String) {
    let rest = rest.trim();
    if let Some(code_start) = rest.find('[') {
        if let Some(code_end) = rest.find(']') {
            let code = rest[code_start + 1..code_end].to_string();
            let after_bracket = rest[code_end + 1..].trim();
            let msg = after_bracket.strip_prefix(':')
                .unwrap_or(after_bracket)
                .trim()
                .to_string();
            return (Some(code), msg);
        }
    }
    if let Some(msg) = rest.strip_prefix(':') {
        return (None, msg.trim().to_string());
    }
    (None, rest.to_string())
}

fn parse_location_line(line: &str) -> (String, usize, usize, Option<String>) {
    let trimmed = line.trim();
    // Pattern: --> file.rs:42:17
    if let Some(loc) = trimmed.strip_prefix("-->") {
        let loc = loc.trim();
        if let Some(colon_pos) = loc.rfind(':') {
            let (path_and_col, col_str) = loc.split_at(colon_pos);
            let column: usize = col_str[1..].parse().unwrap_or(0);
            if let Some(second_colon) = path_and_col.rfind(':') {
                let (path, line_str) = path_and_col.split_at(second_colon);
                let lineno: usize = line_str[1..].parse().unwrap_or(0);
                return (path.trim().to_string(), lineno, column, None);
            }
        }
    }
    (String::new(), 0, 0, None)
}

/// Group diagnostics by file for targeted repair.
pub fn group_by_file(diagnostics: &[CompilerDiagnostic]) -> Vec<(String, Vec<&CompilerDiagnostic>)> {
    let mut groups: std::collections::HashMap<String, Vec<&CompilerDiagnostic>> =
        std::collections::HashMap::new();
    for d in diagnostics {
        if !d.file.is_empty() {
            groups.entry(d.file.clone()).or_default().push(d);
        }
    }
    let mut result: Vec<_> = groups.into_iter().collect();
    result.sort_by(|(a, _), (b, _)| a.cmp(b));
    result
}

/// Check if a diagnostic is fixable by automated means.
pub fn is_fixable(diag: &CompilerDiagnostic) -> bool {
    match diag.code.as_deref() {
        Some(code) => matches!(
            code,
            "E0425"   // cannot find value/function/type in scope — needs use import
            | "E0433"  // failed to resolve: use of undeclared type — needs use import
            | "E0412"  // cannot find type — needs use import
            | "E0063"  // missing fields — needs field addition
            | "E0308"  // mismatched types — type fix
            | "E0382"  // borrow of moved value — clone or reorder
            | "E0004"  // non-exhaustive patterns — add match arm
            | "E0560"  // struct has no field named — field rename
            | "E0599"  // no method named — method name fix
            | "E0428"  // name defined multiple times — remove duplicate
            | "dead_code"  // dead code warning
            | "unused_import"  // unused import
            | "unused_variable"  // unused variable
            | "unused_mut"  // unused mut
        ),
        None => diag.severity == DiagnosticSeverity::Warning,
    }
}

/// Group diagnostics by error code for batch fix strategy.
pub fn group_by_code(diagnostics: &[CompilerDiagnostic]) -> Vec<(Option<String>, Vec<&CompilerDiagnostic>)> {
    let mut groups: std::collections::BTreeMap<Option<String>, Vec<&CompilerDiagnostic>> =
        std::collections::BTreeMap::new();
    for d in diagnostics {
        groups.entry(d.code.clone()).or_default().push(d);
    }
    groups.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_with_code() {
        let output = "error[E0425]: cannot find value `x` in this scope\n  --> src/main.rs:42:17\n";
        let diags = parse_compiler_output(output);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, DiagnosticSeverity::Error);
        assert_eq!(diags[0].code.as_deref(), Some("E0425"));
        assert_eq!(diags[0].message, "cannot find value `x` in this scope");
        assert_eq!(diags[0].file, "src/main.rs");
        assert_eq!(diags[0].line, 42);
        assert_eq!(diags[0].column, 17);
    }

    #[test]
    fn test_parse_error_without_code() {
        let output = "error: expected identifier\n  --> src/lib.rs:10:5\n";
        let diags = parse_compiler_output(output);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, None);
        assert_eq!(diags[0].message, "expected identifier");
        assert_eq!(diags[0].line, 10);
    }

    #[test]
    fn test_parse_warning() {
        let output = "warning[dead_code]: function `foo` is never used\n  --> src/core/mod.rs:50:12\n";
        let diags = parse_compiler_output(output);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, DiagnosticSeverity::Warning);
        assert_eq!(diags[0].code.as_deref(), Some("dead_code"));
    }

    #[test]
    fn test_parse_multiple_diagnostics() {
        let output = "error[E0425]: cannot find value `x` in this scope\n  --> src/a.rs:1:1\nerror[E0433]: failed to resolve: use of undeclared type `Foo`\n  --> src/b.rs:2:3\n";
        let diags = parse_compiler_output(output);
        assert_eq!(diags.len(), 2);
        assert_eq!(diags[0].file, "src/a.rs");
        assert_eq!(diags[0].line, 1);
        assert_eq!(diags[1].file, "src/b.rs");
        assert_eq!(diags[1].line, 2);
    }

    #[test]
    fn test_parse_note_and_help() {
        let output = "error[E0308]: mismatched types\n  --> src/lib.rs:5:1\nnote: expected type `u32`\nhelp: try using a type conversion\n";
        let diags = parse_compiler_output(output);
        assert_eq!(diags.len(), 3);
        assert_eq!(diags[1].severity, DiagnosticSeverity::Note);
        assert_eq!(diags[2].severity, DiagnosticSeverity::Help);
    }

    #[test]
    fn test_group_by_file() {
        let output = "error[E0425]: val x not found\n  --> src/a.rs:1:1\nerror[E0425]: val y not found\n  --> src/a.rs:2:1\nerror[E0433]: type Foo not found\n  --> src/b.rs:1:1\n";
        let diags = parse_compiler_output(output);
        let groups = group_by_file(&diags);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].0, "src/a.rs");
        assert_eq!(groups[0].1.len(), 2);
        assert_eq!(groups[1].0, "src/b.rs");
        assert_eq!(groups[1].1.len(), 1);
    }

    #[test]
    fn test_is_fixable_codes() {
        let mut d = CompilerDiagnostic {
            file: "x.rs".into(), line: 1, column: 1,
            severity: DiagnosticSeverity::Error,
            code: Some("E0425".into()), message: "".into(), span_text: None,
        };
        assert!(is_fixable(&d));
        d.code = Some("E0433".into());
        assert!(is_fixable(&d));
        d.code = Some("unknown".into());
        assert!(!is_fixable(&d));
    }

    #[test]
    fn test_group_by_code() {
        let output = "error[E0425]: val x not found\n  --> src/a.rs:1:1\nerror[E0425]: val y not found\n  --> src/a.rs:2:1\nerror[E0433]: type Foo not found\n  --> src/b.rs:1:1\n";
        let diags = parse_compiler_output(output);
        let groups = group_by_code(&diags);
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_empty_output() {
        let diags = parse_compiler_output("");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_no_diagnostics() {
        let diags = parse_compiler_output("Compiling foo v0.1.0\nFinished dev profile\n");
        assert!(diags.is_empty());
    }
}
