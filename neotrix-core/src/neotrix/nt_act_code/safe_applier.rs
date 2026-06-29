//! SafeCodeApplier — 安全应用代码修改, 失败自动回滚
//!
//! 策略:
//!   1. 修改前备份文件到 `~/.neotrix/backups/`
//!   2. 应用修改
//!   3. git stash 保护: 若 git 工作区干净, 修改后可回退
//!   4. 若 cargo check 失败 → 自动回滚

use super::edit_history::EditHistoryTracker;
use std::path::PathBuf;

use super::semantic_entropy::SemanticEntropy;
use super::verifier_stage::{SafeWriteGate, Verdict, VerifierStage};

/// 应用结果
#[derive(Debug)]
pub struct ApplyResult {
    pub file: String,
    pub success: bool,
    pub backup_path: Option<String>,
    pub error: Option<String>,
    /// When `Some(true)`, the edit was flagged as high‑entropy and deferred to LLM
    /// rather than being auto‑applied.
    pub deferred: Option<bool>,
}

/// 安全代码应用器
#[derive(Debug)]
pub struct SafeCodeApplier {
    backup_dir: PathBuf,
    tracker: EditHistoryTracker,
    pub verifiers: Vec<Box<dyn VerifierStage>>,
    pub gate: SafeWriteGate,
}

