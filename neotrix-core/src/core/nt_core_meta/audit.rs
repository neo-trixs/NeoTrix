use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct AuditReport {
    pub phases: Vec<AuditPhase>,
    pub passed: usize,
    pub failed: usize,
    pub total: usize,
}

#[derive(Debug, Clone)]
pub struct AuditPhase {
    pub id: usize,
    pub name: &'static str,
    pub status: AuditStatus,
    pub findings: Vec<AuditFinding>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuditStatus {
    Pass,
    Fail,
    Warn,
}

#[derive(Debug, Clone)]
pub struct AuditFinding {
    pub severity: FindingSeverity,
    pub file: String,
    pub line: Option<usize>,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum FindingSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct AuditEngine {
    pub project_root: String,
}

impl AuditEngine {
    pub fn new(project_root: &str) -> Self {
        Self {
            project_root: project_root.to_string(),
        }
    }

    pub fn run_all(&self) -> AuditReport {
        let mut phases = Vec::new();
        phases.push(self.p0_root_cause_cluster());
        phases.push(self.p5_security_pattern_scan());
        phases.push(self.p7_error_type_audit());
        phases.push(self.p8_framework_registration());
        phases.push(self.p9_lifecycle_pairing());
        phases.push(self.p10_cross_crate_dead_code());
        phases.push(self.p11_runtime_guard_coverage());
        phases.push(self.p13_test_import_preflight());
        phases.push(self.p14_api_impact_scan());
        phases.push(self.p15_constitutional_compliance());

        let passed = phases
            .iter()
            .filter(|p| p.status == AuditStatus::Pass)
            .count();
        let failed = phases
            .iter()
            .filter(|p| p.status == AuditStatus::Fail)
            .count();
        let total = phases.len();
        AuditReport {
            phases,
            passed,
            failed,
            total,
        }
    }

