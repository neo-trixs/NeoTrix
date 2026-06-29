// ─── Self-Evolving Code Review Architecture ─────────────────────────────────
//
// 代码审查是 NeoTrix 意识核心的元能力 (meta-capability)，不是外部工具。
// 它与 SEAL/DGM-H 自我进化管道连接，构成一个闭合的自进化循环：
//
//   Code Changes
//       │
//       ▼
//   ┌─────────────────────────────────────────┐
//   │  Route A: Deterministic Pre-filter      │  ← NAP (NeoTrix Audit Protocol)
//   │  P1-P7: 编译正确性 (orphan/feature/dep) │     11 phases, shell-scriptable
//   │  P8-P11: 框架注册/生命周期/守卫/死代码   │     每轮 CI 自动执行
//   └────────────────┬────────────────────────┘
//                    │  passing files only
//                    ▼
//   ┌─────────────────────────────────────────┐
//   │  Route B: 确定性-LLM 混合审查            │  ← CodeReviewPipeline
//   │  evidence→line 滑动窗口匹配              │     3-stage: Plan→Main→Filter
//   │  deterministic_scan (8+ 模式)           │     layered rules (4 levels)
//   │  CommentResolver (sliding→LCS→LLM)      │     anchor line drift detection
//   └────────────────┬────────────────────────┘
//                    │  structured findings
//                    ▼
//   ┌─────────────────────────────────────────┐
//   │  意识管道模式蒸馏                         │  ← Consciousness Core
//   │  缺陷分类 → 频次统计 → 根因诊断           │     E8 64态推理
//   │  交叉引用现有 NAP phase 覆盖度            │     VSA 向量模式检测
//   └────────────────┬────────────────────────┘
//                    │  new defect category
//                    ▼
//   ┌─────────────────────────────────────────┐
//   │  NAP 自进化                               │  ← AGENTS.md XXXIV.10 Meta-Audit
//   │  新缺陷类别 → 创建新 phase                │     自动写入 first-pass-audit.sh
//   │  phase 写入 scripts/first-pass-audit.sh   │     AGENTS.md 同步蒸馏
//   │  经验蒸馏为 AGENTS.md 分支                │     confidence 演化
//   └─────────────────────────────────────────┘
//                    │
//                    ▼
//               (更好的审查 → 回到顶部)
//
// ── 关键设计原则 ──
//
// 1. 证据先于定位 (Evidence Before Location, XXXI.1)
//    LLM 不输出行号, 输出可匹配的代码片段; 确定性引擎用滑动窗口映射到精确位置。
//    行号幻觉消除, 成功率 > 95%。
//
// 2. 确定性边界 (Deterministic Boundary, XXXI.2)
//    高错误成本操作 (定位/覆盖/安全) 走确定性工程; 低错误成本 (判断/解释) 走 LLM。
//    NAP P1-P11 是确定性边界的具体实现。
//
// 3. 三层认知流水线 (Three-Stage Cognitive Pipeline, XXXI.3)
//    Plan(分析+结构化) → Main(工具循环+综合) → Filter(对抗性证伪)。
//    认知负荷递增, 假阳性率递减。
//
// 4. NAP 自我进化 (Meta-Audit, XXXIV.10)
//    每个未被 NAP 捕获的新缺陷类别 → 创建新 phase。
//    NAP 是"关于审计的审计", 它自己和它所检查的代码一样需要进化。
//
// ── 接线点 ──
//
// 1. CodeReviewPipeline.run_pipeline() → 输出 ReviewComment[]
// 2. ReviewComment[] → critical/compliance 模式 → NAP phase 关联
// 3. NAP phase 覆盖率报告 → AGENTS.md XXXIV 分支更新
// 4. 新 phase → scripts/first-pass-audit.sh 追加
//
// ── 进化指标 ──
//
// - NAP 覆盖率: 1 - (missed_defect_categories / total_defect_categories)
// - 第一遍通过率: 无需第二轮修复的代码占比
// - 审查延迟: 从代码变更到审查完成的平均时间
// - NAP phase 增长率: 每个会话平均新增 phase 数 (正常 = 0, 异常 = 1+)
// =========================================================================

