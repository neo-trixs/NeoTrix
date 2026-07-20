#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct AuditFinding {
    pub category: &'static str,
    pub severity: AuditSeverity,
    pub file: String,
    pub line: Option<usize>,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct AuditReport {
    pub findings: Vec<AuditFinding>,
    pub ghost_count: usize,
    pub stale_count: usize,
    pub orphan_count: usize,
    pub persistence_fail_count: usize,
}

impl AuditReport {
    pub fn is_clean(&self) -> bool {
        self.ghost_count == 0 && self.stale_count == 0 && self.persistence_fail_count == 0
    }
}

pub fn scan_ghost_modules<P: AsRef<Path>>(root: P) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    let src = root.as_ref();
    if !src.is_dir() {
        findings.push(AuditFinding {
            category: "ghost-scan",
            severity: AuditSeverity::Error,
            file: src.to_string_lossy().to_string(),
            line: None,
            message: "Source directory not found".to_string(),
        });
        return findings;
    }
    walk_mod_files(src, src, &mut findings, 0);
    findings
}

fn walk_mod_files(src: &Path, dir: &Path, findings: &mut Vec<AuditFinding>, depth: usize) {
    if depth > 20 {
        return;
    }
    let mod_rs = dir.join("mod.rs");
    if !mod_rs.exists() {
        return;
    }
    let content = match fs::read_to_string(&mod_rs) {
        Ok(c) => c,
        Err(_) => return,
    };
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let mut cfg = false;
        if line.trim_start().starts_with("#[cfg(") {
            cfg = true;
            i += 1;
        }
        if i >= lines.len() { break; }
        let decl = if cfg {
            lines[i]
        } else {
            line
        };
        let trimmed = decl.trim();
        if let Some(name) = trimmed.strip_prefix("pub(crate) mod ")
            .or_else(|| trimmed.strip_prefix("pub mod "))
            .or_else(|| trimmed.strip_prefix("mod "))
            .and_then(|s| s.strip_suffix(';')) {
            let name = name.trim();
            let rs_file = dir.join(format!("{}.rs", name));
            let sub_mod = dir.join(name).join("mod.rs");
            if !rs_file.exists() && !sub_mod.exists() && !name.contains("::") {
                let rel = mod_rs.strip_prefix(src).unwrap_or(&mod_rs);
                findings.push(AuditFinding {
                    category: "ghost-module",
                    severity: AuditSeverity::Error,
                    file: rel.to_string_lossy().to_string(),
                    line: Some(i + 1),
                    message: format!("Module `{}` declared but no file found (searched: {}, {})",
                        name, rs_file.display(), sub_mod.display()),
                });
            }
        }
        i += 1;
    }
    let mut entries: Vec<PathBuf> = fs::read_dir(dir).unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect();
    entries.sort();
    for sub in entries {
        walk_mod_files(src, &sub, findings, depth + 1);
    }
}

pub fn scan_orphan_files<P: AsRef<Path>>(root: P) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    let src = root.as_ref();
    let all_rs: Vec<PathBuf> = walk_rs_files(src);
    let declared: HashSet<PathBuf> = collect_declared_paths(src);
    for path in &all_rs {
        if !declared.contains(path)
            && !path.to_string_lossy().contains("target/")
            && path.file_name().and_then(|n| n.to_str()) != Some("lib.rs")
            && path.file_name().and_then(|n| n.to_str()) != Some("main.rs")
            && !path.to_string_lossy().contains("/bin/")
        {
            let rel = path.strip_prefix(src).unwrap_or(path);
            findings.push(AuditFinding {
                category: "orphan-file",
                severity: AuditSeverity::Warning,
                file: rel.to_string_lossy().to_string(),
                line: None,
                message: "File exists but is not declared in any mod.rs".to_string(),
            });
        }
    }
    findings
}

fn walk_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !dir.is_dir() { return files; }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walk_rs_files(&path));
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
    files
}

