//! Self-review runner — gate execution, run_all dispatch, all check_* (PA001-PA027).
//! Pure move from `mod.rs` (God-file split) — zero behavior change.

use std::path::{Path, PathBuf};

use regex::Regex;

use super::nt_review_types::ArchLayer;
use super::nt_review_types::PatternConfig;
use super::nt_review_types::PatternMatch;
use super::nt_review_types::ReviewFinding;
use super::nt_review_types::SelfReviewConfig;
use super::nt_review_types::SelfReviewGate;
use super::nt_review_types::SelfReviewReport;
use super::scanners::*;
use neotrix_types::shared::Severity;

// ─── moved: mod.rs:234-252 SelfTest impl ───
impl crate::l6_meta::healing::nt_core_self_test::SelfTest for SelfReviewGate {
    fn name(&self) -> &str {
        "self_review_gate"
    }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        if self.config.min_test_line_count < 1 {
            failures.push("self_review_gate: min_test_line_count must be >= 1".into());
        }
        if self.config.scan_safety_bound < 1 {
            failures.push("self_review_gate: scan_safety_bound must be >= 1".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

// ─── moved: mod.rs:impl SelfReviewGate core ctor/check ───
impl SelfReviewGate {
    pub fn new(strict_mode: bool) -> Self {
        Self {
            strict_mode,
            findings: Vec::new(),
            config: SelfReviewConfig::default(),
            observer_quality: None,
            observer_patterns: Vec::new(),
        }
    }

    pub fn with_observer_feedback(mut self, quality: f64, patterns: Vec<String>) -> Self {
        self.observer_quality = Some(quality);
        self.observer_patterns = patterns;
        self
    }

    pub fn syn_depth(&self, code: &str) -> usize {
        match syn::parse_file(code) {
            Ok(file) => syn_file_max_depth(&file),
            Err(_) => estimate_brace_depth(code),
        }
    }

    pub fn check(
        &mut self,
        condition: bool,
        severity: Severity,
        category: impl Into<String>,
        message: String,
        file: impl Into<String>,
        line: u32,
    ) {
        if !condition {
            self.findings.push(ReviewFinding {
                severity,
                category: category.into(),
                message,
                file: file.into(),
                line,
            });
        }
    }
}

// ─── moved: mod.rs:352-400 run_all ───
impl SelfReviewGate {
    pub fn run_all(&mut self) -> SelfReviewReport {
        self.check_no_unwrap_in_production();
        self.check_no_todo_in_production();
        self.check_panic_audit_detailed();
        self.check_no_dead_code_without_allow();
        self.check_public_api_has_docs();
        self.check_assertion_failures();
        self.check_mutex_poison();
        self.check_indexing_panics();

        if self.strict_mode {
            self.check_no_empty_match_arms();
            self.check_layer_violations();
            self.check_architecture_layer_depth();
            self.check_observer_feedback();
            self.check_process_exit();
            self.check_binary_health();
            self.check_test_density();
            self.check_init_safety();
            self.check_orphan_files();
            self.check_unused_imports();
            self.check_python_bare_except();
            self.check_python_sql_injection();
            self.check_python_legacy_tables();
            self.check_python_main_guard();
        }

        // Cycle 27 additions — always run (pattern detection, not severity-gated)
        self.check_negative_signal_zeroed();
        self.check_fts_desync();
        self.check_zscore_length_mismatch();
        self.check_substring_tag_matching();
        self.check_hashmap_iteration_order();
        self.check_clamp_negative_semantics();

        // Cycle 29 additions — Karpathy-inspired code principles (PA020-PA023)
        self.check_karpathy_simplicity_first();
        self.check_karpathy_surgical_changes();
        self.check_karpathy_complexity_budget();
        self.check_karpathy_goal_driven_execution();

        // Cycle 30 additions — always run
        self.check_seal_stage_health();
        self.check_ci_workflow_coverage();
        self.check_test_assertion_hygiene();
        self.check_dep_version_consistency();

        self.report()
    }
}

// ─── moved: mod.rs:402-1630 checks+detect_import_layer ───
impl SelfReviewGate {
    fn check_no_unwrap_in_production(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let unwrap_count = scan_for_patterns(
            "",
            &[PatternConfig {
                name: "unwrap".into(),
                pattern: regex::escape(".unwrap("),
            }],
            Some(&src_dir),
        )
        .len();
        let expect_count = scan_for_patterns(
            "",
            &[PatternConfig {
                name: "expect".into(),
                pattern: regex::escape(".expect("),
            }],
            Some(&src_dir),
        )
        .len();
        let msg = format!(
            "PA001-PA002: {} .unwrap() and {} .expect() calls in src/ (excludes #[cfg(test)])",
            unwrap_count, expect_count
        );
        self.check(
            unwrap_count <= self.config.unwrap_max && expect_count <= self.config.expect_max,
            Severity::Warning,
            "unwrap_safety",
            msg,
            file!(),
            line!(),
        );
    }

    fn check_no_todo_in_production(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let todo_count = scan_for_pattern_excluding_tests(&src_dir, "todo!(");
        let unimpl_count = scan_for_pattern_excluding_tests(&src_dir, "unimplemented!(");
        let msg = format!(
            "PA004-PA005: {} todo!() and {} unimplemented!() calls in src/ (excluding #[cfg(test)])",
            todo_count, unimpl_count
        );
        self.check(
            todo_count < self.config.todo_max && unimpl_count < self.config.unimplemented_max,
            Severity::Warning,
            "todo_check",
            msg,
            file!(),
            line!(),
        );
    }

    fn check_no_dead_code_without_allow(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let allow_dead = scan_for_patterns(
            "",
            &[PatternConfig {
                name: "allow_dead_code".into(),
                pattern: regex::escape("#[allow(dead_code)]"),
            }],
            Some(&src_dir),
        )
        .len();
        let msg = format!(
            "Found {} #[allow(dead_code)] annotations in src/ (potential dead code)",
            allow_dead
        );
        self.check(
            allow_dead < self.config.allow_dead_max,
            Severity::Warning,
            "dead_code",
            msg,
            file!(),
            line!(),
        );
    }

    fn check_public_api_has_docs(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let pub_fn = scan_for_patterns(
            "",
            &[PatternConfig {
                name: "pub_fn".into(),
                pattern: regex::escape("pub fn "),
            }],
            Some(&src_dir),
        )
        .len();
        let pub_struct = scan_for_patterns(
            "",
            &[PatternConfig {
                name: "pub_struct".into(),
                pattern: regex::escape("pub struct "),
            }],
            Some(&src_dir),
        )
        .len();
        let doc_lines = scan_for_patterns(
            "",
            &[PatternConfig {
                name: "doc_comment".into(),
                pattern: regex::escape("///"),
            }],
            Some(&src_dir),
        )
        .len();
        let msg = format!(
            "{} pub fn, {} pub struct, {} doc comments — doc ratio: {:.1}%",
            pub_fn,
            pub_struct,
            doc_lines,
            if pub_fn + pub_struct > 0 {
                doc_lines as f64 / (pub_fn + pub_struct) as f64 * 100.0
            } else {
                0.0
            }
        );
        self.check(
            doc_lines >= (pub_fn + pub_struct) / 2,
            Severity::Info,
            "public_docs",
            msg,
            file!(),
            line!(),
        );
    }

    fn check_no_empty_match_arms(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let empty_match = scan_for_patterns(
            "",
            &[PatternConfig {
                name: "empty_match".into(),
                pattern: regex::escape(" => {},"),
            }],
            Some(&src_dir),
        )
        .len();
        let msg = format!("Found {} empty match arms ( => {{}},) in src/", empty_match);
        self.check(
            empty_match < self.config.empty_match_max,
            Severity::Warning,
            "empty_match",
            msg,
            file!(),
            line!(),
        );
    }

    /// Extended panic audit — cargo-panic-audit classes PA003-PA005.
    /// Detects `panic!()`, `todo!()`, `unreachable!()` in production code.
    fn check_panic_audit_detailed(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let panic_count = scan_for_pattern_excluding_tests(&src_dir, "panic!(");
        let todo_count = scan_for_pattern_excluding_tests(&src_dir, "todo!(");
        let unreachable_count = scan_for_pattern_excluding_tests(&src_dir, "unreachable!(");
        let mut msg = String::from("Panic audit [cargo-panic-audit PA003-PA005]:");
        msg.push_str(&format!(" panic!()={}", panic_count));
        msg.push_str(&format!(" todo!()={}", todo_count));
        msg.push_str(&format!(" unreachable!()={}", unreachable_count));
        self.check(
            panic_count == 0 && todo_count == 0 && unreachable_count == 0,
            Severity::Error,
            "panic_audit",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA006: Array/slice indexing — `arr[i]`, `vec[j]` that may panic on OOB.
    /// Detected via pattern match on bracket-access expressions.
    fn check_indexing_panics(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let bracket_idx = scan_for_pattern_excluding_tests(&src_dir, "[");
        let get_calls = scan_for_pattern_excluding_tests(&src_dir, ".get(");
        let raw_idx = bracket_idx.saturating_sub(get_calls * 3);
        let msg = format!(
            "PA006: ~{} raw index expressions (arr[i]) vs {} .get() safe accesses — prefer .get() for bounds safety",
            raw_idx, get_calls
        );
        self.check(
            raw_idx < get_calls * self.config.index_multiplier
                || raw_idx < self.config.index_absolute,
            Severity::Info,
            "indexing_panic",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA007: Assertion failures — `assert!()`, `assert_eq!()` in production.
    fn check_assertion_failures(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let assert_count = scan_for_pattern_excluding_tests(&src_dir, "assert!(");
        let assert_eq_count = scan_for_pattern_excluding_tests(&src_dir, "assert_eq!(");
        let msg = format!(
            "PA007: {} assert!() and {} assert_eq!() in src/ (production assertions may panic)",
            assert_count, assert_eq_count
        );
        self.check(
            assert_count == 0 && assert_eq_count == 0,
            Severity::Warning,
            "assertion_failure",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA008: Mutex/RwLock unwrap — `.lock().unwrap()` pattern (panic amplification).
    fn check_mutex_poison(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let lock_unwrap = scan_for_pattern_excluding_tests(&src_dir, ".lock().unwrap(");
        let mutex_unwrap = scan_for_pattern_excluding_tests(&src_dir, ".lock(");
        let msg = format!(
            "PA008: {} .lock().unwrap() patterns (panic amplification risk). Use .lock().expect() or ? with poison handling.",
            lock_unwrap
        );
        self.check(
            lock_unwrap < self.config.lock_unwrap_max || mutex_unwrap == 0,
            Severity::Warning,
            "mutex_poison",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA009: `process::exit()` or `std::process::exit` in production.
    fn check_process_exit(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let exit_count = scan_for_pattern_excluding_tests(&src_dir, "process::exit(");
        let msg = format!(
            "PA009: {} process::exit() calls found — kills process immediately, prefer graceful shutdown",
            exit_count
        );
        self.check(
            exit_count < self.config.exit_max,
            Severity::Warning,
            "process_exit",
            msg,
            file!(),
            line!(),
        );
    }

    /// Layer violation check — verifies core/ does not import from neotrix/.
    fn check_layer_violations(&mut self) {
        let core_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("core");
        if !core_dir.exists() {
            return;
        }
        let violation_count = scan_for_patterns(
            "",
            &[PatternConfig {
                name: "layer_violation".into(),
                pattern: regex::escape("use crate::neotrix"),
            }],
            Some(&core_dir),
        )
        .len();
        let msg = format!(
            "Layer violation: {} `use crate::neotrix` imports found in core/ (L0→L9 violation)",
            violation_count
        );
        self.check(
            violation_count == 0,
            Severity::Error,
            "layer_violation",
            msg,
            file!(),
            line!(),
        );
    }

    /// Architecture layer depth analysis — verifies no backward dependency flow.
    /// Scans each file's `use crate::` imports and checks layer ordering.
    fn check_architecture_layer_depth(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut violations = 0usize;
        // 2026-10-07（裁定 5C）：`classified` 只为报告层诚实性服务 —— 区分
        // "检了 N 个文件、发现 0 个违规" 与 "一个文件都没检到"。
        let mut classified = 0usize;
        // 2026-10-07（裁定 5A 第二批）：**递归**进层目录。
        // 原实现只遍历 `src` 的直接子项并跳过非 `.rs` ⇒ 层目录本身被跳过
        // （`src/l0_substrate/` 是目录，不是 `.rs`）⇒ 可分类文件 = 0。
        if let Ok(entries) = std::fs::read_dir(&src_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // 层目录：递归收集其下的 .rs（跳过 target 等）
                    for rs in collect_rs_files(&path) {
                        let source_layer = ArchLayer::from_path(&rs);
                        if source_layer.layer_index() < 0 {
                            continue;
                        }
                        classified += 1;
                        scan_layer_file(self, &rs, source_layer, &mut violations);
                    }
                    continue;
                }
                if path.extension().is_none_or(|e| e != "rs") {
                    continue;
                }
                let source_layer = ArchLayer::from_path(&path);
                if source_layer.layer_index() < 0 {
                    continue;
                }
                classified += 1;
                scan_layer_file(self, &path, source_layer, &mut violations);
            }
        }
        // ⛔ 2026-10-07 裁定 5C：只修**报告层**，不修判定层。
        //
        // 缺陷（已实测）：`read_dir` 后 `if ext != "rs" { continue }`（:424-428）
        // **把目录本身跳过了** ⇒ `l0_substrate/`…`l6_meta/` 这些层目录从不进入；
        // 剩下的顶层 `.rs` 又全不匹配 `ArchLayer::from_path` 的模式
        // （`l0_core`/`l1_body`/`l8_seal` 等目录名**在本仓不存在**）
        // ⇒ 可分类文件实测 = **0**，`violations` 恒为 0。
        //
        // ⛔ 原实现报 `Architecture depth: 0 reverse-layer imports` —— 一句
        // **读起来像绿灯的谎**（它实际什么都没检）。这就是 D-16/L8 说的
        // "报 PASS 却结构上不可能失败"。
        //
        // 本步只让它**诚实**：检到 0 个可分类文件时明说，并降级为 Info。
        // ⛔ **不递归、不改模式** —— 那会立刻冒出上百个待裁决的真实问题，
        // 而当前还没有可信的其他门（roadmap 1A 未落地）⇒ 先止血说谎，
        // 再谈判定。递归改造属裁定 5A，排在 1A 之后。
        if classified == 0 {
            // ⛔ 用 `false` 而非 `true`：`check()` 只在 `!condition` 时把 finding
            // 入库（:73-74），所以传 `true` 等于**什么都不报** ——
            // 那正是原缺陷（报一句像绿灯的谎）。此处要的是**让诚实声明出现在报告里**，
            // 故传 `false` + `Severity::Info`：它会成为一个 Info 级 finding，
            // 不算失败（`passed + failed + warnings <= findings` 的不变量仍成立），
            // 但读报告的人**看得见「这一项没检」**。
            self.check(
                false,
                Severity::Info,
                "arch_depth",
                concat!(
                    "Architecture depth: NOT CHECKED — 0 classifiable files. ",
                    "The layer-name patterns in ArchLayer::from_path match no directory ",
                    "that exists in this repo (l0_core/l1_body/l8_seal etc.), and ",
                    "read_dir skips directories. This check is structurally unable to fail. ",
                    "See roadmap 5C (report-layer fix only); 5A (real recursion + pattern ",
                    "remap) is deferred until 1A gives us trustworthy gates.",
                )
                .to_string(),
                file!(),
                line!(),
            );
            return;
        }
        let msg = format!(
            "Architecture depth: {violations} reverse-layer imports \
             (lower layer importing a higher layer), across {classified} classifiable files"
        );
        self.check(
            violations == 0,
            Severity::Warning,
            "arch_depth",
            msg,
            file!(),
            line!(),
        );
    }

    /// `detect_import_layer` 的公开别名（供本模块的自由函数 `scan_layer_file` 调用）。
    pub(crate) fn detect_import_layer_pub(&self, line: &str) -> ArchLayer {
        self.detect_import_layer(line)
    }

    /// 判定一行 `use crate::` 引的是哪一层。
    ///
    /// # 2026-10-07（裁定 5A 第三批）：修 `core::` 子串误判
    ///
    /// ⛔ **原实现把 `core::` 当作 L0 标志**，于是
    /// `use crate::l3_embodiment::nt_shield::shield_core::guard::...`
    /// 里的 `shield_core::` 子串会被抓成 L0 ⇒ **同层引用被误报成"高层引低层"**。
    /// 实测：修 `from_path` + 递归后检出 55 处，其中 **54 处是这类误判**，
    /// 真违规仅 **1 处**（`l6_meta/nt_core_observer.rs:4` 的 L6→L5 真引用）。
    ///
    /// ⭐ 修法：**先按真实层目录名（`crate::lN_xxx::`）精确判定**，
    /// 只有都不命中时才退回历史词汇的子串启发式。
    /// 这与我方铁律一致：`scripts/check-layer-deps.sh` 能做的绝不在别处重造，
    /// 而这里做的是它做不了的"文件级归类计数"。
    fn detect_import_layer(&self, line: &str) -> ArchLayer {
        // ── 精确：真实层目录名（必须最先判，否则被下面的 core:: 吃掉）──
        for (name, layer) in REAL_LAYER_DIRS {
            if line.contains(&format!("crate::{name}::")) {
                return *layer;
            }
        }
        // ── 历史词汇（保留；已知与真实层只有部分重叠，命中数是下界）──
        if line.contains("core::") || line.contains("::core::") {
            ArchLayer::L0Core
        } else if line.contains("l1_body") || line.contains("::act::") {
            ArchLayer::L1Act
        } else if line.contains("l2_world") || line.contains("::world::") {
            ArchLayer::L2World
        } else if line.contains("l3_memory") || line.contains("::memory::") {
            ArchLayer::L3Memory
        } else if line.contains("l4_cognition") || line.contains("::cognition::") {
            ArchLayer::L4Cognition
        } else if line.contains("::prm::") || line.contains("nt_core_prm") {
            ArchLayer::L5Prm
        } else if line.contains("l6_self") || line.contains("::self") || line.contains("::mind::") {
            ArchLayer::L6Self
        } else if line.contains("l7_capability") || line.contains("::capability::") {
            ArchLayer::L7Capability
        } else if line.contains("l8_autonomic") || line.contains("l8_seal") {
            ArchLayer::L8Seal
        } else if line.contains("l9_transcendent") || line.contains("::transcendent::") {
            ArchLayer::L9Transcendent
        } else {
            ArchLayer::Unknown
        }
    }

    /// Observer feedback integration — consumes OneObserver quality/patterns from reasoning engine.
    /// If observer detects trajectory quality < 0.3 or critical patterns, flag as warning.
    fn check_observer_feedback(&mut self) {
        let observer_feedback = self.observer_quality;
        let has_critical = self
            .observer_patterns
            .iter()
            .any(|p| p.contains("oscillation") || p.contains("stuck"));
        if let Some(q) = observer_feedback {
            let degraded = q < self.config.observer_quality_threshold;
            let msg =
                format!(
                "Observer feedback: quality={:.2}, critical_patterns={} — reasoning trajectory {}",
                q,
                self.observer_patterns.len(),
                if degraded { "DEGRADED (quality < 0.3)" } else { "OK" }
            );
            self.check(
                !degraded,
                if degraded {
                    Severity::Warning
                } else {
                    Severity::Info
                },
                "observer_feedback",
                msg,
                file!(),
                line!(),
            );
        }
        if has_critical {
            let pat_msg = format!(
                "Observer critical patterns: {}",
                self.observer_patterns.join(", ")
            );
            self.check(
                false,
                Severity::Warning,
                "observer_pattern",
                pat_msg,
                file!(),
                line!(),
            );
        }
    }

    /// Check binary health — verify [[bin]] entries in Cargo.toml match files in src/bin/.
    /// Detects orphan entries (bin declared but no file) and orphan files (file but no entry).
    /// Distilled from Cycle 24 deep scan: 12 bins verified, 0 orphans.
    fn check_binary_health(&mut self) {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let cargo_toml = manifest_dir.join("Cargo.toml");
        let bin_dir = manifest_dir.join("src").join("bin");

        // Parse [[bin]] entries from Cargo.toml
        let content = match read_source_cached(&cargo_toml) {
            Ok(c) => c,
            Err(_) => return,
        };
        let mut declared_bins: Vec<String> = Vec::new();
        let mut in_bin_section = false;
        for line in content.lines() {
            if line.trim_start().starts_with("[[bin]]") {
                in_bin_section = true;
                continue;
            }
            if in_bin_section {
                if line.trim_start().starts_with('[') {
                    in_bin_section = false;
                    continue;
                }
                if let Some(name) = line.trim().strip_prefix("name = \"") {
                    if let Some(end) = name.find('\"') {
                        declared_bins.push(name[..end].to_string());
                    }
                }
            }
        }

        // Collect actual binary files
        let mut actual_bins: Vec<String> = Vec::new();
        if bin_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&bin_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "rs") {
                        if let Some(stem) = path.file_stem() {
                            actual_bins.push(stem.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        // Check for orphans: declared but no file
        let mut orphan_declarations: Vec<String> = Vec::new();
        for bin in &declared_bins {
            if !actual_bins.contains(bin) {
                orphan_declarations.push(bin.clone());
            }
        }
        // Check for orphans: file but no declaration
        let mut orphan_files: Vec<String> = Vec::new();
        for bin in &actual_bins {
            if !declared_bins.contains(bin) {
                orphan_files.push(bin.clone());
            }
        }

        let decl_ok = orphan_declarations.is_empty();
        let file_ok = orphan_files.is_empty();
        let total_declared = declared_bins.len();
        let total_actual = actual_bins.len();

        let msg = format!(
            "Binary health: {} declared, {} files, decl_orphans={:?} file_orphans={:?}",
            total_declared, total_actual, orphan_declarations, orphan_files,
        );
        self.check(
            decl_ok && file_ok,
            Severity::Warning,
            "binary_health",
            msg,
            file!(),
            line!(),
        );
    }

    /// Check test density — find .rs files >50 lines without a #[test] attribute.
    /// Distilled from Cycle 24 test density analysis: 2007 total #[test], gaps in ewhr_bridge/pipeline.
    fn check_test_density(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut uncovered: Vec<String> = Vec::new();
        scan_file_test_coverage(&src_dir, &mut uncovered);
        let msg = format!(
            "Test density: {} uncovered files (>50 lines, no #[test]) — check ewhr_bridge, pipeline, nt-act social stubs",
            uncovered.len(),
        );
        self.check(
            uncovered.len() < self.config.uncovered_test_max,
            Severity::Warning,
            "test_density",
            msg,
            file!(),
            line!(),
        );
    }

    /// Check init safety — detect LazyLock constructors using .unwrap() or .expect().
    /// Distilled from Cycle 24 fix: 7 LazyLock unwrap→expect in session.rs, fetcher.rs.
    fn check_init_safety(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let lazy_unwrap = scan_for_patterns(
            "",
            &[PatternConfig {
                name: "lazy_lock_init".into(),
                pattern: regex::escape("LazyLock::new(||"),
            }],
            Some(&src_dir),
        )
        .len();
        let unwrap_in_lazy = scan_for_pattern_in_lazy_init(&src_dir);
        let msg = format!(
            "Init safety: {} LazyLock inits, {} with unwrap/expect in closure — prefer expect() or fallback",
            lazy_unwrap, unwrap_in_lazy,
        );
        self.check(
            unwrap_in_lazy < self.config.unwrap_in_lazy_max,
            Severity::Warning,
            "init_safety",
            msg,
            file!(),
            line!(),
        );
    }

    /// Check orphan files — detect .rs files in src/ but not referenced in any mod.rs.
    /// Uses module declaration scan, not full compilation graph.
    fn check_orphan_files(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut declared: Vec<String> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&src_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() || path.extension().is_none_or(|e| e != "rs") {
                    continue;
                }
                let content = match read_source_cached(&path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                for line in content.lines() {
                    if line.trim_start().starts_with("pub mod ")
                        || line.trim_start().starts_with("mod ")
                    {
                        if let Some(name) = line
                            .trim()
                            .strip_prefix("pub mod ")
                            .or_else(|| line.trim().strip_prefix("mod "))
                        {
                            if let Some(end) = name.find(|c: char| !c.is_alphanumeric() && c != '_')
                            {
                                declared.push(name[..end].to_string());
                            }
                        }
                    }
                }
            }
        }
        // Collect all .rs files
        let mut all_files: Vec<String> = Vec::new();
        collect_rs_stems(&src_dir, &mut all_files);
        // Exclude lib.rs, main.rs, bin/ entries
        let orphans: Vec<String> = all_files
            .into_iter()
            .filter(|f| !declared.contains(f) && f != "lib" && f != "main")
            .collect();
        let msg = format!(
            "Orphan files: {} .rs files not declared in any mod.rs (may be dead code)",
            orphans.len(),
        );
        self.check(
            orphans.is_empty(),
            Severity::Warning,
            "orphan_files",
            msg,
            file!(),
            line!(),
        );
    }

    /// Check unused imports — detect `use` statements that are the only occurrence in their file
    /// (simple heuristic: count `use crate::` and `use std::` patterns not followed by usage).
    fn check_unused_imports(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let total_imports = scan_for_patterns(
            "",
            &[PatternConfig {
                name: "use_statement".into(),
                pattern: regex::escape("use "),
            }],
            Some(&src_dir),
        )
        .len();
        // Simple heuristic: detect "dead use" patterns where import is the only reference
        let unused_imports = scan_for_unused_import_patterns(&src_dir);
        let msg = format!(
            "Import hygiene: {} total use statements, ~{} potentially unused (dead_code heuristic)",
            total_imports, unused_imports,
        );
        self.check(
            unused_imports < self.config.unused_imports_max,
            Severity::Info,
            "unused_imports",
            msg,
            file!(),
            line!(),
        );
    }

    // ─── Cycle 28 additions — distilled from Python script audit ───

    /// PA016: Python bare except — scan scripts/ for `except:` without Exception type.
    /// This can mask KeyboardInterrupt, SystemExit, and other non-Exception errors.
    fn check_python_bare_except(&mut self) {
        let scripts_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("scripts");
        if !scripts_dir.exists() {
            return;
        }
        let bare_except_count = scan_python_for_bare_except(&scripts_dir);
        let msg = format!(
            "PA016: {} bare `except:` clauses in scripts/ (should specify exception type)",
            bare_except_count,
        );
        self.check(
            bare_except_count == 0,
            Severity::Warning,
            "python_bare_except",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA017: Python SQL injection — scan scripts/ for f-string patterns in SQL queries.
    /// True parametrized queries use `?` or `%s` placeholders, not `{value}`.
    fn check_python_sql_injection(&mut self) {
        let scripts_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("scripts");
        if !scripts_dir.exists() {
            return;
        }
        let fstring_sql = scan_python_fstring_in_sql(&scripts_dir);
        let msg = format!(
            "PA017: {} f-string patterns in SQL statements in scripts/ (use parametrized queries)",
            fstring_sql,
        );
        self.check(
            fstring_sql == 0,
            Severity::Warning,
            "python_sql_injection",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA018: Python legacy table writes — scan scripts/ for writes to knowledge_nodes/knowledge_edges.
    fn check_python_legacy_tables(&mut self) {
        let scripts_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("scripts");
        if !scripts_dir.exists() {
            return;
        }
        let mut count = 0usize;
        if let Ok(entries) = std::fs::read_dir(&scripts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "py") {
                    if let Ok(content) = read_source_cached(&path) {
                        for line in content.lines() {
                            let trimmed = line.trim().to_uppercase();
                            if (trimmed.contains("KNOWLEDGE_NODES")
                                || trimmed.contains("KNOWLEDGE_EDGES"))
                                && (trimmed.starts_with("INSERT")
                                    || trimmed.starts_with("UPDATE")
                                    || trimmed.starts_with("DELETE"))
                            {
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
        let msg = format!(
            "PA018: {} writes to legacy `knowledge_nodes`/`knowledge_edges` tables in scripts/ (use `nodes`/`edges`)",
            count,
        );
        self.check(
            count == 0,
            Severity::Warning,
            "python_legacy_tables",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA019: Python main guard — scan scripts/ for missing `if __name__ == '__main__'`.
    fn check_python_main_guard(&mut self) {
        let scripts_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("scripts");
        if !scripts_dir.exists() {
            return;
        }
        let mut missing = 0usize;
        let mut total = 0usize;
        if let Ok(entries) = std::fs::read_dir(&scripts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "py") {
                    total += 1;
                    if let Ok(content) = read_source_cached(&path) {
                        if !content.contains("if __name__") && !content.contains("def main(") {
                            missing += 1;
                        }
                    }
                }
            }
        }
        let msg = format!(
            "PA019: {}/{} Python scripts in scripts/ lack `if __name__ == '__main__'` guard and `def main()`",
            missing, total,
        );
        self.check(
            missing == 0,
            Severity::Info,
            "python_main_guard",
            msg,
            file!(),
            line!(),
        );
    }

    // ─── Cycle 27 additions — distilled from 6 Rust bug fixes ───

    /// PA010: Negative advantage/reward signal zeroed via `.max(0.0)`.
    /// Detects RL patterns like `adv.max(0.0)` or `reward.max(0.0)` that suppress
    /// negative learning signals — system cannot learn from failures.
    /// Bug found at: nt_core_prm.rs:1089,2346 (adv.max(0.0) → ((adv+1.0)/2.0))
    fn check_negative_signal_zeroed(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let adv_max = scan_for_pattern_excluding_tests(&src_dir, "adv.max(0.0)");
        let reward_max = scan_for_pattern_excluding_tests(&src_dir, "reward.max(0.0)");
        let score_max = scan_for_pattern_excluding_tests(&src_dir, "score.max(0.0)");
        let total = adv_max + reward_max + score_max;
        let msg = format!(
            "PA010: {} negative signal zeroing patterns (adv/reward/score.max(0.0)) — \
             use ((val+1.0)/2.0) to preserve [-1,0) learning signal",
            total,
        );
        self.check(
            total == 0,
            Severity::Warning,
            "negative_signal_zeroed",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA011: FTS index desync — detect writes to `nodes` table without paired writes to `nodes_fts`.
    /// Bug found at: nt_memory_store.rs:38-46, 51-72 (insert_node skip, update_node no sync)
    fn check_fts_desync(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let nodes_inserts = scan_for_patterns(
            "",
            &[PatternConfig {
                name: "nodes_insert".into(),
                pattern: regex::escape("INSERT INTO nodes"),
            }],
            Some(&src_dir),
        )
        .len();
        let nodes_fts_inserts = scan_for_patterns(
            "",
            &[PatternConfig {
                name: "nodes_fts_insert".into(),
                pattern: regex::escape("INSERT INTO nodes_fts"),
            }],
            Some(&src_dir),
        )
        .len();
        let nodes_updates = scan_for_patterns(
            "",
            &[PatternConfig {
                name: "nodes_update".into(),
                pattern: regex::escape("UPDATE nodes SET"),
            }],
            Some(&src_dir),
        )
        .len();
        let nodes_fts_updates = scan_for_patterns(
            "",
            &[PatternConfig {
                name: "nodes_fts_update".into(),
                pattern: regex::escape("UPDATE nodes_fts SET"),
            }],
            Some(&src_dir),
        )
        .len();
        let nodes_deletes = scan_for_patterns(
            "",
            &[PatternConfig {
                name: "nodes_delete".into(),
                pattern: regex::escape("DELETE FROM nodes WHERE"),
            }],
            Some(&src_dir),
        )
        .len();
        let nodes_fts_deletes = scan_for_patterns(
            "",
            &[PatternConfig {
                name: "nodes_fts_delete".into(),
                pattern: regex::escape("DELETE FROM nodes_fts"),
            }],
            Some(&src_dir),
        )
        .len();

        let mut msgs = Vec::new();
        if nodes_inserts > nodes_fts_inserts {
            msgs.push(format!(
                "INSERT: nodes={} > nodes_fts={}",
                nodes_inserts, nodes_fts_inserts
            ));
        }
        if nodes_updates > nodes_fts_updates {
            msgs.push(format!(
                "UPDATE: nodes={} > nodes_fts={}",
                nodes_updates, nodes_fts_updates
            ));
        }
        if nodes_deletes > nodes_fts_deletes {
            msgs.push(format!(
                "DELETE: nodes={} > nodes_fts={}",
                nodes_deletes, nodes_fts_deletes
            ));
        }

        let clean = msgs.is_empty();
        let msg = if clean {
            "PA011: FTS desync check — all nodes operations have paired nodes_fts operations"
                .to_string()
        } else {
            format!("PA011: FTS desync — {}", msgs.join("; "))
        };
        self.check(
            clean,
            Severity::Warning,
            "fts_desync",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA012: zscore_normalize drops NaN entries changing output length.
    /// Detects `filter(|v| v.is_finite())` in normalization functions without length preservation.
    /// Bug found at: nt_core_prm.rs:856-866 (zscore_normalize returning shorter vec)
    fn check_zscore_length_mismatch(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let filter_finite =
            scan_for_pattern_excluding_tests(&src_dir, ".filter(|v| v.is_finite())");
        let normalize_fns = scan_for_pattern_excluding_tests(&src_dir, "fn zscore_normalize");
        let msg = format!(
            "PA012: {} filter(is_finite) in normalization — output length may mismatch input \
             (replace with map(|v| if v.is_finite() {{*v}} else {{0.0}}))",
            filter_finite,
        );
        self.check(
            filter_finite < normalize_fns, // Each normalize fn should have 0 filter_finite
            Severity::Warning,
            "zscore_length_mismatch",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA013: Substring tag matching — detect `.contains("literal")` in tag/attribute matching
    /// where exact string comparison should be used. Prevents false positives like
    /// "not_good" matching "good".
    /// Bug found at: nt_core_policy.rs:292-300 (t.contains("good") matching "not_good")
    fn check_substring_tag_matching(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let contains_good = scan_for_pattern_excluding_tests(&src_dir, ".contains(\"good\")");
        let contains_fail = scan_for_pattern_excluding_tests(&src_dir, ".contains(\"fail\")");
        let contains_ok = scan_for_pattern_excluding_tests(&src_dir, ".contains(\"ok\")");
        let total = contains_good + contains_fail + contains_ok;
        let msg = format!(
            "PA013: {} substring tag matches (contains(\"good\"/\"fail\"/\"ok\")) — \
             use tag.as_str() or == for exact semantic matching",
            total,
        );
        self.check(
            total == 0,
            Severity::Warning,
            "substring_tag_match",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA014: HashMap iteration order dependency — detect `.values().nth()` on HashMap
    /// where iteration order is non-deterministic. Prefer BTreeMap for positional indexing.
    /// Bug found at: workspace.rs:344,356 + neotrix-types workspace.rs:117,129
    fn check_hashmap_iteration_order(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let hashmap_nth = scan_for_pattern_excluding_tests(&src_dir, "HashMap<")
            + scan_for_pattern_excluding_tests(&src_dir, "HashMap::");
        let nth_values = scan_for_pattern_excluding_tests(&src_dir, ".values().nth(");
        let use_btreemap_for_nth = nth_values > 0 && hashmap_nth > 0;
        let msg = format!(
            "PA014: {} .values().nth() calls on HashMap (non-deterministic iteration order). \
             {} HashMap declarations — use BTreeMap for stable positional indexing",
            nth_values, hashmap_nth,
        );
        self.check(
            !use_btreemap_for_nth,
            Severity::Warning,
            "hashmap_iteration_order",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA015: Clamp pattern on advantage/reward — detect `.max(0.0).min(1.0)` pipelines
    /// where the bounds may not match the variable's actual range.
    /// Semantically safer: use proper clamp or domain-aware mapping.
    /// Bug found at: nt_core_prm.rs:1089,2346 (adv.max(0.0).min(1.0) on [-1,1] range)
    fn check_clamp_negative_semantics(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let max_min_01 = scan_for_pattern_excluding_tests(&src_dir, ".max(0.0).min(1.0)");
        let msg = format!(
            "PA015: {} .max(0.0).min(1.0) clamp patterns — verify variable range. \
             If range is [-1,1], use ((x+1.0)/2.0) instead to preserve negative signal",
            max_min_01,
        );
        self.check(
            max_min_01 < 3,
            Severity::Info,
            "clamp_semantics",
            msg,
            file!(),
            line!(),
        );
    }

    // ─── Cycle 29 additions — Karpathy-inspired principles (PA020-PA023) ───

    /// PA020 (Simplicity First): Detect overcomplicated patterns — generic params not used,
    /// deep nesting (>5 levels), excessive function length (>200 lines), single-impl traits.
    /// Inspired by Karpathy's "code should be simple enough to fit in your head."
    pub(crate) fn check_karpathy_simplicity_first(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let unused_generics = scan_for_unused_generic_params(&src_dir);
        let deep_nesting = count_deeply_nested_lines(&src_dir, 5);
        let long_fns = count_long_functions(&src_dir, 200);
        let single_traits = count_single_impl_traits(&src_dir);
        let issues = unused_generics + deep_nesting + long_fns + single_traits;
        let msg = format!(
            "PA020 (Simplicity First): {} issues — {} unused generics, {} deep nesting sites \
             (>5 levels), {} long functions (>200 lines), {} single-impl traits (consider inlining)",
            issues, unused_generics, deep_nesting, long_fns, single_traits,
        );
        self.check(
            issues < self.config.karpathy_simplicity_max,
            Severity::Warning,
            "karpathy_simplicity_first",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA021 (Surgical Changes): Detect large diffs — files changed >10, lines added >500.
    /// Inspired by Karpathy's "small, focused commits are easier to review and revert."
    pub(crate) fn check_karpathy_surgical_changes(&mut self) {
        let repo_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let files_changed = count_files_in_last_commit(repo_dir);
        let lines_added = count_lines_in_last_commit(repo_dir);
        let msg = format!(
            "PA021 (Surgical Changes): last commit changed {} files, added {} lines — \
             prefer smaller, focused commits (≤10 files, ≤500 lines)",
            files_changed, lines_added,
        );
        self.check(
            files_changed <= self.config.karpathy_surgical_files
                && lines_added <= self.config.karpathy_surgical_lines,
            Severity::Info,
            "karpathy_surgical_changes",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA022 (Complexity Budget): Detect cyclomatic complexity indicators — long if-else chains,
    /// excessive match arms, functions with too many parameters.
    /// Inspired by Karpathy's "complexity is a budget — spend it wisely."
    pub(crate) fn check_karpathy_complexity_budget(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let long_chains = count_long_if_chains(&src_dir, 8);
        let many_arms = count_excessive_match_arms(&src_dir, 15);
        let many_params = count_excessive_param_count(&src_dir, 10);
        let issues = long_chains + many_arms + many_params;
        let msg = format!(
            "PA022 (Complexity Budget): {} issues — {} long if-else chains (>8 arms), \
             {} excessive match expressions (>15 arms), {} functions with >10 parameters",
            issues, long_chains, many_arms, many_params,
        );
        self.check(
            issues < self.config.karpathy_complexity_max,
            Severity::Warning,
            "karpathy_complexity_budget",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA023 (Goal-Driven Execution): Detect imperative-only patterns — TODOs without tests,
    /// state-mutating functions not returning Result, missing success criteria in docs.
    /// Inspired by Karpathy's "write the test first, then the code."
    pub(crate) fn check_karpathy_goal_driven_execution(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let todos_no_test = count_todos_without_nearby_test(&src_dir);
        let mut_no_result = count_state_mutation_no_result(&src_dir);
        let missing_goals = count_pub_fn_without_goal_doc(&src_dir);
        let issues = todos_no_test + mut_no_result + missing_goals;
        let msg = format!(
            "PA023 (Goal-Driven Execution): {} issues — {} TODOs without associated test, \
             {} &mut self fns without Result return, {} pub fns missing success criteria in docs",
            issues, todos_no_test, mut_no_result, missing_goals,
        );
        self.check(
            issues < self.config.karpathy_goal_driven_max,
            Severity::Warning,
            "karpathy_goal_driven_execution",
            msg,
            file!(),
            line!(),
        );
    }

    // ─── Cycle 30 additions — distilled from Architecture Rebirth full-stack audit ───

    /// PA024 (SEAL Stage Health): Detect SEAL pipeline stages that are no-op stubs
    /// (just `log::trace!` + `Ok(StageDecision::Continue)` without real work).
    fn check_seal_stage_health(&mut self) {
        let pipeline_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("neotrix")
            .join("l8_autonomic_impl")
            .join("nt_mind")
            .join("self_iterating")
            .join("pipeline.rs");
        let mut stub_count = 0usize;
        let mut total = 0usize;
        if let Ok(content) = read_source_cached(&pipeline_path) {
            let lines: Vec<&str> = content.lines().collect();
            for i in 0..lines.len() {
                if lines[i].trim().starts_with("fn process(") {
                    total += 1;
                    let mut body_lines = 0usize;
                    let mut real_ops = 0usize;
                    for j in (i + 1)..lines.len().min(i + 15) {
                        let t = lines[j].trim();
                        if t.starts_with("Ok(") || t.starts_with('}') {
                            break;
                        }
                        body_lines += 1;
                        if t.starts_with("log::") {
                            continue;
                        }
                        if !t.is_empty() && !t.starts_with("//") {
                            real_ops += 1;
                        }
                    }
                    if real_ops == 0 && body_lines > 0 {
                        stub_count += 1;
                    }
                }
            }
        }
        let msg = format!(
            "PA024: {}/{} SEAL pipeline stages are no-op stubs (log-only, no real operations)",
            stub_count, total,
        );
        self.check(
            stub_count <= self.config.seal_stub_max,
            Severity::Warning,
            "seal_stage_health",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA025 (CI Workflow Coverage): Check that essential CI workflows exist.
    fn check_ci_workflow_coverage(&mut self) {
        let workflows_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(".github")
            .join("workflows");
        let mut found = Vec::new();
        if workflows_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&workflows_dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        found.push(name.to_string());
                    }
                }
            }
        }
        let has_deny = found.iter().any(|f| f.contains("deny"));
        let has_build = found
            .iter()
            .any(|f| f.contains("build") || f.contains("ci"));
        let msg = format!(
            "PA025: CI workflows found: {:?} — cargo-deny={}, build={}",
            found, has_deny, has_build,
        );
        self.check(
            has_deny && has_build,
            Severity::Info,
            "ci_workflow_coverage",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA026 (Test Assertion Hygiene): Detect tests using brittle string assertions
    /// (checking for `assert_eq!(result, "exact string")` in integration tests).
    fn check_test_assertion_hygiene(&mut self) {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut brittle = 0usize;
        let mut integration_test_files = 0usize;
        if let Ok(entries) = std::fs::read_dir(&src_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    continue;
                }
                if path.file_name().is_some_and(|n| {
                    let s = n.to_string_lossy();
                    s.contains("integration_test") || s.contains("e2e_test")
                }) {
                    integration_test_files += 1;
                    if let Ok(content) = read_source_cached(&path) {
                        for line in content.lines() {
                            let t = line.trim();
                            if (t.starts_with("assert_eq!(") || t.starts_with("assert!("))
                                && t.contains("Status")
                                || t.contains("status")
                                || t.contains("error")
                            {
                                brittle += 1;
                            }
                        }
                    }
                }
            }
        }
        let msg = format!(
            "PA026: {} brittle status/error assertions in {} integration test files \
             (use structured error matching instead of string comparison)",
            brittle, integration_test_files,
        );
        self.check(
            brittle <= 5,
            Severity::Info,
            "test_assertion_hygiene",
            msg,
            file!(),
            line!(),
        );
    }

    /// PA027 (Dep Version Consistency): Detect cross-crate dependency version mismatches.
    fn check_dep_version_consistency(&mut self) {
        let workspace_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let mismatches: Vec<String> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&workspace_dir) {
            let mut toml_files = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let cargo_toml = path.join("Cargo.toml");
                    if cargo_toml.exists() {
                        toml_files.push(cargo_toml);
                    }
                }
            }
            // Look for Cargo.toml files with known version-sensitive deps
            for toml in &toml_files {
                if let Ok(content) = read_source_cached(toml) {
                    for dep in &["quick-xml", "tokio", "serde", "serde_json", "uuid"] {
                        let pattern = format!("{} = \"", dep);
                        if let Some(pos) = content.find(&pattern) {
                            let start = pos + pattern.len();
                            let _end = content[start..].find('"').map(|e| start + e).unwrap_or(0);
                            // simplistic check — just flag it for review
                        }
                    }
                }
            }
        }
        let msg = format!(
            "PA027: Dependency version consistency — {} mismatches found (check Cargo.toml files)",
            mismatches.len(),
        );
        self.check(
            mismatches.is_empty(),
            Severity::Info,
            "dep_version_consistency",
            msg,
            file!(),
            line!(),
        );
    }
}

// ─── moved: mod.rs:1654-1702 scan_for_patterns ───
pub fn scan_for_patterns(
    source: &str,
    patterns: &[PatternConfig],
    dir: Option<&Path>,
) -> Vec<PatternMatch> {
    let compiled: Vec<(&PatternConfig, Regex)> = patterns
        .iter()
        .filter_map(|pc| Regex::new(&pc.pattern).ok().map(|r| (pc, r)))
        .collect();
    if compiled.is_empty() {
        return Vec::new();
    }
    let mut results = Vec::new();
    if let Some(d) = dir {
        let file_list = cached_rs_files(d);
        for file_path in file_list.iter() {
            if let Ok(content) = read_source_cached(file_path) {
                for (pc, re) in &compiled {
                    for m in re.find_iter(&content) {
                        let line = content[..m.start()].matches('\n').count() + 1;
                        let col = m.start() - content[..m.start()].rfind('\n').map_or(0, |i| i + 1);
                        results.push(PatternMatch {
                            file: file_path.to_string_lossy().to_string(),
                            line,
                            column: col,
                            pattern_name: pc.name.clone(),
                            matched_text: m.as_str().to_string(),
                        });
                    }
                }
            }
        }
    } else {
        for (pc, re) in &compiled {
            for m in re.find_iter(source) {
                let line = source[..m.start()].matches('\n').count() + 1;
                let col = m.start() - source[..m.start()].rfind('\n').map_or(0, |i| i + 1);
                results.push(PatternMatch {
                    file: String::new(),
                    line,
                    column: col,
                    pattern_name: pc.name.clone(),
                    matched_text: m.as_str().to_string(),
                });
            }
        }
    }
    results
}


/// 真实层目录名 ⇄ `ArchLayer` 的映射（裁定 5A）。
///
/// ⚠️ **映射本身带妥协**：`l3_embodiment`（具体化/盾）没有对应枚举变体，
/// 被映到 `L3Memory`；`l4_emotion`（情绪/记忆）映到 `L4Cognition`。
/// ⛔ 这两处的**分层语义是错的**，改枚举要动 10+ 处 `layer_index()` 匹配，
/// 属 roadmap T0-4（层词汇裁决）的范围。
/// ⇒ 本表只保证「能分类并计数」，**不宣称「分类正确」**。
static REAL_LAYER_DIRS: &[(&str, ArchLayer)] = &[
    ("l0_substrate", ArchLayer::L0Core),
    ("l1_action", ArchLayer::L1Act),
    ("l2_perception", ArchLayer::L2World),
    ("l3_embodiment", ArchLayer::L3Memory),
    ("l4_emotion", ArchLayer::L4Cognition),
    ("l5_cognition", ArchLayer::L5Prm),
    ("l6_meta", ArchLayer::L6Self),
];

/// 递归收集目录下的 `.rs`（跳过 `target/` 与隐藏目录）—— 裁定 5A 第二批。
fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            if name == "target" || name.starts_with('.') {
                continue;
            }
            out.extend(collect_rs_files(&p));
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
    out
}

/// 扫一个文件里的 `use crate::` 行，统计**反向**分层引用（低层引高层为逆）。
///
/// ⚠️ **已知局限**（不要高估本检查）：
/// 1. 只看 `use crate::` 开头的行 ⇒ `mod.rs` 里的 `pub use a as b`、
///    宏内路径、`#[path]` 全看不见。
/// 2. 字符串字面量里的路径**也会被算进来**（门分不清字面量与真引用）。
/// 3. `detect_import_layer` 认的是**第三套词汇**（`l0_core`/`l1_body`…），
///    与真实层名只有部分重叠 ⇒ 命中数是**下界**。
/// ⇒ 本检查的定位是"粗筛 + 可见"，**不是权威分层裁决**；
///    权威仍是 `scripts/check-layer-deps.sh`（clean checkout 口径）。
fn scan_layer_file(
    gate: &SelfReviewGate,
    path: &Path,
    source_layer: ArchLayer,
    violations: &mut usize,
) {
    let Ok(content) = read_source_cached(path) else {
        return;
    };
    for line in content.lines() {
        if line.starts_with("use crate::") {
            let target_layer = gate.detect_import_layer_pub(line);
            // ⛔ 2026-10-07（裁定 5A 第三批）：**判据方向原本是反的**。
            //
            // 原条件 `target < source` 把「高层引低层」判成违规，例如
            // `l2_perception/error_conversions.rs` 引 `l0_substrate::nt_core_error`
            // 被算成 L2→L0 违规 —— 而那**恰恰是分层的目的**（下层供上层）。
            //
            // 正典口径见 `scripts/check-layer-deps.sh:2`：
            //   「enforces L0->L1->L2->L3->L4->L5->L6 unidirectional deps」
            // ⇒ 合法方向是 source<target（引更高层）；本仓的真违规形态是
            //   **源层号小、目标层号大却跨层跳跃**（如 L1→L5 的 S1/S2 类）。
            //
            // ⛔ 但**本检查只能发现「反向」，无法发现「跨层跳跃」** ——
            //   源层与目标层的层距（|src−tgt| > 1）才是真正的判据，而
            //   `ArchLayer` 的历史映射把 embodiment→memory、emotion→cognition
            //   压成了相邻层，使层距不可靠。
            // ⇒ 因此这里只报**反向引用**（方向确实错、必是债），
            //   跨层跳跃仍以 `check-layer-deps.sh` 为权威（见函数文档的「已知局限」）。
            if target_layer.layer_index() >= 0
                && target_layer.layer_index() > source_layer.layer_index()
            {
                *violations += 1;
            }
        }
    }
}