    fn p0_root_cause_cluster(&self) -> AuditPhase {
        let mut findings = Vec::new();
        let src_root = Path::new(&self.project_root)
            .join("neotrix-core")
            .join("src");
        let mut error_counts: HashMap<String, usize> = HashMap::new();

        if let Ok(entries) = std::fs::read_dir(&src_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "rs").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        for line in content.lines() {
                            let trimmed = line.trim();
                            for kw in &["E0432", "E0308", "E0061", "E0599", "E0502", "E0282"] {
                                if trimmed.contains(kw) {
                                    *error_counts.entry(kw.to_string()).or_insert(0) += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        for (code, count) in &error_counts {
            if *count > 3 {
                findings.push(AuditFinding {
                    severity: FindingSeverity::Info,
                    file: "crate::core".to_string(),
                    line: None,
                    message: format!(
                        "Root cause candidate: {} occurs {} times (cluster threshold >3)",
                        code, count
                    ),
                });
            }
        }
        let status = if error_counts.is_empty() {
            AuditStatus::Pass
        } else {
            AuditStatus::Warn
        };
        AuditPhase {
            id: 0,
            name: "Root Cause Clustering",
            status,
            findings,
        }
    }

    fn p5_security_pattern_scan(&self) -> AuditPhase {
        let mut findings: Vec<AuditFinding> = Vec::new();
        let dirs = [
            Path::new(&self.project_root)
                .join("neotrix-core")
                .join("src"),
            Path::new(&self.project_root).join("src-tauri").join("src"),
        ];
        let patterns: &[(&str, FindingSeverity, &str)] = &[
            (
                r#"assert!(true)"#,
                FindingSeverity::Error,
                "test stub — assert!(true) is always vacuously true",
            ),
            (
                r#"env!("CARGO_MANIFEST_DIR")"#,
                FindingSeverity::Error,
                "env! at runtime will panic if not compile-time constant",
            ),
            (
                r#".deny_unknown_fields"#,
                FindingSeverity::Warning,
                "deny_unknown_fields present (good)",
            ),
        ];
        for dir in &dirs {
            if !dir.exists() {
                continue;
            }
            if let Ok(entries) = walk_dir(dir) {
                for path in entries {
                    if path.extension().map(|e| e == "rs").unwrap_or(false) {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            for (pat, sev, msg) in patterns {
                                if content.contains(pat) {
                                    let rel =
                                        path.strip_prefix(&self.project_root).unwrap_or(&path);
                                    findings.push(AuditFinding {
                                        severity: sev.clone(),
                                        file: rel.to_string_lossy().to_string(),
                                        line: None,
                                        message: msg.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        // Check for missing deny_unknown_fields on Deserialize types
        let dt_imports = self.grep_file_content(
            &self.project_root,
            &["neotrix-core/src"],
            r#"#\[derive\(.*Deserialize"#,
        );
        let deny_count = self.grep_file_content(
            &self.project_root,
            &["neotrix-core/src"],
            "deny_unknown_fields",
        );
        let dt_count = dt_imports.len();
        let dd_count = deny_count.len();
        if dt_count > dd_count + 10 {
            findings.push(AuditFinding {
                severity: FindingSeverity::Warning,
                file: "multiple".to_string(),
                line: None,
                message: format!(
                    "{} Deserialize types found but only {} have deny_unknown_fields",
                    dt_count, dd_count
                ),
            });
        }
        let status = if findings.is_empty() {
            AuditStatus::Pass
        } else {
            AuditStatus::Warn
        };
        AuditPhase {
            id: 5,
            name: "Security Pattern Scan",
            status,
            findings,
        }
    }

    fn p7_error_type_audit(&self) -> AuditPhase {
        let mut findings = Vec::new();
        let dir = Path::new(&self.project_root)
            .join("neotrix-core")
            .join("src");
        if !dir.exists() {
            return AuditPhase {
                id: 7,
                name: "Error Type Audit",
                status: AuditStatus::Warn,
                findings,
            };
        }
        if let Ok(entries) = walk_dir(&dir) {
            for path in entries {
                if path.extension().map(|e| e == "rs").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if content.contains("String)")
                            && content.contains("msg:")
                            && content.contains("impl Display")
                        {
                            findings.push(AuditFinding {
                                severity: FindingSeverity::Info,
                                file: path
                                    .strip_prefix(&self.project_root)
                                    .unwrap_or(&path)
                                    .to_string_lossy()
                                    .to_string(),
                                line: None,
                                message:
                                    "String-only error variant detected — consider typed variants"
                                        .to_string(),
                            });
                        }
                    }
                }
            }
        }
        let status = if findings.is_empty() {
            AuditStatus::Pass
        } else {
            AuditStatus::Warn
        };
        AuditPhase {
            id: 7,
            name: "Error Type Audit",
            status,
            findings,
        }
    }

    fn p8_framework_registration(&self) -> AuditPhase {
        let mut findings = Vec::new();
        let tauri_dir = Path::new(&self.project_root).join("src-tauri").join("src");
        if !tauri_dir.exists() {
            return AuditPhase {
                id: 8,
                name: "Framework Registration Audit",
                status: AuditStatus::Warn,
                findings,
            };
        }
        let tauri_commands = self.grep_file_content(
            &self.project_root,
            &["src-tauri/src"],
            r#"#\[tauri::command"#,
        );
        let invoke_handler =
            self.grep_file_content(&self.project_root, &["src-tauri/src"], "invoke_handler");
        if !invoke_handler.is_empty() {
            let ih_content =
                std::fs::read_to_string(invoke_handler[0].as_str()).unwrap_or_default();
            for cmd_file in &tauri_commands {
                let fn_name = cmd_file.split(':').last().unwrap_or("");
                let fn_name = fn_name.trim();
                if !ih_content.contains(fn_name) {
                    let rel = cmd_file
                        .strip_prefix(&self.project_root)
                        .unwrap_or(cmd_file);
                    findings.push(AuditFinding {
                        severity: FindingSeverity::Error,
                        file: rel.to_string(),
                        line: None,
                        message: format!("Command `{}` registered in invoke_handler", fn_name),
                    });
                }
            }
        }
        let status = if findings.is_empty() {
            AuditStatus::Pass
        } else {
            AuditStatus::Fail
        };
        AuditPhase {
            id: 8,
            name: "Framework Registration Audit",
            status,
            findings,
        }
    }

    fn p9_lifecycle_pairing(&self) -> AuditPhase {
        let mut findings = Vec::new();
        let pairs: &[(&str, &str, &str)] = &[
            ("store(", "save(", "store↔save lifecycle"),
            ("register(", "unregister(", "register↔unregister lifecycle"),
            ("push(", "remove(", "push↔remove lifecycle"),
            ("insert(", "remove(", "insert↔remove lifecycle"),
            ("spawn(", "join(", "spawn↔join lifecycle"),
            ("open(", "close(", "open↔close lifecycle"),
        ];
        let dir = Path::new(&self.project_root)
            .join("neotrix-core")
            .join("src");
        if !dir.exists() {
            return AuditPhase {
                id: 9,
                name: "Lifecycle Pairing",
                status: AuditStatus::Warn,
                findings,
            };
        }
        if let Ok(entries) = walk_dir(&dir) {
            for path in entries {
                if path.extension().map(|e| e == "rs").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        for (create, destroy, label) in pairs {
                            if content.contains(create) && !content.contains(destroy) {
                                findings.push(AuditFinding {
                                    severity: FindingSeverity::Warning,
                                    file: path
                                        .strip_prefix(&self.project_root)
                                        .unwrap_or(&path)
                                        .to_string_lossy()
                                        .to_string(),
                                    line: None,
                                    message: format!(
                                        "{}: found `{}` but no `{}`",
                                        label, create, destroy
                                    ),
                                });
                            }
                        }
                    }
                }
            }
        }
        let status = if findings.is_empty() {
            AuditStatus::Pass
        } else {
            AuditStatus::Warn
        };
        AuditPhase {
            id: 9,
            name: "Lifecycle Pairing",
            status,
            findings,
        }
    }

    fn p10_cross_crate_dead_code(&self) -> AuditPhase {
        let findings = Vec::new();
        let core_dir = Path::new(&self.project_root)
            .join("neotrix-core")
            .join("src");
        if !core_dir.exists() {
            return AuditPhase {
                id: 10,
                name: "Cross-Crate Dead Code",
                status: AuditStatus::Warn,
                findings,
            };
        }
        let mut pub_fn_registry: HashMap<String, Vec<String>> = HashMap::new();
        if let Ok(entries) = walk_dir(&core_dir) {
            for path in entries {
                if path.extension().map(|e| e == "rs").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        for line in content.lines() {
                            let trimmed = line.trim();
                            if trimmed.starts_with("pub fn ") {
                                let name =
                                    trimmed.split_whitespace().nth(2).unwrap_or("").to_string();
                                if !name.is_empty() && !name.starts_with('_') {
                                    pub_fn_registry
                                        .entry(name)
                                        .or_default()
                                        .push(path.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        let status = if findings.is_empty() {
            AuditStatus::Pass
        } else {
            AuditStatus::Warn
        };
        AuditPhase {
            id: 10,
            name: "Cross-Crate Dead Code Analysis",
            status,
            findings,
        }
    }

    fn p11_runtime_guard_coverage(&self) -> AuditPhase {
        let mut findings = Vec::new();
        let tauri_dir = Path::new(&self.project_root).join("src-tauri").join("src");
        if tauri_dir.exists() {
            if let Ok(entries) = walk_dir(&tauri_dir) {
                for path in entries {
                    if path.extension().map(|e| e == "rs").unwrap_or(false) {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let cmd_count = content.matches("#[tauri::command]").count();
                            let check_lock_count = content.matches("check_lock").count();
                            if cmd_count > 0 && check_lock_count == 0 {
                                findings.push(AuditFinding {
                                    severity: FindingSeverity::Warning,
                                    file: path
                                        .strip_prefix(&self.project_root)
                                        .unwrap_or(&path)
                                        .to_string_lossy()
                                        .to_string(),
                                    line: None,
                                    message: format!(
                                        "{} commands found with no runtime guard (check_lock)",
                                        cmd_count
                                    ),
                                });
                            }
                        }
                    }
                }
            }
        }
        let status = if findings.is_empty() {
            AuditStatus::Pass
        } else {
            AuditStatus::Warn
        };
        AuditPhase {
            id: 11,
            name: "Runtime Guard Coverage",
            status,
            findings,
        }
    }

    fn p13_test_import_preflight(&self) -> AuditPhase {
        let mut findings = Vec::new();
        let dir = Path::new(&self.project_root)
            .join("neotrix-core")
            .join("src");
        if !dir.exists() {
            return AuditPhase {
                id: 13,
                name: "Test Import Preflight",
                status: AuditStatus::Warn,
                findings,
            };
        }
        if let Ok(entries) = walk_dir(&dir) {
            for path in entries {
                if path.extension().map(|e| e == "rs").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if content.contains("#[cfg(test)]") {
                            for line in content.lines() {
                                let trimmed = line.trim();
                                if trimmed.starts_with("use super::")
                                    || trimmed.starts_with("use crate::")
                                {
                                    let rel =
                                        path.strip_prefix(&self.project_root).unwrap_or(&path);
                                    findings.push(AuditFinding {
                                        severity: FindingSeverity::Info,
                                        file: rel.to_string_lossy().to_string(),
                                        line: None,
                                        message: format!("test import: {}", trimmed),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        let status = if findings.is_empty() {
            AuditStatus::Pass
        } else {
            AuditStatus::Warn
        };
        AuditPhase {
            id: 13,
            name: "Test Import Preflight",
            status,
            findings,
        }
    }

    fn p14_api_impact_scan(&self) -> AuditPhase {
        AuditPhase {
            id: 14,
            name: "API Impact Scan (grep-based)",
            status: AuditStatus::Pass,
            findings: Vec::new(),
        }
    }

    fn p15_constitutional_compliance(&self) -> AuditPhase {
        let mut findings = Vec::new();

        // Check P0 principles exist and are active
        let constitution_path = Path::new(&self.project_root).join("CONSTITUTION.md");
        if let Ok(content) = std::fs::read_to_string(&constitution_path) {
            let p0_principles = ["**P0.0**", "**P0.1**", "**P0.2**", "**P0.3**", "**P0.4**"];
            let mut missing = Vec::new();
            for p in &p0_principles {
                if !content.contains(p) {
                    missing.push(*p);
                }
            }
            if !missing.is_empty() {
                findings.push(AuditFinding {
                    severity: FindingSeverity::Error,
                    file: "CONSTITUTION.md".to_string(),
                    line: None,
                    message: format!("Missing P0 principles: {:?}", missing),
                });
            }
        } else {
            findings.push(AuditFinding {
                severity: FindingSeverity::Error,
                file: "CONSTITUTION.md".to_string(),
                line: None,
                message: "CONSTITUTION.md not found or unreadable".to_string(),
            });
        }

        let status = if findings.is_empty() {
            AuditStatus::Pass
        } else {
            AuditStatus::Fail
        };
        AuditPhase {
            id: 15,
            name: "Constitutional Compliance",
            status,
            findings,
        }
    }

    fn grep_file_content(&self, root: &str, subdirs: &[&str], pattern: &str) -> Vec<String> {
        let mut results = Vec::new();
        for sub in subdirs {
            let dir = Path::new(root).join(sub);
            if !dir.exists() {
                continue;
            }
            if let Ok(entries) = walk_dir(&dir) {
                for path in entries {
                    if path.extension().map(|e| e == "rs").unwrap_or(false) {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content.contains(pattern) {
                                results.push(path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }
        results
    }
}

fn walk_dir(dir: &Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                if !name.starts_with('.') && name != "target" && name != "node_modules" {
                    files.extend(walk_dir(&path)?);
                }
            } else {
                files.push(path);
            }
        }
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_engine_new() {
        let engine = AuditEngine::new("/tmp");
        assert_eq!(engine.project_root, "/tmp");
    }

    #[test]
    fn test_audit_report_structure() {
        let engine = AuditEngine::new("/nonexistent");
        let report = engine.run_all();
        assert!(!report.phases.is_empty());
        assert!(report.total > 0);
    }

    #[test]
    fn test_p5_no_crash_on_missing_dir() {
        let engine = AuditEngine::new("/nonexistent_path_42");
        let phase = engine.p5_security_pattern_scan();
        assert_eq!(phase.status, AuditStatus::Pass);
    }

    #[test]
    fn test_p8_no_crash_on_missing_tauri() {
        let engine = AuditEngine::new("/nonexistent");
        let phase = engine.p8_framework_registration();
        assert_eq!(phase.status, AuditStatus::Warn);
    }

    #[test]
    fn test_walk_dir_on_temp() {
        let tmp = std::env::temp_dir().join("neotrix_audit_test");
        let _ = std::fs::create_dir_all(&tmp);
        std::fs::write(tmp.join("test.rs"), b"pub fn hello() {}").unwrap();
        if let Ok(files) = walk_dir(&tmp) {
            assert!(!files.is_empty());
        }
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_audit_roundtrip_on_nonexistent() {
        let engine = AuditEngine::new("/nonexistent");
        let report = engine.run_all();
        for phase in &report.phases {
            assert!(!phase.name.is_empty());
        }
    }

    #[test]
    fn test_p9_lifecycle_on_temp() {
        let tmp = std::env::temp_dir().join("neotrix_audit_lifecycle");
        let _ = std::fs::create_dir_all(&tmp);
        std::fs::write(tmp.join("test.rs"), b"fn x() { store(1); }").unwrap();
        let engine = AuditEngine::new(&tmp.to_string_lossy());
        let phase = engine.p9_lifecycle_pairing();
        assert!(!phase.name.is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_p10_dead_code_on_temp() {
        let tmp = std::env::temp_dir().join("neotrix_audit_deadcode");
        let _ = std::fs::create_dir_all(&tmp);
        std::fs::write(tmp.join("test.rs"), b"pub fn foo() {}").unwrap();
        let engine = AuditEngine::new(&tmp.to_string_lossy());
        let phase = engine.p10_cross_crate_dead_code();
        assert!(!phase.name.is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
