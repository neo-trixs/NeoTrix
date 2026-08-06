#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

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
    let mut entries: Vec<PathBuf> = match fs::read_dir(dir) {
        Ok(iter) => iter
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir())
            .collect(),
        Err(_) => Vec::new(),
    };
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
        let dir = mod_rs.parent().unwrap_or_else(|| Path::new("."));
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

/// Verify that a list of claimed module files from a prior session actually exist on disk.
/// P68: Session Re-entry Blindspot — cross-session claims must be independently verified.
pub fn verify_prior_session_claims(claimed_modules: &[(&str, &str)]) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    for (module_name, expected_path) in claimed_modules {
        let path = Path::new(expected_path);
        if !path.exists() {
            findings.push(AuditFinding {
                category: "phantom-claim",
                severity: AuditSeverity::Error,
                file: expected_path.to_string(),
                line: None,
                message: format!(
                    "P68: Prior-session claim '{}' at '{}' does not exist on disk",
                    module_name, expected_path
                ),
            });
        }
    }
    findings
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

// ────────────────────────────────────────────────────────────────
// ToolGroundingMonitor — 工具接地失效监控 (R-P49~R-P53 系统性发现)
// 监控 AI 工具的"声称成功 vs 实际成功", 超过阈值触发 CoreEvent。
// 归属 NT-SHIELD/NT-REPAIR verify 能力节点 (R-P42: 强化现有节点,
// 不建平行适配器模块)。
// ────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Default)]
pub struct ToolRecord {
    pub claimed_ok: u64,
    pub actual_ok: u64,
    pub failures: u64,
}

impl ToolRecord {
    pub fn failure_rate(&self) -> f64 {
        let total = self.claimed_ok + self.failures;
        if total == 0 { 0.0 } else { self.failures as f64 / total as f64 }
    }
}

/// 工具接地监控器 — 每次工具调用后记录 claimed/actual, 计算失败率。
/// 阈值自适应: 0-999 次调用用 5%, 1000+ 次用 2%。
#[derive(Debug, Clone, Default)]
pub struct ToolGroundingMonitor {
    pub tools: std::collections::HashMap<String, ToolRecord>,
    pub total_calls: u64,
    pub grounding_failures: u64,
}

impl ToolGroundingMonitor {
    pub fn new() -> Self {
        Self { tools: std::collections::HashMap::new(), total_calls: 0, grounding_failures: 0 }
    }

    /// Record a tool call result: claimed_ok = tool-reported success, actual_ok = true outcome.
    pub fn record_tool_result(&mut self, tool: &str, claimed_ok: bool, actual_ok: bool) {
        self.total_calls += 1;
        let record = self.tools.entry(tool.to_string()).or_insert(ToolRecord {
            claimed_ok: 0,
            actual_ok: 0,
            failures: 0,
        });
        if claimed_ok {
            record.claimed_ok += 1;
        }
        if actual_ok {
            record.actual_ok += 1;
        } else if claimed_ok {
            // Claimed success but actual failure = grounding failure
            record.failures += 1;
            self.grounding_failures += 1;
        }
    }

    /// 根据调用次数自适应收紧阈值: 0-999 次用 5%, 1000+ 用 2%。
    fn effective_threshold(&self) -> f64 {
        if self.total_calls > 1000 { 0.02 } else { 0.05 }
    }

    /// 检查某工具是否触发接地失效阈值。
    pub fn is_degraded(&self, tool: &str) -> bool {
        self.tools.get(tool).map(|r| r.failure_rate() > self.effective_threshold()).unwrap_or(false)
    }

    /// 全工具中是否有任一达到阈值。
    pub fn any_degraded(&self) -> bool {
        let t = self.effective_threshold();
        self.tools.values().any(|r| r.failure_rate() > t)
    }

    /// 生成降级工具清单 (名称, 失败率)。
    pub fn degraded_tools(&self) -> Vec<(String, f64)> {
        let t = self.effective_threshold();
        let mut out: Vec<(String, f64)> = self.tools.iter()
            .filter(|(_, r)| r.failure_rate() > t)
            .map(|(tool, r)| (tool.clone(), r.failure_rate()))
            .collect();
        out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        out
    }

    pub fn summary(&self) -> String {
        let deg = self.degraded_tools();
        if deg.is_empty() {
            format!("tool_grounding: {} calls, 0 degraded", self.total_calls)
        } else {
            let list: Vec<String> = deg.iter().map(|(t, r)| format!("{}={:.0}%", t, r * 100.0)).collect();
            format!("tool_grounding: {} calls, DEGRADED [{}]", self.total_calls, list.join(", "))
        }
    }
}