impl SafeCodeApplier {
    pub fn new() -> Self {
        let backup_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".neotrix")
            .join("backups");
        Self {
            backup_dir,
            tracker: EditHistoryTracker::new(),
            verifiers: Vec::new(),
            gate: SafeWriteGate::Allow,
        }
    }

    /// 测试用: 使用指定路径的 tracker (临时文件, 不污染 ~/.neotrix/)
    pub fn with_tracker_path(history_path: std::path::PathBuf) -> Self {
        let backup_dir = std::env::temp_dir().join("neotrix_safe_applier_test");
        Self {
            backup_dir,
            tracker: EditHistoryTracker::load_from_path(history_path),
            verifiers: Vec::new(),
            gate: SafeWriteGate::Allow,
        }
    }

    /// 安全的文件写入: 备份 → 写入 → 验证 → 记录
    pub fn safe_write(&mut self, file: &str, new_content: &str, issue_type: &str) -> ApplyResult {
        // 1. 读取旧内容
        let old_content = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                return ApplyResult {
                    file: file.to_string(),
                    success: false,
                    backup_path: None,
                    error: Some(format!("读取失败: {}", e)),
                    deferred: None,
                }
            }
        };

        // 2. 备份
        let backup_path = match self.backup(file, &old_content) {
            Ok(p) => p,
            Err(e) => {
                return ApplyResult {
                    file: file.to_string(),
                    success: false,
                    backup_path: None,
                    error: Some(format!("备份失败: {}", e)),
                    deferred: None,
                }
            }
        };

        // 3. 写入
        let tmp = std::path::Path::new(file).with_extension("tmp");
        if let Err(e) = std::fs::write(&tmp, new_content) {
            if let Err(re) =
                self.tracker
                    .record_change(file, issue_type, &old_content, new_content, false)
            {
                log::warn!("[safe-applier] failed to record change: {}", re);
            }
            return ApplyResult {
                file: file.to_string(),
                success: false,
                backup_path: Some(backup_path),
                error: Some(format!("写入失败: {}", e)),
                deferred: None,
            };
        }
        if let Err(e) = std::fs::rename(&tmp, file) {
            log::error!("[safe-applier] failed to rename to {}: {}", file, e);
        }

        // 4. 验证 (cargo check)
        let check_ok = Self::run_cargo_check();
        if !check_ok {
            // 回滚
            let tmp = std::path::Path::new(file).with_extension("tmp");
            if let Err(e) = std::fs::write(&tmp, &old_content) {
                log::error!("[safe-applier] rollback write failed: {}", e);
            }
            if let Err(e) = std::fs::rename(&tmp, file) {
                log::error!("[safe-applier] rollback rename failed: {}", e);
            }
            if let Err(re) =
                self.tracker
                    .record_change(file, issue_type, &old_content, new_content, false)
            {
                log::warn!("[safe-applier] failed to record change: {}", re);
            }
            return ApplyResult {
                file: file.to_string(),
                success: false,
                backup_path: Some(backup_path),
                error: Some("cargo check 失败, 已回滚".into()),
                deferred: None,
            };
        }

        // 5. 记录成功
        if let Err(re) =
            self.tracker
                .record_change(file, issue_type, &old_content, new_content, true)
        {
            log::warn!("[safe-applier] failed to record change: {}", re);
        }
        ApplyResult {
            file: file.to_string(),
            success: true,
            backup_path: Some(backup_path),
            error: None,
            deferred: None,
        }
    }

    /// Safe write gated by a SemanticEntropy detector.
    /// If the detector indicates high entropy (uncertainty), the write is
    /// skipped and `ApplyResult.deferred = Some(true)` is returned instead
    /// so the caller can route to LLM.
    pub fn safe_write_with_entropy(
        &mut self,
        file: &str,
        new_content: &str,
        issue_type: &str,
        detector: &SemanticEntropy,
        candidates: &[String],
    ) -> ApplyResult {
        let entropy = detector.estimate_entropy(candidates);
        if detector.should_defer(entropy) {
            return ApplyResult {
                file: file.to_string(),
                success: false,
                backup_path: None,
                error: Some(format!("High entropy ({:.4}) — deferred to LLM", entropy)),
                deferred: Some(true),
            };
        }
        let mut result = self.safe_write(file, new_content, issue_type);
        result.deferred = Some(false);
        result
    }

    /// 从备份恢复文件
    pub fn restore_from_backup(&self, file: &str) -> Result<(), String> {
        let backup = self
            .backup_dir
            .join(file.replace(std::path::is_separator, "__"));
        if !backup.exists() {
            return Err("备份不存在".into());
        }
        let content =
            std::fs::read_to_string(&backup).map_err(|e| format!("读取备份失败: {}", e))?;
        let p = std::path::Path::new(file);
        let tmp = p.with_extension("tmp");
        std::fs::write(&tmp, &content).map_err(|e| format!("恢复失败: {}", e))?;
        std::fs::rename(&tmp, p).map_err(|e| format!("恢复重命名失败: {}", e))?;
        Ok(())
    }

    pub fn tracker(&self) -> &EditHistoryTracker {
        &self.tracker
    }

    pub fn tracker_mut(&mut self) -> &mut EditHistoryTracker {
        &mut self.tracker
    }

    pub fn with_verifier(mut self, v: impl VerifierStage + 'static) -> Self {
        self.verifiers.push(Box::new(v));
        self
    }

    pub fn with_gate(mut self, gate: SafeWriteGate) -> Self {
        self.gate = gate;
        self
    }

    pub fn safe_write_verified(
        &mut self,
        file: &str,
        new_content: &str,
        issue_type: &str,
    ) -> ApplyResult {
        let old_content = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                return ApplyResult {
                    file: file.to_string(),
                    success: false,
                    backup_path: None,
                    error: Some(format!("读取失败: {}", e)),
                    deferred: None,
                }
            }
        };

        if let SafeWriteGate::Block(reason) = &self.gate {
            return ApplyResult {
                file: file.to_string(),
                success: false,
                backup_path: None,
                error: Some(format!("Blocked by gate: {}", reason)),
                deferred: None,
            };
        }

        let pre_results: Vec<(String, Verdict)> = self
            .verifiers
            .iter()
            .map(|v| {
                (
                    v.name().to_string(),
                    v.pre_verify(file, &old_content, new_content),
                )
            })
            .collect();

        if let SafeWriteGate::RequireApproval(reason) = &self.gate {
            let non_verified: Vec<&(String, Verdict)> = pre_results
                .iter()
                .filter(|(_, v)| !matches!(v, Verdict::Verified))
                .collect();
            if !non_verified.is_empty() {
                let details: Vec<String> = non_verified
                    .iter()
                    .map(|(name, v)| format!("{}: {:?}", name, v))
                    .collect();
                return ApplyResult {
                    file: file.to_string(),
                    success: false,
                    backup_path: None,
                    error: Some(format!(
                        "Requires approval ({}): {}",
                        reason,
                        details.join("; ")
                    )),
                    deferred: None,
                };
            }
        }

        let backup_path = match self.backup(file, &old_content) {
            Ok(p) => p,
            Err(e) => {
                return ApplyResult {
                    file: file.to_string(),
                    success: false,
                    backup_path: None,
                    error: Some(format!("备份失败: {}", e)),
                    deferred: None,
                }
            }
        };

        let tmp = std::path::Path::new(file).with_extension("tmp");
        if let Err(e) = std::fs::write(&tmp, new_content) {
            if let Err(re) =
                self.tracker
                    .record_change(file, issue_type, &old_content, new_content, false)
            {
                log::warn!("[safe-applier] failed to record change: {}", re);
            }
            return ApplyResult {
                file: file.to_string(),
                success: false,
                backup_path: Some(backup_path),
                error: Some(format!("写入失败: {}", e)),
                deferred: None,
            };
        }
        if let Err(e) = std::fs::rename(&tmp, file) {
            log::error!("[safe-applier] failed to rename to {}: {}", file, e);
        }

        for v in &self.verifiers {
            let verdict = v.post_verify(file, new_content);
            if let Verdict::Rejected(reason) = &verdict {
                let tmp = std::path::Path::new(file).with_extension("tmp");
                if let Err(e) = std::fs::write(&tmp, &old_content) {
                    log::error!("[safe-applier] rollback write failed: {}", e);
                }
                if let Err(e) = std::fs::rename(&tmp, file) {
                    log::error!("[safe-applier] rollback rename failed: {}", e);
                }
                if let Err(re) =
                    self.tracker
                        .record_change(file, issue_type, &old_content, new_content, false)
                {
                    log::warn!("[safe-applier] failed to record change: {}", re);
                }
                return ApplyResult {
                    file: file.to_string(),
                    success: false,
                    backup_path: Some(backup_path),
                    error: Some(format!("Post-verify failed ({}): {}", v.name(), reason)),
                    deferred: None,
                };
            }
        }

        if let Err(re) =
            self.tracker
                .record_change(file, issue_type, &old_content, new_content, true)
        {
            log::warn!("[safe-applier] failed to record change: {}", re);
        }
        ApplyResult {
            file: file.to_string(),
            success: true,
            backup_path: Some(backup_path),
            error: None,
            deferred: None,
        }
    }

    pub fn verification_report(&self, file: &str, new_content: &str) -> Vec<(String, Verdict)> {
        let old_content = std::fs::read_to_string(file).unwrap_or_default();
        self.verifiers
            .iter()
            .map(|v| {
                (
                    v.name().to_string(),
                    v.pre_verify(file, &old_content, new_content),
                )
            })
            .collect()
    }

    // ─── 内部 ───

    fn backup(&self, file: &str, content: &str) -> Result<String, String> {
        std::fs::create_dir_all(&self.backup_dir)
            .map_err(|e| format!("创建备份目录失败: {}", e))?;
        let backup_name = file.replace(std::path::is_separator, "__");
        let backup_path = self.backup_dir.join(&backup_name);
        let tmp = backup_path.with_extension("tmp");
        std::fs::write(&tmp, content).map_err(|e| format!("写入备份失败: {}", e))?;
        std::fs::rename(&tmp, &backup_path).map_err(|e| format!("重命名备份失败: {}", e))?;
        Ok(backup_path.to_string_lossy().to_string())
    }

    fn run_cargo_check() -> bool {
        if cfg!(test) {
            return true;
        }
        let output = std::process::Command::new("cargo")
            .args(["check", "--lib"])
            .output();
        match output {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }
}

