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

/// 修复动作建议 — 基于错误码映射到可执行修复动作 (NT-REPAIR 经验库 cycle 246/248)。
///
/// 经验规则 (distill_errors.json error_fix_pairs, 2026-08-06 全量 49 万条实证):
///   1. unwrap/panic 崩溃 (E0308 类型不匹配 / 运行时 panic) → 首选 wrap() 包裹 (6699 次)
///   2. 缺失符号 (E0425/E0433/E0412) → 首选 add 补全 use/定义 (4491 次)
///   3. 借用检查错误 (E0382/E0505) → 首选 clone() 切断借用链
///   4. 非穷尽 match (E0004) → 补 match 分支
///   5. 字段缺失 (E0063) → 补字段
///   6. 方法不存在 (E0599) → 修正方法名
///   7. 重复定义 (E0428) → 移除重复
///   8. 未使用项 (dead_code/unused_import/unused_variable/unused_mut) → cargo fix 清理
///
/// OfficeCLI 模式 (2026-08-15 吸收): 错误码携带 suggestion + valid_range —
/// suggestion 给可立即执行的修复指令, valid_range 声明该建议的适用范围,
/// 使错误响应从"诊断"升级为"可自愈的操作单元"。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixSuggestion {
    /// 错误码 (如 "E0308"), 无码时为 "unknown"
    pub code: String,
    /// 修复动作类别 (wrap / add / clone / match_arm / field / rename / dedup / cleanup)
    pub action: &'static str,
    /// 人类可读的修复指引
    pub guidance: String,
    /// 可立即执行的修复指令 (命令/代码模板) — OfficeCLI suggestion 字段
    pub suggestion: Option<String>,
    /// 适用范围的合法值声明 — OfficeCLI valid_range 字段
    pub valid_range: Option<String>,
}