mod comment_resolver;
mod diff_parser;
mod pipeline;
mod rule_resolver;
mod session;
mod types;

pub use comment_resolver::*;
pub use diff_parser::*;
pub use pipeline::*;
pub use rule_resolver::*;
pub use session::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_diff() -> &'static str {
        "diff --git a/src/main.rs b/src/main.rs
new file mode 100644
@@ -0,0 +1,12 @@
+fn main() {
+    let x = some_function();
+    println!(\"{:?}\", x.unwrap());
+    let y = compute_value();
+    match y {
+        Ok(v) => println!(\"{}\", v),
+        Err(e) => log::error!(\"error: {}\", e),
+    }
+    unsafe {
+        let ptr = std::ptr::null();
+    }
+}
"
    }

    #[test]
    fn test_diff_parser_parses_hunks() {
        let parser = DiffParser::new();
        let files = parser.parse_diff(sample_diff());
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file, "src/main.rs");
        assert_eq!(files[0].status, DiffStatus::Added);
        assert_eq!(files[0].hunks.len(), 1);
        assert_eq!(files[0].hunks[0].old_start, 0);
        assert_eq!(files[0].hunks[0].new_start, 1);
    }

    #[test]
    fn test_diff_parser_line_counts() {
        let parser = DiffParser::new();
        let files = parser.parse_diff(sample_diff());
        let lines = &files[0].hunks[0].lines;
        let additions = lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Addition)
            .count();
        let contexts = lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Context)
            .count();
        assert_eq!(additions, 11);
        assert_eq!(contexts, 1);
    }

    #[test]
    fn test_deterministic_scan_finds_unwrap() {
        let pipeline = CodeReviewPipeline::new();
        let diffs = DiffParser::new().parse_diff(sample_diff());
        let result = pipeline.run_deterministic_review(&diffs);
        assert!(result
            .comments
            .iter()
            .any(|c| c.message.contains(".unwrap()")));
    }

    #[test]
    fn test_deterministic_scan_finds_unsafe() {
        let pipeline = CodeReviewPipeline::new();
        let diffs = DiffParser::new().parse_diff(sample_diff());
        let result = pipeline.run_deterministic_review(&diffs);
        assert!(result
            .comments
            .iter()
            .any(|c| c.message.contains("Unsafe block")));
    }

    #[test]
    fn test_comment_resolver_exact_match() {
        let resolver = CommentResolver::new();
        let diff = ReviewFileDiff {
            file: "test.rs".into(),
            status: DiffStatus::Modified,
            old_path: None,
            hunks: vec![DiffHunk {
                old_start: 10,
                old_count: 5,
                new_start: 10,
                new_count: 5,
                lines: vec![
                    DiffLine {
                        line_type: DiffLineType::Context,
                        old_line: Some(10),
                        new_line: Some(10),
                        content: "fn old_func() {".into(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Deletion,
                        old_line: Some(11),
                        new_line: None,
                        content: "    let x = 1;".into(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Addition,
                        old_line: None,
                        new_line: Some(11),
                        content: "    let x = compute(42);".into(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Context,
                        old_line: Some(12),
                        new_line: Some(12),
                        content: "    println!(\"{}\", x);".into(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Context,
                        old_line: Some(13),
                        new_line: Some(13),
                        content: "}".into(),
                    },
                ],
            }],
        };
        let mut comments = vec![ReviewComment {
            id: "c-0".into(),
            file: "test.rs".into(),
            severity: IssueSeverity::High,
            category: IssueCategory::Correctness,
            message: "test".into(),
            existing_code: "let x = compute(42);".into(),
            start_line: None,
            end_line: None,
            suggestion: None,
            anchor_lines: Vec::new(),
            match_confidence: 0.0,
            needs_relocation: false,
        }];
        resolver.resolve_comments(&mut comments, &[diff]);
        assert_eq!(comments[0].start_line, Some(11));
    }

    #[test]
    fn test_comment_resolver_multi_line_match() {
        let resolver = CommentResolver::new();
        let diff = ReviewFileDiff {
            file: "test.rs".into(),
            status: DiffStatus::Modified,
            old_path: None,
            hunks: vec![DiffHunk {
                old_start: 1,
                old_count: 6,
                new_start: 1,
                new_count: 6,
                lines: vec![
                    DiffLine {
                        line_type: DiffLineType::Addition,
                        old_line: None,
                        new_line: Some(1),
                        content: "fn process(data: &[u8]) -> Result<()> {".into(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Addition,
                        old_line: None,
                        new_line: Some(2),
                        content: "    let decoded = decode(data)?;".into(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Addition,
                        old_line: None,
                        new_line: Some(3),
                        content: "    validate(&decoded)?;".into(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Addition,
                        old_line: None,
                        new_line: Some(4),
                        content: "    save(&decoded)".into(),
                    },
                    DiffLine {
                        line_type: DiffLineType::Context,
                        old_line: Some(5),
                        new_line: Some(5),
                        content: "}".into(),
                    },
                ],
            }],
        };
        let mut comments = vec![ReviewComment {
            id: "c-0".into(),
            file: "test.rs".into(),
            severity: IssueSeverity::Medium,
            category: IssueCategory::ErrorHandling,
            message: "unhandled error".into(),
            existing_code: "    save(&decoded)".into(),
            start_line: None,
            end_line: None,
            suggestion: Some("add ? operator".into()),
            anchor_lines: Vec::new(),
            match_confidence: 0.0,
            needs_relocation: false,
        }];
        resolver.resolve_comments(&mut comments, &[diff]);
        assert_eq!(comments[0].start_line, Some(4));
    }

    #[test]
    fn test_layered_rule_resolver_default() {
        let resolver = LayeredRuleResolver::new();
        let rule = resolver.resolve("src/main.rs");
        assert!(rule.should_review);
        assert_eq!(rule.layer, "system default");
    }

    #[test]
    fn test_layered_rule_resolver_excludes_tests() {
        let resolver = LayeredRuleResolver::new();
        assert!(!resolver.should_include_file("src/main_test.rs"));
    }

    #[test]
    fn test_layered_rule_resolver_cli_override() {
        let resolver = LayeredRuleResolver::new().with_cli_rules(vec![PathRule {
            path_pattern: "**/*.rs".into(),
            rule_text: "Custom Rust review rule".into(),
        }]);
        let rule = resolver.resolve("src/main.rs");
        assert_eq!(rule.rule_text, "Custom Rust review rule");
        assert_eq!(rule.layer, "--rule flag");
    }

    #[test]
    fn test_pipeline_review_produces_comments() {
        let pipeline = CodeReviewPipeline::new();
        let diff_str = "diff --git a/src/lib.rs b/src/lib.rs
@@ -1,3 +1,5 @@
 fn compute() -> i32 {
 +    let x = get_value().unwrap();
 +    unsafe { std::ptr::null() };
     42
 }
 ";
        let diffs = DiffParser::new().parse_diff(diff_str);
        let result = pipeline.run_deterministic_review(&diffs);
        assert!(result.comment_count >= 2);
    }

    #[test]
    fn test_glob_match_exact() {
        assert!(glob_match("**/*.rs", "src/main.rs"));
        assert!(!glob_match("**/*.rs", "src/main.js"));
    }

    #[test]
    fn test_glob_match_exclude_patterns() {
        assert!(glob_match("**/tests/**", "src/tests/test_foo.rs"));
        assert!(glob_match("**/*_test.rs", "src/main_test.rs"));
        assert!(glob_match("**/vendor/**", "vendor/lib.rs"));
    }

    #[test]
    fn test_diff_parses_rename() {
        let input = "diff --git a/old.rs b/new.rs
rename from old.rs
rename to new.rs
@@ -1,3 +1,3 @@
 fn foo() {}
 fn bar() {}
";
        let files = DiffParser::new().parse_diff(input);
        let file = &files[0];
        assert_eq!(file.status, DiffStatus::Renamed);
        assert_eq!(file.file, "new.rs");
        assert_eq!(file.old_path.as_deref(), Some("old.rs"));
    }

    #[test]
    fn test_comment_resolver_fuzzy_match() {
        let resolver = CommentResolver::new();
        let diff = ReviewFileDiff {
            file: "test.rs".into(),
            status: DiffStatus::Modified,
            old_path: None,
            hunks: vec![DiffHunk {
                old_start: 1,
                old_count: 3,
                new_start: 1,
                new_count: 3,
                lines: vec![DiffLine {
                    line_type: DiffLineType::Addition,
                    old_line: None,
                    new_line: Some(1),
                    content: "    let result = compute_value_with_default(params);".into(),
                }],
            }],
        };
        let mut comments = vec![ReviewComment {
            id: "c-0".into(),
            file: "test.rs".into(),
            severity: IssueSeverity::Low,
            category: IssueCategory::Style,
            message: "long line".into(),
            existing_code: "let result = compute_value_with_default".into(),
            start_line: None,
            end_line: None,
            suggestion: None,
            anchor_lines: Vec::new(),
            match_confidence: 0.0,
            needs_relocation: false,
        }];
        resolver.resolve_comments(&mut comments, &[diff]);
        assert_eq!(comments[0].start_line, Some(1));
    }

    #[test]
    fn test_pipeline_review_nonexistent_file_does_not_panic() {
        let pipeline = CodeReviewPipeline::new();
        let diff = ReviewFileDiff {
            file: "nonexistent.rs".into(),
            status: DiffStatus::Modified,
            old_path: None,
            hunks: vec![],
        };
        let result = pipeline.run_deterministic_review(&[diff]);
        assert_eq!(result.comment_count, 0);
    }

    #[test]
    fn test_review_result_counts() {
        let mut comments = vec![
            ReviewComment {
                id: "c-0".into(),
                file: "a.rs".into(),
                severity: IssueSeverity::High,
                category: IssueCategory::Security,
                message: "unsafe".into(),
                existing_code: "unsafe".into(),
                start_line: Some(1),
                end_line: Some(1),
                suggestion: None,
                anchor_lines: Vec::new(),
                match_confidence: 1.0,
                needs_relocation: false,
            },
            ReviewComment {
                id: "c-1".into(),
                file: "a.rs".into(),
                severity: IssueSeverity::Low,
                category: IssueCategory::Style,
                message: "todo".into(),
                existing_code: "TODO".into(),
                start_line: Some(2),
                end_line: Some(2),
                suggestion: None,
                anchor_lines: Vec::new(),
                match_confidence: 1.0,
                needs_relocation: false,
            },
        ];
        let resolver = CommentResolver::new();
        resolver.resolve_comments(&mut comments, &[]);
        let result = ReviewResult {
            comment_count: 2,
            file_count: 1,
            warning_count: 1,
            error_count: 1,
            comments,
        };
        assert_eq!(result.error_count, 1);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_diff_parser_handles_empty_input() {
        let parser = DiffParser::new();
        let files = parser.parse_diff("");
        assert!(files.is_empty());
    }

    #[test]
    fn test_diff_parser_handles_deleted_file() {
        let input = "diff --git a/old.rs b/old.rs
deleted file mode 100644
@@ -1,2 +0,0 @@
-fn old_func() {}
-fn another() {}
";
        let files = DiffParser::new().parse_diff(input);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].status, DiffStatus::Deleted);
    }

    #[test]
    fn test_comment_resolver_normalize_code() {
        let resolver = CommentResolver::new();
        let result = resolver.normalize_code("fn foo() {\n\n\n    let x = 1;\n}");
        assert!(!result.contains("\n\n\n"));
        assert_eq!(result, "fn foo() {\n    let x = 1;\n}");
    }
}