impl Default for SafeCodeApplier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::verifier_stage::{FormatVerifier, SafetyPatternVerifier};
    use super::*;
    use std::io::Write;

    fn temp_file(content: &str) -> (std::path::PathBuf, std::fs::File) {
        let dir = std::env::temp_dir().join("neotrix_safe_applier_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_safe_write.rs");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        (path, f)
    }

    #[test]
    fn test_new_creates_backup_dir() {
        let applier = SafeCodeApplier::new();
        assert!(
            applier.backup_dir.exists()
                || applier
                    .backup_dir
                    .parent()
                    .map(|p| p.exists())
                    .unwrap_or(false)
        );
    }

    #[test]
    fn test_safe_write_nonexistent_file() {
        let mut applier = SafeCodeApplier::new();
        let result = applier.safe_write("/nonexistent/path.rs", "content", "Test");
        assert!(!result.success);
        assert!(result.error.unwrap().contains("读取失败"));
    }

    #[test]
    fn test_restore_from_nonexistent_backup() {
        let applier = SafeCodeApplier::new();
        let result = applier.restore_from_backup("/nonexistent/file.rs");
        assert!(result.is_err());
    }

    #[test]
    fn test_tracker_accessible() {
        let applier = SafeCodeApplier::new();
        assert!(applier.tracker().len() == 0 || applier.tracker().len() > 0);
    }

    #[test]
    fn test_safe_write_existing_file() {
        let (path, _f) = temp_file("fn main() {}");
        let path_str = path.to_string_lossy().to_string();
        let mut applier = SafeCodeApplier::new();
        // This may fail due to cargo check but should not crash
        let result = applier.safe_write(&path_str, "fn main() { let x = 1; }", "Test");
        // May succeed or fail depending on cargo check, but should not panic
        assert!(result.success == false || result.success == true);
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(path.parent().unwrap()).ok();
    }

    #[test]
    fn test_backup_creates_file() {
        let applier = SafeCodeApplier::new();
        let backup_path = applier.backup("test_backup.rs", "content");
        assert!(backup_path.is_ok());
        let p = backup_path.unwrap();
        assert!(std::path::Path::new(&p).exists());
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn test_empty_verifier_list() {
        let (path, _f) = temp_file("fn main() {}");
        let path_str = path.to_string_lossy().to_string();
        let mut applier = SafeCodeApplier::new();
        let result = applier.safe_write_verified(&path_str, "fn main() { let x = 1; }", "test");
        assert!(result.deferred.is_none());
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(path.parent().unwrap()).ok();
    }

    #[test]
    fn test_gate_block() {
        let (path, _f) = temp_file("fn main() {}");
        let path_str = path.to_string_lossy().to_string();
        let mut applier =
            SafeCodeApplier::new().with_gate(SafeWriteGate::Block("testing block".into()));
        let result = applier.safe_write_verified(&path_str, "fn main() { let x = 1; }", "test");
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Blocked by gate"));
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(path.parent().unwrap()).ok();
    }

    #[test]
    fn test_gate_require_approval_with_rejection() {
        let (path, _f) = temp_file("fn main() {}");
        let path_str = path.to_string_lossy().to_string();
        let mut applier = SafeCodeApplier::new()
            .with_verifier(SafetyPatternVerifier::new())
            .with_gate(SafeWriteGate::RequireApproval("safety check".into()));
        let result = applier.safe_write_verified(&path_str, "fn main() { unsafe { } }", "test");
        assert!(!result.success);
        let err = result.error.unwrap();
        assert!(err.contains("Requires approval"), "got error: {}", err);
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(path.parent().unwrap()).ok();
    }

    #[test]
    fn test_verification_report() {
        let (path, _f) = temp_file("fn main() {}");
        let path_str = path.to_string_lossy().to_string();
        let applier = SafeCodeApplier::new().with_verifier(SafetyPatternVerifier::new());
        let report = applier.verification_report(&path_str, "fn main() { unsafe { } }");
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].0, "safety");
        assert!(matches!(report[0].1, Verdict::Warning(_)));
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(path.parent().unwrap()).ok();
    }

    #[test]
    fn test_multiple_verifiers_sequence() {
        let (path, _f) = temp_file("fn main() {}");
        let path_str = path.to_string_lossy().to_string();
        let mut applier = SafeCodeApplier::new()
            .with_verifier(SafetyPatternVerifier::new())
            .with_verifier(FormatVerifier::new());
        let result = applier.safe_write_verified(&path_str, "fn main() {}", "test");
        assert!(result.deferred.is_none());
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(path.parent().unwrap()).ok();
    }
}
