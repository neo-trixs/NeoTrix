//! CoverageAnalyzer — 代码覆盖率分析器
//!
//! P4-04: 扫描 Rust 源码树，统计每个模块的文件数、测试覆盖数、行数。
//! 不需要运行 instrumented 二进制 — 静态分析 `#[cfg(test)]` / `#[test]` 的存在。
//!
//! 用途: 识别未测试模块、超大文件，驱动目标生成和重构决策。

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CoverageReport {
    pub modules: Vec<ModuleCoverage>,
    pub total_files: usize,
    pub tested_files: usize,
    pub untested_files: usize,
    pub overall_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct ModuleCoverage {
    pub name: String,
    pub file_count: usize,
    pub tested_count: usize,
    pub untested_count: usize,
    pub total_lines: usize,
    pub coverage_ratio: f64,
    pub large_files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CoverageAnalyzer {
    project_root: PathBuf,
    modules: Vec<String>,
    large_file_threshold: usize,
}

impl CoverageAnalyzer {
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            modules: vec![],
            large_file_threshold: 800,
        }
    }

    pub fn with_modules(project_root: PathBuf, modules: Vec<String>) -> Self {
        Self {
            project_root,
            modules,
            large_file_threshold: 800,
        }
    }

    pub fn analyze(&self) -> CoverageReport {
        let modules = if self.modules.is_empty() {
            self.discover_modules()
        } else {
            self.modules.clone()
        };

        let mut modules_covered: Vec<ModuleCoverage> = Vec::with_capacity(modules.len());
        for module in &modules {
            modules_covered.push(self.scan_module(module));
        }

        let total_files = modules_covered.iter().map(|m| m.file_count).sum();
        let tested_files = modules_covered.iter().map(|m| m.tested_count).sum();
        let untested_files = modules_covered.iter().map(|m| m.untested_count).sum();
        let overall_ratio = if total_files > 0 {
            tested_files as f64 / total_files as f64
        } else {
            0.0
        };

        CoverageReport {
            modules: modules_covered,
            total_files,
            tested_files,
            untested_files,
            overall_ratio,
        }
    }

    fn discover_modules(&self) -> Vec<String> {
        let src = self.project_root.join("src");
        if !src.exists() {
            return vec![];
        }
        let mut modules = vec![];
        if let Ok(entries) = fs::read_dir(&src) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        modules.push(name.to_string());
                    }
                }
            }
        }
        modules.sort();
        modules
    }

    fn scan_module(&self, module: &str) -> ModuleCoverage {
        let module_path = self.project_root.join("src").join(module);
        let mut file_count = 0usize;
        let mut tested_count = 0usize;
        let mut total_lines = 0usize;
        let mut large_files = vec![];

        let entries = match Self::collect_rs_files(&module_path) {
            Some(files) => files,
            None => {
                return ModuleCoverage {
                    name: module.to_string(),
                    file_count: 0,
                    tested_count: 0,
                    untested_count: 0,
                    total_lines: 0,
                    coverage_ratio: 0.0,
                    large_files: vec![],
                };
            }
        };

        for file_path in &entries {
            file_count += 1;
            let lines = Self::count_lines(file_path);
            total_lines += lines;
            if lines > self.large_file_threshold {
                if let Some(relative) = file_path
                    .strip_prefix(&self.project_root)
                    .ok()
                    .and_then(|p| p.to_str())
                {
                    large_files.push(relative.to_string());
                }
            }
            if Self::is_test_file(file_path) {
                tested_count += 1;
            }
        }

        let untested_count = file_count.saturating_sub(tested_count);
        let coverage_ratio = if file_count > 0 {
            tested_count as f64 / file_count as f64
        } else {
            0.0
        };

        ModuleCoverage {
            name: module.to_string(),
            file_count,
            tested_count,
            untested_count,
            total_lines,
            coverage_ratio,
            large_files,
        }
    }

    fn collect_rs_files(dir: &Path) -> Option<Vec<PathBuf>> {
        if !dir.exists() || !dir.is_dir() {
            return None;
        }
        let mut files = vec![];
        Self::walk_dir(dir, &mut files);
        Some(files)
    }

    fn walk_dir(dir: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    Self::walk_dir(&path, files);
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    files.push(path);
                }
            }
        }
    }

    fn count_lines(path: &Path) -> usize {
        fs::read_to_string(path)
            .ok()
            .map(|s| s.lines().count())
            .unwrap_or(0)
    }

    fn is_test_file(path: &Path) -> bool {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return false,
        };
        content.contains("#[cfg(test)]") || content.contains("#[test]")
    }

    #[cfg(test)]
    fn count_tests_in_file(path: &Path) -> u32 {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return 0,
        };
        content
            .lines()
            .filter(|line| line.trim() == "#[test]")
            .count() as u32
    }

    pub fn summary(&self, report: &CoverageReport) -> String {
        let mut out = String::new();
        out.push_str("📊 Coverage Report\n");

        out.push_str(&format!(
            "  Total: {} files | ✅ tested: {} | ❌ untested: {}\n",
            report.total_files, report.tested_files, report.untested_files
        ));

        out.push_str(&format!(
            "  Coverage: {:.1}%\n",
            report.overall_ratio * 100.0
        ));

        out.push_str("  Modules:\n");
        for module in &report.modules {
            out.push_str(&format!(
                "    {}/: {} files, {:.1}%\n",
                module.name,
                module.file_count,
                module.coverage_ratio * 100.0
            ));
        }

        let total_large: usize = report.modules.iter().map(|m| m.large_files.len()).sum();
        out.push_str(&format!(
            "  Large files (>{}\u{20}lines): {}\n",
            self.large_file_threshold, total_large,
        ));

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("coverage_test_{}", name));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn create_test_file(dir: &Path, rel_path: &str, content: &str) {
        let full_path = dir.join(rel_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full_path, content).unwrap();
    }

    #[test]
    fn test_empty_directory() {
        let root = temp_dir("empty");
        let src = root.join("src");
        fs::create_dir_all(&src).unwrap();
        // create an empty module dir with no files
        fs::create_dir_all(src.join("empty_mod")).unwrap();

        let analyzer = CoverageAnalyzer::with_modules(root, vec!["empty_mod".into()]);
        let report = analyzer.analyze();

        assert_eq!(report.total_files, 0);
        assert_eq!(report.tested_files, 0);
        assert_eq!(report.untested_files, 0);
        assert!((report.overall_ratio - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_mixed_tested_and_untested_files() {
        let root = temp_dir("mixed");
        let module_dir = root.join("src").join("test_mod");
        fs::create_dir_all(&module_dir).unwrap();

        // file with #[cfg(test)]
        create_test_file(&root, "src/test_mod/with_tests.rs", r#"
fn foo() -> i32 { 1 }
#[cfg(test)]
mod tests {
    #[test]
    fn test_foo() { assert_eq!(foo(), 1); }
}
"#);
        // file with #[test]
        create_test_file(&root, "src/test_mod/with_test_attr.rs", r#"
#[test]
fn test_bar() { assert_eq!(2 + 2, 4); }
"#);
        // file without tests
        create_test_file(&root, "src/test_mod/no_tests.rs", r#"
fn baz() -> &'static str { "hello" }
"#);

        let analyzer = CoverageAnalyzer::with_modules(root, vec!["test_mod".into()]);
        let report = analyzer.analyze();

        assert_eq!(report.total_files, 3);
        assert_eq!(report.tested_files, 2);
        assert_eq!(report.untested_files, 1);
        assert!((report.overall_ratio - 2.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_large_file_detection() {
        let root = temp_dir("large_file");
        let module_dir = root.join("src").join("big_mod");
        fs::create_dir_all(&module_dir).unwrap();

        // Create a file with many lines (801 lines to exceed default 800 threshold)
        let mut large_content = String::new();
        for i in 0..801 {
            large_content.push_str(&format!("// line {}\n", i));
        }
        create_test_file(&root, "src/big_mod/huge.rs", &large_content);

        // small file
        create_test_file(&root, "src/big_mod/small.rs", "fn x() {}");

        let analyzer = CoverageAnalyzer::with_modules(root, vec!["big_mod".into()]);
        let report = analyzer.analyze();

        assert_eq!(report.total_files, 2);
        assert_eq!(report.modules.len(), 1);
        assert_eq!(report.modules[0].large_files.len(), 1);
        assert!(report.modules[0].large_files[0].contains("huge.rs"));
    }

    #[test]
    fn test_summary_format() {
        let root = temp_dir("summary");
        let module_dir = root.join("src").join("sum_mod");
        fs::create_dir_all(&module_dir).unwrap();

        create_test_file(&root, "src/sum_mod/tested.rs", r#"
#[test]
fn t() {}
"#);
        create_test_file(&root, "src/sum_mod/untested.rs", "fn u() {}");

        let analyzer = CoverageAnalyzer::with_modules(root.clone(), vec!["sum_mod".into()]);
        let report = analyzer.analyze();
        let summary = analyzer.summary(&report);

        assert!(summary.contains("Coverage Report"));
        assert!(summary.contains("Total: 2 files"));
        assert!(summary.contains("✅ tested: 1"));
        assert!(summary.contains("❌ untested: 1"));
        assert!(summary.contains("Coverage: 50.0%"));
        assert!(summary.contains("sum_mod/: 2 files, 50.0%"));
    }

    #[test]
    fn test_custom_modules_list() {
        let root = temp_dir("custom_mods");
        // Create src dir with 3 modules, but only specify 2
        for mod_name in &["alpha", "beta", "gamma"] {
            let dir = root.join("src").join(mod_name);
            fs::create_dir_all(&dir).unwrap();
            create_test_file(&root, &format!("src/{}/lib.rs", mod_name), "fn x() {}");
        }

        let analyzer = CoverageAnalyzer::with_modules(
            root,
            vec!["alpha".into(), "gamma".into()],
        );
        let report = analyzer.analyze();

        assert_eq!(report.modules.len(), 2);
        assert_eq!(report.total_files, 2);
        assert!(report.modules.iter().any(|m| m.name == "alpha"));
        assert!(report.modules.iter().any(|m| m.name == "gamma"));
        assert!(!report.modules.iter().any(|m| m.name == "beta"));
    }

    #[test]
    fn test_auto_scan_no_modules() {
        let root = temp_dir("auto_scan");
        for mod_name in &["engine", "ui"] {
            let dir = root.join("src").join(mod_name);
            fs::create_dir_all(&dir).unwrap();
            create_test_file(&root, &format!("src/{}/mod.rs", mod_name), "fn x() {}");
        }

        let analyzer = CoverageAnalyzer::new(root);
        let report = analyzer.analyze();

        assert_eq!(report.modules.len(), 2);
        let names: Vec<&str> = report.modules.iter().map(|m| m.name.as_str()).collect();
        assert!(names.contains(&"engine"));
        assert!(names.contains(&"ui"));
    }

    #[test]
    fn test_count_tests_in_file() {
        let root = temp_dir("count_tests");
        let dir = root.join("src").join("count_mod");
        fs::create_dir_all(&dir).unwrap();

        create_test_file(&root, "src/count_mod/multi_test.rs", r#"
fn helper() {}
#[test]
fn t1() {}
#[test]
fn t2() {}
#[test]
fn t3() {}
"#);

        let path = dir.join("multi_test.rs");
        assert_eq!(CoverageAnalyzer::count_tests_in_file(&path), 3);
    }

    #[test]
    fn test_is_test_file_detection() {
        let root = temp_dir("is_test");
        let dir = root.join("src").join("detect_mod");
        fs::create_dir_all(&dir).unwrap();

        create_test_file(&root, "src/detect_mod/has_cfg_test.rs", r#"
#[cfg(test)]
mod tests {
    #[test]
    fn t() {}
}
"#);
        create_test_file(&root, "src/detect_mod/has_test_attr.rs", r#"
#[test]
fn standalone() {}
"#);
        create_test_file(&root, "src/detect_mod/no_test.rs", "fn pure() {}");

        let has_cfg = dir.join("has_cfg_test.rs");
        let has_attr = dir.join("has_test_attr.rs");
        let no_test = dir.join("no_test.rs");

        assert!(CoverageAnalyzer::is_test_file(&has_cfg));
        assert!(CoverageAnalyzer::is_test_file(&has_attr));
        assert!(!CoverageAnalyzer::is_test_file(&no_test));
    }
}