/// 根据错误码生成修复建议。返回 None 表示无已知自动修复策略。
pub fn suggest_fix(diag: &CompilerDiagnostic) -> Option<FixSuggestion> {
    let code = diag.code.as_deref()?;
    let (action, guidance, suggestion, valid_range) = match code {
        // 缺失符号 → add 补全 (经验: 缺失 16535 次, 首选 add 4491 次)
        "E0425" | "E0433" | "E0412" => (
            "add",
            "缺失符号: 补 use 导入或定义缺失的项 (经验: 缺失符号首选 add 补全)",
            Some("use <module>::<Item>; 或 补全缺失的 fn/struct/enum 定义".to_string()),
            Some("仅适用缺失符号类诊断 (E0425/E0433/E0412); 若符号确实不存在需补定义而非导入".to_string()),
        ),
        // 类型不匹配 → 对齐类型 / wrap 包裹
        "E0308" => (
            "type_fix",
            "类型不匹配: 对齐类型或用 as/into 转换; 若为 unwrap 崩溃点则用 wrap() 包裹",
            Some("将右侧值 as 转换, 或 .into() / .to_string() 对齐类型; 若在 unwrap 点改用 Result 处理".to_string()),
            Some("仅适用类型不匹配 (E0308); 需确认目标类型后选择 as/Into/From 之一".to_string()),
        ),
        // 借用/移动 → clone 切断借用链
        "E0382" | "E0505" => (
            "clone",
            "借用/移动错误: 首选 clone() 切断借用链 (经验: 借用检查错误首选 clone)",
            Some("在移动点调用 .clone() 保留原值; 或改用借用 & 引用".to_string()),
            Some("仅适用借用/移动错误 (E0382/E0505); 注意 clone 只适合小对象, 大结构建议改用借用".to_string()),
        ),
        // 非穷尽 match → 补分支
        "E0004" => (
            "match_arm",
            "非穷尽 match: 补全缺失的 match 分支",
            Some("为枚举补全缺失变体的 match 分支, 或用 `_ => {}` 兜底".to_string()),
            Some("仅适用非穷尽 match (E0004); 枚举新增变体需同步补全分支".to_string()),
        ),
        // 字段缺失 → 补字段
        "E0063" => (
            "field",
            "结构体字段缺失: 补全缺失字段初始化",
            Some("在结构体字面量中补全缺失字段: { .., <field>: <value> }".to_string()),
            Some("仅适用字段缺失 (E0063); 需按结构体定义补全所有必填字段".to_string()),
        ),
        // 方法不存在 → 修正方法名
        "E0599" => (
            "method",
            "方法不存在: 修正方法名或补 impl",
            Some("检查方法名拼写, 或在 impl 块中补充该方法定义".to_string()),
            Some("仅适用方法不存在 (E0599); 需确认方法属于当前类型且 trait 已导入".to_string()),
        ),
        // 重复定义 → 去重
        "E0428" => (
            "dedup",
            "重复定义: 移除重复的项定义",
            Some("删除同模块中重复的项定义, 或改用不同名称".to_string()),
            Some("仅适用重复定义 (E0428); 同名项在模块内只能定义一次".to_string()),
        ),
        // 未使用 → cargo fix 清理
        "dead_code" | "unused_import" | "unused_variable" | "unused_mut" => (
            "cleanup",
            "未使用项: 运行 cargo fix --lib --allow-dirty 自动清理",
            Some("cargo fix --lib --allow-dirty".to_string()),
            Some("仅适用未使用警告 (dead_code/unused_import/unused_variable/unused_mut); 若为公开 API 请加 #[allow] 而非删除".to_string()),
        ),
        _ => return None,
    };
    Some(FixSuggestion {
        code: code.to_string(),
        action,
        guidance: guidance.to_string(),
        suggestion,
        valid_range,
    })
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
    fn test_suggest_fix_missing_symbol_add() {
        let d = CompilerDiagnostic {
            file: "x.rs".into(), line: 1, column: 1,
            severity: DiagnosticSeverity::Error,
            code: Some("E0433".into()), message: "".into(), span_text: None,
        };
        let fix = suggest_fix(&d).expect("E0433 should have a fix");
        assert_eq!(fix.action, "add");
        assert!(fix.guidance.contains("add"));
    }

    #[test]
    fn test_suggest_fix_officecli_suggestion_and_range() {
        // OfficeCLI 模式 (2026-08-15): 错误码携带 suggestion + valid_range
        let d = CompilerDiagnostic {
            file: "x.rs".into(), line: 1, column: 1,
            severity: DiagnosticSeverity::Error,
            code: Some("E0425".into()), message: "".into(), span_text: None,
        };
        let fix = suggest_fix(&d).expect("E0425 should have a fix");
        let sug = fix.suggestion.as_deref().expect("suggestion present");
        assert!(sug.contains("use "));
        let range = fix.valid_range.as_deref().expect("valid_range present");
        assert!(range.contains("E0425"));
    }

    #[test]
    fn test_suggest_fix_cleanup_cmd() {
        let d = CompilerDiagnostic {
            file: "x.rs".into(), line: 1, column: 1,
            severity: DiagnosticSeverity::Warning,
            code: Some("unused_import".into()), message: "".into(), span_text: None,
        };
        let fix = suggest_fix(&d).expect("unused_import should have a fix");
        assert_eq!(fix.action, "cleanup");
        assert_eq!(fix.suggestion.as_deref(), Some("cargo fix --lib --allow-dirty"));
    }

    #[test]
    fn test_suggest_fix_unwrap_type_wrap() {
        let d = CompilerDiagnostic {
            file: "x.rs".into(), line: 1, column: 1,
            severity: DiagnosticSeverity::Error,
            code: Some("E0308".into()), message: "".into(), span_text: None,
        };
        let fix = suggest_fix(&d).expect("E0308 should have a fix");
        assert_eq!(fix.action, "type_fix");
        assert!(fix.guidance.contains("wrap"));
    }

    #[test]
    fn test_suggest_fix_borrow_clone() {
        let d = CompilerDiagnostic {
            file: "x.rs".into(), line: 1, column: 1,
            severity: DiagnosticSeverity::Error,
            code: Some("E0382".into()), message: "".into(), span_text: None,
        };
        let fix = suggest_fix(&d).expect("E0382 should have a fix");
        assert_eq!(fix.action, "clone");
        assert!(fix.guidance.contains("clone"));
    }

    #[test]
    fn test_suggest_fix_unknown_none() {
        let d = CompilerDiagnostic {
            file: "x.rs".into(), line: 1, column: 1,
            severity: DiagnosticSeverity::Error,
            code: Some("E9999".into()), message: "".into(), span_text: None,
        };
        assert!(suggest_fix(&d).is_none());
    }

    #[test]
    fn test_suggest_fix_no_code_none() {
        let d = CompilerDiagnostic {
            file: "x.rs".into(), line: 1, column: 1,
            severity: DiagnosticSeverity::Error,
            code: None, message: "".into(), span_text: None,
        };
        assert!(suggest_fix(&d).is_none());
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