impl crate::core::nt_core_self_test::SelfTest for ToolGroundingMonitor {
    fn name(&self) -> &str {
        "tool_grounding"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let mut m = ToolGroundingMonitor::new();
        m.record_tool_result("edit", true, true);
        m.record_tool_result("edit", true, false);
        m.record_tool_result("edit", true, false);
        if !m.is_degraded("edit") {
            failures.push("expected edit failure_rate 0.67 > threshold to degrade".into());
        }
        if m.summary().is_empty() {
            failures.push("summary() should be non-empty".into());
        }
        if failures.is_empty() { Ok(()) } else { Err(failures) }
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

        // Test 4: P68 — Session Re-entry Blindspot: verify claimed prior-session modules
        // This test checks modules that were "created and tested" in prior sessions
        // but might not have persisted to disk.
        let prior_session_claims: &[(&str, &str)] = &[
            ("nt_mind_absorption_registry",
             "neotrix-core/src/neotrix/l8_autonomic_impl/nt_mind_absorption_registry.rs"),
        ];
        let phantom_claims = verify_prior_session_claims(prior_session_claims);
        if !phantom_claims.is_empty() {
            for claim in &phantom_claims {
                failures.push(format!("P68 phantom-claim: {} — {}", claim.file, claim.message));
            }
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
    fn test_no_new_python_kb_scripts() {
        let scripts_dir = std::path::Path::new("scripts");
        if !scripts_dir.exists() {
            return;
        }
        let whitelist: Vec<&str> = vec![
            "auto-absorb.py",
            "crawl-queue-absorb.sh",
            "mass_queue_processor.py",
            "novel-world-absorb.py",
            "generate-evolution-todo.py",
            "kb-generate-embeddings.py",
            "diagnose_absorption.py",
            "kb-init-schema.py",
            "launch-absorb-10h.sh",
            "neotrix-auto-absorb.py",
            "nt_comm_router.py",
            "deep-absorb-resources.py",
            "deep-absorb-fable5.py",
            // 以下脚本功能已被 Rust port (R-P79) — 保留历史脚本, 计入白名单
            "absorb_full_kb.py",      // port: kb_cmds.rs + nt_absorb_mapper.rs
            "absorb_to_capability.py",// port: nt_absorb_mapper.rs
            "kb-embed-pq.py",         // port: nt_memory_embed.rs
            // 历史运维脚本 (零生产引用, 疑似死代码 — 待用户决定迁移/删除)
            "enrich_repo_readmes.py",
            "kb_batch_absorb.py",
            "kb_formation.py",
        ];
        let mut violators = Vec::new();
        for entry in std::fs::read_dir(scripts_dir).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".py") && !name.ends_with(".sh") {
                continue;
            }
            if whitelist.contains(&name.as_str()) {
                continue;
            }
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            if content.contains("sqlite3.connect") {
                violators.push(name);
            }
        }
        assert!(
            violators.is_empty(),
            "New Python/Shell scripts directly writing to KB via sqlite3.connect: {:?}\nAll production data writes must go through Rust modules.",
            violators
        );
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

    #[test]
    fn test_verify_prior_session_claims() {
        // Known-existing file should pass
        let claims = &[("Cargo.toml", "Cargo.toml")];
        let findings = verify_prior_session_claims(claims);
        let phantom: Vec<_> = findings.iter().filter(|f| f.category == "phantom-claim").collect();
        assert!(phantom.is_empty(), "Cargo.toml should exist: {:?}", phantom);

        // Non-existent file should fail
        let bad_claims = &[("phantom_module", "/tmp/neotrix_phantom.rs")];
        let findings = verify_prior_session_claims(bad_claims);
        let phantom: Vec<_> = findings.iter().filter(|f| f.category == "phantom-claim").collect();
        assert_eq!(phantom.len(), 1, "Should find 1 phantom claim");
        assert!(phantom[0].message.contains("P68"));
    }

    #[test]
    fn test_tool_grounding_monitor() {
        let mut m = ToolGroundingMonitor::new();
        assert!(!m.any_degraded());
        m.record_tool_result("edit", true, true);
        m.record_tool_result("write", true, false);
        assert!(!m.is_degraded("edit"));
        assert!(m.is_degraded("write"));
        assert!(m.any_degraded());
        assert_eq!(m.total_calls, 2);
        assert_eq!(m.grounding_failures, 1);
        let deg = m.degraded_tools();
        assert_eq!(deg.len(), 1);
        assert_eq!(deg[0].0, "write");
        assert!(!m.summary().is_empty());
    }
}