fn collect_declared_paths(src: &Path) -> HashSet<PathBuf> {
    let mut declared = HashSet::new();
    let all_mods: Vec<PathBuf> = walk_rs_files(src).into_iter()
        .filter(|p| p.file_name().and_then(|n| n.to_str()) == Some("mod.rs"))
        .collect();

    for mod_rs in &all_mods {
        let dir = mod_rs.parent().unwrap();
        if let Ok(content) = fs::read_to_string(mod_rs) {
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some(name) = trimmed.strip_prefix("pub(crate) mod ").or_else(|| trimmed.strip_prefix("pub mod ")).or_else(|| trimmed.strip_prefix("mod ")).and_then(|s| s.strip_suffix(';')) {
                    let name = name.trim();
                    let rs = dir.join(format!("{}.rs", name));
                    let sub = dir.join(name).join("mod.rs");
                    if rs.exists() { declared.insert(rs); }
                    if sub.exists() { declared.insert(sub); }
                }
            }
        }
    }
    declared
}

pub fn verify_persistence(file_path: &str, expected_pattern: &str) -> bool {
    match fs::read_to_string(file_path) {
        Ok(content) => content.lines().any(|l| l.contains(expected_pattern)),
        Err(e) => {
            eprintln!("  [Audit] Cannot read {} for persistence check: {}", file_path, e);
            false
        }
    }
}

pub fn converge_check<P: AsRef<Path>>(root: P) -> AuditReport {
    let ghost = scan_ghost_modules(root.as_ref());
    let ghost_count = ghost.len();
    let stale = scan_orphan_files(root.as_ref());
    let stale_count = stale.len();
    let mut all = Vec::new();
    all.extend(ghost);
    all.extend(stale);
    AuditReport {
        findings: all,
        ghost_count,
        stale_count,
        orphan_count: 0,
        persistence_fail_count: 0,
    }
}

impl crate::core::nt_core_self_test::SelfTest for ConvergeCheckFn {
    fn name(&self) -> &str {
        "self_audit"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let root = ".";

        // Test 1: scan current tree — should find 0 ghosts if clean
        let ghosts = scan_ghost_modules(root);
        let ghost_count = ghosts.iter().filter(|f| f.category == "ghost-module").count();
        if ghost_count > 0 {
            failures.push(format!("Expected 0 ghost modules, found {}", ghost_count));
        }

        // Test 2: scan current tree — orphans should be 0 (excluding bin/ and legacy)
        let orphans = scan_orphan_files(root);
        let orphan_count = orphans.iter()
            .filter(|f| !f.file.contains("/bin/") && !f.file.contains("target/"))
            .count();
        if orphan_count > 0 {
            failures.push(format!("Expected 0 orphan files, found {}", orphan_count));
        }

        // Test 3: verify_persistence on known-good file
        if !verify_persistence("Cargo.toml", "[package]") {
            failures.push("verify_persistence: should find [package] in Cargo.toml".into());
        }

        if failures.is_empty() { Ok(()) } else { Err(failures) }
    }
}

/// Helper struct to implement SelfTest for the module-level functions.
pub struct ConvergeCheckFn;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_ghost_modules() {
        let findings = scan_ghost_modules(".");
        let ghost_modules: Vec<_> = findings.iter().filter(|f| f.category == "ghost-module").collect();
        assert!(ghost_modules.is_empty(), "Ghost modules found: {:?}", ghost_modules);
    }

    #[test]
    fn test_no_orphans_in_core() {
        let src = Path::new("src/core");
        if src.exists() {
            let findings = scan_orphan_files(src);
            let orphans: Vec<_> = findings.iter().filter(|f| f.category == "orphan-file")
                .filter(|f| {
                    !f.file.contains("/bin/")
                    && !f.file.contains("/tests.")
                    && !f.file.contains("target/")
                    && f.file != "mod.rs"
                })
                .collect();
            assert!(orphans.is_empty(), "Orphan files in core: {:?}", orphans);
        }
    }

    #[test]
    fn test_verify_persistence() {
        let test_file = "/tmp/neotrix_persistence_test.txt";
        let content = "pub mod test_module;";
        std::fs::write(test_file, content).unwrap();
        assert!(verify_persistence(test_file, "pub mod test_module"));
        assert!(!verify_persistence(test_file, "non_existent_pattern"));
        let _ = std::fs::remove_file(test_file);
    }
}
